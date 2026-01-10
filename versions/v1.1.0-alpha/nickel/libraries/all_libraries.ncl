# All Libraries - Single merged bundle for extreme composition
# Following Nickel "composable data" - records merged at will with &

# Import individual libraries
let ContractLibrary = import "contract_library.ncl" in
let HydrationLibrary = import "hydration_library.ncl" in
let EdgeLibrary = import "edge_library.ncl" in
let LayerLibrary = import "layer_library.ncl" in
let ExtractionLibrary = import "extraction_library.ncl" in
let ValidationLibrary = import "validation_library.ncl" in
let TechniqueLibrary = import "technique_library.ncl" in
let ActionLibrary = import "action_library.ncl" in
let ReferenceLibrary = import "reference_library.ncl" in

# Merge all libraries into single composable bundle using &
ContractLibrary & HydrationLibrary & EdgeLibrary & LayerLibrary & ExtractionLibrary & ValidationLibrary & TechniqueLibrary & ActionLibrary & ReferenceLibrary & {
  # Library bundle metadata
    merged_libraries = ["contract", "hydration", "edge", "layer", "extraction", "validation", "technique", "action", "reference"],
    composition_strategy = "extreme_merging",
    nickel_principle = "composable_data"
  }
}