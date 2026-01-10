# NLP Techniques Subdirectory - Specialized technique validation
let ParentTechniques = import "../_directory.ncl" in
let Composer = import "../../../composers/directory_composer.ncl" in

# Inherit from parent techniques directory and specialize for NLP
ParentTechniques & {
  # NLP-specific technique validation
  technique_validation = ParentTechniques.technique_validation & {
    # Validate NLP techniques (language processing workflows)
    validate_nlp_technique = fun technique_json =>
      # Basic infrastructure technique validation
      ParentTechniques.technique_validation.validate_infrastructure_technique technique_json &&

      # NLP-specific constraints
      std.record.has_field "x-familiar-nlp-model" technique_json &&

      # Validate NLP-appropriate step kinds
      let nlp_allowed_kinds = [ParentTechniques.StepKinds.call, ParentTechniques.StepKinds.map, ParentTechniques.StepKinds.transform] in
      std.array.all (fun step =>
        std.array.elem step.kind nlp_allowed_kinds
      ) technique_json.steps
    ,

    # Validate NLP-specific data flow (text processing pipeline)
    validate_nlp_data_flow = fun technique_json =>
      ParentTechniques.technique_validation.validate_infrastructure_data_flow technique_json &&

      # NLP-specific data flow constraints
      # e.g., ensure text flows through tokenization -> processing -> generation
      true
  }
} & Composer.infrastructure.Contract.compose {
  extract_from_raw = Composer.infrastructure.extraction,
  validate_pure = Composer.infrastructure.validation,
  get_layer = "node"
}
