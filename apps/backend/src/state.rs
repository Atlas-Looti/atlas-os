//! Shared application state for the API server.

use atlas_core::Orchestrator;
use atlas_types::config::AppConfig;

use crate::clients::alchemy::AlchemyClient;
use crate::clients::cache::Cache;

/// Backend application state — shared across all request handlers.
pub struct AppState {
    pub config: AppConfig,
    pub orchestrator: Orchestrator,
    pub hl_enabled: bool,
    pub morpho_enabled: bool,
    /// Alchemy multi-chain data API client (None if no API key configured).
    pub alchemy: Option<AlchemyClient>,
    /// Redis cache (None if Redis not available).
    pub cache: Option<Cache>,
}

impl AppState {
    pub async fn from_config(config: &AppConfig) -> anyhow::Result<Self> {
        let orchestrator = Orchestrator::from_config(config, None).await?;

        // Alchemy client — from env var
        let alchemy = std::env::var("ALCHEMY_API_KEY").ok().map(|key| {
            tracing::info!("Alchemy API key found — EVM data APIs enabled");
            AlchemyClient::new(&key)
        });

        // Redis — from env var or default
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let cache = match Cache::new(&redis_url).await {
            Ok(c) => {
                tracing::info!("Redis connected at {redis_url}");
                Some(c)
            }
            Err(e) => {
                tracing::warn!("Redis unavailable ({e}) — running without cache");
                None
            }
        };

        Ok(Self {
            config: config.clone(),
            orchestrator,
            hl_enabled: config.modules.hyperliquid.enabled,
            morpho_enabled: config.modules.morpho.enabled,
            alchemy,
            cache,
        })
    }
}
