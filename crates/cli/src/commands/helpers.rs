//! Shared helpers for CLI commands.

use anyhow::Result;

/// Normalize protocol name aliases.
pub fn normalize_protocol(p: &str) -> String {
    match p.to_lowercase().as_str() {
        "hl" | "hyperliquid" | "perp" => "hyperliquid".to_string(),
        "0x" | "zero_x" | "zerox" | "swap" => "0x".to_string(),
        other => other.to_string(),
    }
}

/// Parse an ISO date string to millisecond timestamp.
/// Accepts "2025-01-01" or "2025-01-01T00:00:00".
pub fn parse_date_to_ms(s: &str) -> Result<i64> {
    use chrono::NaiveDateTime;

    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.and_utc().timestamp_millis());
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = NaiveDateTime::new(d, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        return Ok(dt.and_utc().timestamp_millis());
    }
    anyhow::bail!("Invalid date format: {s}. Use YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS")
}

/// Format a millisecond timestamp to human-readable UTC string.
pub fn format_ms(ms: i64) -> String {
    chrono::DateTime::from_timestamp(ms / 1000, 0)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string())
}
