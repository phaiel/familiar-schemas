# Import typed collection (abstracted from primitives)
let Composer = import "../../composers/directory_composer.ncl" in

# Use typed codegen interface - minimal merged blocks from libraries/primitives
Composer.codegen.Contract.compose {
  extract_from_raw = Composer.codegen.extraction,
  validate_pure = Composer.codegen.validation,
  get_layer = "entity"
}