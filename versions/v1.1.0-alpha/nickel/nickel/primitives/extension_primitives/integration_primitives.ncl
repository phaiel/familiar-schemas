# Extension Integration Primitives - Common integration patterns

{
  # Graph integration primitives
  GraphIntegrationPrimitives = {
    # Base graph metadata
    base_graph_metadata = {
      node_type = "schema",
      edge_types = [],
      properties = {}
    },

    # Node metadata generators
    node_metadata_generators = {
      # For entity schemas
      entity_node = fun schema extension => {
        id = std.record.get_or "$id" schema "unknown",
        type = "entity",
        category = extension.category,
        extension_name = extension.name,
        properties = {
          required = std.record.get_or "required" schema [],
          properties = std.record.get_or "properties" schema {}
        }
      },

      # For infrastructure schemas
      infrastructure_node = fun schema extension => {
        id = std.record.get_or "$id" schema "unknown",
        type = "infrastructure",
        category = extension.category,
        extension_name = extension.name,
        properties = {
          scaling = std.record.get_or "scaling" schema {},
          resources = std.record.get_or "resources" schema {}
        }
      },

      # For topology schemas
      topology_node = fun schema extension => {
        id = std.record.get_or "$id" schema "unknown",
        type = "topology",
        category = extension.category,
        extension_name = extension.name,
        properties = {
          wiring = std.record.get_or "wiring" schema {},
          relationships = std.record.get_or "relationships" schema []
        }
      }
    },

    # Edge metadata generators
    edge_metadata_generators = {
      # Relationship edges
      relationship_edge = fun source target edge_type => {
        source_id = source.id,
        target_id = target.id,
        edge_type = edge_type,
        properties = {
          direction = "directed",
          cardinality = "many_to_one"
        }
      },

      # Dependency edges
      dependency_edge = fun source target => {
        source_id = source.id,
        target_id = target.id,
        edge_type = "depends_on",
        properties = {
          direction = "directed",
          strength = "strong",
          transitivity = true
        }
      }
    }
  },

  # Codegen integration primitives
  CodegenIntegrationPrimitives = {
    # Base codegen metadata
    base_codegen_metadata = {
      target_languages = ["rust"],
      features = [],
      derives = []
    },

    # Language-specific generators
    language_generators = {
      rust = {
        # Derive generators
        derives_by_category = {
          entity = ["Debug", "Clone", "Serialize", "Deserialize"],
          infrastructure = ["Debug", "Clone"],
          topology = ["Debug", "Clone", "Serialize"]
        },

        # Type mapping
        type_mapping = {
          string = "String",
          number = "f64",
          integer = "i64",
          boolean = "bool",
          array = "Vec",
          object = "HashMap"
        },

        # Feature flags
        feature_flags_by_category = {
          entity = ["serde"],
          infrastructure = ["tokio"],
          topology = ["petgraph", "serde"]
        }
      },

      typescript = {
        derives_by_category = {
          entity = [],
          infrastructure = [],
          topology = []
        },
        type_mapping = {
          string = "string",
          number = "number",
          integer = "number",
          boolean = "boolean",
          array = "Array",
          object = "Record"
        }
      }
    },

    # Codegen metadata generators
    metadata_generators = {
      entity_codegen = fun schema extension => {
        language = "rust",
        derives = language_generators.rust.derives_by_category.entity,
        features = language_generators.rust.feature_flags_by_category.entity,
        schema_type = "entity"
      },

      infrastructure_codegen = fun schema extension => {
        language = "rust",
        derives = language_generators.rust.derives_by_category.infrastructure,
        features = language_generators.rust.feature_flags_by_category.infrastructure,
        schema_type = "infrastructure"
      }
    }
  },

  # Validation integration primitives
  ValidationIntegrationPrimitives = {
    # Context-aware validation
    context_validators = {
      topology_context = fun schema =>
        std.record.has_field "relationships" schema &&
        std.record.has_field "wiring" schema,

      entity_context = fun schema =>
        std.record.has_field "properties" schema,

      infrastructure_context = fun schema =>
        std.record.has_field "resources" schema
    },

    # Schema-level validators
    schema_validators = {
      has_required_fields = fun schema required_fields =>
        std.array.all (fun field => std.record.has_field field schema) required_fields,

      field_type_check = fun schema field_checks =>
        std.record.all (fun field_name expected_type =>
          let actual_value = std.record.get_or field_name schema null in
          actual_value == null || std.type.of actual_value == expected_type
        ) field_checks
    }
  }
}