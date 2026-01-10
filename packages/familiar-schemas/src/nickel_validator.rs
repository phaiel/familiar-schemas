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
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Nickel-based validator for architectural and structural compliance
pub struct NickelValidator {
    nickel_available: bool,
    nickel_runtime: Option<NickelRuntime>,
}

/// Internal Nickel runtime for validation execution
struct NickelRuntime;

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

    /// Find the edge declaration contract for a schema path
    fn find_edge_contract_path(&self, _schema_path: &Path) -> Result<PathBuf, ValidationError> {
        let contract_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../familiar-schemas/versions/latest/nickel/extensions/edge-declaration-contract.ncl");

        if contract_path.exists() {
            Ok(contract_path)
        } else {
            Err(ValidationError::ConfigError {
                message: format!("Edge declaration contract not found at {}", contract_path.display()),
            })
        }
    }

    /// Execute edge declaration contract validation
    fn execute_contract_validation(&self, schema: &Value, contract_path: &Path) -> Result<ValidationResult, ValidationError> {
        // Create a Nickel script that imports the contract and validates the schema
        let nickel_script = format!(
            r#"
let contract = import "{}" in
let schema_to_validate = {} in

# Apply the contract
contract.contract schema_to_validate
"#,
            contract_path.display(),
            serde_json::to_string_pretty(schema).map_err(|e| ValidationError::ConfigError {
                message: format!("Failed to serialize schema: {}", e),
            })?
        );

        // Execute the script
        let mut output = std::process::Command::new("nickel")
            .args(&["export", "--format", "json"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to spawn nickel: {}", e),
            })?;

        // Write script to stdin
        if let Some(ref mut stdin) = output.stdin {
            use std::io::Write;
            stdin.write_all(nickel_script.as_bytes())
                .map_err(|e| ValidationError::NickelExecution {
                    message: format!("Failed to write to nickel stdin: {}", e),
                })?;
        }

        // Read result
        let output_result = output.wait_with_output()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to read nickel output: {}", e),
            })?;

        if output_result.status.success() {
            let result: ValidationResult = serde_json::from_slice(&output_result.stdout)
                .map_err(|e| ValidationError::NickelExecution {
                    message: format!("Failed to parse nickel result: {}", e),
                })?;
            Ok(result)
        } else {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            Err(ValidationError::NickelExecution {
                message: format!("Nickel contract validation failed: {}", stderr),
            })
        }
    }

    /// Execute edge extraction from schema
    fn execute_edge_extraction(&self, schema: &Value, contract_path: &Path) -> Result<Vec<Value>, ValidationError> {
        // Create a Nickel script that extracts edges
        let nickel_script = format!(
            r#"
let contract = import "{}" in
let schema_to_extract = {} in

# Extract edges using the contract
contract.functions.extract_edges schema_to_extract
"#,
            contract_path.display(),
            serde_json::to_string_pretty(schema).map_err(|e| ValidationError::ConfigError {
                message: format!("Failed to serialize schema: {}", e),
            })?
        );

        // Execute the script
        let mut output = std::process::Command::new("nickel")
            .args(&["export", "--format", "json"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to spawn nickel: {}", e),
            })?;

        // Write script to stdin
        if let Some(ref mut stdin) = output.stdin {
            use std::io::Write;
            stdin.write_all(nickel_script.as_bytes())
                .map_err(|e| ValidationError::NickelExecution {
                    message: format!("Failed to write to nickel stdin: {}", e),
                })?;
        }

        // Read result
        let output_result = output.wait_with_output()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to read nickel output: {}", e),
            })?;

        if output_result.status.success() {
            let edges: Vec<Value> = serde_json::from_slice(&output_result.stdout)
                .map_err(|e| ValidationError::NickelExecution {
                    message: format!("Failed to parse nickel edge extraction result: {}", e),
                })?;
            Ok(edges)
        } else {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            Err(ValidationError::NickelExecution {
                message: format!("Nickel edge extraction failed: {}", stderr),
            })
        }
    }

    /// Execute Nickel validation for a schema
    fn execute_nickel_validation(&self, nickel_config: &Path, schema: &Value, _schema_path: &Path) -> Result<ValidationResult, ValidationError> {
        // Create a JSON string representation of the schema for Nickel environment variable
        let schema_json = serde_json::to_string(schema)
            .map_err(|e| ValidationError::ConfigError {
                message: format!("Failed to serialize schema: {}", e),
            })?;

        // Execute nickel eval directly on the validation file with environment variables
        let child = Command::new("nickel")
            .args(&["eval", &nickel_config.to_string_lossy()])
            .env("SCHEMA_FILE", &schema_json)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ValidationError::NickelExecution {
                message: format!("Failed to spawn nickel: {}", e),
            })?;

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
        // Check if nickel is available
        let nickel_available = std::process::Command::new("nickel")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        let nickel_runtime = if nickel_available {
            Some(NickelRuntime)
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
    pub fn validate_schema(&self, schema: &Value, schema_path: &Path) -> Result<(), ValidationError> {
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
    pub fn validate_schemas(&self, schemas: &[(Value, PathBuf)]) -> Result<(), ValidationError> {
        for (schema, path) in schemas {
            self.validate_schema(schema, path)?;
        }
        Ok(())
    }

    /// Validate schema against edge declaration contract
    pub fn validate_edge_contract(&self, schema: &Value, schema_path: &Path) -> Result<(), ValidationError> {
        if !self.nickel_available {
            return Err(ValidationError::ConfigError {
                message: "Nickel not available for edge contract validation".to_string()
            });
        }

        let runtime = self.nickel_runtime.as_ref()
            .ok_or_else(|| ValidationError::ConfigError { message: "Nickel runtime not initialized".to_string() })?;

        // Find the edge declaration contract
        let contract_path = runtime.find_edge_contract_path(schema_path)?;

        // Execute Nickel contract validation
        let result = runtime.execute_contract_validation(schema, &contract_path)?;

        if result.valid {
            Ok(())
        } else {
            Err(ValidationError::SchemaInvalid {
                details: format!(
                    "Edge Declaration Contract violations: {}",
                    result.errors.join("; ")
                )
            })
        }
    }

    /// Extract typed edges from schema using edge contract
    pub fn extract_typed_edges(&self, schema: &Value, schema_path: &Path) -> Result<Vec<Value>, ValidationError> {
        if !self.nickel_available {
            return Err(ValidationError::ConfigError {
                message: "Nickel not available for edge extraction".to_string()
            });
        }

        let runtime = self.nickel_runtime.as_ref()
            .ok_or_else(|| ValidationError::ConfigError { message: "Nickel runtime not initialized".to_string() })?;

        // Find the edge declaration contract
        let contract_path = runtime.find_edge_contract_path(schema_path)?;

        // Execute Nickel edge extraction
        runtime.execute_edge_extraction(schema, &contract_path)
    }

    /// Fallback validation when nickel is not available
    ///
    /// Performs basic structural validation without the full Nickel rule engine
    fn fallback_validation(&self, schema: &Value, _schema_path: &Path) -> Result<(), ValidationError> {
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

}

#[cfg(test)]
mod tests {
    use super::*;

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
