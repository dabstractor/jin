//! Jinmap file tracking layer-to-file mappings.
//!
//! This module provides the `Jinmap` structure that tracks which files belong
//! to which layers. The `.jinmap` file is auto-generated after each commit
//! and provides a complete mapping of the Jin layer hierarchy.
//!
//! # Jinmap Format
//!
//! The `.jinmap` file is stored in YAML format at the workspace root:
//!
//! ```yaml
//! version: 1
//! mappings:
//!   "global": ["config/global.json"]
//!   "mode/claude": [".claude/", "CLAUDE.md"]
//!   "project/myapp": ["project-config.json"]
//! meta:
//!   generated_by: "jin"
//!   last_updated: "2025-12-26T10:00:00Z"
//! ```
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::commit::jinmap::Jinmap;
//!
//! // Load existing jinmap or create new
//! let mut jinmap = Jinmap::load_from_disk(&workspace_root)?;
//!
//! // Add a file to a layer
//! jinmap.add_mapping("mode/claude", ".claude/config.json");
//!
//! // Save to disk
//! jinmap.save_to_disk(&workspace_root)?;
//! ```

use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use chrono::{DateTime, Utc};
use indexmap::{map::serde_seq, IndexMap};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ===== JINMAP =====

/// Jinmap file tracking layer-to-file mappings.
///
/// Auto-generated after each commit to track which files
/// belong to which layers.
///
/// # Fields
///
/// - `version`: Format version (currently 1)
/// - `mappings`: Layer path -> list of files mapping (ordered for consistency)
/// - `meta`: Metadata about generation
///
/// # Examples
///
/// ```ignore
/// let jinmap = Jinmap::new();
/// jinmap.add_mapping("global", "config.json");
/// assert_eq!(jinmap.mappings.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jinmap {
    /// Format version
    pub version: u32,
    /// Layer path -> list of files mapping
    #[serde(with = "serde_seq")]
    pub mappings: IndexMap<String, Vec<String>>,
    /// Metadata about generation
    pub meta: JinmapMeta,
}

impl Default for Jinmap {
    fn default() -> Self {
        Self::new()
    }
}

impl Jinmap {
    /// Creates a new empty jinmap.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let jinmap = Jinmap::new();
    /// assert_eq!(jinmap.version, 1);
    /// assert!(jinmap.mappings.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            version: 1,
            mappings: IndexMap::new(),
            meta: JinmapMeta {
                generated_by: "jin".to_string(),
                last_updated: Utc::now(),
            },
        }
    }

    /// Adds or updates a file mapping for a layer.
    ///
    /// If the layer already exists, the file is added to its list
    /// (duplicates are removed). If the file is already present,
    /// the mapping is unchanged.
    ///
    /// # Arguments
    ///
    /// * `layer_path` - Layer path (without "refs/jin/layers/" prefix)
    /// * `file_path` - File path to add
    ///
    /// # Examples
    ///
    /// ```ignore
    /// jinmap.add_mapping("global", "config.json");
    /// jinmap.add_mapping("global", "settings.yaml");
    /// assert_eq!(jinmap.mappings["global"].len(), 2);
    /// ```
    pub fn add_mapping(&mut self, layer_path: &str, file_path: &str) {
        self.mappings
            .entry(layer_path.to_string())
            .or_insert_with(Vec::new)
            .push(file_path.to_string());

        // Remove duplicates and sort
        if let Some(files) = self.mappings.get_mut(layer_path) {
            files.sort();
            files.dedup();
        }

        self.meta.last_updated = Utc::now();
    }

    /// Removes a file from all layer mappings.
    ///
    /// If a layer's file list becomes empty after removal,
    /// the layer entry is removed from the mappings.
    ///
    /// # Arguments
    ///
    /// * `file_path` - File path to remove
    ///
    /// # Examples
    ///
    /// ```ignore
    /// jinmap.add_mapping("global", "config.json");
    /// jinmap.add_mapping("mode/claude", "config.json");
    /// jinmap.remove_mapping("config.json");
    /// assert!(jinmap.mappings.is_empty());
    /// ```
    pub fn remove_mapping(&mut self, file_path: &str) {
        for files in self.mappings.values_mut() {
            files.retain(|f| f != file_path);
        }

        // Remove empty layer entries
        self.mappings.retain(|_, files| !files.is_empty());

        self.meta.last_updated = Utc::now();
    }

    /// Loads jinmap from project root.
    ///
    /// Returns a new empty jinmap if the file doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    ///
    /// # Returns
    ///
    /// - `Ok(Jinmap)` - Loaded or new jinmap
    /// - `Err(JinError)` - Failed to load (parse error, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let jinmap = Jinmap::load_from_disk(&workspace_root)?;
    /// ```
    pub fn load_from_disk(workspace_root: &Path) -> Result<Self> {
        let jinmap_path = workspace_root.join(".jinmap");

        if !jinmap_path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(&jinmap_path)?;

        serde_yaml_ng::from_str(&content)
            .map_err(|e| JinError::Message(format!("Invalid .jinmap format: {}", e)))
    }

    /// Saves jinmap to project root.
    ///
    /// Creates or overwrites the `.jinmap` file at the workspace root.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Jinmap saved successfully
    /// - `Err(JinError)` - Failed to save
    ///
    /// # Examples
    ///
    /// ```ignore
    /// jinmap.save_to_disk(&workspace_root)?;
    /// ```
    pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()> {
        let jinmap_path = workspace_root.join(".jinmap");

        let yaml = serde_yaml_ng::to_string(self)
            .map_err(|e| JinError::Message(format!("Failed to serialize .jinmap: {}", e)))?;

        std::fs::write(&jinmap_path, yaml)?;

        Ok(())
    }

    /// Generates jinmap by scanning all layer refs in Jin repo.
    ///
    /// Walks all layer references and builds a complete mapping of
    /// files to their respective layers.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Jin repository
    /// * `project` - The project name
    ///
    /// # Returns
    ///
    /// - `Ok(Jinmap)` - Generated jinmap
    /// - `Err(JinError)` - Failed to generate
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let jinmap = Jinmap::generate_from_layers(&repo, "myapp")?;
    /// ```
    pub fn generate_from_layers(repo: &JinRepo, project: &str) -> Result<Self> {
        let mut jinmap = Self::new();

        // Get all layer refs
        let layer_refs = repo.list_layer_refs()?;

        for (layer, commit_oid) in layer_refs {
            // Get the commit's tree
            let commit = repo.find_commit(commit_oid)?;
            let tree_id = commit.tree_id();
            let tree = repo.find_tree(tree_id)?;

            // Walk the tree and collect files
            let mut files = Vec::new();
            repo.walk_tree(tree_id, |path, _entry| {
                files.push(path.to_string());
                Ok(())
            })?;

            // Get layer path (strip refs/jin/layers/ prefix)
            let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;
            let layer_path = ref_name.strip_prefix("refs/jin/layers/").ok_or_else(|| {
                JinError::Message(format!("Invalid layer ref format: {}", ref_name))
            })?;

            // Add mapping
            if !files.is_empty() {
                jinmap.mappings.insert(layer_path.to_string(), files);
            }
        }

        jinmap.meta.last_updated = Utc::now();
        Ok(jinmap)
    }

    /// Updates jinmap after a commit.
    ///
    /// Adds new files to the jinmap based on the committed entries.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer that was committed to
    /// * `files` - List of files that were committed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// jinmap.update_from_commit(&layer, &["config.json", "settings.yaml"]);
    /// ```
    pub fn update_from_commit(&mut self, layer: &Layer, files: &[String]) {
        let ref_name = match layer.git_ref() {
            Some(name) => name,
            None => return, // Skip unversioned layers
        };

        let layer_path = match ref_name.strip_prefix("refs/jin/layers/") {
            Some(path) => path,
            None => return,
        };

        for file in files {
            self.add_mapping(layer_path, file);
        }
    }
}

// ===== JINMAP META =====

/// Metadata section of .jinmap file.
///
/// Contains information about when and how the jinmap was generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinmapMeta {
    /// Tool that generated this file
    pub generated_by: String,
    /// When this file was last updated
    pub last_updated: DateTime<Utc>,
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ===== Jinmap Constructor Tests =====

    #[test]
    fn test_jinmap_new() {
        let jinmap = Jinmap::new();
        assert_eq!(jinmap.version, 1);
        assert!(jinmap.mappings.is_empty());
        assert_eq!(jinmap.meta.generated_by, "jin");
    }

    #[test]
    fn test_jinmap_default() {
        let jinmap = Jinmap::default();
        assert_eq!(jinmap.version, 1);
        assert!(jinmap.mappings.is_empty());
    }

    // ===== add_mapping Tests =====

    #[test]
    fn test_jinmap_add_mapping_single() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");

        assert_eq!(jinmap.mappings.len(), 1);
        assert_eq!(jinmap.mappings["global"], vec!["config.json"]);
    }

    #[test]
    fn test_jinmap_add_mapping_multiple_same_layer() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");
        jinmap.add_mapping("global", "settings.yaml");

        assert_eq!(jinmap.mappings["global"].len(), 2);
        // Check sorted order
        assert_eq!(jinmap.mappings["global"][0], "config.json");
        assert_eq!(jinmap.mappings["global"][1], "settings.yaml");
    }

    #[test]
    fn test_jinmap_add_mapping_duplicate() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");
        jinmap.add_mapping("global", "config.json"); // Duplicate

        assert_eq!(jinmap.mappings["global"].len(), 1);
    }

    #[test]
    fn test_jinmap_add_mapping_multiple_layers() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "global.json");
        jinmap.add_mapping("mode/claude", "claude.json");

        assert_eq!(jinmap.mappings.len(), 2);
        assert!(jinmap.mappings.contains_key("global"));
        assert!(jinmap.mappings.contains_key("mode/claude"));
    }

    // ===== remove_mapping Test =====

    #[test]
    fn test_jinmap_remove_mapping() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");
        jinmap.add_mapping("mode/claude", "config.json");

        jinmap.remove_mapping("config.json");

        assert!(jinmap.mappings.is_empty());
    }

    #[test]
    fn test_jinmap_remove_mapping_one_layer() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");
        jinmap.add_mapping("global", "settings.yaml");
        jinmap.add_mapping("mode/claude", "claude.json");

        jinmap.remove_mapping("config.json");

        assert_eq!(jinmap.mappings.len(), 2);
        assert_eq!(jinmap.mappings["global"], vec!["settings.yaml"]);
        assert_eq!(jinmap.mappings["mode/claude"], vec!["claude.json"]);
    }

    #[test]
    fn test_jinmap_remove_mapping_nonexistent() {
        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");

        // Removing non-existent file should not error
        jinmap.remove_mapping("nonexistent.json");

        assert_eq!(jinmap.mappings["global"], vec!["config.json"]);
    }

    // ===== Persistence Tests =====

    #[test]
    fn test_jinmap_persistence_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");
        jinmap.add_mapping("mode/claude", "claude.json");

        jinmap.save_to_disk(temp_dir.path()).unwrap();

        let loaded = Jinmap::load_from_disk(temp_dir.path()).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.mappings.len(), 2);
        assert_eq!(loaded.mappings["global"], vec!["config.json"]);
        assert_eq!(loaded.mappings["mode/claude"], vec!["claude.json"]);
    }

    #[test]
    fn test_jinmap_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();

        let jinmap = Jinmap::load_from_disk(temp_dir.path()).unwrap();

        // Should return new empty jinmap
        assert!(jinmap.mappings.is_empty());
        assert_eq!(jinmap.version, 1);
    }

    #[test]
    fn test_jinmap_yaml_format() {
        let temp_dir = TempDir::new().unwrap();

        let mut jinmap = Jinmap::new();
        jinmap.add_mapping("global", "config.json");

        jinmap.save_to_disk(temp_dir.path()).unwrap();

        let yaml_path = temp_dir.path().join(".jinmap");
        let content = fs::read_to_string(&yaml_path).unwrap();

        // Check YAML structure
        assert!(content.contains("version: 1"));
        assert!(content.contains("mappings:"));
        assert!(content.contains("config.json"));
        assert!(content.contains("generated_by: jin"));
    }

    // ===== update_from_commit Tests =====

    #[test]
    fn test_jinmap_update_from_commit() {
        let mut jinmap = Jinmap::new();

        let layer = Layer::GlobalBase;
        let files = vec!["config.json".to_string(), "settings.yaml".to_string()];

        jinmap.update_from_commit(&layer, &files);

        assert_eq!(jinmap.mappings.len(), 1);
        assert_eq!(jinmap.mappings["global"], files);
    }

    #[test]
    fn test_jinmap_update_from_commit_unversioned_layer() {
        let mut jinmap = Jinmap::new();

        let layer = Layer::UserLocal;
        let files = vec!["local.json".to_string()];

        jinmap.update_from_commit(&layer, &files);

        // UserLocal should be skipped (not versioned)
        assert!(jinmap.mappings.is_empty());
    }

    // ===== JinmapMeta Tests =====

    #[test]
    fn test_jinmap_meta_generated_by() {
        let jinmap = Jinmap::new();
        assert_eq!(jinmap.meta.generated_by, "jin");
    }

    #[test]
    fn test_jinmap_meta_last_updated() {
        let before = Utc::now();
        let jinmap = Jinmap::new();
        let after = Utc::now();

        assert!(jinmap.meta.last_updated >= before);
        assert!(jinmap.meta.last_updated <= after);
    }

    #[test]
    fn test_jinmap_meta_updates_on_add() {
        let mut jinmap = Jinmap::new();
        let first_time = jinmap.meta.last_updated;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        jinmap.add_mapping("global", "config.json");

        assert!(jinmap.meta.last_updated > first_time);
    }
}
