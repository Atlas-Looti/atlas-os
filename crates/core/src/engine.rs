use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use anyhow::{bail, Context, Result};
use hypersdk::hypercore::{
    self as hypercore,
    types::{
        api::{Action, UpdateIsolatedMargin, Response as ExchangeResponse},
        BatchOrder, OrderGrouping, OrderRequest, OrderResponseStatus, OrderTypePlacement,
        TimeInForce, Side, BatchCancel, Cancel, BatchCancelCloid,
        CancelByCloid, UsdSend, CandleInterval,
    },
    Cloid, HttpClient, NonceHandler, PerpMarket,
};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use atlas_types::config::{AppConfig, SizeInput};
use atlas_types::engine::{BuilderFee, BUILDER_ADDRESS, BUILDER_FEE_BPS};
use atlas_types::output::{
    CancelOutput, CancelSingleOutput, CandleRow, CandlesOutput, FillRow,
    FillsOutput, FundingOutput, FundingRow, LeverageOutput, MarginOutput,
    MarketRow, MarketsOutput, OrderResultOutput, OrderRow, OrdersOutput,
    PositionRow, PriceOutput, PriceRow, StatusOutput, TransferOutput,
};

use crate::auth::AuthManager;

/// Generate a random client order ID (Cloid / B128).
fn random_cloid() -> Cloid {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    alloy::primitives::B128::from(bytes)
}

/// Result of an order operation — extracted from SDK response.
#[derive(Debug, Clone)]
pub struct OrderResult {
    pub oid: u64,
    pub status: OrderFillStatus,
    pub output: OrderResultOutput,
}

#[derive(Debug, Clone)]
pub enum OrderFillStatus {
    Filled,
    Resting,
    Error(String),
}

/// The core trading engine. Wraps Hyperliquid SDK (hypersdk) and enforces
/// builder fee injection on every order.
pub struct Engine {
    /// HTTP client for API calls (single unified client).
    pub client: HttpClient,
    /// Signer for EIP-712 signed requests.
    pub signer: PrivateKeySigner,
    /// Thread-safe nonce generator.
    pub nonce: NonceHandler,
    /// Cached perpetual markets for name→index resolution.
    pub perps: Vec<PerpMarket>,
    /// The active profile name.
    pub profile_name: String,
    /// The public address of the active wallet.
    pub address: Address,
    /// Whether we're on testnet.
    pub testnet: bool,
    /// App config — used for trading mode (futures vs CFD) and lot sizing.
    pub config: AppConfig,
}

impl Engine {
    /// Initialize the engine from the active profile.
    ///
    /// Reads `config.toml` → resolves the profile → fetches the private key
    /// from the OS keyring → connects to Hyperliquid.
    pub async fn from_active_profile() -> Result<Self> {
        let config = crate::workspace::load_config()?;
        let profile_name = config.general.active_profile.clone();

        info!(profile = %profile_name, "initializing engine");

        let signer = AuthManager::get_active_signer()
            .context("Failed to load signer for active profile")?;
        let address = signer.address();

        let testnet = config.network.testnet;
        let client = if testnet {
            hypercore::testnet()
        } else {
            hypercore::mainnet()
        };

        // Cache perp markets for name→index lookup and price ticking
        let perps = client.perps().await
            .context("Failed to fetch perpetual markets")?;

        let nonce = NonceHandler::default();

        info!(%address, testnet, markets = perps.len(), "engine ready (hypersdk)");

        Ok(Self {
            client,
            signer,
            nonce,
            perps,
            profile_name,
            address,
            testnet,
            config,
        })
    }

    // ═══════════════════════════════════════════════════════════════════
    //  MARKET RESOLUTION
    // ═══════════════════════════════════════════════════════════════════

    /// Resolve a coin name (e.g. "BTC") to its market index for the API.
    pub fn resolve_asset(&self, coin: &str) -> Result<usize> {
        self.perps
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(coin))
            .map(|m| m.index)
            .ok_or_else(|| anyhow::anyhow!("Unknown perpetual market: {coin}"))
    }

    /// Get the PerpMarket struct for a coin.
    pub fn get_market(&self, coin: &str) -> Result<&PerpMarket> {
        self.perps
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(coin))
            .ok_or_else(|| anyhow::anyhow!("Unknown perpetual market: {coin}"))
    }

    /// Round a price to the valid tick for this market.
    pub fn round_price(&self, coin: &str, price: Decimal) -> Result<Decimal> {
        let market = self.get_market(coin)?;
        market
            .round_price(price)
            .ok_or_else(|| anyhow::anyhow!("Invalid price {price} for {coin}"))
    }

    /// Round a size to the valid lot step for this market.
    /// Hyperliquid perps allow `sz_decimals` decimal places for sizes.
    pub fn round_size(&self, coin: &str, size: Decimal) -> Result<Decimal> {
        let market = self.get_market(coin)?;
        let dp = market.sz_decimals.max(0) as u32;
        Ok(size.round_dp(dp))
    }

    // ═══════════════════════════════════════════════════════════════════
    //  INFO QUERIES
    // ═══════════════════════════════════════════════════════════════════

    /// Get the current mark/mid price for an asset.
    pub async fn get_mark_price(&self, coin: &str) -> Result<f64> {
        let all_mids = self.client.all_mids(None).await
            .context("Failed to fetch mids")?;

        let mid = all_mids.get(coin)
            .ok_or_else(|| anyhow::anyhow!("No price data for {coin}"))?;

        mid.to_f64()
            .ok_or_else(|| anyhow::anyhow!("Cannot convert Decimal to f64 for {coin}"))
    }

    /// Resolve a SizeInput to asset units, fetching mark price if needed.
    /// Returns `(asset_size, display_string)`.
    pub async fn resolve_size_input(
        &self,
        coin: &str,
        input: &SizeInput,
        leverage: Option<u32>,
    ) -> Result<(f64, String)> {
        let lev = leverage.unwrap_or(self.config.trading.default_leverage).max(1);

        // Determine if we need mark price (USDC mode or Raw with USDC default)
        let needs_price = matches!(input,
            SizeInput::Usdc(_) |
            SizeInput::Raw(_)
        ) && (matches!(input, SizeInput::Usdc(_)) ||
              self.config.trading.default_size_mode == atlas_types::config::SizeMode::Usdc);

        if needs_price {
            let mark = self.get_mark_price(coin).await?;
            let (size, margin) = self.config.resolve_size_input(coin, input, mark, Some(lev));

            let display = if let Some(m) = margin {
                let notional = size * mark;
                format!(
                    "${:.2} × {}x = ${:.2} notional → {:.6} {} @ ${:.2}",
                    m, lev, notional, size, coin, mark
                )
            } else {
                self.config.format_size(coin, size)
            };
            Ok((size, display))
        } else {
            let (size, _) = self.config.resolve_size_input(coin, input, 0.0, Some(lev));
            let display = match input {
                SizeInput::Units(u) => format!("{} {}", u, coin),
                SizeInput::Lots(l) => format!("{:.4} lots → {:.6} {}", l, size, coin),
                SizeInput::Raw(raw) => match self.config.trading.default_size_mode {
                    atlas_types::config::SizeMode::Units => self.config.format_size(coin, size),
                    atlas_types::config::SizeMode::Lots => format!("{:.4} lots → {:.6} {}", raw, size, coin),
                    _ => self.config.format_size(coin, size),
                },
                _ => self.config.format_size(coin, size),
            };
            Ok((size, display))
        }
    }

    /// Fetch account summary and print it.
    pub async fn print_account_summary(&self) -> Result<()> {
        let output = self.get_account_summary().await?;
        // Use table display from output module
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  ACCOUNT SUMMARY                                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Profile     : {:<41}║", output.profile);
        println!("║  Address     : {:<41}║", output.address);
        println!("║  Account Val : {:<41}║", output.account_value);
        println!("║  Margin Used : {:<41}║", output.margin_used);
        println!("║  Net Pos     : {:<41}║", output.net_position);
        println!("║  Withdrawable: {:<41}║", output.withdrawable);
        println!("╠══════════════════════════════════════════════════════════╣");

        if output.positions.is_empty() {
            println!("║  No open positions.                                      ║");
        } else {
            println!("║  {:^6} │ {:^10} │ {:^10} │ {:^12} ║", "Coin", "Size", "Entry", "uPnL");
            println!("║  ──────┼────────────┼────────────┼────────────── ║");
            for pos in &output.positions {
                println!(
                    "║  {:^6} │ {:>10} │ {:>10} │ {:>12} ║",
                    pos.coin,
                    pos.size,
                    pos.entry_price,
                    pos.unrealized_pnl,
                );
            }
        }
        println!("╚══════════════════════════════════════════════════════════╝");
        Ok(())
    }

    /// Get account summary as structured data.
    pub async fn get_account_summary(&self) -> Result<StatusOutput> {
        let state = self.client.clearinghouse_state(self.address, None).await
            .context("Failed to fetch account state")?;

        let positions: Vec<PositionRow> = state.asset_positions.iter().map(|pos| {
            let p = &pos.position;
            PositionRow {
                coin: p.coin.clone(),
                size: p.szi.to_string(),
                entry_price: p.entry_px.map(|e| e.to_string()).unwrap_or_else(|| "—".to_string()),
                unrealized_pnl: p.unrealized_pnl.to_string(),
            }
        }).collect();

        Ok(StatusOutput {
            profile: self.profile_name.clone(),
            address: format!("{}", self.address),
            network: if self.testnet { "Testnet".to_string() } else { "Mainnet".to_string() },
            account_value: state.margin_summary.account_value.to_string(),
            margin_used: state.margin_summary.total_margin_used.to_string(),
            net_position: state.margin_summary.total_ntl_pos.to_string(),
            withdrawable: state.withdrawable.to_string(),
            positions,
        })
    }

    /// Print open orders.
    pub async fn print_open_orders(&self) -> Result<()> {
        let output = self.get_open_orders().await?;

        if output.orders.is_empty() {
            println!("No open orders.");
            return Ok(());
        }

        println!("┌────────┬──────┬────────────┬──────────────┬────────────────┐");
        println!("│ Coin   │ Side │ Size       │ Price        │ OID            │");
        println!("├────────┼──────┼────────────┼──────────────┼────────────────┤");
        for o in &output.orders {
            println!(
                "│ {:<6} │ {:<4} │ {:>10} │ {:>12} │ {:>14} │",
                o.coin, o.side, o.size, o.price, o.oid
            );
        }
        println!("└────────┴──────┴────────────┴──────────────┴────────────────┘");
        Ok(())
    }

    /// Get open orders as structured data.
    pub async fn get_open_orders(&self) -> Result<OrdersOutput> {
        let orders = self.client.open_orders(self.address, None).await
            .context("Failed to fetch open orders")?;

        let rows = orders.iter().map(|o| {
            let side = match o.side {
                Side::Bid => "BUY",
                Side::Ask => "SELL",
            };
            OrderRow {
                coin: o.coin.clone(),
                side: side.to_string(),
                size: o.sz.to_string(),
                price: o.limit_px.to_string(),
                oid: o.oid,
            }
        }).collect();

        Ok(OrdersOutput { orders: rows })
    }

    /// Print user fills (recent trades).
    pub async fn print_fills(&self) -> Result<()> {
        let output = self.get_fills().await?;

        if output.fills.is_empty() {
            println!("No recent fills.");
            return Ok(());
        }

        println!("┌────────┬──────┬────────────┬──────────────┬──────────────┬──────────┐");
        println!("│ Coin   │ Side │ Size       │ Price        │ Closed PnL   │ Fee      │");
        println!("├────────┼──────┼────────────┼──────────────┼──────────────┼──────────┤");
        for f in &output.fills {
            println!(
                "│ {:<6} │ {:<4} │ {:>10} │ {:>12} │ {:>12} │ {:>8} │",
                f.coin, f.side, f.size, f.price, f.closed_pnl, f.fee
            );
        }
        println!("└────────┴──────┴────────────┴──────────────┴──────────────┴──────────┘");
        Ok(())
    }

    /// Get fills as structured data.
    pub async fn get_fills(&self) -> Result<FillsOutput> {
        let fills = self.client.user_fills(self.address).await
            .context("Failed to fetch fills")?;

        let rows = fills.iter().take(20).map(|f| {
            let side = match f.side {
                Side::Bid => "BUY",
                Side::Ask => "SELL",
            };
            FillRow {
                coin: f.coin.clone(),
                side: side.to_string(),
                size: f.sz.to_string(),
                price: f.px.to_string(),
                closed_pnl: f.closed_pnl.to_string(),
                fee: f.fee.to_string(),
            }
        }).collect();

        Ok(FillsOutput { fills: rows })
    }

    // ═══════════════════════════════════════════════════════════════════
    //  BUILDER FEE INJECTION
    // ═══════════════════════════════════════════════════════════════════
    //
    //  The builder field is NOT part of the signed data. We:
    //    1. Sign the action normally (without builder)
    //    2. Serialize the ActionRequest to JSON
    //    3. Inject "builder" into the "action" object
    //    4. POST the modified JSON to /exchange
    //
    // ═══════════════════════════════════════════════════════════════════

    /// Place a batch order with builder fee injection.
    /// Returns the parsed order response statuses.
    async fn place_with_builder(
        &self,
        batch: BatchOrder,
        vault_address: Option<Address>,
    ) -> Result<Vec<OrderResponseStatus>> {
        let nonce = self.nonce.next();
        let chain = if self.testnet {
            hypercore::Chain::Testnet
        } else {
            hypercore::Chain::Mainnet
        };

        // 1. Sign the action to get an ActionRequest
        let action: Action = batch.into();
        let signed = action
            .sign_sync(&self.signer, nonce, vault_address, None, chain)
            .context("Failed to sign order action")?;

        // 2. Serialize to JSON value
        let mut json_val = serde_json::to_value(&signed)
            .context("Failed to serialize ActionRequest")?;

        // 3. Inject builder fee into the action object
        let builder = BuilderFee::default();
        if let Some(action_obj) = json_val.get_mut("action") {
            action_obj["builder"] = serde_json::to_value(&builder)?;
        }

        // 4. POST to /exchange
        let base_url = if self.testnet {
            "https://api.hyperliquid-testnet.xyz"
        } else {
            "https://api.hyperliquid.xyz"
        };

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let resp = http
            .post(format!("{}/exchange", base_url))
            .json(&json_val)
            .send()
            .await
            .context("Exchange HTTP request failed")?;

        let status = resp.status();
        let body = resp.text().await
            .context("Failed to read exchange response body")?;

        if !status.is_success() {
            bail!("Exchange HTTP {status}: {body}");
        }

        // 5. Parse response
        // Expected: {"status":"ok","response":{"type":"order","data":{"statuses":[...]}}}
        // or {"status":"err","response":"error msg"}
        let parsed: Value = serde_json::from_str(&body)
            .context(format!("Failed to parse exchange response: {body}"))?;

        let resp_status = parsed.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if resp_status == "err" {
            let err_msg = parsed.get("response")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            bail!("Exchange error: {err_msg}");
        }

        // Parse statuses from the response
        let statuses_val = parsed
            .pointer("/response/data/statuses")
            .ok_or_else(|| anyhow::anyhow!("No statuses in exchange response: {body}"))?;

        let statuses: Vec<OrderResponseStatus> = serde_json::from_value(statuses_val.clone())
            .context(format!("Failed to parse order statuses: {statuses_val}"))?;

        Ok(statuses)
    }

    // ═══════════════════════════════════════════════════════════════════
    //  ORDER PLACEMENT — ALL PATHS INJECT BUILDER FEE
    // ═══════════════════════════════════════════════════════════════════

    /// Parse a TIF string into TimeInForce.
    fn parse_tif(tif: &str) -> Result<TimeInForce> {
        match tif.to_uppercase().as_str() {
            "GTC" => Ok(TimeInForce::Gtc),
            "IOC" => Ok(TimeInForce::Ioc),
            "ALO" => Ok(TimeInForce::Alo),
            _ => bail!("Invalid TIF: {tif}. Use Gtc, Ioc, or Alo"),
        }
    }

    /// Place a limit order with already-resolved asset size.
    /// Used when CLI has already done USDC→units conversion.
    pub async fn limit_order_raw(
        &self,
        coin: &str,
        is_buy: bool,
        size: f64,
        price: f64,
        reduce_only: bool,
        tif: &str,
    ) -> Result<OrderResult> {
        let asset = self.resolve_asset(coin)?;
        let tif_enum = Self::parse_tif(tif)?;

        let px = self.round_price(coin, Decimal::from_f64(price)
            .ok_or_else(|| anyhow::anyhow!("Invalid price: {price}"))?)?;
        let sz = self.round_size(coin, Decimal::from_f64(size)
            .ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?)?;

        if sz.is_zero() {
            bail!("Order size rounds to zero for {coin}");
        }

        let order = OrderRequest {
            asset,
            is_buy,
            reduce_only,
            limit_px: px,
            sz,
            cloid: random_cloid(),
            order_type: OrderTypePlacement::Limit { tif: tif_enum },
        };

        info!(
            coin, side = if is_buy { "BUY" } else { "SELL" },
            %sz, %px, tif, reduce_only,
            mode = %self.config.trading.mode,
            builder = BUILDER_ADDRESS, fee_bps = BUILDER_FEE_BPS,
            "placing limit order (raw size) with builder fee"
        );

        let batch = BatchOrder {
            orders: vec![order],
            grouping: OrderGrouping::Na,
        };

        let statuses = self.place_with_builder(batch, None).await?;
        parse_order_response(&statuses)
    }

    /// Place a limit order with mandatory builder fee injection.
    /// Size is automatically resolved: in CFD mode, `input_size` = lots → converted to asset units.
    ///
    /// ╔══════════════════════════════════════════════════════════════╗
    /// ║  BUILDER FEE INJECTION — MANDATORY ON EVERY ORDER          ║
    /// ╚══════════════════════════════════════════════════════════════╝
    pub async fn limit_order(
        &self,
        coin: &str,
        is_buy: bool,
        input_size: f64,
        price: f64,
        reduce_only: bool,
        tif: &str,
    ) -> Result<OrderResult> {
        let size = self.config.resolve_size(coin, input_size);

        info!(
            coin, side = if is_buy { "BUY" } else { "SELL" },
            input_size, resolved_size = size,
            mode = %self.config.trading.mode,
            "resolving size before limit order"
        );

        self.limit_order_raw(coin, is_buy, size, price, reduce_only, tif).await
    }

    /// Place a market order with already-resolved asset size.
    /// Uses IOC with slippage to simulate market order.
    pub async fn market_open_raw(
        &self,
        coin: &str,
        is_buy: bool,
        size: f64,
        slippage: Option<f64>,
    ) -> Result<OrderResult> {
        let asset = self.resolve_asset(coin)?;
        let slip = slippage.unwrap_or(self.config.trading.default_slippage);

        // Fetch mid price for slippage calculation
        let mids = self.client.all_mids(None).await
            .context("Failed to fetch mids for market order")?;
        let mid = mids.get(coin)
            .ok_or_else(|| anyhow::anyhow!("No mid price for {coin}"))?;

        // Calculate limit price with slippage
        let slip_dec = Decimal::from_f64(slip)
            .ok_or_else(|| anyhow::anyhow!("Invalid slippage: {slip}"))?;
        let slippage_mult = if is_buy {
            Decimal::ONE + slip_dec
        } else {
            Decimal::ONE - slip_dec
        };
        let raw_px = *mid * slippage_mult;
        let px = self.round_price(coin, raw_px)?;

        let sz = self.round_size(coin, Decimal::from_f64(size)
            .ok_or_else(|| anyhow::anyhow!("Invalid size: {size}"))?)?;

        if sz.is_zero() {
            bail!("Order size rounds to zero for {coin}");
        }

        let order = OrderRequest {
            asset,
            is_buy,
            reduce_only: false,
            limit_px: px,
            sz,
            cloid: random_cloid(),
            order_type: OrderTypePlacement::Limit { tif: TimeInForce::Ioc },
        };

        info!(
            coin, side = if is_buy { "BUY" } else { "SELL" },
            %sz, mid = %mid, %px, slippage = slip,
            mode = %self.config.trading.mode,
            builder = BUILDER_ADDRESS, fee_bps = BUILDER_FEE_BPS,
            "placing market open (IOC with slippage) with builder fee"
        );

        let batch = BatchOrder {
            orders: vec![order],
            grouping: OrderGrouping::Na,
        };

        let statuses = self.place_with_builder(batch, None).await?;
        parse_order_response(&statuses)
    }

    /// Place a market order (IOC with slippage) with builder fee injection.
    /// Size is automatically resolved.
    pub async fn market_open(
        &self,
        coin: &str,
        is_buy: bool,
        input_size: f64,
        slippage: Option<f64>,
    ) -> Result<OrderResult> {
        let size = self.config.resolve_size(coin, input_size);

        info!(
            coin, side = if is_buy { "BUY" } else { "SELL" },
            input_size, resolved_size = size,
            mode = %self.config.trading.mode,
            "resolving size before market open"
        );

        self.market_open_raw(coin, is_buy, size, slippage).await
    }

    /// Close a position at market with slippage.
    /// Fetches current position to determine side and size, then sends
    /// a reduce-only IOC order.
    pub async fn market_close(
        &self,
        coin: &str,
        size: Option<f64>,
        slippage: Option<f64>,
    ) -> Result<OrderResult> {
        let asset = self.resolve_asset(coin)?;
        let slip = slippage.unwrap_or(0.05);

        // Fetch current position
        let state = self.client.clearinghouse_state(self.address, None).await
            .context("Failed to fetch account state for close")?;

        let position = state.asset_positions.iter()
            .find(|p| p.position.coin.eq_ignore_ascii_case(coin))
            .ok_or_else(|| anyhow::anyhow!("No open position for {coin}"))?;

        let pos_size = position.position.szi; // positive = long, negative = short
        if pos_size.is_zero() {
            bail!("Position size is zero for {coin}");
        }

        let is_long = pos_size > Decimal::ZERO;
        // To close: sell if long, buy if short
        let is_buy = !is_long;

        // Determine close size
        let close_size = match size {
            Some(s) => {
                let s_dec = Decimal::from_f64(s)
                    .ok_or_else(|| anyhow::anyhow!("Invalid close size: {s}"))?;
                self.round_size(coin, s_dec.min(pos_size.abs()))?
            }
            None => pos_size.abs(),
        };

        if close_size.is_zero() {
            bail!("Close size rounds to zero for {coin}");
        }

        // Get price with slippage
        let mids = self.client.all_mids(None).await
            .context("Failed to fetch mids for close")?;
        let mid = mids.get(coin)
            .ok_or_else(|| anyhow::anyhow!("No mid price for {coin}"))?;

        let slip_dec = Decimal::from_f64(slip)
            .ok_or_else(|| anyhow::anyhow!("Invalid slippage: {slip}"))?;
        let slippage_mult = if is_buy {
            Decimal::ONE + slip_dec
        } else {
            Decimal::ONE - slip_dec
        };
        let raw_px = *mid * slippage_mult;
        let px = self.round_price(coin, raw_px)?;

        let order = OrderRequest {
            asset,
            is_buy,
            reduce_only: true,
            limit_px: px,
            sz: close_size,
            cloid: random_cloid(),
            order_type: OrderTypePlacement::Limit { tif: TimeInForce::Ioc },
        };

        info!(
            coin, side = if is_buy { "BUY" } else { "SELL" },
            %close_size, %pos_size, mid = %mid, %px, slippage = slip,
            builder = BUILDER_ADDRESS, fee_bps = BUILDER_FEE_BPS,
            "closing position (reduce-only IOC) with builder fee"
        );

        let batch = BatchOrder {
            orders: vec![order],
            grouping: OrderGrouping::Na,
        };

        let statuses = self.place_with_builder(batch, None).await?;
        parse_order_response(&statuses)
    }

    // ═══════════════════════════════════════════════════════════════════
    //  CANCEL ORDERS
    // ═══════════════════════════════════════════════════════════════════

    /// Cancel an order by OID.
    pub async fn cancel_order(&self, coin: &str, oid: u64) -> Result<CancelSingleOutput> {
        let asset = self.resolve_asset(coin)?;

        info!(coin, oid, "cancelling order by OID");

        let batch = BatchCancel {
            cancels: vec![Cancel { asset, oid }],
        };

        let result = self.client
            .cancel(&self.signer, batch, self.nonce.next(), None, None)
            .await;

        match result {
            Ok(statuses) => {
                if statuses.iter().any(|s| s.is_err()) {
                    let errs: Vec<_> = statuses.iter()
                        .filter_map(|s| s.error())
                        .collect();
                    bail!("Cancel failed: {}", errs.join(", "));
                }
                Ok(CancelSingleOutput {
                    coin: coin.to_string(),
                    oid,
                    status: "cancelled".to_string(),
                })
            }
            Err(e) => bail!("Cancel error: {}", e.message()),
        }
    }

    /// Cancel an order by CLOID (client order ID).
    pub async fn cancel_order_by_cloid(&self, coin: &str, cloid: Uuid) -> Result<CancelSingleOutput> {
        let asset = self.resolve_asset(coin)?;

        info!(coin, %cloid, "cancelling order by CLOID");

        // Convert UUID to B128 (Cloid)
        let cloid_bytes = cloid.as_bytes();
        let mut cloid_b128 = [0u8; 16];
        cloid_b128.copy_from_slice(cloid_bytes);
        let cloid_val = alloy::primitives::B128::from(cloid_b128);

        let batch = BatchCancelCloid {
            cancels: vec![CancelByCloid { asset: asset as u32, cloid: cloid_val }],
        };

        let result = self.client
            .cancel_by_cloid(&self.signer, batch, self.nonce.next(), None, None)
            .await;

        match result {
            Ok(statuses) => {
                if statuses.iter().any(|s| s.is_err()) {
                    let errs: Vec<_> = statuses.iter()
                        .filter_map(|s| s.error())
                        .collect();
                    bail!("Cancel by CLOID failed: {}", errs.join(", "));
                }
                Ok(CancelSingleOutput {
                    coin: coin.to_string(),
                    oid: 0, // CLOIDs don't have a numeric OID
                    status: format!("cancelled (cloid: {})", cloid),
                })
            }
            Err(e) => bail!("Cancel by CLOID error: {}", e.message()),
        }
    }

    /// Cancel all open orders on a specific asset.
    pub async fn cancel_all_orders(&self, coin: &str) -> Result<CancelOutput> {
        let asset = self.resolve_asset(coin)?;

        let orders = self.client.open_orders(self.address, None).await
            .context("Failed to fetch open orders")?;

        let matching: Vec<_> = orders.iter()
            .filter(|o| o.coin.eq_ignore_ascii_case(coin))
            .collect();

        if matching.is_empty() {
            return Ok(CancelOutput {
                coin: coin.to_string(),
                cancelled: 0,
                total: 0,
                oids: vec![],
            });
        }

        // Batch cancel all matching orders
        let oids: Vec<u64> = matching.iter().map(|o| o.oid).collect();
        let cancels: Vec<Cancel> = matching.iter()
            .map(|o| Cancel { asset, oid: o.oid })
            .collect();

        let total = cancels.len() as u32;
        let batch = BatchCancel { cancels };

        let result = self.client
            .cancel(&self.signer, batch, self.nonce.next(), None, None)
            .await;

        let cancelled = match result {
            Ok(statuses) => {
                statuses.iter().filter(|s| s.is_ok()).count() as u32
            }
            Err(e) => {
                info!(error = %e.message(), "batch cancel had errors");
                0
            }
        };

        Ok(CancelOutput {
            coin: coin.to_string(),
            cancelled,
            total,
            oids,
        })
    }

    // ═══════════════════════════════════════════════════════════════════
    //  LEVERAGE & MARGIN
    // ═══════════════════════════════════════════════════════════════════

    /// Update leverage for an asset.
    ///
    /// hypersdk doesn't expose `updateLeverage` in its `Action` enum, so we
    /// construct and sign the raw exchange request manually using the same
    /// RMP+Agent EIP-712 signing pattern used by all exchange actions.
    pub async fn set_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
    ) -> Result<LeverageOutput> {
        let asset = self.resolve_asset(coin)?;

        info!(coin, leverage, is_cross, "updating leverage");

        let base_url = if self.testnet {
            "https://api.hyperliquid-testnet.xyz"
        } else {
            "https://api.hyperliquid.xyz"
        };

        let action_json = serde_json::json!({
            "type": "updateLeverage",
            "asset": asset,
            "isCross": is_cross,
            "leverage": leverage
        });

        let nonce = self.nonce.next();

        // Sign using the same RMP+Agent pattern as all exchange actions:
        // 1. RMP-serialize action → bytes; append nonce + vault flag
        // 2. keccak256(bytes) → connectionId
        // 3. Build Agent EIP-712 hash manually → signing hash
        // 4. Sign the hash

        let mut rmp_bytes = rmp_serde::to_vec_named(&action_json)
            .context("Failed to RMP serialize leverage action")?;
        rmp_bytes.extend(nonce.to_be_bytes());
        rmp_bytes.push(0u8); // no vault_address

        let connection_id = alloy::primitives::keccak256(&rmp_bytes);

        let source = if self.testnet { "b" } else { "a" };
        let agent_hash = compute_agent_signing_hash(source, connection_id);

        let sig = self.signer.sign_hash_sync(&agent_hash)
            .context("Failed to sign leverage action")?;

        let r_hex = hex::encode(sig.r().to_be_bytes::<32>());
        let s_hex = hex::encode(sig.s().to_be_bytes::<32>());
        let v = if sig.v() { 28u8 } else { 27u8 };

        let signature = serde_json::json!({
            "r": format!("0x{r_hex}"),
            "s": format!("0x{s_hex}"),
            "v": v
        });

        let request_body = serde_json::json!({
            "action": action_json,
            "nonce": nonce,
            "signature": signature,
            "vaultAddress": null
        });

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let resp = http
            .post(format!("{}/exchange", base_url))
            .json(&request_body)
            .send()
            .await
            .context("Leverage update HTTP request failed")?;

        let resp_status = resp.status();
        let body = resp.text().await?;

        if !resp_status.is_success() {
            bail!("Leverage update HTTP {resp_status}: {body}");
        }

        let parsed: Value = serde_json::from_str(&body)
            .context(format!("Failed to parse leverage response: {body}"))?;

        let status = parsed.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if status == "err" {
            let err = parsed.get("response")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            bail!("Leverage update failed: {err}");
        }

        let mode = if is_cross { "cross" } else { "isolated" };
        Ok(LeverageOutput {
            coin: coin.to_string(),
            leverage,
            mode: mode.to_string(),
        })
    }

    /// Update isolated margin for an asset.
    ///
    /// Uses hypersdk's `UpdateIsolatedMargin` action via `sign_and_send_sync`.
    pub async fn update_margin(&self, amount: f64, coin: &str) -> Result<MarginOutput> {
        let asset = self.resolve_asset(coin)?;

        info!(coin, amount, "updating isolated margin");

        // The UpdateIsolatedMargin struct expects:
        // - asset: usize (market index)
        // - is_buy: bool (true for adding margin to long side)
        // - ntli: u64 (margin delta scaled — the raw amount in USD * 1e6 or similar)
        //
        // Looking at the old SDK: it passed the raw f64 amount.
        // The Hyperliquid API expects ntli as a scaled integer.
        // From the Hyperliquid docs: ntli is the margin delta in integer format.
        // Let's use the absolute value and determine is_buy from sign.

        let is_add = amount > 0.0;

        // Fetch current position to determine side (is_buy)
        let state = self.client.clearinghouse_state(self.address, None).await
            .context("Failed to fetch account state for margin update")?;

        let position = state.asset_positions.iter()
            .find(|p| p.position.coin.eq_ignore_ascii_case(coin));

        let is_buy = match position {
            Some(p) => p.position.szi > Decimal::ZERO, // true if long
            None => true, // default to long if no position
        };

        // ntli is the raw integer value — the API seems to use an integer representation
        // Let's convert: amount in USD * 1_000_000 for 6 decimal precision (USDC)
        let ntli = (amount.abs() * 1_000_000.0) as u64;

        let update = UpdateIsolatedMargin {
            asset,
            is_buy,
            ntli,
        };

        let chain = if self.testnet {
            hypercore::Chain::Testnet
        } else {
            hypercore::Chain::Mainnet
        };

        let action: Action = update.into();
        let signed = action
            .sign_sync(&self.signer, self.nonce.next(), None, None, chain)
            .context("Failed to sign margin update")?;

        let resp = self.client.send(signed).await
            .context("Failed to send margin update")?;

        // Parse response
        match resp {
            ExchangeResponse::Ok(_) => {
                let action_str = if is_add { "Added" } else { "Removed" };
                Ok(MarginOutput {
                    coin: coin.to_string(),
                    action: action_str.to_string(),
                    amount: format!("{:.2}", amount.abs()),
                })
            }
            ExchangeResponse::Err(e) => {
                bail!("Margin update failed: {e}");
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    //  TRANSFERS
    // ═══════════════════════════════════════════════════════════════════

    /// Transfer USDC to another address.
    pub async fn transfer_usdc(&self, amount: &str, destination: &str) -> Result<TransferOutput> {
        info!(amount, destination, "transferring USDC");

        let dest: Address = destination.parse()
            .context(format!("Invalid destination address: {destination}"))?;

        let amount_dec: Decimal = amount.parse()
            .context(format!("Invalid amount: {amount}"))?;

        let send = UsdSend {
            destination: dest,
            amount: amount_dec,
            time: self.nonce.next(),
        };

        self.client
            .send_usdc(&self.signer, send, self.nonce.next())
            .await
            .context("USDC transfer failed")?;

        Ok(TransferOutput {
            amount: amount.to_string(),
            destination: destination.to_string(),
        })
    }

    // ─── Market Data ────────────────────────────────────────────────

    /// Get mid prices for specific coins.
    pub async fn get_prices(&self, coins: &[String]) -> Result<PriceOutput> {
        let mids = self.client.all_mids(None).await
            .context("Failed to fetch mid prices")?;

        let prices: Vec<PriceRow> = coins.iter().map(|coin| {
            let price = mids.get(coin.as_str())
                .map(|d| d.to_string())
                .unwrap_or_else(|| "N/A".to_string());
            PriceRow {
                coin: coin.clone(),
                mid_price: price,
            }
        }).collect();

        Ok(PriceOutput { prices })
    }

    /// Get all mid prices.
    pub async fn get_all_prices(&self) -> Result<PriceOutput> {
        let mids = self.client.all_mids(None).await
            .context("Failed to fetch mid prices")?;

        let mut prices: Vec<PriceRow> = mids.iter().map(|(coin, price)| {
            PriceRow {
                coin: coin.clone(),
                mid_price: price.to_string(),
            }
        }).collect();

        // Sort alphabetically
        prices.sort_by(|a, b| a.coin.cmp(&b.coin));

        Ok(PriceOutput { prices })
    }

    /// Get perpetual markets info.
    pub async fn get_perp_markets(&self) -> Result<MarketsOutput> {
        let markets: Vec<MarketRow> = self.perps.iter().map(|m| {
            MarketRow {
                name: m.name.clone(),
                index: m.index,
                max_leverage: m.max_leverage,
                sz_decimals: m.sz_decimals,
            }
        }).collect();

        Ok(MarketsOutput {
            market_type: "perp".to_string(),
            markets,
        })
    }

    /// Get spot markets info.
    pub async fn get_spot_markets(&self) -> Result<MarketsOutput> {
        let spots = self.client.spot().await
            .context("Failed to fetch spot markets")?;

        let markets: Vec<MarketRow> = spots.iter().map(|m| {
            MarketRow {
                name: m.symbol(),
                index: m.index,
                max_leverage: 1, // spot has no leverage
                sz_decimals: 0,  // spot doesn't expose sz_decimals directly
            }
        }).collect();

        Ok(MarketsOutput {
            market_type: "spot".to_string(),
            markets,
        })
    }

    /// Get candle data for a coin.
    pub async fn get_candles(
        &self,
        coin: &str,
        interval: CandleInterval,
        limit: usize,
    ) -> Result<CandlesOutput> {
        // Calculate time range: now - (limit * interval_ms) to now
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let interval_ms = match interval {
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
        };

        let start_time = now_ms.saturating_sub(interval_ms * limit as u64);

        let candles = self.client
            .candle_snapshot(coin, interval, start_time, now_ms)
            .await
            .context(format!("Failed to fetch candles for {coin}"))?;

        let interval_str = match interval {
            CandleInterval::OneMinute => "1m",
            CandleInterval::ThreeMinutes => "3m",
            CandleInterval::FiveMinutes => "5m",
            CandleInterval::FifteenMinutes => "15m",
            CandleInterval::ThirtyMinutes => "30m",
            CandleInterval::OneHour => "1h",
            CandleInterval::TwoHours => "2h",
            CandleInterval::FourHours => "4h",
            CandleInterval::EightHours => "8h",
            CandleInterval::TwelveHours => "12h",
            CandleInterval::OneDay => "1d",
            CandleInterval::ThreeDays => "3d",
            CandleInterval::OneWeek => "1w",
            CandleInterval::OneMonth => "1M",
        };

        let rows: Vec<CandleRow> = candles.iter().map(|c| {
            let time = format_timestamp_ms(c.open_time);
            CandleRow {
                time,
                open: c.open.to_string(),
                high: c.high.to_string(),
                low: c.low.to_string(),
                close: c.close.to_string(),
                volume: c.volume.to_string(),
                trades: c.num_trades,
            }
        }).collect();

        Ok(CandlesOutput {
            coin: coin.to_string(),
            interval: interval_str.to_string(),
            candles: rows,
        })
    }

    /// Get funding rate history for a coin.
    pub async fn get_funding(&self, coin: &str) -> Result<FundingOutput> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Last 7 days of funding
        let start_time = now_ms.saturating_sub(7 * 86_400_000);

        let rates = self.client
            .funding_history(coin, start_time, Some(now_ms))
            .await
            .context(format!("Failed to fetch funding for {coin}"))?;

        let rows: Vec<FundingRow> = rates.iter().map(|r| {
            FundingRow {
                time: format_timestamp_ms(r.time),
                coin: r.coin.clone(),
                rate: r.funding_rate.to_string(),
                premium: r.premium.to_string(),
            }
        }).collect();

        Ok(FundingOutput {
            coin: coin.to_string(),
            rates: rows,
        })
    }
}

/// Format a millisecond timestamp to human-readable UTC string.
fn format_timestamp_ms(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let nanos = ((ms % 1000) * 1_000_000) as u32;

    // Manual UTC formatting without chrono dependency
    // Unix epoch: 1970-01-01T00:00:00Z
    let total_days = secs / 86400;
    let day_secs = (secs % 86400) as u32;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;
    let _ = nanos; // We don't need sub-second precision

    // Civil date from days since epoch (algorithm from Howard Hinnant)
    let z = total_days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02}")
}

// ─── Response parsing helper ────────────────────────────────────────

fn parse_order_response(statuses: &[OrderResponseStatus]) -> Result<OrderResult> {
    if statuses.is_empty() {
        bail!("Empty statuses in response");
    }

    match &statuses[0] {
        OrderResponseStatus::Filled { total_sz, avg_px, oid } => {
            Ok(OrderResult {
                oid: *oid,
                status: OrderFillStatus::Filled,
                output: OrderResultOutput {
                    oid: *oid,
                    status: "filled".into(),
                    total_sz: Some(total_sz.to_string()),
                    avg_px: Some(avg_px.to_string()),
                },
            })
        }
        OrderResponseStatus::Resting { oid, .. } => {
            Ok(OrderResult {
                oid: *oid,
                status: OrderFillStatus::Resting,
                output: OrderResultOutput {
                    oid: *oid,
                    status: "resting".into(),
                    total_sz: None,
                    avg_px: None,
                },
            })
        }
        OrderResponseStatus::Success => {
            Ok(OrderResult {
                oid: 0,
                status: OrderFillStatus::Filled,
                output: OrderResultOutput {
                    oid: 0,
                    status: "accepted".into(),
                    total_sz: None,
                    avg_px: None,
                },
            })
        }
        OrderResponseStatus::Error(msg) => {
            bail!("Exchange error: {msg}");
        }
    }
}

// ─── EIP-712 Agent signing (manual, no sol! macro) ──────────────────
//
// Reproduces the Agent { string source, bytes32 connectionId } EIP-712
// signing hash used by Hyperliquid for exchange actions. This is needed
// for action types (like updateLeverage) not exposed in hypersdk's Action enum.
//
// Domain: name="Exchange", version="1", chainId=1337, verifyingContract=0x0
//
// EIP-712 hash = keccak256(0x1901 ‖ domainSeparator ‖ structHash)
// where structHash = keccak256(typeHash ‖ encoded_fields)

fn compute_agent_signing_hash(
    source: &str,
    connection_id: alloy::primitives::B256,
) -> alloy::primitives::B256 {
    use alloy::primitives::keccak256;

    // Type hash for EIP712Domain
    let domain_type_hash = keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    let mut domain_data = Vec::with_capacity(160);
    domain_data.extend_from_slice(domain_type_hash.as_slice());
    domain_data.extend_from_slice(keccak256(b"Exchange").as_slice());
    domain_data.extend_from_slice(keccak256(b"1").as_slice());
    // chainId = 1337 as uint256 (32 bytes, big-endian)
    let mut chain_id_bytes = [0u8; 32];
    chain_id_bytes[31] = (1337 & 0xFF) as u8;
    chain_id_bytes[30] = ((1337 >> 8) & 0xFF) as u8;
    domain_data.extend_from_slice(&chain_id_bytes);
    // verifyingContract = address(0) padded to 32 bytes
    domain_data.extend_from_slice(&[0u8; 32]);

    let domain_separator = keccak256(&domain_data);

    // Agent type hash: keccak256("Agent(string source,bytes32 connectionId)")
    let agent_type_hash = keccak256(b"Agent(string source,bytes32 connectionId)");

    // Struct hash = keccak256(typeHash ‖ keccak256(source) ‖ connectionId)
    let mut struct_data = Vec::with_capacity(96);
    struct_data.extend_from_slice(agent_type_hash.as_slice());
    struct_data.extend_from_slice(keccak256(source.as_bytes()).as_slice());
    struct_data.extend_from_slice(connection_id.as_slice());

    let struct_hash = keccak256(&struct_data);

    // Final EIP-712 hash: keccak256(0x1901 ‖ domainSeparator ‖ structHash)
    let mut final_data = Vec::with_capacity(66);
    final_data.push(0x19);
    final_data.push(0x01);
    final_data.extend_from_slice(domain_separator.as_slice());
    final_data.extend_from_slice(struct_hash.as_slice());

    keccak256(&final_data)
}
