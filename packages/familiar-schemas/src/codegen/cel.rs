//! CEL (Common Expression Language) evaluation for multi-node constraints and routing
//!
//! This module provides CEL evaluation capabilities for:
//! - Node constraint evaluation (memory, cpu, health checks)
//! - System routing policy evaluation
//! - Runtime decision making based on schema-defined expressions

use cel_interpreter::{Context, Program, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CEL evaluation context for multi-node architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvaluationContext {
    /// Available memory in bytes
    pub available_memory: u64,
    
    /// Current CPU usage (0.0 to 1.0)
    pub cpu_usage: f64,
    
    /// Current queue depth
    pub queue_depth: usize,
    
    /// Number of active jobs
    pub active_jobs: usize,
    
    /// Whether GPU is available
    pub has_gpu: bool,
    
    /// Current error rate (0.0 to 1.0)
    pub error_rate: f64,
    
    /// Schema version as string
    pub schema_version: String,
    
    /// Custom properties from node configuration
    pub custom_properties: HashMap<String, serde_json::Value>,
}

impl NodeEvaluationContext {
    /// Convert to CEL context for evaluation
    pub fn to_cel_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();
        
        context.insert(
            "available_memory".to_string(),
            Value::Int(self.available_memory as i64)
        );
        
        context.insert(
            "cpu_usage".to_string(),
            Value::Float(self.cpu_usage)
        );
        
        context.insert(
            "queue_depth".to_string(),
            Value::Int(self.queue_depth as i64)
        );
        
        context.insert(
            "active_jobs".to_string(),
            Value::Int(self.active_jobs as i64)
        );
        
        context.insert(
            "has_gpu".to_string(),
            Value::Bool(self.has_gpu)
        );
        
        context.insert(
            "error_rate".to_string(),
            Value::Float(self.error_rate)
        );
        
        context.insert(
            "schema_version".to_string(),
            Value::String(self.schema_version.clone())
        );
        
        // Add custom properties
        for (key, value) in &self.custom_properties {
            match value {
                serde_json::Value::String(s) => {
                    context.insert(key.clone(), Value::String(s.clone()));
                }
                serde_json::Value::Number(n) if n.is_i64() => {
                    context.insert(key.clone(), Value::Int(n.as_i64().unwrap()));
                }
                serde_json::Value::Number(n) if n.is_f64() => {
                    context.insert(key.clone(), Value::Float(n.as_f64().unwrap()));
                }
                serde_json::Value::Bool(b) => {
                    context.insert(key.clone(), Value::Bool(*b));
                }
                _ => {
                    // Convert to string for other types
                    context.insert(key.clone(), Value::String(value.to_string()));
                }
            }
        }
        
        context
    }
}

/// CEL evaluator for multi-node architecture expressions
pub struct CeleEvaluator {
    context: Context,
}

impl CeleEvaluator {
    /// Create a new CEL evaluator
    pub fn new() -> Self {
        Self {
            context: Context::default(),
        }
    }
    
    /// Evaluate a constraint expression (returns boolean)
    pub fn evaluate_constraint(&self, expression: &str, ctx: &NodeEvaluationContext) -> Result<bool, String> {
        let program = Program::compile(expression)
            .map_err(|e| format!("CEL compilation error: {}", e))?;
        
        let cel_context = ctx.to_cel_context();
        let result = program.execute(&self.context, &cel_context)
            .map_err(|e| format!("CEL execution error: {}", e))?;
        
        match result {
            Value::Bool(b) => Ok(b),
            _ => Err(format!("Constraint expression '{}' did not evaluate to boolean, got: {:?}", expression, result))
        }
    }
    
    /// Evaluate a routing policy expression (returns string)
    pub fn evaluate_routing_policy(&self, expression: &str, ctx: &NodeEvaluationContext) -> Result<String, String> {
        let program = Program::compile(expression)
            .map_err(|e| format!("CEL compilation error: {}", e))?;
        
        let cel_context = ctx.to_cel_context();
        let result = program.execute(&self.context, &cel_context)
            .map_err(|e| format!("CEL execution error: {}", e))?;
        
        match result {
            Value::String(s) => Ok(s),
            _ => Err(format!("Routing policy '{}' did not evaluate to string, got: {:?}", expression, result))
        }
    }
    
    /// Evaluate a numeric expression (for metrics and thresholds)
    pub fn evaluate_numeric(&self, expression: &str, ctx: &NodeEvaluationContext) -> Result<f64, String> {
        let program = Program::compile(expression)
            .map_err(|e| format!("CEL compilation error: {}", e))?;
        
        let cel_context = ctx.to_cel_context();
        let result = program.execute(&self.context, &cel_context)
            .map_err(|e| format!("CEL execution error: {}", e))?;
        
        match result {
            Value::Int(i) => Ok(i as f64),
            Value::Float(f) => Ok(f),
            _ => Err(format!("Numeric expression '{}' did not evaluate to number, got: {:?}", expression, result))
        }
    }
}

impl Default for CeleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constraint_evaluation() {
        let evaluator = CeleEvaluator::new();
        let mut ctx = NodeEvaluationContext {
            available_memory: 4 * 1024 * 1024 * 1024, // 4Gi
            cpu_usage: 0.5,
            queue_depth: 25,
            active_jobs: 5,
            has_gpu: true,
            error_rate: 0.02,
            schema_version: "1.2.0".to_string(),
            custom_properties: HashMap::new(),
        };
        
        // Test memory constraint
        assert!(evaluator.evaluate_constraint("available_memory > 2147483648", &ctx).unwrap());
        
        // Test CPU constraint
        assert!(evaluator.evaluate_constraint("cpu_usage < 0.8", &ctx).unwrap());
        
        // Test queue depth constraint
        assert!(evaluator.evaluate_constraint("queue_depth < 100", &ctx).unwrap());
        
        // Test GPU availability
        assert!(evaluator.evaluate_constraint("has_gpu == true", &ctx).unwrap());
        
        // Test error rate constraint
        assert!(evaluator.evaluate_constraint("error_rate < 0.05", &ctx).unwrap());
    }
    
    #[test]
    fn test_routing_policy_evaluation() {
        let evaluator = CeleEvaluator::new();
        let ctx = NodeEvaluationContext {
            available_memory: 8 * 1024 * 1024 * 1024, // 8Gi
            cpu_usage: 0.3,
            queue_depth: 10,
            active_jobs: 2,
            has_gpu: true,
            error_rate: 0.01,
            schema_version: "1.2.0".to_string(),
            custom_properties: HashMap::new(),
        };
        
        // Test routing policy
        let result = evaluator.evaluate_routing_policy("'high-memory-pool'", &ctx).unwrap();
        assert_eq!(result, "high-memory-pool");
    }
}
