/**
 * x-familiar-business-rules Extension
 *
 * Defines business rules for domain logic validation.
 * Enables rule-aware graph constraints in slotmap+petgraph.
 *
 * @category domain
 * @graph-value high - enforces business constraints
 * @validation context-dependent - for domain nodes
 */

{
  extension = {
    name = "x-familiar-business-rules",
    description = "Business rule definitions",
    category = "domain",
    required = false,
    version = "1.0.0",
    context_required = ["domain"]
  },

  nickel_type = Array<String>,

  contract = std.contract.custom (fun label value =>
    if std.is_array value && std.array.all (fun item => std.is_string item) value then value
    else std.contract.blame_with label "Business rules must be array of strings: ${std.to_string value}"
  ),

  functions = {
    validate_business_consistency = fun rules graph_context =>
      # Check if business rules are consistent across the domain
      let domain_rules = graph_context.business_rules | [] in
      std.array.all (fun rule => std.array.elem rule domain_rules) rules,

    categorize_rules = fun rules =>
      std.array.map (fun rule =>
        if std.string.contains "must" rule then "invariant"
        else if std.string.contains "should" rule then "guideline"
        else "constraint"
      ) rules
  },

  graph_integration = {
    node_metadata = fun rules => {
      business_rules = rules,
      rule_count = std.array.length rules,
      rule_categories = functions.categorize_rules rules,
      business_constraints = extract_constraints rules
    },

    graph_constraints = fun node graph_context => {
      let rules = node."x-familiar-business-rules" | [] in
      if functions.validate_business_consistency rules graph_context then
        { valid = true, errors = [], warnings = [] }
      else
        { valid = false, errors = ["Business rules violate domain consistency"], warnings = [] }
    }
  }
}
