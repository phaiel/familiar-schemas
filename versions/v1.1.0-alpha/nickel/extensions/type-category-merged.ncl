# x-familiar-type-category Extension - Composed from extension primitives
# Demonstrates how extensions can be built from merged primitives
let ExtensionLibrary = import "../libraries/extension_libraries/extension_library.ncl" in

# Build extension using merged primitives
ExtensionLibrary.extension_builders.build_category_extension "entity" "x-familiar-type-category" "Type system classification for schemas"
& {

  # Override contract with specific validation using merged primitives
  contract = ExtensionLibrary.ContractPrimitives.ValidationPatterns.type_category,

  # Add specific functions using merged primitives
  functions =
    ExtensionLibrary.FunctionPrimitives.ProcessingFunctions
    & {
      get_type_characteristics = fun category =>
        if category == "primitive" then
          { has_subtypes = false, serializable = true, nullable = true }
        else if category == "composite" then
          { has_subtypes = true, serializable = true, nullable = false }
        else if category == "enum" then
          { has_subtypes = true, serializable = true, nullable = false }
        else if category == "union" then
          { has_subtypes = true, serializable = false, nullable = false }
        else
          { has_subtypes = false, serializable = true, nullable = false },

      validate_type_category = fun schema =>
        let category = ExtensionLibrary.FunctionPrimitives.ProcessingFunctions.extract_field schema "x-familiar-type-category" null in
        if category != null then
          ExtensionLibrary.ContractPrimitives.ValidationPatterns.type_category.validate category
        else
          null
    },

  # Enhance graph integration using merged primitives
  graph_integration =
    ExtensionLibrary.IntegrationPrimitives.GraphIntegrationPrimitives
    & {
      node_metadata = fun schema =>
        let base_metadata = ExtensionLibrary.IntegrationPrimitives.GraphIntegrationPrimitives.node_metadata_generators.entity_node schema extension in
        let category = ExtensionLibrary.FunctionPrimitives.ProcessingFunctions.extract_field schema "x-familiar-type-category" null in
        base_metadata
        & {
          type_characteristics = if category != null then functions.get_type_characteristics category else {},
          type_category = category
        }
    },

  # Enhance codegen integration using merged primitives
  codegen =
    ExtensionLibrary.IntegrationPrimitives.CodegenIntegrationPrimitives
    & {
      rust =
        ExtensionLibrary.IntegrationPrimitives.CodegenIntegrationPrimitives.language_generators.rust
        & {
          derives = fun category =>
            if category == "enum" then
              ["Debug", "Clone", "Serialize", "Deserialize", "PartialEq", "Eq"]
            else if category == "primitive" then
              ["Debug", "Clone", "Serialize", "Deserialize", "PartialEq", "Eq", "Hash"]
            else
              ["Debug", "Clone", "Serialize", "Deserialize"]
        }
    },

  # Specify merge strategy using merged primitives
  merge_strategies = ExtensionLibrary.MergePrimitives.CategoryMergeStrategies.singleton_extension
}
