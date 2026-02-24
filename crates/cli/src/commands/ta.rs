//! `atlas market hyperliquid <ta>` — Technical Analysis powered by `ta` crate.
//!
//! Pure-Rust TA library: RSI, MACD, Bollinger Bands, Stochastic, ADX, ATR,
//! EMA, SMA, OBV, CCI, Williams %R, and more.

use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_utils::output::OutputFormat;
use rust_decimal::prelude::*;
use ta::indicators::{
    RelativeStrengthIndex,
    MovingAverageConvergenceDivergence,
    BollingerBands,
    SlowStochastic,
    AverageTrueRange,
    ExponentialMovingAverage,
    SimpleMovingAverage,
    CommodityChannelIndex,
};
use ta::{Next, DataItem, Close, High, Low, Open};

/// Fetch candle data from Hyperliquid and convert to ta::DataItem.
async fn fetch_data_items(ticker: &str, timeframe: &str, count: usize)
    -> Result<(Vec<DataItem>, Vec<f64>)>
{
    let orch = Orchestrator::readonly().await?;
    let perp = orch.perp(None)?;
    let ticker_upper = ticker.to_uppercase();

    let candles = perp.candles(&ticker_upper, timeframe, count).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if candles.is_empty() {
        anyhow::bail!("No candle data for {ticker_upper}");
    }

    let mut items = Vec::with_capacity(candles.len());
    let mut volumes = Vec::with_capacity(candles.len());
    for c in &candles {
        let open = c.open.to_f64().unwrap_or(0.0);
        let high = c.high.to_f64().unwrap_or(0.0);
        let low = c.low.to_f64().unwrap_or(0.0);
        let close = c.close.to_f64().unwrap_or(0.0);
        let volume = c.volume.to_f64().unwrap_or(0.0);
        if let Ok(item) = DataItem::builder()
            .open(open).high(high).low(low).close(close).volume(volume)
            .build()
        {
            items.push(item);
            volumes.push(volume);
        }
    }

    if items.is_empty() {
        anyhow::bail!("Failed to parse candle data");
    }

    Ok((items, volumes))
}

fn print_json(val: &serde_json::Value, pretty: bool) {
    if pretty {
        println!("{}", serde_json::to_string_pretty(val).unwrap());
    } else {
        println!("{}", serde_json::to_string(val).unwrap());
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  RSI
// ═══════════════════════════════════════════════════════════════════════

pub async fn rsi(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;
    let mut rsi_ind = RelativeStrengthIndex::new(period)
        .map_err(|e| anyhow::anyhow!("RSI init: {e}"))?;

    let mut rsi_val = 50.0;
    for item in &items {
        rsi_val = rsi_ind.next(item.close());
    }

    let signal = if rsi_val > 70.0 { "overbought" } else if rsi_val < 30.0 { "oversold" } else { "neutral" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "rsi": format!("{:.2}", rsi_val), "signal": signal,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 RSI({period}) for {t} [{timeframe}]");
            println!("   Value:  {:.2}", rsi_val);
            println!("   Signal: {signal}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  MACD
// ═══════════════════════════════════════════════════════════════════════

pub async fn macd(ticker: &str, timeframe: &str, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, 150).await?;
    let mut macd_ind = MovingAverageConvergenceDivergence::new(12, 26, 9)
        .map_err(|e| anyhow::anyhow!("MACD init: {e}"))?;

    let mut output = ta::indicators::MovingAverageConvergenceDivergenceOutput {
        macd: 0.0, signal: 0.0, histogram: 0.0,
    };
    for item in &items {
        output = macd_ind.next(item.close());
    }

    let trend = if output.histogram > 0.0 { "bullish" } else { "bearish" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe,
                "macd": format!("{:.4}", output.macd),
                "signal": format!("{:.4}", output.signal),
                "histogram": format!("{:.4}", output.histogram),
                "trend": trend,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 MACD for {t} [{timeframe}]");
            println!("   MACD:      {:.4}", output.macd);
            println!("   Signal:    {:.4}", output.signal);
            println!("   Histogram: {:.4}", output.histogram);
            println!("   Trend:     {trend}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  VWAP
// ═══════════════════════════════════════════════════════════════════════

pub async fn vwap(ticker: &str, fmt: OutputFormat) -> Result<()> {
    let (items, volumes) = fetch_data_items(ticker, "1h", 24).await?;

    let mut cum_tp_vol = 0.0f64;
    let mut cum_vol = 0.0f64;
    for (i, item) in items.iter().enumerate() {
        let tp = (item.high() + item.low() + item.close()) / 3.0;
        cum_tp_vol += tp * volumes[i];
        cum_vol += volumes[i];
    }

    let vwap_val = if cum_vol > 0.0 { cum_tp_vol / cum_vol } else { 0.0 };
    let last = items.last().map(|i| i.close()).unwrap_or(0.0);
    let pos = if last > vwap_val { "above" } else { "below" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "vwap": format!("{:.2}", vwap_val),
                "last_price": format!("{:.2}", last), "position": pos,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 VWAP for {t} (24h)");
            println!("   VWAP:       ${:.2}", vwap_val);
            println!("   Last Price: ${:.2}", last);
            println!("   Position:   {pos} VWAP");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  BOLLINGER BANDS
// ═══════════════════════════════════════════════════════════════════════

pub async fn bbands(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;
    let mut bb = BollingerBands::new(period, 2.0_f64)
        .map_err(|e| anyhow::anyhow!("BBANDS init: {e}"))?;

    let mut output = ta::indicators::BollingerBandsOutput {
        average: 0.0, upper: 0.0, lower: 0.0,
    };
    for item in &items {
        output = bb.next(item.close());
    }

    let last = items.last().map(|i| i.close()).unwrap_or(0.0);
    let width = if output.average > 0.0 { (output.upper - output.lower) / output.average * 100.0 } else { 0.0 };
    let pos = if last > output.upper { "above upper" }
        else if last < output.lower { "below lower" }
        else { "within bands" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "upper": format!("{:.2}", output.upper),
                "middle": format!("{:.2}", output.average),
                "lower": format!("{:.2}", output.lower),
                "width_pct": format!("{:.2}", width),
                "position": pos, "last_price": format!("{:.2}", last),
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 Bollinger Bands({period}) for {t} [{timeframe}]");
            println!("   Upper:    ${:.2}", output.upper);
            println!("   Middle:   ${:.2}", output.average);
            println!("   Lower:    ${:.2}", output.lower);
            println!("   Width:    {:.2}%", width);
            println!("   Position: {pos}");
            println!("   Last:     ${:.2}", last);
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  STOCHASTIC
// ═══════════════════════════════════════════════════════════════════════

pub async fn stoch(ticker: &str, timeframe: &str, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, 100).await?;

    // %K via SlowStochastic, %D via EMA of %K
    let mut stoch_ind = SlowStochastic::new(14, 3)
        .map_err(|e| anyhow::anyhow!("STOCH init: {e}"))?;
    let mut d_ema = ExponentialMovingAverage::new(3)
        .map_err(|e| anyhow::anyhow!("STOCH D init: {e}"))?;

    let mut k_val = 50.0;
    let mut d_val = 50.0;
    for item in &items {
        k_val = stoch_ind.next(item);
        d_val = d_ema.next(k_val);
    }

    let signal = if k_val > 80.0 { "overbought" }
        else if k_val < 20.0 { "oversold" }
        else { "neutral" };
    let cross = if k_val > d_val { "bullish (%K > %D)" } else { "bearish (%K < %D)" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe,
                "k": format!("{:.2}", k_val), "d": format!("{:.2}", d_val),
                "signal": signal, "cross": cross,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 Stochastic(14,3) for {t} [{timeframe}]");
            println!("   %K:     {:.2}", k_val);
            println!("   %D:     {:.2}", d_val);
            println!("   Signal: {signal}");
            println!("   Cross:  {cross}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  ADX (computed from DI+/DI- manually — ta crate doesn't have ADX)
// ═══════════════════════════════════════════════════════════════════════

pub async fn adx(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;

    if items.len() < period + 1 {
        anyhow::bail!("Not enough data for ADX({period})");
    }

    // Calculate ADX from scratch: DM+, DM-, TR, then smooth
    let n = items.len();
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];

    for i in 1..n {
        let h = items[i].high();
        let l = items[i].low();
        let ph = items[i-1].high();
        let pl = items[i-1].low();
        let pc = items[i-1].close();

        let up = h - ph;
        let down = pl - l;

        plus_dm[i] = if up > down && up > 0.0 { up } else { 0.0 };
        minus_dm[i] = if down > up && down > 0.0 { down } else { 0.0 };
        tr[i] = (h - l).max((h - pc).abs()).max((l - pc).abs());
    }

    // Smooth with Wilder's method
    let p = period as f64;
    let mut atr_s = tr[1..=period].iter().sum::<f64>();
    let mut pdm_s = plus_dm[1..=period].iter().sum::<f64>();
    let mut mdm_s = minus_dm[1..=period].iter().sum::<f64>();

    let mut dx_values = Vec::new();

    for i in period..n {
        if i > period {
            atr_s = atr_s - atr_s / p + tr[i];
            pdm_s = pdm_s - pdm_s / p + plus_dm[i];
            mdm_s = mdm_s - mdm_s / p + minus_dm[i];
        }

        let pdi = if atr_s > 0.0 { pdm_s / atr_s * 100.0 } else { 0.0 };
        let mdi = if atr_s > 0.0 { mdm_s / atr_s * 100.0 } else { 0.0 };
        let di_sum = pdi + mdi;
        let dx = if di_sum > 0.0 { ((pdi - mdi).abs() / di_sum) * 100.0 } else { 0.0 };
        dx_values.push(dx);
    }

    if dx_values.len() < period {
        anyhow::bail!("Not enough data for ADX({period})");
    }

    let mut adx_val = dx_values[..period].iter().sum::<f64>() / p;
    for dx in dx_values[period..].iter() {
        adx_val = (adx_val * (p - 1.0) + dx) / p;
    }

    let strength = if adx_val > 50.0 { "strong trend" }
        else if adx_val > 25.0 { "trending" }
        else { "weak/no trend" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "adx": format!("{:.2}", adx_val), "strength": strength,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 ADX({period}) for {t} [{timeframe}]");
            println!("   ADX:      {:.2}", adx_val);
            println!("   Strength: {strength}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  ATR
// ═══════════════════════════════════════════════════════════════════════

pub async fn atr(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;
    let mut atr_ind = AverageTrueRange::new(period)
        .map_err(|e| anyhow::anyhow!("ATR init: {e}"))?;

    let mut atr_val = 0.0;
    for item in &items {
        atr_val = atr_ind.next(item);
    }

    let last = items.last().map(|i| i.close()).unwrap_or(0.0);
    let atr_pct = if last > 0.0 { atr_val / last * 100.0 } else { 0.0 };
    let volatility = if atr_pct > 5.0 { "high" } else if atr_pct > 2.0 { "moderate" } else { "low" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "atr": format!("{:.4}", atr_val),
                "atr_pct": format!("{:.2}", atr_pct),
                "volatility": volatility,
                "last_price": format!("{:.2}", last),
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 ATR({period}) for {t} [{timeframe}]");
            println!("   ATR:        ${:.4}", atr_val);
            println!("   ATR%:       {:.2}%", atr_pct);
            println!("   Volatility: {volatility}");
            println!("   Last:       ${:.2}", last);
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  EMA
// ═══════════════════════════════════════════════════════════════════════

pub async fn ema(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;
    let mut ema_ind = ExponentialMovingAverage::new(period)
        .map_err(|e| anyhow::anyhow!("EMA init: {e}"))?;

    let mut ema_val = 0.0;
    for item in &items {
        ema_val = ema_ind.next(item.close());
    }

    let last = items.last().map(|i| i.close()).unwrap_or(0.0);
    let pos = if last > ema_val { "above" } else { "below" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "ema": format!("{:.2}", ema_val),
                "last_price": format!("{:.2}", last), "position": pos,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 EMA({period}) for {t} [{timeframe}]");
            println!("   EMA:  ${:.2}", ema_val);
            println!("   Last: ${:.2}", last);
            println!("   Position: {pos}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  SMA
// ═══════════════════════════════════════════════════════════════════════

pub async fn sma(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;
    let mut sma_ind = SimpleMovingAverage::new(period)
        .map_err(|e| anyhow::anyhow!("SMA init: {e}"))?;

    let mut sma_val = 0.0;
    for item in &items {
        sma_val = sma_ind.next(item.close());
    }

    let last = items.last().map(|i| i.close()).unwrap_or(0.0);
    let pos = if last > sma_val { "above" } else { "below" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "sma": format!("{:.2}", sma_val),
                "last_price": format!("{:.2}", last), "position": pos,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 SMA({period}) for {t} [{timeframe}]");
            println!("   SMA:  ${:.2}", sma_val);
            println!("   Last: ${:.2}", last);
            println!("   Position: {pos}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  OBV (On Balance Volume)
// ═══════════════════════════════════════════════════════════════════════

pub async fn obv(ticker: &str, timeframe: &str, fmt: OutputFormat) -> Result<()> {
    let (items, volumes) = fetch_data_items(ticker, timeframe, 100).await?;

    let mut obv_val = 0.0f64;
    let mut prev_close = items[0].close();
    let mut prev_obv = 0.0;

    for (i, item) in items.iter().enumerate() {
        let close = item.close();
        if i > 0 {
            if close > prev_close {
                obv_val += volumes[i];
            } else if close < prev_close {
                obv_val -= volumes[i];
            }
        }
        if i == items.len() - 2 { prev_obv = obv_val; }
        prev_close = close;
    }

    let obv_trend = if obv_val > prev_obv { "rising" } else { "falling" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe,
                "obv": format!("{:.0}", obv_val), "trend": obv_trend,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 OBV for {t} [{timeframe}]");
            println!("   OBV:   {:.0}", obv_val);
            println!("   Trend: {obv_trend}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  CCI (Commodity Channel Index)
// ═══════════════════════════════════════════════════════════════════════

pub async fn cci(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 100).await?;
    let mut cci_ind = CommodityChannelIndex::new(period)
        .map_err(|e| anyhow::anyhow!("CCI init: {e}"))?;

    let mut cci_val = 0.0;
    for item in &items {
        cci_val = cci_ind.next(item);
    }

    let signal = if cci_val > 100.0 { "overbought" }
        else if cci_val < -100.0 { "oversold" }
        else { "neutral" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "cci": format!("{:.2}", cci_val), "signal": signal,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 CCI({period}) for {t} [{timeframe}]");
            println!("   CCI:    {:.2}", cci_val);
            println!("   Signal: {signal}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  WILLIAMS %R (manual — close vs period high/low)
// ═══════════════════════════════════════════════════════════════════════

pub async fn willr(ticker: &str, timeframe: &str, period: usize, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, period + 50).await?;

    if items.len() < period {
        anyhow::bail!("Not enough data for Williams %R({period})");
    }

    let start = items.len() - period;
    let period_high = items[start..].iter().map(|i| i.high()).fold(f64::MIN, f64::max);
    let period_low = items[start..].iter().map(|i| i.low()).fold(f64::MAX, f64::min);
    let close = items.last().map(|i| i.close()).unwrap_or(0.0);

    let wr = if (period_high - period_low).abs() > f64::EPSILON {
        ((period_high - close) / (period_high - period_low)) * -100.0
    } else { -50.0 };

    let signal = if wr > -20.0 { "overbought" }
        else if wr < -80.0 { "oversold" }
        else { "neutral" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "period": period,
                "willr": format!("{:.2}", wr), "signal": signal,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 Williams %R({period}) for {t} [{timeframe}]");
            println!("   %R:     {:.2}", wr);
            println!("   Signal: {signal}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  SAR (Parabolic SAR — manual Wilder implementation)
// ═══════════════════════════════════════════════════════════════════════

pub async fn sar(ticker: &str, timeframe: &str, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, 100).await?;

    if items.len() < 3 {
        anyhow::bail!("Not enough data for Parabolic SAR");
    }

    // Wilder's SAR
    let af_start = 0.02;
    let af_max = 0.20;
    let af_step = 0.02;

    let mut is_long = items[1].close() > items[0].close();
    let mut sar = if is_long { items[0].low() } else { items[0].high() };
    let mut ep = if is_long { items[1].high() } else { items[1].low() };
    let mut af = af_start;

    for i in 2..items.len() {
        sar = sar + af * (ep - sar);

        if is_long {
            sar = sar.min(items[i-1].low()).min(items[i-2].low());
            if items[i].low() < sar {
                is_long = false;
                sar = ep;
                ep = items[i].low();
                af = af_start;
            } else if items[i].high() > ep {
                ep = items[i].high();
                af = (af + af_step).min(af_max);
            }
        } else {
            sar = sar.max(items[i-1].high()).max(items[i-2].high());
            if items[i].high() > sar {
                is_long = true;
                sar = ep;
                ep = items[i].high();
                af = af_start;
            } else if items[i].low() < ep {
                ep = items[i].low();
                af = (af + af_step).min(af_max);
            }
        }
    }

    let last = items.last().map(|i| i.close()).unwrap_or(0.0);
    let signal = if last > sar { "bullish (price above SAR)" } else { "bearish (price below SAR)" };
    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe,
                "sar": format!("{:.4}", sar),
                "last_price": format!("{:.2}", last), "signal": signal,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 Parabolic SAR for {t} [{timeframe}]");
            println!("   SAR:    ${:.4}", sar);
            println!("   Last:   ${:.2}", last);
            println!("   Signal: {signal}");
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  CANDLESTICK PATTERNS (manual detection)
// ═══════════════════════════════════════════════════════════════════════

pub async fn patterns(ticker: &str, timeframe: &str, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, timeframe, 10).await?;

    if items.len() < 3 {
        anyhow::bail!("Need at least 3 candles for pattern detection");
    }

    let mut detected: Vec<(&str, &str, &str)> = Vec::new();
    let n = items.len();
    let last = &items[n - 1];
    let prev = &items[n - 2];

    let body = (last.close() - last.open()).abs();
    let range = last.high() - last.low();
    let upper_shadow = last.high() - last.close().max(last.open());
    let lower_shadow = last.close().min(last.open()) - last.low();

    // Doji
    if range > 0.0 && body / range < 0.1 {
        detected.push(("Doji", "indecision", "neutral"));
    }

    // Hammer (bullish reversal)
    if range > 0.0 && lower_shadow > body * 2.0 && upper_shadow < body * 0.5 {
        detected.push(("Hammer", "bullish reversal", "bullish"));
    }

    // Shooting Star (bearish reversal)
    if range > 0.0 && upper_shadow > body * 2.0 && lower_shadow < body * 0.5 {
        detected.push(("Shooting Star", "bearish reversal", "bearish"));
    }

    // Bullish Engulfing
    if prev.close() < prev.open() && last.close() > last.open()
        && last.open() <= prev.close() && last.close() >= prev.open()
    {
        detected.push(("Bullish Engulfing", "bullish reversal", "bullish"));
    }

    // Bearish Engulfing
    if prev.close() > prev.open() && last.close() < last.open()
        && last.open() >= prev.close() && last.close() <= prev.open()
    {
        detected.push(("Bearish Engulfing", "bearish reversal", "bearish"));
    }

    // Bullish Harami
    if prev.close() < prev.open() && last.close() > last.open()
        && last.open() > prev.close() && last.close() < prev.open()
    {
        detected.push(("Bullish Harami", "bullish reversal", "bullish"));
    }

    // Bearish Harami
    if prev.close() > prev.open() && last.close() < last.open()
        && last.open() < prev.close() && last.close() > prev.open()
    {
        detected.push(("Bearish Harami", "bearish reversal", "bearish"));
    }

    let t = ticker.to_uppercase();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let pats: Vec<serde_json::Value> = detected.iter().map(|(name, kind, sig)| {
                serde_json::json!({ "pattern": name, "type": kind, "signal": sig })
            }).collect();
            print_json(&serde_json::json!({
                "ticker": t, "timeframe": timeframe, "patterns": pats,
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("🕯️ Candlestick Patterns for {t} [{timeframe}]\n");
            if detected.is_empty() {
                println!("   No patterns detected on latest candle.");
            } else {
                println!("{:<22} {:<22} {:<10}", "PATTERN", "TYPE", "SIGNAL");
                println!("{}", "─".repeat(55));
                for (name, kind, sig) in &detected {
                    println!("{:<22} {:<22} {:<10}", name, kind, sig);
                }
            }
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
//  TREND (multi-indicator composite)
// ═══════════════════════════════════════════════════════════════════════

pub async fn trend(ticker: &str, fmt: OutputFormat) -> Result<()> {
    let (items, _) = fetch_data_items(ticker, "1h", 200).await?;
    let t = ticker.to_uppercase();

    // RSI
    let mut rsi_ind = RelativeStrengthIndex::new(14).unwrap();
    let mut rsi_val = 50.0;
    for item in &items { rsi_val = rsi_ind.next(item.close()); }

    // MACD
    let mut macd_ind = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
    let mut macd_out = ta::indicators::MovingAverageConvergenceDivergenceOutput {
        macd: 0.0, signal: 0.0, histogram: 0.0,
    };
    for item in &items { macd_out = macd_ind.next(item.close()); }

    // SMA 20 & 50
    let mut sma20_ind = SimpleMovingAverage::new(20).unwrap();
    let mut sma50_ind = SimpleMovingAverage::new(50).unwrap();
    let mut sma20_val = 0.0;
    let mut sma50_val = 0.0;
    for item in &items {
        sma20_val = sma20_ind.next(item.close());
        sma50_val = sma50_ind.next(item.close());
    }

    // Bollinger Bands
    let mut bb = BollingerBands::new(20, 2.0_f64).unwrap();
    let mut bb_out = ta::indicators::BollingerBandsOutput {
        average: 0.0, upper: 0.0, lower: 0.0,
    };
    for item in &items { bb_out = bb.next(item.close()); }

    // ATR
    let mut atr_ind = AverageTrueRange::new(14).unwrap();
    let mut atr_val = 0.0;
    for item in &items { atr_val = atr_ind.next(item); }

    let last = items.last().map(|i| i.close()).unwrap_or(0.0);

    // Score
    let mut score = 50i32;
    if rsi_val > 50.0 { score += ((rsi_val - 50.0) * 0.5) as i32; }
    else { score -= ((50.0 - rsi_val) * 0.5) as i32; }
    if macd_out.histogram > 0.0 { score += 12; } else { score -= 12; }
    if macd_out.macd > macd_out.signal { score += 5; } else { score -= 5; }
    if last > sma20_val { score += 8; } else { score -= 8; }
    if sma20_val > sma50_val { score += 8; } else { score -= 8; }
    let score = score.clamp(0, 100);

    let trend_label = if score >= 70 { "bullish" }
        else if score >= 55 { "slightly bullish" }
        else if score >= 45 { "neutral" }
        else if score >= 30 { "slightly bearish" }
        else { "bearish" };

    let recent = items.len().saturating_sub(24);
    let support = items[recent..].iter().map(|i| i.low()).fold(f64::MAX, f64::min);
    let resistance = items[recent..].iter().map(|i| i.high()).fold(0.0f64, f64::max);

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            print_json(&serde_json::json!({
                "ticker": t, "trend": trend_label, "score": score,
                "rsi": format!("{:.2}", rsi_val),
                "macd_histogram": format!("{:.4}", macd_out.histogram),
                "atr": format!("{:.4}", atr_val),
                "sma_20": format!("{:.2}", sma20_val),
                "sma_50": format!("{:.2}", sma50_val),
                "bb_upper": format!("{:.2}", bb_out.upper),
                "bb_lower": format!("{:.2}", bb_out.lower),
                "support": format!("{:.2}", support),
                "resistance": format!("{:.2}", resistance),
                "last_price": format!("{:.2}", last),
            }), matches!(fmt, OutputFormat::JsonPretty));
        }
        OutputFormat::Table => {
            println!("📊 TREND ANALYSIS: {t}");
            println!("   Trend:      {trend_label}");
            println!("   Score:      {score}/100");
            println!("   ─────────────────────");
            println!("   RSI(14):    {:.2}", rsi_val);
            println!("   MACD Hist:  {:.4}", macd_out.histogram);
            println!("   ATR(14):    ${:.4}", atr_val);
            println!("   SMA(20):    ${:.2}", sma20_val);
            println!("   SMA(50):    ${:.2}", sma50_val);
            println!("   BB:         ${:.2} — ${:.2}", bb_out.lower, bb_out.upper);
            println!("   Support:    ${:.2}", support);
            println!("   Resistance: ${:.2}", resistance);
            println!("   Last:       ${:.2}", last);
        }
    }
    Ok(())
}
