# Components Directory Governance - Inherits from global, reusable building blocks
(import "../global.ncl") & {
  # Components are reusable building blocks that can be composed together
  # They must define clear contracts for reuse and composition

  # ============================================================================
  # COMPONENT-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-composition",     # How components can be composed
    "x-familiar-reusability",     # Reusability level (internal/shared/public)
    "x-familiar-dependencies",    # Component dependencies
    "x-familiar-versioning",      # Component versioning strategy
    "x-familiar-testing",         # Testing requirements for components
    "x-familiar-documentation",   # Documentation requirements
    "x-familiar-examples"         # Usage examples
  ],

  required_extensions = (import "../global.ncl").required_extensions @ [
    "x-familiar-reusability"      # All components must specify reusability level
  ],

  # ============================================================================
  # COMPONENT VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # Components must specify reusability level
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-reusability" schema then
        let reusability = schema."x-familiar-reusability" in
        if std.array.elem reusability ["internal", "shared", "public"] then
          schema
        else
          std.contract.blame_with label "Component reusability must be: internal, shared, or public"
      else
        std.contract.blame_with label "Components must specify x-familiar-reusability"
    )),

    # Public components must have comprehensive documentation
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-reusability" == "public" then
        if std.record.has_field "x-familiar-documentation" schema then
          let docs = schema."x-familiar-documentation" in
          if std.record.has_field "description" docs &&
             std.record.has_field "usage" docs &&
             std.record.has_field "examples" docs then
            schema
          else
            std.contract.blame_with label "Public components must have description, usage, and examples in documentation"
        else
          std.contract.blame_with label "Public components must have x-familiar-documentation"
      else
        schema
    )),

    # Components with dependencies must specify them correctly
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-dependencies" schema then
        let deps = schema."x-familiar-dependencies" in
        if std.is_array deps then
          let valid_deps = std.array.all (fun dep =>
            std.record.has_field "name" dep &&
            std.record.has_field "version" dep
          ) deps in
          if valid_deps then
            schema
          else
            std.contract.blame_with label "Component dependencies must specify name and version"
        else
          std.contract.blame_with label "x-familiar-dependencies must be an array"
      else
        schema
    )),

    # Components must specify composition rules if they have complex composition
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-composition" schema then
        let composition = schema."x-familiar-composition" in
        if std.record.has_field "allowed_children" composition ||
           std.record.has_field "required_children" composition then
          schema
        else
          std.contract.blame_with label "Component composition must specify allowed_children or required_children"
      else
        schema  # Composition rules are optional for simple components
    )),

    # Shared components must have testing requirements
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-reusability" == "shared" ||
         schema."x-familiar-reusability" == "public" then
        if std.record.has_field "x-familiar-testing" schema then
          let testing = schema."x-familiar-testing" in
          if std.record.has_field "unit_tests_required" testing &&
             std.record.has_field "integration_tests_required" testing then
            schema
          else
            std.contract.blame_with label "Shared/public components must specify unit_tests_required and integration_tests_required"
        else
          std.contract.blame_with label "Shared/public components must have x-familiar-testing"
      else
        schema
    ))
  ],

  # ============================================================================
  # COMPONENT HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Component-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "components"
    },

    # Component-focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = true,
      usage_tracking = true,
      composition_tracking = true,
      error_reporting = true
    },

    # Component-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "components",
      versioning_enabled = true,
      dependency_tracking = true
    },

    # Component-specific runtime configuration
    "_components" = {
      composition_engine = "modular",
      dependency_injection = true,
      lazy_loading = true,
      hot_reload_support = true,
      testing_framework = "integrated",
      documentation_generation = true
    }
  },

  # ============================================================================
  # COMPOSITION RULES
  # ============================================================================

  composition_rules = {
    # Rules for how components can be composed together
    allowed_compositions = [
      "component_component",    # Component can contain other components
      "component_primitive",    # Component can use primitives
      "component_service"       # Component can depend on services
    ],

    forbidden_compositions = [
      "component_infrastructure", # Components shouldn't directly use infrastructure
      "component_database"        # Components shouldn't directly access databases
    ],

    # Validation for composition relationships
    validate_composition = fun component_a component_b =>
      let a_type = component_a."x-familiar-kind" in
      let b_type = component_b."x-familiar-kind" in
      let composition_key = "${a_type}_${b_type}" in
      if std.array.elem composition_key allowed_compositions then
        true
      else if std.array.elem composition_key forbidden_compositions then
        false
      else
        true  # Allow unknown combinations for flexibility
  }
}
