# Extension Library - Merged composition of extension primitives
let MetadataPrimitives = import "../../primitives/extension_primitives/metadata_primitives.ncl" in
let ContractPrimitives = import "../../primitives/extension_primitives/contract_primitives.ncl" in
let FunctionPrimitives = import "../../primitives/extension_primitives/function_primitives.ncl" in
let IntegrationPrimitives = import "../../primitives/extension_primitives/integration_primitives.ncl" in
let MergePrimitives = import "../../primitives/extension_primitives/merge_primitives.ncl" in

# Merge extension primitives into complete extension library
MetadataPrimitives
& ContractPrimitives
& FunctionPrimitives
& IntegrationPrimitives
& MergePrimitives
& {
  # Add library-specific enhancements

  # Enhanced extension builders
  extension_builders = {
    # Build complete extension from components
    build_extension = fun metadata contract functions integration merge_strategy =>
      {
        extension = metadata,
        contract = contract,
        functions = functions,
        graph_integration = integration.GraphIntegrationPrimitives,
        codegen = integration.CodegenIntegrationPrimitives,
        validation_integration = integration.ValidationIntegrationPrimitives,
        merge_strategies = merge_strategy
      },

    # Build category-specific extension template
    build_category_extension = fun category name description =>
      {
        extension =
          MetadataPrimitives.BaseExtensionMetadata
          & MetadataPrimitives.CategoryTemplates."%{category}"
          & {
            name = name,
            description = description
          },
        contract = ContractPrimitives.BaseContractPatterns.type_check "String",
        functions = FunctionPrimitives.ValidationFunctions,
        graph_integration = IntegrationPrimitives.GraphIntegrationPrimitives,
        codegen = IntegrationPrimitives.CodegenIntegrationPrimitives,
        validation_integration = IntegrationPrimitives.ValidationIntegrationPrimitives,
        merge_strategies = MergePrimitives.StrategySelectors.by_category category
      }
  },

  # Validation helpers using merged primitives
  validation_helpers = {
    validate_extension_structure = fun extension =>
      std.record.has_field "extension" extension
      && std.record.has_field "contract" extension
      && std.record.has_field "functions" extension,

    validate_extension_metadata = fun extension_metadata =>
      ContractPrimitives.BaseContractPatterns.record_with_fields ["name", "description", "category"].validate extension_metadata,

    validate_extension_contract = fun contract =>
      std.type.of contract == "Function" # Contract should be a function
  },

  # Composition helpers using merged primitives
  composition_helpers = {
    compose_extension_with_schema = fun extension schema =>
      let validated_schema = extension.contract.validate schema in
      let processed_schema = extension.functions.validate_schema_structure validated_schema extension.functions in
      processed_schema
      & {
        "_extension_metadata" = extension.extension,
        "_graph_metadata" = extension.graph_integration.make_node_metadata processed_schema extension.extension,
        "_codegen_metadata" = extension.codegen.make_codegen_metadata processed_schema extension.extension
      },

    merge_extensions = fun base_extension override_extension =>
      let merge_strategy = base_extension.merge_strategies.compose in
      merge_strategy base_extension override_extension
  }
}
