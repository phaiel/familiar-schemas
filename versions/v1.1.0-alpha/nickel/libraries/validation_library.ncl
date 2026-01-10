# Validation Library - Merged composition of validation primitives

let ValidationPrimitives = import "../primitives/validation_primitives.ncl" in

# Merge validation primitives into complete validation library
ValidationPrimitives & {
  # Library metadata

  # Combined validation (structure + purity)
  validate_comprehensive = fun schema purity_validator structure_validator =>
    let purity = purity_validator schema in
    let structure = structure_validator schema in
    {
      valid = purity.valid && structure.valid,
      errors = purity.errors @ structure.errors,
      warnings = purity.warnings @ structure.warnings,
      purity_check = purity,
      structure_check = structure
    },

  # Layer-specific validation functions
  validation_functions = {
    entity_validate = fun schema =>
      ValidationPrimitives.EntityValidation.validate_pure_entity schema,

    entity_validate_comprehensive = fun schema =>
      validate_comprehensive
        schema
        ValidationPrimitives.EntityValidation.validate_pure_entity
        ValidationPrimitives.EntityValidation.validate_entity_structure,

    node_validate = fun schema =>
      ValidationPrimitives.NodeValidation.validate_pure_compute schema,

    node_validate_comprehensive = fun schema =>
      validate_comprehensive
        schema
        ValidationPrimitives.NodeValidation.validate_pure_compute
        ValidationPrimitives.NodeValidation.validate_compute_resources,

    topology_validate = ValidationPrimitives.TopologyValidation.validate_pure_topology
  }
}