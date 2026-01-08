//! Nickel Validation Module
//!
//! This module provides validation-only Nickel integration for the familiar-schemas crate.
//! Nickel serves as an enhanced validation engine, checking architectural compliance and
//! structural integrity of schemas.
//!
//! Key Principles:
//! - Validation ONLY - no code generation, no behavioral enhancement
//! - Pure function: "Is this schema structurally/architecturally valid?"
//! - Clean separation from other Nickel capabilities in specialized crates

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Nickel-based validator for architectural and structural compliance
pub struct NickelValidator {
    nickel_available: bool,
    nickel_runtime: Option<NickelRuntime>,
}

/// Internal Nickel runtime for validation execution
struct NickelRuntime {
    workspace_root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Nickel execution failed: {message}")]
    NickelExecution { message: String },

    #[error("Schema validation failed: {details}")]
    SchemaInvalid { details: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl NickelRuntime {
    /// Find the appropriate Nickel validation configuration for a schema path
    fn find_nickel_config(&self, _schema_path: &Path) -> Result<PathBuf, ValidationError> {
        // For now, always use the global validation configuration
        // TODO: Implement directory-specific validation configs
        let global_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../familiar-schemas/versions/latest/nickel/validation.ncl");

        if global_path.exists() {
            Ok(global_path)
        } else {
            Err(ValidationError::ConfigError {
                message: format!("No Nickel validation configuration found at {}", global_path.display()),
            })
        }
    }

    /// Execute Nickel validation for a schema
    fn execute_nickel_validation(&self, nickel_config: &Path, schema: &serde_json::Value, _schema_path: &Path) -> Result<ValidationResult, ValidationError> {
        // Read the base validation script
        let base_script = std::fs::read_to_string(nickel_config)
            .map_err(|e| ValidationError::ConfigError {
                message: format!("Failed to read validation config: {}", e),
            })?;

        // Create a JSON string representation of the schema for Nickel
        let schema_json = serde_json::to_string(schema)
            .map_err(|e| ValidationError::ConfigError {
                message: format!("Failed to serialize schema: {}", e),
            })?;

        // Inject the schema content into the Nickel script
        let nickel_script = base_script.replace(
            "let schema_content = \"\" in",
            &format!("let schema_content = {} in", serde_json::to_string(&schema_json).unwrap())
        );

        // Execute nickel export with the script
        let mut child = Command::new("nickel")
            .args(&["export", "--format", "json"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to spawn nickel: {}", e),
            })?;

        // Write the script to stdin
        if let Some(mut stdin) = child.stdin.take() {
            std::io::Write::write_all(&mut stdin, nickel_script.as_bytes())
                .map_err(|e| ValidationError::NickelExecution {
                    message: format!("Failed to write to nickel stdin: {}", e),
                })?;
        }

        let output = child.wait_with_output()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to get nickel output: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ValidationError::NickelExecution {
                message: format!("Nickel validation failed: {}", stderr),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: ValidationResult = serde_json::from_str(&stdout)
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to parse Nickel output: {}", e),
            })?;

        Ok(result)
    }
}

impl NickelValidator {
    /// Create a new Nickel validator
    pub fn new() -> Result<Self, ValidationError> {
        let workspace_root = Self::find_workspace_root()?;

        // Check if nickel is available
        let nickel_available = std::process::Command::new("nickel")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        let nickel_runtime = if nickel_available {
            Some(NickelRuntime { workspace_root })
        } else {
            None
        };

        Ok(Self {
            nickel_available,
            nickel_runtime,
        })
    }

    /// Validate a single schema file
    ///
    /// Returns Ok(()) if valid, ValidationError if invalid or execution failed
    pub fn validate_schema(&self, schema: &serde_json::Value, schema_path: &Path) -> Result<(), ValidationError> {
        if !self.nickel_available {
            // Fallback to basic validation when nickel is not available
            return self.fallback_validation(schema, schema_path);
        }

        let nickel_runtime = self.nickel_runtime.as_ref()
            .ok_or_else(|| ValidationError::ConfigError {
                message: "Nickel runtime not available".to_string(),
            })?;

        // Find the appropriate Nickel validation configuration
        let nickel_config = nickel_runtime.find_nickel_config(schema_path)?;

        // Execute validation
        let result = nickel_runtime.execute_nickel_validation(&nickel_config, schema, schema_path)?;

        // Check result
        if result.valid {
            Ok(())
        } else {
            Err(ValidationError::SchemaInvalid {
                details: result.errors.join("; "),
            })
        }
    }

    /// Validate multiple schemas (convenience method)
    pub fn validate_schemas(&self, schemas: &[(serde_json::Value, PathBuf)]) -> Result<(), ValidationError> {
        for (schema, path) in schemas {
            self.validate_schema(schema, path)?;
        }
        Ok(())
    }


    /// Fallback validation when nickel is not available
    ///
    /// Performs basic structural validation without the full Nickel rule engine
    fn fallback_validation(&self, schema: &serde_json::Value, _schema_path: &Path) -> Result<(), ValidationError> {
        let mut errors = Vec::new();

        // Check required fields
        if !schema.is_object() {
            errors.push("Schema must be an object".to_string());
        } else {
            let obj = schema.as_object().unwrap();

            // Check for required x-familiar-kind
            if !obj.contains_key("x-familiar-kind") {
                errors.push("Missing required extension: x-familiar-kind".to_string());
            }

            // Check for required title
            if !obj.contains_key("title") {
                errors.push("Missing required field: title".to_string());
            }

            // Check for required type
            if !obj.contains_key("type") {
                errors.push("Missing required field: type".to_string());
            }

            // Basic forbidden extensions check (subset of the full rules)
            let forbidden = [
                "x-familiar-producers",
                "x-familiar-consumers",
                "x-familiar-rust-impl-ids",
                "x-familiar-serde",
            ];

            for key in obj.keys() {
                if forbidden.contains(&key.as_str()) {
                    errors.push(format!("Forbidden extension found: {}", key));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::SchemaInvalid {
                details: format!(
                    "Fallback validation failed (nickel not available): {}",
                    errors.join("; ")
                ),
            })
        }
    }

    /// Find the workspace root by looking for Cargo.toml or similar markers
    fn find_workspace_root() -> Result<PathBuf, ValidationError> {
        // First try CARGO_MANIFEST_DIR approach (most reliable for build scripts)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let manifest_path = PathBuf::from(manifest_dir);
            // From familiar-contracts/build.rs, workspace root is ../../../../
            let workspace_candidate = manifest_path.join("../../../..");
            if workspace_candidate.join("Cargo.toml").exists() {
                return Ok(workspace_candidate);
            }
        }

        // Fallback: walk up from current directory
        let mut current = std::env::current_dir()?;

        loop {
            if current.join("Cargo.toml").exists() {
                // Check if this is a workspace root by looking for workspace members
                let cargo_content = std::fs::read_to_string(current.join("Cargo.toml"))
                    .unwrap_or_default();
                if cargo_content.contains("[workspace]") {
                    return Ok(current);
                }
            }

            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        Err(ValidationError::ConfigError {
            message: "Could not find workspace root".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_nickel_validator_creation() {
        let result = NickelValidator::new();
        // Note: This test may fail if nickel is not installed or workspace structure is different
        // In CI/testing, we might want to mock this
        match result {
            Ok(_) => println!("Validator created successfully"),
            Err(e) => println!("Validator creation failed (expected in some environments): {}", e),
        }
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::SchemaInvalid {
            details: "Invalid extension found".to_string(),
        };
        assert!(error.to_string().contains("Invalid extension found"));
    }
}
