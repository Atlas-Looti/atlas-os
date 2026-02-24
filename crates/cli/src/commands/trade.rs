use anyhow::Result;
use atlas_core::Engine;
use atlas_types::config::{SizeInput, SizeMode};
use atlas_utils::output::{render, OutputFormat};
use atlas_utils::parse;

/// `atlas order <coin> <side> <size> <price> [--reduce-only] [--tif Gtc|Ioc|Alo]`
///
/// Size can be:
///   - `200`      → default mode (USDC by default)
///   - `$200`     → $200 USDC margin
///   - `0.5eth`   → 0.5 asset units
///   - `50lots`   → 50 lots
pub async fn limit_order(
    coin: &str,
    side: &str,
    size_str: &str,
    price: f64,
    reduce_only: bool,
    tif: &str,
    fmt: OutputFormat,
) -> Result<()> {
    let is_buy = parse::parse_side(side)?;
    let size_input = parse::parse_size(size_str)?;
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();
    let lev = engine.config.trading.default_leverage.max(1);

    // For limit orders, we can use the limit price to compute USDC→size
    let size = match &size_input {
        SizeInput::Usdc(margin_usdc) => {
            let notional = margin_usdc * lev as f64;
            let sz = notional / price;
            if fmt == OutputFormat::Table {
                println!(
                    "💰 ${:.2} × {}x = ${:.2} notional → {:.6} {} @ ${:.2}",
                    margin_usdc, lev, notional, sz, coin_upper, price
                );
            }
            sz
        }
        SizeInput::Raw(raw) if engine.config.trading.default_size_mode == SizeMode::Usdc => {
            let notional = raw * lev as f64;
            let sz = notional / price;
            if fmt == OutputFormat::Table {
                println!(
                    "💰 ${:.2} × {}x = ${:.2} notional → {:.6} {} @ ${:.2}",
                    raw, lev, notional, sz, coin_upper, price
                );
            }
            sz
        }
        SizeInput::Units(u) => *u,
        SizeInput::Lots(l) => engine.config.trading.lots.lots_to_size(&coin_upper, *l),
        SizeInput::Raw(raw) => engine.config.resolve_size(&coin_upper, *raw),
    };

    if fmt == OutputFormat::Table {
        let size_display = engine.config.format_size(&coin_upper, size);
        println!(
            "📤 {} {} @ {price} (tif={tif}, reduce_only={reduce_only})",
            if is_buy { "BUY" } else { "SELL" },
            size_display,
        );
    }

    let result = engine
        .limit_order_raw(&coin_upper, is_buy, size, price, reduce_only, tif)
        .await?;

    render(fmt, &result.output)?;
    Ok(())
}

/// `atlas buy <coin> <size> [--leverage 10] [--slippage 0.05]`
pub async fn market_buy(
    coin: &str,
    size_str: &str,
    leverage: Option<u32>,
    slippage: Option<f64>,
    fmt: OutputFormat,
) -> Result<()> {
    let size_input = parse::parse_size(size_str)?;
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    let (size, display) = engine.resolve_size_input(&coin_upper, &size_input, leverage).await?;

    if fmt == OutputFormat::Table {
        println!("📤 MARKET BUY {display}");
    }

    let result = engine.market_open_raw(&coin_upper, true, size, slippage).await?;
    render(fmt, &result.output)?;
    Ok(())
}

/// `atlas sell <coin> <size> [--leverage 10] [--slippage 0.05]`
pub async fn market_sell(
    coin: &str,
    size_str: &str,
    leverage: Option<u32>,
    slippage: Option<f64>,
    fmt: OutputFormat,
) -> Result<()> {
    let size_input = parse::parse_size(size_str)?;
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    let (size, display) = engine.resolve_size_input(&coin_upper, &size_input, leverage).await?;

    if fmt == OutputFormat::Table {
        println!("📤 MARKET SELL {display}");
    }

    let result = engine.market_open_raw(&coin_upper, false, size, slippage).await?;
    render(fmt, &result.output)?;
    Ok(())
}

/// `atlas close <coin> [--size 0.5] [--slippage 0.05]`
pub async fn close_position(
    coin: &str,
    size: Option<f64>,
    slippage: Option<f64>,
    fmt: OutputFormat,
) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    if fmt == OutputFormat::Table {
        let label = match size {
            Some(s) => engine.config.format_size(&coin_upper,
                engine.config.resolve_size(&coin_upper, s)),
            None => "full position".to_string(),
        };
        println!("📤 CLOSE {coin_upper} ({label})");
    }

    // For close, size is in raw asset units (not lots) — it's position-based.
    let result = engine.market_close(&coin_upper, size, slippage).await?;
    render(fmt, &result.output)?;
    Ok(())
}

/// `atlas cancel <coin> [--oid 12345]`
pub async fn cancel(coin: &str, oid: Option<u64>, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let coin_upper = coin.to_uppercase();

    match oid {
        Some(id) => {
            let output = engine.cancel_order(&coin_upper, id).await?;
            render(fmt, &output)?;
        }
        None => {
            let output = engine.cancel_all_orders(&coin_upper).await?;
            render(fmt, &output)?;
        }
    }

    Ok(())
}

/// `atlas orders`
pub async fn list_orders(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let output = engine.get_open_orders().await?;
    render(fmt, &output)?;
    Ok(())
}

/// `atlas fills`
pub async fn list_fills(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let output = engine.get_fills().await?;
    render(fmt, &output)?;
    Ok(())
}
