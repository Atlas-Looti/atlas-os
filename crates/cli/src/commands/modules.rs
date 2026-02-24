//! `atlas modules` — Show enabled/disabled modules.

use anyhow::Result;
use atlas_utils::output::OutputFormat;

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
            println!();
            println!("Edit ~/.atlas-os/atlas.json to enable/disable modules.");
        }
    }

    Ok(())
}
