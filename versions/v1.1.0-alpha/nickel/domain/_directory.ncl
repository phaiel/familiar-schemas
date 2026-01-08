/**
 * Domain Module - Pure Business Logic Governance
 *
 * Ensures schemas in domain/ represent BUSINESS LOGIC: bounded contexts
 * and domain rules that define the "Why" behind system behavior.
 *
 * @orthology This module enforces the "Why" - business requirements that drive behavior
 * @guarantees NO data structures, NO compute specs, NO wiring/topology concerns
 * @ddd Domain-Driven Design principles, bounded contexts, business rules
 *
 * @example
 *   # Extract business logic from mixed schema
 *   let business = (import "domain/_directory.ncl").files._.extract_business_logic(raw_schema)
 *
 *   # Validate business logic purity
 *   let result = (import "domain/_directory.ncl").files._.validate_pure_business(business)
 */

{
  # ============================================================================
  # DOMAIN CONTRACTS - Business Logic Transformation Rules
  # ============================================================================

  # Contract for all files in domain/ - must be pure business logic
  files = {
    _ : {
      # Implement DomainContract interface with type safety
      extract_from_raw = fun raw_schema => extract_business_logic raw_schema,
      validate_pure = fun schema => validate_pure_business schema,
      get_layer = `domain,

      # extract_business_logic : Schema -> BusinessLogic
      # Transforms mixed schemas into pure business logic specifications
      # Strips technical concerns to focus on domain requirements and rules
      extract_business_logic = fun raw_schema => {
        # Strip ALL technical concerns (data, compute, wiring)
        let forbidden_technical = [
          "x-familiar-codegen-targets", "x-familiar-service", "x-familiar-api",
          "x-familiar-queue", "x-familiar-persistence", "x-familiar-resources",
          "x-familiar-node-type", "x-familiar-scaling", "x-familiar-monitoring-enabled",
          "properties", "type", "required", "$ref"  # Data structure fields
        ] in

        let business_only = std.record.remove_all forbidden_technical raw_schema in

        # Ensure domain context
        let with_domain = if std.record.has_field "x-familiar-domain-context" business_only then
          business_only
        else
          let inferred_context = infer_domain_from_path raw_schema."$id" | "business_logic" in
          business_only & {
            "x-familiar-domain-context" = inferred_context
          } in

        with_domain & {
          "x-familiar-layer" = "domain",
          "x-familiar-orthology" = "business_only",
          "x-familiar-ddd-principles" = ["bounded_context", "domain_model"]
        }
      },

      # validate_pure_business : BusinessLogic -> ValidationResult
      # Validates that domain schema contains only business logic (no technical concerns)
      # Ensures orthological purity - domains must be pure business requirements only
      validate_pure_business = fun domain_schema => {
        let technical_indicators = [
          "x-familiar-codegen-targets", "x-familiar-service", "x-familiar-resources",
          "x-familiar-node-type", "properties", "type", "$ref"
        ] in

        let wiring_indicators = [
          "x-familiar-api-endpoints", "bindings", "x-familiar-topology-type"
        ] in

        let has_technical = std.array.any (fun field =>
          std.record.has_field field domain_schema
        ) technical_indicators in

        let has_wiring = std.array.any (fun field =>
          std.record.has_field field domain_schema
        ) wiring_indicators in

        if has_technical then
          { valid = false, errors = ["Domain schemas cannot contain technical concerns. Extract to codegen/infrastructure layers."], warnings = [] }
        else if has_wiring then
          { valid = false, errors = ["Domain schemas cannot contain wiring concerns. Extract to architecture layer."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_business_logic : BusinessLogic -> ValidationResult
      # Ensures domain schema has proper business logic structure and bounded context
      # Validates DDD compliance and business requirement completeness
      validate_business_logic = fun domain_schema => {
        # Must have domain context and some business specification
        let has_context = std.record.has_field "x-familiar-domain-context" domain_schema in
        let has_business_spec = std.record.has_field "x-familiar-business-rules" domain_schema ||
                               std.record.has_field "x-familiar-domain-events" domain_schema ||
                               std.record.has_field "x-familiar-api-contracts" domain_schema in

        if has_context && has_business_spec then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Domain schemas must specify bounded context and business specifications"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # ERROR RECOVERY MECHANISMS
  # ============================================================================

  # recover_domain_errors : BusinessLogic -> ErrorRecovery
  # Attempt to recover from common domain validation errors
  recover_domain_errors = fun domain_schema =>
    (import "../global.ncl").try_recover
      (fun => validate_pure_business domain_schema)
      ["add_domain_context", "infer_business_specs", "fix_missing_fields"]
      (fun error =>
        match error {
          "Domain schemas must specify bounded context and business specifications" =>
            # Recovery: Add inferred domain context and basic business specs
            let inferred_context = infer_domain_from_path domain_schema."$id" | "business_logic" in
            let recovered_schema = domain_schema & {
              "x-familiar-domain-context" = inferred_context,
              "x-familiar-business-rules" = ["inferred_business_rule"]
            } in
            `recovered {
              result = recovered_schema,
              applied_fixes = ["Added inferred domain context and business rules"]
            },
          "Domain schemas cannot contain technical concerns" =>
            # Recovery: Attempt to strip technical fields
            let technical_fields = [
              "x-familiar-codegen-targets", "x-familiar-service",
              "x-familiar-resources", "properties", "type"
            ] in
            let cleaned_schema = std.record.remove_all technical_fields domain_schema in
            `recovered {
              result = cleaned_schema,
              applied_fixes = ["Removed technical concerns from domain schema"]
            },
          _ =>
            # Unrecoverable error
            `unrecoverable {
              errors = [{
                field = "domain",
                expected = "Pure business logic",
                actual = std.to_string error,
                suggestion = "Focus on business requirements, remove technical concerns",
                layer = `domain,
                severity = `error
              }]
            }
        }
      ),

  # ============================================================================
  # CONTRACT TESTING FRAMEWORK
  # ============================================================================

  # Test cases for domain contract validation
  test_cases = [
    # Valid domain test
    {
      name = "valid_domain_schema",
      input = {
        "x-familiar-kind" = "domain",
        "x-familiar-description" = "Valid domain schema",
        "x-familiar-domain-context" = "user_management",
        "x-familiar-business-rules" = ["users_must_have_email", "emails_must_be_unique"],
        "x-familiar-domain-events" = ["UserRegistered", "UserUpdated"]
      },
      expected_output = {
        "x-familiar-kind" = "domain",
        "x-familiar-description" = "Valid domain schema",
        "x-familiar-domain-context" = "user_management",
        "x-familiar-business-rules" = ["users_must_have_email", "emails_must_be_unique"],
        "x-familiar-domain-events" = ["UserRegistered", "UserUpdated"],
        "x-familiar-layer" = "domain",
        "x-familiar-orthology" = "business_only",
        "x-familiar-ddd-principles" = ["bounded_context", "domain_model"]
      },
      expected_errors = [],
      description = "Valid domain schema should pass extraction and validation"
    },

    # Domain with forbidden technical concerns
    {
      name = "domain_with_technical_concerns",
      input = {
        "x-familiar-kind" = "domain",
        "x-familiar-description" = "Domain with forbidden technical fields",
        "x-familiar-domain-context" = "user_management",
        "x-familiar-codegen-targets" = ["rust"],  # FORBIDDEN
        "properties" = { "user_id" = { "type" = "string" } },  # FORBIDDEN
        "x-familiar-service" = { "$ref" = "../nodes/api.json" }  # FORBIDDEN
      },
      expected_output = {
        "x-familiar-kind" = "domain",
        "x-familiar-description" = "Domain with forbidden technical fields",
        "x-familiar-domain-context" = "user_management",
        "x-familiar-layer" = "domain",
        "x-familiar-orthology" = "business_only",
        "x-familiar-ddd-principles" = ["bounded_context", "domain_model"]
      },
      expected_errors = ["Domain schemas cannot contain technical concerns. Extract to codegen/infrastructure layers."],
      description = "Domain with technical concerns should fail validation"
    }
  ],

  # run_tests : _ -> Array<ContractTestResult>
  # Execute all contract tests for this layer
  run_tests = fun =>
    (import "../global.ncl").run_contract_tests files._ test_cases,

  # ============================================================================
  # DOMAIN HELPER FUNCTIONS
  # ============================================================================

  # infer_domain_from_path : String -> String
  # Infers bounded context from schema identifier path
  # Maps file paths to appropriate domain contexts for DDD compliance
  infer_domain_from_path = fun schema_id =>
    if std.is_string schema_id then
      let path_parts = std.string.split "/" schema_id in
      let filename_part = std.array.at path_parts (std.array.length path_parts - 1) in
      let name_parts = std.string.split "." filename_part in
      let base_name = std.array.at name_parts 0 in

      # Infer domain from filename patterns
      if std.string.contains "auth" (std.string.to_lower base_name) then
        "authentication"
      else if std.string.contains "user" (std.string.to_lower base_name) then
        "user_management"
      else if std.string.contains "tenant" (std.string.to_lower base_name) then
        "multi_tenancy"
      else if std.string.contains "conversation" (std.string.to_lower base_name) then
        "conversational_ai"
      else
        "business_logic"
    else
      "business_logic",

  # ============================================================================
  # DOMAIN HYDRATION - Pure Business Logic Context
  # ============================================================================

  hydration = {
    "_domain" = {
      ddd_principles = "enforced",      # Domain-Driven Design
      bounded_contexts = "isolated",    # Context isolation
      business_logic_focus = true,      # Business logic priority
      technical_debt_tracking = true,
      orthology_level = "domain",
      data_structure_free = true,       # Guarantee: no data schemas in domain layer
      compute_spec_free = true,         # Guarantee: no compute specs in domain layer
      wiring_free = true                # Guarantee: no topology wiring in domain layer
    },

    "_metadata" = {
      governance_level = "domain",
      category = "business_logic",
      orthological_role = "the_why"      # Defines WHY the system exists
    },

    "_observability" = {
      domain_event_tracking = true,
      business_rule_monitoring = true,
      context_boundary_checks = true,
      domain_metric_collection = true,
      ddd_compliance_tracking = true
    }
  },

  # ============================================================================
  # SUBDIRECTORY ROUTING
  # ============================================================================

  # Route to appropriate domain subdirectory
  route_subdirectory = fun subcategory =>
    if subcategory == "agentic" then
      import "agentic/_directory.ncl"
    else if subcategory == "api" then
      import "api/_directory.ncl"
    else if subcategory == "auth" then
      import "auth/_directory.ncl"
    else if subcategory == "config" then
      import "config/_directory.ncl"
    else if subcategory == "contracts" then
      import "contracts/_directory.ncl"
    else if subcategory == "conversation" then
      import "conversation/_directory.ncl"
    else if subcategory == "database" then
      import "database/_directory.ncl"
    else if subcategory == "tenant" then
      import "tenant/_directory.ncl"
    else if subcategory == "ui" then
      import "ui/_directory.ncl"
    else if subcategory == "entities_api" then
      import "entities_api/_directory.ncl"
    else
      # Default domain validation
      {
        validate = fun schema_content => {
          valid = true,
          errors = [],
          warnings = ["Unknown domain subcategory: ${subcategory}"]
        }
      }
}
