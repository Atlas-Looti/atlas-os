use anyhow::Result;
use atlas_core::Orchestrator;
use atlas_utils::output::OutputFormat;

/// `atlas sub list`
pub async fn sub_list(fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let subs = perp.subaccounts().await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if subs.is_empty() {
        println!("No subaccounts found.");
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&subs)?
            } else {
                serde_json::to_string(&subs)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("┌──────────────────┬──────────────────────────────────────────────┬───────────────┐");
            println!("│ Name             │ Address                                      │ Value         │");
            println!("├──────────────────┼──────────────────────────────────────────────┼───────────────┤");
            for s in &subs {
                println!("│ {:<16} │ {:<44} │ ${:>11} │",
                    s.name, s.address, s.account_value);
            }
            println!("└──────────────────┴──────────────────────────────────────────────┴───────────────┘");
        }
    }
    Ok(())
}

/// `atlas agent approve <ADDRESS> [--name NAME]`
pub async fn agent_approve(address: &str, name: Option<&str>, fmt: OutputFormat) -> Result<()> {
    let orch = Orchestrator::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let result = perp.approve_agent(address, name).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json = serde_json::json!({
                "agent_address": address,
                "name": name.unwrap_or("(unnamed)"),
                "status": "approved",
                "message": result,
            });
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json)?
            } else {
                serde_json::to_string(&json)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("✅ {result}");
        }
    }
    Ok(())
}
