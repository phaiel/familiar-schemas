/**
 * x-familiar-topology-type Extension
 *
 * Defines topology relationship types in the graph.
 * Enables type-aware edge classification in slotmap+petgraph.
 *
 * @category architecture
 * @graph-value high - classifies graph edge semantics
 * @validation context-dependent - for topology nodes
 */

{
  extension = {
    name = "x-familiar-topology-type",
    description = "Topology relationship classification",
    category = "architecture",
    required = false,
    version = "1.0.0",
    context_required = ["topology"]
  },

  nickel_type = [| `entity_node_binding, `data_flow, `service_mesh, `runtime_topology |],

  contract = std.contract.custom (fun label value =>
    let valid_types = ["entity_node_binding", "data_flow", "service_mesh", "runtime_topology"] in
    if std.array.elem value valid_types then value
    else std.contract.blame_with label "Invalid topology type: ${std.to_string value}"
  ),

  functions = {
    get_topology_characteristics = fun topology_type =>
      if topology_type == "entity_node_binding" then {
        relationship_type = "ownership", directionality = "directed", cardinality = "many_to_one"
      } else if topology_type == "data_flow" then {
        relationship_type = "communication", directionality = "directed", cardinality = "one_to_many"
      } else if topology_type == "service_mesh" then {
        relationship_type = "networking", directionality = "bidirectional", cardinality = "many_to_many"
      } else {
        relationship_type = "execution", directionality = "directed", cardinality = "one_to_one"
      }
  },

  graph_integration = {
    node_metadata = fun topology_type => {
      topology_type = topology_type,
      topology_characteristics = functions.get_topology_characteristics topology_type,
      edge_semantics = get_edge_semantics topology_type
    }
  }
}
