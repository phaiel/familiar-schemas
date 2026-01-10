# Validation Primitives - Common schema validation patterns

{
  # Entity validation primitives
  EntityValidation = {
    # Validate pure component data (no runtime concerns)
    validate_pure_entity = fun schema =>
      let forbidden_runtime = [
        "x-familiar-service", "x-familiar-api", "x-familiar-queue",
        "x-familiar-policy", "x-familiar-persistence", "x-familiar-resources"
      ] in

      let forbidden_infrastructure = [
        "x-familiar-environment", "x-familiar-networking", "x-familiar-storage",
        "x-familiar-compute-limits", "x-familiar-node-type", "x-familiar-scaling"
      ] in

      let forbidden_topology = [
        "x-familiar-depends_on", "x-familiar-communicates_with",
        "x-familiar-contains", "x-familiar-orchestrates"
      ] in

      let all_forbidden = forbidden_runtime @ forbidden_infrastructure @ forbidden_topology in

      let violations = std.record.fields schema
        |> std.array.filter (fun field => std.array.elem field all_forbidden) in

      if std.array.is_empty violations then
        { valid = true, errors = [], warnings = [] }
      else
        {
          valid = false,
          errors = std.array.map (fun field => "Forbidden runtime/infrastructure field in component: %{field}") violations,
          warnings = []
        }
  },

  # Node validation primitives
  NodeValidation = {
    # Validate pure compute resources (no business logic)
    validate_pure_compute = fun schema =>
      let forbidden_business = [
        "x-familiar-business-rules", "x-familiar-domain-context",
        "x-familiar-agent-capabilities", "x-familiar-api-contracts"
      ] in

      let forbidden_entity = [
        "properties", "required", "additionalProperties",
        "x-familiar-component-type", "x-familiar-entity-kind"
      ] in

      let forbidden_topology = [
        "x-familiar-depends_on", "x-familiar-communicates_with",
        "x-familiar-contains", "x-familiar-orchestrates"
      ] in

      let all_forbidden = forbidden_business @ forbidden_entity @ forbidden_topology in

      let violations = std.record.fields schema
        |> std.array.filter (fun field => std.array.elem field all_forbidden) in

      if std.array.is_empty violations then
        { valid = true, errors = [], warnings = [] }
      else
        {
          valid = false,
          errors = std.array.map (fun field => "Forbidden business/entity field in node: %{field}") violations,
          warnings = []
        }
  },

  # Topology validation primitives
  TopologyValidation = {
    # Validate pure topology wiring (no implementation details)
    validate_pure_topology = fun schema =>
      let forbidden_implementation = [
        "x-familiar-service", "x-familiar-api", "x-familiar-resources",
        "x-familiar-scaling", "x-familiar-deploy-config", "properties",
        "x-familiar-component-type", "x-familiar-node-type"
      ] in

      let allowed_topology = [
        "x-familiar-depends_on", "x-familiar-communicates_with",
        "x-familiar-contains", "x-familiar-orchestrates",
        "x-familiar-edges"
      ] in

      let violations = std.record.fields schema
        |> std.array.filter (fun field =>
          std.array.elem field forbidden_implementation &&
          !std.array.elem field allowed_topology) in

      if std.array.is_empty violations then
        { valid = true, errors = [], warnings = [] }
      else
        {
          valid = false,
          errors = std.array.map (fun field => "Forbidden implementation field in topology: %{field}") violations,
          warnings = []
        }
  }
}