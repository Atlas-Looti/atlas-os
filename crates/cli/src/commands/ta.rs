//! `atlas ta` — Technical Analysis commands.
//!
//! Calculates indicators from candle data fetched via the Orchestrator.

use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_utils::output::OutputFormat;
use rust_decimal::prelude::*;


/// `atlas ta rsi <TICKER> [--timeframe 1h] [--period 14]`
pub async fn rsi(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let ticker_upper = ticker.to_uppercase();

    // Fetch enough candles for RSI calculation (period + 1 minimum)
    let candles = perp.candles(&ticker_upper, timeframe, period + 50).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if candles.len() < period + 1 {
        anyhow::bail!("Not enough candle data for RSI (need {} candles, got {})", period + 1, candles.len());
    }

    // Calculate RSI
    let closes: Vec<f64> = candles.iter().map(|c| c.close.to_f64().unwrap_or(0.0)).collect();
    let rsi_value = calculate_rsi(&closes, period);

    let signal = if rsi_value > 70.0 { "overbought" }
        else if rsi_value < 30.0 { "oversold" }
        else { "neutral" };

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "ticker": ticker_upper,
                "timeframe": timeframe,
                "period": period,
                "rsi": format!("{:.2}", rsi_value),
                "signal": signal,
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("📊 RSI({period}) for {ticker_upper} [{timeframe}]");
            println!("   Value:  {:.2}", rsi_value);
            println!("   Signal: {signal}");
        }
    }
    Ok(())
}

/// `atlas ta macd <TICKER> [--timeframe 1h]`
pub async fn macd(ticker: &str, timeframe: &str, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let ticker_upper = ticker.to_uppercase();

    let candles = perp.candles(&ticker_upper, timeframe, 100).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if candles.len() < 26 {
        anyhow::bail!("Not enough data for MACD (need 26+ candles)");
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close.to_f64().unwrap_or(0.0)).collect();
    let (macd_line, signal_line, histogram) = calculate_macd(&closes);

    let trend = if histogram > 0.0 { "bullish" } else { "bearish" };

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "ticker": ticker_upper,
                "timeframe": timeframe,
                "macd": format!("{:.4}", macd_line),
                "signal": format!("{:.4}", signal_line),
                "histogram": format!("{:.4}", histogram),
                "trend": trend,
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("📊 MACD for {ticker_upper} [{timeframe}]");
            println!("   MACD:      {:.4}", macd_line);
            println!("   Signal:    {:.4}", signal_line);
            println!("   Histogram: {:.4}", histogram);
            println!("   Trend:     {trend}");
        }
    }
    Ok(())
}

/// `atlas ta vwap <TICKER>`
pub async fn vwap(ticker: &str, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let ticker_upper = ticker.to_uppercase();

    // Use 1h candles for VWAP
    let candles = perp.candles(&ticker_upper, "1h", 24).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if candles.is_empty() {
        anyhow::bail!("No candle data for VWAP");
    }

    let mut cum_tp_vol = 0.0f64;
    let mut cum_vol = 0.0f64;

    for c in &candles {
        let tp = (c.high.to_f64().unwrap_or(0.0) + c.low.to_f64().unwrap_or(0.0) + c.close.to_f64().unwrap_or(0.0)) / 3.0;
        let vol = c.volume.to_f64().unwrap_or(0.0);
        cum_tp_vol += tp * vol;
        cum_vol += vol;
    }

    let vwap_value = if cum_vol > 0.0 { cum_tp_vol / cum_vol } else { 0.0 };

    let last_price = candles.last().map(|c| c.close.to_f64().unwrap_or(0.0)).unwrap_or(0.0);
    let position = if last_price > vwap_value { "above" } else { "below" };

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "ticker": ticker_upper,
                "vwap": format!("{:.2}", vwap_value),
                "last_price": format!("{:.2}", last_price),
                "position": position,
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("📊 VWAP for {ticker_upper} (24h)");
            println!("   VWAP:       ${:.2}", vwap_value);
            println!("   Last Price: ${:.2}", last_price);
            println!("   Position:   {position} VWAP");
        }
    }
    Ok(())
}

/// `atlas ta trend <TICKER>` — multi-indicator trend signal
pub async fn trend(ticker: &str, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;
    let ticker_upper = ticker.to_uppercase();

    // Fetch 1h candles for multi-indicator analysis
    let candles = perp.candles(&ticker_upper, "1h", 100).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if candles.len() < 26 {
        anyhow::bail!("Not enough data for trend analysis");
    }

    let closes: Vec<f64> = candles.iter().map(|c| c.close.to_f64().unwrap_or(0.0)).collect();
    let last = *closes.last().unwrap_or(&0.0);

    // RSI
    let rsi = calculate_rsi(&closes, 14);

    // MACD
    let (macd_line, signal_line, histogram) = calculate_macd(&closes);

    // Simple moving averages
    let sma_20 = sma(&closes, 20);
    let sma_50 = sma(&closes, 50);

    // Score: 0-100 (0 = extreme bearish, 100 = extreme bullish)
    let mut score = 50i32;

    // RSI contribution (-20 to +20)
    if rsi > 70.0 { score -= 10; } // overbought
    else if rsi > 50.0 { score += ((rsi - 50.0) as i32).min(15); }
    else if rsi < 30.0 { score -= 10; } // oversold
    else { score -= ((50.0 - rsi) as i32).min(15); }

    // MACD contribution (-20 to +20)
    if histogram > 0.0 { score += 15; } else { score -= 15; }
    if macd_line > signal_line { score += 5; } else { score -= 5; }

    // SMA contribution (-20 to +20)
    if last > sma_20 { score += 10; } else { score -= 10; }
    if sma_20 > sma_50 { score += 10; } else { score -= 10; }

    let score = score.clamp(0, 100);
    let trend_label = if score >= 70 { "bullish" }
        else if score >= 55 { "slightly_bullish" }
        else if score >= 45 { "neutral" }
        else if score >= 30 { "slightly_bearish" }
        else { "bearish" };

    // Approximate support/resistance from recent candle ranges
    let recent = &candles[candles.len().saturating_sub(24)..];
    let support = recent.iter().map(|c| c.low.to_f64().unwrap_or(f64::MAX)).fold(f64::MAX, f64::min);
    let resistance = recent.iter().map(|c| c.high.to_f64().unwrap_or(0.0)).fold(0.0f64, f64::max);

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "ticker": ticker_upper,
                "trend": trend_label,
                "score": score,
                "rsi": format!("{:.2}", rsi),
                "macd_histogram": format!("{:.4}", histogram),
                "sma_20": format!("{:.2}", sma_20),
                "sma_50": format!("{:.2}", sma_50),
                "support": format!("{:.2}", support),
                "resistance": format!("{:.2}", resistance),
                "last_price": format!("{:.2}", last),
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("📊 TREND ANALYSIS: {ticker_upper}");
            println!("   Trend:      {trend_label}");
            println!("   Score:      {score}/100");
            println!("   RSI(14):    {:.2}", rsi);
            println!("   MACD Hist:  {:.4}", histogram);
            println!("   SMA(20):    ${:.2}", sma_20);
            println!("   SMA(50):    ${:.2}", sma_50);
            println!("   Support:    ${:.2}", support);
            println!("   Resistance: ${:.2}", resistance);
            println!("   Last:       ${:.2}", last);
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  TA MATH — Pure functions, no dependencies
// ═══════════════════════════════════════════════════════════════════════

fn calculate_rsi(closes: &[f64], period: usize) -> f64 {
    if closes.len() < period + 1 { return 50.0; }

    let mut gains = 0.0f64;
    let mut losses = 0.0f64;

    // Initial average gain/loss
    for i in 1..=period {
        let change = closes[i] - closes[i - 1];
        if change > 0.0 { gains += change; } else { losses += change.abs(); }
    }

    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    // Smoothed RSI
    for i in (period + 1)..closes.len() {
        let change = closes[i] - closes[i - 1];
        let (gain, loss) = if change > 0.0 { (change, 0.0) } else { (0.0, change.abs()) };
        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
    }

    if avg_loss == 0.0 { return 100.0; }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() || period == 0 { return vec![]; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = vec![data[0]];
    for i in 1..data.len() {
        let prev = *result.last().unwrap();
        result.push(data[i] * k + prev * (1.0 - k));
    }
    result
}

fn calculate_macd(closes: &[f64]) -> (f64, f64, f64) {
    let ema_12 = ema(closes, 12);
    let ema_26 = ema(closes, 26);

    if ema_12.is_empty() || ema_26.is_empty() { return (0.0, 0.0, 0.0); }

    let macd_line: Vec<f64> = ema_12.iter().zip(ema_26.iter())
        .map(|(a, b)| a - b)
        .collect();

    let signal = ema(&macd_line, 9);

    let last_macd = *macd_line.last().unwrap_or(&0.0);
    let last_signal = *signal.last().unwrap_or(&0.0);
    let histogram = last_macd - last_signal;

    (last_macd, last_signal, histogram)
}

fn sma(data: &[f64], period: usize) -> f64 {
    if data.len() < period { return 0.0; }
    let slice = &data[data.len() - period..];
    slice.iter().sum::<f64>() / period as f64
}
