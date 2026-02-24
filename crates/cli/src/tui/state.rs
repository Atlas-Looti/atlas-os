use std::collections::HashMap;

use rust_decimal::Decimal;
use tui_input::Input;

#[derive(Default)]
pub enum TradeFocus {
    #[default]
    Coin,
    Size,
    Price,
    Side,
}

pub struct TradePopup {
    pub visible: bool,
    pub focus: TradeFocus,
    pub coin: Input,
    pub size: Input,
    pub price: Input,
    pub side: Input, // BUY or SELL
    pub status: Option<String>,
}

impl Default for TradePopup {
    fn default() -> Self {
        Self {
            visible: false,
            focus: TradeFocus::default(),
            coin: Input::default(),
            size: Input::default(),
            price: Input::default(),
            side: Input::default().with_value("BUY".into()),
            status: None,
        }
    }
}

#[derive(Default)]
pub enum SwapFocus {
    #[default]
    SellToken,
    BuyToken,
    SellAmount,
}

#[derive(Default)]
pub struct SwapPopup {
    pub visible: bool,
    pub focus: SwapFocus,
    pub sell_token: Input,
    pub buy_token: Input,
    pub sell_amount: Input,
    pub status: Option<String>,
}


/// All data the TUI needs to render — fetched from Hyperliquid via Engine.
pub struct App {
    /// Active tab index.
    pub tab: usize,
    /// Tab names.
    pub tabs: Vec<&'static str>,
    /// Show help overlay.
    pub show_help: bool,
    /// Scroll offset for scrollable panels.
    pub scroll: u16,
    /// Tick counter for auto-refresh timing.
    pub tick_count: u64,
    /// Ticks between auto-refreshes (200ms per tick → 50 ticks = 10s).
    pub refresh_interval: u64,

    // ── Account data ────────────────────────────────────────────
    pub profile_name: String,
    pub address: String,
    pub network: String,
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
    pub withdrawable: String,

    // ── Positions ───────────────────────────────────────────────
    pub positions: Vec<PositionRow>,

    // ── Open orders ─────────────────────────────────────────────
    pub open_orders: Vec<OrderRow>,
    /// Selected order index (for cancel keybind).
    pub selected_order: usize,

    // ── Market data ─────────────────────────────────────────────
    pub all_mids: Vec<(String, String)>,
    /// Live mid prices from WebSocket (coin → Decimal).
    pub live_mids: HashMap<String, Decimal>,
    /// Market token index mapping for spot names (e.g. 1 -> PURR).
    pub spot_map: HashMap<usize, String>,

    // ── Connection state ────────────────────────────────────────
    pub connected: bool,
    pub ws_connected: bool,
    pub last_error: Option<String>,
    pub last_refresh: String,
    pub last_ws_update: String,

    // ── Cancel feedback ─────────────────────────────────────────
    pub cancel_status: Option<String>,
    pub cancel_status_tick: u64,

    // ── Popups ──────────────────────────────────────────────────
    pub trade_popup: TradePopup,
    pub swap_popup: SwapPopup,
}

#[derive(Clone)]
pub struct PositionRow {
    pub coin: String,
    pub size: String,
    pub size_dec: Decimal,
    pub entry_px: String,
    pub entry_px_dec: Option<Decimal>,
    pub mark_px: String,
    pub liq_px: String,
    pub upnl: String,
    pub roe: String,
    pub leverage: String,
    pub margin_used: String,
}

#[derive(Clone)]
pub struct OrderRow {
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub oid: u64,
    pub order_type: String,
}

impl App {
    /// Create a new App and attempt initial data fetch.
    pub async fn new() -> Self {
        let config = atlas_core::workspace::load_config().unwrap_or_default();
        let profile_name = config.system.active_profile.clone();
        let network = if config.modules.hyperliquid.config.network == "testnet" {
            "Testnet".to_string()
        } else {
            "Mainnet".to_string()
        };

        let mut app = Self {
            tab: 0,
            tabs: vec!["Dashboard", "Positions", "Orders", "Markets"],
            show_help: false,
            scroll: 0,
            tick_count: 0,
            refresh_interval: 50, // ~10s at 200ms poll

            profile_name,
            address: String::from("—"),
            network,
            account_value: String::from("—"),
            total_margin_used: String::from("—"),
            total_ntl_pos: String::from("—"),
            total_raw_usd: String::from("—"),
            withdrawable: String::from("—"),

            positions: Vec::new(),
            open_orders: Vec::new(),
            selected_order: 0,
            all_mids: Vec::new(),
            live_mids: HashMap::new(),
            spot_map: HashMap::new(),

            connected: false,
            ws_connected: false,
            last_error: None,
            last_refresh: String::from("never"),
            last_ws_update: String::from("—"),

            cancel_status: None,
            cancel_status_tick: 0,

            trade_popup: TradePopup::default(),
            swap_popup: SwapPopup::default(),
        };

        app.refresh().await;
        app
    }

    /// Fetch all data from Hyperliquid. Non-fatal — stores error in state.
    pub async fn refresh(&mut self) {
        match self.fetch_data().await {
            Ok(()) => {
                self.connected = true;
                self.last_error = None;
                self.last_refresh = chrono::Local::now().format("%H:%M:%S").to_string();
            }
            Err(e) => {
                self.connected = false;
                self.last_error = Some(format!("{e:#}"));
            }
        }
    }

    async fn fetch_data(&mut self) -> anyhow::Result<()> {
        use atlas_core::workspace::load_config;
        use atlas_core::AuthManager;
        use hypersdk::hypercore::{self as hypercore, types::Side};

        let config = load_config()?;
        let signer = AuthManager::get_active_signer()?;
        let address = alloy::signers::local::PrivateKeySigner::address(&signer);
        let testnet = config.modules.hyperliquid.config.network == "testnet";
        let client = if testnet {
            hypercore::testnet()
        } else {
            hypercore::mainnet()
        };

        self.address = format!("{}", address);

        // User state (positions + margins) — hypersdk uses clearinghouse_state
        let state = client
            .clearinghouse_state(address, None)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        self.account_value = format!("{}", state.margin_summary.account_value);
        self.total_margin_used = format!("{}", state.margin_summary.total_margin_used);
        self.total_ntl_pos = format!("{}", state.margin_summary.total_ntl_pos);
        self.total_raw_usd = format!("{}", state.margin_summary.total_raw_usd);
        self.withdrawable = format!("{}", state.withdrawable);

        self.positions = state
            .asset_positions
            .iter()
            .map(|ap| {
                let p = &ap.position;
                PositionRow {
                    coin: p.coin.clone(),
                    size: format!("{}", p.szi),
                    size_dec: p.szi,
                    entry_px: p
                        .entry_px
                        .map(|e| format!("{}", e))
                        .unwrap_or_else(|| "—".into()),
                    entry_px_dec: p.entry_px,
                    mark_px: String::from("—"), // updated below with all_mids
                    liq_px: p
                        .liquidation_px
                        .map(|e| format!("{}", e))
                        .unwrap_or_else(|| "—".into()),
                    upnl: format!("{}", p.unrealized_pnl),
                    roe: format!("{}", p.return_on_equity),
                    leverage: format!("{}x", p.leverage.value),
                    margin_used: format!("{}", p.margin_used),
                }
            })
            .collect();

        // Open orders — hypersdk returns BasicOrder with Decimal fields
        let orders = client
            .open_orders(address, None)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        self.open_orders = orders
            .iter()
            .map(|o| {
                let side_str = match o.side {
                    Side::Bid => "BUY".to_string(),
                    Side::Ask => "SELL".to_string(),
                };
                OrderRow {
                    coin: o.coin.clone(),
                    side: side_str,
                    size: format!("{}", o.sz),
                    price: format!("{}", o.limit_px),
                    oid: o.oid,
                    order_type: "Limit".to_string(),
                }
            })
            .collect();

        // Clamp selected order
        if !self.open_orders.is_empty() && self.selected_order >= self.open_orders.len() {
            self.selected_order = self.open_orders.len() - 1;
        }

        // All mids (market prices) — hypersdk returns HashMap<String, Decimal>
        let mids = client
            .all_mids(None)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        // Try getting spot token map to resolve "@100" -> "PURR"
        let orch = crate::factory::readonly().await?;
        let perp = orch.perp(None)?;
        self.spot_map = perp.spot_tokens_map().await.unwrap_or_default();

        let mut resolved_mids = HashMap::new();
        for (k, v) in mids {
            if let Some(stripped) = k.strip_prefix('@') {
                if let Ok(idx) = stripped.parse::<usize>() {
                    if let Some(name) = self.spot_map.get(&idx) {
                        resolved_mids.insert(name.clone(), v);
                        continue;
                    }
                }
            }
            resolved_mids.insert(k, v);
        }

        // Store live mids for PnL calculation
        self.live_mids = resolved_mids.clone();

        // Update mark prices and live PnL in positions
        self.update_positions_from_mids();

        // Sort mids alphabetically for display
        let mut mids_vec: Vec<(String, String)> = resolved_mids
            .into_iter()
            .map(|(k, v)| (k, format!("{}", v)))
            .collect();
        mids_vec.sort_by(|a, b| a.0.cmp(&b.0));
        self.all_mids = mids_vec;

        Ok(())
    }

    /// Update positions with live mid prices and recalculate unrealized PnL.
    pub fn update_positions_from_mids(&mut self) {
        for pos in &mut self.positions {
            if let Some(mid) = self.live_mids.get(&pos.coin) {
                pos.mark_px = format!("{}", mid);

                // Recalculate unrealized PnL: (mark - entry) * size
                if let Some(entry) = pos.entry_px_dec {
                    let pnl = (*mid - entry) * pos.size_dec;
                    pos.upnl = format!("{}", pnl);

                    // Recalculate ROE: pnl / margin_used
                    if let Ok(margin) = pos.margin_used.parse::<Decimal>() {
                        if !margin.is_zero() {
                            let roe = pnl / margin;
                            pos.roe = format!("{}", roe);
                        }
                    }
                }
            }
        }
    }

    /// Handle incoming WebSocket AllMids update.
    pub fn on_ws_mids(&mut self, mids: HashMap<String, Decimal>) {
        self.ws_connected = true;
        self.last_ws_update = chrono::Local::now().format("%H:%M:%S").to_string();

        let mut resolved_mids = HashMap::new();
        for (k, v) in mids {
            if let Some(stripped) = k.strip_prefix('@') {
                if let Ok(idx) = stripped.parse::<usize>() {
                    if let Some(name) = self.spot_map.get(&idx) {
                        resolved_mids.insert(name.clone(), v);
                        continue;
                    }
                }
            }
            resolved_mids.insert(k, v);
        }

        self.live_mids = resolved_mids;

        // Update positions with live prices
        self.update_positions_from_mids();

        // Update markets display
        let mut mids_vec: Vec<(String, String)> = self
            .live_mids
            .iter()
            .map(|(k, v)| (k.clone(), format!("{}", v)))
            .collect();
        mids_vec.sort_by(|a, b| a.0.cmp(&b.0));
        self.all_mids = mids_vec;
    }

    /// Handle WebSocket connected event.
    pub fn on_ws_connected(&mut self) {
        self.ws_connected = true;
    }

    /// Handle WebSocket disconnected event.
    pub fn on_ws_disconnected(&mut self) {
        self.ws_connected = false;
    }

    /// Cancel the currently selected order.
    pub async fn cancel_selected_order(&mut self) {
        if self.open_orders.is_empty() {
            self.cancel_status = Some("No orders to cancel".to_string());
            self.cancel_status_tick = self.tick_count;
            return;
        }

        let order = self.open_orders[self.selected_order].clone();

        match self.do_cancel(&order.coin, order.oid).await {
            Ok(()) => {
                self.cancel_status = Some(format!("Cancelled {} #{}", order.coin, order.oid));
                self.cancel_status_tick = self.tick_count;
                // Remove from local list immediately
                self.open_orders.remove(self.selected_order);
                if self.selected_order > 0 && self.selected_order >= self.open_orders.len() {
                    self.selected_order = self.open_orders.len().saturating_sub(1);
                }
            }
            Err(e) => {
                self.cancel_status = Some(format!("Cancel failed: {e:#}"));
                self.cancel_status_tick = self.tick_count;
            }
        }
    }

    async fn do_cancel(&self, coin: &str, oid: u64) -> anyhow::Result<()> {
        let orch = crate::factory::from_active_profile().await?;
        let perp = orch.perp(None).map_err(|e| anyhow::anyhow!("{e}"))?;
        perp.cancel_order(coin, &oid.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    pub fn set_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tab = idx;
            self.scroll = 0;
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % self.tabs.len();
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        if self.tab == 0 {
            self.tab = self.tabs.len() - 1;
        } else {
            self.tab -= 1;
        }
        self.scroll = 0;
    }

    pub fn scroll_up(&mut self) {
        if self.tab == 2 {
            // Orders tab — move selection
            self.selected_order = self.selected_order.saturating_sub(1);
        } else {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.tab == 2 {
            // Orders tab — move selection
            if !self.open_orders.is_empty() {
                self.selected_order = (self.selected_order + 1).min(self.open_orders.len() - 1);
            }
        } else {
            self.scroll = self.scroll.saturating_add(1);
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        // Clear cancel status after ~3 seconds (15 ticks at 200ms)
        if self.cancel_status.is_some()
            && self.tick_count.saturating_sub(self.cancel_status_tick) > 15
        {
            self.cancel_status = None;
        }
    }

    /// Check if it's time for auto-refresh (full REST refresh for account data).
    pub fn should_refresh(&self) -> bool {
        self.tick_count.is_multiple_of(self.refresh_interval) && self.tick_count > 0
    }

    pub async fn execute_trade(&mut self) {
        let coin = self.trade_popup.coin.value().to_uppercase();
        let size_str = self.trade_popup.size.value();
        let price_str = self.trade_popup.price.value();
        let side = self.trade_popup.side.value().to_uppercase();

        let price_dec = match rust_decimal::Decimal::from_str_exact(price_str) {
            Ok(p) => p,
            Err(_) => {
                self.trade_popup.status = Some("Invalid price format".into());
                return;
            }
        };

        let size_dec = match rust_decimal::Decimal::from_str_exact(size_str) {
            Ok(s) => s,
            Err(_) => {
                self.trade_popup.status = Some("Invalid size format".into());
                return;
            }
        };

        let uni_side = if side == "BUY" {
            atlas_core::types::Side::Buy
        } else {
            atlas_core::types::Side::Sell
        };

        self.trade_popup.status = Some("Submitting...".into());
        match self.do_trade(&coin, uni_side, size_dec, price_dec).await {
            Ok(_) => {
                self.trade_popup.status = Some("Order placed".into());
                self.refresh().await; // Refresh to show new order
            }
            Err(e) => {
                self.trade_popup.status = Some(format!("Error: {e}"));
            }
        }
    }

    async fn do_trade(
        &self,
        coin: &str,
        side: atlas_core::types::Side,
        size: rust_decimal::Decimal,
        price: rust_decimal::Decimal,
    ) -> anyhow::Result<()> {
        let orch = crate::factory::from_active_profile().await?;
        let perp = orch.perp(None).map_err(|e| anyhow::anyhow!("{e}"))?;
        perp.limit_order(coin, side, size, price, false)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    pub async fn execute_swap(&mut self) {
        let sell_token = self.swap_popup.sell_token.value();
        let buy_token = self.swap_popup.buy_token.value();
        let sell_amount = self.swap_popup.sell_amount.value();

        self.swap_popup.status = Some("Executing swap...".into());
        // We clone inputs because `self` will be borrowed through `do_swap` 
        match self.do_swap(sell_token, buy_token, sell_amount).await {
            Ok(hash) => {
                self.swap_popup.status = Some(format!("Swap sent: {hash}"));
                self.refresh().await;
            }
            Err(e) => {
                self.swap_popup.status = Some(format!("Error: {e}"));
            }
        }
    }

    async fn do_swap(&self, sell_token: &str, buy_token: &str, amount: &str) -> anyhow::Result<String> {
        let orch = crate::factory::from_active_profile().await?;
        let swap_mod = orch.swap(None).map_err(|e| anyhow::anyhow!("{e}"))?;
        
        let zerox = swap_mod
            .as_any()
            .downcast_ref::<atlas_zero_x::ZeroXModule>()
            .ok_or_else(|| anyhow::anyhow!("0x module not available"))?;

        let taker = zerox.taker_address().unwrap_or_default();
        let price_resp = zerox.price(
            &atlas_core::types::Chain::Arbitrum, 
            sell_token,
            buy_token,
            amount,
            Some(&taker),
            Some(50), 
        ).await.map_err(|e| anyhow::anyhow!("{e}"))?;

        if !price_resp.liquidity_available {
            anyhow::bail!("No liquidity");
        }

        let sell_dec: rust_decimal::Decimal = amount
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid amount: {amount}"))?;

        let quote = atlas_core::types::SwapQuote {
            protocol: atlas_core::types::Protocol::ZeroX,
            chain: atlas_core::types::Chain::Arbitrum, 
            sell_token: sell_token.to_string(),
            buy_token: buy_token.to_string(),
            sell_amount: sell_dec,
            buy_amount: price_resp.buy_amount.unwrap_or_default().parse().unwrap_or_default(),
            estimated_gas: None,
            price: rust_decimal::Decimal::ZERO,
            allowance_target: price_resp.allowance_target,
            tx_data: None,
        };

        let tx_hash = swap_mod.swap(&quote).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(tx_hash)
    }
}
