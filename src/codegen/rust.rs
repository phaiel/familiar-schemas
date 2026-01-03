//! Rust Code Emitter
//!
//! Generates Rust code from Regions. Pure projection - no raw JSON access.
//! 
//! Key constraint: This module ONLY receives Region - no SchemaGraph, no raw JSON.

use crate::graph::{
    EmitStrategy, EnumVariant, FieldDef, FieldType, JsonScalarKind, TypeKind, UnionVariant,
};

use super::{CodegenContext, Region};

// =============================================================================
// Public API
// =============================================================================

/// Emit Rust code for a Region
/// 
/// Returns None if the region should not generate code (primitive, skip, etc.)
pub fn emit_region(region: &Region, ctx: &CodegenContext) -> Option<String> {
    match &region.emit_strategy {
        EmitStrategy::Skip => None,
        EmitStrategy::ReExportPrimitive => {
            // Could emit a type alias to familiar_primitives
            Some(emit_primitive_alias(region))
        }
        EmitStrategy::Generate | EmitStrategy::GenerateInSccGroup(_) => {
            Some(emit_type(region, ctx))
        }
    }
}

// =============================================================================
// Type Emission
// =============================================================================

fn emit_type(region: &Region, ctx: &CodegenContext) -> String {
    let mut output = String::new();
    
    // Add doc comment if available
    output.push_str(&format!("/// {}\n", region.rust_name));
    
    // Add derives
    output.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]\n");
    
    match &region.type_kind {
        TypeKind::Enum { variants } => {
            emit_enum(&mut output, region, variants);
        }
        TypeKind::TaggedUnion { discriminator, tag_field, variants } => {
            emit_tagged_union(&mut output, region, discriminator, tag_field, variants);
        }
        TypeKind::Struct { fields, flatten_refs } => {
            emit_struct(&mut output, region, fields, flatten_refs, ctx);
        }
        TypeKind::Newtype { inner } => {
            emit_newtype(&mut output, region, inner);
        }
        TypeKind::Alias { target } => {
            emit_alias(&mut output, region, target, ctx);
        }
        TypeKind::Primitive => {
            // Should not reach here - handled by EmitStrategy::Skip
        }
    }
    
    output
}

// =============================================================================
// Enum Emission
// =============================================================================

fn emit_enum(output: &mut String, region: &Region, variants: &[EnumVariant]) {
    output.push_str(&format!("pub enum {} {{\n", region.rust_name));
    
    for variant in variants {
        if variant.needs_rename {
            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", variant.value));
        }
        output.push_str(&format!("    {},\n", variant.rust_name));
    }
    
    output.push_str("}\n");
}

// =============================================================================
// Tagged Union Emission
// =============================================================================

fn emit_tagged_union(
    output: &mut String,
    region: &Region,
    discriminator: &Option<String>,
    tag_field: &Option<String>,
    variants: &[UnionVariant],
) {
    // Add serde tag attribute if discriminator is specified
    if let Some(tag) = tag_field.as_ref().or(discriminator.as_ref()) {
        output.push_str(&format!("#[serde(tag = \"{}\")]\n", tag));
    }
    
    output.push_str(&format!("pub enum {} {{\n", region.rust_name));
    
    for variant in variants {
        // If variant refs another schema, use that type
        if let Some(ref schema_ref) = variant.schema_ref {
            let type_name = extract_type_name(schema_ref);
            output.push_str(&format!("    {}({}),\n", variant.name, type_name));
        } else {
            // Inline variant (unit for now)
            output.push_str(&format!("    {},\n", variant.name));
        }
    }
    
    output.push_str("}\n");
}

// =============================================================================
// Struct Emission
// =============================================================================

fn emit_struct(
    output: &mut String,
    region: &Region,
    fields: &[FieldDef],
    flatten_refs: &[String],
    _ctx: &CodegenContext,
) {
    output.push_str(&format!("pub struct {} {{\n", region.rust_name));
    
    // Emit flattened refs first
    for ref_target in flatten_refs {
        let type_name = extract_type_name(ref_target);
        output.push_str("    #[serde(flatten)]\n");
        output.push_str(&format!("    pub {}: {},\n", to_snake_case(&type_name), type_name));
    }
    
    // Emit fields
    for field in fields {
        let needs_box = region.needs_boxing(&field.json_name);
        emit_field(output, field, needs_box);
    }
    
    output.push_str("}\n");
}

fn emit_field(output: &mut String, field: &FieldDef, needs_box: bool) {
    // Add serde rename if needed
    if field.needs_rename {
        output.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.json_name));
    }
    
    // Skip None if optional
    if !field.required {
        output.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
    }
    
    // Determine type
    let rust_type = field_type_to_rust(&field.field_type, needs_box);
    
    // Wrap in Option if not required
    let full_type = if field.required {
        rust_type
    } else {
        format!("Option<{}>", rust_type)
    };
    
    output.push_str(&format!("    pub {}: {},\n", field.rust_name, full_type));
}

// =============================================================================
// Newtype Emission
// =============================================================================

fn emit_newtype(output: &mut String, region: &Region, inner: &FieldType) {
    let rust_type = field_type_to_rust(inner, false);
    output.push_str(&format!("pub struct {}(pub {});\n", region.rust_name, rust_type));
}

// =============================================================================
// Alias Emission
// =============================================================================

fn emit_alias(output: &mut String, region: &Region, target: &str, _ctx: &CodegenContext) {
    let target_name = extract_type_name(target);
    output.push_str(&format!(
        "pub type {} = {};\n",
        region.rust_name, target_name
    ));
}

fn emit_primitive_alias(region: &Region) -> String {
    format!(
        "// {} is a primitive - re-exported from familiar_primitives\n",
        region.rust_name
    )
}

// =============================================================================
// Type Conversion Utilities
// =============================================================================

/// Convert a FieldType to a Rust type string
fn field_type_to_rust(field_type: &FieldType, needs_box: bool) -> String {
    let base_type = match field_type {
        FieldType::SchemaRef(schema_id) => {
            let type_name = extract_type_name(schema_id);
            if needs_box {
                format!("Box<{}>", type_name)
            } else {
                type_name
            }
        }
        FieldType::Scalar(scalar) => scalar_to_rust(scalar),
        FieldType::Array(inner) => {
            let inner_type = field_type_to_rust(inner, false);
            format!("Vec<{}>", inner_type)
        }
        FieldType::Map(value) => {
            let value_type = field_type_to_rust(value, false);
            format!("std::collections::HashMap<String, {}>", value_type)
        }
        FieldType::InlineObject => "serde_json::Value".to_string(),
        FieldType::Unknown => "serde_json::Value".to_string(),
    };
    
    base_type
}

/// Convert a JSON scalar kind to a Rust type
fn scalar_to_rust(scalar: &JsonScalarKind) -> String {
    match scalar {
        JsonScalarKind::String => "String".to_string(),
        JsonScalarKind::Integer => "i64".to_string(),
        JsonScalarKind::Number => "f64".to_string(),
        JsonScalarKind::Boolean => "bool".to_string(),
        JsonScalarKind::Null => "()".to_string(),
    }
}

/// Extract type name from a schema path/id
fn extract_type_name(schema_ref: &str) -> String {
    let name = schema_ref
        .rsplit('/')
        .next()
        .unwrap_or(schema_ref)
        .trim_end_matches(".schema.json")
        .trim_end_matches(".json");
    
    to_pascal_case(name)
}

/// Convert to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
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
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_type_name() {
        assert_eq!(extract_type_name("primitives/TenantId.schema.json"), "TenantId");
        assert_eq!(extract_type_name("auth/User.schema.json"), "User");
        assert_eq!(extract_type_name("MyType"), "MyType");
    }
    
    #[test]
    fn test_scalar_to_rust() {
        assert_eq!(scalar_to_rust(&JsonScalarKind::String), "String");
        assert_eq!(scalar_to_rust(&JsonScalarKind::Integer), "i64");
        assert_eq!(scalar_to_rust(&JsonScalarKind::Number), "f64");
        assert_eq!(scalar_to_rust(&JsonScalarKind::Boolean), "bool");
    }
}

