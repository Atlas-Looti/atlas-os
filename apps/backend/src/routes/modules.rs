//! Module status endpoint.

use std::sync::Arc;
use axum::{Router, Json, routing::get, extract::State};
use serde_json::{json, Value};
use crate::state::AppState;

async fn list_modules(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "modules": [
            {
                "name": "hyperliquid",
                "type": "perp",
                "enabled": state.hl_enabled,
                "config": {
                    "network": state.config.modules.hyperliquid.config.network,
                    "rpc_url": state.config.modules.hyperliquid.config.rpc_url,
                }
            },
            {
                "name": "morpho",
                "type": "lending",
                "enabled": state.morpho_enabled,
                "config": {
                    "chain": state.config.modules.morpho.config.chain,
                }
            }
        ]
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/modules", get(list_modules))
}
