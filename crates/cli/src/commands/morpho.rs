//! `atlas morpho` commands — Morpho Blue lending protocol.

use anyhow::Result;
use atlas_common::types::Chain;
use atlas_mod_morpho::client::MorphoModule;
use atlas_utils::output::OutputFormat;

/// List Morpho Blue lending markets.
pub async fn markets(chain: &str, fmt: OutputFormat) -> Result<()> {
    let chain_enum = match chain {
        "base" => Chain::Base,
        _ => Chain::Ethereum,
    };

    let module = MorphoModule::new(chain_enum);
    let lending_markets = module.markets_data().await?;

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
    let chain_str = &config.modules.morpho.config.chain;
    let chain_enum = match chain_str.as_str() {
        "base" => Chain::Base,
        _ => Chain::Ethereum,
    };

    // Get active wallet address
    let store = atlas_core::auth::AuthManager::load_store_pub()?;
    let profile = store.wallets.iter()
        .find(|w| w.name == config.system.active_profile)
        .ok_or_else(|| anyhow::anyhow!("No active profile. Run: atlas auth new <name>"))?;

    let module = MorphoModule::new(chain_enum);
    let user_positions = module.positions_data(&profile.address).await?;

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
