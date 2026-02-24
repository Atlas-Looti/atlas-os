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
    about = "Atlas OS — Professional-grade CLI trading engine for Hyperliquid L1",
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

/// CLI-parseable output format (maps to internal OutputFormat).
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

#[derive(Subcommand)]
enum Commands {
    /// Print account summary: profile, balances, positions.
    Status,

    /// Check system health: NTP sync, API latency, config integrity.
    Doctor {
        /// Attempt automatic fixes for detected issues.
        #[arg(long)]
        fix: bool,
    },

    /// Launch the interactive Terminal UI.
    Tui,

    /// Manage wallet profiles and authentication.
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Place a limit order.
    Order {
        /// Asset symbol (e.g. ETH, BTC).
        coin: String,
        /// Order side: buy or sell.
        side: String,
        /// Order size: units, lots (CFD), or USDC ($200, 200$, 200u).
        size: String,
        /// Limit price.
        price: f64,
        /// Reduce-only flag.
        #[arg(long, default_value_t = false)]
        reduce_only: bool,
        /// Time-in-force: Gtc, Ioc, Alo.
        #[arg(long, default_value = "Gtc")]
        tif: String,
    },

    /// Open a market order (IOC with slippage).
    Buy {
        /// Asset symbol (e.g. ETH, BTC).
        coin: String,
        /// Size: units, lots, or USDC ($200, 200$, 200u, 200usdc).
        size: String,
        /// Leverage override (used with USDC sizing).
        #[arg(long)]
        leverage: Option<u32>,
        /// Slippage tolerance (default: 0.05 = 5%).
        #[arg(long)]
        slippage: Option<f64>,
    },

    /// Short-sell at market.
    Sell {
        /// Asset symbol.
        coin: String,
        /// Size: units, lots, or USDC ($200, 200$, 200u, 200usdc).
        size: String,
        /// Leverage override (used with USDC sizing).
        #[arg(long)]
        leverage: Option<u32>,
        /// Slippage tolerance.
        #[arg(long)]
        slippage: Option<f64>,
    },

    /// Close a position at market.
    Close {
        /// Asset symbol.
        coin: String,
        /// Size to close (default: entire position).
        #[arg(long)]
        size: Option<f64>,
        /// Slippage tolerance.
        #[arg(long)]
        slippage: Option<f64>,
    },

    /// Cancel an order or all orders on an asset.
    Cancel {
        /// Asset symbol.
        coin: String,
        /// Order ID to cancel. Omit to cancel all orders on this asset.
        #[arg(long)]
        oid: Option<u64>,
    },

    /// List open orders.
    Orders,

    /// List recent fills (trade history).
    Fills,

    /// Set leverage for an asset.
    Leverage {
        /// Asset symbol.
        coin: String,
        /// Leverage multiplier.
        value: u32,
        /// Use cross margin (default: isolated).
        #[arg(long, default_value_t = false)]
        cross: bool,
    },

    /// Update isolated margin for a position.
    Margin {
        /// Asset symbol.
        coin: String,
        /// Amount to add (positive) or remove (negative).
        amount: f64,
    },

    /// Transfer USDC to another address.
    Transfer {
        /// Amount in USDC.
        amount: String,
        /// Destination address (0x...).
        destination: String,
    },

    // ── Market Data ─────────────────────────────────────────────
    /// Get current mid prices.
    Price {
        /// Asset symbols (e.g. ETH BTC SOL). Omit for all.
        coins: Vec<String>,
        /// Show all prices.
        #[arg(long, default_value_t = false)]
        all: bool,
    },

    /// List available markets.
    Markets {
        /// Show spot markets instead of perps.
        #[arg(long, default_value_t = false)]
        spot: bool,
    },

    /// Get K-line / candlestick data.
    Candles {
        /// Asset symbol (e.g. ETH, BTC).
        coin: String,
        /// Interval: 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w, etc.
        interval: String,
        /// Number of candles to fetch (default: 50).
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },

    /// Get funding rate history for an asset.
    Funding {
        /// Asset symbol (e.g. ETH, BTC).
        coin: String,
    },

    // ── Streaming ───────────────────────────────────────────────
    /// Stream real-time data via WebSocket.
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },

    /// Configure trading settings (mode, leverage, slippage, lots).
    Configure {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },

    /// Calculate position size based on risk management rules.
    Risk {
        #[command(subcommand)]
        action: RiskAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set trading mode: futures (raw size) or cfd (lot-based).
    Mode {
        /// Mode: futures or cfd.
        value: String,
    },
    /// Set how bare numbers are interpreted: usdc (default), units, or lots.
    Size {
        /// Size mode: usdc, units, or lots.
        value: String,
    },
    /// Set default leverage.
    Leverage {
        /// Leverage multiplier.
        value: u32,
    },
    /// Set default slippage (decimal, e.g. 0.05 = 5%).
    Slippage {
        /// Slippage value.
        value: f64,
    },
    /// Set lot size for an asset (CFD mode).
    Lot {
        /// Asset symbol (e.g. BTC, ETH).
        coin: String,
        /// Units of asset per 1 standard lot.
        size: f64,
    },
    /// Show current configuration.
    Show,
}

#[derive(Subcommand)]
enum RiskAction {
    /// Calculate risk-managed position size (connects to Hyperliquid for account data).
    Calc {
        /// Asset symbol (e.g. ETH, BTC).
        coin: String,
        /// Trade side: buy/sell/long/short.
        side: String,
        /// Entry price.
        entry: f64,
        /// Stop-loss price (optional — uses default % if omitted).
        #[arg(long)]
        stop: Option<f64>,
        /// Leverage (optional — uses config default).
        #[arg(long)]
        leverage: Option<u32>,
    },
    /// Calculate offline (no connection required).
    Offline {
        /// Asset symbol.
        coin: String,
        /// Trade side.
        side: String,
        /// Entry price.
        entry: f64,
        /// Account value in USDC.
        account: f64,
        /// Stop-loss price.
        #[arg(long)]
        stop: Option<f64>,
        /// Leverage.
        #[arg(long)]
        leverage: Option<u32>,
    },
}

#[derive(Subcommand)]
enum StreamAction {
    /// Stream live mid prices for all markets.
    Prices,
    /// Stream live trades for a coin.
    Trades {
        /// Asset symbol (e.g. ETH, BTC).
        coin: String,
    },
    /// Stream live order book for a coin.
    Book {
        /// Asset symbol.
        coin: String,
        /// Number of levels to show (default: 10).
        #[arg(long, default_value_t = 10)]
        depth: usize,
    },
    /// Stream live candle updates for a coin.
    Candles {
        /// Asset symbol.
        coin: String,
        /// Interval (e.g. 1m, 5m, 1h, 1d).
        interval: String,
    },
    /// Stream user events (fills, order updates, liquidations).
    User,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Generate a new random EVM wallet.
    New {
        /// Profile name (e.g. "bot-sniper").
        name: String,
    },
    /// Import an existing private key (hex).
    Import {
        /// Profile name.
        name: String,
    },
    /// Switch the active profile.
    Switch {
        /// Profile name to activate.
        name: String,
    },
    /// List all stored profiles.
    List,
}

// ─── Entrypoint ─────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging (controlled via RUST_LOG env var).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    // Ensure the dotfolder workspace exists before any command runs.
    atlas_core::init_workspace()?;

    let cli = Cli::parse();
    let fmt: OutputFormat = cli.output.into();

    match cli.command {
        // ── Info commands ────────────────────────────────────────
        Commands::Status => commands::status::run(fmt).await,
        Commands::Doctor { fix } => commands::doctor::run(fix, fmt).await,
        Commands::Tui => tui::run().await,
        Commands::Auth { action } => match action {
            AuthAction::New { name } => commands::auth::new_wallet(&name),
            AuthAction::Import { name } => commands::auth::import_wallet(&name),
            AuthAction::Switch { name } => commands::auth::switch_profile(&name),
            AuthAction::List => commands::auth::list_profiles(),
        },

        // ── Trading commands ────────────────────────────────────
        Commands::Order { coin, side, size, price, reduce_only, tif } => {
            commands::trade::limit_order(&coin, &side, &size, price, reduce_only, &tif, fmt).await
        }
        Commands::Buy { coin, size, leverage, slippage } => {
            commands::trade::market_buy(&coin, &size, leverage, slippage, fmt).await
        }
        Commands::Sell { coin, size, leverage, slippage } => {
            commands::trade::market_sell(&coin, &size, leverage, slippage, fmt).await
        }
        Commands::Close { coin, size, slippage } => {
            commands::trade::close_position(&coin, size, slippage, fmt).await
        }
        Commands::Cancel { coin, oid } => {
            commands::trade::cancel(&coin, oid, fmt).await
        }
        Commands::Orders => commands::trade::list_orders(fmt).await,
        Commands::Fills => commands::trade::list_fills(fmt).await,

        // ── Account management ──────────────────────────────────
        Commands::Leverage { coin, value, cross } => {
            commands::account::set_leverage(&coin, value, cross, fmt).await
        }
        Commands::Margin { coin, amount } => {
            commands::account::update_margin(&coin, amount, fmt).await
        }
        Commands::Transfer { amount, destination } => {
            commands::account::transfer_usdc(&amount, &destination, fmt).await
        }

        // ── Market data ─────────────────────────────────────────
        Commands::Price { coins, all } => {
            commands::market::price(&coins, all, fmt).await
        }
        Commands::Markets { spot } => {
            commands::market::markets(spot, fmt).await
        }
        Commands::Candles { coin, interval, limit } => {
            commands::market::candles(&coin, &interval, limit, fmt).await
        }
        Commands::Funding { coin } => {
            commands::market::funding(&coin, fmt).await
        }

        // ── Streaming ───────────────────────────────────────────
        Commands::Stream { action } => match action {
            StreamAction::Prices => commands::stream::stream_prices(fmt).await,
            StreamAction::Trades { coin } => commands::stream::stream_trades(&coin, fmt).await,
            StreamAction::Book { coin, depth } => commands::stream::stream_book(&coin, depth, fmt).await,
            StreamAction::Candles { coin, interval } => commands::stream::stream_candles(&coin, &interval, fmt).await,
            StreamAction::User => commands::stream::stream_user(fmt).await,
        },

        // ── Configuration ───────────────────────────────────────
        Commands::Configure { action } => match action {
            None => commands::configure::run(fmt),
            Some(ConfigAction::Mode { value }) => commands::configure::set_mode(&value),
            Some(ConfigAction::Size { value }) => commands::configure::set_size_mode(&value),
            Some(ConfigAction::Leverage { value }) => commands::configure::set_leverage(value),
            Some(ConfigAction::Slippage { value }) => commands::configure::set_slippage(value),
            Some(ConfigAction::Lot { coin, size }) => commands::configure::set_lot_size(&coin, size),
            Some(ConfigAction::Show) => commands::configure::run(fmt),
        },

        // ── Risk calculator ─────────────────────────────────────
        Commands::Risk { action } => match action {
            RiskAction::Calc { coin, side, entry, stop, leverage } => {
                commands::risk::calculate(&coin, &side, entry, stop, leverage, fmt).await
            }
            RiskAction::Offline { coin, side, entry, account, stop, leverage } => {
                commands::risk::calculate_offline(&coin, &side, entry, account, stop, leverage, fmt)
            }
        },
    }
}
