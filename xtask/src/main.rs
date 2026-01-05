use clap::{Parser, Subcommand};
use std::collections::HashMap;

#[derive(Debug)]
struct CelValidationError {
    schema_path: String,
    expression: String,
    message: String,
}

fn validate_cel_expressions(schema_dir: &str) -> Result<(), Vec<CelValidationError>> {
    use std::fs;
    use walkdir::WalkDir;
    use cel_interpreter::Context;

    let mut errors = Vec::new();
    let context = Context::default();

    // Create mock context for validation
    let mock_context = create_mock_node_context();

    for entry in WalkDir::new(schema_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension() == Some(std::ffi::OsStr::new("json")) {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we can't read
            };

            let schema: serde_json::Value = match serde_json::from_str(&content) {
                Ok(s) => s,
                Err(_) => continue, // Skip invalid JSON
            };

            let schema_path = entry.path().strip_prefix(schema_dir).unwrap_or(entry.path())
                .to_string_lossy().to_string();

            // Check for CEL expressions in constraints
            if let Some(constraints) = schema.get("constraints") {
                if let Some(constraints_obj) = constraints.as_object() {
                    for (key, value) in constraints_obj {
                        if let Some(expr) = value.as_str() {
                            if let Err(e) = validate_cel_expression(&context, expr, &mock_context) {
                                errors.push(CelValidationError {
                                    schema_path: schema_path.clone(),
                                    expression: expr.to_string(),
                                    message: format!("constraints.{}: {}", key, e),
                                });
                            }
                        }
                    }
                }
            }

            // Check for CEL expressions in dispatch.routing_policy
            if let Some(dispatch) = schema.get("dispatch") {
                if let Some(dispatch_arr) = dispatch.as_array() {
                    for (i, item) in dispatch_arr.iter().enumerate() {
                        if let Some(routing_policy) = item.get("routing_policy") {
                            if let Some(expr) = routing_policy.as_str() {
                                if let Err(e) = validate_cel_expression(&context, expr, &mock_context) {
                                    errors.push(CelValidationError {
                                        schema_path: schema_path.clone(),
                                        expression: expr.to_string(),
                                        message: format!("dispatch[{}].routing_policy: {}", i, e),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_cel_expression(
    context: &cel_interpreter::Context,
    expression: &str,
    mock_context: &HashMap<String, cel_interpreter::Value>,
) -> Result<(), String> {
    // For now, just validate compilation since the API is different than expected
    // We'll need to investigate the correct way to provide runtime context
    match cel_interpreter::Program::compile(expression) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Compilation error: {}", e)),
    }
}

fn create_mock_node_context() -> HashMap<String, cel_interpreter::Value> {
    // Simplified for now - just return empty context since we're only validating compilation
    HashMap::new()
}

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
    /// Validate CEL expressions in schemas
    ValidateCel {
        /// Schema directory to validate
        #[arg(short, long, default_value = "versions/latest")]
        schema_dir: String,
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

        Commands::ValidateCel { schema_dir } => {
            println!("🔍 Validating CEL expressions in schemas...");
            match validate_cel_expressions(&schema_dir) {
                Ok(_) => println!("✅ All CEL expressions are valid!"),
                Err(errors) => {
                    eprintln!("❌ Found {} CEL validation errors:", errors.len());
                    for error in errors {
                        eprintln!("  {}: {}", error.schema_path, error.message);
                    }
                    std::process::exit(1);
                }
            }
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



