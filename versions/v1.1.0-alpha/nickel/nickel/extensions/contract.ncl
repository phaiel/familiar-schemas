/**
 * x-familiar-contract Extension
 *
 * Defines interface contracts for graph nodes.
 * Provides type-safe interface definitions for slotmap+petgraph nodes.
 *
 * @category interfaces
 * @graph-value high - enables type-safe graph operations
 * @validation optional - for service interfaces
 */

{
  extension = {
    name = "x-familiar-contract",
    description = "Interface contract definitions",
    category = "interfaces",
    required = false,
    version = "1.0.0"
  },

  nickel_type = {
    inputs: Record,
    outputs: Record,
    side_effects: Array<String>
  },

  contract = std.contract.custom (fun label value =>
    if std.is_record value &&
       std.record.has_field "inputs" value &&
       std.record.has_field "outputs" value then value
    else std.contract.blame_with label "Contract must have inputs and outputs: ${std.to_string value}"
  ),

  functions = {
    validate_contract_compatibility = fun contract_a contract_b =>
      # Check if contracts are compatible for composition
      let inputs_match = deep_equal contract_a.inputs contract_b.outputs in
      let outputs_match = deep_equal contract_a.outputs contract_b.inputs in
      inputs_match || outputs_match
  },

  graph_integration = {
    node_metadata = fun contract => {
      interface_contract = contract,
      input_types = std.record.keys contract.inputs,
      output_types = std.record.keys contract.outputs,
      contract_hash = hash_contract contract
    },

    graph_constraints = fun node graph_context => {
      # Validate contract compatibility with connected nodes
      let connected_contracts = get_connected_contracts node graph_context in
      let compatibility_errors = validate_contract_chain node."x-familiar-contract" connected_contracts in
      {
        valid = std.array.is_empty compatibility_errors,
        errors = compatibility_errors,
        warnings = []
      }
    }
  },

  codegen = {
    rust = {
      traits = ["Contract", "Callable"]
    },
    typescript = {
      interfaces = ["Contract", "TypeSafe"]
    }
  }
}
