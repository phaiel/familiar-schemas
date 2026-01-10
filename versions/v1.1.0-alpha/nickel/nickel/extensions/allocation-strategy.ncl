/**
 * x-familiar-allocation-strategy Extension
 *
 * Defines resource allocation strategies for infrastructure nodes.
 * Enables intelligent resource distribution in slotmap+petgraph.
 *
 * @category infrastructure
 * @graph-value medium - optimizes resource allocation
 * @validation optional - for resource-managed nodes
 */

{
  extension = {
    name = "x-familiar-allocation-strategy",
    description = "Resource allocation strategy",
    category = "infrastructure",
    required = false,
    version = "1.0.0"
  },

  nickel_type = [| `first_fit, `best_fit, `worst_fit, `round_robin |],

  contract = std.contract.custom (fun label value =>
    let valid_strategies = ["first_fit", "best_fit", "worst_fit", "round_robin"] in
    if std.array.elem value valid_strategies then value
    else std.contract.blame_with label "Invalid allocation strategy: ${std.to_string value}"
  ),

  functions = {
    get_strategy_characteristics = fun strategy =>
      if strategy == "first_fit" then { efficiency = "fast", fragmentation = "high" }
      else if strategy == "best_fit" then { efficiency = "optimal", fragmentation = "low" }
      else if strategy == "worst_fit" then { efficiency = "balanced", fragmentation = "medium" }
      else { efficiency = "fair", fragmentation = "medium" }
  },

  graph_integration = {
    node_metadata = fun strategy => {
      allocation_strategy = strategy,
      strategy_characteristics = functions.get_strategy_characteristics strategy
    }
  }
}
