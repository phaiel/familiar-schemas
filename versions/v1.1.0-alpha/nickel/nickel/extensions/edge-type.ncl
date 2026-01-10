# x-familiar-edge-type Extension
#
# Defines edge relationship semantics in the graph structure.
# Enables typed edge classification in slotmap+petgraph for the compiler.
#
# @category architecture
# @graph-value high - provides edge semantics for petgraph relationships
# @validation context-dependent - for topology relationships

{
  # ============================================================================
  # EXTENSION METADATA
  # ============================================================================

  extension = {
    name = "x-familiar-edge-type",
    description = "Edge relationship type classification for graph topology",
    category = "architecture",
    required = false,
    version = "1.0.0",
    context_required = ["topology"]  # Only for topology schemas
  },

  # ============================================================================
  # CONTRACT DEFINITION
  # ============================================================================

  contract = std.contract.custom (fun label value =>
    let valid_edge_types = [
      "depends_on", "communicates_with", "contains", "implements",
      "extends", "aggregates", "orchestrates", "transforms",
      "references", "configures", "deploys_to", "routes_to"
    ] in
    if std.array.elem value valid_edge_types then value
    else std.contract.blame_with label "x-familiar-edge-type must be one of: depends_on, communicates_with, contains, implements, extends, aggregates, orchestrates, transforms, references, configures, deploys_to, routes_to. Got: ${std.to_string value}"
  ),

  # ============================================================================
  # MERGE STRATEGIES
  # ============================================================================

  merge_strategies = {
    override = fun base override => override,
    compose = fun base override => override,  # Edge types typically don't compose
    validate = fun base override =>
      if contract.validate override then override
      else std.contract.blame_with "merge" "Invalid x-familiar-edge-type in merge: ${std.to_string override}"
  },

  # ============================================================================
  # DEFAULTS & METADATA
  # ============================================================================

  defaults = {
    validation = true
  },

  metadata = {
    required = false,
    examples = ["depends_on", "communicates_with", "contains"],
    since = "v1.0.0",
    graph_impact = "edge_semantics"
  },

  # ============================================================================
  # FUNCTION LIBRARY
  # ============================================================================

  functions = {
    # Validate edge type
    validate_edge_type = fun value =>
      std.array.elem value [
        "depends_on", "communicates_with", "contains", "implements",
        "extends", "aggregates", "orchestrates", "transforms",
        "references", "configures", "deploys_to", "routes_to"
      ],

    # Get edge characteristics for graph traversal and optimization
    get_edge_characteristics = fun edge_type =>
      if edge_type == "depends_on" then {
        directionality = "directed",
        cardinality = "many_to_one",
        transitivity = true,           # A depends on B, B depends on C → A depends on C
        cyclicity = "acyclic",         # No circular dependencies
        data_flow = "none",            # No data flows through this edge
        strength = "strong"            # Breaking this edge breaks the system
      } else if edge_type == "communicates_with" then {
        directionality = "bidirectional",
        cardinality = "many_to_many",
        transitivity = false,
        cyclicity = "cyclic_allowed",   # Services can communicate in cycles
        data_flow = "bidirectional",
        strength = "medium"
      } else if edge_type == "contains" then {
        directionality = "directed",
        cardinality = "one_to_many",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "containment",
        strength = "strong"
      } else if edge_type == "implements" then {
        directionality = "directed",
        cardinality = "many_to_one",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "inheritance",
        strength = "strong"
      } else if edge_type == "extends" then {
        directionality = "directed",
        cardinality = "one_to_one",
        transitivity = true,
        cyclicity = "acyclic",
        data_flow = "extension",
        strength = "medium"
      } else if edge_type == "aggregates" then {
        directionality = "directed",
        cardinality = "one_to_many",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "aggregation",
        strength = "medium"
      } else if edge_type == "orchestrates" then {
        directionality = "directed",
        cardinality = "one_to_many",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "control_flow",
        strength = "strong"
      } else if edge_type == "transforms" then {
        directionality = "directed",
        cardinality = "one_to_one",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "transformation",
        strength = "medium"
      } else if edge_type == "references" then {
        directionality = "directed",
        cardinality = "many_to_one",
        transitivity = false,
        cyclicity = "cyclic_allowed",
        data_flow = "reference",
        strength = "weak"
      } else if edge_type == "configures" then {
        directionality = "directed",
        cardinality = "one_to_one",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "configuration",
        strength = "medium"
      } else if edge_type == "deploys_to" then {
        directionality = "directed",
        cardinality = "many_to_one",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "deployment",
        strength = "medium"
      } else if edge_type == "routes_to" then {
        directionality = "directed",
        cardinality = "one_to_one",
        transitivity = false,
        cyclicity = "cyclic_allowed",
        data_flow = "routing",
        strength = "weak"
      } else {
        directionality = "directed",
        cardinality = "one_to_one",
        transitivity = false,
        cyclicity = "acyclic",
        data_flow = "unknown",
        strength = "weak"
      },

    # Validate edge compatibility between node types
    validate_edge_compatibility = fun source_node target_node edge_type =>
      let source_kind = std.record.get_or "x-familiar-kind" source_node "unknown" in
      let target_kind = std.record.get_or "x-familiar-kind" target_node "unknown" in
      let source_type = std.record.get_or "x-familiar-node-type" source_node source_kind in
      let target_type = std.record.get_or "x-familiar-node-type" target_node target_kind in

      if edge_type == "depends_on" then
        # Infrastructure depends on infrastructure, entities depend on entities
        source_kind == target_kind ||
        (source_kind == "node" && target_kind == "node")
      else if edge_type == "contains" then
        # Only entities can contain other entities
        source_kind == "entity" && target_kind == "entity"
      else if edge_type == "communicates_with" then
        # Services can communicate with services
        source_type == "api_service" || target_type == "api_service"
      else if edge_type == "implements" then
        # Entities can implement domain concepts
        source_kind == "entity" && target_kind == "domain"
      else if edge_type == "orchestrates" then
        # Infrastructure can orchestrate other infrastructure
        source_kind == "node" && target_kind == "node"
      else if edge_type == "deploys_to" then
        # Anything can be deployed to infrastructure
        target_kind == "node"
      else
        true,  # Allow other combinations

    # Get traversal rules for this edge type
    get_traversal_rules = fun edge_type =>
      let chars = functions.get_edge_characteristics edge_type in
      {
        can_traverse = chars.directionality != "none",
        is_bidirectional = chars.directionality == "bidirectional",
        allows_cycles = chars.cyclicity == "cyclic_allowed",
        transitive = chars.transitivity
      },

    # Get compilation implications for code generation
    get_compilation_implications = fun edge_type =>
      if edge_type == "depends_on" then {
        generate_dependency_injection = true,
        generate_initialization_order = true,
        generate_lifecycle_management = true
      } else if edge_type == "communicates_with" then {
        generate_network_client = true,
        generate_api_calls = true,
        generate_error_handling = true
      } else if edge_type == "contains" then {
        generate_composition_logic = true,
        generate_ecs_relationships = true
      } else if edge_type == "orchestrates" then {
        generate_workflow_coordination = true,
        generate_state_management = true
      } else {
        generate_basic_relationship = true
      },

    # Get optimization hints for edge type
    get_optimization_hints = fun edge_type =>
      if edge_type == "depends_on" then {
        prefer_eager_loading = true,
        cache_relationship = true,
        parallel_initialization = false
      } else if edge_type == "communicates_with" then {
        prefer_async_calls = true,
        implement_circuit_breaker = true,
        add_retry_logic = true
      } else if edge_type == "orchestrates" then {
        use_workflow_engine = true,
        implement_transaction_boundaries = true,
        add_compensation_logic = true
      } else {
        basic_optimization = true
      }
  },

  # ============================================================================
  # GRAPH INTEGRATION
  # ============================================================================

  # Helper function to get node by ID (placeholder)
  get_node_by_id = fun node_id => {
    id = node_id,
    kind = "unknown",
    schema = {}
  },

  # Helper function to generate edge code (placeholder)
  generate_edge_code = fun edge_type edge_list implications graph_context => {
    edge_type = edge_type,
    edge_count = std.array.length edge_list,
    implications = implications,
    generated_code = "placeholder_code"
  },

  graph_integration = {
    # Add semantic metadata to graph edges
    edge_metadata = fun edge_type => {
      edge_type = edge_type,
      characteristics = functions.get_edge_characteristics edge_type,
      traversal_rules = functions.get_traversal_rules edge_type,
      compilation_implications = functions.get_compilation_implications edge_type,
      optimization_hints = functions.get_optimization_hints edge_type
    },

    # Validate edge semantics in the graph
    validate_graph_edges = fun edges =>
      std.array.all (fun edge =>
        let source_node = get_node_by_id edge.source_id in
        let target_node = get_node_by_id edge.target_id in
        let edge_type = std.record.get_or "x-familiar-edge-type" edge "depends_on" in
        functions.validate_edge_compatibility source_node target_node edge_type
      ) edges,

    # Generate code from edge relationships
    generate_from_graph_edges = fun edges graph_context =>
      # Group edges by type for batch processing (manual implementation)
      let edges_by_type = std.array.fold_left (fun acc edge =>
        let edge_type = std.record.get_or "x-familiar-edge-type" edge "depends_on" in
        let existing = std.record.get_or edge_type acc [] in
        std.record.update edge_type (existing @ [edge]) acc
      ) {} edges in

      # Generate code for each edge type
      std.record.map (fun edge_type edge_list =>
        let implications = functions.get_compilation_implications edge_type in
        generate_edge_code edge_type edge_list implications graph_context
      ) edges_by_type,

    # Analyze graph connectivity using edge semantics
    analyze_graph_connectivity = fun graph =>
      let edges = graph.edges in
      let edge_types = std.array.map (fun edge => std.record.get_or "x-familiar-edge-type" edge "depends_on") edges in

      {
        dependency_depth = std.array.length edges,
        communication_patterns = "basic",
        orchestration_complexity = std.array.length edge_types,
        potential_cycles = [],
        optimization_opportunities = []
      }
  },

  # ============================================================================
  # CODE GENERATION IMPLICATIONS
  # ============================================================================

  codegen = {
    rust = {
      traits = fun edge_type =>
        let implications = functions.get_compilation_implications edge_type in
        if implications.generate_dependency_injection then ["Injectable", "DependsOn"]
        else if implications.generate_network_client then ["NetworkClient", "CommunicatesWith"]
        else ["GraphEdge"],

      edge_structs = fun edge_type =>
        if edge_type == "depends_on" then ["DependencyEdge", "DependencyGraph"]
        else if edge_type == "communicates_with" then ["CommunicationEdge", "ServiceMesh"]
        else ["GenericEdge"]
    },

    typescript = {
      interfaces = fun edge_type =>
        if edge_type == "depends_on" then ["Dependency", "Injectable"]
        else if edge_type == "communicates_with" then ["Communicator", "ServiceClient"]
        else ["GraphEdge"]
    }
  },

  # ============================================================================
  # COMPOSITION RULES
  # ============================================================================

  # ============================================================================
  # TOPOLOGY CLASSIFICATION (MERGED FROM topology-type.ncl)
  # ============================================================================

  topology_classification = {
    contract = std.contract.custom (fun label value =>
      let valid_types = [
        "entity_node_binding", "synthesized", "runtime_topology",
        "static_topology", "dynamic_topology", "hierarchical_topology"
      ] in
      if std.array.elem value valid_types then value
      else std.contract.blame_with label "x-familiar-topology-classification must be one of: entity_node_binding, synthesized, runtime_topology, static_topology, dynamic_topology, hierarchical_topology"
    ),

    functions = {
      get_topology_characteristics = fun topology_type =>
        if topology_type == "entity_node_binding" then {
          purpose = "bind_data_to_compute",
          dynamism = "static",
          lifecycle = "compile_time",
          generation_strategy = "synthesized_from_service_refs"
        } else if topology_type == "synthesized" then {
          purpose = "generated_topology",
          dynamism = "static",
          lifecycle = "build_time",
          generation_strategy = "inferred_from_schema_relationships"
        } else if topology_type == "runtime_topology" then {
          purpose = "runtime_orchestration",
          dynamism = "dynamic",
          lifecycle = "runtime",
          generation_strategy = "discovered_at_runtime"
        } else if topology_type == "static_topology" then {
          purpose = "fixed_architecture",
          dynamism = "static",
          lifecycle = "design_time",
          generation_strategy = "manually_defined"
        } else if topology_type == "dynamic_topology" then {
          purpose = "adaptive_architecture",
          dynamism = "dynamic",
          lifecycle = "runtime",
          generation_strategy = "adaptive_discovery"
        } else if topology_type == "hierarchical_topology" then {
          purpose = "layered_architecture",
          dynamism = "static",
          lifecycle = "design_time",
          generation_strategy = "layer_composition"
        } else {
          purpose = "unknown",
          dynamism = "unknown",
          lifecycle = "unknown",
          generation_strategy = "unknown"
        }
    }
  },

  composition_rules = {
    compatible_with = ["x-familiar-architectural-pattern"],
    validation_order = 3,  # Validate after node types
    inheritable = true
  }
}
