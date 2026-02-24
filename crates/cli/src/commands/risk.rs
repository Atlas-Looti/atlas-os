use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_types::output::RiskCalcOutput;
use atlas_utils::output::{render, OutputFormat};
use atlas_utils::parse;
use atlas_utils::risk::{self, RiskInput};
use rust_decimal::prelude::*;

/// `atlas risk calc <coin> <side> <entry_price> [--stop <price>] [--leverage <n>]`
pub async fn calculate(
    coin: &str,
    side: &str,
    entry_price: f64,
    stop_loss: Option<f64>,
    leverage: Option<u32>,
    fmt: OutputFormat,
) -> Result<()> {
    let is_buy = parse::parse_side(side)?;
    let config = atlas_core::workspace::load_config()?;
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    // Get account value and positions from module
    let balances = perp.balances().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let positions = perp.positions().await.map_err(|e| anyhow::anyhow!("{e}"))?;

    let account_value = balances.first()
        .map(|b| b.total.to_f64().unwrap_or(0.0))
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

    let output = risk::calculate_position(&config, &config.modules.hyperliquid.config.risk, &input);

    let current_positions = positions.len();
    let total_exposure: f64 = positions.iter()
        .map(|p| (p.size * p.entry_price.unwrap_or(Decimal::ZERO)).to_f64().unwrap_or(0.0).abs())
        .sum();

    let warnings = risk::validate_risk(&config.modules.hyperliquid.config.risk, &input, &output, current_positions, total_exposure);

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

    let output = risk::calculate_position(&config, &config.modules.hyperliquid.config.risk, &input);
    let warnings = risk::validate_risk(&config.modules.hyperliquid.config.risk, &input, &output, 0, 0.0);

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
