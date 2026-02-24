use atlas_types::config::{AppConfig, TradingMode};

/// Input for calculating a risk-managed position.
#[derive(Debug, Clone)]
pub struct RiskInput {
    /// Asset symbol (e.g. "ETH").
    pub coin: String,
    /// Current mark/mid price of the asset.
    pub mark_price: f64,
    /// Total account value in USDC.
    pub account_value: f64,
    /// Entry price (limit price or current market price).
    pub entry_price: f64,
    /// Stop-loss price. If None, uses default_stop_pct from config.
    pub stop_loss: Option<f64>,
    /// Is this a buy (long) trade?
    pub is_buy: bool,
    /// Leverage to use. If None, uses default_leverage from config.
    pub leverage: Option<u32>,
}

/// Output of the risk calculator.
#[derive(Debug, Clone)]
pub struct RiskOutput {
    /// Calculated position size in asset units.
    pub size: f64,
    /// Position size in lots (CFD mode), or same as size (futures mode).
    pub lots: f64,
    /// Dollar amount at risk.
    pub risk_usd: f64,
    /// Risk as percentage of account.
    pub risk_pct: f64,
    /// Calculated stop-loss price.
    pub stop_loss: f64,
    /// Take-profit price (2:1 R:R by default).
    pub take_profit: f64,
    /// Position notional value.
    pub notional: f64,
    /// Required margin.
    pub margin: f64,
    /// Effective leverage used.
    pub leverage: u32,
    /// Liquidation estimate (simplified).
    pub est_liquidation: f64,
}

/// Warnings from risk validation.
#[derive(Debug, Clone)]
pub struct RiskWarnings {
    pub warnings: Vec<String>,
    pub blocked: bool,
}

/// Calculate position size based on risk parameters.
///
/// The core formula:
/// ```text
///   risk_usd = account_value * max_risk_pct
///   distance = |entry_price - stop_loss| / entry_price
///   raw_size = risk_usd / (entry_price * distance)
///   size     = min(raw_size, max_size_cap)
/// ```
///
/// This works identically for both Futures and CFD modes — the only
/// difference is how `size` is displayed (units vs lots).
pub fn calculate_position(config: &AppConfig, input: &RiskInput) -> RiskOutput {
    let risk_config = &config.risk;
    let leverage = input.leverage.unwrap_or(config.trading.default_leverage);

    // Dollar risk
    let risk_pct = risk_config.effective_risk_pct(&input.coin);
    let risk_usd = input.account_value * risk_pct;

    // Stop-loss distance
    let stop_pct = risk_config.effective_stop_pct(&input.coin);
    let stop_loss = match input.stop_loss {
        Some(sl) => sl,
        None => {
            if input.is_buy {
                input.entry_price * (1.0 - stop_pct)
            } else {
                input.entry_price * (1.0 + stop_pct)
            }
        }
    };

    // Price distance from entry to stop
    let distance = (input.entry_price - stop_loss).abs();

    // Position size: how much can we buy so that if price moves `distance`,
    // we lose exactly `risk_usd`?
    let mut size = if distance > 0.0 {
        risk_usd / distance
    } else {
        0.0
    };

    // Apply max size cap if configured
    if let Some(max) = risk_config.max_size(&input.coin) {
        size = size.min(max);
    }

    // Notional and margin
    let notional = size * input.entry_price;
    let margin = if leverage > 0 {
        notional / leverage as f64
    } else {
        notional
    };

    // Take-profit at 2:1 R:R
    let take_profit = if input.is_buy {
        input.entry_price + (distance * 2.0)
    } else {
        input.entry_price - (distance * 2.0)
    };

    // Simplified liquidation estimate
    // For isolated margin: liq ≈ entry ± (margin / size)
    let est_liquidation = if input.is_buy {
        input.entry_price - (margin / size).max(0.0)
    } else {
        input.entry_price + (margin / size).max(0.0)
    };

    // Convert to lots for CFD mode
    let lots = match config.trading.mode {
        TradingMode::Cfd => config.trading.lots.size_to_lots(&input.coin, size),
        TradingMode::Futures => size,
    };

    let actual_risk_pct = if input.account_value > 0.0 {
        (size * distance) / input.account_value
    } else {
        0.0
    };

    RiskOutput {
        size,
        lots,
        risk_usd: size * distance,
        risk_pct: actual_risk_pct,
        stop_loss,
        take_profit,
        notional,
        margin,
        leverage,
        est_liquidation,
    }
}

/// Validate a trade against risk rules. Returns warnings and whether
/// the trade should be blocked.
pub fn validate_risk(
    config: &AppConfig,
    input: &RiskInput,
    output: &RiskOutput,
    current_positions: usize,
    total_exposure: f64,
) -> RiskWarnings {
    let risk_config = &config.risk;
    let mut warnings = Vec::new();
    let mut blocked = false;

    // Check max positions
    if current_positions >= risk_config.max_positions as usize {
        warnings.push(format!(
            "⛔ Max positions reached ({}/{})",
            current_positions, risk_config.max_positions
        ));
        blocked = true;
    }

    // Check exposure limit
    let new_exposure = total_exposure + output.notional;
    let max_exposure = input.account_value * risk_config.max_exposure_multiplier;
    if new_exposure > max_exposure {
        warnings.push(format!(
            "⚠ Exposure would be ${:.2} (max: ${:.2} = {:.0}x account)",
            new_exposure, max_exposure, risk_config.max_exposure_multiplier
        ));
    }

    // Check risk percentage
    let max_risk = risk_config.effective_risk_pct(&input.coin);
    if output.risk_pct > max_risk * 1.1 {
        // 10% tolerance
        warnings.push(format!(
            "⚠ Risk {:.2}% exceeds max {:.2}%",
            output.risk_pct * 100.0,
            max_risk * 100.0
        ));
    }

    // Check margin vs account
    if output.margin > input.account_value * 0.5 {
        warnings.push(format!(
            "⚠ Margin ${:.2} is >{:.0}% of account",
            output.margin, 50.0
        ));
    }

    // Check if stop-loss is too tight (< 0.5%)
    let stop_distance_pct = (input.entry_price - output.stop_loss).abs() / input.entry_price;
    if stop_distance_pct < 0.005 {
        warnings.push(format!(
            "⚠ Stop-loss very tight ({:.2}% from entry) — high risk of stopout",
            stop_distance_pct * 100.0
        ));
    }

    // Check if stop-loss is too wide (> 10%)
    if stop_distance_pct > 0.10 {
        warnings.push(format!(
            "⚠ Stop-loss wide ({:.2}% from entry) — large risk per unit",
            stop_distance_pct * 100.0
        ));
    }

    RiskWarnings { warnings, blocked }
}

/// Format risk output for display in the terminal.
pub fn format_risk_summary(config: &AppConfig, input: &RiskInput, output: &RiskOutput) -> String {
    let mode_label = match config.trading.mode {
        TradingMode::Futures => "FUTURES",
        TradingMode::Cfd => "CFD",
    };

    let side = if input.is_buy { "LONG" } else { "SHORT" };

    let size_display = match config.trading.mode {
        TradingMode::Futures => format!("{:.6} {}", output.size, input.coin),
        TradingMode::Cfd => format!("{:.4} lots ({:.6} {})", output.lots, output.size, input.coin),
    };

    format!(
        r#"╔══════════════════════════════════════════════════════════╗
║  RISK CALCULATOR — {:<7} mode                      ║
╠══════════════════════════════════════════════════════════╣
║  Asset        : {:<6} {}                              ║
║  Entry Price  : ${:<43.4}║
║  Size         : {:<40}║
║  Notional     : ${:<43.2}║
╠══════════════════════════════════════════════════════════╣
║  Stop-Loss    : ${:<43.4}║
║  Take-Profit  : ${:<43.4}║
║  Est. Liq     : ${:<43.4}║
╠══════════════════════════════════════════════════════════╣
║  Risk (USDC)  : ${:<43.2}║
║  Risk (%)     : {:<43}║
║  Margin Req.  : ${:<43.2}║
║  Leverage     : {:<43}║
╚══════════════════════════════════════════════════════════╝"#,
        mode_label,
        input.coin,
        side,
        input.entry_price,
        size_display,
        output.notional,
        output.stop_loss,
        output.take_profit,
        output.est_liquidation,
        output.risk_usd,
        format!("{:.2}%", output.risk_pct * 100.0),
        output.margin,
        format!("{}x", output.leverage),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_types::config::AppConfig;

    fn default_input() -> RiskInput {
        RiskInput {
            coin: "ETH".to_string(),
            mark_price: 3500.0,
            account_value: 10000.0,
            entry_price: 3500.0,
            stop_loss: None,
            is_buy: true,
            leverage: None,
        }
    }

    #[test]
    fn test_basic_position_sizing() {
        let config = AppConfig::default();
        let input = default_input();
        let output = calculate_position(&config, &input);

        // 2% of $10,000 = $200 risk
        // Default stop = 2% below entry = $3430
        // Distance = $70
        // Size = $200 / $70 ≈ 2.857 ETH
        assert!((output.risk_usd - 200.0).abs() < 1.0);
        assert!(output.size > 2.0 && output.size < 3.5);
        assert!(output.stop_loss < input.entry_price);
    }

    #[test]
    fn test_custom_stop_loss() {
        let config = AppConfig::default();
        let mut input = default_input();
        input.stop_loss = Some(3400.0); // $100 distance

        let output = calculate_position(&config, &input);

        // Distance = $100
        // Size = $200 / $100 = 2.0 ETH
        assert!((output.size - 2.0).abs() < 0.01);
        assert_eq!(output.stop_loss, 3400.0);
    }

    #[test]
    fn test_short_position() {
        let config = AppConfig::default();
        let mut input = default_input();
        input.is_buy = false;

        let output = calculate_position(&config, &input);

        // Stop-loss should be above entry for short
        assert!(output.stop_loss > input.entry_price);
        // Take-profit should be below entry for short
        assert!(output.take_profit < input.entry_price);
    }

    #[test]
    fn test_leverage_affects_margin() {
        let config = AppConfig::default();
        let mut input = default_input();
        input.leverage = Some(10);

        let output = calculate_position(&config, &input);

        // With 10x leverage, margin = notional / 10
        let expected_margin = output.notional / 10.0;
        assert!((output.margin - expected_margin).abs() < 0.01);
        assert_eq!(output.leverage, 10);
    }

    #[test]
    fn test_cfd_mode_converts_to_lots() {
        let mut config = AppConfig::default();
        config.trading.mode = TradingMode::Cfd;

        let input = default_input();
        let output = calculate_position(&config, &input);

        // ETH lot size = 0.01
        // lots = size / 0.01
        let expected_lots = output.size / 0.01;
        assert!((output.lots - expected_lots).abs() < 0.01);
    }

    #[test]
    fn test_futures_mode_lots_equals_size() {
        let config = AppConfig::default();
        let input = default_input();
        let output = calculate_position(&config, &input);

        // In futures mode, lots == size
        assert_eq!(output.lots, output.size);
    }

    #[test]
    fn test_take_profit_2_to_1_rr() {
        let config = AppConfig::default();
        let mut input = default_input();
        input.stop_loss = Some(3400.0); // $100 below entry

        let output = calculate_position(&config, &input);

        // TP should be $200 above entry (2:1 R:R)
        assert!((output.take_profit - 3700.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_max_positions() {
        let config = AppConfig::default();
        let input = default_input();
        let output = calculate_position(&config, &input);

        let warnings = validate_risk(&config, &input, &output, 10, 0.0);
        assert!(warnings.blocked);
    }

    #[test]
    fn test_validate_within_limits() {
        let config = AppConfig::default();
        let input = default_input();
        let output = calculate_position(&config, &input);

        let warnings = validate_risk(&config, &input, &output, 0, 0.0);
        assert!(!warnings.blocked);
    }

    #[test]
    fn test_validate_exposure_warning() {
        let config = AppConfig::default();
        let mut input = default_input();
        input.account_value = 1000.0;
        let output = calculate_position(&config, &input);

        // Already at high exposure
        let warnings = validate_risk(&config, &input, &output, 0, 2500.0);
        let has_exposure_warning = warnings.warnings.iter().any(|w| w.contains("Exposure"));
        assert!(has_exposure_warning);
    }

    #[test]
    fn test_asset_override() {
        let mut config = AppConfig::default();
        config.risk.asset_overrides.insert(
            "BTC".to_string(),
            atlas_types::risk::AssetRiskOverride {
                max_risk_pct: Some(0.01), // 1% for BTC
                default_stop_pct: Some(0.03), // 3% stop
                max_size: Some(0.1), // max 0.1 BTC
            },
        );

        assert_eq!(config.risk.effective_risk_pct("BTC"), 0.01);
        assert_eq!(config.risk.effective_risk_pct("ETH"), 0.02); // default
        assert_eq!(config.risk.effective_stop_pct("BTC"), 0.03);
        assert_eq!(config.risk.max_size("BTC"), Some(0.1));
        assert_eq!(config.risk.max_size("ETH"), None);
    }

    #[test]
    fn test_max_size_cap() {
        let mut config = AppConfig::default();
        config.risk.asset_overrides.insert(
            "ETH".to_string(),
            atlas_types::risk::AssetRiskOverride {
                max_risk_pct: None,
                default_stop_pct: None,
                max_size: Some(0.5), // cap at 0.5 ETH
            },
        );

        let input = default_input();
        let output = calculate_position(&config, &input);

        // Should be capped at 0.5 ETH
        assert!(output.size <= 0.5);
    }

    #[test]
    fn test_zero_distance_zero_size() {
        let config = AppConfig::default();
        let mut input = default_input();
        input.stop_loss = Some(input.entry_price); // 0 distance

        let output = calculate_position(&config, &input);
        assert_eq!(output.size, 0.0);
    }
}
