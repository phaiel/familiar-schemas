# Extraction Primitives - Common schema extraction patterns

{
  # Entity extraction primitives
  EntityExtraction = {
    # Extract pure component data from mixed schemas
    extract_entity_data = fun raw_schema =>
      let forbidden_runtime = [
        "x-familiar-service", "x-familiar-api", "x-familiar-queue",
        "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources",
        "x-familiar-monitoring-enabled", "x-familiar-node-type",
        "x-familiar-scaling", "x-familiar-deploy-config"
      ] in

      let forbidden_infrastructure = [
        "x-familiar-environment", "x-familiar-networking", "x-familiar-storage",
        "x-familiar-compute-limits", "x-familiar-load-balancing"
      ] in

      let forbidden_topology = [
        "x-familiar-depends_on", "x-familiar-communicates_with",
        "x-familiar-contains", "x-familiar-orchestrates",
        "x-familiar-deploys_to", "x-familiar-routes_to"
      ] in

      # Filter out all non-entity concerns
      std.record.filter
        (fun key _ =>
          !std.array.elem key (forbidden_runtime @ forbidden_infrastructure @ forbidden_topology))
        raw_schema,

    # Extract pure entity types from mixed schemas
    extract_entity_types = fun raw_schema =>
      let type_fields = [
        "$schema", "$id", "title", "description", "type",
        "properties", "required", "additionalProperties",
        "x-familiar-kind", "x-familiar-description"
      ] in

      std.record.filter
        (fun key _ => std.array.elem key type_fields)
        raw_schema
  },

  # Node extraction primitives
  NodeExtraction = {
    # Extract compute resources from mixed schemas
    extract_compute_resources = fun raw_schema =>
      let compute_fields = [
        "x-familiar-node-type", "x-familiar-resources", "x-familiar-scaling",
        "x-familiar-monitoring-enabled", "x-familiar-deploy-config",
        "x-familiar-environment", "x-familiar-networking"
      ] in

      # Filter to compute concerns only
      let compute_only = std.record.filter
        (fun key _ =>
          std.array.elem key compute_fields ||
          key == "x-familiar-kind" ||
          key == "x-familiar-description" ||
          key == "$schema" ||
          key == "$id" ||
          key == "title" ||
          key == "description")
        raw_schema in

      # Ensure required compute fields exist with defaults
      let with_defaults = compute_only & {
        "x-familiar-node-type" = if std.record.has_field "x-familiar-node-type" compute_only then
          compute_only."x-familiar-node-type" else "generic",
        "x-familiar-resources" = if std.record.has_field "x-familiar-resources" compute_only then
          compute_only."x-familiar-resources" else { cpu = "1", memory = "1Gi" },
        "x-familiar-scaling" = if std.record.has_field "x-familiar-scaling" compute_only then
          compute_only."x-familiar-scaling" else { min = 1, max = 1 }
      } in with_defaults,

    # Extract topology wiring from mixed schemas
    extract_topology_wiring = fun raw_schema =>
      let topology_fields = [
        "x-familiar-depends_on", "x-familiar-communicates_with",
        "x-familiar-contains", "x-familiar-orchestrates",
        "x-familiar-deploys_to", "x-familiar-routes_to",
        "x-familiar-edges"
      ] in

      std.record.filter
        (fun key _ => std.array.elem key topology_fields)
        raw_schema
  },

  # Domain extraction primitives
  DomainExtraction = {
    # Extract business domain logic from mixed schemas
    extract_domain_logic = fun raw_schema =>
      let domain_fields = [
        "x-familiar-business-rules", "x-familiar-domain-context",
        "x-familiar-agent-capabilities", "x-familiar-api",
        "x-familiar-auth", "x-familiar-tenant"
      ] in

      std.record.filter
        (fun key _ => std.array.elem key domain_fields)
        raw_schema
  }
}