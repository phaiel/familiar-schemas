# x-familiar-persistence Extension Schema
# Defines data persistence behavior and database mapping

{
  extension = "x-familiar-persistence",
  description = "Defines how data should be persisted and accessed",
  category = "data",

  # JSON Schema definition
  schema = {
    type = "object",
    properties = {
      table = { type = "string" },
      primary_key = {
        oneOf = [
          { type = "string" },
          { type = "array", items = { type = "string" } }
        ]
      },
      relations = {
        type = "array",
        items = {
          type = "object",
          properties = {
            name = { type = "string" },
            type = { type = "string", enum = ["belongs_to", "has_many", "has_one", "many_to_many"] },
            target = { type = "string" },
            foreign_key = { type = "string" }
          },
          required = ["name", "type", "target"]
        }
      },
      constraints = {
        type = "object",
        properties = {
          unique = { type = "array", items = { type = "string" } },
          indexes = { type = "array", items = { type = "string" } }
        }
      }
    },
    required = ["table"]
  },

  # Validation contract
  contract = std.contract.custom (fun label value =>
    if std.record.has_field "table" value then
      if std.string.length value.table > 0 then
        value
      else
        std.contract.blame_with label "x-familiar-persistence.table cannot be empty"
    else
      std.contract.blame_with label "x-familiar-persistence must specify table"
  ),

  # Code generation implications (comprehensive)
  codegen = {
    rust = {
      sea_orm_entity = fun persistence schema_name =>
        let table_name = persistence.table in
        let pk_fields = get_primary_key_fields persistence schema_name in
        {
          entity_struct = generate_entity_struct schema_name table_name,
          model_struct = generate_model_struct schema_name,
          primary_key_enum = generate_primary_key_enum pk_fields,
          relation_enums = generate_relation_enums persistence.relations or [],
          entity_impl = generate_entity_impl table_name pk_fields,
          crud_impl = generate_crud_impl schema_name table_name
        },

      diesel_model = fun persistence schema_name =>
        let table_name = persistence.table in
        {
          table_macro = "#[derive(Queryable, Identifiable)]\n#[table_name = \"${table_name}\"]",
          insertable_struct = generate_insertable_struct schema_name,
          changeset_struct = generate_changeset_struct schema_name
        }
    },

    typescript = {
      typeorm_entity = fun persistence schema_name =>
        let table_name = persistence.table in
        {
          decorators = [
            "@Entity('${table_name}')",
            "@Index(['${get_primary_key_fields persistence schema_name | std.array.join \"', '\" }'])"
          ],
          class_definition = generate_typeorm_class schema_name persistence,
          relations = generate_typeorm_relations persistence.relations or []
        },

      prisma_model = fun persistence schema_name =>
        let table_name = persistence.table in
        {
          model_definition = "model ${schema_name} {\n  ${generate_prisma_fields persistence}\n  ${generate_prisma_relations persistence.relations or []}\n}"
        }
    },

    python = {
      sqlalchemy_model = fun persistence schema_name =>
        let table_name = persistence.table in
        {
          class_definition = "class ${schema_name}(Base):\n    __tablename__ = '${table_name}'\n    ${generate_sqlalchemy_columns persistence}\n    ${generate_sqlalchemy_relationships persistence.relations or []}"
        },

      pydantic_model = fun persistence schema_name =>
        # For API responses, simpler than full ORM
        {
          class_definition = "class ${schema_name}(BaseModel):\n    ${generate_pydantic_fields persistence}"
        }
    }
  },

  # Runtime implications
  runtime = {
    connection_requirements = fun persistence => {
      database_type = infer_database_type persistence,
      connection_pool = true,
      transaction_support = true,
      migration_support = true
    },

    observability = fun persistence => {
      query_metrics = true,
      connection_pool_monitoring = true,
      slow_query_logging = persistence.table != "cache" && persistence.table != "sessions",
      audit_trail = is_sensitive_table persistence.table
    },

    performance = fun persistence => {
      caching_strategy = determine_caching_strategy persistence,
      indexing_strategy = extract_indexes persistence,
      partitioning_strategy = determine_partitioning persistence
    }
  },

  # Migration implications
  migrations = {
    generate_up_migration = fun persistence old_persistence schema_name =>
      let changes = compare_persistence_schemas old_persistence persistence in
      generate_sql_migration changes schema_name,

    generate_down_migration = fun persistence old_persistence schema_name =>
      let changes = compare_persistence_schemas persistence old_persistence in
      generate_sql_migration changes schema_name
  },

  # Directory restrictions
  directory_rules = {
    allowed_in = ["entities", "infrastructure"],
    forbidden_in = ["ui", "components"],
    required_for_kind = ["entity"]
  }
}

# Helper functions
let get_primary_key_fields = fun persistence schema_name =>
  if std.record.has_field "primary_key" persistence then
    if std.is_array persistence.primary_key then
      persistence.primary_key
    else
      [persistence.primary_key]
  else
    ["id"]  # Default fallback

let infer_database_type = fun persistence =>
  # Simple inference based on table naming conventions
  if std.string.contains persistence.table "cache" then
    "redis"
  else if std.string.contains persistence.table "search" then
    "elasticsearch"
  else
    "postgresql"  # Default

let is_sensitive_table = fun table_name =>
  std.array.elem table_name ["users", "credentials", "payments", "audit_logs"]

let determine_caching_strategy = fun persistence =>
  if is_sensitive_table persistence.table then
    "no_cache"
  else if std.record.has_field "relations" persistence && std.array.length persistence.relations > 3 then
    "complex_cache"
  else
    "standard_cache"
