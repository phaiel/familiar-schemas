/**
 * Infrastructure Edge Semantics
 *
 * Inherits global edge types and specializes them for infrastructure/compute concerns.
 * Defines how edges behave in the context of nodes, resources, and deployment.
 *
 * @category infrastructure
 * @graph-value high - specializes edges for compute infrastructure
 * @validation context-aware - infrastructure-specific edge constraints
 */

let global_edges = import "../extensions/edge-type.ncl" in

{
  # ============================================================================
  # INHERITED & SPECIALIZED EDGE TYPES
  # ============================================================================

  # Inherit all global edge types with infrastructure-specific specializations
  edge_types = global_edges.edge_types & {
    # Add infrastructure-specific edge types
    provides = {
      directionality = "directed",
      cardinality = "one_to_many",
      transitivity = false,
      cyclicity = "acyclic",
      data_flow = "resource_provisioning",
      strength = "strong",
      context = "resource_management",
      compilation_implications = {
        generate_resource_allocation = true,
        generate_capacity_planning = true,
        generate_monitoring_setup = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true
      }
    },

    consumes = {
      directionality = "directed",
      cardinality = "many_to_one",
      transitivity = false,
      cyclicity = "acyclic",
      data_flow = "resource_consumption",
      strength = "medium",
      context = "resource_utilization",
      compilation_implications = {
        generate_resource_requests = true,
        generate_quota_management = true,
        generate_usage_monitoring = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true
      }
    },

    # Specialize existing edges for infrastructure context
    depends_on = global_edges.edge_types.depends_on & {
      context = "infrastructure_dependencies",
      compilation_implications = global_edges.edge_types.depends_on.compilation_implications & {
        generate_infrastructure_dependencies = true,
        generate_startup_ordering = true,
        generate_health_dependencies = true
      },
      traversal_rules = global_edges.edge_types.depends_on.traversal_rules & {
        infrastructure_cycles_forbidden = true,
        startup_order_critical = true
      }
    },

    deploys_to = global_edges.edge_types.deploys_to & {
      context = "physical_deployment",
      compilation_implications = {
        generate_deployment_scripts = true,
        generate_infrastructure_templates = true,
        generate_rollback_procedures = true,
        generate_scaling_automation = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = true,
        deployment_order_critical = true
      }
    },

    contains = global_edges.edge_types.contains & {
      context = "infrastructure_containment",
      compilation_implications = {
        generate_namespace_isolation = true,
        generate_resource_sharing = true,
        generate_cleanup_procedures = true
      },
      traversal_rules = {
        can_traverse = true,
        allows_cycles = false,
        topological_sort_required = false  # Containment is hierarchical, not dependency-based
      }
    }
  },

  # ============================================================================
  # INFRASTRUCTURE-SPECIFIC VALIDATION RULES
  # ============================================================================

  validation_rules = {
    # Edge combinations allowed in infrastructure schemas
    allowed_edge_combinations = [
      # Node to Resource provisioning
      {
        source_kind = "node",
        target_kind = "resource",
        allowed_edges = ["provides", "consumes", "depends_on"]
      },

      # Node to Node relationships
      {
        source_kind = "node",
        target_kind = "node",
        allowed_edges = ["depends_on", "communicates_with", "contains"]
      },

      # System to Node deployment
      {
        source_kind = "system",
        target_kind = "node",
        allowed_edges = ["deploys_to"]
      },

      # Resource to Resource dependencies
      {
        source_kind = "resource",
        target_kind = "resource",
        allowed_edges = ["depends_on"]
      }
    ],

    # Prohibited patterns in infrastructure
    prohibited_patterns = [
      # Resources shouldn't contain other resources (use dependencies)
      {
        source_kind = "resource",
        target_kind = "resource",
        prohibited_edges = ["contains"]
      },

      # Systems shouldn't provide resources (that's node's job)
      {
        source_kind = "system",
        target_kind = "resource",
        prohibited_edges = ["provides"]
      }
    ],

    # Infrastructure-specific constraints
    infrastructure_constraints = {
      # Maximum nodes per resource
      max_consumers_per_resource = 10,

      # Require explicit resource specifications
      allow_implicit_resources = false,

      # Enforce resource dependency chains
      max_resource_dependency_depth = 5
    }
  },

  # ============================================================================
  # INFRASTRUCTURE-SPECIFIC FUNCTIONS
  # ============================================================================

  functions = {
    # Validate edge compatibility in infrastructure context
    validate_infrastructure_edge = fun source_kind target_kind edge_type =>
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

    # Get edge metadata with infrastructure specialization
    get_infrastructure_edge_metadata = fun edge_type =>
      if std.record.has_field edge_type edge_types then
        edge_types."%{edge_type}"
      else
        global_edges.functions.get_edge_characteristics edge_type,

    # Analyze infrastructure resource utilization
    analyze_resource_utilization = fun edges =>
      let resource_edges = std.array.filter (fun edge =>
        std.array.elem edge.edge_type ["provides", "consumes"]
      ) edges in

      let providers = std.array.filter (fun edge => edge.edge_type == "provides") resource_edges in
      let consumers = std.array.filter (fun edge => edge.edge_type == "consumes") resource_edges in

      {
        total_providers = std.array.length providers,
        total_consumers = std.array.length consumers,
        resource_contention = calculate_resource_contention resource_edges,
        utilization_efficiency = calculate_utilization_efficiency providers consumers,
        scaling_recommendations = generate_scaling_recommendations resource_edges
      },

    # Calculate resource contention (over-subscription)
    calculate_resource_contention = fun resource_edges =>
      let provides_by_resource = std.array.group_by (fun edge => edge.target_ref) (
        std.array.filter (fun edge => edge.edge_type == "provides") resource_edges
      ) in

      let consumes_by_resource = std.array.group_by (fun edge => edge.source_schema) (
        std.array.filter (fun edge => edge.edge_type == "consumes") resource_edges
      ) in

      # Simplified contention calculation
      let over_subscribed = std.record.filter (fun resource consumers =>
        std.array.length consumers > validation_rules.infrastructure_constraints.max_consumers_per_resource
      ) consumes_by_resource in

      std.record.map (fun resource consumers => {
        resource = resource,
        consumer_count = std.array.length consumers,
        max_allowed = validation_rules.infrastructure_constraints.max_consumers_per_resource,
        over_subscribed = std.array.length consumers > validation_rules.infrastructure_constraints.max_consumers_per_resource
      }) over_subscribed,

    # Calculate utilization efficiency
    calculate_utilization_efficiency = fun providers consumers =>
      let total_providers = std.array.length providers in
      let total_consumers = std.array.length consumers in

      if total_providers == 0 then
        0.0
      else
        (total_consumers / total_providers) * 100.0,

    # Generate scaling recommendations
    generate_scaling_recommendations = fun resource_edges =>
      let contention = calculate_resource_contention resource_edges in

      if std.record.is_empty contention then
        []
      else
        std.record.map (fun resource info =>
          if info.over_subscribed then
            {
              resource = resource,
              action = "scale_up",
              reason = "over_subscribed",
              recommended_capacity = info.consumer_count * 2
            }
          else
            {
              resource = resource,
              action = "optimize",
              reason = "under_utilized",
              recommended_capacity = info.consumer_count
            }
        ) contention,

    # Validate resource dependency chains
    validate_resource_dependencies = fun edges =>
      let dependency_edges = std.array.filter (fun edge =>
        edge.edge_type == "depends_on" &&
        (std.string.contains "resource" edge.source_schema ||
         std.string.contains "resource" edge.target_ref)
      ) edges in

      # Check for cycles in resource dependencies
      let cycles = detect_resource_dependency_cycles dependency_edges in

      if std.array.is_empty cycles then
        { valid = true, errors = [], warnings = [] }
      else
        {
          valid = false,
          errors = std.array.map (fun cycle =>
            "Resource dependency cycle detected: ${std.string.join " -> " cycle}"
          ) cycles,
          warnings = []
        },

    # Detect cycles in resource dependencies (simplified)
    detect_resource_dependency_cycles = fun dependency_edges =>
      # In practice, would need proper cycle detection algorithm
      # For now, return empty (no cycles detected)
      [],

    # Generate infrastructure deployment ordering
    generate_deployment_ordering = fun edges =>
      let deployment_edges = std.array.filter (fun edge =>
        edge.edge_type == "deploys_to"
      ) edges in

      let containment_edges = std.array.filter (fun edge =>
        edge.edge_type == "contains"
      ) edges in

      # Combine deployment and containment for ordering
      let all_structural_edges = deployment_edges @ containment_edges in

      {
        deployment_phases = group_by_deployment_phase all_structural_edges,
        startup_order = calculate_startup_order all_structural_edges,
        shutdown_order = calculate_shutdown_order all_structural_edges
      },

    # Group edges by deployment phase
    group_by_deployment_phase = fun edges =>
      let infrastructure_edges = std.array.filter (fun edge =>
        std.string.contains "infrastructure" edge.target_ref
      ) edges in

      let application_edges = std.array.filter (fun edge =>
        !std.string.contains "infrastructure" edge.target_ref
      ) edges in

      {
        infrastructure = infrastructure_edges,
        applications = application_edges
      },

    # Calculate startup order (reverse of dependencies)
    calculate_startup_order = fun edges =>
      # Simplified topological sort for startup
      let dependency_edges = std.array.filter (fun edge =>
        edge.edge_type == "depends_on"
      ) edges in

      # In practice, would implement topological sort
      std.array.map (fun edge => edge.source_schema) dependency_edges,

    # Calculate shutdown order (follow dependencies)
    calculate_shutdown_order = fun edges =>
      # Simplified reverse topological sort for shutdown
      let dependency_edges = std.array.filter (fun edge =>
        edge.edge_type == "depends_on"
      ) edges in

      # In practice, would implement reverse topological sort
      std.array.map (fun edge => edge.target_ref) dependency_edges
  },

  # ============================================================================
  # COMPILATION INTEGRATION
  # ============================================================================

  compilation = {
    # Generate infrastructure code from edges
    generate_from_infrastructure_edges = fun edges graph_context =>
      # Group edges by type for batch processing
      let edges_by_type = std.array.group_by (fun edge => edge.edge_type) edges in

      # Generate infrastructure-specific code for each edge type
      std.record.map (fun edge_type edge_list =>
        let metadata = get_infrastructure_edge_metadata edge_type in
        generate_infrastructure_edge_code edge_type edge_list metadata graph_context
      ) edges_by_type,

    # Generate resource allocation manifests
    generate_resource_manifests = fun edges =>
      let resource_edges = std.array.filter (fun edge =>
        std.array.elem edge.edge_type ["provides", "consumes"]
      ) edges in

      let manifests_by_resource = std.array.group_by (fun edge => edge.target_ref) resource_edges in

      std.record.map (fun resource_ref edges => {
        resource = resource_ref,
        providers = std.array.filter (fun edge => edge.edge_type == "provides") edges,
        consumers = std.array.filter (fun edge => edge.edge_type == "consumes") edges,
        allocation_strategy = "fair_sharing",
        monitoring_enabled = true
      }) manifests_by_resource,

    # Generate deployment automation
    generate_deployment_automation = fun edges =>
      let deployment_ordering = functions.generate_deployment_ordering edges in

      {
        startup_script = generate_startup_script deployment_ordering.startup_order,
        shutdown_script = generate_shutdown_script deployment_ordering.shutdown_order,
        health_checks = generate_health_checks edges,
        rollback_procedures = generate_rollback_procedures edges
      },

    # Generate startup script
    generate_startup_script = fun startup_order =>
      std.array.map (fun component =>
        "start_component ${component}"
      ) startup_order,

    # Generate shutdown script
    generate_shutdown_script = fun shutdown_order =>
      std.array.map (fun component =>
        "stop_component ${component}"
      ) shutdown_order,

    # Generate health checks
    generate_health_checks = fun edges =>
      let nodes = std.array.uniq (
        std.array.flat_map (fun edge => [edge.source_schema, edge.target_ref]) edges
      ) in

      std.array.map (fun node => {
        component = node,
        health_endpoint = "/health",
        timeout = "30s",
        interval = "10s"
      }) nodes,

    # Generate rollback procedures
    generate_rollback_procedures = fun edges =>
      let deployment_edges = std.array.filter (fun edge =>
        edge.edge_type == "deploys_to"
      ) edges in

      std.array.map (fun edge => {
        component = edge.source_schema,
        target_infrastructure = edge.target_ref,
        rollback_command = "rollback_component ${edge.source_schema}",
        cleanup_commands = ["cleanup_resources ${edge.source_schema}"]
      }) deployment_edges
  }
}