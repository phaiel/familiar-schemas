# Infrastructure Actions Directory - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Actions are triggered by queues and have async execution

  # Add action-specific extensions
  allowed_extensions = (import "../_directory.ncl").allowed_extensions @ [
    "x-familiar-trigger",     # How actions are triggered
    "x-familiar-inputs",      # Action input specifications
    "x-familiar-outputs"      # Action output specifications
  ],

  # Actions require queue binding
  required_extensions = (import "../_directory.ncl").required_extensions @ ["x-familiar-queue"],

  # Actions don't persist data directly
  forbidden_extensions = (import "../_directory.ncl").forbidden_extensions @ [
    "x-familiar-persistence",
    "x-familiar-table"
  ],

  validation_rules = (import "../_directory.ncl").validation_rules @ [
    # Actions must have valid queue references
    (std.contract.custom (fun label schema =>
      if std.record.has_field "x-familiar-queue" schema then
        if std.record.has_field "$ref" schema."x-familiar-queue" then
          schema
        else
          std.contract.blame_with label "Actions must reference valid queues"
      else
        std.contract.blame_with label "Actions must specify x-familiar-queue"
    ))
  ],

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      execution_model = "async",
      retry_policy = "exponential_backoff",
      timeout_policy = "action_specific"
    },

    "_actions" = {
      default_timeout = "5m",
      max_retries = 3,
      backoff_multiplier = 2.0
    }
  }
}
