# Technique Library - Validation contracts for ISA compliance and CEL expressions
# Following Nickel "composable data" principles with extreme & merging

{
  # ============================================================================
  # STEP KIND DEFINITIONS (Architectural Constants)
  # ============================================================================

  StepKinds = {
    call = "call",           # Execute single action
    switch = "switch",       # Conditional branching
    map = "map",             # Transform arrays
    parallel = "parallel",   # Parallel execution
    transform = "transform"  # Data transformation
  },

  # ============================================================================
  # CEL EXPRESSION VALIDATION
  # ============================================================================

  CELValidation = {
    # Validate $.step.field reference patterns
    validate_cel_reference = fun expr available_step_ids =>
      if std.is_string expr then
        let parts = std.string.split "." expr in
        if std.array.length parts >= 2 && std.array.elem_at 0 parts == "$" then
          let step_id = std.array.elem_at 1 parts in
          std.array.elem step_id available_step_ids ||
          step_id == "input"  # Allow $.input.* references
        else
          true  # Non-$. expressions are allowed
      else
        true  # Non-string expressions are valid for now
    ,

    # Recursively validate all CEL expressions in an object
    validate_cel_expressions_in_object = fun obj available_step_ids =>
      if std.is_record obj then
        let field_validations = std.record.map (fun _field_name field_value =>
          if std.is_string field_value then
            CELValidation.validate_cel_reference field_value available_step_ids
          else if std.is_record field_value then
            CELValidation.validate_cel_expressions_in_object field_value available_step_ids
          else if std.is_array field_value then
            std.array.all (fun item =>
              if std.is_string item then
                CELValidation.validate_cel_reference item available_step_ids
              else if std.is_record item then
                CELValidation.validate_cel_expressions_in_object item available_step_ids
              else
                true
            ) field_value
          else
            true
        ) obj in
        std.record.values field_validations |> std.array.all std.function.id
      else
        true
    ,

    # Validate step arguments contain valid CEL expressions
    validate_step_args = fun args available_step_ids =>
      CELValidation.validate_cel_expressions_in_object args available_step_ids
    ,

    # Validate return object CEL expressions
    validate_return_expressions = fun return_obj available_step_ids =>
      CELValidation.validate_cel_expressions_in_object return_obj available_step_ids
  },

  # ============================================================================
  # ISA COMPLIANCE VALIDATION (Input-Steps-Action-Return)
  # ============================================================================

  ISAValidation = {
    # Validate input contract structure
    validate_input_contract = fun input =>
      std.is_record input && std.record.has_field "$\ref" input
    ,

    # Validate output contract structure
    validate_output_contract = fun output =>
      std.is_record output && std.record.has_field "$\ref" output
    ,

    # Validate individual step structure
    validate_step = fun step =>
      std.is_record step &&
      std.record.has_field "id" step &&
      std.record.has_field "kind" step &&
      std.record.has_field "action" step &&
      std.is_string step.id &&
      std.is_string step.kind &&
      std.array.elem step.kind [
        StepKinds.call,
        StepKinds.switch,
        StepKinds.map,
        StepKinds.parallel,
        StepKinds.transform
      ]
    ,

    # Validate steps array structure
    validate_steps_structure = fun steps =>
      std.is_array steps &&
      std.array.length steps > 0 &&
      std.array.all ISAValidation.validate_step steps &&
      # Check for unique step IDs
      (let step_ids = std.array.map (fun step => step.id) steps in
       std.array.length step_ids == std.array.length (std.array.unique step_ids))
    ,

    # Validate data flow (no circular dependencies in CEL expressions)
    validate_data_flow = fun steps return_obj =>
      let available_step_ids = std.array.map (fun step => step.id) steps in
      # For now, just check that return expressions reference valid steps
      CELValidation.validate_return_expressions return_obj available_step_ids
  },

  # ============================================================================
  # TECHNIQUE CONTRACT (Complete Validation)
  # ============================================================================

  TechniqueContract = {
    validate = fun technique_json =>
      # Basic structure validation
      std.is_record technique_json &&
      std.record.has_field "input" technique_json &&
      std.record.has_field "output" technique_json &&
      std.record.has_field "steps" technique_json &&
      std.record.has_field "return" technique_json &&
      std.record.has_field "x-familiar-kind" technique_json &&

      # ISA compliance
      ISAValidation.validate_input_contract technique_json.input &&
      ISAValidation.validate_output_contract technique_json.output &&
      ISAValidation.validate_steps_structure technique_json.steps &&
      ISAValidation.validate_data_flow technique_json.steps technique_json.return &&

      # CEL expression validation
      (let available_step_ids = std.array.map (fun step => step.id) technique_json.steps in
       CELValidation.validate_return_expressions technique_json.return available_step_ids &&

       # Validate each step's args
       std.array.all (fun step =>
         if std.record.has_field "args" step then
           CELValidation.validate_step_args step.args available_step_ids
         else
           true  # Steps don't require args
       ) technique_json.steps)
  },

  # ============================================================================
  # LIBRARY METADATA
  # ============================================================================

  library_metadata = {
    name = "technique_library",
    version = "1.0.0",
    description = "ISA compliance and CEL expression validation for techniques",
    provides = ["StepKinds", "CELValidation", "ISAValidation", "TechniqueContract"],
    dependencies = [],
    nickel_principle = "composable_data"
  }
}