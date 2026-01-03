//! Schema Export CLI
//!
//! Schema-First Architecture:
//! - Schemas are the source of truth (stored in familiar-schemas)
//! - This tool exports schemas FROM the registry, not TO the registry
//! - Protobuf schemas are collected from familiar-core/proto (hand-written, source of truth)
//!
//! For new schema versions, manually add schemas to the registry and use `schema-drift`
//! to validate Rust types match.

use std::path::PathBuf;
use std::fs;
use clap::Parser;
use familiar_schemas::{SchemaRegistry, Schema, SchemaType, SchemaVersion};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "schema-export")]
#[command(about = "Export Protobuf schemas from Familiar workspace to the registry")]
struct Cli {
    /// Path to schema registry
    #[arg(short, long, default_value = ".")]
    registry: PathBuf,

    /// Path to workspace root (for collecting Protobuf schemas)
    #[arg(short, long)]
    workspace: Option<PathBuf>,

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
    println!("Schema Export (Schema-First Mode)");
    println!("  Version: {}", cli.version);
    println!();

    let mut schemas = Vec::new();
    let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(ref workspace) = cli.workspace {
        println!("Workspace: {:?}", workspace);
        println!();
        
        // In Schema-First mode, we only collect Protobuf schemas from familiar-core/proto
        // These .proto files are hand-written and are the source of truth for Kafka messages
        let proto_dir = workspace.join("familiar-core/proto");
        if proto_dir.exists() {
            println!("Protobuf schemas - {:?}", proto_dir);
            let count_before = schemas.len();
            collect_protobuf_schemas(&proto_dir, &mut schemas, &mut seen_keys)?;
            println!("   Collected: {} schemas", schemas.len() - count_before);
        } else {
            println!("No Protobuf schemas found at {:?}", proto_dir);
        }
    } else {
        println!("No workspace specified. In schema-first mode, schemas are");
        println!("manually maintained in the registry. Use --workspace to collect");
        println!("Protobuf schemas from familiar-core/proto.");
        println!();
        println!("For JSON schemas, add them directly to the registry.");
    }

    println!();
    println!("Collection Summary:");
    println!("  Total schemas: {}", schemas.len());
    
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
        println!("Dry run - not registering schemas");
        return Ok(());
    }

    if schemas.is_empty() {
        println!();
        println!("No schemas to register.");
        return Ok(());
    }

    // Register with the registry
    println!();
    println!("Registering version {}...", cli.version);
    
    let mut registry = SchemaRegistry::open(&cli.registry)?;
    let version = SchemaVersion::parse(&cli.version)?;
    
    registry.register_version(
        version,
        schemas,
        cli.author.as_deref(),
        cli.message.as_deref(),
    )?;

    println!("Successfully registered version {}", cli.version);
    Ok(())
}

fn collect_protobuf_schemas(
    dir: &PathBuf,
    schemas: &mut Vec<Schema>,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }
        
        if path.extension().map_or(true, |e| e != "proto") {
            continue;
        }

        let filename = path.file_stem().unwrap().to_string_lossy().to_string();
        let content_str = fs::read_to_string(path)?;
        
        // Store proto content as a JSON string value for consistency with drift tool
        let content = serde_json::Value::String(content_str);
        
        // Protobuf schemas use "kafka" category
        let schema_type = SchemaType::Protobuf;
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
    Ok(())
}
