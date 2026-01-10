# Directory Composer - Extreme Composition with & and import
# Following Nickel "composable data" principle - records merged at will

# Import individual libraries for composition
let ContractLibrary = import "../libraries/contract_library.ncl" in
let HydrationLibrary = import "../libraries/hydration_library.ncl" in
let EdgeLibrary = import "../libraries/edge_library.ncl" in
let LayerLibrary = import "../libraries/layer_library.ncl" in
let ExtractionLibrary = import "../libraries/extraction_library.ncl" in
let ValidationLibrary = import "../libraries/validation_library.ncl" in

# Create composable building blocks using & (Nickel best practice)
let BaseContract = ContractLibrary & {
  hydration = HydrationLibrary,
  run_tests = ContractLibrary.enhanced_test_runner
} in

let EnhancedContract = BaseContract & {
  extraction = ExtractionLibrary.extraction_functions,
  validation = ValidationLibrary.validation_functions
} in

# Create layer-specific compositions using extreme & merging
let ArchitectureComposition = BaseContract & {
  layer_config = { get_layer = "entity" },
  extraction = ExtractionLibrary.extraction_functions.entity_extract,
  validation = ValidationLibrary.validation_functions.entity_validate
} in

let InfrastructureComposition = BaseContract & {
  layer_config = { get_layer = "node" },
  extraction = ExtractionLibrary.extraction_functions.node_extract,
  validation = ValidationLibrary.validation_functions.node_validate
} in

let CodegenComposition = BaseContract & {
  layer_config = { get_layer = "entity" },
  extraction = ExtractionLibrary.extraction_functions.entity_extract,
  validation = ValidationLibrary.validation_functions.entity_validate
} in

let DomainComposition = BaseContract & {
  layer_config = { get_layer = "domain" },
  extraction = ExtractionLibrary.extraction_functions.domain_extract,
  validation = ValidationLibrary.validation_functions.entity_validate  # Domain uses entity validation
} in

{
  # Typed collections using extreme & composition
  architecture = ArchitectureComposition & {
    Contract = {
      compose = fun config => ArchitectureComposition & { files = { contract = config } }
    }
  },

  infrastructure = InfrastructureComposition & {
    Contract = {
      compose = fun config => InfrastructureComposition & { files = { contract = config } }
    }
  },

  codegen = CodegenComposition & {
    Contract = {
      compose = fun config => CodegenComposition & { files = { contract = config } }
    }
  },

  domain = DomainComposition & {
    Contract = {
      compose = fun config => DomainComposition & { files = { contract = config } }
    }
  },

  # Metadata about the extreme composition approach
  metadata = {
    composition_strategy = "extreme_merging",
    nickel_principles = ["composable_data", "modular_configurations"],
    merge_operations = ["import", "&"],
    abstraction_level = "merged_libraries"
  }
}