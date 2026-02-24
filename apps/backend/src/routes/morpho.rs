//! Morpho lending endpoints.

use std::sync::Arc;
use axum::{Router, Json, routing::get, extract::State};
use serde_json::{json, Value};

use atlas_common::types::Chain;
use atlas_mod_morpho::client::MorphoModule;
use crate::state::AppState;

/// GET /api/morpho/markets — list Morpho Blue lending markets.
async fn morpho_markets(State(state): State<Arc<AppState>>) -> Json<Value> {
    if !state.morpho_enabled {
        return Json(json!({ "error": "Morpho module disabled" }));
    }

    let chain = match state.config.modules.morpho.config.chain.as_str() {
        "base" => Chain::Base,
        _ => Chain::Ethereum,
    };

    let module = MorphoModule::new(chain);
    match module.markets_data().await {
        Ok(markets) => {
            let rows: Vec<Value> = markets.iter().map(|m| {
                json!({
                    "market_id": m.market_id,
                    "collateral": m.collateral_asset,
                    "loan": m.loan_asset,
                    "supply_apy": m.supply_apy.to_string(),
                    "borrow_apy": m.borrow_apy.to_string(),
                    "total_supply": m.total_supply.to_string(),
                    "utilization": m.utilization.to_string(),
                })
            }).collect();
            Json(json!({ "markets": rows }))
        }
        Err(e) => Json(json!({ "error": format!("{e}") })),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/morpho/markets", get(morpho_markets))
}
