use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::risk::RiskConfig;

/// How the user specifies position size.
///
/// Atlas supports multiple ways to express size:
///   - **USDC**:   `atlas buy ETH $200` or `200` (if default_size_mode = usdc)
///   - **Units**:  `atlas buy ETH 0.5eth` or `0.5` (if default_size_mode = units)
///   - **Lots**:   `atlas buy ETH 50lots` or `50` (if default_size_mode = lots)
///
/// Explicit suffixes always override default_size_mode.
#[derive(Debug, Clone, PartialEq)]
pub enum SizeInput {
    /// Raw value — interpreted based on `default_size_mode` config.
    Raw(f64),
    /// Explicit USDC margin: `$200`, `200$`, `200u`, `200usdc`.
    Usdc(f64),
    /// Explicit asset units: `0.5eth`, `0.5units`.
    Units(f64),
    /// Explicit lot count: `50lots`, `50l`.
    Lots(f64),
}

/// Top-level configuration stored in `$HOME/.atlas-os/atlas.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub trading: TradingConfig,
    pub risk: RiskConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// The currently active profile name (maps to a keyring entry).
    pub active_profile: String,
    /// Enable verbose tracing output.
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Trading mode: "futures" (raw size) or "cfd" (lot-based).
    pub mode: TradingMode,
    /// How bare numbers are interpreted: "usdc", "units", or "lots".
    /// Default: "usdc" — most intuitive for all traders.
    #[serde(default = "default_size_mode")]
    pub default_size_mode: SizeMode,
    /// Default leverage multiplier for new positions.
    pub default_leverage: u32,
    /// Default slippage tolerance (0.05 = 5%).
    pub default_slippage: f64,
    /// CFD lot configuration (only used in CFD mode).
    pub lots: LotConfig,
}

/// How bare numbers (without suffix) are interpreted in trade commands.
///
/// Examples with `atlas buy ETH 200`:
///   - `Usdc`  → $200 margin (needs mark price + leverage to compute size)
///   - `Units` → 200 ETH (raw asset units)
///   - `Lots`  → 200 lots (CFD lot-based, converted via lot table)
///
/// Explicit suffixes always override: `$200`, `200u`, `0.5eth`, `50lots`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SizeMode {
    /// Bare numbers = USDC margin. Most intuitive for casual traders.
    Usdc,
    /// Bare numbers = asset units. For pro traders who think in units.
    Units,
    /// Bare numbers = lots. For CFD-style trading.
    Lots,
}

fn default_size_mode() -> SizeMode {
    SizeMode::Usdc
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

/// Trading mode determines how size is interpreted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradingMode {
    /// Standard futures: size is in asset units (0.1 ETH, 1 BTC).
    Futures,
    /// CFD-style: size is in lots. Atlas converts lots → asset units
    /// using the lot size table before sending to Hyperliquid.
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

/// Lot size configuration for CFD mode.
///
/// Standard lot sizes (customizable per asset):
/// - 1 standard lot  = `standard_lot_size` units of the asset
/// - 1 mini lot      = 0.1 standard lot
/// - 1 micro lot     = 0.01 standard lot
///
/// Example: if BTC standard lot = 1.0, then:
///   atlas buy BTC 0.1   → 0.1 lots = 0.1 BTC
///   atlas buy BTC 1     → 1 lot    = 1.0 BTC
///   atlas buy BTC 0.01  → 0.01 lot = 0.01 BTC (micro lot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotConfig {
    /// Default lot size for assets not in the custom table.
    /// In asset units per 1 standard lot.
    pub default_lot_size: f64,
    /// Per-asset lot size overrides. Key = coin symbol (e.g. "BTC").
    /// Value = units of asset per 1 standard lot.
    #[serde(default)]
    pub assets: HashMap<String, f64>,
}

impl LotConfig {
    /// Get the lot size for a given asset.
    pub fn lot_size(&self, coin: &str) -> f64 {
        self.assets
            .get(coin)
            .copied()
            .unwrap_or(self.default_lot_size)
    }

    /// Convert lots → asset units.
    pub fn lots_to_size(&self, coin: &str, lots: f64) -> f64 {
        lots * self.lot_size(coin)
    }

    /// Convert asset units → lots.
    pub fn size_to_lots(&self, coin: &str, size: f64) -> f64 {
        let lot = self.lot_size(coin);
        if lot == 0.0 {
            size
        } else {
            size / lot
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Hyperliquid RPC endpoint (mainnet or testnet).
    pub rpc_url: String,
    /// Use testnet mode.
    pub testnet: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut default_assets = HashMap::new();
        // Popular assets with sensible default lot sizes
        default_assets.insert("BTC".to_string(), 0.001); // 1 lot = 0.001 BTC (~$100 at 100k)
        default_assets.insert("ETH".to_string(), 0.01); // 1 lot = 0.01 ETH (~$35 at 3.5k)
        default_assets.insert("SOL".to_string(), 1.0); // 1 lot = 1 SOL
        default_assets.insert("DOGE".to_string(), 100.0); // 1 lot = 100 DOGE
        default_assets.insert("ARB".to_string(), 10.0); // 1 lot = 10 ARB
        default_assets.insert("AVAX".to_string(), 1.0); // 1 lot = 1 AVAX
        default_assets.insert("MATIC".to_string(), 100.0); // 1 lot = 100 MATIC
        default_assets.insert("LINK".to_string(), 1.0); // 1 lot = 1 LINK
        default_assets.insert("OP".to_string(), 10.0); // 1 lot = 10 OP
        default_assets.insert("SUI".to_string(), 10.0); // 1 lot = 10 SUI

        Self {
            general: GeneralConfig {
                active_profile: String::from("default"),
                verbose: false,
            },
            trading: TradingConfig {
                mode: TradingMode::Futures,
                default_size_mode: SizeMode::Usdc,
                default_leverage: 1,
                default_slippage: 0.05,
                lots: LotConfig {
                    default_lot_size: 1.0,
                    assets: default_assets,
                },
            },
            risk: RiskConfig::default(),
            network: NetworkConfig {
                rpc_url: String::from("https://api.hyperliquid.xyz"),
                testnet: false,
            },
        }
    }
}

impl AppConfig {
    /// Serialize to JSON string for writing to disk.
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from a JSON string.
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Check if we're in CFD lot mode.
    pub fn is_cfd(&self) -> bool {
        self.trading.mode == TradingMode::Cfd
    }

    /// Convert user input size to actual asset size, respecting trading mode.
    /// In futures mode: returns size as-is.
    /// In CFD mode: converts lots → asset units.
    pub fn resolve_size(&self, coin: &str, input_size: f64) -> f64 {
        match self.trading.mode {
            TradingMode::Futures => input_size,
            TradingMode::Cfd => self.trading.lots.lots_to_size(coin, input_size),
        }
    }

    /// Resolve a `SizeInput` to asset units.
    ///
    /// - `Raw(x)` → depends on `default_size_mode`:
    ///   - `Usdc`  → treat as USDC margin
    ///   - `Units` → treat as asset units (with CFD lot conversion if needed)
    ///   - `Lots`  → treat as lots → convert to units
    /// - `Usdc(x)` → x is margin in USDC (explicit)
    /// - `Units(x)` → x is asset units (explicit, bypasses lot conversion)
    /// - `Lots(x)` → x is lots → convert to units via lot table
    ///
    /// Returns `(asset_size, margin_usdc_if_applicable)`.
    pub fn resolve_size_input(
        &self,
        coin: &str,
        input: &SizeInput,
        mark_price: f64,
        leverage: Option<u32>,
    ) -> (f64, Option<f64>) {
        let lev = leverage.unwrap_or(self.trading.default_leverage).max(1) as f64;

        match input {
            // Explicit types — always do the same thing regardless of config
            SizeInput::Usdc(margin_usdc) => {
                if mark_price <= 0.0 {
                    (0.0, Some(*margin_usdc))
                } else {
                    let notional = margin_usdc * lev;
                    let size = notional / mark_price;
                    (size, Some(*margin_usdc))
                }
            }
            SizeInput::Units(units) => (*units, None),
            SizeInput::Lots(lots) => {
                let size = self.trading.lots.lots_to_size(coin, *lots);
                (size, None)
            }

            // Raw — interpret based on default_size_mode
            SizeInput::Raw(raw) => match self.trading.default_size_mode {
                SizeMode::Usdc => {
                    if mark_price <= 0.0 {
                        (0.0, Some(*raw))
                    } else {
                        let notional = raw * lev;
                        let size = notional / mark_price;
                        (size, Some(*raw))
                    }
                }
                SizeMode::Units => {
                    let size = self.resolve_size(coin, *raw);
                    (size, None)
                }
                SizeMode::Lots => {
                    let size = self.trading.lots.lots_to_size(coin, *raw);
                    (size, None)
                }
            },
        }
    }

    /// Format a SizeInput for display before price is known.
    pub fn format_size_input(&self, coin: &str, input: &SizeInput) -> String {
        match input {
            SizeInput::Usdc(usd) => format!("${:.2} USDC", usd),
            SizeInput::Units(u) => format!("{} {}", u, coin),
            SizeInput::Lots(l) => {
                let size = self.trading.lots.lots_to_size(coin, *l);
                format!("{:.4} lots ({} {})", l, size, coin)
            }
            SizeInput::Raw(raw) => match self.trading.default_size_mode {
                SizeMode::Usdc => format!("${:.2} USDC", raw),
                SizeMode::Units => self.format_size(coin, self.resolve_size(coin, *raw)),
                SizeMode::Lots => {
                    let size = self.trading.lots.lots_to_size(coin, *raw);
                    format!("{:.4} lots ({} {})", raw, size, coin)
                }
            },
        }
    }

    /// Format size for display, respecting trading mode.
    pub fn format_size(&self, coin: &str, raw_size: f64) -> String {
        match self.trading.mode {
            TradingMode::Futures => format!("{raw_size} {coin}"),
            TradingMode::Cfd => {
                let lots = self.trading.lots.size_to_lots(coin, raw_size);
                format!("{lots:.4} lots ({raw_size} {coin})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.active_profile, "default");
        assert!(!config.general.verbose);
        assert_eq!(config.trading.mode, TradingMode::Futures);
        assert_eq!(config.trading.default_leverage, 1);
        assert!(!config.network.testnet);
    }

    #[test]
    fn test_config_roundtrip_json() {
        let config = AppConfig::default();
        let json_str = config.to_json_string().unwrap();
        let parsed = AppConfig::from_json_str(&json_str).unwrap();
        assert_eq!(parsed.general.active_profile, config.general.active_profile);
        assert_eq!(parsed.trading.mode, config.trading.mode);
        assert_eq!(parsed.network.testnet, config.network.testnet);
    }

    #[test]
    fn test_config_cfd_mode_roundtrip() {
        let mut config = AppConfig::default();
        config.trading.mode = TradingMode::Cfd;
        let json_str = config.to_json_string().unwrap();
        let parsed = AppConfig::from_json_str(&json_str).unwrap();
        assert_eq!(parsed.trading.mode, TradingMode::Cfd);
        assert!(parsed.is_cfd());
    }

    #[test]
    fn test_lot_size_default() {
        let config = AppConfig::default();
        // Unknown asset falls back to default_lot_size
        assert_eq!(config.trading.lots.lot_size("UNKNOWN"), 1.0);
    }

    #[test]
    fn test_lot_size_custom_asset() {
        let config = AppConfig::default();
        // BTC has custom lot size
        assert_eq!(config.trading.lots.lot_size("BTC"), 0.001);
        assert_eq!(config.trading.lots.lot_size("ETH"), 0.01);
    }

    #[test]
    fn test_lots_to_size_conversion() {
        let config = AppConfig::default();
        let lots = &config.trading.lots;
        // 1 lot of BTC = 0.001 BTC
        assert!((lots.lots_to_size("BTC", 1.0) - 0.001).abs() < 1e-10);
        // 10 lots of BTC = 0.01 BTC
        assert!((lots.lots_to_size("BTC", 10.0) - 0.01).abs() < 1e-10);
        // 1 lot of ETH = 0.01 ETH
        assert!((lots.lots_to_size("ETH", 1.0) - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_size_to_lots_conversion() {
        let config = AppConfig::default();
        let lots = &config.trading.lots;
        // 0.001 BTC = 1 lot
        assert!((lots.size_to_lots("BTC", 0.001) - 1.0).abs() < 1e-10);
        // 0.05 ETH = 5 lots
        assert!((lots.size_to_lots("ETH", 0.05) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_resolve_size_futures_mode() {
        let config = AppConfig::default(); // default = futures
        assert!(!config.is_cfd());
        // In futures mode, size passes through unchanged
        assert_eq!(config.resolve_size("ETH", 0.5), 0.5);
        assert_eq!(config.resolve_size("BTC", 1.0), 1.0);
    }

    #[test]
    fn test_resolve_size_cfd_mode() {
        let mut config = AppConfig::default();
        config.trading.mode = TradingMode::Cfd;
        assert!(config.is_cfd());
        // In CFD mode, 1 lot ETH = 0.01 ETH
        assert!((config.resolve_size("ETH", 1.0) - 0.01).abs() < 1e-10);
        // 5 lots ETH = 0.05 ETH
        assert!((config.resolve_size("ETH", 5.0) - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_format_size_futures() {
        let config = AppConfig::default();
        assert_eq!(config.format_size("ETH", 0.5), "0.5 ETH");
    }

    #[test]
    fn test_format_size_cfd() {
        let mut config = AppConfig::default();
        config.trading.mode = TradingMode::Cfd;
        // 0.01 ETH = 1 lot (ETH lot size = 0.01)
        let display = config.format_size("ETH", 0.01);
        assert!(display.contains("1.0000 lots"));
        assert!(display.contains("0.01 ETH"));
    }

    #[test]
    fn test_trading_mode_display() {
        assert_eq!(format!("{}", TradingMode::Futures), "futures");
        assert_eq!(format!("{}", TradingMode::Cfd), "cfd");
    }

    // ── SizeInput / USDC sizing tests ───────────────────────────

    #[test]
    fn test_resolve_size_input_raw_default_usdc() {
        let config = AppConfig::default(); // default_size_mode = Usdc
        assert_eq!(config.trading.default_size_mode, SizeMode::Usdc);
        // Raw(200) with USDC default → $200 margin, 1x lev, ETH@3500
        // notional = $200, size = 200/3500 ≈ 0.05714
        let (size, margin) = config.resolve_size_input("ETH", &SizeInput::Raw(200.0), 3500.0, None);
        assert!((size - 200.0 / 3500.0).abs() < 1e-6);
        assert_eq!(margin, Some(200.0));
    }

    #[test]
    fn test_resolve_size_input_raw_units_mode() {
        let mut config = AppConfig::default();
        config.trading.default_size_mode = SizeMode::Units;
        // Raw(0.5) with Units default → 0.5 ETH
        let (size, margin) = config.resolve_size_input("ETH", &SizeInput::Raw(0.5), 3500.0, None);
        assert_eq!(size, 0.5);
        assert!(margin.is_none());
    }

    #[test]
    fn test_resolve_size_input_raw_lots_mode() {
        let mut config = AppConfig::default();
        config.trading.default_size_mode = SizeMode::Lots;
        // Raw(100) with Lots default → 100 × 0.01 ETH/lot = 1.0 ETH
        let (size, margin) = config.resolve_size_input("ETH", &SizeInput::Raw(100.0), 3500.0, None);
        assert!((size - 1.0).abs() < 1e-10);
        assert!(margin.is_none());
    }

    #[test]
    fn test_resolve_size_input_explicit_usdc() {
        let mut config = AppConfig::default();
        config.trading.default_size_mode = SizeMode::Units; // even in units mode
                                                            // Usdc(200) is always USDC regardless of default_size_mode
        let (size, margin) =
            config.resolve_size_input("ETH", &SizeInput::Usdc(200.0), 3500.0, Some(10));
        let expected = (200.0 * 10.0) / 3500.0;
        assert!((size - expected).abs() < 1e-6);
        assert_eq!(margin, Some(200.0));
    }

    #[test]
    fn test_resolve_size_input_explicit_units() {
        let config = AppConfig::default(); // usdc default
                                           // Units(0.5) is always 0.5 regardless of default_size_mode
        let (size, margin) = config.resolve_size_input("ETH", &SizeInput::Units(0.5), 3500.0, None);
        assert_eq!(size, 0.5);
        assert!(margin.is_none());
    }

    #[test]
    fn test_resolve_size_input_explicit_lots() {
        let config = AppConfig::default(); // usdc default
                                           // Lots(100) → 100 × 0.01 = 1.0 ETH regardless of default_size_mode
        let (size, margin) =
            config.resolve_size_input("ETH", &SizeInput::Lots(100.0), 3500.0, None);
        assert!((size - 1.0).abs() < 1e-10);
        assert!(margin.is_none());
    }

    #[test]
    fn test_resolve_size_input_usdc_btc() {
        let config = AppConfig::default();
        // $500 margin, 5x leverage, BTC at $100,000
        let (size, _) =
            config.resolve_size_input("BTC", &SizeInput::Usdc(500.0), 100_000.0, Some(5));
        assert!((size - 0.025).abs() < 1e-6);
    }

    #[test]
    fn test_resolve_size_input_usdc_zero_price() {
        let config = AppConfig::default();
        let (size, margin) = config.resolve_size_input("ETH", &SizeInput::Usdc(200.0), 0.0, None);
        assert_eq!(size, 0.0);
        assert_eq!(margin, Some(200.0));
    }

    #[test]
    fn test_format_size_input_raw_usdc_mode() {
        let config = AppConfig::default();
        let display = config.format_size_input("ETH", &SizeInput::Raw(200.0));
        assert_eq!(display, "$200.00 USDC");
    }

    #[test]
    fn test_format_size_input_raw_units_mode() {
        let mut config = AppConfig::default();
        config.trading.default_size_mode = SizeMode::Units;
        let display = config.format_size_input("ETH", &SizeInput::Raw(0.5));
        assert_eq!(display, "0.5 ETH");
    }

    #[test]
    fn test_format_size_input_explicit_usdc() {
        let config = AppConfig::default();
        let display = config.format_size_input("ETH", &SizeInput::Usdc(200.0));
        assert_eq!(display, "$200.00 USDC");
    }

    #[test]
    fn test_format_size_input_explicit_units() {
        let config = AppConfig::default();
        let display = config.format_size_input("ETH", &SizeInput::Units(0.5));
        assert_eq!(display, "0.5 ETH");
    }

    #[test]
    fn test_size_mode_default_is_usdc() {
        let config = AppConfig::default();
        assert_eq!(config.trading.default_size_mode, SizeMode::Usdc);
    }

    #[test]
    fn test_size_mode_display() {
        assert_eq!(format!("{}", SizeMode::Usdc), "usdc");
        assert_eq!(format!("{}", SizeMode::Units), "units");
        assert_eq!(format!("{}", SizeMode::Lots), "lots");
    }
}
