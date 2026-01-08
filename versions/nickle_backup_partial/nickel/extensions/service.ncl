# x-familiar-service Extension Schema
# Defines service registration and discovery behavior

{
  extension = "x-familiar-service",
  description = "Defines how a schema participates in service architecture",
  category = "architecture",

  # JSON Schema definition
  schema = {
    type = "object",
    properties = {
      name = { type = "string" },
      version = { type = "string" },
      protocol = {
        type = "string",
        enum = ["http", "grpc", "websocket", "tcp", "udp"]
      },
      endpoints = {
        type = "array",
        items = {
          type = "object",
          properties = {
            path = { type = "string" },
            method = { type = "string", enum = ["GET", "POST", "PUT", "DELETE", "PATCH"] },
            description = { type = "string" }
          }
        }
      },
      dependencies = {
        type = "array",
        items = { type = "string" }
      },
      health_check = {
        type = "object",
        properties = {
          path = { type = "string" },
          interval = { type = "string" },
          timeout = { type = "string" }
        }
      }
    },
    required = ["name"]
  },

  # Validation contract
  contract = std.contract.custom (fun label value =>
    if std.record.has_field "name" value then
      if std.string.length value.name > 0 then
        # Validate endpoints if present
        if std.record.has_field "endpoints" value then
          let endpoints = value.endpoints in
          if std.is_array endpoints then
            let valid_endpoints = std.array.all (fun endpoint =>
              std.record.has_field "path" endpoint && std.record.has_field "method" endpoint
            ) endpoints in
            if valid_endpoints then
              value
            else
              std.contract.blame_with label "Service endpoints must have path and method"
          else
            std.contract.blame_with label "x-familiar-service.endpoints must be an array"
        else
          value
      else
        std.contract.blame_with label "x-familiar-service.name cannot be empty"
    else
      std.contract.blame_with label "x-familiar-service must specify name"
  ),

  # Code generation implications
  codegen = {
    rust = {
      service_trait = fun service schema_name =>
        let service_name = service.name in
        {
          trait_definition = "pub trait ${service_name} {\n    ${generate_service_methods service.endpoints or []}\n}",
          client_impl = generate_service_client service schema_name,
          server_impl = generate_service_server service schema_name
        },

      tower_service = fun service schema_name =>
        let service_name = service.name in
        {
          service_struct = "pub struct ${service_name}Service {\n    ${generate_service_fields service}\n}",
          layer_impl = generate_tower_layer service,
          middleware_stack = generate_middleware_stack service
        },

      actix_web_routes = fun service schema_name =>
        let endpoints = service.endpoints or [] in
        {
          route_handlers = std.array.map (fun endpoint =>
            generate_actix_route endpoint schema_name
          ) endpoints,
          route_registration = generate_route_config endpoints service.name
        }
    },

    typescript = {
      express_routes = fun service schema_name =>
        let endpoints = service.endpoints or [] in
        {
          route_definitions = std.array.map (fun endpoint =>
            generate_express_route endpoint schema_name
          ) endpoints,
          router_setup = generate_express_router endpoints service.name
        },

      nestjs_controller = fun service schema_name =>
        let endpoints = service.endpoints or [] in
        {
          controller_decorator = "@Controller('${service.name}')",
          method_decorators = std.array.map (fun endpoint =>
            generate_nestjs_method endpoint
          ) endpoints,
          service_injection = generate_nestjs_service_injection service
        }
    },

    python = {
      fastapi_routes = fun service schema_name =>
        let endpoints = service.endpoints or [] in
        {
          route_decorators = std.array.map (fun endpoint =>
            generate_fastapi_decorator endpoint
          ) endpoints,
          route_functions = std.array.map (fun endpoint =>
            generate_fastapi_function endpoint schema_name
          ) endpoints,
          router_setup = generate_fastapi_router endpoints service.name
        },

      flask_routes = fun service schema_name =>
        let endpoints = service.endpoints or [] in
        {
          route_decorators = std.array.map (fun endpoint =>
            "@app.route('${endpoint.path}', methods=['${endpoint.method}'])"
          ) endpoints
        }
    }
  },

  # Runtime implications
  runtime = {
    service_discovery = fun service => {
      registration_required = true,
      service_name = service.name,
      service_version = service.version or "1.0.0",
      protocol = service.protocol or "http",
      health_check_enabled = std.record.has_field "health_check" service
    },

    observability = fun service => {
      metrics_enabled = true,
      request_tracing = service.protocol == "http",
      dependency_tracking = std.record.has_field "dependencies" service,
      performance_monitoring = true,
      error_tracking = true
    },

    resilience = fun service => {
      circuit_breaker = std.record.has_field "health_check" service,
      retry_policy = "exponential_backoff",
      timeout_policy = "service_specific",
      load_balancing = true,
      rate_limiting = service.protocol == "http"
    }
  },

  # Directory restrictions
  directory_rules = {
    allowed_in = ["infrastructure"],
    forbidden_in = ["entities", "ui", "components"],
    required_for_kind = ["node", "system"]
  },

  # Service mesh implications
  service_mesh = {
    istio_config = fun service => {
      virtual_service = generate_istio_virtual_service service,
      destination_rule = generate_istio_destination_rule service,
      peer_authentication = generate_istio_peer_authentication service
    },

    linkerd_config = fun service => {
      service_profile = generate_linkerd_service_profile service,
      traffic_split = generate_linkerd_traffic_split service
    }
  }
}

# Helper functions for service mesh generation
let generate_istio_virtual_service = fun service =>
  "apiVersion: networking.istio.io/v1beta1\nkind: VirtualService\nmetadata:\n  name: ${service.name}\nspec:\n  http:\n  - route:\n    - destination:\n        host: ${service.name}"

let generate_linkerd_service_profile = fun service =>
  "apiVersion: linkerd.io/v1alpha2\nkind: ServiceProfile\nmetadata:\n  name: ${service.name}\n  namespace: default\nspec:\n  routes:\n  ${generate_linkerd_routes service.endpoints or []}"
