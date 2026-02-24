//! Atlas OS Backend — API Gateway
//!
//! Responsibilities:
//! - REST API for frontend dashboard
//! - RPC proxy (protect premium API keys)
//! - WebSocket relay for real-time data
//! - Module status and health checks

mod routes;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Initialize workspace (config, keystore, etc.)
    atlas_core::init_workspace()?;
    let config = atlas_core::workspace::load_config()?;

    tracing::info!("Atlas OS Backend starting...");

    let state = Arc::new(AppState::from_config(&config)?);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::api_router())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
