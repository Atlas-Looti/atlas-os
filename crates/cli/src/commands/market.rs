use anyhow::Result;
use atlas_utils::output::OutputFormat;
use atlas_core::Engine;

/// Render a PriceOutput (table or JSON).
fn render_prices(output: &atlas_types::output::PriceOutput, fmt: OutputFormat) {
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
fn render_markets(output: &atlas_types::output::MarketsOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("Market type: {}\n", output.market_type.to_uppercase());
            println!("{:<15} {:>6} {:>10} {:>12}", "NAME", "INDEX", "MAX LEV", "SZ DECIMALS");
            println!("{}", "─".repeat(45));
            for m in &output.markets {
                println!(
                    "{:<15} {:>6} {:>10}x {:>12}",
                    m.name, m.index, m.max_leverage, m.sz_decimals
                );
            }
            println!("\nTotal: {} markets", output.markets.len());
        }
    }
}

/// Render a CandlesOutput (table or JSON).
fn render_candles(output: &atlas_types::output::CandlesOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("{} — {} candles\n", output.coin, output.interval);
            println!(
                "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>6}",
                "TIME", "OPEN", "HIGH", "LOW", "CLOSE", "VOLUME", "TRADES"
            );
            println!("{}", "─".repeat(90));
            for c in &output.candles {
                println!(
                    "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>6}",
                    c.time, c.open, c.high, c.low, c.close, c.volume, c.trades
                );
            }
        }
    }
}

/// Render a FundingOutput (table or JSON).
fn render_funding(output: &atlas_types::output::FundingOutput, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(output).unwrap()),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(output).unwrap()),
        OutputFormat::Table => {
            println!("{} — Funding Rate History\n", output.coin);
            println!(
                "{:<20} {:>12} {:>15} {:>15}",
                "TIME", "COIN", "RATE", "PREMIUM"
            );
            println!("{}", "─".repeat(65));
            for r in &output.rates {
                println!(
                    "{:<20} {:>12} {:>15} {:>15}",
                    r.time, r.coin, r.rate, r.premium
                );
            }
        }
    }
}

/// `atlas price <COINS...>` or `atlas price --all`
pub async fn price(coins: &[String], all: bool, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = if all || coins.is_empty() {
        engine.get_all_prices().await?
    } else {
        engine.get_prices(coins).await?
    };

    render_prices(&output, fmt);
    Ok(())
}

/// `atlas markets` or `atlas markets --spot`
pub async fn markets(spot: bool, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = if spot {
        engine.get_spot_markets().await?
    } else {
        engine.get_perp_markets().await?
    };

    render_markets(&output, fmt);
    Ok(())
}

/// `atlas candles <COIN> <INTERVAL>` with optional --limit
pub async fn candles(coin: &str, interval: &str, limit: usize, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let ci = parse_candle_interval(interval)?;
    let output = engine.get_candles(coin, ci, limit).await?;

    render_candles(&output, fmt);
    Ok(())
}

/// `atlas funding <COIN>`
pub async fn funding(coin: &str, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    let output = engine.get_funding(coin).await?;

    render_funding(&output, fmt);
    Ok(())
}

/// Parse user-provided interval string to CandleInterval.
fn parse_candle_interval(s: &str) -> Result<hypersdk::hypercore::CandleInterval> {
    use hypersdk::hypercore::CandleInterval;
    match s.to_lowercase().as_str() {
        "1m" => Ok(CandleInterval::OneMinute),
        "3m" => Ok(CandleInterval::ThreeMinutes),
        "5m" => Ok(CandleInterval::FiveMinutes),
        "15m" => Ok(CandleInterval::FifteenMinutes),
        "30m" => Ok(CandleInterval::ThirtyMinutes),
        "1h" => Ok(CandleInterval::OneHour),
        "2h" => Ok(CandleInterval::TwoHours),
        "4h" => Ok(CandleInterval::FourHours),
        "8h" => Ok(CandleInterval::EightHours),
        "12h" => Ok(CandleInterval::TwelveHours),
        "1d" => Ok(CandleInterval::OneDay),
        "3d" => Ok(CandleInterval::ThreeDays),
        "1w" => Ok(CandleInterval::OneWeek),
        "1M" | "1mo" => Ok(CandleInterval::OneMonth),
        _ => anyhow::bail!(
            "Invalid interval: {s}. Valid: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 12h, 1d, 3d, 1w, 1M"
        ),
    }
}
