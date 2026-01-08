/**
 * Architecture Module - Pure Topology/Wiring Governance
 *
 * Ensures schemas in architecture/ represent TOPOLOGIES: wiring relationships
 * between entities (data) and nodes (compute) that define data flow patterns.
 *
 * @orthology This module enforces the "How" - relationships and data flows
 * @guarantees NO data definitions, NO compute specs, PURE wiring/topology only
 * @topology Entity-to-node mappings, service meshes, data flow graphs
 *
 * @example
 *   # Extract topology from service bindings
 *   let topology = (import "architecture/_directory.ncl").files._.extract_topology_wiring(raw_schema)
 *
 *   # Validate topology purity
 *   let result = (import "architecture/_directory.ncl").files._.validate_pure_topology(topology)
 */

(import "../global.ncl") & {
  # ============================================================================
  # ARCHITECTURE CONTRACTS - Topology Wiring Rules
  # ============================================================================

  # Import type system from global
  let global = import "../global.ncl" in

  # Contract for all files in architecture/ - must be pure topology wiring
  files = {
    _ : {
      # Implement TopologyContract interface with type safety
      extract_from_raw = fun raw_schema => extract_topology_wiring raw_schema,
      validate_pure = fun schema => validate_pure_topology schema,
      get_layer = `topology,

      # extract_topology_wiring : Schema -> Topology
      # Transforms x-familiar-service bindings into topology wiring specifications
      # Converts runtime service references into architectural relationship graphs
      extract_topology_wiring = fun raw_schema => {
        # Extract relationship/service bindings as topology wiring
        let topology_bindings = if std.record.has_field "x-familiar-service" raw_schema then
          let service_ref = raw_schema."x-familiar-service" in
          [{
            entity = {"$ref": raw_schema."$id"},
            node = service_ref."$ref" | service_ref,
            relationship = "processes",
            extracted_from = raw_schema."$id"
          }]
        else
          [] in

        {
          "x-familiar-kind" = "topology",
          "x-familiar-topology-type" = "entity_node_binding",
          "x-familiar-layer" = "topology",
          "x-familiar-orthology" = "wiring_only",
          "bindings" = topology_bindings,
          "x-familiar-extracted-from" = raw_schema."$id" | "unknown"
        }
      },

      # synthesize_topology : Array<EntityRef> -> Array<NodeRef> -> Topology
      # Creates complete topology wiring from separate entity and node specifications
      # Generates all possible entity-node relationships for architectural composition
      synthesize_topology = fun entity_refs node_refs => {
        # Create wiring between all entity-node combinations
        let all_bindings = std.array.flat_map
          (fun entity_ref =>
            std.array.map
              (fun node_ref => {
                entity = entity_ref,
                node = node_ref,
                relationship = "processes",  # Default relationship
                protocol = "inferred",
                data_flow = "bidirectional"
              })
              node_refs
          )
          entity_refs in

        {
          "x-familiar-kind" = "topology",
          "x-familiar-topology-type" = "synthesized",
          "x-familiar-layer" = "topology",
          "x-familiar-orthology" = "wiring_only",
          "bindings" = all_bindings,
          "x-familiar-synthesis-timestamp" = "generated"
        }
      },

      # validate_pure_topology : Topology -> ValidationResult
      # Type-safe validation returning structured ValidationResult
      # Validates that topology contains only wiring relationships (no data/compute specs)
      validate_pure_topology = fun topology_schema => {
        let data_indicators = [
          "properties", "type", "required",
          "x-familiar-codegen-targets", "x-familiar-business-rules"
        ] in

        let compute_indicators = [
          "x-familiar-resources", "x-familiar-scaling", "x-familiar-deploy-config",
          "x-familiar-node-type", "x-familiar-monitoring-enabled"
        ] in

        let has_data = std.array.any (fun field =>
          std.record.has_field field topology_schema
        ) data_indicators in

        let has_compute = std.array.any (fun field =>
          std.record.has_field field topology_schema
        ) compute_indicators in

        if has_data then
          { valid = false, errors = ["Topology schemas cannot contain data definitions. Extract to codegen layer."], warnings = [] }
        else if has_compute then
          { valid = false, errors = ["Topology schemas cannot contain compute specifications. Extract to infrastructure layer."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_topology_bindings : Topology -> ValidationResult
      # Ensures topology schema has proper binding structure and relationships
      # Validates that topology represents meaningful architectural connections
      validate_topology_bindings = fun topology_schema => {
        if std.record.has_field "bindings" topology_schema then
          let bindings = topology_schema."bindings" in
          if std.is_array bindings && std.array.length bindings > 0 then
            # Validate each binding has required fields
            let valid_bindings = std.array.all (fun binding =>
              std.record.has_field "entity" binding &&
              std.record.has_field "node" binding &&
              std.record.has_field "relationship" binding
            ) bindings in

            if valid_bindings then
              { valid = true, errors = [], warnings = [] }
            else
              { valid = false, errors = ["Topology bindings must have entity, node, and relationship fields"], warnings = [] }
          else
            { valid = false, errors = ["Topology schemas must have non-empty bindings array"], warnings = [] }
        else
          { valid = false, errors = ["Topology schemas must define bindings"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # ERROR RECOVERY MECHANISMS
  # ============================================================================

  # recover_topology_errors : Topology -> ErrorRecovery
  # Attempt to recover from common topology validation errors
  recover_topology_errors = fun topology_schema =>
    (import "../global.ncl").try_recover
      (fun => validate_pure_topology topology_schema)
      ["add_missing_bindings", "fix_binding_structure", "infer_relationships"]
      (fun error =>
        match error {
          "Topology schemas must define bindings" =>
            # Recovery: Add empty bindings array
            `recovered {
              result = topology_schema & { "bindings" = [] },
              applied_fixes = ["Added missing bindings array"]
            },
          "Topology bindings must have entity, node, and relationship fields" =>
            # Recovery: Attempt to fix binding structure
            let fixed_bindings = fix_binding_structure topology_schema.bindings in
            `recovered {
              result = topology_schema & { "bindings" = fixed_bindings },
              applied_fixes = ["Fixed binding structure"]
            },
          _ =>
            # Unrecoverable error
            `unrecoverable {
              errors = [{
                field = "topology",
                expected = "Valid topology structure",
                actual = std.to_string error,
                suggestion = "Check topology bindings and structure",
                layer = `topology,
                severity = `error
              }]
            }
        }
      ),

  # fix_binding_structure : Array<_> -> Array<_>
  # Attempt to fix malformed topology bindings
  fix_binding_structure = fun bindings =>
    std.array.map
      (fun binding =>
        if std.record.has_field "entity" binding &&
           std.record.has_field "node" binding then
          # Has required fields, ensure relationship
          if std.record.has_field "relationship" binding then
            binding
          else
            binding & { "relationship" = "processes" }
        else
          # Missing required fields, create minimal binding
          {
            "entity" = binding.entity | "unknown_entity",
            "node" = binding.node | "unknown_node",
            "relationship" = "processes"
          }
      )
      bindings,

  # ============================================================================
  # CONTRACT TESTING FRAMEWORK
  # ============================================================================

  # Test cases for topology contract validation
  test_cases = [
    # Valid topology test
    {
      name = "valid_topology_schema",
      input = {
        "x-familiar-kind" = "topology",
        "x-familiar-topology-type" = "entity_node_binding",
        "x-familiar-description" = "Valid topology",
        "bindings" = [{
          "entity" = { "$ref" = "../../codegen/entities/User.json" },
          "node" = { "$ref" = "../../infrastructure/nodes/ApiService.json" },
          "relationship" = "serves"
        }]
      },
      expected_output = {
        "x-familiar-kind" = "topology",
        "x-familiar-topology-type" = "entity_node_binding",
        "x-familiar-description" = "Valid topology",
        "bindings" = [{
          "entity" = { "$ref" = "../../codegen/entities/User.json" },
          "node" = { "$ref" = "../../infrastructure/nodes/ApiService.json" },
          "relationship" = "serves"
        }],
        "x-familiar-layer" = "topology",
        "x-familiar-orthology" = "wiring_only"
      },
      expected_errors = [],
      description = "Valid topology schema should pass extraction and validation"
    },

    # Topology with forbidden data/compute
    {
      name = "topology_with_data",
      input = {
        "x-familiar-kind" = "topology",
        "x-familiar-description" = "Topology with forbidden data",
        "bindings" = [{
          "entity" = { "$ref" = "../../codegen/entities/User.json" },
          "node" = { "$ref" = "../../infrastructure/nodes/ApiService.json" },
          "relationship" = "serves"
        }],
        "properties" = { "user_id" = { "type" = "string" } }  # FORBIDDEN
      },
      expected_output = {
        "x-familiar-kind" = "topology",
        "x-familiar-description" = "Topology with forbidden data",
        "bindings" = [{
          "entity" = { "$ref" = "../../codegen/entities/User.json" },
          "node" = { "$ref" = "../../infrastructure/nodes/ApiService.json" },
          "relationship" = "serves"
        }],
        "x-familiar-layer" = "topology",
        "x-familiar-orthology" = "wiring_only"
      },
      expected_errors = ["Topology schemas cannot contain data definitions. Extract to codegen layer."],
      description = "Topology with data definitions should fail validation"
    }
  ],

  # run_tests : _ -> Array<ContractTestResult>
  # Execute all contract tests for this layer
  run_tests = fun =>
    (import "../global.ncl").run_contract_tests files._ test_cases,


  # ============================================================================
  # ARCHITECTURE HYDRATION - Topology Wiring Context
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Topology-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "architecture",
      orthological_role = "the_how"      # Defines HOW entities connect to nodes
    },

    # Topology-focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = true,            # Topology metrics are crucial
      topology_health_tracking = true,
      wiring_validation_events = true,
      architectural_compliance_tracking = true
    },

    # Topology-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "topology",
      wiring_validation = true,
      relationship_enforcement = true,
      topology_resolution = "runtime"
    },

    # Architecture-specific runtime configuration
    "_architecture" = {
      pattern_enforcement = "strict",
      boundary_validation = "continuous",
      layer_separation = "enforced",
      cross_cutting_weaving = "aspect_oriented",
      orthology_level = "topology",
      data_definition_free = true,      # Guarantee: no data schemas in topology layer
      compute_spec_free = true          # Guarantee: no compute specs in topology layer
    },

    "_topology" = {
      wiring_enforcement = "strict",
      relationship_validation = "continuous",
      data_flow_tracking = true,
      service_mesh_integration = true,
      observability_injection = true
    }
  },


  # ============================================================================
  # SUBDIRECTORY ROUTING
  # ============================================================================

  # Route to appropriate architecture subdirectory
  route_subdirectory = fun subcategory =>
    if subcategory == "patterns" then
      import "patterns/_directory.ncl"
    else if subcategory == "topologies" then
      import "topologies/_directory.ncl"
    else if subcategory == "compositions" then
      import "compositions/_directory.ncl"
    else
      # Default architecture validation
      {
        validate = fun schema_content => {
          valid = true,
          errors = [],
          warnings = ["Unknown architecture subcategory: ${subcategory}"]
        }
      }
}
