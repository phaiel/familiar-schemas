#!/bin/bash
# Generate SVG from schema dependency graph using familiar-schemas crate
# Usage: ./generate_svg.sh

set -e

echo "Generating schema dependency graph SVG..."
cargo run --release --bin schema-graph-export -- --format svg --output schemas.svg
echo "Generated schemas.svg ($(du -h schemas.svg | cut -f1))"
