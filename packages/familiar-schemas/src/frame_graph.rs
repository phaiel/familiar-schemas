//! Frame Graph Builder for HECS-based Schema Compilation
//!
//! Builds transient dependency graphs from schema relationships and performs
//! topological sorting for compilation ordering.

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt;
use crate::edge_inheritance::{EdgeInheritanceResolver, EdgeMetadata};
use crate::SchemaArchitectureError;

/// Entity identifier in the Frame Graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub schema_ref: String,
    pub entity_type: String,
}

impl EntityId {
    pub fn new(schema_ref: String, entity_type: String) -> Self {
        Self {
            schema_ref,
            entity_type,
        }
    }
}

/// Frame Graph for compilation ordering
pub struct FrameGraph {
    graph: DiGraph<EntityId, EdgeMetadata>,
    node_map: HashMap<EntityId, NodeIndex>,
    inheritance_resolver: EdgeInheritanceResolver,
}

impl FrameGraph {
    pub fn new(inheritance_resolver: EdgeInheritanceResolver) -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            inheritance_resolver,
        }
    }

    /// Add an entity to the graph
    pub fn add_entity(&mut self, entity_id: EntityId) -> NodeIndex {
        if let Some(&node_idx) = self.node_map.get(&entity_id) {
            return node_idx;
        }

        let node_idx = self.graph.add_node(entity_id.clone());
        self.node_map.insert(entity_id, node_idx);
        node_idx
    }

    /// Add a typed edge between entities
    pub fn add_edge(&mut self, source: EntityId, target: EntityId, edge_type: &str) -> Result<(), SchemaArchitectureError> {
        // Resolve edge semantics through inheritance
        let dummy_path = std::path::Path::new(&source.schema_ref);
        let metadata = self.inheritance_resolver.resolve_edge_semantics(dummy_path, edge_type)?;

        // Add nodes if they don't exist
        let source_idx = self.add_entity(source);
        let target_idx = self.add_entity(target);

        // Add edge with metadata
        self.graph.add_edge(source_idx, target_idx, metadata);

        Ok(())
    }

    /// Build compilation order using topological sorting
    pub fn build_compilation_order(&self) -> Result<Vec<EntityId>, FrameGraphError> {
        // Perform topological sort
        let sorted_indices = toposort(&self.graph, None)
            .map_err(|cycle: petgraph::algo::Cycle<NodeIndex>| FrameGraphError::CyclicalDependency(cycle.node_id()))?;

        // Convert back to EntityIds
        let mut compilation_order: Vec<EntityId> = Vec::new();
        for node_idx in sorted_indices {
            if let Some(entity_id) = self.graph.node_weight(node_idx) {
                compilation_order.push(entity_id.clone());
            }
        }

        Ok(compilation_order)
    }

    /// Validate graph constraints based on edge metadata
    pub fn validate_constraints(&self) -> Result<(), FrameGraphError> {
        // Check for cycles in acyclic edges
        for edge_idx in self.graph.edge_indices() {
            let (source_idx, target_idx) = self.graph.edge_endpoints(edge_idx)
                .ok_or(FrameGraphError::InvalidEdge(edge_idx))?;

            let metadata = self.graph.edge_weight(edge_idx)
                .ok_or(FrameGraphError::InvalidEdge(edge_idx))?;

            // If edge doesn't allow cycles, check for cycles involving this edge
            if !metadata.traversal_rules.allows_cycles {
                if self.detect_cycle_involving_edge(source_idx, target_idx) {
                    return Err(FrameGraphError::AcyclicEdgeViolation {
                        edge_type: metadata.edge_type.clone(),
                        source: self.graph.node_weight(source_idx).unwrap().schema_ref.clone(),
                        target: self.graph.node_weight(target_idx).unwrap().schema_ref.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Detect if there's a cycle involving a specific edge
    fn detect_cycle_involving_edge(&self, source_idx: NodeIndex, target_idx: NodeIndex) -> bool {
        // Simple cycle detection - check if target can reach source
        // In practice, would use more sophisticated cycle detection
        use petgraph::algo::has_path_connecting;

        has_path_connecting(&self.graph, target_idx, source_idx, None)
    }

    /// Get edges that require topological sorting
    pub fn get_topological_edges(&self) -> Vec<&EdgeMetadata> {
        self.graph.edge_weights()
            .filter(|metadata| metadata.traversal_rules.topological_sort_required)
            .collect()
    }

    /// Analyze graph for optimization opportunities
    pub fn analyze_optimization_opportunities(&self) -> GraphAnalysis {
        let mut analysis = GraphAnalysis::default();

        // Count edges by type
        for metadata in self.graph.edge_weights() {
            *analysis.edge_type_counts.entry(metadata.edge_type.clone()).or_insert(0) += 1;
        }

        // Find parallel edges (same source/target with different edge types)
        let mut parallel_edges = HashMap::new();
        for edge_idx in self.graph.edge_indices() {
            if let Some((source_idx, target_idx)) = self.graph.edge_endpoints(edge_idx) {
                let key = (source_idx, target_idx);
                parallel_edges.entry(key).or_insert_with(Vec::new).push(edge_idx);
            }
        }

        analysis.parallel_relationships = parallel_edges.into_iter()
            .filter(|(_key, edges)| edges.len() > 1)
            .map(|((source_idx, target_idx), edge_indices)| {
                let source = self.graph.node_weight(source_idx).unwrap().schema_ref.clone();
                let target = self.graph.node_weight(target_idx).unwrap().schema_ref.clone();
                let edge_types: Vec<String> = edge_indices.into_iter()
                    .filter_map(|idx| self.graph.edge_weight(idx))
                    .map(|m| m.edge_type.clone())
                    .collect();

                ParallelRelationship { source, target, edge_types }
            })
            .collect();

        // Calculate graph density
        let node_count = self.graph.node_count() as f64;
        let edge_count = self.graph.edge_count() as f64;
        analysis.density = if node_count > 1.0 {
            (2.0 * edge_count) / (node_count * (node_count - 1.0))
        } else {
            0.0
        };

        analysis
    }

    /// Get strongly connected components (for cycle analysis)
    pub fn find_strongly_connected_components(&self) -> Vec<Vec<EntityId>> {
        use petgraph::algo::kosaraju_scc;

        kosaraju_scc(&self.graph).into_iter()
            .filter(|component: &Vec<NodeIndex>| component.len() > 1) // Only non-trivial components
            .map(|component_indices: Vec<NodeIndex>| {
                component_indices.into_iter()
                    .filter_map(|idx| self.graph.node_weight(idx))
                    .cloned()
                    .collect()
            })
            .collect()
    }
}

/// Frame Graph Builder - constructs FrameGraphs from schema collections
pub struct FrameGraphBuilder {
    inheritance_resolver: EdgeInheritanceResolver,
}

impl FrameGraphBuilder {
    pub fn new(inheritance_resolver: EdgeInheritanceResolver) -> Self {
        Self {
            inheritance_resolver,
        }
    }

    /// Build Frame Graph from schema collection
    pub fn build_from_schemas(&self, schemas: &[serde_json::Value]) -> Result<FrameGraph, FrameGraphError> {
        let mut frame_graph = FrameGraph::new(self.inheritance_resolver.clone());

        // Extract edges from all schemas
        for schema in schemas {
            self.extract_edges_from_schema(schema, &mut frame_graph)?;
        }

        // Validate constraints
        frame_graph.validate_constraints()?;

        Ok(frame_graph)
    }

    /// Extract edges from a single schema
    fn extract_edges_from_schema(&self, schema: &serde_json::Value, frame_graph: &mut FrameGraph) -> Result<(), FrameGraphError> {
        let schema_ref = schema.get("$id")
            .and_then(|id| id.as_str())
            .unwrap_or("unknown_schema")
            .to_string();

        let entity_type = schema.get("x-familiar-kind")
            .and_then(|kind| kind.as_str())
            .unwrap_or("unknown")
            .to_string();

        let source_entity = EntityId::new(schema_ref.clone(), entity_type);

        // Extract edges from relationship fields
        let relationship_fields = [
            "x-familiar-depends", "x-familiar-reads", "x-familiar-writes",
            "x-familiar-components", "x-familiar-systems", "x-familiar-resources",
            "x-familiar-service", "x-familiar-queue"
        ];

        for field_name in &relationship_fields {
            if let Some(field_value) = schema.get(field_name) {
                if let Some(relationships) = field_value.as_array() {
                    for relationship in relationships {
                        self.process_relationship(&source_entity, relationship, frame_graph)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single relationship object
    fn process_relationship(&self, source_entity: &EntityId, relationship: &serde_json::Value, frame_graph: &mut FrameGraph) -> Result<(), FrameGraphError> {
        // Check if it's the new typed format
        if let (Some(edge_type), Some(target_obj)) = (
            relationship.get("edge_type").and_then(|et| et.as_str()),
            relationship.get("target")
        ) {
            if let Some(target_ref) = target_obj.get("$ref").and_then(|tr| tr.as_str()) {
                let target_entity = EntityId::new(target_ref.to_string(), "unknown".to_string());
                frame_graph.add_edge(source_entity.clone(), target_entity, edge_type)?;
            }
        }

        Ok(())
    }

    /// Build compilation plan from Frame Graph
    pub fn build_compilation_plan(&self, frame_graph: &FrameGraph) -> Result<CompilationPlan, FrameGraphError> {
        let compilation_order = frame_graph.build_compilation_order()?;
        let analysis = frame_graph.analyze_optimization_opportunities();

        Ok(CompilationPlan {
            entity_order: compilation_order,
            analysis: analysis.clone(),
            optimization_suggestions: self.generate_optimization_suggestions(&analysis),
        })
    }

    /// Generate optimization suggestions based on analysis
    fn generate_optimization_suggestions(&self, analysis: &GraphAnalysis) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest batching for high graph density
        if analysis.density > 0.7 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "high_density".to_string(),
                description: "Graph has high density - consider batching operations".to_string(),
                impact: OptimizationImpact::High,
            });
        }

        // Suggest consolidating parallel relationships
        if !analysis.parallel_relationships.is_empty() {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: "parallel_relationships".to_string(),
                description: format!("Found {} parallel relationships that could be consolidated", analysis.parallel_relationships.len()),
                impact: OptimizationImpact::Medium,
            });
        }

        // Suggest optimizations based on edge type distribution
        for (edge_type, count) in &analysis.edge_type_counts {
            if *count > 10 {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: "frequent_edge_type".to_string(),
                    description: format!("Edge type '{}' used {} times - consider specialized handling", edge_type, count),
                    impact: OptimizationImpact::Low,
                });
            }
        }

        suggestions
    }
}

/// Compilation plan derived from Frame Graph analysis
#[derive(Debug, Clone)]
pub struct CompilationPlan {
    pub entity_order: Vec<EntityId>,
    pub analysis: GraphAnalysis,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

/// Graph analysis results
#[derive(Debug, Clone, Default)]
pub struct GraphAnalysis {
    pub edge_type_counts: HashMap<String, usize>,
    pub parallel_relationships: Vec<ParallelRelationship>,
    pub density: f64,
}

/// Parallel relationship between entities
#[derive(Debug, Clone)]
pub struct ParallelRelationship {
    pub source: String,
    pub target: String,
    pub edge_types: Vec<String>,
}

/// Optimization suggestion
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub impact: OptimizationImpact,
}

#[derive(Debug, Clone)]
pub enum OptimizationImpact {
    Low,
    Medium,
    High,
}

/// Frame Graph errors
#[derive(Debug)]
pub enum FrameGraphError {
    CyclicalDependency(NodeIndex),
    InvalidEdge(petgraph::graph::EdgeIndex),
    AcyclicEdgeViolation {
        edge_type: String,
        source: String,
        target: String,
    },
    SchemaError(SchemaArchitectureError),
    JsonError(serde_json::Error),
}

impl fmt::Display for FrameGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameGraphError::CyclicalDependency(node) => {
                write!(f, "Cyclical dependency detected involving node: {:?}", node)
            }
            FrameGraphError::InvalidEdge(edge) => {
                write!(f, "Invalid edge index: {:?}", edge)
            }
            FrameGraphError::AcyclicEdgeViolation { edge_type, source, target } => {
                write!(f, "Acyclic edge violation: {} between {} and {} creates a cycle", edge_type, source, target)
            }
            FrameGraphError::SchemaError(err) => {
                write!(f, "Schema processing error: {}", err)
            }
            FrameGraphError::JsonError(err) => {
                write!(f, "JSON parsing error: {}", err)
            }
        }
    }
}

impl std::error::Error for FrameGraphError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FrameGraphError::SchemaError(err) => Some(err),
            FrameGraphError::JsonError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<SchemaArchitectureError> for FrameGraphError {
    fn from(err: SchemaArchitectureError) -> Self {
        FrameGraphError::SchemaError(err)
    }
}

impl From<serde_json::Error> for FrameGraphError {
    fn from(err: serde_json::Error) -> Self {
        FrameGraphError::JsonError(err)
    }
}

impl Default for OptimizationImpact {
    fn default() -> Self {
        OptimizationImpact::Low
    }
}