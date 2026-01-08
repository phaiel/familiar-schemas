# Infrastructure Systems Directory - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Systems coordinate multiple nodes and services

  required_extensions = (import "../_directory.ncl").required_extensions @ [
    "x-familiar-nodes"  # Systems manage nodes
  ],

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "system",
      orchestration_enabled = true
    }
  }
}
