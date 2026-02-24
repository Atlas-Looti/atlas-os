//! Hyperliquid protocol module — implements PerpModule trait.

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use anyhow::Result;
use async_trait::async_trait;
use hypersdk::hypercore::{
    self as hypercore,
    types::{
        api::Action,
        BatchCancel, BatchOrder, Cancel, OrderGrouping, OrderRequest,
        OrderResponseStatus, OrderTypePlacement, TimeInForce,
        CandleInterval,
    },
    Cloid, HttpClient, NonceHandler, PerpMarket,
};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use atlas_common::constants::*;
use atlas_common::error::*;
use atlas_common::traits::PerpModule;
use atlas_common::types::*;

use crate::convert::*;
use crate::signing::compute_agent_signing_hash;

/// Builder fee payload injected into order JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuilderFee {
    b: String,
    f: u16,
}

impl Default for BuilderFee {
    fn default() -> Self {
        Self {
            b: BUILDER_ADDRESS_EVM.to_string(),
            f: BUILDER_FEE_BPS,
        }
    }
}

/// Generate a random client order ID.
fn random_cloid() -> Cloid {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    alloy::primitives::B128::from(bytes)
}

/// Parse candle interval string to SDK enum.
fn parse_interval(s: &str) -> Result<CandleInterval, AtlasError> {
    match s {
        "1m" => Ok(CandleInterval::OneMinute),
        "3m" => Ok(CandleInterval::ThreeMinutes),
        "5m" => Ok(CandleInterval::FiveMinutes),
        "15m" => Ok(CandleInterval::FifteenMinutes),
        "30m" => Ok(CandleInterval::ThirtyMinutes),
        "1h" => Ok(CandleInterval::OneHour),
        "2h" => Ok(CandleInterval::TwoHours),
        "4h" => Ok(CandleInterval::FourHours),
        "8h" => Ok(CandleInterval::EightHours),
        "12h" => Ok(CandleInterval::TwelveHours),
        "1d" => Ok(CandleInterval::OneDay),
        "3d" => Ok(CandleInterval::ThreeDays),
        "1w" => Ok(CandleInterval::OneWeek),
        "1M" => Ok(CandleInterval::OneMonth),
        _ => Err(AtlasError::Other(format!("Invalid interval: {s}"))),
    }
}

fn interval_to_ms(i: &CandleInterval) -> u64 {
    match i {
        CandleInterval::OneMinute => 60_000,
        CandleInterval::ThreeMinutes => 180_000,
        CandleInterval::FiveMinutes => 300_000,
        CandleInterval::FifteenMinutes => 900_000,
        CandleInterval::ThirtyMinutes => 1_800_000,
        CandleInterval::OneHour => 3_600_000,
        CandleInterval::TwoHours => 7_200_000,
        CandleInterval::FourHours => 14_400_000,
        CandleInterval::EightHours => 28_800_000,
        CandleInterval::TwelveHours => 43_200_000,
        CandleInterval::OneDay => 86_400_000,
        CandleInterval::ThreeDays => 259_200_000,
        CandleInterval::OneWeek => 604_800_000,
        CandleInterval::OneMonth => 2_592_000_000,
    }
}

/// The Hyperliquid module — wraps the SDK and implements PerpModule.
pub struct HyperliquidModule {
    pub client: HttpClient,
    pub signer: PrivateKeySigner,
    pub nonce: NonceHandler,
    pub perps: Vec<PerpMarket>,
    pub address: Address,
    pub testnet: bool,
}

impl HyperliquidModule {
    /// Create from signer and network config.
    pub async fn new(
        signer: PrivateKeySigner,
        testnet: bool,
    ) -> Result<Self, AtlasError> {
        let address = signer.address();
        let client = if testnet {
            hypercore::testnet()
        } else {
            hypercore::mainnet()
        };

        let perps = client.perps().await
            .map_err(|e| AtlasError::Network(format!("Failed to fetch markets: {e}")))?;

        let nonce = NonceHandler::default();

        info!(%address, testnet, markets = perps.len(), "Hyperliquid module ready");

        Ok(Self { client, signer, nonce, perps, address, testnet })
    }

    /// Resolve coin name to market index.
    fn resolve_asset(&self, coin: &str) -> Result<usize, AtlasError> {
        self.perps
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(coin))
            .map(|m| m.index)
            .ok_or_else(|| AtlasError::AssetNotFound(coin.to_string()))
    }

    /// Get PerpMarket for a coin.
    fn get_market(&self, coin: &str) -> Result<&PerpMarket, AtlasError> {
        self.perps
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(coin))
            .ok_or_else(|| AtlasError::AssetNotFound(coin.to_string()))
    }

    /// Round price to valid tick.
    fn round_price(&self, coin: &str, price: Decimal) -> Result<Decimal, AtlasError> {
        let market = self.get_market(coin)?;
        market.round_price(price)
            .ok_or_else(|| AtlasError::Other(format!("Invalid price {price} for {coin}")))
    }

    /// Round size to valid lot step.
    fn round_size(&self, coin: &str, size: Decimal) -> Result<Decimal, AtlasError> {
        let market = self.get_market(coin)?;
        let dp = market.sz_decimals.max(0) as u32;
        Ok(size.round_dp(dp))
    }

    /// Chain identifier for signing.
    fn chain(&self) -> hypercore::Chain {
        if self.testnet { hypercore::Chain::Testnet } else { hypercore::Chain::Mainnet }
    }

    /// Base URL for direct HTTP requests.
    fn base_url(&self) -> &str {
        if self.testnet { HL_TESTNET_RPC } else { HL_MAINNET_RPC }
    }

    /// Place a batch order with builder fee injection.
    async fn place_with_builder(
        &self,
        batch: BatchOrder,
    ) -> Result<Vec<OrderResponseStatus>, AtlasError> {
        let nonce = self.nonce.next();
        let action: Action = batch.into();
        let signed = action
            .sign_sync(&self.signer, nonce, None, None, self.chain())
            .map_err(|e| AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("Sign failed: {e}"),
            })?;

        let mut json_val = serde_json::to_value(&signed)
            .map_err(|e| AtlasError::Other(format!("Serialize failed: {e}")))?;

        // Inject builder fee
        let builder = BuilderFee::default();
        if let Some(action_obj) = json_val.get_mut("action") {
            action_obj["builder"] = serde_json::to_value(&builder)
                .map_err(|e| AtlasError::Other(e.to_string()))?;
        }

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        let resp = http
            .post(format!("{}/exchange", self.base_url()))
            .json(&json_val)
            .send()
            .await
            .map_err(|e| AtlasError::Network(format!("Exchange request failed: {e}")))?;

        let status = resp.status();
        let body = resp.text().await
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("HTTP {status}: {body}"),
            });
        }

        let parsed: Value = serde_json::from_str(&body)
            .map_err(|_| AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("Bad response: {body}"),
            })?;

        if parsed.get("status").and_then(|v| v.as_str()) == Some("err") {
            let msg = parsed.get("response")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(AtlasError::OrderRejected(msg.to_string()));
        }

        let statuses_val = parsed.pointer("/response/data/statuses")
            .ok_or_else(|| AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("No statuses: {body}"),
            })?;

        serde_json::from_value(statuses_val.clone())
            .map_err(|e| AtlasError::Other(format!("Parse statuses: {e}")))
    }

    /// Parse SDK order response to universal OrderResult.
    fn parse_response(&self, statuses: &[OrderResponseStatus]) -> AtlasResult<OrderResult> {
        if statuses.is_empty() {
            return Err(AtlasError::Other("Empty response".into()));
        }

        match &statuses[0] {
            OrderResponseStatus::Filled { total_sz, avg_px, oid } => Ok(OrderResult {
                protocol: Protocol::Hyperliquid,
                order_id: oid.to_string(),
                status: OrderStatus::Filled,
                filled_size: Some(*total_sz),
                avg_price: Some(*avg_px),
                message: None,
            }),
            OrderResponseStatus::Resting { oid, .. } => Ok(OrderResult {
                protocol: Protocol::Hyperliquid,
                order_id: oid.to_string(),
                status: OrderStatus::Open,
                filled_size: None,
                avg_price: None,
                message: None,
            }),
            OrderResponseStatus::Success => Ok(OrderResult {
                protocol: Protocol::Hyperliquid,
                order_id: "0".into(),
                status: OrderStatus::Filled,
                filled_size: None,
                avg_price: None,
                message: Some("accepted".into()),
            }),
            OrderResponseStatus::Error(msg) => {
                Err(AtlasError::OrderRejected(msg.clone()))
            }
        }
    }
}

#[async_trait]
impl PerpModule for HyperliquidModule {
    fn protocol(&self) -> Protocol {
        Protocol::Hyperliquid
    }

    async fn markets(&self) -> AtlasResult<Vec<Market>> {
        Ok(self.perps.iter().map(perp_market_to_universal).collect())
    }

    async fn ticker(&self, symbol: &str) -> AtlasResult<Ticker> {
        let mids = self.client.all_mids(None).await
            .map_err(|e| AtlasError::Network(format!("Fetch mids: {e}")))?;

        let mid = mids.get(symbol)
            .ok_or_else(|| AtlasError::AssetNotFound(symbol.to_string()))?;

        Ok(mid_to_ticker(symbol, mid))
    }

    async fn all_tickers(&self) -> AtlasResult<Vec<Ticker>> {
        let mids = self.client.all_mids(None).await
            .map_err(|e| AtlasError::Network(format!("Fetch mids: {e}")))?;

        let mut tickers: Vec<Ticker> = mids.iter()
            .map(|(coin, mid)| mid_to_ticker(coin, mid))
            .collect();
        tickers.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        Ok(tickers)
    }

    async fn candles(
        &self,
        symbol: &str,
        interval: &str,
        limit: usize,
    ) -> AtlasResult<Vec<Candle>> {
        let ci = parse_interval(interval)?;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap()
            .as_millis() as u64;
        let start = now_ms.saturating_sub(interval_to_ms(&ci) * limit as u64);

        let raw = self.client.candle_snapshot(symbol, ci, start, now_ms).await
            .map_err(|e| AtlasError::Network(format!("Fetch candles: {e}")))?;

        Ok(raw.iter().map(|c| Candle {
            open_time_ms: c.open_time,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
            trades: Some(c.num_trades),
        }).collect())
    }

    async fn funding(&self, symbol: &str) -> AtlasResult<Vec<FundingRate>> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap()
            .as_millis() as u64;
        let start = now_ms.saturating_sub(7 * 86_400_000);

        let rates = self.client.funding_history(symbol, start, Some(now_ms)).await
            .map_err(|e| AtlasError::Network(format!("Fetch funding: {e}")))?;

        Ok(rates.iter().map(|r| FundingRate {
            symbol: r.coin.clone(),
            protocol: Protocol::Hyperliquid,
            rate: r.funding_rate,
            premium: Some(r.premium),
            timestamp_ms: r.time,
            next_funding_ms: None,
        }).collect())
    }

    async fn orderbook(&self, _symbol: &str, _depth: usize) -> AtlasResult<OrderBook> {
        // L2Book is WebSocket-only on Hyperliquid
        Err(AtlasError::Other("Orderbook is WebSocket-only. Use `atlas stream book`.".into()))
    }

    async fn market_order(
        &self,
        symbol: &str,
        side: Side,
        size: Decimal,
        slippage: Option<f64>,
    ) -> AtlasResult<OrderResult> {
        let asset = self.resolve_asset(symbol)?;
        let is_buy = side_to_is_buy(&side);
        let slip = slippage.unwrap_or(0.05);

        let mids = self.client.all_mids(None).await
            .map_err(|e| AtlasError::Network(format!("Fetch mids: {e}")))?;
        let mid = mids.get(symbol)
            .ok_or_else(|| AtlasError::AssetNotFound(symbol.to_string()))?;

        let slip_dec = Decimal::from_f64(slip)
            .ok_or_else(|| AtlasError::Other("Invalid slippage".into()))?;
        let mult = if is_buy { Decimal::ONE + slip_dec } else { Decimal::ONE - slip_dec };
        let px = self.round_price(symbol, *mid * mult)?;
        let sz = self.round_size(symbol, size)?;

        if sz.is_zero() {
            return Err(AtlasError::Other(format!("Size rounds to zero for {symbol}")));
        }

        info!(
            symbol, side = %side, %sz, %px, slippage = slip,
            builder = BUILDER_ADDRESS_EVM, fee_bps = BUILDER_FEE_BPS,
            "HL market order with builder fee"
        );

        let order = OrderRequest {
            asset, is_buy, reduce_only: false,
            limit_px: px, sz, cloid: random_cloid(),
            order_type: OrderTypePlacement::Limit { tif: TimeInForce::Ioc },
        };

        let batch = BatchOrder { orders: vec![order], grouping: OrderGrouping::Na };
        let statuses = self.place_with_builder(batch).await?;
        self.parse_response(&statuses)
    }

    async fn limit_order(
        &self,
        symbol: &str,
        side: Side,
        size: Decimal,
        price: Decimal,
        reduce_only: bool,
    ) -> AtlasResult<OrderResult> {
        let asset = self.resolve_asset(symbol)?;
        let is_buy = side_to_is_buy(&side);
        let px = self.round_price(symbol, price)?;
        let sz = self.round_size(symbol, size)?;

        if sz.is_zero() {
            return Err(AtlasError::Other(format!("Size rounds to zero for {symbol}")));
        }

        info!(
            symbol, side = %side, %sz, %px, reduce_only,
            builder = BUILDER_ADDRESS_EVM, fee_bps = BUILDER_FEE_BPS,
            "HL limit order with builder fee"
        );

        let order = OrderRequest {
            asset, is_buy, reduce_only,
            limit_px: px, sz, cloid: random_cloid(),
            order_type: OrderTypePlacement::Limit { tif: TimeInForce::Gtc },
        };

        let batch = BatchOrder { orders: vec![order], grouping: OrderGrouping::Na };
        let statuses = self.place_with_builder(batch).await?;
        self.parse_response(&statuses)
    }

    async fn close_position(
        &self,
        symbol: &str,
        size: Option<Decimal>,
        slippage: Option<f64>,
    ) -> AtlasResult<OrderResult> {
        let asset = self.resolve_asset(symbol)?;
        let slip = slippage.unwrap_or(0.05);

        let state = self.client.clearinghouse_state(self.address, None).await
            .map_err(|e| AtlasError::Network(format!("Fetch state: {e}")))?;

        let position = state.asset_positions.iter()
            .find(|p| p.position.coin.eq_ignore_ascii_case(symbol))
            .ok_or_else(|| AtlasError::Other(format!("No position for {symbol}")))?;

        let pos_size = position.position.szi;
        if pos_size.is_zero() {
            return Err(AtlasError::Other(format!("Position size is zero for {symbol}")));
        }

        let is_long = pos_size > Decimal::ZERO;
        let is_buy = !is_long;

        let close_size = match size {
            Some(s) => self.round_size(symbol, s.min(pos_size.abs()))?,
            None => pos_size.abs(),
        };

        let mids = self.client.all_mids(None).await
            .map_err(|e| AtlasError::Network(format!("Fetch mids: {e}")))?;
        let mid = mids.get(symbol)
            .ok_or_else(|| AtlasError::AssetNotFound(symbol.to_string()))?;

        let slip_dec = Decimal::from_f64(slip)
            .ok_or_else(|| AtlasError::Other("Invalid slippage".into()))?;
        let mult = if is_buy { Decimal::ONE + slip_dec } else { Decimal::ONE - slip_dec };
        let px = self.round_price(symbol, *mid * mult)?;

        let order = OrderRequest {
            asset, is_buy, reduce_only: true,
            limit_px: px, sz: close_size, cloid: random_cloid(),
            order_type: OrderTypePlacement::Limit { tif: TimeInForce::Ioc },
        };

        let batch = BatchOrder { orders: vec![order], grouping: OrderGrouping::Na };
        let statuses = self.place_with_builder(batch).await?;
        self.parse_response(&statuses)
    }

    async fn cancel_order(&self, symbol: &str, order_id: &str) -> AtlasResult<()> {
        let asset = self.resolve_asset(symbol)?;
        let oid: u64 = order_id.parse()
            .map_err(|_| AtlasError::Other(format!("Invalid OID: {order_id}")))?;

        let batch = BatchCancel { cancels: vec![Cancel { asset, oid }] };
        self.client.cancel(&self.signer, batch, self.nonce.next(), None, None).await
            .map_err(|e| AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("Cancel failed: {}", e.message()),
            })?;
        Ok(())
    }

    async fn cancel_all(&self, symbol: &str) -> AtlasResult<u32> {
        let asset = self.resolve_asset(symbol)?;
        let orders = self.client.open_orders(self.address, None).await
            .map_err(|e| AtlasError::Network(format!("Fetch orders: {e}")))?;

        let matching: Vec<_> = orders.iter()
            .filter(|o| o.coin.eq_ignore_ascii_case(symbol))
            .collect();

        if matching.is_empty() { return Ok(0); }

        let cancels: Vec<Cancel> = matching.iter()
            .map(|o| Cancel { asset, oid: o.oid })
            .collect();
        let total = cancels.len() as u32;

        let batch = BatchCancel { cancels };
        let _ = self.client.cancel(&self.signer, batch, self.nonce.next(), None, None).await;

        Ok(total)
    }

    async fn open_orders(&self) -> AtlasResult<Vec<Order>> {
        let orders = self.client.open_orders(self.address, None).await
            .map_err(|e| AtlasError::Network(format!("Fetch orders: {e}")))?;

        Ok(orders.iter().map(|o| Order {
            protocol: Protocol::Hyperliquid,
            symbol: o.coin.clone(),
            side: convert_side(&o.side),
            order_type: OrderType::Limit,
            size: o.sz,
            price: Some(o.limit_px),
            filled_size: None,
            status: OrderStatus::Open,
            order_id: o.oid.to_string(),
            timestamp_ms: o.timestamp,
        }).collect())
    }

    async fn positions(&self) -> AtlasResult<Vec<Position>> {
        let state = self.client.clearinghouse_state(self.address, None).await
            .map_err(|e| AtlasError::Network(format!("Fetch state: {e}")))?;

        Ok(state.asset_positions.iter().map(|ap| {
            let p = &ap.position;
            let is_long = p.szi > Decimal::ZERO;
            Position {
                protocol: Protocol::Hyperliquid,
                symbol: p.coin.clone(),
                side: if is_long { Side::Buy } else { Side::Sell },
                size: p.szi.abs(),
                entry_price: p.entry_px,
                mark_price: None,
                unrealized_pnl: Some(p.unrealized_pnl),
                leverage: Some(p.leverage.value.to_u32().unwrap_or(1)),
                margin: None,
                liquidation_price: p.liquidation_px,
            }
        }).collect())
    }

    async fn fills(&self) -> AtlasResult<Vec<Fill>> {
        let fills = self.client.user_fills(self.address).await
            .map_err(|e| AtlasError::Network(format!("Fetch fills: {e}")))?;

        Ok(fills.iter().take(50).map(|f| Fill {
            protocol: Protocol::Hyperliquid,
            symbol: f.coin.clone(),
            side: convert_side(&f.side),
            price: f.px,
            size: f.sz,
            fee: f.fee,
            realized_pnl: Some(f.closed_pnl),
            order_id: f.oid.to_string(),
            tx_hash: Some(f.hash.clone()),
            timestamp_ms: f.time,
        }).collect())
    }

    async fn balances(&self) -> AtlasResult<Vec<Balance>> {
        let state = self.client.clearinghouse_state(self.address, None).await
            .map_err(|e| AtlasError::Network(format!("Fetch state: {e}")))?;

        Ok(vec![Balance {
            protocol: Protocol::Hyperliquid,
            chain: Chain::HyperliquidL1,
            asset: "USDC".into(),
            total: state.margin_summary.account_value,
            available: state.withdrawable,
            locked: state.margin_summary.total_margin_used,
        }])
    }

    async fn set_leverage(
        &self,
        symbol: &str,
        leverage: u32,
        is_cross: bool,
    ) -> AtlasResult<()> {
        let asset = self.resolve_asset(symbol)?;

        let action_json = serde_json::json!({
            "type": "updateLeverage",
            "asset": asset,
            "isCross": is_cross,
            "leverage": leverage
        });

        let nonce = self.nonce.next();
        let mut rmp_bytes = rmp_serde::to_vec_named(&action_json)
            .map_err(|e| AtlasError::Other(format!("RMP serialize: {e}")))?;
        rmp_bytes.extend(nonce.to_be_bytes());
        rmp_bytes.push(0u8);

        let connection_id = alloy::primitives::keccak256(&rmp_bytes);
        let source = if self.testnet { "b" } else { "a" };
        let agent_hash = compute_agent_signing_hash(source, connection_id);

        let sig = self.signer.sign_hash_sync(&agent_hash)
            .map_err(|e| AtlasError::Auth(format!("Sign failed: {e}")))?;

        let r_hex = hex::encode(sig.r().to_be_bytes::<32>());
        let s_hex = hex::encode(sig.s().to_be_bytes::<32>());
        let v = if sig.v() { 28u8 } else { 27u8 };

        let request_body = serde_json::json!({
            "action": action_json,
            "nonce": nonce,
            "signature": { "r": format!("0x{r_hex}"), "s": format!("0x{s_hex}"), "v": v },
            "vaultAddress": null
        });

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        let resp = http
            .post(format!("{}/exchange", self.base_url()))
            .json(&request_body)
            .send().await
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        let body = resp.text().await
            .map_err(|e| AtlasError::Network(e.to_string()))?;

        let parsed: Value = serde_json::from_str(&body)
            .map_err(|_| AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("Bad response: {body}"),
            })?;

        if parsed.get("status").and_then(|v| v.as_str()) == Some("err") {
            let msg = parsed.get("response").and_then(|v| v.as_str()).unwrap_or("Unknown");
            return Err(AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    async fn transfer(&self, amount: Decimal, destination: &str) -> AtlasResult<String> {
        let dest: Address = destination.parse()
            .map_err(|_| AtlasError::Other(format!("Invalid address: {destination}")))?;

        let send = hypersdk::hypercore::types::UsdSend {
            destination: dest,
            amount,
            time: self.nonce.next(),
        };

        self.client.send_usdc(&self.signer, send, self.nonce.next()).await
            .map_err(|e| AtlasError::Protocol {
                protocol: "hyperliquid".into(),
                message: format!("Transfer failed: {e}"),
            })?;

        Ok(format!("Transferred {} USDC to {}", amount, destination))
    }
}
