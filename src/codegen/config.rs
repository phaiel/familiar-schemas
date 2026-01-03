//! Codegen Configuration
//!
//! Two-tier configuration:
//! - Global (CodegenConfig): Language-agnostic settings for naming, diagnostics, primitives
//! - Per-language (RenderProfile): Type mappings, optionality, union encoding, layout
//!
//! Key principle: Classification (SchemaShape, SCC, TypeKind) is config-free.
//! Only emission/rendering uses configuration.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

// =============================================================================
// Global Configuration (Language-Agnostic)
// =============================================================================

/// Global codegen configuration - affects all languages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenConfig {
    /// Naming conventions
    pub naming: NamingConfig,
    
    /// Diagnostics thresholds
    pub diagnostics: DiagnosticsConfig,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            naming: NamingConfig::default(),
            diagnostics: DiagnosticsConfig::default(),
        }
    }
}

/// Naming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    /// Acronyms to preserve (e.g., ID, URL, UUID, API)
    pub acronyms: HashSet<String>,
    
    /// Whether to preserve all-caps enum variants
    pub preserve_screaming_case: bool,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            acronyms: ["ID", "URL", "UUID", "API", "HTTP", "JSON", "XML", "SQL", "URI", "UI", "IO"]
                .iter().map(|s| s.to_string()).collect(),
            preserve_screaming_case: true,
        }
    }
}

/// Diagnostics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsConfig {
    /// How to handle ambiguous oneOf patterns
    pub ambiguous_oneof: DiagnosticLevel,
    
    /// How to handle alias chains (alias of alias)
    pub alias_of_alias: DiagnosticLevel,
    
    /// How to handle unknown schema patterns
    pub unknown_pattern: DiagnosticLevel,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            ambiguous_oneof: DiagnosticLevel::Error,
            alias_of_alias: DiagnosticLevel::Warn,
            unknown_pattern: DiagnosticLevel::Warn,
        }
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warn,
    Ignore,
}

// =============================================================================
// Render Profile (Per-Language)
// =============================================================================

/// Language-specific rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderProfile {
    /// Language identifier
    pub language: Language,
    
    /// Type mappings for JSON scalars and formats
    pub types: TypeMappings,
    
    /// How to handle optional/nullable fields
    pub optional: OptionalStrategy,
    
    /// How to encode unions/enums
    pub unions: UnionEncoding,
    
    /// File/module layout strategy
    pub layout: LayoutStrategy,
    
    /// Language-specific keyword escapes
    pub keyword_escape: String,
}

/// Supported target languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    Python,
}

/// Type mappings for JSON scalar types and formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMappings {
    /// JSON string -> language type
    pub string: String,
    /// JSON integer -> language type
    pub integer: String,
    /// JSON number -> language type
    pub number: String,
    /// JSON boolean -> language type
    pub boolean: String,
    
    /// Format-specific mappings
    pub datetime: String,
    pub uuid: String,
    pub date: String,
    pub time: String,
    pub decimal: Option<String>,
    pub uri: Option<String>,
    pub email: Option<String>,
    
    /// Unknown/any type
    pub any: String,
}

/// How to handle optional and nullable fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalStrategy {
    /// How to represent "missing" (field not present)
    pub missing: OptionalRepr,
    
    /// How to represent "nullable" (field present but null)
    pub nullable: OptionalRepr,
    
    /// Whether to distinguish missing vs null
    pub distinguish_missing_null: bool,
}

/// Representation of optional values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionalRepr {
    /// Rust: Option<T>
    Option,
    /// TypeScript: T | null
    UnionNull,
    /// TypeScript: foo?: T
    QuestionMark,
    /// Python: Optional[T]
    OptionalType,
    /// Python: T | None
    UnionNone,
}

/// How to encode unions and enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionEncoding {
    /// Tagged union configuration
    pub tagged: TaggedUnionConfig,
    
    /// String enum configuration
    pub string_enum: StringEnumConfig,
}

/// Tagged union configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedUnionConfig {
    /// Discriminator field name (default: "type")
    pub discriminator: String,
    
    /// For Rust serde: use internal/external/adjacent tagging
    pub serde_tag_style: Option<SerdeTagStyle>,
    
    /// Content field name for adjacent tagging
    pub content_field: Option<String>,
}

/// Serde tag style for Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SerdeTagStyle {
    /// #[serde(tag = "type")]
    Internal,
    /// #[serde(tag = "type", content = "data")]
    Adjacent,
    /// #[serde(untagged)]
    Untagged,
    /// Default external tagging
    External,
}

/// String enum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringEnumConfig {
    /// Casing for variant names in code
    pub variant_casing: Casing,
    
    /// For Rust: use #[serde(rename_all = "...")]
    pub serde_rename_all: Option<String>,
}

/// Casing convention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Casing {
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
}

/// File/module layout strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutStrategy {
    /// Single file with all types
    SingleFile,
    /// Rust: mod tree with mod.rs files
    CrateModTree,
    /// TypeScript: barrel exports (index.ts)
    Barrel,
    /// Python: package with __init__.py
    Package,
}

// =============================================================================
// Default Profiles
// =============================================================================

impl RenderProfile {
    /// Create the default Rust profile
    pub fn rust() -> Self {
        Self {
            language: Language::Rust,
            types: TypeMappings {
                string: "String".to_string(),
                integer: "i64".to_string(),
                number: "f64".to_string(),
                boolean: "bool".to_string(),
                datetime: "chrono::DateTime<chrono::Utc>".to_string(),
                uuid: "uuid::Uuid".to_string(),
                date: "chrono::NaiveDate".to_string(),
                time: "chrono::NaiveTime".to_string(),
                decimal: Some("rust_decimal::Decimal".to_string()),
                uri: Some("String".to_string()), // Could use url::Url
                email: Some("String".to_string()), // Validated at construction
                any: "serde_json::Value".to_string(),
            },
            optional: OptionalStrategy {
                missing: OptionalRepr::Option,
                nullable: OptionalRepr::Option,
                distinguish_missing_null: false, // Rust Option handles both
            },
            unions: UnionEncoding {
                tagged: TaggedUnionConfig {
                    discriminator: "type".to_string(),
                    serde_tag_style: Some(SerdeTagStyle::Internal),
                    content_field: None,
                },
                string_enum: StringEnumConfig {
                    variant_casing: Casing::PascalCase,
                    serde_rename_all: Some("snake_case".to_string()),
                },
            },
            layout: LayoutStrategy::SingleFile,
            keyword_escape: "r#".to_string(),
        }
    }
    
    /// Create the "Rust-like" TypeScript profile (strict + zod)
    pub fn typescript_strict() -> Self {
        Self {
            language: Language::TypeScript,
            types: TypeMappings {
                string: "string".to_string(),
                integer: "number".to_string(),
                number: "number".to_string(),
                boolean: "boolean".to_string(),
                datetime: "Date".to_string(), // Or string for ISO
                uuid: "string".to_string(),   // UUID as branded type or string
                date: "string".to_string(),   // ISO date string
                time: "string".to_string(),   // ISO time string
                decimal: Some("string".to_string()), // Decimal as string for precision
                uri: Some("string".to_string()),
                email: Some("string".to_string()),
                any: "unknown".to_string(), // Prefer unknown over any
            },
            optional: OptionalStrategy {
                missing: OptionalRepr::QuestionMark, // foo?: T
                nullable: OptionalRepr::UnionNull,   // T | null (explicit)
                distinguish_missing_null: true,      // Be explicit
            },
            unions: UnionEncoding {
                tagged: TaggedUnionConfig {
                    discriminator: "kind".to_string(), // "kind" is common in TS
                    serde_tag_style: None,
                    content_field: None,
                },
                string_enum: StringEnumConfig {
                    variant_casing: Casing::ScreamingSnakeCase, // ENUM_VALUE
                    serde_rename_all: None,
                },
            },
            layout: LayoutStrategy::Barrel,
            keyword_escape: "_".to_string(),
        }
    }
    
    /// Create the "Rust-like" Python profile (pydantic v2 + strict)
    pub fn python_strict() -> Self {
        Self {
            language: Language::Python,
            types: TypeMappings {
                string: "str".to_string(),
                integer: "int".to_string(),
                number: "float".to_string(),
                boolean: "bool".to_string(),
                datetime: "datetime.datetime".to_string(),
                uuid: "uuid.UUID".to_string(),
                date: "datetime.date".to_string(),
                time: "datetime.time".to_string(),
                decimal: Some("decimal.Decimal".to_string()),
                uri: Some("str".to_string()),
                email: Some("pydantic.EmailStr".to_string()),
                any: "Any".to_string(),
            },
            optional: OptionalStrategy {
                missing: OptionalRepr::OptionalType, // Optional[T]
                nullable: OptionalRepr::UnionNone,   // T | None
                distinguish_missing_null: true,
            },
            unions: UnionEncoding {
                tagged: TaggedUnionConfig {
                    discriminator: "kind".to_string(),
                    serde_tag_style: None,
                    content_field: None,
                },
                string_enum: StringEnumConfig {
                    variant_casing: Casing::ScreamingSnakeCase,
                    serde_rename_all: None,
                },
            },
            layout: LayoutStrategy::Package,
            keyword_escape: "_".to_string(),
        }
    }
}

// =============================================================================
// Render Helpers
// =============================================================================

impl RenderProfile {
    /// Escape a keyword if needed
    pub fn escape_keyword(&self, name: &str) -> String {
        let keywords = match self.language {
            Language::Rust => &RUST_KEYWORDS[..],
            Language::TypeScript => &TS_KEYWORDS[..],
            Language::Python => &PYTHON_KEYWORDS[..],
        };
        
        if keywords.contains(&name) {
            format!("{}{}", self.keyword_escape, name)
        } else {
            name.to_string()
        }
    }
    
    /// Get the type string for a JSON scalar
    pub fn scalar_type(&self, scalar: &str) -> &str {
        match scalar {
            "string" => &self.types.string,
            "integer" => &self.types.integer,
            "number" => &self.types.number,
            "boolean" => &self.types.boolean,
            _ => &self.types.any,
        }
    }
    
    /// Get the type string for a JSON format
    pub fn format_type(&self, format: &str) -> Option<&str> {
        match format {
            "date-time" => Some(&self.types.datetime),
            "uuid" => Some(&self.types.uuid),
            "date" => Some(&self.types.date),
            "time" => Some(&self.types.time),
            "decimal" => self.types.decimal.as_deref(),
            "uri" | "uri-reference" => self.types.uri.as_deref(),
            "email" => self.types.email.as_deref(),
            _ => None,
        }
    }
    
    /// Wrap a type to make it optional (missing)
    pub fn wrap_optional(&self, type_str: &str) -> String {
        match self.optional.missing {
            OptionalRepr::Option => format!("Option<{}>", type_str),
            OptionalRepr::UnionNull => format!("{} | null", type_str),
            OptionalRepr::QuestionMark => type_str.to_string(), // Property itself is optional
            OptionalRepr::OptionalType => format!("Optional[{}]", type_str),
            OptionalRepr::UnionNone => format!("{} | None", type_str),
        }
    }
    
    /// Wrap a type to make it nullable
    pub fn wrap_nullable(&self, type_str: &str) -> String {
        match self.optional.nullable {
            OptionalRepr::Option => format!("Option<{}>", type_str),
            OptionalRepr::UnionNull => format!("{} | null", type_str),
            OptionalRepr::QuestionMark => format!("{} | null", type_str),
            OptionalRepr::OptionalType => format!("Optional[{}]", type_str),
            OptionalRepr::UnionNone => format!("{} | None", type_str),
        }
    }
    
    /// Wrap a type in a container (array)
    pub fn wrap_array(&self, type_str: &str) -> String {
        match self.language {
            Language::Rust => format!("Vec<{}>", type_str),
            Language::TypeScript => format!("{}[]", type_str),
            Language::Python => format!("list[{}]", type_str),
        }
    }
    
    /// Wrap a type in a map
    pub fn wrap_map(&self, value_type: &str) -> String {
        match self.language {
            Language::Rust => format!("std::collections::HashMap<String, {}>", value_type),
            Language::TypeScript => format!("Record<string, {}>", value_type),
            Language::Python => format!("dict[str, {}]", value_type),
        }
    }
    
    /// Wrap a type in Box (Rust only, for recursion)
    pub fn wrap_box(&self, type_str: &str) -> String {
        match self.language {
            Language::Rust => format!("Box<{}>", type_str),
            _ => type_str.to_string(), // Other languages handle recursion automatically
        }
    }
}

// =============================================================================
// Keywords
// =============================================================================

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while",
    "async", "await", "dyn", "abstract", "become", "box", "do", "final",
    "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
];

const TS_KEYWORDS: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger",
    "default", "delete", "do", "else", "enum", "export", "extends", "false",
    "finally", "for", "function", "if", "import", "in", "instanceof", "new",
    "null", "return", "super", "switch", "this", "throw", "true", "try",
    "typeof", "var", "void", "while", "with", "as", "implements", "interface",
    "let", "package", "private", "protected", "public", "static", "yield",
    "any", "boolean", "constructor", "declare", "get", "module", "require",
    "number", "set", "string", "symbol", "type", "from", "of",
];

const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break",
    "class", "continue", "def", "del", "elif", "else", "except", "finally",
    "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal",
    "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_profile_defaults() {
        let profile = RenderProfile::rust();
        assert_eq!(profile.types.string, "String");
        assert_eq!(profile.types.integer, "i64");
        assert_eq!(profile.keyword_escape, "r#");
    }
    
    #[test]
    fn test_typescript_profile_defaults() {
        let profile = RenderProfile::typescript_strict();
        assert_eq!(profile.types.string, "string");
        assert_eq!(profile.types.any, "unknown");
        assert!(profile.optional.distinguish_missing_null);
    }
    
    #[test]
    fn test_keyword_escape() {
        let rust = RenderProfile::rust();
        assert_eq!(rust.escape_keyword("type"), "r#type");
        assert_eq!(rust.escape_keyword("name"), "name");
        
        let ts = RenderProfile::typescript_strict();
        assert_eq!(ts.escape_keyword("class"), "_class");
    }
    
    #[test]
    fn test_wrap_optional() {
        let rust = RenderProfile::rust();
        assert_eq!(rust.wrap_optional("String"), "Option<String>");
        
        let ts = RenderProfile::typescript_strict();
        assert_eq!(ts.wrap_nullable("string"), "string | null");
    }
    
    #[test]
    fn test_wrap_containers() {
        let rust = RenderProfile::rust();
        assert_eq!(rust.wrap_array("i64"), "Vec<i64>");
        assert_eq!(rust.wrap_map("String"), "std::collections::HashMap<String, String>");
        
        let ts = RenderProfile::typescript_strict();
        assert_eq!(ts.wrap_array("number"), "number[]");
        assert_eq!(ts.wrap_map("string"), "Record<string, string>");
    }
}

