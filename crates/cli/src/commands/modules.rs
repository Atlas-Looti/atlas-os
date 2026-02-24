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

/// `atlas configure module set <module> <key> <value> [<value2>]`
///
/// Handles all per-module config keys per PRD:
///   hl: network, mode, default-size-mode, default-leverage, default-slippage, lot <COIN> <size>
///   0x: default-chain, default-slippage-bps
pub fn config_set(module: &str, values: &[String], fmt: OutputFormat) -> Result<()> {
    if values.is_empty() {
        anyhow::bail!("Usage: atlas configure module set <module> <key> <value>");
    }
    let key = values[0].as_str();
    let mut config = atlas_core::workspace::load_config()?;
    let resolved = resolve_module(module)?;

    match resolved {
        "hyperliquid" => {
            let hl = &mut config.modules.hyperliquid.config;
            match key {
                "network" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set hl network <mainnet|testnet>"))?;
                    if v != "mainnet" && v != "testnet" {
                        anyhow::bail!("Invalid network: {v}. Must be 'mainnet' or 'testnet'.");
                    }
                    hl.network = v.to_string();
                }
                "mode" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set hl mode <futures|cfd>"))?;
                    hl.mode = v.parse().map_err(|_| anyhow::anyhow!("Invalid mode: {v}. Must be 'futures' or 'cfd'."))?;
                }
                "default-size-mode" | "size-mode" | "size" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set hl default-size-mode <usdc|units|lots>"))?;
                    hl.default_size_mode = v.parse().map_err(|_| anyhow::anyhow!("Invalid size mode: {v}. Must be 'usdc', 'units', or 'lots'."))?;
                }
                "default-leverage" | "leverage" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set hl leverage <n>"))?;
                    hl.default_leverage = v.parse().map_err(|_| anyhow::anyhow!("Invalid leverage: {v}"))?;
                }
                "default-slippage" | "slippage" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set hl slippage <0.05>"))?;
                    hl.default_slippage = v.parse().map_err(|_| anyhow::anyhow!("Invalid slippage: {v}"))?;
                }
                "lot" => {
                    let coin = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set hl lot <COIN> <size>"))?;
                    let size: f64 = values.get(2)
                        .ok_or_else(|| anyhow::anyhow!("Usage: set hl lot {coin} <size>"))?
                        .parse()
                        .map_err(|_| anyhow::anyhow!("Invalid lot size"))?;
                    hl.lots.assets.insert(coin.to_uppercase(), size);
                }
                _ => anyhow::bail!(
                    "Unknown key '{key}' for hyperliquid.\n\
                    Available: network, mode, default-size-mode, leverage, slippage, lot"
                ),
            }
        }
        "zero_x" => {
            let zx = &mut config.modules.zero_x.config;
            match key {
                "default-chain" | "chain" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set 0x default-chain <ethereum|arbitrum|base>"))?;
                    zx.default_chain = v.to_string();
                }
                "default-slippage-bps" | "slippage-bps" | "slippage" => {
                    let v = values.get(1).ok_or_else(|| anyhow::anyhow!("Usage: set 0x slippage-bps <100>"))?;
                    zx.default_slippage_bps = v.parse().map_err(|_| anyhow::anyhow!("Invalid slippage bps: {v}"))?;
                }
                _ => anyhow::bail!(
                    "Unknown key '{key}' for zero_x.\n\
                    Available: default-chain, default-slippage-bps"
                ),
            }
        }
        _ => unreachable!(),
    }

    atlas_core::workspace::save_config(&config)?;

    let display_val = values[1..].join(" ");
    if fmt == OutputFormat::Table {
        println!("✓ {resolved}.{key} = {display_val}");
    } else {
        json_ok(
            fmt,
            "config_set",
            resolved,
            Some(("key", &format!("{key}={display_val}"))),
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
