/// All data the TUI needs to render — fetched from Hyperliquid via Engine.
#[allow(dead_code)]
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
    /// Ticks between auto-refreshes (200ms per tick → 25 ticks = 5s).
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

    // ── Market data ─────────────────────────────────────────────
    pub all_mids: Vec<(String, String)>,

    // ── Connection state ────────────────────────────────────────
    pub connected: bool,
    pub last_error: Option<String>,
    pub last_refresh: String,
}

#[derive(Clone)]
pub struct PositionRow {
    pub coin: String,
    pub size: String,
    pub entry_px: String,
    pub mark_px: String,
    pub liq_px: String,
    pub upnl: String,
    pub roe: String,
    pub leverage: String,
    pub margin_used: String,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct OrderRow {
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub oid: u64,
    pub order_type: String,
    pub timestamp: u64,
}

impl App {
    /// Create a new App and attempt initial data fetch.
    pub async fn new() -> Self {
        let config = atlas_core::workspace::load_config().unwrap_or_default();
        let profile_name = config.general.active_profile.clone();
        let network = if config.network.testnet {
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
            refresh_interval: 25, // ~5s at 200ms poll

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
            all_mids: Vec::new(),

            connected: false,
            last_error: None,
            last_refresh: String::from("never"),
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
        use atlas_core::Engine;
        use hypersdk::hypercore::types::Side;

        let engine = Engine::from_active_profile().await?;
        self.address = format!("{}", engine.address);

        // User state (positions + margins) — hypersdk uses clearinghouse_state
        let state = engine.client.clearinghouse_state(engine.address, None).await
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
                    entry_px: p.entry_px
                        .map(|e| format!("{}", e))
                        .unwrap_or_else(|| "—".into()),
                    mark_px: String::from("—"), // updated below with all_mids
                    liq_px: p.liquidation_px
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
        let orders = engine.client.open_orders(engine.address, None).await
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
                    timestamp: o.timestamp,
                }
            })
            .collect();

        // All mids (market prices) — hypersdk returns HashMap<String, Decimal>
        let mids = engine.client.all_mids(None).await
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        // Update mark prices in positions
        for pos in &mut self.positions {
            if let Some(mid) = mids.get(&pos.coin) {
                pos.mark_px = format!("{}", mid);
            }
        }

        // Sort mids alphabetically for display
        let mut mids_vec: Vec<(String, String)> = mids
            .into_iter()
            .map(|(k, v)| (k, format!("{}", v)))
            .collect();
        mids_vec.sort_by(|a, b| a.0.cmp(&b.0));
        self.all_mids = mids_vec;

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
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        // Auto-refresh is handled by the caller checking tick_count
        // We don't refresh in tick() to keep it sync — caller does it async.
    }

    /// Check if it's time for auto-refresh.
    pub fn should_refresh(&self) -> bool {
        self.tick_count % self.refresh_interval == 0 && self.tick_count > 0
    }
}

// Default for AppConfig is defined in atlas_types::config
