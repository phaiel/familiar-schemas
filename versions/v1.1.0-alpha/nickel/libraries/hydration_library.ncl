# Hydration Library - Merged composition of hydration primitives

let HydrationPrimitives = import "../primitives/hydration_primitives.ncl" in

# Merge hydration primitives into complete hydration library
HydrationPrimitives.BaseHydration & {
  # Merge metadata primitives
  _metadata = HydrationPrimitives.MetadataPrimitives,

  # Merge observability primitives
  _observability = HydrationPrimitives.ObservabilityPrimitives,

  # Merge operations primitives
  _operations = HydrationPrimitives.OperationsPrimitives,

  # Library metadata

  # Layer-specific hydration composers
  compose_layer_hydration = fun layer_name =>
    HydrationPrimitives.BaseHydration & {
      _metadata = HydrationPrimitives.MetadataPrimitives & {
        governance_level = layer_name
      },
      _observability = HydrationPrimitives.ObservabilityPrimitives,
      _operations = HydrationPrimitives.OperationsPrimitives
    } & (if std.record.has_field layer_name HydrationPrimitives.LayerHydrationPrimitives then
      HydrationPrimitives.LayerHydrationPrimitives."%{layer_name}"
    else {}),

  # Enhanced merging with validation
  validated_merge = fun base overrides =>
    let merged = base & overrides in
    let has_required_fields = std.record.has_field "_metadata" merged &&
                              std.record.has_field "_observability" merged in
    if has_required_fields then merged
    else std.contract.blame "Hydration merge missing required fields"
}