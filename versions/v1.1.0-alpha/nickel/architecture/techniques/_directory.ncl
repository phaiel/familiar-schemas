# Architecture Techniques Directory - ISA compliance and CEL validation
# Inherits from global libraries with technique-specific specializations

let Libraries = import "../../libraries/all_libraries.ncl" in

# Technique validation contracts for architecture layer
{
  # Inherit all global libraries
  Libraries,

  # Architecture-specific technique validation
  technique_validation = {
    # Validate architectural techniques (high-level workflows)
    validate_architecture_technique = fun technique_json =>
      # Basic technique validation
      Libraries.TechniqueContract.validate technique_json &&

      # Architecture-specific constraints
      std.record.has_field "x-familiar-architectural-pattern" technique_json &&

      # Validate step kinds are appropriate for architecture layer
      let allowed_kinds = [Libraries.StepKinds.call, Libraries.StepKinds.switch, Libraries.StepKinds.parallel] in
      std.array.all (fun step =>
        std.array.elem step.kind allowed_kinds
      ) technique_json.steps
    ,

    # Validate CEL expressions follow architectural data flow
    validate_architectural_data_flow = fun technique_json =>
      Libraries.CELValidation.validate_return_expressions
        technique_json.return
        (std.array.map (fun step => step.id) technique_json.steps)
  },

  # Export validation functions for subdirectory inheritance
  TechniqueValidation = technique_validation,

  # Directory-level validation contract
  validate_directory = fun directory_path =>
    # Architecture techniques directory validation
    # This would validate all technique files in the directory
    { valid = true, errors = [], warnings = [] }
}