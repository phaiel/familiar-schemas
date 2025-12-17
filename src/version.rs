//! Schema versioning utilities

use semver::Version;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;

/// A complete schema version with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Semantic version (e.g., "1.2.3")
    pub version: Version,
    /// Git commit hash for this version
    pub commit_hash: Option<String>,
    /// Git tag name (e.g., "v1.2.3")
    pub tag: Option<String>,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// Who created this version
    pub created_by: Option<String>,
    /// Release notes or changelog
    pub notes: Option<String>,
    /// Previous version (for compatibility tracking)
    pub previous_version: Option<String>,
}

impl SchemaVersion {
    /// Create a new schema version
    pub fn new(version: Version) -> Self {
        Self {
            version,
            commit_hash: None,
            tag: None,
            created_at: Utc::now(),
            created_by: None,
            notes: None,
            previous_version: None,
        }
    }

    /// Create from a version string
    pub fn parse(version_str: &str) -> Result<Self, semver::Error> {
        // Strip leading 'v' if present
        let version_str = version_str.strip_prefix('v').unwrap_or(version_str);
        let version = Version::parse(version_str)?;
        Ok(Self::new(version))
    }

    /// Get the version string (e.g., "1.2.3")
    pub fn version_string(&self) -> String {
        self.version.to_string()
    }

    /// Get the tag string (e.g., "v1.2.3")
    pub fn tag_string(&self) -> String {
        format!("v{}", self.version)
    }

    /// Get the directory name for this version
    pub fn dir_name(&self) -> String {
        self.tag_string()
    }

    /// Check if this is a major version bump from another version
    pub fn is_major_bump_from(&self, other: &SchemaVersion) -> bool {
        self.version.major > other.version.major
    }

    /// Check if this is a minor version bump from another version
    pub fn is_minor_bump_from(&self, other: &SchemaVersion) -> bool {
        self.version.major == other.version.major
            && self.version.minor > other.version.minor
    }

    /// Check if this is a patch version bump from another version
    pub fn is_patch_bump_from(&self, other: &SchemaVersion) -> bool {
        self.version.major == other.version.major
            && self.version.minor == other.version.minor
            && self.version.patch > other.version.patch
    }

    /// Bump major version
    pub fn bump_major(&self) -> Self {
        let mut new_version = self.clone();
        new_version.version = Version::new(
            self.version.major + 1,
            0,
            0,
        );
        new_version.previous_version = Some(self.version_string());
        new_version.created_at = Utc::now();
        new_version.commit_hash = None;
        new_version.tag = None;
        new_version
    }

    /// Bump minor version
    pub fn bump_minor(&self) -> Self {
        let mut new_version = self.clone();
        new_version.version = Version::new(
            self.version.major,
            self.version.minor + 1,
            0,
        );
        new_version.previous_version = Some(self.version_string());
        new_version.created_at = Utc::now();
        new_version.commit_hash = None;
        new_version.tag = None;
        new_version
    }

    /// Bump patch version
    pub fn bump_patch(&self) -> Self {
        let mut new_version = self.clone();
        new_version.version = Version::new(
            self.version.major,
            self.version.minor,
            self.version.patch + 1,
        );
        new_version.previous_version = Some(self.version_string());
        new_version.created_at = Utc::now();
        new_version.commit_hash = None;
        new_version.tag = None;
        new_version
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.version)
    }
}

impl PartialEq for SchemaVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl Eq for SchemaVersion {}

impl PartialOrd for SchemaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchemaVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = SchemaVersion::parse("1.2.3").unwrap();
        assert_eq!(v.version_string(), "1.2.3");
        assert_eq!(v.tag_string(), "v1.2.3");
    }

    #[test]
    fn test_version_with_v_prefix() {
        let v = SchemaVersion::parse("v1.2.3").unwrap();
        assert_eq!(v.version_string(), "1.2.3");
    }

    #[test]
    fn test_version_bumps() {
        let v = SchemaVersion::parse("1.2.3").unwrap();
        
        let major = v.bump_major();
        assert_eq!(major.version_string(), "2.0.0");
        
        let minor = v.bump_minor();
        assert_eq!(minor.version_string(), "1.3.0");
        
        let patch = v.bump_patch();
        assert_eq!(patch.version_string(), "1.2.4");
    }
}

