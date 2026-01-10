/**
 * x-familiar-type-category Extension
 *
 * Defines the type system classification for graph nodes.
 * Provides type system semantics for code generation in slotmap+petgraph.
 *
 * @category entity
 * @graph-value high - enables type-aware code generation
 * @validation context-dependent - for typed schemas
 */

{
  extension = {
    name = "x-familiar-type-category",
    description = "Type system classification for schemas",
    category = "entity",
    required = false,
    version = "1.0.0"
  },

  nickel_type = [| `primitive, `composite, `enum, `union, `reference |],

  contract = std.contract.custom (fun label value =>
    let valid_categories = ["primitive", "composite", "enum", "union", "reference"] in
    if std.array.elem value valid_categories then value
    else std.contract.blame_with label "Invalid type category: ${std.to_string value}"
  ),

  functions = {
    get_type_characteristics = fun category =>
      if category == "primitive" then { has_subtypes = false, serializable = true }
      else if category == "composite" then { has_subtypes = true, serializable = true }
      else if category == "enum" then { has_subtypes = true, serializable = true }
      else if category == "union" then { has_subtypes = true, serializable = false }
      else { has_subtypes = false, serializable = true }
  },

  graph_integration = {
    node_metadata = fun type_category => {
      type_category = type_category,
      characteristics = functions.get_type_characteristics type_category,
      type_system_role = "definition"
    }
  },

  codegen = {
    rust = {
      derives = fun category =>
        if category == "enum" then ["Debug", "Clone", "Serialize", "Deserialize"]
        else ["Debug", "Clone", "Serialize", "Deserialize"]
    }
  }
}
