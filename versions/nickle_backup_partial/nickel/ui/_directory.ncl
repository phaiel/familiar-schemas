# UI Directory Governance - Inherits from global, presentation-only
(import "../global.ncl") & {
  # UI schemas define presentation and user interface components
  # They are strictly forbidden from infrastructure concerns

  # ============================================================================
  # UI-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-visual",         # Visual properties and styling
    "x-familiar-layout",         # Layout specifications
    "x-familiar-interaction",    # User interaction patterns
    "x-familiar-accessibility",  # Accessibility requirements
    "x-familiar-responsive",     # Responsive design specifications
    "x-familiar-theme",          # Theming support
    "x-familiar-validation-ui"   # UI-specific validation rules
  ],

  # UI schemas cannot have infrastructure concerns
  forbidden_extensions = [
    "x-familiar-service",        # UI doesn't provide services
    "x-familiar-queue",          # UI doesn't trigger queues
    "x-familiar-policy",         # UI doesn't have runtime policies
    "x-familiar-endpoint",       # UI doesn't expose endpoints
    "x-familiar-persistence",    # UI doesn't persist data directly
    "x-familiar-pgo",            # UI doesn't need performance optimization
    "x-familiar-health-check",   # UI doesn't need health checks
    "x-familiar-connection-pool", # UI doesn't manage connections
    "x-familiar-replication-factor", # UI doesn't handle replication
    "x-familiar-partitions",     # UI doesn't manage partitions
    "x-familiar-retention",      # UI doesn't handle data retention
    "x-familiar-topics",         # UI doesn't use message topics
    "x-familiar-timeout",        # UI doesn't have timeouts
    "x-familiar-retries",        # UI doesn't retry operations
    "x-familiar-rate-limit"      # UI doesn't rate limit
  ],

  # ============================================================================
  # UI VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # UI schemas cannot have any infrastructure extensions
    (std.contract.custom (fun label schema =>
      let forbidden_found = std.array.filter
        (fun ext => std.record.has_field ext schema)
        forbidden_extensions in
      if std.array.is_empty forbidden_found then
        schema
      else
        std.contract.blame_with label "UI schemas cannot have infrastructure extensions: ${std.to_string forbidden_found}. UI is presentation-only."
    )),

    # UI schemas must specify visual properties
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-visual" schema then
        schema
      else
        std.contract.blame_with label "UI schemas should specify x-familiar-visual properties"
    )),

    # UI components must have accessibility considerations
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-accessibility" schema then
        let accessibility = schema."x-familiar-accessibility" in
        if std.record.has_field "aria_label_required" accessibility then
          schema
        else
          std.contract.blame_with label "UI accessibility must specify aria_label_required"
      else
        std.contract.blame_with label "UI schemas must specify x-familiar-accessibility"
    )),

    # Interactive UI elements must specify interaction patterns
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-interaction" schema then
        let interaction = schema."x-familiar-interaction" in
        if std.record.has_field "event_handlers" interaction then
          schema
        else
          std.contract.blame_with label "UI interactions must specify event_handlers"
      else
        # Not all UI elements are interactive, so this is optional
        schema
    ))
  ],

  # ============================================================================
  # UI HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # UI-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "ui"
    },

    # Client-side focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = false,    # Usually handled by frontend monitoring
      tracing_enabled = false,    # Usually handled by frontend tracing
      user_experience_tracking = true,
      error_boundary_reporting = true
    },

    # UI-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "presentation",
      client_side_rendering = true,
      hydration_mismatch_detection = true
    },

    # UI-specific runtime configuration
    "_ui" = {
      rendering_engine = "react",
      styling_approach = "css_modules",
      state_management = "context",
      responsive_breakpoints = {
        mobile = "768px",
        tablet = "1024px",
        desktop = "1440px"
      },
      theme_support = true,
      i18n_enabled = true
    }
  }
}
