/**
 * x-familiar-architectural-pattern Extension
 *
 * Defines architectural patterns for graph structure validation.
 * Enables pattern-aware graph analysis in slotmap+petgraph.
 *
 * @category architecture
 * @graph-value high - validates graph structure patterns
 * @validation context-dependent - for architecture nodes
 */

{
  extension = {
    name = "x-familiar-architectural-pattern",
    description = "Architectural pattern classification",
    category = "architecture",
    required = false,
    version = "1.0.0",
    context_required = ["topology"]
  },

  nickel_type = [|
    `layered, `microservices, `event_driven, `hexagonal,
    `cqrs, `serverless, `monolithic, `modular, `pipeline
  |],

  contract = std.contract.custom (fun label value =>
    let valid_patterns = [
      "layered", "microservices", "event_driven", "hexagonal",
      "cqrs", "serverless", "monolithic", "modular", "pipeline"
    ] in
    if std.array.elem value valid_patterns then value
    else std.contract.blame_with label "Invalid architectural pattern: ${std.to_string value}"
  ),

  functions = {
    get_pattern_characteristics = fun pattern =>
      if pattern == "layered" then {
        coupling = "loose", communication = "strict_layers", scalability = "vertical"
      } else if pattern == "microservices" then {
        coupling = "loose", communication = "api_contracts", scalability = "horizontal"
      } else if pattern == "event_driven" then {
        coupling = "loose", communication = "events", scalability = "horizontal"
      } else {
        coupling = "tight", communication = "direct", scalability = "vertical"
      },

    validate_pattern_constraints = fun pattern graph_structure =>
      # Pattern-specific graph validation
      if pattern == "layered" then
        validate_layered_constraints graph_structure
      else if pattern == "microservices" then
        validate_microservices_constraints graph_structure
      else
        true
  },

  graph_integration = {
    node_metadata = fun pattern => {
      architectural_pattern = pattern,
      pattern_characteristics = functions.get_pattern_characteristics pattern,
      structural_constraints = get_pattern_constraints pattern
    },

    graph_constraints = fun node graph_context => {
      let pattern = node."x-familiar-architectural-pattern" in
      if functions.validate_pattern_constraints pattern graph_context then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Graph structure violates ${pattern} pattern constraints"], warnings = [] }
    }
  }
}
