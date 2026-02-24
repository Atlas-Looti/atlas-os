use anyhow::Result;
use atlas_core::Engine;
use atlas_types::output::RiskCalcOutput;
use atlas_utils::output::{render, OutputFormat};
use atlas_utils::parse;
use atlas_utils::risk::{self, RiskInput};

/// `atlas risk calc <coin> <side> <entry_price> [--stop <price>] [--leverage <n>]`
/// Calculate position size based on risk management rules.
pub async fn calculate(
    coin: &str,
    side: &str,
    entry_price: f64,
    stop_loss: Option<f64>,
    leverage: Option<u32>,
    fmt: OutputFormat,
) -> Result<()> {
    let is_buy = parse::parse_side(side)?;
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    // Fetch account value — hypersdk uses clearinghouse_state
    let state = engine.client.clearinghouse_state(engine.address, None).await
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let account_value: f64 = state.margin_summary.account_value
        .to_string()
        .parse()
        .unwrap_or(0.0);

    let input = RiskInput {
        coin: coin_upper.clone(),
        mark_price: entry_price,
        account_value,
        entry_price,
        stop_loss,
        is_buy,
        leverage,
    };

    let output = risk::calculate_position(&engine.config, &input);

    // Validate
    let current_positions = state.asset_positions.len();
    let total_exposure: f64 = state.asset_positions.iter()
        .map(|p| p.position.position_value.to_string().parse::<f64>().unwrap_or(0.0).abs())
        .sum();

    let warnings = risk::validate_risk(
        &engine.config,
        &input,
        &output,
        current_positions,
        total_exposure,
    );

    let risk_output = RiskCalcOutput {
        coin: coin_upper,
        side: if is_buy { "long".into() } else { "short".into() },
        entry_price,
        size: output.size,
        lots: output.lots,
        notional: output.notional,
        stop_loss: output.stop_loss,
        take_profit: output.take_profit,
        est_liquidation: output.est_liquidation,
        risk_usd: output.risk_usd,
        risk_pct: output.risk_pct,
        margin: output.margin,
        leverage: output.leverage,
        warnings: warnings.warnings.clone(),
        blocked: warnings.blocked,
    };

    render(fmt, &risk_output)?;
    Ok(())
}

/// `atlas risk offline <coin> <side> <entry> <account_value> [--stop <price>] [--leverage <n>]`
/// Calculate without connecting to Hyperliquid.
pub fn calculate_offline(
    coin: &str,
    side: &str,
    entry_price: f64,
    account_value: f64,
    stop_loss: Option<f64>,
    leverage: Option<u32>,
    fmt: OutputFormat,
) -> Result<()> {
    let is_buy = parse::parse_side(side)?;
    let config = atlas_core::workspace::load_config()?;
    let coin_upper = coin.to_uppercase();

    let input = RiskInput {
        coin: coin_upper.clone(),
        mark_price: entry_price,
        account_value,
        entry_price,
        stop_loss,
        is_buy,
        leverage,
    };

    let output = risk::calculate_position(&config, &input);

    let warnings = risk::validate_risk(&config, &input, &output, 0, 0.0);

    let risk_output = RiskCalcOutput {
        coin: coin_upper,
        side: if is_buy { "long".into() } else { "short".into() },
        entry_price,
        size: output.size,
        lots: output.lots,
        notional: output.notional,
        stop_loss: output.stop_loss,
        take_profit: output.take_profit,
        est_liquidation: output.est_liquidation,
        risk_usd: output.risk_usd,
        risk_pct: output.risk_pct,
        margin: output.margin,
        leverage: output.leverage,
        warnings: warnings.warnings.clone(),
        blocked: warnings.blocked,
    };

    render(fmt, &risk_output)?;
    Ok(())
}
