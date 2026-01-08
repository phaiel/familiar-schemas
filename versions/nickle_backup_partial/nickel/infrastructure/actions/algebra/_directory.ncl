# Algebra Actions Subdirectory - Inherits from actions
(import "../_directory.ncl") & {
  # Algebra actions handle mathematical tensor operations

  hydration = (import "../_directory.ncl").hydration & {
    "_actions" = (import "../_directory.ncl").hydration."_actions" & {
      compute_profile = "high_precision",
      memory_profile = "tensor_operations",
      gpu_required = true,
      timeout_policy = "compute_intensive"
    },

    "_algebra" = {
      tensor_operations = true,
      precision_requirements = "double",
      gpu_acceleration = "cuda",
      memory_pool = "tensor"
    }
  }
}
