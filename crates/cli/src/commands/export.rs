//! `atlas export` — export cached data to CSV or JSON files.

use std::collections::HashMap;
use std::io::Write;

use anyhow::{Context, Result};
use atlas_core::db::AtlasDb;
use atlas_types::db::FillFilter;
use atlas_types::output::ExportOutput;
use atlas_utils::output::{render, OutputFormat};
use rust_decimal::Decimal;

/// Parse an ISO date string to millisecond timestamp.
fn parse_date_to_ms(s: &str) -> Result<i64> {
    use chrono::NaiveDateTime;

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
    chrono::DateTime::from_timestamp(ms / 1000, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

/// Generate an export file path.
fn export_path(kind: &str, ext: &str) -> Result<std::path::PathBuf> {
    let data_dir = atlas_core::workspace::root_dir()?.join("data");
    std::fs::create_dir_all(&data_dir)?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Ok(data_dir.join(format!("export-{kind}-{ts}.{ext}")))
}

/// `atlas export trades [--csv|--json] [--coin COIN] [--from DATE] [--to DATE]`
pub fn run_export_trades(
    use_json: bool,
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
        limit: None,
    };

    let fills = db.query_fills(&filter)?;

    if use_json {
        // Export as JSON
        let path = export_path("trades", "json")?;

        #[derive(serde::Serialize)]
        struct TradeRow {
            coin: String,
            side: String,
            size: String,
            price: String,
            pnl: String,
            fee: String,
            time: String,
            hash: String,
        }

        let rows: Vec<TradeRow> = fills.iter().map(|f| TradeRow {
            coin: f.coin.clone(),
            side: f.side.clone(),
            size: f.sz.clone(),
            price: f.px.clone(),
            pnl: f.closed_pnl.clone(),
            fee: f.fee.clone(),
            time: format_ms(f.time_ms),
            hash: f.hash.clone(),
        }).collect();

        let json = serde_json::to_string_pretty(&rows)?;
        std::fs::write(&path, &json)
            .with_context(|| format!("Failed to write {}", path.display()))?;

        let output = ExportOutput {
            path: path.display().to_string(),
            rows: rows.len(),
            format: "json".to_string(),
        };
        render(fmt, &output)?;
    } else {
        // Export as CSV
        let path = export_path("trades", "csv")?;
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to create {}", path.display()))?;

        writeln!(file, "coin,side,size,price,pnl,fee,time,hash")?;
        for f in &fills {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                f.coin, f.side, f.sz, f.px, f.closed_pnl, f.fee,
                format_ms(f.time_ms), f.hash,
            )?;
        }

        let output = ExportOutput {
            path: path.display().to_string(),
            rows: fills.len(),
            format: "csv".to_string(),
        };
        render(fmt, &output)?;
    }

    Ok(())
}

/// `atlas export pnl [--csv|--json] [--from DATE] [--to DATE]`
pub fn run_export_pnl(
    use_json: bool,
    from: Option<&str>,
    to: Option<&str>,
    fmt: OutputFormat,
) -> Result<()> {
    let db = AtlasDb::open()?;

    let from_ms = from.map(parse_date_to_ms).transpose()?;
    let to_ms = to.map(parse_date_to_ms).transpose()?;

    let filter = FillFilter {
        coin: None,
        from_ms,
        to_ms,
        limit: None,
    };

    let fills = db.query_fills(&filter)?;

    // Aggregate by coin
    let mut by_coin: HashMap<String, (Decimal, Decimal, usize)> = HashMap::new();
    for fill in &fills {
        let pnl: Decimal = fill.closed_pnl.parse().unwrap_or(Decimal::ZERO);
        let fee: Decimal = fill.fee.parse().unwrap_or(Decimal::ZERO);
        let entry = by_coin.entry(fill.coin.clone()).or_insert((Decimal::ZERO, Decimal::ZERO, 0));
        entry.0 += pnl;
        entry.1 += fee;
        entry.2 += 1;
    }

    let mut rows: Vec<(String, Decimal, Decimal, usize)> = by_coin.into_iter()
        .map(|(c, (pnl, fees, trades))| (c, pnl, fees, trades))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    if use_json {
        let path = export_path("pnl", "json")?;

        #[derive(serde::Serialize)]
        struct PnlRow {
            coin: String,
            pnl: String,
            fees: String,
            net_pnl: String,
            trades: usize,
        }

        let export_rows: Vec<PnlRow> = rows.iter().map(|(c, pnl, fees, trades)| PnlRow {
            coin: c.clone(),
            pnl: pnl.to_string(),
            fees: fees.to_string(),
            net_pnl: (*pnl - *fees).to_string(),
            trades: *trades,
        }).collect();

        let json = serde_json::to_string_pretty(&export_rows)?;
        std::fs::write(&path, &json)
            .with_context(|| format!("Failed to write {}", path.display()))?;

        let output = ExportOutput {
            path: path.display().to_string(),
            rows: export_rows.len(),
            format: "json".to_string(),
        };
        render(fmt, &output)?;
    } else {
        let path = export_path("pnl", "csv")?;
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to create {}", path.display()))?;

        writeln!(file, "coin,pnl,fees,net_pnl,trades")?;
        for (c, pnl, fees, trades) in &rows {
            writeln!(file, "{},{},{},{},{}", c, pnl, fees, *pnl - *fees, trades)?;
        }

        let output = ExportOutput {
            path: path.display().to_string(),
            rows: rows.len(),
            format: "csv".to_string(),
        };
        render(fmt, &output)?;
    }

    Ok(())
}
