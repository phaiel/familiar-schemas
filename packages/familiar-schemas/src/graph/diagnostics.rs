//! Diagnostics
//!
//! Collects warnings and errors during analysis passes.
//! Enables fail-fast on ambiguous cases with good error messages.

use serde::{Deserialize, Serialize};
use std::fmt;

use super::analysis::BoxedEdge;
use super::SchemaId;

// =============================================================================
// Diagnostic Codes
// =============================================================================

/// Diagnostic code for categorizing issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticCode {
    // === Pattern Detection ===
    /// oneOf has mixed const/object variants
    AmbiguousOneOf,
    /// Enum variants would have same name after PascalCase conversion
    EnumVariantConflict,
    
    // === Composition ===
    /// x-familiar-composition annotation doesn't match actual fields
    CompositionMismatch,
    /// Alias of alias chain (A -> B -> C)
    AliasOfAlias,
    /// $ref target not found in graph
    UnresolvedRef,
    
    // === Primitives ===
    /// Schema has same shape as primitive but different ID
    ShapeMismatchPrimitive,
    /// Duplicate definition of a primitive
    DuplicatePrimitive,
    
    // === SCC/Boxing ===
    /// Boxed edge computed outside of SCC (indicates bug)
    BoxedEdgeNotInScc,
    /// Self-referential type without boxing
    UnboxedSelfRef,
    /// SCC members have inconsistent emit strategies
    SccEmitMismatch,
    
    // === General ===
    /// Schema is missing required x-familiar-kind
    MissingKind,
    /// Unknown or unhandled schema pattern
    UnknownPattern,
    /// Type name collision
    TypeNameCollision,
}

impl DiagnosticCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AmbiguousOneOf => "E001",
            Self::EnumVariantConflict => "E002",
            Self::CompositionMismatch => "E003",
            Self::AliasOfAlias => "W001",
            Self::UnresolvedRef => "E004",
            Self::ShapeMismatchPrimitive => "W002",
            Self::DuplicatePrimitive => "E005",
            Self::BoxedEdgeNotInScc => "E006",
            Self::UnboxedSelfRef => "E007",
            Self::SccEmitMismatch => "E008",
            Self::MissingKind => "W003",
            Self::UnknownPattern => "W004",
            Self::TypeNameCollision => "E009",
        }
    }
    
    pub fn severity(&self) -> Severity {
        match self {
            Self::AmbiguousOneOf
            | Self::EnumVariantConflict
            | Self::CompositionMismatch
            | Self::UnresolvedRef
            | Self::DuplicatePrimitive
            | Self::BoxedEdgeNotInScc
            | Self::UnboxedSelfRef
            | Self::SccEmitMismatch
            | Self::TypeNameCollision => Severity::Error,
            
            Self::AliasOfAlias
            | Self::ShapeMismatchPrimitive
            | Self::MissingKind
            | Self::UnknownPattern => Severity::Warning,
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// Severity
// =============================================================================

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

// =============================================================================
// Diagnostic Item
// =============================================================================

/// A single diagnostic item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    /// Schema that caused this diagnostic
    pub schema_id: SchemaId,
    /// Diagnostic code
    pub code: DiagnosticCode,
    /// Human-readable message
    pub message: String,
    /// Additional context (e.g., related schemas, field paths)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<String>,
}

impl DiagnosticItem {
    pub fn new(schema_id: impl Into<SchemaId>, code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            schema_id: schema_id.into(),
            code,
            message: message.into(),
            context: Vec::new(),
        }
    }
    
    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }
    
    pub fn severity(&self) -> Severity {
        self.code.severity()
    }
}

impl fmt::Display for DiagnosticItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} ({})",
            self.code,
            self.code.severity(),
            self.message,
            self.schema_id
        )?;
        
        for ctx in &self.context {
            write!(f, "\n  - {}", ctx)?;
        }
        
        Ok(())
    }
}

// =============================================================================
// Diagnostics Collection
// =============================================================================

/// Collection of diagnostics from analysis passes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Diagnostics {
    items: Vec<DiagnosticItem>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a diagnostic item
    pub fn push(&mut self, item: DiagnosticItem) {
        self.items.push(item);
    }
    
    /// Add an error
    pub fn error(
        &mut self,
        schema_id: impl Into<SchemaId>,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) {
        self.push(DiagnosticItem::new(schema_id, code, message));
    }
    
    /// Add a warning
    pub fn warning(
        &mut self,
        schema_id: impl Into<SchemaId>,
        code: DiagnosticCode,
        message: impl Into<String>,
    ) {
        self.push(DiagnosticItem::new(schema_id, code, message));
    }
    
    /// Add diagnostic for boxed edge not in SCC
    pub fn boxed_edge_not_in_scc(&mut self, edge: &BoxedEdge, reason: &str) {
        self.push(
            DiagnosticItem::new(
                &edge.from_schema,
                DiagnosticCode::BoxedEdgeNotInScc,
                format!(
                    "Boxed edge from '{}' to '{}' is not within its SCC",
                    edge.from_schema, edge.to_schema
                ),
            )
            .with_context(reason.to_string())
            .with_context(format!("SCC ID: {}", edge.scc_id)),
        );
    }
    
    /// Add diagnostic for unresolved ref
    pub fn unresolved_ref(&mut self, schema_id: impl Into<SchemaId>, ref_target: &str) {
        self.push(DiagnosticItem::new(
            schema_id,
            DiagnosticCode::UnresolvedRef,
            format!("$ref target '{}' not found in schema graph", ref_target),
        ));
    }
    
    /// Add diagnostic for alias chain
    pub fn alias_chain(
        &mut self,
        schema_id: impl Into<SchemaId>,
        chain: &[SchemaId],
    ) {
        self.push(
            DiagnosticItem::new(
                schema_id,
                DiagnosticCode::AliasOfAlias,
                format!("Alias chain detected: {} -> ...", chain.first().unwrap_or(&"?".to_string())),
            )
            .with_context(format!("Chain: {}", chain.join(" -> "))),
        );
    }
    
    /// Add diagnostic for enum variant conflict
    pub fn enum_variant_conflict(
        &mut self,
        schema_id: impl Into<SchemaId>,
        original_values: &[&str],
        conflicting_name: &str,
    ) {
        self.push(
            DiagnosticItem::new(
                schema_id,
                DiagnosticCode::EnumVariantConflict,
                format!(
                    "Enum variants would collide after PascalCase conversion: '{}'",
                    conflicting_name
                ),
            )
            .with_context(format!("Original values: {:?}", original_values)),
        );
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|i| i.severity() == Severity::Error)
    }
    
    /// Get all errors
    pub fn errors(&self) -> impl Iterator<Item = &DiagnosticItem> {
        self.items.iter().filter(|i| i.severity() == Severity::Error)
    }
    
    /// Get all warnings
    pub fn warnings(&self) -> impl Iterator<Item = &DiagnosticItem> {
        self.items.iter().filter(|i| i.severity() == Severity::Warning)
    }
    
    /// Get all items
    pub fn all(&self) -> &[DiagnosticItem] {
        &self.items
    }
    
    /// Get total count
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    /// Count errors
    pub fn error_count(&self) -> usize {
        self.errors().count()
    }
    
    /// Count warnings
    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }
    
    /// Merge another Diagnostics into this one
    pub fn merge(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }
    
    /// Format all diagnostics for display
    pub fn format_all(&self) -> String {
        let mut output = String::new();
        
        for item in &self.items {
            output.push_str(&format!("{}\n", item));
        }
        
        if self.has_errors() {
            output.push_str(&format!(
                "\n{} error(s), {} warning(s)\n",
                self.error_count(),
                self.warning_count()
            ));
        } else if !self.is_empty() {
            output.push_str(&format!("\n{} warning(s)\n", self.warning_count()));
        }
        
        output
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_all())
    }
}

impl IntoIterator for Diagnostics {
    type Item = DiagnosticItem;
    type IntoIter = std::vec::IntoIter<DiagnosticItem>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a DiagnosticItem;
    type IntoIter = std::slice::Iter<'a, DiagnosticItem>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_diagnostic_severity() {
        assert_eq!(DiagnosticCode::AmbiguousOneOf.severity(), Severity::Error);
        assert_eq!(DiagnosticCode::AliasOfAlias.severity(), Severity::Warning);
    }
    
    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        diags.error("schema1", DiagnosticCode::UnresolvedRef, "ref not found");
        diags.warning("schema2", DiagnosticCode::MissingKind, "no x-familiar-kind");
        
        assert_eq!(diags.error_count(), 1);
        assert_eq!(diags.warning_count(), 1);
        assert!(diags.has_errors());
    }
}

