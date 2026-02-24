//! Shared application state for the API server.

use atlas_types::config::AppConfig;

/// Backend application state — shared across all request handlers.
pub struct AppState {
    pub config: AppConfig,
    pub hl_enabled: bool,
    pub morpho_enabled: bool,
}

impl AppState {
    pub fn from_config(config: &AppConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.clone(),
            hl_enabled: config.modules.hyperliquid.enabled,
            morpho_enabled: config.modules.morpho.enabled,
        })
    }
}
