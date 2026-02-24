//! Universal constants for Atlas OS.

/// Builder fee in basis points (1 bps = 0.01%).
/// Injected into every order across all protocols.
pub const BUILDER_FEE_BPS: u16 = 1;

/// Builder address for Hyperliquid (EVM).
/// Replace with actual revenue wallet before production.
pub const BUILDER_ADDRESS_EVM: &str = "0x0000000000000000000000000000000000000000";

/// Supported protocol identifiers.
pub const PROTOCOL_HYPERLIQUID: &str = "hyperliquid";
pub const PROTOCOL_MORPHO: &str = "morpho";

/// Default RPC endpoints.
pub const HL_MAINNET_RPC: &str = "https://api.hyperliquid.xyz";
pub const HL_TESTNET_RPC: &str = "https://api.hyperliquid-testnet.xyz";
