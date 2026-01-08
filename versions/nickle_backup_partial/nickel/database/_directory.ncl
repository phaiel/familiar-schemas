# Database Directory Governance - Inherits from global, data persistence focus
(import "../global.ncl") & {
  # Database schemas define data persistence and storage models
  # They should use modern x-familiar-persistence over legacy extensions

  # ============================================================================
  # DATABASE-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-persistence",     # Modern persistence definition (REQUIRED)
    "x-familiar-table",           # Legacy: migrate to persistence.table
    "x-familiar-primary-key",     # Legacy: migrate to persistence.primary_key
    "x-familiar-relations",       # Legacy: migrate to persistence.relations
    "x-familiar-column-type",     # Legacy: migrate to persistence schema
    "x-familiar-unique",          # Legacy: migrate to persistence constraints
    "x-familiar-index",           # Legacy: migrate to persistence indexes
    "x-familiar-foreign-key",     # Legacy: migrate to persistence.relations
    "x-familiar-migration",       # Database migration specifications
    "x-familiar-backup-strategy"  # Data backup requirements
  ],

  # Database schemas require persistence definitions
  required_extensions = (import "../global.ncl").required_extensions @ [
    "x-familiar-persistence"      # Modern persistence definition
  ],

  # ============================================================================
  # DATABASE VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # Database schemas must use modern persistence (not legacy extensions)
    (std.contract.custom (fun label schema =>
      let has_legacy = std.record.has_field "x-familiar-table" schema ||
                      std.record.has_field "x-familiar-primary-key" schema ||
                      std.record.has_field "x-familiar-relations" schema in
      let has_modern = std.record.has_field "x-familiar-persistence" schema in

      if has_legacy && not has_modern then
        std.contract.blame_with label "Database schemas must use x-familiar-persistence instead of legacy extensions (x-familiar-table, x-familiar-primary-key, etc.). Migrate to modern persistence format."
      else if has_legacy && has_modern then
        std.contract.blame_with label "Database schemas cannot mix legacy and modern extensions. Remove legacy extensions after migration."
      else
        schema
    )),

    # Modern persistence must be complete and valid
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-persistence" schema then
        let persistence = schema."x-familiar-persistence" in

        # Must specify table name
        if not (std.record.has_field "table" persistence) then
          std.contract.blame_with label "x-familiar-persistence must specify table name"
        else
          # Must specify primary key
          if not (std.record.has_field "primary_key" persistence) then
            std.contract.blame_with label "x-familiar-persistence must specify primary_key"
          else
            # Validate primary key format
            let pk = persistence.primary_key in
            if std.is_string pk || std.is_array pk then
              schema
            else
              std.contract.blame_with label "primary_key must be string or array of strings"
          end
        end
      else
        schema
    )),

    # Database relations must be properly specified
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-persistence" schema then
        let persistence = schema."x-familiar-persistence" in
        if std.record.has_field "relations" persistence then
          let relations = persistence.relations in
          if std.is_array relations then
            # Validate each relation
            let valid_relations = std.array.all (fun relation =>
              std.record.has_field "type" relation &&
              std.record.has_field "target" relation &&
              std.array.elem relation.type ["belongs_to", "has_many", "has_one", "many_to_many"]
            ) relations in
            if valid_relations then
              schema
            else
              std.contract.blame_with label "Database relations must specify valid type and target fields"
          else
            std.contract.blame_with label "x-familiar-persistence.relations must be an array"
        else
          schema
      else
        schema
    )),

    # Database schemas should specify backup strategy for critical data
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-backup-strategy" schema then
        let strategy = schema."x-familiar-backup-strategy" in
        if std.array.elem strategy ["continuous", "daily", "weekly", "monthly", "none"] then
          schema
        else
          std.contract.blame_with label "Invalid backup strategy. Must be: continuous, daily, weekly, monthly, or none"
      else
        # Optional for now, but recommended
        schema
    ))
  ],

  # ============================================================================
  # DATABASE HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Database-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "database"
    },

    # Database-focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = true,
      performance_metrics = true,
      query_performance_monitoring = true,
      connection_pool_monitoring = true
    },

    # Database-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "database",
      backup_enabled = true,
      migration_support = true,
      connection_pooling = true
    },

    # Database-specific runtime configuration
    "_database" = {
      connection_pool = {
        min_connections = 5,
        max_connections = 20,
        connection_timeout = "30s"
      },
      migration_strategy = "versioned",
      backup_schedule = "daily_02_00",
      read_replica_support = true,
      query_logging = {
        slow_query_threshold = "1s",
        enabled = true
      }
    }
  },

  # ============================================================================
  # MIGRATION SUPPORT
  # ============================================================================

  migration_support = {
    legacy_mappings = {
      "x-familiar-table" = "x-familiar-persistence.table",
      "x-familiar-primary-key" = "x-familiar-persistence.primary_key",
      "x-familiar-relations" = "x-familiar-persistence.relations",
      "x-familiar-column-type" = "x-familiar-persistence.schema",
      "x-familiar-unique" = "x-familiar-persistence.constraints.unique",
      "x-familiar-index" = "x-familiar-persistence.indexes"
    },

    migration_required = true,
    migration_deadline = "2024-12-31"
  }
}
