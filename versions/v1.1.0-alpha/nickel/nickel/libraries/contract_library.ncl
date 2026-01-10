# Contract Library - Merged composition of contract primitives

let ContractPrimitives = import "../primitives/contract_primitives.ncl" in

# Merge contract primitives into complete contract library
ContractPrimitives.InterfacePrimitives & ContractPrimitives.FileStructurePrimitives & {
  # Add library-specific enhancements

  # Enhanced test execution using merged primitives
  enhanced_test_runner = fun contract test_cases =>
    ContractPrimitives.TestPrimitives.run_tests contract {} test_cases & {
      enhanced = true,
      library_version = "1.0"
    }
}