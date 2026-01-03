# Familiar Schema Registry

A **versioned, append-only** schema registry for managing Rust, AVRO, TypeScript, and Python schemas across the Familiar platform.

## 🏗️ Architecture

```
familiar-schemas/
├── versions/
│   ├── v0.1.0/
│   │   ├── rust/
│   │   │   ├── entities/
│   │   │   ├── primitives/
│   │   │   └── components/
│   │   ├── avro/
│   │   │   ├── CommandEnvelope.avsc
│   │   │   ├── EventEnvelope.avsc
│   │   │   └── TraceEnvelope.avsc
│   │   ├── manifest.json
│   │   └── checksums.sha256
│   ├── v0.2.0/
│   └── latest -> v0.2.0
├── src/               # Rust library & CLI tools
├── scripts/           # Generation scripts
└── Cargo.toml
```

## 🎯 Schema-First Architecture

This registry is the **source of truth** for all types across the Familiar platform:

```
JSON Schema (familiar-schemas)
    │
    ├──▶ TypeScript (generate-typescript.sh)
    │      └── familiar-ui, familiar-api clients
    │
    ├──▶ Pydantic (datamodel-codegen) [future]
    │      └── Windmill scripts
    │
    └──▶ Rust (manual, drift-checked)
           └── familiar-core, familiar-worker
```

**Workflow:**
1. Define types as JSON Schema in `familiar-schemas`
2. Generate TypeScript/Pydantic from the schema
3. Manually maintain Rust types, validated by `schema-drift`

## 🔒 Immutability Guarantees

- **Append-Only**: Once a version is registered, it **cannot be modified**
- **Git-Backed**: All changes are committed and tagged
- **Checksum Verified**: SHA256 checksums ensure data integrity
- **Version Controlled**: Full audit trail of all schema changes

## 🚀 Quick Start

### Initialize the Registry

```bash
# Initialize a new registry
cargo run --bin schema-registry -- init

# Or specify a path
cargo run --bin schema-registry -- init ./my-schemas
```

### Export Schemas from familiar-core

```bash
# Export current schemas as v0.1.0
./scripts/generate.sh --version 0.1.0

# Or use the CLI directly
cargo run --bin schema-export -- \
    --source ../docs/v4/familiar-core \
    --version 0.1.0 \
    --author "Your Name"
```

### List Versions

```bash
cargo run --bin schema-registry -- list
```

### Get a Schema

```bash
# Get latest version
cargo run --bin schema-registry -- get CommandEnvelope

# Get specific version
cargo run --bin schema-registry -- get CommandEnvelope --version v0.1.0
```

### Generate TypeScript Types (Schema-First)

```bash
# Generate TypeScript from latest version
./scripts/generate-typescript.sh

# Generate from specific version
./scripts/generate-typescript.sh --version v0.7.0

# Custom output directory
./scripts/generate-typescript.sh --output ../docs/v4/familiar-ui/types/generated
```

**Requires**: `npm install -g json-schema-to-typescript`

### Check Compatibility

```bash
# Compare two versions
cargo run --bin schema-registry -- diff v0.1.0 v0.2.0

# Check for breaking changes
cargo run --bin schema-validator -- breaking
```

### Verify Checksums

```bash
# Verify a specific version
cargo run --bin schema-validator -- checksums v0.1.0

# Verify all versions
cargo run --bin schema-validator -- checksums all
```

## 📊 CLI Commands

### schema-registry

| Command | Description |
|---------|-------------|
| `init` | Initialize a new registry |
| `list` | List all versions |
| `show <version>` | Show version details |
| `get <name>` | Get a specific schema |
| `diff <old> <new>` | Compare two versions |
| `export <version>` | Export to directory |
| `stats` | Show registry statistics |

### schema-validator

| Command | Description |
|---------|-------------|
| `checksums [version]` | Validate checksums |
| `compatibility` | Check version compatibility |
| `breaking` | Check for breaking changes |
| `report` | Generate compatibility report |

### schema-export

| Option | Description |
|--------|-------------|
| `--source` | Path to familiar-core |
| `--version` | Version to create |
| `--author` | Author name |
| `--dry-run` | Preview without registering |

## 🔧 Rust Library Usage

```rust
use familiar_schemas::{SchemaRegistry, SchemaVersion, Schema, SchemaType};

// Open registry
let mut registry = SchemaRegistry::open("./schemas")?;

// Get latest version
let latest = registry.latest_version();

// Get a schema
if let Some(schema) = registry.get_schema("CommandEnvelope", None) {
    println!("Schema: {:?}", schema);
}

// Check compatibility
let results = registry.check_compatibility("v0.1.0", "v0.2.0")?;
for (name, result) in results {
    if result.is_breaking {
        println!("BREAKING: {}", name);
    }
}
```

## 📦 Schema Types

| Type | Description | Extension |
|------|-------------|-----------|
| `RustEntity` | Entity types from familiar-core | `.json` |
| `RustPrimitive` | Primitive types | `.json` |
| `RustComponent` | Component types | `.json` |
| `RustType` | Other Rust types | `.json` |
| `Avro` | Kafka/Redpanda schemas | `.avsc` |
| `TypeScript` | Generated TS types | `.ts` |
| `Python` | Generated Python models | `.py` |

## 🔄 CI/CD Integration

### GitHub Actions

```yaml
name: Schema Validation
on:
  pull_request:
    paths:
      - 'familiar-core/src/**/*.rs'
      - 'familiar-core/schemas/**/*.avsc'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Check for Breaking Changes
      run: |
        cd familiar-schemas
        cargo run --bin schema-validator -- breaking --from v0.1.0
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

if git diff --name-only | grep -E "(src/.*\.rs|schemas/.*\.avsc)"; then
    echo "Schema changes detected - validating..."
    cd familiar-schemas
    cargo run --bin schema-validator -- breaking
fi
```

## 📋 Versioning Strategy

- **MAJOR**: Breaking changes (field removal, type changes)
- **MINOR**: Backward-compatible additions (new optional fields)
- **PATCH**: Bug fixes, documentation updates

## 🏆 Best Practices

1. **Never modify existing versions** - Always create new versions
2. **Run compatibility checks** before merging PRs
3. **Use semantic versioning** consistently
4. **Document breaking changes** in release notes
5. **Verify checksums** in CI/CD pipelines

## 📄 License

MIT


