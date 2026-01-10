# Contract Primitives - Basic building blocks for contract composition
{
  # Core contract interface primitives
  InterfacePrimitives = {
    extract_from_raw = fun schema => schema, # Base implementation
    validate_pure = fun schema => { valid = true, errors = [], warnings = [] },
    get_layer = "unknown"
  },

  # File structure primitives
  FileStructurePrimitives = {
    contract_file =
      {
        _ : InterfacePrimitives
      }
  },

  # Test execution primitives
  TestPrimitives = {
    run_tests = fun contract file test_cases =>
      # Base test runner - directories override with specific logic
      { executed = true, test_cases = test_cases, contract = contract, file = file }
  }
}
