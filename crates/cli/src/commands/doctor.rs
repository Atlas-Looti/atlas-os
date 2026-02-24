//! `atlas doctor` — system health checks.

use anyhow::Result;
use atlas_core::output::{render, OutputFormat};
use atlas_core::output::{DoctorCheck, DoctorOutput};

/// `atlas doctor [--fix]` — system health checks.
pub async fn run(fix: bool, fmt: OutputFormat) -> Result<()> {
    // ── Check 1: Profile ────────────────────────────────────────────
    let config_result = atlas_core::workspace::load_config();
    let profile_check = match (
        &config_result,
        atlas_core::auth::AuthManager::load_store_pub(),
    ) {
        (Ok(cfg), Ok(store)) if !store.wallets.is_empty() => {
            let active = &cfg.system.active_profile;
            if store.exists(active) {
                DoctorCheck::ok("profile", active)
            } else {
                DoctorCheck::fail(
                    "profile",
                    format!(
                        "Active profile '{active}' not found. Run: atlas profile generate {active}"
                    ),
                )
            }
        }
        _ => DoctorCheck::fail(
            "profile",
            "Run: atlas profile generate main — creates a new wallet profile",
        ),
    };

    // ── Check 2: Keyring ────────────────────────────────────────────
    let wallets_path = atlas_core::workspace::resolve("keystore/wallets.json")?;
    let keyring_check = if wallets_path.exists() {
        DoctorCheck::ok_bare("keyring")
    } else {
        DoctorCheck::fail(
            "keyring",
            "Run: atlas profile generate main — initializes keystore",
        )
    };

    // ── Check 3: API Key ────────────────────────────────────────────
    let api_key_check = match atlas_core::workspace::load_config() {
        Ok(config) if config.system.api_key.is_some() => DoctorCheck::ok_bare("api_key"),
        _ => DoctorCheck::fail(
            "api_key",
            "Run: atlas configure system api-key <key> — get key from apps/frontend → Settings",
        ),
    };

    // ── Check 4: Backend/Network latency ────────────────────────────
    let (backend_check, hl_check) = match check_api_latency().await {
        Ok(ms) => {
            let mut backend = DoctorCheck::ok("backend", format!("{ms}ms"));
            backend.latency_ms = Some(ms);

            // ── Check 5: Hyperliquid module ──────────────────────────
            let hl = match atlas_core::workspace::load_config() {
                Ok(cfg) if cfg.modules.hyperliquid.enabled => {
                    let net = cfg.modules.hyperliquid.config.network.clone();
                    let mut check = DoctorCheck::ok("hyperliquid", &net);
                    check.network = Some(net);
                    check
                }
                _ => DoctorCheck::fail(
                    "hyperliquid",
                    "Run: atlas configure module enable hyperliquid && atlas configure module set hl network mainnet",
                ),
            };
            (backend, hl)
        }
        Err(_) => {
            let backend = DoctorCheck::fail(
                "backend",
                "Hyperliquid API unreachable — check network connectivity",
            );
            let hl = DoctorCheck::fail(
                "hyperliquid",
                "Cannot connect to Hyperliquid — check network connectivity",
            );
            (backend, hl)
        }
    };

    let checks = vec![
        profile_check,
        keyring_check,
        api_key_check,
        backend_check,
        hl_check,
    ];

    let all_ok = checks.iter().all(|c| c.status == "ok");
    let output = DoctorOutput { checks };

    if fmt != OutputFormat::Table {
        render(fmt, &output)?;
        return Ok(());
    }

    // ── Table mode — human-friendly ──────────────────────────────────
    println!("┌─────────────────────────────────────────────┐");
    println!("│  ATLAS DOCTOR                               │");
    println!("├─────────────────────────────────────────────┤");

    for check in &output.checks {
        let icon = if check.status == "ok" { "✓" } else { "✗" };
        let label = format!("{:<14}", check.name);
        if check.status == "ok" {
            let val = check.value.as_deref().unwrap_or("");
            let display = if val.is_empty() {
                icon.to_string()
            } else {
                format!("{icon} ({val})")
            };
            println!("│  {label}: {:<27}│", display);
        } else {
            let fix = check
                .fix
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(26)
                .collect::<String>();
            println!("│  {label}: {icon} → {:<26}│", fix);
        }
    }

    if fix {
        println!("├─────────────────────────────────────────────┤");
        println!("│  --fix: Re-initializing workspace...        │");
        atlas_core::init_workspace()?;
        println!("│  ✓ Workspace re-initialized.                │");
    } else if !all_ok {
        println!("├─────────────────────────────────────────────┤");
        println!("│  Issues found. Run with --fix to repair.    │");
    } else {
        println!("├─────────────────────────────────────────────┤");
        println!("│  ✓ All systems operational.                 │");
    }

    println!("└─────────────────────────────────────────────┘");
    Ok(())
}

async fn check_api_latency() -> Result<u64> {
    let start = std::time::Instant::now();
    let client = hypersdk::hypercore::mainnet();
    let _ = client.all_mids(None).await?;
    Ok(start.elapsed().as_millis() as u64)
}
