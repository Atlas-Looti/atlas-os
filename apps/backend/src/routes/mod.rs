//! API routes for Atlas OS backend.

pub mod health;
pub mod markets;
pub mod modules;
pub mod morpho;
pub mod portfolio;

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
        .merge(portfolio::router())
}
