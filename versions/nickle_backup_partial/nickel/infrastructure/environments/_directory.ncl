# Infrastructure Environments Directory - Inherits from infrastructure
(import "../_directory.ncl") & {
  # Environments define deployment configurations

  required_extensions = (import "../_directory.ncl").required_extensions @ [
    "x-familiar-environment-type"  # Must specify dev/staging/prod
  ],

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      monitoring_category = "environment",
      multi_region_support = true
    }
  }
}
