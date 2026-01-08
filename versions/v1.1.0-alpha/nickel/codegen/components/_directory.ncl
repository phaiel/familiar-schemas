/**
 * Components Module - Pure Component Data Governance
 *
 * Ensures schemas in codegen/components/ represent pure ECS COMPONENT data
 * structures that can be composed into entities.
 *
 * @orthology This module enforces pure component data structures
 * @guarantees NO runtime concerns, NO entity logic, NO composition wiring
 *
 * @example
 *   # Extract component data from mixed schema
 *   let component = (import "components/_directory.ncl").files._.extract_entity_data(raw_schema)
 *
 *   # Validate component purity
 *   let result = (import "components/_directory.ncl").files._.validate_pure_entity(component)
 */

{
  # ============================================================================
  # COMPONENT CONTRACTS - Component Data Rules
  # ============================================================================

  # Contract for all files in codegen/components/ - must be pure component data
  files = {
    _ : {
      # Implement EntityContract interface with type safety for components
      extract_from_raw = fun raw_schema => extract_entity_data raw_schema,
      validate_pure = fun schema => validate_pure_entity schema,
      get_layer = `entity,

      # extract_entity_data : Schema -> Component
      # Transforms mixed schemas into pure component data structures
      # Components are pure data that can be attached to entities
      extract_entity_data = fun raw_schema => {
        # Strip ALL runtime and infrastructure concerns from components
        let forbidden_runtime = [
          "x-familiar-service", "x-familiar-api", "x-familiar-queue",
          "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-node-type",
          "x-familiar-scaling", "x-familiar-deploy-config"
        ] in

        let pure_component = std.record.remove_all forbidden_runtime raw_schema in

        # Ensure required codegen metadata for components
        let with_codegen = if std.record.has_field "x-familiar-codegen-targets" pure_component then
          pure_component
        else
          pure_component & {
            "x-familiar-codegen-targets" = ["rust", "typescript"]
          } in

        with_codegen & {
          "x-familiar-layer" = "entity",
          "x-familiar-orthology" = "component_data_only",
          "x-familiar-component-type" = "ecs_component"
        }
      },

      # validate_pure_entity : Component -> ValidationResult
      # Validates that extracted component contains no runtime concerns
      # Ensures orthological purity - components must be pure ECS data only
      validate_pure_entity = fun component_schema => {
        let runtime_indicators = [
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",
          "x-familiar-monitoring-enabled", "x-familiar-deploy-config"
        ] in

        let has_runtime = std.array.any (fun field =>
          std.record.has_field field component_schema
        ) runtime_indicators in

        if has_runtime then
          { valid = false, errors = ["Component schemas cannot contain runtime concerns. Extract to infrastructure/topology layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_entity_structure : Component -> ValidationResult
      # Ensures component has proper ECS data structure
      # Validates that component represents a meaningful ECS component
      validate_entity_structure = fun component_schema => {
        # Must have component-like properties (ECS component pattern)
        let has_properties = std.record.has_field "properties" component_schema in
        let has_type = std.record.has_field "type" component_schema in
        let has_component_indicators = std.record.has_field "x-familiar-component-type" component_schema in

        if has_properties || has_type || has_component_indicators then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Component schemas must define data structure (properties, type, or x-familiar-component-type)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # COMPONENT HYDRATION - Pure Component Data Context
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    "_codegen" = (import "../global.ncl").hydration."_codegen" & {
      component_generation = true,
      ecs_patterns = true,
      composition_support = true
    },

    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "codegen.components",
      orthological_role = "component_data"
    },

    "_observability" = (import "../global.ncl").hydration."_observability" & {
      component_metrics = true,
      composition_tracking = true,
      ecs_usage_tracking = true
    }
  }
}
