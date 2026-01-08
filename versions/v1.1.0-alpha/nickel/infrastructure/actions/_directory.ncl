/**
 * Actions Infrastructure Module - Pure Compute Resources Governance
 *
 * Ensures schemas in infrastructure/actions/ represent pure COMPUTE RESOURCES
 * for action execution and processing capabilities.
 *
 * @orthology This module enforces pure compute specifications for actions
 * @guarantees NO business logic, NO data definitions, NO entity schemas
 *
 * @example
 *   # Extract compute resources from mixed schema
 *   let compute_resources = (import "actions/_directory.ncl").files._.extract_compute_resources(raw_schema)
 *
 *   # Validate compute resource purity
 *   let result = (import "actions/_directory.ncl").files._.validate_pure_compute(compute_resources)
 */

{
  # ============================================================================
  # ACTIONS INFRASTRUCTURE CONTRACTS - Compute Resource Rules
  # ============================================================================

  # Contract for all files in infrastructure/actions/ - must be pure compute resources
  files = {
    _ : {
      # Implement NodeContract interface with type safety for action infrastructure
      extract_compute_resources = fun raw_schema => extract_compute_resources raw_schema,
      validate_pure_compute = fun schema => validate_pure_compute schema,
      get_layer = `node,

      # extract_compute_resources : Schema -> ComputeResources
      # Transforms mixed schemas into pure compute resource specifications
      # Actions infrastructure focuses on execution environments and processing capabilities
      extract_compute_resources = fun raw_schema => {
        # Strip ALL business logic and data concerns from compute resources
        let forbidden_business = [
          "x-familiar-business-rules", "x-familiar-domain-events", "x-familiar-aggregates",
          "properties", "type", "enum", "format",  # No data definitions
          "x-familiar-codegen-targets", "x-familiar-component-type"  # No entity concerns
        ] in

        let pure_compute = std.record.remove_all forbidden_business raw_schema in

        # Ensure required infrastructure metadata for actions
        let with_infra = if std.record.has_field "x-familiar-node-type" pure_compute then
          pure_compute
        else
          pure_compute & {
            "x-familiar-node-type" = "action_execution"
          } in

        with_infra & {
          "x-familiar-layer" = "node",
          "x-familiar-orthology" = "action_compute_only"
        }
      },

      # validate_pure_compute : ComputeResources -> ValidationResult
      # Validates that extracted compute resources contain no business logic
      # Ensures orthological purity - compute resources must be pure infrastructure only
      validate_pure_compute = fun compute_schema => {
        let business_indicators = [
          "x-familiar-business-rules", "x-familiar-domain-events", "x-familiar-aggregates",
          "properties", "type", "enum", "format",  # No data definitions
          "x-familiar-codegen-targets", "x-familiar-component-type"  # No entity concerns
        ] in

        let has_business = std.array.any (fun field =>
          std.record.has_field field compute_schema
        ) business_indicators in

        if has_business then
          { valid = false, errors = ["Action infrastructure schemas cannot contain business logic. Extract to domain layer."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_compute_resources : ComputeResources -> ValidationResult
      # Ensures compute resources have proper infrastructure structure
      # Validates that compute resources represent meaningful action execution capabilities
      validate_compute_resources = fun compute_schema => {
        # Must have infrastructure-like properties (compute specifications)
        let has_node_type = std.record.has_field "x-familiar-node-type" compute_schema in
        let has_resources = std.record.has_field "x-familiar-resources" compute_schema in
        let has_capabilities = std.record.has_field "x-familiar-capabilities" compute_schema in
        let has_execution = std.record.has_field "x-familiar-execution-mode" compute_schema in

        if has_node_type || has_resources || has_capabilities || has_execution then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Action infrastructure schemas must define compute resources (node type, resources, capabilities, or execution mode)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # ACTIONS INFRASTRUCTURE HYDRATION - Pure Compute Resource Context
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    "_infrastructure" = (import "../_directory.ncl").hydration."_infrastructure" & {
      action_execution = true,
      processing_capabilities = true,
      workflow_orchestration = true
    },

    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "infrastructure.actions",
      orthological_role = "action_compute_resources"
    },

    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      action_usage_metrics = true,
      execution_monitoring = true,
      performance_tracking = true
    }
  }
}
