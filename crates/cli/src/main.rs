mod commands;
mod tui;

use anyhow::Result;
use atlas_utils::output::OutputFormat;
use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::EnvFilter;

// ─── CLI Definition ─────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "atlas",
    about = "Atlas OS — DeFi Operating System.\nUnix-philosophy CLI: each command does one thing, outputs JSON for AI agents.",
    version,
    propagate_version = true
)]
struct Cli {
    /// Output format: table (default), json, json-pretty.
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
//  TOP-LEVEL COMMANDS — Maps to M.0 / M.1 / M.2 / M.3
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum Commands {
    // ── M.0: CORE SYSTEM (Otak OS) ──────────────────────────────

    /// Manage wallet profiles (generate, use, list).
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Manage modules (list, enable, disable, config).
    Module {
        #[command(subcommand)]
        action: ModuleAction,
    },

    /// System configuration.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Check system health: NTP sync, API latency, config integrity.
    Doctor {
        #[arg(long)]
        fix: bool,
    },

    /// Launch the interactive Terminal UI.
    Tui,

    /// Print account summary: profile, balances, positions.
    Status,

    // ── M.1: CORE DATA & ANALYTICS (Mata OS) ────────────────────

    /// Market data: price, funding, orderbook, candles.
    Market {
        #[command(subcommand)]
        action: MarketAction,
    },

    /// Technical analysis: RSI, MACD, VWAP, trend.
    Ta {
        #[command(subcommand)]
        action: TaAction,
    },

    /// Stream real-time data via WebSocket.
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },

    // ── M.2: FINANCIAL PRIMITIVES ───────────────────────────────

    /// EVM operations: balance, send (Ethereum, Base, Arbitrum, etc).
    Evm {
        #[command(subcommand)]
        action: EvmAction,
    },

    // ── M.3: DAPP MODULES (Tangan Eksekutor) ────────────────────

    /// Perpetual trading (Hyperliquid, dYdX, etc).
    Perp {
        #[command(subcommand)]
        action: PerpAction,
    },

    /// Morpho lending protocol: supply, withdraw, borrow.
    Morpho {
        #[command(subcommand)]
        action: MorphoAction,
    },

    /// Spot trading: buy, sell, balance, transfer.
    Spot {
        #[command(subcommand)]
        action: SpotAction,
    },

    // ── Utilities ───────────────────────────────────────────────

    /// Risk calculator: position sizing from risk rules.
    Risk {
        #[command(subcommand)]
        action: RiskAction,
    },

    /// Query cached trade/order history and PnL.
    History {
        #[command(subcommand)]
        action: HistoryAction,
    },

    /// Export cached data to CSV or JSON.
    Export {
        #[command(subcommand)]
        action: ExportAction,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  M.0: CORE SYSTEM — Profile, Module, Config
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum ProfileAction {
    /// Generate a new random EVM wallet profile.
    Generate {
        /// Profile name (e.g. "main", "bot-sniper").
        name: String,
    },
    /// Import an existing private key (hex).
    Import {
        /// Profile name.
        name: String,
    },
    /// Switch the active profile.
    Use {
        /// Profile name to activate.
        name: String,
    },
    /// List all stored profiles.
    List,
}

#[derive(Subcommand)]
enum ModuleAction {
    /// List all modules and their status.
    List,
    /// Enable a module.
    Enable {
        /// Module name (e.g. hyperliquid, morpho).
        name: String,
    },
    /// Disable a module.
    Disable {
        /// Module name.
        name: String,
    },
    /// Set a module config value.
    Config {
        #[command(subcommand)]
        action: ModuleConfigAction,
    },
}

#[derive(Subcommand)]
enum ModuleConfigAction {
    /// Set a config key-value for a module.
    Set {
        /// Module name (e.g. hyperliquid, morpho).
        module: String,
        /// Config key (e.g. network, chain, rpc_url).
        key: String,
        /// Config value.
        value: String,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set trading mode: futures or cfd.
    Mode { value: String },
    /// Set how bare numbers are interpreted: usdc, units, or lots.
    Size { value: String },
    /// Set default leverage.
    Leverage { value: u32 },
    /// Set default slippage (decimal, e.g. 0.05 = 5%).
    Slippage { value: f64 },
    /// Set lot size for an asset (CFD mode).
    Lot { coin: String, size: f64 },
    /// Show current configuration.
    Show,
}

// ═══════════════════════════════════════════════════════════════════════
//  M.1: CORE DATA & ANALYTICS — Market, TA
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum MarketAction {
    /// Get current mid price for a ticker.
    Price {
        /// Ticker symbol(s) (e.g. BTC ETH). Omit for all.
        tickers: Vec<String>,
        #[arg(long, default_value_t = false)]
        all: bool,
    },
    /// Get funding rate history for a ticker.
    Funding {
        /// Ticker symbol (e.g. BTC).
        ticker: String,
    },
    /// Get order book snapshot.
    Orderbook {
        /// Ticker symbol.
        ticker: String,
        /// Depth (number of levels).
        #[arg(long, default_value_t = 10)]
        depth: usize,
    },
    /// Get K-line / candlestick data.
    Candles {
        /// Ticker symbol.
        ticker: String,
        /// Interval: 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w.
        #[arg(long, default_value = "1h")]
        timeframe: String,
        /// Number of candles (default: 50).
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// List all available markets.
    List {
        /// Show spot markets instead of perps.
        #[arg(long, default_value_t = false)]
        spot: bool,
    },
}

#[derive(Subcommand)]
enum TaAction {
    /// Calculate RSI (Relative Strength Index).
    Rsi {
        /// Ticker symbol.
        ticker: String,
        /// Timeframe: 1m, 5m, 15m, 1h, 4h, 1d.
        #[arg(long, default_value = "1h")]
        timeframe: String,
        /// RSI period (default: 14).
        #[arg(long, default_value_t = 14)]
        period: usize,
    },
    /// Calculate MACD.
    Macd {
        /// Ticker symbol.
        ticker: String,
        /// Timeframe.
        #[arg(long, default_value = "1h")]
        timeframe: String,
    },
    /// Calculate VWAP (Volume Weighted Average Price).
    Vwap {
        /// Ticker symbol.
        ticker: String,
    },
    /// Aggregated trend signal (bullish/bearish with confidence score).
    Trend {
        /// Ticker symbol.
        ticker: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  M.2: FINANCIAL PRIMITIVES — EVM
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum EvmAction {
    /// Get token balance on an EVM chain.
    Balance {
        /// Chain: ethereum, base, arbitrum, hyperevm.
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Token symbol (default: ETH for native).
        #[arg(long)]
        token: Option<String>,
    },
    /// Send tokens on an EVM chain.
    Send {
        /// Destination address (0x...).
        to: String,
        /// Amount to send.
        amount: String,
        /// Chain: ethereum, base, arbitrum, hyperevm.
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Token symbol (default: ETH for native).
        #[arg(long)]
        token: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  M.3: DAPP MODULES — Perp, Morpho, Spot
// ═══════════════════════════════════════════════════════════════════════

#[derive(Subcommand)]
enum PerpAction {
    /// Market buy (IOC with slippage).
    Buy {
        /// Ticker (e.g. BTC, ETH).
        ticker: String,
        /// Size: units, lots, or USDC ($200, 200u, 200lots).
        size: String,
        /// Leverage override.
        #[arg(long)]
        leverage: Option<u32>,
        /// Slippage tolerance.
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Market sell / short.
    Sell {
        ticker: String,
        size: String,
        #[arg(long)]
        leverage: Option<u32>,
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Close a position.
    Close {
        ticker: String,
        /// Partial close size (omit for full).
        #[arg(long)]
        size: Option<f64>,
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Place a limit order.
    Order {
        ticker: String,
        /// Side: buy or sell.
        side: String,
        size: String,
        /// Limit price.
        price: f64,
        #[arg(long, default_value_t = false)]
        reduce_only: bool,
    },
    /// Cancel order(s).
    Cancel {
        ticker: String,
        /// Order ID (omit to cancel all).
        #[arg(long)]
        oid: Option<u64>,
    },
    /// List open positions.
    Positions,
    /// List open orders.
    Orders,
    /// List recent fills.
    Fills,
    /// Set leverage.
    Leverage {
        ticker: String,
        value: u32,
        #[arg(long, default_value_t = false)]
        cross: bool,
    },
    /// Update isolated margin.
    Margin {
        ticker: String,
        amount: f64,
    },
    /// Transfer USDC to another address.
    Transfer {
        amount: String,
        destination: String,
    },
    /// Sync fills/orders to local DB cache.
    Sync {
        #[arg(long)]
        full: bool,
    },
    /// Vault: view details and deposits.
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    /// Subaccounts: list and manage.
    Sub {
        #[command(subcommand)]
        action: SubAction,
    },
    /// Agent wallet approval.
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
}

#[derive(Subcommand)]
enum MorphoAction {
    /// List Morpho Blue lending markets.
    Markets {
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    /// Show your lending positions.
    Positions,
    /// Supply collateral.
    Supply {
        /// Asset symbol.
        asset: String,
        /// Amount to supply.
        size: String,
    },
    /// Withdraw collateral.
    Withdraw {
        asset: String,
        size: String,
    },
    /// Borrow.
    Borrow {
        asset: String,
        size: String,
    },
}

#[derive(Subcommand)]
enum SpotAction {
    /// Buy a spot token at market.
    Buy {
        base: String,
        size: f64,
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Sell a spot token at market.
    Sell {
        base: String,
        size: f64,
        #[arg(long)]
        slippage: Option<f64>,
    },
    /// Show spot balances.
    Balance,
    /// Transfer between perps/spot/EVM.
    Transfer {
        /// Direction: to-spot, to-perps, to-evm.
        direction: String,
        amount: String,
        #[arg(long)]
        token: Option<String>,
    },
}

#[derive(Subcommand)]
enum VaultAction {
    Details { vault: String },
    Deposits,
}

#[derive(Subcommand)]
enum SubAction {
    List,
}

#[derive(Subcommand)]
enum AgentAction {
    Approve {
        address: String,
        #[arg(long)]
        name: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  UTILITIES
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
        #[arg(long)] csv: bool,
        #[arg(long)] json: bool,
        #[arg(long)] coin: Option<String>,
        #[arg(long)] from: Option<String>,
        #[arg(long)] to: Option<String>,
    },
    Pnl {
        #[arg(long)] csv: bool,
        #[arg(long)] json: bool,
        #[arg(long)] from: Option<String>,
        #[arg(long)] to: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
//  ENTRYPOINT — Command dispatch
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
        // ── M.0: CORE SYSTEM ────────────────────────────────────
        Commands::Profile { action } => match action {
            ProfileAction::Generate { name } => commands::auth::new_wallet(&name),
            ProfileAction::Import { name } => commands::auth::import_wallet(&name),
            ProfileAction::Use { name } => commands::auth::switch_profile(&name),
            ProfileAction::List => commands::auth::list_profiles(),
        },

        Commands::Module { action } => match action {
            ModuleAction::List => commands::modules::run(fmt),
            ModuleAction::Enable { name } => commands::modules::enable(&name, fmt),
            ModuleAction::Disable { name } => commands::modules::disable(&name, fmt),
            ModuleAction::Config { action: ModuleConfigAction::Set { module, key, value } } => {
                commands::modules::config_set(&module, &key, &value, fmt)
            },
        },

        Commands::Config { action } => match action {
            ConfigAction::Mode { value } => commands::configure::set_mode(&value),
            ConfigAction::Size { value } => commands::configure::set_size_mode(&value),
            ConfigAction::Leverage { value } => commands::configure::set_leverage(value),
            ConfigAction::Slippage { value } => commands::configure::set_slippage(value),
            ConfigAction::Lot { coin, size } => commands::configure::set_lot_size(&coin, size),
            ConfigAction::Show => commands::configure::run(fmt),
        },

        Commands::Doctor { fix } => commands::doctor::run(fix, fmt).await,
        Commands::Tui => tui::run().await,
        Commands::Status => commands::status::run(fmt).await,

        // ── M.1: CORE DATA & ANALYTICS ──────────────────────────
        Commands::Market { action } => match action {
            MarketAction::Price { tickers, all } => {
                commands::market::price(&tickers, all, fmt).await
            }
            MarketAction::Funding { ticker } => {
                commands::market::funding(&ticker, fmt).await
            }
            MarketAction::Orderbook { ticker, depth } => {
                commands::market::orderbook(&ticker, depth, fmt).await
            }
            MarketAction::Candles { ticker, timeframe, limit } => {
                commands::market::candles(&ticker, &timeframe, limit, fmt).await
            }
            MarketAction::List { spot } => {
                commands::market::markets(spot, fmt).await
            }
        },

        Commands::Ta { action } => match action {
            TaAction::Rsi { ticker, timeframe, period } => {
                commands::ta::rsi(&ticker, &timeframe, period, fmt).await
            }
            TaAction::Macd { ticker, timeframe } => {
                commands::ta::macd(&ticker, &timeframe, fmt).await
            }
            TaAction::Vwap { ticker } => {
                commands::ta::vwap(&ticker, fmt).await
            }
            TaAction::Trend { ticker } => {
                commands::ta::trend(&ticker, fmt).await
            }
        },

        Commands::Stream { action } => match action {
            StreamAction::Prices => commands::stream::stream_prices(fmt).await,
            StreamAction::Trades { ticker } => commands::stream::stream_trades(&ticker, fmt).await,
            StreamAction::Book { ticker, depth } => commands::stream::stream_book(&ticker, depth, fmt).await,
            StreamAction::Candles { ticker, interval } => commands::stream::stream_candles(&ticker, &interval, fmt).await,
            StreamAction::User => commands::stream::stream_user(fmt).await,
        },

        // ── M.2: FINANCIAL PRIMITIVES ───────────────────────────
        Commands::Evm { action } => match action {
            EvmAction::Balance { chain, token } => {
                commands::evm::balance(&chain, token.as_deref(), fmt).await
            }
            EvmAction::Send { to, amount, chain, token } => {
                commands::evm::send(&to, &amount, &chain, token.as_deref(), fmt).await
            }
        },

        // ── M.3: DAPP MODULES ───────────────────────────────────
        Commands::Perp { action } => {
            let config = atlas_core::workspace::load_config()?;
            if !config.modules.hyperliquid.enabled {
                anyhow::bail!("Perp module (Hyperliquid) is disabled. Run: atlas module enable hyperliquid");
            }
            match action {
                PerpAction::Buy { ticker, size, leverage, slippage } => {
                    commands::trade::market_buy(&ticker, &size, leverage, slippage, fmt).await
                }
                PerpAction::Sell { ticker, size, leverage, slippage } => {
                    commands::trade::market_sell(&ticker, &size, leverage, slippage, fmt).await
                }
                PerpAction::Close { ticker, size, slippage } => {
                    commands::trade::close_position(&ticker, size, slippage, fmt).await
                }
                PerpAction::Order { ticker, side, size, price, reduce_only } => {
                    commands::trade::limit_order(&ticker, &side, &size, price, reduce_only, "Gtc", fmt).await
                }
                PerpAction::Cancel { ticker, oid } => {
                    commands::trade::cancel(&ticker, oid, fmt).await
                }
                PerpAction::Positions => commands::status::run(fmt).await,
                PerpAction::Orders => commands::trade::list_orders(fmt).await,
                PerpAction::Fills => commands::trade::list_fills(fmt).await,
                PerpAction::Leverage { ticker, value, cross } => {
                    commands::account::set_leverage(&ticker, value, cross, fmt).await
                }
                PerpAction::Margin { ticker, amount } => {
                    commands::account::update_margin(&ticker, amount, fmt).await
                }
                PerpAction::Transfer { amount, destination } => {
                    commands::account::transfer_usdc(&amount, &destination, fmt).await
                }
                PerpAction::Sync { full } => {
                    commands::history::run_sync(full, fmt).await
                }
                PerpAction::Vault { action } => match action {
                    VaultAction::Details { vault } => commands::vault::vault_details(&vault, fmt).await,
                    VaultAction::Deposits => commands::vault::vault_deposits(fmt).await,
                },
                PerpAction::Sub { action } => match action {
                    SubAction::List => commands::sub::sub_list(fmt).await,
                },
                PerpAction::Agent { action } => match action {
                    AgentAction::Approve { address, name } => {
                        commands::sub::agent_approve(&address, name.as_deref(), fmt).await
                    }
                },
            }
        },

        Commands::Morpho { action } => {
            let config = atlas_core::workspace::load_config()?;
            if !config.modules.morpho.enabled {
                anyhow::bail!("Morpho module is disabled. Run: atlas module enable morpho");
            }
            match action {
                MorphoAction::Markets { chain } => commands::morpho::markets(&chain, fmt).await,
                MorphoAction::Positions => commands::morpho::positions(fmt).await,
                MorphoAction::Supply { asset, size } => {
                    commands::morpho::supply(&asset, &size, fmt).await
                }
                MorphoAction::Withdraw { asset, size } => {
                    commands::morpho::withdraw(&asset, &size, fmt).await
                }
                MorphoAction::Borrow { asset, size } => {
                    commands::morpho::borrow(&asset, &size, fmt).await
                }
            }
        },

        Commands::Spot { action } => match action {
            SpotAction::Buy { base, size, slippage } => {
                commands::spot::spot_buy(&base, size, slippage, fmt).await
            }
            SpotAction::Sell { base, size, slippage } => {
                commands::spot::spot_sell(&base, size, slippage, fmt).await
            }
            SpotAction::Balance => commands::spot::spot_balance(fmt).await,
            SpotAction::Transfer { direction, amount, token } => {
                commands::spot::spot_transfer(&direction, &amount, token.as_deref(), fmt).await
            }
        },

        // ── Utilities ───────────────────────────────────────────
        Commands::Risk { action } => match action {
            RiskAction::Calc { coin, side, entry, stop, leverage } => {
                commands::risk::calculate(&coin, &side, entry, stop, leverage, fmt).await
            }
            RiskAction::Offline { coin, side, entry, account, stop, leverage } => {
                commands::risk::calculate_offline(&coin, &side, entry, account, stop, leverage, fmt)
            }
        },

        Commands::History { action } => match action {
            HistoryAction::Trades { coin, from, to, limit } => {
                commands::history::run_trades(coin.as_deref(), from.as_deref(), to.as_deref(), limit, fmt)
            }
            HistoryAction::Orders { coin, status, limit } => {
                commands::history::run_orders(coin.as_deref(), status.as_deref(), limit, fmt)
            }
            HistoryAction::Pnl { coin, from, to } => {
                commands::history::run_pnl(coin.as_deref(), from.as_deref(), to.as_deref(), fmt)
            }
        },

        Commands::Export { action } => match action {
            ExportAction::Trades { csv: _, json, coin, from, to } => {
                commands::export::run_export_trades(json, coin.as_deref(), from.as_deref(), to.as_deref(), fmt)
            }
            ExportAction::Pnl { csv: _, json, from, to } => {
                commands::export::run_export_pnl(json, from.as_deref(), to.as_deref(), fmt)
            }
        },
    }
}
