/**
 * Tools Infrastructure Module - Pure Compute Resources Governance
 *
 * Ensures schemas in infrastructure/tools/ represent pure COMPUTE RESOURCES
 * for tool execution and capabilities.
 *
 * @orthology This module enforces pure compute specifications for tools
 * @guarantees NO business logic, NO data definitions, NO entity schemas
 *
 * @example
 *   # Extract compute resources from mixed schema
 *   let compute_resources = (import "tools/_directory.ncl").files._.extract_compute_resources(raw_schema)
 *
 *   # Validate compute resource purity
 *   let result = (import "tools/_directory.ncl").files._.validate_pure_compute(compute_resources)
 */

{
  # ============================================================================
  # TOOLS INFRASTRUCTURE CONTRACTS - Compute Resource Rules
  # ============================================================================

  # Contract for all files in infrastructure/tools/ - must be pure compute resources
  files = {
    _ : {
      # Implement NodeContract interface with type safety for tool infrastructure
      extract_compute_resources = fun raw_schema => extract_compute_resources raw_schema,
      validate_pure_compute = fun schema => validate_pure_compute schema,
      get_layer = `node,

      # extract_compute_resources : Schema -> ComputeResources
      # Transforms mixed schemas into pure compute resource specifications
      # Tools infrastructure focuses on execution environments and capabilities
      extract_compute_resources = fun raw_schema => {
        # Strip ALL business logic and data concerns from compute resources
        let forbidden_business = [
          "x-familiar-business-rules", "x-familiar-domain-events", "x-familiar-aggregates",
          "properties", "type", "enum", "format",  # No data definitions
          "x-familiar-codegen-targets", "x-familiar-component-type"  # No entity concerns
        ] in

        let pure_compute = std.record.remove_all forbidden_business raw_schema in

        # Ensure required infrastructure metadata for tools
        let with_infra = if std.record.has_field "x-familiar-node-type" pure_compute then
          pure_compute
        else
          pure_compute & {
            "x-familiar-node-type" = "tool_execution"
          } in

        with_infra & {
          "x-familiar-layer" = "node",
          "x-familiar-orthology" = "tool_compute_only"
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
          { valid = false, errors = ["Tool infrastructure schemas cannot contain business logic. Extract to domain layer."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_compute_resources : ComputeResources -> ValidationResult
      # Ensures compute resources have proper infrastructure structure
      # Validates that compute resources represent meaningful tool execution capabilities
      validate_compute_resources = fun compute_schema => {
        # Must have infrastructure-like properties (compute specifications)
        let has_node_type = std.record.has_field "x-familiar-node-type" compute_schema in
        let has_resources = std.record.has_field "x-familiar-resources" compute_schema in
        let has_capabilities = std.record.has_field "x-familiar-capabilities" compute_schema in
        let has_contract = std.record.has_field "x-familiar-contract" compute_schema in

        if has_node_type || has_resources || has_capabilities || has_contract then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Tool infrastructure schemas must define compute resources (node type, resources, capabilities, or contract)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # TOOLS INFRASTRUCTURE HYDRATION - Pure Compute Resource Context
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    "_infrastructure" = (import "../_directory.ncl").hydration."_infrastructure" & {
      tool_execution = true,
      capability_management = true,
      resource_allocation = true
    },

    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "infrastructure.tools",
      orthological_role = "tool_compute_resources"
    },

    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      tool_usage_metrics = true,
      capability_monitoring = true,
      execution_tracking = true
    }
  },

  # ============================================================================
  # TOOLS-SPECIFIC VALIDATION RULES
  # ============================================================================

  # Tools cannot connect to familiar-worker or familiar-daemon nodes
  forbidden_connections = ["familiar-worker", "familiar-daemon"],

  # Validate that tools don't connect to forbidden nodes
  validate_tool_connections = fun tool_schema =>
    if std.record.has_field "x-familiar-connections" tool_schema then
      let connections = tool_schema."x-familiar-connections" in
      if std.is_array connections then
        let has_forbidden = std.array.any (fun conn =>
          std.array.elem conn forbidden_connections
        ) connections in

        if has_forbidden then
          { valid = false, errors = ["Tools cannot connect to familiar-worker or familiar-daemon nodes"], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      else
        { valid = true, errors = [], warnings = [] }  # No array connections, allow
    else
      { valid = true, errors = [], warnings = [] }  # No connections specified, allow
}
