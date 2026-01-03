//! Code Generation
//!
//! Generates code from schemas using the graph infrastructure.
//! 
//! Architecture:
//! - CodegenContext: Immutable after build() - holds all analysis results
//! - Region: Pure projection of pre-computed analysis for a single type
//! - Emitters: Language-specific code generators that consume Regions
//!
//! The key constraint: Emitters NEVER read raw schema JSON - only Region fields.

pub mod rust;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::graph::{
    BoxedEdge, Classification, Classifier, CycleHandling, Diagnostics, EmitStrategy,
    SchemaGraph, SchemaId, SchemaShape, SccAnalysis, TypeKind,
    compute_scc_analysis, detect_all_shapes, validate_boxed_edges,
};

// =============================================================================
// Region
// =============================================================================

/// A Region is a pure projection of pre-computed analysis for a single schema.
/// 
/// It contains ONLY what the emitter needs - no raw JSON access.
/// All classification decisions are made BEFORE Region extraction.
#[derive(Debug, Clone)]
pub struct Region {
    /// The schema being generated
    pub schema_id: SchemaId,
    
    /// Immediate dependencies (for import generation)
    pub deps: Vec<SchemaId>,
    
    /// Pre-computed type classification
    pub type_kind: TypeKind,
    
    /// Pre-computed emit strategy
    pub emit_strategy: EmitStrategy,
    
    /// Pre-computed cycle handling
    pub cycle_handling: CycleHandling,
    
    /// Rust type name to generate
    pub rust_name: String,
    
    /// Boxed fields in this schema (keyed by field path)
    pub boxed_fields: Vec<BoxedEdge>,
}

impl Region {
    /// Check if a field needs boxing
    pub fn needs_boxing(&self, field_name: &str) -> bool {
        use crate::graph::FieldPathSegment;
        self.boxed_fields.iter().any(|e| {
            matches!(e.field_path.first(), Some(FieldPathSegment::Field(name)) if name == field_name)
        })
    }
    
    /// Should this region be generated?
    pub fn should_generate(&self) -> bool {
        matches!(
            self.emit_strategy,
            EmitStrategy::Generate | EmitStrategy::GenerateInSccGroup(_)
        )
    }
    
    /// Get SCC group ID if in a cycle
    pub fn scc_group(&self) -> Option<usize> {
        match self.emit_strategy {
            EmitStrategy::GenerateInSccGroup(id) => Some(id),
            _ => self.cycle_handling.scc_id,
        }
    }
}

// =============================================================================
// CodegenContext
// =============================================================================

/// Immutable codegen context - frozen after build().
/// 
/// Contains all analysis results. Emitters access this through Regions.
pub struct CodegenContext {
    /// The underlying schema graph
    graph: SchemaGraph,
    
    /// Detected shapes for all schemas
    shapes: HashMap<SchemaId, SchemaShape>,
    
    /// Classifications for all schemas
    classifications: HashMap<SchemaId, Classification>,
    
    /// SCC analysis results
    scc_analysis: SccAnalysis,
    
    /// Diagnostics collected during analysis
    diagnostics: Diagnostics,
    
    /// Set of primitive schema IDs (in familiar-primitives)
    primitives: HashSet<SchemaId>,
}

impl CodegenContext {
    /// Build context from a schema graph.
    /// 
    /// Returns Err(Diagnostics) if there are errors that prevent codegen.
    pub fn build(graph: SchemaGraph, primitives: HashSet<SchemaId>) -> Result<Self, Diagnostics> {
        let mut diagnostics = Diagnostics::new();
        
        // Phase 1: Detect shapes (can run in parallel with SCC)
        let shapes = detect_all_shapes(&graph);
        
        // Phase 2: Compute SCCs and boxing
        let scc_analysis = compute_scc_analysis(&graph);
        
        // Validate boxed edges are within SCCs
        let validation_errors = validate_boxed_edges(&scc_analysis);
        for err in validation_errors {
            diagnostics.boxed_edge_not_in_scc(&err.edge, &err.reason);
        }
        
        // Phase 3: Classify all schemas
        let classifier = Classifier::new(&graph, &shapes, &scc_analysis, primitives.clone());
        let classifications = classifier.classify_all();
        
        // Check for errors
        if diagnostics.has_errors() {
            return Err(diagnostics);
        }
        
        Ok(Self {
            graph,
            shapes,
            classifications,
            scc_analysis,
            diagnostics,
            primitives,
        })
    }
    
    /// Extract a Region for a schema.
    /// 
    /// Region is a pure projection - no raw JSON access.
    pub fn region(&self, schema_id: &str) -> Option<Region> {
        let classification = self.classifications.get(schema_id)?;
        let cycle_handling = self.scc_analysis.get(schema_id)
            .cloned()
            .unwrap_or_default();
        
        // Get immediate dependencies
        let deps: Vec<SchemaId> = self.graph.refs_out(schema_id)
            .into_iter()
            .cloned()
            .collect();
        
        Some(Region {
            schema_id: schema_id.to_string(),
            deps,
            type_kind: classification.type_kind.clone(),
            emit_strategy: classification.emit_strategy.clone(),
            cycle_handling: cycle_handling.clone(),
            rust_name: classification.rust_name.clone(),
            boxed_fields: cycle_handling.boxed_fields,
        })
    }
    
    /// Get all schema IDs in topological order (dependencies first)
    pub fn topo_order(&self) -> Vec<&SchemaId> {
        // For SCCs, group members together
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        
        for schema_id in self.graph.all_ids() {
            if visited.contains(schema_id) {
                continue;
            }
            
            // Check if part of SCC
            if let Some(scc) = self.scc_analysis.get_scc(schema_id) {
                for member in &scc.members {
                    if visited.insert(member.clone()) {
                        result.push(member);
                    }
                }
            } else {
                visited.insert(schema_id.clone());
                result.push(schema_id);
            }
        }
        
        result
    }
    
    /// Get all regions for schemas that should be generated
    pub fn regions_to_generate(&self) -> Vec<Region> {
        // Track seen names to avoid duplicates
        let mut seen_names: HashSet<String> = HashSet::new();
        
        self.topo_order()
            .into_iter()
            .filter_map(|id| self.region(id))
            .filter(|r| r.should_generate())
            .filter(|r| {
                // Skip types that conflict with Rust standard library
                let conflicts_with_stdlib = matches!(
                    r.rust_name.as_str(),
                    "String" | "Vec" | "Option" | "Result" | "Box" | "Rc" | "Arc" |
                    "HashMap" | "HashSet" | "BTreeMap" | "BTreeSet" | "RefCell" | "Cell" |
                    "Mutex" | "RwLock" | "Debug" | "Clone" | "Default" | "Copy" | "Send" | "Sync"
                );
                if conflicts_with_stdlib {
                    return false; // Skip stdlib conflicts
                }
                
                // Skip duplicate names (keep first occurrence)
                if seen_names.contains(&r.rust_name) {
                    return false;
                }
                seen_names.insert(r.rust_name.clone());
                true
            })
            .collect()
    }
    
    /// Get diagnostics
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    
    /// Get the underlying graph (for import resolution)
    pub fn graph(&self) -> &SchemaGraph {
        &self.graph
    }
    
    /// Get schema count
    pub fn schema_count(&self) -> usize {
        self.graph.schema_count()
    }
    
    /// Get SCC count
    pub fn scc_count(&self) -> usize {
        self.scc_analysis.groups.len()
    }
    
    /// Check if a schema is a primitive
    pub fn is_primitive(&self, schema_id: &str) -> bool {
        self.primitives.contains(schema_id)
    }
}

// =============================================================================
// Generated Output
// =============================================================================

/// Output from code generation
#[derive(Debug, Clone)]
pub struct GeneratedOutput {
    /// Generated code as a string
    pub code: String,
    /// Number of types generated
    pub type_count: usize,
    /// Any warnings during generation
    pub diagnostics: Diagnostics,
}

// =============================================================================
// Public API
// =============================================================================

/// Generate Rust code from a schema directory
pub fn generate_rust(schema_dir: &Path, primitives: HashSet<SchemaId>) -> Result<GeneratedOutput, Diagnostics> {
    let graph = SchemaGraph::from_directory(schema_dir)
        .map_err(|e| {
            let mut d = Diagnostics::new();
            d.error("", crate::graph::DiagnosticCode::UnknownPattern, e.to_string());
            d
        })?;
    
    let ctx = CodegenContext::build(graph, primitives)?;
    
    let regions = ctx.regions_to_generate();
    let mut output = String::new();
    let mut type_count = 0;
    let gen_diags = Diagnostics::new();
    
    // Generate header
    output.push_str("//! Generated from JSON schemas - DO NOT EDIT\n");
    output.push_str("//!\n");
    output.push_str("//! This file is generated by `cargo xtask codegen generate`.\n");
    output.push_str("//! To regenerate, run that command from the workspace root.\n\n");
    
    // Standard imports
    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use schemars::JsonSchema;\n");
    
    // Import primitives from familiar_primitives (re-exported via super in lib.rs)
    // Only import types that actually exist in familiar-primitives
    output.push_str("\n// Primitives from familiar_primitives\n");
    output.push_str("#[allow(unused_imports)]\n");
    output.push_str("use super::{\n");
    output.push_str("    // Validated float types\n");
    output.push_str("    NormalizedFloat, SignedNormalizedFloat, QuantizedCoord,\n");
    output.push_str("    // ID types\n");
    output.push_str("    TenantId, UserId, SessionId, ThreadId, MessageId, ChannelId,\n");
    output.push_str("    CourseId, ShuttleId, EntityId,\n");
    output.push_str("    InvitationId, JoinRequestId, MagicLinkId, AuditLogId,\n");
    output.push_str("    ConsentRecordId, TaskId, ExportRequestId, DeletionRequestId,\n");
    output.push_str("    // Other primitives\n");
    output.push_str("    Email, Temperature, MaxTokens, InviteCode, PasswordHash, SessionToken,\n");
    output.push_str("};\n");
    output.push_str("// Re-export types from dependencies\n");
    output.push_str("pub use super::{DateTime, Utc, Uuid};\n");
    output.push_str("// Timestamp alias for schema compatibility\n");
    output.push_str("pub type Timestamp = DateTime<Utc>;\n");
    output.push_str("pub type UUID = Uuid;\n\n");
    
    // Generate each type
    for region in regions {
        if let Some(code) = rust::emit_region(&region, &ctx) {
            output.push_str(&code);
            output.push_str("\n");
            type_count += 1;
        }
    }
    
    Ok(GeneratedOutput {
        code: output,
        type_count,
        diagnostics: gen_diags,
    })
}

