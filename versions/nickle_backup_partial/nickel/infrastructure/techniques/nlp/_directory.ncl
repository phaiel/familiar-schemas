# NLP Techniques Subdirectory - Inherits from techniques
(import "../_directory.ncl") & {
  # NLP techniques define reusable language processing patterns

  hydration = (import "../_directory.ncl").hydration & {
    "_operations" = (import "../_directory.ncl").hydration."_operations" & {
      processing_model = "batch_nlp",
      linguistic_optimization = true
    },

    "_nlp_techniques" = {
      parallel_processing = true,
      model_sharing = true,
      resource_pooling = true
    }
  }
}
