# Import typed collection with extreme & composition
let Composer = import "../composers/directory_composer.ncl" in

# Use extreme & composition - minimal merged blocks from libraries
Composer.codegen.Contract.compose {
  extract_from_raw = Composer.codegen.extraction,
  validate_pure = Composer.codegen.validation,
  get_layer = "entity"
}