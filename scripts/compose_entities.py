#!/usr/bin/env python3
"""
Refactor physics entities to use component composition via $ref.

This script transforms flat entity schemas into composed structures:
- identity: { $ref: Identity.schema.json } - for id, tenant_id, created_at
- physics: { $ref: FieldExcitation.schema.json } - for amplitude, energy, position, velocity, temperature

Usage:
    python scripts/compose_entities.py [--dry-run]
"""

import json
import sys
from pathlib import Path

# Physics entities that need composition
PHYSICS_ENTITIES = [
    "Thread", "Moment", "Intent", "Bond", 
    "Pulse", "Focus", "Filament", "Motif"
]

# Fields that belong to Identity component
IDENTITY_FIELDS = {"id", "tenant_id", "created_at"}

# Fields that belong to FieldExcitation component  
PHYSICS_FIELDS = {"amplitude", "energy", "position", "velocity", "temperature", 
                  "position_workspace", "velocity_workspace"}

def compose_entity(schema_path: Path, dry_run: bool = False) -> dict:
    """Transform a flat entity schema to use component composition."""
    
    with open(schema_path) as f:
        schema = json.load(f)
    
    props = schema.get("properties", {})
    required = set(schema.get("required", []))
    
    # Separate fields into components vs entity-specific
    identity_in_schema = {k: v for k, v in props.items() if k in IDENTITY_FIELDS}
    physics_in_schema = {k: v for k, v in props.items() if k in PHYSICS_FIELDS}
    own_fields = {k: v for k, v in props.items() 
                  if k not in IDENTITY_FIELDS and k not in PHYSICS_FIELDS}
    
    print(f"\n{schema_path.stem}:")
    print(f"  Identity fields found: {list(identity_in_schema.keys())}")
    print(f"  Physics fields found: {list(physics_in_schema.keys())}")
    print(f"  Own fields: {list(own_fields.keys())}")
    
    # Build new properties with composition
    new_props = {}
    new_required = []
    
    # Keep identity fields DIRECT (not composed) - they have typed IDs!
    # Each entity has ThreadId, MomentId, etc. not generic UUID
    for k, v in identity_in_schema.items():
        new_props[k] = v
        if k in required:
            new_required.append(k)
    
    # Add physics component if entity has physics fields
    if physics_in_schema:
        new_props["physics"] = {
            "$ref": "../components/FieldExcitation.schema.json", 
            "description": "Field excitation physics state"
        }
        new_required.append("physics")
    
    # Add entity-specific fields
    for k, v in own_fields.items():
        new_props[k] = v
        if k in required:
            new_required.append(k)
    
    # Update schema
    schema["properties"] = new_props
    schema["required"] = new_required
    
    # Add note about composition
    if "x-familiar-composition" not in schema:
        schema["x-familiar-composition"] = {
            "physics": list(physics_in_schema.keys()),
            "note": "identity fields kept direct (typed IDs)"
        }
    
    if not dry_run:
        with open(schema_path, "w") as f:
            json.dump(schema, f, indent=2)
        print(f"  ✓ Updated {schema_path}")
    else:
        print(f"  [DRY RUN] Would update {schema_path}")
    
    return schema


def main():
    dry_run = "--dry-run" in sys.argv
    
    schema_dir = Path(__file__).parent.parent / "versions" / "latest" / "json-schema" / "entities"
    
    if not schema_dir.exists():
        print(f"Error: Schema directory not found: {schema_dir}")
        sys.exit(1)
    
    print(f"{'[DRY RUN] ' if dry_run else ''}Composing physics entities...")
    print(f"Schema directory: {schema_dir}")
    
    for entity_name in PHYSICS_ENTITIES:
        schema_path = schema_dir / f"{entity_name}.schema.json"
        if schema_path.exists():
            compose_entity(schema_path, dry_run)
        else:
            print(f"\n⚠ {entity_name}: Schema not found at {schema_path}")
    
    print("\n" + "="*60)
    if dry_run:
        print("Dry run complete. Run without --dry-run to apply changes.")
    else:
        print("Done! Now run codegen to regenerate types:")
        print("  cd /path/to/workspace && cargo xtask codegen generate")


if __name__ == "__main__":
    main()

