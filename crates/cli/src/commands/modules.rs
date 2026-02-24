//! `atlas module` — Module management (list, enable, disable, config).

use anyhow::Result;
use atlas_core::output::OutputFormat;

fn json_ok(fmt: OutputFormat, action: &str, module: &str, extra: Option<(&str, &str)>) {
    if fmt != OutputFormat::Table {
        let mut data = serde_json::json!({"action": action, "module": module});
        if let Some((k, v)) = extra {
            data[k] = serde_json::Value::String(v.to_string());
        }
        let map = serde_json::json!({"ok": true, "data": data});
        println!("{}", serde_json::to_string(&map).unwrap_or_default());
    }
}

/// `atlas module list`
pub fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    let modules = vec![
        (
            "hyperliquid",
            "Perpetual Trading",
            config.modules.hyperliquid.enabled,
            format!("network={}", config.modules.hyperliquid.config.network,),
        ),
        (
            "zero_x",
            "DEX Aggregator (0x)",
            config.modules.zero_x.enabled,
            String::from("proxied via backend"),
        ),
    ];

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json_modules: Vec<serde_json::Value> = modules
                .iter()
                .map(|(name, desc, enabled, cfg)| {
                    serde_json::json!({
                        "name": name,
                        "description": desc,
                        "enabled": enabled,
                        "config": cfg,
                    })
                })
                .collect();
            let json = if matches!(fmt, OutputFormat::JsonPretty) {
                serde_json::to_string_pretty(&json_modules)?
            } else {
                serde_json::to_string(&json_modules)?
            };
            println!("{json}");
        }
        OutputFormat::Table => {
            println!("╔══════════════════════════════════════════════════════════════╗");
            println!("║  ATLAS OS — MODULES                                        ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            for (name, desc, enabled, cfg) in &modules {
                let status = if *enabled { "✓ ON " } else { "✗ OFF" };
                println!("║  {} │ {:<14} │ {:<24} ║", status, name, desc);
                println!("║        │ {:<49} ║", cfg);
            }
            println!("╚══════════════════════════════════════════════════════════════╝");
        }
    }

    Ok(())
}

/// `atlas module enable <name>`
pub fn enable(name: &str, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    let resolved = resolve_module(name)?;

    match resolved {
        "hyperliquid" => config.modules.hyperliquid.enabled = true,
        "zero_x" => config.modules.zero_x.enabled = true,
        _ => unreachable!(),
    }
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        println!("✓ Module '{resolved}' enabled.");
    } else {
        json_ok(fmt, "enable", resolved, None);
    }
    Ok(())
}

/// `atlas module disable <name>`
pub fn disable(name: &str, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    let resolved = resolve_module(name)?;

    match resolved {
        "hyperliquid" => config.modules.hyperliquid.enabled = false,
        "zero_x" => config.modules.zero_x.enabled = false,
        _ => unreachable!(),
    }
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        println!("✗ Module '{resolved}' disabled.");
    } else {
        json_ok(fmt, "disable", resolved, None);
    }
    Ok(())
}

/// `atlas module config set <module> <key> <value>`
pub fn config_set(module: &str, key: &str, value: &str, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    let resolved = resolve_module(module)?;

    match resolved {
        "hyperliquid" => match key {
            "network" => {
                if value != "mainnet" && value != "testnet" {
                    anyhow::bail!("Invalid network: {value}. Must be 'mainnet' or 'testnet'.");
                }
                config.modules.hyperliquid.config.network = value.to_string();
            }
            _ => anyhow::bail!("Unknown key '{key}' for hyperliquid. Available: network"),
        },
        "zero_x" => anyhow::bail!("No configurable keys for zero_x yet."),
        _ => unreachable!(),
    }

    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        println!("✓ {resolved}.{key} = {value}");
    } else {
        json_ok(
            fmt,
            "config_set",
            resolved,
            Some(("key", &format!("{key}={value}"))),
        );
    }
    Ok(())
}

fn resolve_module(name: &str) -> Result<&'static str> {
    match name.to_lowercase().as_str() {
        "hyperliquid" | "hl" | "perp" => Ok("hyperliquid"),
        "zero_x" | "0x" | "swap" => Ok("zero_x"),
        _ => anyhow::bail!("Unknown module: {name}. Available: hyperliquid, zero_x"),
    }
}
