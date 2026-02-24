use anyhow::Result;
use atlas_core::Engine;
use atlas_types::output::StatusOutput;
use atlas_utils::output::{render, OutputFormat, TableDisplay};

/// `atlas status` — fast textual summary, no TUI.
pub async fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    // For JSON mode, we need connection data — skip the pre-connection header
    if fmt != OutputFormat::Table {
        match Engine::from_active_profile().await {
            Ok(engine) => {
                let output = engine.get_account_summary().await?;
                render(fmt, &output)?;
            }
            Err(e) => {
                // Even in JSON mode, output a structured error
                let output = StatusOutput {
                    profile: config.general.active_profile.clone(),
                    address: "unknown".into(),
                    network: if config.network.testnet { "Testnet".into() } else { "Mainnet".into() },
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

    // Table mode — show connection header first
    println!("┌─────────────────────────────────────────────┐");
    println!("│  ATLAS STATUS                               │");
    println!("├─────────────────────────────────────────────┤");
    println!("│  Active Profile : {:<26}│", config.general.active_profile);
    println!(
        "│  Network        : {:<26}│",
        if config.network.testnet {
            "Testnet"
        } else {
            "Mainnet"
        }
    );
    println!("│  RPC            : {:<26}│", config.network.rpc_url);
    println!("├─────────────────────────────────────────────┤");

    // Attempt to connect and fetch balance.
    match Engine::from_active_profile().await {
        Ok(engine) => {
            println!("│  Connection     : ✓ OK                      │");
            println!("└─────────────────────────────────────────────┘");
            println!();
            let output = engine.get_account_summary().await?;
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
