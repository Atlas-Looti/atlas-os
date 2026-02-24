//! `atlas module` — Module management (list, enable, disable, config).

use anyhow::Result;
use atlas_utils::output::OutputFormat;

/// `atlas module list`
pub fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    let modules = vec![
        ("hyperliquid", "Perpetual Trading", config.modules.hyperliquid.enabled,
         format!("network={}, rpc={}", config.modules.hyperliquid.config.network, config.modules.hyperliquid.config.rpc_url)),
        ("morpho", "DeFi Lending", config.modules.morpho.enabled,
         format!("chain={}", config.modules.morpho.config.chain)),
    ];

    match fmt {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            let json_modules: Vec<serde_json::Value> = modules.iter().map(|(name, desc, enabled, cfg)| {
                serde_json::json!({
                    "name": name,
                    "description": desc,
                    "enabled": enabled,
                    "config": cfg,
                })
            }).collect();
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
pub fn enable(name: &str, _fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;

    match name.to_lowercase().as_str() {
        "hyperliquid" | "hl" | "perp" => {
            config.modules.hyperliquid.enabled = true;
            atlas_core::workspace::save_config(&config)?;
            println!("✓ Module 'hyperliquid' enabled.");
        }
        "morpho" | "lending" => {
            config.modules.morpho.enabled = true;
            atlas_core::workspace::save_config(&config)?;
            println!("✓ Module 'morpho' enabled.");
        }
        _ => anyhow::bail!("Unknown module: {name}. Available: hyperliquid, morpho"),
    }
    Ok(())
}

/// `atlas module disable <name>`
pub fn disable(name: &str, _fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;

    match name.to_lowercase().as_str() {
        "hyperliquid" | "hl" | "perp" => {
            config.modules.hyperliquid.enabled = false;
            atlas_core::workspace::save_config(&config)?;
            println!("✗ Module 'hyperliquid' disabled.");
        }
        "morpho" | "lending" => {
            config.modules.morpho.enabled = false;
            atlas_core::workspace::save_config(&config)?;
            println!("✗ Module 'morpho' disabled.");
        }
        _ => anyhow::bail!("Unknown module: {name}. Available: hyperliquid, morpho"),
    }
    Ok(())
}

/// `atlas module config set <module> <key> <value>`
pub fn config_set(module: &str, key: &str, value: &str, _fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;

    match module.to_lowercase().as_str() {
        "hyperliquid" | "hl" => {
            match key {
                "network" => {
                    if value != "mainnet" && value != "testnet" {
                        anyhow::bail!("Invalid network: {value}. Must be 'mainnet' or 'testnet'.");
                    }
                    config.modules.hyperliquid.config.network = value.to_string();
                }
                "rpc_url" | "rpc" => {
                    config.modules.hyperliquid.config.rpc_url = value.to_string();
                }
                _ => anyhow::bail!("Unknown key '{key}' for hyperliquid. Available: network, rpc_url"),
            }
        }
        "morpho" => {
            match key {
                "chain" => {
                    if value != "ethereum" && value != "base" {
                        anyhow::bail!("Invalid chain: {value}. Must be 'ethereum' or 'base'.");
                    }
                    config.modules.morpho.config.chain = value.to_string();
                }
                _ => anyhow::bail!("Unknown key '{key}' for morpho. Available: chain"),
            }
        }
        _ => anyhow::bail!("Unknown module: {module}. Available: hyperliquid, morpho"),
    }

    atlas_core::workspace::save_config(&config)?;
    println!("✓ {module}.{key} = {value}");
    Ok(())
}
