# Extension Merge Primitives - Common merge strategy patterns

{
  # Base merge strategies
  BaseMergeStrategies = {
    override = fun base override => override,

    compose = fun base override =>
      if std.type.of base == "Record && std.type.of override == "Record then
        base & override
      else override,

    validate_and_merge = fun validator => fun base override =>
      if validator override then base & override
      else std.contract.blame_with "merge" "Invalid value in merge: ${std.to_string override}",

    deep_merge = fun base override =>
      if std.type.of base == "Record && std.type.of override == "Record then
        std.record.map (fun key base_value =>
          let override_value = std.record.get_or key override null in
          if override_value != null then
            if std.type.of base_value == "Record && std.type.of override_value == "Record then
              deep_merge base_value override_value
            else override_value
          else base_value
        ) base
      else override
  },

  # Category-specific merge strategies
  CategoryMergeStrategies = {
    # For extensions that should not compose (like edge types)
    singleton_extension = {
      override = BaseMergeStrategies.override,
      compose = BaseMergeStrategies.override,  # No composition
      validate = BaseMergeStrategies.validate_and_merge (fun x => true)
    },

    # For extensions that can compose (like metadata)
    composable_extension = {
      override = BaseMergeStrategies.override,
      compose = BaseMergeStrategies.compose,
      validate = BaseMergeStrategies.validate_and_merge (fun x => std.type.of x == "Record)
    },

    # For extensions with deep merging (like complex configurations)
    deep_merge_extension = {
      override = BaseMergeStrategies.override,
      compose = BaseMergeStrategies.deep_merge,
      validate = BaseMergeStrategies.validate_and_merge (fun x => std.type.of x == "Record)
    }
  },

  # Merge strategy selectors
  StrategySelectors = {
    # Select strategy based on extension category
    by_category = fun category =>
      if category == "architecture" then CategoryMergeStrategies.singleton_extension
      else if category == "entity" then CategoryMergeStrategies.composable_extension
      else CategoryMergeStrategies.deep_merge_extension,

    # Select strategy based on merge behavior
    by_behavior = fun behavior =>
      if behavior == "singleton" then CategoryMergeStrategies.singleton_extension
      else if behavior == "composable" then CategoryMergeStrategies.composable_extension
      else CategoryMergeStrategies.deep_merge_extension
  },

  # Merge helpers
  MergeHelpers = {
    # Safe merge with error handling
    safe_merge = fun base override strategy =>
      try strategy.compose base override
      catch {
        std.contract.blame_with "merge" "Merge failed: ${std.to_string %catch}"
      },

    # Conditional merge
    conditional_merge = fun condition base override strategy =>
      if condition base override then strategy.compose base override
      else base,

    # Merge with validation
    validated_merge = fun validator base override strategy =>
      if validator override then strategy.compose base override
      else std.contract.blame_with "merge" "Validation failed for merge value: ${std.to_string override}"
  }
}