# Resources Subdirectory Governance - Inherits from infrastructure
(import "../_directory.ncl") & {

  # Extensions that don't make sense for resources
  forbidden_extensions = (import "../_directory.ncl").forbidden_extensions @ [
    "x-familiar-persistence",        # Resources provide persistence, don't define it
    "x-familiar-table",              # Resources provide storage, don't define tables
    "x-familiar-queue"               # Resources are used by queues, not queues themselves
  ],

  # ============================================================================
  # RESOURCE VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../_directory.ncl").validation_rules @ [
    # Resource type validation (stricter than infrastructure level)
    (std.contract.custom (fun label schema =>
      let resource_type = schema."x-familiar-resource-type" in
      if std.array.elem resource_type [
        "kafka_cluster", "redis_cluster", "postgresql", "mongodb", "s3_bucket",
        "dynamodb", "elasticsearch", "rabbitmq", "gpu_cluster", "model_registry"
      ] then
        schema
      else
        std.contract.blame_with label "Unknown resource type: ${resource_type}. Valid types: kafka_cluster, redis_cluster, postgresql, mongodb, s3_bucket, dynamodb, elasticsearch, rabbitmq, gpu_cluster, model_registry"
    )),

    # Resources must have endpoints
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-endpoint" schema then
        schema
      else
        std.contract.blame_with label "Resources must specify x-familiar-endpoint"
    )),

    # Storage resources must specify retention
    (std.contract.custom (fun label schema =>
      let resource_type = schema."x-familiar-resource-type" in
      if std.array.elem resource_type ["kafka_cluster", "s3_bucket", "dynamodb", "elasticsearch"] then
        if std.record.has_field "x-familiar-retention" schema then
          schema
        else
          std.contract.blame_with label "Storage resources (${resource_type}) must specify x-familiar-retention"
      else
        schema
    )),

    # Message resources must specify topics/partitions
    (std.contract.custom (fun label schema =>
      let resource_type = schema."x-familiar-resource-type" in
      if std.array.elem resource_type ["kafka_cluster", "rabbitmq"] then
        if std.record.has_field "x-familiar-topics" schema then
          if std.record.has_field "x-familiar-partitions" schema then
            schema
          else
            std.contract.blame_with label "Message resources must specify x-familiar-partitions"
        else
          std.contract.blame_with label "Message resources must specify x-familiar-topics"
      else
        schema
    )),

    # Database resources must specify connection pooling
    (std.contract.custom (fun label schema =>
      let resource_type = schema."x-familiar-resource-type" in
      if std.array.elem resource_type ["postgresql", "mongodb", "redis_cluster"] then
        if std.record.has_field "x-familiar-connection-pool" schema then
          schema
        else
          std.contract.blame_with label "Database resources must specify x-familiar-connection-pool"
      else
        schema
    ))
  ],

  # ============================================================================
  # RESOURCE HYDRATION
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    # Resource-specific metadata
    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "infrastructure.resources"
    },

    # Enhanced observability for resources
    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      resource_monitoring = true,
      performance_metrics = true,
      capacity_alerts = true,
      health_check_path = "/health"
    },

    # Resource-specific operations
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "resource",
      auto_scaling = true,
      backup_strategy = "continuous",
      maintenance_window = "sunday_02_00",
      upgrade_strategy = "rolling"
    },

    # Resource-type-specific defaults
    "_resources" = {
      # Applied based on resource type
      kafka_cluster = {
        default_partitions = 3,
        default_replication_factor = 1,
        retention_policy = "7d",
        monitoring_topics = ["__consumer_offsets", "__transaction_state"]
      },

      postgresql = {
        connection_pool_size = 10,
        max_connections = 100,
        ssl_mode = "require",
        backup_frequency = "daily"
      },

      redis_cluster = {
        max_memory = "1gb",
        eviction_policy = "allkeys-lru",
        cluster_enabled = true
      }
    }
  },

  # ============================================================================
  # RESOURCE-TYPE SPECIFIC RULES
  # ============================================================================

  resource_type_rules = {
    kafka_cluster = {
      required_fields = ["x-familiar-topics", "x-familiar-partitions"],
      recommended_fields = ["x-familiar-replication-factor", "x-familiar-retention"],
      hydration_defaults = {
        monitoring_topics = ["__consumer_offsets", "__transaction_state"],
        default_partitions = 3
      }
    },

    postgresql = {
      required_fields = ["x-familiar-connection-pool"],
      recommended_fields = ["x-familiar-ssl-mode"],
      hydration_defaults = {
        max_connections = 100,
        connection_timeout = "30s"
      }
    },

    s3_bucket = {
      required_fields = ["x-familiar-retention"],
      recommended_fields = ["x-familiar-versioning", "x-familiar-encryption"],
      hydration_defaults = {
        versioning_enabled = true,
        encryption = "AES256"
      }
    }
  }
}
