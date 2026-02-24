use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_types::output::*;
use atlas_utils::output::OutputFormat;
use atlas_utils::format::format_timestamp_ms;

/// Render a PriceOutput (table or JSON).
fn render_prices(output: &PriceOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("{:<12} {:>15}", "COIN", "MID PRICE");
            println!("{}", "─".repeat(28));
            for p in &output.prices {
                println!("{:<12} {:>15}", p.coin, p.mid_price);
            }
        }
    }
}

/// Render a MarketsOutput (table or JSON).
fn render_markets(output: &MarketsOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("Market type: {}\n", output.market_type.to_uppercase());
            println!("{:<15} {:>6} {:>10} {:>12}", "NAME", "INDEX", "MAX LEV", "SZ DECIMALS");
            println!("{}", "─".repeat(45));
            for m in &output.markets {
                println!("{:<15} {:>6} {:>10}x {:>12}", m.name, m.index, m.max_leverage, m.sz_decimals);
            }
            println!("\nTotal: {} markets", output.markets.len());
        }
    }
}

/// Render a CandlesOutput (table or JSON).
fn render_candles(output: &CandlesOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("{} — {} candles\n", output.coin, output.interval);
            println!("{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>6}",
                "TIME", "OPEN", "HIGH", "LOW", "CLOSE", "VOLUME", "TRADES");
            println!("{}", "─".repeat(90));
            for c in &output.candles {
                println!("{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>6}",
                    c.time, c.open, c.high, c.low, c.close, c.volume, c.trades);
            }
        }
    }
}

/// Render a FundingOutput (table or JSON).
fn render_funding(output: &FundingOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("{} — Funding Rate History\n", output.coin);
            println!("{:<20} {:>12} {:>15} {:>15}", "TIME", "COIN", "RATE", "PREMIUM");
            println!("{}", "─".repeat(65));
            for r in &output.rates {
                println!("{:<20} {:>12} {:>15} {:>15}", r.time, r.coin, r.rate, r.premium);
            }
        }
    }
}

/// `atlas price <COINS...>` or `atlas price --all`
pub async fn price(coins: &[String], all: bool, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let tickers = if all || coins.is_empty() {
        perp.all_tickers().await.map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        let mut result = Vec::new();
        for c in coins {
            let t = perp.ticker(&c.to_uppercase()).await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            result.push(t);
        }
        result
    };

    let prices: Vec<PriceRow> = tickers.iter().map(|t| PriceRow {
        coin: t.symbol.clone(),
        mid_price: t.mid_price.to_string(),
    }).collect();

    render_prices(&PriceOutput { prices }, fmt);
    Ok(())
}

/// `atlas markets` or `atlas markets --spot`
pub async fn markets(spot: bool, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let market_list = perp.markets().await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<MarketRow> = market_list.iter().map(|m| MarketRow {
        name: m.symbol.clone(),
        index: 0, // universal Market doesn't have index
        max_leverage: m.max_leverage.unwrap_or(1) as u64,
        sz_decimals: m.sz_decimals.unwrap_or(0) as i64,
    }).collect();

    let market_type = if spot { "spot" } else { "perp" };
    render_markets(&MarketsOutput { market_type: market_type.into(), markets: rows }, fmt);
    Ok(())
}

/// `atlas candles <COIN> <INTERVAL>` with optional --limit
pub async fn candles(coin: &str, interval: &str, limit: usize, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let candle_data = perp.candles(&coin_upper, interval, limit).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<CandleRow> = candle_data.iter().map(|c| CandleRow {
        time: format_timestamp_ms(c.open_time_ms),
        open: c.open.to_string(),
        high: c.high.to_string(),
        low: c.low.to_string(),
        close: c.close.to_string(),
        volume: c.volume.to_string(),
        trades: c.trades.unwrap_or(0),
    }).collect();

    render_candles(&CandlesOutput {
        coin: coin_upper,
        interval: interval.into(),
        candles: rows,
    }, fmt);
    Ok(())
}

/// `atlas funding <COIN>`
pub async fn funding(coin: &str, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let rates = perp.funding(&coin_upper).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<FundingRow> = rates.iter().map(|r| FundingRow {
        time: format_timestamp_ms(r.timestamp_ms),
        coin: r.symbol.clone(),
        rate: r.rate.to_string(),
        premium: r.premium.map(|p| p.to_string()).unwrap_or_else(|| "—".into()),
    }).collect();

    render_funding(&FundingOutput { coin: coin_upper, rates: rows }, fmt);
    Ok(())
}
