use anyhow::{bail, Result};
use atlas_types::config::{SizeMode, TradingMode};
use atlas_types::output::ConfigOutput;
use atlas_utils::output::{render, OutputFormat, TableDisplay};
use atlas_utils::prompt::prompt;

/// `atlas configure` — interactive configuration setup or show config.
pub fn run(fmt: OutputFormat) -> Result<()> {
    let config = atlas_core::workspace::load_config()?;

    let output = ConfigOutput {
        mode: config.trading.mode.to_string(),
        size_mode: format!("{} (bare numbers = {})", config.trading.default_size_mode, size_mode_hint(&config.trading.default_size_mode)),
        leverage: config.trading.default_leverage,
        slippage: config.trading.default_slippage,
        network: if config.network.testnet { "Testnet".into() } else { "Mainnet".into() },
        lots: config.trading.lots.assets.clone(),
    };

    // JSON mode: just dump config and exit
    if fmt != OutputFormat::Table {
        return render(fmt, &output);
    }

    // Table mode: show config, then interactive menu
    output.print_table();
    println!();
    println!("Edit which setting?");
    println!("  1) Trading Mode (futures / cfd)");
    println!("  2) Size Mode (usdc / units / lots)");
    println!("  3) Default Leverage");
    println!("  4) Default Slippage");
    println!("  5) Network (mainnet / testnet)");
    println!("  6) Lot Sizes (CFD mode)");
    println!("  q) Save & Exit");
    println!();

    let mut config = config;
    loop {
        let input = prompt("Choice")?;
        match input.trim() {
            "1" => configure_mode(&mut config)?,
            "2" => configure_size_mode(&mut config)?,
            "3" => configure_leverage(&mut config)?,
            "4" => configure_slippage(&mut config)?,
            "5" => configure_network(&mut config)?,
            "6" => configure_lots(&mut config)?,
            "q" | "Q" | "" => break,
            _ => println!("Invalid choice. Try 1-6 or q."),
        }
    }

    atlas_core::workspace::save_config(&config)?;
    println!("✓ Configuration saved.");
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

fn configure_mode(config: &mut atlas_types::config::AppConfig) -> Result<()> {
    println!();
    println!("  Trading Mode:");
    println!("    futures — Position on Hyperliquid perpetual futures");
    println!("    cfd     — CFD-style with lot sizing (wraps futures)");
    println!();
    let current = &config.trading.mode;
    let input = prompt(&format!("Mode [{}]", current))?;
    if !input.trim().is_empty() {
        config.trading.mode = parse_mode(input.trim())?;
        println!("  → Mode: {}", config.trading.mode);
    }
    Ok(())
}

fn configure_size_mode(config: &mut atlas_types::config::AppConfig) -> Result<()> {
    println!();
    println!("  Size Mode (how bare numbers in commands are interpreted):");
    println!("    usdc  — `atlas buy ETH 200` = $200 margin (recommended)");
    println!("    units — `atlas buy ETH 0.5` = 0.5 ETH (pro trader)");
    println!("    lots  — `atlas buy ETH 50`  = 50 lots (CFD style)");
    println!();
    println!("  Tip: Explicit suffixes always override: $200, 0.5eth, 50lots");
    println!();
    let current = &config.trading.default_size_mode;
    let input = prompt(&format!("Size Mode [{}]", current))?;
    if !input.trim().is_empty() {
        config.trading.default_size_mode = parse_size_mode(input.trim())?;
        println!("  → Size Mode: {} ({})",
            config.trading.default_size_mode,
            size_mode_hint(&config.trading.default_size_mode));
    }
    Ok(())
}

fn configure_leverage(config: &mut atlas_types::config::AppConfig) -> Result<()> {
    let input = prompt(&format!("Default Leverage [{}]", config.trading.default_leverage))?;
    if !input.trim().is_empty() {
        let v: u32 = input.trim().parse()
            .map_err(|_| anyhow::anyhow!("Invalid number"))?;
        config.trading.default_leverage = v;
        println!("  → Leverage: {v}x");
    }
    Ok(())
}

fn configure_slippage(config: &mut atlas_types::config::AppConfig) -> Result<()> {
    println!("  Enter as decimal (0.05 = 5%, 0.01 = 1%)");
    let input = prompt(&format!("Slippage [{:.2}]", config.trading.default_slippage))?;
    if !input.trim().is_empty() {
        let v: f64 = input.trim().parse()
            .map_err(|_| anyhow::anyhow!("Invalid number"))?;
        config.trading.default_slippage = v;
        println!("  → Slippage: {:.1}%", v * 100.0);
    }
    Ok(())
}

fn configure_network(config: &mut atlas_types::config::AppConfig) -> Result<()> {
    let current = if config.network.testnet { "testnet" } else { "mainnet" };
    let input = prompt(&format!("Network [{}]", current))?;
    match input.trim().to_lowercase().as_str() {
        "mainnet" | "main" => {
            config.network.testnet = false;
            config.network.rpc_url = "https://api.hyperliquid.xyz".to_string();
            println!("  → Mainnet");
        }
        "testnet" | "test" => {
            config.network.testnet = true;
            config.network.rpc_url = "https://api.hyperliquid-testnet.xyz".to_string();
            println!("  → Testnet");
        }
        "" => {}
        _ => println!("  Invalid. Use 'mainnet' or 'testnet'."),
    }
    Ok(())
}

fn configure_lots(config: &mut atlas_types::config::AppConfig) -> Result<()> {
    println!();
    println!("  Current lot sizes (1 lot = X units of asset):");
    println!("  ┌──────────┬───────────────────┐");
    println!("  │ Asset    │ Lot Size          │");
    println!("  ├──────────┼───────────────────┤");

    let mut sorted: Vec<_> = config.trading.lots.assets.iter().collect();
    sorted.sort_by_key(|(k, _)| (*k).clone());
    for (coin, size) in &sorted {
        println!("  │ {:<8} │ {:>17} │", coin, size);
    }
    println!("  ├──────────┼───────────────────┤");
    println!("  │ Default  │ {:>17} │", config.trading.lots.default_lot_size);
    println!("  └──────────┴───────────────────┘");
    println!();
    println!("  Set lot size: enter 'COIN SIZE' (e.g. 'BTC 0.01') or 'q' to go back");

    loop {
        let input = prompt("Lot")?;
        let trimmed = input.trim();
        if trimmed == "q" || trimmed.is_empty() {
            break;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            println!("  Format: COIN SIZE (e.g. 'ETH 0.1')");
            continue;
        }

        let coin = parts[0].to_uppercase();
        match parts[1].parse::<f64>() {
            Ok(size) if size > 0.0 => {
                config.trading.lots.assets.insert(coin.clone(), size);
                println!("  → {coin}: 1 lot = {size} units");
            }
            _ => println!("  Invalid size. Must be a positive number."),
        }
    }
    Ok(())
}

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
