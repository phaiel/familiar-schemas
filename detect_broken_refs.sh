#!/bin/bash

# Detect broken $ref references in JSON schemas
# This is critical for maintaining schema integrity

set -e

echo "🔍 Detecting broken $ref references..."
echo "====================================="

# Create temp file for results
TEMP_FILE=$(mktemp)
BROKEN_COUNT=0

# Find all JSON files with $ref
while IFS= read -r json_file; do
    # Extract all $ref values from this file
    refs=$(grep -o '"$ref":\s*"[^"]*"' "$json_file" | sed 's/"$ref":\s*"\([^"]*\)"/\1/')

    for ref in $refs; do
        # Skip external refs (http/https)
        if [[ $ref == http* ]]; then
            continue
        fi

        # Convert relative ref to absolute path from the json_file's directory
        json_dir=$(dirname "$json_file")
        abs_ref_path="$json_dir/$ref"

        # Normalize the path (resolve .. and .)
        abs_ref_path=$(cd "$(dirname "$abs_ref_path")" 2>/dev/null && pwd)/$(basename "$abs_ref_path" 2>/dev/null)

        # Check if the referenced file exists
        if [[ ! -f "$abs_ref_path" ]]; then
            echo "❌ BROKEN REF: $json_file" >> "$TEMP_FILE"
            echo "   $ref → $abs_ref_path (NOT FOUND)" >> "$TEMP_FILE"
            ((BROKEN_COUNT++))
        fi
    done
done < <(find versions/latest/json-schema -name "*.json")

# Report results
if [[ $BROKEN_COUNT -eq 0 ]]; then
    echo "✅ No broken references found!"
else
    echo "❌ Found $BROKEN_COUNT broken references:"
    echo ""
    cat "$TEMP_FILE"
fi

# Cleanup
rm -f "$TEMP_FILE"

echo ""
echo "====================================="
echo "Note: This checked the first 50 JSON files for performance."
echo "Run with full scan: find versions/latest/json-schema -name \"*.json\" | wc -l"
echo "Total JSON files: $(find versions/latest/json-schema -name "*.json" | wc -l)"
