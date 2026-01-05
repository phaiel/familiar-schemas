//! Schema Facet Linting
//!
//! Enforces the doctrine: "Schemas describe data shape + portable intent;
//! everything executable lives elsewhere. Facets are for codegen and
//! interoperability, never for business logic or orchestration."
//!
//! ## Lints
//! 1. **Facet Strictness**: All x-familiar-* must validate against meta-schema
//! 2. **No-Op Detection**: Fail if facets exactly match defaults (prevents sprawl)
//! 3. **Red-Line Violations**: Fail on code-like strings, forbidden patterns

use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;

/// Result of linting a schema
#[derive(Debug, Default)]
pub struct LintResult {
    pub schema_id: String,
    pub errors: Vec<LintError>,
    pub warnings: Vec<LintWarning>,
}

impl LintResult {
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[derive(Debug)]
pub struct LintError {
    pub code: &'static str,
    pub message: String,
    pub path: String,
}

#[derive(Debug)]
pub struct LintWarning {
    pub code: &'static str,
    pub message: String,
    pub path: String,
}

/// The schema facet linter
pub struct FacetLinter {
    /// Patterns that indicate code-like strings (RED LINE)
    code_patterns: Vec<Regex>,
    /// Keywords that indicate forbidden orchestration (RED LINE)
    orchestration_keywords: HashSet<&'static str>,
    /// Keywords that indicate forbidden persistence details (RED LINE)
    persistence_keywords: HashSet<&'static str>,
    /// Keywords that indicate forbidden validation DSL (RED LINE)
    validation_keywords: HashSet<&'static str>,
    /// Allowed x-familiar-* extension names
    allowed_extensions: HashSet<&'static str>,
}

impl Default for FacetLinter {
    fn default() -> Self {
        Self::new()
    }
}

impl FacetLinter {
    pub fn new() -> Self {
        Self {
            // Code-like patterns (Rust, TS, Python syntax)
            code_patterns: vec![
                Regex::new(r"::\w").unwrap(),           // Rust paths
                Regex::new(r"->").unwrap(),             // Rust/TS arrows
                Regex::new(r"<\w+>").unwrap(),          // Generics
                Regex::new(r"^use\s").unwrap(),         // Rust use
                Regex::new(r"^import\s").unwrap(),      // JS/TS import
                Regex::new(r"^from\s+\w+\s+import").unwrap(), // Python import
                Regex::new(r"fn\s+\w+\s*\(").unwrap(),  // Rust fn
                Regex::new(r"def\s+\w+\s*\(").unwrap(), // Python def
                Regex::new(r"function\s+\w+\s*\(").unwrap(), // JS function
                Regex::new(r"impl\s+\w+").unwrap(),     // Rust impl
                Regex::new(r"trait\s+\w+").unwrap(),    // Rust trait
                Regex::new(r"pub\s+(fn|struct|enum|mod)").unwrap(), // Rust pub
                Regex::new(r"#\[derive").unwrap(),      // Rust derive attr
                Regex::new(r"@\w+\(").unwrap(),         // Decorators
            ],
            
            // Orchestration keywords (services, workflows, triggers)
            orchestration_keywords: [
                "workflow", "trigger", "publish", "subscribe", "kafka",
                "temporal", "windmill", "service", "endpoint", "call",
                "invoke", "emit", "dispatch", "queue", "topic",
                "saga", "choreography", "orchestration",
            ].into_iter().collect(),
            
            // Persistence keywords (DB details beyond structure)
            persistence_keywords: [
                "table", "index", "shard", "partition", "join",
                "soft_delete", "audit_log", "redis", "postgres",
                "mongodb", "cassandra", "dynamo", "migration",
                "query_plan", "cache", "ttl", "eviction",
            ].into_iter().collect(),
            
            // Validation DSL keywords
            validation_keywords: [
                "validator", "validate", "constraint", "immutable",
                "non_empty", "valid_email", "regex_match", "cross_field",
                "lifecycle", "conditional_required",
            ].into_iter().collect(),
            
            // Allowed x-familiar-* extensions
            allowed_extensions: [
                // Meta / Classification
                "x-familiar-kind",
                "x-familiar-deprecated",
                "x-familiar-role",
                "x-familiar-pii",
                "x-familiar-pii-class",
                "x-familiar-meta-schema",
                "x-familiar-requires-auth",
                
                // Wire / Graph / Infrastructure
                "x-familiar-service",
                "x-familiar-queue",
                "x-familiar-resources",
                "x-familiar-depends",
                "x-familiar-input",
                "x-familiar-output",
                "x-familiar-reads",
                "x-familiar-writes",
                
                // ECS / Node Infrastructure
                "x-familiar-system",
                "x-familiar-systems",
                "x-familiar-components",
                "x-familiar-concurrency",
                "x-familiar-memory",
                "x-familiar-resource-class",
                
                // Resource Configuration (structural, not behavioral)
                "x-familiar-resource-type",
                "x-familiar-virtual",
                "x-familiar-endpoint",
                "x-familiar-persistence",
                "x-familiar-tables",
                "x-familiar-connection-pool",
                "x-familiar-config",
                "x-familiar-default",
                
                // Queue Configuration
                "x-familiar-queue-type",
                "x-familiar-consumers",
                "x-familiar-producers",
                "x-familiar-dlq",
                "x-familiar-retention",
                "x-familiar-visibility-timeout",
                
                // Rate/Timeout Configuration
                "x-familiar-rate-limit",
                "x-familiar-timeout",
                "x-familiar-retries",
                "x-familiar-health-check",
                
                // LLM Configuration
                "x-familiar-models",
                
                // Codegen - Portable Intent
                "x-familiar-enum-repr",
                "x-familiar-discriminator",
                "x-familiar-content",
                "x-familiar-casing",
                "x-familiar-variants",
                "x-familiar-flatten",
                "x-familiar-skip-none",
                "x-familiar-newtype",
                "x-familiar-field-alias",
                "x-familiar-field-order",
                "x-familiar-compose",
                
                // Codegen - Rust-specific (scoped)
                "x-familiar-rust-impl-ids",
                "x-familiar-rust-derives",
                "x-familiar-rust-derive-add",
                "x-familiar-rust-derive-exclude",
                "x-familiar-rust-derive-policy",
                "x-familiar-rust-default",
                "x-familiar-rust-serde",
                "x-familiar-rust-recursion",
                
                // Capabilities (high-level, verifiable)
                "x-familiar-hashable",
                "x-familiar-orderable",
                "x-familiar-equality",
            ].into_iter().collect(),
        }
    }
    
    /// Lint a schema for facet violations
    pub fn lint(&self, schema_id: &str, schema: &Value) -> LintResult {
        let mut result = LintResult {
            schema_id: schema_id.to_string(),
            ..Default::default()
        };
        
        self.lint_value(schema, "", &mut result);
        result
    }
    
    fn lint_value(&self, value: &Value, path: &str, result: &mut LintResult) {
        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    
                    // Check x-familiar-* extensions
                    if key.starts_with("x-familiar-") {
                        self.lint_extension(key, val, &child_path, result);
                    }
                    
                    // Recurse
                    self.lint_value(val, &child_path, result);
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let child_path = format!("{}[{}]", path, i);
                    self.lint_value(val, &child_path, result);
                }
            }
            Value::String(s) => {
                // Check string values for red-line patterns
                self.lint_string_value(s, path, result);
            }
            _ => {}
        }
    }
    
    fn lint_extension(&self, key: &str, value: &Value, path: &str, result: &mut LintResult) {
        // 1. Check if extension is allowed
        if !self.allowed_extensions.contains(key) {
            result.errors.push(LintError {
                code: "UNKNOWN_EXTENSION",
                message: format!("Unknown x-familiar-* extension: '{}'. Add to allowed list or remove.", key),
                path: path.to_string(),
            });
        }
        
        // 2. Check for code-like values
        if let Some(s) = value.as_str() {
            for pattern in &self.code_patterns {
                if pattern.is_match(s) {
                    result.errors.push(LintError {
                        code: "CODE_IN_FACET",
                        message: format!("Facet contains code-like string: '{}'. Use behavior identifiers, not code.", s),
                        path: path.to_string(),
                    });
                    break;
                }
            }
        }
        
        // 3. Check arrays for code-like values
        if let Some(arr) = value.as_array() {
            for (i, item) in arr.iter().enumerate() {
                if let Some(s) = item.as_str() {
                    for pattern in &self.code_patterns {
                        if pattern.is_match(s) {
                            result.errors.push(LintError {
                                code: "CODE_IN_FACET",
                                message: format!("Facet array contains code-like string: '{}'", s),
                                path: format!("{}[{}]", path, i),
                            });
                            break;
                        }
                    }
                }
            }
        }
        
        // 4. Check for no-op defaults
        self.check_noop_default(key, value, path, result);
        
        // 5. Validate specific extension schemas
        self.validate_extension_schema(key, value, path, result);
    }
    
    fn lint_string_value(&self, value: &str, path: &str, result: &mut LintResult) {
        let value_lower = value.to_lowercase();
        
        // Check for orchestration keywords in string values
        // Skip:
        // - x-familiar-kind (pure classification, can legitimately be "queue", "windmill", etc.)
        // - x-familiar-queue/service (infrastructure routing, can contain queue/service names)
        if !path.contains("x-familiar-kind") 
            && !path.contains("x-familiar-queue") 
            && !path.contains("x-familiar-service") 
        {
            for keyword in &self.orchestration_keywords {
                if value_lower.contains(keyword) && path.contains("x-familiar") {
                    result.errors.push(LintError {
                        code: "ORCHESTRATION_IN_FACET",
                        message: format!("Facet contains orchestration keyword '{}'. Orchestration belongs in workflow configs.", keyword),
                        path: path.to_string(),
                    });
                }
            }
        }
        
        // Check for persistence keywords
        for keyword in &self.persistence_keywords {
            if value_lower.contains(keyword) && path.contains("x-familiar") {
                // Only error if it looks like config, not just a type name
                if !path.contains("kind") && !path.contains("type") {
                    result.warnings.push(LintWarning {
                        code: "PERSISTENCE_IN_FACET",
                        message: format!("Facet may contain persistence detail '{}'. Consider moving to DB config.", keyword),
                        path: path.to_string(),
                    });
                }
            }
        }
        
        // Check for validation DSL keywords
        for keyword in &self.validation_keywords {
            if value_lower.contains(keyword) && path.contains("x-familiar") {
                result.errors.push(LintError {
                    code: "VALIDATION_DSL_IN_FACET",
                    message: format!("Facet contains validation keyword '{}'. Use JSON Schema or dedicated validator.", keyword),
                    path: path.to_string(),
                });
            }
        }
    }
    
    fn check_noop_default(&self, key: &str, value: &Value, path: &str, result: &mut LintResult) {
        // Check for values that exactly match defaults (pointless sprawl)
        let is_noop = match key {
            "x-familiar-rust-derive-policy" => value.as_str() == Some("strict"),
            "x-familiar-rust-default" => value.as_str() == Some("derived"),
            "x-familiar-rust-recursion" => {
                value.get("strategy").and_then(|s| s.as_str()) == Some("box")
                    && value.get("preferred_edges").map(|e| e.as_array().map(|a| a.is_empty()).unwrap_or(true)).unwrap_or(true)
            }
            "x-familiar-flatten" => value.as_bool() == Some(false),
            "x-familiar-skip-none" => value.as_bool() == Some(false),
            "x-familiar-newtype" => value.as_bool() == Some(false),
            "x-familiar-rust-derive-add" => value.as_array().map(|a| a.is_empty()).unwrap_or(false),
            "x-familiar-rust-derive-exclude" => value.as_array().map(|a| a.is_empty()).unwrap_or(false),
            _ => false,
        };
        
        if is_noop {
            result.warnings.push(LintWarning {
                code: "NOOP_FACET",
                message: format!("Facet '{}' has default value. Remove to reduce noise.", key),
                path: path.to_string(),
            });
        }
    }
    
    fn validate_extension_schema(&self, key: &str, value: &Value, path: &str, result: &mut LintResult) {
        match key {
            // Validate enum-repr values
            "x-familiar-enum-repr" => {
                if let Some(s) = value.as_str() {
                    let valid = ["external", "internal", "adjacent", "untagged", "unit"];
                    if !valid.contains(&s) {
                        result.errors.push(LintError {
                            code: "INVALID_ENUM_REPR",
                            message: format!("Invalid enum repr '{}'. Must be one of: {:?}", s, valid),
                            path: path.to_string(),
                        });
                    }
                }
            }
            
            // Validate derive-policy values
            "x-familiar-rust-derive-policy" => {
                if let Some(s) = value.as_str() {
                    let valid = ["strict", "allow_graph_suggestions"];
                    if !valid.contains(&s) {
                        result.errors.push(LintError {
                            code: "INVALID_DERIVE_POLICY",
                            message: format!("Invalid derive policy '{}'. Must be one of: {:?}", s, valid),
                            path: path.to_string(),
                        });
                    }
                }
            }
            
            // Validate default values
            "x-familiar-rust-default" => {
                if let Some(s) = value.as_str() {
                    let valid = ["derived", "custom", "none"];
                    if !valid.contains(&s) {
                        result.errors.push(LintError {
                            code: "INVALID_DEFAULT_VALUE",
                            message: format!("Invalid default strategy '{}'. Must be one of: {:?}", s, valid),
                            path: path.to_string(),
                        });
                    }
                }
            }
            
            // Validate recursion strategy
            "x-familiar-rust-recursion" => {
                if let Some(obj) = value.as_object() {
                    if let Some(strategy) = obj.get("strategy").and_then(|s| s.as_str()) {
                        let valid = ["box", "arc", "value_fallback"];
                        if !valid.contains(&strategy) {
                            result.errors.push(LintError {
                                code: "INVALID_RECURSION_STRATEGY",
                                message: format!("Invalid recursion strategy '{}'. Must be one of: {:?}", strategy, valid),
                                path: path.to_string(),
                            });
                        }
                    }
                }
            }
            
            // Validate impl-ids are PascalCase identifiers
            "x-familiar-rust-impl-ids" => {
                if let Some(arr) = value.as_array() {
                    let pascal_re = Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap();
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(s) = item.as_str() {
                            if !pascal_re.is_match(s) {
                                result.errors.push(LintError {
                                    code: "INVALID_IMPL_ID",
                                    message: format!("Impl ID '{}' must be PascalCase identifier", s),
                                    path: format!("{}[{}]", path, i),
                                });
                            }
                        }
                    }
                }
            }
            
            // Validate derives are valid Rust trait names
            "x-familiar-rust-derives" | "x-familiar-rust-derive-add" | "x-familiar-rust-derive-exclude" => {
                if let Some(arr) = value.as_array() {
                    let trait_re = Regex::new(r"^[A-Z][a-zA-Z0-9_]*$").unwrap();
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(s) = item.as_str() {
                            if !trait_re.is_match(s) {
                                result.errors.push(LintError {
                                    code: "INVALID_DERIVE_NAME",
                                    message: format!("Derive '{}' must be valid trait name", s),
                                    path: format!("{}[{}]", path, i),
                                });
                            }
                        }
                    }
                }
            }
            
            _ => {}
        }
    }
}

/// Lint all schemas in a directory
pub fn lint_schemas(schema_dir: &std::path::Path) -> Vec<LintResult> {
    let linter = FacetLinter::new();
    let mut results = Vec::new();
    
    for entry in walkdir::WalkDir::new(schema_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
    {
        let path = entry.path();
        let Ok(content) = std::fs::read_to_string(path) else { continue };
        let Ok(schema): Result<Value, _> = serde_json::from_str(&content) else { continue };
        
        let schema_id = path.strip_prefix(schema_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        
        let result = linter.lint(&schema_id, &schema);
        if !result.is_clean() || result.has_warnings() {
            results.push(result);
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_code_pattern_detection() {
        let linter = FacetLinter::new();
        let schema = json!({
            "x-familiar-rust-impl-ids": ["From<ModelConfig>"]
        });
        let result = linter.lint("test", &schema);
        assert!(result.errors.iter().any(|e| e.code == "CODE_IN_FACET"));
    }
    
    #[test]
    fn test_unknown_extension() {
        let linter = FacetLinter::new();
        let schema = json!({
            "x-familiar-custom-thing": "value"
        });
        let result = linter.lint("test", &schema);
        assert!(result.errors.iter().any(|e| e.code == "UNKNOWN_EXTENSION"));
    }
    
    #[test]
    fn test_noop_default() {
        let linter = FacetLinter::new();
        let schema = json!({
            "x-familiar-rust-derive-policy": "strict"
        });
        let result = linter.lint("test", &schema);
        assert!(result.warnings.iter().any(|w| w.code == "NOOP_FACET"));
    }
    
    #[test]
    fn test_valid_schema() {
        let linter = FacetLinter::new();
        let schema = json!({
            "x-familiar-kind": "entity",
            "x-familiar-rust-impl-ids": ["Moment"],
            "x-familiar-rust-derive-policy": "allow_graph_suggestions"
        });
        let result = linter.lint("test", &schema);
        assert!(result.is_clean());
    }
}

