//! Rust Code Emitter
//!
//! Generates Rust code from Regions using RenderProfile for configuration.
//! 
//! Key constraints:
//! - This module ONLY receives Region + RenderProfile - no raw JSON
//! - Type names come from Region.canonical_name (already resolved)
//! - Type mappings come from RenderProfile (configurable)

use crate::graph::{
    EmitStrategy, EnumVariant, FieldDef, TypeKind, UnionVariant,
};

use super::{CodegenContext, Region, RenderProfile};

// =============================================================================
// Public API
// =============================================================================

/// Emit Rust code for a Region
/// 
/// Returns None if the region should not generate code (primitive, skip, etc.)
pub fn emit_region(region: &Region, ctx: &CodegenContext, profile: &RenderProfile) -> Option<String> {
    match &region.emit_strategy {
        EmitStrategy::Skip => None,
        EmitStrategy::ReExportPrimitive => {
            // Primitives are re-exported from familiar_primitives
            Some(emit_primitive_comment(region))
        }
        EmitStrategy::Generate | EmitStrategy::GenerateInSccGroup(_) => {
            Some(emit_type(region, ctx, profile))
        }
    }
}

// =============================================================================
// Type Emission
// =============================================================================

fn emit_type(region: &Region, ctx: &CodegenContext, profile: &RenderProfile) -> String {
    let mut output = String::new();
    
    // Add doc comment
    output.push_str(&format!("/// {}\n", region.canonical_name));
    
    // Add derives (could be configurable via profile in the future)
    output.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]\n");
    
    match &region.type_kind {
        TypeKind::Enum { variants } => {
            emit_enum(&mut output, region, variants, profile);
        }
        TypeKind::TaggedUnion { discriminator, tag_field, variants } => {
            emit_tagged_union(&mut output, region, discriminator, tag_field, variants, ctx, profile);
        }
        TypeKind::Struct { fields, flatten_refs } => {
            emit_struct(&mut output, region, fields, flatten_refs, ctx, profile);
        }
        TypeKind::Newtype { inner } => {
            emit_newtype(&mut output, region, inner, ctx, profile);
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

fn emit_enum(output: &mut String, region: &Region, variants: &[EnumVariant], profile: &RenderProfile) {
    // Add serde rename_all if configured
    if let Some(ref rename_all) = profile.unions.string_enum.serde_rename_all {
        output.push_str(&format!("#[serde(rename_all = \"{}\")]\n", rename_all));
    }
    
    output.push_str(&format!("pub enum {} {{\n", region.canonical_name));
    
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
    ctx: &CodegenContext,
    _profile: &RenderProfile,
) {
    // Determine tag attribute based on profile settings
    let tag = tag_field.as_ref()
        .or(discriminator.as_ref())
        .cloned()
        .unwrap_or_else(|| "type".to_string());
    
    output.push_str(&format!("#[serde(tag = \"{}\")]\n", tag));
    
    output.push_str(&format!("pub enum {} {{\n", region.canonical_name));
    
    for variant in variants {
        // If variant refs another schema, use that type
        if let Some(ref schema_ref) = variant.schema_ref {
            // Use name resolver to get canonical name
            let type_name = if let Some(resolved) = ctx.name_resolver().resolve_ref(schema_ref) {
                resolved.canonical_name.clone()
            } else {
                extract_type_name(schema_ref)
            };
            output.push_str(&format!("    {}({}),\n", variant.name, type_name));
        } else {
            // Unit variant
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
    ctx: &CodegenContext,
    profile: &RenderProfile,
) {
    output.push_str(&format!("pub struct {} {{\n", region.canonical_name));
    
    // Emit flattened refs first
    for ref_target in flatten_refs {
        let type_name = if let Some(resolved) = ctx.name_resolver().resolve_ref(ref_target) {
            resolved.canonical_name.clone()
        } else {
            extract_type_name(ref_target)
        };
        output.push_str("    #[serde(flatten)]\n");
        output.push_str(&format!("    pub {}: {},\n", to_snake_case(&type_name), type_name));
    }
    
    // Emit fields
    for field in fields {
        let needs_box = region.needs_boxing(&field.json_name);
        emit_field(output, field, needs_box, ctx, profile);
    }
    
    output.push_str("}\n");
}

fn emit_field(
    output: &mut String, 
    field: &FieldDef, 
    needs_box: bool,
    ctx: &CodegenContext,
    profile: &RenderProfile,
) {
    // Add serde rename if needed
    if field.needs_rename {
        output.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.json_name));
    }
    
    // Skip None if optional
    if !field.required {
        output.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
    }
    
    // Resolve type using context and profile
    let rust_type = ctx.resolve_field_type(&field.field_type, needs_box, profile);
    
    // Wrap in Option if not required
    let full_type = if field.required {
        rust_type
    } else {
        profile.wrap_optional(&rust_type)
    };
    
    // Escape field name if it's a keyword
    let field_name = profile.escape_keyword(&field.rust_name);
    
    output.push_str(&format!("    pub {}: {},\n", field_name, full_type));
}

// =============================================================================
// Newtype Emission
// =============================================================================

fn emit_newtype(
    output: &mut String, 
    region: &Region, 
    inner: &crate::graph::FieldType,
    ctx: &CodegenContext,
    profile: &RenderProfile,
) {
    let rust_type = ctx.resolve_field_type(inner, false, profile);
    output.push_str(&format!("pub struct {}(pub {});\n", region.canonical_name, rust_type));
}

// =============================================================================
// Alias Emission
// =============================================================================

fn emit_alias(output: &mut String, region: &Region, target: &str, ctx: &CodegenContext) {
    let target_name = if let Some(resolved) = ctx.name_resolver().resolve_ref(target) {
        resolved.canonical_name.clone()
    } else {
        extract_type_name(target)
    };
    output.push_str(&format!(
        "pub type {} = {};\n",
        region.canonical_name, target_name
    ));
}

fn emit_primitive_comment(region: &Region) -> String {
    format!(
        "// {} is a primitive - re-exported from familiar_primitives\n",
        region.canonical_name
    )
}

// =============================================================================
// Helper Utilities
// =============================================================================

/// Extract type name from a schema path/id (fallback when not in name resolver)
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
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("TenantId"), "tenant_id");
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        // All-caps is treated as individual letters
        assert_eq!(to_snake_case("API"), "api");
    }
    
    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("tenant_id"), "TenantId");
        assert_eq!(to_pascal_case("user-profile"), "UserProfile");
    }
}
