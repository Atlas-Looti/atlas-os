use anyhow::Result;
use atlas_core::Engine;
use atlas_utils::output::{render, OutputFormat};

/// `atlas leverage <coin> <value> [--cross]`
pub async fn set_leverage(coin: &str, value: u32, cross: bool, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    let output = engine.set_leverage(value, &coin_upper, cross).await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas margin <coin> <amount>`
pub async fn update_margin(coin: &str, amount: f64, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    let output = engine.update_margin(amount, &coin_upper).await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas transfer <amount> <destination>`
pub async fn transfer_usdc(amount: &str, destination: &str, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.transfer_usdc(amount, destination).await?;
    render(fmt, &output)?;
    Ok(())
}
