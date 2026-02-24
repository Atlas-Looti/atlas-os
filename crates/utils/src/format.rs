//! Formatting utilities — timestamps, etc.

/// Format a millisecond timestamp to human-readable UTC string.
/// Uses the Howard Hinnant algorithm — no chrono dependency.
pub fn format_timestamp_ms(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let total_days = secs / 86400;
    let day_secs = (secs % 86400) as u32;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    // Civil date from days since epoch (Howard Hinnant algorithm)
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

/// Convert universal OrderResult to CLI OrderResultOutput.
pub fn order_result_to_output(r: &atlas_common::types::OrderResult) -> atlas_types::output::OrderResultOutput {
    atlas_types::output::OrderResultOutput {
        oid: r.order_id.parse().unwrap_or(0),
        status: format!("{:?}", r.status).to_lowercase(),
        total_sz: r.filled_size.map(|s| s.to_string()),
        avg_px: r.avg_price.map(|p| p.to_string()),
    }
}
