use anyhow::Result;
use futures::StreamExt;
use hypersdk::hypercore::{
    self as hypercore,
    types::{Incoming, Subscription},
    ws::Event,
};
use rust_decimal::Decimal;
use std::collections::HashMap;

use atlas_core::fmt::format_timestamp_ms;
use atlas_core::output::OutputFormat;
use atlas_core::workspace::load_config;
use atlas_core::AuthManager;

/// Build HL websocket client from config (no Engine needed).
fn build_ws_client(testnet: bool) -> hypersdk::hypercore::HttpClient {
    if testnet {
        hypercore::testnet()
    } else {
        hypercore::mainnet()
    }
}

/// `atlas stream prices` — live mid prices for all markets
pub async fn stream_prices(fmt: OutputFormat) -> Result<()> {
    let config = load_config()?;
    let testnet = config.modules.hyperliquid.config.network == "testnet";
    let core = build_ws_client(testnet);

    eprintln!("🔴 Streaming all mid prices (Ctrl+C to stop)...\n");

    let mut ws = core.websocket();
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
    let config = load_config()?;
    let testnet = config.modules.hyperliquid.config.network == "testnet";
    let core = build_ws_client(testnet);

    let mut ws = core.websocket();
    ws.subscribe(Subscription::Trades {
        coin: coin.to_string(),
    });

    eprintln!("🔴 Streaming {coin} trades (Ctrl+C to stop)...\n");

    if fmt == OutputFormat::Table {
        println!(
            "{:<20} {:>6} {:>14} {:>14} {:>10}",
            "TIME", "SIDE", "PRICE", "SIZE", "HASH"
        );
        println!("{}", "─".repeat(68));
    }

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::Trades(trades)) = event {
            for trade in &trades {
                match fmt {
                    OutputFormat::Json => {
                        println!("{}", serde_json::to_string(trade).unwrap_or_default())
                    }
                    OutputFormat::JsonPretty => println!(
                        "{}",
                        serde_json::to_string_pretty(trade).unwrap_or_default()
                    ),
                    OutputFormat::Table => {
                        let time = format_timestamp_ms(trade.time);
                        println!(
                            "{:<20} {:>6} {:>14} {:>14} {:>10}",
                            time,
                            trade.side,
                            trade.px,
                            trade.sz,
                            &trade.hash[..10]
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
    let config = load_config()?;
    let testnet = config.modules.hyperliquid.config.network == "testnet";
    let core = build_ws_client(testnet);

    let mut ws = core.websocket();
    ws.subscribe(Subscription::L2Book {
        coin: coin.to_string(),
    });

    eprintln!("🔴 Streaming {coin} order book (Ctrl+C to stop)...\n");

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::L2Book(book)) = event {
            match fmt {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string(&book).unwrap_or_default())
                }
                OutputFormat::JsonPretty => println!(
                    "{}",
                    serde_json::to_string_pretty(&book).unwrap_or_default()
                ),
                OutputFormat::Table => {
                    print!("\x1B[2J\x1B[H");
                    println!("📖 {} Order Book\n", book.coin);
                    let bids = book.bids();
                    let asks = book.asks();
                    let show = depth.min(bids.len()).min(asks.len());
                    println!(
                        "{:>14} {:>14}  |  {:>14} {:>14}",
                        "BID SIZE", "BID PRICE", "ASK PRICE", "ASK SIZE"
                    );
                    println!("{}", "─".repeat(65));
                    for i in 0..show {
                        println!(
                            "{:>14} {:>14}  |  {:>14} {:>14}",
                            bids[i].sz, bids[i].px, asks[i].px, asks[i].sz
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
    let config = load_config()?;
    let testnet = config.modules.hyperliquid.config.network == "testnet";
    let core = build_ws_client(testnet);

    let mut ws = core.websocket();
    ws.subscribe(Subscription::Candle {
        coin: coin.to_string(),
        interval: interval.to_string(),
    });

    eprintln!("🔴 Streaming {coin} {interval} candles (Ctrl+C to stop)...\n");

    if fmt == OutputFormat::Table {
        println!(
            "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "TIME", "OPEN", "HIGH", "LOW", "CLOSE", "VOLUME"
        );
        println!("{}", "─".repeat(84));
    }

    while let Some(event) = ws.next().await {
        if let Event::Message(Incoming::Candle(candle)) = event {
            match fmt {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string(&candle).unwrap_or_default())
                }
                OutputFormat::JsonPretty => println!(
                    "{}",
                    serde_json::to_string_pretty(&candle).unwrap_or_default()
                ),
                OutputFormat::Table => {
                    let time = format_timestamp_ms(candle.open_time);
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
    let config = load_config()?;
    let testnet = config.modules.hyperliquid.config.network == "testnet";
    let core = build_ws_client(testnet);
    let signer = AuthManager::get_active_signer()?;
    let address = alloy::signers::local::PrivateKeySigner::address(&signer);

    let mut ws = core.websocket();
    ws.subscribe(Subscription::UserFills { user: address });
    ws.subscribe(Subscription::OrderUpdates { user: address });

    eprintln!(
        "🔴 Streaming user events for {} (Ctrl+C to stop)...\n",
        address
    );

    while let Some(event) = ws.next().await {
        match event {
            Event::Message(Incoming::UserFills { user: _, fills }) => {
                for fill in &fills {
                    match fmt {
                        OutputFormat::Json | OutputFormat::JsonPretty => {
                            // PRD canonical NDJSON event format
                            let canonical = serde_json::json!({
                                "event": "fill",
                                "order_id": fill.oid,
                                "symbol": fill.coin,
                                "side": format!("{:?}", fill.side).to_lowercase(),
                                "size": fill.sz,
                                "price": fill.px,
                                "fee": fill.fee,
                                "timestamp": fill.time,
                            });
                            // NDJSON: one JSON per line, no wrapper envelope
                            println!("{}", serde_json::to_string(&canonical).unwrap_or_default());
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
                            // PRD canonical NDJSON event format
                            let event_type =
                                match format!("{:?}", update.status).to_lowercase().as_str() {
                                    s if s.contains("cancel") => "order_cancelled",
                                    s if s.contains("fill") => "order_filled",
                                    _ => "order_update",
                                };
                            let canonical = serde_json::json!({
                                "event": event_type,
                                "order_id": update.order.oid,
                                "symbol": update.order.coin,
                                "side": format!("{:?}", update.order.side).to_lowercase(),
                                "size": update.order.sz,
                                "price": update.order.limit_px,
                                "status": format!("{:?}", update.status).to_lowercase(),
                                "timestamp": update.order.timestamp,
                            });
                            println!("{}", serde_json::to_string(&canonical).unwrap_or_default());
                        }
                        OutputFormat::Table => {
                            println!(
                                "📋 ORDER: {} {:?} {} {} @ {}",
                                update.order.coin,
                                update.status,
                                update.order.side,
                                update.order.sz,
                                update.order.limit_px
                            );
                        }
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
        OutputFormat::Json => println!("{}", serde_json::to_string(mids).unwrap_or_default()),
        OutputFormat::JsonPretty => {
            println!("{}", serde_json::to_string_pretty(mids).unwrap_or_default())
        }
        OutputFormat::Table => {
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
