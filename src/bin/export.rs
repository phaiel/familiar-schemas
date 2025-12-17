//! Schema Export CLI
//!
//! Exports schemas from familiar-core and familiar-primitives to the registry.

use std::path::PathBuf;
use std::fs;
use clap::Parser;
use familiar_schemas::{SchemaRegistry, Schema, SchemaType, SchemaVersion};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "schema-export")]
#[command(about = "Export schemas from familiar-core to the registry")]
struct Cli {
    /// Path to schema registry
    #[arg(short, long, default_value = ".")]
    registry: PathBuf,

    /// Path to familiar-core (source of schemas)
    #[arg(short, long)]
    source: PathBuf,

    /// Version to create (e.g., "0.1.0")
    #[arg(short, long)]
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
    println!("📦 Schema Export");
    println!("  Source: {:?}", cli.source);
    println!("  Version: {}", cli.version);
    println!();

    let mut schemas = Vec::new();

    // 1. Collect JSON Schemas from generated/schemas/
    let json_schemas_dir = cli.source.join("generated/schemas");
    if json_schemas_dir.exists() {
        println!("📂 Loading JSON Schemas from {:?}", json_schemas_dir);
        collect_json_schemas(&json_schemas_dir, &mut schemas)?;
    }

    // 2. Collect AVRO schemas
    let avro_schemas_dir = cli.source.join("schemas");
    if avro_schemas_dir.exists() {
        println!("📂 Loading AVRO schemas from {:?}", avro_schemas_dir);
        collect_avro_schemas(&avro_schemas_dir, &mut schemas)?;
    }

    // 3. Load manifest for additional metadata
    let manifest_path = cli.source.join("generated/manifest.json");
    if manifest_path.exists() {
        println!("📂 Loading manifest from {:?}", manifest_path);
        // We could use this for additional type information
    }

    println!();
    println!("📊 Collected schemas:");
    println!("  Total: {}", schemas.len());
    
    let mut type_counts: std::collections::HashMap<SchemaType, usize> = std::collections::HashMap::new();
    for schema in &schemas {
        *type_counts.entry(schema.schema_type).or_insert(0) += 1;
    }
    for (schema_type, count) in &type_counts {
        println!("  {:?}: {}", schema_type, count);
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

fn collect_json_schemas(dir: &PathBuf, schemas: &mut Vec<Schema>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "json") {
            let filename = path.file_stem().unwrap().to_string_lossy();
            
            // Determine schema type based on path
            let schema_type = if path.to_string_lossy().contains("agentic") {
                SchemaType::RustType
            } else if filename.ends_with("_schema") {
                SchemaType::JsonSchema
            } else {
                SchemaType::JsonSchema
            };

            let content: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
            
            // Extract schema name from filename (remove .schema.json suffix)
            let name = filename
                .trim_end_matches(".schema")
                .trim_end_matches("_schema")
                .to_string();

            let mut schema = Schema::new(name, schema_type, content);
            schema.source_path = Some(path.to_string_lossy().to_string());
            schemas.push(schema);
        }
    }
    Ok(())
}

fn collect_avro_schemas(dir: &PathBuf, schemas: &mut Vec<Schema>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |e| e == "avsc") {
            let filename = path.file_stem().unwrap().to_string_lossy().to_string();
            let content: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
            
            let mut schema = Schema::new(filename, SchemaType::Avro, content);
            schema.source_path = Some(path.to_string_lossy().to_string());
            schemas.push(schema);
        }
    }
    Ok(())
}

