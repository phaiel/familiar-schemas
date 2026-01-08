/**
 * DEPRECATED: Windmill Directory
 *
 * This directory is deprecated and should not be used for new schemas.
 * 
 * @deprecated This directory contains legacy windmill-related schemas that
 * should be migrated to appropriate domain or infrastructure directories.
 * 
 * All new schemas should use the orthological structure with proper
 * separation of concerns (codegen, infrastructure, architecture, domain).
 */

# Deprecated directory - always fails validation
{
  files = {
    _ : {
      extract_topology_wiring = fun raw_schema => raw_schema & {
        "x-familiar-deprecation-warning" = "Windmill directory is deprecated",
        "x-familiar-migration-required" = true
      },
      validate_pure_topology = fun schema => {
        valid = false,
        errors = ["Windmill directory is deprecated. Migrate schemas to appropriate orthological directories."],
        warnings = []
      },
      get_layer = `deprecated
    }
  },

  # No hydration for deprecated directories
  hydration = {}
}
