//! Schema Registry
//!
//! Manages versioned, append-only schema storage with Git-backed immutability.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use git2::{Repository, Signature, Commit, Oid};

use crate::schema::{Schema, SchemaEntry, SchemaType, VersionManifest};
use crate::version::SchemaVersion;
use crate::compatibility::{CompatibilityChecker, CompatibilityResult};
use crate::error::{Result, SchemaError};

/// The main schema registry
pub struct SchemaRegistry {
    /// Path to the registry root
    root: PathBuf,
    /// Git repository
    repo: Repository,
    /// Cached version manifests
    manifests: HashMap<String, VersionManifest>,
}

impl SchemaRegistry {
    /// Open an existing registry or create a new one
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        
        // Ensure versions directory exists
        fs::create_dir_all(root.join("versions"))?;
        
        // Open or init git repository
        let repo = match Repository::open(&root) {
            Ok(repo) => repo,
            Err(_) => Repository::init(&root)?,
        };

        let mut registry = Self {
            root,
            repo,
            manifests: HashMap::new(),
        };

        // Load existing manifests
        registry.load_manifests()?;

        Ok(registry)
    }

    /// Get the root path of the registry
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get all available versions, sorted
    pub fn versions(&self) -> Vec<&SchemaVersion> {
        let mut versions: Vec<_> = self.manifests.values().map(|m| &m.version).collect();
        versions.sort();
        versions
    }

    /// Get the latest version
    pub fn latest_version(&self) -> Option<&SchemaVersion> {
        self.versions().last().copied()
    }

    /// Get a manifest by version
    pub fn get_manifest(&self, version: &str) -> Option<&VersionManifest> {
        let version_str = version.strip_prefix('v').unwrap_or(version);
        self.manifests.get(version_str)
    }

    /// Get a schema by name and version
    pub fn get_schema(&self, name: &str, version: Option<&str>) -> Option<&SchemaEntry> {
        let version_str = match version {
            Some(v) => v.strip_prefix('v').unwrap_or(v).to_string(),
            None => self.latest_version()?.version_string(),
        };
        
        self.manifests
            .get(&version_str)
            .and_then(|m| m.get_schema(name))
    }

    /// Get all schemas of a specific type from the latest version
    pub fn get_schemas_by_type(&self, schema_type: SchemaType) -> Vec<&SchemaEntry> {
        self.latest_version()
            .and_then(|v| self.manifests.get(&v.version_string()))
            .map(|m| m.get_schemas_by_type(schema_type))
            .unwrap_or_default()
    }

    /// Register a new version with schemas
    /// 
    /// This is an append-only operation - existing versions cannot be modified
    /// 
    /// Directory structure: versions/{version}/{schema_type}/{category}/{filename}
    /// Example: versions/v0.3.0/json-schema/auth/User.schema.json
    pub fn register_version(
        &mut self,
        version: SchemaVersion,
        schemas: Vec<Schema>,
        author: Option<&str>,
        message: Option<&str>,
    ) -> Result<()> {
        let version_str = version.version_string();
        
        // Check if version already exists - IMMUTABILITY CHECK
        if self.manifests.contains_key(&version_str) {
            return Err(SchemaError::ImmutabilityViolation {
                name: "version".to_string(),
                version: version_str,
            });
        }

        // Create version directory
        let version_dir = self.root.join("versions").join(version.dir_name());
        if version_dir.exists() {
            return Err(SchemaError::AlreadyExists {
                name: "version".to_string(),
                version: version_str,
            });
        }
        fs::create_dir_all(&version_dir)?;

        // Create schema entries
        let mut entries = Vec::new();
        for schema in schemas {
            let mut entry = SchemaEntry::new(schema, version.clone());
            entry.created_by = author.map(String::from);
            
            // Directory structure: {schema_type}/{category}/{filename}
            // e.g., json-schema/auth/User.schema.json
            let type_dir = version_dir.join(entry.schema.schema_type.dir_name());
            let category_dir = type_dir.join(&entry.schema.category);
            fs::create_dir_all(&category_dir)?;
            
            let schema_path = category_dir.join(entry.schema.filename());
            let content = serde_json::to_string_pretty(&entry.schema.content)?;
            fs::write(&schema_path, &content)?;
            
            entries.push(entry);
        }

        // Create manifest
        let manifest = VersionManifest::new(version.clone(), entries);
        let manifest_path = version_dir.join("manifest.json");
        let manifest_content = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, &manifest_content)?;

        // Create checksums file
        let checksums_path = version_dir.join("checksums.sha256");
        let checksums_content: String = manifest
            .schemas
            .iter()
            .map(|s| format!("{}  {}/{}/{}", 
                s.checksum, 
                s.schema.schema_type.dir_name(),
                s.schema.category,
                s.schema.filename()
            ))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&checksums_path, &checksums_content)?;

        // Update latest symlink
        let latest_link = self.root.join("versions").join("latest");
        if latest_link.exists() {
            fs::remove_file(&latest_link)?;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(version.dir_name(), &latest_link)?;

        // Git commit
        self.git_commit(
            &format!("Release {}", version.dir_name()),
            author,
            message,
        )?;

        // Git tag
        self.git_tag(&version.tag_string(), message)?;

        // Cache manifest
        self.manifests.insert(version_str, manifest);

        Ok(())
    }

    /// Check compatibility between two versions
    pub fn check_compatibility(
        &self,
        old_version: &str,
        new_version: &str,
    ) -> Result<HashMap<String, CompatibilityResult>> {
        let old_manifest = self.manifests.get(old_version.strip_prefix('v').unwrap_or(old_version))
            .ok_or_else(|| SchemaError::NotFound {
                name: "version".to_string(),
                version: old_version.to_string(),
            })?;

        let new_manifest = self.manifests.get(new_version.strip_prefix('v').unwrap_or(new_version))
            .ok_or_else(|| SchemaError::NotFound {
                name: "version".to_string(),
                version: new_version.to_string(),
            })?;

        let checker = CompatibilityChecker::new();
        let mut results = HashMap::new();

        for old_entry in &old_manifest.schemas {
            if let Some(new_entry) = new_manifest.get_schema(&old_entry.schema.name) {
                let result = checker.check(old_entry, new_entry)?;
                results.insert(old_entry.schema.name.clone(), result);
            } else {
                // Schema was removed
                results.insert(
                    old_entry.schema.name.clone(),
                    CompatibilityResult::incompatible(
                        vec![],
                        format!("Schema '{}' was removed", old_entry.schema.name),
                    ),
                );
            }
        }

        Ok(results)
    }

    /// Verify all checksums in a version
    pub fn verify_version(&self, version: &str) -> Result<bool> {
        let manifest = self.manifests.get(version.strip_prefix('v').unwrap_or(version))
            .ok_or_else(|| SchemaError::NotFound {
                name: "version".to_string(),
                version: version.to_string(),
            })?;

        Ok(manifest.verify_all())
    }

    /// Load all manifests from disk
    fn load_manifests(&mut self) -> Result<()> {
        let versions_dir = self.root.join("versions");
        if !versions_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip symlinks and non-directories
            if path.is_symlink() || !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                let content = fs::read_to_string(&manifest_path)?;
                let manifest: VersionManifest = serde_json::from_str(&content)?;
                self.manifests.insert(manifest.version.version_string(), manifest);
            }
        }

        Ok(())
    }

    /// Create a Git commit
    fn git_commit(&self, summary: &str, author: Option<&str>, message: Option<&str>) -> Result<Oid> {
        let mut index = self.repo.index()?;
        index.add_all(["versions/*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let oid = index.write_tree()?;
        let tree = self.repo.find_tree(oid)?;

        let author_name = author.unwrap_or("Schema Registry");
        let sig = Signature::now(author_name, "schemas@familiar.dev")?;

        let full_message = match message {
            Some(msg) => format!("{}\n\n{}", summary, msg),
            None => summary.to_string(),
        };

        let parent_commit = self.get_head_commit();
        let parents: Vec<&Commit> = parent_commit.iter().collect();

        let commit_oid = self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &full_message,
            &tree,
            &parents,
        )?;

        Ok(commit_oid)
    }

    /// Create a Git tag
    fn git_tag(&self, tag_name: &str, message: Option<&str>) -> Result<()> {
        let obj = self.repo.revparse_single("HEAD")?;
        let sig = Signature::now("Schema Registry", "schemas@familiar.dev")?;

        let default_message = format!("Release {}", tag_name);
        let tag_message = message.unwrap_or(&default_message);
        self.repo.tag(tag_name, &obj, &sig, tag_message, false)?;

        Ok(())
    }

    /// Get the HEAD commit if it exists
    fn get_head_commit(&self) -> Option<Commit<'_>> {
        self.repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok())
    }

    /// Export schemas to a directory (for external consumption)
    pub fn export_version(&self, version: &str, output_dir: impl AsRef<Path>) -> Result<()> {
        let manifest = self.manifests.get(version.strip_prefix('v').unwrap_or(version))
            .ok_or_else(|| SchemaError::NotFound {
                name: "version".to_string(),
                version: version.to_string(),
            })?;

        let output = output_dir.as_ref();
        fs::create_dir_all(output)?;

        for entry in &manifest.schemas {
            // Export to: {schema_type}/{category}/{filename}
            let type_dir = output.join(entry.schema.schema_type.dir_name());
            let category_dir = type_dir.join(&entry.schema.category);
            fs::create_dir_all(&category_dir)?;

            let schema_path = category_dir.join(entry.schema.filename());
            let content = serde_json::to_string_pretty(&entry.schema.content)?;
            fs::write(&schema_path, &content)?;
        }

        // Write manifest
        let manifest_path = output.join("manifest.json");
        let manifest_content = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, &manifest_content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_registry() {
        let dir = tempdir().unwrap();
        let registry = SchemaRegistry::open(dir.path()).unwrap();
        assert!(registry.versions().is_empty());
    }

    #[test]
    fn test_register_version() {
        let dir = tempdir().unwrap();
        let mut registry = SchemaRegistry::open(dir.path()).unwrap();

        let version = SchemaVersion::parse("0.1.0").unwrap();
        let schema = Schema::with_category(
            "TestSchema",
            SchemaType::JsonSchema,
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            }),
            "test",
        );

        registry.register_version(version, vec![schema], None, None).unwrap();

        assert_eq!(registry.versions().len(), 1);
        assert!(registry.get_schema("TestSchema", Some("0.1.0")).is_some());
    }

    #[test]
    fn test_immutability() {
        let dir = tempdir().unwrap();
        let mut registry = SchemaRegistry::open(dir.path()).unwrap();

        let version = SchemaVersion::parse("0.1.0").unwrap();
        let schema = Schema::with_category("Test", SchemaType::JsonSchema, serde_json::json!({}), "test");

        registry.register_version(version.clone(), vec![schema.clone()], None, None).unwrap();

        // Try to register same version again - should fail
        let result = registry.register_version(version, vec![schema], None, None);
        assert!(result.is_err());
    }
}
