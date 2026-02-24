use anyhow::Result;
use atlas_core::AuthManager;
use atlas_utils::output::OutputFormat;

/// `atlas profile generate <name>`
pub fn generate_wallet(name: &str, fmt: OutputFormat) -> Result<()> {
    let (profile_name, address, private_key) = AuthManager::create_new_wallet(name)?;

    if fmt != OutputFormat::Table {
        let json = serde_json::json!({
            "name": profile_name,
            "address": address,
            "private_key": private_key,
        });
        if matches!(fmt, OutputFormat::JsonPretty) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", serde_json::to_string(&json)?);
        }
        return Ok(());
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  NEW WALLET CREATED                                        ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Profile : {:<49}║", profile_name);
    println!("║  Address : {:<49}║", address);
    println!("║  Private : {:<49}║", private_key);
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ⚠  BACK UP YOUR PRIVATE KEY NOW. It will NOT be shown     ║");
    println!("║     again. It is stored ONLY in your OS secure keyring.     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    Ok(())
}

/// `atlas profile import <name>` — prompts for the private key on stdin.
pub fn import_wallet(name: &str, fmt: OutputFormat) -> Result<()> {
    if fmt == OutputFormat::Table {
        println!("Enter private key (hex, with or without 0x prefix):");
    }

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read input: {e}"))?;
    let key = input.trim();

    if key.is_empty() {
        anyhow::bail!("No key provided");
    }

    let (profile_name, address) = AuthManager::import_wallet(name, key)?;

    if fmt != OutputFormat::Table {
        let json = serde_json::json!({
            "ok": true,
            "name": profile_name,
            "address": address,
        });
        if matches!(fmt, OutputFormat::JsonPretty) {
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            println!("{}", serde_json::to_string(&json)?);
        }
    } else {
        println!("✓ Imported profile '{profile_name}' → {address}");
    }
    Ok(())
}

/// `atlas profile use <name>`
pub fn switch_profile(name: &str, fmt: OutputFormat) -> Result<()> {
    AuthManager::switch_profile(name)?;

    if fmt != OutputFormat::Table {
        let json = serde_json::json!({"ok": true, "profile": name});
        println!("{}", serde_json::to_string(&json)?);
    } else {
        println!("✓ Active profile switched to '{name}'");
    }
    Ok(())
}

/// `atlas profile list`
pub fn list_profiles(fmt: OutputFormat) -> Result<()> {
    let store = AuthManager::load_store_pub()?;
    let config = atlas_core::workspace::load_config()?;

    if fmt != OutputFormat::Table {
        let profiles: Vec<serde_json::Value> = store.wallets.iter().map(|w| {
            serde_json::json!({
                "name": w.name,
                "address": w.address,
                "active": w.name == config.system.active_profile,
            })
        }).collect();
        if matches!(fmt, OutputFormat::JsonPretty) {
            println!("{}", serde_json::to_string_pretty(&profiles)?);
        } else {
            println!("{}", serde_json::to_string(&profiles)?);
        }
        return Ok(());
    }

    if store.wallets.is_empty() {
        println!("No profiles found. Create one with: atlas profile generate <name>");
        return Ok(());
    }

    println!("┌──────────────────┬────────────────────────────────────────────┬──────────┐");
    println!("│ Profile          │ Address                                    │ Active   │");
    println!("├──────────────────┼────────────────────────────────────────────┼──────────┤");
    for w in &store.wallets {
        let active = if w.name == config.system.active_profile { "  ●" } else { "" };
        println!("│ {:<16} │ {:<42} │ {:<8} │", w.name, w.address, active);
    }
    println!("└──────────────────┴────────────────────────────────────────────┴──────────┘");
    Ok(())
}
