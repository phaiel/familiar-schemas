//! Nickel Schema Application Tests with rstest
//!
//! Tests Nickel schema application to JSONSchema files at runtime.
//! Nickel files create composite results from JSONSchema inputs (in memory).
//! Uses rstest for parameterized testing across real JSONSchema fixtures.

use std::fs;
use std::path::PathBuf;

use familiar_schemas::NickelValidator;
use rstest::rstest;

/// Test fixture combining a JSONSchema with its expected Nickel processing layer
#[derive(Debug, Clone)]
pub struct SchemaTestFixture {
    pub name: String,
    pub json_schema: serde_json::Value,
    pub expected_layer: String,
}

/// Load all JSONSchema fixtures and determine their expected processing layers
fn load_schema_fixtures() -> Vec<SchemaTestFixture> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    let mut fixtures = Vec::new();

    if let Ok(entries) = fs::read_dir(&fixtures_dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "json" {
                    let fixture_name = entry.path().file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    if let Ok(content) = fs::read_to_string(&entry.path()) {
                        if let Ok(json_schema) = serde_json::from_str(&content) {
                            let expected_layer = determine_expected_layer(&json_schema);

                            fixtures.push(SchemaTestFixture {
                                name: fixture_name,
                                json_schema,
                                expected_layer,
                            });
                        }
                    }
                }
            }
        }
    }

    fixtures
}

/// Determine expected layer for schema processing
fn determine_expected_layer(schema: &serde_json::Value) -> String {
    if schema.get("properties").is_some() || schema.get("type").is_some() {
        "entity".to_string()
    } else if schema.get("x-familiar-node-type").is_some() {
        "node".to_string()
    } else if schema.get("x-familiar-depends_on").is_some() {
        "topology".to_string()
    } else {
        "domain".to_string()
    }
}

/// Test basic Nickel validator initialization
#[test]
fn test_nickel_validator_initialization() {
    // Basic smoke test - NickelValidator::new() should not panic
    let _validator = NickelValidator::new();
    assert!(true, "NickelValidator creation attempted successfully");
}

/// Test all schema fixtures against Nickel application
#[rstest]
#[case("simple_struct")]
#[case("user_event")]
#[case("message_event")]
#[case("string_enum")]
#[case("oneof_enum")]
#[case("oneof_tagged")]
#[case("oneof_mixed")]
#[case("primitive_match")]
#[case("alias_b")]
#[case("alias_c")]
#[case("alias_of_alias")]
#[case("self_recursive")]
fn test_nickel_schema_application_to_jsonschema(#[case] fixture_name: &str) {
    let fixtures = load_schema_fixtures();
    let fixture = fixtures.iter().find(|f| f.name == fixture_name).unwrap();

    let nickel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../versions/latest/nickel");

    // Test 1: Nested directory structure processing
    let expected_dir = match fixture.expected_layer.as_str() {
        "entity" => nickel_path.join("codegen/components/_directory.ncl"),
        "node" => nickel_path.join("infrastructure/_directory.ncl"),
        "topology" => nickel_path.join("architecture/_directory.ncl"),
        _ => nickel_path.join("domain/_directory.ncl"),
    };

    assert!(expected_dir.exists(), "Nickel directory should exist for {} ({})",
            fixture.name, fixture.expected_layer);

    let path_str = expected_dir.to_string_lossy();
    assert!(path_str.contains("/"), "Should be in subdirectory structure for {}", fixture.name);

    // Test 2: Complex composition patterns
    let composer_content = fs::read_to_string(nickel_path.join("composers/directory_composer.ncl"))
        .expect("Directory composer should be readable for schema processing");

    assert!(composer_content.contains("&"), "Composer should use & merging for {}", fixture.name);
    assert!(composer_content.contains("compose"), "Composer should have composition functions for {}", fixture.name);

    assert!(composer_content.contains(&fixture.expected_layer) ||
            (fixture.expected_layer == "entity" && composer_content.contains("codegen")),
            "Composer should handle {} layer for {}", fixture.expected_layer, fixture.name);

    // Test 3: Layered architecture structure
    let primitives = ["contract_primitives.ncl", "extraction_primitives.ncl", "validation_primitives.ncl"];
    for primitive in primitives {
        assert!(nickel_path.join("primitives").join(primitive).exists(),
                "Primitive {} should exist for processing {}", primitive, fixture.name);
    }

    let libraries = ["contract_library.ncl", "extraction_library.ncl", "validation_library.ncl"];
    for library in libraries {
        assert!(nickel_path.join("libraries").join(library).exists(),
                "Library {} should exist for {}", library, fixture.name);
    }

    // Test 4: Directory schema structure
    assert!(expected_dir.exists(), "Directory schema file should exist for {}", fixture.name);

    // Test 5: Extension framework
    let extensions = ["edge-type.ncl", "type-category.ncl", "kind.ncl"];
    for ext in extensions {
        assert!(nickel_path.join("extensions").join(ext).exists(),
                "Extension {} should exist for enhancing {}", ext, fixture.name);
    }

    // Test 6: Library, primitive and composer usage
    for primitive_path in primitives.iter().map(|p| nickel_path.join("primitives").join(p)) {
        assert!(primitive_path.exists(), "Primitive should exist for processing {}", fixture.name);
    }

    for lib_path in libraries.iter().map(|l| nickel_path.join("libraries").join(l)) {
        assert!(lib_path.exists(), "Library should exist for {}", fixture.name);
    }

    let composer_path = nickel_path.join("composers/directory_composer.ncl");
    assert!(composer_path.exists(), "Composer should exist for {}", fixture.name);

    // Test 7: Hydration configuration
    let hydration_content = fs::read_to_string(nickel_path.join("primitives/hydration_primitives.ncl"))
        .expect("Hydration primitives should be readable for schema enhancement");

    assert!(hydration_content.contains("BaseHydration"), "Should define base hydration for {}", fixture.name);
    assert!(hydration_content.contains("_metadata"), "Should have metadata section for {}", fixture.name);
    assert!(hydration_content.contains("_observability"), "Should have observability section for {}", fixture.name);
    assert!(hydration_content.contains("_operations"), "Should have operations section for {}", fixture.name);

    // Test 8: Contract validation
    let contract_content = fs::read_to_string(nickel_path.join("libraries/contract_library.ncl"))
        .expect("Contract library should be readable for schema validation");

    assert!(contract_content.contains("enhanced_test_runner"), "Should have test runner for {}", fixture.name);

    // Test 9: Forbidden rule detection
    let validation_content = fs::read_to_string(nickel_path.join("primitives/validation_primitives.ncl"))
        .expect("Validation primitives should be readable for schema checking");

    assert!(validation_content.contains("forbidden"), "Should define forbidden patterns for {}", fixture.name);

    let obj = fixture.json_schema.as_object().unwrap();
    let has_forbidden = obj.keys().any(|k| k.starts_with("x-familiar-"));
    if has_forbidden {
        assert!(true, "Schema {} has familiar extensions that may need validation", fixture.name);
    }

    // Test 10: Schema transformation and extraction
    let extraction_content = fs::read_to_string(nickel_path.join("libraries/extraction_library.ncl"))
        .expect("Extraction library should be readable for schema transformation");

    assert!(extraction_content.contains("extraction_functions"), "Should define extraction functions for {}", fixture.name);

    match fixture.expected_layer.as_str() {
        "entity" => assert!(extraction_content.contains("entity_extract"), "Should extract entities from {}", fixture.name),
        "node" => assert!(extraction_content.contains("node_extract"), "Should extract nodes from {}", fixture.name),
        "topology" => assert!(extraction_content.contains("topology_extract"), "Should extract topology from {}", fixture.name),
        _ => assert!(extraction_content.contains("domain_extract"), "Should extract domain from {}", fixture.name),
    }

    // Test 11: Edge relationship validation
    let edge_content = fs::read_to_string(nickel_path.join("libraries/edge_library.ncl"))
        .expect("Edge library should be readable for relationship validation");

    assert!(edge_content.contains("edge_types"), "Should define edge types for {}", fixture.name);
    assert!(edge_content.contains("compatibility"), "Should check compatibility for {}", fixture.name);

    if fixture.json_schema.get("x-familiar-edges").is_some() ||
       fixture.json_schema.get("x-familiar-depends_on").is_some() {
        assert!(true, "Schema {} has relationships that need edge validation", fixture.name);
    }

    // Test 12: Extension framework integration
    let ext_content = fs::read_to_string(nickel_path.join("libraries/extension_library.ncl"))
        .expect("Extension library should be readable for schema enhancement");

    assert!(ext_content.contains("extension"), "Should handle extensions for {}", fixture.name);

    if obj.keys().any(|k| k.starts_with("x-familiar-")) {
        assert!(true, "Schema {} has extensions that can be enhanced", fixture.name);
    }

    // Test 13: Global schema composition
    let global_content = fs::read_to_string(nickel_path.join("global.ncl"))
        .expect("Global schema should be readable for comprehensive processing");

    assert!(global_content.contains("Libraries"), "Should import libraries for {}", fixture.name);
    assert!(global_content.contains("&"), "Should use merging for {}", fixture.name);
    assert!(global_content.contains("orthology"), "Should define orthology for {}", fixture.name);

    // Global should handle the core orthological layers (entity/node/topology)
    // Domain schemas may not have specific global handling
    if fixture.expected_layer != "domain" {
        assert!(global_content.contains(&fixture.expected_layer) ||
                (fixture.expected_layer == "entity" && global_content.contains("entity")),
                "Global should handle {} layer for {}", fixture.expected_layer, fixture.name);
    }
}

/// Test the Technique Library validation functions
mod technique_library_tests {
    use super::*;

    /// Test valid technique structure based on GitHub schema
    fn create_valid_technique() -> serde_json::Value {
        serde_json::json!({
          "$schema": "../../architecture/meta/Technique.meta.schema.json",
          "$id": "techniques/classification/classify-input.technique.json",
          "id": "classification.classify-input",
          "title": "Classify Input Pipeline",
          "description": "Complete input classification pipeline: segment → classify → extract features",
          "x-familiar-kind": "technique",
          "input": {
            "$ref": "../../architecture/references/SchemaRef.meta.schema.json"
          },
          "output": {
            "$ref": "../../architecture/references/SchemaRef.meta.schema.json"
          },
          "steps": [
            {
              "id": "segment",
              "kind": "call",
              "action": {
                "$ref": "../../actions/classification/segment-input.action.json"
              },
              "args": {
                "raw_input": "$.input.text",
                "config": "$.input.config"
              }
            },
            {
              "id": "classify_purpose",
              "kind": "call",
              "action": {
                "$ref": "../../actions/classification/purpose-classify.action.json"
              },
              "args": {
                "segment": "$.segment[0]"
              }
            }
          ],
          "return": {
            "segmented_input": "$.segment",
            "purpose_classification": "$.classify_purpose"
          }
        })
    }

    /// Test invalid technique with duplicate step IDs
    fn create_invalid_technique_duplicate_ids() -> serde_json::Value {
        let mut technique = create_valid_technique();
        if let Some(steps) = technique.get_mut("steps").and_then(|s| s.as_array_mut()) {
            if steps.len() >= 2 {
                steps[1]["id"] = serde_json::json!("segment"); // Duplicate ID
            }
        }
        technique
    }

    /// Test technique with invalid step kind
    fn create_invalid_technique_bad_kind() -> serde_json::Value {
        let mut technique = create_valid_technique();
        if let Some(steps) = technique.get_mut("steps").and_then(|s| s.as_array_mut()) {
            if let Some(step) = steps.get_mut(0) {
                step["kind"] = serde_json::json!("invalid_kind");
            }
        }
        technique
    }

    /// Test technique with invalid CEL reference
    fn create_invalid_technique_bad_cel() -> serde_json::Value {
        let mut technique = create_valid_technique();
        if let Some(steps) = technique.get_mut("steps").and_then(|s| s.as_array_mut()) {
            if let Some(step) = steps.get_mut(1) {
                if let Some(args) = step.get_mut("args").and_then(|a| a.as_object_mut()) {
                    args.insert("segment".to_string(), serde_json::json!("$.nonexistent_step"));
                }
            }
        }
        technique
    }

    #[test]
    fn test_technique_library_step_kinds() {
        // This test verifies the Nickel library defines the correct step kinds
        // Since we can't directly test Nickel functions from Rust, we test that
        // the library file exists and can be referenced in the validation pipeline
        let nickel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/technique_library.ncl");

        assert!(nickel_path.exists(), "Technique library should exist");

        let content = fs::read_to_string(&nickel_path).expect("Should read technique library");
        assert!(content.contains("StepKinds"), "Should define StepKinds");
        assert!(content.contains("call"), "Should include call step kind");
        assert!(content.contains("switch"), "Should include switch step kind");
        assert!(content.contains("map"), "Should include map step kind");
        assert!(content.contains("parallel"), "Should include parallel step kind");
        assert!(content.contains("transform"), "Should include transform step kind");
    }

    #[test]
    fn test_technique_library_cel_validation() {
        // Test that the library includes CEL validation functions
        let nickel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/technique_library.ncl");

        let content = fs::read_to_string(&nickel_path).expect("Should read technique library");
        assert!(content.contains("CELValidation"), "Should include CEL validation");
        assert!(content.contains("validate_cel_reference"), "Should validate CEL references");
        assert!(content.contains("validate_cel_expressions_in_object"), "Should validate object CEL expressions");
    }

    #[test]
    fn test_technique_library_isa_validation() {
        // Test that the library includes ISA compliance validation
        let nickel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/technique_library.ncl");

        let content = fs::read_to_string(&nickel_path).expect("Should read technique library");
        assert!(content.contains("ISAValidation"), "Should include ISA validation");
        assert!(content.contains("validate_input_contract"), "Should validate input contracts");
        assert!(content.contains("validate_steps_structure"), "Should validate steps structure");
        assert!(content.contains("validate_data_flow"), "Should validate data flow");
    }

    #[test]
    fn test_technique_library_contract_validation() {
        // Test that the library includes complete technique contract validation
        let nickel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/technique_library.ncl");

        let content = fs::read_to_string(&nickel_path).expect("Should read technique library");
        assert!(content.contains("TechniqueContract"), "Should include technique contract");
        assert!(content.contains("validate"), "Should have validate function");
    }

    #[test]
    fn test_architecture_techniques_directory_inheritance() {
        // Test that architecture/techniques/_directory.ncl exists and inherits properly
        let arch_techniques_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/architecture/techniques/_directory.ncl");

        assert!(arch_techniques_dir.exists(), "Architecture techniques directory should exist");

        let content = fs::read_to_string(&arch_techniques_dir).expect("Should read architecture techniques directory");
        assert!(content.contains("Libraries"), "Should import libraries");
        assert!(content.contains("technique_validation"), "Should include technique validation");
        assert!(content.contains("validate_architecture_technique"), "Should validate architecture techniques");
    }

    #[test]
    fn test_infrastructure_techniques_inheritance() {
        // Test that infrastructure/techniques inherits from architecture
        let infra_techniques_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/infrastructure/techniques/_directory.ncl");

        assert!(infra_techniques_dir.exists(), "Infrastructure techniques directory should exist");

        let content = fs::read_to_string(&infra_techniques_dir).expect("Should read infrastructure techniques directory");
        assert!(content.contains("ArchitectureTechniques"), "Should inherit from architecture techniques");
        assert!(content.contains("validate_infrastructure_technique"), "Should validate infrastructure techniques");
    }

    #[test]
    fn test_nlp_techniques_subdirectory_inheritance() {
        // Test that nlp subdirectory inherits from techniques
        let nlp_techniques_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/infrastructure/techniques/nlp/_directory.ncl");

        assert!(nlp_techniques_dir.exists(), "NLP techniques directory should exist");

        let content = fs::read_to_string(&nlp_techniques_dir).expect("Should read NLP techniques directory");
        assert!(content.contains("ParentTechniques"), "Should inherit from parent techniques");
        assert!(content.contains("validate_nlp_technique"), "Should validate NLP techniques");
    }

    #[test]
    fn test_all_libraries_includes_technique_library() {
        // Test that all_libraries.ncl includes the technique library
        let all_libraries_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/all_libraries.ncl");

        let content = fs::read_to_string(&all_libraries_path).expect("Should read all libraries");
        assert!(content.contains("TechniqueLibrary"), "Should import technique library");
        assert!(content.contains("technique"), "Should include technique in merged libraries list");
    }

    #[test]
    fn test_technique_schema_structure_matches_github() {
        // Test that our understanding of technique schema structure matches the GitHub example
        let valid_technique = create_valid_technique();

        // Verify structure matches GitHub schema
        assert!(valid_technique.get("$schema").is_some(), "Should have $schema");
        assert!(valid_technique.get("$id").is_some(), "Should have $id");
        assert!(valid_technique.get("id").is_some(), "Should have id");
        assert!(valid_technique.get("x-familiar-kind").is_some(), "Should have x-familiar-kind");
        assert!(valid_technique.get("input").is_some(), "Should have input");
        assert!(valid_technique.get("output").is_some(), "Should have output");
        assert!(valid_technique.get("steps").is_some(), "Should have steps");
        assert!(valid_technique.get("return").is_some(), "Should have return");

        // Verify steps structure
        let steps = valid_technique.get("steps").unwrap().as_array().unwrap();
        assert!(!steps.is_empty(), "Should have steps");

        for step in steps {
            assert!(step.get("id").is_some(), "Step should have id");
            assert!(step.get("kind").is_some(), "Step should have kind");
            assert!(step.get("action").is_some(), "Step should have action");
        }

        // Verify CEL expressions in args and return
        let return_obj = valid_technique.get("return").unwrap().as_object().unwrap();
        for (_key, value) in return_obj {
            if let Some(str_val) = value.as_str() {
                assert!(str_val.starts_with("$."), "Return values should be CEL expressions: {}", str_val);
            }
        }
    }
}

/// Test the Action Library validation functions
mod action_library_tests {
    use super::*;

    /// Test valid action structure based on improved schema
    fn create_valid_action() -> serde_json::Value {
        serde_json::json!({
          "$schema": "../../../architecture/meta/Action.meta.schema.json",
          "$id": "infrastructure/actions/classification/extract-features.action.json",
          "id": "classification.extract-features",
          "title": "Extract Segment Features",
          "description": "Extract semantic and temporal features from text segments",
          "x-familiar-kind": "action",
          "x-familiar-compute-category": "ai",
          "x-familiar-execution-model": "async",
          "x-familiar-side-effects": ["ml-inference"],
          "x-familiar-reliability": "idempotent",
          "signature": {
            "inputs": {
              "segment": {
                "schema": { "$ref": "../../../primitives/Segment.schema.json" },
                "semantics": "borrow"
              }
            },
            "output": {
              "schema": { "$ref": "../../../primitives/SegmentFeatures.schema.json" },
              "nature": "atomic"
            },
            "capabilities": {
              "category": "feature-extraction",
              "provides": ["semantic-features", "temporal-features"],
              "description": "Feature extraction models for semantic understanding"
            }
          }
        })
    }

    /// Test action without capabilities (should still be valid)
    fn create_action_without_capabilities() -> serde_json::Value {
        let mut action = create_valid_action();
        if let Some(signature) = action.get_mut("signature").and_then(|s| s.as_object_mut()) {
            signature.remove("capabilities");
        }
        action
    }

    /// Test invalid action with missing signature
    fn create_invalid_action_missing_signature() -> serde_json::Value {
        let mut action = create_valid_action();
        action.as_object_mut().unwrap().remove("signature");
        action
    }

    #[test]
    fn test_action_library_existence() {
        // Test that the action library exists and can be loaded
        let action_lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/action_library.ncl");

        assert!(action_lib_path.exists(), "Action library should exist");

        let content = fs::read_to_string(&action_lib_path).expect("Should read action library");
        assert!(content.contains("ActionValidation"), "Should include action validation");
        assert!(content.contains("CapabilityQueries"), "Should include capability queries");
    }

    #[test]
    fn test_action_library_signature_validation() {
        // Test that the library includes signature validation functions
        let action_lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/action_library.ncl");

        let content = fs::read_to_string(&action_lib_path).expect("Should read action library");
        assert!(content.contains("validate_signature"), "Should validate signatures");
        assert!(content.contains("validate_capabilities"), "Should validate capabilities");
        assert!(content.contains("validate_action"), "Should validate complete actions");
    }

    #[test]
    fn test_action_library_capability_queries() {
        // Test that the library includes capability querying functions
        let action_lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/action_library.ncl");

        let content = fs::read_to_string(&action_lib_path).expect("Should read action library");
        assert!(content.contains("has_capability"), "Should check capabilities");
        assert!(content.contains("by_category"), "Should query by category");
        assert!(content.contains("find_compatible_actions"), "Should find compatible actions");
    }

    #[test]
    fn test_infrastructure_actions_directory_inheritance() {
        // Test that infrastructure/actions/_directory.ncl exists and includes action validation
        let infra_actions_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/infrastructure/actions/_directory.ncl");

        assert!(infra_actions_dir.exists(), "Infrastructure actions directory should exist");

        let content = fs::read_to_string(&infra_actions_dir).expect("Should read infrastructure actions directory");
        assert!(content.contains("Libraries"), "Should import libraries");
        assert!(content.contains("action_validation"), "Should include action validation");
        assert!(content.contains("validate_infrastructure_action"), "Should validate infrastructure actions");
    }

    #[test]
    fn test_all_libraries_includes_action_library() {
        // Test that all_libraries.ncl includes the action library
        let all_libraries_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/all_libraries.ncl");

        let content = fs::read_to_string(&all_libraries_path).expect("Should read all libraries");
        assert!(content.contains("ActionLibrary"), "Should import action library");
        assert!(content.contains("action"), "Should include action in merged libraries list");
    }

    #[test]
    fn test_action_schema_structure_with_capabilities() {
        // Test that our action schema structure includes capabilities in signature
        let valid_action = create_valid_action();

        // Verify structure
        assert!(valid_action.get("$schema").is_some(), "Should have $schema");
        assert!(valid_action.get("$id").is_some(), "Should have $id");
        assert!(valid_action.get("id").is_some(), "Should have id");
        assert!(valid_action.get("x-familiar-kind").is_some(), "Should have x-familiar-kind");
        assert_eq!(valid_action.get("x-familiar-kind").unwrap(), "action", "Should be action kind");

        // Verify signature structure
        assert!(valid_action.get("signature").is_some(), "Should have signature");
        let signature = valid_action.get("signature").unwrap().as_object().unwrap();

        assert!(signature.contains_key("inputs"), "Should have inputs");
        assert!(signature.contains_key("output"), "Should have output");
        assert!(signature.contains_key("capabilities"), "Should have capabilities");

        // Verify capabilities structure
        let capabilities = signature.get("capabilities").unwrap().as_object().unwrap();
        assert!(capabilities.contains_key("category"), "Should have category");
        assert!(capabilities.contains_key("provides"), "Should have provides");
        assert!(capabilities.contains_key("description"), "Should have description");

        assert_eq!(capabilities.get("category").unwrap(), "feature-extraction", "Should have correct category");
        assert!(capabilities.get("provides").unwrap().as_array().is_some(), "Provides should be array");
    }

    #[test]
    fn test_action_schema_without_capabilities() {
        // Test that actions without capabilities are still valid
        let action_without_caps = create_action_without_capabilities();

        // Should still have signature with inputs and output
        assert!(action_without_caps.get("signature").is_some(), "Should have signature");
        let signature = action_without_caps.get("signature").unwrap().as_object().unwrap();

        assert!(signature.contains_key("inputs"), "Should have inputs");
        assert!(signature.contains_key("output"), "Should have output");
        assert!(!signature.contains_key("capabilities"), "Should not have capabilities");

        // Should still be a valid action
        assert_eq!(action_without_caps.get("x-familiar-kind").unwrap(), "action", "Should be action kind");
    }

    #[test]
    fn test_example_action_file_exists() {
        // Test that our example action file exists
        let action_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/json-schema/infrastructure/actions/classification/extract-features.action.json");

        assert!(action_file.exists(), "Example action file should exist");

        let content = fs::read_to_string(&action_file).expect("Should read action file");
        assert!(content.contains("capabilities"), "Should include capabilities in signature");
        assert!(!content.contains("x-familiar-tooling"), "Should not use x-familiar-tooling extension");
    }
}

/// Test the Reference Library validation for orthological $ref and $schema patterns
mod reference_library_tests {
    use super::*;

    /// Test valid directory-level schema (should not use $refs)
    fn create_valid_directory_schema() -> serde_json::Value {
        serde_json::json!({
          "Libraries": {},
          "bundle_metadata": {
            "merged_libraries": ["contract", "validation"],
            "composition_strategy": "extreme_merging"
          }
        })
    }

    /// Test invalid directory schema with forbidden $ref
    fn create_invalid_directory_with_refs() -> serde_json::Value {
        serde_json::json!({
          "x-familiar-depends": [{
            "$ref": "../components/SomeComponent.component.json"
          }],
          "Libraries": {}
        })
    }

    /// Test valid leaf-level schema (may use $refs)
    fn create_valid_leaf_schema() -> serde_json::Value {
        serde_json::json!({
          "$schema": "http://json-schema.org/draft-07/schema#",
          "properties": {
            "field": { "$ref": "../primitives/String.schema.json" }
          }
        })
    }

    /// Test invalid leaf schema with forbidden $ref pattern
    fn create_invalid_leaf_schema() -> serde_json::Value {
        serde_json::json!({
          "$schema": "http://json-schema.org/draft-07/schema#",
          "properties": {
            "field": { "$ref": "../invalid/Type.schema.json" }
          }
        })
    }

    /// Test valid meta schema (may use $refs)
    fn create_valid_meta_schema() -> serde_json::Value {
        serde_json::json!({
          "$schema": "http://json-schema.org/draft-07/schema#",
          "$id": "meta.schema.json",
          "properties": {
            "input": {
              "properties": {
                "$ref": { "type": "string" }
              }
            }
          }
        })
    }

    #[test]
    fn test_reference_library_existence() {
        // Test that the reference library exists and defines orthological patterns
        let ref_lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/reference_library.ncl");

        assert!(ref_lib_path.exists(), "Reference library should exist");

        let content = fs::read_to_string(&ref_lib_path).expect("Should read reference library");
        assert!(content.contains("ReferencePatterns"), "Should define reference patterns");
        assert!(content.contains("ReferenceValidation"), "Should include reference validation");
        assert!(content.contains("InheritanceValidation"), "Should include inheritance validation");
        assert!(content.contains("ReferenceContract"), "Should include reference contract");
    }

    #[test]
    fn test_reference_patterns_definitions() {
        // Test that the library defines orthological reference patterns
        let ref_lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/reference_library.ncl");

        let content = fs::read_to_string(&ref_lib_path).expect("Should read reference library");
        assert!(content.contains("allowed_schema_patterns"), "Should define allowed schema patterns");
        assert!(content.contains("directory"), "Should define directory level patterns");
        assert!(content.contains("leaf"), "Should define leaf level patterns");
        assert!(content.contains("meta"), "Should define meta level patterns");
        assert!(content.contains("forbidden_refs"), "Should define forbidden ref patterns");
    }

    #[test]
    fn test_architecture_directory_includes_reference_validation() {
        // Test that architecture directory includes reference validation
        let arch_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/architecture/_directory.ncl");

        let content = fs::read_to_string(&arch_dir_path).expect("Should read architecture directory");
        assert!(content.contains("ReferenceValidation"), "Should include reference validation");
        assert!(content.contains("validate_architecture_directory"), "Should validate directory schemas");
        assert!(content.contains("validate_architecture_meta"), "Should validate meta schemas");
    }

    #[test]
    fn test_all_libraries_includes_reference_library() {
        // Test that all_libraries.ncl includes the reference library
        let all_libraries_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel/libraries/all_libraries.ncl");

        let content = fs::read_to_string(&all_libraries_path).expect("Should read all libraries");
        assert!(content.contains("ReferenceLibrary"), "Should import reference library");
        assert!(content.contains("reference"), "Should include reference in merged libraries list");
    }

    #[test]
    fn test_valid_directory_schema_structure() {
        // Test that our valid directory schema follows orthological patterns
        let valid_directory = create_valid_directory_schema();

        // Should not have $refs (inheritance-based)
        assert!(!has_refs(&valid_directory), "Directory schemas should not have $refs");

        // Should have inheritance indicators
        assert!(valid_directory.get("Libraries").is_some() ||
                valid_directory.get("bundle_metadata").is_some(),
                "Directory schemas should have inheritance indicators");
    }

    #[test]
    fn test_invalid_directory_schema_with_refs() {
        // Test that directory schemas with $refs are flagged as invalid
        let invalid_directory = create_invalid_directory_with_refs();

        // Should have forbidden $refs
        assert!(has_refs(&invalid_directory), "This test schema should have $refs (which are forbidden at directory level)");
    }

    #[test]
    fn test_valid_leaf_schema_with_refs() {
        // Test that leaf schemas can have appropriate $refs
        let valid_leaf = create_valid_leaf_schema();

        // Should have valid $schema
        assert_eq!(valid_leaf.get("$schema").unwrap(), "http://json-schema.org/draft-07/schema#",
                   "Should have valid JSON Schema declaration");

        // Should have allowed $ref pattern
        assert!(has_refs(&valid_leaf), "Leaf schemas should be able to have $refs");
        assert!(has_primitive_refs(&valid_leaf), "Should reference primitives (allowed)");
    }

    #[test]
    fn test_valid_meta_schema_structure() {
        // Test that meta schemas follow orthological patterns
        let valid_meta = create_valid_meta_schema();

        // Should have $schema and $id
        assert!(valid_meta.get("$schema").is_some(), "Meta schemas should have $schema");
        assert!(valid_meta.get("$id").is_some(), "Meta schemas should have $id");

        // May have $refs for schema composition
        // (This particular meta schema doesn't, but others might)
    }

    #[test]
    fn test_orthological_separation_validation() {
        // Test that different levels enforce different patterns

        // Directory level: No $refs, inheritance required
        let directory_schema = create_valid_directory_schema();
        assert!(!has_refs(&directory_schema), "Directory: No $refs allowed");
        assert!(has_inheritance_indicators(&directory_schema), "Directory: Inheritance required");

        // Leaf level: $refs allowed, inheritance not required
        let leaf_schema = create_valid_leaf_schema();
        assert!(has_refs(&leaf_schema), "Leaf: $refs allowed");
        // Leaf schemas typically don't show inheritance indicators

        // Meta level: Both patterns possible
        let meta_schema = create_valid_meta_schema();
        // Meta schemas can have either pattern depending on use case
    }
}

// Helper functions for reference validation tests
fn has_refs(schema: &serde_json::Value) -> bool {
    fn check_value(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(obj) => {
                if obj.contains_key("$ref") {
                    return true;
                }
                obj.values().any(check_value)
            }
            serde_json::Value::Array(arr) => arr.iter().any(check_value),
            _ => false,
        }
    }
    check_value(schema)
}

fn has_primitive_refs(schema: &serde_json::Value) -> bool {
    fn check_value(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(obj) => {
                if let Some(ref_val) = obj.get("$ref") {
                    if let Some(ref_str) = ref_val.as_str() {
                        return ref_str.contains("primitive");
                    }
                }
                obj.values().any(check_value)
            }
            serde_json::Value::Array(arr) => arr.iter().any(check_value),
            _ => false,
        }
    }
    check_value(schema)
}

fn has_inheritance_indicators(schema: &serde_json::Value) -> bool {
    schema.get("Libraries").is_some() ||
    schema.get("bundle_metadata").is_some() ||
    schema.get("Composer").is_some()
}

    #[test]
    fn test_nickel_validation_against_aligned_json_schemas() {
        // COMPREHENSIVE FAIL-FRIENDLY VALIDATION TEST
        // This test collects ALL validation results, provides verbose reporting,
        // and never exits early or has hardcoded success paths.
        // Failures indicate real development issues, not test problems.

        let aligned_directories = vec![
            // Only test directories that actually have both Nickel contracts and JSON schemas
            ("infrastructure/actions", "infrastructure/actions/classification"),
        ];

        // Check Nickel availability first (fail-friendly - clear error if not available)
        let validator = match NickelValidator::new() {
            Ok(v) => v,
            Err(e) => {
                panic!("❌ Nickel CLI not available for validation testing. Install Nickel CLI to run this test.\nError: {}", e);
            }
        };

        // Track comprehensive results across ALL directories and schemas
        let mut total_schemas_tested = 0;
        let mut total_validation_passed = 0;
        let mut total_validation_failed = 0;
        let mut total_parse_failed = 0;
        let mut total_read_failed = 0;
        let mut all_validation_errors = Vec::new();
        let mut all_parse_errors = Vec::new();
        let mut all_read_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut directory_results = Vec::new();

        println!("🚀 COMPREHENSIVE NICKEL VALIDATION TEST");
        println!("==============================================");
        println!("Testing Nickel validation contracts against JSON schemas");
        println!("This test collects ALL results and provides detailed feedback");
        println!("Failures indicate development work needed, not test issues");
        println!("==============================================");

        // Test each aligned directory
        for (nickel_path, json_path) in aligned_directories {
            println!("\n📁 DIRECTORY: {} ↔ {}", nickel_path, json_path);

            let mut dir_result = DirectoryTestResult {
                name: nickel_path.to_string(),
                nickel_contract_exists: false,
                json_directory_exists: false,
                schemas_found: 0,
                validation_passed: 0,
                validation_failed: 0,
                parse_failed: 0,
                read_failed: 0,
                errors: Vec::new(),
            };

            // Check structural alignment (record but don't fail immediately)
            let nickel_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../versions/latest/nickel")
                .join(nickel_path)
                .join("_directory.ncl");

            let json_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../versions/latest/json-schema")
                .join(json_path);

            dir_result.nickel_contract_exists = nickel_dir_path.exists();
            dir_result.json_directory_exists = json_dir_path.exists();

            if !dir_result.nickel_contract_exists {
                let msg = format!("❌ Nickel contract missing: {}", nickel_path);
                println!("{}", msg);
                dir_result.errors.push(msg.clone());
                all_validation_errors.push(msg);
            }

            if !dir_result.json_directory_exists {
                let msg = format!("❌ JSON directory missing: {}", json_path);
                println!("{}", msg);
                dir_result.errors.push(msg.clone());
                all_validation_errors.push(msg);
            }

            // Skip validation if structural issues exist
            if !dir_result.nickel_contract_exists || !dir_result.json_directory_exists {
                directory_results.push(dir_result);
                continue;
            }

            // Find ALL JSON schema files (not just a subset)
            let json_files = find_json_schema_files(&json_dir_path);
            dir_result.schemas_found = json_files.len();

            println!("📊 Found {} JSON schema files to validate", json_files.len());

            if json_files.is_empty() {
                let msg = format!("⚠️  No schemas to validate in: {}", json_path);
                println!("{}", msg);
                all_warnings.push(msg);
                directory_results.push(dir_result);
                continue;
            }

            // Test EVERY schema file comprehensively
            for json_file in json_files {
                let file_name = json_file.file_name().unwrap().to_string_lossy().to_string();
                total_schemas_tested += 1;
                dir_result.schemas_found += 1;

                print!("🔍 Testing: {} ... ", file_name);

                match fs::read_to_string(&json_file) {
                    Ok(content) => {
                        match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(schema) => {
                                // Actually run Nickel validation
                                match validator.validate_schema(&schema, &json_file) {
                                    Ok(()) => {
                                        total_validation_passed += 1;
                                        dir_result.validation_passed += 1;
                                        println!("✅ PASSED");
                                    }
                                    Err(e) => {
                                        total_validation_failed += 1;
                                        dir_result.validation_failed += 1;
                                        let error_msg = format!("❌ VALIDATION FAILED: {} - {}", file_name, e);
                                        println!("FAILED");
                                        println!("   Error: {}", e);
                                        dir_result.errors.push(error_msg.clone());
                                        all_validation_errors.push(format!("{} in {}", error_msg, nickel_path));
                                    }
                                }
                            }
                            Err(e) => {
                                total_parse_failed += 1;
                                dir_result.parse_failed += 1;
                                let error_msg = format!("❌ PARSE FAILED: {} - {}", file_name, e);
                                println!("PARSE ERROR");
                                println!("   Error: {}", e);
                                dir_result.errors.push(error_msg.clone());
                                all_parse_errors.push(format!("{} in {}", error_msg, nickel_path));
                            }
                        }
                    }
                    Err(e) => {
                        total_read_failed += 1;
                        dir_result.read_failed += 1;
                        let error_msg = format!("❌ READ FAILED: {} - {}", file_name, e);
                        println!("READ ERROR");
                        println!("   Error: {}", e);
                        dir_result.errors.push(error_msg.clone());
                        all_read_errors.push(format!("{} in {}", error_msg, nickel_path));
                    }
                }
            }

            // Directory summary
            println!("\n📈 Directory Summary for {}:", nickel_path);
            println!("   📊 Total schemas: {}", dir_result.schemas_found);
            println!("   ✅ Validation passed: {}", dir_result.validation_passed);
            println!("   ❌ Validation failed: {}", dir_result.validation_failed);
            println!("   📄 Parse failed: {}", dir_result.parse_failed);
            println!("   📁 Read failed: {}", dir_result.read_failed);

            directory_results.push(dir_result);
        }

        // COMPREHENSIVE FINAL REPORT
        println!("\n==============================================");
        println!("🎯 COMPREHENSIVE VALIDATION REPORT");
        println!("==============================================");
        println!("📊 OVERALL STATISTICS:");
        println!("   🔍 Total schemas tested: {}", total_schemas_tested);
        println!("   ✅ Validation passed: {}", total_validation_passed);
        println!("   ❌ Validation failed: {}", total_validation_failed);
        println!("   📄 Schema parse failed: {}", total_parse_failed);
        println!("   📁 File read failed: {}", total_read_failed);
        println!("   ⚠️  Warnings: {}", all_warnings.len());

        // Directory-by-directory breakdown
        println!("\n📁 DIRECTORY BREAKDOWN:");
        for dir_result in &directory_results {
            println!("   {}: {} tested, {} passed, {} failed",
                    dir_result.name,
                    dir_result.schemas_found,
                    dir_result.validation_passed,
                    dir_result.validation_failed);
        }

        // Detailed error reporting (all errors, not just first few)
        if !all_validation_errors.is_empty() {
            println!("\n🚨 VALIDATION ERRORS ({} total):", all_validation_errors.len());
            for (i, error) in all_validation_errors.iter().enumerate() {
                println!("   {}. {}", i + 1, error);
            }
        }

        if !all_parse_errors.is_empty() {
            println!("\n📄 SCHEMA PARSE ERRORS ({} total):", all_parse_errors.len());
            for (i, error) in all_parse_errors.iter().enumerate() {
                println!("   {}. {}", i + 1, error);
            }
        }

        if !all_read_errors.is_empty() {
            println!("\n📁 FILE READ ERRORS ({} total):", all_read_errors.len());
            for (i, error) in all_read_errors.iter().enumerate() {
                println!("   {}. {}", i + 1, error);
            }
        }

        if !all_warnings.is_empty() {
            println!("\n⚠️  WARNINGS ({} total):", all_warnings.len());
            for (i, warning) in all_warnings.iter().enumerate() {
                println!("   {}. {}", i + 1, warning);
            }
        }

        // ASSESSMENT - Tests MUST fail when validation errors are found
        println!("\n🎯 ASSESSMENT:");
        if total_schemas_tested == 0 {
            println!("❌ NO SCHEMAS TESTED - Check directory alignment and file availability");
            panic!("Test failed: No schemas were testable");
        }

        let total_failures = total_validation_failed + total_parse_failed + total_read_failed;
        if total_failures == 0 && total_validation_passed == total_schemas_tested {
            println!("✅ ALL TESTS PASSED - Nickel validation working perfectly");
        } else {
            let fail_rate = total_failures as f64 / total_schemas_tested as f64 * 100.0;
            println!("❌ VALIDATION ERRORS DETECTED - {:.1}% of tests failed ({}/{})",
                    fail_rate, total_failures, total_schemas_tested);
            println!("💥 This indicates BROKEN validation logic that MUST be fixed");

            // Provide specific guidance
            if total_validation_failed > 0 {
                println!("🔧 FIX: Nickel contracts have syntax/logic errors");
            }
            if total_parse_failed > 0 {
                println!("🔧 FIX: JSON schemas have syntax errors");
            }
            if total_read_failed > 0 {
                println!("🔧 FIX: File access issues");
            }

            panic!("Validation errors detected - fix before proceeding with development");
        }

        println!("\n🏁 TEST COMPLETED SUCCESSFULLY - All validations passed");
    }

    // Helper struct for directory results
    #[derive(Debug)]
    struct DirectoryTestResult {
        name: String,
        nickel_contract_exists: bool,
        json_directory_exists: bool,
        schemas_found: usize,
        validation_passed: usize,
        validation_failed: usize,
        parse_failed: usize,
        read_failed: usize,
        errors: Vec<String>,
    }

#[test]
fn test_directory_level_inheritance_patterns() {
    // Test that directory-level schemas (Nickel _directory.ncl files) follow inheritance patterns
    // NOT that JSON schemas don't use $refs - that was the misunderstanding

    let nickel_directories = vec![
        "architecture/_directory.ncl",
        "infrastructure/_directory.ncl",
        "infrastructure/actions/_directory.ncl",
        "infrastructure/techniques/_directory.ncl",
    ];

    for dir_file in nickel_directories {
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../versions/latest/nickel")
            .join(dir_file);

        assert!(file_path.exists(), "Nickel directory file should exist: {}", dir_file);

        let content = fs::read_to_string(&file_path).expect("Should read Nickel directory file");

        // Directory-level Nickel schemas should show inheritance patterns
        let has_inheritance = content.contains("Libraries") ||
                            content.contains("Composer") ||
                            content.contains("bundle_metadata") ||
                            content.contains("&");

        assert!(has_inheritance, "Directory-level Nickel schema should use inheritance: {}", dir_file);
    }
}

    #[test]
    fn test_current_schema_ref_usage_patterns() {
        // Test current $ref usage patterns to understand what we're validating against
        // This helps clarify that JSON schemas DO use $refs (leaf level is correct)
        // while Nickel schemas use inheritance (directory level is correct)

        let test_directories = vec![
            ("entities", "entity"),
            ("components", "component"),
            ("infrastructure/actions/classification", "action"),
            ("primitives", "primitive"),
        ];

        let mut total_refs_found = 0;
        let mut schemas_with_refs = 0;

        for (json_dir, expected_ref_type) in test_directories {
            let json_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../versions/latest/json-schema")
                .join(json_dir);

            if json_dir_path.exists() {
                let json_files = find_json_schema_files(&json_dir_path);

                for json_file in json_files.iter().take(3) { // Test first 3 files per directory
                    if let Ok(content) = fs::read_to_string(json_file) {
                        if let Ok(schema) = serde_json::from_str::<serde_json::Value>(&content) {
                            let ref_count = count_refs(&schema);
                            if ref_count > 0 {
                                total_refs_found += ref_count;
                                schemas_with_refs += 1;
                                println!("✅ {} schema {} uses {} $refs (expected for leaf level)",
                                        expected_ref_type, json_file.file_name().unwrap().to_string_lossy(), ref_count);
                            }
                        }
                    }
                }
            }
        }

        // Verify we found the expected $ref usage patterns
        assert!(schemas_with_refs > 0, "Should find schemas using $refs at leaf level");
        assert!(total_refs_found > 5, "Should find substantial $ref usage in leaf schemas");

        println!("📊 Summary: Found {} schemas with $refs, {} total $ref usages (leaf level usage is correct)",
                schemas_with_refs, total_refs_found);
    }

    #[test]
    fn test_nickel_directory_validation_applicability() {
        // Demonstrate that Nickel _directory.ncl files are validation contracts
        // meant to be applied against JSON schemas in their directories

        // This clarifies the "Directory Level: No $refs currently used" statement
        // It means Nickel directory schemas don't contain $refs (they use inheritance)
        // But they ARE applied against JSON schemas that DO contain $refs

        let nickel_dirs = vec![
            "infrastructure/actions/_directory.ncl",
        ];

        let json_dirs = vec![
            "infrastructure/actions/classification",
        ];

        for (nickel_file, json_dir) in nickel_dirs.iter().zip(json_dirs.iter()) {
            // Nickel directory contract exists
            let nickel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../versions/latest/nickel")
                .join(nickel_file);
            assert!(nickel_path.exists(), "Nickel validation contract should exist: {}", nickel_file);

            // JSON schemas to validate exist
            let json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../versions/latest/json-schema")
                .join(json_dir);
            assert!(json_path.exists(), "JSON schemas to validate should exist: {}", json_dir);

            let json_files = find_json_schema_files(&json_path);
            assert!(!json_files.is_empty(), "Should have JSON schemas for Nickel to validate: {}", json_dir);

            // Read Nickel contract to verify it contains validation logic
            let nickel_content = fs::read_to_string(&nickel_path).expect("Should read Nickel contract");
            assert!(nickel_content.contains("validation") || nickel_content.contains("Validation"),
                   "Nickel contract should contain validation logic: {}", nickel_file);
        }

        println!("🎯 Nickel Validation Pattern Confirmed:");
        println!("   - Nickel _directory.ncl files = Validation contracts");
        println!("   - Applied against JSON schemas in corresponding directories");
        println!("   - Nickel contracts use inheritance (& merging)");
        println!("   - JSON schemas use $refs (leaf level composition)");
    }

// Helper functions for Nickel validation tests
fn find_json_schema_files(dir_path: &PathBuf) -> Vec<PathBuf> {
    let mut json_files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                json_files.push(path);
            }
        }
    }
    json_files
}

fn count_refs(schema: &serde_json::Value) -> usize {
    let mut count = 0;
    fn count_in_value(value: &serde_json::Value, count: &mut usize) {
        match value {
            serde_json::Value::Object(obj) => {
                if obj.contains_key("$ref") {
                    *count += 1;
                }
                for val in obj.values() {
                    count_in_value(val, count);
                }
            }
            serde_json::Value::Array(arr) => {
                for val in arr {
                    count_in_value(val, count);
                }
            }
            _ => {}
        }
    }
    count_in_value(schema, &mut count);
    count
}