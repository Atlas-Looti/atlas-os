use anyhow::Result;
use futures::StreamExt;
use hypersdk::hypercore::{
    self as hypercore,
    types::{Incoming, Subscription},
    ws::Event,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

use atlas_core::Engine;
use atlas_utils::output::OutputFormat;

/// `atlas stream prices` — live mid prices for all markets
pub async fn stream_prices(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;

    println!("🔴 Streaming all mid prices (Ctrl+C to stop)...\n");

    let mut ws = engine.client.websocket();
    ws.subscribe(Subscription::AllMids { dex: None });

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::AllMids { dex: _, mids }) = event {
            render_mids_update(&mids, fmt);
        }
    }

    Ok(())
}

/// `atlas stream trades <COIN>` — live trade feed
pub async fn stream_trades(coin: &str, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let _ = engine; // just to validate profile

    let core = hypercore::mainnet();
    let mut ws = core.websocket();
    ws.subscribe(Subscription::Trades { coin: coin.to_string() });

    println!("🔴 Streaming {coin} trades (Ctrl+C to stop)...\n");

    match fmt {
        OutputFormat::Table => {
            println!("{:<20} {:>6} {:>14} {:>14} {:>10}", "TIME", "SIDE", "PRICE", "SIZE", "HASH");
            println!("{}", "─".repeat(68));
        }
        _ => {}
    }

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::Trades(trades)) = event {
            for trade in &trades {
                match fmt {
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(trade).unwrap_or_default());
                    }
                    OutputFormat::JsonPretty => {
                        println!("{}", serde_json::to_string_pretty(trade).unwrap_or_default());
                    }
                    OutputFormat::Table => {
                        let time = format_ts(trade.time);
                        let side = &trade.side;
                        println!(
                            "{:<20} {:>6} {:>14} {:>14} {:>10}",
                            time, side, trade.px, trade.sz, &trade.hash[..10]
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// `atlas stream book <COIN>` — live order book
pub async fn stream_book(coin: &str, depth: usize, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let _ = engine;

    let core = hypercore::mainnet();
    let mut ws = core.websocket();
    ws.subscribe(Subscription::L2Book { coin: coin.to_string() });

    println!("🔴 Streaming {coin} order book (Ctrl+C to stop)...\n");

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::L2Book(book)) = event {
            match fmt {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string(&book).unwrap_or_default());
                }
                OutputFormat::JsonPretty => {
                    println!("{}", serde_json::to_string_pretty(&book).unwrap_or_default());
                }
                OutputFormat::Table => {
                    // Clear and reprint
                    print!("\x1B[2J\x1B[H"); // clear screen
                    println!("📖 {} Order Book\n", book.coin);
                    let bids = book.bids();
                    let asks = book.asks();
                    let show = depth.min(bids.len()).min(asks.len());

                    println!("{:>14} {:>14}  |  {:>14} {:>14}", "BID SIZE", "BID PRICE", "ASK PRICE", "ASK SIZE");
                    println!("{}", "─".repeat(65));

                    for i in 0..show {
                        let bid = &bids[i];
                        let ask = &asks[i];
                        println!(
                            "{:>14} {:>14}  |  {:>14} {:>14}",
                            bid.sz, bid.px, ask.px, ask.sz
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// `atlas stream candles <COIN> <INTERVAL>` — live candle updates
pub async fn stream_candles(coin: &str, interval: &str, fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let _ = engine;

    let core = hypercore::mainnet();
    let mut ws = core.websocket();
    ws.subscribe(Subscription::Candle {
        coin: coin.to_string(),
        interval: interval.to_string(),
    });

    println!("🔴 Streaming {coin} {interval} candles (Ctrl+C to stop)...\n");

    match fmt {
        OutputFormat::Table => {
            println!(
                "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
                "TIME", "OPEN", "HIGH", "LOW", "CLOSE", "VOLUME"
            );
            println!("{}", "─".repeat(84));
        }
        _ => {}
    }

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::Candle(candle)) = event {
            match fmt {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string(&candle).unwrap_or_default());
                }
                OutputFormat::JsonPretty => {
                    println!("{}", serde_json::to_string_pretty(&candle).unwrap_or_default());
                }
                OutputFormat::Table => {
                    let time = format_ts(candle.open_time);
                    println!(
                        "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
                        time, candle.open, candle.high, candle.low, candle.close, candle.volume
                    );
                }
            }
        }
    }

    Ok(())
}

/// `atlas stream user` — live user events (fills, orders, liquidations)
pub async fn stream_user(fmt: OutputFormat) -> Result<()> {
    let engine = Engine::from_active_profile().await?;
    let address = engine.address;

    let core = hypercore::mainnet();
    let mut ws = core.websocket();

    // Subscribe to user fills and order updates
    ws.subscribe(Subscription::UserFills { user: address });
    ws.subscribe(Subscription::OrderUpdates { user: address });
    ws.subscribe(Subscription::UserEvents { user: address });

    println!("🔴 Streaming user events for {} (Ctrl+C to stop)...\n", address);

    while let Some(event) = ws.next().await {
        match event {
            Event::Message(Incoming::UserFills { user: _, fills }) => {
                for fill in &fills {
                    match fmt {
                        OutputFormat::Json | OutputFormat::JsonPretty => {
                            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                                serde_json::to_string_pretty(fill)
                            } else {
                                serde_json::to_string(fill)
                            };
                            println!("{}", json.unwrap_or_default());
                        }
                        OutputFormat::Table => {
                            println!(
                                "📝 FILL: {} {} {} @ {} (fee: {})",
                                fill.coin, fill.side, fill.sz, fill.px, fill.fee
                            );
                        }
                    }
                }
            }
            Event::Message(Incoming::OrderUpdates(updates)) => {
                for update in &updates {
                    match fmt {
                        OutputFormat::Json | OutputFormat::JsonPretty => {
                            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                                serde_json::to_string_pretty(update)
                            } else {
                                serde_json::to_string(update)
                            };
                            println!("{}", json.unwrap_or_default());
                        }
                        OutputFormat::Table => {
                            println!(
                                "📋 ORDER: {} {:?} {} {} @ {}",
                                update.order.coin,
                                update.status,
                                update.order.side,
                                update.order.sz,
                                update.order.limit_px,
                            );
                        }
                    }
                }
            }
            Event::Message(Incoming::UserEvents(user_event)) => {
                match fmt {
                    OutputFormat::Json | OutputFormat::JsonPretty => {
                        let json = if matches!(fmt, OutputFormat::JsonPretty) {
                            serde_json::to_string_pretty(&user_event)
                        } else {
                            serde_json::to_string(&user_event)
                        };
                        println!("{}", json.unwrap_or_default());
                    }
                    OutputFormat::Table => {
                        println!("⚡ USER EVENT: {:?}", user_event);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn render_mids_update(mids: &HashMap<String, Decimal>, fmt: OutputFormat) {
    match fmt {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(mids).unwrap_or_default());
        }
        OutputFormat::JsonPretty => {
            println!("{}", serde_json::to_string_pretty(mids).unwrap_or_default());
        }
        OutputFormat::Table => {
            // Clear screen and show all prices
            print!("\x1B[2J\x1B[H");
            println!("💹 Live Mid Prices\n");
            println!("{:<12} {:>15}", "COIN", "PRICE");
            println!("{}", "─".repeat(28));

            let mut sorted: Vec<_> = mids.iter().collect();
            sorted.sort_by_key(|(k, _)| (*k).clone());

            for (coin, price) in &sorted {
                println!("{:<12} {:>15}", coin, price);
            }
        }
    }
}

/// Format a millisecond timestamp to UTC string.
fn format_ts(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let total_days = secs / 86400;
    let day_secs = (secs % 86400) as u32;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    let z = total_days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02}")
}
