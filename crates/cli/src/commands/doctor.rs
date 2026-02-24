use anyhow::Result;
use atlas_types::output::DoctorOutput;
use atlas_utils::output::{render, OutputFormat, TableDisplay};

/// `atlas doctor [--fix]` — system health checks.
pub async fn run(fix: bool, fmt: OutputFormat) -> Result<()> {
    // ── Check 1: Config integrity ───────────────────────────────
    let config_ok = atlas_core::workspace::load_config().is_ok();

    // ── Check 2: Keystore exists ────────────────────────────────
    let wallets_path = atlas_core::workspace::resolve("keystore/wallets.json")?;
    let keystore_ok = wallets_path.exists();

    let output = DoctorOutput {
        config_ok,
        keystore_ok,
        ntp_ok: None,        // not implemented yet
        api_latency_ms: None, // not implemented yet
    };

    if fmt != OutputFormat::Table {
        render(fmt, &output)?;
        return Ok(());
    }

    // Table mode with fix support
    output.print_table();

    println!("├─────────────────────────────────────────────┤");

    if fix {
        println!("│  --fix: Re-initializing workspace...        │");
        atlas_core::init_workspace()?;
        println!("│  Workspace re-initialized.                  │");
    } else {
        println!("│  Run with --fix to attempt auto-repair.     │");
    }

    println!("└─────────────────────────────────────────────┘");
    Ok(())
}
