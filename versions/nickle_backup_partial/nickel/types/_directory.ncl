# Types Directory Governance - Inherits from global, type system focus
(import "../global.ncl") & {
  # Type schemas define reusable data types and validation rules
  # They must be composable and follow type system best practices

  # ============================================================================
  # TYPE-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-type-category",   # Type classification (primitive/composite/enum)
    "x-familiar-validation-rules", # Additional validation beyond JSON Schema
    "x-familiar-type-conversion",  # Type conversion rules
    "x-familiar-serialization",   # Serialization preferences
    "x-familiar-inheritance",     # Type inheritance relationships
    "x-familiar-polymorphism"     # Polymorphic type definitions
  ],

  required_extensions = (import "../global.ncl").required_extensions @ [
    "x-familiar-type-category"    # All types must be categorized
  ],

  # ============================================================================
  # TYPE VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # Types must specify their category
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-type-category" schema then
        let category = schema."x-familiar-type-category" in
        if std.array.elem category ["primitive", "composite", "enum", "union", "reference"] then
          schema
        else
          std.contract.blame_with label "Invalid type category: ${category}. Must be primitive, composite, enum, union, or reference"
      else
        std.contract.blame_with label "Types must specify x-familiar-type-category"
    )),

    # Enum types must define their values
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-type-category" == "enum" then
        if std.record.has_field "enum" schema then
          let enum_values = schema.enum in
          if std.is_array enum_values && std.array.length enum_values > 0 then
            schema
          else
            std.contract.blame_with label "Enum types must define non-empty enum array"
        else
          std.contract.blame_with label "Enum types must have enum property"
      else
        schema
    )),

    # Composite types must have properties
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-type-category" == "composite" then
        if std.record.has_field "properties" schema then
          let props = schema.properties in
          if std.is_record props && std.record.length props > 0 then
            schema
          else
            std.contract.blame_with label "Composite types must define properties object with at least one property"
        else
          std.contract.blame_with label "Composite types must have properties"
      else
        schema
    )),

    # Union types must have oneOf or anyOf
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-type-category" == "union" then
        if std.record.has_field "oneOf" schema || std.record.has_field "anyOf" schema then
          schema
        else
          std.contract.blame_with label "Union types must have oneOf or anyOf"
      else
        schema
    )),

    # Reference types must specify target
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-type-category" == "reference" then
        if std.record.has_field "$ref" schema then
          schema
        else
          std.contract.blame_with label "Reference types must have $ref"
      else
        schema
    )),

    # Types should avoid serialization concerns (belongs in CompilerConfig)
    (std.contract.custom (fun label schema =>
      let serialization_concerns = [
        "x-familiar-casing", "x-familiar-flatten", "x-familiar-skip-none",
        "x-familiar-enum-repr", "x-familiar-discriminator"
      ] in
      let found_concerns = std.array.filter
        (fun ext => std.record.has_field ext schema)
        serialization_concerns in
      if std.array.is_empty found_concerns then
        schema
      else
        std.contract.blame_with label "Types should not specify serialization concerns (handled by CompilerConfig): ${std.to_string found_concerns}"
    ))
  ],

  # ============================================================================
  # TYPE HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Type-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "types"
    },

    # Type system observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = false,  # Usually not needed for types
      type_validation_tracking = true
    },

    # Type-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "types",
      type_checking_enabled = true,
      schema_validation = true
    },

    # Type-specific runtime configuration
    "_types" = {
      strict_validation = true,
      type_inference = true,
      polymorphic_dispatch = true,
      type_erasure = false,
      runtime_type_checking = true
    }
  },

  # ============================================================================
  # TYPE SYSTEM STANDARDS
  # ============================================================================

  type_standards = {
    naming_conventions = {
      primitives = "lowerCamelCase",
      composites = "UpperCamelCase",
      enums = "UpperCamelCase"
    },

    validation_standards = {
      required_descriptions = true,
      example_values = "recommended",
      default_values = "when_appropriate"
    },

    composition_rules = {
      allow_circular_refs = false,
      max_depth = 10,
      prefer_composition_over_inheritance = true
    }
  }
}
