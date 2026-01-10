# Reference Library - Validation contracts for $ref and $schema usage patterns
# Following Nickel "composable data" principles with orthological reference validation
{
  # ============================================================================
  # REFERENCE PATTERN DEFINITIONS (Orthological Constants)
  # ============================================================================

  ReferencePatterns = {
    # Allowed $schema patterns by level
    allowed_schema_patterns = {
      directory = ["http://json-schema.org/draft-07/schema#", "http://json-schema.org/draft-04/schema#"],
      leaf = ["http://json-schema.org/draft-07/schema#", "http://json-schema.org/draft-04/schema#"],
      meta = ["http://json-schema.org/draft-07/schema#", "http://json-schema.org/draft-04/schema#"]
    },

    # $ref usage patterns by level
    ref_usage_patterns = {
      # Directory level: NO $refs allowed (use inheritance)
      directory = {
        forbidden_refs = true,
        allowed_ref_types = [],
        inheritance_required = true
      },
      # Leaf level: $refs allowed for composition
      leaf = {
        forbidden_refs = false,
        allowed_ref_types = ["primitive", "entity", "component", "node", "system"],
        inheritance_allowed = false
      },
      # Meta level: $refs allowed for schema composition
      meta = {
        forbidden_refs = false,
        allowed_ref_types = ["primitive", "definition"],
        inheritance_allowed = true
      }
    }
  },

  # ============================================================================
  # REFERENCE VALIDATION
  # ============================================================================

  ReferenceValidation = {
    # Validate $schema usage by level
    validate_schema_usage = fun schema level =>
      if std.record.has_field "$schema" schema then
        let schema_value = schema."$schema" in
        let allowed_patterns = std.record.get level ReferencePatterns.allowed_schema_patterns in
        std.array.elem schema_value allowed_patterns
      else
        # $schema is optional but recommended
        true,

    # Validate $ref usage by level
    validate_ref_usage = fun schema level =>
      let level_config = std.record.get level ReferencePatterns.ref_usage_patterns in
      if level_config.forbidden_refs then
        # Directory level: No $refs allowed
        let has_refs =
          std.record.fields schema
          |> std.array.any (fun field =>
            if std.record.has_field field schema then
              let field_value = std.record.get field schema in
              std.is_record field_value && std.record.has_field "$ref" field_value
            else
              false
          )
        in
        if has_refs then
          {
            valid = false,
            errors = ["Directory-level schemas must use inheritance, not $refs"],
            warnings = []
          }
        else
          { valid = true, errors = [], warnings = [] }
      else
        # Leaf/meta level: Validate $ref patterns
        let refs = ReferenceValidation.extract_all_refs schema in
        let invalid_refs =
          std.array.filter
            (fun ref =>
              !ReferenceValidation.validate_ref_pattern ref level_config.allowed_ref_types
            )
            refs
        in
        if std.array.is_empty invalid_refs then
          { valid = true, errors = [], warnings = [] }
        else
          {
            valid = false,
            errors = std.array.map (fun ref => "Invalid $ref pattern: %{ref}") invalid_refs,
            warnings = []
          },

    # Extract all $ref values from a schema
    extract_all_refs = fun schema =>
      let rec extract_refs_recursively = fun value =>
        if std.is_record value then
          let direct_ref =
            if std.record.has_field "$ref" value then
              [value."$ref"]
            else
              []
          in
          let nested_refs =
            std.record.values value
            |> std.array.map extract_refs_recursively
            |> std.array.fold_left (fun acc arr => acc @ arr) []
          in
          direct_ref @ nested_refs
        else if std.is_array value then
          std.array.map extract_refs_recursively value
          |> std.array.fold_left (fun acc arr => acc @ arr) []
        else
          []
      in
      extract_refs_recursively schema,

    # Validate a single $ref pattern
    validate_ref_pattern = fun ref_value allowed_types =>
      # Check if ref follows allowed patterns
      std.array.any
        (fun ref_type =>
          std.string.contains ref_value ref_type
        )
        allowed_types
  },

  # ============================================================================
  # INHERITANCE VALIDATION
  # ============================================================================

  InheritanceValidation = {
    # Validate inheritance usage by level
    validate_inheritance_usage = fun schema level =>
      let level_config = std.record.get level ReferencePatterns.ref_usage_patterns in
      if level_config.inheritance_required then
        # Directory level should show signs of inheritance
        let has_inheritance_indicators =
          std.record.fields schema
          |> std.array.any (fun field =>
            # Look for merged record patterns or inheritance markers
            std.string.contains field "Libraries"
            || std.string.contains field "Composer"
            || std.record.has_field "bundle_metadata" schema
          )
        in
        if has_inheritance_indicators then
          { valid = true, errors = [], warnings = [] }
        else
          {
            valid = false,
            errors = ["Directory-level schemas should use inheritance patterns"],
            warnings = []
          }
      else
        { valid = true, errors = [], warnings = [] }
  },

  # ============================================================================
  # COMPREHENSIVE REFERENCE CONTRACT
  # ============================================================================

  ReferenceContract = {
    # Validate complete reference usage for a schema
    validate = fun schema level =>
      let schema_validation = ReferenceValidation.validate_schema_usage schema level in
      let ref_validation = ReferenceValidation.validate_ref_usage schema level in
      let inheritance_validation = InheritanceValidation.validate_inheritance_usage schema level in

      if schema_validation && ref_validation.valid && inheritance_validation.valid then
        {
          valid = true,
          errors = [],
          warnings = ref_validation.warnings @ inheritance_validation.warnings
        }
      else
        {
          valid = false,
          errors =
            (if ref_validation.valid then [] else ref_validation.errors)
            @ (if inheritance_validation.valid then [] else inheritance_validation.errors)
            @ (if schema_validation then [] else ["Invalid $schema for %{level} level"]),
          warnings = ref_validation.warnings @ inheritance_validation.warnings
        }
  },

  # ============================================================================
  # LIBRARY METADATA
  # ============================================================================

  library_metadata = {
    name = "reference_library",
    version = "1.0.0",
    description = "Orthological validation for $ref and $schema usage patterns",
    provides = ["ReferencePatterns", "ReferenceValidation", "InheritanceValidation", "ReferenceContract"],
    dependencies = [],
    nickel_principle = "composable_data"
  }
}
