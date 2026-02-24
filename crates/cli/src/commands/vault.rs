use anyhow::Result;
use atlas_core::output::OutputFormat;

/// `atlas vault details <VAULT_ADDRESS>`
pub async fn vault_details(vault: &str, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let details = perp
        .vault_details(vault)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&details)?
            } else {
                serde_json::to_string(&details)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("╔══════════════════════════════════════════════════════════════╗");
            println!("║  VAULT: {:<51} ║", details.name);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║  Address:   {:<48} ║", details.address);
            println!("║  Leader:    {:<48} ║", details.leader);
            println!("║  Portfolio: ${:<47} ║", details.portfolio_value);
            println!("║  Followers: {:<48} ║", details.followers);
            if let Some(apr) = details.apr {
                println!(
                    "║  APR:       {:<48} ║",
                    format!("{}%", apr * rust_decimal::Decimal::ONE_HUNDRED)
                );
            }
            if let Some(pnl) = details.pnl_all_time {
                println!("║  All-time PnL: ${:<44} ║", pnl);
            }
            println!("╚══════════════════════════════════════════════════════════════╝");
        }
    }
    Ok(())
}

/// `atlas vault deposits`
pub async fn vault_deposits(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let deposits = perp
        .vault_deposits()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if deposits.is_empty() {
        println!("No vault deposits found.");
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&deposits)?
            } else {
                serde_json::to_string(&deposits)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("┌──────────────────────────────────────────────┬───────────────┐");
            println!("│ Vault                                        │ Equity        │");
            println!("├──────────────────────────────────────────────┼───────────────┤");
            for d in &deposits {
                println!("│ {:<44} │ ${:>11} │", d.vault_address, d.equity);
            }
            println!("└──────────────────────────────────────────────┴───────────────┘");
        }
    }
    Ok(())
}
