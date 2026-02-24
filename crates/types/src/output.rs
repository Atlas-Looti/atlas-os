//! Structured output types for JSON/table rendering.
//!
//! Every data-producing command returns one of these types.
//! They all derive `Serialize` for JSON output, and implement
//! `TableDisplay` for human-readable table rendering.

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
    pub coin: String,
    pub size: String,
    pub entry_price: Option<String>,
    pub unrealized_pnl: Option<String>,
}

// ─── Orders ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OrdersOutput {
    pub orders: Vec<OrderRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderRow {
    pub coin: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub oid: u64,
}

// ─── Fills ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FillsOutput {
    pub fills: Vec<FillRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FillRow {
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
    pub oid: u64,
    /// "filled", "resting", "accepted"
    pub status: String,
    pub total_sz: Option<String>,
    pub avg_px: Option<String>,
}

// ─── Cancel ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CancelOutput {
    pub coin: String,
    pub cancelled: u32,
    pub total: u32,
    pub oids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CancelSingleOutput {
    pub coin: String,
    pub oid: u64,
    pub status: String,
}

// ─── Leverage ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct LeverageOutput {
    pub coin: String,
    pub leverage: u32,
    pub mode: String,
}

// ─── Margin ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct MarginOutput {
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
}

impl DoctorCheck {
    pub fn ok(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "ok".into(),
            value: Some(value.into()),
            fix: None,
        }
    }

    pub fn ok_bare(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "ok".into(),
            value: None,
            fix: None,
        }
    }

    pub fn fail(name: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: "fail".into(),
            value: None,
            fix: Some(fix.into()),
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
    pub coin: String,
    pub mid_price: String,
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
                size: "0.5".into(),
                entry_price: Some("3500.00".into()),
                unrealized_pnl: Some("25.00".into()),
            }],
            open_orders: 2,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"profile\":\"default\""));
        assert!(json.contains("\"coin\":\"ETH\""));
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
        assert!(json.contains("\"oid\":12345"));
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
            status: "filled".into(),
            total_sz: Some("0.5".into()),
            avg_px: Some("3500.00".into()),
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
                },
                PriceRow {
                    coin: "ETH".into(),
                    mid_price: "3521.25".into(),
                },
            ],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"coin\":\"BTC\""));
        assert!(json.contains("\"mid_price\":\"105234.50\""));
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
                    size: "1.5".into(),
                    entry_price: Some("3500.00".into()),
                    unrealized_pnl: Some("100.00".into()),
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
        assert!(json.contains("\"coin\":\"ETH\""));
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
}
