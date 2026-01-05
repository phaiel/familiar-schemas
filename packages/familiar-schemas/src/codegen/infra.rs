//! Infrastructure manifest generation for multi-node deployments
//!
//! This module generates Kubernetes manifests from logical schema definitions:
//! - Reads node schemas with CEL constraints and resource requirements
//! - Reads infrastructure environment configs (scaling, replicas)
//! - Generates K8s Deployments, Services, HPAs, ConfigMaps, etc.
//! - Outputs to manifests/ directory for ArgoCD consumption

use crate::graph::{SchemaGraph, SchemaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Infrastructure environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraEnvironment {
    pub name: String,
    pub nodes: HashMap<String, NodeScaling>,
    pub global: GlobalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeScaling {
    pub min_replicas: usize,
    pub max_replicas: usize,
    pub scaling: Option<ScalingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub metric: String,
    pub target: serde_json::Value,
    pub cooldown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub version_compatibility: String,
    pub default_timeout: String,
}

/// Node configuration extracted from schema
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: String,
    pub constraints: Vec<String>,
    pub resources: ResourceRequirements,
    pub systems: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub storage: Option<String>,
}

/// Infrastructure manifest generator
pub struct InfraGenerator {
    graph: SchemaGraph,
}

impl InfraGenerator {
    /// Create a new infrastructure generator
    pub fn new(graph: SchemaGraph) -> Self {
        Self { graph }
    }
    
    /// Generate infrastructure manifests for an environment
    pub fn generate(&self, env: &InfraEnvironment, output_dir: &Path) -> Result<GeneratedManifests, InfraError> {
        let mut manifests = GeneratedManifests::new();
        
        // Extract node configurations from schemas
        let node_configs = self.extract_node_configs()?;
        
        // Generate manifests for each node
        for (node_id, scaling) in &env.nodes {
            if let Some(node_config) = node_configs.get(node_id) {
                let node_manifests = self.generate_node_manifests(node_config, scaling, env)?;
                manifests.merge(node_manifests);
            }
        }
        
        // Write manifests to output directory
        manifests.write_to_dir(output_dir)?;
        
        Ok(manifests)
    }
    
    /// Extract node configurations from schema graph
    fn extract_node_configs(&self) -> Result<HashMap<String, NodeConfig>, InfraError> {
        let mut configs = HashMap::new();
        
        for schema_id in self.graph.all_ids() {
            if let Some(schema) = self.graph.get_schema(schema_id) {
                if let Some(kind) = schema.get("x-familiar-kind").and_then(|v| v.as_str()) {
                    if kind == "node" {
                        let config = self.extract_node_config(schema_id, schema)?;
                        configs.insert(schema_id.clone(), config);
                    }
                }
            }
        }
        
        Ok(configs)
    }
    
    /// Extract configuration from a single node schema
    fn extract_node_config(&self, schema_id: &str, schema: &serde_json::Value) -> Result<NodeConfig, InfraError> {
        let constraints = if let Some(constraints_obj) = schema.get("constraints") {
            if let Some(obj) = constraints_obj.as_object() {
                obj.values()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        let resources = if let Some(resources_obj) = schema.get("resources") {
            ResourceRequirements {
                cpu: resources_obj.get("cpu").and_then(|v| v.as_str()).map(|s| s.to_string()),
                memory: resources_obj.get("memory").and_then(|v| v.as_str()).map(|s| s.to_string()),
                storage: resources_obj.get("storage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            }
        } else {
            ResourceRequirements {
                cpu: None,
                memory: None,
                storage: None,
            }
        };
        
        let systems = if let Some(systems_arr) = schema.get("systems") {
            if let Some(arr) = systems_arr.as_array() {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        Ok(NodeConfig {
            id: schema_id.to_string(),
            constraints,
            resources,
            systems,
        })
    }
    
    /// Generate Kubernetes manifests for a node
    fn generate_node_manifests(&self, node: &NodeConfig, scaling: &NodeScaling, env: &InfraEnvironment) -> Result<GeneratedManifests, InfraError> {
        let mut manifests = GeneratedManifests::new();
        
        // Generate Deployment
        let deployment = self.generate_deployment(node, scaling, env)?;
        manifests.add("deployment.yaml", deployment);
        
        // Generate Service
        let service = self.generate_service(node)?;
        manifests.add("service.yaml", service);
        
        // Generate HPA if scaling is configured
        if let Some(scaling_config) = &scaling.scaling {
            let hpa = self.generate_hpa(node, scaling_config)?;
            manifests.add("hpa.yaml", hpa);
        }
        
        // Generate ConfigMap for constraints
        let configmap = self.generate_configmap(node)?;
        manifests.add("configmap.yaml", configmap);
        
        Ok(manifests)
    }
    
    /// Generate Kubernetes Deployment manifest
    fn generate_deployment(&self, node: &NodeConfig, scaling: &NodeScaling, env: &InfraEnvironment) -> Result<String, InfraError> {
        let cpu = node.resources.cpu.as_deref().unwrap_or("1000m");
        let memory = node.resources.memory.as_deref().unwrap_or("1Gi");
        
        let manifest = format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {node_id}
  labels:
    app: {node_id}
    environment: {env_name}
spec:
  replicas: {min_replicas}
  selector:
    matchLabels:
      app: {node_id}
  template:
    metadata:
      labels:
        app: {node_id}
        environment: {env_name}
    spec:
      containers:
      - name: {node_id}
        image: familiar/{node_id}:latest
        resources:
          requests:
            cpu: {cpu}
            memory: {memory}
          limits:
            cpu: {cpu}
            memory: {memory}
        env:
        - name: SCHEMA_VERSION
          value: "{version_compat}"
        - name: DEFAULT_TIMEOUT
          value: "{default_timeout}"
        ports:
        - containerPort: 8080
          name: http
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
"#,
            node_id = node.id,
            env_name = env.name,
            min_replicas = scaling.min_replicas,
            cpu = cpu,
            memory = memory,
            version_compat = env.global.version_compatibility,
            default_timeout = env.global.default_timeout
        );
        
        Ok(manifest)
    }
    
    /// Generate Kubernetes Service manifest
    fn generate_service(&self, node: &NodeConfig) -> Result<String, InfraError> {
        let manifest = format!(
            r#"apiVersion: v1
kind: Service
metadata:
  name: {node_id}
  labels:
    app: {node_id}
spec:
  selector:
    app: {node_id}
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  type: ClusterIP
"#,
            node_id = node.id
        );
        
        Ok(manifest)
    }
    
    /// Generate HorizontalPodAutoscaler manifest
    fn generate_hpa(&self, node: &NodeConfig, scaling: &ScalingConfig) -> Result<String, InfraError> {
        let target_value = match &scaling.target {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => scaling.target.to_string(),
        };
        
        let manifest = format!(
            r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {node_id}-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {node_id}
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: {target_value}
  behavior:
    scaleDown:
      stabilizationWindowSeconds: {cooldown_seconds}
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
"#,
            node_id = node.id,
            target_value = target_value,
            cooldown_seconds = 300 // 5 minutes default
        );
        
        Ok(manifest)
    }
    
    /// Generate ConfigMap for node constraints and configuration
    fn generate_configmap(&self, node: &NodeConfig) -> Result<String, InfraError> {
        let constraints_yaml = node.constraints.join("\n");
        
        let manifest = format!(
            r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {node_id}-config
data:
  constraints.txt: |
{constraints_yaml}
  node-id: "{node_id}"
"#,
            node_id = node.id,
            constraints_yaml = constraints_yaml
        );
        
        Ok(manifest)
    }
}

/// Collection of generated manifests
#[derive(Debug)]
pub struct GeneratedManifests {
    manifests: HashMap<String, String>,
}

impl GeneratedManifests {
    fn new() -> Self {
        Self {
            manifests: HashMap::new(),
        }
    }
    
    fn add(&mut self, filename: &str, content: String) {
        self.manifests.insert(filename.to_string(), content);
    }
    
    fn merge(&mut self, other: GeneratedManifests) {
        self.manifests.extend(other.manifests);
    }
    
    fn write_to_dir(&self, dir: &Path) -> Result<(), InfraError> {
        use std::fs;
        
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        
        for (filename, content) in &self.manifests {
            let file_path = dir.join(filename);
            fs::write(file_path, content)?;
        }
        
        Ok(())
    }
}

/// Infrastructure generation errors
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
    
    #[error("Invalid scaling configuration: {0}")]
    InvalidScaling(String),
}

/// Generate infrastructure manifests from schemas and environment config
pub fn generate_infrastructure(
    schema_dir: &Path,
    env_config_path: &Path,
    output_dir: &Path,
) -> Result<(), InfraError> {
    // Load schema graph
    let graph = SchemaGraph::from_directory(schema_dir)?;
    
    // Load environment configuration
    let env_config_content = std::fs::read_to_string(env_config_path)?;
    let env: InfraEnvironment = serde_json::from_str(&env_config_content)?;
    
    // Generate manifests
    let generator = InfraGenerator::new(graph);
    generator.generate(&env, output_dir)?;
    
    Ok(())
}
