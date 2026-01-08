# LLM Actions Subdirectory - Inherits from actions
(import "../_directory.ncl") & {
  # LLM actions handle AI language model operations

  hydration = (import "../_directory.ncl").hydration & {
    "_actions" = (import "../_directory.ncl").hydration."_actions" & {
      compute_profile = "gpu_accelerated",
      memory_profile = "model_loading",
      timeout_policy = "llm_inference"
    },

    "_llm" = {
      gpu_required = true,
      model_caching = true,
      token_limits = {
        input_max = 32768,
        output_max = 4096
      }
    }
  }
}
