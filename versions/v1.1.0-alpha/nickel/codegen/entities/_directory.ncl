# Entities Directory - Data entity validation rules
# Entities define the core business objects and their relationships

{
  allowed_extensions = [
    "x-familiar-kind",
    "x-familiar-description",
    "x-familiar-persistence",
    "x-familiar-meta-schema"
  ],

  forbidden_extensions = [
    "x-familiar-service",
    "x-familiar-queue",
    "x-familiar-policy",
    "x-familiar-api",
    "x-familiar-resources",
    "x-familiar-visual",
    "x-familiar-layout",
    "x-familiar-styling"
  ],

  required_extensions = [
    "x-familiar-kind",
    "x-familiar-description",
    "x-familiar-persistence"
  ],

  validation_rules = [],

  hydration = {
    "_entities" = {
      orm_integration = "sea_orm",
      migration_support = true,
      caching_strategy = "entity_based",
      validation_layer = "comprehensive"
    },

    "_operations" = {
      execution_model = "sync",
      transaction_support = true,
      optimistic_locking = true
    }
  },

  validate = fun schema_content => {
    let schema = try {
      std.from_json schema_content
    } catch _ => {
      valid = false,
      errors = ["Invalid JSON schema content"],
      warnings = []
    },

    let missing_required = std.array.filter
      (fun ext => !(std.record.has_field ext schema))
      required_extensions,

    let required_errors = if std.array.length missing_required > 0 then
      std.array.map (fun ext => "Missing required extension: ${ext}") missing_required
    else [],

    let schema_keys = std.record.fields schema,
    let extension_keys = std.array.filter (std.string.starts_with "x-familiar-") schema_keys,
    let forbidden_found = std.array.filter
      (fun ext => std.array.elem ext forbidden_extensions)
      extension_keys,

    let forbidden_errors = if std.array.length forbidden_found > 0 then
      std.array.map (fun ext => "Forbidden extension found: ${ext}") forbidden_found
    else [],

    let all_errors = required_errors @ forbidden_errors,

    {
      valid = std.array.length all_errors == 0,
      errors = all_errors,
      warnings = []
    }
  }
}