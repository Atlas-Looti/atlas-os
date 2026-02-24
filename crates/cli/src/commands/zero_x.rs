//! `atlas zero-x` commands — 0x DEX aggregator (multi-chain swaps).

use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_common::types::Chain;
use atlas_utils::output::OutputFormat;

/// Parse chain string to Chain enum.
fn parse_chain(chain: &str) -> Result<Chain> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" | "1" => Ok(Chain::Ethereum),
        "arbitrum" | "arb" | "42161" => Ok(Chain::Arbitrum),
        "base" | "8453" => Ok(Chain::Base),
        _ => anyhow::bail!(
            "Unsupported chain: {chain}. Supported: ethereum, arbitrum, base"
        ),
    }
}

/// `atlas zero-x quote <sell_token> <buy_token> <amount> [--chain ethereum]`
pub async fn quote(
    sell_token: &str,
    buy_token: &str,
    amount: &str,
    chain: &str,
    slippage_bps: Option<u32>,
    fmt: OutputFormat,
) -> Result<()> {
    let chain_enum = parse_chain(chain)?;
    let orch = Orchestrator::readonly().await?;
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Use the 0x module directly for chain-aware price
    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_mod_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    let resp = zerox
        .price(&chain_enum, sell_token, buy_token, amount, None, slippage_bps)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !resp.liquidity_available {
        println!("⚠️  No liquidity available for this pair on {chain}");
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "chain": chain,
                "sell_token": sell_token,
                "buy_token": buy_token,
                "sell_amount": resp.sell_amount,
                "buy_amount": resp.buy_amount,
                "min_buy_amount": resp.min_buy_amount,
                "gas_price": resp.gas_price,
                "allowance_target": resp.allowance_target,
                "route": resp.route,
                "fees": resp.fees,
                "issues": resp.issues,
                "liquidity_available": resp.liquidity_available,
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            let sell_amt = resp.sell_amount.as_deref().unwrap_or("—");
            let buy_amt = resp.buy_amount.as_deref().unwrap_or("—");
            let min_buy = resp.min_buy_amount.as_deref().unwrap_or("—");

            println!("┌─────────────────────────────────────────────────┐");
            println!("│  0x SWAP QUOTE                                  │");
            println!("├─────────────────────────────────────────────────┤");
            println!("│  Chain         : {:<30} │", chain);
            println!("│  Sell Token    : {:<30} │", &sell_token[..sell_token.len().min(30)]);
            println!("│  Buy Token     : {:<30} │", &buy_token[..buy_token.len().min(30)]);
            println!("│  Sell Amount   : {:<30} │", sell_amt);
            println!("│  Buy Amount    : {:<30} │", buy_amt);
            println!("│  Min Buy Amt   : {:<30} │", min_buy);
            println!("├─────────────────────────────────────────────────┤");

            // Show route
            if let Some(route) = &resp.route {
                let sources: Vec<String> = route
                    .fills
                    .iter()
                    .map(|f| format!("{} ({}%)", f.source, f.proportion_bps.parse::<f64>().unwrap_or(0.0) / 100.0))
                    .collect();
                println!("│  Route         : {:<30} │", sources.join(", ").chars().take(30).collect::<String>());
            }

            // Show issues
            if let Some(issues) = &resp.issues {
                if let Some(allowance) = &issues.allowance {
                    println!("│  ⚠ Allowance   : set on {:<23} │", &allowance.spender[..allowance.spender.len().min(23)]);
                }
                if let Some(balance) = &issues.balance {
                    println!("│  ⚠ Balance     : need {:<25} │", &balance.expected);
                }
            }

            println!("└─────────────────────────────────────────────────┘");
        }
    }

    Ok(())
}

/// `atlas zero-x chains` — list supported chains.
pub async fn chains(fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::readonly().await?;
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_mod_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    let resp = zerox
        .supported_chains()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&resp)?
            } else {
                serde_json::to_string(&resp)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("{:<12} NAME", "CHAIN ID");
            println!("{}", "─".repeat(35));
            for c in &resp.chains {
                println!("{:<12} {}", c.chain_id, c.chain_name);
            }
            println!("\nTotal: {} chains", resp.chains.len());
        }
    }

    Ok(())
}

/// `atlas zero-x sources [--chain ethereum]` — list liquidity sources.
pub async fn sources(chain: &str, fmt: OutputFormat) -> Result<()> {
    let chain_enum = parse_chain(chain)?;
    let orch = Orchestrator::readonly().await?;
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_mod_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    let resp = zerox
        .sources(&chain_enum)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&resp)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&resp)?),
        OutputFormat::Table => {
            println!("Liquidity sources for {chain}:\n");
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }

    Ok(())
}

/// `atlas zero-x trades [--start <ts>] [--end <ts>]` — trade analytics.
pub async fn trades(
    start: Option<u64>,
    end: Option<u64>,
    fmt: OutputFormat,
) -> Result<()> {
    let orch = Orchestrator::readonly().await?;
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_mod_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    let resp = zerox
        .swap_trades(None, start, end)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&resp)?
            } else {
                serde_json::to_string(&resp)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            if resp.trades.is_empty() {
                println!("No completed swap trades found.");
                return Ok(());
            }
            println!("{:<12} {:<10} {:<44} {:<15} APP", "CHAIN", "VOLUME", "TX HASH", "TAKER");
            println!("{}", "─".repeat(95));
            for t in &resp.trades {
                let vol = t.volume_usd.as_deref().unwrap_or("—");
                let taker_short = if t.taker.len() > 12 {
                    format!("{}..{}", &t.taker[..6], &t.taker[t.taker.len()-4..])
                } else {
                    t.taker.clone()
                };
                println!(
                    "{:<12} ${:<9} {:<44} {:<15} {}",
                    t.chain_name, vol, t.transaction_hash, taker_short, t.app_name
                );
            }
            println!("\nTotal: {} trades", resp.trades.len());
        }
    }

    Ok(())
}
