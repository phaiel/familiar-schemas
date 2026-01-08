# Primitives Directory Governance - Inherits from global, basic types
(import "../global.ncl") & {
  # Primitive schemas define the most basic reusable types
  # They should be minimal, stable, and highly reusable

  # ============================================================================
  # PRIMITIVE-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-primitive-type",   # The primitive type category
    "x-familiar-constraints",      # Basic validation constraints
    "x-familiar-examples",         # Usage examples
    "x-familiar-compatibility"     # Backward compatibility notes
  ],

  # Primitives don't require additional extensions beyond global
  required_extensions = (import "../global.ncl").required_extensions,

  # ============================================================================
  # PRIMITIVE VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # Primitives must be simple and focused
    (std.contract.custom (fun label schema =>
      let property_count = std.record.length schema in
      if property_count > 10 then
        std.contract.blame_with label "Primitives should be simple (max 10 properties). Consider splitting into multiple primitives."
      else
        schema
    )),

    # Primitives should not have complex nesting
    (std.contract.custom (fun label schema =>
      if std.record.has_field "properties" schema then
        let props = schema.properties in
        let nested_objects = std.record.filter (fun k v =>
          std.is_record v && std.record.has_field "properties" v
        ) props in
        if std.record.is_empty nested_objects then
          schema
        else
          std.contract.blame_with label "Primitives should not have deeply nested objects. Use references to other primitives instead."
      else
        schema
    )),

    # Primitives must have clear, simple validation
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-constraints" schema then
        let constraints = schema."x-familiar-constraints" in
        let constraint_count = std.record.length constraints in
        if constraint_count <= 3 then
          schema
        else
          std.contract.blame_with label "Primitives should have simple constraints (max 3). Complex validation belongs in composite types."
      else
        schema  # Constraints are optional for very basic primitives
    )),

    # Primitives must provide usage examples
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-examples" schema then
        let examples = schema."x-familiar-examples" in
        if std.is_array examples && std.array.length examples >= 1 then
          schema
        else
          std.contract.blame_with label "Primitives must provide at least one usage example"
      else
        std.contract.blame_with label "Primitives must have x-familiar-examples"
    )),

    # Primitives should be stable and well-documented
    (std.contract.custom (fun label schema =>
      if std.record.has_field "description" schema then
        let desc = schema.description in
        if std.string.length desc > 20 then
          schema
        else
          std.contract.blame_with label "Primitive descriptions must be detailed (min 20 characters)"
      else
        std.contract.blame_with label "Primitives must have detailed descriptions"
    ))
  ],

  # ============================================================================
  # PRIMITIVE HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Primitive-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "primitives"
    },

    # Minimal observability for primitives
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = false,  # Usually not needed for primitives
      validation_tracking = true
    },

    # Primitive-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "primitives",
      caching_enabled = true,  # Primitives can be heavily cached
      precompilation = true    # Primitives can be precompiled
    },

    # Primitive-specific runtime configuration
    "_primitives" = {
      builtin_optimization = true,
      compile_time_validation = true,
      runtime_overhead = "minimal",
      memory_layout = "compact"
    }
  },

  # ============================================================================
  # PRIMITIVE DESIGN PRINCIPLES
  # ============================================================================

  design_principles = {
    simplicity = "Primitives should do one thing well",
    reusability = "Primitives should be highly reusable across contexts",
    stability = "Primitives should rarely change once established",
    performance = "Primitives should have minimal runtime overhead",

    # Validation for design principle compliance
    validate_principles = fun primitive =>
      let props = std.record.length primitive in
      let has_examples = std.record.has_field "x-familiar-examples" primitive in
      let has_good_desc = std.record.has_field "description" primitive &&
                         std.string.length primitive.description > 20 in
      let is_simple = props <= 10 in

      if has_examples && has_good_desc && is_simple then
        true
      else
        false
  }
}
