/// Builder fee constants for protocol revenue injection.
///
/// ╔══════════════════════════════════════════════════════════════════╗
/// ║  BUILDER FEE — MANDATORY REVENUE LAYER                        ║
/// ║                                                                ║
/// ║  Every order submitted through Atlas MUST include the builder  ║
/// ║  parameter pointing to BUILDER_ADDRESS. This is the sole      ║
/// ║  monetization mechanism of the protocol. Do NOT remove or      ║
/// ║  bypass this. Any order path that skips builder fee injection  ║
/// ║  is a critical bug.                                            ║
/// ╚══════════════════════════════════════════════════════════════════╝

use serde::{Deserialize, Serialize};

/// The address that receives builder fees on Hyperliquid.
/// Replace with the actual revenue wallet before production deployment.
pub const BUILDER_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Builder fee in basis points (10 bps = 0.1%).
pub const BUILDER_FEE_BPS: u16 = 10;

/// Builder fee payload injected into the exchange action JSON.
///
/// The Hyperliquid API accepts this as part of the order action:
/// ```json
/// {
///   "action": {
///     "type": "order",
///     "orders": [...],
///     "grouping": "na",
///     "builder": { "b": "0xADDRESS", "f": 10 }
///   }
/// }
/// ```
///
/// The builder field is NOT part of the signed data (excluded from the
/// RMP hash), so it can be injected after signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderFee {
    /// Builder address (hex with 0x prefix).
    pub b: String,
    /// Fee in basis points (e.g. 10 = 0.1%).
    pub f: u16,
}

impl Default for BuilderFee {
    fn default() -> Self {
        Self {
            b: BUILDER_ADDRESS.to_string(),
            f: BUILDER_FEE_BPS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_address_is_valid_hex() {
        assert!(BUILDER_ADDRESS.starts_with("0x"));
        assert_eq!(BUILDER_ADDRESS.len(), 42);
    }

    #[test]
    fn test_builder_fee_reasonable() {
        // Fee should be between 1 and 100 bps
        assert!(BUILDER_FEE_BPS >= 1);
        assert!(BUILDER_FEE_BPS <= 100);
    }

    #[test]
    fn test_builder_fee_default() {
        let fee = BuilderFee::default();
        assert_eq!(fee.b, BUILDER_ADDRESS);
        assert_eq!(fee.f, BUILDER_FEE_BPS);
    }

    #[test]
    fn test_builder_fee_serialization() {
        let fee = BuilderFee::default();
        let json = serde_json::to_string(&fee).unwrap();
        assert!(json.contains("\"b\""));
        assert!(json.contains("\"f\""));
    }
}
