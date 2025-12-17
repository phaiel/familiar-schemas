//! Familiar Schema Registry
//!
//! A versioned, append-only schema registry for managing Rust, AVRO, TypeScript,
//! and Python schemas across the Familiar platform.
//!
//! ## Features
//!
//! - **Immutable Storage**: Schemas are stored as Git commits, ensuring full history
//! - **Semantic Versioning**: All schemas follow semver for compatibility tracking
//! - **Checksum Validation**: SHA256 checksums ensure data integrity
//! - **Compatibility Checking**: Automated detection of breaking changes
//! - **Multi-Language Support**: Rust, AVRO, TypeScript, Python schemas
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
//! │   ├── avro/
//! │   │   ├── CommandEnvelope.avsc
//! │   │   ├── EventEnvelope.avsc
//! │   │   └── TraceEnvelope.avsc
//! │   ├── manifest.json
//! │   └── checksums.sha256
//! ├── v0.2.0/
//! └── latest -> v0.2.0
//! ```

pub mod registry;
pub mod schema;
pub mod version;
pub mod compatibility;
pub mod checksum;
pub mod error;

pub use registry::SchemaRegistry;
pub use schema::{Schema, SchemaType, SchemaEntry};
pub use version::SchemaVersion;
pub use compatibility::{CompatibilityChecker, CompatibilityResult};
pub use checksum::Checksum;
pub use error::{SchemaError, Result};

