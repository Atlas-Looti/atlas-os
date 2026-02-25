//! Formatting utilities shared across CLI, TUI, and core.

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
pub fn order_result_to_output(r: &crate::types::OrderResult) -> crate::output::OrderResultOutput {
    crate::output::OrderResultOutput {
        oid: r.order_id.parse().unwrap_or(0),
        coin: r.coin.clone().unwrap_or_default(),
        side: r
            .side
            .as_ref()
            .map(|s| format!("{s:?}").to_lowercase())
            .unwrap_or_default(),
        status: format!("{:?}", r.status).to_lowercase(),
        total_sz: r.filled_size.map(|s| s.to_string()),
        avg_px: r.avg_price.map(|p| p.to_string()),
        filled: r.filled_size.map(|s| s.to_string()),
        fee: r.fee.map(|f| f.to_string()),
        builder_fee_bps: crate::constants::BUILDER_FEE_BPS as u32,
        protocol: format!("{}", r.protocol),
        timestamp: r.timestamp,
    }
}

/// Truncate a numeric string to reasonable display width.
/// Adapts decimal places based on magnitude.
pub fn truncate_number(s: &str) -> String {
    if s == "—" || s == "-" {
        return s.to_string();
    }
    if let Ok(n) = s.parse::<f64>() {
        if n.abs() >= 100_000.0 {
            format!("{:.0}", n)
        } else if n.abs() >= 1000.0 {
            format!("{:.2}", n)
        } else if n.abs() >= 1.0 {
            format!("{:.4}", n)
        } else {
            format!("{:.6}", n)
        }
    } else {
        truncate_str(s, 12).to_string()
    }
}

/// Format a numeric string as USD (e.g. "$1,234.56").
pub fn format_usd(s: &str) -> String {
    if s == "—" || s == "-" {
        return s.to_string();
    }
    if let Ok(n) = s.parse::<f64>() {
        if n.abs() >= 1_000_000.0 {
            format!("${:.2}M", n / 1_000_000.0)
        } else if n.abs() >= 1_000.0 {
            format!("${:.2}K", n / 1_000.0)
        } else {
            format!("${:.2}", n)
        }
    } else {
        s.to_string()
    }
}

/// Format a numeric string as USD without abbreviation.
pub fn format_usd_full(s: &str) -> String {
    if s == "—" || s == "-" {
        return s.to_string();
    }
    if let Ok(n) = s.parse::<f64>() {
        format!("${:.2}", n)
    } else {
        s.to_string()
    }
}

/// Format a decimal ratio as percentage (e.g. 0.05 → "5.00%").
pub fn format_pct(s: &str) -> String {
    if s == "—" || s == "-" {
        return s.to_string();
    }
    if let Ok(n) = s.parse::<f64>() {
        format!("{:.2}%", n * 100.0)
    } else {
        s.to_string()
    }
}

/// Truncate an EVM address for display: 0x1234...abcd
pub fn truncate_address(s: &str) -> String {
    if s.len() > 12 {
        format!("{}...{}", &s[..6], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}

/// Generic string truncation.
pub fn truncate_str(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

/// Format a side boolean as display string.
pub fn format_side(is_buy: bool) -> &'static str {
    if is_buy {
        "BUY"
    } else {
        "SELL"
    }
}

/// Format a side string from SDK ("B"/"S") to display.
pub fn format_side_letter(s: &str) -> &'static str {
    if s == "B" {
        "BUY"
    } else {
        "SELL"
    }
}

/// Determine if a numeric string is positive, negative, or zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
    Zero,
}

/// Get the sign of a numeric string.
pub fn sign_of(s: &str) -> Sign {
    let trimmed = s.trim();
    if trimmed.starts_with('-') {
        Sign::Negative
    } else if trimmed == "0" || trimmed == "0.0" || trimmed == "0.00" || trimmed == "—" {
        Sign::Zero
    } else {
        Sign::Positive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_number_large() {
        assert_eq!(truncate_number("1234567.89"), "1234568");
    }

    #[test]
    fn test_truncate_number_medium() {
        assert_eq!(truncate_number("12345.6789"), "12345.68");
    }

    #[test]
    fn test_truncate_number_small() {
        assert_eq!(truncate_number("123.456789"), "123.4568");
    }

    #[test]
    fn test_truncate_number_tiny() {
        assert_eq!(truncate_number("0.00123456"), "0.001235");
    }

    #[test]
    fn test_truncate_number_dash() {
        assert_eq!(truncate_number("—"), "—");
    }

    #[test]
    fn test_truncate_number_negative() {
        assert_eq!(truncate_number("-45.6789"), "-45.6789");
    }

    #[test]
    fn test_truncate_number_zero() {
        assert_eq!(truncate_number("0"), "0.000000");
    }

    #[test]
    fn test_format_usd_millions() {
        assert_eq!(format_usd("1234567.89"), "$1.23M");
    }

    #[test]
    fn test_format_usd_thousands() {
        assert_eq!(format_usd("12345.67"), "$12.35K");
    }

    #[test]
    fn test_format_usd_normal() {
        assert_eq!(format_usd("123.45"), "$123.45");
    }

    #[test]
    fn test_format_usd_dash() {
        assert_eq!(format_usd("—"), "—");
    }

    #[test]
    fn test_format_usd_negative() {
        assert_eq!(format_usd("-500.00"), "$-500.00");
    }

    #[test]
    fn test_format_usd_full() {
        assert_eq!(format_usd_full("12345.678"), "$12345.68");
        assert_eq!(format_usd_full("—"), "—");
    }

    #[test]
    fn test_format_pct_positive() {
        assert_eq!(format_pct("0.05"), "5.00%");
    }

    #[test]
    fn test_format_pct_negative() {
        assert_eq!(format_pct("-0.12"), "-12.00%");
    }

    #[test]
    fn test_format_pct_zero() {
        assert_eq!(format_pct("0"), "0.00%");
    }

    #[test]
    fn test_truncate_address_long() {
        assert_eq!(
            truncate_address("0xe8Ecb4D59690d1E1748217e1b56B73D51A8Bc94C"),
            "0xe8Ec...c94C"
        );
    }

    #[test]
    fn test_truncate_address_short() {
        assert_eq!(truncate_address("0x1234"), "0x1234");
    }

    #[test]
    fn test_truncate_str_within_limit() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_at_limit() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_str_over_limit() {
        assert_eq!(truncate_str("hello world", 5), "hello");
    }

    #[test]
    fn test_format_side() {
        assert_eq!(format_side(true), "BUY");
        assert_eq!(format_side(false), "SELL");
    }

    #[test]
    fn test_format_side_letter() {
        assert_eq!(format_side_letter("B"), "BUY");
        assert_eq!(format_side_letter("S"), "SELL");
    }

    #[test]
    fn test_sign_of_negative() {
        assert_eq!(sign_of("-123.45"), Sign::Negative);
        assert_eq!(sign_of("-0.01"), Sign::Negative);
    }

    #[test]
    fn test_sign_of_positive() {
        assert_eq!(sign_of("123.45"), Sign::Positive);
        assert_eq!(sign_of("0.01"), Sign::Positive);
    }

    #[test]
    fn test_sign_of_zero() {
        assert_eq!(sign_of("0"), Sign::Zero);
        assert_eq!(sign_of("0.0"), Sign::Zero);
        assert_eq!(sign_of("0.00"), Sign::Zero);
        assert_eq!(sign_of("—"), Sign::Zero);
    }
}
