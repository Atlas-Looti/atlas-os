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
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  ACCOUNT SUMMARY                                       ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║  Profile     : {:<41}║", self.profile);
        println!("║  Address     : {:<41}║", self.address);
        println!("║  Account Val : {:<41}║", self.account_value);
        println!("║  Margin Used : {:<41}║", self.margin_used);
        println!("║  Net Pos     : {:<41}║", self.net_position);
        println!("║  Withdrawable: {:<41}║", self.withdrawable);
        println!("╠══════════════════════════════════════════════════════════╣");

        if self.positions.is_empty() {
            println!("║  No open positions.                                      ║");
        } else {
            println!("║  {:^6} │ {:^10} │ {:^10} │ {:^12} ║", "Coin", "Size", "Entry", "uPnL");
            println!("║  ──────┼────────────┼────────────┼────────────── ║");
            for pos in &self.positions {
                println!(
                    "║  {:^6} │ {:>10} │ {:>10} │ {:>12} ║",
                    pos.coin, pos.size, pos.entry_price, pos.unrealized_pnl,
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
