//! Shared application state for the API server.

use atlas_core::Orchestrator;
use atlas_types::config::AppConfig;

/// Backend application state — shared across all request handlers.
pub struct AppState {
    pub config: AppConfig,
    pub orchestrator: Orchestrator,
    pub hl_enabled: bool,
    pub morpho_enabled: bool,
}

impl AppState {
    pub async fn from_config(config: &AppConfig) -> anyhow::Result<Self> {
        // Backend runs in read-only mode (no signer) for market data proxying.
        // Authenticated endpoints will load signer on-demand.
        let orchestrator = Orchestrator::from_config(config, None).await?;

        Ok(Self {
            config: config.clone(),
            orchestrator,
            hl_enabled: config.modules.hyperliquid.enabled,
            morpho_enabled: config.modules.morpho.enabled,
        })
    }
}
