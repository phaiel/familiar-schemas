# Extension Contract Primitives - Common contract patterns for extensions

{
  # Base contract patterns
  BaseContractPatterns = {
    # String enum validation
    string_enum = fun valid_values => std.contract.custom (fun label value =>
      if std.array.elem value valid_values then value
      else std.contract.blame_with label "Must be one of: ${std.string.join ", " valid_values}. Got: ${std.to_string value}"
    ),

    # Type validation
    type_check = fun expected_type => std.contract.custom (fun label value =>
      let actual_type = std.type.of value in
      if actual_type == expected_type then value
      else std.contract.blame_with label "Expected ${std.to_string expected_type}, got ${std.to_string actual_type}"
    ),

    # Array validation
    array_of = fun element_contract => std.contract.custom (fun label value =>
      if std.type.of value == "Array then
        std.array.map (fun elem => element_contract.validate elem) value
      else std.contract.blame_with label "Expected array, got ${std.to_string (std.type.of value)}"
    ),

    # Record field validation
    record_with_fields = fun required_fields => std.contract.custom (fun label value =>
      if std.type.of value == "Record then
        let missing_fields = std.array.filter (fun field => !std.record.has_field field value) required_fields in
        if std.array.is_empty missing_fields then value
        else std.contract.blame_with label "Missing required fields: ${std.string.join ", " missing_fields}"
      else std.contract.blame_with label "Expected record, got ${std.to_string (std.type.of value)}"
    )
  },

  # Common validation patterns
  ValidationPatterns = {
    # Edge type validation
    edge_type = BaseContractPatterns.string_enum [
      "depends_on", "communicates_with", "contains", "implements",
      "extends", "aggregates", "orchestrates", "transforms",
      "references", "configures", "deploys_to", "routes_to"
    ],

    # Type category validation
    type_category = BaseContractPatterns.string_enum [
      "primitive", "composite", "enum", "union", "reference"
    ],

    # Component type validation
    component_type = BaseContractPatterns.string_enum [
      "entity", "system", "component", "primitive", "node"
    ]
  },

  # Contract composition helpers
  CompositionHelpers = {
    # Combine multiple contracts with AND logic
    combine_contracts = fun contracts => std.contract.custom (fun label value =>
      std.array.fold_left (fun acc contract => contract.validate acc) value contracts
    ),

    # Make contract optional
    optional = fun contract => std.contract.custom (fun label value =>
      if value == null then value else contract.validate value
    )
  }
}