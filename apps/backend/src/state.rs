//! Shared application state for the API server.

use atlas_core::Orchestrator;
use atlas_types::config::AppConfig;

use crate::clients::alchemy::AlchemyClient;
use crate::clients::cache::Cache;
use crate::clients::coingecko::{CoinGeckoClient, CoinGeckoTier};

/// Backend application state — shared across all request handlers.
pub struct AppState {
    pub config: AppConfig,
    pub orchestrator: Orchestrator,
    pub hl_enabled: bool,
    pub morpho_enabled: bool,
    /// Alchemy multi-chain data API client (None if no API key configured).
    pub alchemy: Option<AlchemyClient>,
    /// CoinGecko market data API client (None if no API key configured).
    pub coingecko: Option<CoinGeckoClient>,
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

        // CoinGecko client — from env var
        let coingecko = std::env::var("COINGECKO_API_KEY").ok().map(|key| {
            let tier = match std::env::var("COINGECKO_TIER")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "pro" => CoinGeckoTier::Pro,
                _ => CoinGeckoTier::Demo,
            };
            tracing::info!("CoinGecko API key found — tier: {:?}, market data APIs enabled", tier);
            CoinGeckoClient::new(&key, tier)
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
            coingecko,
            cache,
        })
    }
}
