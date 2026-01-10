# Schema Composition Engine
# Uses extension schemas to understand and compose full schema behavior

let extension_schemas = {
  "x-familiar-kind" = import "../extensions/kind.ncl",
  "x-familiar-persistence" = import "../extensions/persistence.ncl",
  "x-familiar-service" = import "../extensions/service.ncl"
}

# Main composition function using Nickel's & operator
let compose_schema_with_extensions = fun base_schema =>
  # Extract extensions from schema
  let extensions = extract_extensions base_schema in

  # Load extension schemas
  let loaded_extensions = std.record.map (fun ext_name _ =>
    extension_schemas."${ext_name}" or {}
  ) extensions

  # Create composition context
  let composition_context = {
    schema = base_schema,
    extensions = extensions,
    extension_schemas = loaded_extensions,
    directory_path = get_directory_path base_schema,
    inheritance_chain = []  # Will be filled by directory processing
  }

  # Apply extension behaviors
  let schema_with_behaviors = apply_extension_behaviors composition_context in

  # Apply directory rules
  let schema_with_directory = apply_directory_rules schema_with_behaviors in

  # Generate derived schemas
  let derived_schemas = generate_derived_schemas schema_with_directory in

  # Return composed result
  schema_with_directory & {
    _composition = {
      applied_extensions = std.record.keys extensions,
      derived_schemas = derived_schemas,
      composition_timestamp = std.to_string (std.contract.current_time)
    }
  }

# Extract all x-familiar-* extensions from a schema
let extract_extensions = fun schema =>
  std.record.filter (fun key _ => std.string.starts_with key "x-familiar-") schema

# Apply behaviors defined in extension schemas
let apply_extension_behaviors = fun context =>
  let schema = context.schema in
  let extensions = context.extensions in

  # Apply each extension's behavior
  std.record.fold (fun acc ext_name ext_value =>
    let ext_schema = context.extension_schemas."${ext_name}" or {} in
    let behavior_result = apply_extension_behavior ext_schema ext_value schema in
    acc & behavior_result
  ) schema extensions

# Apply a single extension's behavior
let apply_extension_behavior = fun ext_schema ext_value base_schema =>
  if std.record.has_field "codegen" ext_schema then
    let codegen_rules = ext_schema.codegen in
    apply_codegen_behavior codegen_rules ext_value base_schema
  else if std.record.has_field "runtime" ext_schema then
    let runtime_rules = ext_schema.runtime in
    apply_runtime_behavior runtime_rules ext_value base_schema
  else
    {}  # No behavior to apply

# Apply codegen behavior from extension
let apply_codegen_behavior = fun codegen_rules ext_value base_schema =>
  # This would be expanded to actually generate code
  # For now, just mark what would be generated
  {
    _codegen = {
      rust_struct = would_generate_rust_struct codegen_rules ext_value base_schema,
      typescript_interface = would_generate_typescript_interface codegen_rules ext_value base_schema,
      python_model = would_generate_python_model codegen_rules ext_value base_schema
    }
  }

# Apply runtime behavior from extension
let apply_runtime_behavior = fun runtime_rules ext_value base_schema =>
  if std.record.has_field "observability" runtime_rules then
    let obs_rules = runtime_rules.observability in
    if std.is_function obs_rules then
      let obs_config = obs_rules ext_value in
      { _observability = obs_config }
    else
      {}
  else
    {}

# Apply directory-specific rules
let apply_directory_rules = fun schema =>
  # This would use the directory _directory.ncl files
  # For now, just mark that directory rules would be applied
  schema & {
    _directory_rules_applied = true
  }

# Generate derived schemas based on extensions
let generate_derived_schemas = fun schema =>
  let extensions = extract_extensions schema in

  # Generate database migration if persistence is defined
  let db_migration = if std.record.has_field "x-familiar-persistence" extensions then
    generate_db_migration schema
  else
    null

  # Generate API routes if service is defined
  let api_routes = if std.record.has_field "x-familiar-service" extensions then
    generate_api_routes schema
  else
    null

  # Generate GraphQL schema if appropriate
  let graphql_schema = if should_generate_graphql schema then
    generate_graphql_schema schema
  else
    null

  {
    database_migration = db_migration,
    api_routes = api_routes,
    graphql_schema = graphql_schema
  }

# Helper functions
let get_directory_path = fun schema =>
  # This would determine the schema's directory path
  # For now, return a placeholder
  "infrastructure/resources"

let would_generate_rust_struct = fun codegen_rules ext_value base_schema =>
  "Would generate Rust struct based on ${base_schema.title or 'Unknown'} with extension ${std.to_string ext_value}"

let would_generate_typescript_interface = fun codegen_rules ext_value base_schema =>
  "Would generate TypeScript interface for ${base_schema.title or 'Unknown'}"

let would_generate_python_model = fun codegen_rules ext_value base_schema =>
  "Would generate Python model for ${base_schema.title or 'Unknown'}"

let generate_db_migration = fun schema =>
  "CREATE TABLE ${schema.'x-familiar-persistence'.table} (...);"

let generate_api_routes = fun schema =>
  let service = schema."x-familiar-service" in
  "Generated API routes for service ${service.name}"

let should_generate_graphql = fun schema =>
  # Logic to determine if GraphQL schema should be generated
  std.record.has_field "x-familiar-kind" schema &&
  schema."x-familiar-kind" == "entity"

let generate_graphql_schema = fun schema =>
  "type ${schema.title or 'Unknown'} { id: ID! }"

# Advanced composition: Merge multiple schemas with conflict resolution
let compose_multiple_schemas = fun schemas =>
  std.array.fold (fun acc schema =>
    acc & resolve_conflicts acc schema
  ) {} schemas

# Conflict resolution strategy
let resolve_conflicts = fun base_schema overlay_schema =>
  # Extension-specific conflict resolution
  let base_extensions = extract_extensions base_schema in
  let overlay_extensions = extract_extensions overlay_schema in

  # For each conflicting extension, apply resolution rules
  let resolved_extensions = std.record.map (fun ext_name overlay_value =>
    let base_value = base_extensions."${ext_name}" or null in
    if base_value != null then
      resolve_extension_conflict ext_name base_value overlay_value
    else
      overlay_value
  ) overlay_extensions

  # Merge non-extension parts with overlay taking precedence
  let base_without_extensions = remove_extensions base_schema in
  let overlay_without_extensions = remove_extensions overlay_schema in

  base_without_extensions & overlay_without_extensions & resolved_extensions

# Resolve conflicts between extension values
let resolve_extension_conflict = fun ext_name base_value overlay_value =>
  if ext_name == "x-familiar-persistence" then
    # For persistence, merge table definitions
    base_value & overlay_value
  else if ext_name == "x-familiar-service" then
    # For services, overlay takes precedence
    overlay_value
  else
    # Default: overlay wins
    overlay_value

# Remove extensions from schema
let remove_extensions = fun schema =>
  std.record.filter (fun key _ => not (std.string.starts_with key "x-familiar-")) schema

# Export the main composition function
{
  compose_schema_with_extensions = compose_schema_with_extensions,
  compose_multiple_schemas = compose_multiple_schemas,
  extract_extensions = extract_extensions,
  apply_extension_behaviors = apply_extension_behaviors
}
