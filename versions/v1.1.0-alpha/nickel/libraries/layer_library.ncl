# Layer Library - Merged composition of layer primitives

let LayerPrimitives = import "../primitives/layer_primitives.ncl" in

# Merge layer primitives into complete layer library
{
  # Layer types
  layer_types = LayerPrimitives.LayerTypePrimitives,

  # Layer functions
  layer_functions = LayerPrimitives.LayerFunctionPrimitives,

  # Compatibility rules
  compatibility_rules = LayerPrimitives.LayerCompatibilityPrimitives,

  # Library metadata

  # Layer validation
  validate_layer_schema = fun schema layer_name =>
    let schema_kind = if std.record.has_field "x-familiar-kind" schema then
      schema."x-familiar-kind"
    else "unknown" in
    let allowed_kinds = if std.record.has_field "%{layer_name}_schemas" compatibility_rules then
      compatibility_rules."%{layer_name}_schemas" else [] in
    let is_compatible = std.array.elem schema_kind allowed_kinds in

    if is_compatible then
      {valid = true, errors = [], warnings = []}
    else
      {valid = false, errors = ["Schema kind '%{schema_kind}' not compatible with %{layer_name} layer"], warnings = []},

  # Get layer-specific functions
  get_layer_functions = fun layer_name =>
    if std.record.has_field layer_name layer_functions then
      layer_functions."%{layer_name}"
    else {
      extract_from_raw = fun schema => schema,
      validate_pure = fun schema => {valid = true, errors = [], warnings = []}
    }
}