//! Name Resolution Pass
//!
//! Builds a canonical mapping from schema IDs to Rust type names, handling:
//! - Type origin classification (Primitive, Generated, External, Stdlib)
//! - Name collision detection and resolution
//! - Field type reference resolution
//!
//! This pass runs AFTER shape detection and SCC analysis, BEFORE emission.
//! The result is stored in CodegenContext and exposed through Regions.
//!
//! Key principle: Name resolution is language-AGNOSTIC. It determines
//! canonical type names and origins, not language-specific rendering.

use std::collections::{HashMap, HashSet};

use super::config::NamingConfig;
use crate::graph::{SchemaGraph, SchemaId};

// =============================================================================
// Type Origin
// =============================================================================

/// Where a type comes from (determines how to reference it in generated code)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeOrigin {
    /// Lives in familiar-primitives crate (don't generate, import)
    Primitive,
    
    /// Generated in this codegen run
    Generated,
    
    /// External type from a dependency (DateTime, Uuid, etc.)
    External {
        /// The canonical type path (e.g., "chrono::DateTime<chrono::Utc>")
        canonical_path: String,
    },
    
    /// Rust standard library type (String, Vec, etc.) - never generate
    Stdlib,
}

// =============================================================================
// Resolved Name Entry
// =============================================================================

/// Entry in the name resolution map
#[derive(Debug, Clone)]
pub struct ResolvedName {
    /// The canonical type name (language-agnostic PascalCase)
    pub canonical_name: String,
    
    /// Where this type comes from
    pub origin: TypeOrigin,
    
    /// Original schema path (for diagnostics)
    pub schema_path: String,
    
    /// Whether this name was disambiguated due to collision
    pub disambiguated: bool,
    
    /// Directory this schema is in (for namespace)
    pub directory: Option<String>,
}

// =============================================================================
// Name Resolver
// =============================================================================

/// Resolves schema IDs to canonical type names with collision handling.
/// 
/// This is language-AGNOSTIC. It determines:
/// - Canonical names (PascalCase)
/// - Type origins (Primitive, Generated, External, Stdlib)
/// - Collision disambiguation
/// 
/// Language-specific rendering (casing, escaping) is done by RenderProfile.
pub struct NameResolver {
    /// schema_id -> resolved name entry
    resolved: HashMap<SchemaId, ResolvedName>,
    
    /// Tracks which canonical names are in use to detect collisions
    name_to_schema: HashMap<String, SchemaId>,
    
    /// Stdlib types that should never be generated
    stdlib_names: HashSet<String>,
    
    /// Actual primitive type names (implemented in familiar-primitives)
    primitive_names: HashSet<String>,
    
    /// Naming configuration
    naming_config: NamingConfig,
}

impl NameResolver {
    /// Create a new resolver with config
    pub fn new(naming_config: NamingConfig) -> Self {
        let stdlib_names = [
            "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
            "HashMap", "HashSet", "BTreeMap", "BTreeSet", "RefCell", "Cell",
            "Mutex", "RwLock", "Debug", "Clone", "Default", "Copy", "Send", "Sync",
            "bool", "i8", "i16", "i32", "i64", "i128", "isize",
            "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64",
        ].iter().map(|s| s.to_string()).collect();
        
        // Types that actually exist in familiar-primitives crate
        // These are the ONLY types that should be treated as primitives
        let primitive_names = [
            // ID types
            "TenantId", "UserId", "SessionId", "ThreadId", "MessageId", "ChannelId",
            "CourseId", "ShuttleId", "EntityId", "InvitationId", "JoinRequestId",
            "MagicLinkId", "AuditLogId", "ConsentRecordId", "TaskId",
            "ExportRequestId", "DeletionRequestId",
            // Validated string types
            "Email", "InviteCode", "PasswordHash", "SessionToken", "ApiKey",
            // Numeric types
            "NormalizedFloat", "SignedNormalizedFloat", "Temperature", "MaxTokens",
            "QuantizedCoord", "DbPoolSize",
            // Other types
            "TokenUsage", "DbConnectionString", "Timestamp", "UUID", "InviteRole",
        ].iter().map(|s| s.to_string()).collect();
        
        Self {
            resolved: HashMap::new(),
            name_to_schema: HashMap::new(),
            stdlib_names,
            primitive_names,
            naming_config,
        }
    }
    
    /// Build the resolution map from a schema graph.
    /// 
    /// This identifies primitives by type name (must exist in familiar-primitives),
    /// detects collisions, and assigns unique canonical names to all schemas.
    pub fn build(graph: &SchemaGraph, naming_config: NamingConfig) -> Self {
        let mut resolver = Self::new(naming_config);
        
        // First pass: identify primitives by name (not just directory)
        // A schema is only a primitive if its name is in the primitive_names set
        for schema_id in graph.all_ids() {
            let base_name = resolver.extract_base_name(schema_id);
            let directory = resolver.extract_directory(schema_id);
            let is_actual_primitive = resolver.primitive_names.contains(&base_name);
            
            if is_actual_primitive {
                resolver.resolved.insert(schema_id.clone(), ResolvedName {
                    canonical_name: base_name.clone(),
                    origin: TypeOrigin::Primitive,
                    schema_path: schema_id.clone(),
                    disambiguated: false,
                    directory,
                });
                resolver.name_to_schema.insert(base_name, schema_id.clone());
            }
        }
        
        // Second pass: resolve all other schemas, handling collisions
        for schema_id in graph.all_ids() {
            if resolver.resolved.contains_key(schema_id) {
                continue; // Already resolved (primitive)
            }
            
            let base_name = resolver.extract_base_name(schema_id);
            let directory = resolver.extract_directory(schema_id);
            
            // Check for stdlib conflict
            if resolver.stdlib_names.contains(&base_name) {
                resolver.resolved.insert(schema_id.clone(), ResolvedName {
                    canonical_name: base_name.clone(),
                    origin: TypeOrigin::Stdlib,
                    schema_path: schema_id.clone(),
                    disambiguated: false,
                    directory,
                });
                continue;
            }
            
            // Check for collision with existing name
            let (canonical_name, disambiguated) = if resolver.name_to_schema.contains_key(&base_name) {
                // Collision! Use directory prefix to disambiguate
                let prefix = resolver.extract_directory_prefix(schema_id);
                let disambiguated_name = if prefix.is_empty() {
                    // No good prefix, append directory path
                    format!("{}_{}", base_name, schema_id.replace('/', "_").replace('.', "_"))
                } else {
                    format!("{}{}", prefix, base_name)
                };
                (disambiguated_name, true)
            } else {
                (base_name.clone(), false)
            };
            
            resolver.name_to_schema.insert(canonical_name.clone(), schema_id.clone());
            resolver.resolved.insert(schema_id.clone(), ResolvedName {
                canonical_name,
                origin: TypeOrigin::Generated,
                schema_path: schema_id.clone(),
                disambiguated,
                directory,
            });
        }
        
        resolver
    }
    
    /// Extract the directory a schema is in
    fn extract_directory(&self, schema_id: &str) -> Option<String> {
        let parts: Vec<&str> = schema_id.split('/').collect();
        if parts.len() >= 2 {
            Some(parts[parts.len() - 2].to_string())
        } else {
            None
        }
    }
    
    /// Extract base type name from schema path
    fn extract_base_name(&self, schema_id: &str) -> String {
        let file_name = schema_id
            .rsplit('/')
            .next()
            .unwrap_or(schema_id)
            .trim_end_matches(".schema.json")
            .trim_end_matches(".json");
        
        // Replace dots with underscores (dots are invalid in Rust identifiers)
        // e.g., "Codegen.meta" -> "CodegenMeta"
        let file_name = file_name.replace('.', "_");
        
        // If the file name has no separators (already PascalCase), preserve it
        if !file_name.contains('_') && !file_name.contains('-') && !file_name.contains(' ') {
            return file_name;
        }
        
        self.to_pascal_case(&file_name)
    }
    
    /// Extract directory prefix for disambiguation
    fn extract_directory_prefix(&self, schema_id: &str) -> String {
        let parts: Vec<&str> = schema_id.split('/').collect();
        if parts.len() >= 2 {
            let dir_name = parts[parts.len() - 2];
            // Skip if it's a common directory that doesn't add meaning
            if matches!(dir_name, "json-schema" | "schemas" | "types" | ".." | "." | "src") {
                return String::new();
            }
            // If already PascalCase (no separators), preserve it
            if !dir_name.contains('_') && !dir_name.contains('-') && !dir_name.contains(' ') {
                let mut chars = dir_name.chars();
                return match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().chain(chars).collect()
                    }
                };
            }
            return self.to_pascal_case(dir_name);
        }
        String::new()
    }
    
    /// Convert string to PascalCase, respecting acronyms
    fn to_pascal_case(&self, s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut capitalize_next = true;
        let mut current_word = String::new();
        
        for c in s.chars() {
            if c == '_' || c == '-' || c == ' ' {
                // End of word - check if it's an acronym
                if !current_word.is_empty() {
                    result.push_str(&self.case_word(&current_word));
                    current_word.clear();
                }
                capitalize_next = true;
            } else if capitalize_next {
                current_word.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                current_word.push(c);
            }
        }
        
        // Handle last word
        if !current_word.is_empty() {
            result.push_str(&self.case_word(&current_word));
        }
        
        result
    }
    
    /// Apply casing to a word, preserving acronyms
    fn case_word(&self, word: &str) -> String {
        let upper = word.to_uppercase();
        
        // Check if it's a known acronym
        if self.naming_config.acronyms.contains(&upper) {
            return upper;
        }
        
        // Check if it's all uppercase (preserve if configured)
        if self.naming_config.preserve_screaming_case && word.chars().all(|c| c.is_ascii_uppercase()) {
            return word.to_string();
        }
        
        // Standard PascalCase: first letter upper, rest lower
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                let mut result = first.to_uppercase().to_string();
                for c in chars {
                    result.push(c.to_ascii_lowercase());
                }
                result
            }
        }
    }
    
    /// Get the resolved name for a schema ID
    pub fn get(&self, schema_id: &str) -> Option<&ResolvedName> {
        self.resolved.get(schema_id)
    }
    
    /// Get the canonical type name for a schema ID
    pub fn canonical_name(&self, schema_id: &str) -> Option<&str> {
        self.resolved.get(schema_id).map(|r| r.canonical_name.as_str())
    }
    
    /// Get the type origin for a schema ID
    pub fn origin(&self, schema_id: &str) -> Option<&TypeOrigin> {
        self.resolved.get(schema_id).map(|r| &r.origin)
    }
    
    /// Check if a schema is a primitive
    pub fn is_primitive(&self, schema_id: &str) -> bool {
        self.resolved.get(schema_id)
            .map(|r| r.origin == TypeOrigin::Primitive)
            .unwrap_or(false)
    }
    
    /// Check if a schema is stdlib (should not be generated)
    pub fn is_stdlib(&self, schema_id: &str) -> bool {
        self.resolved.get(schema_id)
            .map(|r| r.origin == TypeOrigin::Stdlib)
            .unwrap_or(false)
    }
    
    /// Check if a schema should be generated
    pub fn should_generate(&self, schema_id: &str) -> bool {
        self.resolved.get(schema_id)
            .map(|r| r.origin == TypeOrigin::Generated)
            .unwrap_or(false)
    }
    
    /// Resolve a $ref to a canonical name
    pub fn resolve_ref(&self, ref_target: &str) -> Option<&ResolvedName> {
        // First try direct lookup
        if let Some(resolved) = self.resolved.get(ref_target) {
            return Some(resolved);
        }
        
        // Try normalized path (remove leading ../)
        let normalized = self.normalize_ref(ref_target);
        self.resolved.get(&normalized)
    }
    
    /// Normalize a $ref path for lookup
    fn normalize_ref(&self, ref_path: &str) -> String {
        let mut path = ref_path.to_string();
        while path.starts_with("../") {
            path = path[3..].to_string();
        }
        path
    }
    
    /// Get all resolved names
    pub fn all_resolved(&self) -> impl Iterator<Item = (&SchemaId, &ResolvedName)> {
        self.resolved.iter()
    }
    
    /// Get all primitives (for generating import statement)
    pub fn primitives(&self) -> impl Iterator<Item = &str> {
        self.resolved.iter()
            .filter(|(_, r)| r.origin == TypeOrigin::Primitive)
            .map(|(_, r)| r.canonical_name.as_str())
    }
    
    /// Get count of each type origin
    pub fn stats(&self) -> NameResolverStats {
        let mut stats = NameResolverStats::default();
        for entry in self.resolved.values() {
            match entry.origin {
                TypeOrigin::Primitive => stats.primitives += 1,
                TypeOrigin::Generated => stats.generated += 1,
                TypeOrigin::External { .. } => stats.external += 1,
                TypeOrigin::Stdlib => stats.stdlib += 1,
            }
            if entry.disambiguated {
                stats.disambiguated += 1;
            }
        }
        stats
    }
}

/// Statistics from name resolution
#[derive(Debug, Default)]
pub struct NameResolverStats {
    pub primitives: usize,
    pub generated: usize,
    pub external: usize,
    pub stdlib: usize,
    pub disambiguated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_config() -> NamingConfig {
        NamingConfig::default()
    }
    
    #[test]
    fn test_primitive_names() {
        let resolver = NameResolver::new(test_config());
        
        // Types in the primitive_names set should be recognized
        assert!(resolver.primitive_names.contains("TenantId"));
        assert!(resolver.primitive_names.contains("NormalizedFloat"));
        assert!(resolver.primitive_names.contains("Email"));
        
        // Types not in the set should not be primitives
        assert!(!resolver.primitive_names.contains("Thread"));
        assert!(!resolver.primitive_names.contains("Model"));
        assert!(!resolver.primitive_names.contains("AIProvider")); // In primitives/ dir but not implemented
    }
    
    #[test]
    fn test_extract_base_name() {
        let resolver = NameResolver::new(test_config());
        
        // Already PascalCase - preserve exactly
        assert_eq!(resolver.extract_base_name("primitives/TenantId.schema.json"), "TenantId");
        assert_eq!(resolver.extract_base_name("entities/Thread.schema.json"), "Thread");
        assert_eq!(resolver.extract_base_name("primitives/AIProvider.schema.json"), "AIProvider");
        
        // Has separators - convert to PascalCase
        assert_eq!(resolver.extract_base_name("some_type"), "SomeType");
    }
    
    #[test]
    fn test_acronym_preservation() {
        let resolver = NameResolver::new(test_config());
        
        // With separators, ID/UUID/API/URL should be preserved as uppercase
        assert_eq!(resolver.to_pascal_case("tenant_id"), "TenantID");
        assert_eq!(resolver.to_pascal_case("user-uuid"), "UserUUID");
        assert_eq!(resolver.to_pascal_case("api_url"), "APIURL");
    }
    
    #[test]
    fn test_extract_directory_prefix() {
        let resolver = NameResolver::new(test_config());
        
        assert_eq!(resolver.extract_directory_prefix("database/Model.schema.json"), "Database");
        assert_eq!(resolver.extract_directory_prefix("auth/Model.schema.json"), "Auth");
        assert_eq!(resolver.extract_directory_prefix("Model.schema.json"), "");
    }
}
