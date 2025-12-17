//! Schema Export CLI
//!
//! Exports schemas from familiar-primitives, familiar-contracts, and familiar-core to the registry.
//! Supports both single-crate and workspace-wide collection.

use std::path::PathBuf;
use std::fs;
use clap::Parser;
use familiar_schemas::{SchemaRegistry, Schema, SchemaType, SchemaVersion};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "schema-export")]
#[command(about = "Export schemas from Familiar workspace to the registry")]
struct Cli {
    /// Path to schema registry
    #[arg(short, long, default_value = ".")]
    registry: PathBuf,

    /// Path to workspace root (collects from all crates)
    #[arg(short, long)]
    workspace: Option<PathBuf>,

    /// Path to a single crate (alternative to --workspace)
    #[arg(short, long)]
    source: Option<PathBuf>,

    /// Version to create (e.g., "0.1.0")
    #[arg(short('V'), long)]
    version: String,

    /// Author name
    #[arg(short, long)]
    author: Option<String>,

    /// Release notes
    #[arg(short, long)]
    message: Option<String>,

    /// Dry run - don't actually register
    #[arg(long)]
    dry_run: bool,
}

/// Crate information for collection
struct CrateInfo {
    name: &'static str,
    schemas_dir: &'static str,
    category: &'static str,
}

const WORKSPACE_CRATES: &[CrateInfo] = &[
    CrateInfo {
        name: "familiar-primitives",
        schemas_dir: "generated/schemas",
        category: "primitives",
    },
    CrateInfo {
        name: "familiar-contracts",
        schemas_dir: "generated/schemas",
        category: "contracts",
    },
    CrateInfo {
        name: "familiar-core",
        schemas_dir: "generated/schemas",
        category: "core",
    },
];

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Schema Export");
    println!("  Version: {}", cli.version);
    println!();

    let mut schemas = Vec::new();
    // Track seen schemas by key (schema_type/category/name) to deduplicate
    let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut duplicate_count = 0;

    if let Some(ref workspace) = cli.workspace {
        // Workspace mode - collect from all crates
        // Order matters: familiar-primitives is processed first, so its schemas take precedence
        println!("🔍 Workspace mode: {:?}", workspace);
        println!();
        
        for crate_info in WORKSPACE_CRATES {
            let crate_path = workspace.join(crate_info.name);
            let schemas_dir = crate_path.join(crate_info.schemas_dir);
            
            if schemas_dir.exists() {
                println!("📂 {} - {:?}", crate_info.name, schemas_dir);
                let count_before = schemas.len();
                let dup_before = duplicate_count;
                collect_json_schemas_dedup(&schemas_dir, crate_info.category, crate_info.name, &mut schemas, &mut seen_keys, &mut duplicate_count)?;
                let collected = schemas.len() - count_before;
                let skipped = duplicate_count - dup_before;
                println!("   Collected: {} schemas{}", collected, 
                    if skipped > 0 { format!(" (skipped {} duplicates)", skipped) } else { String::new() });
            } else {
                println!("⚠️  {} - No schemas found at {:?}", crate_info.name, schemas_dir);
            }
        }
        
        // Also collect AVRO schemas from familiar-core
        let avro_dir = workspace.join("familiar-core/schemas");
        if avro_dir.exists() {
            println!("📂 AVRO schemas - {:?}", avro_dir);
            let count_before = schemas.len();
            collect_avro_schemas_dedup(&avro_dir, &mut schemas, &mut seen_keys)?;
            println!("   Collected: {} schemas", schemas.len() - count_before);
        }
    } else if let Some(ref source) = cli.source {
        // Single crate mode (no deduplication needed)
        println!("📂 Single crate mode: {:?}", source);
        
        let json_schemas_dir = source.join("generated/schemas");
        if json_schemas_dir.exists() {
            println!("📂 Loading JSON Schemas from {:?}", json_schemas_dir);
            collect_json_schemas_dedup(&json_schemas_dir, "types", "unknown", &mut schemas, &mut seen_keys, &mut duplicate_count)?;
        }

        let avro_schemas_dir = source.join("schemas");
        if avro_schemas_dir.exists() {
            println!("📂 Loading AVRO schemas from {:?}", avro_schemas_dir);
            collect_avro_schemas_dedup(&avro_schemas_dir, &mut schemas, &mut seen_keys)?;
        }
    } else {
        return Err("Either --workspace or --source must be specified".into());
    }
    
    if duplicate_count > 0 {
        println!();
        println!("ℹ️  Skipped {} duplicate schemas (familiar-primitives takes precedence)", duplicate_count);
    }

    println!();
    println!("📊 Collection Summary:");
    println!("  Total schemas: {}", schemas.len());
    
    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<&Schema>> = std::collections::HashMap::new();
    for schema in &schemas {
        by_category.entry(schema.category.clone()).or_default().push(schema);
    }
    
    for (category, cat_schemas) in &by_category {
        println!("  {} ({}):", category, cat_schemas.len());
        for schema in cat_schemas.iter().take(5) {
            println!("    - {}", schema.name);
        }
        if cat_schemas.len() > 5 {
            println!("    ... and {} more", cat_schemas.len() - 5);
        }
    }
    
    // Group by type
    let mut type_counts: std::collections::HashMap<SchemaType, usize> = std::collections::HashMap::new();
    for schema in &schemas {
        *type_counts.entry(schema.schema_type).or_insert(0) += 1;
    }
    println!();
    println!("  By type:");
    for (schema_type, count) in &type_counts {
        println!("    {:?}: {}", schema_type, count);
    }

    if cli.dry_run {
        println!();
        println!("🔍 Dry run - not registering schemas");
        return Ok(());
    }

    // Register with the registry
    println!();
    println!("📝 Registering version {}...", cli.version);
    
    let mut registry = SchemaRegistry::open(&cli.registry)?;
    let version = SchemaVersion::parse(&cli.version)?;
    
    registry.register_version(
        version,
        schemas,
        cli.author.as_deref(),
        cli.message.as_deref(),
    )?;

    println!("✅ Successfully registered version {}", cli.version);
    Ok(())
}

fn extract_category(path: &str) -> Option<String> {
    // Extract category from path like ".../generated/schemas/auth/User.schema.json"
    let parts: Vec<&str> = path.split('/').collect();
    
    // Find "schemas" in path and get the next component
    for (i, part) in parts.iter().enumerate() {
        if *part == "schemas" && i + 1 < parts.len() {
            let next = parts[i + 1];
            // Skip if it's a .json file (no category subdirectory)
            if !next.ends_with(".json") {
                return Some(next.to_string());
            }
        }
    }
    None
}

fn collect_json_schemas_dedup(
    dir: &PathBuf, 
    default_category: &str, 
    source_crate: &str,
    schemas: &mut Vec<Schema>, 
    seen_keys: &mut std::collections::HashSet<String>,
    duplicate_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Only process .schema.json or .json files
        if !path.is_file() {
            continue;
        }
        
        let filename = path.file_name().unwrap().to_string_lossy();
        if !filename.ends_with(".json") {
            continue;
        }
        
        // Skip manifest.json and README files
        if filename == "manifest.json" || filename.starts_with("README") {
            continue;
        }
        
        let stem = path.file_stem().unwrap().to_string_lossy();
        let path_str = path.to_string_lossy().to_string();
        
        // All JSON schemas use SchemaType::JsonSchema
        let schema_type = SchemaType::JsonSchema;
        
        // Extract category from path (e.g., "auth" from ".../schemas/auth/User.schema.json")
        let category = extract_category(&path_str).unwrap_or_else(|| default_category.to_string());

        // Extract schema name from filename (remove .schema suffix if present)
        let name = stem
            .trim_end_matches(".schema")
            .trim_end_matches("_schema")
            .to_string();

        // Create unique key: schema_type/category/name
        let key = format!("{}/{}/{}", schema_type.dir_name(), category, name);
        
        // Skip if we've already seen this schema (earlier crates take precedence)
        if seen_keys.contains(&key) {
            *duplicate_count += 1;
            continue;
        }
        seen_keys.insert(key);

        let content: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;

        let mut schema = Schema::new(name, schema_type, content);
        schema.source_path = Some(path_str);
        schema.set_category(&category);
        schema.set_source_crate(source_crate);
        schemas.push(schema);
    }
    Ok(())
}

fn collect_avro_schemas_dedup(
    dir: &PathBuf, 
    schemas: &mut Vec<Schema>,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |e| e == "avsc") {
            let filename = path.file_stem().unwrap().to_string_lossy().to_string();
            let content: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
            
            // AVRO schemas use "kafka" category
            let schema_type = SchemaType::Avro;
            let category = "kafka";
            
            // Create unique key: schema_type/category/name
            let key = format!("{}/{}/{}", schema_type.dir_name(), category, filename);
            
            // Skip if we've already seen this schema
            if seen_keys.contains(&key) {
                continue;
            }
            seen_keys.insert(key);
            
            let mut schema = Schema::new(filename, schema_type, content);
            schema.source_path = Some(path.to_string_lossy().to_string());
            schema.set_category(category);
            schema.set_source_crate("familiar-core");
            schemas.push(schema);
        }
    }
    Ok(())
}
