# Infrastructure subdirectory - minimal merged blocks
let Composer = import "../../composers/directory_composer.ncl" in
Composer.infrastructure.Contract.compose {
  extract_from_raw = Composer.infrastructure.extraction,
  validate_pure = Composer.infrastructure.validation,
  get_layer = "node"
}
