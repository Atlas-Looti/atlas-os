//! API routes for Atlas OS backend.

pub mod health;
pub mod modules;
pub mod markets;
pub mod morpho;

use std::sync::Arc;
use axum::Router;
use crate::state::AppState;

/// Build the API router with all routes.
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::router())
        .merge(modules::router())
        .merge(markets::router())
        .merge(morpho::router())
}
