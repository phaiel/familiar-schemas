use std::path::PathBuf;
use clap::Parser;
use familiar_schemas::SchemaGraph;

#[derive(Parser)]
#[command(name = "schema-graph-export")]
#[command(about = "Export schema dependency graph to DOT/SVG format")]
struct Cli {
    /// Path to schema directory (defaults to current directory)
    #[arg(short, long)]
    schema_dir: Option<PathBuf>,

    /// Output file (defaults to schemas.dot)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: dot or svg
    #[arg(short, long, default_value = "dot")]
    format: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let schema_dir = cli.schema_dir.unwrap_or_else(|| PathBuf::from("."));

    println!("Loading schema graph from: {:?}", schema_dir);
    let graph = SchemaGraph::from_directory(&schema_dir)?;

    println!("Graph loaded: {} schemas, {} edges",
             graph.schema_count(),
             graph.edge_count());

    let dot_content = graph.to_dot();

    match cli.format.as_str() {
        "dot" => {
            let output_path = cli.output.unwrap_or_else(|| PathBuf::from("schemas.dot"));
            std::fs::write(&output_path, &dot_content)?;
            println!("✅ Exported DOT to: {:?}", output_path);
        }
        "svg" => {
            let output_path = cli.output.unwrap_or_else(|| PathBuf::from("schemas.svg"));

            // Write DOT to temp file, then convert to SVG
            let temp_dot = output_path.with_extension("temp.dot");
            std::fs::write(&temp_dot, &dot_content)?;

            // Use graphviz to convert DOT to SVG
            let output = std::process::Command::new("dot")
                .args(["-Tsvg", temp_dot.to_str().unwrap(), "-o", output_path.to_str().unwrap()])
                .output()?;

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_dot);

            if output.status.success() {
                println!("✅ Exported SVG to: {:?}", output_path);
            } else {
                eprintln!("❌ GraphViz conversion failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("❌ Invalid format. Use 'dot' or 'svg'");
            std::process::exit(1);
        }
    }

    Ok(())
}
