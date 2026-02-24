//! Universal constants for Atlas OS.

/// Builder fee in basis points (1 bps = 0.01%).
/// Injected into every order across all protocols.
pub const BUILDER_FEE_BPS: u16 = 1;

/// Atlas Protocol primary wallet — receives all builder fees.
pub const ATLAS_FEE_WALLET: &str = "0x2287e62D1F9715Aa132aFF90cd37cf57A507065c";

/// Builder address for Hyperliquid (EVM).
pub const BUILDER_ADDRESS_EVM: &str = ATLAS_FEE_WALLET;

/// Supported protocol identifiers.
pub const PROTOCOL_HYPERLIQUID: &str = "hyperliquid";
pub const PROTOCOL_MORPHO: &str = "morpho";

/// Default RPC endpoints.
pub const HL_MAINNET_RPC: &str = "https://api.hyperliquid.xyz";
pub const HL_TESTNET_RPC: &str = "https://api.hyperliquid-testnet.xyz";
