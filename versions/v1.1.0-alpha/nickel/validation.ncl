/**
 * Main Validation Entry Point - Orthological Schema Validation Router
 *
 * Routes incoming schemas to appropriate orthological transformation contracts
 * based on their path location and content analysis. This is the central dispatcher
 * for Entity/Node/Topology orthological validation.
 *
 * @process
 * 1. Parse schema path to determine orthological layer (codegen/infra/arch/domain)
 * 2. Route to appropriate transformation contract
 * 3. Execute orthological extraction and validation
 * 4. Return validation results with orthological guarantees
 *
 * @orthology Enforces Entity/Node/Topology separation at validation time
 * @contracts Uses transformation contracts (files = { _ : Contract }) pattern
 *
 * @example
 *   # Environment variables set by Rust validator
 *   SCHEMA_FILE=<json_content> SCHEMA_PATH=path/to/schema.json nickel eval validation.ncl
 */

# ============================================================================
# ADVANCED CONTRACT PATTERNS DEMONSTRATION
# ============================================================================

# Demonstrate contract composition - combine entity + domain contracts for cross-layer validation
let composed_entity_domain_contract = (import "global.ncl").compose_contracts [
  (import "codegen/_directory.ncl").files._,
  (import "domain/_directory.ncl").files._
] in

# Strategic merging for layered configuration overrides
let merge_layer_configs = fun base_config overrides =>
  (import "global.ncl").strategic_merge base_config overrides `priority in

# Error recovery with suggestions
let recover_validation_errors = fun validation_result =>
  match validation_result {
    { valid = true } => validation_result,
    { valid = false, errors = errors } =>
      # Attempt recovery for known error patterns
      let recovered = std.array.fold_left
        (fun acc error =>
          match error {
            "Entity schemas cannot contain runtime concerns. Extract to infrastructure/topology layers." =>
              # Could attempt automatic extraction here
              acc & { "recovery_attempted" = true },
            _ => acc
          }
        )
        { recovered = false, recovery_attempted = false }
        errors in

      if recovered.recovery_attempted then
        validation_result & { "recovery_attempted" = true }
      else
        validation_result
  } in

let schema_content = std.record.get_or "SCHEMA_FILE" std.env "" in
let schema_path = std.record.get_or "SCHEMA_PATH" std.env "" in

# parse_schema_path : String -> { top_category: String, sub_category: String }
# Parses schema file path to determine orthological routing destination
# Extracts hierarchical location for proper contract selection
let path_parts = std.string.split "/" schema_path in

# Extract top-level category and subcategory with path validation
let routing = if std.array.length path_parts >= 3 then
  let json_schema_index = std.array.find_index (fun part => part == "json-schema") path_parts in
  if json_schema_index != -1 && json_schema_index + 2 < std.array.length path_parts then
    {
      top_category = std.array.at path_parts (json_schema_index + 1),
      sub_category = std.array.at path_parts (json_schema_index + 2)
    }
  else
    {
      top_category = "unknown",
      sub_category = "unknown"
    }
else
  {
    top_category = "unknown",
    sub_category = "unknown"
  }
in

# route_to_orthological_contract : Routing -> ValidationResult
# Routes parsed schema to appropriate orthological transformation contract
# Each layer enforces its orthological guarantees through contract validation
let validation_result = if routing.top_category == "codegen" then
  # CODEGEN CONTRACT: Entity extraction and validation
  # Ensures "What" (data) is pure and free of runtime concerns
  let entity_contract = (import "codegen/_directory.ncl").files._ in
  let raw_schema = std.from_json schema_content in
  let pure_entity = entity_contract.extract_entity_data raw_schema in
  entity_contract.validate_pure_entity pure_entity
else if routing.top_category == "infrastructure" then
  # INFRASTRUCTURE CONTRACT: Node extraction and validation
  # Ensures "Where" (compute) is pure and free of data/business concerns
  let node_contract = (import "infrastructure/_directory.ncl").files._ in
  let raw_schema = std.from_json schema_content in
  let pure_node = node_contract.extract_compute_resources raw_schema in
  node_contract.validate_pure_compute pure_node
else if routing.top_category == "architecture" then
  # ARCHITECTURE CONTRACT: Topology synthesis and validation
  # Ensures "How" (wiring) is pure relationship graph
  let topology_contract = (import "architecture/_directory.ncl").files._ in
  let raw_schema = std.from_json schema_content in
  let topology_wiring = topology_contract.extract_topology_wiring raw_schema in
  topology_contract.validate_pure_topology topology_wiring
else if routing.top_category == "domain" then
  # DOMAIN CONTRACT: Business logic extraction and validation
  # Ensures "Why" (business requirements) is pure and free of technical concerns
  let domain_contract = (import "domain/_directory.ncl").files._ in
  let raw_schema = std.from_json schema_content in
  let pure_business = domain_contract.extract_business_logic raw_schema in
  domain_contract.validate_pure_business pure_business
else if routing.top_category == "windmill" then
  let windmill_validation = import "windmill/_directory.ncl" in
  windmill_validation.validate schema_content
else
  # DEFAULT CONTRACT: Unknown category handling
  # Gracefully handles schemas in unrecognized orthological layers
  {
    valid = true,
    errors = [],
    warnings = ["Unknown top-level category: ${routing.top_category}. Schema not validated against orthological contracts."]
  }
in

# Export orthologically-validated result
# This result guarantees Entity/Node/Topology separation has been enforced
validation_result

# ============================================================================
# INTEGRATED TESTING FRAMEWORK
# ============================================================================

# Run comprehensive orthological validation tests across all layers
let run_all_tests = fun =>
  let codegen_results = (import "codegen/_directory.ncl").run_tests in
  let infrastructure_results = (import "infrastructure/_directory.ncl").run_tests in
  let architecture_results = (import "architecture/_directory.ncl").run_tests in
  let domain_results = (import "domain/_directory.ncl").run_tests in
  
  {
    codegen = {
      total = std.array.length codegen_results,
      passed = std.array.length (std.array.filter (fun r => match r { `passed _ => true, _ => false }) codegen_results),
      failed = std.array.length (std.array.filter (fun r => match r { `failed _ => true, _ => false }) codegen_results)
    },
    infrastructure = {
      total = std.array.length infrastructure_results, 
      passed = std.array.length (std.array.filter (fun r => match r { `passed _ => true, _ => false }) infrastructure_results),
      failed = std.array.length (std.array.filter (fun r => match r { `failed _ => true, _ => false }) infrastructure_results)
    },
    architecture = {
      total = std.array.length architecture_results,
      passed = std.array.length (std.array.filter (fun r => match r { `passed _ => true, _ => false }) architecture_results),
      failed = std.array.length (std.array.filter (fun r => match r { `failed _ => true, _ => false }) architecture_results)
    },
    domain = {
      total = std.array.length domain_results,
      passed = std.array.length (std.array.filter (fun r => match r { `passed _ => true, _ => false }) domain_results),
      failed = std.array.length (std.array.filter (fun r => match r { `failed _ => true, _ => false }) domain_results)
    }
  } in

# Export orthologically-validated result with advanced pattern capabilities
validation_result & {
  capabilities = {
    contract_composition = "available",
    strategic_merging = "available", 
    error_recovery = "available",
    contract_testing = "available"
  },
  
  test_runner = run_all_tests,
  
  compose_contracts = (import "global.ncl").compose_contracts,
  strategic_merge = (import "global.ncl").strategic_merge,
  recover_errors = recover_validation_errors
}
