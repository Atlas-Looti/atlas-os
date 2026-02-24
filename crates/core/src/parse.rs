use anyhow::{bail, Result};
use crate::config::SizeInput;

/// Parse "buy"/"sell"/"long"/"short" into a boolean (true = buy).
pub fn parse_side(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "buy" | "b" | "long" | "l" => Ok(true),
        "sell" | "s" | "short" | "sh" => Ok(false),
        _ => bail!("Invalid side '{s}'. Use: buy, sell, long, short, b, s"),
    }
}

/// Parse a numeric string, stripping optional leading '$' or '%'.
pub fn parse_amount(s: &str) -> Result<f64> {
    let cleaned = s
        .trim()
        .trim_start_matches('$')
        .trim_end_matches('%')
        .trim();

    cleaned
        .parse::<f64>()
        .map_err(|_| anyhow::anyhow!("Invalid number: '{s}'"))
}

/// Parse a size input string into a `SizeInput`.
///
/// Explicit suffixes (always override default_size_mode):
///   - `"$200"`, `"200$"`, `"200usdc"`, `"200u"` → `Usdc(200.0)`
///   - `"0.5eth"`, `"0.5units"` → `Units(0.5)`
///   - `"50lots"`, `"50l"` → `Lots(50.0)`
///
/// Bare numbers: `"200"` → `Raw(200.0)` — interpreted by config's `default_size_mode`.
pub fn parse_size(s: &str) -> Result<SizeInput> {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        bail!("Size cannot be empty");
    }

    // ── USDC prefix: "$200" ──
    if let Some(stripped) = trimmed.strip_prefix('$') {
        let num = stripped.trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid USDC amount: '{s}'"))?;
        return Ok(SizeInput::Usdc(val));
    }

    let lower = trimmed.to_lowercase();

    // ── USDC suffix: "200$", "200usdc", "200u" ──
    if lower.ends_with('$') {
        let num = &trimmed[..trimmed.len() - 1].trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid USDC amount: '{s}'"))?;
        return Ok(SizeInput::Usdc(val));
    }
    if lower.ends_with("usdc") {
        let num = &trimmed[..trimmed.len() - 4].trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid USDC amount: '{s}'"))?;
        return Ok(SizeInput::Usdc(val));
    }

    // ── Lots suffix: "50lots", "50lot", "50l" ──
    if lower.ends_with("lots") {
        let num = &trimmed[..trimmed.len() - 4].trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid lot amount: '{s}'"))?;
        return Ok(SizeInput::Lots(val));
    }
    if lower.ends_with("lot") {
        let num = &trimmed[..trimmed.len() - 3].trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid lot amount: '{s}'"))?;
        return Ok(SizeInput::Lots(val));
    }
    if lower.ends_with('l') && !lower.ends_with("al") {
        // "50l" but not "0.5hal" — check that prefix is numeric
        let num = &trimmed[..trimmed.len() - 1].trim();
        if let Ok(val) = num.parse::<f64>() {
            return Ok(SizeInput::Lots(val));
        }
    }

    // ── Units suffix: "0.5eth", "0.5btc", "0.5units", "0.5unit" ──
    if lower.ends_with("units") {
        let num = &trimmed[..trimmed.len() - 5].trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid unit amount: '{s}'"))?;
        return Ok(SizeInput::Units(val));
    }
    if lower.ends_with("unit") {
        let num = &trimmed[..trimmed.len() - 4].trim();
        let val: f64 = num.parse()
            .map_err(|_| anyhow::anyhow!("Invalid unit amount: '{s}'"))?;
        return Ok(SizeInput::Units(val));
    }
    // Common asset suffixes → explicit units
    for suffix in &["eth", "btc", "sol", "doge", "arb", "avax", "matic", "link", "op", "sui", "bnb", "xrp", "ada", "dot", "atom"] {
        if lower.ends_with(suffix) {
            let num = &trimmed[..trimmed.len() - suffix.len()].trim();
            if let Ok(val) = num.parse::<f64>() {
                return Ok(SizeInput::Units(val));
            }
        }
    }

    // ── "u" shorthand for USDC: "200u" ──
    if lower.ends_with('u') {
        let num = &trimmed[..trimmed.len() - 1].trim();
        if let Ok(val) = num.parse::<f64>() {
            return Ok(SizeInput::Usdc(val));
        }
    }

    // ── Default: bare number → Raw (interpreted by config) ──
    let val: f64 = trimmed.parse()
        .map_err(|_| anyhow::anyhow!(
            "Invalid size: '{s}'. Examples: 200 (default mode), $200 (USDC), 0.5eth (units), 50lots"
        ))?;
    Ok(SizeInput::Raw(val))
}

/// Parse a hex-encoded address, validating basic format.
pub fn parse_address(s: &str) -> Result<String> {
    let addr = s.trim();
    if !addr.starts_with("0x") || addr.len() != 42 {
        bail!("Invalid address '{addr}'. Must be 0x-prefixed and 42 chars.");
    }
    // Validate hex characters
    if !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Invalid hex in address '{addr}'.");
    }
    Ok(addr.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_side_buy_variants() {
        assert!(parse_side("buy").unwrap());
        assert!(parse_side("Buy").unwrap());
        assert!(parse_side("BUY").unwrap());
        assert!(parse_side("b").unwrap());
        assert!(parse_side("B").unwrap());
        assert!(parse_side("long").unwrap());
        assert!(parse_side("LONG").unwrap());
        assert!(parse_side("l").unwrap());
    }

    #[test]
    fn test_parse_side_sell_variants() {
        assert!(!parse_side("sell").unwrap());
        assert!(!parse_side("Sell").unwrap());
        assert!(!parse_side("SELL").unwrap());
        assert!(!parse_side("s").unwrap());
        assert!(!parse_side("short").unwrap());
        assert!(!parse_side("SHORT").unwrap());
        assert!(!parse_side("sh").unwrap());
    }

    #[test]
    fn test_parse_side_invalid() {
        assert!(parse_side("invalid").is_err());
        assert!(parse_side("").is_err());
        assert!(parse_side("hold").is_err());
    }

    #[test]
    fn test_parse_amount_plain() {
        assert_eq!(parse_amount("123.45").unwrap(), 123.45);
        assert_eq!(parse_amount("0").unwrap(), 0.0);
        assert_eq!(parse_amount("-50").unwrap(), -50.0);
    }

    #[test]
    fn test_parse_amount_with_dollar() {
        assert_eq!(parse_amount("$100").unwrap(), 100.0);
        assert_eq!(parse_amount("$99.99").unwrap(), 99.99);
    }

    #[test]
    fn test_parse_amount_with_percent() {
        assert_eq!(parse_amount("5%").unwrap(), 5.0);
    }

    #[test]
    fn test_parse_amount_whitespace() {
        assert_eq!(parse_amount("  42  ").unwrap(), 42.0);
    }

    #[test]
    fn test_parse_amount_invalid() {
        assert!(parse_amount("abc").is_err());
        assert!(parse_amount("").is_err());
    }

    #[test]
    fn test_parse_address_valid() {
        let addr = parse_address("0xe8Ecb4D59690d1E1748217e1b56B73D51A8Bc94C").unwrap();
        assert_eq!(addr, "0xe8Ecb4D59690d1E1748217e1b56B73D51A8Bc94C");
    }

    #[test]
    fn test_parse_address_valid_lowercase() {
        let addr = parse_address("0x0000000000000000000000000000000000000000").unwrap();
        assert_eq!(addr, "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn test_parse_address_no_prefix() {
        assert!(parse_address("e8Ecb4D59690d1E1748217e1b56B73D51A8Bc94C").is_err());
    }

    #[test]
    fn test_parse_address_too_short() {
        assert!(parse_address("0x123").is_err());
    }

    #[test]
    fn test_parse_address_invalid_chars() {
        assert!(parse_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG").is_err());
    }

    #[test]
    fn test_parse_address_with_whitespace() {
        let addr = parse_address("  0xe8Ecb4D59690d1E1748217e1b56B73D51A8Bc94C  ").unwrap();
        assert!(addr.starts_with("0x"));
    }

    // ── parse_size tests ────────────────────────────────────────

    #[test]
    fn test_parse_size_raw_number() {
        assert_eq!(parse_size("0.5").unwrap(), SizeInput::Raw(0.5));
        assert_eq!(parse_size("100").unwrap(), SizeInput::Raw(100.0));
        assert_eq!(parse_size("0.001").unwrap(), SizeInput::Raw(0.001));
    }

    #[test]
    fn test_parse_size_dollar_prefix() {
        assert_eq!(parse_size("$200").unwrap(), SizeInput::Usdc(200.0));
        assert_eq!(parse_size("$50.5").unwrap(), SizeInput::Usdc(50.5));
        assert_eq!(parse_size("$1000").unwrap(), SizeInput::Usdc(1000.0));
    }

    #[test]
    fn test_parse_size_dollar_suffix() {
        assert_eq!(parse_size("200$").unwrap(), SizeInput::Usdc(200.0));
        assert_eq!(parse_size("50.5$").unwrap(), SizeInput::Usdc(50.5));
    }

    #[test]
    fn test_parse_size_usdc_suffix() {
        assert_eq!(parse_size("200usdc").unwrap(), SizeInput::Usdc(200.0));
        assert_eq!(parse_size("200USDC").unwrap(), SizeInput::Usdc(200.0));
        assert_eq!(parse_size("50.5usdc").unwrap(), SizeInput::Usdc(50.5));
    }

    #[test]
    fn test_parse_size_u_shorthand() {
        assert_eq!(parse_size("200u").unwrap(), SizeInput::Usdc(200.0));
        assert_eq!(parse_size("50u").unwrap(), SizeInput::Usdc(50.0));
    }

    #[test]
    fn test_parse_size_explicit_units() {
        assert_eq!(parse_size("0.5eth").unwrap(), SizeInput::Units(0.5));
        assert_eq!(parse_size("0.001btc").unwrap(), SizeInput::Units(0.001));
        assert_eq!(parse_size("10sol").unwrap(), SizeInput::Units(10.0));
        assert_eq!(parse_size("0.5units").unwrap(), SizeInput::Units(0.5));
        assert_eq!(parse_size("1.0unit").unwrap(), SizeInput::Units(1.0));
    }

    #[test]
    fn test_parse_size_explicit_lots() {
        assert_eq!(parse_size("50lots").unwrap(), SizeInput::Lots(50.0));
        assert_eq!(parse_size("50lot").unwrap(), SizeInput::Lots(50.0));
        assert_eq!(parse_size("50l").unwrap(), SizeInput::Lots(50.0));
        assert_eq!(parse_size("100.5lots").unwrap(), SizeInput::Lots(100.5));
    }

    #[test]
    fn test_parse_size_whitespace() {
        assert_eq!(parse_size("  $200  ").unwrap(), SizeInput::Usdc(200.0));
        assert_eq!(parse_size("  0.5  ").unwrap(), SizeInput::Raw(0.5));
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("$abc").is_err());
    }
}
