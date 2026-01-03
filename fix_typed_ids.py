#!/usr/bin/env python3
"""
Update database schemas to use typed ID references instead of raw UUIDs.
"""

import json
from pathlib import Path

SCHEMA_BASE = Path("versions/v1.1.0-alpha/json-schema/database")

# Map of schema -> field -> (ref_path, nullable)
UPDATES = {
    "MessageModel.schema.json": {
        "sender_id": ("../primitives/UserId.schema.json", True),
        "parent_id": ("../primitives/MessageId.schema.json", True),
    },
    "ChannelModel.schema.json": {
        "owner_id": ("../primitives/UserId.schema.json", True),
    },
    "UserModel.schema.json": {
        "primary_tenant_id": ("../primitives/TenantId.schema.json", True),
    },
    "FamiliarEntityModel.schema.json": {
        "id": ("../primitives/EntityId.schema.json", False),
        "reviewed_by": ("../primitives/UserId.schema.json", True),
        "source_channel_id": ("../primitives/ChannelId.schema.json", True),
        "source_message_id": ("../primitives/MessageId.schema.json", True),
    },
    "FamilyInvitationModel.schema.json": {
        "invited_by": ("../primitives/UserId.schema.json", True),
    },
    "JoinRequestModel.schema.json": {
        "reviewed_by": ("../primitives/UserId.schema.json", True),
    },
    "AuditLogEntryModel.schema.json": {
        "actor_id": ("../primitives/UserId.schema.json", True),
    },
}

def make_ref(ref_path: str, nullable: bool) -> dict:
    """Create a $ref property, optionally nullable using anyOf."""
    if nullable:
        return {
            "anyOf": [
                {"$ref": ref_path},
                {"type": "null"}
            ]
        }
    else:
        return {"$ref": ref_path}

def update_schema(schema_file: str, field_updates: dict) -> None:
    """Update fields in a schema file."""
    path = SCHEMA_BASE / schema_file
    
    if not path.exists():
        print(f"✗ Not found: {schema_file}")
        return
    
    with open(path, 'r') as f:
        schema = json.load(f)
    
    properties = schema.get("properties", {})
    
    for field_name, (ref_path, nullable) in field_updates.items():
        if field_name in properties:
            old_val = properties[field_name]
            properties[field_name] = make_ref(ref_path, nullable)
            print(f"  {field_name}: uuid -> {ref_path.split('/')[-1].replace('.schema.json', '')}{' (nullable)' if nullable else ''}")
        else:
            print(f"  ✗ Field not found: {field_name}")
    
    with open(path, 'w') as f:
        json.dump(schema, f, indent=2)
        f.write('\n')
    
    print(f"✓ {schema_file}")

def main():
    print("Updating database schemas to use typed ID references...\n")
    
    for schema_file, field_updates in UPDATES.items():
        update_schema(schema_file, field_updates)
        print()
    
    print("Done!")

if __name__ == "__main__":
    import os
    os.chdir(Path(__file__).parent)
    main()

