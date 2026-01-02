//! JinMap: Layer-to-file mapping metadata
//!
//! The JinMap maintains a persistent record of which files belong to which layers
//! in Jin's 9-layer hierarchy. This enables:
//! - Repository state recovery if Git refs are corrupted
//! - Fast layer-to-file lookups without walking Git trees
//! - Validation by the repair command
//!
//! The JinMap is stored at `.jin/.jinmap` in YAML format for human readability
//! and emergency manual editing.

use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps, TreeOps};
use git2::Oid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// JinMap: Layer-to-file mapping metadata
///
/// Tracks which files belong to which layers in the 9-layer hierarchy.
/// Stored at `.jin/.jinmap` in YAML format.
///
/// Format per PRD Section 16:
/// ```yaml
/// version: 1
/// mappings:
///   "refs/jin/layers/mode/claude":
///     - ".claude/config.json"
///     - ".claude/prompt.md"
///   "refs/jin/layers/project/myproject":
///     - "config/settings.json"
/// meta:
///   generated-by: jin
///   last-updated: "2025-01-01T12:00:00Z"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinMap {
    /// Format version (for future migration support)
    #[serde(default = "default_version")]
    pub version: u32,

    /// Layer ref path -> list of file paths
    ///
    /// Key: Git ref path like "refs/jin/layers/mode/claude"
    /// Value: List of file paths relative to workspace root
    pub mappings: HashMap<String, Vec<String>>,

    /// Metadata about the JinMap file
    #[serde(default)]
    pub meta: JinMapMeta,
}

/// Metadata for JinMap file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinMapMeta {
    /// Tool that generated this file
    #[serde(default = "default_generated_by")]
    pub generated_by: String,

    /// Last update timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

fn default_version() -> u32 {
    1
}

fn default_generated_by() -> String {
    "jin".to_string()
}

impl Default for JinMap {
    fn default() -> Self {
        Self {
            version: default_version(),
            mappings: HashMap::new(),
            meta: JinMapMeta::default(),
        }
    }
}

impl Default for JinMapMeta {
    fn default() -> Self {
        Self {
            generated_by: default_generated_by(),
            last_updated: None,
        }
    }
}

impl JinMap {
    /// Load the JinMap from disk
    ///
    /// Returns a default JinMap if the file doesn't exist (first-run pattern).
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_yaml::from_str(&content).map_err(|e| JinError::Parse {
                format: "YAML".to_string(),
                message: e.to_string(),
            })
        } else {
            Ok(Self::default())
        }
    }

    /// Save the JinMap to disk
    ///
    /// Uses atomic write pattern: write to temp file, then rename.
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self).map_err(|e| JinError::Parse {
            format: "YAML".to_string(),
            message: e.to_string(),
        })?;

        // Atomic write pattern - use temp file in same directory
        // For .jinmap, we need to append .tmp directly since with_extension doesn't work well for dotfiles
        let temp_path = path.with_file_name(format!("{}.tmp", path.file_name().unwrap().to_string_lossy()));
        std::fs::write(&temp_path, content).map_err(JinError::Io)?;
        std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;

        Ok(())
    }

    /// Get the default path for the JinMap file
    pub fn default_path() -> PathBuf {
        PathBuf::from(".jin").join(".jinmap")
    }

    /// Update JinMap with layer mappings from recent commits
    ///
    /// This method reads the committed tree objects for each layer and
    /// collects all file paths to build accurate mappings.
    ///
    /// # Arguments
    ///
    /// * `layer_commits` - Slice of (Layer, commit Oid) tuples from the commit
    /// * `context` - Project context for mode/scope/project values
    /// * `repo` - Jin repository for reading tree objects
    pub fn update_from_commits(
        &mut self,
        layer_commits: &[(Layer, Oid)],
        context: &ProjectContext,
        repo: &JinRepo,
    ) -> Result<()> {
        for (layer, commit_oid) in layer_commits {
            // Get layer ref path for mapping key
            let ref_path = layer.ref_path(
                context.mode.as_deref(),
                context.scope.as_deref(),
                context.project.as_deref(),
            );

            // Get the tree from the commit
            let commit = repo.find_commit(*commit_oid)?;
            let tree_oid = commit.tree_id();

            // Collect files from the tree
            let files = self.walk_layer_tree(repo, tree_oid)?;

            // Add or update the mapping
            if !files.is_empty() {
                self.mappings.insert(ref_path, files);
            } else {
                // Remove entry if all files were deleted
                self.mappings.remove(&ref_path);
            }
        }

        // Update metadata timestamp
        self.meta.last_updated = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Add a layer mapping directly
    ///
    /// This is a convenience method for testing or manual updates.
    pub fn add_layer_mapping(&mut self, layer_ref: &str, files: Vec<String>) {
        if files.is_empty() {
            self.mappings.remove(layer_ref);
        } else {
            self.mappings.insert(layer_ref.to_string(), files);
        }
        self.meta.last_updated = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Walk a layer tree and collect all file paths
    ///
    /// Returns a vector of file paths relative to the workspace root.
    /// Only includes files (blobs), not directories.
    fn walk_layer_tree(&self, repo: &JinRepo, tree_oid: Oid) -> Result<Vec<String>> {
        // Use TreeOps::list_tree_files which already handles recursive walking
        // and returns only files (not directories)
        repo.list_tree_files(tree_oid)
            .map_err(|e| JinError::Other(format!("Failed to walk layer tree: {}", e)))
    }

    /// Get all files for a specific layer ref path
    pub fn get_layer_files(&self, layer_ref: &str) -> Option<&[String]> {
        self.mappings.get(layer_ref).map(|v| v.as_slice())
    }

    /// Get all layer ref paths that have mappings
    pub fn layer_refs(&self) -> Vec<&String> {
        self.mappings.keys().collect()
    }

    /// Check if a file is mapped to any layer
    pub fn contains_file(&self, file_path: &str) -> bool {
        self.mappings
            .values()
            .any(|files| files.contains(&file_path.to_string()))
    }

    /// Get the total number of file mappings across all layers
    pub fn total_file_count(&self) -> usize {
        self.mappings.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create an isolated test environment
    fn create_test_jinmap() -> (TempDir, JinMap) {
        let temp = TempDir::new().unwrap();
        let jinmap = JinMap::default();
        (temp, jinmap)
    }

    #[test]
    fn test_jinmap_default() {
        let jinmap = JinMap::default();
        assert_eq!(jinmap.version, 1);
        assert!(jinmap.mappings.is_empty());
        assert_eq!(jinmap.meta.generated_by, "jin");
        assert!(jinmap.meta.last_updated.is_none());
    }

    #[test]
    fn test_jinmap_serialize_deserialize() {
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping(
            "refs/jin/layers/mode/claude",
            vec![
                ".claude/config.json".to_string(),
                ".claude/prompt.md".to_string(),
            ],
        );

        let yaml = serde_yaml::to_string(&jinmap).unwrap();
        let parsed: JinMap = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.mappings.len(), 1);
        assert!(parsed.mappings.contains_key("refs/jin/layers/mode/claude"));
        let files = parsed.mappings.get("refs/jin/layers/mode/claude").unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&".claude/config.json".to_string()));
    }

    #[test]
    fn test_jinmap_yaml_format() {
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping(
            "refs/jin/layers/mode/claude",
            vec![".claude/config.json".to_string()],
        );

        let yaml = serde_yaml::to_string(&jinmap).unwrap();

        // Verify YAML structure
        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("mappings:"));
        assert!(yaml.contains("refs/jin/layers/mode/claude:"));
        assert!(yaml.contains(".claude/config.json"));
        // Note: generated-by: jin is in the meta section
    }

    #[test]
    fn test_jinmap_save_load_roundtrip() {
        let temp = TempDir::new().unwrap();
        let jinmap_path = temp.path().join(".jin").join(".jinmap");

        // Create .jin directory
        std::fs::create_dir_all(jinmap_path.parent().unwrap()).unwrap();

        // Create and save a JinMap
        let mut original = JinMap::default();
        original.add_layer_mapping(
            "refs/jin/layers/global",
            vec!["config.json".to_string(), "settings.json".to_string()],
        );

        // Manually write the JinMap content
        let content = serde_yaml::to_string(&original).unwrap();
        std::fs::write(&jinmap_path, content).unwrap();

        // Load from the file path
        let file_content = std::fs::read_to_string(&jinmap_path).unwrap();
        let loaded: JinMap = serde_yaml::from_str(&file_content).unwrap();

        assert_eq!(loaded.version, original.version);
        assert_eq!(loaded.mappings.len(), original.mappings.len());
        assert_eq!(
            loaded.mappings.get("refs/jin/layers/global"),
            original.mappings.get("refs/jin/layers/global")
        );
    }

    #[test]
    fn test_jinmap_load_creates_default() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Don't create a file, load should return default
        let jinmap = JinMap::load().unwrap();
        assert_eq!(jinmap.version, 1);
        assert!(jinmap.mappings.is_empty());
    }

    #[test]
    fn test_jinmap_add_layer_mapping() {
        let mut jinmap = JinMap::default();

        jinmap.add_layer_mapping(
            "refs/jin/layers/mode/claude",
            vec!["file1.txt".to_string(), "file2.txt".to_string()],
        );

        assert_eq!(jinmap.mappings.len(), 1);
        let files = jinmap
            .get_layer_files("refs/jin/layers/mode/claude")
            .unwrap();
        assert_eq!(files.len(), 2);

        // Verify timestamp was updated
        assert!(jinmap.meta.last_updated.is_some());
    }

    #[test]
    fn test_jinmap_add_empty_mapping_removes_entry() {
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping("refs/jin/layers/mode/claude", vec!["file.txt".to_string()]);

        assert_eq!(jinmap.mappings.len(), 1);

        // Add empty list - should remove the entry
        jinmap.add_layer_mapping("refs/jin/layers/mode/claude", vec![]);

        assert!(jinmap.mappings.is_empty());
    }

    #[test]
    fn test_jinmap_get_layer_files() {
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping(
            "refs/jin/layers/project/test",
            vec!["config.json".to_string()],
        );

        // Existing layer
        let files = jinmap.get_layer_files("refs/jin/layers/project/test");
        assert!(files.is_some());
        assert_eq!(files.unwrap().len(), 1);

        // Non-existing layer
        let files = jinmap.get_layer_files("refs/jin/layers/nonexistent");
        assert!(files.is_none());
    }

    #[test]
    fn test_jinmap_layer_refs() {
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping("refs/jin/layers/mode/claude", vec!["file.txt".to_string()]);
        jinmap.add_layer_mapping(
            "refs/jin/layers/project/test",
            vec!["config.json".to_string()],
        );

        let refs = jinmap.layer_refs();
        assert_eq!(refs.len(), 2);
        let ref_strings: Vec<&str> = refs.iter().map(|s| s.as_str()).collect();
        assert!(ref_strings.contains(&"refs/jin/layers/mode/claude"));
        assert!(ref_strings.contains(&"refs/jin/layers/project/test"));
    }

    #[test]
    fn test_jinmap_contains_file() {
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping(
            "refs/jin/layers/mode/claude",
            vec![".claude/config.json".to_string(), "prompt.md".to_string()],
        );

        assert!(jinmap.contains_file(".claude/config.json"));
        assert!(jinmap.contains_file("prompt.md"));
        assert!(!jinmap.contains_file("nonexistent.txt"));
    }

    #[test]
    fn test_jinmap_total_file_count() {
        let mut jinmap = JinMap::default();
        assert_eq!(jinmap.total_file_count(), 0);

        jinmap.add_layer_mapping(
            "refs/jin/layers/mode/claude",
            vec!["file1.txt".to_string(), "file2.txt".to_string()],
        );
        assert_eq!(jinmap.total_file_count(), 2);

        jinmap.add_layer_mapping(
            "refs/jin/layers/project/test",
            vec!["config.json".to_string()],
        );
        assert_eq!(jinmap.total_file_count(), 3);
    }

    #[test]
    fn test_jinmap_update_from_commits() {
        let (_temp, repo) = {
            let temp = TempDir::new().unwrap();
            let repo_path = temp.path().join(".jin");
            let repo = JinRepo::create_at(&repo_path).unwrap();
            (temp, repo)
        };

        // Create a test tree with files
        use crate::git::ObjectOps;
        let blob1 = repo.create_blob(b"content1").unwrap();
        let blob2 = repo.create_blob(b"content2").unwrap();
        let tree_oid = repo
            .create_tree_from_paths(&[
                ("config.json".to_string(), blob1),
                ("src/main.rs".to_string(), blob2),
            ])
            .unwrap();

        // Create a commit
        let commit_oid = repo
            .create_commit(None, "Test commit", tree_oid, &[])
            .unwrap();

        // Update JinMap from the commit
        let mut jinmap = JinMap::default();
        let context = ProjectContext::default();
        jinmap
            .update_from_commits(&[(Layer::GlobalBase, commit_oid)], &context, &repo)
            .unwrap();

        // Verify mappings were created
        assert_eq!(jinmap.mappings.len(), 1);
        let files = jinmap.get_layer_files("refs/jin/layers/global").unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"config.json".to_string()));
        assert!(files.contains(&"src/main.rs".to_string()));
    }

    #[test]
    fn test_jinmap_meta_timestamp_updated() {
        let mut jinmap = JinMap::default();
        assert!(jinmap.meta.last_updated.is_none());

        jinmap.add_layer_mapping("refs/jin/layers/test", vec!["file.txt".to_string()]);
        assert!(jinmap.meta.last_updated.is_some());

        // Verify timestamp is valid ISO 8601
        let timestamp = jinmap.meta.last_updated.as_ref().unwrap();
        assert!(timestamp.len() > 0);
        // Should be parseable as ISO 8601
        chrono::DateTime::parse_from_rfc3339(timestamp).unwrap();
    }
}
