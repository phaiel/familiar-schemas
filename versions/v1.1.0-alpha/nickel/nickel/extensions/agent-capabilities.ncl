/**
 * x-familiar-agent-capabilities Extension
 *
 * Defines AI agent capabilities for orchestration.
 * Enables capability-aware agent orchestration in slotmap+petgraph.
 *
 * @category domain
 * @graph-value medium - enables agent orchestration
 * @validation optional - for agent nodes
 */

{
  extension = {
    name = "x-familiar-agent-capabilities",
    description = "AI agent capability definitions",
    category = "domain",
    required = false,
    version = "1.0.0"
  },

  nickel_type = Array<String>,

  contract = std.contract.custom (fun label value =>
    if std.is_array value && std.array.all (fun item => std.is_string item) value then value
    else std.contract.blame_with label "Agent capabilities must be array of strings: ${std.to_string value}"
  ),

  functions = {
    has_capability = fun capabilities target_capability =>
      std.array.elem target_capability capabilities,

    match_agent_to_task = fun agent_capabilities required_capabilities =>
      std.array.all (fun req => functions.has_capability agent_capabilities req) required_capabilities,

    rank_agents_by_capability = fun agents required_capabilities =>
      std.array.sort_by (fun agent =>
        let agent_caps = agent."x-familiar-agent-capabilities" | [] in
        std.array.length (std.array.filter (fun req => std.array.elem req agent_caps) required_capabilities)
      ) agents
  },

  graph_integration = {
    node_metadata = fun capabilities => {
      agent_capabilities = capabilities,
      capability_count = std.array.length capabilities,
      primary_capabilities = std.array.take 3 capabilities
    },

    graph_constraints = fun node graph_context => {
      let capabilities = node."x-familiar-agent-capabilities" | [] in
      let required_caps = graph_context.required_agent_capabilities | [] in
      
      if std.array.all (fun req => functions.has_capability capabilities req) required_caps then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Agent lacks required capabilities"], warnings = [] }
    }
  }
}
