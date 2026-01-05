#!/bin/bash

# Script to detect and fix broken $ref paths in JSON schemas
# Run this after major reorganizations to repair references

set -e

echo "🔍 Detecting broken references..."

# Find all $ref values in JSON files
BROKEN_REFS=$(find versions/latest/json-schema -name "*.json" -exec grep -l '"\$ref"' {} \; | xargs grep '"\$ref"' | grep -v '"\$ref": "\.\./' | wc -l)

echo "Found $BROKEN_REFS potential reference issues"

# Common fixes for known reorganizations
echo "🔧 Applying known fixes..."

# Fix corrupted double-replaced paths (domai../domains/ → domains/)
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../domai../domains/|../domains/|g' {} \;

# Fix domain references (agentic, auth, conversation, windmill moved to domains/)
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../agentic/|../domains/agentic/|g' {} \;
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../auth/|../domains/auth/|g' {} \;
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../conversation/|../domains/conversation/|g' {} \;
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../windmill/|../domains/windmill/|g' {} \;

# Fix architecture references (ecs/ moved to architecture/)
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../ecs/|../architecture/meta/|g' {} \;

# Fix infrastructure references (nodes, systems, queues moved to infrastructure/)
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../nodes/|../infrastructure/nodes/|g' {} \;
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../systems/|../infrastructure/systems/|g' {} \;
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../queues/|../infrastructure/queues/|g' {} \;

# Fix meta references
find versions/latest/json-schema -name "*.json" -exec sed -i '' 's|../meta/PortSemantics|../architecture/enums/PortSemantics|g' {} \;

echo "✅ Reference fixes applied"
echo "🔍 Running validation..."

# Run validation
if cargo run --release --bin schema-validator -- checksums latest >/dev/null 2>&1; then
    echo "✅ All references valid!"
else
    echo "❌ Some references still broken. Check the output above."
    exit 1
fi
