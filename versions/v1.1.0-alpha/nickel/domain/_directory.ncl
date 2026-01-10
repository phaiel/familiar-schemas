# Import typed collection with extreme & composition
let Composer = import "../composers/directory_composer.ncl" in

# Use extreme & composition - minimal merged blocks from libraries
Composer.domain.Contract.compose {
  extract_from_raw = Composer.domain.extraction,
  validate_pure = Composer.domain.validation,
  get_layer = "domain"
}