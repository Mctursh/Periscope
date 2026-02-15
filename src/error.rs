//! Error types for Periscope

use thiserror::Error;

/// Main error type for Periscope operations
#[derive(Debug, Error)]
pub enum PeriscopeError {
    #[error("Program {0} does not have an IDL account")]
    IdlNotFound(String),

    #[error("Failed to decompress IDL data: {0}")]
    DecompressionError(String),

    #[error("Failed to parse IDL JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("RPC error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),

    #[error("Invalid program ID: {0}")]
    InvalidProgramId(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("HTTP error {status}: {url}")]
    HttpError { status: u16, url: String },
}

/// Result type alias for Periscope operations
pub type PeriscopeResult<T> = Result<T, PeriscopeError>;
