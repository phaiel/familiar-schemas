# Edge Library - Merged composition of edge primitives

let EdgePrimitives = import "../primitives/edge_primitives.ncl" in

# Merge edge primitives into complete edge library
EdgePrimitives.EdgeValidationPrimitives & {
  # Merge edge types
  edge_types = EdgePrimitives.EdgeTypePrimitives,

  # Merge compatibility rules
  compatibility_rules = EdgePrimitives.EdgeCompatibilityPrimitives,

  # Library metadata

  # Enhanced edge validation with compatibility checking
  validate_edge_with_compatibility = fun schema source_kind target_kind edge_type =>
    let base_validation = EdgePrimitives.EdgeValidationPrimitives.validate_with_edge_contract schema in
    let compatibility_key = "%{source_kind}_to_%{target_kind}" in
    let allowed_edges = if std.record.has_field compatibility_key compatibility_rules then
      compatibility_rules."%{compatibility_key}"
    else [] in
    let is_compatible = std.array.elem edge_type allowed_edges in

    if base_validation.valid && is_compatible then
      {valid = true, errors = [], warnings = []}
    else if !is_compatible then
      {valid = false, errors = ["Edge type '%{edge_type}' not compatible between %{source_kind} and %{target_kind}"], warnings = []}
    else
      base_validation,

  # Edge type lookup with validation
  get_edge_type = fun edge_name =>
    let edge_type = edge_types."%{edge_name}" in
    if edge_type != {} then edge_type
    else std.contract.blame "Unknown edge type: %{edge_name}"
}