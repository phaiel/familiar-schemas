//! Schema types and structures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::checksum::Checksum;
use crate::version::SchemaVersion;

/// Type of schema - represents the FORMAT, not the source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaType {
    /// JSON Schema - the universal validation format
    JsonSchema,
    /// AVRO schemas for Kafka/Redpanda
    Avro,
    /// TypeScript type definitions (future)
    TypeScript,
    /// Python Pydantic models (future)
    Python,
    /// OpenAPI specifications (future)
    OpenApi,
}

impl SchemaType {
    /// Get the directory name for this schema type
    pub fn dir_name(&self) -> &'static str {
        match self {
            SchemaType::JsonSchema => "json-schema",
            SchemaType::Avro => "avro",
            SchemaType::TypeScript => "typescript",
            SchemaType::Python => "python",
            SchemaType::OpenApi => "openapi",
        }
    }

    /// Get the file extension for this schema type
    pub fn extension(&self) -> &'static str {
        match self {
            SchemaType::JsonSchema => "schema.json",
            SchemaType::Avro => "avsc",
            SchemaType::TypeScript => "d.ts",
            SchemaType::Python => "py",
            SchemaType::OpenApi => "yaml",
        }
    }
}

/// A single schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Unique name of the schema (e.g., "User", "CommandEnvelope")
    pub name: String,
    /// Type/format of schema (JsonSchema, Avro, etc.)
    pub schema_type: SchemaType,
    /// The actual schema content (as JSON)
    pub content: serde_json::Value,
    /// Category for organization (e.g., "auth", "primitives", "tools")
    pub category: String,
    /// Source crate (e.g., "familiar-primitives", "familiar-core", "familiar-contracts")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_crate: Option<String>,
    /// Original source file path (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

impl Schema {
    /// Create a new schema
    pub fn new(name: impl Into<String>, schema_type: SchemaType, content: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            schema_type,
            content,
            category: "types".to_string(), // default category
            source_crate: None,
            source_path: None,
        }
    }

    /// Create a new schema with category
    pub fn with_category(
        name: impl Into<String>,
        schema_type: SchemaType,
        content: serde_json::Value,
        category: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            schema_type,
            content,
            category: category.into(),
            source_crate: None,
            source_path: None,
        }
    }

    /// Set the category
    pub fn set_category(&mut self, category: impl Into<String>) {
        self.category = category.into();
    }

    /// Set the source crate
    pub fn set_source_crate(&mut self, crate_name: impl Into<String>) {
        self.source_crate = Some(crate_name.into());
    }

    /// Compute the checksum for this schema
    pub fn checksum(&self) -> Checksum {
        Checksum::from_json(&self.content)
    }

    /// Get the filename for this schema
    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, self.schema_type.extension())
    }
}

/// A versioned schema entry in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEntry {
    /// The schema definition
    pub schema: Schema,
    /// Version of this schema
    pub version: SchemaVersion,
    /// SHA256 checksum of the content
    pub checksum: Checksum,
    /// When this entry was created
    pub created_at: DateTime<Utc>,
    /// Who created this entry
    pub created_by: Option<String>,
    /// Versions this schema is compatible with
    pub compatible_with: Vec<String>,
    /// Whether this is a breaking change from the previous version
    pub breaking_change: bool,
    /// Change notes
    pub change_notes: Option<String>,
}

impl SchemaEntry {
    /// Create a new schema entry
    pub fn new(schema: Schema, version: SchemaVersion) -> Self {
        let checksum = schema.checksum();
        Self {
            schema,
            version,
            checksum,
            created_at: Utc::now(),
            created_by: None,
            compatible_with: Vec::new(),
            breaking_change: false,
            change_notes: None,
        }
    }

    /// Verify the checksum matches the content
    pub fn verify_checksum(&self) -> bool {
        let computed = self.schema.checksum();
        self.checksum == computed
    }

    /// Get a unique key for this entry
    pub fn key(&self) -> String {
        format!("{}/{}", self.schema.name, self.version)
    }
}

/// Manifest containing all schemas for a version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Version of this manifest
    pub version: SchemaVersion,
    /// All schemas in this version
    pub schemas: Vec<SchemaEntry>,
    /// When this manifest was created
    pub created_at: DateTime<Utc>,
    /// Total checksum of all schemas
    pub manifest_checksum: Checksum,
    /// Statistics
    pub stats: ManifestStats,
}

/// Statistics about a version manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestStats {
    pub total_schemas: usize,
    pub json_schemas: usize,
    pub avro_schemas: usize,
    pub typescript_schemas: usize,
    pub python_schemas: usize,
    /// Categories and their counts
    #[serde(default)]
    pub by_category: std::collections::HashMap<String, usize>,
    /// Source crates and their counts
    #[serde(default)]
    pub by_source_crate: std::collections::HashMap<String, usize>,
}

impl VersionManifest {
    /// Create a new manifest from schemas
    pub fn new(version: SchemaVersion, schemas: Vec<SchemaEntry>) -> Self {
        // Count by type
        let json_schemas = schemas.iter().filter(|s| s.schema.schema_type == SchemaType::JsonSchema).count();
        let avro_schemas = schemas.iter().filter(|s| s.schema.schema_type == SchemaType::Avro).count();
        let typescript_schemas = schemas.iter().filter(|s| s.schema.schema_type == SchemaType::TypeScript).count();
        let python_schemas = schemas.iter().filter(|s| s.schema.schema_type == SchemaType::Python).count();

        // Count by category
        let mut by_category: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for s in &schemas {
            *by_category.entry(s.schema.category.clone()).or_insert(0) += 1;
        }

        // Count by source crate
        let mut by_source_crate: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for s in &schemas {
            if let Some(ref crate_name) = s.schema.source_crate {
                *by_source_crate.entry(crate_name.clone()).or_insert(0) += 1;
            }
        }

        let stats = ManifestStats {
            total_schemas: schemas.len(),
            json_schemas,
            avro_schemas,
            typescript_schemas,
            python_schemas,
            by_category,
            by_source_crate,
        };

        // Compute manifest checksum from all schema checksums
        let checksums: Vec<String> = schemas.iter().map(|s| s.checksum.to_string()).collect();
        let combined = checksums.join(",");
        let manifest_checksum = Checksum::from_str(&combined);

        Self {
            version,
            schemas,
            created_at: Utc::now(),
            manifest_checksum,
            stats,
        }
    }

    /// Verify all schema checksums
    pub fn verify_all(&self) -> bool {
        self.schemas.iter().all(|s| s.verify_checksum())
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&SchemaEntry> {
        self.schemas.iter().find(|s| s.schema.name == name)
    }

    /// Get all schemas of a specific type
    pub fn get_schemas_by_type(&self, schema_type: SchemaType) -> Vec<&SchemaEntry> {
        self.schemas.iter().filter(|s| s.schema.schema_type == schema_type).collect()
    }

    /// Get all schemas in a category
    pub fn get_schemas_by_category(&self, category: &str) -> Vec<&SchemaEntry> {
        self.schemas.iter().filter(|s| s.schema.category == category).collect()
    }
}
