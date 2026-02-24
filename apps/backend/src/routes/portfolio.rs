//! Portfolio API routes.
//!
//! GET /api/portfolio/:address — Multi-chain wallet portfolio
//! GET /api/portfolio/:address/tokens — Token balances only
//! GET /api/tokens/:network/:contract/metadata — Token metadata

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;

use crate::services::portfolio::PortfolioService;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct PortfolioQuery {
    /// Comma-separated networks (default: eth-mainnet,base-mainnet,arb-mainnet)
    networks: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/portfolio/{address}", get(get_portfolio))
        .route("/tokens/{network}/{contract}/metadata", get(get_token_metadata))
}

/// GET /api/portfolio/:address?networks=eth-mainnet,base-mainnet
async fn get_portfolio(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(query): Query<PortfolioQuery>,
) -> Json<serde_json::Value> {
    let alchemy = match &state.alchemy {
        Some(a) => a,
        None => return Json(serde_json::json!({ "error": "Alchemy API not configured. Set ALCHEMY_API_KEY env var." })),
    };

    let networks_str = query.networks.unwrap_or_else(|| "eth-mainnet,base-mainnet,arb-mainnet".to_string());
    let networks: Vec<&str> = networks_str.split(',').map(|s| s.trim()).collect();

    // Use cache if available, otherwise a dummy no-op cache
    let result = match &state.cache {
        Some(cache) => PortfolioService::get_portfolio(alchemy, cache, &address, &networks).await,
        None => {
            // No cache — direct call
            match alchemy.get_portfolio(&address, &networks).await {
                Ok(tokens) => {
                    let total: f64 = tokens.iter()
                        .filter_map(|t| {
                            let p = t.token_prices.as_ref()?.iter().find(|p| p.currency == "usd")?;
                            let price: f64 = p.value.parse().ok()?;
                            let decimals = t.token_metadata.as_ref()?.decimals.unwrap_or(18) as u32;
                            let bal = hex_to_f64(&t.token_balance, decimals);
                            Some(bal * price)
                        })
                        .sum();
                    Ok(crate::services::portfolio::WalletPortfolio {
                        address: address.clone(),
                        networks: networks.iter().map(|s| s.to_string()).collect(),
                        tokens,
                        total_value_usd: total,
                    })
                }
                Err(e) => Err(e),
            }
        }
    };

    match result {
        Ok(portfolio) => Json(serde_json::json!({
            "address": portfolio.address,
            "networks": portfolio.networks,
            "total_value_usd": portfolio.total_value_usd,
            "token_count": portfolio.tokens.len(),
            "tokens": portfolio.tokens,
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/tokens/:network/:contract/metadata
async fn get_token_metadata(
    State(state): State<Arc<AppState>>,
    Path((network, contract)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    let alchemy = match &state.alchemy {
        Some(a) => a,
        None => return Json(serde_json::json!({ "error": "Alchemy API not configured" })),
    };

    let result = match &state.cache {
        Some(cache) => PortfolioService::get_token_metadata(alchemy, cache, &network, &contract).await,
        None => alchemy.get_token_metadata(&network, &contract).await,
    };

    match result {
        Ok(meta) => Json(serde_json::json!(meta)),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

fn hex_to_f64(hex_str: &str, decimals: u32) -> f64 {
    let hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let value = u128::from_str_radix(hex, 16).unwrap_or(0);
    value as f64 / 10f64.powi(decimals as i32)
}
