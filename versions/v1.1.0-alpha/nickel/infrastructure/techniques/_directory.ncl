# Infrastructure Techniques Directory - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Techniques define reusable processing patterns

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "technique",
      performance_optimization = true
    }
  }
}
