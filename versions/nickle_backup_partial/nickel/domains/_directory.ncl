# Domains Directory Governance - Inherits from global, domain-specific logic
(import "../global.ncl") & {
  # Domain schemas define business logic and domain-specific rules
  # They must be isolated from infrastructure and presentation concerns

  # ============================================================================
  # DOMAIN-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-domain-boundary",    # Domain boundaries and contexts
    "x-familiar-business-rules",     # Business rule specifications
    "x-familiar-domain-events",      # Domain events
    "x-familiar-aggregates",         # Aggregate root definitions
    "x-familiar-value-objects",      # Value object specifications
    "x-familiar-domain-services",    # Domain service contracts
    "x-familiar-invariant-rules"     # Business invariant rules
  ],

  # Domain schemas require business context
  required_extensions = (import "../global.ncl").required_extensions @ [
    "x-familiar-domain-boundary"     # Domains must define their boundaries
  ],

  # Infrastructure concerns forbidden in domains
  forbidden_extensions = [
    "x-familiar-service",        # Domains shouldn't directly expose services
    "x-familiar-endpoint",       # Domains shouldn't define endpoints
    "x-familiar-persistence",    # Persistence is infrastructure concern
    "x-familiar-queue",          # Messaging is infrastructure concern
    "x-familiar-policy",         # Policies are infrastructure concern
    "x-familiar-pgo",            # Performance is infrastructure concern
    "x-familiar-visual",         # UI is presentation concern
    "x-familiar-field-alias",    # UI is presentation concern
    "x-familiar-field-order"     # UI is presentation concern
  ],

  # ============================================================================
  # DOMAIN VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # Domains cannot have infrastructure extensions
    (std.contract.custom (fun label schema =>
      let forbidden_found = std.array.filter
        (fun ext => std.record.has_field ext schema)
        forbidden_extensions in
      if std.array.is_empty forbidden_found then
        schema
      else
        std.contract.blame_with label "Domain schemas cannot have infrastructure extensions: ${std.to_string forbidden_found}. Domains should focus on business logic."
    )),

    # Domain boundaries must be specified
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-domain-boundary" schema then
        let boundary = schema."x-familiar-domain-boundary" in
        if std.record.has_field "context" boundary then
          schema
        else
          std.contract.blame_with label "Domain boundaries must specify context"
      else
        std.contract.blame_with label "Domains must specify x-familiar-domain-boundary"
    )),

    # Business rules must be well-defined
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-business-rules" schema then
        let rules = schema."x-familiar-business-rules" in
        if std.is_array rules then
          let valid_rules = std.array.all (fun rule =>
            std.record.has_field "name" rule &&
            std.record.has_field "condition" rule &&
            std.record.has_field "action" rule
          ) rules in
          if valid_rules then
            schema
          else
            std.contract.blame_with label "Business rules must specify name, condition, and action"
        else
          std.contract.blame_with label "x-familiar-business-rules must be an array"
      else
        schema  # Business rules are optional
    )),

    # Domain events must follow event sourcing patterns
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-domain-events" schema then
        let events = schema."x-familiar-domain-events" in
        if std.is_array events then
          let valid_events = std.array.all (fun event =>
            std.record.has_field "event_type" event &&
            std.record.has_field "payload" event &&
            std.record.has_field "metadata" event
          ) events in
          if valid_events then
            schema
          else
            std.contract.blame_with label "Domain events must specify event_type, payload, and metadata"
        else
          std.contract.blame_with label "x-familiar-domain-events must be an array"
      else
        schema
    )),

    # Aggregates must define their boundaries
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-aggregates" schema then
        let aggregates = schema."x-familiar-aggregates" in
        if std.is_array aggregates then
          let valid_aggregates = std.array.all (fun aggregate =>
            std.record.has_field "name" aggregate &&
            std.record.has_field "entities" aggregate &&
            std.record.has_field "invariants" aggregate
          ) aggregates in
          if valid_aggregates then
            schema
          else
            std.contract.blame_with label "Aggregates must specify name, entities, and invariants"
        else
          std.contract.blame_with label "x-familiar-aggregates must be an array"
      else
        schema
    )),

    # Domain services must define their contracts
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-domain-services" schema then
        let services = schema."x-familiar-domain-services" in
        if std.is_array services then
          let valid_services = std.array.all (fun service =>
            std.record.has_field "name" service &&
            std.record.has_field "input" service &&
            std.record.has_field "output" service
          ) services in
          if valid_services then
            schema
          else
            std.contract.blame_with label "Domain services must specify name, input, and output"
        else
          std.contract.blame_with label "x-familiar-domain-services must be an array"
      else
        schema
    ))
  ],

  # ============================================================================
  # DOMAIN HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # Domain-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "domains"
    },

    # Domain-focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = true,
      business_metric_tracking = true,
      domain_event_tracking = true
    },

    # Domain-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "domains",
      business_rule_validation = true,
      domain_event_publishing = true,
      aggregate_consistency = true
    },

    # Domain-specific runtime configuration
    "_domains" = {
      bounded_context_isolation = true,
      domain_event_sourcing = true,
      aggregate_pattern = true,
      value_object_immutability = true,
      business_rule_engine = true
    }
  },

  # ============================================================================
  # DOMAIN-DRIVEN DESIGN PRINCIPLES
  # ============================================================================

  ddd_principles = {
    # Core principles that domain schemas must follow
    principles = [
      "business_logic_isolation",
      "ubiquitous_language",
      "bounded_contexts",
      "aggregate_boundaries",
      "domain_events",
      "value_object_immutability"
    ],

    # Validation for DDD principle compliance
    validate_ddd_compliance = fun schema =>
      let has_boundary = std.record.has_field "x-familiar-domain-boundary" schema in
      let no_infrastructure = std.array.all (fun ext =>
        not (std.record.has_field ext schema)
      ) forbidden_extensions in
      let has_business_focus = std.record.has_field "x-familiar-business-rules" schema ||
                              std.record.has_field "x-familiar-domain-events" schema ||
                              std.record.has_field "x-familiar-aggregates" schema in

      has_boundary && no_infrastructure && has_business_focus
  },

  # Subdomain governance templates
  subdomain_templates = {
    core = {
      # Core domain: critical business functionality
      extensions = ["x-familiar-business-rules", "x-familiar-domain-events", "x-familiar-aggregates"],
      hydration_overrides = {
        business_criticality = "high",
        testing_requirements = "comprehensive"
      }
    },

    supporting = {
      # Supporting domain: necessary but not core
      extensions = ["x-familiar-domain-services", "x-familiar-value-objects"],
      hydration_overrides = {
        business_criticality = "medium",
        testing_requirements = "standard"
      }
    },

    generic = {
      # Generic domain: commodity functionality
      extensions = ["x-familiar-value-objects"],
      hydration_overrides = {
        business_criticality = "low",
        testing_requirements = "basic"
      }
    }
  }
}
