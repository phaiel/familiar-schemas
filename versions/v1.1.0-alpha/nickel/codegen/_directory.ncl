/**
 * Codegen Module - Pure Entity Data Governance
 *
 * Ensures schemas in codegen/ represent ENTITIES: pure data structures
 * in "Typespace" that can be safely code-generated without runtime concerns.
 *
 * @orthology This module enforces the "What" - pure data that gets processed by infrastructure
 * @guarantees NO runtime code generation, NO service bindings, NO infrastructure references
 *
 * @example
 *   # Extract pure entity from mixed schema
 *   let entity = (import "codegen/_directory.ncl").files._.extract_entity_data(raw_schema)
 *
 *   # Validate entity purity
 *   let result = (import "codegen/_directory.ncl").files._.validate_pure_entity(entity)
 */

{
  # ============================================================================
  # CODEGEN CONTRACTS - Entity Transformation Rules
  # ============================================================================

  # Contract for all files in codegen/ - must be pure data entities
  files = {
    _ : {
      # Implement EntityContract interface with type safety
      extract_from_raw = fun raw_schema => extract_entity_data raw_schema,
      validate_pure = fun schema => validate_pure_entity schema,
      get_layer = `entity,

      # extract_entity_data : Schema -> Entity
      # Transforms mixed schemas into pure entity data structures by stripping runtime concerns
      # This is the core orthological extraction for the "What" layer
      extract_entity_data = fun raw_schema => {
        # Strip ALL runtime and infrastructure concerns
        let forbidden_runtime = [
          "x-familiar-service", "x-familiar-api", "x-familiar-queue",
          "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-node-type",
          "x-familiar-scaling", "x-familiar-deploy-config"
        ] in

        let pure_entity = std.record.remove_all forbidden_runtime raw_schema in

        # Ensure required codegen metadata
        let with_codegen = if std.record.has_field "x-familiar-codegen-targets" pure_entity then
          pure_entity
        else
          pure_entity & {
            "x-familiar-codegen-targets" = ["rust", "typescript"]
          } in

        with_codegen & {
          "x-familiar-layer" = "entity",
          "x-familiar-orthology" = "data_only"
        }
      },

      # validate_pure_entity : Entity -> ValidationResult
      # Validates that extracted entity contains no runtime concerns
      # Ensures orthological purity - entities must be pure data only
      validate_pure_entity = fun entity_schema => {
        let runtime_indicators = [
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-deploy-config"
        ] in

        let has_runtime = std.array.any (fun field =>
          std.record.has_field field entity_schema
        ) runtime_indicators in

        if has_runtime then
          { valid = false, errors = ["Entity schemas cannot contain runtime concerns. Extract to infrastructure/topology layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_entity_structure : Entity -> ValidationResult
      # Ensures entity has proper data structure elements (properties, type, $ref)
      # Validates that entity represents a meaningful data construct
      validate_entity_structure = fun entity_schema => {
        # Must have properties or be a primitive type reference
        let has_properties = std.record.has_field "properties" entity_schema in
        let has_ref = std.record.has_field "$ref" entity_schema in
        let has_type = std.record.has_field "type" entity_schema in

        if has_properties || has_ref || has_type then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Entity schemas must define data structure (properties, $ref, or type)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # CONTRACT TESTING FRAMEWORK
  # ============================================================================

  # Built-in test cases for entity contract validation
  test_cases = [
    # Valid entity test
    {
      name = "valid_entity_schema",
      input = {
        "type" = "object",
        "properties" = {
          "id" = { "type" = "string", "format" = "uuid" },
          "name" = { "type" = "string" }
        },
        "x-familiar-kind" = "entity",
        "x-familiar-description" = "A valid entity"
      },
      expected_output = {
        "type" = "object",
        "properties" = {
          "id" = { "type" = "string", "format" = "uuid" },
          "name" = { "type" = "string" }
        },
        "x-familiar-kind" = "entity",
        "x-familiar-description" = "A valid entity",
        "x-familiar-codegen-targets" = ["rust", "typescript"],
        "x-familiar-layer" = "entity",
        "x-familiar-orthology" = "data_only"
      },
      expected_errors = [],
      description = "Valid entity schema should pass extraction and validation"
    },

    # Entity with forbidden infrastructure
    {
      name = "entity_with_infrastructure",
      input = {
        "type" = "object",
        "properties" = { "id" = { "type" = "string" } },
        "x-familiar-kind" = "entity",
        "x-familiar-description" = "Entity with forbidden fields",
        "x-familiar-service" = { "$ref" = "../nodes/worker.json" },  # FORBIDDEN
        "x-familiar-resources" = { "cpu" = "2" }  # FORBIDDEN
      },
      expected_output = {
        "type" = "object",
        "properties" = { "id" = { "type" = "string" } },
        "x-familiar-kind" = "entity",
        "x-familiar-description" = "Entity with forbidden fields",
        "x-familiar-codegen-targets" = ["rust", "typescript"],
        "x-familiar-layer" = "entity",
        "x-familiar-orthology" = "data_only"
      },
      expected_errors = ["Entity schemas cannot contain runtime concerns. Extract to infrastructure/topology layers."],
      description = "Entity with infrastructure concerns should fail validation"
    }
  ],

  # run_tests : _ -> Array<ContractTestResult>
  # Execute all contract tests for this layer
  run_tests = fun =>
    (import "../global.ncl").run_contract_tests files._ test_cases,

  # ============================================================================
  # CODEGEN HYDRATION - Data Generation Context
  # ============================================================================

  hydration = {
    "_codegen" = {
      generation_strategy = "tvm_pattern",  # Template-View-Model
      purity_level = "entity_data_only",    # ENTITIES only - no compute, no wiring
      orthology_level = "data",
      template_engine = "configurable",
      type_safety = "strict",
      cross_language_support = true,
      runtime_concern_free = true        # Guarantee: no runtime code generation
    },

    "_metadata" = {
      governance_level = "codegen",
      category = "entity_data",
      orthological_role = "the_what"      # Defines WHAT gets processed
    },

    "_observability" = {
      codegen_metrics = true,
      template_usage_tracking = true,
      generation_success_rates = true,
      cross_language_consistency = true,
      entity_purity_tracking = true       # Track that entities remain pure
    }
  },

  # ============================================================================
  # SUBDIRECTORY CONTRACTS - Entity Type Specialization
  # ============================================================================

  # Route to specialized entity contracts
  route_subdirectory = fun subcategory =>
    if subcategory == "entities" then
      import "entities/_directory.ncl"
    else if subcategory == "types" then
      import "types/_directory.ncl"
    else if subcategory == "components" then
      import "components/_directory.ncl"
    else if subcategory == "primitives" then
      import "primitives/_directory.ncl"
    else
      # Default entity contract
      files.{_}.extract_entity_data
}
