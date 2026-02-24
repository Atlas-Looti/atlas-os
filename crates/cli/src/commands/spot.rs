use anyhow::Result;
use atlas_core::output::{SpotBalanceOutput, SpotBalanceRow, SpotOrderOutput, SpotTransferOutput};
use atlas_core::output::{render, OutputFormat};
use rust_decimal::prelude::*;

/// `atlas spot buy <BASE> <SIZE> [--slippage N]`
pub async fn spot_buy(base: &str, size: f64, slippage: Option<f64>, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let base_upper = base.to_uppercase();

    let size_dec = Decimal::from_f64(size)
        .ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?;

    let result = perp.spot_market_order(&base_upper, atlas_core::types::Side::Buy, size_dec, slippage).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let output = SpotOrderOutput {
        market: format!("{}/USDC", base_upper),
        side: "BUY".into(),
        oid: result.order_id.parse().unwrap_or(0),
        status: format!("{:?}", result.status).to_lowercase(),
        total_sz: result.filled_size.map(|s| s.to_string()),
        avg_px: result.avg_price.map(|p| p.to_string()),
    };
    render(fmt, &output)?;
    Ok(())
}

/// `atlas spot sell <BASE> <SIZE> [--slippage N]`
pub async fn spot_sell(base: &str, size: f64, slippage: Option<f64>, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let base_upper = base.to_uppercase();

    let size_dec = Decimal::from_f64(size)
        .ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?;

    let result = perp.spot_market_order(&base_upper, atlas_core::types::Side::Sell, size_dec, slippage).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let output = SpotOrderOutput {
        market: format!("{}/USDC", base_upper),
        side: "SELL".into(),
        oid: result.order_id.parse().unwrap_or(0),
        status: format!("{:?}", result.status).to_lowercase(),
        total_sz: result.filled_size.map(|s| s.to_string()),
        avg_px: result.avg_price.map(|p| p.to_string()),
    };
    render(fmt, &output)?;
    Ok(())
}

/// `atlas spot balance`
pub async fn spot_balance(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let balances = perp.spot_balances().await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<SpotBalanceRow> = balances.iter().map(|b| SpotBalanceRow {
        coin: b.token.clone(),
        total: b.total.to_string(),
        held: b.held.to_string(),
        available: b.available.to_string(),
    }).collect();

    render(fmt, &SpotBalanceOutput { balances: rows })?;
    Ok(())
}

/// `atlas spot transfer <DIRECTION> <AMOUNT> [--token TOKEN]`
pub async fn spot_transfer(
    direction: &str,
    amount: &str,
    token: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let amount_dec: Decimal = amount.parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount: {amount}"))?;

    let dir = direction.to_lowercase();
    let tk = token.unwrap_or("USDC");

    let _result = perp.internal_transfer(&dir, amount_dec, Some(tk)).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let display_dir = match dir.as_str() {
        "to-spot" | "perps-to-spot" => "perps → spot",
        "to-perps" | "spot-to-perps" => "spot → perps",
        "to-evm" | "spot-to-evm" => "spot → EVM",
        _ => &dir,
    };

    let output = SpotTransferOutput {
        direction: display_dir.to_string(),
        token: tk.to_uppercase(),
        amount: amount.to_string(),
    };
    render(fmt, &output)?;
    Ok(())
}
