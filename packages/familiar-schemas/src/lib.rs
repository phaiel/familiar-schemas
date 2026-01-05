//! Familiar Schema Registry - Pure Library
//!
//! A pure, immutable schema library containing only schema definitions and core types.
//! All runtime processing, code generation, and tooling has been moved to separate crates
//! (familiar-graph, familiar-codegen, xtask) to maintain clean separation of concerns.
//!
//! ## What this crate contains:
//! - Pure schema type definitions
//! - Schema loading and parsing
//! - Version handling
//! - Checksum computation
//! - Error types
//!
//! ## What was moved out:
//! - Graph analysis → `familiar-graph` crate
//! - Code generation → `familiar-codegen` crate
//! - CLI tools → `xtask` in `familiar-architecture`
//! - Configuration management → `familiar-config`
//! - Registry management → Runtime tooling
//! - Compatibility checking → Runtime tooling
//! - Linting → Runtime tooling

pub mod schema;
pub mod version;
pub mod checksum;
pub mod error;

pub use schema::{Schema, SchemaType, SchemaEntry};
pub use version::SchemaVersion;
pub use checksum::Checksum;
pub use error::{SchemaError, Result};

