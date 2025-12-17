//! Schema compatibility checking
//!
//! Validates that schema changes are backward compatible and detects breaking changes.

use serde::{Deserialize, Serialize};
use similar::{TextDiff, ChangeTag};
use crate::schema::{SchemaEntry, SchemaType};
use crate::error::{Result, SchemaError};

/// Result of a compatibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    /// Whether the schemas are compatible
    pub is_compatible: bool,
    /// Whether this is a breaking change
    pub is_breaking: bool,
    /// List of changes detected
    pub changes: Vec<SchemaChange>,
    /// Summary of the compatibility check
    pub summary: String,
}

impl CompatibilityResult {
    /// Create a compatible result
    pub fn compatible(changes: Vec<SchemaChange>) -> Self {
        let summary = if changes.is_empty() {
            "No changes detected".to_string()
        } else {
            format!("{} compatible changes detected", changes.len())
        };
        Self {
            is_compatible: true,
            is_breaking: false,
            changes,
            summary,
        }
    }

    /// Create an incompatible result
    pub fn incompatible(changes: Vec<SchemaChange>, reason: impl Into<String>) -> Self {
        Self {
            is_compatible: false,
            is_breaking: true,
            changes,
            summary: reason.into(),
        }
    }
}

/// A detected change between schema versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Path to the changed element (e.g., "properties.name.type")
    pub path: String,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
    /// Whether this change is breaking
    pub is_breaking: bool,
    /// Human-readable description
    pub description: String,
}

/// Type of schema change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// A new field was added
    FieldAdded,
    /// A field was removed
    FieldRemoved,
    /// A field's type changed
    TypeChanged,
    /// A field was renamed
    FieldRenamed,
    /// A field's optionality changed (required <-> optional)
    OptionalityChanged,
    /// Default value changed
    DefaultChanged,
    /// Documentation/description changed
    DocumentationChanged,
    /// Enum variant added
    EnumVariantAdded,
    /// Enum variant removed
    EnumVariantRemoved,
    /// Schema format changed
    FormatChanged,
    /// Other change
    Other,
}

impl ChangeType {
    /// Check if this change type is typically breaking
    pub fn is_typically_breaking(&self) -> bool {
        matches!(
            self,
            ChangeType::FieldRemoved
                | ChangeType::TypeChanged
                | ChangeType::EnumVariantRemoved
                | ChangeType::OptionalityChanged
        )
    }
}

/// Compatibility checker for schema versions
pub struct CompatibilityChecker {
    /// Strict mode - any change is considered breaking
    strict_mode: bool,
}

impl CompatibilityChecker {
    /// Create a new compatibility checker
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Enable strict mode
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Check compatibility between two schema entries
    pub fn check(&self, old: &SchemaEntry, new: &SchemaEntry) -> Result<CompatibilityResult> {
        // Must be same schema name
        if old.schema.name != new.schema.name {
            return Err(SchemaError::IncompatibleChange(
                "Cannot compare schemas with different names".to_string(),
            ));
        }

        // Must be same schema type
        if old.schema.schema_type != new.schema.schema_type {
            return Err(SchemaError::IncompatibleChange(
                "Cannot compare schemas with different types".to_string(),
            ));
        }

        let changes = self.detect_changes(old, new)?;
        let breaking_count = changes.iter().filter(|c| c.is_breaking).count();

        if self.strict_mode && !changes.is_empty() {
            Ok(CompatibilityResult::incompatible(
                changes,
                format!("Strict mode: {} changes detected", breaking_count),
            ))
        } else if breaking_count > 0 {
            Ok(CompatibilityResult::incompatible(
                changes,
                format!("{} breaking changes detected", breaking_count),
            ))
        } else {
            Ok(CompatibilityResult::compatible(changes))
        }
    }

    /// Detect changes between two schema entries
    fn detect_changes(&self, old: &SchemaEntry, new: &SchemaEntry) -> Result<Vec<SchemaChange>> {
        let mut changes = Vec::new();

        match old.schema.schema_type {
            SchemaType::Avro => {
                self.detect_avro_changes(&old.schema.content, &new.schema.content, &mut changes)?;
            }
            SchemaType::JsonSchema => {
                self.detect_json_schema_changes(&old.schema.content, &new.schema.content, "", &mut changes)?;
            }
            _ => {
                // For other types, do a simple text diff
                self.detect_text_changes(old, new, &mut changes)?;
            }
        }

        Ok(changes)
    }

    /// Detect changes in AVRO schemas
    fn detect_avro_changes(
        &self,
        old: &serde_json::Value,
        new: &serde_json::Value,
        changes: &mut Vec<SchemaChange>,
    ) -> Result<()> {
        // Check for field changes in AVRO record schemas
        if let (Some(old_fields), Some(new_fields)) = (
            old.get("fields").and_then(|f| f.as_array()),
            new.get("fields").and_then(|f| f.as_array()),
        ) {
            // Build maps of field names to fields
            let old_field_map: std::collections::HashMap<_, _> = old_fields
                .iter()
                .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(|n| (n, f)))
                .collect();

            let new_field_map: std::collections::HashMap<_, _> = new_fields
                .iter()
                .filter_map(|f| f.get("name").and_then(|n| n.as_str()).map(|n| (n, f)))
                .collect();

            // Check for removed fields
            for (name, old_field) in &old_field_map {
                if !new_field_map.contains_key(name) {
                    changes.push(SchemaChange {
                        change_type: ChangeType::FieldRemoved,
                        path: format!("fields.{}", name),
                        old_value: Some(old_field.to_string()),
                        new_value: None,
                        is_breaking: true,
                        description: format!("Field '{}' was removed", name),
                    });
                }
            }

            // Check for added fields
            for (name, new_field) in &new_field_map {
                if !old_field_map.contains_key(name) {
                    // Check if new field has a default - if not, it's breaking
                    let has_default = new_field.get("default").is_some();
                    changes.push(SchemaChange {
                        change_type: ChangeType::FieldAdded,
                        path: format!("fields.{}", name),
                        old_value: None,
                        new_value: Some(new_field.to_string()),
                        is_breaking: !has_default,
                        description: if has_default {
                            format!("Field '{}' was added with default value", name)
                        } else {
                            format!("Field '{}' was added without default (breaking)", name)
                        },
                    });
                }
            }

            // Check for type changes in existing fields
            for (name, old_field) in &old_field_map {
                if let Some(new_field) = new_field_map.get(name) {
                    let old_type = old_field.get("type");
                    let new_type = new_field.get("type");
                    if old_type != new_type {
                        changes.push(SchemaChange {
                            change_type: ChangeType::TypeChanged,
                            path: format!("fields.{}.type", name),
                            old_value: old_type.map(|t| t.to_string()),
                            new_value: new_type.map(|t| t.to_string()),
                            is_breaking: true,
                            description: format!("Field '{}' type changed", name),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect changes in JSON Schema
    fn detect_json_schema_changes(
        &self,
        old: &serde_json::Value,
        new: &serde_json::Value,
        path: &str,
        changes: &mut Vec<SchemaChange>,
    ) -> Result<()> {
        // Check properties in JSON Schema
        if let (Some(old_props), Some(new_props)) = (
            old.get("properties").and_then(|p| p.as_object()),
            new.get("properties").and_then(|p| p.as_object()),
        ) {
            // Check for removed properties
            for (name, old_prop) in old_props {
                let prop_path = if path.is_empty() {
                    format!("properties.{}", name)
                } else {
                    format!("{}.properties.{}", path, name)
                };

                if !new_props.contains_key(name) {
                    changes.push(SchemaChange {
                        change_type: ChangeType::FieldRemoved,
                        path: prop_path,
                        old_value: Some(old_prop.to_string()),
                        new_value: None,
                        is_breaking: true,
                        description: format!("Property '{}' was removed", name),
                    });
                }
            }

            // Check for added properties
            for (name, new_prop) in new_props {
                let prop_path = if path.is_empty() {
                    format!("properties.{}", name)
                } else {
                    format!("{}.properties.{}", path, name)
                };

                if !old_props.contains_key(name) {
                    // Check if required
                    let old_required = old.get("required")
                        .and_then(|r| r.as_array())
                        .map(|r| r.iter().any(|v| v.as_str() == Some(name)))
                        .unwrap_or(false);
                    let new_required = new.get("required")
                        .and_then(|r| r.as_array())
                        .map(|r| r.iter().any(|v| v.as_str() == Some(name)))
                        .unwrap_or(false);

                    changes.push(SchemaChange {
                        change_type: ChangeType::FieldAdded,
                        path: prop_path,
                        old_value: None,
                        new_value: Some(new_prop.to_string()),
                        is_breaking: new_required && !old_required,
                        description: if new_required {
                            format!("Required property '{}' was added (breaking)", name)
                        } else {
                            format!("Optional property '{}' was added", name)
                        },
                    });
                }
            }

            // Check for type changes
            for (name, old_prop) in old_props {
                if let Some(new_prop) = new_props.get(name) {
                    let prop_path = if path.is_empty() {
                        format!("properties.{}", name)
                    } else {
                        format!("{}.properties.{}", path, name)
                    };

                    let old_type = old_prop.get("type");
                    let new_type = new_prop.get("type");
                    if old_type != new_type {
                        changes.push(SchemaChange {
                            change_type: ChangeType::TypeChanged,
                            path: format!("{}.type", prop_path),
                            old_value: old_type.map(|t| t.to_string()),
                            new_value: new_type.map(|t| t.to_string()),
                            is_breaking: true,
                            description: format!("Property '{}' type changed", name),
                        });
                    }

                    // Recurse into nested objects
                    if old_prop.get("type").and_then(|t| t.as_str()) == Some("object") {
                        self.detect_json_schema_changes(old_prop, new_prop, &prop_path, changes)?;
                    }
                }
            }
        }

        // Check enum changes
        if let (Some(old_enum), Some(new_enum)) = (
            old.get("enum").and_then(|e| e.as_array()),
            new.get("enum").and_then(|e| e.as_array()),
        ) {
            let old_set: std::collections::HashSet<_> = old_enum.iter().collect();
            let new_set: std::collections::HashSet<_> = new_enum.iter().collect();

            // Removed variants
            for removed in old_set.difference(&new_set) {
                changes.push(SchemaChange {
                    change_type: ChangeType::EnumVariantRemoved,
                    path: format!("{}.enum", path),
                    old_value: Some(removed.to_string()),
                    new_value: None,
                    is_breaking: true,
                    description: format!("Enum variant {} was removed", removed),
                });
            }

            // Added variants
            for added in new_set.difference(&old_set) {
                changes.push(SchemaChange {
                    change_type: ChangeType::EnumVariantAdded,
                    path: format!("{}.enum", path),
                    old_value: None,
                    new_value: Some(added.to_string()),
                    is_breaking: false,
                    description: format!("Enum variant {} was added", added),
                });
            }
        }

        Ok(())
    }

    /// Detect text-based changes (fallback)
    fn detect_text_changes(
        &self,
        old: &SchemaEntry,
        new: &SchemaEntry,
        changes: &mut Vec<SchemaChange>,
    ) -> Result<()> {
        let old_text = serde_json::to_string_pretty(&old.schema.content)?;
        let new_text = serde_json::to_string_pretty(&new.schema.content)?;

        let diff = TextDiff::from_lines(&old_text, &new_text);
        
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    changes.push(SchemaChange {
                        change_type: ChangeType::Other,
                        path: "content".to_string(),
                        old_value: Some(change.value().to_string()),
                        new_value: None,
                        is_breaking: true,
                        description: "Line removed".to_string(),
                    });
                }
                ChangeTag::Insert => {
                    changes.push(SchemaChange {
                        change_type: ChangeType::Other,
                        path: "content".to_string(),
                        old_value: None,
                        new_value: Some(change.value().to_string()),
                        is_breaking: false,
                        description: "Line added".to_string(),
                    });
                }
                ChangeTag::Equal => {}
            }
        }

        Ok(())
    }
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Schema, SchemaType};
    use crate::version::SchemaVersion;

    #[test]
    fn test_compatible_field_addition() {
        let old_content = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });

        let new_content = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            }
        });

        let old = SchemaEntry::new(
            Schema::new("User", SchemaType::JsonSchema, old_content),
            SchemaVersion::parse("1.0.0").unwrap(),
        );

        let new = SchemaEntry::new(
            Schema::new("User", SchemaType::JsonSchema, new_content),
            SchemaVersion::parse("1.1.0").unwrap(),
        );

        let checker = CompatibilityChecker::new();
        let result = checker.check(&old, &new).unwrap();

        assert!(result.is_compatible);
        assert!(!result.is_breaking);
    }

    #[test]
    fn test_breaking_field_removal() {
        let old_content = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            }
        });

        let new_content = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });

        let old = SchemaEntry::new(
            Schema::new("User", SchemaType::JsonSchema, old_content),
            SchemaVersion::parse("1.0.0").unwrap(),
        );

        let new = SchemaEntry::new(
            Schema::new("User", SchemaType::JsonSchema, new_content),
            SchemaVersion::parse("2.0.0").unwrap(),
        );

        let checker = CompatibilityChecker::new();
        let result = checker.check(&old, &new).unwrap();

        assert!(!result.is_compatible);
        assert!(result.is_breaking);
    }
}

