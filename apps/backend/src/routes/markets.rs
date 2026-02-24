//! Market data endpoints (Hyperliquid perps).

use std::sync::Arc;
use axum::{Router, Json, routing::get, extract::State};
use serde_json::{json, Value};

use crate::state::AppState;

/// GET /api/markets — list all perp markets (no auth needed).
async fn list_markets(State(state): State<Arc<AppState>>) -> Json<Value> {
    if !state.hl_enabled {
        return Json(json!({ "error": "Hyperliquid module disabled" }));
    }

    // Create a read-only client (no signer needed for market data)
    let testnet = state.config.modules.hyperliquid.config.network == "testnet";
    let client = if testnet {
        hypersdk::hypercore::testnet()
    } else {
        hypersdk::hypercore::mainnet()
    };

    match client.perps().await {
        Ok(perps) => {
            let markets: Vec<Value> = perps.iter().map(|m| {
                json!({
                    "symbol": m.name,
                    "index": m.index,
                    "max_leverage": m.max_leverage,
                    "sz_decimals": m.sz_decimals,
                })
            }).collect();
            Json(json!({ "markets": markets }))
        }
        Err(e) => Json(json!({ "error": format!("Failed to fetch markets: {e}") })),
    }
}

/// GET /api/prices — all mid prices.
async fn all_prices(State(state): State<Arc<AppState>>) -> Json<Value> {
    if !state.hl_enabled {
        return Json(json!({ "error": "Hyperliquid module disabled" }));
    }

    let testnet = state.config.modules.hyperliquid.config.network == "testnet";
    let client = if testnet {
        hypersdk::hypercore::testnet()
    } else {
        hypersdk::hypercore::mainnet()
    };

    match client.all_mids(None).await {
        Ok(mids) => {
            let prices: Vec<Value> = mids.iter().map(|(coin, price)| {
                json!({ "coin": coin, "price": price.to_string() })
            }).collect();
            Json(json!({ "prices": prices }))
        }
        Err(e) => Json(json!({ "error": format!("Failed to fetch prices: {e}") })),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/markets", get(list_markets))
        .route("/prices", get(all_prices))
}
