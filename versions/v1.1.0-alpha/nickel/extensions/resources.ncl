/**
 * x-familiar-resources Extension
 *
 * Defines resource requirements for infrastructure nodes.
 * Enables resource-aware scheduling in slotmap+petgraph.
 *
 * @category infrastructure
 * @graph-value high - drives resource allocation
 * @validation required - for infrastructure nodes
 */

{
  extension = {
    name = "x-familiar-resources",
    description = "Resource requirements for compute nodes",
    category = "infrastructure",
    required = false,
    version = "1.0.0",
    context_required = ["node"]
  },

  nickel_type = {
    cpu: String,
    memory: String,
    storage: String,
    replicas: Number
  },

  contract = std.contract.custom (fun label value =>
    if std.is_record value &&
       std.record.has_field "cpu" value &&
       std.record.has_field "memory" value then value
    else std.contract.blame_with label "Resources must specify cpu and memory: ${std.to_string value}"
  ),

  functions = {
    calculate_resource_cost = fun resources =>
      let cpu_cores = std.string.to_number (std.string.split "." resources.cpu | [0] | std.array.first) in
      let memory_gb = std.string.to_number (std.string.split "Gi" resources.memory | [0] | std.array.first) in
      cpu_cores * 2 + memory_gb,

    check_resource_compatibility = fun required available =>
      let req_cpu = parse_cpu required.cpu in
      let avail_cpu = parse_cpu available.cpu in
      let req_mem = parse_memory required.memory in
      let avail_mem = parse_memory available.memory in
      req_cpu <= avail_cpu && req_mem <= avail_mem
  },

  graph_integration = {
    node_metadata = fun resources => {
      resource_requirements = resources,
      resource_cost = functions.calculate_resource_cost resources,
      scheduling_class = get_scheduling_class resources
    },

    graph_constraints = fun node graph_context => {
      let resources = node."x-familiar-resources" in
      let available_resources = graph_context.available_resources | {} in
      
      if functions.check_resource_compatibility resources available_resources then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Insufficient resources available"], warnings = [] }
    }
  },

  codegen = {
    rust = {
      structs = ["ResourceRequirements", "ResourceAllocation"]
    }
  }
}
