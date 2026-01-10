# Domain subdirectory - minimal merged blocks
let Composer = import "../../composers/directory_composer.ncl" in
Composer.domain.Contract.compose {
  extract_from_raw = Composer.domain.extraction,
  validate_pure = Composer.codegen.validation,
  get_layer = "domain"
}
