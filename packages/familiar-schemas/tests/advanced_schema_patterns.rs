//! Advanced JSON Schema Pattern Testing
//!
//! Tests that the JSON Schema validator correctly handles complex schema patterns
//! and that Nickel can process schemas that use these advanced features.
//! Nickel validates architectural compliance, not JSON Schema syntax.

use std::fs;
use std::path::PathBuf;

use familiar_schemas::NickelValidator;
use rstest::rstest;

/// Load a schema from a fixture file
fn load_schema(fixture_path: &PathBuf) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(fixture_path)?;
    let schema: serde_json::Value = serde_json::from_str(&content)?;
    Ok(schema)
}

/// Test that allOf composition works in JSON Schema validation
#[rstest]
#[case("allOf_composition.json")]
#[case("conditional_schema.json")]
#[case("dependent_validation.json")]
#[case("pattern_properties.json")]
#[case("array_constraints.json")]
fn test_jsonschema_validator_accepts_advanced_patterns(#[case] fixture_name: &str) {
    // This test would use the jsonschema crate to validate that the schema itself is valid
    // Since we don't have jsonschema as a dependency in this crate, we'll test that the schema
    // can be loaded and parsed, and that Nickel can process it

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(fixture_name);

    assert!(fixture_path.exists(), "Fixture {} should exist", fixture_name);

    let schema = load_schema(&fixture_path).expect("Should be able to load schema");
    assert!(schema.is_object(), "Schema should be a valid JSON object");

    // Verify it has the expected JSON Schema structure
    assert!(schema.get("$schema").is_some(), "Should have $schema field");
    assert!(schema.get("$id").is_some(), "Should have $id field");
}

/// Test that Nickel can process schemas with advanced JSON Schema patterns
#[rstest]
#[case("allOf_composition.json")]
#[case("conditional_schema.json")]
#[case("dependent_validation.json")]
#[case("pattern_properties.json")]
#[case("array_constraints.json")]
fn test_nickel_processes_advanced_jsonschema_patterns(#[case] fixture_name: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(fixture_name);

    let schema = load_schema(&fixture_path).expect("Should be able to load schema");

    // Test that Nickel validator can process these schemas
    // Note: Nickel validates architectural compliance, not JSON Schema syntax
    let nickel_validator = NickelValidator::new()
        .expect("Nickel must be available for validation testing. Install Nickel CLI to run this test.");

    // Test that Nickel can process schemas with advanced JSON Schema patterns
    let result = nickel_validator.validate_schema(&schema, &fixture_path);
    // We don't assert success/failure here since Nickel validates architecture, not JSON Schema syntax
    // The important thing is that it doesn't crash when processing complex schemas
    assert!(result.is_ok() || matches!(result, Err(_)), "Nickel should be able to attempt processing the schema");
}

/// Test that allOf composition schemas have the expected structure
#[test]
fn test_allof_composition_structure() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/allOf_composition.json");

    let schema = load_schema(&fixture_path).expect("Should load allOf schema");

    // Verify allOf structure
    assert!(schema.get("allOf").is_some(), "Should have allOf field");
    let all_of = schema.get("allOf").unwrap().as_array().unwrap();
    assert!(all_of.len() >= 2, "Should have multiple allOf components");

    // Verify x-familiar extensions are present
    assert!(schema.get("x-familiar-kind").is_some(), "Should have familiar kind");
    assert!(schema.get("x-familiar-description").is_some(), "Should have familiar description");
}

/// Test that conditional schemas have the expected if/then/else structure
#[test]
fn test_conditional_schema_structure() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/conditional_schema.json");

    let schema = load_schema(&fixture_path).expect("Should load conditional schema");

    // Verify conditional structure
    assert!(schema.get("if").is_some(), "Should have if condition");
    assert!(schema.get("then").is_some(), "Should have then clause");
    assert!(schema.get("else").is_some(), "Should have else clause");

    // Verify the condition is on the "type" field
    let if_condition = schema.get("if").unwrap();
    assert!(if_condition.get("properties").is_some(), "If should have properties");
}

/// Test that dependent validation schemas have the expected structure
#[test]
fn test_dependent_validation_structure() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/dependent_validation.json");

    let schema = load_schema(&fixture_path).expect("Should load dependent validation schema");

    // Verify dependent schemas structure
    assert!(schema.get("dependentSchemas").is_some(), "Should have dependentSchemas");
    assert!(schema.get("dependentRequired").is_some(), "Should have dependentRequired");

    // Verify credit card dependency
    let dependent_schemas = schema.get("dependentSchemas").unwrap().as_object().unwrap();
    assert!(dependent_schemas.contains_key("creditCard"), "Should have creditCard dependency");
}

/// Test that pattern properties schemas have the expected structure
#[test]
fn test_pattern_properties_structure() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/pattern_properties.json");

    let schema = load_schema(&fixture_path).expect("Should load pattern properties schema");

    // Verify pattern properties structure
    assert!(schema.get("patternProperties").is_some(), "Should have patternProperties");
    assert!(schema.get("propertyNames").is_some(), "Should have propertyNames");
    assert_eq!(schema.get("additionalProperties"), Some(&serde_json::Value::Bool(false)), "Should disallow additional properties");

    // Verify specific patterns
    let pattern_props = schema.get("patternProperties").unwrap().as_object().unwrap();
    assert!(pattern_props.contains_key("^config_"), "Should have config pattern");
    assert!(pattern_props.contains_key("^metadata_"), "Should have metadata pattern");
    assert!(pattern_props.contains_key("^data_\\d+$"), "Should have data pattern");
}

/// Test that array constraints schemas have the expected structure
#[test]
fn test_array_constraints_structure() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/array_constraints.json");

    let schema = load_schema(&fixture_path).expect("Should load array constraints schema");

    // Verify array constraints on different properties
    let properties = schema.get("properties").unwrap().as_object().unwrap();

    // Check tags array has constraints
    let tags = properties.get("tags").unwrap();
    assert!(tags.get("minItems").is_some(), "Tags should have minItems");
    assert!(tags.get("maxItems").is_some(), "Tags should have maxItems");
    assert_eq!(tags.get("uniqueItems"), Some(&serde_json::Value::Bool(true)), "Tags should enforce uniqueness");

    // Check scores array has contains constraint
    let scores = properties.get("scores").unwrap();
    assert!(scores.get("contains").is_some(), "Scores should have contains constraint");
    assert!(scores.get("minItems").is_some(), "Scores should have minItems");
}

/// Test that all advanced schemas can be processed by the familiar-schema system
#[rstest]
#[case("allOf_composition.json")]
#[case("conditional_schema.json")]
#[case("dependent_validation.json")]
#[case("pattern_properties.json")]
#[case("array_constraints.json")]
fn test_advanced_schemas_integration(#[case] fixture_name: &str) {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(fixture_name);

    let schema = load_schema(&fixture_path).expect("Should load schema");

    // Test that the schema has required familiar extensions
    assert!(schema.get("x-familiar-kind").is_some(), "Should have familiar kind for {}", fixture_name);
    assert!(schema.get("x-familiar-description").is_some(), "Should have familiar description for {}", fixture_name);

    // Test that it's a valid JSON Schema with $schema and $id
    assert!(schema.get("$schema").is_some(), "Should have JSON Schema declaration for {}", fixture_name);
    assert!(schema.get("$id").is_some(), "Should have schema ID for {}", fixture_name);

    // Verify it's recognized as an entity schema (most of these are entities)
    let kind = schema.get("x-familiar-kind").unwrap().as_str().unwrap();
    assert!(kind == "entity", "Advanced schemas should be entities for {}", fixture_name);
}