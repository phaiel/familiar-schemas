//! Type Classification
//!
//! Determines HOW to generate code for each schema, consuming:
//! - SchemaShape (from patterns.rs)
//! - CycleHandling (from analysis.rs)
//! - Primitives set (schemas that exist in familiar-primitives)
//!
//! Classification produces TypeKind + EmitStrategy which are language-agnostic.
//! Language-specific lowering happens in the emitters.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::analysis::{CycleHandling, SccAnalysis};
use super::patterns::{JsonScalarKind, PropertyShape, SchemaShape};
use super::{SchemaGraph, SchemaId};

// =============================================================================
// Enum Variant
// =============================================================================

/// A variant in an enum type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Original value from schema
    pub value: String,
    /// Rust-safe name (PascalCase, escaped if keyword)
    pub rust_name: String,
    /// Whether to emit #[serde(rename = "...")] 
    pub needs_rename: bool,
}

impl EnumVariant {
    pub fn from_value(value: &str) -> Self {
        let rust_name = to_pascal_case(value);
        let needs_rename = rust_name != value;
        Self {
            value: value.to_string(),
            rust_name,
            needs_rename,
        }
    }
    
    /// Create variant using x-familiar-variants mapping if available
    /// 
    /// If the schema has `x-familiar-variants: { "MOMENT": "Moment" }`,
    /// we keep MOMENT as the Rust variant name (matching code expectations)
    /// and add serde rename for serialization.
    pub fn from_value_with_extensions(
        value: &str, 
        extensions: &super::patterns::CodegenExtensions,
    ) -> Self {
        // If we have a variant mapping, use the original value as Rust name
        // (this preserves SCREAMING_CASE when that's what the schema specifies)
        if let Some(ref variants) = extensions.variants {
            if variants.contains_key(value) {
                // Value is in mapping - keep original as Rust variant name
                // The mapping's value is what JSON serializes to (but we may need reverse)
                return Self {
                    value: value.to_string(),
                    rust_name: value.to_string(), // Keep SCREAMING_CASE
                    needs_rename: false, // No rename needed - serialize as-is
                };
            }
        }
        
        // If casing is SCREAMING_SNAKE_CASE, preserve original
        if let Some(ref casing) = extensions.casing {
            if casing == "SCREAMING_SNAKE_CASE" {
                return Self {
                    value: value.to_string(),
                    rust_name: value.to_string(), // Keep original
                    needs_rename: false,
                };
            }
        }
        
        // Default: convert to PascalCase
        Self::from_value(value)
    }
}

// =============================================================================
// Union Variant
// =============================================================================

/// A variant in a tagged union
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionVariant {
    /// Variant name (for Rust enum)
    pub name: String,
    /// Schema ID this variant references (if $ref)
    pub schema_ref: Option<SchemaId>,
    /// Inline type if not a $ref
    pub inline_type: Option<InlineType>,
}

/// Inline type definition (for unions with inline variants)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineType {
    pub properties: Vec<FieldDef>,
}

// =============================================================================
// Field Definition
// =============================================================================

/// A field in a struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    /// Original JSON name
    pub json_name: String,
    /// Rust-safe field name (snake_case, escaped if keyword)
    pub rust_name: String,
    /// Whether to emit #[serde(rename = "...")]
    pub needs_rename: bool,
    /// Is this field required?
    pub required: bool,
    /// Field type (language-agnostic)
    pub field_type: FieldType,
}

/// Language-agnostic field type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    /// Reference to another schema
    SchemaRef(SchemaId),
    /// JSON scalar type
    Scalar(JsonScalarKind),
    /// Array of another type (dynamic size)
    Array(Box<FieldType>),
    /// Fixed-size array (known at compile time)
    FixedArray { items: Box<FieldType>, size: usize },
    /// Tuple type (heterogeneous fixed-size array)
    Tuple(Vec<FieldType>),
    /// Map with string keys
    Map(Box<FieldType>),
    /// Inline anonymous object
    InlineObject,
    /// Unknown/any type
    Unknown,
}

// =============================================================================
// Type Kind (Language-Agnostic)
// =============================================================================

/// What kind of type to generate (language-agnostic classification)
/// 
/// Note: This does NOT contain Rust-specific types like `RustType`.
/// Language-specific lowering happens in emitters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    /// Simple string enum: `enum Foo { A, B, C }`
    Enum {
        variants: Vec<EnumVariant>,
    },
    
    /// Tagged union: `enum Foo { A(TypeA), B(TypeB) }`
    TaggedUnion {
        discriminator: Option<String>,
        tag_field: Option<String>,
        variants: Vec<UnionVariant>,
    },
    
    /// Struct with named fields
    Struct {
        fields: Vec<FieldDef>,
        /// Whether to flatten allOf refs
        flatten_refs: Vec<SchemaId>,
    },
    
    /// Newtype wrapper around another type
    Newtype {
        /// The wrapped type (schema ref or scalar)
        inner: FieldType,
    },
    
    /// Type alias (re-export from another location)
    Alias {
        target: SchemaId,
    },
    
    /// Primitive type that lives in familiar-primitives
    /// Do not generate - just reference
    Primitive,
}

// =============================================================================
// Emit Strategy
// =============================================================================

/// What to do with this schema during code generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmitStrategy {
    /// Generate the type normally
    Generate,
    
    /// Re-export from familiar-primitives (don't generate)
    ReExportPrimitive,
    
    /// Skip entirely (primitive handled elsewhere)
    Skip,
    
    /// Generate as part of an SCC group (deterministic order)
    /// The usize is the SCC group ID for ordering
    GenerateInSccGroup(usize),
}

// =============================================================================
// Classification Result
// =============================================================================

/// Classification result for a single schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub schema_id: SchemaId,
    pub type_kind: TypeKind,
    pub emit_strategy: EmitStrategy,
    /// Rust type name to generate
    pub rust_name: String,
}

// =============================================================================
// Classifier
// =============================================================================

/// Classifies all schemas in a graph
pub struct Classifier<'a> {
    graph: &'a SchemaGraph,
    shapes: &'a HashMap<SchemaId, SchemaShape>,
    scc_analysis: &'a SccAnalysis,
    primitives: HashSet<SchemaId>,
}

impl<'a> Classifier<'a> {
    pub fn new(
        graph: &'a SchemaGraph,
        shapes: &'a HashMap<SchemaId, SchemaShape>,
        scc_analysis: &'a SccAnalysis,
        primitives: HashSet<SchemaId>,
    ) -> Self {
        Self {
            graph,
            shapes,
            scc_analysis,
            primitives,
        }
    }
    
    /// Classify all schemas
    pub fn classify_all(&self) -> HashMap<SchemaId, Classification> {
        let mut classifications = HashMap::with_capacity(self.graph.schema_count());
        
        for schema_id in self.graph.all_ids() {
            if let Some(classification) = self.classify(schema_id) {
                classifications.insert(schema_id.clone(), classification);
            }
        }
        
        classifications
    }
    
    /// Classify a single schema
    pub fn classify(&self, schema_id: &str) -> Option<Classification> {
        let shape = self.shapes.get(schema_id)?;
        let cycle_handling = self.scc_analysis.get(schema_id);
        let rust_name = self.compute_rust_name(schema_id);
        
        // Check if this is a primitive (skip based on canonical ID, not shape)
        if self.primitives.contains(schema_id) {
            return Some(Classification {
                schema_id: schema_id.to_string(),
                type_kind: TypeKind::Primitive,
                emit_strategy: EmitStrategy::Skip,
                rust_name,
            });
        }
        
        let (type_kind, emit_strategy) = self.classify_shape(schema_id, shape, cycle_handling);
        
        Some(Classification {
            schema_id: schema_id.to_string(),
            type_kind,
            emit_strategy,
            rust_name,
        })
    }
    
    fn classify_shape(
        &self,
        _schema_id: &str,
        shape: &SchemaShape,
        cycle_handling: Option<&CycleHandling>,
    ) -> (TypeKind, EmitStrategy) {
        let emit_strategy = if let Some(ch) = cycle_handling {
            if let Some(scc_id) = ch.scc_id {
                EmitStrategy::GenerateInSccGroup(scc_id)
            } else {
                EmitStrategy::Generate
            }
        } else {
            EmitStrategy::Generate
        };
        
        let type_kind = match shape {
            SchemaShape::StringEnum { values, extensions } => {
                TypeKind::Enum {
                    variants: values.iter()
                        .map(|v| EnumVariant::from_value_with_extensions(v, extensions))
                        .collect(),
                }
            }
            
            SchemaShape::OneOfStringEnum { variants, extensions } => {
                TypeKind::Enum {
                    variants: variants.iter()
                        .map(|v| EnumVariant::from_value_with_extensions(v, extensions))
                        .collect(),
                }
            }
            
            SchemaShape::OneOfObjects { variants, discriminator } => {
                let union_variants: Vec<UnionVariant> = variants
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        // Name should always be present after patterns.rs fix, but provide fallback
                        let name = v.name.clone().unwrap_or_else(|| format!("Variant{}", i));
                        UnionVariant {
                            name: to_pascal_case(&name),
                            schema_ref: v.ref_target.clone(),
                            inline_type: None, // TODO: handle inline objects
                        }
                    })
                    .collect();
                
                TypeKind::TaggedUnion {
                    discriminator: discriminator.clone(),
                    tag_field: discriminator.clone(), // Often the same
                    variants: union_variants,
                }
            }
            
            SchemaShape::OneOfMixed { .. } => {
                // Fall back to unknown/any type
                TypeKind::Newtype {
                    inner: FieldType::Unknown,
                }
            }
            
            SchemaShape::FormattedString { format } => {
                // Check if format suggests a specific type
                match format.as_deref() {
                    Some("date-time") | Some("date") | Some("time") => {
                        TypeKind::Newtype {
                            inner: FieldType::Scalar(JsonScalarKind::String),
                        }
                    }
                    Some("uuid") => {
                        TypeKind::Newtype {
                            inner: FieldType::Scalar(JsonScalarKind::String),
                        }
                    }
                    Some("email") | Some("uri") | Some("hostname") => {
                        TypeKind::Newtype {
                            inner: FieldType::Scalar(JsonScalarKind::String),
                        }
                    }
                    _ => {
                        TypeKind::Newtype {
                            inner: FieldType::Scalar(JsonScalarKind::String),
                        }
                    }
                }
            }
            
            SchemaShape::Numeric { json_type, .. } => {
                TypeKind::Newtype {
                    inner: FieldType::Scalar(json_type.clone()),
                }
            }
            
            SchemaShape::Boolean => {
                TypeKind::Newtype {
                    inner: FieldType::Scalar(JsonScalarKind::Boolean),
                }
            }
            
            SchemaShape::Object { properties, additional_properties: _, extensions: _, defaults: _ } => {
                let fields: Vec<FieldDef> = properties
                    .iter()
                    .map(|p| self.property_to_field(p))
                    .collect();
                
                TypeKind::Struct {
                    fields,
                    flatten_refs: Vec::new(),
                }
            }
            
            SchemaShape::Ref { target } => {
                // Check if target is a primitive
                if self.primitives.contains(target) {
                    return (
                        TypeKind::Alias { target: target.clone() },
                        EmitStrategy::ReExportPrimitive,
                    );
                }
                
                TypeKind::Alias { target: target.clone() }
            }
            
            SchemaShape::AllOf { refs, inline_properties } => {
                let fields: Vec<FieldDef> = inline_properties
                    .iter()
                    .map(|p| self.property_to_field(p))
                    .collect();
                
                TypeKind::Struct {
                    fields,
                    flatten_refs: refs.clone(),
                }
            }
            
            SchemaShape::Array { items } => {
                let inner_type = self.shape_to_field_type(items);
                TypeKind::Newtype {
                    inner: FieldType::Array(Box::new(inner_type)),
                }
            }
            
            SchemaShape::FixedArray { items, size } => {
                let inner_type = self.shape_to_field_type(items);
                TypeKind::Newtype {
                    inner: FieldType::FixedArray {
                        items: Box::new(inner_type),
                        size: *size,
                    },
                }
            }
            
            SchemaShape::TupleArray { items } => {
                let tuple_types: Vec<FieldType> = items
                    .iter()
                    .map(|i| self.shape_to_field_type(i))
                    .collect();
                TypeKind::Newtype {
                    inner: FieldType::Tuple(tuple_types),
                }
            }
            
            SchemaShape::Map { value_type } => {
                let inner_type = self.property_type_to_field_type(value_type);
                TypeKind::Newtype {
                    inner: FieldType::Map(Box::new(inner_type)),
                }
            }
            
            SchemaShape::Unknown { .. } => {
                TypeKind::Newtype {
                    inner: FieldType::Unknown,
                }
            }
        };
        
        (type_kind, emit_strategy)
    }
    
    fn property_to_field(&self, prop: &PropertyShape) -> FieldDef {
        // Handle special characters in field names
        // $ is not valid in Rust identifiers, so strip it
        let sanitized_name = if prop.name.starts_with('$') {
            format!("schema_{}", &prop.name[1..])
        } else {
            prop.name.clone()
        };
        
        let rust_name = to_snake_case(&sanitized_name);
        let needs_rename = rust_name != prop.name;
        
        FieldDef {
            json_name: prop.name.clone(),
            rust_name,
            needs_rename,
            required: prop.required,
            field_type: self.property_type_to_field_type(&prop.shape),
        }
    }
    
    fn property_type_to_field_type(
        &self,
        prop_type: &super::patterns::PropertyTypeShape,
    ) -> FieldType {
        use super::patterns::PropertyTypeShape;
        
        match prop_type {
            PropertyTypeShape::Ref(target) => FieldType::SchemaRef(target.clone()),
            PropertyTypeShape::Scalar(kind) => FieldType::Scalar(kind.clone()),
            PropertyTypeShape::Array { items } => {
                FieldType::Array(Box::new(self.property_type_to_field_type(items)))
            }
            PropertyTypeShape::FixedArray { items, size } => {
                FieldType::FixedArray {
                    items: Box::new(self.property_type_to_field_type(items)),
                    size: *size,
                }
            }
            PropertyTypeShape::Tuple { items } => {
                FieldType::Tuple(
                    items.iter()
                        .map(|i| self.property_type_to_field_type(i))
                        .collect()
                )
            }
            PropertyTypeShape::InlineObject => FieldType::InlineObject,
            PropertyTypeShape::Unknown => FieldType::Unknown,
        }
    }
    
    fn shape_to_field_type(&self, shape: &SchemaShape) -> FieldType {
        match shape {
            SchemaShape::Ref { target } => FieldType::SchemaRef(target.clone()),
            SchemaShape::FormattedString { .. } => FieldType::Scalar(JsonScalarKind::String),
            SchemaShape::Numeric { json_type, .. } => FieldType::Scalar(json_type.clone()),
            SchemaShape::Boolean => FieldType::Scalar(JsonScalarKind::Boolean),
            SchemaShape::Array { items } => {
                FieldType::Array(Box::new(self.shape_to_field_type(items)))
            }
            SchemaShape::FixedArray { items, size } => {
                FieldType::FixedArray {
                    items: Box::new(self.shape_to_field_type(items)),
                    size: *size,
                }
            }
            SchemaShape::TupleArray { items } => {
                FieldType::Tuple(
                    items.iter()
                        .map(|i| self.shape_to_field_type(i))
                        .collect()
                )
            }
            _ => FieldType::Unknown,
        }
    }
    
    fn compute_rust_name(&self, schema_id: &str) -> String {
        // Try to get title from schema node
        if let Some(node) = self.graph.get(schema_id) {
            if let Some(title) = &node.title {
                return to_pascal_case(title);
            }
        }
        
        // Fall back to extracting from path
        let name = schema_id
            .rsplit('/')
            .next()
            .unwrap_or(schema_id)
            .trim_end_matches(".schema.json")
            .trim_end_matches(".json");
        
        to_pascal_case(name)
    }
}

// =============================================================================
// Naming Utilities
// =============================================================================

/// Convert string to PascalCase
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    
    // Check if all uppercase (like SCREAMING_SNAKE_CASE)
    let is_all_caps = s.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c == '-');
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if is_all_caps {
            // Convert to lowercase for SCREAMING_SNAKE_CASE
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    
    // Handle keywords - add trailing underscore
    if is_rust_keyword(&result.to_lowercase()) {
        result.push('_');
    }
    
    result
}

/// Convert string to snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_lower = false;
    
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if prev_lower {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else if c == '-' || c == ' ' {
            result.push('_');
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_ascii_lowercase();
        }
    }
    
    // Handle keywords
    if is_rust_keyword(&result) {
        result.push('_');
    }
    
    result
}

/// Check if a string is a Rust keyword
fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn" |
        "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" |
        "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" |
        "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" |
        "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" |
        // Reserved for future use
        "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" |
        "priv" | "try" | "typeof" | "unsized" | "virtual" | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("PENDING"), "Pending");
        assert_eq!(to_pascal_case("SCREAMING_SNAKE"), "ScreamingSnake");
        // Keywords get trailing underscore
        assert_eq!(to_pascal_case("type"), "Type_");
        assert_eq!(to_pascal_case("Type"), "Type_");
    }
    
    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
        assert_eq!(to_snake_case("type"), "type_"); // Keyword
    }
    
    #[test]
    fn test_enum_variant_from_value() {
        let v = EnumVariant::from_value("PENDING");
        assert_eq!(v.rust_name, "Pending");
        assert!(v.needs_rename); // PENDING != Pending
        
        let v = EnumVariant::from_value("Active");
        assert_eq!(v.rust_name, "Active");
        assert!(!v.needs_rename);
    }
}

