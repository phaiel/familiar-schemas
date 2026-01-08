/**
 * x-familiar-codegen-targets Extension
 *
 * Defines code generation targets for graph nodes.
 * Enables multi-language code generation from slotmap+petgraph structure.
 *
 * @category generation
 * @graph-value high - drives code generation pipeline
 * @validation optional - defaults provided
 */

{
  extension = {
    name = "x-familiar-codegen-targets",
    description = "Target languages for code generation",
    category = "generation",
    required = false,
    version = "1.0.0"
  },

  nickel_type = Array<String>,

  contract = std.contract.custom (fun label value =>
    if std.is_array value &&
       std.array.all (fun target =>
         std.array.elem target ["rust", "typescript", "python", "go", "java"]
       ) value then value
    else std.contract.blame_with label "Invalid codegen targets: ${std.to_string value}"
  ),

  defaults = {
    targets = ["rust", "typescript"]
  },

  functions = {
    get_target_capabilities = fun targets =>
      std.array.map (fun target =>
        if target == "rust" then { systems_programming = true, performance = true }
        else if target == "typescript" then { web_development = true, type_safety = true }
        else if target == "python" then { scripting = true, libraries = true }
        else if target == "go" then { concurrency = true, simplicity = true }
        else { general_purpose = true }
      ) targets
  },

  graph_integration = {
    node_metadata = fun targets => {
      codegen_targets = targets,
      target_capabilities = functions.get_target_capabilities targets,
      generation_pipeline = "multi_language"
    }
  },

  codegen = {
    self_referential = true,  # This extension drives its own code generation
    pipeline_integration = true
  }
}
