//! Health check endpoint.

use std::sync::Arc;
use axum::{Router, Json, routing::get};
use serde_json::{json, Value};
use crate::state::AppState;

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "atlas-os-backend",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health))
}
