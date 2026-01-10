# Simple Nickel validation for testing
# Validates that a JSON schema can be parsed and has basic required fields

let schema_content = std.record.get_or "SCHEMA_FILE" std.env "" in

let validation_result = if schema_content == "" then
  {
    valid = false,
    errors = ["No schema content provided"],
    warnings = []
  }
else
  let schema = std.from_json schema_content in

  # Basic validation: check if schema has required fields
  let has_dollar_schema = std.record.has_field "$schema" schema in
  let has_kind = std.record.has_field "x-familiar-kind" schema in

  if has_dollar_schema && has_kind then
    {
      valid = true,
      errors = [],
      warnings = []
    }
  else
    let missing_fields = (if has_dollar_schema then [] else ["$schema"]) @
                        (if has_kind then [] else ["x-familiar-kind"]) in
    {
      valid = false,
      errors = ["Missing required fields: " ++ std.string.join ", " missing_fields],
      warnings = []
    }
in

# Export orthologically-validated result
validation_result
