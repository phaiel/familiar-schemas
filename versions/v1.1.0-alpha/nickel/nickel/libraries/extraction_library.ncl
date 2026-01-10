# Extraction Library - Merged composition of extraction primitives

let ExtractionPrimitives = import "../primitives/extraction_primitives.ncl" in

# Merge extraction primitives into complete extraction library
ExtractionPrimitives & {
  # Library metadata

  # Enhanced extraction with validation
  extract_with_validation = fun raw_schema extract_fn validate_fn =>
    let extracted = extract_fn raw_schema in
    let validation = validate_fn extracted in
    {
      data = extracted,
      validation = validation,
      success = validation.valid
    },

  # Layer-specific extraction functions
  extraction_functions = {
    entity_extract = ExtractionPrimitives.EntityExtraction.extract_entity_data,
    node_extract = ExtractionPrimitives.NodeExtraction.extract_compute_resources,
    topology_extract = ExtractionPrimitives.NodeExtraction.extract_topology_wiring,
    domain_extract = ExtractionPrimitives.DomainExtraction.extract_domain_logic
  }
}