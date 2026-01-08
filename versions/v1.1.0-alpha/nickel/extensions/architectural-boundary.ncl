/**
 * x-familiar-architectural-boundary Extension
 *
 * Defines architectural boundaries for graph structure organization.
 * Enables boundary-aware graph partitioning in slotmap+petgraph.
 *
 * @category architecture
 * @graph-value medium - organizes graph structure
 * @validation optional - for boundary definitions
 */

{
  extension = {
    name = "x-familiar-architectural-boundary",
    description = "Architectural boundary definitions",
    category = "architecture",
    required = false,
    version = "1.0.0"
  },

  nickel_type = {
    type: String,
    name: String,
    responsibilities: Array<String>
  },

  contract = std.contract.custom (fun label value =>
    if std.is_record value &&
       std.record.has_field "type" value &&
       std.record.has_field "name" value then value
    else std.contract.blame_with label "Boundary must have type and name: ${std.to_string value}"
  ),

  functions = {
    get_boundary_type = fun boundary =>
      boundary.type,

    validate_boundary_consistency = fun boundary graph_context =>
      # Check if boundary aligns with architectural context
      let boundary_type = functions.get_boundary_type boundary in
      let allowed_types = ["service", "module", "layer", "context", "aggregate"] in
      std.array.elem boundary_type allowed_types
  },

  graph_integration = {
    node_metadata = fun boundary => {
      boundary_definition = boundary,
      boundary_type = functions.get_boundary_type boundary,
      boundary_name = boundary.name,
      boundary_responsibilities = boundary.responsibilities
    }
  }
}
