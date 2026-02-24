use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_types::output::{StatusOutput, PositionRow};
use atlas_utils::output::{render, OutputFormat, TableDisplay};

/// `atlas status` — fast textual summary, no TUI.
pub async fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    if fmt != OutputFormat::Table {
        let orch_res = Orchestrator::from_active_profile().await;
        match orch_res {
            Ok(orch) => {
                let perp = orch.perp(None).map_err(|e| anyhow::anyhow!("{e}"))?;
                let balances = perp.balances().await.map_err(|e| anyhow::anyhow!("{e}"))?;
                let positions = perp.positions().await.map_err(|e| anyhow::anyhow!("{e}"))?;
                let bal = balances.first();

                let pos_rows: Vec<PositionRow> = positions.iter().map(|p| PositionRow {
                    coin: p.symbol.clone(),
                    size: p.size.to_string(),
                    entry_price: p.entry_price.map(|e| e.to_string()).unwrap_or_else(|| "—".into()),
                    unrealized_pnl: p.unrealized_pnl.map(|u| u.to_string()).unwrap_or_else(|| "—".into()),
                }).collect();

                let output = StatusOutput {
                    profile: config.system.active_profile.clone(),
                    address: config.system.active_profile.clone(),
                    network: if config.modules.hyperliquid.config.network == "testnet" { "Testnet".to_string() } else { "Mainnet".to_string() },
                    account_value: bal.map(|b| b.total.to_string()).unwrap_or_else(|| "—".into()),
                    margin_used: bal.map(|b| b.locked.to_string()).unwrap_or_else(|| "—".into()),
                    net_position: "—".into(),
                    withdrawable: bal.map(|b| b.available.to_string()).unwrap_or_else(|| "—".into()),
                    positions: pos_rows,
                };
                render(fmt, &output)?;
            }
            Err(e) => {
                let output = StatusOutput {
                    profile: config.system.active_profile.clone(),
                    address: "unknown".into(),
                    network: if config.modules.hyperliquid.config.network == "testnet" { "Testnet".into() } else { "Mainnet".into() },
                    account_value: "—".into(),
                    margin_used: "—".into(),
                    net_position: "—".into(),
                    withdrawable: "—".into(),
                    positions: vec![],
                };
                render(fmt, &output)?;
                eprintln!("Warning: connection failed: {e:#}");
            }
        }
        return Ok(());
    }

    // Table mode
    println!("┌─────────────────────────────────────────────┐");
    println!("│  ATLAS STATUS                               │");
    println!("├─────────────────────────────────────────────┤");
    println!("│  Active Profile : {:<26}│", config.system.active_profile);
    println!("│  Network        : {:<26}│",
        if config.modules.hyperliquid.config.network == "testnet" { "Testnet" } else { "Mainnet" }
    );
    println!("│  RPC            : {:<26}│", config.modules.hyperliquid.config.rpc_url);
    println!("├─────────────────────────────────────────────┤");

    match Orchestrator::from_active_profile().await {
        Ok(orch) => {
            let perp = orch.perp(None).map_err(|e| anyhow::anyhow!("{e}"))?;
            let balances = perp.balances().await.map_err(|e| anyhow::anyhow!("{e}"))?;
            let positions = perp.positions().await.map_err(|e| anyhow::anyhow!("{e}"))?;
            let bal = balances.first();

            let pos_rows: Vec<PositionRow> = positions.iter().map(|p| PositionRow {
                coin: p.symbol.clone(),
                size: p.size.to_string(),
                entry_price: p.entry_price.map(|e| e.to_string()).unwrap_or_else(|| "—".into()),
                unrealized_pnl: p.unrealized_pnl.map(|u| u.to_string()).unwrap_or_else(|| "—".into()),
            }).collect();

            let output = StatusOutput {
                profile: config.system.active_profile.clone(),
                address: config.system.active_profile.clone(),
                network: if config.modules.hyperliquid.config.network == "testnet" { "Testnet".to_string() } else { "Mainnet".to_string() },
                account_value: bal.map(|b| b.total.to_string()).unwrap_or_else(|| "—".into()),
                margin_used: bal.map(|b| b.locked.to_string()).unwrap_or_else(|| "—".into()),
                net_position: "—".into(),
                withdrawable: bal.map(|b| b.available.to_string()).unwrap_or_else(|| "—".into()),
                positions: pos_rows,
            };

            println!("│  Connection     : ✓ OK                      │");
            println!("└─────────────────────────────────────────────┘");
            println!();
            output.print_table();
        }
        Err(e) => {
            println!("│  Connection     : ✗ FAILED                  │");
            println!("│  Error          : {:<26}│", format!("{e:#}").chars().take(26).collect::<String>());
            println!("└─────────────────────────────────────────────┘");
            println!();
            println!("Hint: Run `atlas auth list` to check profiles, or `atlas doctor` to diagnose.");
        }
    }

    Ok(())
}
