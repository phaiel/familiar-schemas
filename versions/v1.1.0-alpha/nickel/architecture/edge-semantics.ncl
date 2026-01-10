# Architecture Edge Semantics
#
# Inherits global edge types and specializes them for architectural/topology concerns.
# Defines how edges behave in the context of system architecture and data flow.
#
# @category architecture
# @graph-value high - specializes edges for topology relationships
# @validation context-aware - topology-specific edge constraints

let global_edges = import "../extensions/edge-type.ncl" in

{
  # ============================================================================
  # INHERITED & SPECIALIZED EDGE TYPES
  # ============================================================================

  # Define architecture-specific edge types for topology concerns
  edge_types = {
    # Add architecture-specific edge types
    entity_node_binding = {
      directionality = "directed",
      cardinality = "many_to_one",
      transitivity = false,
      cyclicity = "acyclic",
      data_flow = "deployment",
      strength = "strong",
      context = "topology_deployment",
      compilation_implications = {
        generate_deployment_mapping = true,
        generate_service_discovery = true,
        generate_load_balancing = false
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true
      }
    },

    service_mesh_binding = {
      directionality = "bidirectional",
      cardinality = "many_to_many",
      transitivity = false,
      cyclicity = "cyclic_allowed",
      data_flow = "service_discovery",
      strength = "medium",
      context = "service_orchestration",
      compilation_implications = {
        generate_service_mesh_config = true,
        generate_health_checks = true,
        generate_circuit_breakers = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = true,
        topological_sort_required = false
      }
    },

    # Define architecture-specific edge types
    depends_on = {
      directionality = "directed",
      cardinality = "many_to_one",
      transitivity = true,
      cyclicity = "acyclic",
      data_flow = "none",
      strength = "strong",
      context = "architecture_dependencies",
      compilation_implications = {
        generate_topology_aware_injection = true,
        generate_deployment_ordering = true,
        generate_resource_dependencies = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true,
        cycle_detection_critical = true
      }
    },

    orchestrates = {
      directionality = "directed",
      cardinality = "one_to_many",
      transitivity = false,
      cyclicity = "acyclic",
      data_flow = "control_flow",
      strength = "strong",
      context = "workflow_orchestration",
      compilation_implications = {
        generate_workflow_engine = true,
        generate_state_management = true,
        generate_error_propagation = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true,
        workflow_cycles_allowed = false
      }
    },

    communicates_with = {
      directionality = "bidirectional",
      cardinality = "many_to_many",
      transitivity = false,
      cyclicity = "cyclic_allowed",
      data_flow = "bidirectional",
      strength = "medium",
      context = "inter_service_communication",
      compilation_implications = {
        generate_api_clients = true,
        generate_message_queues = true,
        generate_protocol_buffers = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = true,
        topological_sort_required = false,
        communication_cycles_allowed = true
      }
    },

    deploys_to = {
      context = "infrastructure_deployment",
      compilation_implications = {
        generate_deployment_manifests = true,
        generate_infrastructure_as_code = true,
        generate_scaling_policies = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true
      }
    }
  },

  # ============================================================================
  # ARCHITECTURE-SPECIFIC VALIDATION RULES
  # ============================================================================

  validation_rules = {
    # Edge combinations allowed in architecture schemas
    allowed_edge_combinations = [
      # Entity to Node bindings (core topology)
      {
        source_kind = "entity",
        target_kind = "node",
        allowed_edges = ["entity_node_binding", "deploys_to"]
      },

      # Node to Node orchestration
      {
        source_kind = "node",
        target_kind = "node",
        allowed_edges = ["orchestrates", "communicates_with", "depends_on"]
      },

      # System to System communication
      {
        source_kind = "system",
        target_kind = "system",
        allowed_edges = ["orchestrates", "communicates_with", "depends_on"]
      },

      # Component to Resource dependencies
      {
        source_kind = "component",
        target_kind = "resource",
        allowed_edges = ["depends_on", "uses"]
      }
    ],

    # Prohibited patterns in architecture
    prohibited_patterns = [
      # Entities shouldn't communicate directly (must go through nodes)
      {
        source_kind = "entity",
        target_kind = "entity",
        prohibited_edges = ["communicates_with"]
      },

      # Systems shouldn't bind directly to entities (use nodes)
      {
        source_kind = "system",
        target_kind = "entity",
        prohibited_edges = ["entity_node_binding"]
      }
    ],

    # Topology-specific constraints
    topology_constraints = {
      # Maximum depth for dependency chains
      max_dependency_depth = 10,

      # Require explicit relationship types
      allow_implicit_relationships = false,

      # Enforce acyclic graphs where specified
      enforce_acyclic_topology = true
    }
  },

  # ============================================================================
  # ARCHITECTURE-SPECIFIC FUNCTIONS
  # ============================================================================

  functions = {
    # Validate edge compatibility in architecture context
    validate_architecture_edge = fun source_kind target_kind edge_type =>
      let allowed_combinations = validation_rules.allowed_edge_combinations in
      let matching_rules = std.array.filter (fun rule =>
        rule.source_kind == source_kind && rule.target_kind == target_kind
      ) allowed_combinations in

      if std.array.is_empty matching_rules then
        false  # No rules allow this source->target combination
      else
        # Check if edge_type is allowed for this source->target
        std.array.any (fun rule =>
          std.array.elem edge_type rule.allowed_edges
        ) matching_rules,

    # Get edge metadata with architecture specialization
    get_architecture_edge_metadata = fun edge_type =>
      if std.record.has_field edge_type edge_types then
        std.record.get edge_type edge_types
      else
        global_edges.functions.get_edge_characteristics edge_type,

    # Analyze topology connectivity using edge semantics
    analyze_topology_connectivity = fun edges =>
      let edge_types_used = std.array.map (fun edge => edge.edge_type) edges in
      let unique_edge_types = std.array.fold_left (fun acc type =>
        if std.array.elem type acc then acc else acc @ [type]
      ) [] edge_types_used in

      {
        dependency_depth = calculate_dependency_depth edges,
        communication_patterns = analyze_communication_patterns unique_edge_types,
        orchestration_complexity = calculate_orchestration_complexity edges,
        potential_cycles = detect_potential_cycles edges edge_types,
        optimization_opportunities = identify_optimization_opportunities edges
      },

    # Calculate dependency depth (longest chain)
    calculate_dependency_depth = fun edges =>
      # Simplified depth calculation - in practice would need graph analysis
      let depends_edges = std.array.filter (fun edge => edge.edge_type == "depends_on") edges in
      std.array.length depends_edges,

    # Analyze communication patterns
    analyze_communication_patterns = fun edge_types =>
      let has_service_mesh = std.array.elem "service_mesh_binding" edge_types in
      let has_direct_comm = std.array.elem "communicates_with" edge_types in

      if has_service_mesh && has_direct_comm then
        "hybrid_service_mesh"
      else if has_service_mesh then
        "service_mesh_only"
      else if has_direct_comm then
        "direct_communication"
      else
        "no_communication",

    # Calculate orchestration complexity
    calculate_orchestration_complexity = fun edges =>
      let orchestration_edges = std.array.filter (fun edge =>
        std.array.elem edge.edge_type ["orchestrates", "entity_node_binding"]
      ) edges in
      std.array.length orchestration_edges,

    # Detect potential cycles using edge semantics
    detect_potential_cycles = fun edges edge_types =>
      let cyclic_allowed_edges = std.array.filter (fun edge_type =>
        let edge_metadata = std.record.get edge_type edge_types in
        edge_metadata.cyclicity == "cyclic_allowed"
      ) (std.record.fields edge_types) in

      let potentially_cyclic_edges = std.array.filter (fun edge =>
        std.array.elem edge.edge_type cyclic_allowed_edges
      ) edges in

      if std.array.is_empty potentially_cyclic_edges then
        []
      else
        std.array.map (fun edge => {
          edge = edge,
          risk_level = "medium",
          mitigation = "add_explicit_cycle_breakers"
        }) potentially_cyclic_edges,

    # Identify optimization opportunities
    identify_optimization_opportunities = fun edges =>
      let opportunities = [] in

      # Check for redundant paths
      let redundant_paths = detect_redundant_paths edges in
      let opportunities = if std.array.is_empty redundant_paths then
        opportunities
      else
        opportunities @ [{
          type = "redundant_paths",
          description = "Multiple paths between same nodes detected",
          edges = redundant_paths,
          optimization = "consolidate_paths"
        }] in

      # Check for potential batching opportunities
      let batching_ops = detect_batching_opportunities edges in
      let opportunities = if std.array.is_empty batching_ops then
        opportunities
      else
        opportunities @ [{
          type = "batching_opportunities",
          description = "Multiple similar edges could be batched",
          edges = batching_ops,
          optimization = "implement_edge_batching"
        }] in

      opportunities,

    # Helper: Detect redundant paths (simplified)
    detect_redundant_paths = fun edges =>
      # In practice, would need graph analysis to detect multiple paths
      [],

    # Helper: Detect batching opportunities
    detect_batching_opportunities = fun edges =>
      let depends_edges = std.array.filter (fun edge => edge.edge_type == "depends_on") edges in
      if std.array.length depends_edges > 3 then
        depends_edges  # If many dependencies, suggest batching
      else
        []
  },

  # ============================================================================
  # COMPILATION INTEGRATION
  # ============================================================================

  compilation = {
    # Generate code from architectural edges
    generate_from_architecture_edges = fun edges graph_context =>
      # Group edges by type for batch processing (simplified - no group_by available)
      let edge_types = std.array.map (fun edge => edge.edge_type) edges in
      let unique_types = std.array.uniq edge_types in

      # Generate architecture-specific code for each edge type
      std.array.map (fun edge_type =>
        let edge_list = std.array.filter (fun edge => edge.edge_type == edge_type) edges in
        let metadata = functions.get_architecture_edge_metadata edge_type in
        {
          edge_type = edge_type,
          edges = edge_list,
          metadata = metadata,
          generated_code = "placeholder_code"
        }
      ) unique_types,

    # Generate deployment topology
    generate_deployment_topology = fun edges =>
      let entity_bindings = std.array.filter (fun edge =>
        edge.edge_type == "entity_node_binding"
      ) edges in

      # Manual grouping by target_ref (simplified)
      let target_refs = std.array.map (fun edge => edge.target_ref) entity_bindings in
      let unique_targets = std.array.uniq target_refs in

      std.array.map (fun node_ref => {
        node = node_ref,
        entities = std.array.map (fun edge => edge.source_schema)
          (std.array.filter (fun edge => edge.target_ref == node_ref) entity_bindings),
        deployment_strategy = "rolling",
        health_checks = true
      }) unique_targets,

    # Generate service mesh configuration
    generate_service_mesh_config = fun edges =>
      let communication_edges = std.array.filter (fun edge =>
        edge.edge_type == "communicates_with"
      ) edges in

      {
        services = std.array.uniq (
          std.array.fold_left (fun acc edge => acc @ [edge.source_schema, edge.target_ref]) [] communication_edges
        ),
        communication_patterns = std.array.map (fun edge => {
          source = edge.source_schema,
          target = edge.target_ref,
          protocol = "http",
          timeout = "30s"
        }) communication_edges
      }
  }
}