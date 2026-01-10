# Import typed collection (abstracted from primitives)
let Composer = import "../composers/directory_composer.ncl" in

# Use typed infrastructure interface - minimal merged blocks from libraries/primitives
Composer.infrastructure.Contract.compose {
  extract_from_raw = Composer.infrastructure.extraction,
  validate_pure = Composer.infrastructure.validation,
  get_layer = "node"
}