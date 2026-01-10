/**
 * x-familiar-kind Extension
 *
 * Defines the architectural role of a schema in the graph structure.
 * Provides semantic classification for graph nodes (slotmap indices).
 *
 * @category identity
 * @graph-value high - provides node semantics for slotmap+petgraph
 * @validation strict - required for all schemas
 */

{
  # ============================================================================
  # EXTENSION METADATA
  # ============================================================================

  extension = {
    name = "x-familiar-kind",
    description = "Architectural role classification for graph nodes",
    category = "identity",
    required = true,
    version = "1.0.0"
  },

  # ============================================================================
  # NICKEL TYPE SYSTEM INTEGRATION
  # ============================================================================

  nickel_type = [| `entity, `node, `topology, `domain |],

  # ============================================================================
  # CONTRACT DEFINITION
  # ============================================================================

  contract = std.contract.custom (fun label value =>
    if std.array.elem value ["entity", "node", "topology", "domain"] then
      value
    else
      std.contract.blame_with label "x-familiar-kind must be one of: entity, node, topology, domain. Got: ${std.to_string value}"
  ),

  # ============================================================================
  # MERGE STRATEGIES
  # ============================================================================

  merge_strategies = {
    override = fun base override => override,
    compose = fun base override => override,  # Kind cannot be composed, only overridden
    validate = fun base override =>
      if contract.validate override then override
      else std.contract.blame_with "merge" "Invalid x-familiar-kind in merge: ${std.to_string override}"
  },

  # ============================================================================
  # DEFAULTS & METADATA
  # ============================================================================

  defaults = {
    validation = true
  },

  metadata = {
    required = true,
    examples = ["entity", "node", "topology", "domain"],
    since = "v1.0.0",
    graph_impact = "node_classification"
  },

  # ============================================================================
  # FUNCTION LIBRARY
  # ============================================================================

  functions = {
    # Validate kind value
    validate_kind = fun value =>
      std.array.elem value ["entity", "node", "topology", "domain"],

    # Get orthological layer from kind
    get_layer = fun kind =>
      if kind == "entity" then `codegen
      else if kind == "node" then `infrastructure
      else if kind == "topology" then `architecture
      else `domain,

    # Get directory placement for kind
    get_directory = fun kind =>
      if kind == "entity" then "codegen"
      else if kind == "node" then "infrastructure"
      else if kind == "topology" then "architecture"
      else "domain",

    # Check if kind represents data (for codegen)
    is_data_kind = fun kind =>
      kind == "entity",

    # Check if kind represents compute (for infrastructure)
    is_compute_kind = fun kind =>
      kind == "node",

    # Check if kind represents wiring (for architecture)
    is_wiring_kind = fun kind =>
      kind == "topology",

    # Check if kind represents business logic (for domain)
    is_business_kind = fun kind =>
      kind == "domain"
  },

  # ============================================================================
  # GRAPH INTEGRATION
  # ============================================================================

  graph_integration = {
    # Add semantic metadata to graph nodes
    node_metadata = fun kind_value => {
      node_type = kind_value,
      orthological_layer = functions.get_layer kind_value,
      is_data_node = functions.is_data_kind kind_value,
      is_compute_node = functions.is_compute_kind kind_value,
      is_wiring_node = functions.is_wiring_kind kind_value,
      is_business_node = functions.is_business_kind kind_value
    },

    # Validate node fits in graph structure
    graph_constraints = fun node graph_context => {
      # Ensure node kind is compatible with graph structure
      let layer = functions.get_layer node."x-familiar-kind" in
      let allowed_layers = graph_context.allowed_layers | ["codegen", "infrastructure", "architecture", "domain"] in

      if std.array.elem layer allowed_layers then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Node kind ${node."x-familiar-kind"} not allowed in current graph context"], warnings = [] }
    },

    # Generate code from graph context
    generate_from_graph = fun node graph_context => {
      let kind = node."x-familiar-kind" in
      let layer = functions.get_layer kind in

      if layer == `codegen then
        # Generate data structures
        generate_entity_code node graph_context
      else if layer == `infrastructure then
        # Generate infrastructure code
        generate_infrastructure_code node graph_context
      else if layer == `architecture then
        # Generate wiring code
        generate_architecture_code node graph_context
      else
        # Generate domain code
        generate_domain_code node graph_context
    }
  },

  # ============================================================================
  # CODE GENERATION IMPLICATIONS
  # ============================================================================

  codegen = {
    rust = {
      derives = fun kind =>
        if kind == "entity" then
          ["Debug", "Clone", "Serialize", "Deserialize"]
        else
          ["Debug", "Clone"],

      traits = fun kind =>
        if kind == "entity" then
          ["Entity"]
        else if kind == "node" then
          ["Node"]
        else
          []
    },

    typescript = {
      interfaces = fun kind =>
        if kind == "entity" then
          ["Entity", "Serializable"]
        else if kind == "node" then
          ["Node", "Executable"]
        else
          ["Serializable"]
    }
  },

  # ============================================================================
  # COMPOSITION RULES
  # ============================================================================

  composition_rules = {
    # How this extension composes with others
    compatible_with = ["x-familiar-description", "x-familiar-contract"],

    # Validation order (this should be validated first)
    validation_order = 1,

    # Inheritance rules
    inheritable = false  # Kind cannot be inherited, must be explicit
  }
}
