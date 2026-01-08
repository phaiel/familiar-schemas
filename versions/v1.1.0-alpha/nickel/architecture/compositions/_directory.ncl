/**
 * Compositions Architecture Module - Pure Topology Wiring Governance
 *
 * Ensures schemas in architecture/compositions/ represent pure TOPOLOGY WIRING
 * for system compositions and component assemblies.
 *
 * @orthology This module enforces pure wiring between system components
 * @guarantees NO data definitions, NO compute specs, NO business logic
 *
 * @example
 *   # Extract topology wiring from mixed schema
 *   let topology = (import "compositions/_directory.ncl").files._.extract_topology_wiring(raw_schema)
 *
 *   # Validate topology purity
 *   let result = (import "compositions/_directory.ncl").files._.validate_pure_topology(topology)
 */

{
  # ============================================================================
  # COMPOSITIONS ARCHITECTURE CONTRACTS - Topology Wiring Rules
  # ============================================================================

  # Contract for all files in architecture/compositions/ - must be pure topology wiring
  files = {
    _ : {
      # Implement TopologyContract interface with type safety for composition architecture
      extract_topology_wiring = fun raw_schema => extract_topology_wiring raw_schema,
      validate_pure_topology = fun schema => validate_pure_topology schema,
      get_layer = `topology,

      # extract_topology_wiring : Schema -> Topology
      # Transforms mixed schemas into pure topology wiring specifications
      # Compositions architecture focuses on component assemblies and system wiring
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

        # Ensure required architecture metadata for compositions
        let with_arch = if std.record.has_field "x-familiar-composition-type" pure_topology then
          pure_topology
        else
          pure_topology & {
            "x-familiar-composition-type" = "system_assembly"
          } in

        with_arch & {
          "x-familiar-layer" = "topology",
          "x-familiar-orthology" = "composition_wiring"
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
          { valid = false, errors = ["Composition architecture schemas cannot contain data, compute, or business concerns. Extract to appropriate layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_topology_bindings : Topology -> ValidationResult
      # Ensures topology has proper architectural structure
      # Validates that topology represents meaningful system compositions
      validate_topology_bindings = fun topology_schema => {
        # Must have topology-like properties (system compositions)
        let has_composition = std.record.has_field "x-familiar-composition-type" topology_schema in
        let has_bindings = std.record.has_field "bindings" topology_schema in
        let has_assembly = std.record.has_field "x-familiar-assembly-rules" topology_schema in
        let has_dependencies = std.record.has_field "x-familiar-dependencies" topology_schema in

        if has_composition || has_bindings || has_assembly || has_dependencies then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Composition architecture schemas must define topology structure (composition type, bindings, assembly rules, or dependencies)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # COMPOSITIONS ARCHITECTURE HYDRATION - Pure Topology Wiring Context
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    "_architecture" = (import "../_directory.ncl").hydration."_architecture" & {
      system_compositions = true,
      component_assemblies = true,
      dependency_wiring = true
    },

    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "architecture.compositions",
      orthological_role = "composition_topology_wiring"
    },

    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      composition_metrics = true,
      assembly_monitoring = true,
      dependency_tracking = true
    }
  }
}
