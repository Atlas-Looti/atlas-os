//! Convert Hyperliquid SDK types → Atlas OS universal types.

use atlas_core::types::*;
use hypersdk::hypercore::types::Side as HlSide;

/// Convert HL Side to universal Side.
pub fn convert_side(side: &HlSide) -> Side {
    match side {
        HlSide::Bid => Side::Buy,
        HlSide::Ask => Side::Sell,
    }
}

/// Convert universal Side to HL is_buy bool.
pub fn side_to_is_buy(side: &Side) -> bool {
    matches!(side, Side::Buy)
}

/// Format a millisecond timestamp to UTC string (no chrono).
pub fn format_timestamp_ms(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let total_days = secs / 86400;
    let day_secs = (secs % 86400) as u32;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

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

/// Convert HL PerpMarket to universal Market.
pub fn perp_market_to_universal(m: &hypersdk::hypercore::PerpMarket) -> Market {
    Market {
        symbol: format!("{}-PERP", m.name),
        base: m.name.clone(),
        quote: "USD".to_string(),
        protocol: Protocol::Hyperliquid,
        chain: Chain::HyperliquidL1,
        market_type: MarketType::Perp,
        mark_price: None,
        index_price: None,
        volume_24h: None,
        open_interest: None,
        funding_rate: None,
        max_leverage: Some(m.max_leverage as u32),
        min_size: None,
        tick_size: None,
        sz_decimals: Some(m.sz_decimals as i32),
    }
}


