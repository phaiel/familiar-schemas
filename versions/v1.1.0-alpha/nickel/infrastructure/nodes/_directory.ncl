# Infrastructure Nodes Directory - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Nodes provide service endpoints and host systems

  required_extensions = (import "../_directory.ncl").required_extensions @ [
    "x-familiar-service"  # Nodes must provide services
  ],

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "node",
      health_check_enabled = true
    }
  }
}
