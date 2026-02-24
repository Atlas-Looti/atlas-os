//! Protocol traits — every module implements these.
//!
//! This is the contract between core and modules. The core engine
//! dispatches commands to the appropriate module via these traits.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use crate::error::AtlasResult;
use crate::types::*;

/// Core trading operations — perp protocols (Hyperliquid, dYdX, GMX, etc.)
#[async_trait]
pub trait PerpModule: Send + Sync {
    /// Protocol identifier.
    fn protocol(&self) -> Protocol;

    /// List available perp markets.
    async fn markets(&self) -> AtlasResult<Vec<Market>>;

    /// Get ticker for a symbol.
    async fn ticker(&self, symbol: &str) -> AtlasResult<Ticker>;

    /// Get all tickers.
    async fn all_tickers(&self) -> AtlasResult<Vec<Ticker>>;

    /// Get candle data.
    async fn candles(
        &self,
        symbol: &str,
        interval: &str,
        limit: usize,
    ) -> AtlasResult<Vec<Candle>>;

    /// Get funding rate history.
    async fn funding(&self, symbol: &str) -> AtlasResult<Vec<FundingRate>>;

    /// Get order book.
    async fn orderbook(&self, symbol: &str, depth: usize) -> AtlasResult<OrderBook>;

    /// Place a market order.
    async fn market_order(
        &self,
        symbol: &str,
        side: Side,
        size: Decimal,
        slippage: Option<f64>,
    ) -> AtlasResult<OrderResult>;

    /// Place a limit order.
    async fn limit_order(
        &self,
        symbol: &str,
        side: Side,
        size: Decimal,
        price: Decimal,
        reduce_only: bool,
    ) -> AtlasResult<OrderResult>;

    /// Close a position.
    async fn close_position(
        &self,
        symbol: &str,
        size: Option<Decimal>,
        slippage: Option<f64>,
    ) -> AtlasResult<OrderResult>;

    /// Cancel an order by ID.
    async fn cancel_order(&self, symbol: &str, order_id: &str) -> AtlasResult<()>;

    /// Cancel all orders on a symbol.
    async fn cancel_all(&self, symbol: &str) -> AtlasResult<u32>;

    /// Get open orders.
    async fn open_orders(&self) -> AtlasResult<Vec<Order>>;

    /// Get positions.
    async fn positions(&self) -> AtlasResult<Vec<Position>>;

    /// Get recent fills.
    async fn fills(&self) -> AtlasResult<Vec<Fill>>;

    /// Get account balances.
    async fn balances(&self) -> AtlasResult<Vec<Balance>>;

    /// Set leverage.
    async fn set_leverage(
        &self,
        symbol: &str,
        leverage: u32,
        is_cross: bool,
    ) -> AtlasResult<()>;

    /// Update isolated margin for a position.
    async fn update_margin(&self, symbol: &str, amount: Decimal) -> AtlasResult<()>;

    /// Transfer USDC.
    async fn transfer(&self, amount: Decimal, destination: &str) -> AtlasResult<String>;

    /// Cancel an order by client order ID.
    async fn cancel_by_cloid(&self, symbol: &str, cloid: &str) -> AtlasResult<()> {
        // Default: fall back to cancel_order if not supported
        self.cancel_order(symbol, cloid).await
    }

    // ── Spot operations (optional — not all perp protocols have spot) ──

    /// Get spot token balances. Returns empty vec if not supported.
    async fn spot_balances(&self) -> AtlasResult<Vec<SpotBalance>> {
        Ok(vec![])
    }

    /// Place a spot market order. Returns error if not supported.
    async fn spot_market_order(
        &self,
        _base: &str,
        _side: Side,
        _size: Decimal,
        _slippage: Option<f64>,
    ) -> AtlasResult<OrderResult> {
        Err(crate::error::AtlasError::Other("Spot trading not supported on this protocol".into()))
    }

    /// Internal transfer between sub-wallets (perps↔spot↔evm).
    async fn internal_transfer(
        &self,
        _direction: &str,
        _amount: Decimal,
        _token: Option<&str>,
    ) -> AtlasResult<String> {
        Err(crate::error::AtlasError::Other("Internal transfers not supported on this protocol".into()))
    }

    // ── Vault / subaccount operations (optional) ────────────────

    /// Get vault details.
    async fn vault_details(&self, _vault_address: &str) -> AtlasResult<VaultDetails> {
        Err(crate::error::AtlasError::Other("Vaults not supported on this protocol".into()))
    }

    /// Get user's vault deposits.
    async fn vault_deposits(&self) -> AtlasResult<Vec<VaultDeposit>> {
        Ok(vec![])
    }

    /// List subaccounts.
    async fn subaccounts(&self) -> AtlasResult<Vec<SubAccount>> {
        Ok(vec![])
    }

    /// Approve an agent wallet.
    async fn approve_agent(&self, _agent_address: &str, _name: Option<&str>) -> AtlasResult<String> {
        Err(crate::error::AtlasError::Other("Agent approval not supported on this protocol".into()))
    }
}

/// Market data provider — read-only, no auth needed.
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    fn protocol(&self) -> Protocol;
    async fn markets(&self) -> AtlasResult<Vec<Market>>;
    async fn ticker(&self, symbol: &str) -> AtlasResult<Ticker>;
    async fn all_tickers(&self) -> AtlasResult<Vec<Ticker>>;
    async fn candles(&self, symbol: &str, interval: &str, limit: usize) -> AtlasResult<Vec<Candle>>;
    async fn funding(&self, symbol: &str) -> AtlasResult<Vec<FundingRate>>;
}

/// Lending protocol operations — Morpho, Aave, Compound, etc.
#[async_trait]
pub trait LendingModule: Send + Sync {
    fn protocol(&self) -> Protocol;

    /// List available lending markets.
    async fn markets(&self) -> AtlasResult<Vec<LendingMarket>>;

    /// Get user's supply/borrow positions.
    async fn positions(&self, user: &str) -> AtlasResult<Vec<LendingPosition>>;

    /// Supply collateral.
    async fn supply(&self, market_id: &str, amount: Decimal) -> AtlasResult<String>;

    /// Withdraw collateral.
    async fn withdraw(&self, market_id: &str, amount: Decimal) -> AtlasResult<String>;

    /// Borrow.
    async fn borrow(&self, market_id: &str, amount: Decimal) -> AtlasResult<String>;

    /// Repay.
    async fn repay(&self, market_id: &str, amount: Decimal) -> AtlasResult<String>;
}

/// Lending market info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingMarket {
    pub protocol: Protocol,
    pub chain: Chain,
    pub market_id: String,
    pub collateral_asset: String,
    pub loan_asset: String,
    pub supply_apy: Decimal,
    pub borrow_apy: Decimal,
    pub total_supply: Decimal,
    pub total_borrow: Decimal,
    pub utilization: Decimal,
    pub ltv: Decimal,
    pub lltv: Decimal,
}

/// Lending position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingPosition {
    pub protocol: Protocol,
    pub chain: Chain,
    pub market_id: String,
    pub collateral_asset: String,
    pub loan_asset: String,
    pub supplied: Decimal,
    pub borrowed: Decimal,
    pub health_factor: Option<Decimal>,
}
