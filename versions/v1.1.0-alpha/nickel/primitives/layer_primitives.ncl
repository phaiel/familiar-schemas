# Layer Primitives - Building blocks for architectural layer definitions

{
  # Layer type primitives
  LayerTypePrimitives = {
    entity = "entity",
    node = "node",
    topology = "topology"
  },

  # Layer-specific function primitives
  LayerFunctionPrimitives = {
    architecture = {
      extract_from_raw = fun schema => schema,  # Placeholder - implemented in directory
      validate_pure = fun schema => {valid = true, errors = [], warnings = []}
    },

    infrastructure = {
      extract_from_raw = fun schema => schema,  # Placeholder - implemented in directory
      validate_pure = fun schema => {valid = true, errors = [], warnings = []}
    },

    codegen = {
      extract_from_raw = fun schema => schema,  # Placeholder - implemented in directory
      validate_pure = fun schema => {valid = true, errors = [], warnings = []}
    }
  },

  # Layer compatibility primitives
  LayerCompatibilityPrimitives = {
    architecture_schemas = ["topology"],
    infrastructure_schemas = ["node", "resource", "system"],
    codegen_schemas = ["entity", "component", "primitive"]
  }
}