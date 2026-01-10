# Edge Primitives - Building blocks for relationship semantics

{
  # Edge validation primitives
  EdgeValidationPrimitives = {
    validate_with_edge_contract = fun schema => schema,  # Placeholder - will be implemented by extensions
    extract_typed_relationships = fun schema => []  # Placeholder - will be implemented by extensions
  },

  # Edge type primitives
  EdgeTypePrimitives = {
    depends_on = {directionality = "directed", cyclicity = "acyclic"},
    orchestrates = {directionality = "directed", cardinality = "one_to_many"},
    communicates_with = {directionality = "bidirectional", cyclicity = "cyclic_allowed"},
    contains = {directionality = "directed", cardinality = "one_to_many"},
    deploys_to = {directionality = "directed", cardinality = "many_to_one"}
  },

  # Edge compatibility primitives
  EdgeCompatibilityPrimitives = {
    entity_to_node = ["entity_node_binding", "deploys_to"],
    system_to_component = ["orchestrates", "depends_on"],
    component_to_resource = ["consumes", "depends_on"],
    node_to_system = ["contains", "orchestrates"]
  }
}