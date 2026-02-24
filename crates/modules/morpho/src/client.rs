//! Morpho Blue lending protocol module.
//!
//! MVP: Read-only market data via Morpho Blue API.
//! Uses the Morpho Blue GraphQL/REST API for market info.
//! On-chain execution (supply/withdraw/borrow/repay) via RPC in future phases.

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use tracing::info;

use atlas_common::error::*;
use atlas_common::traits::{LendingModule, LendingMarket, LendingPosition};
use atlas_common::types::*;

/// Morpho Blue API base URL.
const MORPHO_API_BASE: &str = "https://blue-api.morpho.org/graphql";

/// Morpho Blue lending module.
pub struct MorphoModule {
    /// HTTP client for API calls.
    http: reqwest::Client,
    /// Chain to query (Ethereum mainnet or Base).
    pub chain: Chain,
}

impl MorphoModule {
    pub fn new(chain: Chain) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");

        info!(chain = %chain, "Morpho module initialized");

        Self { http, chain }
    }

    /// Query the Morpho Blue GraphQL API.
    async fn graphql_query(&self, query: &str) -> AtlasResult<serde_json::Value> {
        let body = serde_json::json!({ "query": query });

        let resp = self.http
            .post(MORPHO_API_BASE)
            .json(&body)
            .send()
            .await
            .map_err(|e| AtlasError::Network(format!("Morpho API request failed: {e}")))?;

        let status = resp.status();
        let text = resp.text().await
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(AtlasError::Protocol {
                protocol: "morpho".into(),
                message: format!("HTTP {status}: {text}"),
            });
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| AtlasError::Other(format!("Parse Morpho response: {e}")))?;

        Ok(parsed)
    }

    fn chain_id(&self) -> u64 {
        match self.chain {
            Chain::Ethereum => 1,
            Chain::Base => 8453,
            _ => 1,
        }
    }
}

#[async_trait]
impl LendingModule for MorphoModule {
    fn protocol(&self) -> Protocol {
        Protocol::Morpho
    }

    async fn markets(&self) -> AtlasResult<Vec<LendingMarket>> {
        let chain_id = self.chain_id();
        let query = format!(r#"
            {{
                markets(where: {{ chainId_in: [{chain_id}] }}, first: 50) {{
                    items {{
                        uniqueKey
                        loanAsset {{ symbol decimals }}
                        collateralAsset {{ symbol decimals }}
                        state {{
                            supplyApy
                            borrowApy
                            supplyAssetsUsd
                            borrowAssetsUsd
                            utilization
                        }}
                        lltv
                    }}
                }}
            }}
        "#);

        let data = self.graphql_query(&query).await?;

        let empty = vec![];
        let items = data
            .pointer("/data/markets/items")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty);

        let mut markets = Vec::new();
        for item in items {
            let market_id = item.get("uniqueKey")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let collateral = item.pointer("/collateralAsset/symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let loan = item.pointer("/loanAsset/symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let state = item.get("state").cloned().unwrap_or_default();

            let supply_apy = parse_decimal_field(&state, "supplyApy");
            let borrow_apy = parse_decimal_field(&state, "borrowApy");
            let total_supply = parse_decimal_field(&state, "supplyAssetsUsd");
            let total_borrow = parse_decimal_field(&state, "borrowAssetsUsd");
            let utilization = parse_decimal_field(&state, "utilization");
            let lltv = parse_decimal_field(item, "lltv");

            markets.push(LendingMarket {
                protocol: Protocol::Morpho,
                chain: self.chain.clone(),
                market_id,
                collateral_asset: collateral,
                loan_asset: loan,
                supply_apy,
                borrow_apy,
                total_supply,
                total_borrow,
                utilization,
                ltv: Decimal::ZERO, // Morpho uses LLTV not LTV
                lltv,
            });
        }

        Ok(markets)
    }

    async fn positions(&self, user: &str) -> AtlasResult<Vec<LendingPosition>> {
        let chain_id = self.chain_id();
        let query = format!(r#"
            {{
                marketPositions(
                    where: {{ userAddress_in: ["{user}"], chainId_in: [{chain_id}] }}
                    first: 50
                ) {{
                    items {{
                        market {{
                            uniqueKey
                            collateralAsset {{ symbol }}
                            loanAsset {{ symbol }}
                        }}
                        supplyAssetsUsd
                        borrowAssetsUsd
                        healthFactor
                    }}
                }}
            }}
        "#);

        let data = self.graphql_query(&query).await?;

        let empty = vec![];
        let items = data
            .pointer("/data/marketPositions/items")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty);

        let mut positions = Vec::new();
        for item in items {
            let market = item.get("market").cloned().unwrap_or_default();
            let market_id = market.get("uniqueKey")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let collateral = market.pointer("/collateralAsset/symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let loan = market.pointer("/loanAsset/symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string();

            let supplied = parse_decimal_field(item, "supplyAssetsUsd");
            let borrowed = parse_decimal_field(item, "borrowAssetsUsd");
            let health = parse_decimal_field_opt(item, "healthFactor");

            if supplied.is_zero() && borrowed.is_zero() {
                continue;
            }

            positions.push(LendingPosition {
                protocol: Protocol::Morpho,
                chain: self.chain.clone(),
                market_id,
                collateral_asset: collateral,
                loan_asset: loan,
                supplied,
                borrowed,
                health_factor: health,
            });
        }

        Ok(positions)
    }

    async fn supply(&self, _market_id: &str, _amount: Decimal) -> AtlasResult<String> {
        Err(AtlasError::Other("On-chain supply not implemented yet. Use the frontend.".into()))
    }

    async fn withdraw(&self, _market_id: &str, _amount: Decimal) -> AtlasResult<String> {
        Err(AtlasError::Other("On-chain withdraw not implemented yet. Use the frontend.".into()))
    }

    async fn borrow(&self, _market_id: &str, _amount: Decimal) -> AtlasResult<String> {
        Err(AtlasError::Other("On-chain borrow not implemented yet. Use the frontend.".into()))
    }

    async fn repay(&self, _market_id: &str, _amount: Decimal) -> AtlasResult<String> {
        Err(AtlasError::Other("On-chain repay not implemented yet. Use the frontend.".into()))
    }
}

fn parse_decimal_field(val: &serde_json::Value, field: &str) -> Decimal {
    val.get(field)
        .and_then(|v| v.as_f64())
        .and_then(Decimal::from_f64)
        .unwrap_or(Decimal::ZERO)
}

fn parse_decimal_field_opt(val: &serde_json::Value, field: &str) -> Option<Decimal> {
    val.get(field)
        .and_then(|v| v.as_f64())
        .and_then(Decimal::from_f64)
}
