#!/bin/bash
# Generate TypeScript types from JSON Schema
#
# Usage:
#   ./scripts/generate-typescript.sh                              # Generate from latest version
#   ./scripts/generate-typescript.sh --version v0.7.0             # Generate from specific version
#   ./scripts/generate-typescript.sh --output ../docs/v4/familiar-core/bindings  # Custom output
#
# Requirements:
#   npm install -g json-schema-to-typescript
#
# This script converts JSON Schema files from familiar-schemas into TypeScript
# interfaces, enabling schema-first development across the codebase.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REGISTRY_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION="latest"
OUTPUT_DIR="${REGISTRY_ROOT}/generated/typescript"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  -v, --version VERSION   Schema version to use (default: latest)"
            echo "  -o, --output DIR        Output directory (default: generated/typescript)"
            echo "  -h, --help              Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Resolve version symlink if "latest"
if [ "$VERSION" = "latest" ]; then
    if [ -L "${REGISTRY_ROOT}/versions/latest" ]; then
        VERSION=$(readlink "${REGISTRY_ROOT}/versions/latest" | xargs basename)
    else
        # Find the highest version
        VERSION=$(ls -1 "${REGISTRY_ROOT}/versions" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1)
    fi
fi

SCHEMA_DIR="${REGISTRY_ROOT}/versions/${VERSION}/json-schema"

echo "╔════════════════════════════════════════════════╗"
echo "║   JSON Schema → TypeScript Generator           ║"
echo "╚════════════════════════════════════════════════╝"
echo ""
echo "📂 Source:  ${SCHEMA_DIR}"
echo "📦 Version: ${VERSION}"
echo "📁 Output:  ${OUTPUT_DIR}"
echo ""

# Validate schema directory
if [ ! -d "$SCHEMA_DIR" ]; then
    echo "❌ Schema directory not found: $SCHEMA_DIR"
    exit 1
fi

# Check if json-schema-to-typescript is installed
if ! command -v json2ts &> /dev/null; then
    echo "❌ json-schema-to-typescript not found."
    echo ""
    echo "Install with:"
    echo "  npm install -g json-schema-to-typescript"
    echo ""
    echo "Or use npx:"
    echo "  npx json-schema-to-typescript"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate an index file
INDEX_FILE="${OUTPUT_DIR}/index.ts"
echo "// Generated TypeScript types from JSON Schema" > "$INDEX_FILE"
echo "// Source: familiar-schemas/${VERSION}" >> "$INDEX_FILE"
echo "// Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$INDEX_FILE"
echo "//" >> "$INDEX_FILE"
echo "// DO NOT EDIT - Regenerate with: ./scripts/generate-typescript.sh" >> "$INDEX_FILE"
echo "" >> "$INDEX_FILE"

TOTAL_SCHEMAS=0
GENERATED=0

# Walk through all JSON schema directories
for category_dir in "$SCHEMA_DIR"/*; do
    if [ -d "$category_dir" ]; then
        category=$(basename "$category_dir")
        
        # Skip manifest.json if it's a file
        if [ ! -d "$category_dir" ]; then
            continue
        fi
        
        echo "📁 Processing: $category/"
        
        # Create category subdirectory
        mkdir -p "${OUTPUT_DIR}/${category}"
        
        # Process each schema file in the category
        for schema_file in "$category_dir"/*.schema.json "$category_dir"/*.json; do
            if [ -f "$schema_file" ]; then
                TOTAL_SCHEMAS=$((TOTAL_SCHEMAS + 1))
                
                filename=$(basename "$schema_file")
                # Remove .schema.json or .json extension
                typename="${filename%.schema.json}"
                typename="${typename%.json}"
                
                output_file="${OUTPUT_DIR}/${category}/${typename}.ts"
                
                # Generate TypeScript
                if json2ts "$schema_file" -o "$output_file" --bannerComment "" 2>/dev/null; then
                    GENERATED=$((GENERATED + 1))
                    echo "   ✓ ${category}/${typename}.ts"
                    
                    # Add to index
                    echo "export * from './${category}/${typename}';" >> "$INDEX_FILE"
                else
                    echo "   ⚠ Failed: ${category}/${typename}"
                fi
            fi
        done
    fi
done

echo ""
echo "════════════════════════════════════════════════"
echo "✅ TypeScript generation complete!"
echo ""
echo "   Schemas processed: ${TOTAL_SCHEMAS}"
echo "   Types generated:   ${GENERATED}"
echo "   Output:            ${OUTPUT_DIR}"
echo ""
echo "Import types in your code:"
echo "   import { YourType } from './generated/typescript';"
echo ""

