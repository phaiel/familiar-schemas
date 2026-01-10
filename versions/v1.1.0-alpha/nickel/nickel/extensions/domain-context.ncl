/**
 * x-familiar-domain-context Extension
 *
 * Defines bounded context for domain logic validation.
 * Enables domain-aware graph constraints in slotmap+petgraph.
 *
 * @category domain
 * @graph-value high - validates domain boundaries
 * @validation context-dependent - for domain nodes
 */

{
  extension = {
    name = "x-familiar-domain-context",
    description = "Bounded context classification",
    category = "domain",
    required = false,
    version = "1.0.0",
    context_required = ["domain"]
  },

  nickel_type = String,

  contract = std.contract.custom (fun label value =>
    if std.is_string value && std.string.length value > 0 then value
    else std.contract.blame_with label "Domain context must be non-empty string: ${std.to_string value}"
  ),

  functions = {
    validate_context_boundaries = fun context graph_context =>
      # Check if domain context aligns with architectural boundaries
      let domain_contexts = graph_context.domain_contexts | [] in
      std.array.elem context domain_contexts,

    get_context_relationships = fun context =>
      # Define valid relationships between domain contexts
      if context == "user_management" then ["authentication", "authorization"]
      else if context == "data_processing" then ["storage", "computation"]
      else ["generic"]
  },

  graph_integration = {
    node_metadata = fun context => {
      domain_context = context,
      context_relationships = functions.get_context_relationships context,
      boundary_requirements = get_boundary_requirements context
    },

    graph_constraints = fun node graph_context => {
      let context = node."x-familiar-domain-context" in
      if functions.validate_context_boundaries context graph_context then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Domain context ${context} violates boundary constraints"], warnings = [] }
    }
  }
}
