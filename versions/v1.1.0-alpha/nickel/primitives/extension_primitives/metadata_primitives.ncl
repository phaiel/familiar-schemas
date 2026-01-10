# Extension Metadata Primitives - Common extension metadata patterns

{
  # Base extension metadata structure
  BaseExtensionMetadata = {
    name = "unknown",
    description = "No description provided",
    category = "unknown",
    required = false,
    version = "1.0.0"
  },

  # Category-specific metadata templates
  CategoryTemplates = {
    architecture = {
      category = "architecture",
      graph_value = "medium",
      validation = "context-dependent",
      context_required = []
    },

    entity = {
      category = "entity",
      graph_value = "high",
      validation = "schema-level",
      context_required = []
    },

    infrastructure = {
      category = "infrastructure",
      graph_value = "high",
      validation = "runtime",
      context_required = []
    },

    codegen = {
      category = "codegen",
      graph_value = "critical",
      validation = "build-time",
      context_required = []
    }
  },

  # Validation level primitives
  ValidationLevels = {
    none = "none",
    basic = "basic",
    context_dependent = "context-dependent",
    schema_level = "schema-level",
    strict = "strict"
  },

  # Graph value primitives
  GraphValues = {
    none = "none",
    low = "low",
    medium = "medium",
    high = "high",
    critical = "critical"
  }
}