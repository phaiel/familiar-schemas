# Architecture subdirectory - minimal merged blocks
let Composer = import "../../composers/directory_composer.ncl" in
Composer.architecture.Contract.compose {
  extract_from_raw = Composer.architecture.extraction,
  validate_pure = Composer.architecture.validation,
  get_layer = "topology"
}
