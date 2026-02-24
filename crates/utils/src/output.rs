//! Unified output rendering: JSON or human-readable table.
//!
//! Usage:
//! ```ignore
//! use atlas_utils::output::{OutputFormat, render};
//!
//! let data = StatusOutput { ... };
//! render(format, &data)?;
//! ```

use anyhow::Result;
use serde::Serialize;

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

/// Render structured output — JSON or table depending on format.
///
/// For JSON formats, uses `serde_json` serialization.
/// For table format, calls `TableDisplay::print_table()`.
pub fn render<T: Serialize + TableDisplay>(format: OutputFormat, data: &T) -> Result<()> {
    match format {
        OutputFormat::Table => {
            data.print_table();
            Ok(())
        }
        OutputFormat::Json => {
            let json = serde_json::to_string(data)?;
            println!("{json}");
            Ok(())
        }
        OutputFormat::JsonPretty => {
            let json = serde_json::to_string_pretty(data)?;
            println!("{json}");
            Ok(())
        }
    }
}

/// Render just the JSON formats (for types that handle their own table display).
/// Returns true if JSON was rendered, false if table mode was requested.
pub fn render_json_or<T: Serialize>(format: OutputFormat, data: &T) -> Result<bool> {
    match format {
        OutputFormat::Table => Ok(false),
        OutputFormat::Json => {
            let json = serde_json::to_string(data)?;
            println!("{json}");
            Ok(true)
        }
        OutputFormat::JsonPretty => {
            let json = serde_json::to_string_pretty(data)?;
            println!("{json}");
            Ok(true)
        }
    }
}

// ─── TableDisplay implementations for output types ──────────────────

use atlas_types::output::*;

impl TableDisplay for StatusOutput {
    fn print_table(&self) {
        let dash = "—";
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  ACCOUNT SUMMARY                                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Profile     : {:<41}║", self.profile);
        println!("║  Address     : {:<41}║", self.address);
        println!("║  Account Val : {:<41}║", self.account_value.as_deref().unwrap_or(dash));
        println!("║  Margin Used : {:<41}║", self.margin_used.as_deref().unwrap_or(dash));
        println!("║  Net Pos     : {:<41}║", self.net_position.as_deref().unwrap_or(dash));
        println!("║  Withdrawable: {:<41}║", self.withdrawable.as_deref().unwrap_or(dash));
        println!("╠══════════════════════════════════════════════════════════╣");

        if self.positions.is_empty() {
            println!("║  No open positions.                                      ║");
        } else {
            println!("║  {:^6} │ {:^10} │ {:^10} │ {:^12} ║", "Coin", "Size", "Entry", "uPnL");
            println!("║  ──────┼────────────┼────────────┼────────────── ║");
            for pos in &self.positions {
                println!(
                    "║  {:^6} │ {:>10} │ {:>10} │ {:>12} ║",
                    pos.coin, pos.size,
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
                println!("✓ Order FILLED (oid: {}, size: {}, avg_px: {})", self.oid, sz, px);
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
        println!("✓ Cancelled {}/{} orders on {}.", self.cancelled, self.total, self.coin);
    }
}

impl TableDisplay for CancelSingleOutput {
    fn print_table(&self) {
        println!("✓ Order {} on {} cancelled.", self.oid, self.coin);
    }
}

impl TableDisplay for LeverageOutput {
    fn print_table(&self) {
        println!("✓ {} leverage set to {}x ({})", self.coin, self.leverage, self.mode);
    }
}

impl TableDisplay for MarginOutput {
    fn print_table(&self) {
        println!("✓ {} ${} margin on {}", self.action, self.amount, self.coin);
    }
}

impl TableDisplay for TransferOutput {
    fn print_table(&self) {
        println!("✓ Transferred ${} USDC to {}", self.amount, self.destination);
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
        println!("║  Slippage  : {:<43}║", format!("{:.1}%", self.slippage * 100.0));
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
        println!(
            "│  Config         : {}                        │",
            if self.config_ok { "✓" } else { "✗" }
        );
        match self.ntp_ok {
            Some(true) => println!("│  NTP Sync       : ✓                        │"),
            Some(false) => println!("│  NTP Sync       : ✗                        │"),
            None => println!("│  NTP Sync       : ⏳ (not implemented)       │"),
        }
        match self.api_latency_ms {
            Some(ms) => println!("│  API Latency    : {}ms{:>24}│", ms, ""),
            None => println!("│  API Latency    : ⏳ (not implemented)       │"),
        }
        println!(
            "│  Keystore       : {}                        │",
            if self.keystore_ok { "✓" } else { "✗" }
        );
        println!("└─────────────────────────────────────────────┘");
    }
}

impl TableDisplay for RiskCalcOutput {
    fn print_table(&self) {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  RISK CALCULATOR                                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Asset        : {:<6} {:<34}║", self.coin, self.side.to_uppercase());
        println!("║  Entry Price  : ${:<43.4}║", self.entry_price);
        println!("║  Size         : {:<40}║", format!("{:.6} {}", self.size, self.coin));
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
        println!("║  Risk (%)     : {:<43}║", format!("{:.2}%", self.risk_pct * 100.0));
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
        println!("✓ Transferred {} {} ({})", self.amount, self.token, self.direction);
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
        println!("║  Leader Frac  : {:<41}║", format!("{}%", self.leader_fraction));
        println!("║  Commission   : {:<41}║", format!("{}%", self.leader_commission));
        println!("║  Distributable: ${:<40}║", self.max_distributable);
        println!("║  Withdrawable : ${:<40}║", self.max_withdrawable);
        println!("║  Followers    : {:<41}║", self.follower_count);
        println!("║  Closed       : {:<41}║", if self.is_closed { "Yes" } else { "No" });
        println!("║  Deposits     : {:<41}║", if self.allow_deposits { "Allowed" } else { "Closed" });
        println!("╠══════════════════════════════════════════════════════════╣");

        if !self.description.is_empty() {
            println!("║  {:<55}║", self.description.chars().take(55).collect::<String>());
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
            println!("║  {:<20} │ {:>12} │ {:>12} │ {:>4} ║", "User", "Equity", "PnL", "Days");
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

        println!("┌──────────────────────────────────────────────┬──────────────┬──────────────────┐");
        println!("│ Vault Address                                │ Equity       │ Locked Until      │");
        println!("├──────────────────────────────────────────────┼──────────────┼──────────────────┤");
        for d in &self.deposits {
            let locked = d.locked_until.as_deref().unwrap_or("—");
            println!(
                "│ {:<44} │ {:>12} │ {:<16} │",
                d.vault_address, d.equity, locked,
            );
        }
        println!("├──────────────────────────────────────────────┼──────────────┼──────────────────┤");
        println!(
            "│ {:<44} │ {:>12} │ {:<16} │",
            "TOTAL", self.total_equity, "",
        );
        println!("└──────────────────────────────────────────────┴──────────────┴──────────────────┘");
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
                println!("║  {:^6} │ {:^10} │ {:^10} │ {:^12} ║", "Coin", "Size", "Entry", "uPnL");
                println!("║  ──────┼────────────┼────────────┼────────────── ║");
                for pos in &sub.positions {
                    println!(
                        "║  {:^6} │ {:>10} │ {:>10} │ {:>12} ║",
                        pos.coin, pos.size,
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
        println!("✓ Agent {} approved (name: {}, status: {})", self.agent_address, name_display, self.status);
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
        println!("║  Win/Loss     : {:<41}║", format!("{} / {}", self.win_count, self.loss_count));
        println!("║  Win Rate     : {:<41}║", self.win_rate);
        println!("╠══════════════════════════════════════════════════════════╣");

        if !self.by_coin.is_empty() {
            println!("║  BREAKDOWN BY COIN                                     ║");
            println!("║  {:<8} │ {:>12} │ {:>10} │ {:>6}      ║", "Coin", "PnL", "Fees", "Trades");
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
        println!("✓ Sync {} — fills: {}, orders: {}", self.status, self.fills_synced, self.orders_synced);
    }
}

impl TableDisplay for ExportOutput {
    fn print_table(&self) {
        println!("✓ Exported {} rows ({}) → {}", self.rows, self.format, self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
