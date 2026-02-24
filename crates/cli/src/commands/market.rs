use anyhow::Result;
use atlas_core::fmt::format_timestamp_ms;
use atlas_core::output::OutputFormat;
use atlas_core::output::*;
use rust_decimal::prelude::*;

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
            println!(
                "{:<15} {:>6} {:>10} {:>12}",
                "NAME", "INDEX", "MAX LEV", "SZ DECIMALS"
            );
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
fn render_candles(output: &CandlesOutput, fmt: OutputFormat) {
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
fn render_funding(output: &FundingOutput, fmt: OutputFormat) {
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
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;

    let tickers = if all || coins.is_empty() {
        perp.all_tickers()
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        let mut result = Vec::new();
        for c in coins {
            let t = perp
                .ticker(&c.to_uppercase())
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            result.push(t);
        }
        result
    };

    let prices: Vec<PriceRow> = tickers
        .iter()
        .map(|t| PriceRow {
            coin: t.symbol.clone(),
            mid_price: t.mid_price.to_string(),
        })
        .collect();

    render_prices(&PriceOutput { prices }, fmt);
    Ok(())
}

/// `atlas markets` or `atlas markets --spot`
pub async fn markets(spot: bool, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;

    let market_list = perp.markets().await.map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<MarketRow> = market_list
        .iter()
        .map(|m| MarketRow {
            name: m.symbol.clone(),
            index: 0, // universal Market doesn't have index
            max_leverage: m.max_leverage.unwrap_or(1) as u64,
            sz_decimals: m.sz_decimals.unwrap_or(0) as i64,
        })
        .collect();

    let market_type = if spot { "spot" } else { "perp" };
    render_markets(
        &MarketsOutput {
            market_type: market_type.into(),
            markets: rows,
        },
        fmt,
    );
    Ok(())
}

/// `atlas candles <COIN> <INTERVAL>` with optional --limit
pub async fn candles(coin: &str, interval: &str, limit: usize, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let candle_data = perp
        .candles(&coin_upper, interval, limit)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<CandleRow> = candle_data
        .iter()
        .map(|c| CandleRow {
            time: format_timestamp_ms(c.open_time_ms),
            open: c.open.to_string(),
            high: c.high.to_string(),
            low: c.low.to_string(),
            close: c.close.to_string(),
            volume: c.volume.to_string(),
            trades: c.trades.unwrap_or(0),
        })
        .collect();

    render_candles(
        &CandlesOutput {
            coin: coin_upper,
            interval: interval.into(),
            candles: rows,
        },
        fmt,
    );
    Ok(())
}

/// `atlas funding <COIN>`
pub async fn funding(coin: &str, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let rates = perp
        .funding(&coin_upper)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let rows: Vec<FundingRow> = rates
        .iter()
        .map(|r| FundingRow {
            time: format_timestamp_ms(r.timestamp_ms),
            coin: r.symbol.clone(),
            rate: r.rate.to_string(),
            premium: r
                .premium
                .map(|p| p.to_string())
                .unwrap_or_else(|| "—".into()),
        })
        .collect();

    render_funding(
        &FundingOutput {
            coin: coin_upper,
            rates: rows,
        },
        fmt,
    );
    Ok(())
}

/// `atlas market orderbook <TICKER> [--depth 10]`
pub async fn orderbook(ticker: &str, depth: usize, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;
    let ticker_upper = ticker.to_uppercase();

    match perp.orderbook(&ticker_upper, depth).await {
        Ok(book) => {
            match fmt {
                OutputFormat::Json | OutputFormat::JsonPretty => {
                    let json = serde_json::json!({
                        "ticker": ticker_upper,
                        "bids": book.bids.iter().map(|b| {
                            serde_json::json!({"price": b.price.to_string(), "size": b.size.to_string()})
                        }).collect::<Vec<_>>(),
                        "asks": book.asks.iter().map(|a| {
                            serde_json::json!({"price": a.price.to_string(), "size": a.size.to_string()})
                        }).collect::<Vec<_>>(),
                    });
                    let s = if matches!(fmt, OutputFormat::JsonPretty) {
                        serde_json::to_string_pretty(&json)?
                    } else {
                        serde_json::to_string(&json)?
                    };
                    println!("{s}");
                }
                OutputFormat::Table => {
                    println!("📖 {} Order Book (depth={})\n", ticker_upper, depth);
                    println!(
                        "{:>14} {:>14}  |  {:>14} {:>14}",
                        "BID SIZE", "BID PRICE", "ASK PRICE", "ASK SIZE"
                    );
                    println!("{}", "─".repeat(65));
                    let show = depth.min(book.bids.len()).min(book.asks.len());
                    for i in 0..show {
                        println!(
                            "{:>14} {:>14}  |  {:>14} {:>14}",
                            book.bids[i].size,
                            book.bids[i].price,
                            book.asks[i].price,
                            book.asks[i].size
                        );
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            // Orderbook might be WebSocket-only on some protocols
            if fmt != OutputFormat::Table {
                let json = serde_json::json!({
                    "error": format!("{e}"),
                    "hint": format!("atlas stream book {ticker_upper}"),
                });
                println!("{}", serde_json::to_string(&json)?);
            } else {
                println!("⚠️  Orderbook: {e}");
                println!(
                    "   Use `atlas stream book {ticker_upper}` for live order book via WebSocket."
                );
            }
            Ok(())
        }
    }
}

/// `atlas market info <COIN>` — detailed market info with OI, volume, spread.
pub async fn info(coin: &str, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;
    let coin_upper = coin.to_uppercase();

    let ticker = perp
        .ticker(&coin_upper)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let markets = perp.markets().await.map_err(|e| anyhow::anyhow!("{e}"))?;
    let market = markets.iter().find(|m| m.symbol == coin_upper);

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "symbol": coin_upper,
                "mid_price": ticker.mid_price.to_string(),
                "best_bid": ticker.best_bid.map(|b| b.to_string()),
                "best_ask": ticker.best_ask.map(|a| a.to_string()),
                "spread": ticker.best_bid.and_then(|b| ticker.best_ask.map(|a| (a - b).to_string())),
                "spread_bps": ticker.best_bid.and_then(|b| ticker.best_ask.map(|a| {
                    if ticker.mid_price > Decimal::ZERO {
                        ((a - b) / ticker.mid_price * Decimal::from(10000)).round_dp(2).to_string()
                    } else { "0".to_string() }
                })),
                "volume_24h": ticker.volume_24h.map(|v| v.to_string()),
                "change_24h_pct": ticker.change_24h_pct.map(|c| c.to_string()),
                "open_interest": market.and_then(|m| m.open_interest.map(|o| o.to_string())),
                "mark_price": market.and_then(|m| m.mark_price.map(|p| p.to_string())),
                "index_price": market.and_then(|m| m.index_price.map(|p| p.to_string())),
                "max_leverage": market.and_then(|m| m.max_leverage),
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            let spread = ticker.best_bid.and_then(|b| ticker.best_ask.map(|a| a - b));
            let spread_bps = spread.map(|s| {
                if ticker.mid_price > Decimal::ZERO {
                    (s / ticker.mid_price * Decimal::from(10000)).round_dp(2)
                } else {
                    Decimal::ZERO
                }
            });

            println!("┌─────────────────────────────────────────────────┐");
            println!(
                "│  {} — Market Info{:>width$}│",
                coin_upper,
                "",
                width = 47 - coin_upper.len() - 15
            );
            println!("├─────────────────────────────────────────────────┤");
            println!("│  Mid Price     : ${:<29}│", ticker.mid_price);
            println!(
                "│  Best Bid      : ${:<29}│",
                ticker.best_bid.map(|b| b.to_string()).unwrap_or("—".into())
            );
            println!(
                "│  Best Ask      : ${:<29}│",
                ticker.best_ask.map(|a| a.to_string()).unwrap_or("—".into())
            );
            println!(
                "│  Spread        : ${:<22} ({} bps)│",
                spread.map(|s| s.to_string()).unwrap_or("—".into()),
                spread_bps.map(|s| s.to_string()).unwrap_or("—".into())
            );
            println!("├─────────────────────────────────────────────────┤");
            println!(
                "│  24h Volume    : ${:<29}│",
                ticker
                    .volume_24h
                    .map(|v| format!("{:.0}", v))
                    .unwrap_or("—".into())
            );
            println!(
                "│  24h Change    : {:<30}│",
                ticker
                    .change_24h_pct
                    .map(|c| format!("{:+.2}%", c))
                    .unwrap_or("—".into())
            );

            if let Some(m) = market {
                println!("├─────────────────────────────────────────────────┤");
                println!(
                    "│  Mark Price    : ${:<29}│",
                    m.mark_price.map(|p| p.to_string()).unwrap_or("—".into())
                );
                println!(
                    "│  Index Price   : ${:<29}│",
                    m.index_price.map(|p| p.to_string()).unwrap_or("—".into())
                );
                println!(
                    "│  Open Interest : ${:<29}│",
                    m.open_interest
                        .map(|o| format!("{:.0}", o))
                        .unwrap_or("—".into())
                );
                println!(
                    "│  Max Leverage  : {:<30}│",
                    m.max_leverage
                        .map(|l| format!("{}x", l))
                        .unwrap_or("—".into())
                );
            }
            println!("└─────────────────────────────────────────────────┘");
        }
    }

    Ok(())
}

/// `atlas market top [--sort volume|change|oi] [--limit 20] [--reverse]`
pub async fn top(sort_by: &str, limit: usize, reverse: bool, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;

    let mut tickers = perp
        .all_tickers()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Sort
    match sort_by {
        "volume" | "vol" => {
            tickers.sort_by(|a, b| {
                let va = a.volume_24h.unwrap_or(Decimal::ZERO);
                let vb = b.volume_24h.unwrap_or(Decimal::ZERO);
                vb.cmp(&va)
            });
        }
        "change" | "chg" | "gainers" => {
            tickers.sort_by(|a, b| {
                let ca = a.change_24h_pct.unwrap_or(Decimal::ZERO);
                let cb = b.change_24h_pct.unwrap_or(Decimal::ZERO);
                cb.cmp(&ca)
            });
        }
        "losers" => {
            tickers.sort_by(|a, b| {
                let ca = a.change_24h_pct.unwrap_or(Decimal::ZERO);
                let cb = b.change_24h_pct.unwrap_or(Decimal::ZERO);
                ca.cmp(&cb)
            });
        }
        "price" => {
            tickers.sort_by(|a, b| b.mid_price.cmp(&a.mid_price));
        }
        _ => {
            tickers.sort_by(|a, b| {
                let va = a.volume_24h.unwrap_or(Decimal::ZERO);
                let vb = b.volume_24h.unwrap_or(Decimal::ZERO);
                vb.cmp(&va)
            });
        }
    }

    if reverse {
        tickers.reverse();
    }

    tickers.truncate(limit);

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let rows: Vec<serde_json::Value> = tickers
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "symbol": t.symbol,
                        "mid_price": t.mid_price.to_string(),
                        "volume_24h": t.volume_24h.map(|v| v.to_string()),
                        "change_24h_pct": t.change_24h_pct.map(|c| c.to_string()),
                        "best_bid": t.best_bid.map(|b| b.to_string()),
                        "best_ask": t.best_ask.map(|a| a.to_string()),
                    })
                })
                .collect();
            let json = serde_json::json!({ "markets": rows });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            let title = match sort_by {
                "gainers" | "change" | "chg" => "Top Gainers",
                "losers" => "Top Losers",
                "price" => "By Price",
                _ => "By Volume",
            };
            println!("📊 {} (top {})\n", title, limit);
            println!(
                "{:<12} {:>14} {:>16} {:>10}",
                "COIN", "PRICE", "24h VOLUME", "24h CHG"
            );
            println!("{}", "─".repeat(55));
            for t in &tickers {
                let vol = t
                    .volume_24h
                    .map(|v| {
                        if v >= Decimal::from(1_000_000) {
                            format!("${:.1}M", v.to_f64().unwrap_or(0.0) / 1_000_000.0)
                        } else if v >= Decimal::from(1_000) {
                            format!("${:.1}K", v.to_f64().unwrap_or(0.0) / 1_000.0)
                        } else {
                            format!("${:.0}", v)
                        }
                    })
                    .unwrap_or("—".into());
                let chg = t
                    .change_24h_pct
                    .map(|c| format!("{:+.2}%", c))
                    .unwrap_or("—".into());
                println!(
                    "{:<12} {:>14} {:>16} {:>10}",
                    t.symbol, t.mid_price, vol, chg
                );
            }
        }
    }

    Ok(())
}

/// `atlas market spread <COINS...>` — bid-ask spreads.
pub async fn spread(coins: &[String], fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;

    let tickers = if coins.is_empty() {
        perp.all_tickers()
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        let mut result = Vec::new();
        for c in coins {
            let t = perp
                .ticker(&c.to_uppercase())
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            result.push(t);
        }
        result
    };

    // Filter to only those with bid/ask data
    let with_spread: Vec<_> = tickers
        .iter()
        .filter(|t| t.best_bid.is_some() && t.best_ask.is_some())
        .collect();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let rows: Vec<serde_json::Value> = with_spread
                .iter()
                .filter_map(|t| {
                    let bid = t.best_bid?;
                    let ask = t.best_ask?;
                    let spread_abs = ask - bid;
                    let spread_bps = if t.mid_price > Decimal::ZERO {
                        (spread_abs / t.mid_price * Decimal::from(10000)).round_dp(2)
                    } else {
                        Decimal::ZERO
                    };
                    Some(serde_json::json!({
                        "symbol": t.symbol,
                        "bid": bid.to_string(),
                        "ask": ask.to_string(),
                        "spread": spread_abs.to_string(),
                        "spread_bps": spread_bps.to_string(),
                        "mid": t.mid_price.to_string(),
                    }))
                })
                .collect();
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&rows)?
            } else {
                serde_json::to_string(&rows)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!(
                "{:<12} {:>14} {:>14} {:>12} {:>8}",
                "COIN", "BID", "ASK", "SPREAD", "BPS"
            );
            println!("{}", "─".repeat(63));
            for t in &with_spread {
                let (Some(bid), Some(ask)) = (t.best_bid, t.best_ask) else {
                    continue;
                };
                let (bid, ask) = (bid, ask);
                let spread_abs = ask - bid;
                let spread_bps = if t.mid_price > Decimal::ZERO {
                    (spread_abs / t.mid_price * Decimal::from(10000)).round_dp(2)
                } else {
                    Decimal::ZERO
                };
                println!(
                    "{:<12} {:>14} {:>14} {:>12} {:>8}",
                    t.symbol, bid, ask, spread_abs, spread_bps
                );
            }
        }
    }

    Ok(())
}

/// `atlas market search <query>` — search markets by name.
pub async fn search(query: &str, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;

    let all_markets = perp.markets().await.map_err(|e| anyhow::anyhow!("{e}"))?;

    let q = query.to_uppercase();
    let matches: Vec<_> = all_markets
        .iter()
        .filter(|m| m.symbol.contains(&q) || m.base.to_uppercase().contains(&q))
        .collect();

    if matches.is_empty() {
        println!("No markets matching '{query}'");
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let rows: Vec<serde_json::Value> = matches
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "symbol": m.symbol,
                        "base": m.base,
                        "quote": m.quote,
                        "market_type": format!("{:?}", m.market_type),
                        "mark_price": m.mark_price.map(|p| p.to_string()),
                        "max_leverage": m.max_leverage,
                    })
                })
                .collect();
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&rows)?
            } else {
                serde_json::to_string(&rows)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!(
                "🔍 Markets matching '{}' ({} found)\n",
                query,
                matches.len()
            );
            println!(
                "{:<15} {:<8} {:<8} {:>12} {:>8}",
                "SYMBOL", "BASE", "QUOTE", "MARK PRICE", "MAX LEV"
            );
            println!("{}", "─".repeat(55));
            for m in &matches {
                println!(
                    "{:<15} {:<8} {:<8} {:>12} {:>8}",
                    m.symbol,
                    m.base,
                    m.quote,
                    m.mark_price
                        .map(|p| format!("${}", p))
                        .unwrap_or("—".into()),
                    m.max_leverage
                        .map(|l| format!("{}x", l))
                        .unwrap_or("—".into())
                );
            }
        }
    }

    Ok(())
}

/// `atlas market summary` — quick market dashboard.
pub async fn summary(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::readonly().await?;
    let perp = orch.perp(None)?;

    let tickers = perp
        .all_tickers()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let total = tickers.len();
    let total_volume: Decimal = tickers.iter().filter_map(|t| t.volume_24h).sum();

    let gainers = tickers
        .iter()
        .filter(|t| t.change_24h_pct.map(|c| c > Decimal::ZERO).unwrap_or(false))
        .count();
    let losers = tickers
        .iter()
        .filter(|t| t.change_24h_pct.map(|c| c < Decimal::ZERO).unwrap_or(false))
        .count();

    let mut sorted_by_change = tickers.clone();
    sorted_by_change.sort_by(|a, b| {
        let ca = a.change_24h_pct.unwrap_or(Decimal::ZERO);
        let cb = b.change_24h_pct.unwrap_or(Decimal::ZERO);
        cb.cmp(&ca)
    });

    let top3_gainers: Vec<_> = sorted_by_change.iter().take(3).collect();
    let top3_losers: Vec<_> = sorted_by_change.iter().rev().take(3).collect();

    let mut sorted_by_vol = tickers.clone();
    sorted_by_vol.sort_by(|a, b| {
        let va = a.volume_24h.unwrap_or(Decimal::ZERO);
        let vb = b.volume_24h.unwrap_or(Decimal::ZERO);
        vb.cmp(&va)
    });
    let top3_volume: Vec<_> = sorted_by_vol.iter().take(3).collect();

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "total_markets": total,
                "total_volume_24h": total_volume.to_string(),
                "gainers": gainers,
                "losers": losers,
                "top_gainers": top3_gainers.iter().map(|t| serde_json::json!({
                    "symbol": t.symbol, "change": t.change_24h_pct.map(|c| c.to_string())
                })).collect::<Vec<_>>(),
                "top_losers": top3_losers.iter().map(|t| serde_json::json!({
                    "symbol": t.symbol, "change": t.change_24h_pct.map(|c| c.to_string())
                })).collect::<Vec<_>>(),
                "top_volume": top3_volume.iter().map(|t| serde_json::json!({
                    "symbol": t.symbol, "volume": t.volume_24h.map(|v| v.to_string())
                })).collect::<Vec<_>>(),
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            let vol_str = if total_volume >= Decimal::from(1_000_000_000) {
                format!(
                    "${:.2}B",
                    total_volume.to_f64().unwrap_or(0.0) / 1_000_000_000.0
                )
            } else if total_volume >= Decimal::from(1_000_000) {
                format!(
                    "${:.1}M",
                    total_volume.to_f64().unwrap_or(0.0) / 1_000_000.0
                )
            } else {
                format!("${:.0}", total_volume)
            };

            println!("┌─────────────────────────────────────────────────┐");
            println!("│  📊 MARKET SUMMARY                              │");
            println!("├─────────────────────────────────────────────────┤");
            println!("│  Markets       : {:<30} │", total);
            println!("│  24h Volume    : {:<30} │", vol_str);
            println!("│  Gainers       : {:<14} Losers: {:<9} │", gainers, losers);
            println!("├─────────────────────────────────────────────────┤");
            println!("│  🟢 Top Gainers                                 │");
            for t in &top3_gainers {
                let chg = t
                    .change_24h_pct
                    .map(|c| format!("{:+.2}%", c))
                    .unwrap_or("—".into());
                println!("│    {:<12} {:>12}  ${:<18} │", t.symbol, chg, t.mid_price);
            }
            println!("│  🔴 Top Losers                                  │");
            for t in &top3_losers {
                let chg = t
                    .change_24h_pct
                    .map(|c| format!("{:+.2}%", c))
                    .unwrap_or("—".into());
                println!("│    {:<12} {:>12}  ${:<18} │", t.symbol, chg, t.mid_price);
            }
            println!("│  📈 Top Volume                                  │");
            for t in &top3_volume {
                let vol = t
                    .volume_24h
                    .map(|v| {
                        if v >= Decimal::from(1_000_000) {
                            format!("${:.1}M", v.to_f64().unwrap_or(0.0) / 1_000_000.0)
                        } else {
                            format!("${:.0}", v)
                        }
                    })
                    .unwrap_or("—".into());
                println!("│    {:<12} {:>12}  ${:<18} │", t.symbol, vol, t.mid_price);
            }
            println!("└─────────────────────────────────────────────────┘");
        }
    }

    Ok(())
}
