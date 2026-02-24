mod commands;
mod tui;

use anyhow::Result;
use atlas_common::error::AtlasError;
use atlas_utils::output::OutputFormat;
use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "atlas",
    about = "Atlas OS — DeFi Operating System.\nMulti-protocol, Unix-philosophy CLI. Each command does one thing, outputs JSON for AI agents.",
    version,
    propagate_version = true
)]
struct Cli {
    #[arg(long, short = 'o', global = true, default_value = "table")]
    output: CliOutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliOutputFormat {
    Table,
    Json,
    JsonPretty,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(f: CliOutputFormat) -> OutputFormat {
        match f {
            CliOutputFormat::Table => OutputFormat::Table,
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::JsonPretty => OutputFormat::JsonPretty,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  TOP-LEVEL — Clean hierarchy for 100+ protocol scale
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum Commands {
    // ── CORE OS ─────────────────────────────────────────────────
    /// Manage wallet profiles (generate, import, use, list).
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Configure everything: system, modules, trading.
    Configure {
        #[command(subcommand)]
        action: ConfigureAction,
    },

    /// Print account summary.
    Status,

    /// Check system health.
    Doctor {
        #[arg(long)]
        fix: bool,
    },

    /// Launch interactive Terminal UI.
    Tui,

    // ── MARKET DATA & ANALYTICS ─────────────────────────────────
    /// Market data & technical analysis: price, funding, orderbook, ta.
    Market {
        #[command(subcommand)]
        action: MarketAction,
    },

    /// Stream real-time data via WebSocket.
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },

    // ── PROTOCOL MODULES (namespaced per protocol) ──────────────
    /// Hyperliquid DEX: perp trading, spot trading, vaults.
    #[command(alias = "hl")]
    Hyperliquid {
        #[command(subcommand)]
        action: HyperliquidAction,
    },

    /// 0x Protocol: multi-chain DEX aggregator (swaps).
    #[command(name = "zero-x", alias = "0x", alias = "swap")]
    ZeroX {
        #[command(subcommand)]
        action: ZeroXAction,
    },

    // ── UTILITIES ───────────────────────────────────────────────
    /// Query cached history and PnL.
    History {
        #[command(subcommand)]
        action: HistoryAction,
    },

    /// Export data to CSV/JSON.
    Export {
        #[command(subcommand)]
        action: ExportAction,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  PROFILE
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum ProfileAction {
    /// Generate a new random EVM wallet.
    Generate {
        /// Profile name (e.g. main, trading, test).
        name: String,
    },
    /// Import an existing private key.
    Import {
        /// Profile name.
        name: String,
    },
    /// Switch active profile.
    Use {
        /// Profile name to activate.
        name: String,
    },
    /// List all profiles.
    List,
    /// Export the private key of a profile.
    Export {
        /// Profile name to export.
        name: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  CONFIGURE — Single place for ALL configuration
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum ConfigureAction {
    /// Show all current configuration.
    Show,

    /// System-level settings.
    System {
        #[command(subcommand)]
        action: SystemConfigAction,
    },

    /// Module management: list, enable, disable, set config.
    Module {
        #[command(subcommand)]
        action: ModuleConfigAction,
    },

    /// Trading settings: mode, size, leverage, slippage, lots.
    Trading {
        #[command(subcommand)]
        action: TradingConfigAction,
    },
}

#[derive(Subcommand)]
enum SystemConfigAction {
    /// Set active profile.
    Profile { name: String },
    /// Toggle verbose mode.
    Verbose {
        /// Enable or disable (true/false).
        enabled: String,
    },
    /// Set Atlas backend API key.
    #[command(name = "api-key")]
    ApiKey { key: String },
}

#[derive(Subcommand)]
enum ModuleConfigAction {
    /// List all modules.
    List,
    /// Enable a module.
    Enable { name: String },
    /// Disable a module.
    Disable { name: String },
    /// Set a module config key.
    Set {
        /// Module name (e.g. hyperliquid).
        module: String,
        /// Config key (e.g. network, chain).
        key: String,
        /// Config value.
        value: String,
    },
}

#[derive(Subcommand)]
enum TradingConfigAction {
    /// Set trading mode: futures or cfd.
    Mode { value: String },
    /// Set default size interpretation: usdc, units, or lots.
    Size { value: String },
    /// Set default leverage.
    Leverage { value: u32 },
    /// Set default slippage (e.g. 0.05 = 5%).
    Slippage { value: f64 },
    /// Set lot size for an asset (CFD mode).
    Lot { coin: String, size: f64 },
}

// ═══════════════════════════════════════════════════════════════════════
//  MARKET — Data + Technical Analysis (unified)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum MarketAction {
    // ── Hyperliquid market data (direct) ───────────────────────
    /// Hyperliquid perp/spot market data.
    #[command(alias = "hl")]
    Hyperliquid {
        #[command(subcommand)]
        action: MarketHlAction,
    },

    // ── DEX / onchain market data (via backend → CoinGecko) ───
    /// Onchain DEX market data (pools, tokens, networks).
    Dex {
        #[command(subcommand)]
        action: MarketDexAction,
    },

    // ── Cross-protocol / macro (via backend → CoinGecko) ──────
    /// Global crypto market stats.
    Global,
    /// Trending coins.
    Trending,
    /// Detailed coin info (e.g. bitcoin, ethereum).
    Coin { id: String },
    /// Top gainers & losers across all crypto.
    Movers {
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Global DeFi market stats.
    Defi,
}

/// `atlas market hyperliquid <action>` — Hyperliquid-specific market data.
#[derive(Subcommand)]
enum MarketHlAction {
    /// List available markets.
    List {
        #[arg(long, default_value_t = false)]
        spot: bool,
    },
    /// Get current mid price.
    Price {
        tickers: Vec<String>,
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// Get funding rate history.
    Funding { ticker: String },
    /// Get order book snapshot.
    Orderbook {
        ticker: String,
        #[arg(long, default_value_t = 10)]
        depth: usize,
    },
    /// Get candlestick data.
    Candles {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Detailed market info (price, spread, OI, volume).
    Info { coin: String },
    /// Top markets by volume, gainers, or losers.
    Top {
        #[arg(long, default_value = "volume")]
        sort: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = false)]
        reverse: bool,
    },
    /// Bid-ask spread for one or more coins.
    Spread { coins: Vec<String> },
    /// Search markets by symbol or name.
    Search { query: String },
    /// Quick market dashboard (gainers, losers, volume leaders).
    Summary,

    // ── Technical Analysis (TA-Lib) ──────────────────────────
    /// Calculate RSI.
    Rsi {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 14)]
        period: usize,
    },
    /// Calculate MACD.
    Macd {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
    },
    /// Calculate VWAP.
    Vwap { ticker: String },
    /// Multi-indicator trend signal (bullish/bearish + score).
    Trend { ticker: String },
    /// Bollinger Bands.
    Bbands {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 20)]
        period: usize,
    },
    /// Stochastic oscillator.
    Stoch {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
    },
    /// Average Directional Index (trend strength).
    Adx {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 14)]
        period: usize,
    },
    /// Average True Range (volatility).
    Atr {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 14)]
        period: usize,
    },
    /// Exponential Moving Average.
    Ema {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 20)]
        period: usize,
    },
    /// Simple Moving Average.
    Sma {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 20)]
        period: usize,
    },
    /// On Balance Volume.
    Obv {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
    },
    /// Commodity Channel Index.
    Cci {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 20)]
        period: usize,
    },
    /// Williams %R.
    Willr {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
        #[arg(long, default_value_t = 14)]
        period: usize,
    },
    /// Parabolic SAR.
    Sar {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
    },
    /// Candlestick pattern recognition.
    Patterns {
        ticker: String,
        #[arg(long, default_value = "1h")]
        timeframe: String,
    },
}

/// `atlas market dex <action>` — Onchain DEX data (via CoinGecko).
#[derive(Subcommand)]
enum MarketDexAction {
    /// Trending liquidity pools.
    Trending {
        /// Network filter (ethereum, base, arbitrum, solana...).
        #[arg(long)]
        network: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Newly created pools.
    New {
        #[arg(long)]
        network: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Top pools on a network.
    Pools {
        /// Network (ethereum, base, arbitrum, solana...).
        network: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Pool details by address.
    Pool {
        /// Network (ethereum, base, ...).
        network: String,
        /// Pool contract address.
        address: String,
    },
    /// Token info by address.
    Token {
        /// Network (ethereum, base, ...).
        network: String,
        /// Token contract address.
        address: String,
    },
    /// List supported networks.
    Networks,
    /// List DEXes on a network.
    Dexes { network: String },
    /// Search onchain tokens/pools.
    Search { query: String },
}

// ═══════════════════════════════════════════════════════════════════════
//  HYPERLIQUID — Protocol namespace (perp + spot + vault + sub + risk)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum HyperliquidAction {
    /// Perpetual futures trading.
    Perp {
        #[command(subcommand)]
        action: HlPerpAction,
    },
    /// Spot trading.
    Spot {
        #[command(subcommand)]
        action: HlSpotAction,
    },
    /// Vault management.
    Vault {
        #[command(subcommand)]
        action: HlVaultAction,
    },
    /// Subaccount management.
    Sub {
        #[command(subcommand)]
        action: HlSubAction,
    },
    /// Agent wallet approval.
    Agent {
        #[command(subcommand)]
        action: HlAgentAction,
    },
    /// Sync data to local DB cache.
    Sync {
        #[arg(long)]
        full: bool,
    },
    /// Risk calculator (uses this module's risk config).
    Risk {
        #[command(subcommand)]
        action: RiskAction,
    },
}

#[derive(Subcommand)]
enum HlPerpAction {
    /// Market buy.
    Buy {
        /// Coin symbol (e.g. ETH, BTC, SOL).
        ticker: String,
        /// Size: 200 (default mode), $200 (USDC margin), 0.5eth (units), 10lots.
        size: String,
        /// Override leverage for size calculation.
        #[arg(long)]
        leverage: Option<u32>,
        /// Slippage tolerance (e.g. 0.05 = 5%).
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Market sell / short.
    Sell {
        /// Coin symbol (e.g. ETH, BTC, SOL).
        ticker: String,
        /// Size: 200 (default mode), $200 (USDC margin), 0.5eth (units), 10lots.
        size: String,
        /// Override leverage for size calculation.
        #[arg(long)]
        leverage: Option<u32>,
        /// Slippage tolerance (e.g. 0.05 = 5%).
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Close position.
    Close {
        /// Coin symbol.
        ticker: String,
        /// Partial close size (omit to close full position).
        #[arg(long)]
        size: Option<f64>,
        /// Slippage tolerance.
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Place limit order.
    Order {
        /// Coin symbol.
        ticker: String,
        /// Side: buy/sell (or long/short, b/s).
        side: String,
        /// Size (same formats as buy/sell).
        size: String,
        /// Limit price in USD.
        price: f64,
        /// Close-only order (won't open new positions).
        #[arg(long, default_value_t = false)]
        reduce_only: bool,
    },
    /// Cancel order(s). Without --oid, cancels all orders for the coin.
    Cancel {
        /// Coin symbol.
        ticker: String,
        /// Specific order ID to cancel.
        #[arg(long)]
        oid: Option<u64>,
    },
    /// List open positions.
    Positions,
    /// List open orders.
    Orders,
    /// List recent fills.
    Fills,
    /// Set leverage for a coin.
    Leverage {
        /// Coin symbol.
        ticker: String,
        /// Leverage multiplier (e.g. 10).
        value: u32,
        /// Use cross margin (default: isolated).
        #[arg(long, default_value_t = false)]
        cross: bool,
    },
    /// Update isolated margin for a position.
    Margin {
        /// Coin symbol.
        ticker: String,
        /// Amount to add (positive) or remove (negative).
        amount: f64,
    },
    /// Transfer USDC to another address.
    Transfer {
        /// Amount of USDC.
        amount: String,
        /// Destination EVM address (0x...).
        destination: String,
    },
}

#[derive(Subcommand)]
enum HlSpotAction {
    /// Buy spot token.
    Buy {
        /// Token symbol (e.g. PURR, HYPE).
        base: String,
        /// Amount to buy.
        size: f64,
        /// Slippage tolerance.
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Sell spot token.
    Sell {
        /// Token symbol.
        base: String,
        /// Amount to sell.
        size: f64,
        /// Slippage tolerance.
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Show spot token balances.
    Balance,
    /// Internal transfer (perps↔spot↔EVM).
    Transfer {
        /// Direction: to-spot, to-perps, or to-evm.
        direction: String,
        /// Amount to transfer.
        amount: String,
        /// Token (default: USDC).
        #[arg(long)]
        token: Option<String>,
    },
}

#[derive(Subcommand)]
enum HlVaultAction {
    /// Vault details.
    Details { vault: String },
    /// Your vault deposits.
    Deposits,
}

#[derive(Subcommand)]
enum HlSubAction {
    /// List subaccounts.
    List,
}

#[derive(Subcommand)]
enum HlAgentAction {
    /// Approve agent wallet.
    Approve {
        address: String,
        #[arg(long)]
        name: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  0x — Protocol namespace (multi-chain DEX aggregator)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum ZeroXAction {
    /// Get indicative swap price quote.
    Quote {
        /// Sell token contract address.
        sell_token: String,
        /// Buy token contract address.
        buy_token: String,
        /// Amount to sell (in base units / wei).
        amount: String,
        /// Chain to swap on (ethereum, arbitrum, base).
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Max slippage in basis points (default 100 = 1%).
        #[arg(long)]
        slippage: Option<u32>,
    },
    /// List chains supported by 0x.
    Chains,
    /// List liquidity sources on a chain.
    Sources {
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    /// View completed swap trade analytics.
    Trades {
        /// Start timestamp (unix seconds).
        #[arg(long)]
        start: Option<u64>,
        /// End timestamp (unix seconds).
        #[arg(long)]
        end: Option<u64>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  UTILITIES — Stream, Risk, History, Export
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum StreamAction {
    /// Stream all mid prices in real-time.
    Prices,
    /// Stream trades for a specific coin.
    Trades {
        /// Coin symbol (e.g. BTC, ETH).
        ticker: String,
    },
    /// Stream order book updates for a coin.
    Book {
        /// Coin symbol.
        ticker: String,
        /// Number of price levels per side.
        #[arg(long, default_value_t = 10)]
        depth: usize,
    },
    /// Stream candlestick updates for a coin.
    Candles {
        /// Coin symbol.
        ticker: String,
        /// Candle interval (e.g. 1m, 5m, 1h).
        interval: String,
    },
    /// Stream user account updates (fills, orders).
    User,
}

#[derive(Subcommand)]
enum RiskAction {
    Calc {
        coin: String,
        side: String,
        entry: f64,
        #[arg(long)]
        stop: Option<f64>,
        #[arg(long)]
        leverage: Option<u32>,
    },
    Offline {
        coin: String,
        side: String,
        entry: f64,
        account: f64,
        #[arg(long)]
        stop: Option<f64>,
        #[arg(long)]
        leverage: Option<u32>,
    },
}

#[derive(Subcommand)]
enum HistoryAction {
    Trades {
        /// Filter by protocol (hyperliquid, 0x). Default: all.
        #[arg(long, alias = "proto")]
        protocol: Option<String>,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Orders {
        /// Filter by protocol (hyperliquid, 0x). Default: all.
        #[arg(long, alias = "proto")]
        protocol: Option<String>,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Pnl {
        /// Filter by protocol (hyperliquid, 0x). Default: all.
        #[arg(long, alias = "proto")]
        protocol: Option<String>,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
}

#[derive(Subcommand)]
enum ExportAction {
    Trades {
        #[arg(long, alias = "proto")]
        protocol: Option<String>,
        #[arg(long)]
        csv: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
    Pnl {
        #[arg(long, alias = "proto")]
        protocol: Option<String>,
        #[arg(long)]
        csv: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  ENTRYPOINT
// ═══════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    if let Err(e) = atlas_core::init_workspace() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    let cli = Cli::parse();
    let fmt: OutputFormat = cli.output.into();

    let result = run(cli.command, fmt).await;

    if let Err(e) = result {
        if fmt != OutputFormat::Table {
            // PRD-compliant structured error JSON to stdout for machine consumers
            // Try to extract AtlasError for structured output, else wrap as UNKNOWN_ERROR
            let json = classify_error(&e);
            println!("{}", serde_json::to_string(&json).unwrap_or_default());
            let exit_code = json["error"]["category"]
                .as_str()
                .map(|c| match c {
                    "network" => 2,
                    "system" => 3,
                    _ => 1,
                })
                .unwrap_or(1);
            std::process::exit(exit_code);
        } else {
            eprintln!("Error: {e:#}");
            // Determine exit code from error chain
            let exit_code = exit_code_from_error(&e);
            std::process::exit(exit_code);
        }
    }
}

/// Classify an anyhow error into PRD-compliant structured JSON.
///
/// If the error chain contains an `AtlasError`, use its structured detail.
/// Otherwise, heuristically classify from the error message.
fn classify_error(e: &anyhow::Error) -> serde_json::Value {
    // Try to downcast to AtlasError first
    if let Some(atlas_err) = e.downcast_ref::<AtlasError>() {
        return atlas_err.to_json();
    }

    // Heuristic classification from error message
    let msg = format!("{e:#}");
    let lower = msg.to_lowercase();

    let (code, category, recoverable, hints) = if lower.contains("timeout")
        || lower.contains("connection refused")
        || lower.contains("network")
        || lower.contains("unreachable")
    {
        (
            "NETWORK_ERROR",
            "network",
            true,
            vec!["Check network connectivity", "Retry in a few seconds"],
        )
    } else if lower.contains("keyring") || lower.contains("keystore") {
        (
            "KEYRING_ERROR",
            "auth",
            false,
            vec!["Check OS keyring service is running"],
        )
    } else if lower.contains("no profile") || lower.contains("no wallet") {
        (
            "NO_PROFILE",
            "auth",
            true,
            vec!["Run: atlas profile generate main"],
        )
    } else if lower.contains("config") || lower.contains("atlas.json") {
        (
            "CONFIG_ERROR",
            "config",
            true,
            vec!["Run: atlas doctor --fix"],
        )
    } else if lower.contains("invalid") || lower.contains("parse") {
        (
            "VALIDATION_ERROR",
            "validation",
            true,
            vec!["Check command parameters"],
        )
    } else {
        ("UNKNOWN_ERROR", "system", false, vec![] as Vec<&str>)
    };

    serde_json::json!({
        "ok": false,
        "error": {
            "code": code,
            "message": msg,
            "category": category,
            "recoverable": recoverable,
            "hints": hints,
        }
    })
}

/// Determine exit code from an anyhow error chain.
fn exit_code_from_error(e: &anyhow::Error) -> i32 {
    if let Some(atlas_err) = e.downcast_ref::<AtlasError>() {
        return atlas_err.exit_code();
    }

    let msg = format!("{e:#}").to_lowercase();
    if msg.contains("timeout")
        || msg.contains("network")
        || msg.contains("unreachable")
        || msg.contains("connection refused")
    {
        2 // network
    } else {
        1 // user error (default)
    }
}

async fn run(command: Commands, fmt: OutputFormat) -> Result<()> {
    match command {
        // ── CORE OS ─────────────────────────────────────────────
        Commands::Profile { action } => match action {
            ProfileAction::Generate { name } => commands::auth::generate_wallet(&name, fmt),
            ProfileAction::Import { name } => commands::auth::import_wallet(&name, fmt),
            ProfileAction::Use { name } => commands::auth::switch_profile(&name, fmt),
            ProfileAction::List => commands::auth::list_profiles(fmt),
            ProfileAction::Export { name } => commands::auth::export_wallet(&name, fmt),
        },

        Commands::Configure { action } => match action {
            ConfigureAction::Show => commands::configure::run(fmt),
            ConfigureAction::System { action } => match action {
                SystemConfigAction::Profile { name } => commands::auth::switch_profile(&name, fmt),
                SystemConfigAction::Verbose { enabled } => {
                    let mut config = atlas_core::workspace::load_config()?;
                    let val = enabled.to_lowercase() == "true" || enabled == "1" || enabled == "on";
                    config.system.verbose = val;
                    atlas_core::workspace::save_config(&config)?;
                    if fmt == OutputFormat::Table {
                        println!("✓ verbose = {val}");
                    } else {
                        println!(
                            "{}",
                            serde_json::json!({"ok": true, "data": {"key": "verbose", "value": val}})
                        );
                    }
                    Ok(())
                }
                SystemConfigAction::ApiKey { key } => {
                    let mut config = atlas_core::workspace::load_config()?;
                    config.system.api_key = Some(key.clone());
                    atlas_core::workspace::save_config(&config)?;
                    if fmt == OutputFormat::Table {
                        println!("✓ api_key = {key}");
                    } else {
                        println!(
                            "{}",
                            serde_json::json!({"ok": true, "data": {"key": "api_key", "value": key}})
                        );
                    }
                    Ok(())
                }
            },
            ConfigureAction::Module { action } => match action {
                ModuleConfigAction::List => commands::modules::run(fmt),
                ModuleConfigAction::Enable { name } => commands::modules::enable(&name, fmt),
                ModuleConfigAction::Disable { name } => commands::modules::disable(&name, fmt),
                ModuleConfigAction::Set { module, key, value } => {
                    commands::modules::config_set(&module, &key, &value, fmt)
                }
            },
            ConfigureAction::Trading { action } => match action {
                TradingConfigAction::Mode { value } => commands::configure::set_mode(&value, fmt),
                TradingConfigAction::Size { value } => {
                    commands::configure::set_size_mode(&value, fmt)
                }
                TradingConfigAction::Leverage { value } => {
                    commands::configure::set_leverage(value, fmt)
                }
                TradingConfigAction::Slippage { value } => {
                    commands::configure::set_slippage(value, fmt)
                }
                TradingConfigAction::Lot { coin, size } => {
                    commands::configure::set_lot_size(&coin, size, fmt)
                }
            },
        },

        Commands::Status => commands::status::run(fmt).await,
        Commands::Doctor { fix } => commands::doctor::run(fix, fmt).await,
        Commands::Tui => tui::run().await,

        // ── MARKET DATA & ANALYTICS ─────────────────────────────
        Commands::Market { action } => match action {
            MarketAction::Hyperliquid { action } => match action {
                MarketHlAction::List { spot } => commands::market::markets(spot, fmt).await,
                MarketHlAction::Price { tickers, all } => {
                    commands::market::price(&tickers, all, fmt).await
                }
                MarketHlAction::Funding { ticker } => commands::market::funding(&ticker, fmt).await,
                MarketHlAction::Orderbook { ticker, depth } => {
                    commands::market::orderbook(&ticker, depth, fmt).await
                }
                MarketHlAction::Candles {
                    ticker,
                    timeframe,
                    limit,
                } => commands::market::candles(&ticker, &timeframe, limit, fmt).await,
                MarketHlAction::Info { coin } => commands::market::info(&coin, fmt).await,
                MarketHlAction::Top {
                    sort,
                    limit,
                    reverse,
                } => commands::market::top(&sort, limit, reverse, fmt).await,
                MarketHlAction::Spread { coins } => commands::market::spread(&coins, fmt).await,
                MarketHlAction::Search { query } => commands::market::search(&query, fmt).await,
                MarketHlAction::Summary => commands::market::summary(fmt).await,
                MarketHlAction::Rsi {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::rsi(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Macd { ticker, timeframe } => {
                    commands::ta::macd(&ticker, &timeframe, fmt).await
                }
                MarketHlAction::Vwap { ticker } => commands::ta::vwap(&ticker, fmt).await,
                MarketHlAction::Trend { ticker } => commands::ta::trend(&ticker, fmt).await,
                MarketHlAction::Bbands {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::bbands(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Stoch { ticker, timeframe } => {
                    commands::ta::stoch(&ticker, &timeframe, fmt).await
                }
                MarketHlAction::Adx {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::adx(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Atr {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::atr(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Ema {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::ema(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Sma {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::sma(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Obv { ticker, timeframe } => {
                    commands::ta::obv(&ticker, &timeframe, fmt).await
                }
                MarketHlAction::Cci {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::cci(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Willr {
                    ticker,
                    timeframe,
                    period,
                } => commands::ta::willr(&ticker, &timeframe, period, fmt).await,
                MarketHlAction::Sar { ticker, timeframe } => {
                    commands::ta::sar(&ticker, &timeframe, fmt).await
                }
                MarketHlAction::Patterns { ticker, timeframe } => {
                    commands::ta::patterns(&ticker, &timeframe, fmt).await
                }
            },
            MarketAction::Dex { action } => match action {
                MarketDexAction::Trending { network, limit } => {
                    commands::coingecko::dex_trending(network.as_deref(), limit, fmt).await
                }
                MarketDexAction::New { network, limit } => {
                    commands::coingecko::dex_new(network.as_deref(), limit, fmt).await
                }
                MarketDexAction::Pools { network, limit } => {
                    commands::coingecko::dex_top_pools(&network, limit, fmt).await
                }
                MarketDexAction::Pool { network, address } => {
                    commands::coingecko::dex_pool_detail(&network, &address, fmt).await
                }
                MarketDexAction::Token { network, address } => {
                    commands::coingecko::dex_token_info(&network, &address, fmt).await
                }
                MarketDexAction::Networks => commands::coingecko::dex_networks(fmt).await,
                MarketDexAction::Dexes { network } => {
                    commands::coingecko::dex_dexes(&network, fmt).await
                }
                MarketDexAction::Search { query } => {
                    commands::coingecko::dex_search(&query, fmt).await
                }
            },
            MarketAction::Global => commands::coingecko::global(fmt).await,
            MarketAction::Trending => commands::coingecko::trending(fmt).await,
            MarketAction::Coin { id } => commands::coingecko::coin(&id, fmt).await,
            MarketAction::Movers { limit } => commands::coingecko::movers(limit, fmt).await,
            MarketAction::Defi => commands::coingecko::defi(fmt).await,
        },

        Commands::Stream { action } => match action {
            StreamAction::Prices => commands::stream::stream_prices(fmt).await,
            StreamAction::Trades { ticker } => commands::stream::stream_trades(&ticker, fmt).await,
            StreamAction::Book { ticker, depth } => {
                commands::stream::stream_book(&ticker, depth, fmt).await
            }
            StreamAction::Candles { ticker, interval } => {
                commands::stream::stream_candles(&ticker, &interval, fmt).await
            }
            StreamAction::User => commands::stream::stream_user(fmt).await,
        },

        // ── HYPERLIQUID ─────────────────────────────────────────
        Commands::Hyperliquid { action } => {
            let config = atlas_core::workspace::load_config()?;
            if !config.modules.hyperliquid.enabled {
                anyhow::bail!("Hyperliquid module is disabled. Run: atlas configure module enable hyperliquid");
            }
            match action {
                HyperliquidAction::Perp { action } => match action {
                    HlPerpAction::Buy {
                        ticker,
                        size,
                        leverage,
                        slippage,
                    } => commands::trade::market_buy(&ticker, &size, leverage, slippage, fmt).await,
                    HlPerpAction::Sell {
                        ticker,
                        size,
                        leverage,
                        slippage,
                    } => {
                        commands::trade::market_sell(&ticker, &size, leverage, slippage, fmt).await
                    }
                    HlPerpAction::Close {
                        ticker,
                        size,
                        slippage,
                    } => commands::trade::close_position(&ticker, size, slippage, fmt).await,
                    HlPerpAction::Order {
                        ticker,
                        side,
                        size,
                        price,
                        reduce_only,
                    } => {
                        commands::trade::limit_order(
                            &ticker,
                            &side,
                            &size,
                            price,
                            reduce_only,
                            "Gtc",
                            fmt,
                        )
                        .await
                    }
                    HlPerpAction::Cancel { ticker, oid } => {
                        commands::trade::cancel(&ticker, oid, fmt).await
                    }
                    HlPerpAction::Positions => commands::trade::list_positions(fmt).await,
                    HlPerpAction::Orders => commands::trade::list_orders(fmt).await,
                    HlPerpAction::Fills => commands::trade::list_fills(fmt).await,
                    HlPerpAction::Leverage {
                        ticker,
                        value,
                        cross,
                    } => commands::account::set_leverage(&ticker, value, cross, fmt).await,
                    HlPerpAction::Margin { ticker, amount } => {
                        commands::account::update_margin(&ticker, amount, fmt).await
                    }
                    HlPerpAction::Transfer {
                        amount,
                        destination,
                    } => commands::account::transfer_usdc(&amount, &destination, fmt).await,
                },
                HyperliquidAction::Spot { action } => match action {
                    HlSpotAction::Buy {
                        base,
                        size,
                        slippage,
                    } => commands::spot::spot_buy(&base, size, slippage, fmt).await,
                    HlSpotAction::Sell {
                        base,
                        size,
                        slippage,
                    } => commands::spot::spot_sell(&base, size, slippage, fmt).await,
                    HlSpotAction::Balance => commands::spot::spot_balance(fmt).await,
                    HlSpotAction::Transfer {
                        direction,
                        amount,
                        token,
                    } => {
                        commands::spot::spot_transfer(&direction, &amount, token.as_deref(), fmt)
                            .await
                    }
                },
                HyperliquidAction::Vault { action } => match action {
                    HlVaultAction::Details { vault } => {
                        commands::vault::vault_details(&vault, fmt).await
                    }
                    HlVaultAction::Deposits => commands::vault::vault_deposits(fmt).await,
                },
                HyperliquidAction::Sub { action } => match action {
                    HlSubAction::List => commands::sub::sub_list(fmt).await,
                },
                HyperliquidAction::Agent { action } => match action {
                    HlAgentAction::Approve { address, name } => {
                        commands::sub::agent_approve(&address, name.as_deref(), fmt).await
                    }
                },
                HyperliquidAction::Sync { full } => commands::history::run_sync(full, fmt).await,
                HyperliquidAction::Risk { action } => match action {
                    RiskAction::Calc {
                        coin,
                        side,
                        entry,
                        stop,
                        leverage,
                    } => commands::risk::calculate(&coin, &side, entry, stop, leverage, fmt).await,
                    RiskAction::Offline {
                        coin,
                        side,
                        entry,
                        account,
                        stop,
                        leverage,
                    } => commands::risk::calculate_offline(
                        &coin, &side, entry, account, stop, leverage, fmt,
                    ),
                },
            }
        }

        // ── 0x ─────────────────────────────────────────────────
        Commands::ZeroX { action } => {
            let config = atlas_core::workspace::load_config()?;
            if !config.modules.zero_x.enabled {
                anyhow::bail!("0x module is disabled. Run: atlas configure module enable zero_x");
            }
            match action {
                ZeroXAction::Quote {
                    sell_token,
                    buy_token,
                    amount,
                    chain,
                    slippage,
                } => {
                    commands::zero_x::quote(&sell_token, &buy_token, &amount, &chain, slippage, fmt)
                        .await
                }
                ZeroXAction::Chains => commands::zero_x::chains(fmt).await,
                ZeroXAction::Sources { chain } => commands::zero_x::sources(&chain, fmt).await,
                ZeroXAction::Trades { start, end } => {
                    commands::zero_x::trades(start, end, fmt).await
                }
            }
        }

        // ── UTILITIES ───────────────────────────────────────────
        Commands::History { action } => match action {
            HistoryAction::Trades {
                protocol,
                coin,
                from,
                to,
                limit,
            } => commands::history::run_trades(
                protocol.as_deref(),
                coin.as_deref(),
                from.as_deref(),
                to.as_deref(),
                limit,
                fmt,
            ),
            HistoryAction::Orders {
                protocol,
                coin,
                status,
                limit,
            } => commands::history::run_orders(
                protocol.as_deref(),
                coin.as_deref(),
                status.as_deref(),
                limit,
                fmt,
            ),
            HistoryAction::Pnl {
                protocol,
                coin,
                from,
                to,
            } => commands::history::run_pnl(
                protocol.as_deref(),
                coin.as_deref(),
                from.as_deref(),
                to.as_deref(),
                fmt,
            ),
        },

        Commands::Export { action } => match action {
            ExportAction::Trades {
                protocol,
                csv: _,
                json,
                coin,
                from,
                to,
            } => commands::export::run_export_trades(
                protocol.as_deref(),
                json,
                coin.as_deref(),
                from.as_deref(),
                to.as_deref(),
                fmt,
            ),
            ExportAction::Pnl {
                protocol,
                csv: _,
                json,
                from,
                to,
            } => commands::export::run_export_pnl(
                protocol.as_deref(),
                json,
                from.as_deref(),
                to.as_deref(),
                fmt,
            ),
        },
    }
}
