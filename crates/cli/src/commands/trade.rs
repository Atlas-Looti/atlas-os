use anyhow::Result;
use atlas_core::config::{SizeInput, SizeMode};
use atlas_core::fmt::order_result_to_output;
use atlas_core::output::{render, OutputFormat};
use atlas_core::output::{
    CancelOutput, CancelSingleOutput, FillRow, FillsOutput, OrderRow, OrdersOutput, PositionRow,
};
use atlas_core::parse;
use atlas_core::workspace::load_config;
use rust_decimal::prelude::*;

/// `atlas order <coin> <side> <size> <price> [--reduce-only] [--tif Gtc|Ioc|Alo]`
pub async fn limit_order(
    coin: &str,
    side: &str,
    size_str: &str,
    price: f64,
    reduce_only: bool,
    _tif: &str,
    fmt: OutputFormat,
) -> Result<()> {
    let is_buy = parse::parse_side(side)?;
    let size_input = parse::parse_size(size_str)?;
    let config = load_config()?;
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();
    let hl_cfg = &config.modules.hyperliquid.config;
    let lev = hl_cfg.default_leverage.max(1);

    let price_dec =
        Decimal::from_f64(price).ok_or_else(|| anyhow::anyhow!("Invalid price: {price}"))?;

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
        SizeInput::Raw(raw) if hl_cfg.default_size_mode == SizeMode::Usdc => {
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
        SizeInput::Lots(l) => hl_cfg.lots.lots_to_size(&coin_upper, *l),
        SizeInput::Raw(raw) => {
            let (size, _) =
                hl_cfg.resolve_size_input(&coin_upper, &SizeInput::Raw(*raw), price, Some(lev));
            size
        }
    };

    let size_dec =
        Decimal::from_f64(size).ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?;
    let uni_side = if is_buy {
        atlas_core::types::Side::Buy
    } else {
        atlas_core::types::Side::Sell
    };

    let result = perp
        .limit_order(&coin_upper, uni_side, size_dec, price_dec, reduce_only)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    render(fmt, &order_result_to_output(&result))?;
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
    let config = load_config()?;
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();
    let hl_cfg = &config.modules.hyperliquid.config;
    let lev = leverage.unwrap_or(hl_cfg.default_leverage).max(1);

    let ticker = perp
        .ticker(&coin_upper)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mark = ticker.mid_price.to_f64().unwrap_or(0.0);
    let (size, _) = hl_cfg.resolve_size_input(&coin_upper, &size_input, mark, Some(lev));

    if fmt == OutputFormat::Table {
        println!("📤 MARKET BUY {}", hl_cfg.format_size(&coin_upper, size));
    }

    let size_dec =
        Decimal::from_f64(size).ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?;

    let effective_slippage = slippage.or(Some(hl_cfg.default_slippage));

    let result = perp
        .market_order(
            &coin_upper,
            atlas_core::types::Side::Buy,
            size_dec,
            effective_slippage,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    render(fmt, &order_result_to_output(&result))?;
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
    let config = load_config()?;
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();
    let hl_cfg = &config.modules.hyperliquid.config;
    let lev = leverage.unwrap_or(hl_cfg.default_leverage).max(1);

    let ticker = perp
        .ticker(&coin_upper)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mark = ticker.mid_price.to_f64().unwrap_or(0.0);
    let (size, _) = hl_cfg.resolve_size_input(&coin_upper, &size_input, mark, Some(lev));

    if fmt == OutputFormat::Table {
        println!("📤 MARKET SELL {}", hl_cfg.format_size(&coin_upper, size));
    }

    let size_dec =
        Decimal::from_f64(size).ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?;

    let effective_slippage = slippage.or(Some(hl_cfg.default_slippage));

    let result = perp
        .market_order(
            &coin_upper,
            atlas_core::types::Side::Sell,
            size_dec,
            effective_slippage,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    render(fmt, &order_result_to_output(&result))?;
    Ok(())
}

/// `atlas close <coin> [--size 0.5] [--slippage 0.05]`
pub async fn close_position(
    coin: &str,
    size: Option<f64>,
    slippage: Option<f64>,
    fmt: OutputFormat,
) -> Result<()> {
    let config = load_config()?;
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let size_dec = size.and_then(Decimal::from_f64);
    let effective_slippage = slippage.or(Some(config.modules.hyperliquid.config.default_slippage));

    let result = perp
        .close_position(&coin_upper, size_dec, effective_slippage)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    render(fmt, &order_result_to_output(&result))?;
    Ok(())
}

/// `atlas cancel <coin> [--oid 12345]`
pub async fn cancel(coin: &str, oid: Option<u64>, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    match oid {
        Some(id) => {
            perp.cancel_order(&coin_upper, &id.to_string())
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            render(
                fmt,
                &CancelSingleOutput {
                    coin: coin_upper,
                    oid: id,
                    status: "cancelled".into(),
                },
            )?;
        }
        None => {
            let count = perp
                .cancel_all(&coin_upper)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            render(
                fmt,
                &CancelOutput {
                    coin: coin_upper,
                    cancelled: count,
                    total: count,
                    oids: vec![],
                },
            )?;
        }
    }
    Ok(())
}

/// `atlas orders`
pub async fn list_orders(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let orders = perp
        .open_orders()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<OrderRow> = orders
        .iter()
        .map(|o| OrderRow {
            coin: o.symbol.clone(),
            side: format!("{:?}", o.side),
            size: o.size.to_string(),
            price: o.price.map(|p| p.to_string()).unwrap_or_else(|| "—".into()),
            oid: o.order_id.parse().unwrap_or(0),
        })
        .collect();

    render(fmt, &OrdersOutput { orders: rows })?;
    Ok(())
}

/// `atlas fills`
pub async fn list_fills(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let fills = perp.fills().await.map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<FillRow> = fills
        .iter()
        .map(|f| FillRow {
            coin: f.symbol.clone(),
            side: format!("{:?}", f.side),
            size: f.size.to_string(),
            price: f.price.to_string(),
            fee: f.fee.to_string(),
            closed_pnl: f
                .realized_pnl
                .map(|p| p.to_string())
                .unwrap_or_else(|| "—".into()),
        })
        .collect();

    render(fmt, &FillsOutput { fills: rows })?;
    Ok(())
}

/// `atlas hyperliquid perp positions` — dedicated positions view.
pub async fn list_positions(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let positions = perp.positions().await.map_err(|e| anyhow::anyhow!("{e}"))?;

    if positions.is_empty() {
        if fmt == OutputFormat::Table {
            println!("No open positions.");
        } else {
            println!("[]");
        }
        return Ok(());
    }

    let rows: Vec<PositionRow> = positions
        .iter()
        .map(|p| PositionRow {
            coin: p.symbol.clone(),
            side: if p.size > rust_decimal::Decimal::ZERO { "long".into() } else { "short".into() },
            size: p.size.to_string(),
            entry_price: p.entry_price.map(|e| e.to_string()),
            mark_price: p.mark_price.map(|m| m.to_string()),
            unrealized_pnl: p.unrealized_pnl.map(|u| u.to_string()),
            liquidation_price: p.liquidation_price.map(|l| l.to_string()),
            leverage: p.leverage,
            margin_mode: p.margin_mode.clone(),
            protocol: "hyperliquid".into(),
        })
        .collect();

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&rows)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&rows)?),
        OutputFormat::Table => {
            println!(
                "{:<12} {:>14} {:>14} {:>14}",
                "COIN", "SIZE", "ENTRY", "uPnL"
            );
            println!("{}", "─".repeat(56));
            for r in &rows {
                println!(
                    "{:<12} {:>14} {:>14} {:>14}",
                    r.coin,
                    r.size,
                    r.entry_price.as_deref().unwrap_or("—"),
                    r.unrealized_pnl.as_deref().unwrap_or("—")
                );
            }
        }
    }

    Ok(())
}
