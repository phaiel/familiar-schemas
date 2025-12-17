//! Schema types and structures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::checksum::Checksum;
use crate::version::SchemaVersion;

/// Type of schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaType {
    /// Rust entity types (from familiar-core/src/entities)
    RustEntity,
    /// Rust primitive types (from familiar-primitives)
    RustPrimitive,
    /// Rust component types (from familiar-core/src/components)
    RustComponent,
    /// Rust type definitions (from familiar-core/src/types)
    RustType,
    /// AVRO schemas for Kafka/Redpanda
    Avro,
    /// TypeScript types (generated)
    TypeScript,
    /// Python/Pydantic models (generated)
    Python,
    /// JSON Schema definitions
    JsonSchema,
    /// OpenAPI specifications
    OpenApi,
}

impl SchemaType {
    /// Get the directory name for this schema type
    pub fn dir_name(&self) -> &'static str {
        match self {
            SchemaType::RustEntity => "rust/entities",
            SchemaType::RustPrimitive => "rust/primitives",
            SchemaType::RustComponent => "rust/components",
            SchemaType::RustType => "rust/types",
            SchemaType::Avro => "avro",
            SchemaType::TypeScript => "typescript",
            SchemaType::Python => "python",
            SchemaType::JsonSchema => "json-schema",
            SchemaType::OpenApi => "openapi",
        }
    }

    /// Get the file extension for this schema type
    pub fn extension(&self) -> &'static str {
        match self {
            SchemaType::RustEntity | SchemaType::RustPrimitive 
            | SchemaType::RustComponent | SchemaType::RustType => "json",
            SchemaType::Avro => "avsc",
            SchemaType::TypeScript => "ts",
            SchemaType::Python => "py",
            SchemaType::JsonSchema => "schema.json",
            SchemaType::OpenApi => "yaml",
        }
    }
}

/// A single schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Unique name of the schema (e.g., "User", "CommandEnvelope")
    pub name: String,
    /// Type of schema
    pub schema_type: SchemaType,
    /// The actual schema content (as JSON)
    pub content: serde_json::Value,
    /// Original source file path (if applicable)
    pub source_path: Option<String>,
    /// Category for organization (e.g., "auth", "primitives", "agentic")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

impl Schema {
    /// Create a new schema
    pub fn new(name: impl Into<String>, schema_type: SchemaType, content: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            schema_type,
            content,
            source_path: None,
            category: None,
        }
    }

    /// Create a new schema with category
    pub fn with_category(name: impl Into<String>, schema_type: SchemaType, content: serde_json::Value, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schema_type,
            content,
            source_path: None,
            category: Some(category.into()),
        }
    }

    /// Set the category
    pub fn set_category(&mut self, category: impl Into<String>) {
        self.category = Some(category.into());
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
    pub rust_entities: usize,
    pub rust_primitives: usize,
    pub rust_components: usize,
    pub rust_types: usize,
    pub avro_schemas: usize,
    pub typescript_schemas: usize,
    pub python_schemas: usize,
}

impl VersionManifest {
    /// Create a new manifest from schemas
    pub fn new(version: SchemaVersion, schemas: Vec<SchemaEntry>) -> Self {
        let stats = ManifestStats {
            total_schemas: schemas.len(),
            rust_entities: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::RustEntity).count(),
            rust_primitives: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::RustPrimitive).count(),
            rust_components: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::RustComponent).count(),
            rust_types: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::RustType).count(),
            avro_schemas: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::Avro).count(),
            typescript_schemas: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::TypeScript).count(),
            python_schemas: schemas.iter().filter(|s| s.schema.schema_type == SchemaType::Python).count(),
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
}
