//! Familiar Schema Registry
//!
//! A versioned, append-only schema registry for managing Rust, Protobuf, AVRO, TypeScript,
//! and Python schemas across the Familiar platform.
//!
//! ## Features
//!
//! - **Immutable Storage**: Schemas are stored as Git commits, ensuring full history
//! - **Semantic Versioning**: All schemas follow semver for compatibility tracking
//! - **Checksum Validation**: SHA256 checksums ensure data integrity
//! - **Compatibility Checking**: Automated detection of breaking changes
//! - **Multi-Language Support**: Rust, AVRO, TypeScript, Python schemas
//! - **Configuration**: Flexible config via files, environment variables
//!
//! ## Architecture
//!
//! ```text
//! versions/
//! ├── v0.1.0/
//! │   ├── rust/
//! │   │   ├── entities/
//! │   │   ├── primitives/
//! │   │   └── components/
//! │   ├── protobuf/
//! │   │   ├── envelope_v1.proto
//! │   │   └── payload.proto
//! │   ├── json-schema/
//! │   │   ├── auth/
//! │   │   ├── tools/
//! │   │   └── agentic/
//! │   ├── manifest.json
//! │   └── checksums.sha256
//! ├── v0.2.0/
//! └── latest -> v0.2.0
//! ```
//!
//! ## Configuration
//!
//! Configuration is loaded from (in order of precedence):
//! 1. Environment variables (SCHEMAS_REGISTRY__PATH, etc.)
//! 2. Command-line specified config file
//! 3. Local schemas.toml or .schemas.toml
//! 4. XDG config directory (~/.config/familiar/schemas/schemas.toml)
//! 5. Built-in defaults
//!
//! Example configuration:
//! ```toml
//! [registry]
//! path = "./familiar-schemas"
//! default_author = "Eric Theiss"
//! immutable = true
//!
//! [export]
//! output_format = "pretty"
//!
//! [workspace]
//! root = "../docs/v4"
//! ```

pub mod config;
pub mod registry;
pub mod schema;
pub mod version;
pub mod compatibility;
pub mod checksum;
pub mod error;
pub mod lint;

pub use config::SchemaConfig;
pub use registry::SchemaRegistry;
pub use schema::{Schema, SchemaType, SchemaEntry};
pub use version::SchemaVersion;
pub use compatibility::{CompatibilityChecker, CompatibilityResult};
pub use checksum::Checksum;
pub use error::{SchemaError, Result};

