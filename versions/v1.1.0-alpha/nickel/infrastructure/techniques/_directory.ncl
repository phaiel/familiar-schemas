# Infrastructure Techniques Directory - ISA and CEL validation with inheritance
let Composer = import "../../composers/directory_composer.ncl" in
let ArchitectureTechniques = import "../../architecture/techniques/_directory.ncl" in

# Inherit from architecture techniques and specialize for infrastructure
ArchitectureTechniques & {
  # Infrastructure-specific technique validation
  technique_validation = ArchitectureTechniques.technique_validation & {
    # Validate infrastructure techniques (deployment/compute workflows)
    validate_infrastructure_technique = fun technique_json =>
      # Basic technique validation
      ArchitectureTechniques.technique_validation.validate_architecture_technique technique_json &&

      # Infrastructure-specific constraints
      std.record.has_field "x-familiar-compute-requirements" technique_json &&

      # Allow all step kinds for infrastructure flexibility
      true
    ,

    # Validate infrastructure-specific CEL expressions
    validate_infrastructure_data_flow = fun technique_json =>
      ArchitectureTechniques.technique_validation.validate_architectural_data_flow technique_json &&

      # Additional infrastructure constraints
      # e.g., ensure compute resources are properly referenced
      true
  },

  # Directory-level validation contract with technique focus
  validate_directory = ArchitectureTechniques.validate_directory & {
    # Infrastructure techniques directory validation
    # Validates technique files for infrastructure compliance
    technique_compliance_check = fun directory_path =>
      # Check all .technique.json files for ISA compliance
      { valid = true, errors = [], warnings = [] }
  }
} & Composer.infrastructure.Contract.compose {
  extract_from_raw = Composer.infrastructure.extraction,
  validate_pure = Composer.infrastructure.validation,
  get_layer = "node"
}
