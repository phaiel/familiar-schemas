# Global Orthological Governance Module
# Foundation for all Familiar schema validation and transformation.
# Establishes orthological principles: Entity/Node/Topology separation.

# Import merged library bundle for extreme & composition
let Libraries = import "libraries/all_libraries.ncl" in

# Global governance through extreme & merging of libraries
Libraries & {
  # Orthological metadata merged with library bundle
  orthology = {
    level = "global",
    description = "Entity/Node/Topology orthological governance for all Familiar schemas",
    entity_node_topology = {
      entity = "data_purity",      # What gets processed
      node = "compute_purity",     # Where processing happens
      topology = "wiring_purity"   # How they connect
    }
  }
}