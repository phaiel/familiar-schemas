/**
 * x-familiar-capabilities Extension
 *
 * Defines what a node can do in the graph.
 * Enables capability-aware graph queries and orchestration.
 *
 * @category infrastructure
 * @graph-value high - enables capability-based orchestration
 * @validation optional - for functional nodes
 */

{
  extension = {
    name = "x-familiar-capabilities",
    description = "Node capability definitions",
    category = "infrastructure",
    required = false,
    version = "1.0.0"
  },

  nickel_type = Array<String>,

  contract = std.contract.custom (fun label value =>
    if std.is_array value && std.array.all (fun item => std.is_string item) value then value
    else std.contract.blame_with label "Capabilities must be array of strings: ${std.to_string value}"
  ),

  functions = {
    has_capability = fun capabilities target_capability =>
      std.array.elem target_capability capabilities,

    find_compatible_nodes = fun required_capabilities available_nodes =>
      std.array.filter (fun node =>
        let node_caps = node."x-familiar-capabilities" | [] in
        std.array.all (fun req_cap =>
          std.array.elem req_cap node_caps
        ) required_capabilities
      ) available_nodes
  },

  graph_integration = {
    node_metadata = fun capabilities => {
      node_capabilities = capabilities,
      capability_count = std.array.length capabilities,
      primary_capability = std.array.first capabilities
    },

    graph_constraints = fun node graph_context => {
      let capabilities = node."x-familiar-capabilities" | [] in
      let required_capabilities = graph_context.required_capabilities | [] in
      
      if std.array.all (fun req => std.array.elem req capabilities) required_capabilities then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Missing required capabilities"], warnings = [] }
    }
  }
}
