/**
 * Global Orthological Governance Module
 *
 * Foundation for all Familiar schema validation and transformation.
 * Establishes orthological principles: Entity/Node/Topology separation.
 *
 * @overview This module defines the core architectural governance that ensures
 * schemas maintain clean separation between data (entities), compute (nodes),
 * and wiring (topology). All other modules inherit from these global rules.
 *
 * @orthology
 * - Entity: Pure data structures in "Typespace" (codegen/)
 * - Node: Compute resources and runtime environments (infrastructure/)
 * - Topology: Wiring between entities and nodes (architecture/)
 *
 * @example
 *   # Basic schema validation
 *   let result = (import "global.ncl").validate_schema my_schema
 *
 *   # Access orthological metadata
 *   let principles = (import "global.ncl").orthology.entity_node_topology
 */

{
  # ============================================================================
  # TYPE SYSTEM DEFINITIONS - Custom Types for Orthological Contracts
  # ============================================================================

  # Orthological layer types - compile-time type safety
  OrthologicalLayer = [| `entity, `node, `topology, `domain |] in

  # Schema kind enumeration - replaces runtime string checks
  SchemaKind = [|
    `node, `system, `action, `entity, `resource, `queue,
    `component, `ui, `tenant, `technique, `environment
  |] in

  # Pattern category types for architecture validation
  PatternCategory = [| `creational, `structural, `behavioral, `concurrency, `architectural |] in

  # Validation result union type - type-safe error handling
  ValidationResult = { valid: Bool, errors: Array<String>, warnings: Array<String> } in

  # Structured error type with context
  ValidationError = {
    field | String,
    expected | String,
    actual | String,
    suggestion | String,
    severity | [| `error, `warning, `info |]
  } in

  # Contract interface types - reusable contract definitions
  SchemaContract = {
    validate | SchemaContract -> Schema,
    transform | SchemaContract -> Schema -> Schema,
    extract | SchemaContract -> Schema -> { data: Schema, metadata: { layer: OrthologicalLayer } }
  } in

  # Orthological metadata
  orthology = {
    level = "global",
    description = "Entity/Node/Topology orthological governance for all Familiar schemas",
    entity_node_topology = {
      entity = "data_purity",      # What gets processed
      node = "compute_purity",     # Where processing happens
      topology = "wiring_purity"   # How they connect
    }
  },

  # ============================================================================
  # TYPE SYSTEM - Rich Types for Orthological Governance
  # ============================================================================

  # Core architectural role types (enum replacement)
  let FamiliarKind = [|
    `node, `system, `action, `entity, `resource, `queue,
    `component, `ui, `tenant, `technique, `environment
  |] in

  # Orthological layer types
  let OrthologicalLayer = [| `entity, `node, `topology, `domain |] in

  # Pattern category types for architecture
  let PatternCategory = [| `creational, `structural, `behavioral, `concurrency, `architectural |] in

  # Topology types
  let TopologyType = [| `entity_node_binding, `synthesized, `service_mesh, `data_flow |] in

  # Validation result types (union types)
  let ValidationResult = [|
    `valid { },
    `invalid { errors: Array<String>, warnings: Array<String> },
    `error { message: String, context: String }
  |] in

  # Schema transformation result types
  let TransformationResult = [|
    `success { original: _, transformed: _, layer: OrthologicalLayer },
    `failed { original: _, errors: Array<String> }
  |] in

  # ============================================================================
  # CONTRACT TYPES - Reusable Validation Contracts
  # ============================================================================

  # Base contract type for all orthological transformations
  let OrthologicalContract = {
    extract_from_raw | OrthologicalContract -> _ -> _,
    validate_pure | OrthologicalContract -> _ -> ValidationResult,
    get_layer | OrthologicalContract -> OrthologicalLayer
  } in

  # Entity contract type (codegen layer)
  let EntityContract = OrthologicalContract & {
    extract_entity_data | EntityContract -> _ -> _,
    validate_pure_entity | EntityContract -> _ -> ValidationResult,
    validate_entity_structure | EntityContract -> _ -> ValidationResult
  } in

  # Node contract type (infrastructure layer)
  let NodeContract = OrthologicalContract & {
    extract_compute_resources | NodeContract -> _ -> _,
    validate_pure_compute | NodeContract -> _ -> ValidationResult,
    validate_compute_resources | NodeContract -> _ -> ValidationResult
  } in

  # Topology contract type (architecture layer)
  let TopologyContract = OrthologicalContract & {
    extract_topology_wiring | TopologyContract -> _ -> _,
    synthesize_topology | TopologyContract -> Array<_> -> Array<_> -> _,
    validate_pure_topology | TopologyContract -> _ -> ValidationResult,
    validate_topology_bindings | TopologyContract -> _ -> ValidationResult
  } in

  # Domain contract type (business logic layer)
  let DomainContract = OrthologicalContract & {
    extract_business_logic | DomainContract -> _ -> _,
    validate_pure_business | DomainContract -> _ -> ValidationResult,
    validate_business_logic | DomainContract -> _ -> ValidationResult
  } in

  # Error types for structured error handling
  let ValidationError = {
    field | String,
    expected | String,
    actual | String,
    suggestion | String,
    layer | OrthologicalLayer,
    severity | [| `error, `warning, `info |]
  } in

  # Union type for validation outcomes
  ValidationOutcome = [| `valid { result: _ }, `invalid { errors: Array<ValidationError>, warnings: Array<String> } |] in

  # Runtime type checking functions
  let TypeChecker = {
    # is_schema_kind : _ -> Bool
    # Runtime type check for SchemaKind enum values
    is_schema_kind = fun value =>
      std.array.elem value [
        `node, `system, `action, `entity, `resource, `queue,
        `component, `ui, `tenant, `technique, `environment
      ],

    # is_pattern_category : _ -> Bool
    # Runtime type check for PatternCategory enum values
    is_pattern_category = fun value =>
      std.array.elem value [`creational, `structural, `behavioral, `concurrency, `architectural],

    # is_orthological_layer : _ -> Bool
    # Runtime type check for OrthologicalLayer enum values
    is_orthological_layer = fun value =>
      std.array.elem value [`entity, `node, `topology, `domain],

    # safe_enum_conversion : String -> Array<_> -> String -> _
    # Type-safe enum conversion with fallback error
    safe_enum_conversion = fun input_value valid_values type_name =>
      if std.array.elem input_value valid_values then
        input_value
      else
        std.contract.blame_with "type_check" "Invalid ${type_name} value: ${std.to_string input_value}"
  } in

  # ============================================================================
  # TYPE-SAFE VALIDATION FUNCTIONS
  # ============================================================================

  # validate_kind_type : String -> ValidationOutcome
  # Type-safe validation using enum types instead of runtime checks
  let validate_kind_type = fun kind_string =>
    # Convert string to enum value for type checking
    let kind_enum = try {
      match kind_string {
        "node" => `node,
        "system" => `system,
        "action" => `action,
        "entity" => `entity,
        "resource" => `resource,
        "queue" => `queue,
        "component" => `component,
        "ui" => `ui,
        "tenant" => `tenant,
        "technique" => `technique,
        "environment" => `environment,
        _ => std.contract.blame_with "enum_conversion" "Unknown schema kind: ${kind_string}"
      }
    } catch "enum_conversion" => {
      # Fallback to runtime check if enum conversion fails
      if TypeChecker.is_schema_kind (std.string.to_lower kind_string) then
        `node  # Default fallback
      else
        std.contract.blame_with "type_check" "Invalid schema kind: ${kind_string}"
    } in

    # Use TypeChecker for runtime validation
    if TypeChecker.is_schema_kind kind_enum then
      `valid { result = kind_enum }
    else
      `invalid {
        errors = [{
          field = "x-familiar-kind",
          expected = "Valid SchemaKind enum",
          actual = kind_string,
          suggestion = "Use one of: node, system, action, entity, resource, queue, component, ui, tenant, technique, environment",
          layer = `entity,
          severity = `error
        }],
        warnings = []
      } in

  # validate_pattern_category : String -> ValidationOutcome
  # Type-safe pattern category validation using enum types
  let validate_pattern_category = fun category_string =>
    # Convert string to enum value
    let category_enum = try {
      match category_string {
        "creational" => `creational,
        "structural" => `structural,
        "behavioral" => `behavioral,
        "concurrency" => `concurrency,
        "architectural" => `architectural,
        _ => std.contract.blame_with "enum_conversion" "Unknown pattern category: ${category_string}"
      }
    } catch "enum_conversion" => {
      # Fallback to runtime check
      if TypeChecker.is_pattern_category (std.string.to_lower category_string) then
        `creational  # Default fallback
      else
        std.contract.blame_with "type_check" "Invalid pattern category: ${category_string}"
    } in

    # Use TypeChecker for validation
    if TypeChecker.is_pattern_category category_enum then
      `valid { result = category_enum }
    else
      `invalid {
        errors = [{
          field = "x-familiar-pattern-category",
          expected = "Valid PatternCategory enum",
          actual = category_string,
          suggestion = "Use one of: creational, structural, behavioral, concurrency, architectural",
          layer = `topology,
          severity = `error
        }],
        warnings = []
      } in

  # create_validation_error : String -> String -> String -> String -> OrthologicalLayer -> [| `error, `warning, `info |] -> ValidationError
  # Type-safe error construction with severity levels
  let create_validation_error = fun field expected actual suggestion layer severity =>
    {
      field = field,
      expected = expected,
      actual = actual,
      suggestion = suggestion,
      layer = layer,
      severity = severity
    } : ValidationError in

  # Helper functions for common error types
  let create_schema_error = fun field expected actual suggestion =>
    create_validation_error field expected actual suggestion `entity `error in

  let create_infrastructure_error = fun field expected actual suggestion =>
    create_validation_error field expected actual suggestion `node `error in

  let create_topology_error = fun field expected actual suggestion =>
    create_validation_error field expected actual suggestion `topology `error in

  let create_domain_warning = fun field expected actual suggestion =>
    create_validation_error field expected actual suggestion `domain `warning in

  # ============================================================================
  # ADVANCED CONTRACT PATTERNS
  # ============================================================================

  # Contract composition - combine multiple contracts
  # compose_contracts : Array<OrthologicalContract> -> OrthologicalContract
  compose_contracts = fun contracts => {
    extract_from_raw = fun raw_schema =>
      # Chain extraction through all contracts
      std.array.fold_left (fun acc contract => contract.extract_from_raw acc) raw_schema contracts,

    validate_pure = fun schema =>
      # Validate against all contracts, collect all errors
      let results = std.array.map (fun contract => contract.validate_pure schema) contracts in
      let all_errors = std.array.flat_map (fun result => result.errors) results in
      let all_warnings = std.array.flat_map (fun result => result.warnings) results in
      let all_valid = std.array.all (fun result => result.valid) results in
      { valid = all_valid, errors = all_errors, warnings = all_warnings },

    get_layer = `entity  # Default to entity layer for composed contracts
  } in

  # Strategic merging for configuration layers
  # MergeStrategy union type
  MergeStrategy = [|
    `override,  # New values completely replace old
    `deep_merge,  # Recursively merge nested structures
    `append,     # Append arrays, override scalars
    `priority    # Use priority field to resolve conflicts
  |] in

  # strategic_merge : _ -> _ -> MergeStrategy -> _
  # Advanced merging with configurable strategies
  strategic_merge = fun base override strategy =>
    match strategy {
      `override => override,
      `deep_merge => std.record.merge_deep base override,
      `append => merge_append base override,
      `priority => merge_by_priority base override
    } in

  # merge_append : _ -> _ -> _
  # Append arrays, override other values
  merge_append = fun base override =>
    if std.is_record base && std.is_record override then
      std.record.map
        (fun key base_val =>
          if std.record.has_field key override then
            let override_val = override."${key}" in
            if std.is_array base_val && std.is_array override_val then
              base_val @ override_val  # Append arrays
            else
              override_val  # Override other types
          else
            base_val
        )
        base
    else
      override in

  # merge_by_priority : _ -> _ -> _
  # Use priority field to resolve merge conflicts
  merge_by_priority = fun base override =>
    if std.is_record base && std.is_record override then
      std.record.map
        (fun key base_val =>
          if std.record.has_field key override then
            let override_val = override."${key}" in
            let base_priority = if std.record.has_field "priority" base_val then base_val.priority else 0 in
            let override_priority = if std.record.has_field "priority" override_val then override_val.priority else 0 in

            if override_priority > base_priority then
              override_val
            else
              base_val
          else
            base_val
        )
        base
    else
      override in

  # ============================================================================
  # CONTRACT TESTING FRAMEWORK
  # ============================================================================

  # TestCase type for contract testing
  TestCase = {
    name | String,
    input | _,
    expected_output | _,
    expected_errors | Array<String>,
    description | String
  } in

  # ContractTestResult union type
  ContractTestResult = [|
    `passed { test_case: TestCase },
    `failed { test_case: TestCase, actual_output: _, actual_errors: Array<String>, reason: String }
  |] in

  # run_contract_tests : OrthologicalContract -> Array<TestCase> -> Array<ContractTestResult>
  # Execute contract tests and return results
  run_contract_tests = fun contract test_cases =>
    std.array.map
      (fun test_case =>
        try {
          let actual_output = contract.extract_from_raw test_case.input in
          let validation_result = contract.validate_pure actual_output in

          if validation_result.valid &&
             (std.array.is_empty test_case.expected_errors ||
              std.array.length validation_result.errors == std.array.length test_case.expected_errors) then
            `passed { test_case = test_case }
          else
            `failed {
              test_case = test_case,
              actual_output = actual_output,
              actual_errors = validation_result.errors,
              reason = "Validation failed or error count mismatch"
            }
        } catch error => {
          `failed {
            test_case = test_case,
            actual_output = null,
            actual_errors = ["Exception: ${std.to_string error}"],
            reason = "Contract threw exception"
          }
        }
      )
      test_cases in

  # ============================================================================
  # TYPE-SAFE ERROR RECOVERY MECHANISMS
  # ============================================================================

  # Error recovery union type
  ErrorRecovery = [|
    `recovered { result: _, applied_fixes: Array<String> },
    `unrecoverable { errors: Array<ValidationError> }
  |] in

  # try_recover : (_ -> _) -> Array<String> -> (_ -> ErrorRecovery) -> ErrorRecovery
  # Type-safe error recovery with suggestions
  let try_recover = fun operation recovery_strategies recovery_function =>
    try {
      let result = operation in
      `recovered { result = result, applied_fixes = [] }
    } catch error => {
      recovery_function error
    } in

  # safe_schema_parse : String -> ErrorRecovery
  # Graceful JSON parsing with recovery suggestions
  let safe_schema_parse = fun schema_json =>
    try {
      let parsed = std.from_json schema_json in
      `recovered { result = parsed, applied_fixes = [] }
    } catch "json_parse_error" => {
      `unrecoverable {
        errors = [{
          field = "schema_content",
          expected = "Valid JSON",
          actual = "Invalid JSON",
          suggestion = "Check JSON syntax, validate brackets, quotes, and commas",
          layer = `entity,
          severity = `error
        }]
      }
    } in

  # safe_field_access : _ -> String -> _ -> ErrorRecovery
  # Type-safe field access with fallback
  let safe_field_access = fun record field_name fallback =>
    if std.record.has_field field_name record then
      `recovered { result = record."${field_name}", applied_fixes = [] }
    else
      `recovered { result = fallback, applied_fixes = ["Added missing field: ${field_name}"] } in

  # Schema metadata
  schema_info = {
    level = "global",
    description = "Global architectural governance for all Familiar schemas"
  },

  # ============================================================================
  # CORE EXTENSIONS (Required everywhere)
  # ============================================================================

  # The fundamental extensions that every schema must have
  required_extensions = [
    "x-familiar-kind",        # Every schema declares its architectural role
    "x-familiar-description"  # Every schema must be documented
  ],

  # Base extensions allowed everywhere (can be restricted by directories)
  allowed_extensions = [
    # Identity & Documentation
    "x-familiar-kind",           # node, system, action, entity, resource, queue, component, ui, tenant
    "x-familiar-description",    # Human-readable description
    "x-familiar-meta-schema",    # Reference to governing metaschema

    # Basic Architectural Placement
    "x-familiar-service",        # Which service provides this
    "x-familiar-resources",      # Which resources this depends on

    # Basic Configuration
    "x-familiar-default",        # Default values
    "x-familiar-default-value",  # Alternative default syntax

    # Basic Validation & Types
    "x-familiar-enum-values",    # For enum constraints
    "x-familiar-variants",       # For union type variants
  ],

  # ============================================================================
  # VALIDATION RULES (Apply everywhere)
  # Core validation contracts that enforce fundamental schema requirements
  # ============================================================================

  # validate_schema : Schema -> ValidationResult
  # Core schema validation function that applies all global rules with type safety
  validate_schema = fun schema => {
    # Apply all global validation rules
    let results = std.array.map (fun rule => rule "global" schema) validation_rules in

    # Check if all validations passed (rules return the schema if valid)
    let all_valid = std.array.all (fun result =>
      std.record.has_field "x-familiar-kind" result ||
      std.record.has_field "$schema" result
    ) results in

    # Return typed ValidationResult
    if all_valid then
      { valid = true, errors = [], warnings = [] } : ValidationResult
    else
      { valid = false, errors = ["Global validation failed"], warnings = [] } : ValidationResult
  },

  # ============================================================================
  # X-FAMILIAR EXTENSION SCHEMAS - Comprehensive Governance
  # ============================================================================

  # Comprehensive schemas for all permitted x-familiar-* extensions
  # These define the structure, validation, and permitted usage of each extension
  extension_schemas = {

    # ============================================================================
    # CORE METADATA EXTENSIONS
    # ============================================================================

    "x-familiar-kind" = {
      type = "string",
      enum = ["entity", "node", "topology", "domain"],
      description = "Defines the orthological layer classification",
      required = true,
      validation = fun value =>
        std.array.elem value ["entity", "node", "topology", "domain"]
    },

    "x-familiar-layer" = {
      type = "string",
      enum = ["entity", "node", "topology", "domain"],
      description = "Defines the architectural layer (alias for x-familiar-kind)",
      required = false,
      validation = fun value =>
        std.array.elem value ["entity", "node", "topology", "domain"]
    },

    "x-familiar-orthology" = {
      type = "string",
      description = "Describes the orthological purity guarantee",
      examples = ["data_only", "compute_only", "wiring_only", "business_only"],
      validation = fun value => std.is_string value
    },

    "x-familiar-description" = {
      type = "string",
      description = "Human-readable description of the schema purpose",
      required = false,
      validation = fun value => std.is_string value && std.string.length value > 0
    },

    # ============================================================================
    # ENTITY LAYER EXTENSIONS (Pure Data)
    # ============================================================================

    "x-familiar-codegen-targets" = {
      type = "array",
      items = { type = "string", enum = ["rust", "typescript", "python", "go"] },
      description = "Target languages for code generation",
      default = ["rust", "typescript"],
      validation = fun value =>
        std.is_array value &&
        std.array.all (fun item => std.array.elem item ["rust", "typescript", "python", "go"]) value
    },

    "x-familiar-component-type" = {
      type = "string",
      enum = ["ecs_component", "data_structure", "value_object", "entity"],
      description = "Type of component in ECS architecture",
      validation = fun value =>
        std.array.elem value ["ecs_component", "data_structure", "value_object", "entity"]
    },

    "x-familiar-type-category" = {
      type = "string",
      enum = ["primitive", "composite", "enum", "union", "reference"],
      description = "Category classification for type definitions",
      validation = fun value =>
        std.array.elem value ["primitive", "composite", "enum", "union", "reference"]
    },

    # ============================================================================
    # INFRASTRUCTURE LAYER EXTENSIONS (Pure Compute)
    # ============================================================================

    "x-familiar-node-type" = {
      type = "string",
      enum = ["api_service", "worker", "database", "cache", "queue", "tool_execution", "resource_allocation", "action_execution"],
      description = "Type of compute node in the infrastructure",
      validation = fun value =>
        std.array.elem value ["api_service", "worker", "database", "cache", "queue", "tool_execution", "resource_allocation", "action_execution"]
    },

    "x-familiar-resources" = {
      type = "object",
      properties = {
        cpu = { type = "string", pattern = "^[0-9]+(\\.[0-9]+)?$" },
        memory = { type = "string", pattern = "^[0-9]+(Gi|Mi|Ki)$" },
        storage = { type = "string", pattern = "^[0-9]+(Gi|Mi|Ki)$", required = false },
        replicas = { type = "integer", minimum = 1, required = false }
      },
      description = "Compute resource requirements",
      validation = fun value =>
        std.is_record value &&
        std.record.has_field "cpu" value &&
        std.record.has_field "memory" value
    },

    "x-familiar-scaling" = {
      type = "object",
      properties = {
        min_replicas = { type = "integer", minimum = 0, default = 1 },
        max_replicas = { type = "integer", minimum = 1, default = 10 },
        target_cpu_utilization = { type = "integer", minimum = 1, maximum = 100, default = 70 },
        enabled = { type = "boolean", default = true }
      },
      description = "Auto-scaling configuration",
      validation = fun value =>
        std.is_record value &&
        (std.record.get_or "min_replicas" value 1) <= (std.record.get_or "max_replicas" value 10)
    },

    "x-familiar-capabilities" = {
      type = "array",
      items = { type = "string" },
      description = "List of capabilities provided by this compute resource",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-execution-mode" = {
      type = "string",
      enum = ["sync", "async", "batch", "stream"],
      description = "Execution mode for compute operations",
      validation = fun value =>
        std.array.elem value ["sync", "async", "batch", "stream"]
    },

    "x-familiar-allocation-strategy" = {
      type = "string",
      enum = ["first_fit", "best_fit", "worst_fit", "round_robin"],
      description = "Resource allocation strategy",
      validation = fun value =>
        std.array.elem value ["first_fit", "best_fit", "worst_fit", "round_robin"]
    },

    "x-familiar-contract" = {
      type = "object",
      properties = {
        inputs = { type = "object", description = "Input specifications" },
        outputs = { type = "object", description = "Output specifications" },
        side_effects = { type = "array", items = { type = "string" }, required = false }
      },
      description = "Interface contract for tools and services",
      validation = fun value =>
        std.is_record value &&
        std.record.has_field "inputs" value &&
        std.record.has_field "outputs" value
    },

    # ============================================================================
    # ARCHITECTURE LAYER EXTENSIONS (Pure Topology)
    # ============================================================================

    "x-familiar-topology-type" = {
      type = "string",
      enum = ["entity_node_binding", "data_flow", "service_mesh", "runtime_topology"],
      description = "Type of topology relationship",
      validation = fun value =>
        std.array.elem value ["entity_node_binding", "data_flow", "service_mesh", "runtime_topology"]
    },

    "x-familiar-architectural-pattern" = {
      type = "string",
      enum = ["layered", "microservices", "event_driven", "hexagonal", "cqrs", "serverless", "monolithic", "modular", "pipeline"],
      description = "Architectural pattern classification",
      validation = fun value =>
        std.array.elem value ["layered", "microservices", "event_driven", "hexagonal", "cqrs", "serverless", "monolithic", "modular", "pipeline"]
    },

    "x-familiar-architectural-layer" = {
      type = "string",
      enum = ["presentation", "application", "domain", "infrastructure", "data", "integration", "cross_cutting"],
      description = "Architectural layer within a pattern",
      validation = fun value =>
        std.array.elem value ["presentation", "application", "domain", "infrastructure", "data", "integration", "cross_cutting"]
    },

    "x-familiar-architectural-boundary" = {
      type = "object",
      properties = {
        type = { type = "string", enum = ["service", "module", "layer", "context", "aggregate"] },
        name = { type = "string" },
        responsibilities = { type = "array", items = { type = "string" } }
      },
      description = "Architectural boundary definition",
      validation = fun value =>
        std.is_record value &&
        std.record.has_field "type" value &&
        std.record.has_field "name" value
    },

    "x-familiar-relationships" = {
      type = "array",
      items = {
        type = "object",
        properties = {
          from = { type = "string" },
          to = { type = "string" },
          type = { type = "string", enum = ["depends", "uses", "implements", "extends", "composes"] },
          cardinality = { type = "string", pattern = "^[0-9*]+:[0-9*]+$", required = false }
        }
      },
      description = "Topology relationships between components",
      validation = fun value =>
        std.is_array value &&
        std.array.all (fun rel =>
          std.is_record rel &&
          std.record.has_field "from" rel &&
          std.record.has_field "to" rel &&
          std.record.has_field "type" rel
        ) value
    },

    "x-familiar-data-flow" = {
      type = "array",
      items = {
        type = "object",
        properties = {
          source = { type = "string" },
          target = { type = "string" },
          data_type = { type = "string" },
          volume = { type = "string", enum = ["low", "medium", "high"], required = false },
          frequency = { type = "string", enum = ["realtime", "batch", "event"], required = false }
        }
      },
      description = "Data flow topology between nodes",
      validation = fun value =>
        std.is_array value &&
        std.array.all (fun flow =>
          std.is_record flow &&
          std.record.has_field "source" flow &&
          std.record.has_field "target" flow &&
          std.record.has_field "data_type" flow
        ) value
    },

    "x-familiar-composition-type" = {
      type = "string",
      enum = ["system_assembly", "service_composition", "module_aggregation"],
      description = "Type of system composition",
      validation = fun value =>
        std.array.elem value ["system_assembly", "service_composition", "module_aggregation"]
    },

    "x-familiar-assembly-rules" = {
      type = "array",
      items = { type = "string" },
      description = "Rules for component assembly",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    # ============================================================================
    # DOMAIN LAYER EXTENSIONS (Pure Business Logic)
    # ============================================================================

    "x-familiar-domain-context" = {
      type = "string",
      description = "Bounded context name in Domain-Driven Design",
      validation = fun value => std.is_string value && std.string.length value > 0
    },

    "x-familiar-business-rules" = {
      type = "array",
      items = { type = "string" },
      description = "Business rules governing this domain concept",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-domain-events" = {
      type = "array",
      items = { type = "string" },
      description = "Domain events that can be emitted",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-ddd-principles" = {
      type = "array",
      items = {
        type = "string",
        enum = ["bounded_context", "domain_model", "ubiquitous_language", "aggregate", "entity", "value_object", "domain_service", "application_service", "repository", "factory"]
      },
      description = "DDD principles applied to this schema",
      validation = fun value =>
        std.is_array value &&
        std.array.all (fun item =>
          std.array.elem item ["bounded_context", "domain_model", "ubiquitous_language", "aggregate", "entity", "value_object", "domain_service", "application_service", "repository", "factory"]
        ) value
    },

    "x-familiar-aggregates" = {
      type = "array",
      items = { type = "string" },
      description = "Aggregate roots that contain this entity",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-agent-capabilities" = {
      type = "array",
      items = { type = "string" },
      description = "Capabilities available to AI agents",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-ui-behaviors" = {
      type = "array",
      items = { type = "string" },
      description = "User interface behaviors and interactions",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-api-contracts" = {
      type = "array",
      items = {
        type = "object",
        properties = {
          name = { type = "string" },
          method = { type = "string", enum = ["GET", "POST", "PUT", "DELETE", "PATCH"] },
          path = { type = "string" },
          input_schema = { type = "object", required = false },
          output_schema = { type = "object", required = false }
        }
      },
      description = "API contract definitions",
      validation = fun value =>
        std.is_array value &&
        std.array.all (fun contract =>
          std.is_record contract &&
          std.record.has_field "name" contract &&
          std.record.has_field "method" contract &&
          std.record.has_field "path" contract
        ) value
    },

    # ============================================================================
    # INFRASTRUCTURE INTEGRATION EXTENSIONS
    # ============================================================================

    "x-familiar-service" = {
      type = "object",
      properties = {
        "$ref" = { type = "string", description = "Reference to service definition" },
        interface = { type = "string", required = false },
        protocol = { type = "string", enum = ["http", "grpc", "websocket", "tcp"], required = false }
      },
      description = "Service interface definition (deprecated - use topology layer)",
      deprecated = true,
      validation = fun value => std.is_record value
    },

    "x-familiar-deploy-config" = {
      type = "object",
      properties = {
        environment = { type = "string", enum = ["development", "staging", "production"] },
        replicas = { type = "integer", minimum = 1 },
        health_check = { type = "object", required = false }
      },
      description = "Deployment configuration (belongs in infrastructure layer)",
      validation = fun value => std.is_record value
    },

    "x-familiar-monitoring-enabled" = {
      type = "boolean",
      description = "Whether monitoring is enabled",
      default = false,
      validation = fun value => std.is_bool value
    },

    # ============================================================================
    # LEGACY COMPATIBILITY EXTENSIONS
    # ============================================================================

    "x-familiar-meta-schema" = {
      type = "string",
      description = "Legacy meta-schema reference",
      deprecated = true,
      validation = fun value => std.is_string value
    },

    "x-familiar-visual" = {
      type = "object",
      description = "Visual presentation metadata",
      validation = fun value => std.is_record value
    },

    "x-familiar-layout" = {
      type = "object",
      description = "Layout configuration",
      validation = fun value => std.is_record value
    },

    "x-familiar-styling" = {
      type = "object",
      description = "Styling configuration",
      validation = fun value => std.is_record value
    },

    # ============================================================================
    # CODE GENERATION EXTENSIONS
    # ============================================================================

    "x-familiar-rust-derives" = {
      type = "array",
      items = { type = "string" },
      description = "Rust derive macros to apply",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-enum-values" = {
      type = "array",
      description = "Enum value definitions",
      validation = fun value => std.is_array value
    },

    "x-familiar-variants" = {
      type = "array",
      items = { type = "object" },
      description = "Union/enum variant definitions",
      validation = fun value => std.is_array value
    },

    # ============================================================================
    # SCHEMA COMPOSITION EXTENSIONS
    # ============================================================================

    "x-familiar-dependencies" = {
      type = "array",
      items = {
        type = "object",
        properties = {
          name = { type = "string" },
          version = { type = "string" },
          optional = { type = "boolean", default = false }
        }
      },
      description = "Schema dependencies",
      validation = fun value =>
        std.is_array value &&
        std.array.all (fun dep =>
          std.is_record dep && std.record.has_field "name" dep && std.record.has_field "version" dep
        ) value
    },

    "x-familiar-depends" = {
      type = "array",
      items = { type = "string" },
      description = "Simple dependency references",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    # ============================================================================
    # OBSERVABILITY EXTENSIONS
    # ============================================================================

    "x-familiar-reads" = {
      type = "array",
      items = { type = "string" },
      description = "Read access patterns",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-writes" = {
      type = "array",
      items = { type = "string" },
      description = "Write access patterns",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    # ============================================================================
    # PERFORMANCE EXTENSIONS
    # ============================================================================

    "x-familiar-pgo" = {
      type = "object",
      properties = {
        profile_guided_optimization = { type = "boolean", default = false },
        instrumentation = { type = "boolean", default = false },
        optimization_level = { type = "string", enum = ["none", "basic", "advanced"], default = "none" }
      },
      description = "Profile-guided optimization settings",
      validation = fun value => std.is_record value
    },

    # ============================================================================
    # SCHEMA VALIDATION EXTENSIONS
    # ============================================================================

    "x-familiar-pattern-category" = {
      type = "string",
      enum = ["creational", "structural", "behavioral"],
      description = "Design pattern category",
      validation = fun value =>
        std.array.elem value ["creational", "structural", "behavioral"]
    },

    "x-familiar-default" = {
      description = "Default value specification",
      validation = fun value => true  # Any value is valid
    },

    "x-familiar-default-value" = {
      description = "Alternative default value specification",
      validation = fun value => true  # Any value is valid
    },

    # ============================================================================
    # SPECIALIZED EXTENSIONS
    # ============================================================================

    "x-familiar-persistence" = {
      type = "object",
      properties = {
        strategy = { type = "string", enum = ["database", "file", "memory", "cache"] },
        table = { type = "string", required = false },
        key = { type = "string", required = false }
      },
      description = "Data persistence configuration",
      validation = fun value =>
        std.is_record value && std.record.has_field "strategy" value
    },

    "x-familiar-policy" = {
      type = "object",
      properties = {
        type = { type = "string", enum = ["security", "access", "retention", "compliance"] },
        rules = { type = "array", items = { type = "string" } }
      },
      description = "Policy configuration",
      validation = fun value =>
        std.is_record value &&
        std.record.has_field "type" value &&
        std.record.has_field "rules" value
    },

    "x-familiar-queue" = {
      type = "object",
      properties = {
        type = { type = "string", enum = ["fifo", "priority", "delayed"] },
        retention = { type = "string", required = false },
        dead_letter = { type = "boolean", default = false }
      },
      description = "Queue configuration",
      validation = fun value => std.is_record value && std.record.has_field "type" value
    },

    "x-familiar-api" = {
      type = "object",
      properties = {
        endpoints = { type = "array", items = { type = "string" } },
        version = { type = "string" },
        deprecated = { type = "boolean", default = false }
      },
      description = "API configuration",
      validation = fun value => std.is_record value
    },

    "x-familiar-environment" = {
      type = "string",
      enum = ["development", "staging", "production", "testing"],
      description = "Target environment",
      validation = fun value =>
        std.array.elem value ["development", "staging", "production", "testing"]
    },

    "x-familiar-environment-type" = {
      type = "string",
      enum = ["kubernetes", "docker", "serverless", "vm", "bare_metal"],
      description = "Infrastructure environment type",
      validation = fun value =>
        std.array.elem value ["kubernetes", "docker", "serverless", "vm", "bare_metal"]
    },

    "x-familiar-networking" = {
      type = "object",
      properties = {
        ports = { type = "array", items = { type = "integer" } },
        protocols = { type = "array", items = { type = "string" } },
        load_balancer = { type = "boolean", default = false }
      },
      description = "Networking configuration",
      validation = fun value => std.is_record value
    },

    "x-familiar-retention" = {
      type = "object",
      properties = {
        period = { type = "string", pattern = "^[0-9]+[smhd]$" },
        strategy = { type = "string", enum = ["delete", "archive", "compress"] }
      },
      description = "Data retention policy",
      validation = fun value =>
        std.is_record value &&
        std.record.has_field "period" value &&
        std.record.has_field "strategy" value
    },

    "x-familiar-nodes" = {
      type = "array",
      items = { type = "string" },
      description = "List of infrastructure nodes",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-api-endpoints" = {
      type = "array",
      items = { type = "string" },
      description = "API endpoint definitions",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    "x-familiar-connections" = {
      type = "array",
      items = { type = "string" },
      description = "Node connection references",
      validation = fun value =>
        std.is_array value && std.array.all (fun item => std.is_string item) value
    },

    # ============================================================================
    # METADATA EXTENSIONS
    # ============================================================================

    "x-familiar-extracted-from" = {
      type = "string",
      description = "Source schema that this was extracted from",
      validation = fun value => std.is_string value
    },

    "x-familiar-synthesis-timestamp" = {
      type = "string",
      description = "Timestamp when this schema was synthesized",
      validation = fun value => std.is_string value
    },

    "x-familiar-deprecation-warning" = {
      type = "string",
      description = "Deprecation warning message",
      validation = fun value => std.is_string value
    },

    "x-familiar-migration-required" = {
      type = "boolean",
      description = "Indicates schema requires migration",
      validation = fun value => std.is_bool value
    }
  },

  validation_rules = [
    # validate_kind : Schema -> Schema
    # Type-safe validation using enum types instead of runtime string checks
    # Ensures every schema declares its architectural kind/role
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-kind" schema then
        let kind = schema."x-familiar-kind" in
        match validate_kind_type kind with
        | `valid {} => schema
        | `invalid { errors = errs, warnings = warns } =>
          std.contract.blame_with label (std.string.join "; " errs)
        | `error { message = msg } =>
          std.contract.blame_with label "Validation error: ${msg}"
      else
        std.contract.blame_with label "Missing required x-familiar-kind. All schemas must declare their architectural role."
    )),

    # validate_description : Schema -> Schema
    # Ensures schemas are properly documented with meaningful descriptions
    # Critical for maintainability and developer experience
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-description" schema then
        let desc = schema."x-familiar-description" in
        if std.string.length desc > 10 then
          schema
        else
          std.contract.blame_with label "x-familiar-description too short (min 10 chars). Must provide meaningful documentation."
      else
        std.contract.blame_with label "Missing required x-familiar-description. All schemas must be documented."
    )),

    # validate_meta_schema : Schema -> Schema
    # Ensures schemas reference their governing meta-schema contracts
    # This creates the chain of validation from global → layer → specific
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-meta-schema" schema then
        let meta = schema."x-familiar-meta-schema" in
        if std.record.has_field "$ref" meta then
          let ref = meta."$ref" in
          if std.string.contains ref "../" then
            schema
          else
            std.contract.blame_with label "Meta-schema refs must use relative paths (../) to reference governing contracts"
        else
          std.contract.blame_with label "x-familiar-meta-schema must have $ref field pointing to validation contract"
      else
        std.contract.blame_with label "Missing x-familiar-meta-schema. Schemas must reference their governing contracts."
    )),

    # Service references must be valid
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-service" schema then
        let service = schema."x-familiar-service" in
        if std.record.has_field "$ref" service then
          schema
        else
          std.contract.blame_with label "x-familiar-service must have $ref"
      else
        schema  # Service refs are optional globally
    )),

    # Resources must be an array of refs
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-resources" schema then
        let resources = schema."x-familiar-resources" in
        if std.is_array resources then
          let all_have_refs = std.array.all
            (fun r => std.record.has_field "$ref" r)
            resources in
          if all_have_refs then
            schema
          else
            std.contract.blame_with label "All resources must have $ref"
        else
          std.contract.blame_with label "x-familiar-resources must be array"
      else
        schema  # Resources are optional globally
    ))
  ],

  # ============================================================================
  # GLOBAL HYDRATION (Applied to every schema)
  # ============================================================================

  hydration = {
    # Metadata injected everywhere
    "_metadata" = {
      created_at = std.to_string (std.contract.current_time),
      schema_version = "1.0",
      governance_level = "global",
      processed_by = "nickel-governance"
    },

    # Basic observability (can be enhanced by directories)
    "_observability" = {
      logging_enabled = true,
      metrics_enabled = false,  # Directories can override
      tracing_enabled = false   # Directories can override
    },

    # Basic operational defaults
    "_operations" = {
      monitoring_category = "infrastructure",
      alert_on_failure = true,
      retry_policy = "none"
    }
  },

  # ============================================================================
  # GOVERNANCE METADATA
  # ============================================================================

  governance = {
    version = "1.0",
    last_updated = "2024-01-07",
    description = "Global architectural governance for all Familiar schemas",

    # Extensions that require special approval
    restricted_extensions = [
      "x-familiar-pgo",           # Performance optimization
      "x-familiar-rust-derives",  # Code generation
      "x-familiar-visual"         # UI concerns
    ],

    # Extensions that are deprecated
    deprecated_extensions = [
      "x-familiar-depends",       # Use x-familiar-resources
      "x-familiar-reads",         # Computed by graph analysis
      "x-familiar-writes"         # Computed by graph analysis
    ]
  }
}
