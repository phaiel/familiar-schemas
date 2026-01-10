/**
 * x-familiar-scaling Extension
 *
 * Defines auto-scaling rules for infrastructure nodes.
 * Enables dynamic scaling in slotmap+petgraph resource management.
 *
 * @category infrastructure
 * @graph-value high - enables dynamic resource management
 * @validation optional - for scalable nodes
 */

{
  extension = {
    name = "x-familiar-scaling",
    description = "Auto-scaling configuration",
    category = "infrastructure",
    required = false,
    version = "1.0.0"
  },

  nickel_type = {
    min_replicas: Number,
    max_replicas: Number,
    target_cpu_utilization: Number,
    enabled: Bool
  },

  contract = std.contract.custom (fun label value =>
    if std.is_record value &&
       (std.record.get_or "min_replicas" value 1) <= (std.record.get_or "max_replicas" value 10) then value
    else std.contract.blame_with label "Invalid scaling configuration: ${std.to_string value}"
  ),

  functions = {
    calculate_scaling_headroom = fun scaling =>
      scaling.max_replicas - scaling.min_replicas,

    get_scaling_policy = fun scaling =>
      if scaling.enabled then "horizontal" else "fixed"
  },

  graph_integration = {
    node_metadata = fun scaling => {
      scaling_config = scaling,
      scaling_headroom = functions.calculate_scaling_headroom scaling,
      scaling_policy = functions.get_scaling_policy scaling
    }
  }
}
