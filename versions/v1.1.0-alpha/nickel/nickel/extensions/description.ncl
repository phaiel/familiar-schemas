/**
 * x-familiar-description Extension
 *
 * Provides human-readable documentation for graph nodes.
 * Enables self-documenting graph structures in slotmap+petgraph.
 *
 * @category metadata
 * @graph-value medium - improves graph usability
 * @validation optional - documentation
 */

{
  extension = {
    name = "x-familiar-description",
    description = "Human-readable documentation",
    category = "metadata",
    required = false,
    version = "1.0.0"
  },

  nickel_type = String,

  contract = std.contract.custom (fun label value =>
    if std.is_string value && std.string.length value > 0 then value
    else std.contract.blame_with label "Description must be non-empty string: ${std.to_string value}"
  ),

  functions = {
    generate_summary = fun description =>
      let words = std.string.split " " description in
      if std.array.length words > 10 then
        std.string.join " " (std.array.take 10 words) ++ "..."
      else
        description,

    extract_keywords = fun description =>
      std.string.split " " description
  },

  graph_integration = {
    node_metadata = fun description => {
      description = description,
      summary = functions.generate_summary description,
      keywords = functions.extract_keywords description,
      documentation_type = "human_readable"
    }
  },

  merge_strategies = {
    compose = fun base override => base ++ ". " ++ override
  }
}
