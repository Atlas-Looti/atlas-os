//! Core orchestrator — routes commands to the correct protocol module.
//!
//! The orchestrator holds all active modules and provides a unified API
//! for the CLI, TUI, and backend to consume.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use atlas_common::traits::{LendingModule, PerpModule, SwapModule};
use atlas_common::types::*;
use atlas_types::config::AppConfig;

use crate::auth::AuthManager;
use crate::workspace::load_config;

/// The core orchestrator — holds all protocol modules.
pub struct Orchestrator {
    /// Perp modules keyed by protocol name.
    perp_modules: HashMap<String, Arc<dyn PerpModule>>,
    /// Lending modules keyed by protocol name.
    lending_modules: HashMap<String, Arc<dyn LendingModule>>,
    /// Swap modules keyed by protocol name.
    swap_modules: HashMap<String, Arc<dyn SwapModule>>,
    /// Default perp protocol (used when user doesn't specify).
    pub default_perp: Option<String>,
    /// Default lending protocol.
    pub default_lending: Option<String>,
    /// Default swap protocol.
    pub default_swap: Option<String>,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            perp_modules: HashMap::new(),
            lending_modules: HashMap::new(),
            swap_modules: HashMap::new(),
            default_perp: None,
            default_lending: None,
            default_swap: None,
        }
    }

    /// Register a perp module.
    pub fn add_perp(&mut self, module: Arc<dyn PerpModule>) {
        let name = module.protocol().to_string();
        if self.default_perp.is_none() {
            self.default_perp = Some(name.clone());
        }
        info!(protocol = %name, "registered perp module");
        self.perp_modules.insert(name, module);
    }

    /// Register a lending module.
    pub fn add_lending(&mut self, module: Arc<dyn LendingModule>) {
        let name = module.protocol().to_string();
        if self.default_lending.is_none() {
            self.default_lending = Some(name.clone());
        }
        info!(protocol = %name, "registered lending module");
        self.lending_modules.insert(name, module);
    }

    /// Register a swap module.
    pub fn add_swap(&mut self, module: Arc<dyn SwapModule>) {
        let name = module.protocol().to_string();
        if self.default_swap.is_none() {
            self.default_swap = Some(name.clone());
        }
        info!(protocol = %name, "registered swap module");
        self.swap_modules.insert(name, module);
    }

    /// Get a perp module by name, or the default.
    pub fn perp(&self, protocol: Option<&str>) -> Result<&Arc<dyn PerpModule>> {
        let name = protocol
            .map(|s| s.to_string())
            .or_else(|| self.default_perp.clone())
            .ok_or_else(|| anyhow::anyhow!("No perp module registered"))?;
        self.perp_modules
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("Unknown perp protocol: {name}"))
    }

    /// Get a lending module by name, or the default.
    pub fn lending(&self, protocol: Option<&str>) -> Result<&Arc<dyn LendingModule>> {
        let name = protocol
            .map(|s| s.to_string())
            .or_else(|| self.default_lending.clone())
            .ok_or_else(|| anyhow::anyhow!("No lending module registered"))?;
        self.lending_modules
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("Unknown lending protocol: {name}"))
    }

    /// Get a swap module by name, or the default.
    pub fn swap(&self, protocol: Option<&str>) -> Result<&Arc<dyn SwapModule>> {
        let name = protocol
            .map(|s| s.to_string())
            .or_else(|| self.default_swap.clone())
            .ok_or_else(|| anyhow::anyhow!("No swap module registered"))?;
        self.swap_modules
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("Unknown swap protocol: {name}"))
    }

    /// List all registered protocols.
    pub fn protocols(&self) -> Vec<ProtocolInfo> {
        let mut protos = Vec::new();
        for name in self.perp_modules.keys() {
            protos.push(ProtocolInfo {
                name: name.clone(),
                module_type: "perp".into(),
            });
        }
        for name in self.lending_modules.keys() {
            protos.push(ProtocolInfo {
                name: name.clone(),
                module_type: "lending".into(),
            });
        }
        for name in self.swap_modules.keys() {
            protos.push(ProtocolInfo {
                name: name.clone(),
                module_type: "swap".into(),
            });
        }
        protos
    }

    // ═══════════════════════════════════════════════════════════════════
    //  AGGREGATED QUERIES — combine data from all modules
    // ═══════════════════════════════════════════════════════════════════

    /// Get all markets from all perp modules.
    pub async fn all_markets(&self) -> Result<Vec<Market>> {
        let mut markets = Vec::new();
        for module in self.perp_modules.values() {
            match module.markets().await {
                Ok(m) => markets.extend(m),
                Err(e) => info!(error = %e, "failed to fetch markets from module"),
            }
        }
        Ok(markets)
    }

    /// Get all tickers from all perp modules.
    pub async fn all_tickers(&self) -> Result<Vec<Ticker>> {
        let mut tickers = Vec::new();
        for module in self.perp_modules.values() {
            match module.all_tickers().await {
                Ok(t) => tickers.extend(t),
                Err(e) => info!(error = %e, "failed to fetch tickers from module"),
            }
        }
        tickers.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        Ok(tickers)
    }

    /// Get all positions from all perp modules.
    pub async fn all_positions(&self) -> Result<Vec<Position>> {
        let mut positions = Vec::new();
        for module in self.perp_modules.values() {
            match module.positions().await {
                Ok(p) => positions.extend(p),
                Err(e) => info!(error = %e, "failed to fetch positions from module"),
            }
        }
        Ok(positions)
    }

    /// Get all balances from all modules.
    pub async fn all_balances(&self) -> Result<Vec<Balance>> {
        let mut balances = Vec::new();
        for module in self.perp_modules.values() {
            match module.balances().await {
                Ok(b) => balances.extend(b),
                Err(e) => info!(error = %e, "failed to fetch balances from module"),
            }
        }
        Ok(balances)
    }

    /// Build an Orchestrator from config — registers enabled modules.
    ///
    /// For perp modules that require signing (Hyperliquid), the signer
    /// must be provided. For read-only usage, pass `None`.
    pub async fn from_config(
        config: &AppConfig,
        signer: Option<alloy::signers::local::PrivateKeySigner>,
    ) -> Result<Self> {
        let mut orch = Self::new();

        // ── Hyperliquid (perp) ──────────────────────────────────
        if config.modules.hyperliquid.enabled {
            let testnet = config.modules.hyperliquid.config.network == "testnet";
            let hl = match signer {
                Some(s) => atlas_mod_hyperliquid::client::HyperliquidModule::new(s, testnet).await,
                None => {
                    atlas_mod_hyperliquid::client::HyperliquidModule::new_readonly(testnet).await
                }
            }
            .map_err(|e| anyhow::anyhow!("{e}"))?;
            orch.add_perp(Arc::new(hl));
            info!("Hyperliquid perp module loaded");
        }

        // ── 0x (swap) ───────────────────────────────────────────
        if config.modules.zero_x.enabled {
            let backend_url = "https://api.atlas-os.ai".to_string(); // Hardcoded default backend
            let default_chain =
                atlas_mod_zero_x::parse_chain(&config.modules.zero_x.config.default_chain);
            let default_slippage_bps = config.modules.zero_x.config.default_slippage_bps;
            let zero_x = atlas_mod_zero_x::client::ZeroXModule::new(backend_url)
                .with_api_key(config.system.api_key.clone())
                .with_defaults(default_chain, default_slippage_bps);
            orch.add_swap(Arc::new(zero_x));
            info!("0x swap module loaded");
        }

        Ok(orch)
    }

    /// Convenience: load config, load active wallet signer, and build Orchestrator.
    ///
    /// This is the main entry point for CLI commands that need trading access.
    pub async fn from_active_profile() -> Result<Self> {
        let config = load_config()?;
        let signer = AuthManager::load_active_signer(&config)?;
        Self::from_config(&config, Some(signer)).await
    }

    /// Convenience: build a read-only Orchestrator (no signer needed).
    ///
    /// For commands that only need market data (prices, markets, candles, funding).
    pub async fn readonly() -> Result<Self> {
        let config = load_config()?;
        Self::from_config(&config, None).await
    }
}

/// Protocol registration info.
#[derive(Debug, Clone)]
pub struct ProtocolInfo {
    pub name: String,
    pub module_type: String,
}
