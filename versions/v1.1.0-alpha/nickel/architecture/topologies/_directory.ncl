/**
 * Topologies Architecture Module - Pure Topology Wiring Governance
 *
 * Ensures schemas in architecture/topologies/ represent pure TOPOLOGY WIRING
 * for system topologies and runtime relationships.
 *
 * @orthology This module enforces pure wiring between entities and nodes
 * @guarantees NO data definitions, NO compute specs, NO business logic
 *
 * @example
 *   # Extract topology wiring from mixed schema
 *   let topology = (import "topologies/_directory.ncl").files._.extract_topology_wiring(raw_schema)
 *
 *   # Validate topology purity
 *   let result = (import "topologies/_directory.ncl").files._.validate_pure_topology(topology)
 */

{
  # ============================================================================
  # TOPOLOGIES ARCHITECTURE CONTRACTS - Topology Wiring Rules
  # ============================================================================

  # Contract for all files in architecture/topologies/ - must be pure topology wiring
  files = {
    _ : {
      # Implement TopologyContract interface with type safety for topology architecture
      extract_topology_wiring = fun raw_schema => extract_topology_wiring raw_schema,
      validate_pure_topology = fun schema => validate_pure_topology schema,
      get_layer = `topology,

      # extract_topology_wiring : Schema -> Topology
      # Transforms mixed schemas into pure topology wiring specifications
      # Topologies architecture focuses on runtime relationships and data flow
      extract_topology_wiring = fun raw_schema => {
        # Strip ALL data, compute, and business concerns from topology wiring
        let forbidden_concerns = [
          "x-familiar-business-rules", "x-familiar-domain-events", "x-familiar-aggregates",
          "properties", "type", "enum", "format",  # No data definitions
          "x-familiar-codegen-targets", "x-familiar-component-type",  # No entity concerns
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",  # No compute concerns
          "x-familiar-scaling", "x-familiar-deploy-config"  # No infrastructure concerns
        ] in

        let pure_topology = std.record.remove_all forbidden_concerns raw_schema in

        # Ensure required architecture metadata for topologies
        let with_arch = if std.record.has_field "x-familiar-topology-type" pure_topology then
          pure_topology
        else
          pure_topology & {
            "x-familiar-topology-type" = "runtime_topology"
          } in

        with_arch & {
          "x-familiar-layer" = "topology",
          "x-familiar-orthology" = "topology_wiring"
        }
      },

      # validate_pure_topology : Topology -> ValidationResult
      # Validates that extracted topology contains no data/compute/business concerns
      # Ensures orthological purity - topology must be pure wiring only
      validate_pure_topology = fun topology_schema => {
        let concern_indicators = [
          "x-familiar-business-rules", "x-familiar-domain-events", "x-familiar-aggregates",
          "properties", "type", "enum", "format",  # No data definitions
          "x-familiar-codegen-targets", "x-familiar-component-type",  # No entity concerns
          "x-familiar-service", "x-familiar-node-type", "x-familiar-resources",  # No compute concerns
          "x-familiar-scaling", "x-familiar-deploy-config"  # No infrastructure concerns
        ] in

        let has_concerns = std.array.any (fun field =>
          std.record.has_field field topology_schema
        ) concern_indicators in

        if has_concerns then
          { valid = false, errors = ["Topology architecture schemas cannot contain data, compute, or business concerns. Extract to appropriate layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_topology_bindings : Topology -> ValidationResult
      # Ensures topology has proper architectural structure
      # Validates that topology represents meaningful runtime relationships
      validate_topology_bindings = fun topology_schema => {
        # Must have topology-like properties (runtime relationships)
        let has_topology_type = std.record.has_field "x-familiar-topology-type" topology_schema in
        let has_bindings = std.record.has_field "bindings" topology_schema in
        let has_relationships = std.record.has_field "x-familiar-relationships" topology_schema in
        let has_data_flow = std.record.has_field "x-familiar-data-flow" topology_schema in

        if has_topology_type || has_bindings || has_relationships || has_data_flow then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Topology architecture schemas must define topology structure (topology type, bindings, relationships, or data flow)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # TOPOLOGIES ARCHITECTURE HYDRATION - Pure Topology Wiring Context
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    "_architecture" = (import "../_directory.ncl").hydration."_architecture" & {
      runtime_topologies = true,
      relationship_mapping = true,
      data_flow_wiring = true
    },

    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "architecture.topologies",
      orthological_role = "topology_wiring"
    },

    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      topology_metrics = true,
      relationship_monitoring = true,
      data_flow_tracking = true
    }
  }
}
