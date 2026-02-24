//! API routes for Atlas OS backend.

pub mod coingecko;
pub mod health;
pub mod markets;
pub mod modules;
pub mod morpho;
pub mod portfolio;
pub mod zerox;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

/// Build the API router with all routes.
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::router())
        .merge(modules::router())
        .merge(markets::router())
        .merge(morpho::router())
        .merge(portfolio::router())
        .merge(coingecko::router())
        .merge(zerox::router())
}
