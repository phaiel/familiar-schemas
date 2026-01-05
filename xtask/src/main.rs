use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "familiar-schemas")]
#[command(about = "Schema management and analysis toolkit")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze schema health and relationships
    Analyze,
    /// Fix broken schema references
    Fix,
    /// Export schema graph visualization
    Graph {
        /// Output format (svg)
        #[arg(short, long, default_value = "svg")]
        format: String,
    },
    /// Interactive schema exploration
    Explore,
    /// Generate infrastructure manifests from schemas
    Infra {
        /// Environment config file (e.g., infra/production.env.json)
        #[arg(short, long)]
        env: String,
        /// Output directory for manifests
        #[arg(short, long, default_value = "manifests")]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze => {
            println!("🔍 Running schema analysis...");
            println!("💡 Analysis not yet implemented - use direct binaries");
        }

        Commands::Fix => {
            println!("🔧 Running schema fixing...");
            println!("💡 Fixing not yet implemented - use direct binaries");
        }

        Commands::Graph { format } => {
            println!("📊 Generating {} schema graph...", format);
            let output_file = format!("schemas.{}", format);
            println!("💡 Output will be saved to: {}", output_file);
            run_command(&[
                "cargo", "run", "-p", "familiar-schemas", "--bin", "schema-graph-export",
                "--", "--format", &format, "--output", &output_file
            ]);
        }

        Commands::Explore => {
            println!("🎯 Starting interactive exploration...");
            println!("💡 Exploration not yet implemented - use graph-export for DOT format");
        }

        Commands::Infra { env, output } => {
            println!("🏗️  Generating infrastructure manifests...");
            println!("📋 Environment: {}", env);
            println!("📁 Output directory: {}", output);

            // Create output directory if it doesn't exist
            std::fs::create_dir_all(&output).unwrap_or_else(|e| {
                eprintln!("❌ Failed to create output directory: {}", e);
                std::process::exit(1);
            });

            run_command(&[
                "cargo", "run", "-p", "familiar-schemas", "--bin", "infra-generator",
                "--", "--env", &env, "--output", &output
            ]);
        }
    }
}

fn run_command(args: &[&str]) {
    use std::process::Command;
    println!("💡 Running: {}", args.join(" "));
    let status = Command::new(args[0])
        .args(&args[1..])
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        eprintln!("❌ Command failed with exit code: {}", status.code().unwrap_or(-1));
    }
}



