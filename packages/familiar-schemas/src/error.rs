//! Error types for the schema library

use thiserror::Error;

/// Result type for schema operations
pub type Result<T> = std::result::Result<T, SchemaError>;

/// Schema library errors
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

    #[error("Invalid schema format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Semver error: {0}")]
    Semver(#[from] semver::Error),
}








