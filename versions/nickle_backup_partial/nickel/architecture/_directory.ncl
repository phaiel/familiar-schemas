# Architecture Directory Governance - Inherits from global, architectural schemas
(import "../global.ncl") & {
  # Architecture schemas define the system's architectural patterns and structures
  # They must be abstract, reusable, and follow architectural best practices

  # ============================================================================
  # ARCHITECTURE-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-architectural-pattern", # Pattern type (layered, microservices, etc.)
    "x-familiar-architectural-layer",   # Which layer this belongs to
    "x-familiar-cross-cutting",        # Cross-cutting concerns
    "x-familiar-architectural-boundary", # Architectural boundaries
    "x-familiar-architectural-roles",  # Roles in the architecture
    "x-familiar-architectural-flows"   # Data/control flows
  ],

  required_extensions = (import "../global.ncl").required_extensions @ [
    "x-familiar-architectural-pattern" # Architecture schemas must declare their pattern
  ],

  # ============================================================================
  # ARCHITECTURE VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # Architecture schemas must specify their pattern
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-architectural-pattern" schema then
        let pattern = schema."x-familiar-architectural-pattern" in
        if std.array.elem pattern [
          "layered", "microservices", "event-driven", "hexagonal",
          "cqrs", "serverless", "monolithic", "modular", "pipeline"
        ] then
          schema
        else
          std.contract.blame_with label "Unknown architectural pattern: ${pattern}"
      else
        std.contract.blame_with label "Architecture schemas must specify x-familiar-architectural-pattern"
    )),

    # Architecture schemas must define their layer
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-architectural-layer" schema then
        let layer = schema."x-familiar-architectural-layer" in
        if std.array.elem layer [
          "presentation", "application", "domain", "infrastructure",
          "data", "integration", "cross-cutting"
        ] then
          schema
        else
          std.contract.blame_with label "Invalid architectural layer: ${layer}"
      else
        std.contract.blame_with label "Architecture schemas must specify x-familiar-architectural-layer"
    )),

    # Cross-cutting concerns must be properly categorized
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-cross-cutting" schema then
        let concerns = schema."x-familiar-cross-cutting" in
        if std.is_array concerns then
          let valid_concerns = std.array.all (fun concern =>
            std.array.elem concern [
              "logging", "security", "caching", "monitoring", "tracing",
              "error-handling", "transaction-management", "performance"
            ]
          ) concerns in
          if valid_concerns then
            schema
          else
            std.contract.blame_with label "Invalid cross-cutting concerns specified"
        else
          std.contract.blame_with label "x-familiar-cross-cutting must be an array"
      else
        schema
    )),

    # Architectural boundaries must be clearly defined
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-architectural-boundary" schema then
        let boundary = schema."x-familiar-architectural-boundary" in
        if std.record.has_field "type" boundary then
          let boundary_type = boundary.type in
          if std.array.elem boundary_type [
            "service", "module", "layer", "context", "aggregate"
          ] then
            schema
          else
            std.contract.blame_with label "Invalid boundary type: ${boundary_type}"
        else
          std.contract.blame_with label "Architectural boundaries must specify type"
      else
        schema
    )),

    # Architecture schemas should be abstract and reusable
    (std.contract.custom (fun label schema =>
      # Check for concrete implementation details that don't belong in architecture
      let implementation_concerns = [
        "x-familiar-endpoint", "x-familiar-database-url", "x-familiar-file-path"
      ] in
      let found_concerns = std.array.filter
        (fun ext => std.record.has_field ext schema)
        implementation_concerns in
      if std.array.is_empty found_concerns then
        schema
      else
        std.contract.blame_with label "Architecture schemas should be abstract, not contain implementation details: ${std.to_string found_concerns}"
    ))
  ],

  # ============================================================================
  # ARCHITECTURE HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Architecture-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "architecture"
    },

    # Architecture-focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = false,  # Usually not needed for architectural schemas
      architectural_compliance_tracking = true
    },

    # Architecture-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "architecture",
      architectural_validation = true,
      pattern_enforcement = true
    },

    # Architecture-specific runtime configuration
    "_architecture" = {
      pattern_validation = true,
      boundary_enforcement = true,
      layer_isolation = true,
      cross_cutting_injection = true,
      architectural_monitoring = true
    }
  },

  # ============================================================================
  # ARCHITECTURAL PATTERNS
  # ============================================================================

  supported_patterns = {
    layered = {
      layers = ["presentation", "application", "domain", "infrastructure"],
      communication = "strict_layer_boundaries"
    },

    microservices = {
      communication = "api_contracts",
      deployment = "independent",
      scaling = "horizontal"
    },

    event_driven = {
      communication = "events",
      coupling = "loose",
      consistency = "eventual"
    },

    hexagonal = {
      ports = "interfaces",
      adapters = "implementation_details",
      core_isolation = true
    }
  },

  # Validation for pattern compliance
  validate_pattern_compliance = fun schema pattern =>
    let layer = schema."x-familiar-architectural-layer" in
    let pattern_rules = supported_patterns."${pattern}" in

    if std.record.has_field "layers" pattern_rules then
      let allowed_layers = pattern_rules.layers in
      if std.array.elem layer allowed_layers then
        true
      else
        false
    else
      true  # Pattern doesn't restrict layers
}
