# Infrastructure Actions Directory - Basic action validation and capability querying
let Composer = import "../../composers/directory_composer.ncl" in
let Libraries = import "../../libraries/all_libraries.ncl" in

# Specialized action validation for infrastructure layer
Libraries & {
  # Action-specific validation
  action_validation = {
    # Validate infrastructure actions (atomic operations)
    validate_infrastructure_action = fun action_json =>
      # Basic action validation
      Libraries.ActionValidation.validate_action action_json &&

      # Infrastructure-specific constraints
      std.record.has_field "x-familiar-compute-category" action_json &&

      # Validate execution model is appropriate for infrastructure
      let valid_models = ["sync", "async", "stream"] in
      if std.record.has_field "x-familiar-execution-model" action_json then
        std.array.elem action_json."x-familiar-execution-model" valid_models
      else
        true  # execution-model is optional
    ,

    # Validate action capabilities for infrastructure compatibility
    validate_action_capabilities = fun action_json =>
      if std.record.has_field "capabilities" action_json.signature then
        Libraries.ActionValidation.validate_capabilities action_json.signature.capabilities
      else
        true  # capabilities are optional
  },

  # Export validation functions for subdirectory inheritance
  ActionValidation = action_validation
} & Composer.infrastructure.Contract.compose {
  extract_from_raw = Composer.infrastructure.extraction,
  validate_pure = Composer.infrastructure.validation,
  get_layer = "node"
}