//! Schema Graph Analysis
//!
//! Computes strongly connected components (SCCs), determines boxing requirements,
//! and provides cycle handling metadata for code generation.

use petgraph::algo::kosaraju_scc;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::{SchemaGraph, SchemaId};

// =============================================================================
// Field Path Segment
// =============================================================================

/// A segment in a field path, precisely identifying where boxing is needed.
/// 
/// More robust than `Vec<String>` because it handles:
/// - Composition/allOf flattening
/// - Array item schemas  
/// - Renamed fields due to casing/keyword escaping
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldPathSegment {
    /// A named field in an object
    Field(String),
    /// An array index (for tuple types or fixed arrays)
    Index(usize),
    /// The value type in a map (additionalProperties)
    MapValue,
    /// An allOf composition reference (index into allOf array)
    AllOf(usize),
    /// A oneOf variant reference (index into oneOf array)
    OneOf(usize),
    /// An anyOf variant reference (index into anyOf array)
    AnyOf(usize),
    /// Array items type
    ArrayItems,
}

impl std::fmt::Display for FieldPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(name) => write!(f, ".{}", name),
            Self::Index(i) => write!(f, "[{}]", i),
            Self::MapValue => write!(f, "[*]"),
            Self::AllOf(i) => write!(f, "<allOf:{}>", i),
            Self::OneOf(i) => write!(f, "<oneOf:{}>", i),
            Self::AnyOf(i) => write!(f, "<anyOf:{}>", i),
            Self::ArrayItems => write!(f, "[]"),
        }
    }
}

/// Full path from schema root to a field that needs boxing
pub type FieldPath = Vec<FieldPathSegment>;

/// Format a field path as a string
pub fn format_field_path(path: &FieldPath) -> String {
    if path.is_empty() {
        return String::from("<root>");
    }
    path.iter().map(|s| s.to_string()).collect::<String>()
}

// =============================================================================
// Boxing Edge
// =============================================================================

/// A field edge that needs boxing to break cycles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoxedEdge {
    /// The schema containing the field that needs boxing
    pub from_schema: SchemaId,
    /// Path from schema root to the field
    pub field_path: FieldPath,
    /// The target schema being referenced (which creates the cycle)
    pub to_schema: SchemaId,
    /// The SCC this edge is part of (for validation)
    pub scc_id: usize,
}

impl BoxedEdge {
    /// Create a composite key for this edge
    pub fn key(&self) -> (SchemaId, FieldPath) {
        (self.from_schema.clone(), self.field_path.clone())
    }
}

// =============================================================================
// SCC Group
// =============================================================================

/// A strongly connected component (cycle group) in the schema graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SccGroup {
    /// Unique identifier for this SCC
    pub id: usize,
    /// All schemas in this SCC
    pub members: Vec<SchemaId>,
    /// Edges within this SCC that need boxing to break cycles
    pub boxed_edges: Vec<BoxedEdge>,
    /// Whether this is a self-referential cycle (single schema refs itself)
    pub is_self_referential: bool,
}

// =============================================================================
// Cycle Handling
// =============================================================================

/// Cycle handling metadata for a single schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CycleHandling {
    /// Which SCC this schema belongs to (None = acyclic)
    pub scc_id: Option<usize>,
    /// Does this type directly reference itself?
    pub is_self_referential: bool,
    /// Specific field paths in THIS schema that need Box<T>
    pub boxed_fields: Vec<BoxedEdge>,
}

impl CycleHandling {
    /// Returns true if this schema is involved in any cycle
    pub fn is_cyclic(&self) -> bool {
        self.scc_id.is_some()
    }
    
    /// Returns true if a specific field path needs boxing
    pub fn needs_boxing(&self, path: &FieldPath) -> bool {
        self.boxed_fields.iter().any(|e| &e.field_path == path)
    }
}

// =============================================================================
// Analysis Result
// =============================================================================

/// Complete SCC analysis result for the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SccAnalysis {
    /// All SCCs in the graph (only cycles with >1 member OR self-refs)
    pub groups: Vec<SccGroup>,
    /// Per-schema cycle handling metadata
    pub cycle_handling: HashMap<SchemaId, CycleHandling>,
    /// Total number of boxed edges
    pub total_boxed_edges: usize,
}

impl SccAnalysis {
    /// Get cycle handling for a schema
    pub fn get(&self, schema_id: &str) -> Option<&CycleHandling> {
        self.cycle_handling.get(schema_id)
    }
    
    /// Check if a schema is in a cycle
    pub fn is_cyclic(&self, schema_id: &str) -> bool {
        self.cycle_handling.get(schema_id)
            .map(|h| h.is_cyclic())
            .unwrap_or(false)
    }
    
    /// Get the SCC group for a schema
    pub fn get_scc(&self, schema_id: &str) -> Option<&SccGroup> {
        let scc_id = self.cycle_handling.get(schema_id)?.scc_id?;
        self.groups.get(scc_id)
    }
}

// =============================================================================
// Analysis Functions
// =============================================================================

/// Compute SCC analysis for a schema graph
pub fn compute_scc_analysis(graph: &SchemaGraph) -> SccAnalysis {
    let scc_indices = kosaraju_scc(&graph.graph);
    
    let mut groups = Vec::new();
    let mut cycle_handling: HashMap<SchemaId, CycleHandling> = HashMap::new();
    let mut total_boxed_edges = 0;
    
    // First pass: identify SCCs
    for scc in scc_indices {
        if scc.len() == 1 {
            // Check for self-referential cycle
            let node_idx = scc[0];
            if let Some(schema_id) = graph.graph.node_weight(node_idx) {
                let has_self_ref = graph.graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .any(|e| e.target() == node_idx);
                
                if has_self_ref {
                    let scc_id = groups.len();
                    
                    // Find the self-referential edges
                    let boxed_edges = find_self_ref_edges(graph, schema_id, scc_id);
                    total_boxed_edges += boxed_edges.len();
                    
                    groups.push(SccGroup {
                        id: scc_id,
                        members: vec![schema_id.clone()],
                        boxed_edges: boxed_edges.clone(),
                        is_self_referential: true,
                    });
                    
                    cycle_handling.insert(schema_id.clone(), CycleHandling {
                        scc_id: Some(scc_id),
                        is_self_referential: true,
                        boxed_fields: boxed_edges,
                    });
                } else {
                    // Acyclic schema
                    cycle_handling.insert(schema_id.clone(), CycleHandling::default());
                }
            }
        } else {
            // Multi-member SCC (mutual recursion)
            let scc_id = groups.len();
            let members: Vec<SchemaId> = scc.iter()
                .filter_map(|idx| graph.graph.node_weight(*idx).cloned())
                .collect();
            
            // Find edges within this SCC that need boxing
            let boxed_edges = find_scc_boxed_edges(graph, &members, scc_id);
            total_boxed_edges += boxed_edges.len();
            
            // Build per-schema boxed fields
            let mut schema_boxed: HashMap<SchemaId, Vec<BoxedEdge>> = HashMap::new();
            for edge in &boxed_edges {
                schema_boxed.entry(edge.from_schema.clone())
                    .or_default()
                    .push(edge.clone());
            }
            
            for member in &members {
                cycle_handling.insert(member.clone(), CycleHandling {
                    scc_id: Some(scc_id),
                    is_self_referential: false,
                    boxed_fields: schema_boxed.remove(member).unwrap_or_default(),
                });
            }
            
            groups.push(SccGroup {
                id: scc_id,
                members,
                boxed_edges,
                is_self_referential: false,
            });
        }
    }
    
    SccAnalysis {
        groups,
        cycle_handling,
        total_boxed_edges,
    }
}

/// Find self-referential edges in a schema
fn find_self_ref_edges(graph: &SchemaGraph, schema_id: &str, scc_id: usize) -> Vec<BoxedEdge> {
    let mut edges = Vec::new();
    
    let Some(raw) = graph.get_raw(schema_id) else {
        return edges;
    };
    
    // Check properties
    if let Some(props) = raw.get("properties").and_then(|v| v.as_object()) {
        for (name, prop) in props {
            if let Some(ref_target) = prop.get("$ref").and_then(|v| v.as_str()) {
                if ref_resolves_to(graph, schema_id, ref_target, schema_id) {
                    edges.push(BoxedEdge {
                        from_schema: schema_id.to_string(),
                        field_path: vec![FieldPathSegment::Field(name.clone())],
                        to_schema: schema_id.to_string(),
                        scc_id,
                    });
                }
            }
            
            // Check items in array properties
            if let Some(items) = prop.get("items") {
                if let Some(ref_target) = items.get("$ref").and_then(|v| v.as_str()) {
                    if ref_resolves_to(graph, schema_id, ref_target, schema_id) {
                        edges.push(BoxedEdge {
                            from_schema: schema_id.to_string(),
                            field_path: vec![
                                FieldPathSegment::Field(name.clone()),
                                FieldPathSegment::ArrayItems,
                            ],
                            to_schema: schema_id.to_string(),
                            scc_id,
                        });
                    }
                }
            }
        }
    }
    
    // Check oneOf/anyOf/allOf
    for (keyword, segment_fn) in [
        ("oneOf", FieldPathSegment::OneOf as fn(usize) -> FieldPathSegment),
        ("anyOf", FieldPathSegment::AnyOf as fn(usize) -> FieldPathSegment),
        ("allOf", FieldPathSegment::AllOf as fn(usize) -> FieldPathSegment),
    ] {
        if let Some(arr) = raw.get(keyword).and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                if let Some(ref_target) = item.get("$ref").and_then(|v| v.as_str()) {
                    if ref_resolves_to(graph, schema_id, ref_target, schema_id) {
                        edges.push(BoxedEdge {
                            from_schema: schema_id.to_string(),
                            field_path: vec![segment_fn(i)],
                            to_schema: schema_id.to_string(),
                            scc_id,
                        });
                    }
                }
            }
        }
    }
    
    edges
}

/// Find edges within an SCC that need boxing
fn find_scc_boxed_edges(graph: &SchemaGraph, members: &[SchemaId], scc_id: usize) -> Vec<BoxedEdge> {
    let member_set: HashSet<&str> = members.iter().map(|s| s.as_str()).collect();
    let mut edges = Vec::new();
    
    for schema_id in members {
        let Some(raw) = graph.get_raw(schema_id) else {
            continue;
        };
        
        // Check properties
        if let Some(props) = raw.get("properties").and_then(|v| v.as_object()) {
            for (name, prop) in props {
                if let Some(ref_target) = prop.get("$ref").and_then(|v| v.as_str()) {
                    if let Some(resolved) = resolve_ref_target(graph, schema_id, ref_target) {
                        if member_set.contains(resolved.as_str()) && resolved != *schema_id {
                            edges.push(BoxedEdge {
                                from_schema: schema_id.clone(),
                                field_path: vec![FieldPathSegment::Field(name.clone())],
                                to_schema: resolved,
                                scc_id,
                            });
                        }
                    }
                }
                
                // Check items
                if let Some(items) = prop.get("items") {
                    if let Some(ref_target) = items.get("$ref").and_then(|v| v.as_str()) {
                        if let Some(resolved) = resolve_ref_target(graph, schema_id, ref_target) {
                            if member_set.contains(resolved.as_str()) {
                                edges.push(BoxedEdge {
                                    from_schema: schema_id.clone(),
                                    field_path: vec![
                                        FieldPathSegment::Field(name.clone()),
                                        FieldPathSegment::ArrayItems,
                                    ],
                                    to_schema: resolved,
                                    scc_id,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Check composition
        for (keyword, segment_fn) in [
            ("oneOf", FieldPathSegment::OneOf as fn(usize) -> FieldPathSegment),
            ("anyOf", FieldPathSegment::AnyOf as fn(usize) -> FieldPathSegment),
            ("allOf", FieldPathSegment::AllOf as fn(usize) -> FieldPathSegment),
        ] {
            if let Some(arr) = raw.get(keyword).and_then(|v| v.as_array()) {
                for (i, item) in arr.iter().enumerate() {
                    if let Some(ref_target) = item.get("$ref").and_then(|v| v.as_str()) {
                        if let Some(resolved) = resolve_ref_target(graph, schema_id, ref_target) {
                            if member_set.contains(resolved.as_str()) {
                                edges.push(BoxedEdge {
                                    from_schema: schema_id.clone(),
                                    field_path: vec![segment_fn(i)],
                                    to_schema: resolved,
                                    scc_id,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Select minimal set of edges to break cycles (simple heuristic: pick one per target)
    // This is a simplification - full cycle breaking would use min-feedback-arc-set
    let mut selected: HashMap<SchemaId, BoxedEdge> = HashMap::new();
    for edge in edges {
        selected.entry(edge.to_schema.clone()).or_insert(edge);
    }
    
    selected.into_values().collect()
}

/// Check if a ref target resolves to a specific schema
fn ref_resolves_to(graph: &SchemaGraph, from_schema: &str, ref_target: &str, expected: &str) -> bool {
    resolve_ref_target(graph, from_schema, ref_target)
        .map(|resolved| resolved == expected)
        .unwrap_or(false)
}

/// Resolve a $ref target to a schema ID
pub fn resolve_ref_target(graph: &SchemaGraph, from_schema: &str, ref_target: &str) -> Option<String> {
    // Handle local refs
    if ref_target.starts_with('#') {
        return None; // Local def, not a schema ref
    }
    
    // Try direct match
    if let Some(id) = graph.resolve(ref_target) {
        return Some(id.clone());
    }
    
    // Try resolving relative to from_schema's path
    if let Some(from_node) = graph.get(from_schema) {
        let parent = from_node.path.parent()?;
        let resolved_path = parent.join(ref_target);
        let normalized = resolved_path.to_string_lossy()
            .replace("\\", "/")
            .split('/')
            .filter(|s| *s != ".")
            .fold(Vec::new(), |mut acc, part| {
                if part == ".." {
                    acc.pop();
                } else {
                    acc.push(part.to_string());
                }
                acc
            })
            .join("/");
        
        if let Some(id) = graph.resolve(&normalized) {
            return Some(id.clone());
        }
    }
    
    None
}

// =============================================================================
// Validation
// =============================================================================

/// Validate that all boxed edges are within SCCs (diagnostic check)
pub fn validate_boxed_edges(analysis: &SccAnalysis) -> Vec<BoxedEdgeValidationError> {
    let mut errors = Vec::new();
    
    for group in &analysis.groups {
        let member_set: HashSet<&str> = group.members.iter().map(|s| s.as_str()).collect();
        
        for edge in &group.boxed_edges {
            // Check from_schema is in SCC
            if !member_set.contains(edge.from_schema.as_str()) {
                errors.push(BoxedEdgeValidationError {
                    edge: edge.clone(),
                    reason: format!(
                        "from_schema '{}' not in SCC {} members: {:?}",
                        edge.from_schema, group.id, group.members
                    ),
                });
            }
            
            // Check to_schema is in SCC (unless self-ref)
            if !group.is_self_referential && !member_set.contains(edge.to_schema.as_str()) {
                errors.push(BoxedEdgeValidationError {
                    edge: edge.clone(),
                    reason: format!(
                        "to_schema '{}' not in SCC {} members: {:?}",
                        edge.to_schema, group.id, group.members
                    ),
                });
            }
        }
    }
    
    errors
}

/// Validation error for a boxed edge
#[derive(Debug, Clone)]
pub struct BoxedEdgeValidationError {
    pub edge: BoxedEdge,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_path_segment_display() {
        assert_eq!(FieldPathSegment::Field("name".into()).to_string(), ".name");
        assert_eq!(FieldPathSegment::Index(0).to_string(), "[0]");
        assert_eq!(FieldPathSegment::MapValue.to_string(), "[*]");
        assert_eq!(FieldPathSegment::AllOf(1).to_string(), "<allOf:1>");
        assert_eq!(FieldPathSegment::ArrayItems.to_string(), "[]");
    }
    
    #[test]
    fn test_format_field_path() {
        let path = vec![
            FieldPathSegment::Field("children".into()),
            FieldPathSegment::ArrayItems,
        ];
        assert_eq!(format_field_path(&path), ".children[]");
    }
}

