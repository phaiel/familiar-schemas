# Architecture Directory - Orthological reference and inheritance validation
let Composer = import "../composers/directory_composer.ncl" in
let Libraries = import "../libraries/all_libraries.ncl" in

# Architecture-specific reference validation
Libraries & {
  # Reference validation for architecture layer (directory level)
  reference_validation = {
    # Validate architecture directory schemas (should use inheritance, not $refs)
    validate_architecture_directory = fun schema =>
      Libraries.ReferenceContract.validate schema "directory"
    ,

    # Validate architecture meta schemas (may use $refs for schema composition)
    validate_architecture_meta = fun schema =>
      Libraries.ReferenceContract.validate schema "meta"
  },

  # Export validation functions
  ReferenceValidation = reference_validation
} & Composer.architecture.Contract.compose {
  extract_from_raw = Composer.architecture.extraction,
  validate_pure = Composer.architecture.validation,
  get_layer = "topology"
}