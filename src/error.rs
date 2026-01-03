//! Error types for the schema registry

use thiserror::Error;

/// Result type for schema operations
pub type Result<T> = std::result::Result<T, SchemaError>;

/// Schema registry errors
#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Schema not found: {name} version {version}")]
    NotFound { name: String, version: String },

    #[error("Schema already exists: {name} version {version}")]
    AlreadyExists { name: String, version: String },

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Breaking change detected: {0}")]
    BreakingChange(String),

    #[error("Invalid schema format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Semver error: {0}")]
    Semver(#[from] semver::Error),

    #[error("Schema immutability violation: cannot modify existing schema {name} v{version}")]
    ImmutabilityViolation { name: String, version: String },

    #[error("Compatibility check failed: {0}")]
    IncompatibleChange(String),
}







