//! Schema Loading
//!
//! Loads schemas from filesystem or embedded directory, extracts metadata,
//! builds the dependency graph, and computes SCCs.

use include_dir::Dir;
use petgraph::algo::kosaraju_scc;
use petgraph::graph::DiGraph;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{
    CodegenMeta, EdgeKind, FieldRef, SchemaGraph, SchemaId, SchemaNode,
};

/// Configuration for schema loading
#[derive(Debug, Clone)]
pub struct LoadConfig {
    /// Skip schemas matching these path prefixes
    pub skip_prefixes: Vec<String>,
    /// Only load schemas matching these path prefixes
    pub include_prefixes: Vec<String>,
}

impl Default for LoadConfig {
    fn default() -> Self {
        Self {
            skip_prefixes: vec![
                "target/".to_string(),           // Rust build artifacts
                ".git/".to_string(),             // Git repository
                "node_modules/".to_string(),     // Node.js dependencies
                ".cargo/".to_string(),           // Cargo metadata
                "artifacts/".to_string(),        // Generated artifacts
            ],
            include_prefixes: Vec::new(),
        }
    }
}

/// Load schemas from a filesystem directory
pub fn load_from_directory(schema_dir: &Path, config: &LoadConfig) -> anyhow::Result<SchemaGraph> {
    // Pre-count files for capacity estimation
    let file_count = WalkDir::new(schema_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
        .count();
    
    let schema_count = file_count.max(100);
    
    let mut graph = DiGraph::with_capacity(schema_count, schema_count * 3);
    let mut schemas = HashMap::with_capacity(schema_count);
    let mut by_path = HashMap::with_capacity(schema_count);
    let mut by_name: HashMap<String, Vec<SchemaId>> = HashMap::with_capacity(schema_count);
    let mut raw_schemas = HashMap::with_capacity(schema_count);
    let mut node_indices = HashMap::with_capacity(schema_count);
    let mut hasher = Sha256::new();
    let mut pending_refs: Vec<(SchemaId, String, EdgeKind)> = Vec::with_capacity(schema_count * 3);
    
    for entry in WalkDir::new(schema_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }
        
        let relative_path = path.strip_prefix(schema_dir)?.to_path_buf();
        let relative_str = relative_path.to_string_lossy();
        
        // Apply include/skip filters
        if !config.include_prefixes.is_empty() {
            if !config.include_prefixes.iter().any(|p| relative_str.starts_with(p)) {
                continue;
            }
        }
        if config.skip_prefixes.iter().any(|p| relative_str.starts_with(p)) {
            continue;
        }
        
        let content = fs::read_to_string(path)?;
        hasher.update(content.as_bytes());

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON in {}: {}", path.display(), e))?;
        
        let (id, node) = extract_node(&json, &relative_path);
        
        // Collect refs
        collect_refs(&json, &id, &relative_path, &mut pending_refs);
        
        // Add node to graph
        let node_idx = graph.add_node(id.clone());
        
        let mut node = node;
        node.node_idx = Some(node_idx);
        
        // Index by path
        by_path.insert(relative_path, id.clone());
        
        // Index by name
        let name = node.title.clone().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .replace(".schema", "")
        });
        by_name.entry(name).or_default().push(id.clone());
        
        node_indices.insert(id.clone(), node_idx);
        schemas.insert(id.clone(), node);
        raw_schemas.insert(id, json);
    }
    
    // Create edges
    for (from_id, ref_target, edge_kind) in pending_refs {
        let to_id = resolve_ref(&ref_target, &schemas, &by_path);
        
        if let (Some(&from_idx), Some(to_id)) = (node_indices.get(&from_id), to_id) {
            if let Some(&to_idx) = node_indices.get(to_id) {
                graph.add_edge(from_idx, to_idx, edge_kind);
            }
        }
    }
    
    let bundle_hash = format!("{:x}", hasher.finalize());
    
    // Compute SCCs
    let scc_indices = kosaraju_scc(&graph);
    let scc_groups: Vec<Vec<SchemaId>> = scc_indices
        .into_iter()
        .filter(|scc| scc.len() > 1)
        .map(|scc| {
            scc.into_iter()
                .filter_map(|idx| graph.node_weight(idx).cloned())
                .collect()
        })
        .collect();
    
    Ok(SchemaGraph {
        graph,
        schemas,
        by_path,
        by_name,
        raw_schemas,
        node_indices,
        bundle_hash,
        scc_groups,
        artifacts: HashMap::new(),
        schema_to_artifacts: HashMap::new(),
        artifact_to_schema: HashMap::new(),
        file_to_artifacts: HashMap::new(),
        lang_to_artifacts: HashMap::new(),
    })
}

/// Load schemas from embedded directory (compiled via include_dir!)
pub fn load_from_embedded(
    embedded_dir: &'static Dir<'static>,
    config: &LoadConfig,
) -> anyhow::Result<SchemaGraph> {
    let mut files_to_process: Vec<(&Path, &str)> = Vec::with_capacity(512);
    collect_embedded_files(embedded_dir, &mut files_to_process);
    
    let schema_count = files_to_process.len();
    
    let mut graph = DiGraph::with_capacity(schema_count, schema_count * 3);
    let mut schemas = HashMap::with_capacity(schema_count);
    let mut by_path = HashMap::with_capacity(schema_count);
    let mut by_name: HashMap<String, Vec<SchemaId>> = HashMap::with_capacity(schema_count);
    let mut raw_schemas = HashMap::with_capacity(schema_count);
    let mut node_indices = HashMap::with_capacity(schema_count);
    let mut hasher = Sha256::new();
    let mut pending_refs: Vec<(SchemaId, String, EdgeKind)> = Vec::with_capacity(schema_count * 3);
    
    for (path, content) in files_to_process {
        let relative_str = path.to_string_lossy();
        
        // Apply filters
        if !config.include_prefixes.is_empty() {
            if !config.include_prefixes.iter().any(|p| relative_str.starts_with(p)) {
                continue;
            }
        }
        if config.skip_prefixes.iter().any(|p| relative_str.starts_with(p)) {
            continue;
        }
        
        hasher.update(content.as_bytes());
        
        let json: serde_json::Value = match serde_json::from_str(content) {
            Ok(j) => j,
            Err(_) => continue,
        };
        
        let relative_path = path.to_path_buf();
        let (id, node) = extract_node(&json, &relative_path);
        
        collect_refs(&json, &id, &relative_path, &mut pending_refs);
        
        let node_idx = graph.add_node(id.clone());
        let mut node = node;
        node.node_idx = Some(node_idx);
        
        by_path.insert(relative_path, id.clone());
        
        let name = node.title.clone().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .replace(".schema", "")
        });
        by_name.entry(name).or_default().push(id.clone());
        
        node_indices.insert(id.clone(), node_idx);
        schemas.insert(id.clone(), node);
        raw_schemas.insert(id, json);
    }
    
    for (from_id, ref_target, edge_kind) in pending_refs {
        let to_id = resolve_ref(&ref_target, &schemas, &by_path);
        
        if let (Some(&from_idx), Some(to_id)) = (node_indices.get(&from_id), to_id) {
            if let Some(&to_idx) = node_indices.get(to_id) {
                graph.add_edge(from_idx, to_idx, edge_kind);
            }
        }
    }
    
    let bundle_hash = format!("{:x}", hasher.finalize());
    
    let scc_indices = kosaraju_scc(&graph);
    let scc_groups: Vec<Vec<SchemaId>> = scc_indices
        .into_iter()
        .filter(|scc| scc.len() > 1)
        .map(|scc| {
            scc.into_iter()
                .filter_map(|idx| graph.node_weight(idx).cloned())
                .collect()
        })
        .collect();
    
    Ok(SchemaGraph {
        graph,
        schemas,
        by_path,
        by_name,
        raw_schemas,
        node_indices,
        bundle_hash,
        scc_groups,
        artifacts: HashMap::new(),
        schema_to_artifacts: HashMap::new(),
        artifact_to_schema: HashMap::new(),
        file_to_artifacts: HashMap::new(),
        lang_to_artifacts: HashMap::new(),
    })
}

/// Recursively collect JSON files from embedded directory
fn collect_embedded_files<'a>(dir: &'a Dir<'static>, files: &mut Vec<(&'a Path, &'a str)>) {
    for file in dir.files() {
        let path = file.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Some(content) = file.contents_utf8() {
                files.push((path, content));
            }
        }
    }
    
    for subdir in dir.dirs() {
        collect_embedded_files(subdir, files);
    }
}

/// Extract SchemaNode from JSON
fn extract_node(json: &serde_json::Value, relative_path: &Path) -> (SchemaId, SchemaNode) {
    let id = json.get("$id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| relative_path.to_string_lossy().to_string());
    
    let title = json.get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let kind = json.get("x-familiar-kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let service = json.get("x-familiar-service")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let codegen = extract_codegen(json);
    let fields = extract_fields(json, relative_path);
    
    let node = SchemaNode {
        id: id.clone(),
        path: relative_path.to_path_buf(),
        title,
        kind,
        service,
        fields,
        codegen,
        node_idx: None,
    };
    
    (id, node)
}

/// Extract codegen metadata from x-familiar-* extensions
fn extract_codegen(json: &serde_json::Value) -> Option<CodegenMeta> {
    let mut meta = CodegenMeta::default();
    let mut has_any = false;
    
    if let Some(v) = json.get("x-familiar-enum-repr").and_then(|v| v.as_str()) {
        meta.enum_repr = Some(v.to_string());
        has_any = true;
    }
    if let Some(v) = json.get("x-familiar-discriminator").and_then(|v| v.as_str()) {
        meta.discriminator = Some(v.to_string());
        has_any = true;
    }
    if let Some(v) = json.get("x-familiar-content").and_then(|v| v.as_str()) {
        meta.content = Some(v.to_string());
        has_any = true;
    }
    if let Some(v) = json.get("x-familiar-casing").and_then(|v| v.as_str()) {
        meta.casing = Some(v.to_string());
        has_any = true;
    }
    if let Some(v) = json.get("x-familiar-flatten").and_then(|v| v.as_bool()) {
        meta.flatten = Some(v);
        has_any = true;
    }
    if let Some(v) = json.get("x-familiar-skip-none").and_then(|v| v.as_bool()) {
        meta.skip_none = Some(v);
        has_any = true;
    }
    if let Some(v) = json.get("x-familiar-newtype").and_then(|v| v.as_bool()) {
        meta.newtype = Some(v);
        has_any = true;
    }
    
    if has_any { Some(meta) } else { None }
}

/// Extract fields from properties
fn extract_fields(json: &serde_json::Value, current_path: &Path) -> Vec<FieldRef> {
    let mut fields = Vec::new();
    
    let required: HashSet<String> = json.get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    
    if let Some(props) = json.get("properties").and_then(|v| v.as_object()) {
        for (name, prop) in props {
            let ty_ref = prop.get("$ref")
                .and_then(|v| v.as_str())
                .map(|s| normalize_ref(s, current_path));
            
            let inline_kind = if ty_ref.is_none() {
                prop.get("type").and_then(|v| v.as_str()).map(String::from)
            } else {
                None
            };
            
            fields.push(FieldRef {
                name: name.clone(),
                ty_ref,
                inline_kind,
                required: Some(required.contains(name)),
            });
        }
    }
    
    fields
}

/// Normalize a $ref to a canonical path
fn normalize_ref(ref_str: &str, current_path: &Path) -> String {
    if ref_str.starts_with('#') {
        return ref_str.to_string();
    }
    
    if ref_str.starts_with("http://") || ref_str.starts_with("https://") {
        return ref_str.to_string();
    }
    
    let parent = current_path.parent().unwrap_or(Path::new(""));
    let resolved = parent.join(ref_str);
    
    let mut components = Vec::new();
    for component in resolved.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::Normal(s) => {
                components.push(s.to_string_lossy().to_string());
            }
            _ => {}
        }
    }
    
    components.join("/")
}

/// Collect all $refs from a schema
fn collect_refs(
    json: &serde_json::Value,
    schema_id: &str,
    current_path: &Path,
    refs: &mut Vec<(SchemaId, String, EdgeKind)>,
) {
    match json {
        serde_json::Value::Object(obj) => {
            // Direct $ref
            if let Some(ref_val) = obj.get("$ref").and_then(|v| v.as_str()) {
                if !ref_val.starts_with('#') {
                    refs.push((
                        schema_id.to_string(),
                        normalize_ref(ref_val, current_path),
                        EdgeKind::Ref,
                    ));
                }
            }
            
            // allOf
            if let Some(arr) = obj.get("allOf").and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                        if !ref_val.starts_with('#') {
                            refs.push((
                                schema_id.to_string(),
                                normalize_ref(ref_val, current_path),
                                EdgeKind::AllOf,
                            ));
                        }
                    }
                    collect_refs(item, schema_id, current_path, refs);
                }
            }
            
            // oneOf
            if let Some(arr) = obj.get("oneOf").and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                        if !ref_val.starts_with('#') {
                            refs.push((
                                schema_id.to_string(),
                                normalize_ref(ref_val, current_path),
                                EdgeKind::OneOf,
                            ));
                        }
                    }
                    collect_refs(item, schema_id, current_path, refs);
                }
            }
            
            // anyOf
            if let Some(arr) = obj.get("anyOf").and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                        if !ref_val.starts_with('#') {
                            refs.push((
                                schema_id.to_string(),
                                normalize_ref(ref_val, current_path),
                                EdgeKind::AnyOf,
                            ));
                        }
                    }
                    collect_refs(item, schema_id, current_path, refs);
                }
            }
            
            // items
            if let Some(items) = obj.get("items") {
                if let Some(ref_val) = items.get("$ref").and_then(|v| v.as_str()) {
                    if !ref_val.starts_with('#') {
                        refs.push((
                            schema_id.to_string(),
                            normalize_ref(ref_val, current_path),
                            EdgeKind::Items,
                        ));
                    }
                }
                collect_refs(items, schema_id, current_path, refs);
            }
            
            // additionalProperties
            if let Some(add_props) = obj.get("additionalProperties") {
                if let Some(ref_val) = add_props.get("$ref").and_then(|v| v.as_str()) {
                    if !ref_val.starts_with('#') {
                        refs.push((
                            schema_id.to_string(),
                            normalize_ref(ref_val, current_path),
                            EdgeKind::AdditionalProperties,
                        ));
                    }
                }
                collect_refs(add_props, schema_id, current_path, refs);
            }
            
            // properties
            if let Some(props) = obj.get("properties").and_then(|v| v.as_object()) {
                for (_name, prop) in props {
                    if let Some(ref_val) = prop.get("$ref").and_then(|v| v.as_str()) {
                        if !ref_val.starts_with('#') {
                            refs.push((
                                schema_id.to_string(),
                                normalize_ref(ref_val, current_path),
                                EdgeKind::Property,
                            ));
                        }
                    }
                    collect_refs(prop, schema_id, current_path, refs);
                }
            }
            
            // Recurse into other values
            for (key, value) in obj {
                if !["$ref", "allOf", "oneOf", "anyOf", "items", "additionalProperties", "properties"].contains(&key.as_str()) {
                    collect_refs(value, schema_id, current_path, refs);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_refs(item, schema_id, current_path, refs);
            }
        }
        _ => {}
    }
}

/// Resolve a ref target to a schema $id
fn resolve_ref<'a>(
    ref_target: &str,
    schemas: &'a HashMap<SchemaId, SchemaNode>,
    by_path: &'a HashMap<PathBuf, SchemaId>,
) -> Option<&'a SchemaId> {
    // Try as direct $id
    if schemas.contains_key(ref_target) {
        return schemas.get(ref_target).map(|n| &n.id);
    }
    
    // Try as path
    let path = PathBuf::from(ref_target);
    if let Some(id) = by_path.get(&path) {
        return Some(id);
    }
    
    // Try normalized path
    let normalized = ref_target.replace(".schema.json", ".schema.json");
    if let Some(id) = by_path.get(&PathBuf::from(&normalized)) {
        return Some(id);
    }
    
    None
}

