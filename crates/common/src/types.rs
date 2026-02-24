//! Universal types shared across all protocol modules.
//!
//! Every module converts its protocol-specific data into these types.
//! CLI/TUI/frontend consume only these — never protocol-specific structs.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Protocol identifier — which DEX/protocol this data comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Hyperliquid,
    Morpho,
    // Future: Dydx, Gmx, Vertex, Jupiter, Drift, ...
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Hyperliquid => write!(f, "hyperliquid"),
            Protocol::Morpho => write!(f, "morpho"),
        }
    }
}

/// Chain identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    Ethereum,
    Arbitrum,
    Base,
    Solana,
    HyperliquidL1,
    // Future: Optimism, Polygon, etc.
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Arbitrum => write!(f, "arbitrum"),
            Chain::Base => write!(f, "base"),
            Chain::Solana => write!(f, "solana"),
            Chain::HyperliquidL1 => write!(f, "hyperliquid-l1"),
        }
    }
}

/// Universal market representation — works for perps across any protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub protocol: Protocol,
    pub chain: Chain,
    pub market_type: MarketType,
    pub mark_price: Option<Decimal>,
    pub index_price: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub open_interest: Option<Decimal>,
    pub funding_rate: Option<Decimal>,
    pub max_leverage: Option<u32>,
    pub min_size: Option<Decimal>,
    pub tick_size: Option<Decimal>,
    pub sz_decimals: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketType {
    Perp,
    Spot,
    Lending,
}

/// Universal candle (OHLCV).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub open_time_ms: u64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub trades: Option<u64>,
}

/// Universal trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub protocol: Protocol,
    pub symbol: String,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
    pub timestamp_ms: u64,
    pub tx_hash: Option<String>,
}

/// Universal order book level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    pub price: Decimal,
    pub size: Decimal,
    pub count: Option<u32>,
}

/// Universal order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub protocol: Protocol,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
    pub timestamp_ms: Option<u64>,
}

/// Universal funding rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    pub symbol: String,
    pub protocol: Protocol,
    pub rate: Decimal,
    pub premium: Option<Decimal>,
    pub timestamp_ms: u64,
    pub next_funding_ms: Option<u64>,
}

/// Universal ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub protocol: Protocol,
    pub mid_price: Decimal,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub change_24h_pct: Option<Decimal>,
}

/// Trade side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// Universal position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub protocol: Protocol,
    pub symbol: String,
    pub side: Side,
    pub size: Decimal,
    pub entry_price: Option<Decimal>,
    pub mark_price: Option<Decimal>,
    pub unrealized_pnl: Option<Decimal>,
    pub leverage: Option<u32>,
    pub margin: Option<Decimal>,
    pub liquidation_price: Option<Decimal>,
}

/// Universal order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub protocol: Protocol,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub size: Decimal,
    pub price: Option<Decimal>,
    pub filled_size: Option<Decimal>,
    pub status: OrderStatus,
    pub order_id: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Open,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

/// Universal fill (executed trade).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub protocol: Protocol,
    pub symbol: String,
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
    pub fee: Decimal,
    pub realized_pnl: Option<Decimal>,
    pub order_id: String,
    pub tx_hash: Option<String>,
    pub timestamp_ms: u64,
}

/// Universal balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub protocol: Protocol,
    pub chain: Chain,
    pub asset: String,
    pub total: Decimal,
    pub available: Decimal,
    pub locked: Decimal,
}

/// Order execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub protocol: Protocol,
    pub order_id: String,
    pub status: OrderStatus,
    pub filled_size: Option<Decimal>,
    pub avg_price: Option<Decimal>,
    pub message: Option<String>,
}
