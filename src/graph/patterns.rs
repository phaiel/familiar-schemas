//! Schema Shape Detection
//!
//! Detects structural patterns from raw JSON schemas. This is pure shape detection -
//! NO codegen decisions happen here. Classification uses these shapes to determine
//! TypeKind and EmitStrategy.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{SchemaGraph, SchemaId};

// =============================================================================
// JSON Scalar Kinds (language-agnostic)
// =============================================================================

/// JSON scalar type (before language-specific lowering)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JsonScalarKind {
    String,
    Integer,
    Number,
    Boolean,
    Null,
}

impl JsonScalarKind {
    pub fn from_json_type(type_str: &str) -> Option<Self> {
        match type_str {
            "string" => Some(Self::String),
            "integer" => Some(Self::Integer),
            "number" => Some(Self::Number),
            "boolean" => Some(Self::Boolean),
            "null" => Some(Self::Null),
            _ => None,
        }
    }
}

// =============================================================================
// Property Shape
// =============================================================================

/// Shape of a single property in an object schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyShape {
    /// Property name
    pub name: String,
    /// Is this property required?
    pub required: bool,
    /// Property type shape
    pub shape: PropertyTypeShape,
}

/// How a property's type is defined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyTypeShape {
    /// References another schema via $ref
    Ref(String),
    /// Inline scalar type
    Scalar(JsonScalarKind),
    /// Inline array with items (dynamic size)
    Array { items: Box<PropertyTypeShape> },
    /// Fixed-size array (minItems == maxItems)
    FixedArray { items: Box<PropertyTypeShape>, size: usize },
    /// Tuple array (items is an array, not object - each position has specific type)
    Tuple { items: Vec<PropertyTypeShape> },
    /// Inline object (anonymous)
    InlineObject,
    /// No type specified
    Unknown,
}

// =============================================================================
// Object Variant (for oneOf)
// =============================================================================

/// A variant in a oneOf with object type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectVariant {
    /// Variant name (from title or generated)
    pub name: Option<String>,
    /// Properties in this variant
    pub properties: Vec<String>,
    /// If this is a $ref, the target
    pub ref_target: Option<String>,
}

// =============================================================================
// Schema Shape
// =============================================================================

/// Codegen extension metadata extracted from x-familiar-* properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodegenExtensions {
    /// Variant name mapping from x-familiar-variants (JSON value -> Rust name)
    pub variants: Option<std::collections::HashMap<String, String>>,
    /// Casing convention from x-familiar-casing
    pub casing: Option<String>,
    /// Whether to generate Default derive (if x-familiar-rust-impl-ids exists)
    pub generate_default: bool,
    /// Impl block IDs from x-familiar-rust-impl-ids
    pub rust_impl_ids: Vec<String>,
    /// Field alias from x-familiar-field-alias (for composition)
    pub field_alias: Option<String>,
}

/// Raw schema shape detected from JSON structure.
/// 
/// This is pure pattern detection - no codegen decisions.
/// Classification (classify.rs) consumes these to determine TypeKind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaShape {
    /// `{"type": "string", "enum": [...]}` - top-level string enum
    StringEnum {
        values: Vec<String>,
        /// Codegen extensions from schema
        extensions: CodegenExtensions,
    },
    
    /// `{"oneOf": [...]}` with all const/enum string variants
    OneOfStringEnum {
        variants: Vec<String>,
        /// Codegen extensions from schema
        extensions: CodegenExtensions,
    },
    
    /// `{"oneOf": [...]}` with object variants (tagged union)
    OneOfObjects {
        variants: Vec<ObjectVariant>,
        /// Discriminator field if specified via x-familiar-discriminator
        discriminator: Option<String>,
    },
    
    /// `{"oneOf": [...]}` with mixed types (needs diagnostic)
    OneOfMixed {
        description: String,
    },
    
    /// `{"type": "string"}` with optional format but no enum
    FormattedString {
        format: Option<String>,
    },
    
    /// `{"type": "integer"}` or `{"type": "number"}`  
    Numeric {
        json_type: JsonScalarKind,
        format: Option<String>,
    },
    
    /// `{"type": "boolean"}`
    Boolean,
    
    /// Has "properties" object - a struct
    Object {
        properties: Vec<PropertyShape>,
        /// Additional properties type if specified
        additional_properties: Option<Box<PropertyTypeShape>>,
        /// Codegen extensions from schema
        extensions: CodegenExtensions,
        /// Default values for fields from JSON Schema
        defaults: std::collections::HashMap<String, serde_json::Value>,
    },
    
    /// Pure $ref to another schema (type alias)
    Ref {
        target: String,
    },
    
    /// `{"allOf": [...]}` composition
    AllOf {
        refs: Vec<String>,
        /// Inline properties added on top of allOf
        inline_properties: Vec<PropertyShape>,
    },
    
    /// `{"type": "array", "items": ...}` (dynamic size)
    Array {
        items: Box<SchemaShape>,
    },
    
    /// `{"type": "array", "items": ..., "minItems": N, "maxItems": N}` (fixed size)
    FixedArray {
        items: Box<SchemaShape>,
        size: usize,
    },
    
    /// `{"type": "array", "items": [...]}` where items is an array (tuple pattern)
    TupleArray {
        items: Vec<SchemaShape>,
    },
    
    /// Map type via additionalProperties
    Map {
        value_type: Box<PropertyTypeShape>,
    },
    
    /// Unrecognized/complex schema (needs diagnostic)
    Unknown {
        description: String,
    },
}

// =============================================================================
// Extension Extraction
// =============================================================================

/// Extract codegen extensions from x-familiar-* properties
fn extract_extensions(schema: &serde_json::Value) -> CodegenExtensions {
    let mut ext = CodegenExtensions::default();
    
    // x-familiar-variants: { "MOMENT": "Moment", ... }
    if let Some(variants) = schema.get("x-familiar-variants").and_then(|v| v.as_object()) {
        let map: std::collections::HashMap<String, String> = variants
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect();
        if !map.is_empty() {
            ext.variants = Some(map);
        }
    }
    
    // x-familiar-casing: "SCREAMING_SNAKE_CASE"
    if let Some(casing) = schema.get("x-familiar-casing").and_then(|v| v.as_str()) {
        ext.casing = Some(casing.to_string());
    }
    
    // x-familiar-default: true
    if let Some(default) = schema.get("x-familiar-default").and_then(|v| v.as_bool()) {
        ext.generate_default = default;
    }
    
    // x-familiar-rust-impl-ids: ["FieldExcitation"]
    if let Some(impl_ids) = schema.get("x-familiar-rust-impl-ids").and_then(|v| v.as_array()) {
        ext.rust_impl_ids = impl_ids
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        // If this schema has Rust impl blocks, it likely needs Default
        ext.generate_default = true;
        eprintln!("DEBUG: Found x-familiar-rust-impl-ids for schema, setting generate_default=true");
    }
    
    // x-familiar-field-alias: "physics"
    if let Some(alias) = schema.get("x-familiar-field-alias").and_then(|v| v.as_str()) {
        ext.field_alias = Some(alias.to_string());
    }
    
    ext
}

// =============================================================================
// Shape Detection
// =============================================================================

/// Detect the shape of a single schema
pub fn detect_shape(schema: &serde_json::Value) -> SchemaShape {
    // Debug: check if this schema has rust impl ids
    if schema.get("x-familiar-rust-impl-ids").is_some() {
        eprintln!("DEBUG: detect_shape called for schema with x-familiar-rust-impl-ids");
    }
    // Check for $ref first (pure alias)
    // Allow $ref with metadata fields like $schema, $id, title, description, x-familiar-*
    if let Some(ref_target) = schema.get("$ref").and_then(|v| v.as_str()) {
        if let Some(obj) = schema.as_object() {
            let non_meta_keys = obj.keys().filter(|k| {
                !k.starts_with('$') && 
                !k.starts_with("x-") && 
                *k != "title" && 
                *k != "description"
            }).count();
            
            if non_meta_keys == 0 {
                // Pure ref (only metadata fields)
                return SchemaShape::Ref {
                    target: ref_target.to_string(),
                };
            }
        }
    }
    
    // Check for oneOf
    if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_array()) {
        return detect_one_of_shape(schema, one_of);
    }
    
    // Check for allOf
    if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
        return detect_all_of_shape(schema, all_of);
    }
    
    // Check for type
    let json_type = schema.get("type").and_then(|v| v.as_str());
    
    match json_type {
        Some("string") => {
            // Check for enum
            if let Some(values) = schema.get("enum").and_then(|v| v.as_array()) {
                let string_values: Vec<String> = values
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                
                if string_values.len() == values.len() {
                    return SchemaShape::StringEnum { 
                        values: string_values,
                        extensions: extract_extensions(schema),
                    };
                }
            }
            
            let format = schema.get("format").and_then(|v| v.as_str()).map(String::from);
            SchemaShape::FormattedString { format }
        }
        
        Some("integer") => SchemaShape::Numeric {
            json_type: JsonScalarKind::Integer,
            format: schema.get("format").and_then(|v| v.as_str()).map(String::from),
        },
        
        Some("number") => SchemaShape::Numeric {
            json_type: JsonScalarKind::Number,
            format: schema.get("format").and_then(|v| v.as_str()).map(String::from),
        },
        
        Some("boolean") => SchemaShape::Boolean,
        
        Some("array") => {
            let min_items = schema.get("minItems").and_then(|v| v.as_u64());
            let max_items = schema.get("maxItems").and_then(|v| v.as_u64());
            
            // Check if items is an array (tuple pattern per JSON Schema)
            if let Some(items_array) = schema.get("items").and_then(|v| v.as_array()) {
                // Tuple: items is an array of types
                let tuple_items: Vec<SchemaShape> = items_array
                    .iter()
                    .map(|item| detect_shape(item))
                    .collect();
                SchemaShape::TupleArray { items: tuple_items }
            } else if let Some(items) = schema.get("items") {
                // items is an object (single type for all elements)
                // Check for fixed size: minItems == maxItems
                if let (Some(min), Some(max)) = (min_items, max_items) {
                    if min == max {
                        return SchemaShape::FixedArray {
                            items: Box::new(detect_shape(items)),
                            size: min as usize,
                        };
                    }
                }
                SchemaShape::Array {
                    items: Box::new(detect_shape(items)),
                }
            } else {
                SchemaShape::Unknown {
                    description: "array without items".to_string(),
                }
            }
        }
        
        Some("object") | None => {
            // Check if it has properties (struct) or additionalProperties (map)
            if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                let required: std::collections::HashSet<String> = schema
                    .get("required")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                
                let properties: Vec<PropertyShape> = props
                    .iter()
                    .map(|(name, prop)| PropertyShape {
                        name: name.clone(),
                        required: required.contains(name),
                        shape: detect_property_type(prop),
                    })
                    .collect();

                // Extract default values from properties
                let defaults: std::collections::HashMap<String, serde_json::Value> = props
                    .iter()
                    .filter_map(|(name, prop)| {
                        prop.get("default").map(|default| {
                            eprintln!("DEBUG: Found default for {}: {:?}", name, default);
                            (name.clone(), default.clone())
                        })
                    })
                    .collect();

                let additional_properties = schema
                    .get("additionalProperties")
                    .filter(|v| !v.is_boolean() || v.as_bool() != Some(false))
                    .map(|v| Box::new(detect_property_type(v)));

                SchemaShape::Object {
                    properties,
                    additional_properties,
                    extensions: extract_extensions(schema),
                    defaults,
                }
            } else if let Some(add_props) = schema.get("additionalProperties") {
                if !add_props.is_boolean() {
                    SchemaShape::Map {
                        value_type: Box::new(detect_property_type(add_props)),
                    }
                } else {
                    SchemaShape::Unknown {
                        description: "object with boolean additionalProperties".to_string(),
                    }
                }
            } else if json_type.is_none() && schema.get("enum").is_some() {
                // Enum without type
                if let Some(values) = schema.get("enum").and_then(|v| v.as_array()) {
                    let string_values: Vec<String> = values
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    
                    if string_values.len() == values.len() {
                        return SchemaShape::StringEnum { 
                            values: string_values,
                            extensions: extract_extensions(schema),
                        };
                    }
                }
                SchemaShape::Unknown {
                    description: "enum with non-string values".to_string(),
                }
            } else {
                SchemaShape::Unknown {
                    description: format!("object without properties, type={:?}", json_type),
                }
            }
        }
        
        Some(other) => SchemaShape::Unknown {
            description: format!("unknown type: {}", other),
        },
    }
}

/// Detect oneOf shape
fn detect_one_of_shape(schema: &serde_json::Value, one_of: &[serde_json::Value]) -> SchemaShape {
    let discriminator = schema
        .get("x-familiar-discriminator")
        .and_then(|v| v.as_str())
        .map(String::from);
    
    // Check if all variants are const/enum strings
    let all_string_consts = one_of.iter().all(|v| {
        v.get("const").and_then(|c| c.as_str()).is_some()
            || (v.get("type").and_then(|t| t.as_str()) == Some("string")
                && v.get("enum").and_then(|e| e.as_array()).map(|a| a.len() == 1).unwrap_or(false))
    });
    
    if all_string_consts {
        let variants: Vec<String> = one_of
            .iter()
            .filter_map(|v| {
                v.get("const")
                    .and_then(|c| c.as_str())
                    .map(String::from)
                    .or_else(|| {
                        v.get("enum")
                            .and_then(|e| e.as_array())
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
            })
            .collect();
        
        return SchemaShape::OneOfStringEnum { 
            variants,
            extensions: extract_extensions(schema),
        };
    }
    
    // Check if all variants are objects or refs
    let all_objects_or_refs = one_of.iter().all(|v| {
        v.get("$ref").is_some()
            || v.get("properties").is_some()
            || v.get("type").and_then(|t| t.as_str()) == Some("object")
    });
    
    if all_objects_or_refs {
        let variants: Vec<ObjectVariant> = one_of
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let name = v.get("title").and_then(|t| t.as_str()).map(String::from);
                let ref_target = v.get("$ref").and_then(|r| r.as_str()).map(String::from);
                let properties = v
                    .get("properties")
                    .and_then(|p| p.as_object())
                    .map(|p| p.keys().cloned().collect())
                    .unwrap_or_default();
                
                // Use title, then $ref type name, then indexed fallback "Variant0", "Variant1", etc.
                let derived_name = name
                    .or_else(|| ref_target.as_ref().map(|r| extract_type_name(r)))
                    .unwrap_or_else(|| format!("Variant{}", i));
                
                ObjectVariant {
                    name: Some(derived_name),
                    properties,
                    ref_target,
                }
            })
            .collect();
        
        return SchemaShape::OneOfObjects {
            variants,
            discriminator,
        };
    }
    
    // Mixed oneOf
    SchemaShape::OneOfMixed {
        description: format!("oneOf with {} mixed variants", one_of.len()),
    }
}

/// Detect allOf shape
fn detect_all_of_shape(schema: &serde_json::Value, all_of: &[serde_json::Value]) -> SchemaShape {
    let refs: Vec<String> = all_of
        .iter()
        .filter_map(|v| v.get("$ref").and_then(|r| r.as_str()).map(String::from))
        .collect();
    
    // Check for inline properties on top of allOf
    let inline_properties: Vec<PropertyShape> = schema
        .get("properties")
        .and_then(|v| v.as_object())
        .map(|props| {
            let required: std::collections::HashSet<String> = schema
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            props
                .iter()
                .map(|(name, prop)| PropertyShape {
                    name: name.clone(),
                    required: required.contains(name),
                    shape: detect_property_type(prop),
                })
                .collect()
        })
        .unwrap_or_default();
    
    SchemaShape::AllOf {
        refs,
        inline_properties,
    }
}

/// Detect property type shape
fn detect_property_type(prop: &serde_json::Value) -> PropertyTypeShape {
    // Check for direct $ref
    if let Some(ref_target) = prop.get("$ref").and_then(|v| v.as_str()) {
        return PropertyTypeShape::Ref(ref_target.to_string());
    }
    
    // Check for allOf with single $ref (common pattern for adding description to a $ref)
    // e.g., {"allOf": [{"$ref": "..."}], "description": "..."}
    if let Some(all_of) = prop.get("allOf").and_then(|v| v.as_array()) {
        // If allOf has a single $ref element, treat as Ref
        let refs: Vec<&str> = all_of
            .iter()
            .filter_map(|v| v.get("$ref").and_then(|r| r.as_str()))
            .collect();
        
        if refs.len() == 1 && all_of.len() == 1 {
            return PropertyTypeShape::Ref(refs[0].to_string());
        }
        
        // Multiple refs in allOf - could be composition, fall back to first ref
        if !refs.is_empty() {
            return PropertyTypeShape::Ref(refs[0].to_string());
        }
    }
    
    // Check for anyOf (used for optional types like "null | T")
    if let Some(any_of) = prop.get("anyOf").and_then(|v| v.as_array()) {
        // Look for non-null type
        for variant in any_of {
            if variant.get("type").and_then(|t| t.as_str()) != Some("null") {
                return detect_property_type(variant);
            }
        }
    }
    
    let json_type = prop.get("type").and_then(|v| v.as_str());
    
    match json_type {
        Some("string") => PropertyTypeShape::Scalar(JsonScalarKind::String),
        Some("integer") => PropertyTypeShape::Scalar(JsonScalarKind::Integer),
        Some("number") => PropertyTypeShape::Scalar(JsonScalarKind::Number),
        Some("boolean") => PropertyTypeShape::Scalar(JsonScalarKind::Boolean),
        Some("null") => PropertyTypeShape::Scalar(JsonScalarKind::Null),
        Some("array") => {
            let min_items = prop.get("minItems").and_then(|v| v.as_u64());
            let max_items = prop.get("maxItems").and_then(|v| v.as_u64());
            
            // Check if items is an array (tuple pattern per JSON Schema)
            if let Some(items_array) = prop.get("items").and_then(|v| v.as_array()) {
                // Tuple: items is an array of types, each position has its own type
                let tuple_items: Vec<PropertyTypeShape> = items_array
                    .iter()
                    .map(|item| detect_property_type(item))
                    .collect();
                PropertyTypeShape::Tuple { items: tuple_items }
            } else if let Some(items) = prop.get("items") {
                // items is an object (single type for all elements)
                // Check for fixed size: minItems == maxItems
                if let (Some(min), Some(max)) = (min_items, max_items) {
                    if min == max {
                        return PropertyTypeShape::FixedArray {
                            items: Box::new(detect_property_type(items)),
                            size: min as usize,
                        };
                    }
                }
                PropertyTypeShape::Array {
                    items: Box::new(detect_property_type(items)),
                }
            } else {
                PropertyTypeShape::Unknown
            }
        }
        Some("object") => PropertyTypeShape::InlineObject,
        _ => PropertyTypeShape::Unknown,
    }
}

/// Extract type name from a $ref path
fn extract_type_name(ref_path: &str) -> String {
    ref_path
        .rsplit('/')
        .next()
        .unwrap_or(ref_path)
        .trim_end_matches(".schema.json")
        .trim_end_matches(".json")
        .to_string()
}

// =============================================================================
// Batch Detection
// =============================================================================

/// Detect shapes for all schemas in a graph
pub fn detect_all_shapes(graph: &SchemaGraph) -> HashMap<SchemaId, SchemaShape> {
    let mut shapes = HashMap::with_capacity(graph.schema_count());
    
    for schema_id in graph.all_ids() {
        if let Some(raw) = graph.get_raw(schema_id) {
            shapes.insert(schema_id.clone(), detect_shape(raw));
        }
    }
    
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_detect_string_enum() {
        let schema = json!({
            "type": "string",
            "enum": ["pending", "active", "completed"]
        });
        
        match detect_shape(&schema) {
            SchemaShape::StringEnum { values, .. } => {
                assert_eq!(values, vec!["pending", "active", "completed"]);
            }
            other => panic!("Expected StringEnum, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_one_of_string_enum() {
        let schema = json!({
            "oneOf": [
                { "const": "admin" },
                { "const": "member" },
                { "const": "guest" }
            ]
        });
        
        match detect_shape(&schema) {
            SchemaShape::OneOfStringEnum { variants, .. } => {
                assert_eq!(variants, vec!["admin", "member", "guest"]);
            }
            other => panic!("Expected OneOfStringEnum, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_object() {
        let schema = json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "count": { "type": "integer" }
            },
            "required": ["id"]
        });
        
        match detect_shape(&schema) {
            SchemaShape::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                assert!(properties.iter().find(|p| p.name == "id" && p.required).is_some());
                assert!(properties.iter().find(|p| p.name == "count" && !p.required).is_some());
            }
            other => panic!("Expected Object, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_ref() {
        let schema = json!({
            "$ref": "primitives/TenantId.schema.json"
        });
        
        match detect_shape(&schema) {
            SchemaShape::Ref { target } => {
                assert_eq!(target, "primitives/TenantId.schema.json");
            }
            other => panic!("Expected Ref, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_fixed_array() {
        // FieldExcitation.position pattern: fixed-size array of 3
        let schema = json!({
            "type": "array",
            "items": { "$ref": "primitives/QuantizedCoord.schema.json" },
            "minItems": 3,
            "maxItems": 3
        });
        
        match detect_shape(&schema) {
            SchemaShape::FixedArray { items, size } => {
                assert_eq!(size, 3);
                match *items {
                    SchemaShape::Ref { target } => {
                        assert!(target.contains("QuantizedCoord"));
                    }
                    other => panic!("Expected Ref items, got {:?}", other),
                }
            }
            other => panic!("Expected FixedArray, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_tuple_array() {
        // QuantumState.amplitudes inner pattern: tuple of (number, number)
        let schema = json!({
            "type": "array",
            "items": [
                { "type": "number" },
                { "type": "number" }
            ],
            "minItems": 2,
            "maxItems": 2
        });
        
        match detect_shape(&schema) {
            SchemaShape::TupleArray { items } => {
                assert_eq!(items.len(), 2);
                for item in &items {
                    match item {
                        SchemaShape::Numeric { json_type, .. } => {
                            assert_eq!(*json_type, JsonScalarKind::Number);
                        }
                        other => panic!("Expected Numeric items, got {:?}", other),
                    }
                }
            }
            other => panic!("Expected TupleArray, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_property_fixed_array() {
        // Property-level fixed array detection
        let prop = json!({
            "type": "array",
            "items": { "type": "number" },
            "minItems": 3,
            "maxItems": 3
        });
        
        match detect_property_type(&prop) {
            PropertyTypeShape::FixedArray { items, size } => {
                assert_eq!(size, 3);
                match *items {
                    PropertyTypeShape::Scalar(JsonScalarKind::Number) => {}
                    other => panic!("Expected Scalar Number, got {:?}", other),
                }
            }
            other => panic!("Expected FixedArray, got {:?}", other),
        }
    }
    
    #[test]
    fn test_detect_property_tuple() {
        // Property-level tuple detection
        let prop = json!({
            "type": "array",
            "items": [
                { "type": "number" },
                { "type": "string" }
            ]
        });
        
        match detect_property_type(&prop) {
            PropertyTypeShape::Tuple { items } => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], PropertyTypeShape::Scalar(JsonScalarKind::Number)));
                assert!(matches!(items[1], PropertyTypeShape::Scalar(JsonScalarKind::String)));
            }
            other => panic!("Expected Tuple, got {:?}", other),
        }
    }
}

