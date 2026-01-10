use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> std::io::Result<()> {
    let root_dir = Path::new(".");

    // File extension to section mapping
    let file_types = vec![
        ("rs", "Rust Source Files"),
        ("toml", "Cargo Configuration Files"),
        ("ncl", "Nickel Configuration Files"),
        ("json", "JSON Schema Files"),
    ];

    let mut sections: BTreeMap<&str, Vec<(PathBuf, String)>> = BTreeMap::new();

    // Initialize sections
    for (_, section_name) in &file_types {
        sections.insert(*section_name, Vec::new());
    }

    // Walk the directory tree
    fn visit_dirs(dir: &Path, sections: &mut BTreeMap<&str, Vec<(PathBuf, String)>>, file_types: &[(&str, &str)]) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let path_str = path.to_string_lossy();

                if path.is_dir() {
                    // Only allow specific top-level directories
                    if let Some(dir_name) = path.file_name() {
                        let dir_str = dir_name.to_string_lossy();
                        if dir_str == "target" || dir_str == ".git" || dir_str.starts_with('.') {
                            continue;
                        }
                        // Allow traversal into: src, packages, tests, and versions (with restrictions)
                        match dir_str.as_ref() {
                            "src" | "packages" | "tests" => {
                                // Allow all subdirs under these
                            },
                            "versions" => {
                                // Allow traversal into versions directory, but we'll filter files later
                            },
                            _ => {
                                // For subdirectories, check if we're in an allowed parent
                                if !path_str.contains("/src/") && !path_str.contains("/packages/") && !path_str.contains("/tests/") && !path_str.contains("/versions/") {
                                    continue;
                                }
                                // Additional check for versions: only allow v1.1.0-alpha
                                if path_str.contains("/versions/") && !path_str.contains("/v1.1.0-alpha/") && dir_str != "v1.1.0-alpha" {
                                    continue;
                                }
                            }
                        }
                    }
                    visit_dirs(&path, sections, file_types)?;
                } else if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy();

                    for (target_ext, section_name) in file_types {
                        if ext_str == *target_ext {
                            // Only include Cargo.toml files
                            if *target_ext == "toml" && path.file_name().unwrap_or_default() != "Cargo.toml" {
                                continue;
                            }

                            // For Rust files: from src/, packages/, and tests/
                            if *target_ext == "rs" {
                                if !path_str.contains("/src/") && !path_str.contains("/packages/") && !path_str.contains("/tests/") {
                                    continue;
                                }
                            }

                            // For JSON and Nickel files: only from v1.1.0-alpha/
                            if (*target_ext == "json" || *target_ext == "ncl") {
                                if !path_str.contains("/v1.1.0-alpha/") {
                                    continue;
                                }
                            }

                            // Include test files (don't skip them)

                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    sections.get_mut(*section_name).unwrap().push((path.clone(), content));
                                }
                                Err(e) => {
                                    eprintln!("Warning: Could not read file {:?}: {}", path, e);
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // Visit all directories
    visit_dirs(root_dir, &mut sections, &file_types)?;

    // Print statistics first
    println!("Found (filtered):");
    for (section_name, files) in &sections {
        println!("  {}: {} files", section_name, files.len());
    }

    // Generate markdown
    let mut markdown = String::new();
    markdown.push_str("# Familiar Schemas Code Collection (Focused)\n\n");
    markdown.push_str("This document contains focused Rust, Cargo, Nickel, and JSON Schema code from the familiar-schemas crate.\n\n");
    markdown.push_str("**Focused selection to stay under 1M tokens:**\n");
    markdown.push_str("- **Rust:** `src/`, `packages/`, and `tests/` directories\n");
    markdown.push_str("- **Cargo:** Only `Cargo.toml` files\n");
    markdown.push_str("- **Nickel:** Only `versions/v1.1.0-alpha/` directory\n");
    markdown.push_str("- **JSON Schema:** Only `versions/v1.1.0-alpha/` directory\n");
    markdown.push_str("- **Excluded:** Older schema versions and unrelated directories\n\n");
    markdown.push_str("Generated automatically from the codebase.\n\n");

    for (section_name, files) in &sections {
        if files.is_empty() {
            continue;
        }

        markdown.push_str(&format!("## {}\n\n", section_name));

        // Sort files by path for consistent output
        let mut sorted_files = files.clone();
        sorted_files.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (path, content) in sorted_files {
            let relative_path = path.strip_prefix(root_dir).unwrap_or(&path);
            markdown.push_str(&format!("### {}\n\n", relative_path.display()));
            markdown.push_str("```");
            match *section_name {
                "Rust Source Files" => markdown.push_str("rust"),
                "Cargo Configuration Files" => markdown.push_str("toml"),
                "Nickel Configuration Files" => markdown.push_str("nickel"),
                "JSON Schema Files" => markdown.push_str("json"),
                _ => {}
            }
            markdown.push_str(&format!("\n{}\n```\n\n", content));
        }
    }

    // Write to file
    let mut file = File::create("familiar_schemas_code_filtered.md")?;
    file.write_all(markdown.as_bytes())?;

    println!("Filtered markdown file generated: familiar_schemas_code_filtered.md");
    Ok(())
}