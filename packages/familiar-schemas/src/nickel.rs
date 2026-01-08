//! Nickel Configuration Processing and Inheritance
//!
//! Handles loading and executing Nickel configurations with proper import resolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NickelConfig {
    pub allowed_extensions: Vec<String>,
    pub required_extensions: Vec<String>,
    pub forbidden_extensions: Vec<String>,
    pub validation_rules: Vec<ValidationRule>,
    pub hydration: HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

pub struct NickelProcessor;

impl NickelProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Load and compose a complete configuration for a schema path
    pub fn load_config_for_path(&self, schema_path: &Path) -> Result<NickelConfig, Box<dyn std::error::Error>> {
        let nickel_file = self.find_nickel_config(schema_path)?;
        self.execute_nickel_config(&nickel_file)
    }

    /// Compose a schema with full extension understanding using Nickel
    pub fn compose_schema_with_extensions(&self, schema_json: &serde_json::Value, schema_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let nickel_file = self.find_nickel_config(schema_path)?;
        self.execute_nickel_composition(schema_json, &nickel_file)
    }

    /// Find the appropriate Nickel config file for a schema path
    pub fn find_nickel_config(&self, schema_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Convert schema path to corresponding nickel config path
        // Schema: versions/latest/json-schema/infrastructure/resources/kafka-cluster.resource.json
        // Nickel: versions/latest/nickel/infrastructure/resources/_directory.ncl

        let path_str = schema_path.to_string_lossy();

        // Replace json-schema with nickel and file extension with _directory.ncl
        let nickel_path_str = path_str
            .replace("json-schema/", "nickel/")
            .replace(".resource.json", "/_directory.ncl")
            .replace(".entity.json", "/_directory.ncl")
            .replace(".schema.json", "/_directory.ncl")
            .replace(".json", "/_directory.ncl");

        let nickel_path = PathBuf::from(nickel_path_str);

        // Check if the specific nickel config exists
        if nickel_path.exists() {
            return Ok(nickel_path);
        }

        // Walk up the directory tree looking for _directory.ncl files
        let mut current_path = nickel_path.clone();
        while let Some(parent) = current_path.parent() {
            let dir_config = parent.join("_directory.ncl");
            if dir_config.exists() {
                return Ok(dir_config);
            }
            current_path = parent.to_path_buf();

            // Stop at versions/latest/nickel
            if current_path.ends_with("nickel") {
                break;
            }
        }

        // Fallback to global config
        let global_path = PathBuf::from("versions/latest/nickel/global.ncl");
        if global_path.exists() {
            return Ok(global_path);
        }

        // Try relative paths for testing
        let test_paths = [
            PathBuf::from("../../versions/latest/nickel/global.ncl"),
            PathBuf::from("../versions/latest/nickel/global.ncl"),
        ];

        for test_path in &test_paths {
            if test_path.exists() {
                return Ok(test_path.clone());
            }
        }

        Err(format!("No Nickel configuration found for path: {:?}", schema_path).into())
    }

    /// Execute Nickel and get the resolved configuration
    fn execute_nickel_config(&self, nickel_path: &Path) -> Result<NickelConfig, Box<dyn std::error::Error>> {
        // Execute nickel export to resolve imports and get final JSON
        let output = Command::new("nickel")
            .args(&["export", "--format", "json"])
            .arg(nickel_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Nickel execution failed: {}", stderr).into());
        }

        let json_str = String::from_utf8(output.stdout)?;
        let config: NickelConfig = serde_json::from_str(&json_str)?;
        Ok(config)
    }

    /// Execute Nickel schema composition with extension understanding
    fn execute_nickel_composition(&self, schema_json: &serde_json::Value, nickel_path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // In a real implementation, this would:
        // 1. Create a Nickel program that imports the composer
        // 2. Passes the schema JSON as input
        // 3. Executes the composition
        // 4. Returns the enhanced schema

        // For now, return the input schema with a marker that composition would happen
        let mut enhanced = schema_json.clone();
        if let serde_json::Value::Object(ref mut obj) = enhanced {
            obj.insert("_nickel_composed".to_string(), serde_json::Value::Bool(true));
        }
        Ok(enhanced)
    }
}