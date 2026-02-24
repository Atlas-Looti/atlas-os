//! Orchestrator factory — builds the orchestrator with active modules.
//!
//! Lives in `cli` because `core` must NOT depend on protocol modules
//! (that would create circular deps).

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use atlas_core::auth::AuthManager;
use atlas_core::config::AppConfig;
use atlas_core::workspace::load_config;
use atlas_core::Orchestrator;

/// Build an Orchestrator from config — registers enabled modules.
pub async fn from_config(
    config: &AppConfig,
    signer: Option<alloy::signers::local::PrivateKeySigner>,
) -> Result<Orchestrator> {
    let mut orch = Orchestrator::new();

    // ── Hyperliquid (perp) ──────────────────────────────────
    if config.modules.hyperliquid.enabled {
        let testnet = config.modules.hyperliquid.config.network == "testnet";
        let hl = match signer.clone() {
            Some(s) => atlas_hl::client::HyperliquidModule::new(s, testnet).await,
            None => atlas_hl::client::HyperliquidModule::new_readonly(testnet).await,
        }
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        orch.add_perp(Arc::new(hl));
        info!("Hyperliquid perp module loaded");
    }

    // ── 0x (swap) ───────────────────────────────────────────
    if config.modules.zero_x.enabled {
        let backend_url = "https://api.atlas-os.ai".to_string();
        let default_chain = atlas_zero_x::parse_chain(&config.modules.zero_x.config.default_chain);
        let default_slippage_bps = config.modules.zero_x.config.default_slippage_bps;
        let mut zero_x = atlas_zero_x::client::ZeroXModule::new(backend_url)
            .with_api_key(config.system.api_key.clone())
            .with_defaults(default_chain, default_slippage_bps);

        // Pass signer for on-chain execution (same wallet as HL)
        if let Some(ref s) = signer {
            zero_x = zero_x.with_signer(s.clone());
        }

        orch.add_swap(Arc::new(zero_x));
        info!("0x swap module loaded");
    }

    Ok(orch)
}

/// Load config, load active wallet signer, and build Orchestrator.
pub async fn from_active_profile() -> Result<Orchestrator> {
    let config = load_config()?;
    let signer = AuthManager::load_active_signer(&config)?;
    from_config(&config, Some(signer)).await
}

/// Build a read-only Orchestrator (no signer needed).
pub async fn readonly() -> Result<Orchestrator> {
    let config = load_config()?;
    from_config(&config, None).await
}
