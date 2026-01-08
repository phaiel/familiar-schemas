/**
 * Infrastructure Module - Pure Node/Compute Governance
 *
 * Ensures schemas in infrastructure/ represent NODES: compute resources
 * and processing capabilities that execute entity transformations.
 *
 * @orthology This module enforces the "Where" - runtime execution environments
 * @guarantees NO business logic, NO data definitions, NO wiring/topology concerns
 * @runtime Pure compute specifications for deployment and scaling
 *
 * @example
 *   # Extract compute specs from mixed schema
 *   let node = (import "infrastructure/_directory.ncl").files._.extract_compute_resources(raw_schema)
 *
 *   # Validate compute purity
 *   let result = (import "infrastructure/_directory.ncl").files._.validate_pure_compute(node)
 */

{
  # ============================================================================
  # INFRASTRUCTURE CONTRACTS - Node Transformation Rules
  # ============================================================================

  # Contract for all files in infrastructure/ - must be pure compute nodes
  files = {
    _ : {
      # Implement NodeContract interface with type safety
      extract_from_raw = fun raw_schema => extract_compute_resources raw_schema,
      validate_pure = fun schema => validate_pure_compute schema,
      get_layer = `node,

      # extract_compute_resources : Schema -> Node
      # Transforms mixed schemas into pure compute resource specifications
      # Extracts runtime deployment and scaling configurations
      extract_compute_resources = fun raw_schema => {
        # Extract ONLY compute-related fields
        let compute_fields = [
          "x-familiar-node-type", "x-familiar-resources", "x-familiar-scaling",
          "x-familiar-monitoring-enabled", "x-familiar-deploy-config",
          "x-familiar-environment", "x-familiar-networking"
        ] in

        # Filter to compute concerns only
        let compute_only = std.record.filter
          (fun key _ =>
            std.array.elem key compute_fields ||
            key == "x-familiar-kind" ||
            key == "x-familiar-description" ||
            key == "$schema" ||
            key == "$id"
          )
          raw_schema in

        compute_only & {
          "x-familiar-layer" = "node",
          "x-familiar-orthology" = "compute_only",
          "x-familiar-extracted-from" = raw_schema."$id" | "unknown"
        }
      },

      # validate_pure_compute : Node -> ValidationResult
      # Validates that node contains only compute specifications (no data/business logic)
      # Ensures orthological purity - nodes must be pure compute resources only
      validate_pure_compute = fun compute_schema => {
        let data_indicators = [
          "properties", "type", "required", "additionalProperties",
          "x-familiar-business-rules", "x-familiar-domain-context"
        ] in

        let topology_indicators = [
          "x-familiar-service", "x-familiar-api-endpoints", "bindings"
        ] in

        let has_data = std.array.any (fun field =>
          std.record.has_field field compute_schema
        ) data_indicators in

        let has_topology = std.array.any (fun field =>
          std.record.has_field field compute_schema
        ) topology_indicators in

        if has_data then
          { valid = false, errors = ["Compute schemas cannot contain data definitions. Extract to codegen layer."], warnings = [] }
        else if has_topology then
          { valid = false, errors = ["Compute schemas cannot contain topology wiring. Extract to architecture layer."], warnings = [] }
        else
          { valid = true, errors = [], warnings = [] }
      },

      # validate_compute_resources : Node -> ValidationResult
      # Ensures compute nodes specify required resource allocations (CPU, memory)
      # Validates deployment prerequisites for infrastructure provisioning
      validate_compute_resources = fun compute_schema => {
        if std.record.has_field "x-familiar-resources" compute_schema then
          let resources = compute_schema."x-familiar-resources" in
          let has_cpu = std.record.has_field "cpu" resources in
          let has_memory = std.record.has_field "memory" resources in

          if has_cpu && has_memory then
            { valid = true, errors = [], warnings = [] }
          else
            { valid = false, errors = ["Compute resources must specify cpu and memory requirements"], warnings = [] }
        else
          { valid = false, errors = ["Compute schemas must define x-familiar-resources"], warnings = [] }
      }
    }
  },

  # ============================================================================
  # CONTRACT COMPOSITION & STRATEGIC MERGING
  # ============================================================================

  # Compose multiple node contracts (e.g., base + specialized)
  # compose_node_contracts : Array<NodeContract> -> NodeContract
  compose_node_contracts = fun node_contracts =>
    (import "../global.ncl").compose_contracts node_contracts,

  # Strategic merging for node configurations
  # merge_node_configs : _ -> _ -> MergeStrategy -> _
  merge_node_configs = fun base_config override_config strategy =>
    (import "../global.ncl").strategic_merge base_config override_config strategy,

  # Environment-specific node configuration merging
  # merge_environment_config : _ -> String -> _
  merge_environment_config = fun base_node_config environment =>
    let env_overrides = get_environment_overrides environment in
    merge_node_configs base_node_config env_overrides `priority,

  # get_environment_overrides : String -> _
  # Get environment-specific configuration overrides
  get_environment_overrides = fun environment =>
    match environment {
      "production" => {
        "x-familiar-resources" = {
          "cpu" = "4",
          "memory" = "8Gi",
          "replicas" = 5,
          "priority" = 100
        },
        "x-familiar-scaling" = {
          "min_replicas" = 3,
          "max_replicas" = 10,
          "priority" = 100
        }
      },
      "staging" => {
        "x-familiar-resources" = {
          "cpu" = "2",
          "memory" = "4Gi",
          "replicas" = 2,
          "priority" = 50
        },
        "x-familiar-scaling" = {
          "min_replicas" = 1,
          "max_replicas" = 3,
          "priority" = 50
        }
      },
      "development" => {
        "x-familiar-resources" = {
          "cpu" = "1",
          "memory" = "2Gi",
          "replicas" = 1,
          "priority" = 10
        },
        "x-familiar-scaling" = {
          "enabled" = false,
          "priority" = 10
        }
      },
      _ => {}
    },

  # ============================================================================
  # CONTRACT TESTING FRAMEWORK
  # ============================================================================

  # Test cases for node contract validation
  test_cases = [
    # Valid node test
    {
      name = "valid_node_schema",
      input = {
        "x-familiar-kind" = "node",
        "x-familiar-node-type" = "api_service",
        "x-familiar-description" = "Valid API node",
        "x-familiar-resources" = {
          "cpu" = "2",
          "memory" = "4Gi"
        }
      },
      expected_output = {
        "x-familiar-kind" = "node",
        "x-familiar-node-type" = "api_service",
        "x-familiar-description" = "Valid API node",
        "x-familiar-resources" = {
          "cpu" = "2",
          "memory" = "4Gi"
        },
        "x-familiar-layer" = "node",
        "x-familiar-orthology" = "compute_only"
      },
      expected_errors = [],
      description = "Valid node schema should pass extraction and validation"
    },

    # Node with forbidden data/business logic
    {
      name = "node_with_business_logic",
      input = {
        "x-familiar-kind" = "node",
        "x-familiar-description" = "Node with forbidden business logic",
        "x-familiar-resources" = { "cpu" = "2" },
        "properties" = { "user_id" = { "type" = "string" } },  # FORBIDDEN
        "x-familiar-business-rules" = ["rule1"]  # FORBIDDEN
      },
      expected_output = {
        "x-familiar-kind" = "node",
        "x-familiar-description" = "Node with forbidden business logic",
        "x-familiar-resources" = { "cpu" = "2" },
        "x-familiar-layer" = "node",
        "x-familiar-orthology" = "compute_only"
      },
      expected_errors = ["Compute schemas cannot contain data definitions. Extract to codegen layer."],
      description = "Node with business logic should fail validation"
    }
  ],

  # run_tests : _ -> Array<ContractTestResult>
  # Execute all contract tests for this layer
  run_tests = fun =>
    (import "../global.ncl").run_contract_tests files._ test_cases,

  # ============================================================================
  # INFRASTRUCTURE HYDRATION - Compute Execution Context
  # ============================================================================

  hydration = {
    "_infrastructure" = {
      monitoring = true,
      logging = "structured", 
      metrics = "prometheus",
      tracing = "opentelemetry",
      orthology_level = "compute",
      business_logic_free = true,       # Guarantee: no business logic in compute layer
      data_definition_free = true       # Guarantee: no data schemas in compute layer
    },

    "_operations" = {
      deployment_strategy = "rolling",
      scaling_policy = "horizontal", 
      health_checks = true,
      resource_management = "kubernetes",
      service_mesh = "istio"
    },

    "_compute" = {
      isolation_level = "container",
      scheduling_policy = "kubernetes_scheduler", 
      resource_quotas = true,
      auto_scaling = true,
      fault_tolerance = "replication"
    }
  }
}
