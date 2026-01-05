use clap::Parser;
use familiar_schemas::codegen::{generate_infrastructure, InfraError};
use std::path::PathBuf;

/// Infrastructure manifest generator for multi-node deployments
#[derive(Parser)]
#[command(name = "infra-generator")]
#[command(about = "Generate Kubernetes manifests from schema definitions")]
struct Cli {
    /// Path to schema directory
    #[arg(short, long, default_value = "versions/latest")]
    schema_dir: PathBuf,

    /// Environment config file (e.g., infra/production.env.json)
    #[arg(short, long)]
    env: PathBuf,

    /// Output directory for manifests
    #[arg(short, long, default_value = "manifests")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("🏗️  Generating infrastructure manifests...");
    println!("📂 Schema directory: {:?}", cli.schema_dir);
    println!("📋 Environment config: {:?}", cli.env);
    println!("📁 Output directory: {:?}", cli.output);

    // Generate infrastructure manifests
    match generate_infrastructure(&cli.schema_dir, &cli.env, &cli.output) {
        Ok(()) => {
            println!("✅ Infrastructure manifests generated successfully!");
            println!("📋 Check the {:?} directory for Kubernetes manifests", cli.output);
        }
        Err(InfraError::Io(e)) => {
            eprintln!("❌ IO Error: {}", e);
            std::process::exit(1);
        }
        Err(InfraError::Json(e)) => {
            eprintln!("❌ JSON parsing error: {}", e);
            std::process::exit(1);
        }
        Err(InfraError::SchemaNotFound(id)) => {
            eprintln!("❌ Schema not found: {}", id);
            std::process::exit(1);
        }
        Err(InfraError::InvalidScaling(msg)) => {
            eprintln!("❌ Invalid scaling configuration: {}", msg);
            std::process::exit(1);
        }
    }

    Ok(())
}
