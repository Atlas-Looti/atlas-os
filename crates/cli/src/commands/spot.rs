use anyhow::Result;
use atlas_core::Engine;
use atlas_utils::output::{render, OutputFormat};

/// `atlas spot buy <BASE> <SIZE> [--slippage N]`
pub async fn spot_buy(base: &str, size: f64, slippage: Option<f64>, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let base_upper = base.to_uppercase();

    let output = engine.spot_market_order(&base_upper, true, size, slippage).await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas spot sell <BASE> <SIZE> [--slippage N]`
pub async fn spot_sell(base: &str, size: f64, slippage: Option<f64>, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let base_upper = base.to_uppercase();

    let output = engine.spot_market_order(&base_upper, false, size, slippage).await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas spot balance`
pub async fn spot_balance(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.get_spot_balances().await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas spot transfer <DIRECTION> <AMOUNT> [--token TOKEN]`
pub async fn spot_transfer(
    direction: &str,
    amount: &str,
    token: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = match direction.to_lowercase().as_str() {
        "to-spot" | "perps-to-spot" => {
            engine.transfer_perps_to_spot(amount).await?
        }
        "to-perps" | "spot-to-perps" => {
            engine.transfer_spot_to_perps(amount).await?
        }
        "to-evm" | "spot-to-evm" => {
            let tk = token.unwrap_or("USDC");
            engine.transfer_spot_to_evm(tk, amount).await?
        }
        _ => anyhow::bail!(
            "Invalid transfer direction: {direction}. Use: to-spot, to-perps, or to-evm"
        ),
    };

    render(fmt, &output)?;
    Ok(())
}
