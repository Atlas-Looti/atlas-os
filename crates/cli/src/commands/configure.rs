use anyhow::{bail, Result};
use atlas_types::config::{SizeMode, TradingMode};
use atlas_types::output::ConfigOutput;
use atlas_utils::output::{render, OutputFormat};

/// `atlas configure show` — display current config (non-interactive).
pub fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    let output = ConfigOutput {
        mode: config.trading.mode.to_string(),
        size_mode: format!("{} (bare numbers = {})", config.trading.default_size_mode, size_mode_hint(&config.trading.default_size_mode)),
        leverage: config.trading.default_leverage,
        slippage: config.trading.default_slippage,
        network: if config.modules.hyperliquid.config.network == "testnet" { "Testnet".into() } else { "Mainnet".into() },
        lots: config.trading.lots.assets.clone(),
    };

    render(fmt, &output)?;

    if fmt == OutputFormat::Table {
        println!();
        println!("Tip: Edit settings with `atlas configure trading` or `atlas configure system`.");
    }

    Ok(())
}

/// `atlas configure mode <futures|cfd>`
pub fn set_mode(mode_str: &str) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.mode = parse_mode(mode_str)?;
    atlas_core::workspace::save_config(&config)?;

    match config.trading.mode {
        TradingMode::Futures => {
            println!("✓ Mode set to FUTURES — sizes are in asset units (e.g. 0.1 ETH)");
        }
        TradingMode::Cfd => {
            println!("✓ Mode set to CFD — sizes are in lots");
            println!("  Example: `atlas buy ETH 1` = 1 lot = {} ETH",
                config.trading.lots.lot_size("ETH"));
            println!("  Configure lot sizes with: atlas configure");
        }
    }
    Ok(())
}

/// `atlas configure size <usdc|units|lots>`
pub fn set_size_mode(mode_str: &str) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.default_size_mode = parse_size_mode(mode_str)?;
    atlas_core::workspace::save_config(&config)?;

    match config.trading.default_size_mode {
        SizeMode::Usdc => {
            println!("✓ Size mode: USDC — bare numbers are USDC margin");
            println!("  `atlas buy ETH 200` = $200 margin");
            println!("  `atlas buy ETH 200 --leverage 10` = $200 × 10x = $2000 notional");
            println!("  Override: `0.5eth` (units), `50lots` (lots)");
        }
        SizeMode::Units => {
            println!("✓ Size mode: UNITS — bare numbers are asset units");
            println!("  `atlas buy ETH 0.5` = 0.5 ETH");
            println!("  Override: `$200` (USDC), `50lots` (lots)");
        }
        SizeMode::Lots => {
            println!("✓ Size mode: LOTS — bare numbers are lot counts");
            println!("  `atlas buy ETH 50` = 50 lots = {} ETH",
                config.trading.lots.lots_to_size("ETH", 50.0));
            println!("  Override: `$200` (USDC), `0.5eth` (units)");
        }
    }
    Ok(())
}

/// `atlas configure leverage <value>`
pub fn set_leverage(value: u32) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.default_leverage = value;
    atlas_core::workspace::save_config(&config)?;
    println!("✓ Default leverage set to {value}x");
    Ok(())
}

/// `atlas configure slippage <value>`
pub fn set_slippage(value: f64) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    config.trading.default_slippage = value;
    atlas_core::workspace::save_config(&config)?;
    println!("✓ Default slippage set to {:.1}%", value * 100.0);
    Ok(())
}

/// `atlas configure lot <coin> <size>`
pub fn set_lot_size(coin: &str, size: f64) -> Result<()> {
    let mut config = atlas_core::workspace::load_config()?;
    let coin_upper = coin.to_uppercase();
    config.trading.lots.assets.insert(coin_upper.clone(), size);
    atlas_core::workspace::save_config(&config)?;
    println!("✓ {coin_upper} lot size set to {size} units per lot");
    Ok(())
}

// ─── Interactive helpers ────────────────────────────────────────────







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
