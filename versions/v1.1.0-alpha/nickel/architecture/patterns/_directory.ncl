/**
 * Patterns Architecture Module - Pure Topology Wiring Governance
 *
 * Ensures schemas in architecture/patterns/ represent pure TOPOLOGY WIRING
 * for architectural patterns and structural relationships.
 *
 * @orthology This module enforces pure wiring between entities and nodes
 * @guarantees NO data definitions, NO compute specs, NO business logic
 *
 * @example
 *   # Extract topology wiring from mixed schema
 *   let topology = (import "patterns/_directory.ncl").files._.extract_topology_wiring(raw_schema)
 *
 *   # Validate topology purity
 *   let result = (import "patterns/_directory.ncl").files._.validate_pure_topology(topology)
 */

{
  # ============================================================================
  # PATTERNS ARCHITECTURE CONTRACTS - Topology Wiring Rules
  # ============================================================================

  # Contract for all files in architecture/patterns/ - must be pure topology wiring
  files = {
    _ : {
      # Implement TopologyContract interface with type safety for pattern architecture
      extract_topology_wiring = fun raw_schema => extract_topology_wiring raw_schema,
      validate_pure_topology = fun schema => validate_pure_topology schema,
      get_layer = `topology,

      # extract_topology_wiring : Schema -> Topology
      # Transforms mixed schemas into pure topology wiring specifications
      # Patterns architecture focuses on structural relationships and architectural patterns
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

        # Ensure required architecture metadata for patterns
        let with_arch = if std.record.has_field "x-familiar-architectural-pattern" pure_topology then
          pure_topology
        else
          pure_topology & {
            "x-familiar-architectural-pattern" = "pattern_template"
          } in

        with_arch & {
          "x-familiar-layer" = "topology",
          "x-familiar-orthology" = "architectural_pattern_wiring"
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
          { valid = false, errors = ["Pattern architecture schemas cannot contain data, compute, or business concerns. Extract to appropriate layers."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_topology_bindings : Topology -> ValidationResult
      # Ensures topology has proper architectural structure
      # Validates that topology represents meaningful architectural patterns
      validate_topology_bindings = fun topology_schema => {
        # Must have topology-like properties (architectural patterns)
        let has_pattern = std.record.has_field "x-familiar-architectural-pattern" topology_schema in
        let has_layer = std.record.has_field "x-familiar-architectural-layer" topology_schema in
        let has_boundary = std.record.has_field "x-familiar-architectural-boundary" topology_schema in
        let has_relationships = std.record.has_field "x-familiar-relationships" topology_schema in

        if has_pattern || has_layer || has_boundary || has_relationships then
          { valid = true, errors = [], warnings = [] }
        else
          { valid = false, errors = ["Pattern architecture schemas must define topology structure (architectural pattern, layer, boundary, or relationships)"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # PATTERNS ARCHITECTURE HYDRATION - Pure Topology Wiring Context
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    "_architecture" = (import "../_directory.ncl").hydration."_architecture" & {
      pattern_templates = true,
      architectural_relationships = true,
      structural_wiring = true
    },

    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "architecture.patterns",
      orthological_role = "pattern_topology_wiring"
    },

    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      pattern_usage_metrics = true,
      architectural_monitoring = true,
      relationship_tracking = true
    }
  }
}
