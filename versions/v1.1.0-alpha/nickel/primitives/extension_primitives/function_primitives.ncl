# Extension Function Primitives - Common function patterns for extensions

{
  # Validation function patterns
  ValidationFunctions = {
    # Generic field validation
    validate_field = fun schema field_name validator =>
      let field_value = std.record.get field_name schema in
      validator field_value,

    # Array element validation
    validate_array_elements = fun array validator =>
      std.array.map validator array,

    # Conditional validation based on context
    validate_conditional = fun condition validator value =>
      if condition value then validator value else value,

    # Schema-level validation
    validate_schema_structure = fun schema validators =>
      std.record.map (fun field_name validator =>
        let field_value = std.record.get_or field_name schema null in
        if field_value != null then validator field_value else null
      ) validators
  },

  # Processing function patterns
  ProcessingFunctions = {
    # Extract field with default
    extract_field = fun schema field_name default =>
      std.record.get_or field_name schema default,

    # Transform field value
    transform_field = fun schema field_name transformer =>
      let value = std.record.get_or field_name schema null in
      if value != null then transformer value else null,

    # Merge field into record
    merge_field = fun base_record field_name field_value =>
      base_record & { "%{field_name}" = field_value },

    # Filter fields by predicate
    filter_fields = fun record predicate =>
      std.record.filter (fun field_name field_value => predicate field_name field_value) record
  },

  # Lookup function patterns
  LookupFunctions = {
    # Simple key-value lookup
    lookup = fun map key default =>
      std.record.get_or key map default,

    # Lookup with transformation
    lookup_transform = fun map key transformer default =>
      let value = std.record.get_or key map default in
      transformer value,

    # Multi-key lookup
    lookup_multi = fun map keys default =>
      let results = std.array.map (fun key => std.record.get_or key map default) keys in
      std.array.filter (fun result => result != default) results
  },

  # Integration function patterns
  IntegrationFunctions = {
    # Graph node metadata generation
    make_node_metadata = fun schema extension_data =>
      {
        schema_id = std.record.get_or "id" schema null,
        extension_name = extension_data.name,
        category = extension_data.category,
        graph_value = extension_data.graph_value or "medium"
      } & extension_data,

    # Code generation metadata
    make_codegen_metadata = fun schema extension_data =>
      {
        target_languages = extension_data.codegen_targets or ["rust"],
        schema_type = extension_data.category,
        validation_level = extension_data.validation or "basic"
      } & extension_data
  }
}