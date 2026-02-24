//! Atlas OS Backend — API Gateway & Data Service
//!
//! Responsibilities:
//! - REST API for frontend dashboard + CLI
//! - Multi-chain EVM data via Alchemy (token balances, prices, portfolio)
//! - Redis caching for rate-limit management and performance
//! - Protocol module proxy (Hyperliquid, Morpho)
//! - WebSocket relay for real-time data (planned)

mod clients;
mod routes;
mod services;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    atlas_core::init_workspace()?;
    let config = atlas_core::workspace::load_config()?;

    tracing::info!("Atlas OS Backend starting...");

    let state = Arc::new(AppState::from_config(&config).await?);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::api_router())
        .layer(cors)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{addr}");
    tracing::info!("Endpoints:");
    tracing::info!("  GET /api/health");
    tracing::info!("  GET /api/modules");
    tracing::info!("  GET /api/markets");
    tracing::info!("  GET /api/portfolio/:address");
    tracing::info!("  GET /api/tokens/:network/:contract/metadata");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
