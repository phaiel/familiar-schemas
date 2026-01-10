/**
 * Codegen Edge Semantics
 *
 * Inherits global edge types and specializes them for code generation/data concerns.
 * Defines how edges behave in the context of entities, types, and code generation.
 *
 * @category codegen
 * @graph-value high - specializes edges for data relationships
 * @validation context-aware - codegen-specific edge constraints
 */

let global_edges = import "../extensions/edge-type.ncl" in

{
  # ============================================================================
  # INHERITED & SPECIALIZED EDGE TYPES
  # ============================================================================

  # Inherit all global edge types with codegen-specific specializations
  edge_types = global_edges.edge_types & {
    # Add codegen-specific edge types
    inherits_from = {
      directionality = "directed",
      cardinality = "many_to_one",
      transitivity = true,
      cyclicity = "acyclic",
      data_flow = "type_inheritance",
      strength = "strong",
      context = "type_hierarchy",
      compilation_implications = {
        generate_inheritance_code = true,
        generate_polymorphic_dispatch = true,
        generate_type_checks = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true,
        inheritance_cycles_forbidden = true
      }
    },

    composes = {
      directionality = "directed",
      cardinality = "one_to_many",
      transitivity = false,
      cyclicity = "acyclic",
      data_flow = "composition",
      strength = "medium",
      context = "object_composition",
      compilation_implications = {
        generate_composition_code = true,
        generate_lifecycle_management = true,
        generate_null_checks = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = false  # Composition is structural, not dependency-based
      }
    },

    # Specialize existing edges for codegen context
    depends_on = global_edges.edge_types.depends_on & {
      context = "codegen_dependencies",
      compilation_implications = global_edges.edge_types.depends_on.compilation_implications & {
        generate_import_statements = true,
        generate_initialization_order = true,
        generate_dependency_injection_setup = true
      },
      traversal_rules = global_edges.edge_types.depends_on.traversal_rules & {
        codegen_cycles_forbidden = true,
        import_order_critical = true
      }
    },

    references = global_edges.edge_types.references & {
      context = "type_references",
      compilation_implications = {
        generate_type_imports = true,
        generate_forward_declarations = true,
        generate_namespace_imports = false
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = true,  # Type references can be circular (mutual references)
        topological_sort_required = false
      }
    },

    implements = global_edges.edge_types.implements & {
      context = "interface_implementation",
      compilation_implications = {
        generate_interface_implementations = true,
        generate_method_stubs = true,
        generate_type_assertions = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true
      }
    }
  },

  # ============================================================================
  # CODEGEN-SPECIFIC VALIDATION RULES
  # ============================================================================

  validation_rules = {
    # Edge combinations allowed in codegen schemas
    allowed_edge_combinations = [
      # Entity to Entity relationships
      {
        source_kind = "entity",
        target_kind = "entity",
        allowed_edges = ["depends_on", "references", "inherits_from", "composes"]
      },

      # Entity to Component composition
      {
        source_kind = "entity",
        target_kind = "component",
        allowed_edges = ["depends_on", "references"]
      },

      # Component to Component relationships
      {
        source_kind = "component",
        target_kind = "component",
        allowed_edges = ["depends_on", "references", "implements"]
      },

      # Primitive type references
      {
        source_kind = "entity",
        target_kind = "primitive",
        allowed_edges = ["references"]
      }
    ],

    # Prohibited patterns in codegen
    prohibited_patterns = [
      # Entities shouldn't inherit from components (use composition)
      {
        source_kind = "entity",
        target_kind = "component",
        prohibited_edges = ["inherits_from"]
      },

      # Components shouldn't inherit from entities (wrong direction)
      {
        source_kind = "component",
        target_kind = "entity",
        prohibited_edges = ["inherits_from"]
      }
    ],

    # Codegen-specific constraints
    codegen_constraints = {
      # Maximum inheritance depth
      max_inheritance_depth = 3,

      # Allow circular references between types
      allow_circular_type_references = true,

      # Maximum composition depth
      max_composition_depth = 5
    }
  },

  # ============================================================================
  # CODEGEN-SPECIFIC FUNCTIONS
  # ============================================================================

  functions = {
    # Validate edge compatibility in codegen context
    validate_codegen_edge = fun source_kind target_kind edge_type =>
      let allowed_combinations = validation_rules.allowed_edge_combinations in
      let matching_rules = std.array.filter (fun rule =>
        rule.source_kind == source_kind && rule.target_kind == target_kind
      ) allowed_combinations in

      if std.array.is_empty matching_rules then
        false  # No rules allow this source->target combination
      else
        # Check if edge_type is allowed for this source->target
        std.array.any (fun rule =>
          std.array.elem edge_type rule.allowed_edges
        ) matching_rules,

    # Get edge metadata with codegen specialization
    get_codegen_edge_metadata = fun edge_type =>
      if std.record.has_field edge_type edge_types then
        edge_types."%{edge_type}"
      else
        global_edges.functions.get_edge_characteristics edge_type,

    # Analyze type dependencies and imports
    analyze_type_dependencies = fun edges =>
      let dependency_edges = std.array.filter (fun edge =>
        std.array.elem edge.edge_type ["depends_on", "references"]
      ) edges in

      let inheritance_edges = std.array.filter (fun edge =>
        edge.edge_type == "inherits_from"
      ) edges in

      let composition_edges = std.array.filter (fun edge =>
        edge.edge_type == "composes"
      ) edges in

      {
        import_graph = build_import_graph dependency_edges,
        inheritance_hierarchy = build_inheritance_hierarchy inheritance_edges,
        composition_structure = build_composition_structure composition_edges,
        circular_dependencies = detect_circular_dependencies dependency_edges,
        optimization_opportunities = find_codegen_optimizations edges
      },

    # Build import graph for code generation
    build_import_graph = fun dependency_edges =>
      let imports_by_source = std.array.group_by (fun edge => edge.source_schema) dependency_edges in

      std.record.map (fun source imports => {
        source_type = source,
        required_imports = std.array.map (fun edge => {
          target_type = edge.target_ref,
          import_type = if edge.edge_type == "depends_on" then "direct" else "indirect",
          nullable = false
        }) imports
      }) imports_by_source,

    # Build inheritance hierarchy
    build_inheritance_hierarchy = fun inheritance_edges =>
      let hierarchy = std.array.group_by (fun edge => edge.target_ref) inheritance_edges in

      std.record.map (fun base_type inheritors => {
        base_type = base_type,
        inheriting_types = std.array.map (fun edge => edge.source_schema) inheritors,
        inheritance_depth = calculate_inheritance_depth inheritance_edges base_type
      }) hierarchy,

    # Calculate inheritance depth
    calculate_inheritance_depth = fun inheritance_edges base_type =>
      # Simplified depth calculation
      let direct_inheritors = std.array.filter (fun edge =>
        edge.target_ref == base_type
      ) inheritance_edges in
      std.array.length direct_inheritors,

    # Build composition structure
    build_composition_structure = fun composition_edges =>
      let compositions_by_owner = std.array.group_by (fun edge => edge.source_schema) composition_edges in

      std.record.map (fun owner components => {
        owner_type = owner,
        composed_components = std.array.map (fun edge => {
          component_type = edge.target_ref,
          composition_type = "aggregation",
          lifecycle_managed = true
        }) components
      }) compositions_by_owner,

    # Detect circular dependencies
    detect_circular_dependencies = fun dependency_edges =>
      # Simplified cycle detection - in practice would need proper graph algorithms
      let sources = std.array.map (fun edge => edge.source_schema) dependency_edges in
      let targets = std.array.map (fun edge => edge.target_ref) dependency_edges in

      # Check for obvious cycles (A->B and B->A)
      let potential_cycles = std.array.filter (fun source =>
        std.array.elem source targets
      ) sources in

      if std.array.is_empty potential_cycles then
        []
      else
        std.array.map (fun source => [source, "circular_dependency_detected"]) potential_cycles,

    # Find codegen optimizations
    find_codegen_optimizations = fun edges =>
      let opportunities = [] in

      # Check for redundant imports
      let redundant_imports = detect_redundant_imports edges in
      let opportunities = if std.array.is_empty redundant_imports then
        opportunities
      else
        opportunities @ [{
          type = "redundant_imports",
          description = "Multiple imports of same type detected",
          items = redundant_imports,
          optimization = "consolidate_imports"
        }] in

      # Check for deep inheritance chains
      let deep_inheritance = detect_deep_inheritance edges in
      let opportunities = if std.array.is_empty deep_inheritance then
        opportunities
      else
        opportunities @ [{
          type = "deep_inheritance",
          description = "Inheritance chains exceed recommended depth",
          items = deep_inheritance,
          optimization = "flatten_inheritance"
        }] in

      opportunities,

    # Detect redundant imports
    detect_redundant_imports = fun edges =>
      let reference_edges = std.array.filter (fun edge =>
        edge.edge_type == "references"
      ) edges in

      let references_by_target = std.array.group_by (fun edge => edge.target_ref) reference_edges in

      let redundant = std.record.filter (fun target sources =>
        std.array.length sources > 1  # Referenced by multiple sources
      ) references_by_target in

      std.record.fields redundant,

    # Detect deep inheritance chains
    detect_deep_inheritance = fun edges =>
      let inheritance_edges = std.array.filter (fun edge =>
        edge.edge_type == "inherits_from"
      ) edges in

      let inheritance_chains = build_inheritance_chains inheritance_edges in

      std.array.filter (fun chain =>
        std.array.length chain > validation_rules.codegen_constraints.max_inheritance_depth
      ) inheritance_chains,

    # Build inheritance chains (simplified)
    build_inheritance_chains = fun inheritance_edges =>
      # Simplified - in practice would need proper graph traversal
      std.array.map (fun edge => [edge.source_schema, edge.target_ref]) inheritance_edges,

    # Generate code generation ordering
    generate_codegen_ordering = fun edges =>
      let dependency_edges = std.array.filter (fun edge =>
        edge.edge_type == "depends_on"
      ) edges in

      let inheritance_edges = std.array.filter (fun edge =>
        edge.edge_type == "inherits_from"
      ) edges in

      {
        compilation_order = calculate_compilation_order dependency_edges,
        inheritance_order = calculate_inheritance_order inheritance_edges,
        import_order = calculate_import_order dependency_edges
      },

    # Calculate compilation order (topological sort of dependencies)
    calculate_compilation_order = fun dependency_edges =>
      # Simplified topological ordering
      let sources = std.array.uniq (
        std.array.map (fun edge => edge.source_schema) dependency_edges
      ) in

      # Reverse dependencies to get compilation order
      std.array.reverse sources,

    # Calculate inheritance order (base classes first)
    calculate_inheritance_order = fun inheritance_edges =>
      # Simplified - base classes should be compiled first
      let targets = std.array.uniq (
        std.array.map (fun edge => edge.target_ref) inheritance_edges
      ) in

      targets,

    # Calculate import order (dependencies first)
    calculate_import_order = fun dependency_edges =>
      # Simplified - dependencies should be imported first
      let targets = std.array.uniq (
        std.array.map (fun edge => edge.target_ref) dependency_edges
      ) in

      targets
  },

  # ============================================================================
  # COMPILATION INTEGRATION
  # ============================================================================

  compilation = {
    # Generate code from codegen edges
    generate_from_codegen_edges = fun edges graph_context =>
      # Group edges by type for batch processing
      let edges_by_type = std.array.group_by (fun edge => edge.edge_type) edges in

      # Generate codegen-specific code for each edge type
      std.record.map (fun edge_type edge_list =>
        let metadata = get_codegen_edge_metadata edge_type in
        generate_codegen_edge_code edge_type edge_list metadata graph_context
      ) edges_by_type,

    # Generate import statements
    generate_import_statements = fun edges =>
      let type_analysis = functions.analyze_type_dependencies edges in

      std.record.map (fun source info =>
        let imports = std.array.map (fun imp =>
          if imp.import_type == "direct" then
            "import ${imp.target_type};"
          else
            "import type { ${imp.target_type} };"
        ) info.required_imports in

        {
          source_file = source,
          imports = imports,
          consolidated = std.array.length imports > 1
        }
      ) type_analysis.import_graph,

    # Generate inheritance code
    generate_inheritance_code = fun edges =>
      let inheritance_hierarchy = functions.analyze_type_dependencies edges in

      std.record.flat_map (fun base_type info =>
        std.array.map (fun inheritor =>
          {
            base_type = base_type,
            inheritor_type = inheritor,
            inheritance_code = "class ${inheritor} extends ${base_type}",
            interface_required = info.inheritance_depth > 1
          }
        ) info.inheriting_types
      ) inheritance_hierarchy.inheritance_hierarchy,

    # Generate composition code
    generate_composition_code = fun edges =>
      let composition_structure = functions.analyze_type_dependencies edges in

      std.record.flat_map (fun owner info =>
        std.array.map (fun component =>
          {
            owner_type = owner,
            component_type = component.component_type,
            composition_code = "private ${component.component_type} component;",
            lifecycle_code = "this.component = new ${component.component_type}();"
          }
        ) info.composed_components
      ) composition_structure.composition_structure,

    # Generate type assertions
    generate_type_assertions = fun edges =>
      let implementation_edges = std.array.filter (fun edge =>
        edge.edge_type == "implements"
      ) edges in

      std.array.map (fun edge =>
        {
          implementor = edge.source_schema,
          interface = edge.target_ref,
          assertion_code = "assert ${edge.source_schema} instanceof ${edge.target_ref};",
          compile_time_check = true
        }
      ) implementation_edges
  }
}