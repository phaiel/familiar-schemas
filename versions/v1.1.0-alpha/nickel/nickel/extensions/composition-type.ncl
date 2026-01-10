/**
 * x-familiar-composition-type Extension
 *
 * Defines composition patterns for graph assembly.
 * Enables composition-aware graph construction in slotmap+petgraph.
 *
 * @category architecture
 * @graph-value medium - guides graph composition
 * @validation optional - for composition definitions
 */

{
  extension = {
    name = "x-familiar-composition-type",
    description = "System composition patterns",
    category = "architecture",
    required = false,
    version = "1.0.0"
  },

  nickel_type = [| `system_assembly, `service_composition, `module_aggregation |],

  contract = std.contract.custom (fun label value =>
    let valid_types = ["system_assembly", "service_composition", "module_aggregation"] in
    if std.array.elem value valid_types then value
    else std.contract.blame_with label "Invalid composition type: ${std.to_string value}"
  ),

  functions = {
    get_composition_strategy = fun composition_type =>
      if composition_type == "system_assembly" then {
        assembly_strategy = "hierarchical", dependency_resolution = "topological"
      } else if composition_type == "service_composition" then {
        assembly_strategy = "contract_based", dependency_resolution = "interface_matching"
      } else {
        assembly_strategy = "modular", dependency_resolution = "capability_matching"
      }
  },

  graph_integration = {
    node_metadata = fun composition_type => {
      composition_type = composition_type,
      composition_strategy = functions.get_composition_strategy composition_type,
      assembly_pattern = get_assembly_pattern composition_type
    }
  }
}
