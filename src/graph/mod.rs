//! Schema Dependency Graph
//!
//! Primary data structure using petgraph for $ref/allOf/anyOf/oneOf dependencies.
//! Handles cycles correctly (SCCs). Provides fast lookup via HashMap indexes.
//!
//! This module is shared between:
//! - MCP server (schema introspection tools)
//! - Codegen (type generation)
//!
//! Both consume the same SchemaGraph to ensure consistency.

pub mod loader;
pub mod analysis;
pub mod patterns;
pub mod classify;
pub mod diagnostics;

// Re-export key types from submodules
pub use analysis::{
    BoxedEdge, CycleHandling, FieldPath, FieldPathSegment, SccAnalysis, SccGroup,
    compute_scc_analysis, validate_boxed_edges,
};
pub use patterns::{
    JsonScalarKind, PropertyShape, PropertyTypeShape, SchemaShape, ObjectVariant,
    CodegenExtensions, detect_shape, detect_all_shapes,
};
pub use classify::{
    Classification, Classifier, EmitStrategy, EnumVariant, FieldDef, FieldType,
    TypeKind, UnionVariant, to_pascal_case, to_snake_case,
};
pub use diagnostics::{
    DiagnosticCode, DiagnosticItem, Diagnostics, Severity,
};

use include_dir::Dir;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// --- IO Port Architecture Types ---

/// Action definition with signature-based IO ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub signature: SignatureDef,
}

/// Signature defining input/output contracts with memory semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureDef {
    pub inputs: HashMap<String, InputPortDef>,
    pub output: OutputPortDef,
}

/// Input port definition with memory semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputPortDef {
    pub schema: SchemaRef,
    #[serde(default = "default_borrow_semantics")]
    pub semantics: String, // "borrow", "move", "borrow_mut", "clone"
    #[serde(default)]
    pub optional: bool,
}

/// Output port definition with execution nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPortDef {
    pub schema: SchemaRef,
    #[serde(default = "default_atomic_nature")]
    pub nature: String, // "atomic", "stream", "future"
}

/// Schema reference ($ref)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

// Default value helpers
fn default_borrow_semantics() -> String { "borrow".to_string() }
fn default_atomic_nature() -> String { "atomic".to_string() }

// --- Technique ISA Types ---

/// Technique definition with constrained ISA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechniqueDef {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input: SchemaRef,
    pub output: SchemaRef,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub return_expr: Option<serde_json::Value>,
}

/// Constrained Instruction Set Architecture - exactly 5 step types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Step {
    #[serde(rename = "call")]
    Call(CallStep),
    #[serde(rename = "switch")]
    Switch(SwitchStep),
    #[serde(rename = "map")]
    Map(MapStep),
    #[serde(rename = "parallel")]
    Parallel(ParallelStep),
    #[serde(rename = "transform")]
    Transform(TransformStep),
}

/// Execute a Rust Action (the workhorse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStep {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub action: SchemaRef,
    #[serde(default)]
    pub args: HashMap<String, String>, // CEL expressions
    #[serde(default)]
    pub retry: Option<RetryConfig>,
    #[serde(default)]
    pub timeout: Option<String>,
    #[serde(default)]
    pub visual: Option<VisualCoords>,
}

/// Retry configuration for call steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
}

fn default_max_attempts() -> u32 { 3 }

/// Exclusive branching (the logic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchStep {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub branches: Vec<Branch>,
    #[serde(default)]
    pub default_branch: Vec<Step>,
    #[serde(default)]
    pub visual: Option<VisualCoords>,
}

/// Branch in a switch statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub condition: String, // CEL expression
    pub steps: Vec<Step>,
}

/// Iteration/fan-out (the scaler)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapStep {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub items: String, // CEL expression evaluating to array
    #[serde(default = "default_iterator")]
    pub iterator: String, // Variable name for current item
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    pub steps: Vec<Step>,
    #[serde(default)]
    pub visual: Option<VisualCoords>,
}

fn default_iterator() -> String { "item".to_string() }
fn default_concurrency() -> u32 { 5 }

/// Concurrent execution of different branches (the speed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelStep {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub branches: HashMap<String, Vec<Step>>, // Named branches
    #[serde(default)]
    pub visual: Option<VisualCoords>,
}

/// Pure data reshaping (the glue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformStep {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub output: serde_json::Value, // Object constructed using CEL expressions
    #[serde(default)]
    pub visual: Option<VisualCoords>,
}

/// Visual layout coordinates for flow diagrams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualCoords {
    pub x: f64,
    pub y: f64,
}

// Re-export loader functions
pub use loader::{LoadConfig, load_from_directory, load_from_embedded};

/// Canonical schema identifier (the $id field or generated from path)
pub type SchemaId = String;

/// Unique identifier for a graph node (schema or artifact)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeId {
    /// Schema node: "primitives/TenantId.schema.json"
    Schema(SchemaId),
    /// Artifact node: "rust:TenantId" or "typescript:TenantId"
    Artifact { lang: String, type_name: String },
}

impl NodeId {
    pub fn schema(id: impl Into<String>) -> Self {
        NodeId::Schema(id.into())
    }
    
    pub fn artifact(lang: impl Into<String>, type_name: impl Into<String>) -> Self {
        NodeId::Artifact { lang: lang.into(), type_name: type_name.into() }
    }
    
    pub fn as_schema(&self) -> Option<&str> {
        match self {
            NodeId::Schema(id) => Some(id),
            _ => None,
        }
    }
    
    pub fn as_artifact(&self) -> Option<(&str, &str)> {
        match self {
            NodeId::Artifact { lang, type_name } => Some((lang, type_name)),
            _ => None,
        }
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeId::Schema(id) => write!(f, "schema:{}", id),
            NodeId::Artifact { lang, type_name } => write!(f, "artifact:{}:{}", lang, type_name),
        }
    }
}

/// Types of edges in the schema dependency graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Standard $ref dependency
    Ref,
    /// allOf composition (inheritance/mixin)
    AllOf,
    /// oneOf discriminated union variant
    OneOf,
    /// anyOf union type option  
    AnyOf,
    /// items array element type
    Items,
    /// additionalProperties map value type
    AdditionalProperties,
    /// Property field type
    Property,
    /// Schema generates this artifact (Schema → Artifact edge)
    GeneratesTo,
    /// Artifact depends on schema (Artifact → Schema edge for cross-file deps)
    DependsOn,
}

/// Unique artifact identifier: "lang:type_name" (e.g., "rust:TenantId")
pub type ArtifactId = String;

/// Generated artifact location for a specific language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    /// Unique artifact ID (lang:type_name)
    #[serde(default)]
    pub id: ArtifactId,
    /// Language: "rust", "typescript", "python"
    pub lang: String,
    /// Relative file path from workspace root
    pub file: PathBuf,
    /// Line number where the type definition starts (1-indexed)
    pub line: u32,
    /// Generated type name (may differ from schema title due to naming conventions)
    pub type_name: String,
    /// Type kind: "struct", "enum", "newtype", "type_alias"
    pub type_kind: String,
}

impl GeneratedArtifact {
    /// Create artifact ID from lang and type_name
    pub fn make_id(lang: &str, type_name: &str) -> ArtifactId {
        format!("{}:{}", lang, type_name)
    }
    
    /// Get or generate the artifact ID
    pub fn artifact_id(&self) -> ArtifactId {
        if self.id.is_empty() {
            Self::make_id(&self.lang, &self.type_name)
        } else {
            self.id.clone()
        }
    }
}

/// Codegen metadata extracted from x-familiar-* extensions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodegenMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_repr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flatten: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_none: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newtype: Option<bool>,
    /// Generated artifacts for this schema (populated by codegen)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<GeneratedArtifact>,
}

/// Reference to a field's type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ty_ref: Option<SchemaId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Minimal schema node data (no full schema by default)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaNode {
    /// Canonical $id
    pub id: SchemaId,
    /// File path relative to schema root
    pub path: PathBuf,
    /// Schema title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// x-familiar-kind
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// x-familiar-service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// Field references (name + type ref or inline kind)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldRef>,
    /// Codegen metadata from x-familiar-* extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codegen: Option<CodegenMeta>,
    /// Graph node index for fast lookup
    #[serde(skip)]
    pub node_idx: Option<NodeIndex>,
}

/// Node in a closure result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureNode {
    pub id: SchemaId,
    pub depth: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub scc_boundary: bool,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: SchemaId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub path: PathBuf,
    pub score: i64,
}

/// Lint warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintWarning {
    pub code: String,
    pub message: String,
    pub severity: String,
}

/// The schema dependency graph
pub struct SchemaGraph {
    /// Primary graph structure - DAG with potential cycles
    pub(crate) graph: DiGraph<SchemaId, EdgeKind>,
    
    /// Minimal node data indexed by $id
    pub(crate) schemas: HashMap<SchemaId, SchemaNode>,
    
    /// Index: file path -> $id
    pub(crate) by_path: HashMap<PathBuf, SchemaId>,
    
    /// Index: name (title or filename) -> list of $ids (names can collide!)
    pub(crate) by_name: HashMap<String, Vec<SchemaId>>,
    
    /// Raw JSON stored separately (lazy loaded on request)
    pub(crate) raw_schemas: HashMap<SchemaId, serde_json::Value>,
    
    /// Node index lookup: $id -> NodeIndex
    pub(crate) node_indices: HashMap<SchemaId, NodeIndex>,
    
    /// Bundle hash for caching/determinism
    pub bundle_hash: String,
    
    /// Strongly connected components (circular ref groups)
    pub(crate) scc_groups: Vec<Vec<SchemaId>>,
    
    // ========== Artifact Graph Indexes (O(1) lookups) ==========
    
    /// All artifacts indexed by ID
    pub(crate) artifacts: HashMap<ArtifactId, GeneratedArtifact>,
    
    /// Index: schema_id -> artifact_ids (one schema can generate multiple artifacts: rust, ts, py)
    pub(crate) schema_to_artifacts: HashMap<SchemaId, Vec<ArtifactId>>,
    
    /// Index: artifact_id -> schema_id (reverse lookup)
    pub(crate) artifact_to_schema: HashMap<ArtifactId, SchemaId>,
    
    /// Index: file_path -> artifact_ids (all types in a file)
    pub(crate) file_to_artifacts: HashMap<PathBuf, Vec<ArtifactId>>,
    
    /// Index: lang -> artifact_ids (all artifacts for a language)
    pub(crate) lang_to_artifacts: HashMap<String, Vec<ArtifactId>>,
}

impl SchemaGraph {
    /// Load schemas from a directory
    pub fn from_directory(schema_dir: &Path) -> anyhow::Result<Self> {
        load_from_directory(schema_dir, &LoadConfig::default())
    }
    
    /// Load schemas from embedded directory (compiled into binary via include_dir!)
    pub fn from_embedded(embedded_dir: &'static Dir<'static>) -> anyhow::Result<Self> {
        load_from_embedded(embedded_dir, &LoadConfig::default())
    }
    
    // ========== Public API ==========
    
    /// Get schema count
    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }
    
    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    
    /// Get SCC count (number of circular ref groups)
    pub fn scc_count(&self) -> usize {
        self.scc_groups.len()
    }
    
    /// Get SCC groups
    pub fn scc_groups(&self) -> &[Vec<SchemaId>] {
        &self.scc_groups
    }
    
    /// Resolve a query (name/path/id) to canonical $id
    pub fn resolve(&self, query: &str) -> Option<&SchemaId> {
        // Try as direct $id
        if self.schemas.contains_key(query) {
            return Some(&self.schemas.get(query)?.id);
        }
        
        // Try as path
        let path = PathBuf::from(query);
        if let Some(id) = self.by_path.get(&path) {
            return Some(id);
        }
        
        // Try as name (return first match)
        if let Some(ids) = self.by_name.get(query) {
            return ids.first();
        }
        
        // Try case-insensitive name match
        let query_lower = query.to_lowercase();
        for (name, ids) in &self.by_name {
            if name.to_lowercase() == query_lower {
                return ids.first();
            }
        }
        
        None
    }
    
    /// Get schema node by $id
    pub fn get(&self, id: &str) -> Option<&SchemaNode> {
        self.schemas.get(id)
    }
    
    /// Get raw JSON schema by $id
    pub fn get_raw(&self, id: &str) -> Option<&serde_json::Value> {
        self.raw_schemas.get(id)
    }
    
    /// Get immediate outgoing refs (dependencies)
    pub fn refs_out(&self, id: &str) -> Vec<&SchemaId> {
        let Some(&node_idx) = self.node_indices.get(id) else {
            return Vec::new();
        };
        
        self.graph
            .edges_directed(node_idx, Direction::Outgoing)
            .filter_map(|e| self.graph.node_weight(e.target()))
            .collect()
    }
    
    /// Get immediate incoming refs (dependents)
    pub fn refs_in(&self, id: &str) -> Vec<&SchemaId> {
        let Some(&node_idx) = self.node_indices.get(id) else {
            return Vec::new();
        };
        
        self.graph
            .edges_directed(node_idx, Direction::Incoming)
            .filter_map(|e| self.graph.node_weight(e.source()))
            .collect()
    }
    
    /// Get transitive closure (all deps or dependents)
    pub fn closure(
        &self,
        id: &str,
        direction: Direction,
        max_depth: Option<usize>,
    ) -> Vec<ClosureNode> {
        let Some(&start_idx) = self.node_indices.get(id) else {
            return Vec::new();
        };
        
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![(start_idx, 0usize)];
        
        // Track which SCCs we've seen
        let scc_set: HashSet<&SchemaId> = self.scc_groups
            .iter()
            .flatten()
            .collect();
        
        while let Some((node_idx, depth)) = stack.pop() {
            if let Some(max) = max_depth {
                if depth > max {
                    continue;
                }
            }
            
            if !visited.insert(node_idx) {
                continue;
            }
            
            if node_idx != start_idx {
                if let Some(node_id) = self.graph.node_weight(node_idx) {
                    result.push(ClosureNode {
                        id: node_id.clone(),
                        depth,
                        scc_boundary: scc_set.contains(node_id),
                    });
                }
            }
            
            let edges = self.graph.edges_directed(node_idx, direction);
            for edge in edges {
                let next = match direction {
                    Direction::Outgoing => edge.target(),
                    Direction::Incoming => edge.source(),
                };
                stack.push((next, depth + 1));
            }
        }
        
        // Sort by depth
        result.sort_by_key(|n| n.depth);
        result
    }
    
    /// Search schemas by name (fuzzy)
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;
        
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(i64, &SchemaNode)> = Vec::new();
        
        for node in self.schemas.values() {
            // Match against title
            if let Some(title) = &node.title {
                if let Some(score) = matcher.fuzzy_match(title, query) {
                    results.push((score, node));
                    continue;
                }
            }
            
            // Match against filename
            let filename = node.path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if let Some(score) = matcher.fuzzy_match(filename, query) {
                results.push((score, node));
            }
        }
        
        // Sort by score descending
        results.sort_by(|a, b| b.0.cmp(&a.0));
        
        results
            .into_iter()
            .take(limit)
            .map(|(score, node)| SearchResult {
                id: node.id.clone(),
                title: node.title.clone(),
                kind: node.kind.clone(),
                path: node.path.clone(),
                score,
            })
            .collect()
    }
    
    /// List all schemas by x-familiar-kind
    pub fn list_by_kind(&self, kind: &str) -> Vec<&SchemaId> {
        self.schemas
            .values()
            .filter(|n| n.kind.as_deref() == Some(kind))
            .map(|n| &n.id)
            .collect()
    }
    
    /// Get all unique kinds
    pub fn all_kinds(&self) -> Vec<String> {
        let mut kinds: Vec<String> = self.schemas
            .values()
            .filter_map(|n| n.kind.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        kinds.sort();
        kinds
    }
    
    /// Generate import statements for a schema
    pub fn imports_for(&self, id: &str, lang: &str) -> Vec<String> {
        let deps = self.closure(id, Direction::Outgoing, Some(1));
        let mut imports = Vec::new();
        
        // Helper to get type name from a node
        fn get_type_name(node: &SchemaNode) -> String {
            node.title.clone().unwrap_or_else(|| {
                node.path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .replace(".schema", "")
            })
        }
        
        // Add self
        if let Some(node) = self.get(id) {
            let name = get_type_name(node);
            let dir = node.path.parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");
            
            match lang {
                "rust" => {
                    imports.push(format!("use crate::{}::{};", dir.replace('/', "::"), name));
                }
                "typescript" | "ts" => {
                    imports.push(format!("import {{ {} }} from '@familiar/schemas/{}';", name, dir));
                }
                "python" | "py" => {
                    imports.push(format!("from familiar.schemas.{} import {}", dir.replace('/', "."), name));
                }
                _ => {}
            }
        }
        
        // Add deps
        for dep in deps {
            if let Some(node) = self.get(&dep.id) {
                let name = get_type_name(node);
                let dir = node.path.parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                
                match lang {
                    "rust" => {
                        imports.push(format!("use crate::{}::{};", dir.replace('/', "::"), name));
                    }
                    "typescript" | "ts" => {
                        imports.push(format!("import {{ {} }} from '@familiar/schemas/{}';", name, dir));
                    }
                    "python" | "py" => {
                        imports.push(format!("from familiar.schemas.{} import {}", dir.replace('/', "."), name));
                    }
                    _ => {}
                }
            }
        }
        
        imports.sort();
        imports.dedup();
        imports
    }
    
    /// Lint union schemas for ambiguity issues
    pub fn lint_unions(&self, id: &str) -> Vec<LintWarning> {
        let mut warnings = Vec::new();
        
        let Some(raw) = self.get_raw(id) else {
            return warnings;
        };
        
        // Check for untagged oneOf without discriminator
        if raw.get("oneOf").is_some() {
            let has_discriminator = raw.get("x-familiar-discriminator").is_some();
            let has_repr = raw.get("x-familiar-enum-repr").is_some();
            
            if !has_discriminator && !has_repr {
                warnings.push(LintWarning {
                    code: "UNTAGGED_UNION".to_string(),
                    message: "oneOf without x-familiar-discriminator or x-familiar-enum-repr may cause ambiguous parsing".to_string(),
                    severity: "warning".to_string(),
                });
            }
        }
        
        // Check for anyOf used where allOf might be intended
        if let Some(any_of) = raw.get("anyOf").and_then(|v| v.as_array()) {
            let all_objects = any_of.iter().all(|item| {
                item.get("type").map(|t| t == "object").unwrap_or(false)
                    || item.get("properties").is_some()
            });
            
            if all_objects {
                warnings.push(LintWarning {
                    code: "ANYOF_OBJECTS".to_string(),
                    message: "anyOf with all object types might be better as allOf (composition) or oneOf (union)".to_string(),
                    severity: "info".to_string(),
                });
            }
        }
        
        // Check for missing x-familiar-kind
        if raw.get("x-familiar-kind").is_none() {
            warnings.push(LintWarning {
                code: "MISSING_KIND".to_string(),
                message: "Schema is missing x-familiar-kind extension".to_string(),
                severity: "info".to_string(),
            });
        }
        
        warnings
    }
    
    /// Get all schemas
    pub fn all_schemas(&self) -> impl Iterator<Item = &SchemaNode> {
        self.schemas.values()
    }
    
    /// Get all schema IDs
    pub fn all_ids(&self) -> impl Iterator<Item = &SchemaId> {
        self.schemas.keys()
    }
    
    // ========== Artifact Management ==========
    
    /// Load artifact indexes from JSON files
    pub fn load_artifact_indexes(&mut self, artifacts_dir: &Path) -> anyhow::Result<usize> {
        let mut loaded = 0;
        
        for entry in WalkDir::new(artifacts_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if !path.to_string_lossy().ends_with(".artifacts.json") {
                continue;
            }
            
            let content = fs::read_to_string(path)?;
            let artifacts: Vec<serde_json::Value> = serde_json::from_str(&content)?;
            
            for artifact in artifacts {
                let schema_path = artifact.get("schema_path").and_then(|v| v.as_str());
                let lang = artifact.get("lang").and_then(|v| v.as_str());
                let file = artifact.get("file").and_then(|v| v.as_str());
                let line = artifact.get("line").and_then(|v| v.as_u64()).map(|l| l as u32);
                let type_name = artifact.get("type_name").and_then(|v| v.as_str());
                let type_kind = artifact.get("type_kind").and_then(|v| v.as_str());
                
                if let (Some(schema_path), Some(lang), Some(file), Some(line), Some(type_name), Some(type_kind)) 
                    = (schema_path, lang, file, line, type_name, type_kind) 
                {
                    let schema_id = self.by_path.get(&PathBuf::from(schema_path))
                        .cloned()
                        .or_else(|| self.resolve(schema_path).cloned());
                    
                    if let Some(id) = schema_id {
                        self.register_artifact(&id, GeneratedArtifact {
                            id: GeneratedArtifact::make_id(lang, type_name),
                            lang: lang.to_string(),
                            file: PathBuf::from(file),
                            line,
                            type_name: type_name.to_string(),
                            type_kind: type_kind.to_string(),
                        });
                        loaded += 1;
                    }
                }
            }
        }
        
        Ok(loaded)
    }
    
    /// Register a generated artifact for a schema
    pub fn register_artifact(&mut self, schema_id: &str, mut artifact: GeneratedArtifact) {
        let artifact_id = artifact.artifact_id();
        artifact.id = artifact_id.clone();
        
        self.artifacts.insert(artifact_id.clone(), artifact.clone());
        
        self.schema_to_artifacts
            .entry(schema_id.to_string())
            .or_default()
            .push(artifact_id.clone());
        
        self.artifact_to_schema.insert(artifact_id.clone(), schema_id.to_string());
        
        self.file_to_artifacts
            .entry(artifact.file.clone())
            .or_default()
            .push(artifact_id.clone());
        
        self.lang_to_artifacts
            .entry(artifact.lang.clone())
            .or_default()
            .push(artifact_id.clone());
        
        if let Some(node) = self.schemas.get_mut(schema_id) {
            let codegen = node.codegen.get_or_insert_with(CodegenMeta::default);
            if let Some(existing) = codegen.artifacts.iter_mut().find(|a| a.lang == artifact.lang) {
                *existing = artifact;
            } else {
                codegen.artifacts.push(artifact);
            }
        }
    }
    
    /// Get all artifacts for a schema - O(1)
    pub fn get_artifacts(&self, schema_id: &str) -> Vec<&GeneratedArtifact> {
        self.schema_to_artifacts
            .get(schema_id)
            .map(|ids| ids.iter().filter_map(|id| self.artifacts.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get schema ID for an artifact - O(1)
    pub fn get_artifact_schema(&self, artifact_id: &str) -> Option<&str> {
        self.artifact_to_schema.get(artifact_id).map(|s| s.as_str())
    }
    
    /// Get all artifacts in a file - O(1)
    pub fn get_file_artifacts(&self, file: &Path) -> Vec<&GeneratedArtifact> {
        self.file_to_artifacts
            .get(file)
            .map(|ids| ids.iter().filter_map(|id| self.artifacts.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get all artifacts for a language - O(1)
    pub fn get_lang_artifacts(&self, lang: &str) -> Vec<&GeneratedArtifact> {
        self.lang_to_artifacts
            .get(lang)
            .map(|ids| ids.iter().filter_map(|id| self.artifacts.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Get artifact by ID - O(1)
    pub fn get_artifact_by_id(&self, artifact_id: &str) -> Option<&GeneratedArtifact> {
        self.artifacts.get(artifact_id)
    }
    
    /// Get total artifact count
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }
    
    /// Get artifact for a schema in a specific language - O(1)
    pub fn get_artifact(&self, schema_id: &str, lang: &str) -> Option<&GeneratedArtifact> {
        let artifact_id = GeneratedArtifact::make_id(lang, 
            self.schemas.get(schema_id)
                .and_then(|n| n.title.as_ref())
                .map(|t| t.as_str())
                .unwrap_or_else(|| schema_id.rsplit('/').next().unwrap_or(schema_id).trim_end_matches(".schema.json"))
        );
        self.artifacts.get(&artifact_id)
    }
    
    /// Get all schemas that have generated artifacts for a language
    pub fn schemas_with_artifacts(&self, lang: &str) -> Vec<(&SchemaId, &GeneratedArtifact)> {
        self.lang_to_artifacts
            .get(lang)
            .map(|artifact_ids| {
                artifact_ids.iter()
                    .filter_map(|aid| {
                        let artifact = self.artifacts.get(aid)?;
                        let schema_id = self.artifact_to_schema.get(aid)?;
                        Some((schema_id, artifact))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get coverage stats: how many schemas have artifacts per language
    pub fn artifact_coverage(&self) -> HashMap<String, (usize, usize)> {
        let total = self.schemas.len();
        self.lang_to_artifacts
            .iter()
            .map(|(lang, artifacts)| (lang.clone(), (artifacts.len(), total)))
            .collect()
    }
    
    /// Find all artifacts that would be affected by changing a schema
    pub fn affected_artifacts(&self, schema_id: &str) -> Vec<&GeneratedArtifact> {
        let dependents = self.closure(schema_id, Direction::Incoming, None);
        let mut affected = self.get_artifacts(schema_id);
        for dep in dependents {
            affected.extend(self.get_artifacts(&dep.id));
        }
        affected
    }
    
    /// Find all schemas needed to generate a specific artifact
    pub fn artifact_dependencies(&self, artifact_id: &str) -> Vec<&SchemaNode> {
        let Some(schema_id) = self.artifact_to_schema.get(artifact_id) else {
            return Vec::new();
        };
        
        let deps = self.closure(schema_id, Direction::Outgoing, None);
        let mut result = Vec::new();
        if let Some(node) = self.schemas.get(schema_id) {
            result.push(node);
        }
        for dep in deps {
            if let Some(node) = self.schemas.get(&dep.id) {
                result.push(node);
            }
        }
        result
    }
    
    /// Find co-located artifacts (same file)
    pub fn colocated_artifacts(&self, artifact_id: &str) -> Vec<&GeneratedArtifact> {
        let Some(artifact) = self.artifacts.get(artifact_id) else {
            return Vec::new();
        };

        self.get_file_artifacts(&artifact.file)
            .into_iter()
            .filter(|a| a.artifact_id() != artifact_id)
            .collect()
    }

    /// Export the schema dependency graph to GraphViz DOT format
    pub fn to_dot(&self) -> String {
        let mut output = String::new();

        // Header with styling
        output.push_str("digraph SchemaGraph {\n");
        output.push_str("  rankdir=LR;\n");
        output.push_str("  bgcolor=\"#1e1e1e\";\n");
        output.push_str("  node [shape=box, style=\"filled,rounded\", fontname=\"Helvetica\", fontsize=10, fontcolor=\"white\", color=\"#404040\"];\n");
        output.push_str("  edge [fontname=\"Helvetica\", fontsize=8, fontcolor=\"#808080\"];\n");
        output.push_str("\n");

        // Color mapping for different schema kinds
        let color_map = [
            ("component", "#FF9800"),
            ("entity", "#00BCD4"),
            ("tool", "#9C27B0"),
            ("request", "#F44336"),
            ("response", "#4CAF50"),
            ("enum", "#FF5722"),
            ("primitive", "#607D8B"),
            ("action", "#2196F3"),
            ("technique", "#FF9800"),
            ("resource", "#795548"),
            ("node", "#607D8B"),
            ("system", "#9C27B0"),
        ];

        // Nodes
        for (id, node) in &self.schemas {
            let title = node.title.as_deref().unwrap_or_else(|| {
                // Extract title from path if not available
                node.path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(id.rsplit('/').next().unwrap_or(id))
                    .trim_end_matches(".schema.json")
            });

            let color = color_map.iter()
                .find(|(kind, _)| node.kind.as_deref() == Some(*kind))
                .map(|(_, color)| *color)
                .unwrap_or("#9E9E9E"); // Default gray

            let node_id = id.replace("/", "_").replace(".", "_").replace("-", "_");
            output.push_str(&format!("  \"{}\" [label=\"{}\", fillcolor=\"{}\"];\n", node_id, title, color));
        }

        output.push_str("\n");

        // Edges
        for edge in self.graph.edge_references() {
            let source_idx = edge.source();
            let target_idx = edge.target();

            if let (Some(source_id), Some(target_id)) = (
                self.graph.node_weight(source_idx),
                self.graph.node_weight(target_idx)
            ) {
                let source_node_id = source_id.replace("/", "_").replace(".", "_").replace("-", "_");
                let target_node_id = target_id.replace("/", "_").replace(".", "_").replace("-", "_");

                // Only include edges between schemas we have nodes for
                if self.schemas.contains_key(source_id) && self.schemas.contains_key(target_id) {
                    output.push_str(&format!("  \"{}\" -> \"{}\";\n", source_node_id, target_node_id));
                }
            }
        }

        output.push_str("}\n");
        output
    }
}

