use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use reqwest::Client;
use serde_json::Value;

use crate::state::AppState;
use atlas_common::error::{AtlasError, AtlasResult};

/// 0x API v2 base URL.
const ZEROX_API_BASE: &str = "https://api.0x.org";

/// Build the ZeroX proxy router.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/zerox/swap/allowance-holder/price", get(proxy_price))
        .route("/zerox/swap/allowance-holder/quote", get(proxy_quote))
        .route("/zerox/swap/chains", get(proxy_chains))
        .route("/zerox/sources", get(proxy_sources))
        .route("/zerox/trade-analytics/swap", get(proxy_trade_analytics))
}

/// Generic proxy handler for 0x API requests.
async fn proxy_request(
    state: &AppState,
    endpoint: &str,
    query: &[(String, String)],
) -> AtlasResult<Json<Value>> {
    let api_key = state.zerox_api_key.as_deref().unwrap_or_default();

    if api_key.is_empty() {
        return Err(AtlasError::Other(
            "ZEROX_API_KEY is not configured on the backend".into(),
        ));
    }

    let url = format!("{}{}", ZEROX_API_BASE, endpoint);
    let client = Client::new();

    let resp = client
        .get(&url)
        .header("0x-api-key", api_key)
        .header("0x-version", "v2")
        .query(&query)
        .send()
        .await
        .map_err(|e| AtlasError::Network(format!("Failed to reach 0x API: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AtlasError::Protocol {
            protocol: "0x".into(),
            message: format!("0x API error {status}: {text}"),
        });
    }

    let json: Value = resp
        .json()
        .await
        .map_err(|e| AtlasError::Other(format!("Failed to parse 0x response: {e}")))?;

    Ok(Json(json))
}

// ── Handlers ──────────────────────────────────────────────────────────

async fn proxy_price(
    State(state): State<Arc<AppState>>,
    Query(query): Query<Vec<(String, String)>>,
) -> Json<Value> {
    match proxy_request(&state, "/swap/allowance-holder/price", &query).await {
        Ok(data) => data,
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn proxy_quote(
    State(state): State<Arc<AppState>>,
    Query(query): Query<Vec<(String, String)>>,
) -> Json<Value> {
    match proxy_request(&state, "/swap/allowance-holder/quote", &query).await {
        Ok(data) => data,
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn proxy_chains(
    State(state): State<Arc<AppState>>,
    Query(query): Query<Vec<(String, String)>>,
) -> Json<Value> {
    match proxy_request(&state, "/swap/chains", &query).await {
        Ok(data) => data,
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn proxy_sources(
    State(state): State<Arc<AppState>>,
    Query(query): Query<Vec<(String, String)>>,
) -> Json<Value> {
    match proxy_request(&state, "/sources", &query).await {
        Ok(data) => data,
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn proxy_trade_analytics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<Vec<(String, String)>>,
) -> Json<Value> {
    match proxy_request(&state, "/trade-analytics/swap", &query).await {
        Ok(data) => data,
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}
