# Tools Directory Governance - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Tools are infrastructure components that provide specific capabilities
  # They must define clear contracts for their interfaces and behaviors

  # ============================================================================
  # TOOL-SPECIFIC EXTENSIONS
  # ============================================================================

  allowed_extensions = (import "../_directory.ncl").allowed_extensions @ [
    "x-familiar-contract",       # Tool interface contract (inputs/outputs)
    "x-familiar-capabilities",   # What the tool can do
    "x-familiar-requirements",   # System requirements for the tool
    "x-familiar-tool-category",  # Classification of tool type
    "x-familiar-execution-mode"  # How the tool executes (sync/async/batch)
  ],

  required_extensions = (import "../_directory.ncl").required_extensions @ [
    "x-familiar-contract"        # All tools must define their interface
  ],

  # ============================================================================
  # TOOL VALIDATION RULES
  # ============================================================================

  validation_rules = (import "../_directory.ncl").validation_rules @ [
    # Tools must have complete contract specifications
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-contract" schema then
        let contract = schema."x-familiar-contract" in
        if std.record.has_field "inputs" contract &&
           std.record.has_field "outputs" contract then
          # Validate input/output specifications
          let inputs = contract.inputs in
          let outputs = contract.outputs in
          if std.is_record inputs && std.is_record outputs then
            schema
          else
            std.contract.blame_with label "Tool contract inputs and outputs must be objects"
        else
          std.contract.blame_with label "Tool contract must specify inputs and outputs"
      else
        std.contract.blame_with label "Tools must have x-familiar-contract"
    )),

    # Tools must specify their category
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-tool-category" schema then
        let category = schema."x-familiar-tool-category" in
        if std.array.elem category [
          "data_processing", "computation", "communication", "storage",
          "analysis", "generation", "validation", "transformation"
        ] then
          schema
        else
          std.contract.blame_with label "Unknown tool category: ${category}"
      else
        std.contract.blame_with label "Tools must specify x-familiar-tool-category"
    )),

    # Tools must specify execution mode
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-execution-mode" schema then
        let mode = schema."x-familiar-execution-mode" in
        if std.array.elem mode ["sync", "async", "batch", "streaming"] then
          schema
        else
          std.contract.blame_with label "Invalid execution mode: ${mode}. Must be: sync, async, batch, or streaming"
      else
        std.contract.blame_with label "Tools must specify x-familiar-execution-mode"
    )),

    # Async tools must have queue bindings
    (std.contract.custom (fun label schema =>
      if schema."x-familiar-execution-mode" == "async" then
        if std.record.has_field "x-familiar-queue" schema then
          schema
        else
          std.contract.blame_with label "Async tools must specify x-familiar-queue"
      else
        schema
    ))
  ],

  # ============================================================================
  # TOOL HYDRATION
  # ============================================================================

  hydration = (import "../_directory.ncl").hydration & {
    # Tool-specific metadata
    "_metadata" = (import "../_directory.ncl").hydration."_metadata" & {
      governance_level = "infrastructure.tools"
    },

    # Enhanced observability for tools
    "_observability" = (import "../_directory.ncl").hydration."_observability" & {
      metrics_enabled = true,
      performance_metrics = true,
      usage_tracking = true,
      error_reporting = true
    },

    # Tool-specific operations
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "tools",
      circuit_breaker_enabled = true,
      rate_limiting_enabled = true,
      tool_execution_tracking = true
    },

    # Tool-specific runtime configuration
    "_tools" = {
      execution_timeout = "30s",
      retry_policy = "exponential_backoff",
      max_concurrent_executions = 10,
      resource_limits = {
        cpu = "500m",
        memory = "1Gi"
      }
    }
  }
}
