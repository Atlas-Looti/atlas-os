//! `atlas morpho` commands — Morpho Blue lending protocol.

use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_utils::output::OutputFormat;

/// List Morpho Blue lending markets.
pub async fn markets(chain: &str, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::readonly().await?;
    let lending = orch.lending(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let lending_markets = lending.markets().await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if lending_markets.is_empty() {
        println!("No Morpho markets found.");
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&lending_markets)?
            } else {
                serde_json::to_string(&lending_markets)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("Morpho Blue — {} markets (chain: {})\n", lending_markets.len(), chain);
            println!("┌──────────────────────────┬──────────┬──────────┬───────────────┬───────────────┐");
            println!("│ Market                   │ Sup. APY │ Bor. APY │ Total Supply  │ Utilization   │");
            println!("├──────────────────────────┼──────────┼──────────┼───────────────┼───────────────┤");
            for m in &lending_markets {
                let name = format!("{}/{}", m.collateral_asset, m.loan_asset);
                println!(
                    "│ {:<24} │ {:>7.2}% │ {:>7.2}% │ ${:>11.0} │ {:>11.2}% │",
                    name,
                    m.supply_apy * rust_decimal::Decimal::from(100),
                    m.borrow_apy * rust_decimal::Decimal::from(100),
                    m.total_supply,
                    m.utilization * rust_decimal::Decimal::from(100),
                );
            }
            println!("└──────────────────────────┴──────────┴──────────┴───────────────┴───────────────┘");
        }
    }

    Ok(())
}

/// Show user's Morpho Blue lending positions.
pub async fn positions(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    // Get active wallet address
    let store = atlas_core::auth::AuthManager::load_store_pub()?;
    let profile = store.wallets.iter()
        .find(|w| w.name == config.system.active_profile)
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run: atlas profile generate <name>"))?;

    let orch = Orchestrator::readonly().await?;
    let lending = orch.lending(None).map_err(|e| anyhow::anyhow!("{e}"))?;

    let user_positions = lending.positions(&profile.address).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if user_positions.is_empty() {
        println!("No Morpho positions found for {}.", profile.address);
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&user_positions)?
            } else {
                serde_json::to_string(&user_positions)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("┌──────────────────────────┬───────────────┬───────────────┬──────────────┐");
            println!("│ Market                   │ Supplied      │ Borrowed      │ Health       │");
            println!("├──────────────────────────┼───────────────┼───────────────┼──────────────┤");
            for p in &user_positions {
                let name = format!("{}/{}", p.collateral_asset, p.loan_asset);
                let health = p.health_factor
                    .map(|h| format!("{:.2}", h))
                    .unwrap_or_else(|| "—".into());
                println!(
                    "│ {:<24} │ ${:>11.2} │ ${:>11.2} │ {:>12} │",
                    name, p.supplied, p.borrowed, health,
                );
            }
            println!("└──────────────────────────┴───────────────┴───────────────┴──────────────┘");
        }
    }

    Ok(())
}
