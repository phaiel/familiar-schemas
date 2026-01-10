# Codegen subdirectory - minimal merged blocks
let Composer = import "../../composers/directory_composer.ncl" in
Composer.codegen.Contract.compose {
  extract_from_raw = Composer.codegen.extraction,
  validate_pure = Composer.codegen.validation,
  get_layer = "entity"
}
