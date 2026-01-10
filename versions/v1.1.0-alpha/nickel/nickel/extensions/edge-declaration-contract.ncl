/**
 * x-familiar-edge-declaration-contract Extension
 *
 * CONTRACT ENFORCER for explicit edge type declarations in JSON schemas.
 * Prohibits bare $ref arrays and requires typed edge structures for Frame Graph compatibility.
 *
 * This contract ensures schemas declare relationships using:
 *   "x-familiar-edges": [
 *     { "edge_type": "depends_on", "target": {"$ref": "Component.json"} }
 *   ]
 *
 * Instead of the old implicit:
 *   "x-familiar-depends": [{"$ref": "Component.json"}]
 *
 * @category architecture
 * @graph-value critical - enforces Frame Graph structure
 * @validation schema-level - validates entire schema structure
 * @contract strict - no exceptions for Frame Graph compatibility
 */

{
  extension = {
    name = "x-familiar-edge-declaration-contract",
    description = "Contract enforcer for explicit edge type declarations",
    category = "architecture",
    required = true,  # This MAKES it a contract - schemas MUST comply
    version = "1.0.0",
    context_required = ["topology", "entity", "component", "system", "node"]
  },

  # ============================================================================
  # STRUCTURAL CONTRACT - The Core Enforcement
  # ============================================================================

  contract = std.contract.custom (fun label schema =>
    let relationship_fields = [
      "x-familiar-depends", "x-familiar-reads", "x-familiar-writes",
      "x-familiar-components", "x-familiar-systems", "x-familiar-resources",
      "x-familiar-service", "x-familiar-queue"
    ] in

    # Check if this schema needs edge validation
    let needs_validation = std.record.has_field "x-familiar-kind" schema &&
                          std.array.elem schema."x-familiar-kind" [
                            "entity", "component", "system", "node", "topology"
                          ] in

    if needs_validation then
      # Validate all relationship fields
      let violations = std.array.flat_map (fun field_name =>
        if std.record.has_field field_name schema then
          let field_value = schema."%{field_name}" in
          validate_relationship_field field_name field_value
        else
          []
      ) relationship_fields in

      if std.array.is_empty violations then
        schema  # Valid - return the schema unchanged
      else
        std.contract.blame_with label
          "Edge Declaration Contract Violations: Use x-familiar-edges array with explicit edge types instead of bare $ref arrays"
    else
      schema  # Skip validation for schemas that don't declare relationships
  ),

  # ============================================================================
  # VALIDATION FUNCTIONS
  # ============================================================================

  functions = {
    # Validate a single relationship field
    validate_relationship_field = fun field_name field_value =>
      if is_array_of_refs field_value then
        ["Bare $ref arrays prohibited. Use {edge_type, target} objects instead."]
      else if is_array_of_edge_declarations field_value then
        []  # Valid new format
      else if std.is_array field_value then
        ["Array elements must be {edge_type, target} objects."]
      else
        ["Must be array of {edge_type, target} objects."],

    # Check if value is old-style bare $ref array
    is_array_of_refs = fun value =>
      std.is_array value &&
      std.array.length value > 0 &&
      std.array.any (fun item =>
        std.is_record item && std.record.has_field "$ref" item &&
        !std.record.has_field "edge_type" item
      ) value,

    # Check if value is new-style edge declarations
    is_array_of_edge_declarations = fun value =>
      std.is_array value &&
      std.array.all (fun item =>
        std.is_record item &&
        std.record.has_field "edge_type" item &&
        std.record.has_field "target" item &&
        std.is_record item.target &&
        std.record.has_field "$ref" item.target
      ) value,

    # ============================================================================
    # EXTRACTION FUNCTIONS - For Frame Graph Building
    # ============================================================================

    # Extract all edges from a schema for Frame Graph construction
    extract_edges = fun schema =>
      let relationship_fields = [
        "x-familiar-depends", "x-familiar-reads", "x-familiar-writes",
        "x-familiar-components", "x-familiar-systems", "x-familiar-resources",
        "x-familiar-service", "x-familiar-queue"
      ] in

      std.array.flat_map (fun field_name =>
        if std.record.has_field field_name schema then
          let declarations = schema."%{field_name}" in
          if is_array_of_edge_declarations declarations then
            std.array.map (fun decl => {
              source_schema = schema."$id" | schema,
              edge_type = decl.edge_type,
              target_ref = decl.target."$ref",
              field_type = field_name,
              metadata = decl.metadata | {}
            }) declarations
          else
            []  # Skip invalid declarations (will be caught by contract)
        else
          []
      ) relationship_fields,

    # ============================================================================
    # MIGRATION FUNCTIONS - Transform old schemas
    # ============================================================================

    # Inject default edge types for common patterns (migration helper)
    inject_default_edge_types = fun schema =>
      let inject_field = fun field_name default_edge_type schema =>
        if std.record.has_field field_name schema then
          let old_value = schema."%{field_name}" in
          if is_array_of_refs old_value then
            # Transform old format to new format
            let new_value = std.array.map (fun ref_obj => {
              edge_type = default_edge_type,
              target = ref_obj,
              metadata = { migrated_from = field_name }
            }) old_value in
            schema & { "%{field_name}" = new_value }
          else
            schema
        else
          schema
      in

      # Apply migrations with defaults based on field semantics
      schema
      |> inject_field "x-familiar-depends" "depends_on"
      |> inject_field "x-familiar-reads" "references"
      |> inject_field "x-familiar-writes" "transforms"
      |> inject_field "x-familiar-components" "contains"
      |> inject_field "x-familiar-systems" "orchestrates"
      |> inject_field "x-familiar-resources" "depends_on"
      |> inject_field "x-familiar-service" "deploys_to"
      |> inject_field "x-familiar-queue" "communicates_with",

    # ============================================================================
    # VALIDATION HELPERS
    # ============================================================================

    # Get allowed edge types for a given context
    get_allowed_edge_types = fun schema_kind =>
      let global_edges = import "../edge-type.ncl" in
      let edge_names = std.record.fields global_edges.edge_types in

      # Context-specific restrictions
      if schema_kind == "entity" then
        # Entities can only reference and depend
        std.array.filter (fun edge => std.array.elem edge ["depends_on", "references"]) edge_names
      else if schema_kind == "component" then
        # Components have broader relationship options
        edge_names
      else if schema_kind == "system" then
        # Systems orchestrate and communicate
        std.array.filter (fun edge => std.array.elem edge ["orchestrates", "communicates_with", "depends_on"]) edge_names
      else if schema_kind == "node" then
        # Nodes contain systems and provide resources
        std.array.filter (fun edge => std.array.elem edge ["contains", "provides", "depends_on"]) edge_names
      else
        edge_names,  # Default to all

    # Validate edge type is allowed for schema kind
    validate_edge_type_allowed = fun edge_type schema_kind =>
      let allowed = functions.get_allowed_edge_types schema_kind in
      std.array.elem edge_type allowed
  },

  # ============================================================================
  # MERGE STRATEGIES - Strict enforcement
  # ============================================================================

  merge_strategies = {
    override = fun base override =>
      # Only allow override if both are valid edge declarations
      if contract.validate base && contract.validate override then
        override
      else
        std.contract.blame "Cannot merge invalid edge declarations",

    compose = fun base override =>
      std.contract.blame "Edge declarations cannot be composed - use explicit override",

    validate = fun base override =>
      if contract.validate override then override
      else std.contract.blame_with "merge" "Invalid edge declaration in merge"
  },

  # ============================================================================
  # METADATA & EXAMPLES
  # ============================================================================

  defaults = {
    validation = true,
    migration = false  # Don't auto-migrate by default
  },

  metadata = {
    required = true,
    examples = [
      # Valid new format
      {
        "x-familiar-edges": [
          {
            "edge_type": "depends_on",
            "target": {"$ref": "../components/DatabasePool.component.json"},
            "metadata": {"strength": "strong"}
          }
        ]
      }
    ],
    since = "v1.1.0",
    breaking_change = true,  # This enforces a breaking change to Frame Graph format
    graph_impact = "enforced_topology"
  },

  # ============================================================================
  # TESTING FRAMEWORK
  # ============================================================================

  test_cases = [
    # Valid edge declarations
    {
      name = "valid_edge_declarations",
      input = {
        "x-familiar-kind" = "component",
        "x-familiar-depends" = [
          {
            "edge_type" = "depends_on",
            "target" = {"$ref" = "../components/DatabasePool.component.json"}
          }
        ]
      },
      expected_valid = true,
      description = "Valid edge declarations should pass contract validation"
    },

    # Invalid: bare $ref array
    {
      name = "bare_ref_array_violation",
      input = {
        "x-familiar-kind" = "component",
        "x-familiar-depends" = [
          {"$ref" = "../components/DatabasePool.component.json"}
        ]
      },
      expected_valid = false,
      description = "Bare $ref arrays should violate the contract"
    },

    # Invalid: missing target field
    {
      name = "missing_target_field",
      input = {
        "x-familiar-kind" = "component",
        "x-familiar-depends" = [
          {
            "edge_type" = "depends_on"
            # Missing target field
          }
        ]
      },
      expected_valid = false,
      description = "Missing target field should violate contract"
    }
  ]
}