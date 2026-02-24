use anyhow::Result;
use atlas_core::output::{LeverageOutput, MarginOutput, TransferOutput};
use atlas_core::output::{render, OutputFormat};
use rust_decimal::prelude::*;

/// `atlas leverage <coin> <value> [--cross]`
pub async fn set_leverage(coin: &str, value: u32, cross: bool, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    perp.set_leverage(&coin_upper, value, cross).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let output = LeverageOutput {
        coin: coin_upper,
        leverage: value,
        mode: if cross { "cross" } else { "isolated" }.to_string(),
    };
    render(fmt, &output)?;
    Ok(())
}

/// `atlas margin <coin> <amount>`
pub async fn update_margin(coin: &str, amount: f64, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let dec_amount = Decimal::from_f64(amount)
        .ok_or_else(|| anyhow::anyhow!("Invalid amount: {amount}"))?;

    perp.update_margin(&coin_upper, dec_amount).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let output = MarginOutput {
        coin: coin_upper,
        action: if amount > 0.0 { "Added" } else { "Removed" }.to_string(),
        amount: format!("{:.2}", amount.abs()),
    };
    render(fmt, &output)?;
    Ok(())
}

/// `atlas transfer <amount> <destination>`
pub async fn transfer_usdc(amount: &str, destination: &str, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let dec_amount: Decimal = amount.parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount: {amount}"))?;

    perp.transfer(dec_amount, destination).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let output = TransferOutput {
        amount: amount.to_string(),
        destination: destination.to_string(),
    };
    render(fmt, &output)?;
    Ok(())
}
