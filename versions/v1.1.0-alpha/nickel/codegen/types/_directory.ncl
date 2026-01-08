/**
 * Types Module - Pure Type Data Governance
 *
 * Ensures schemas in codegen/types/ represent pure TYPE definitions
 * that can be used in entities and components.
 *
 * @orthology This module enforces pure type definitions
 * @guarantees NO runtime concerns, NO infrastructure, NO business logic
 *
 * @example
 *   # Extract type definition from mixed schema
 *   let type_def = (import "types/_directory.ncl").files._.extract_entity_data(raw_schema)
 *
 *   # Validate type purity
 *   let result = (import "types/_directory.ncl").files._.validate_pure_entity(type_def)
 */

{
  # ============================================================================
  # TYPE CONTRACTS - Type Definition Rules
  # ============================================================================

  # Contract for all files in codegen/types/ - must be pure type definitions
  files = {
    _ : {
      # Implement EntityContract interface with type safety for types
      extract_from_raw = fun raw_schema => extract_entity_data raw_schema,
      validate_pure = fun schema => validate_pure_entity schema,
      get_layer = `entity,


      # extract_entity_data : Schema -> Type
      # Transforms mixed schemas into pure type definitions
      # Types are pure data type definitions without runtime concerns
      extract_entity_data = fun raw_schema => {
        # Strip ALL runtime and infrastructure concerns from types
        let forbidden_runtime = [
          "x-familiar-service", "x-familiar-api", "x-familiar-queue",
          "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-node-type",
          "x-familiar-scaling", "x-familiar-deploy-config"
        ] in

        let pure_type = std.record.remove_all forbidden_runtime raw_schema in

        # Ensure required codegen metadata for types
        let with_codegen = if std.record.has_field "x-familiar-codegen-targets" pure_type then
          pure_type
        else
          pure_type & {
            "x-familiar-codegen-targets" = ["rust", "typescript"]
          } in

        with_codegen & {
          "x-familiar-layer" = "entity",
          "x-familiar-orthology" = "type_definition_only",
          "x-familiar-type-category" = "composite"  # Default, can be overridden
        }
      },

      # validate_pure_entity : Type -> ValidationResult
      # Validates that extracted type contains no runtime concerns
      # Ensures orthological purity - types must be pure data definitions only
      validate_pure_entity = fun type_schema => {
        let runtime_indicators = [
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-deploy-config"
        ] in

        let has_runtime = std.array.any (fun field =>
          std.record.has_field field type_schema
        ) runtime_indicators in

        if has_runtime then
          { valid = false, errors = ["Type schemas cannot contain runtime concerns. Extract to infrastructure/topology layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_entity_structure : Type -> ValidationResult
      # Ensures type has proper type definition structure
      # Validates that type represents a meaningful data type
      validate_entity_structure = fun type_schema => {
        # Must have type-like properties (JSON Schema type definitions)
        let has_type = std.record.has_field "type" type_schema in
        let has_enum = std.record.has_field "enum" type_schema in
        let has_properties = std.record.has_field "properties" type_schema in
        let has_type_category = std.record.has_field "x-familiar-type-category" type_schema in

        if has_type || has_enum || has_properties || has_type_category then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Type schemas must define type structure (type, enum, properties, or x-familiar-type-category)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # TYPE HYDRATION - Pure Type Definition Context
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    "_codegen" = (import "../global.ncl").hydration."_codegen" & {
      type_generation = true,
      type_safety = true,
      validation_rules = true
    },

    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "codegen.types",
      orthological_role = "type_definition"
    },

    "_observability" = (import "../global.ncl").hydration."_observability" & {
      type_usage_tracking = true,
      validation_metrics = true,
      type_safety_metrics = true
    }
  }
}
