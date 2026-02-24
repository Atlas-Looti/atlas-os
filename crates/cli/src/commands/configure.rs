use anyhow::Result;
use atlas_core::config::SizeMode;
use atlas_core::output::ConfigOutput;
use atlas_core::output::{render, OutputFormat};

/// `atlas configure show` — display current config (non-interactive).
pub fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;
    let hl = &config.modules.hyperliquid.config;

    // JSON gets clean machine-readable values; table gets human-friendly text
    if fmt != OutputFormat::Table {
        let json = serde_json::json!({
            "mode": hl.mode.to_string(),
            "size_mode": hl.default_size_mode.to_string(),
            "leverage": hl.default_leverage,
            "slippage": hl.default_slippage,
            "network": hl.network,
            "lots": hl.lots.assets,
        });
        let s = if matches!(fmt, OutputFormat::JsonPretty) {
            serde_json::to_string_pretty(&json)?
        } else {
            serde_json::to_string(&json)?
        };
        println!("{s}");
        return Ok(());
    }

    let output = ConfigOutput {
        mode: hl.mode.to_string(),
        size_mode: format!(
            "{} (bare numbers = {})",
            hl.default_size_mode,
            size_mode_hint(&hl.default_size_mode)
        ),
        leverage: hl.default_leverage,
        slippage: hl.default_slippage,
        network: if hl.network == "testnet" {
            "Testnet".into()
        } else {
            "Mainnet".into()
        },
        lots: hl.lots.assets.clone(),
    };

    render(OutputFormat::Table, &output)?;

    println!();
    println!("Tip: Use `atlas configure module set hl <key> <value>` to change settings.");

    Ok(())
}

fn size_mode_hint(mode: &SizeMode) -> &'static str {
    match mode {
        SizeMode::Usdc => "USDC margin",
        SizeMode::Units => "asset units",
        SizeMode::Lots => "lot count",
    }
}
