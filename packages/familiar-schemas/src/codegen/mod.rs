//! Code Generation
//!
//! Generates code from schemas using the graph infrastructure.
//! 
//! Architecture:
//! - CodegenConfig: Global settings (naming, diagnostics) - language-agnostic
//! - NameResolver: Schema ID -> canonical name mapping with collision handling
//! - RenderProfile: Per-language rendering settings (type mappings, optionality)
//! - CodegenContext: Immutable after build() - holds all analysis results
//! - Region: Pure projection of pre-computed analysis for a single type
//! - Emitters: Language-specific code generators that consume Regions + RenderProfile
//!
//! Key principle: Classification (SchemaShape, SCC, TypeKind) is config-free.
//! Only emission/rendering uses configuration (RenderProfile).

pub mod cel;
pub mod config;
pub mod infra;
pub mod names;
pub mod rust;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::graph::{
    BoxedEdge, Classification, Classifier, CycleHandling, Diagnostics, EmitStrategy,
    SchemaGraph, SchemaId, SchemaShape, SccAnalysis, TypeKind, FieldType, JsonScalarKind,
    compute_scc_analysis, detect_all_shapes, validate_boxed_edges,
};

pub use cel::{CeleEvaluator, NodeEvaluationContext};
pub use config::{CodegenConfig, RenderProfile, NamingConfig, Language};
pub use infra::{generate_infrastructure, InfraEnvironment, InfraError, InfraGenerator};
pub use names::{NameResolver, ResolvedName, TypeOrigin, NameResolverStats};

// =============================================================================
// Region
// =============================================================================

/// A Region is a pure projection of pre-computed analysis for a single schema.
/// 
/// It contains ONLY what the emitter needs - no raw JSON access.
/// All classification decisions are made BEFORE Region extraction.
/// All type names are ALREADY RESOLVED - no $ref strings.
#[derive(Debug, Clone)]
pub struct Region {
    /// The schema being generated
    pub schema_id: SchemaId,
    
    /// Pre-computed type classification
    pub type_kind: TypeKind,
    
    /// Pre-computed emit strategy
    pub emit_strategy: EmitStrategy,
    
    /// Pre-computed cycle handling
    pub cycle_handling: CycleHandling,
    
    /// Resolved canonical type name (collision-free)
    pub canonical_name: String,
    
    /// Type origin (Primitive, Generated, External, Stdlib)
    pub origin: TypeOrigin,
    
    /// Boxed fields in this schema (keyed by field path)
    pub boxed_fields: Vec<BoxedEdge>,
    
    /// Directory this schema is in (for namespace info)
    pub directory: Option<String>,
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
        // Only generate if origin is Generated and emit strategy allows it
        self.origin == TypeOrigin::Generated && matches!(
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
    
    /// Global codegen configuration
    config: CodegenConfig,
    
    /// Name resolver with collision handling
    name_resolver: NameResolver,
    
    /// Detected shapes for all schemas
    #[allow(dead_code)]
    shapes: HashMap<SchemaId, SchemaShape>,
    
    /// Classifications for all schemas
    classifications: HashMap<SchemaId, Classification>,
    
    /// SCC analysis results
    scc_analysis: SccAnalysis,
    
    /// Diagnostics collected during analysis
    diagnostics: Diagnostics,
}

impl CodegenContext {
    /// Build context from a schema graph with default config.
    /// 
    /// Primitives are automatically detected from directory structure
    /// (any schema in a "primitives/" directory).
    pub fn build(graph: SchemaGraph) -> Result<Self, Diagnostics> {
        Self::build_with_config(graph, CodegenConfig::default())
    }
    
    /// Build context from a schema graph with custom config.
    pub fn build_with_config(graph: SchemaGraph, config: CodegenConfig) -> Result<Self, Diagnostics> {
        let mut diagnostics = Diagnostics::new();
        
        // Phase 1: Build name resolver (detects primitives by directory)
        let name_resolver = NameResolver::build(&graph, config.naming.clone());
        
        // Phase 2: Detect shapes
        let shapes = detect_all_shapes(&graph);
        
        // Phase 3: Compute SCCs and boxing
        let scc_analysis = compute_scc_analysis(&graph);
        
        // Validate boxed edges are within SCCs
        let validation_errors = validate_boxed_edges(&scc_analysis);
        for err in validation_errors {
            diagnostics.boxed_edge_not_in_scc(&err.edge, &err.reason);
        }
        
        // Phase 4: Classify all schemas
        // Build primitives set from name resolver
        let primitives: HashSet<SchemaId> = name_resolver
            .all_resolved()
            .filter(|(_, r)| r.origin == TypeOrigin::Primitive)
            .map(|(id, _)| id.clone())
            .collect();
        
        let classifier = Classifier::new(&graph, &shapes, &scc_analysis, primitives);
        let classifications = classifier.classify_all();
        
        // Check for errors
        if diagnostics.has_errors() {
            return Err(diagnostics);
        }
        
        Ok(Self {
            graph,
            config,
            name_resolver,
            shapes,
            classifications,
            scc_analysis,
            diagnostics,
        })
    }
    
    /// Extract a Region for a schema.
    /// 
    /// Region is a pure projection - no raw JSON access.
    /// Type names are already resolved.
    pub fn region(&self, schema_id: &str) -> Option<Region> {
        let classification = self.classifications.get(schema_id)?;
        let resolved_name = self.name_resolver.get(schema_id)?;
        let cycle_handling = self.scc_analysis.get(schema_id)
            .cloned()
            .unwrap_or_default();
        
        Some(Region {
            schema_id: schema_id.to_string(),
            type_kind: classification.type_kind.clone(),
            emit_strategy: classification.emit_strategy.clone(),
            cycle_handling: cycle_handling.clone(),
            canonical_name: resolved_name.canonical_name.clone(),
            origin: resolved_name.origin.clone(),
            boxed_fields: cycle_handling.boxed_fields,
            directory: resolved_name.directory.clone(),
        })
    }
    
    /// Resolve a field type to a language-specific type string using RenderProfile
    pub fn resolve_field_type(&self, field_type: &FieldType, needs_box: bool, profile: &RenderProfile) -> String {
        match field_type {
            FieldType::SchemaRef(ref_target) => {
                let base_type = if let Some(resolved) = self.name_resolver.resolve_ref(ref_target) {
                    resolved.canonical_name.clone()
                } else {
                    // Unknown ref - extract name from path
                    ref_target
                        .rsplit('/')
                        .next()
                        .unwrap_or(ref_target)
                        .trim_end_matches(".schema.json")
                        .trim_end_matches(".json")
                        .to_string()
                };
                
                if needs_box {
                    profile.wrap_box(&base_type)
                } else {
                    base_type
                }
            }
            FieldType::Scalar(scalar) => {
                let scalar_name = match scalar {
                    JsonScalarKind::String => "string",
                    JsonScalarKind::Integer => "integer",
                    JsonScalarKind::Number => "number",
                    JsonScalarKind::Boolean => "boolean",
                    JsonScalarKind::Null => "null",
                };
                profile.scalar_type(scalar_name).to_string()
            }
            FieldType::Array(inner) => {
                let inner_type = self.resolve_field_type(inner, false, profile);
                profile.wrap_array(&inner_type)
            }
            FieldType::FixedArray { items, size } => {
                let inner_type = self.resolve_field_type(items, false, profile);
                profile.wrap_fixed_array(&inner_type, *size)
            }
            FieldType::Tuple(items) => {
                let types: Vec<String> = items
                    .iter()
                    .map(|t| self.resolve_field_type(t, false, profile))
                    .collect();
                profile.wrap_tuple(&types)
            }
            FieldType::Map(value) => {
                let value_type = self.resolve_field_type(value, false, profile);
                profile.wrap_map(&value_type)
            }
            FieldType::InlineObject | FieldType::Unknown => {
                profile.types.any.clone()
            }
        }
    }
    
    /// Get all schema IDs in topological order (dependencies first)
    pub fn topo_order(&self) -> Vec<&SchemaId> {
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
        self.topo_order()
            .into_iter()
            .filter_map(|id| self.region(id))
            .filter(|r| r.should_generate())
            .collect()
    }
    
    /// Get diagnostics
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    
    /// Get the underlying graph
    pub fn graph(&self) -> &SchemaGraph {
        &self.graph
    }
    
    /// Get the name resolver
    pub fn name_resolver(&self) -> &NameResolver {
        &self.name_resolver
    }
    
    /// Get the config
    pub fn config(&self) -> &CodegenConfig {
        &self.config
    }
    
    /// Get schema count
    pub fn schema_count(&self) -> usize {
        self.graph.schema_count()
    }
    
    /// Get SCC count
    pub fn scc_count(&self) -> usize {
        self.scc_analysis.groups.len()
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

/// Generate Rust code from a schema directory with default config.
/// 
/// Primitives are automatically detected from directory structure
/// (any schema in a "primitives/" directory is treated as a primitive).
pub fn generate_rust(schema_dir: &Path) -> Result<GeneratedOutput, Diagnostics> {
    generate_rust_with_config(schema_dir, CodegenConfig::default(), RenderProfile::rust())
}

/// Generate Rust code with custom config and profile.
pub fn generate_rust_with_config(
    schema_dir: &Path, 
    config: CodegenConfig,
    profile: RenderProfile,
) -> Result<GeneratedOutput, Diagnostics> {
    let graph = SchemaGraph::from_directory(schema_dir)
        .map_err(|e| {
            let mut d = Diagnostics::new();
            d.error("", crate::graph::DiagnosticCode::UnknownPattern, e.to_string());
            d
        })?;
    
    let ctx = CodegenContext::build_with_config(graph, config)?;
    
    // Get name resolver stats for header comment
    let stats = ctx.name_resolver().stats();
    
    let regions = ctx.regions_to_generate();
    let mut output = String::new();
    let mut type_count = 0;
    let gen_diags = Diagnostics::new();
    
    // Generate header
    output.push_str("//! Generated from JSON schemas - DO NOT EDIT\n");
    output.push_str("//!\n");
    output.push_str("//! This file is generated by `cargo xtask codegen generate`.\n");
    output.push_str("//! To regenerate, run that command from the workspace root.\n");
    output.push_str("//!\n");
    output.push_str(&format!("//! Stats: {} primitives (skipped), {} generated, {} disambiguated\n\n", 
        stats.primitives, stats.generated, stats.disambiguated));
    
    // Standard imports
    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use schemars::JsonSchema;\n\n");
    
    // Import all primitives that were detected
    let primitive_names: Vec<&str> = ctx.name_resolver().primitives().collect();
    if !primitive_names.is_empty() {
        output.push_str("// Primitives from familiar_primitives (auto-detected from primitives/ directory)\n");
        output.push_str("#[allow(unused_imports)]\n");
        output.push_str("use super::{\n");
        
        // Write primitives in sorted chunks for readability
        let mut sorted_primitives: Vec<&str> = primitive_names;
        sorted_primitives.sort();
        
        for chunk in sorted_primitives.chunks(6) {
            output.push_str("    ");
            output.push_str(&chunk.join(", "));
            output.push_str(",\n");
        }
        
        output.push_str("    // Re-exported from dependencies\n");
        output.push_str("    DateTime, Utc, Uuid,\n");
        output.push_str("};\n\n");
    }
    
    // Generate each type
    for region in regions {
        if let Some(code) = rust::emit_region(&region, &ctx, &profile) {
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

/// Generate TypeScript code (future)
#[allow(dead_code)]
pub fn generate_typescript(schema_dir: &Path) -> Result<GeneratedOutput, Diagnostics> {
    let _profile = RenderProfile::typescript_strict();
    // TODO: Implement TypeScript emitter
    let _ = schema_dir;
    Err({
        let mut d = Diagnostics::new();
        d.error("", crate::graph::DiagnosticCode::UnknownPattern, "TypeScript emitter not yet implemented".to_string());
        d
    })
}

/// Generate Python code (future)
#[allow(dead_code)]
pub fn generate_python(schema_dir: &Path) -> Result<GeneratedOutput, Diagnostics> {
    let _profile = RenderProfile::python_strict();
    // TODO: Implement Python emitter
    let _ = schema_dir;
    Err({
        let mut d = Diagnostics::new();
        d.error("", crate::graph::DiagnosticCode::UnknownPattern, "Python emitter not yet implemented".to_string());
        d
    })
}
