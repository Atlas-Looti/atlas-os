//! Universal error types for Atlas OS.
//!
//! PRD-compliant structured error model. Every error carries:
//! - **code**: machine-readable error code (e.g. `SLIPPAGE_EXCEEDED`)
//! - **category**: error class (`auth`, `config`, `execution`, `network`, `validation`, `system`)
//! - **recoverable**: whether the agent can retry or fix
//! - **hints**: actionable suggestions for recovery
//!
//! JSON output format:
//! ```json
//! {
//!   "ok": false,
//!   "error": {
//!     "code": "INSUFFICIENT_MARGIN",
//!     "message": "Not enough margin for this trade",
//!     "category": "execution",
//!     "recoverable": true,
//!     "hints": ["Reduce position size", "Add margin with: atlas hl perp margin add ETH 100"]
//!   }
//! }
//! ```

use serde::Serialize;
use thiserror::Error;

/// Error category — determines exit code and recovery strategy.
///
/// Exit codes (PRD spec):
/// - `0`: success
/// - `1`: user error (auth, config, validation)
/// - `2`: network error
/// - `3`: system error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    Auth,
    Config,
    Execution,
    Network,
    Validation,
    System,
}

impl ErrorCategory {
    /// PRD exit code for this category.
    pub fn exit_code(self) -> i32 {
        match self {
            ErrorCategory::Auth => 1,
            ErrorCategory::Config => 1,
            ErrorCategory::Validation => 1,
            ErrorCategory::Execution => 1,
            ErrorCategory::Network => 2,
            ErrorCategory::System => 3,
        }
    }
}

/// Structured error detail for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub category: ErrorCategory,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

/// Top-level error type for all Atlas OS operations.
///
/// Each variant maps to a specific error code, category, and recovery info.
/// Use the `detail()` method to get the structured representation.
#[derive(Debug, Error)]
pub enum AtlasError {
    // ── Auth ─────────────────────────────────────────────────────────
    #[error("No profile configured")]
    NoProfile,

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("API key missing")]
    ApiKeyMissing,

    #[error("Authentication error: {0}")]
    Auth(String),

    // ── Config ───────────────────────────────────────────────────────
    #[error("Module '{0}' is disabled")]
    ModuleDisabled(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Network mismatch: {0}")]
    NetworkMismatch(String),

    #[error("Configuration error: {0}")]
    Config(String),

    // ── Execution ────────────────────────────────────────────────────
    #[error("Slippage exceeded: {0}")]
    SlippageExceeded(String),

    #[error("Insufficient margin: {0}")]
    InsufficientMargin(String),

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Protocol error ({protocol}): {message}")]
    Protocol { protocol: String, message: String },

    // ── Network ─────────────────────────────────────────────────────
    #[error("Backend unreachable: {0}")]
    BackendUnreachable(String),

    #[error("Protocol timeout: {0}")]
    ProtocolTimeout(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Network error: {0}")]
    Network(String),

    // ── Validation ───────────────────────────────────────────────────
    #[error("Invalid size: {0}")]
    InvalidSize(String),

    #[error("Invalid ticker: {0}")]
    InvalidTicker(String),

    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    // ── System ───────────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("{0}")]
    Other(String),
}

impl AtlasError {
    /// Get the structured error detail for JSON output.
    pub fn detail(&self) -> ErrorDetail {
        match self {
            // Auth
            AtlasError::NoProfile => ErrorDetail {
                code: "NO_PROFILE".into(),
                message: self.to_string(),
                category: ErrorCategory::Auth,
                recoverable: true,
                hints: vec!["Run: atlas profile generate main".into()],
            },
            AtlasError::KeyringError(msg) => ErrorDetail {
                code: "KEYRING_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::Auth,
                recoverable: false,
                hints: vec!["Check OS keyring service is running".into()],
            },
            AtlasError::ApiKeyMissing => ErrorDetail {
                code: "API_KEY_MISSING".into(),
                message: self.to_string(),
                category: ErrorCategory::Auth,
                recoverable: true,
                hints: vec![
                    "Run: atlas configure system api-key <key>".into(),
                    "Get key from apps/frontend → Settings → API Keys".into(),
                ],
            },
            AtlasError::Auth(msg) => ErrorDetail {
                code: "AUTH_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::Auth,
                recoverable: false,
                hints: vec![],
            },

            // Config
            AtlasError::ModuleDisabled(module) => ErrorDetail {
                code: "MODULE_DISABLED".into(),
                message: format!("Module '{module}' is disabled"),
                category: ErrorCategory::Config,
                recoverable: true,
                hints: vec![format!("Run: atlas configure module enable {module}")],
            },
            AtlasError::InvalidConfig(msg) => ErrorDetail {
                code: "INVALID_CONFIG".into(),
                message: msg.clone(),
                category: ErrorCategory::Config,
                recoverable: true,
                hints: vec!["Check atlas.json or run: atlas doctor --output json".into()],
            },
            AtlasError::NetworkMismatch(msg) => ErrorDetail {
                code: "NETWORK_MISMATCH".into(),
                message: msg.clone(),
                category: ErrorCategory::Config,
                recoverable: true,
                hints: vec!["Run: atlas configure module set hl network mainnet".into()],
            },
            AtlasError::Config(msg) => ErrorDetail {
                code: "CONFIG_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::Config,
                recoverable: true,
                hints: vec![],
            },

            // Execution
            AtlasError::SlippageExceeded(msg) => ErrorDetail {
                code: "SLIPPAGE_EXCEEDED".into(),
                message: msg.clone(),
                category: ErrorCategory::Execution,
                recoverable: true,
                hints: vec![
                    "Increase --slippage tolerance".into(),
                    "Retry immediately — volatility event".into(),
                ],
            },
            AtlasError::InsufficientMargin(msg) => ErrorDetail {
                code: "INSUFFICIENT_MARGIN".into(),
                message: msg.clone(),
                category: ErrorCategory::Execution,
                recoverable: true,
                hints: vec![
                    "Reduce position size".into(),
                    "Add margin with: atlas hl perp margin add <coin> <amount>".into(),
                ],
            },
            AtlasError::PositionNotFound(msg) => ErrorDetail {
                code: "POSITION_NOT_FOUND".into(),
                message: msg.clone(),
                category: ErrorCategory::Execution,
                recoverable: false,
                hints: vec!["Check open positions: atlas hl perp positions --output json".into()],
            },
            AtlasError::OrderRejected(msg) => ErrorDetail {
                code: "ORDER_REJECTED".into(),
                message: msg.clone(),
                category: ErrorCategory::Execution,
                recoverable: true,
                hints: vec!["Check order parameters and account state".into()],
            },
            AtlasError::InsufficientBalance(msg) => ErrorDetail {
                code: "INSUFFICIENT_BALANCE".into(),
                message: msg.clone(),
                category: ErrorCategory::Execution,
                recoverable: true,
                hints: vec!["Check balance: atlas status --output json".into()],
            },
            AtlasError::Protocol { protocol, message } => ErrorDetail {
                code: "PROTOCOL_ERROR".into(),
                message: format!("{protocol}: {message}"),
                category: ErrorCategory::Execution,
                recoverable: true,
                hints: vec![],
            },

            // Network
            AtlasError::BackendUnreachable(msg) => ErrorDetail {
                code: "BACKEND_UNREACHABLE".into(),
                message: msg.clone(),
                category: ErrorCategory::Network,
                recoverable: true,
                hints: vec![
                    "Check network connectivity".into(),
                    "Retry in a few seconds".into(),
                ],
            },
            AtlasError::ProtocolTimeout(msg) => ErrorDetail {
                code: "PROTOCOL_TIMEOUT".into(),
                message: msg.clone(),
                category: ErrorCategory::Network,
                recoverable: true,
                hints: vec!["Retry — server may be temporarily slow".into()],
            },
            AtlasError::RateLimited(msg) => ErrorDetail {
                code: "RATE_LIMITED".into(),
                message: msg.clone(),
                category: ErrorCategory::Network,
                recoverable: true,
                hints: vec!["Wait a few seconds and retry".into()],
            },
            AtlasError::Network(msg) => ErrorDetail {
                code: "NETWORK_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::Network,
                recoverable: true,
                hints: vec!["Check network connectivity".into()],
            },

            // Validation
            AtlasError::InvalidSize(msg) => ErrorDetail {
                code: "INVALID_SIZE".into(),
                message: msg.clone(),
                category: ErrorCategory::Validation,
                recoverable: true,
                hints: vec!["Size must be a positive number. Use: 200, 0.5eth, 10lots".into()],
            },
            AtlasError::InvalidTicker(msg) => ErrorDetail {
                code: "INVALID_TICKER".into(),
                message: msg.clone(),
                category: ErrorCategory::Validation,
                recoverable: true,
                hints: vec![
                    "List available markets: atlas market hyperliquid list --output json".into(),
                ],
            },
            AtlasError::UnsupportedChain(msg) => ErrorDetail {
                code: "UNSUPPORTED_CHAIN".into(),
                message: msg.clone(),
                category: ErrorCategory::Validation,
                recoverable: true,
                hints: vec!["Check supported chains: atlas 0x chains --output json".into()],
            },
            AtlasError::AssetNotFound(msg) => ErrorDetail {
                code: "ASSET_NOT_FOUND".into(),
                message: msg.clone(),
                category: ErrorCategory::Validation,
                recoverable: true,
                hints: vec![
                    "Check available assets: atlas market hyperliquid list --output json".into(),
                ],
            },

            // System
            AtlasError::Database(msg) => ErrorDetail {
                code: "DATABASE_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::System,
                recoverable: false,
                hints: vec!["Run: atlas doctor --fix".into()],
            },
            AtlasError::Internal(msg) => ErrorDetail {
                code: "INTERNAL_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::System,
                recoverable: false,
                hints: vec![],
            },
            AtlasError::Other(msg) => ErrorDetail {
                code: "UNKNOWN_ERROR".into(),
                message: msg.clone(),
                category: ErrorCategory::System,
                recoverable: false,
                hints: vec![],
            },
        }
    }

    /// PRD exit code: 0 success, 1 user error, 2 network, 3 system.
    pub fn exit_code(&self) -> i32 {
        self.detail().category.exit_code()
    }

    /// Serialize this error as the PRD-compliant JSON error envelope.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "ok": false,
            "error": self.detail(),
        })
    }
}

pub type AtlasResult<T> = Result<T, AtlasError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detail_slippage() {
        let err = AtlasError::SlippageExceeded("Price moved 2.3% > 1% limit".into());
        let detail = err.detail();
        assert_eq!(detail.code, "SLIPPAGE_EXCEEDED");
        assert_eq!(detail.category, ErrorCategory::Execution);
        assert!(detail.recoverable);
        assert!(!detail.hints.is_empty());
    }

    #[test]
    fn test_error_exit_codes() {
        assert_eq!(AtlasError::NoProfile.exit_code(), 1);
        assert_eq!(AtlasError::Network("timeout".into()).exit_code(), 2);
        assert_eq!(AtlasError::Database("corrupt".into()).exit_code(), 3);
        assert_eq!(AtlasError::InvalidSize("bad".into()).exit_code(), 1);
    }

    #[test]
    fn test_error_json_format() {
        let err = AtlasError::InsufficientMargin("Need $500 more".into());
        let json = err.to_json();
        assert_eq!(json["ok"], false);
        assert_eq!(json["error"]["code"], "INSUFFICIENT_MARGIN");
        assert_eq!(json["error"]["category"], "execution");
        assert_eq!(json["error"]["recoverable"], true);
        assert!(json["error"]["hints"].is_array());
    }

    #[test]
    fn test_error_json_no_empty_hints() {
        let err = AtlasError::Auth("unknown".into());
        let _json = err.to_json();
        // hints should not be present when empty (skip_serializing_if)
        let detail = err.detail();
        let serialized = serde_json::to_string(&detail).unwrap();
        assert!(!serialized.contains("\"hints\""));
    }

    #[test]
    fn test_protocol_error_detail() {
        let err = AtlasError::Protocol {
            protocol: "hyperliquid".into(),
            message: "Order size too small".into(),
        };
        let detail = err.detail();
        assert_eq!(detail.code, "PROTOCOL_ERROR");
        assert!(detail.message.contains("hyperliquid"));
    }

    #[test]
    fn test_all_categories_have_exit_codes() {
        // Auth/Config/Validation → 1
        assert_eq!(ErrorCategory::Auth.exit_code(), 1);
        assert_eq!(ErrorCategory::Config.exit_code(), 1);
        assert_eq!(ErrorCategory::Validation.exit_code(), 1);
        assert_eq!(ErrorCategory::Execution.exit_code(), 1);
        // Network → 2
        assert_eq!(ErrorCategory::Network.exit_code(), 2);
        // System → 3
        assert_eq!(ErrorCategory::System.exit_code(), 3);
    }
}
