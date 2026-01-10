# Working example of layered Nickel architecture
# This demonstrates the concept without complex syntax
let make_interface = fun layer_name =>
  {
    name = layer_name,
    contract = {
      compose = fun config =>
        {
          layer = layer_name,
          config = config
        }
    },
    hydration = {},
    edge = {},
    layer = { type = layer_name }
  }
in

{
  architecture = make_interface "architecture",
  infrastructure = make_interface "infrastructure",
  codegen = make_interface "codegen",

  metadata = {
    description = "Typed collections from merged primitives",
    abstraction_level = "typed_collections",
    primitive_layers_hidden = true
  }
}
