//! Schema Drift Detection CLI
//!
//! Compares currently generated schemas against stored schemas in the registry.
//! Detects drift and reports changes for manual review.
//!
//! Usage:
//!   schema-drift --workspace /path/to/familiar/docs/v4 --registry /path/to/familiar-schemas
//!   schema-drift --help

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs;

use clap::Parser;
use ignore::WalkBuilder;
use familiar_schemas::{SchemaRegistry, SchemaType};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "schema-drift")]
#[command(about = "Detect schema drift between current Rust types and stored schemas")]
struct Cli {
    /// Path to the workspace root (contains familiar-core, familiar-primitives, etc.)
    #[arg(short, long)]
    workspace: PathBuf,

    /// Path to the schema registry
    #[arg(short, long, default_value = ".")]
    registry: PathBuf,

    /// Version to compare against (default: latest)
    #[arg(short = 'v', long)]
    version: Option<String>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Fail on any changes (not just breaking)
    #[arg(long)]
    strict: bool,

    /// Verbose output
    #[arg(long)]
    verbose: bool,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Schema Drift Detection\n");

    // Load stored schemas from registry
    println!("📂 Loading stored schemas from: {:?}", cli.registry);
    let registry = SchemaRegistry::open(&cli.registry)?;
    
    let version_str = cli.version.as_deref();
    let stored_schemas = load_stored_schemas(&registry, version_str)?;
    println!("   Found {} stored schemas\n", stored_schemas.len());

    // Generate fresh schemas from current source
    println!("📂 Scanning workspace: {:?}", cli.workspace);
    let current_schemas = scan_current_schemas(&cli.workspace, cli.verbose)?;
    println!("   Found {} current schemas\n", current_schemas.len());

    // Compare
    let drift_report = compare_schemas(&stored_schemas, &current_schemas, cli.strict);

    // Output report
    match cli.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&drift_report)?);
        }
        _ => {
            print_text_report(&drift_report, cli.verbose);
        }
    }

    // Exit code based on drift
    if drift_report.has_breaking_changes {
        eprintln!("\n❌ BREAKING CHANGES DETECTED - Schema update required!");
        std::process::exit(2);
    } else if drift_report.has_any_changes && cli.strict {
        eprintln!("\n⚠️  Changes detected (strict mode) - Review required");
        std::process::exit(1);
    } else if drift_report.has_any_changes {
        eprintln!("\n⚠️  Non-breaking changes detected - Consider updating schema version");
        std::process::exit(0);
    } else {
        eprintln!("\n✅ No drift detected - Schemas are in sync");
        std::process::exit(0);
    }
}

/// Load stored schemas from the registry
fn load_stored_schemas(
    registry: &SchemaRegistry,
    version: Option<&str>,
) -> Result<HashMap<String, StoredSchema>, Box<dyn std::error::Error>> {
    let mut schemas = HashMap::new();
    
    // Get the manifest for the specified version (or latest)
    let manifest = if let Some(v) = version {
        registry.get_manifest(v)
            .ok_or_else(|| format!("Version '{}' not found in registry", v))?
    } else {
        registry.get_manifest("latest")
            .or_else(|| registry.latest_version().and_then(|v| registry.get_manifest(&v.version_string())))
            .ok_or("No versions found in registry")?
    };

    // Load each schema entry
    for entry in &manifest.schemas {
        schemas.insert(entry.schema.name.clone(), StoredSchema {
            name: entry.schema.name.clone(),
            schema_type: entry.schema.schema_type,
            content: entry.schema.content.clone(),
            checksum: entry.checksum.to_string(),
        });
    }

    Ok(schemas)
}

#[derive(Debug, Clone)]
struct StoredSchema {
    name: String,
    schema_type: SchemaType,
    content: serde_json::Value,
    checksum: String,
}

/// Scan current schemas from the workspace
fn scan_current_schemas(
    workspace: &PathBuf,
    verbose: bool,
) -> Result<HashMap<String, CurrentSchema>, Box<dyn std::error::Error>> {
    let mut schemas = HashMap::new();

    // Directories to scan for generated schemas
    let schema_dirs = [
        ("familiar-primitives/generated/schemas", "primitives"),
        ("familiar-contracts/generated/schemas", "contracts"),
        ("familiar-core/generated/schemas", "core"),
    ];

    for (rel_path, category) in schema_dirs {
        let dir = workspace.join(rel_path);
        if !dir.exists() {
            if verbose {
                eprintln!("   ⚠️  Directory not found: {:?}", dir);
            }
            continue;
        }

        // Use ignore crate for fast walking
        let walker = WalkBuilder::new(&dir)
            .hidden(false)
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // Only process .json files
            if !path.is_file() {
                continue;
            }
            
            let filename = match path.file_name() {
                Some(f) => f.to_string_lossy(),
                None => continue,
            };
            
            if !filename.ends_with(".json") || filename == "manifest.json" {
                continue;
            }

            // Read and parse schema
            let content_str = fs::read_to_string(path)?;
            let content: serde_json::Value = serde_json::from_str(&content_str)?;
            
            // Extract name from filename
            let name = filename
                .trim_end_matches(".schema.json")
                .trim_end_matches(".json")
                .to_string();

            schemas.insert(name.clone(), CurrentSchema {
                name,
                category: category.to_string(),
                content,
                path: path.to_path_buf(),
            });
        }
    }

    Ok(schemas)
}

#[derive(Debug, Clone)]
struct CurrentSchema {
    name: String,
    #[allow(dead_code)]
    category: String,
    content: serde_json::Value,
    #[allow(dead_code)]
    path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DriftReport {
    has_any_changes: bool,
    has_breaking_changes: bool,
    added: Vec<String>,
    removed: Vec<String>,
    changed: Vec<SchemaChange>,
    unchanged: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SchemaChange {
    name: String,
    is_breaking: bool,
    changes: Vec<String>,
}

/// Compare stored vs current schemas
fn compare_schemas(
    stored: &HashMap<String, StoredSchema>,
    current: &HashMap<String, CurrentSchema>,
    _strict: bool,
) -> DriftReport {
    let stored_names: HashSet<_> = stored.keys().cloned().collect();
    let current_names: HashSet<_> = current.keys().cloned().collect();

    // Find added schemas (in current but not stored)
    let added: Vec<_> = current_names.difference(&stored_names).cloned().collect();

    // Find removed schemas (in stored but not current)
    let removed: Vec<_> = stored_names.difference(&current_names).cloned().collect();

    // Find changed schemas (in both, but content differs)
    let mut changed = Vec::new();
    let mut unchanged_count = 0;

    for name in stored_names.intersection(&current_names) {
        let stored_schema = &stored[name];
        let current_schema = &current[name];

        // Compare content
        if stored_schema.content == current_schema.content {
            unchanged_count += 1;
            continue;
        }

        // Detect what changed
        let changes = detect_changes(&stored_schema.content, &current_schema.content);
        
        // Check if breaking
        let is_breaking = changes.iter().any(|c| 
            c.contains("removed") || c.contains("type changed") || c.contains("now required")
        );

        changed.push(SchemaChange {
            name: name.clone(),
            is_breaking,
            changes,
        });
    }

    let has_any_changes = !added.is_empty() || !removed.is_empty() || !changed.is_empty();
    let has_breaking_changes = !removed.is_empty() || changed.iter().any(|c| c.is_breaking);

    DriftReport {
        has_any_changes,
        has_breaking_changes,
        added,
        removed,
        changed,
        unchanged: unchanged_count,
    }
}

/// Detect specific changes between two JSON values
fn detect_changes(old: &serde_json::Value, new: &serde_json::Value) -> Vec<String> {
    let mut changes = Vec::new();

    // Compare properties if both are objects with properties
    if let (Some(old_props), Some(new_props)) = (
        old.get("properties").and_then(|p| p.as_object()),
        new.get("properties").and_then(|p| p.as_object()),
    ) {
        // Check for removed properties
        for key in old_props.keys() {
            if !new_props.contains_key(key) {
                changes.push(format!("Property '{}' removed", key));
            }
        }

        // Check for added properties
        for key in new_props.keys() {
            if !old_props.contains_key(key) {
                changes.push(format!("Property '{}' added", key));
            }
        }

        // Check for type changes
        for key in old_props.keys() {
            if let (Some(old_prop), Some(new_prop)) = (old_props.get(key), new_props.get(key)) {
                let old_type = old_prop.get("type");
                let new_type = new_prop.get("type");
                if old_type != new_type {
                    changes.push(format!(
                        "Property '{}' type changed: {:?} -> {:?}",
                        key, old_type, new_type
                    ));
                }
            }
        }
    }

    // Check required changes
    if let (Some(old_req), Some(new_req)) = (
        old.get("required").and_then(|r| r.as_array()),
        new.get("required").and_then(|r| r.as_array()),
    ) {
        let old_set: HashSet<_> = old_req.iter().filter_map(|v| v.as_str()).collect();
        let new_set: HashSet<_> = new_req.iter().filter_map(|v| v.as_str()).collect();

        for added_req in new_set.difference(&old_set) {
            changes.push(format!("Property '{}' is now required", added_req));
        }
        for removed_req in old_set.difference(&new_set) {
            changes.push(format!("Property '{}' is no longer required", removed_req));
        }
    }

    if changes.is_empty() {
        changes.push("Content changed (details unavailable)".to_string());
    }

    changes
}

/// Print human-readable report
fn print_text_report(report: &DriftReport, verbose: bool) {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("                     SCHEMA DRIFT REPORT                           ");
    println!("═══════════════════════════════════════════════════════════════════\n");

    if !report.added.is_empty() {
        println!("📗 NEW SCHEMAS ({}):", report.added.len());
        for name in &report.added {
            println!("   + {}", name);
        }
        println!();
    }

    if !report.removed.is_empty() {
        println!("📕 REMOVED SCHEMAS ({}) [BREAKING]:", report.removed.len());
        for name in &report.removed {
            println!("   - {}", name);
        }
        println!();
    }

    if !report.changed.is_empty() {
        let breaking_count = report.changed.iter().filter(|c| c.is_breaking).count();
        let non_breaking_count = report.changed.len() - breaking_count;

        if breaking_count > 0 {
            println!("🔴 BREAKING CHANGES ({}):", breaking_count);
            for change in report.changed.iter().filter(|c| c.is_breaking) {
                println!("   ⚠️  {}", change.name);
                if verbose {
                    for c in &change.changes {
                        println!("      - {}", c);
                    }
                }
            }
            println!();
        }

        if non_breaking_count > 0 {
            println!("🟡 NON-BREAKING CHANGES ({}):", non_breaking_count);
            for change in report.changed.iter().filter(|c| !c.is_breaking) {
                println!("   📝 {}", change.name);
                if verbose {
                    for c in &change.changes {
                        println!("      - {}", c);
                    }
                }
            }
            println!();
        }
    }

    println!("📊 SUMMARY:");
    println!("   Unchanged: {}", report.unchanged);
    println!("   Added:     {}", report.added.len());
    println!("   Removed:   {} {}", report.removed.len(), 
        if !report.removed.is_empty() { "[BREAKING]" } else { "" });
    println!("   Changed:   {}", report.changed.len());
    
    let breaking_count = report.changed.iter().filter(|c| c.is_breaking).count();
    if breaking_count > 0 {
        println!("   Breaking:  {} [BREAKING]", breaking_count);
    }
}
