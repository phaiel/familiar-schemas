#!/bin/bash
# Generate and register a new schema version
#
# Usage: 
#   ./scripts/generate.sh                    # Auto-detect version from git
#   ./scripts/generate.sh --version 0.2.0    # Explicit version
#   ./scripts/generate.sh --dry-run          # Preview only
#
# This script:
# 1. Collects schemas from familiar-core
# 2. Generates checksums
# 3. Commits to git as an immutable version
# 4. Creates a git tag

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REGISTRY_ROOT="$(dirname "$SCRIPT_DIR")"
FAMILIAR_CORE="${FAMILIAR_CORE:-../docs/v4/familiar-core}"

# Parse arguments
VERSION=""
DRY_RUN=false
AUTHOR="${GIT_AUTHOR_NAME:-$(git config user.name 2>/dev/null || echo 'Schema Registry')}"
MESSAGE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -a|--author)
            AUTHOR="$2"
            shift 2
            ;;
        -m|--message)
            MESSAGE="$2"
            shift 2
            ;;
        -s|--source)
            FAMILIAR_CORE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Auto-detect version if not specified
if [ -z "$VERSION" ]; then
    # Try to get the latest tag and bump patch
    LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    LATEST_VERSION="${LATEST_TAG#v}"
    
    # Parse major.minor.patch
    IFS='.' read -r MAJOR MINOR PATCH <<< "$LATEST_VERSION"
    PATCH=$((PATCH + 1))
    VERSION="${MAJOR}.${MINOR}.${PATCH}"
fi

echo "╔════════════════════════════════════════╗"
echo "║       Schema Registry Generator        ║"
echo "╚════════════════════════════════════════╝"
echo ""
echo "📦 Version: v${VERSION}"
echo "📂 Source:  ${FAMILIAR_CORE}"
echo "👤 Author:  ${AUTHOR}"
echo ""

# Validate source directory
if [ ! -d "$FAMILIAR_CORE" ]; then
    echo "❌ Source directory not found: $FAMILIAR_CORE"
    exit 1
fi

# Check for existing version
VERSION_DIR="${REGISTRY_ROOT}/versions/v${VERSION}"
if [ -d "$VERSION_DIR" ]; then
    echo "❌ Version v${VERSION} already exists!"
    echo "   Schema versions are IMMUTABLE - create a new version instead."
    exit 1
fi

if [ "$DRY_RUN" = true ]; then
    echo "🔍 DRY RUN - No changes will be made"
    echo ""
fi

# Build export command
CMD="cargo run --bin schema-export -- \
    --registry '${REGISTRY_ROOT}' \
    --source '${FAMILIAR_CORE}' \
    --version '${VERSION}' \
    --author '${AUTHOR}'"

if [ -n "$MESSAGE" ]; then
    CMD="$CMD --message '$MESSAGE'"
fi

if [ "$DRY_RUN" = true ]; then
    CMD="$CMD --dry-run"
fi

# Run export
echo "🚀 Running schema export..."
echo ""
cd "$REGISTRY_ROOT"
eval $CMD

if [ "$DRY_RUN" = false ]; then
    echo ""
    echo "✅ Successfully created version v${VERSION}"
    echo ""
    echo "📋 Next steps:"
    echo "   git push origin main --tags"
fi








