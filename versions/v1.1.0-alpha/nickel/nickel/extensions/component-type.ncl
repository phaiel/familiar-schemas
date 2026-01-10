/**
 * x-familiar-component-type Extension
 *
 * Defines the ECS component classification for graph nodes.
 * Provides semantic meaning for component composition in slotmap+petgraph.
 *
 * @category entity
 * @graph-value high - classifies ECS components for composition
 * @validation context-dependent - only required for component schemas
 */

{
  # ============================================================================
  # EXTENSION METADATA
  # ============================================================================

  extension = {
    name = "x-familiar-component-type",
    description = "ECS component classification for entity composition",
    category = "entity",
    required = false,
    version = "1.0.0",
    context_required = ["entity"]  # Only for entity schemas
  },

  # ============================================================================
  # NICKEL TYPE SYSTEM INTEGRATION
  # ============================================================================

  nickel_type = [| `ecs_component, `data_structure, `value_object, `entity |],

  # ============================================================================
  # CONTRACT DEFINITION
  # ============================================================================

  contract = std.contract.custom (fun label value =>
    let valid_types = ["ecs_component", "data_structure", "value_object", "entity"] in
    if std.array.elem value valid_types then
      value
    else
      std.contract.blame_with label "x-familiar-component-type must be one of: ecs_component, data_structure, value_object, entity. Got: ${std.to_string value}"
  ),

  # ============================================================================
  # MERGE STRATEGIES
  # ============================================================================

  merge_strategies = {
    override = fun base override => override,
    compose = fun base override => override,
    validate = fun base override =>
      if contract.validate override then override
      else std.contract.blame_with "merge" "Invalid x-familiar-component-type in merge: ${std.to_string override}"
  },

  # ============================================================================
  # DEFAULTS & METADATA
  # ============================================================================

  defaults = {
    validation = true
  },

  metadata = {
    required = false,
    examples = ["ecs_component", "data_structure", "value_object"],
    since = "v1.0.0",
    graph_impact = "component_composition"
  },

  # ============================================================================
  # FUNCTION LIBRARY
  # ============================================================================

  functions = {
    validate_component_type = fun value =>
      std.array.elem value ["ecs_component", "data_structure", "value_object", "entity"],

    # Get ECS characteristics
    get_ecs_properties = fun component_type =>
      if component_type == "ecs_component" then {
        has_data = true,
        has_behavior = false,
        storable = true,
        serializable = true
      } else if component_type == "data_structure" then {
        has_data = true,
        has_behavior = false,
        storable = true,
        serializable = true
      } else if component_type == "value_object" then {
        has_data = true,
        has_behavior = false,
        storable = false,
        serializable = true
      } else {  # entity
        has_data = true,
        has_behavior = true,
        storable = true,
        serializable = true
      },

    # Check composition compatibility
    can_compose_with = fun component_a component_b =>
      let props_a = functions.get_ecs_properties component_a in
      let props_b = functions.get_ecs_properties component_b in
      # Entities can contain components, but not vice versa
      if component_a == "entity" then
        component_b != "entity"
      else if component_b == "entity" then
        false
      else
        true,

    # Get storage requirements
    get_storage_requirements = fun component_type =>
      if component_type == "ecs_component" then "persistent"
      else if component_type == "data_structure" then "persistent"
      else if component_type == "value_object" then "transient"
      else "persistent"
  },

  # ============================================================================
  # GRAPH INTEGRATION
  # ============================================================================

  graph_integration = {
    # Add ECS metadata to component nodes
    node_metadata = fun component_type_value => {
      component_type = component_type_value,
      ecs_properties = functions.get_ecs_properties component_type_value,
      storage_requirements = functions.get_storage_requirements component_type_value,
      composition_capabilities = get_composition_capabilities component_type_value
    },

    # Validate component composition in graph
    graph_constraints = fun node graph_context => {
      let component_type = node."x-familiar-component-type" in
      let connected_nodes = graph_context.neighbors | [] in

      # Check composition compatibility with connected components
      let compatibility_issues = std.array.filter_map
        (fun neighbor =>
          if std.record.has_field "x-familiar-component-type" neighbor then
            let neighbor_type = neighbor."x-familiar-component-type" in
            if functions.can_compose_with component_type neighbor_type then
              null
            else
              "Component ${component_type} cannot compose with ${neighbor_type}"
          else
            null
        )
        connected_nodes in

      if std.array.is_empty compatibility_issues then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = compatibility_issues, warnings = [] }
    },

    # Generate ECS code from graph relationships
    generate_from_graph = fun node graph_context => {
      let component_type = node."x-familiar-component-type" in
      let ecs_props = functions.get_ecs_properties component_type in

      # Generate component-specific code
      generate_component_struct component_type node graph_context &
      generate_storage_logic component_type node graph_context &
      generate_composition_logic component_type node graph_context
    }
  },

  # ============================================================================
  # CODE GENERATION IMPLICATIONS
  # ============================================================================

  codegen = {
    rust = {
      derives = fun component_type =>
        let base = ["Debug", "Clone"] in
        if component_type == "ecs_component" then
          base @ ["Serialize", "Deserialize", "Default"]
        else if component_type == "value_object" then
          base @ ["Serialize", "Deserialize", "PartialEq", "Eq", "Hash"]
        else
          base @ ["Serialize", "Deserialize"],

      traits = fun component_type =>
        if component_type == "ecs_component" then ["Component"]
        else if component_type == "entity" then ["Entity"]
        else []
    },

    typescript = {
      interfaces = fun component_type =>
        if component_type == "ecs_component" then ["Component", "Serializable"]
        else if component_type == "value_object" then ["ValueObject", "Equatable"]
        else ["Serializable"]
    }
  },

  # ============================================================================
  # COMPOSITION RULES
  # ============================================================================

  composition_rules = {
    compatible_with = ["x-familiar-type-category", "x-familiar-codegen-targets"],
    validation_order = 2,
    inheritable = true
  }
}
