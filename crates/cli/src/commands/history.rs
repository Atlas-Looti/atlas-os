//! `atlas history` — query cached trade/order history and PnL.

use std::collections::HashMap;

use anyhow::Result;
use atlas_core::db::AtlasDb;
use atlas_core::Engine;
use atlas_types::db::{FillFilter, OrderFilter};
use atlas_types::output::{
    OrderHistoryOutput, OrderHistoryRow, PnlByCoinRow, PnlSummaryOutput,
    SyncOutput, TradeHistoryOutput, TradeHistoryRow,
};
use atlas_utils::output::{render, OutputFormat};
use rust_decimal::Decimal;

/// Parse an ISO date string to millisecond timestamp.
/// Accepts "2025-01-01" or "2025-01-01T00:00:00".
fn parse_date_to_ms(s: &str) -> Result<i64> {
    use chrono::NaiveDateTime;

    // Try datetime first, then date-only
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.and_utc().timestamp_millis());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = NaiveDateTime::new(d, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        return Ok(dt.and_utc().timestamp_millis());
    }
    anyhow::bail!("Invalid date format: {s}. Use YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS")
}

/// Format a millisecond timestamp to human-readable UTC string.
fn format_ms(ms: i64) -> String {
    let secs = ms / 1000;
    let dt = chrono::DateTime::from_timestamp(secs, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string());
    dt
}

/// `atlas history trades [--coin COIN] [--from DATE] [--to DATE] [--limit N]`
pub fn run_trades(
    coin: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    limit: usize,
    fmt: OutputFormat,
) -> Result<()> {
    let db = AtlasDb::open()?;

    let from_ms = from.map(parse_date_to_ms).transpose()?;
    let to_ms = to.map(parse_date_to_ms).transpose()?;

    let filter = FillFilter {
        coin: coin.map(|c| c.to_uppercase()),
        from_ms,
        to_ms,
        limit: Some(limit),
    };

    let fills = db.query_fills(&filter)?;

    let trades: Vec<TradeHistoryRow> = fills.iter().map(|f| {
        TradeHistoryRow {
            coin: f.coin.clone(),
            side: f.side.clone(),
            size: f.sz.clone(),
            price: f.px.clone(),
            pnl: f.closed_pnl.clone(),
            fee: f.fee.clone(),
            time: format_ms(f.time_ms),
        }
    }).collect();

    let total = trades.len();
    let output = TradeHistoryOutput { trades, total };
    render(fmt, &output)?;
    Ok(())
}

/// `atlas history orders [--coin COIN] [--status STATUS] [--limit N]`
pub fn run_orders(
    coin: Option<&str>,
    status: Option<&str>,
    limit: usize,
    fmt: OutputFormat,
) -> Result<()> {
    let db = AtlasDb::open()?;

    let filter = OrderFilter {
        coin: coin.map(|c| c.to_uppercase()),
        status: status.map(|s| s.to_lowercase()),
        limit: Some(limit),
    };

    let orders = db.query_orders(&filter)?;

    let rows: Vec<OrderHistoryRow> = orders.iter().map(|o| {
        OrderHistoryRow {
            coin: o.coin.clone(),
            side: o.side.clone(),
            size: o.sz.clone(),
            price: o.limit_px.clone(),
            oid: o.oid,
            status: o.status.clone(),
            order_type: o.order_type.clone(),
            time: format_ms(o.timestamp_ms),
        }
    }).collect();

    let total = rows.len();
    let output = OrderHistoryOutput { orders: rows, total };
    render(fmt, &output)?;
    Ok(())
}

/// `atlas history pnl [--coin COIN] [--from DATE] [--to DATE]`
pub fn run_pnl(
    coin: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let db = AtlasDb::open()?;

    let from_ms = from.map(parse_date_to_ms).transpose()?;
    let to_ms = to.map(parse_date_to_ms).transpose()?;

    let filter = FillFilter {
        coin: coin.map(|c| c.to_uppercase()),
        from_ms,
        to_ms,
        limit: None, // get all for PnL computation
    };

    let fills = db.query_fills(&filter)?;

    let mut total_pnl = Decimal::ZERO;
    let mut total_fees = Decimal::ZERO;
    let mut win_count = 0usize;
    let mut loss_count = 0usize;
    let mut by_coin: HashMap<String, (Decimal, Decimal, usize)> = HashMap::new();

    for fill in &fills {
        let pnl: Decimal = fill.closed_pnl.parse().unwrap_or(Decimal::ZERO);
        let fee: Decimal = fill.fee.parse().unwrap_or(Decimal::ZERO);

        total_pnl += pnl;
        total_fees += fee;

        if pnl > Decimal::ZERO {
            win_count += 1;
        } else if pnl < Decimal::ZERO {
            loss_count += 1;
        }

        let entry = by_coin.entry(fill.coin.clone()).or_insert((Decimal::ZERO, Decimal::ZERO, 0));
        entry.0 += pnl;
        entry.1 += fee;
        entry.2 += 1;
    }

    let net_pnl = total_pnl - total_fees;
    let trade_count = fills.len();
    let closing_count = win_count + loss_count;
    let win_rate = if closing_count > 0 {
        format!("{:.1}%", (win_count as f64 / closing_count as f64) * 100.0)
    } else {
        "N/A".to_string()
    };

    let mut coin_rows: Vec<PnlByCoinRow> = by_coin.into_iter().map(|(c, (pnl, fees, trades))| {
        PnlByCoinRow {
            coin: c,
            pnl: pnl.to_string(),
            fees: fees.to_string(),
            trades,
        }
    }).collect();
    coin_rows.sort_by(|a, b| a.coin.cmp(&b.coin));

    let output = PnlSummaryOutput {
        total_pnl: total_pnl.to_string(),
        total_fees: total_fees.to_string(),
        net_pnl: net_pnl.to_string(),
        trade_count,
        win_count,
        loss_count,
        win_rate,
        by_coin: coin_rows,
    };

    render(fmt, &output)?;
    Ok(())
}

/// `atlas history sync [--full]`
pub async fn run_sync(_full: bool, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let db = AtlasDb::open()?;

    let (fills, orders) = engine.sync_all(&db).await?;

    let output = SyncOutput {
        fills_synced: fills,
        orders_synced: orders,
        status: "complete".to_string(),
    };

    render(fmt, &output)?;
    Ok(())
}
