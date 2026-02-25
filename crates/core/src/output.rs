// Structured output types for JSON/table rendering.
//
// Every data-producing command returns one of these types.
// They all derive `Serialize` for JSON output, and implement
// `TableDisplay` for human-readable table rendering.

use std::collections::HashMap;

use serde::Serialize;

// ─── Status ─────────────────────────────────────────────────────────

/// PRD-compliant status output.
///
/// ```json
/// {
///   "ok": true,
///   "data": {
///     "profile": "main",
///     "address": "0x...",
///     "modules": ["hyperliquid"],
///     "balances": [{ "asset": "USDC", "total": "5000.00", "available": "4800.00", "protocol": "hyperliquid" }],
///     "positions": [...],
///     "open_orders": 2
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct StatusOutput {
    pub profile: String,
    pub address: String,
    pub network: String,
    pub modules: Vec<String>,
    pub balances: Vec<BalanceRow>,
    pub account_value: Option<String>,
    pub margin_used: Option<String>,
    pub net_position: Option<String>,
    pub withdrawable: Option<String>,
    pub positions: Vec<PositionRow>,
    pub open_orders: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BalanceRow {
    pub asset: String,
    pub total: String,
    pub available: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionRow {
    #[serde(rename = "symbol")]
    pub coin: String,
    pub side: String,
    pub size: String,
    pub entry_price: Option<String>,
    pub mark_price: Option<String>,
    pub unrealized_pnl: Option<String>,
    pub liquidation_price: Option<String>,
    pub leverage: Option<u32>,
    pub margin_mode: Option<String>,
    pub protocol: String,
}

// ─── Orders ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OrdersOutput {
    pub orders: Vec<OrderRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderRow {
    #[serde(rename = "symbol")]
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    #[serde(rename = "order_id")]
    pub oid: u64,
}

// ─── Fills ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FillsOutput {
    pub fills: Vec<FillRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FillRow {
    #[serde(rename = "symbol")]
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub closed_pnl: String,
    pub fee: String,
}

// ─── Order result (place/close) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OrderResultOutput {
    #[serde(rename = "order_id")]
    pub oid: u64,
    #[serde(rename = "symbol")]
    pub coin: String,
    pub side: String,
    #[serde(rename = "size")]
    pub total_sz: Option<String>,
    #[serde(rename = "price")]
    pub avg_px: Option<String>,
    pub filled: Option<String>,
    /// "filled", "resting", "accepted"
    pub status: String,
    pub fee: Option<String>,
    pub builder_fee_bps: u32,
    pub protocol: String,
    pub timestamp: Option<u64>,
}

// ─── Cancel ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CancelOutput {
    #[serde(rename = "symbol")]
    pub coin: String,
    pub cancelled: u32,
    pub total: u32,
    #[serde(rename = "order_ids")]
    pub oids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CancelSingleOutput {
    #[serde(rename = "symbol")]
    pub coin: String,
    #[serde(rename = "order_id")]
    pub oid: u64,
    pub status: String,
}

// ─── Leverage ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct LeverageOutput {
    #[serde(rename = "symbol")]
    pub coin: String,
    pub leverage: u32,
    pub mode: String,
}

// ─── Margin ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct MarginOutput {
    #[serde(rename = "symbol")]
    pub coin: String,
    pub action: String,
    pub amount: String,
}

// ─── Transfer ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TransferOutput {
    pub amount: String,
    pub destination: String,
}

// ─── Risk ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct RiskCalcOutput {
    pub coin: String,
    pub side: String,
    pub entry_price: f64,
    pub size: f64,
    pub lots: f64,
    pub notional: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub est_liquidation: f64,
    pub risk_usd: f64,
    pub risk_pct: f64,
    pub margin: f64,
    pub leverage: u32,
    pub warnings: Vec<String>,
    pub blocked: bool,
}

// ─── Config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ConfigOutput {
    pub mode: String,
    pub size_mode: String,
    pub leverage: u32,
    pub slippage: f64,
    pub network: String,
    pub lots: HashMap<String, f64>,
}

// ─── Doctor ─────────────────────────────────────────────────────────

/// PRD-compliant doctor check result.
///
/// Status is "ok" or "fail". On failure, `fix` contains the actionable hint.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

impl DoctorCheck {
    pub fn ok(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "ok".into(),
            value: Some(value.into()),
            fix: None,
            latency_ms: None,
            network: None,
        }
    }

    pub fn ok_bare(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "ok".into(),
            value: None,
            fix: None,
            latency_ms: None,
            network: None,
        }
    }

    pub fn fail(name: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "fail".into(),
            value: None,
            fix: Some(fix.into()),
            latency_ms: None,
            network: None,
        }
    }
}

/// PRD-compliant `atlas doctor --output json` output.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorOutput {
    pub checks: Vec<DoctorCheck>,
}

// ─── Market Data: Price ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PriceOutput {
    pub prices: Vec<PriceRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceRow {
    #[serde(rename = "symbol")]
    pub coin: String,
    #[serde(rename = "price")]
    pub mid_price: String,
    pub protocol: String,
}

// ─── Market Data: Markets ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct MarketsOutput {
    pub market_type: String,
    pub markets: Vec<MarketRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketRow {
    pub name: String,
    pub index: usize,
    pub max_leverage: u64,
    pub sz_decimals: i64,
}

// ─── Market Data: Candles ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CandlesOutput {
    pub coin: String,
    pub interval: String,
    pub candles: Vec<CandleRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandleRow {
    pub time: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub trades: u64,
}

// ─── Market Data: Funding ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FundingOutput {
    pub coin: String,
    pub rates: Vec<FundingRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FundingRow {
    pub time: String,
    pub coin: String,
    pub rate: String,
    pub premium: String,
}

// ─── Spot Balance ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SpotBalanceOutput {
    pub balances: Vec<SpotBalanceRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpotBalanceRow {
    pub coin: String,
    pub total: String,
    pub held: String,
    pub available: String,
}

// ─── Spot Order ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SpotOrderOutput {
    pub market: String,
    pub side: String,
    pub oid: u64,
    pub status: String,
    pub total_sz: Option<String>,
    pub avg_px: Option<String>,
}

// ─── Spot Transfer ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SpotTransferOutput {
    pub direction: String,
    pub token: String,
    pub amount: String,
}

// ─── Vault ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct VaultDetailsOutput {
    pub name: String,
    pub address: String,
    pub leader: String,
    pub description: String,
    pub apr: String,
    pub leader_fraction: String,
    pub leader_commission: String,
    pub max_distributable: String,
    pub max_withdrawable: String,
    pub follower_count: usize,
    pub is_closed: bool,
    pub allow_deposits: bool,
    pub followers: Vec<VaultFollowerRow>,
    pub user_state: Option<VaultUserStateRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultFollowerRow {
    pub user: String,
    pub equity: String,
    pub pnl: String,
    pub days_following: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultUserStateRow {
    pub equity: String,
    pub pnl: String,
    pub all_time_pnl: String,
    pub days_following: u64,
    pub lockup_until: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultDepositsOutput {
    pub deposits: Vec<VaultDepositRow>,
    pub total_equity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultDepositRow {
    pub vault_address: String,
    pub equity: String,
    pub locked_until: Option<String>,
}

// ─── Subaccounts ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SubAccountsOutput {
    pub subaccounts: Vec<SubAccountRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubAccountRow {
    pub name: String,
    pub address: String,
    pub account_value: String,
    pub total_position: String,
    pub margin_used: String,
    pub withdrawable: String,
    pub positions: Vec<PositionRow>,
    pub spot_balances: Vec<SpotBalanceRow>,
}

// ─── Agent ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AgentApproveOutput {
    pub agent_address: String,
    pub agent_name: String,
    pub status: String,
}

// ─── Auth ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AuthListOutput {
    pub profiles: Vec<AuthProfileRow>,
    pub active: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthProfileRow {
    pub name: String,
    pub address: String,
    pub active: bool,
}

// ─── History (trade/order/pnl from local DB cache) ──────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TradeHistoryOutput {
    pub trades: Vec<TradeHistoryRow>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradeHistoryRow {
    pub protocol: String,
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub pnl: String,
    pub fee: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderHistoryOutput {
    pub orders: Vec<OrderHistoryRow>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderHistoryRow {
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub oid: i64,
    pub status: String,
    pub order_type: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PnlSummaryOutput {
    pub total_pnl: String,
    pub total_fees: String,
    pub net_pnl: String,
    pub trade_count: usize,
    pub win_count: usize,
    pub loss_count: usize,
    pub win_rate: String,
    pub by_coin: Vec<PnlByCoinRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PnlByCoinRow {
    pub coin: String,
    pub pnl: String,
    pub fees: String,
    pub trades: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncOutput {
    pub fills_synced: usize,
    pub orders_synced: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportOutput {
    pub path: String,
    pub rows: usize,
    pub format: String,
}

// Unified output rendering: JSON or human-readable table.
//
// Usage:
// ```ignore
// use crate::output::{OutputFormat, render};
//
// let data = StatusOutput { ... };
// render(format, &data)?;
// ```

/// Output format for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable table (default).
    Table,
    /// Compact JSON (for piping to jq, scripts).
    Json,
    /// Pretty-printed JSON (for reading).
    JsonPretty,
}

/// Trait for types that can render as a human-readable table.
///
/// Implement this on each structured output type to define
/// how it looks in table mode.
pub trait TableDisplay {
    fn print_table(&self);
}

/// A generic API response wrapper for JSON output.
///
/// This struct provides a consistent envelope for JSON responses,
/// indicating success or failure and containing the data or error details.
#[derive(Serialize)]
pub struct ApiResponse<T> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

/// Render structured output — JSON or table depending on format.
///
/// For JSON formats, uses `serde_json` serialization and wraps the output
/// in an `ApiResponse` envelope (`{"ok":true,"data":...}` or `{"ok":false,"error":...}`).
/// For table format, calls `TableDisplay::print_table()`.
pub fn render<T: Serialize + TableDisplay>(format: OutputFormat, data: &T) -> anyhow::Result<()> {
    match format {
        OutputFormat::Table => {
            data.print_table();
            Ok(())
        }
        OutputFormat::Json => {
            let response = ApiResponse {
                ok: true,
                data: Some(data),
                error: None,
            };
            let json = serde_json::to_string(&response)?;
            println!("{json}");
            Ok(())
        }
        OutputFormat::JsonPretty => {
            let response = ApiResponse {
                ok: true,
                data: Some(data),
                error: None,
            };
            let json = serde_json::to_string_pretty(&response)?;
            println!("{json}");
            Ok(())
        }
    }
}

/// Render just the JSON formats (for types that handle their own table display).
/// Returns true if JSON was rendered, false if table mode was requested.
///
/// This function also wraps the JSON output in an `ApiResponse` envelope.
pub fn render_json_or<T: Serialize>(format: OutputFormat, data: &T) -> anyhow::Result<bool> {
    match format {
        OutputFormat::Table => Ok(false),
        OutputFormat::Json => {
            let response = ApiResponse {
                ok: true,
                data: Some(data),
                error: None,
            };
            let json = serde_json::to_string(&response)?;
            println!("{json}");
            Ok(true)
        }
        OutputFormat::JsonPretty => {
            let response = ApiResponse {
                ok: true,
                data: Some(data),
                error: None,
            };
            let json = serde_json::to_string_pretty(&response)?;
            println!("{json}");
            Ok(true)
        }
    }
}

// ─── TableDisplay implementations for output types ──────────────────

impl TableDisplay for StatusOutput {
    fn print_table(&self) {
        let dash = "—";
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  ACCOUNT SUMMARY                                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Profile     : {:<41}║", self.profile);
        println!("║  Address     : {:<41}║", self.address);
        println!("║  Network     : {:<41}║", self.network);
        println!(
            "║  Modules     : {:<41}║",
            if self.modules.is_empty() {
                "none".to_string()
            } else {
                self.modules.join(", ")
            }
        );
        println!(
            "║  Account Val : {:<41}║",
            self.account_value.as_deref().unwrap_or(dash)
        );
        println!(
            "║  Margin Used : {:<41}║",
            self.margin_used.as_deref().unwrap_or(dash)
        );
        println!(
            "║  Net Pos     : {:<41}║",
            self.net_position.as_deref().unwrap_or(dash)
        );
        println!(
            "║  Withdrawable: {:<41}║",
            self.withdrawable.as_deref().unwrap_or(dash)
        );
        println!("║  Open Orders : {:<41}║", self.open_orders);
        println!("╠══════════════════════════════════════════════════════════╣");

        if self.positions.is_empty() {
            println!("║  No open positions.                                      ║");
        } else {
            println!(
                "║  {:^6} │ {:^10} │ {:^10} │ {:^12} ║",
                "Coin", "Size", "Entry", "uPnL"
            );
            println!("║  ──────┼────────────┼────────────┼────────────── ║");
            for pos in &self.positions {
                println!(
                    "║  {:^6} │ {:>10} │ {:>10} │ {:>12} ║",
                    pos.coin,
                    pos.size,
                    pos.entry_price.as_deref().unwrap_or(dash),
                    pos.unrealized_pnl.as_deref().unwrap_or(dash),
                );
            }
        }
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

impl TableDisplay for OrdersOutput {
    fn print_table(&self) {
        if self.orders.is_empty() {
            println!("No open orders.");
            return;
        }

        println!("┌────────┬──────┬────────────┬──────────────┬────────────────┐");
        println!("│ Coin   │ Side │ Size       │ Price        │ OID            │");
        println!("├────────┼──────┼────────────┼──────────────┼────────────────┤");
        for o in &self.orders {
            println!(
                "│ {:<6} │ {:<4} │ {:>10} │ {:>12} │ {:>14} │",
                o.coin, o.side, o.size, o.price, o.oid,
            );
        }
        println!("└────────┴──────┴────────────┴──────────────┴────────────────┘");
    }
}

impl TableDisplay for FillsOutput {
    fn print_table(&self) {
        if self.fills.is_empty() {
            println!("No recent fills.");
            return;
        }

        println!("┌────────┬──────┬────────────┬──────────────┬──────────────┬──────────┐");
        println!("│ Coin   │ Side │ Size       │ Price        │ Closed PnL   │ Fee      │");
        println!("├────────┼──────┼────────────┼──────────────┼──────────────┼──────────┤");
        for f in &self.fills {
            println!(
                "│ {:<6} │ {:<4} │ {:>10} │ {:>12} │ {:>12} │ {:>8} │",
                f.coin, f.side, f.size, f.price, f.closed_pnl, f.fee,
            );
        }
        println!("└────────┴──────┴────────────┴──────────────┴──────────────┴──────────┘");
    }
}

impl TableDisplay for OrderResultOutput {
    fn print_table(&self) {
        match self.status.as_str() {
            "filled" => {
                let sz = self.total_sz.as_deref().unwrap_or("—");
                let px = self.avg_px.as_deref().unwrap_or("—");
                println!(
                    "✓ Order FILLED (oid: {}, size: {}, avg_px: {})",
                    self.oid, sz, px
                );
            }
            "resting" => {
                println!("✓ Order RESTING (oid: {})", self.oid);
            }
            _ => {
                println!("✓ Order accepted (oid: {})", self.oid);
            }
        }
    }
}

impl TableDisplay for CancelOutput {
    fn print_table(&self) {
        println!(
            "✓ Cancelled {}/{} orders on {}.",
            self.cancelled, self.total, self.coin
        );
    }
}

impl TableDisplay for CancelSingleOutput {
    fn print_table(&self) {
        println!("✓ Order {} on {} cancelled.", self.oid, self.coin);
    }
}

impl TableDisplay for LeverageOutput {
    fn print_table(&self) {
        println!(
            "✓ {} leverage set to {}x ({})",
            self.coin, self.leverage, self.mode
        );
    }
}

impl TableDisplay for MarginOutput {
    fn print_table(&self) {
        println!("✓ {} ${} margin on {}", self.action, self.amount, self.coin);
    }
}

impl TableDisplay for TransferOutput {
    fn print_table(&self) {
        println!(
            "✓ Transferred ${} USDC to {}",
            self.amount, self.destination
        );
    }
}

impl TableDisplay for ConfigOutput {
    fn print_table(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  ATLAS CONFIGURATION                                   ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Mode      : {:<43}║", self.mode);
        println!("║  Size Mode : {:<43}║", self.size_mode);
        println!("║  Leverage  : {:<43}║", format!("{}x", self.leverage));
        println!(
            "║  Slippage  : {:<43}║",
            format!("{:.1}%", self.slippage * 100.0)
        );
        println!("║  Network   : {:<43}║", self.network);
        println!("╠══════════════════════════════════════════════════════════╣");
        if !self.lots.is_empty() {
            println!("║  Lot Sizes:                                            ║");
            let mut sorted: Vec<_> = self.lots.iter().collect();
            sorted.sort_by_key(|(k, _)| (*k).clone());
            for (coin, size) in &sorted {
                println!("║    {:<6} : {:<39}║", coin, format!("{} units/lot", size));
            }
        }
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

impl TableDisplay for DoctorOutput {
    fn print_table(&self) {
        println!("┌─────────────────────────────────────────────┐");
        println!("│  ATLAS DOCTOR                               │");
        println!("├─────────────────────────────────────────────┤");
        for check in &self.checks {
            let icon = if check.status == "ok" { "✓" } else { "✗" };
            let label = format!("{:<14}", check.name);
            if check.status == "ok" {
                let val = check.value.as_deref().unwrap_or("");
                let display = if val.is_empty() {
                    icon.to_string()
                } else {
                    format!("{icon} ({val})")
                };
                println!("│  {label}: {:<27}│", display);
            } else {
                let fix = check
                    .fix
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(26)
                    .collect::<String>();
                println!("│  {label}: {icon} → {:<26}│", fix);
            }
        }
        let all_ok = self.checks.iter().all(|c| c.status == "ok");
        println!("├─────────────────────────────────────────────┤");
        if all_ok {
            println!("│  ✓ All systems operational.                 │");
        } else {
            println!("│  Issues found. Run with --fix to repair.    │");
        }
        println!("└─────────────────────────────────────────────┘");
    }
}

impl TableDisplay for RiskCalcOutput {
    fn print_table(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  RISK CALCULATOR                                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!(
            "║  Asset        : {:<6} {:<34}║",
            self.coin,
            self.side.to_uppercase()
        );
        println!("║  Entry Price  : ${:<43.4}║", self.entry_price);
        println!(
            "║  Size         : {:<40}║",
            format!("{:.6} {}", self.size, self.coin)
        );
        if (self.lots - self.size).abs() > 0.0001 {
            println!("║  Lots         : {:<40.4}║", self.lots);
        }
        println!("║  Notional     : ${:<43.2}║", self.notional);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Stop-Loss    : ${:<43.4}║", self.stop_loss);
        println!("║  Take-Profit  : ${:<43.4}║", self.take_profit);
        println!("║  Est. Liq     : ${:<43.4}║", self.est_liquidation);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Risk (USDC)  : ${:<43.2}║", self.risk_usd);
        println!(
            "║  Risk (%)     : {:<43}║",
            format!("{:.2}%", self.risk_pct * 100.0)
        );
        println!("║  Margin Req.  : ${:<43.2}║", self.margin);
        println!("║  Leverage     : {:<43}║", format!("{}x", self.leverage));
        println!("╚══════════════════════════════════════════════════════════╝");

        if !self.warnings.is_empty() {
            println!();
            for w in &self.warnings {
                println!("{w}");
            }
            if self.blocked {
                println!();
                println!("❌ Trade BLOCKED by risk rules.");
            }
        }
    }
}

impl TableDisplay for SpotBalanceOutput {
    fn print_table(&self) {
        if self.balances.is_empty() {
            println!("No spot token balances.");
            return;
        }

        println!("┌──────────┬──────────────┬──────────────┬──────────────┐");
        println!("│ Token    │ Total        │ Held         │ Available    │");
        println!("├──────────┼──────────────┼──────────────┼──────────────┤");
        for b in &self.balances {
            println!(
                "│ {:<8} │ {:>12} │ {:>12} │ {:>12} │",
                b.coin, b.total, b.held, b.available,
            );
        }
        println!("└──────────┴──────────────┴──────────────┴──────────────┘");
    }
}

impl TableDisplay for SpotOrderOutput {
    fn print_table(&self) {
        match self.status.as_str() {
            "filled" => {
                let sz = self.total_sz.as_deref().unwrap_or("—");
                let px = self.avg_px.as_deref().unwrap_or("—");
                println!(
                    "✓ Spot {} {} FILLED (oid: {}, size: {}, avg_px: {})",
                    self.side, self.market, self.oid, sz, px
                );
            }
            "resting" => {
                println!(
                    "✓ Spot {} {} RESTING (oid: {})",
                    self.side, self.market, self.oid
                );
            }
            _ => {
                println!(
                    "✓ Spot {} {} accepted (oid: {})",
                    self.side, self.market, self.oid
                );
            }
        }
    }
}

impl TableDisplay for SpotTransferOutput {
    fn print_table(&self) {
        println!(
            "✓ Transferred {} {} ({})",
            self.amount, self.token, self.direction
        );
    }
}

impl TableDisplay for VaultDetailsOutput {
    fn print_table(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  VAULT DETAILS                                         ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Name         : {:<41}║", self.name);
        println!("║  Address      : {:<41}║", self.address);
        println!("║  Leader       : {:<41}║", self.leader);
        println!("║  APR          : {:<41}║", format!("{}%", self.apr));
        println!(
            "║  Leader Frac  : {:<41}║",
            format!("{}%", self.leader_fraction)
        );
        println!(
            "║  Commission   : {:<41}║",
            format!("{}%", self.leader_commission)
        );
        println!("║  Distributable: ${:<40}║", self.max_distributable);
        println!("║  Withdrawable : ${:<40}║", self.max_withdrawable);
        println!("║  Followers    : {:<41}║", self.follower_count);
        println!(
            "║  Closed       : {:<41}║",
            if self.is_closed { "Yes" } else { "No" }
        );
        println!(
            "║  Deposits     : {:<41}║",
            if self.allow_deposits {
                "Allowed"
            } else {
                "Closed"
            }
        );
        println!("╠══════════════════════════════════════════════════════════╣");

        if !self.description.is_empty() {
            println!(
                "║  {:<55}║",
                self.description.chars().take(55).collect::<String>()
            );
            println!("╠══════════════════════════════════════════════════════════╣");
        }

        if let Some(state) = &self.user_state {
            println!("║  YOUR POSITION                                         ║");
            println!("║  Equity       : ${:<40}║", state.equity);
            println!("║  PnL          : ${:<40}║", state.pnl);
            println!("║  All-time PnL : ${:<40}║", state.all_time_pnl);
            println!("║  Days         : {:<41}║", state.days_following);
            if let Some(lockup) = &state.lockup_until {
                println!("║  Locked Until : {:<41}║", lockup);
            }
            println!("╠══════════════════════════════════════════════════════════╣");
        }

        if !self.followers.is_empty() {
            println!("║  TOP FOLLOWERS                                         ║");
            println!(
                "║  {:<20} │ {:>12} │ {:>12} │ {:>4} ║",
                "User", "Equity", "PnL", "Days"
            );
            println!("║  ────────────────────┼──────────────┼──────────────┼──────║");
            for f in self.followers.iter().take(10) {
                let user_short = if f.user.len() > 20 {
                    format!("{}…", &f.user[..19])
                } else {
                    f.user.clone()
                };
                println!(
                    "║  {:<20} │ {:>12} │ {:>12} │ {:>4} ║",
                    user_short, f.equity, f.pnl, f.days_following,
                );
            }
        }
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

impl TableDisplay for VaultDepositsOutput {
    fn print_table(&self) {
        if self.deposits.is_empty() {
            println!("No vault deposits.");
            return;
        }

        println!(
            "┌──────────────────────────────────────────────┬──────────────┬──────────────────┐"
        );
        println!(
            "│ Vault Address                                │ Equity       │ Locked Until      │"
        );
        println!(
            "├──────────────────────────────────────────────┼──────────────┼──────────────────┤"
        );
        for d in &self.deposits {
            let locked = d.locked_until.as_deref().unwrap_or("—");
            println!(
                "│ {:<44} │ {:>12} │ {:<16} │",
                d.vault_address, d.equity, locked,
            );
        }
        println!(
            "├──────────────────────────────────────────────┼──────────────┼──────────────────┤"
        );
        println!(
            "│ {:<44} │ {:>12} │ {:<16} │",
            "TOTAL", self.total_equity, "",
        );
        println!(
            "└──────────────────────────────────────────────┴──────────────┴──────────────────┘"
        );
    }
}

impl TableDisplay for SubAccountsOutput {
    fn print_table(&self) {
        if self.subaccounts.is_empty() {
            println!("No subaccounts.");
            return;
        }

        for sub in &self.subaccounts {
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║  SUBACCOUNT: {:<43}║", sub.name);
            println!("╠══════════════════════════════════════════════════════════╣");
            println!("║  Address      : {:<41}║", sub.address);
            println!("║  Account Val  : ${:<40}║", sub.account_value);
            println!("║  Total Pos    : ${:<40}║", sub.total_position);
            println!("║  Margin Used  : ${:<40}║", sub.margin_used);
            println!("║  Withdrawable : ${:<40}║", sub.withdrawable);
            println!("╠══════════════════════════════════════════════════════════╣");

            if sub.positions.is_empty() {
                println!("║  No open positions.                                    ║");
            } else {
                println!(
                    "║  {:^6} │ {:^10} │ {:^10} │ {:^12} ║",
                    "Coin", "Size", "Entry", "uPnL"
                );
                println!("║  ──────┼────────────┼────────────┼────────────── ║");
                for pos in &sub.positions {
                    println!(
                        "║  {:^6} │ {:>10} │ {:>10} │ {:>12} ║",
                        pos.coin,
                        pos.size,
                        pos.entry_price.as_deref().unwrap_or("—"),
                        pos.unrealized_pnl.as_deref().unwrap_or("—"),
                    );
                }
            }

            if !sub.spot_balances.is_empty() {
                println!("╠══════════════════════════════════════════════════════════╣");
                println!("║  SPOT BALANCES                                         ║");
                for b in &sub.spot_balances {
                    println!("║    {:<6} : {:<46}║", b.coin, b.total);
                }
            }
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
        }

        println!("Total subaccounts: {}", self.subaccounts.len());
    }
}

impl TableDisplay for AgentApproveOutput {
    fn print_table(&self) {
        let name_display = if self.agent_name.is_empty() {
            "(unnamed)"
        } else {
            &self.agent_name
        };
        println!(
            "✓ Agent {} approved (name: {}, status: {})",
            self.agent_address, name_display, self.status
        );
    }
}

impl TableDisplay for TradeHistoryOutput {
    fn print_table(&self) {
        if self.trades.is_empty() {
            println!("No trade history cached. Run `atlas history sync` first.");
            return;
        }

        println!("┌────────┬──────┬────────────┬──────────────┬──────────────┬──────────┬─────────────────────┐");
        println!("│ Coin   │ Side │ Size       │ Price        │ PnL          │ Fee      │ Time                │");
        println!("├────────┼──────┼────────────┼──────────────┼──────────────┼──────────┼─────────────────────┤");
        for t in &self.trades {
            println!(
                "│ {:<6} │ {:<4} │ {:>10} │ {:>12} │ {:>12} │ {:>8} │ {:>19} │",
                t.coin, t.side, t.size, t.price, t.pnl, t.fee, t.time,
            );
        }
        println!("└────────┴──────┴────────────┴──────────────┴──────────────┴──────────┴─────────────────────┘");
        println!("Total: {} trades", self.total);
    }
}

impl TableDisplay for OrderHistoryOutput {
    fn print_table(&self) {
        if self.orders.is_empty() {
            println!("No order history cached. Run `atlas history sync` first.");
            return;
        }

        println!("┌────────┬──────┬────────────┬──────────────┬────────────────┬──────────┬─────────────────────┐");
        println!("│ Coin   │ Side │ Size       │ Price        │ OID            │ Status   │ Time                │");
        println!("├────────┼──────┼────────────┼──────────────┼────────────────┼──────────┼─────────────────────┤");
        for o in &self.orders {
            println!(
                "│ {:<6} │ {:<4} │ {:>10} │ {:>12} │ {:>14} │ {:>8} │ {:>19} │",
                o.coin, o.side, o.size, o.price, o.oid, o.status, o.time,
            );
        }
        println!("└────────┴──────┴────────────┴──────────────┴────────────────┴──────────┴─────────────────────┘");
        println!("Total: {} orders", self.total);
    }
}

impl TableDisplay for PnlSummaryOutput {
    fn print_table(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  PNL SUMMARY                                           ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Total PnL    : ${:<40}║", self.total_pnl);
        println!("║  Total Fees   : ${:<40}║", self.total_fees);
        println!("║  Net PnL      : ${:<40}║", self.net_pnl);
        println!("║  Trades       : {:<41}║", self.trade_count);
        println!(
            "║  Win/Loss     : {:<41}║",
            format!("{} / {}", self.win_count, self.loss_count)
        );
        println!("║  Win Rate     : {:<41}║", self.win_rate);
        println!("╠══════════════════════════════════════════════════════════╣");

        if !self.by_coin.is_empty() {
            println!("║  BREAKDOWN BY COIN                                     ║");
            println!(
                "║  {:<8} │ {:>12} │ {:>10} │ {:>6}      ║",
                "Coin", "PnL", "Fees", "Trades"
            );
            println!("║  ────────┼──────────────┼────────────┼────────────  ║");
            for row in &self.by_coin {
                println!(
                    "║  {:<8} │ {:>12} │ {:>10} │ {:>6}      ║",
                    row.coin, row.pnl, row.fees, row.trades,
                );
            }
        }
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}

impl TableDisplay for SyncOutput {
    fn print_table(&self) {
        println!(
            "✓ Sync {} — fills: {}, orders: {}",
            self.status, self.fills_synced, self.orders_synced
        );
    }
}

impl TableDisplay for ExportOutput {
    fn print_table(&self) {
        println!(
            "✓ Exported {} rows ({}) → {}",
            self.rows, self.format, self.path
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_output_serializes() {
        let output = StatusOutput {
            profile: "default".into(),
            address: "0x1234".into(),
            network: "Mainnet".into(),
            modules: vec!["hyperliquid".into()],
            balances: vec![BalanceRow {
                asset: "USDC".into(),
                total: "10000.00".into(),
                available: "9500.00".into(),
                protocol: "hyperliquid".into(),
            }],
            account_value: Some("10000.00".into()),
            margin_used: Some("500.00".into()),
            net_position: Some("2500.00".into()),
            withdrawable: Some("9500.00".into()),
            positions: vec![PositionRow {
                coin: "ETH".into(),
                side: "long".into(),
                size: "0.5".into(),
                entry_price: Some("3500.00".into()),
                mark_price: Some("3550.00".into()),
                unrealized_pnl: Some("25.00".into()),
                liquidation_price: Some("2800.00".into()),
                leverage: Some(5),
                margin_mode: Some("isolated".into()),
                protocol: "hyperliquid".into(),
            }],
            open_orders: 2,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"profile\":\"default\""));
        assert!(json.contains("\"symbol\":\"ETH\""));
        assert!(json.contains("\"modules\""));
        assert!(json.contains("\"open_orders\":2"));
        assert!(json.contains("\"balances\""));
    }

    #[test]
    fn test_orders_output_serializes() {
        let output = OrdersOutput {
            orders: vec![OrderRow {
                coin: "BTC".into(),
                side: "BUY".into(),
                size: "0.01".into(),
                price: "50000.00".into(),
                oid: 12345,
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"order_id\":12345"));
    }

    #[test]
    fn test_fills_output_serializes() {
        let output = FillsOutput {
            fills: vec![FillRow {
                coin: "ETH".into(),
                side: "SELL".into(),
                size: "1.0".into(),
                price: "3500.00".into(),
                closed_pnl: "100.00".into(),
                fee: "1.50".into(),
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"closed_pnl\":\"100.00\""));
    }

    #[test]
    fn test_order_result_output_serializes() {
        let output = OrderResultOutput {
            oid: 999,
            coin: "BTC".into(),
            side: "sell".into(),
            total_sz: Some("0.5".into()),
            avg_px: Some("3500.00".into()),
            filled: Some("0.5".into()),
            status: "filled".into(),
            fee: Some("0.05".into()),
            builder_fee_bps: 1,
            protocol: "hyperliquid".into(),
            timestamp: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"status\":\"filled\""));
    }

    #[test]
    fn test_cancel_output_serializes() {
        let output = CancelOutput {
            coin: "ETH".into(),
            cancelled: 3,
            total: 5,
            oids: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"cancelled\":3"));
    }

    #[test]
    fn test_risk_calc_output_serializes() {
        let output = RiskCalcOutput {
            coin: "ETH".into(),
            side: "long".into(),
            entry_price: 3500.0,
            size: 2.857,
            lots: 285.7,
            notional: 10000.0,
            stop_loss: 3400.0,
            take_profit: 3700.0,
            est_liquidation: 3100.0,
            risk_usd: 200.0,
            risk_pct: 0.02,
            margin: 1000.0,
            leverage: 10,
            warnings: vec!["⚠ test warning".into()],
            blocked: false,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"risk_usd\":200.0"));
        assert!(json.contains("\"blocked\":false"));
    }

    #[test]
    fn test_config_output_serializes() {
        let mut lots = HashMap::new();
        lots.insert("BTC".into(), 0.001);
        let output = ConfigOutput {
            mode: "futures".into(),
            size_mode: "usdc".into(),
            leverage: 10,
            slippage: 0.05,
            network: "Mainnet".into(),
            lots,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"leverage\":10"));
    }

    #[test]
    fn test_doctor_output_serializes() {
        let output = DoctorOutput {
            checks: vec![
                DoctorCheck::ok("profile", "main"),
                DoctorCheck::ok_bare("keyring"),
                DoctorCheck::fail("api_key", "Run: atlas configure system api-key <key>"),
            ],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"checks\""));
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"status\":\"fail\""));
        assert!(json.contains("\"fix\""));
    }

    #[test]
    fn test_json_pretty_format() {
        let output = StatusOutput {
            profile: "default".into(),
            address: "0x1234".into(),
            network: "Mainnet".into(),
            modules: vec!["hyperliquid".into()],
            balances: vec![],
            account_value: Some("10000.00".into()),
            margin_used: Some("500.00".into()),
            net_position: Some("2500.00".into()),
            withdrawable: Some("9500.00".into()),
            positions: vec![],
            open_orders: 0,
        };
        let pretty = serde_json::to_string_pretty(&output).unwrap();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    #[test]
    fn test_price_output_serializes() {
        let output = PriceOutput {
            prices: vec![
                PriceRow {
                    coin: "BTC".into(),
                    mid_price: "105234.50".into(),
                    protocol: "hyperliquid".into(),
                },
                PriceRow {
                    coin: "ETH".into(),
                    mid_price: "3521.25".into(),
                    protocol: "hyperliquid".into(),
                },
            ],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"symbol\":\"BTC\""));
        assert!(json.contains("\"price\":\"105234.50\""));
    }

    #[test]
    fn test_markets_output_serializes() {
        let output = MarketsOutput {
            market_type: "perp".into(),
            markets: vec![MarketRow {
                name: "ETH".into(),
                index: 1,
                max_leverage: 50,
                sz_decimals: 4,
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"max_leverage\":50"));
    }

    #[test]
    fn test_candles_output_serializes() {
        let output = CandlesOutput {
            coin: "BTC".into(),
            interval: "1h".into(),
            candles: vec![CandleRow {
                time: "2026-02-24 08:00:00".into(),
                open: "105000".into(),
                high: "105500".into(),
                low: "104800".into(),
                close: "105300".into(),
                volume: "1234.5".into(),
                trades: 456,
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"trades\":456"));
        assert!(json.contains("\"interval\":\"1h\""));
    }

    #[test]
    fn test_spot_balance_output_serializes() {
        let output = SpotBalanceOutput {
            balances: vec![SpotBalanceRow {
                coin: "USDC".into(),
                total: "1000.00".into(),
                held: "50.00".into(),
                available: "950.00".into(),
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"coin\":\"USDC\""));
        assert!(json.contains("\"available\":\"950.00\""));
    }

    #[test]
    fn test_spot_order_output_serializes() {
        let output = SpotOrderOutput {
            market: "PURR/USDC".into(),
            side: "BUY".into(),
            oid: 42,
            status: "filled".into(),
            total_sz: Some("100.0".into()),
            avg_px: Some("0.50".into()),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"market\":\"PURR/USDC\""));
        assert!(json.contains("\"oid\":42"));
        assert!(json.contains("\"status\":\"filled\""));
    }

    #[test]
    fn test_spot_transfer_output_serializes() {
        let output = SpotTransferOutput {
            direction: "perps → spot".into(),
            token: "USDC".into(),
            amount: "500.00".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"direction\":\"perps → spot\""));
        assert!(json.contains("\"token\":\"USDC\""));
    }

    #[test]
    fn test_funding_output_serializes() {
        let output = FundingOutput {
            coin: "ETH".into(),
            rates: vec![FundingRow {
                time: "2026-02-24 08:00:00".into(),
                coin: "ETH".into(),
                rate: "0.00012".into(),
                premium: "0.00005".into(),
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"rate\":\"0.00012\""));
    }

    #[test]
    fn test_vault_details_output_serializes() {
        let output = VaultDetailsOutput {
            name: "Test Vault".into(),
            address: "0xabc".into(),
            leader: "0xdef".into(),
            description: "A test vault".into(),
            apr: "12.50".into(),
            leader_fraction: "10.00".into(),
            leader_commission: "5.00".into(),
            max_distributable: "50000.00".into(),
            max_withdrawable: "40000.00".into(),
            follower_count: 42,
            is_closed: false,
            allow_deposits: true,
            followers: vec![VaultFollowerRow {
                user: "0x1234".into(),
                equity: "10000.00".into(),
                pnl: "500.00".into(),
                days_following: 30,
            }],
            user_state: Some(VaultUserStateRow {
                equity: "5000.00".into(),
                pnl: "250.00".into(),
                all_time_pnl: "1000.00".into(),
                days_following: 60,
                lockup_until: Some("2026-03-01".into()),
            }),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"name\":\"Test Vault\""));
        assert!(json.contains("\"follower_count\":42"));
        assert!(json.contains("\"allow_deposits\":true"));
        assert!(json.contains("\"all_time_pnl\":\"1000.00\""));
    }

    #[test]
    fn test_vault_deposits_output_serializes() {
        let output = VaultDepositsOutput {
            deposits: vec![VaultDepositRow {
                vault_address: "0xabc".into(),
                equity: "5000.00".into(),
                locked_until: Some("2026-03-01".into()),
            }],
            total_equity: "5000.00".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"total_equity\":\"5000.00\""));
        assert!(json.contains("\"vault_address\":\"0xabc\""));
    }

    #[test]
    fn test_subaccounts_output_serializes() {
        let output = SubAccountsOutput {
            subaccounts: vec![SubAccountRow {
                name: "bot-1".into(),
                address: "0x5678".into(),
                account_value: "10000.00".into(),
                total_position: "5000.00".into(),
                margin_used: "1000.00".into(),
                withdrawable: "9000.00".into(),
                positions: vec![PositionRow {
                    coin: "ETH".into(),
                    side: "long".into(),
                    size: "1.5".into(),
                    entry_price: Some("3500.00".into()),
                    mark_price: None,
                    unrealized_pnl: Some("100.00".into()),
                    liquidation_price: None,
                    leverage: None,
                    margin_mode: None,
                    protocol: "hyperliquid".into(),
                }],
                spot_balances: vec![SpotBalanceRow {
                    coin: "USDC".into(),
                    total: "500.00".into(),
                    held: "0.00".into(),
                    available: "500.00".into(),
                }],
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"name\":\"bot-1\""));
        assert!(json.contains("\"account_value\":\"10000.00\""));
        assert!(json.contains("\"symbol\":\"ETH\""));
    }

    #[test]
    fn test_agent_approve_output_serializes() {
        let output = AgentApproveOutput {
            agent_address: "0xagent".into(),
            agent_name: "my-bot".into(),
            status: "approved".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"agent_address\":\"0xagent\""));
        assert!(json.contains("\"status\":\"approved\""));
    }

    #[test]
    fn test_trade_history_output_serializes() {
        let output = TradeHistoryOutput {
            trades: vec![TradeHistoryRow {
                protocol: "hyperliquid".into(),
                coin: "ETH".into(),
                side: "Buy".into(),
                size: "0.5".into(),
                price: "3500.00".into(),
                pnl: "100.00".into(),
                fee: "1.75".into(),
                time: "2026-02-24 08:00:00".into(),
            }],
            total: 1,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"coin\":\"ETH\""));
        assert!(json.contains("\"total\":1"));
    }

    #[test]
    fn test_order_history_output_serializes() {
        let output = OrderHistoryOutput {
            orders: vec![OrderHistoryRow {
                coin: "BTC".into(),
                side: "Sell".into(),
                size: "0.01".into(),
                price: "105000.00".into(),
                oid: 42,
                status: "filled".into(),
                order_type: "Limit".into(),
                time: "2026-02-24 09:00:00".into(),
            }],
            total: 1,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"oid\":42"));
        assert!(json.contains("\"status\":\"filled\""));
    }

    #[test]
    fn test_pnl_summary_output_serializes() {
        let output = PnlSummaryOutput {
            total_pnl: "500.00".into(),
            total_fees: "25.00".into(),
            net_pnl: "475.00".into(),
            trade_count: 10,
            win_count: 7,
            loss_count: 3,
            win_rate: "70.0%".into(),
            by_coin: vec![PnlByCoinRow {
                coin: "ETH".into(),
                pnl: "300.00".into(),
                fees: "15.00".into(),
                trades: 6,
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"net_pnl\":\"475.00\""));
        assert!(json.contains("\"win_rate\":\"70.0%\""));
    }

    #[test]
    fn test_sync_output_serializes() {
        let output = SyncOutput {
            fills_synced: 50,
            orders_synced: 30,
            status: "complete".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"fills_synced\":50"));
        assert!(json.contains("\"orders_synced\":30"));
    }

    #[test]
    fn test_export_output_serializes() {
        let output = ExportOutput {
            path: "/home/user/.atlas-os/data/export-trades-123.csv".into(),
            rows: 100,
            format: "csv".into(),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"rows\":100"));
        assert!(json.contains("\"format\":\"csv\""));
    }

    #[test]
    fn test_render_json() {
        let data = OrdersOutput { orders: vec![] };
        // Just verify it doesn't panic
        render(OutputFormat::Json, &data).unwrap();
    }

    #[test]
    fn test_render_json_pretty() {
        let data = OrdersOutput { orders: vec![] };
        render(OutputFormat::JsonPretty, &data).unwrap();
    }

    #[test]
    fn test_render_table() {
        let data = OrdersOutput { orders: vec![] };
        render(OutputFormat::Table, &data).unwrap();
    }

    #[test]
    fn test_render_json_or_returns_false_for_table() {
        let data = OrdersOutput { orders: vec![] };
        let was_json = render_json_or(OutputFormat::Table, &data).unwrap();
        assert!(!was_json);
    }

    #[test]
    fn test_render_json_or_returns_true_for_json() {
        let data = OrdersOutput { orders: vec![] };
        let was_json = render_json_or(OutputFormat::Json, &data).unwrap();
        assert!(was_json);
    }
}
