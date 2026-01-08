/**
 * x-familiar-node-type Extension
 *
 * Defines the specific type of infrastructure node in the graph.
 * Provides classification for compute resources in slotmap+petgraph.
 *
 * @category infrastructure
 * @graph-value high - classifies infrastructure nodes for scheduling
 * @validation context-dependent - only required for node kinds
 */

{
  # ============================================================================
  # EXTENSION METADATA
  # ============================================================================

  extension = {
    name = "x-familiar-node-type",
    description = "Infrastructure node classification for compute resources",
    category = "infrastructure",
    required = false,
    version = "1.0.0",
    context_required = ["node"]  # Only required when x-familiar-kind = "node"
  },

  # ============================================================================
  # NICKEL TYPE SYSTEM INTEGRATION
  # ============================================================================

  nickel_type = [|
    `api_service, `worker, `database, `cache, `queue,
    `tool_execution, `resource_allocation, `action_execution
  |],

  # ============================================================================
  # CONTRACT DEFINITION
  # ============================================================================

  contract = std.contract.custom (fun label value =>
    let valid_types = [
      "api_service", "worker", "database", "cache", "queue",
      "tool_execution", "resource_allocation", "action_execution"
    ] in
    if std.array.elem value valid_types then
      value
    else
      std.contract.blame_with label "x-familiar-node-type must be one of: api_service, worker, database, cache, queue, tool_execution, resource_allocation, action_execution. Got: ${std.to_string value}"
  ),

  # ============================================================================
  # MERGE STRATEGIES
  # ============================================================================

  merge_strategies = {
    override = fun base override => override,
    compose = fun base override => override,  # Node type cannot be composed
    validate = fun base override =>
      if contract.validate override then override
      else std.contract.blame_with "merge" "Invalid x-familiar-node-type in merge: ${std.to_string override}"
  },

  # ============================================================================
  # DEFAULTS & METADATA
  # ============================================================================

  defaults = {
    validation = true
  },

  metadata = {
    required = false,
    examples = ["api_service", "worker", "database"],
    since = "v1.0.0",
    graph_impact = "node_scheduling"
  },

  # ============================================================================
  # FUNCTION LIBRARY
  # ============================================================================

  functions = {
    # Validate node type
    validate_node_type = fun value =>
      std.array.elem value [
        "api_service", "worker", "database", "cache", "queue",
        "tool_execution", "resource_allocation", "action_execution"
      ],

    # Get execution characteristics
    get_execution_model = fun node_type =>
      if node_type == "api_service" then "request_response"
      else if node_type == "worker" then "background_processing"
      else if node_type == "database" then "data_access"
      else if node_type == "cache" then "fast_access"
      else if node_type == "queue" then "message_processing"
      else if node_type == "tool_execution" then "tool_invocation"
      else if node_type == "resource_allocation" then "resource_management"
      else "task_execution",

    # Get scaling characteristics
    get_scaling_profile = fun node_type =>
      if node_type == "api_service" then "horizontal_scaling"
      else if node_type == "worker" then "horizontal_scaling"
      else if node_type == "database" then "vertical_scaling"
      else if node_type == "cache" then "horizontal_scaling"
      else if node_type == "queue" then "horizontal_scaling"
      else "fixed_scaling",

    # Check if node type supports persistence
    supports_persistence = fun node_type =>
      std.array.elem node_type ["database", "cache", "queue"],

    # Check if node type is externally accessible
    is_externally_accessible = fun node_type =>
      std.array.elem node_type ["api_service", "database", "cache"]
  },

  # ============================================================================
  # GRAPH INTEGRATION
  # ============================================================================

  graph_integration = {
    # Add scheduling metadata to infrastructure nodes
    node_metadata = fun node_type_value => {
      node_type = node_type_value,
      execution_model = functions.get_execution_model node_type_value,
      scaling_profile = functions.get_scaling_profile node_type_value,
      supports_persistence = functions.supports_persistence node_type_value,
      externally_accessible = functions.is_externally_accessible node_type_value,
      scheduling_priority = get_scheduling_priority node_type_value
    },

    # Validate infrastructure node constraints
    graph_constraints = fun node graph_context => {
      let node_type = node."x-familiar-node-type" in
      let infra_context = graph_context.infrastructure | {} in

      # Check resource compatibility
      let compatible_resources = check_resource_compatibility node_type infra_context in
      let scaling_allowed = check_scaling_allowed node_type infra_context in

      if compatible_resources && scaling_allowed then
        { valid = true, errors = [], warnings = [] }
      else
        {
          valid = false,
          errors = std.array.filter (fun x => x != null) [
            if compatible_resources then null else "Node type ${node_type} incompatible with available resources",
            if scaling_allowed then null else "Node type ${node_type} scaling not supported in current context"
          ],
          warnings = []
        }
    },

    # Generate infrastructure code from graph
    generate_from_graph = fun node graph_context => {
      let node_type = node."x-familiar-node-type" in

      # Generate node-specific infrastructure code
      generate_node_infrastructure node_type node graph_context &
      generate_networking_config node_type node graph_context &
      generate_monitoring_config node_type node graph_context
    }
  },

  # ============================================================================
  # CODE GENERATION IMPLICATIONS
  # ============================================================================

  codegen = {
    rust = {
      derives = ["Debug", "Clone", "Serialize", "Deserialize"],
      traits = fun node_type =>
        if node_type == "api_service" then ["Handler", "Routable"]
        else if node_type == "worker" then ["Worker", "Processable"]
        else if node_type == "database" then ["Database", "Queryable"]
        else ["Node"]
    },

    typescript = {
      interfaces = fun node_type =>
        if node_type == "api_service" then ["ApiService", "RequestHandler"]
        else if node_type == "worker" then ["Worker", "TaskProcessor"]
        else ["InfrastructureNode"]
    }
  },

  # ============================================================================
  # COMPOSITION RULES
  # ============================================================================

  composition_rules = {
    compatible_with = ["x-familiar-resources", "x-familiar-scaling", "x-familiar-capabilities"],
    validation_order = 2,
    inheritable = true
  }
}
