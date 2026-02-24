mod commands;
mod tui;

use anyhow::Result;
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
enum CliOutputFormat { Table, Json, JsonPretty }

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
    Hyperliquid {
        #[command(subcommand)]
        action: HyperliquidAction,
    },

    /// Morpho Blue: DeFi lending & borrowing.
    Morpho {
        #[command(subcommand)]
        action: MorphoAction,
    },

    /// 0x Protocol: multi-chain DEX aggregator (swaps).
    #[command(name = "zero-x", alias = "0x")]
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
    Generate { name: String },
    /// Import an existing private key.
    Import { name: String },
    /// Switch active profile.
    Use { name: String },
    /// List all profiles.
    List,
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
    Verbose { enabled: bool },
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
        /// Module name (e.g. hyperliquid, morpho).
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
    /// List available markets.
    List {
        #[arg(long, default_value_t = false)]
        spot: bool,
    },

    // ── Technical Analysis (under market) ───────────────────────
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
        ticker: String,
        size: String,
        #[arg(long)] leverage: Option<u32>,
        #[arg(long)] slippage: Option<f64>,
    },
    /// Market sell / short.
    Sell {
        ticker: String,
        size: String,
        #[arg(long)] leverage: Option<u32>,
        #[arg(long)] slippage: Option<f64>,
    },
    /// Close position.
    Close {
        ticker: String,
        #[arg(long)] size: Option<f64>,
        #[arg(long)] slippage: Option<f64>,
    },
    /// Place limit order.
    Order {
        ticker: String,
        side: String,
        size: String,
        price: f64,
        #[arg(long, default_value_t = false)] reduce_only: bool,
    },
    /// Cancel order(s).
    Cancel {
        ticker: String,
        #[arg(long)] oid: Option<u64>,
    },
    /// List open positions.
    Positions,
    /// List open orders.
    Orders,
    /// List recent fills.
    Fills,
    /// Set leverage.
    Leverage { ticker: String, value: u32, #[arg(long, default_value_t = false)] cross: bool },
    /// Update isolated margin.
    Margin { ticker: String, amount: f64 },
    /// Transfer USDC.
    Transfer { amount: String, destination: String },
}

#[derive(Subcommand)]
enum HlSpotAction {
    /// Buy spot token.
    Buy { base: String, size: f64, #[arg(long)] slippage: Option<f64> },
    /// Sell spot token.
    Sell { base: String, size: f64, #[arg(long)] slippage: Option<f64> },
    /// Show balances.
    Balance,
    /// Internal transfer (perps↔spot↔EVM).
    Transfer { direction: String, amount: String, #[arg(long)] token: Option<String> },
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
    Approve { address: String, #[arg(long)] name: Option<String> },
}

// ═══════════════════════════════════════════════════════════════════════
//  MORPHO — Protocol namespace
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum MorphoAction {
    /// List lending markets.
    Markets { #[arg(long, default_value = "ethereum")] chain: String },
    /// Show lending positions.
    Positions,
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
    Prices,
    Trades { ticker: String },
    Book { ticker: String, #[arg(long, default_value_t = 10)] depth: usize },
    Candles { ticker: String, interval: String },
    User,
}

#[derive(Subcommand)]
enum RiskAction {
    Calc {
        coin: String, side: String, entry: f64,
        #[arg(long)] stop: Option<f64>,
        #[arg(long)] leverage: Option<u32>,
    },
    Offline {
        coin: String, side: String, entry: f64, account: f64,
        #[arg(long)] stop: Option<f64>,
        #[arg(long)] leverage: Option<u32>,
    },
}

#[derive(Subcommand)]
enum HistoryAction {
    Trades {
        #[arg(long)] coin: Option<String>,
        #[arg(long)] from: Option<String>,
        #[arg(long)] to: Option<String>,
        #[arg(long, default_value_t = 50)] limit: usize,
    },
    Orders {
        #[arg(long)] coin: Option<String>,
        #[arg(long)] status: Option<String>,
        #[arg(long, default_value_t = 50)] limit: usize,
    },
    Pnl {
        #[arg(long)] coin: Option<String>,
        #[arg(long)] from: Option<String>,
        #[arg(long)] to: Option<String>,
    },
}

#[derive(Subcommand)]
enum ExportAction {
    Trades {
        #[arg(long)] csv: bool, #[arg(long)] json: bool,
        #[arg(long)] coin: Option<String>,
        #[arg(long)] from: Option<String>, #[arg(long)] to: Option<String>,
    },
    Pnl {
        #[arg(long)] csv: bool, #[arg(long)] json: bool,
        #[arg(long)] from: Option<String>, #[arg(long)] to: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  ENTRYPOINT
// ═══════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    atlas_core::init_workspace()?;

    let cli = Cli::parse();
    let fmt: OutputFormat = cli.output.into();

    match cli.command {
        // ── CORE OS ─────────────────────────────────────────────
        Commands::Profile { action } => match action {
            ProfileAction::Generate { name } => commands::auth::new_wallet(&name),
            ProfileAction::Import { name } => commands::auth::import_wallet(&name),
            ProfileAction::Use { name } => commands::auth::switch_profile(&name),
            ProfileAction::List => commands::auth::list_profiles(),
        },

        Commands::Configure { action } => match action {
            ConfigureAction::Show => commands::configure::run(fmt),
            ConfigureAction::System { action } => match action {
                SystemConfigAction::Profile { name } => commands::auth::switch_profile(&name),
                SystemConfigAction::Verbose { enabled } => {
                    let mut config = atlas_core::workspace::load_config()?;
                    config.system.verbose = enabled;
                    atlas_core::workspace::save_config(&config)?;
                    println!("✓ verbose = {enabled}");
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
                TradingConfigAction::Mode { value } => commands::configure::set_mode(&value),
                TradingConfigAction::Size { value } => commands::configure::set_size_mode(&value),
                TradingConfigAction::Leverage { value } => commands::configure::set_leverage(value),
                TradingConfigAction::Slippage { value } => commands::configure::set_slippage(value),
                TradingConfigAction::Lot { coin, size } => commands::configure::set_lot_size(&coin, size),
            },
        },

        Commands::Status => commands::status::run(fmt).await,
        Commands::Doctor { fix } => commands::doctor::run(fix, fmt).await,
        Commands::Tui => tui::run().await,

        // ── MARKET DATA & ANALYTICS ─────────────────────────────
        Commands::Market { action } => match action {
            MarketAction::Price { tickers, all } => commands::market::price(&tickers, all, fmt).await,
            MarketAction::Funding { ticker } => commands::market::funding(&ticker, fmt).await,
            MarketAction::Orderbook { ticker, depth } => commands::market::orderbook(&ticker, depth, fmt).await,
            MarketAction::Candles { ticker, timeframe, limit } => commands::market::candles(&ticker, &timeframe, limit, fmt).await,
            MarketAction::List { spot } => commands::market::markets(spot, fmt).await,
            // TA under market
            MarketAction::Rsi { ticker, timeframe, period } => commands::ta::rsi(&ticker, &timeframe, period, fmt).await,
            MarketAction::Macd { ticker, timeframe } => commands::ta::macd(&ticker, &timeframe, fmt).await,
            MarketAction::Vwap { ticker } => commands::ta::vwap(&ticker, fmt).await,
            MarketAction::Trend { ticker } => commands::ta::trend(&ticker, fmt).await,
        },

        Commands::Stream { action } => match action {
            StreamAction::Prices => commands::stream::stream_prices(fmt).await,
            StreamAction::Trades { ticker } => commands::stream::stream_trades(&ticker, fmt).await,
            StreamAction::Book { ticker, depth } => commands::stream::stream_book(&ticker, depth, fmt).await,
            StreamAction::Candles { ticker, interval } => commands::stream::stream_candles(&ticker, &interval, fmt).await,
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
                    HlPerpAction::Buy { ticker, size, leverage, slippage } => commands::trade::market_buy(&ticker, &size, leverage, slippage, fmt).await,
                    HlPerpAction::Sell { ticker, size, leverage, slippage } => commands::trade::market_sell(&ticker, &size, leverage, slippage, fmt).await,
                    HlPerpAction::Close { ticker, size, slippage } => commands::trade::close_position(&ticker, size, slippage, fmt).await,
                    HlPerpAction::Order { ticker, side, size, price, reduce_only } => commands::trade::limit_order(&ticker, &side, &size, price, reduce_only, "Gtc", fmt).await,
                    HlPerpAction::Cancel { ticker, oid } => commands::trade::cancel(&ticker, oid, fmt).await,
                    HlPerpAction::Positions => commands::status::run(fmt).await,
                    HlPerpAction::Orders => commands::trade::list_orders(fmt).await,
                    HlPerpAction::Fills => commands::trade::list_fills(fmt).await,
                    HlPerpAction::Leverage { ticker, value, cross } => commands::account::set_leverage(&ticker, value, cross, fmt).await,
                    HlPerpAction::Margin { ticker, amount } => commands::account::update_margin(&ticker, amount, fmt).await,
                    HlPerpAction::Transfer { amount, destination } => commands::account::transfer_usdc(&amount, &destination, fmt).await,
                },
                HyperliquidAction::Spot { action } => match action {
                    HlSpotAction::Buy { base, size, slippage } => commands::spot::spot_buy(&base, size, slippage, fmt).await,
                    HlSpotAction::Sell { base, size, slippage } => commands::spot::spot_sell(&base, size, slippage, fmt).await,
                    HlSpotAction::Balance => commands::spot::spot_balance(fmt).await,
                    HlSpotAction::Transfer { direction, amount, token } => commands::spot::spot_transfer(&direction, &amount, token.as_deref(), fmt).await,
                },
                HyperliquidAction::Vault { action } => match action {
                    HlVaultAction::Details { vault } => commands::vault::vault_details(&vault, fmt).await,
                    HlVaultAction::Deposits => commands::vault::vault_deposits(fmt).await,
                },
                HyperliquidAction::Sub { action } => match action {
                    HlSubAction::List => commands::sub::sub_list(fmt).await,
                },
                HyperliquidAction::Agent { action } => match action {
                    HlAgentAction::Approve { address, name } => commands::sub::agent_approve(&address, name.as_deref(), fmt).await,
                },
                HyperliquidAction::Sync { full } => commands::history::run_sync(full, fmt).await,
                HyperliquidAction::Risk { action } => match action {
                    RiskAction::Calc { coin, side, entry, stop, leverage } => commands::risk::calculate(&coin, &side, entry, stop, leverage, fmt).await,
                    RiskAction::Offline { coin, side, entry, account, stop, leverage } => commands::risk::calculate_offline(&coin, &side, entry, account, stop, leverage, fmt),
                },
            }
        },

        // ── MORPHO ──────────────────────────────────────────────
        Commands::Morpho { action } => {
            let config = atlas_core::workspace::load_config()?;
            if !config.modules.morpho.enabled {
                anyhow::bail!("Morpho module is disabled. Run: atlas configure module enable morpho");
            }
            match action {
                MorphoAction::Markets { chain } => commands::morpho::markets(&chain, fmt).await,
                MorphoAction::Positions => commands::morpho::positions(fmt).await,
            }
        },

        // ── UTILITIES ───────────────────────────────────────────
        Commands::History { action } => match action {
            HistoryAction::Trades { coin, from, to, limit } => commands::history::run_trades(coin.as_deref(), from.as_deref(), to.as_deref(), limit, fmt),
            HistoryAction::Orders { coin, status, limit } => commands::history::run_orders(coin.as_deref(), status.as_deref(), limit, fmt),
            HistoryAction::Pnl { coin, from, to } => commands::history::run_pnl(coin.as_deref(), from.as_deref(), to.as_deref(), fmt),
        },

        Commands::Export { action } => match action {
            ExportAction::Trades { csv: _, json, coin, from, to } => commands::export::run_export_trades(json, coin.as_deref(), from.as_deref(), to.as_deref(), fmt),
            ExportAction::Pnl { csv: _, json, from, to } => commands::export::run_export_pnl(json, from.as_deref(), to.as_deref(), fmt),
        },
    }
}
