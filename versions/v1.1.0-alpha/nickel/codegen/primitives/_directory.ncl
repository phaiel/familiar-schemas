/**
 * Primitives Module - Pure Primitive Data Governance
 *
 * Ensures schemas in codegen/primitives/ represent pure PRIMITIVE type definitions
 * that form the foundation of the type system.
 *
 * @orthology This module enforces pure primitive type definitions
 * @guarantees NO runtime concerns, NO complex structures, NO business logic
 *
 * @example
 *   # Extract primitive definition from mixed schema
 *   let primitive_def = (import "primitives/_directory.ncl").files._.extract_entity_data(raw_schema)
 *
 *   # Validate primitive purity
 *   let result = (import "primitives/_directory.ncl").files._.validate_pure_entity(primitive_def)
 */

{
  # ============================================================================
  # PRIMITIVE CONTRACTS - Primitive Type Rules
  # ============================================================================

  # Contract for all files in codegen/primitives/ - must be pure primitive types
  files = {
    _ : {
      # Implement EntityContract interface with type safety for primitives
      extract_from_raw = fun raw_schema => extract_entity_data raw_schema,
      validate_pure = fun schema => validate_pure_entity schema,
      get_layer = `entity,

      # extract_entity_data : Schema -> Primitive
      # Transforms mixed schemas into pure primitive type definitions
      # Primitives are the fundamental building blocks of the type system
      extract_entity_data = fun raw_schema => {
        # Strip ALL runtime and infrastructure concerns from primitives
        let forbidden_runtime = [
          "x-familiar-service", "x-familiar-api", "x-familiar-queue",
          "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-node-type",
          "x-familiar-scaling", "x-familiar-deploy-config"
        ] in

        let pure_primitive = std.record.remove_all forbidden_runtime raw_schema in

        # Ensure required codegen metadata for primitives
        let with_codegen = if std.record.has_field "x-familiar-codegen-targets" pure_primitive then
          pure_primitive
        else
          pure_primitive & {
            "x-familiar-codegen-targets" = ["rust", "typescript"]
          } in

        with_codegen & {
          "x-familiar-layer" = "entity",
          "x-familiar-orthology" = "primitive_type_only",
          "x-familiar-type-category" = "primitive"
        }
      },

      # validate_pure_entity : Primitive -> ValidationResult
      # Validates that extracted primitive contains no runtime concerns
      # Ensures orthological purity - primitives must be pure fundamental types only
      validate_pure_entity = fun primitive_schema => {
        let runtime_indicators = [
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-deploy-config"
        ] in

        let has_runtime = std.array.any (fun field =>
          std.record.has_field field primitive_schema
        ) runtime_indicators in

        if has_runtime then
          { valid = false, errors = ["Primitive schemas cannot contain runtime concerns. Extract to infrastructure/topology layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_entity_structure : Primitive -> ValidationResult
      # Ensures primitive has proper fundamental type structure
      # Validates that primitive represents a meaningful fundamental type
      validate_entity_structure = fun primitive_schema => {
        # Must have primitive-like properties (fundamental JSON Schema types)
        let has_type = std.record.has_field "type" primitive_schema in
        let type_value = if has_type then primitive_schema.type else null in
        let is_primitive_type = has_type && std.array.elem type_value ["string", "number", "integer", "boolean", "null"] in
        let has_format = std.record.has_field "format" primitive_schema in
        let has_enum = std.record.has_field "enum" primitive_schema in

        if is_primitive_type || has_format || has_enum then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Primitive schemas must define fundamental type (string, number, integer, boolean, null) or have format/enum"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # PRIMITIVE HYDRATION - Pure Primitive Type Context
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    "_codegen" = (import "../global.ncl").hydration."_codegen" & {
      primitive_generation = true,
      type_foundation = true,
      fundamental_types = true
    },

    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "codegen.primitives",
      orthological_role = "primitive_definition"
    },

    "_observability" = (import "../global.ncl").hydration."_observability" & {
      primitive_usage_tracking = true,
      type_foundation_metrics = true,
      fundamental_type_metrics = true
    }
  }
}
