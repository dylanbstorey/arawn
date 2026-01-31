//! Runtime catalog for managing WASM runtime modules.
//!
//! The catalog persists as `catalog.toml` alongside `builtin/` and `custom/`
//! directories that hold `.wasm` modules.
//!
//! ```text
//! <runtimes_dir>/
//!   catalog.toml
//!   builtin/
//!     passthrough.wasm
//!     http.wasm
//!   custom/
//!     my_transform.wasm
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::PipelineError;

/// Category of a runtime module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCategory {
    Builtin,
    Custom,
}

/// A single runtime entry in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Relative path to the `.wasm` file (relative to the runtimes directory).
    pub path: String,
    /// Whether this is a built-in or custom runtime.
    pub category: RuntimeCategory,
}

/// Serialization wrapper for the catalog TOML file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CatalogFile {
    /// Map of runtime name → entry.
    #[serde(default)]
    runtimes: BTreeMap<String, CatalogEntry>,
}

/// In-memory runtime catalog with CRUD operations and persistence.
pub struct RuntimeCatalog {
    /// Root directory containing catalog.toml, builtin/, custom/.
    root: PathBuf,
    /// In-memory entries.
    entries: BTreeMap<String, CatalogEntry>,
}

impl RuntimeCatalog {
    /// Load or initialize a catalog from the given runtimes directory.
    ///
    /// Creates `builtin/` and `custom/` subdirectories if they don't exist.
    /// If `catalog.toml` doesn't exist, starts with an empty catalog.
    pub fn load(root: &Path) -> Result<Self, PipelineError> {
        // Ensure directory structure
        let builtin_dir = root.join("builtin");
        let custom_dir = root.join("custom");
        for dir in [root, &builtin_dir, &custom_dir] {
            std::fs::create_dir_all(dir).map_err(|e| {
                PipelineError::InvalidWorkflow(format!(
                    "Failed to create directory {}: {}",
                    dir.display(),
                    e
                ))
            })?;
        }

        let catalog_path = root.join("catalog.toml");
        let entries = if catalog_path.exists() {
            let content = std::fs::read_to_string(&catalog_path).map_err(|e| {
                PipelineError::InvalidWorkflow(format!(
                    "Failed to read {}: {}",
                    catalog_path.display(),
                    e
                ))
            })?;
            let file: CatalogFile = toml::from_str(&content).map_err(|e| {
                PipelineError::InvalidWorkflow(format!("Invalid catalog TOML: {}", e))
            })?;
            file.runtimes
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            root: root.to_path_buf(),
            entries,
        })
    }

    /// Persist the current catalog to `catalog.toml`.
    pub fn save(&self) -> Result<(), PipelineError> {
        let file = CatalogFile {
            runtimes: self.entries.clone(),
        };
        let content = toml::to_string_pretty(&file).map_err(|e| {
            PipelineError::InvalidWorkflow(format!("Failed to serialize catalog: {}", e))
        })?;
        let path = self.root.join("catalog.toml");
        std::fs::write(&path, content).map_err(|e| {
            PipelineError::InvalidWorkflow(format!("Failed to write {}: {}", path.display(), e))
        })?;
        Ok(())
    }

    /// Add or update a runtime entry and persist.
    pub fn add(&mut self, name: &str, entry: CatalogEntry) -> Result<(), PipelineError> {
        self.entries.insert(name.to_string(), entry);
        self.save()
    }

    /// Remove a runtime entry and persist. Returns the removed entry if it existed.
    pub fn remove(&mut self, name: &str) -> Result<Option<CatalogEntry>, PipelineError> {
        let removed = self.entries.remove(name);
        self.save()?;
        Ok(removed)
    }

    /// Get a runtime entry by name.
    pub fn get(&self, name: &str) -> Option<&CatalogEntry> {
        self.entries.get(name)
    }

    /// List all runtime entries.
    pub fn list(&self) -> &BTreeMap<String, CatalogEntry> {
        &self.entries
    }

    /// Resolve the absolute path to a runtime's `.wasm` file.
    pub fn resolve_path(&self, name: &str) -> Option<PathBuf> {
        self.entries.get(name).map(|e| self.root.join(&e.path))
    }

    /// The root directory of the catalog.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_creates_directories() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("runtimes");
        let _catalog = RuntimeCatalog::load(&root).unwrap();
        assert!(root.join("builtin").is_dir());
        assert!(root.join("custom").is_dir());
    }

    #[test]
    fn test_empty_catalog() {
        let tmp = TempDir::new().unwrap();
        let catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        assert!(catalog.list().is_empty());
        assert!(catalog.get("nonexistent").is_none());
    }

    #[test]
    fn test_add_and_get() {
        let tmp = TempDir::new().unwrap();
        let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        catalog
            .add(
                "http",
                CatalogEntry {
                    description: "HTTP fetch runtime".into(),
                    path: "builtin/http.wasm".into(),
                    category: RuntimeCategory::Builtin,
                },
            )
            .unwrap();

        let entry = catalog.get("http").unwrap();
        assert_eq!(entry.description, "HTTP fetch runtime");
        assert_eq!(entry.category, RuntimeCategory::Builtin);
    }

    #[test]
    fn test_remove() {
        let tmp = TempDir::new().unwrap();
        let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        catalog
            .add(
                "temp",
                CatalogEntry {
                    description: "".into(),
                    path: "custom/temp.wasm".into(),
                    category: RuntimeCategory::Custom,
                },
            )
            .unwrap();
        assert!(catalog.get("temp").is_some());

        let removed = catalog.remove("temp").unwrap();
        assert!(removed.is_some());
        assert!(catalog.get("temp").is_none());
    }

    #[test]
    fn test_remove_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        let removed = catalog.remove("nope").unwrap();
        assert!(removed.is_none());
    }

    #[test]
    fn test_list_returns_all() {
        let tmp = TempDir::new().unwrap();
        let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        catalog
            .add(
                "a",
                CatalogEntry {
                    description: "".into(),
                    path: "builtin/a.wasm".into(),
                    category: RuntimeCategory::Builtin,
                },
            )
            .unwrap();
        catalog
            .add(
                "b",
                CatalogEntry {
                    description: "".into(),
                    path: "custom/b.wasm".into(),
                    category: RuntimeCategory::Custom,
                },
            )
            .unwrap();
        assert_eq!(catalog.list().len(), 2);
    }

    #[test]
    fn test_roundtrip_persistence() {
        let tmp = TempDir::new().unwrap();
        {
            let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
            catalog
                .add(
                    "http",
                    CatalogEntry {
                        description: "Fetch URLs".into(),
                        path: "builtin/http.wasm".into(),
                        category: RuntimeCategory::Builtin,
                    },
                )
                .unwrap();
            catalog
                .add(
                    "my_transform",
                    CatalogEntry {
                        description: "Custom transform".into(),
                        path: "custom/my_transform.wasm".into(),
                        category: RuntimeCategory::Custom,
                    },
                )
                .unwrap();
        }
        // Reload from disk
        let catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        assert_eq!(catalog.list().len(), 2);
        let http = catalog.get("http").unwrap();
        assert_eq!(http.description, "Fetch URLs");
        assert_eq!(http.category, RuntimeCategory::Builtin);
    }

    #[test]
    fn test_resolve_path() {
        let tmp = TempDir::new().unwrap();
        let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        catalog
            .add(
                "http",
                CatalogEntry {
                    description: "".into(),
                    path: "builtin/http.wasm".into(),
                    category: RuntimeCategory::Builtin,
                },
            )
            .unwrap();
        let resolved = catalog.resolve_path("http").unwrap();
        assert_eq!(resolved, tmp.path().join("builtin/http.wasm"));
        assert!(catalog.resolve_path("missing").is_none());
    }

    #[test]
    fn test_add_overwrites() {
        let tmp = TempDir::new().unwrap();
        let mut catalog = RuntimeCatalog::load(tmp.path()).unwrap();
        catalog
            .add(
                "rt",
                CatalogEntry {
                    description: "v1".into(),
                    path: "custom/rt.wasm".into(),
                    category: RuntimeCategory::Custom,
                },
            )
            .unwrap();
        catalog
            .add(
                "rt",
                CatalogEntry {
                    description: "v2".into(),
                    path: "custom/rt.wasm".into(),
                    category: RuntimeCategory::Custom,
                },
            )
            .unwrap();
        assert_eq!(catalog.get("rt").unwrap().description, "v2");
        assert_eq!(catalog.list().len(), 1);
    }
}
