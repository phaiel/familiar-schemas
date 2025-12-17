//! Schema Validator CLI
//!
//! Validates schemas and checks compatibility between versions.

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use familiar_schemas::{SchemaRegistry, SchemaType};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "schema-validator")]
#[command(about = "Validate schemas and check compatibility")]
struct Cli {
    /// Path to schema registry
    #[arg(short, long, default_value = ".")]
    registry: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate all checksums in a version
    Checksums {
        /// Version to validate (or "all")
        #[arg(default_value = "latest")]
        version: String,
    },

    /// Check compatibility between versions
    Compatibility {
        /// Base version
        #[arg(short, long)]
        from: String,
        /// Target version
        #[arg(short, long)]
        to: String,
        /// Strict mode - any change is breaking
        #[arg(long)]
        strict: bool,
    },

    /// Check for breaking changes in the latest version
    Breaking {
        /// Compare against this version
        #[arg(short, long)]
        from: Option<String>,
    },

    /// Generate a compatibility report
    Report {
        /// Output file (JSON)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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
    let registry = SchemaRegistry::open(&cli.registry)?;

    match cli.command {
        Commands::Checksums { version } => {
            if version == "all" {
                println!("🔍 Validating all versions...");
                let mut all_valid = true;
                
                for v in registry.versions() {
                    let valid = registry.verify_version(&v.version_string())?;
                    if valid {
                        println!("  ✅ {} - valid", v);
                    } else {
                        println!("  ❌ {} - INVALID", v);
                        all_valid = false;
                    }
                }

                if !all_valid {
                    std::process::exit(1);
                }
            } else {
                let v = if version == "latest" {
                    registry.latest_version()
                        .ok_or("No versions registered")?
                        .version_string()
                } else {
                    version.clone()
                };

                if registry.verify_version(&v)? {
                    println!("✅ Version {} - all checksums valid", v);
                } else {
                    println!("❌ Version {} - checksum validation FAILED", v);
                    std::process::exit(1);
                }
            }
            Ok(())
        }

        Commands::Compatibility { from, to, strict } => {
            println!("🔍 Checking compatibility: {} -> {}", from, to);
            if strict {
                println!("  (strict mode enabled)");
            }
            println!();

            let results = registry.check_compatibility(&from, &to)?;

            let mut has_breaking = false;
            
            for (name, result) in &results {
                if result.is_breaking {
                    has_breaking = true;
                    println!("❌ {} - BREAKING CHANGE", name);
                    println!("   {}", result.summary);
                    for change in &result.changes {
                        if change.is_breaking {
                            println!("   └─ {} at {}", change.description, change.path);
                        }
                    }
                } else if !result.changes.is_empty() {
                    println!("✅ {} - {} changes (compatible)", name, result.changes.len());
                }
            }

            println!();
            if has_breaking {
                println!("❌ Breaking changes detected!");
                std::process::exit(1);
            } else {
                println!("✅ All changes are backward compatible");
            }
            Ok(())
        }

        Commands::Breaking { from } => {
            let latest = registry.latest_version()
                .ok_or("No versions registered")?
                .version_string();

            let base = from.unwrap_or_else(|| {
                let versions = registry.versions();
                if versions.len() < 2 {
                    latest.clone()
                } else {
                    versions[versions.len() - 2].version_string()
                }
            });

            println!("🔍 Checking for breaking changes: {} -> {}", base, latest);
            println!();

            let results = registry.check_compatibility(&base, &latest)?;

            let breaking: Vec<_> = results
                .iter()
                .filter(|(_, r)| r.is_breaking)
                .collect();

            if breaking.is_empty() {
                println!("✅ No breaking changes detected");
            } else {
                println!("❌ {} schema(s) with breaking changes:", breaking.len());
                for (name, result) in breaking {
                    println!();
                    println!("  {} ({} changes)", name, result.changes.len());
                    for change in &result.changes {
                        if change.is_breaking {
                            println!("    └─ {}", change.description);
                        }
                    }
                }
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Report { output } => {
            let versions = registry.versions();
            
            let mut report = serde_json::json!({
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "versions": versions.len(),
                "latest": registry.latest_version().map(|v| v.to_string()),
                "compatibility": {}
            });

            // Check compatibility between consecutive versions
            for window in versions.windows(2) {
                let old_v = &window[0].version_string();
                let new_v = &window[1].version_string();
                
                let results = registry.check_compatibility(old_v, new_v)?;
                
                let breaking_count = results.values().filter(|r| r.is_breaking).count();
                let compatible_count = results.values().filter(|r| !r.is_breaking && !r.changes.is_empty()).count();
                
                report["compatibility"][format!("{} -> {}", old_v, new_v)] = serde_json::json!({
                    "breaking_schemas": breaking_count,
                    "compatible_schemas": compatible_count,
                    "unchanged_schemas": results.len() - breaking_count - compatible_count
                });
            }

            let report_json = serde_json::to_string_pretty(&report)?;

            if let Some(path) = output {
                std::fs::write(&path, &report_json)?;
                println!("✅ Report written to {:?}", path);
            } else {
                println!("{}", report_json);
            }

            Ok(())
        }
    }
}

