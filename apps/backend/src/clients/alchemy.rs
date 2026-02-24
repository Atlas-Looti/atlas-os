//! Alchemy API client — JSON-RPC + REST Data APIs.
//!
//! Provides multi-chain EVM data: token balances, metadata, prices, portfolio.
//! Rate-limit aware with exponential backoff.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Alchemy client for a specific network.
#[derive(Clone)]
pub struct AlchemyClient {
    http: Client,
    api_key: String,
}

// ── JSON-RPC Types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct JsonRpcRequest<'a, T: Serialize> {
    jsonrpc: &'a str,
    method: &'a str,
    params: T,
    id: u64,
}

#[derive(Deserialize, Debug)]
pub struct JsonRpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

// ── Token API Response Types ────────────────────────────────────────

#[derive(Deserialize, Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct TokenBalance {
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(rename = "tokenBalance")]
    pub token_balance: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct TokenBalancesResult {
    pub address: String,
    #[serde(rename = "tokenBalances")]
    pub token_balances: Vec<TokenBalance>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct TokenMetadata {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
    pub logo: Option<String>,
}

// ── Portfolio API Response Types ────────────────────────────────────

#[derive(Serialize)]
struct PortfolioAddress<'a> {
    address: &'a str,
    networks: &'a [&'a str],
}

#[derive(Serialize)]
struct PortfolioRequest<'a> {
    addresses: Vec<PortfolioAddress<'a>>,
    #[serde(rename = "withMetadata")]
    with_metadata: bool,
    #[serde(rename = "withPrices")]
    with_prices: bool,
    #[serde(rename = "includeNativeTokens")]
    include_native_tokens: bool,
    #[serde(rename = "includeErc20Tokens")]
    include_erc20_tokens: bool,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct PortfolioToken {
    pub address: String,
    pub network: String,
    #[serde(rename = "tokenAddress")]
    pub token_address: Option<String>,
    #[serde(rename = "tokenBalance")]
    pub token_balance: String,
    #[serde(rename = "tokenMetadata")]
    pub token_metadata: Option<PortfolioTokenMetadata>,
    #[serde(rename = "tokenPrices")]
    pub token_prices: Option<Vec<PortfolioTokenPrice>>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct PortfolioTokenMetadata {
    pub decimals: Option<u8>,
    pub logo: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct PortfolioTokenPrice {
    pub currency: String,
    pub value: String,
    #[serde(rename = "lastUpdatedAt")]
    pub last_updated_at: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PortfolioData {
    tokens: Vec<PortfolioToken>,
}

#[derive(Deserialize, Debug)]
struct PortfolioResponse {
    data: PortfolioData,
}

impl AlchemyClient {
    /// Create a new Alchemy client with the given API key.
    pub fn new(api_key: &str) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            api_key: api_key.to_string(),
        }
    }

    /// Build JSON-RPC URL for a specific network.
    fn rpc_url(&self, network: &str) -> String {
        format!("https://{}.g.alchemy.com/v2/{}", network, self.api_key)
    }

    /// Build Data API URL.
    fn data_url(&self, path: &str) -> String {
        format!("https://api.g.alchemy.com/data/v1/{}/{}", self.api_key, path)
    }

    /// Execute a JSON-RPC call with retry on 429.
    async fn rpc_call<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        network: &str,
        method: &str,
        params: P,
    ) -> anyhow::Result<R> {
        let url = self.rpc_url(network);
        let body = JsonRpcRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: 1,
        };

        let mut retries = 0u32;
        let max_retries = 3;

        loop {
            let resp = self.http.post(&url)
                .json(&body)
                .send()
                .await?;

            if resp.status() == 429 {
                retries += 1;
                if retries > max_retries {
                    anyhow::bail!("Alchemy rate limited after {max_retries} retries");
                }
                let wait = Duration::from_millis(1000 * 2u64.pow(retries - 1));
                warn!("Alchemy 429 — retrying in {:?} (attempt {retries}/{max_retries})", wait);
                tokio::time::sleep(wait).await;
                continue;
            }

            let result: JsonRpcResponse<R> = resp.json().await?;

            if let Some(err) = result.error {
                anyhow::bail!("Alchemy RPC error {}: {}", err.code, err.message);
            }

            return result.result.ok_or_else(|| anyhow::anyhow!("Alchemy returned null result"));
        }
    }

    // ── Token API ───────────────────────────────────────────────

    /// Get ERC-20 token balances for an address on a specific network.
    #[allow(dead_code)]
    pub async fn get_token_balances(
        &self,
        network: &str,
        address: &str,
        token_addresses: Option<Vec<String>>,
    ) -> anyhow::Result<TokenBalancesResult> {
        let params: serde_json::Value = match token_addresses {
            Some(tokens) => serde_json::json!([address, tokens]),
            None => serde_json::json!([address, "erc20"]),
        };
        self.rpc_call(network, "alchemy_getTokenBalances", params).await
    }

    /// Get token metadata (name, symbol, decimals, logo).
    pub async fn get_token_metadata(
        &self,
        network: &str,
        contract_address: &str,
    ) -> anyhow::Result<TokenMetadata> {
        self.rpc_call(network, "alchemy_getTokenMetadata", [contract_address]).await
    }

    // ── Portfolio API (multi-chain) ─────────────────────────────

    /// Get full portfolio: tokens + prices + metadata across multiple chains.
    pub async fn get_portfolio(
        &self,
        address: &str,
        networks: &[&str],
    ) -> anyhow::Result<Vec<PortfolioToken>> {
        let url = self.data_url("assets/tokens/by-address");

        let body = PortfolioRequest {
            addresses: vec![PortfolioAddress { address, networks }],
            with_metadata: true,
            with_prices: true,
            include_native_tokens: true,
            include_erc20_tokens: true,
        };

        let mut retries = 0u32;
        loop {
            let resp = self.http.post(&url)
                .json(&body)
                .send()
                .await?;

            if resp.status() == 429 {
                retries += 1;
                if retries > 3 {
                    anyhow::bail!("Alchemy Portfolio API rate limited");
                }
                tokio::time::sleep(Duration::from_millis(1000 * 2u64.pow(retries - 1))).await;
                continue;
            }

            let data: PortfolioResponse = resp.json().await?;
            return Ok(data.data.tokens);
        }
    }
}
