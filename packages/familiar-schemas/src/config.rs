//! Configuration management for the Schema Registry
//!
//! Supports loading configuration from:
//! - Default values
//! - Config file (schemas.toml)
//! - Environment variables (SCHEMAS_*)
//!
//! ## Example config file (schemas.toml):
//! ```toml
//! [registry]
//! path = "./familiar-schemas"
//! default_author = "Eric Theiss"
//! immutable = true
//!
//! [export]
//! output_format = "pretty"
//! include_checksums = true
//!
//! [workspace]
//! root = "../docs/v4"
//! crates = ["familiar-primitives", "familiar-contracts", "familiar-core"]
//!
//! [categories]
//! primitives = "familiar-primitives"
//! contracts = "familiar-contracts"
//! core = "familiar-core"
//! ```

use config_crate::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for the schema registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    /// Registry settings
    #[serde(default)]
    pub registry: RegistryConfig,
    
    /// Export settings
    #[serde(default)]
    pub export: ExportConfig,
    
    /// Workspace settings
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    
    /// Category mappings
    #[serde(default)]
    pub categories: CategoryConfig,
    
    /// Validation settings
    #[serde(default)]
    pub validation: ValidationConfig,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Path to the schema registry root
    #[serde(default = "default_registry_path")]
    pub path: PathBuf,
    
    /// Default author for commits
    #[serde(default)]
    pub default_author: Option<String>,
    
    /// Whether the registry is immutable (append-only)
    #[serde(default = "default_true")]
    pub immutable: bool,
    
    /// Git remote URL (if syncing)
    #[serde(default)]
    pub git_remote: Option<String>,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Output format (pretty or compact)
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
    
    /// Include checksums file
    #[serde(default = "default_true")]
    pub include_checksums: bool,
    
    /// Include manifest file
    #[serde(default = "default_true")]
    pub include_manifest: bool,
    
    /// Create latest symlink
    #[serde(default = "default_true")]
    pub create_latest_symlink: bool,
}

/// Output format for JSON
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Pretty,
    Compact,
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Path to workspace root
    #[serde(default)]
    pub root: Option<PathBuf>,
    
    /// Crates to collect schemas from
    #[serde(default = "default_crates")]
    pub crates: Vec<CrateConfig>,
}

/// Configuration for a single crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateConfig {
    /// Crate name
    pub name: String,
    
    /// Path to generated schemas (relative to crate root)
    #[serde(default = "default_schemas_dir")]
    pub schemas_dir: String,
    
    /// Default category for schemas from this crate
    #[serde(default)]
    pub default_category: Option<String>,
}

/// Category mappings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryConfig {
    /// Map of category name to source crate/directory
    #[serde(flatten)]
    pub mappings: std::collections::HashMap<String, String>,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable strict JSON Schema validation
    #[serde(default = "default_true")]
    pub strict_json_schema: bool,
    
    /// Enable AVRO schema validation
    #[serde(default = "default_true")]
    pub validate_avro: bool,
    
    /// Fail on breaking changes
    #[serde(default)]
    pub fail_on_breaking: bool,
    
    /// Paths to ignore during drift detection
    #[serde(default)]
    pub ignore_paths: Vec<String>,
}

// Default value functions
fn default_registry_path() -> PathBuf {
    PathBuf::from(".")
}

fn default_true() -> bool {
    true
}

fn default_output_format() -> OutputFormat {
    OutputFormat::Pretty
}

fn default_crates() -> Vec<CrateConfig> {
    vec![
        CrateConfig {
            name: "familiar-primitives".to_string(),
            schemas_dir: "generated/schemas".to_string(),
            default_category: Some("primitives".to_string()),
        },
        CrateConfig {
            name: "familiar-contracts".to_string(),
            schemas_dir: "generated/schemas".to_string(),
            default_category: Some("contracts".to_string()),
        },
        CrateConfig {
            name: "familiar-core".to_string(),
            schemas_dir: "generated/schemas".to_string(),
            default_category: Some("core".to_string()),
        },
    ]
}

fn default_schemas_dir() -> String {
    "generated/schemas".to_string()
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            path: default_registry_path(),
            default_author: None,
            immutable: true,
            git_remote: None,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Pretty,
            include_checksums: true,
            include_manifest: true,
            create_latest_symlink: true,
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            root: None,
            crates: default_crates(),
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_json_schema: true,
            validate_avro: true,
            fail_on_breaking: false,
            ignore_paths: vec![
                "src/analysis/".to_string(),
                "src/bin/".to_string(),
                "tests/".to_string(),
            ],
        }
    }
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
            export: ExportConfig::default(),
            workspace: WorkspaceConfig::default(),
            categories: CategoryConfig::default(),
            validation: ValidationConfig::default(),
        }
    }
}

impl SchemaConfig {
    /// Load configuration from default locations
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(None)
    }
    
    /// Load configuration from a specific file
    pub fn load_from(config_path: Option<&str>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        
        // Load from default locations
        let config_locations = [
            "schemas.toml",
            ".schemas.toml",
            "config/schemas.toml",
        ];
        
        for location in config_locations {
            builder = builder.add_source(File::with_name(location).required(false));
        }
        
        // Load from XDG config directory
        if let Some(config_dir) = directories::ProjectDirs::from("dev", "familiar", "schemas") {
            let xdg_config = config_dir.config_dir().join("schemas.toml");
            if xdg_config.exists() {
                builder = builder.add_source(
                    File::from(xdg_config).required(false)
                );
            }
        }
        
        // Load from specified path
        if let Some(path) = config_path {
            builder = builder.add_source(File::with_name(path).required(true));
        }
        
        // Load from environment variables (SCHEMAS_*)
        builder = builder.add_source(
            Environment::with_prefix("SCHEMAS")
                .separator("__")
                .try_parsing(true)
        );
        
        let config = builder.build()?;
        config.try_deserialize()
    }
    
    /// Save configuration to a file
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }
    
    /// Get the registry path (resolves relative paths)
    pub fn registry_path(&self) -> PathBuf {
        if self.registry.path.is_absolute() {
            self.registry.path.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(&self.registry.path)
        }
    }
    
    /// Get the workspace root path
    pub fn workspace_root(&self) -> Option<PathBuf> {
        self.workspace.root.as_ref().map(|p| {
            if p.is_absolute() {
                p.clone()
            } else {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(p)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = SchemaConfig::default();
        assert!(config.registry.immutable);
        assert_eq!(config.workspace.crates.len(), 3);
    }
    
    #[test]
    fn test_serialize_config() {
        let config = SchemaConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[registry]"));
        assert!(toml_str.contains("[export]"));
    }
}
