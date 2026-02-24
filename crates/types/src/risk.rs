use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Risk management configuration.
///
/// Atlas supports two approaches to risk:
///
/// **1. Fixed USDC risk** (recommended for both modes):
///    "I want to risk $50 on this trade" → Atlas calculates the correct
///    position size based on entry, stop-loss, and leverage.
///
/// **2. Percentage risk**:
///    "I want to risk 2% of my account" → Atlas reads account value,
///    computes dollar risk, then calculates position size.
///
/// Both work in Futures and CFD modes. In CFD mode, the result is
/// additionally converted to lots for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maximum risk per trade as percentage of account value (0.02 = 2%).
    pub max_risk_pct: f64,
    /// Maximum number of concurrent open positions.
    pub max_positions: u32,
    /// Maximum total exposure as multiple of account value.
    /// E.g. 3.0 = total position value can't exceed 3x account value.
    pub max_exposure_multiplier: f64,
    /// Default stop-loss distance in percentage from entry (0.02 = 2%).
    pub default_stop_pct: f64,
    /// Per-asset risk overrides.
    #[serde(default)]
    pub asset_overrides: HashMap<String, AssetRiskOverride>,
}

/// Per-asset risk override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRiskOverride {
    /// Override max risk percentage for this asset.
    pub max_risk_pct: Option<f64>,
    /// Override default stop-loss distance.
    pub default_stop_pct: Option<f64>,
    /// Maximum position size in asset units (hard cap).
    pub max_size: Option<f64>,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_risk_pct: 0.02,           // 2% of account per trade
            max_positions: 10,
            max_exposure_multiplier: 3.0,
            default_stop_pct: 0.02,       // 2% stop-loss distance
            asset_overrides: HashMap::new(),
        }
    }
}

impl RiskConfig {
    /// Get effective max risk pct for an asset.
    pub fn effective_risk_pct(&self, coin: &str) -> f64 {
        self.asset_overrides
            .get(coin)
            .and_then(|o| o.max_risk_pct)
            .unwrap_or(self.max_risk_pct)
    }

    /// Get effective stop-loss pct for an asset.
    pub fn effective_stop_pct(&self, coin: &str) -> f64 {
        self.asset_overrides
            .get(coin)
            .and_then(|o| o.default_stop_pct)
            .unwrap_or(self.default_stop_pct)
    }

    /// Get optional max size cap for an asset (in asset units).
    pub fn max_size(&self, coin: &str) -> Option<f64> {
        self.asset_overrides
            .get(coin)
            .and_then(|o| o.max_size)
    }
}
