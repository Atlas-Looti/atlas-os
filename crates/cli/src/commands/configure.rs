use anyhow::{bail, Result};
use atlas_types::config::{SizeMode, TradingMode};
use atlas_types::output::ConfigOutput;
use atlas_utils::output::{render, OutputFormat};

/// Helper: print JSON ack for write operations.
fn json_ack(fmt: OutputFormat, action: &str, key: &str, value: &serde_json::Value) {
    if fmt != OutputFormat::Table {
        let json = serde_json::json!({"ok": true, "action": action, "key": key, "value": value});
        println!("{}", serde_json::to_string(&json).unwrap_or_default());
    }
}

/// `atlas configure show` — display current config (non-interactive).
pub fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    // JSON gets clean machine-readable values; table gets human-friendly text
    if fmt != OutputFormat::Table {
        let json = serde_json::json!({
            "mode": config.trading.mode.to_string(),
            "size_mode": config.trading.default_size_mode.to_string(),
            "leverage": config.trading.default_leverage,
            "slippage": config.trading.default_slippage,
            "network": config.modules.hyperliquid.config.network,
            "lots": config.trading.lots.assets,
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
        mode: config.trading.mode.to_string(),
        size_mode: format!("{} (bare numbers = {})", config.trading.default_size_mode, size_mode_hint(&config.trading.default_size_mode)),
        leverage: config.trading.default_leverage,
        slippage: config.trading.default_slippage,
        network: if config.modules.hyperliquid.config.network == "testnet" { "Testnet".into() } else { "Mainnet".into() },
        lots: config.trading.lots.assets.clone(),
    };

    render(OutputFormat::Table, &output)?;

    println!();
    println!("Tip: Edit settings with `atlas configure trading` or `atlas configure system`.");

    Ok(())
}

/// `atlas configure mode <futures|cfd>`
pub fn set_mode(mode_str: &str, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.mode = parse_mode(mode_str)?;
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        match config.trading.mode {
            TradingMode::Futures => println!("✓ Mode set to FUTURES — sizes are in asset units (e.g. 0.1 ETH)"),
            TradingMode::Cfd => {
                println!("✓ Mode set to CFD — sizes are in lots");
                println!("  Example: `atlas buy ETH 1` = 1 lot = {} ETH", config.trading.lots.lot_size("ETH"));
            }
        }
    } else {
        json_ack(fmt, "set_mode", "mode", &serde_json::Value::String(config.trading.mode.to_string()));
    }
    Ok(())
}

/// `atlas configure size <usdc|units|lots>`
pub fn set_size_mode(mode_str: &str, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.default_size_mode = parse_size_mode(mode_str)?;
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        match config.trading.default_size_mode {
            SizeMode::Usdc => {
                println!("✓ Size mode: USDC — bare numbers are USDC margin");
                println!("  `atlas buy ETH 200` = $200 margin");
            }
            SizeMode::Units => {
                println!("✓ Size mode: UNITS — bare numbers are asset units");
                println!("  `atlas buy ETH 0.5` = 0.5 ETH");
            }
            SizeMode::Lots => {
                println!("✓ Size mode: LOTS — bare numbers are lot counts");
                println!("  `atlas buy ETH 50` = 50 lots = {} ETH", config.trading.lots.lots_to_size("ETH", 50.0));
            }
        }
    } else {
        json_ack(fmt, "set_size_mode", "size_mode", &serde_json::Value::String(config.trading.default_size_mode.to_string()));
    }
    Ok(())
}

/// `atlas configure leverage <value>`
pub fn set_leverage(value: u32, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.default_leverage = value;
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        println!("✓ Default leverage set to {value}x");
    } else {
        json_ack(fmt, "set_leverage", "leverage", &serde_json::json!(value));
    }
    Ok(())
}

/// `atlas configure slippage <value>`
pub fn set_slippage(value: f64, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.default_slippage = value;
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        println!("✓ Default slippage set to {:.1}%", value * 100.0);
    } else {
        json_ack(fmt, "set_slippage", "slippage", &serde_json::json!(value));
    }
    Ok(())
}

/// `atlas configure lot <coin> <size>`
pub fn set_lot_size(coin: &str, size: f64, fmt: OutputFormat) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    let coin_upper = coin.to_uppercase();
    config.trading.lots.assets.insert(coin_upper.clone(), size);
    atlas_core::workspace::save_config(&config)?;

    if fmt == OutputFormat::Table {
        println!("✓ {coin_upper} lot size set to {size} units per lot");
    } else {
        json_ack(fmt, "set_lot_size", &coin_upper, &serde_json::json!(size));
    }
    Ok(())
}

// ─── Parsers ────────────────────────────────────────────────────────

fn parse_mode(s: &str) -> Result<TradingMode> {
    match s.to_lowercase().as_str() {
        "futures" | "future" | "f" => Ok(TradingMode::Futures),
        "cfd" | "c" | "lots" | "lot" => Ok(TradingMode::Cfd),
        _ => bail!("Invalid mode '{s}'. Use 'futures' or 'cfd'"),
    }
}

fn parse_size_mode(s: &str) -> Result<SizeMode> {
    match s.to_lowercase().as_str() {
        "usdc" | "usd" | "dollar" | "$" | "d" => Ok(SizeMode::Usdc),
        "units" | "unit" | "u" | "raw" => Ok(SizeMode::Units),
        "lots" | "lot" | "l" => Ok(SizeMode::Lots),
        _ => bail!("Invalid size mode '{s}'. Use 'usdc', 'units', or 'lots'"),
    }
}

fn size_mode_hint(mode: &SizeMode) -> &'static str {
    match mode {
        SizeMode::Usdc => "USDC margin",
        SizeMode::Units => "asset units",
        SizeMode::Lots => "lot count",
    }
}
