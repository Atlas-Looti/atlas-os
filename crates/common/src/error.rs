//! Universal error types for Atlas OS.

use thiserror::Error;

/// Top-level error type for all Atlas OS operations.
#[derive(Debug, Error)]
pub enum AtlasError {
    #[error("Protocol error ({protocol}): {message}")]
    Protocol {
        protocol: String,
        message: String,
    },

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("{0}")]
    Other(String),
}

pub type AtlasResult<T> = Result<T, AtlasError>;
