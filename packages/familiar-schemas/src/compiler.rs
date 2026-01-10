//! Schema Architecture Validator & Compiler Configuration
//!
//! Enforces the strict consolidation policy at build time to prevent
//! unauthorized x-familiar-* extensions from being committed.
//!
//! ## Compiler Configuration
//! Platform-wide settings that control code generation behavior.
//! These settings are NOT configurable per-schema - they're global platform decisions.
//!
//! ## Future: Directory Structure Alignment
//! TODO: Restructure directories to directly align with graph regions per TVM BYOC:
//! - `infrastructure/` → CPU/GPU regions
//! - `entities/` → Memory regions
//! - `actions/` → Operator regions
//! - `techniques/` → Composite regions
//! - `primitives/` → Atomic regions
//!
//! This will create a direct mapping between filesystem structure and
//! compilation graph topology, enabling better optimization and caching.

use thiserror::Error;

/// Metaschema validation for directory-specific schema rules
#[cfg(feature = "metaschema-validation")]
use jsonschema::{Draft, JSONSchema};
#[cfg(feature = "metaschema-validation")]
use miette::{Diagnostic, NamedSource, SourceSpan};

/// Platform-wide compiler configuration that controls code generation behavior.
/// These settings are NOT per-schema - they're global platform decisions.
///
/// Schemas define WHAT the data structure is.
/// CompilerConfig defines HOW it's represented in generated code.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Default field name to use for internal tagging in discriminated unions
    pub discriminator_field: String,

    /// Default casing strategy for enum variants (PascalCase, camelCase, snake_case, etc.)
    pub variant_casing: Casing,

    /// Whether to use `serde(rename_all = "...")` by default
    pub default_rename_all: Option<String>,

    /// Whether to skip serializing None values by default
    pub skip_none_by_default: bool,

    /// Whether to flatten nested structures by default (discouraged)
    pub flatten_by_default: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Casing {
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            discriminator_field: "kind".to_string(),
            variant_casing: Casing::PascalCase,
            default_rename_all: Some("snake_case".to_string()),
            skip_none_by_default: true,
            flatten_by_default: false,
        }
    }
}

#[derive(Error, Debug)]
pub enum SchemaArchitectureError {
    #[error("❌ Schema Architecture Violation\n❌ UNAUTHORIZED EXTENSION: '{extension}'\n📋 ALLOWED EXTENSIONS (9 total):\n• Structural: x-familiar-kind, x-familiar-service, x-familiar-queue, x-familiar-resources\n• Objects: x-familiar-persistence, x-familiar-serde, x-familiar-api, x-familiar-policy, x-familiar-visual\n• Legacy: x-familiar-description, x-familiar-meta-schema\n🔧 FIX: Consolidate '{extension}' into appropriate object or remove if redundant.\n📍 Location: {path}")]
    UnauthorizedExtension {
        extension: String,
        path: String,
    },

    #[error("❌ Schema Architecture Violation\n❌ REDUNDANT FIELD: '{field}'\n📋 This field is calculated automatically by the Graph Compiler.\n🔧 FIX: Remove this field - it will be computed at build time.\n📍 Location: {path}")]
    RedundantField {
        field: String,
        path: String,
    },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_config_defaults() {
        let config = CompilerConfig::default();

        assert_eq!(config.discriminator_field, "kind");
        assert_eq!(config.variant_casing, Casing::PascalCase);
        assert_eq!(config.default_rename_all, Some("snake_case".to_string()));
        assert!(config.skip_none_by_default);
        assert!(!config.flatten_by_default);
    }
}
