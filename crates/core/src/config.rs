use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::risk::RiskConfig;

// ═══════════════════════════════════════════════════════════════════════
//  SIZE INPUT — how the user expresses position size
// ═══════════════════════════════════════════════════════════════════════

/// How the user specifies position size.
///
/// Atlas supports multiple ways to express size:
///   - **USDC**:   `atlas hl perp buy ETH 200` or `$200`
///   - **Units**:  `atlas hl perp buy ETH 0.5eth`
///   - **Lots**:   `atlas hl perp buy ETH 10lots`
///
/// Explicit suffixes always override the module's `default_size_mode`.
#[derive(Debug, Clone, PartialEq)]
pub enum SizeInput {
    /// Raw value — interpreted based on module's `default_size_mode`.
    Raw(f64),
    /// Explicit USDC margin: `$200`, `200$`, `200u`, `200usdc`.
    Usdc(f64),
    /// Explicit asset units: `0.5eth`, `0.5units`.
    Units(f64),
    /// Explicit lot count: `50lots`, `50l`.
    Lots(f64),
}

// ═══════════════════════════════════════════════════════════════════════
//  APP CONFIG — top-level, stored at ~/.atlas-os/atlas.json
// ═══════════════════════════════════════════════════════════════════════

/// Top-level configuration stored in `$HOME/.atlas-os/atlas.json`.
///
/// ```json
/// {
///   "system": {
///     "active_profile": "main",
///     "api_key": "ak_...",
///     "verbose": false
///   },
///   "modules": {
///     "hyperliquid": {
///       "enabled": true,
///       "network": "mainnet",
///       "mode": "futures",
///       "default_size_mode": "usdc",
///       "default_leverage": 5,
///       "default_slippage": 0.05,
///       "lots": { ... },
///       "risk": { ... }
///     },
///     "zero_x": {
///       "enabled": false,
///       "default_slippage_bps": 100
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// System-wide settings (profile, API key, verbosity).
    pub system: SystemConfig,
    /// Per-module configurations — each protocol owns its own settings.
    #[serde(default)]
    pub modules: ModulesConfig,
}

// ═══════════════════════════════════════════════════════════════════════
//  SYSTEM CONFIG
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// The currently active wallet profile name.
    pub active_profile: String,

    /// API key obtained from apps/frontend — required to authenticate
    /// with the Atlas OS backend gateway (apps/backend).
    ///
    /// Users obtain this key by logging in at the frontend dashboard.
    /// Without this key, commands that depend on the backend proxy
    /// (0x swaps, EVM RPC, market data) will fail with auth errors.
    #[serde(default)]
    pub api_key: Option<String>,

    /// Enable verbose tracing output.
    #[serde(default)]
    pub verbose: bool,
}

// ═══════════════════════════════════════════════════════════════════════
//  MODULES CONFIG — each protocol owns its own trading settings
// ═══════════════════════════════════════════════════════════════════════

/// Per-module configuration. Adding a new protocol = add a new field here.
///
/// Each module is fully self-contained: its enabled flag, protocol-specific
/// settings, trading defaults, lot table, and risk config all live here.
/// There is no global trading config — different protocols have different
/// concepts of size, leverage, and risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    #[serde(default = "default_hl_config")]
    pub hyperliquid: ModuleEntry<HyperliquidConfig>,

    #[serde(default = "default_zero_x_config")]
    pub zero_x: ModuleEntry<ZeroXConfig>,
}

/// A module entry: enabled flag + module-specific config (flattened into JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntry<T> {
    pub enabled: bool,
    #[serde(flatten)]
    pub config: T,
}

// ═══════════════════════════════════════════════════════════════════════
//  HYPERLIQUID MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════

/// Full configuration for the Hyperliquid module.
///
/// Trading defaults and risk settings live here — NOT in a global
/// trading block — because each protocol has its own concepts of leverage,
/// lot sizes, and risk parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidConfig {
    // ── Network ───────────────────────────────────────────────────────
    /// Network: "mainnet" or "testnet".
    #[serde(default = "default_hl_network")]
    pub network: String,

    // ── Trading defaults ──────────────────────────────────────────────
    /// Trading mode: "futures" (raw size) or "cfd" (lot-based).
    #[serde(default)]
    pub mode: TradingMode,

    /// How bare numbers are interpreted: "usdc", "units", or "lots".
    /// Default: "usdc" — most intuitive for new users.
    #[serde(default)]
    pub default_size_mode: SizeMode,

    /// Default leverage multiplier for new perp positions.
    #[serde(default = "default_leverage")]
    pub default_leverage: u32,

    /// Default slippage tolerance (0.05 = 5%).
    #[serde(default = "default_slippage")]
    pub default_slippage: f64,

    // ── CFD lot table ─────────────────────────────────────────────────
    /// Lot size configuration (only used in CFD mode).
    #[serde(default)]
    pub lots: LotConfig,

    // ── Risk ──────────────────────────────────────────────────────────
    /// Risk management settings for this module.
    #[serde(default)]
    pub risk: RiskConfig,
}

impl HyperliquidConfig {
    /// Resolve a `SizeInput` to (asset_units, margin_usdc_if_applicable).
    pub fn resolve_size_input(
        &self,
        coin: &str,
        input: &SizeInput,
        mark_price: f64,
        leverage_override: Option<u32>,
    ) -> (f64, Option<f64>) {
        let lev = leverage_override.unwrap_or(self.default_leverage).max(1) as f64;

        match input {
            SizeInput::Usdc(margin) => {
                if mark_price <= 0.0 {
                    (0.0, Some(*margin))
                } else {
                    let size = (margin * lev) / mark_price;
                    (size, Some(*margin))
                }
            }
            SizeInput::Units(units) => (*units, None),
            SizeInput::Lots(lots) => {
                let size = self.lots.lots_to_size(coin, *lots);
                (size, None)
            }
            SizeInput::Raw(raw) => match self.default_size_mode {
                SizeMode::Usdc => {
                    if mark_price <= 0.0 {
                        (0.0, Some(*raw))
                    } else {
                        let size = (raw * lev) / mark_price;
                        (size, Some(*raw))
                    }
                }
                SizeMode::Units => {
                    let size = match self.mode {
                        TradingMode::Futures => *raw,
                        TradingMode::Cfd => self.lots.lots_to_size(coin, *raw),
                    };
                    (size, None)
                }
                SizeMode::Lots => {
                    let size = self.lots.lots_to_size(coin, *raw);
                    (size, None)
                }
            },
        }
    }

    /// Format size for display.
    pub fn format_size(&self, coin: &str, raw_size: f64) -> String {
        match self.mode {
            TradingMode::Futures => format!("{raw_size} {coin}"),
            TradingMode::Cfd => {
                let lots = self.lots.size_to_lots(coin, raw_size);
                format!("{lots:.4} lots ({raw_size} {coin})")
            }
        }
    }

    /// Is CFD mode active?
    pub fn is_cfd(&self) -> bool {
        self.mode == TradingMode::Cfd
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  0x MODULE CONFIG
// ═══════════════════════════════════════════════════════════════════════

/// Configuration for the 0x swap module.
///
/// 0x is a DEX aggregator — its concept of "trading" differs from perps.
/// Slippage is expressed in basis points, there's no leverage or lots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroXConfig {
    /// Default slippage in basis points (100 = 1%). Default: 100.
    #[serde(default = "default_zero_x_slippage")]
    pub default_slippage_bps: u32,

    /// Default chain to swap on. Default: "ethereum".
    #[serde(default = "default_zero_x_chain")]
    pub default_chain: String,
}

impl Default for ZeroXConfig {
    fn default() -> Self {
        Self {
            default_slippage_bps: 100,
            default_chain: "ethereum".into(),
        }
    }
}

fn default_zero_x_slippage() -> u32 {
    100 // 1%
}
fn default_zero_x_chain() -> String {
    "ethereum".into()
}

// ═══════════════════════════════════════════════════════════════════════
//  TRADING ENUMS + LOT CONFIG
// ═══════════════════════════════════════════════════════════════════════

/// Trading mode for perp protocols.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    /// Standard futures: size is in asset units.
    #[default]
    Futures,
    /// CFD-style: size is in lots, converted via lot table.
    Cfd,
}

impl std::fmt::Display for TradingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingMode::Futures => write!(f, "futures"),
            TradingMode::Cfd => write!(f, "cfd"),
        }
    }
}

/// How bare numbers (without suffix) are interpreted in trade commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SizeMode {
    /// Bare number = USDC margin. Default — most intuitive.
    #[default]
    Usdc,
    /// Bare number = asset units.
    Units,
    /// Bare number = lots.
    Lots,
}

impl std::fmt::Display for SizeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeMode::Usdc => write!(f, "usdc"),
            SizeMode::Units => write!(f, "units"),
            SizeMode::Lots => write!(f, "lots"),
        }
    }
}

/// Lot size configuration for CFD mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotConfig {
    /// Default lot size for assets not in the custom table (in asset units per lot).
    pub default_lot_size: f64,
    /// Per-asset overrides. Key = coin symbol (e.g. "BTC"). Value = units per lot.
    #[serde(default)]
    pub assets: HashMap<String, f64>,
}

impl LotConfig {
    pub fn lot_size(&self, coin: &str) -> f64 {
        self.assets
            .get(coin)
            .copied()
            .unwrap_or(self.default_lot_size)
    }
    pub fn lots_to_size(&self, coin: &str, lots: f64) -> f64 {
        lots * self.lot_size(coin)
    }
    pub fn size_to_lots(&self, coin: &str, size: f64) -> f64 {
        let lot = self.lot_size(coin);
        if lot == 0.0 {
            size
        } else {
            size / lot
        }
    }
}

impl Default for LotConfig {
    fn default() -> Self {
        let mut assets = HashMap::new();
        assets.insert("BTC".to_string(), 0.001);
        assets.insert("ETH".to_string(), 0.01);
        assets.insert("SOL".to_string(), 1.0);
        assets.insert("DOGE".to_string(), 100.0);
        assets.insert("ARB".to_string(), 10.0);
        assets.insert("AVAX".to_string(), 1.0);
        assets.insert("MATIC".to_string(), 100.0);
        assets.insert("LINK".to_string(), 1.0);
        assets.insert("OP".to_string(), 10.0);
        assets.insert("SUI".to_string(), 10.0);
        Self {
            default_lot_size: 1.0,
            assets,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  DEFAULTS
// ═══════════════════════════════════════════════════════════════════════

fn default_hl_config() -> ModuleEntry<HyperliquidConfig> {
    ModuleEntry {
        enabled: true,
        config: HyperliquidConfig::default(),
    }
}

fn default_zero_x_config() -> ModuleEntry<ZeroXConfig> {
    ModuleEntry {
        enabled: false,
        config: ZeroXConfig::default(),
    }
}

fn default_hl_network() -> String {
    "mainnet".into()
}
fn default_leverage() -> u32 {
    1
}
fn default_slippage() -> f64 {
    0.05
}

impl Default for HyperliquidConfig {
    fn default() -> Self {
        Self {
            network: "mainnet".into(),
            mode: TradingMode::Futures,
            default_size_mode: SizeMode::Usdc,
            default_leverage: 1,
            default_slippage: 0.05,
            lots: LotConfig::default(),
            risk: RiskConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                active_profile: "default".into(),
                api_key: None,
                verbose: false,
            },
            modules: ModulesConfig::default(),
        }
    }
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            hyperliquid: default_hl_config(),
            zero_x: default_zero_x_config(),
        }
    }
}

impl AppConfig {
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.system.active_profile, "default");
        assert!(!config.system.verbose);
        assert!(config.system.api_key.is_none());
        assert!(config.modules.hyperliquid.enabled);
        assert_eq!(config.modules.hyperliquid.config.network, "mainnet");
        assert_eq!(config.modules.hyperliquid.config.mode, TradingMode::Futures);
        assert_eq!(
            config.modules.hyperliquid.config.default_size_mode,
            SizeMode::Usdc
        );
        assert_eq!(config.modules.hyperliquid.config.default_leverage, 1);
        assert!(!config.modules.zero_x.enabled);
    }

    #[test]
    fn test_config_roundtrip_json() {
        let config = AppConfig::default();
        let json = config.to_json_string().unwrap();
        let parsed = AppConfig::from_json_str(&json).unwrap();
        assert_eq!(parsed.system.active_profile, config.system.active_profile);
        assert_eq!(
            parsed.modules.hyperliquid.config.network,
            config.modules.hyperliquid.config.network
        );
        assert_eq!(parsed.modules.zero_x.enabled, config.modules.zero_x.enabled);
    }

    #[test]
    fn test_hl_resolve_size_usdc() {
        let cfg = HyperliquidConfig::default(); // USDC mode, 1x lev
        let (size, margin) = cfg.resolve_size_input("ETH", &SizeInput::Usdc(200.0), 3500.0, None);
        assert!((size - 200.0 / 3500.0).abs() < 1e-6);
        assert_eq!(margin, Some(200.0));
    }

    #[test]
    fn test_hl_resolve_size_units() {
        let cfg = HyperliquidConfig::default();
        let (size, margin) = cfg.resolve_size_input("ETH", &SizeInput::Units(0.5), 3500.0, None);
        assert_eq!(size, 0.5);
        assert!(margin.is_none());
    }

    #[test]
    fn test_hl_lot_defaults() {
        let cfg = HyperliquidConfig::default();
        assert_eq!(cfg.lots.lot_size("BTC"), 0.001);
        assert_eq!(cfg.lots.lot_size("ETH"), 0.01);
        assert_eq!(cfg.lots.lots_to_size("ETH", 100.0), 1.0);
    }

    #[test]
    fn test_zero_x_defaults() {
        let cfg = ZeroXConfig::default();
        assert_eq!(cfg.default_slippage_bps, 100);
        assert_eq!(cfg.default_chain, "ethereum");
    }

    #[test]
    fn test_no_global_api_url() {
        // Ensure api_url does NOT exist at top level — backend URL is hardcoded in code
        let json = AppConfig::default().to_json_string().unwrap();
        assert!(!json.contains("api_url"));
    }

    #[test]
    fn test_api_key_optional() {
        let mut config = AppConfig::default();
        assert!(config.system.api_key.is_none());
        config.system.api_key = Some("ak_test_123".into());
        let json = config.to_json_string().unwrap();
        let parsed = AppConfig::from_json_str(&json).unwrap();
        assert_eq!(parsed.system.api_key.as_deref(), Some("ak_test_123"));
    }
}
