//! Schema Config CLI
//!
//! View and manage schema registry configuration.

use clap::{Parser, Subcommand};
use familiar_schemas::SchemaConfig;

#[derive(Parser)]
#[command(name = "schema-config")]
#[command(about = "View and manage schema registry configuration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current configuration
    Show {
        /// Config file to load (optional)
        #[arg(short, long)]
        config: Option<String>,
        
        /// Output as TOML
        #[arg(long)]
        toml: bool,
        
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    
    /// Initialize a new config file
    Init {
        /// Output path (default: schemas.toml)
        #[arg(short, long, default_value = "schemas.toml")]
        output: String,
    },
    
    /// Validate configuration
    Validate {
        /// Config file to validate
        #[arg(short, long)]
        config: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Show { config, toml, json } => {
            let cfg = SchemaConfig::load_from(config.as_deref())?;
            
            if json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else if toml {
                println!("{}", ::toml::to_string_pretty(&cfg)?);
            } else {
                // Pretty print
                println!("📋 Schema Registry Configuration\n");
                println!("Registry:");
                println!("  Path: {:?}", cfg.registry.path);
                println!("  Author: {:?}", cfg.registry.default_author);
                println!("  Immutable: {}", cfg.registry.immutable);
                
                println!("\nExport:");
                println!("  Format: {:?}", cfg.export.output_format);
                println!("  Checksums: {}", cfg.export.include_checksums);
                println!("  Manifest: {}", cfg.export.include_manifest);
                
                println!("\nWorkspace:");
                println!("  Root: {:?}", cfg.workspace.root);
                println!("  Crates:");
                for c in &cfg.workspace.crates {
                    println!("    - {} ({})", c.name, c.schemas_dir);
                }
                
                println!("\nValidation:");
                println!("  Strict JSON Schema: {}", cfg.validation.strict_json_schema);
                println!("  Validate AVRO: {}", cfg.validation.validate_avro);
                println!("  Fail on breaking: {}", cfg.validation.fail_on_breaking);
                
                if !cfg.categories.mappings.is_empty() {
                    println!("\nCategories:");
                    for (k, v) in &cfg.categories.mappings {
                        println!("  {} -> {}", k, v);
                    }
                }
            }
        }
        
        Commands::Init { output } => {
            let cfg = SchemaConfig::default();
            cfg.save(&output)?;
            println!("✅ Created config file: {}", output);
        }
        
        Commands::Validate { config } => {
            match SchemaConfig::load_from(config.as_deref()) {
                Ok(cfg) => {
                    println!("✅ Configuration is valid");
                    println!("   Registry: {:?}", cfg.registry.path);
                    println!("   Workspace: {:?}", cfg.workspace.root);
                    println!("   Crates: {}", cfg.workspace.crates.len());
                }
                Err(e) => {
                    eprintln!("❌ Configuration error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    
    Ok(())
}
