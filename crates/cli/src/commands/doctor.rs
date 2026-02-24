//! `atlas doctor` — system health checks.

use anyhow::Result;
use atlas_types::output::DoctorOutput;
use atlas_utils::output::{render, OutputFormat};

/// `atlas doctor [--fix]` — system health checks.
pub async fn run(fix: bool, fmt: OutputFormat) -> Result<()> {
    // ── Check 1: Config integrity ───────────────────────────────
    let config_ok = atlas_core::workspace::load_config().is_ok();

    // ── Check 2: Keystore exists ────────────────────────────────
    let wallets_path = atlas_core::workspace::resolve("keystore/wallets.json")?;
    let keystore_ok = wallets_path.exists();

    // ── Check 3: API connectivity ───────────────────────────────
    let api_latency_ms = check_api_latency().await.ok();

    // ── Check 4: Profile check ──────────────────────────────────
    let profile_ok = atlas_core::auth::AuthManager::load_store_pub()
        .map(|s| !s.wallets.is_empty())
        .unwrap_or(false);

    let output = DoctorOutput {
        config_ok,
        keystore_ok,
        ntp_ok: None,
        api_latency_ms,
    };

    if fmt != OutputFormat::Table {
        render(fmt, &output)?;
        return Ok(());
    }

    // Table mode
    println!("┌─────────────────────────────────────────────┐");
    println!("│  ATLAS DOCTOR                               │");
    println!("├─────────────────────────────────────────────┤");

    print_check("Config", config_ok, "atlas configure show");
    print_check("Keystore", keystore_ok, "atlas profile generate <name>");
    print_check("Wallet Profile", profile_ok, "atlas profile generate main");

    match api_latency_ms {
        Some(ms) => {
            let quality = if ms < 200 { "✓ excellent" }
                else if ms < 500 { "✓ good" }
                else if ms < 1000 { "⚠ slow" }
                else { "✗ very slow" };
            println!("│  API Latency    : {} ({ms}ms){:>width$}│",
                quality, "", width = 20_usize.saturating_sub(quality.len()));
        }
        None => {
            println!("│  API Latency    : ✗ unreachable              │");
        }
    }

    // Module status
    if let Ok(config) = atlas_core::workspace::load_config() {
        println!("├─────────────────────────────────────────────┤");
        println!("│  Modules:                                   │");
        let hl = if config.modules.hyperliquid.enabled { "✓ enabled" } else { "✗ disabled" };
        let morpho = if config.modules.morpho.enabled { "✓ enabled" } else { "✗ disabled" };
        let zx = if config.modules.zero_x.enabled { "✓ enabled" } else { "✗ disabled" };
        println!("│    Hyperliquid  : {:<25}│", hl);
        println!("│    Morpho       : {:<25}│", morpho);
        println!("│    0x Swap      : {:<25}│", zx);

        if config.modules.zero_x.enabled && config.modules.zero_x.config.api_key.is_empty() {
            println!("│    ⚠ 0x: API key not set                    │");
        }
    }

    println!("├─────────────────────────────────────────────┤");

    if fix {
        println!("│  --fix: Re-initializing workspace...        │");
        atlas_core::init_workspace()?;
        println!("│  ✓ Workspace re-initialized.                │");
    } else if !config_ok || !keystore_ok || !profile_ok {
        println!("│  Issues found. Run with --fix to repair.    │");
    } else {
        println!("│  ✓ All systems operational.                 │");
    }

    println!("└─────────────────────────────────────────────┘");
    Ok(())
}

fn print_check(name: &str, ok: bool, hint: &str) {
    let icon = if ok { "✓" } else { "✗" };
    let pad = 14 - name.len();
    if ok {
        println!("│  {name}{:>pad$}: {icon}{:>30}│", "", "");
    } else {
        let hint_short = if hint.len() > 28 { &hint[..28] } else { hint };
        println!("│  {name}{:>pad$}: {icon} → {:<26}│", "", hint_short);
    }
}

async fn check_api_latency() -> Result<u64> {
    let start = std::time::Instant::now();
    let client = hypersdk::hypercore::mainnet();
    let _ = client.all_mids(None).await?;
    Ok(start.elapsed().as_millis() as u64)
}
