# API Directory Governance - Inherits from global, interface contracts
(import "../global.ncl") & {
  # API schemas define interface contracts between systems
  # They must specify clear request/response formats and error handling

  # ============================================================================
  # API-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../global.ncl").allowed_extensions @ [
    "x-familiar-http-method",     # HTTP method (GET, POST, PUT, DELETE)
    "x-familiar-http-path",       # API endpoint path
    "x-familiar-request-format",  # Request payload format
    "x-familiar-response-format", # Response payload format
    "x-familiar-error-responses", # Error response specifications
    "x-familiar-rate-limiting",   # API rate limiting rules
    "x-familiar-authentication",  # Authentication requirements
    "x-familiar-versioning",      # API versioning strategy
    "x-familiar-deprecation"      # API deprecation policies
  ],

  required_extensions = (import "../global.ncl").required_extensions @ [
    "x-familiar-http-method",     # APIs must specify HTTP method
    "x-familiar-http-path"        # APIs must specify endpoint path
  ],

  # ============================================================================
  # API VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../global.ncl").validation_rules @ [
    # APIs must specify valid HTTP methods
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-http-method" schema then
        let method = schema."x-familiar-http-method" in
        if std.array.elem method ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"] then
          schema
        else
          std.contract.blame_with label "Invalid HTTP method: ${method}. Must be GET, POST, PUT, DELETE, PATCH, HEAD, or OPTIONS"
      else
        std.contract.blame_with label "APIs must specify x-familiar-http-method"
    )),

    # API paths must be valid
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-http-path" schema then
        let path = schema."x-familiar-http-path" in
        if std.string.starts_with path "/" then
          # Basic path validation - should start with / and not have invalid characters
          if std.string.contains path " " then
            std.contract.blame_with label "API paths cannot contain spaces"
          else
            schema
        else
          std.contract.blame_with label "API paths must start with /"
      else
        std.contract.blame_with label "APIs must specify x-familiar-http-path"
    )),

    # APIs must specify response formats
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-response-format" schema then
        let format = schema."x-familiar-response-format" in
        if std.array.elem format ["json", "xml", "text", "binary", "stream"] then
          schema
        else
          std.contract.blame_with label "Invalid response format: ${format}. Must be json, xml, text, binary, or stream"
      else
        std.contract.blame_with label "APIs must specify x-familiar-response-format"
    )),

    # APIs must specify error responses
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-error-responses" schema then
        let errors = schema."x-familiar-error-responses" in
        if std.is_array errors then
          let has_400 = std.array.any (fun err =>
            std.record.has_field "status_code" err &&
            err.status_code >= 400 && err.status_code < 500
          ) errors in
          let has_500 = std.array.any (fun err =>
            std.record.has_field "status_code" err &&
            err.status_code >= 500 && err.status_code < 600
          ) errors in
          if has_400 && has_500 then
            schema
          else
            std.contract.blame_with label "APIs must specify both 4xx and 5xx error responses"
        else
          std.contract.blame_with label "x-familiar-error-responses must be an array"
      else
        std.contract.blame_with label "APIs must specify x-familiar-error-responses"
    )),

    # Authentication requirements must be specified for sensitive operations
    (std.contract.custom (fun label schema =>
      let method = schema."x-familiar-http-method" in
      let requires_auth = std.array.elem method ["POST", "PUT", "DELETE", "PATCH"] in
      if requires_auth then
        if std.record.has_field "x-familiar-authentication" schema then
          let auth = schema."x-familiar-authentication" in
          if std.record.has_field "required" auth && auth.required == true then
            schema
          else
            std.contract.blame_with label "State-changing APIs must require authentication"
        else
          std.contract.blame_with label "State-changing APIs must specify x-familiar-authentication"
      else
        schema  # GET, HEAD, OPTIONS typically don't require auth
    )),

    # APIs must specify versioning strategy
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-versioning" schema then
        let versioning = schema."x-familiar-versioning" in
        if std.array.elem versioning ["url", "header", "query_param", "none"] then
          schema
        else
          std.contract.blame_with label "Invalid versioning strategy: ${versioning}. Must be url, header, query_param, or none"
      else
        std.contract.blame_with label "APIs must specify x-familiar-versioning"
    ))
  ],

  # ============================================================================
  # API HYDRATION
  # ============================================================================

  hydration = (import "../global.ncl").hydration & {
    # API-specific metadata
    "_metadata" = (import "../global.ncl").hydration."_metadata" & {
      governance_level = "api"
    },

    # API-focused observability
    "_observability" = (import "../global.ncl").hydration."_observability" & {
      metrics_enabled = true,
      request_tracking = true,
      response_time_monitoring = true,
      error_rate_tracking = true,
      api_usage_analytics = true
    },

    # API-specific operations
    "_operations" = (import "../global.ncl").hydration."_operations" & {
      monitoring_category = "api",
      rate_limiting_enabled = true,
      authentication_enabled = true,
      cors_handling = true,
      request_validation = true
    },

    # API-specific runtime configuration
    "_api" = {
      default_timeout = "30s",
      max_request_size = "10MB",
      rate_limiting = {
        requests_per_minute = 60,
        burst_limit = 10
      },
      cors = {
        allow_credentials = true,
        max_age = "86400"
      },
      middleware = [
        "authentication",
        "rate_limiting",
        "request_logging",
        "error_handling"
      ]
    }
  },

  # ============================================================================
  # API CONTRACT STANDARDS
  # ============================================================================

  api_standards = {
    # REST API design standards
    rest_compliance = {
      resource_naming = "plural_nouns",
      http_status_codes = "standard",
      content_negotiation = "accept_header",
      pagination = "cursor_based"
    },

    # Error response standards
    error_format = {
      status_code = "number",
      error_code = "string",
      message = "string",
      details = "object"  # Optional detailed error information
    },

    # Versioning standards
    versioning_standards = {
      header_versioning = "X-API-Version",
      url_versioning = "/v{version}/",
      default_version = "v1",
      sunset_policy = "12_months"
    }
  }
}
