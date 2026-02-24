use anyhow::Result;
use atlas_core::output::OutputFormat;

/// `atlas sub list`
pub async fn sub_list(fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let subs = perp
        .subaccounts()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if subs.is_empty() {
        println!("No subaccounts found.");
        return Ok(());
    }

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let envelope = serde_json::json!({"ok": true, "data": subs});
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&envelope)?
            } else {
                serde_json::to_string(&envelope)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("┌──────────────────┬──────────────────────────────────────────────┬───────────────┐");
            println!("│ Name             │ Address                                      │ Value         │");
            println!("├──────────────────┼──────────────────────────────────────────────┼───────────────┤");
            for s in &subs {
                println!(
                    "│ {:<16} │ {:<44} │ ${:>11} │",
                    s.name, s.address, s.account_value
                );
            }
            println!("└──────────────────┴──────────────────────────────────────────────┴───────────────┘");
        }
    }
    Ok(())
}

/// `atlas agent approve <ADDRESS> [--name NAME]`
pub async fn agent_approve(address: &str, name: Option<&str>, fmt: OutputFormat) -> Result<()> {
    let orch = crate::factory::from_active_profile().await?;
    let perp = orch.perp(None)?;

    let result = perp
        .approve_agent(address, name)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let data = serde_json::json!({
                "agent_address": address,
                "name": name.unwrap_or("(unnamed)"),
                "status": "approved",
                "message": result,
            });
            let envelope = serde_json::json!({"ok": true, "data": data});
            let s = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&envelope)?
            } else {
                serde_json::to_string(&envelope)?
            };
            println!("{s}");
        }
        OutputFormat::Table => {
            println!("✅ {result}");
        }
    }
    Ok(())
}
