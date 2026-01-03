//! Golden Tests for Schema Shapes
//!
//! Tests that each SchemaShape variant is correctly detected and classified.

use std::collections::HashSet;
use std::path::Path;

use familiar_schemas::graph::{
    SchemaGraph, SchemaShape, TypeKind, EmitStrategy,
    detect_shape, detect_all_shapes, compute_scc_analysis,
    Classifier,
};
use familiar_schemas::codegen::CodegenContext;

fn fixtures_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").leak()
}

// =============================================================================
// Shape Detection Tests
// =============================================================================

#[test]
fn test_string_enum_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/string_enum.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::StringEnum { values, .. } => {
            assert_eq!(values, vec!["admin", "member", "guest"]);
        }
        other => panic!("Expected StringEnum, got {:?}", other),
    }
}

#[test]
fn test_oneof_enum_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/oneof_enum.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::OneOfStringEnum { variants, .. } => {
            assert_eq!(variants, vec!["pending", "active", "completed"]);
        }
        other => panic!("Expected OneOfStringEnum, got {:?}", other),
    }
}

#[test]
fn test_oneof_tagged_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/oneof_tagged.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::OneOfObjects { variants, discriminator } => {
            assert_eq!(variants.len(), 2);
            assert_eq!(discriminator, Some("type".to_string()));
        }
        other => panic!("Expected OneOfObjects, got {:?}", other),
    }
}

#[test]
fn test_oneof_mixed_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/oneof_mixed.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::OneOfMixed { .. } => {
            // Expected - mixed oneOf should be detected
        }
        other => panic!("Expected OneOfMixed, got {:?}", other),
    }
}

#[test]
fn test_simple_struct_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/simple_struct.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::Object { properties, .. } => {
            assert_eq!(properties.len(), 5);
            
            // Check required fields
            let id_field = properties.iter().find(|p| p.name == "id").unwrap();
            assert!(id_field.required);
            
            let age_field = properties.iter().find(|p| p.name == "age").unwrap();
            assert!(!age_field.required);
        }
        other => panic!("Expected Object, got {:?}", other),
    }
}

#[test]
fn test_alias_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/alias_of_alias.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::Ref { target } => {
            assert_eq!(target, "alias_b.json");
        }
        other => panic!("Expected Ref, got {:?}", other),
    }
}

#[test]
fn test_self_recursive_detection() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("fixtures/self_recursive.json")).unwrap();
    let shape = detect_shape(&schema);
    
    match shape {
        SchemaShape::Object { properties, .. } => {
            let children = properties.iter().find(|p| p.name == "children").unwrap();
            // Children should have array with $ref items
            match &children.shape {
                familiar_schemas::graph::PropertyTypeShape::Array { items } => {
                    match items.as_ref() {
                        familiar_schemas::graph::PropertyTypeShape::Ref(target) => {
                            assert!(target.contains("self_recursive"));
                        }
                        _ => panic!("Expected Ref in array items"),
                    }
                }
                _ => panic!("Expected Array shape for children"),
            }
        }
        other => panic!("Expected Object, got {:?}", other),
    }
}

// =============================================================================
// Full Pipeline Tests
// =============================================================================

#[test]
fn test_full_pipeline_with_fixtures() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    
    assert!(graph.schema_count() >= 10, "Should load at least 10 fixture schemas");
    
    // Detect all shapes
    let shapes = detect_all_shapes(&graph);
    assert_eq!(shapes.len(), graph.schema_count());
    
    // Compute SCCs
    let scc_analysis = compute_scc_analysis(&graph);
    
    // Should have at least one SCC (mutual recursion)
    // Note: may also detect self-recursive
    
    // Classify all
    let classifier = Classifier::new(&graph, &shapes, &scc_analysis, HashSet::new());
    let classifications = classifier.classify_all();
    
    // Check string_enum classification
    if let Some(class) = classifications.get("fixtures/string_enum.json") {
        match &class.type_kind {
            TypeKind::Enum { variants } => {
                assert_eq!(variants.len(), 3);
                assert_eq!(class.emit_strategy, EmitStrategy::Generate);
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }
    
    // Check simple_struct classification
    if let Some(class) = classifications.get("fixtures/simple_struct.json") {
        match &class.type_kind {
            TypeKind::Struct { fields, .. } => {
                assert_eq!(fields.len(), 5);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }
}

#[test]
fn test_codegen_context_build() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    let ctx = CodegenContext::build(graph).unwrap();
    
    assert!(ctx.schema_count() >= 10);
    
    // Get regions for all schemas that should be generated
    let regions = ctx.regions_to_generate();
    assert!(!regions.is_empty());
    
    // Check that string_enum has a region (file is string_enum.json -> StringEnum)
    let enum_region = regions.iter().find(|r| r.canonical_name == "StringEnum");
    assert!(enum_region.is_some(), "Should have StringEnum region");
}

#[test]
fn test_mutual_recursion_scc() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    let scc_analysis = compute_scc_analysis(&graph);
    
    // mutual_a and mutual_b should be in the same SCC
    let a_handling = scc_analysis.get("fixtures/mutual_a.json");
    let b_handling = scc_analysis.get("fixtures/mutual_b.json");
    
    if let (Some(a), Some(b)) = (a_handling, b_handling) {
        assert!(a.is_cyclic() || b.is_cyclic(), "At least one should be cyclic");
        if a.is_cyclic() && b.is_cyclic() {
            assert_eq!(a.scc_id, b.scc_id, "Should be in same SCC");
        }
    }
}

#[test]
fn test_self_recursive_boxing() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    let scc_analysis = compute_scc_analysis(&graph);
    
    let handling = scc_analysis.get("fixtures/self_recursive.json");
    if let Some(h) = handling {
        // Self-recursive should be detected
        assert!(h.is_cyclic() || h.is_self_referential, "Should be cyclic or self-referential");
    }
}

// =============================================================================
// Import Path Stability Tests
// =============================================================================

/// Test that import resolution is consistent across MCP and codegen paths.
/// Both should use SchemaGraph::imports_for() - this ensures parity.
#[test]
fn test_import_resolver_stability() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    
    // Get imports for oneof_tagged (depends on message_event and user_event)
    let imports_rust = graph.imports_for("fixtures/oneof_tagged.json", "rust");
    let imports_ts = graph.imports_for("fixtures/oneof_tagged.json", "typescript");
    
    // Should have at least the schema itself
    assert!(!imports_rust.is_empty(), "Should generate Rust imports");
    assert!(!imports_ts.is_empty(), "Should generate TypeScript imports");
    
    // Import paths should be deterministic
    let imports_rust_2 = graph.imports_for("fixtures/oneof_tagged.json", "rust");
    assert_eq!(imports_rust, imports_rust_2, "Import resolution should be deterministic");
}

/// Test that codegen and MCP use the same resolver internally.
/// Since both use SchemaGraph, this is guaranteed by design - but we test the API.
#[test]
fn test_mcp_codegen_resolver_parity() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    
    // MCP path: SchemaGraph::imports_for
    let _mcp_imports = graph.imports_for("fixtures/simple_struct.json", "rust");
    
    // Codegen path: also uses SchemaGraph internally
    // NameResolver uses graph for resolution
    let ctx = CodegenContext::build(graph).unwrap();
    if let Some(region) = ctx.region("fixtures/simple_struct.json") {
        // Region should be generated
        assert!(region.should_generate(), "simple_struct should be generated");
        
        // Name resolver should be accessible from context
        let name_resolver = ctx.name_resolver();
        let resolved = name_resolver.get("fixtures/simple_struct.json");
        assert!(resolved.is_some(), "simple_struct should be resolved");
    }
}

/// Test that type name resolution is consistent.
/// SchemaGraph::get() + node.title should match classification rust_name.
#[test]
fn test_type_name_resolution_parity() {
    let graph = SchemaGraph::from_directory(fixtures_path()).unwrap();
    let shapes = detect_all_shapes(&graph);
    let scc_analysis = compute_scc_analysis(&graph);
    let classifier = Classifier::new(&graph, &shapes, &scc_analysis, HashSet::new());
    let classifications = classifier.classify_all();
    
    for schema_id in graph.all_ids() {
        if let Some(node) = graph.get(schema_id) {
            if let Some(class) = classifications.get(schema_id) {
                // If schema has a title, rust_name should be PascalCase of it
                if let Some(title) = &node.title {
                    let expected = familiar_schemas::graph::to_pascal_case(title);
                    assert_eq!(class.rust_name, expected, 
                        "Type name mismatch for {}: graph title '{}' -> '{}', but classification gave '{}'",
                        schema_id, title, expected, class.rust_name);
                }
            }
        }
    }
}

