//! `atlas zero-x` commands — 0x DEX aggregator (multi-chain swaps).

use anyhow::Result;
use atlas_core::output::OutputFormat;
use atlas_core::types::Chain;

/// Parse chain string to Chain enum.
fn parse_chain(chain: &str) -> Result<Chain> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" | "1" => Ok(Chain::Ethereum),
        "arbitrum" | "arb" | "42161" => Ok(Chain::Arbitrum),
        "base" | "8453" => Ok(Chain::Base),
        _ => anyhow::bail!("Unsupported chain: {chain}. Supported: ethereum, arbitrum, base"),
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
    // Try to load signer for taker address (better price simulation)
    let orch = match crate::factory::from_active_profile().await {
        Ok(o) => o,
        Err(_) => crate::factory::readonly().await?,
    };
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    // Use the 0x module directly for chain-aware price
    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    let taker = zerox.taker_address();
    let resp = zerox
        .price(
            &chain_enum,
            sell_token,
            buy_token,
            amount,
            taker.as_deref(),
            slippage_bps,
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !resp.liquidity_available {
        println!("⚠️  No liquidity available for this pair on {chain}");
        return Ok(());
    }

    let allowance_required = resp
        .issues
        .as_ref()
        .and_then(|i| i.allowance.as_ref())
        .is_some();
    let allowance_spender = resp
        .issues
        .as_ref()
        .and_then(|i| i.allowance.as_ref())
        .map(|a| a.spender.as_str());

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "ok": true,
                "data": {
                    "chain": chain,
                    "sell_token": sell_token,
                    "buy_token": buy_token,
                    "sell_amount": resp.sell_amount,
                    "buy_amount": resp.buy_amount,
                    "min_buy_amount": resp.min_buy_amount,
                    "gas_price": resp.gas_price,
                    "allowance_target": resp.allowance_target,
                    "allowance_required": allowance_required,
                    "allowance_spender": allowance_spender,
                    "route": resp.route,
                    "fees": resp.fees,
                    "issues": resp.issues,
                    "liquidity_available": resp.liquidity_available,
                }
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
            println!(
                "│  Sell Token    : {:<30} │",
                &sell_token[..sell_token.len().min(30)]
            );
            println!(
                "│  Buy Token     : {:<30} │",
                &buy_token[..buy_token.len().min(30)]
            );
            println!("│  Sell Amount   : {:<30} │", sell_amt);
            println!("│  Buy Amount    : {:<30} │", buy_amt);
            println!("│  Min Buy Amt   : {:<30} │", min_buy);
            println!("├─────────────────────────────────────────────────┤");

            // Show route
            if let Some(route) = &resp.route {
                let sources: Vec<String> = route
                    .fills
                    .iter()
                    .map(|f| {
                        format!(
                            "{} ({}%)",
                            f.source,
                            f.proportion_bps.parse::<f64>().unwrap_or(0.0) / 100.0
                        )
                    })
                    .collect();
                println!(
                    "│  Route         : {:<30} │",
                    sources.join(", ").chars().take(30).collect::<String>()
                );
            }

            // Show issues
            if let Some(issues) = &resp.issues {
                if let Some(allowance) = &issues.allowance {
                    println!(
                        "│  ⚠ Allowance   : set on {:<23} │",
                        &allowance.spender[..allowance.spender.len().min(23)]
                    );
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
    let orch = crate::factory::readonly().await?;
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_zero_x::ZeroXModule>()
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
    let orch = crate::factory::readonly().await?;
    let swap = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let zerox = swap
        .as_any()
        .downcast_ref::<atlas_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    let resp = zerox.sources(&chain_enum).await.map_err(|e| {
        let msg = e.to_string();
        if msg.contains("404") || msg.contains("Not Found") {
            anyhow::anyhow!(
                "Backend does not implement liquidity sources yet. Use `atlas zero-x chains` for supported chains."
            )
        } else {
            anyhow::anyhow!("{e}")
        }
    })?;

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

/// `atlas zero-x swap <sell_token> <buy_token> <amount> [--chain ethereum] [--yes]`
pub async fn swap(
    sell_token: &str,
    buy_token: &str,
    amount: &str,
    chain: &str,
    slippage_bps: Option<u32>,
    skip_confirm: bool,
    fmt: OutputFormat,
) -> Result<()> {
    let chain_enum = parse_chain(chain)?;
    let orch = crate::factory::from_active_profile().await?;
    let swap_mod = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let zerox = swap_mod
        .as_any()
        .downcast_ref::<atlas_zero_x::ZeroXModule>()
        .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

    // 1. Get indicative price first
    let taker = zerox
        .taker_address()
        .ok_or_else(|| anyhow::anyhow!("No wallet loaded. Run: atlas profile import"))?;

    let slippage = slippage_bps.unwrap_or(zerox.default_slippage_bps);

    println!("⏳ Getting swap quote...");
    let price_resp = zerox
        .price(
            &chain_enum,
            sell_token,
            buy_token,
            amount,
            Some(&taker),
            Some(slippage),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !price_resp.liquidity_available {
        anyhow::bail!("No liquidity available for this pair on {chain}");
    }

    let sell_amt = price_resp.sell_amount.as_deref().unwrap_or(amount);
    let buy_amt = price_resp.buy_amount.as_deref().unwrap_or("?");
    let min_buy = price_resp.min_buy_amount.as_deref().unwrap_or("?");

    // 2. Show quote and confirm
    if !skip_confirm {
        println!("┌─────────────────────────────────────────────────┐");
        println!("│  0x SWAP — CONFIRM EXECUTION                    │");
        println!("├─────────────────────────────────────────────────┤");
        println!("│  Chain         : {:<30} │", chain);
        println!(
            "│  Sell          : {:<30} │",
            &sell_token[..sell_token.len().min(30)]
        );
        println!(
            "│  Buy           : {:<30} │",
            &buy_token[..buy_token.len().min(30)]
        );
        println!("│  Sell Amount   : {:<30} │", sell_amt);
        println!("│  Buy Amount    : {:<30} │", buy_amt);
        println!("│  Min Buy (slip): {:<30} │", min_buy);
        println!("│  Slippage      : {:<30} │", format!("{} bps", slippage));
        println!("│  Taker         : {:<30} │", &taker[..taker.len().min(30)]);
        println!("└─────────────────────────────────────────────────┘");

        // Show issues
        if let Some(ref issues) = price_resp.issues {
            if let Some(ref allowance) = issues.allowance {
                println!(
                    "  ⚠ Token approval needed (spender: {})",
                    &allowance.spender[..allowance.spender.len().min(42)]
                );
            }
            if let Some(ref balance) = issues.balance {
                println!(
                    "  ⚠ Insufficient balance (need: {}, have: {})",
                    balance.expected, balance.actual
                );
            }
        }

        print!("\nExecute this swap? (y/N): ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Swap cancelled.");
            return Ok(());
        }
    }

    // 3. Execute the swap via SwapModule trait
    println!("⏳ Executing swap on-chain...");

    let sell_dec: rust_decimal::Decimal = amount
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount: {amount}"))?;

    // Build a SwapQuote for the trait's swap() method
    let quote = atlas_core::types::SwapQuote {
        protocol: atlas_core::types::Protocol::ZeroX,
        chain: chain_enum,
        sell_token: sell_token.to_string(),
        buy_token: buy_token.to_string(),
        sell_amount: sell_dec,
        buy_amount: buy_amt.parse().unwrap_or(rust_decimal::Decimal::ZERO),
        estimated_gas: None,
        price: rust_decimal::Decimal::ZERO,
        allowance_target: price_resp.allowance_target.clone(),
        tx_data: None, // swap() gets its own firm quote internally
    };

    let tx_hash = swap_mod
        .swap(&quote)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // 4. Output result
    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "ok": true,
                "data": {
                    "tx_hash": tx_hash,
                    "chain": chain,
                    "sell_token": sell_token,
                    "buy_token": buy_token,
                    "sell_amount": sell_amt,
                    "buy_amount": buy_amt,
                    "status": "confirmed"
                }
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("✅ Swap executed successfully!");
            println!("   TX Hash: {tx_hash}");
            println!("   Chain: {chain}");
            println!(
                "   Sold: {sell_amt} of {}",
                &sell_token[..sell_token.len().min(20)]
            );
            println!(
                "   Bought: ~{buy_amt} of {}",
                &buy_token[..buy_token.len().min(20)]
            );
        }
    }

    Ok(())
}
