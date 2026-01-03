//! Schema Registry CLI
//!
//! Commands for managing the schema registry.

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use familiar_schemas::{SchemaRegistry, SchemaVersion, SchemaType};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "schema-registry")]
#[command(about = "Versioned, append-only schema registry")]
struct Cli {
    /// Path to schema registry
    #[arg(short, long, default_value = ".")]
    registry: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new schema registry
    Init {
        /// Path to create registry
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// List all versions
    List,

    /// Show details of a version
    Show {
        /// Version to show (e.g., "v0.1.0" or "latest")
        version: String,
    },

    /// Get a specific schema
    Get {
        /// Schema name
        name: String,
        /// Version (optional, defaults to latest)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Verify checksums for a version
    Verify {
        /// Version to verify
        version: String,
    },

    /// Compare two versions
    Diff {
        /// Old version
        old: String,
        /// New version
        new: String,
    },

    /// Export schemas to a directory
    Export {
        /// Version to export
        version: String,
        /// Output directory
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Show registry statistics
    Stats,
}

fn main() {
    // Initialize tracing
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
    match cli.command {
        Commands::Init { path } => {
            println!("📦 Initializing schema registry at {:?}", path);
            let registry = SchemaRegistry::open(&path)?;
            println!("✅ Registry initialized at {:?}", registry.root());
            Ok(())
        }

        Commands::List => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            let versions = registry.versions();
            
            if versions.is_empty() {
                println!("No versions registered yet.");
            } else {
                println!("📚 Registered versions:");
                for v in versions {
                    let is_latest = Some(v) == registry.latest_version();
                    let marker = if is_latest { " (latest)" } else { "" };
                    println!("  {} {}{}", v, v.created_at.format("%Y-%m-%d"), marker);
                }
            }
            Ok(())
        }

        Commands::Show { version } => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            let v = if version == "latest" {
                registry.latest_version()
                    .ok_or("No versions registered")?
                    .version_string()
            } else {
                version.clone()
            };

            let manifest = registry.get_manifest(&v)
                .ok_or_else(|| format!("Version {} not found", version))?;

            println!("📦 Version: {}", manifest.version);
            println!("📅 Created: {}", manifest.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("🔒 Checksum: {}", manifest.manifest_checksum);
            println!();
            println!("📊 Statistics:");
            println!("  Total schemas: {}", manifest.stats.total_schemas);
            println!("  JSON schemas: {}", manifest.stats.json_schemas);
            println!("  AVRO schemas: {}", manifest.stats.avro_schemas);
            println!("  TypeScript schemas: {}", manifest.stats.typescript_schemas);
            println!("  Python schemas: {}", manifest.stats.python_schemas);
            println!();
            println!("  By category:");
            for (cat, count) in &manifest.stats.by_category {
                println!("    {}: {}", cat, count);
            }
            println!("📄 Schemas:");
            for entry in &manifest.schemas {
                println!("  {} ({:?})", entry.schema.name, entry.schema.schema_type);
            }
            Ok(())
        }

        Commands::Get { name, version } => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            let v = version.as_deref();

            if let Some(entry) = registry.get_schema(&name, v) {
                println!("{}", serde_json::to_string_pretty(&entry.schema.content)?);
            } else {
                eprintln!("Schema '{}' not found", name);
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Verify { version } => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            
            if registry.verify_version(&version)? {
                println!("✅ All checksums verified for {}", version);
            } else {
                eprintln!("❌ Checksum verification failed for {}", version);
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Diff { old, new } => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            let results = registry.check_compatibility(&old, &new)?;

            println!("📊 Compatibility check: {} -> {}", old, new);
            println!();

            let mut breaking_count = 0;
            let mut compatible_count = 0;

            for (name, result) in &results {
                if result.is_breaking {
                    breaking_count += 1;
                    println!("❌ {} - BREAKING", name);
                    for change in &result.changes {
                        if change.is_breaking {
                            println!("   └─ {}: {}", change.path, change.description);
                        }
                    }
                } else if !result.changes.is_empty() {
                    compatible_count += 1;
                    println!("✅ {} - compatible ({} changes)", name, result.changes.len());
                }
            }

            println!();
            println!("Summary: {} breaking, {} compatible", breaking_count, compatible_count);

            if breaking_count > 0 {
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Export { version, output } => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            registry.export_version(&version, &output)?;
            println!("✅ Exported {} to {:?}", version, output);
            Ok(())
        }

        Commands::Stats => {
            let registry = SchemaRegistry::open(&cli.registry)?;
            let versions = registry.versions();

            println!("📊 Registry Statistics");
            println!();
            println!("Total versions: {}", versions.len());
            
            if let Some(latest) = registry.latest_version() {
                println!("Latest version: {}", latest);
                
                if let Some(manifest) = registry.get_manifest(&latest.version_string()) {
                    println!("Total schemas: {}", manifest.stats.total_schemas);
                }
            }

            // Schema type breakdown
            println!();
            println!("Schemas by type (latest):");
            for schema_type in [
                SchemaType::JsonSchema,
                SchemaType::Avro,
                SchemaType::TypeScript,
                SchemaType::Python,
            ] {
                let count = registry.get_schemas_by_type(schema_type).len();
                if count > 0 {
                    println!("  {:?}: {}", schema_type, count);
                }
            }

            Ok(())
        }
    }
}








