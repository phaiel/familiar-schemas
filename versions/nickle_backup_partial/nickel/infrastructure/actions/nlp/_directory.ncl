# NLP Actions Subdirectory - Inherits from actions
(import "../_directory.ncl") & {
  # NLP actions handle natural language processing

  hydration = (import "../_directory.ncl").hydration & {
    "_actions" = (import "../_directory.ncl").hydration."_actions" & {
      compute_profile = "cpu_optimized",
      memory_profile = "linguistic_models",
      batch_processing = true
    },

    "_nlp" = {
      linguistic_resources = true,
      parallel_processing = true,
      model_preloading = true
    }
  }
}
