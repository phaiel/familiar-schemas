/**
 * Database Domain Module - Pure Business Logic Governance
 *
 * Ensures schemas in domain/database/ represent pure BUSINESS LOGIC
 * related to data persistence and access patterns.
 *
 * @orthology This module enforces pure business logic for data operations
 * @guarantees NO technical concerns, NO infrastructure, NO data definitions
 *
 * @example
 *   # Extract business logic from mixed schema
 *   let business_logic = (import "database/_directory.ncl").files._.extract_business_logic(raw_schema)
 *
 *   # Validate business logic purity
 *   let result = (import "database/_directory.ncl").files._.validate_pure_business(business_logic)
 */

{
  # ============================================================================
  # DATABASE DOMAIN CONTRACTS - Business Logic Rules
  # ============================================================================

  # Contract for all files in domain/database/ - must be pure business logic
  files = {
    _ : {
      # Implement DomainContract interface with type safety for database domain
      extract_business_logic = fun raw_schema => extract_business_logic raw_schema,
      validate_pure_business = fun schema => validate_pure_business schema,
      get_layer = `domain,

      # extract_business_logic : Schema -> BusinessLogic
      # Transforms mixed schemas into pure business logic for data operations
      # Database domain focuses on business rules for data access and persistence
      extract_business_logic = fun raw_schema => {
        # Strip ALL technical and infrastructure concerns from business logic
        let forbidden_technical = [
          "x-familiar-service", "x-familiar-api", "x-familiar-queue",
          "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-node-type",
          "x-familiar-scaling", "x-familiar-deploy-config",
          "properties", "type", "enum", "format"  # No data definitions
        ] in

        let pure_business = std.record.remove_all forbidden_technical raw_schema in

        # Ensure required domain metadata for database business logic
        pure_business & {
          "x-familiar-layer" = "domain",
          "x-familiar-orthology" = "database_business_logic",
          "x-familiar-domain-context" = "data_persistence",
          "x-familiar-ddd-principles" = ["domain_model", "bounded_context"]
        }
      },

      # validate_pure_business : BusinessLogic -> ValidationResult
      # Validates that extracted business logic contains no technical concerns
      # Ensures orthological purity - business logic must be pure domain concepts only
      validate_pure_business = fun business_schema => {
        let technical_indicators = [
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-deploy-config",
          "properties", "type", "enum", "format"  # No data definitions
        ] in

        let has_technical = std.array.any (fun field =>
          std.record.has_field field business_schema
        ) technical_indicators in

        if has_technical then
          { valid = false, errors = ["Database domain schemas cannot contain technical concerns. Extract to infrastructure layer."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_business_logic : BusinessLogic -> ValidationResult
      # Ensures business logic has proper domain-driven design structure
      # Validates that business logic represents meaningful domain concepts
      validate_business_logic = fun business_schema => {
        # Must have domain-like properties (DDD patterns)
        let has_domain_context = std.record.has_field "x-familiar-domain-context" business_schema in
        let has_business_rules = std.record.has_field "x-familiar-business-rules" business_schema in
        let has_domain_events = std.record.has_field "x-familiar-domain-events" business_schema in
        let has_aggregates = std.record.has_field "x-familiar-aggregates" business_schema in

        if has_domain_context || has_business_rules || has_domain_events || has_aggregates then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Database domain schemas must define business logic (domain context, business rules, domain events, or aggregates)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # DATABASE DOMAIN HYDRATION - Pure Business Logic Context
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    "_domain" = (import "../global.ncl").hydration."_domain" & {
      database_business_logic = true,
      data_access_patterns = true,
      persistence_rules = true
    },

    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "domain.database",
      orthological_role = "database_business_logic"
    },

    "_observability" = (import "../global.ncl").hydration."_observability" & {
      business_logic_metrics = true,
      domain_event_tracking = true,
      data_access_monitoring = true
    }
  }
}
