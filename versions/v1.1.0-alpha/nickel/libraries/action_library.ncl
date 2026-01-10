# Action Library - Basic validation for atomic action signatures
# Following Nickel "composable data" principles with validation contracts

{
  # ============================================================================
  # ACTION VALIDATION
  # ============================================================================

  ActionValidation = {
    # Validate action signature structure
    validate_signature = fun signature =>
      std.is_record signature &&
      std.record.has_field "inputs" signature &&
      std.record.has_field "output" signature &&
      # Inputs should be a record of parameter definitions
      std.is_record signature.inputs &&
      # Output should be a single schema reference or object
      std.is_record signature.output
    ,

    # Validate action capabilities (if present)
    validate_capabilities = fun capabilities =>
      if std.is_record capabilities then
        std.record.has_field "category" capabilities &&
        std.record.has_field "provides" capabilities &&
        std.is_string capabilities.category &&
        std.is_array capabilities.provides
      else
        true  # Capabilities are optional
    ,

    # Validate complete action schema
    validate_action = fun action_json =>
      std.is_record action_json &&
      std.record.has_field "signature" action_json &&
      ActionValidation.validate_signature action_json.signature &&
      (if std.record.has_field "capabilities" action_json.signature then
        ActionValidation.validate_capabilities action_json.signature.capabilities
       else
        true) &&
      # Basic structure validation
      std.record.has_field "id" action_json &&
      std.record.has_field "x-familiar-kind" action_json &&
      action_json.x-familiar-kind == "action"
  },

  # ============================================================================
  # CAPABILITY QUERYING
  # ============================================================================

  CapabilityQueries = {
    # Check if action provides specific capability
    has_capability = fun action capabilities_needed =>
      if std.record.has_field "capabilities" action.signature &&
         std.record.has_field "provides" action.signature.capabilities then
        let provided_caps = action.signature.capabilities.provides in
        std.array.all (fun needed => std.array.elem needed provided_caps) capabilities_needed
      else
        false
    ,

    # Get actions by category
    by_category = fun actions category =>
      std.array.filter (fun action =>
        std.record.has_field "capabilities" action.signature &&
        action.signature.capabilities.category == category
      ) actions
    ,

    # Find actions that provide required capabilities
    find_compatible_actions = fun actions required_capabilities =>
      std.array.filter (fun action =>
        CapabilityQueries.has_capability action required_capabilities
      ) actions
  },

  # ============================================================================
  # LIBRARY METADATA
  # ============================================================================

  library_metadata = {
    name = "action_library",
    version = "1.0.0",
    description = "Basic validation and capability querying for atomic actions",
    provides = ["ActionValidation", "CapabilityQueries"],
    dependencies = [],
    nickel_principle = "composable_data"
  }
}