//! Structured output types for JSON/table rendering.
//!
//! Every data-producing command returns one of these types.
//! They all derive `Serialize` for JSON output, and implement
//! `TableDisplay` for human-readable table rendering.

use std::collections::HashMap;

use serde::Serialize;

// ─── Status ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct StatusOutput {
    pub profile: String,
    pub address: String,
    pub network: String,
    pub account_value: String,
    pub margin_used: String,
    pub net_position: String,
    pub withdrawable: String,
    pub positions: Vec<PositionRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PositionRow {
    pub coin: String,
    pub size: String,
    pub entry_price: String,
    pub unrealized_pnl: String,
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

#[derive(Debug, Clone, Serialize)]
pub struct DoctorOutput {
    pub config_ok: bool,
    pub keystore_ok: bool,
    pub ntp_ok: Option<bool>,
    pub api_latency_ms: Option<u64>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_output_serializes() {
        let output = StatusOutput {
            profile: "default".into(),
            address: "0x1234".into(),
            network: "Mainnet".into(),
            account_value: "10000.00".into(),
            margin_used: "500.00".into(),
            net_position: "2500.00".into(),
            withdrawable: "9500.00".into(),
            positions: vec![PositionRow {
                coin: "ETH".into(),
                size: "0.5".into(),
                entry_price: "3500.00".into(),
                unrealized_pnl: "25.00".into(),
            }],
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"profile\":\"default\""));
        assert!(json.contains("\"coin\":\"ETH\""));
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
            config_ok: true,
            keystore_ok: true,
            ntp_ok: None,
            api_latency_ms: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"config_ok\":true"));
    }

    #[test]
    fn test_json_pretty_format() {
        let output = StatusOutput {
            profile: "default".into(),
            address: "0x1234".into(),
            network: "Mainnet".into(),
            account_value: "10000.00".into(),
            margin_used: "500.00".into(),
            net_position: "2500.00".into(),
            withdrawable: "9500.00".into(),
            positions: vec![],
        };
        let pretty = serde_json::to_string_pretty(&output).unwrap();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  "));
    }

    #[test]
    fn test_price_output_serializes() {
        let output = PriceOutput {
            prices: vec![
                PriceRow { coin: "BTC".into(), mid_price: "105234.50".into() },
                PriceRow { coin: "ETH".into(), mid_price: "3521.25".into() },
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
}
