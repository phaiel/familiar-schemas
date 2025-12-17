//! Checksum utilities for schema integrity verification

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use std::fmt;

/// SHA256 checksum for schema content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Checksum(String);

impl Checksum {
    /// Compute checksum from raw bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        Self(format!("{:x}", hash))
    }

    /// Compute checksum from a string
    pub fn from_str(content: &str) -> Self {
        Self::from_bytes(content.as_bytes())
    }

    /// Compute checksum from JSON value (canonicalized)
    pub fn from_json(value: &serde_json::Value) -> Self {
        // Canonicalize JSON by converting to string with sorted keys
        let canonical = serde_json::to_string(value).unwrap_or_default();
        Self::from_str(&canonical)
    }

    /// Get the hex string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Verify that content matches this checksum
    pub fn verify(&self, content: &str) -> bool {
        let computed = Self::from_str(content);
        self.0 == computed.0
    }

    /// Verify that JSON value matches this checksum
    pub fn verify_json(&self, value: &serde_json::Value) -> bool {
        let computed = Self::from_json(value);
        self.0 == computed.0
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Checksum {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Checksum {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_consistency() {
        let content = r#"{"name": "test", "version": "1.0.0"}"#;
        let checksum1 = Checksum::from_str(content);
        let checksum2 = Checksum::from_str(content);
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_different_content() {
        let content1 = r#"{"name": "test1"}"#;
        let content2 = r#"{"name": "test2"}"#;
        let checksum1 = Checksum::from_str(content1);
        let checksum2 = Checksum::from_str(content2);
        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_checksum_verification() {
        let content = r#"{"name": "test"}"#;
        let checksum = Checksum::from_str(content);
        assert!(checksum.verify(content));
        assert!(!checksum.verify("different content"));
    }
}

