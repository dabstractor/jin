//! Workspace metadata tracking for Jin
//!
//! Tracks the last applied configuration to enable three-way merge diffs
//! and idempotent apply operations.

use crate::core::{JinError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Metadata tracking the last applied workspace configuration
///
/// This enables three-way merge diffs (Kubernetes-style) by storing:
/// - What layers were applied
/// - What files were written
/// - Content hashes for each file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// RFC3339 timestamp of when configuration was applied
    pub timestamp: String,
    /// Layer names that were merged and applied
    pub applied_layers: Vec<String>,
    /// Map of file paths to their content hashes (Git blob OID)
    pub files: HashMap<PathBuf, String>,
}

impl WorkspaceMetadata {
    /// Create a new empty workspace metadata
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            applied_layers: Vec::new(),
            files: HashMap::new(),
        }
    }

    /// Load workspace metadata from disk
    ///
    /// # Returns
    ///
    /// The loaded metadata, or an error if the file doesn't exist or is invalid
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content).map_err(|e| JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            })
        } else {
            Err(JinError::NotFound(path.display().to_string()))
        }
    }

    /// Save workspace metadata to disk
    ///
    /// Creates the parent directory if it doesn't exist.
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })?;

        // Atomic write pattern: write to temp file, then rename
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content)?;
        std::fs::rename(&temp_path, &path)?;

        Ok(())
    }

    /// Update the metadata with new timestamp
    pub fn update_timestamp(&mut self) {
        self.timestamp = chrono::Utc::now().to_rfc3339();
    }

    /// Add a file to the metadata
    pub fn add_file(&mut self, path: PathBuf, content_hash: String) {
        self.files.insert(path, content_hash);
    }

    /// Remove a file from the metadata
    pub fn remove_file(&mut self, path: &Path) {
        self.files.remove(path);
    }

    /// Get the default path for workspace metadata
    pub fn default_path() -> PathBuf {
        // Check JIN_DIR environment variable first for test isolation
        if let Ok(jin_dir) = std::env::var("JIN_DIR") {
            return PathBuf::from(jin_dir)
                .join("workspace")
                .join("last_applied.json");
        }
        PathBuf::from(".jin")
            .join("workspace")
            .join("last_applied.json")
    }
}

impl Default for WorkspaceMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_metadata_new() {
        let meta = WorkspaceMetadata::new();
        assert!(!meta.timestamp.is_empty());
        assert!(meta.applied_layers.is_empty());
        assert!(meta.files.is_empty());
    }

    #[test]
    fn test_workspace_metadata_default() {
        let meta = WorkspaceMetadata::default();
        assert!(!meta.timestamp.is_empty());
    }

    #[test]
    fn test_workspace_metadata_add_remove_file() {
        let mut meta = WorkspaceMetadata::new();

        meta.add_file(PathBuf::from(".claude/config.json"), "abc123".to_string());
        assert_eq!(meta.files.len(), 1);
        assert_eq!(
            meta.files.get(Path::new(".claude/config.json")),
            Some(&"abc123".to_string())
        );

        meta.remove_file(Path::new(".claude/config.json"));
        assert!(meta.files.is_empty());
    }

    #[test]
    fn test_workspace_metadata_update_timestamp() {
        let mut meta = WorkspaceMetadata::new();
        let original_timestamp = meta.timestamp.clone();

        // Sleep briefly to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        meta.update_timestamp();

        assert_ne!(meta.timestamp, original_timestamp);
    }

    #[test]
    fn test_workspace_metadata_save_load() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let mut meta = WorkspaceMetadata::new();
        meta.applied_layers = vec!["global".to_string(), "mode/claude".to_string()];
        meta.add_file(PathBuf::from(".claude/config.json"), "abc123".to_string());

        // Save
        meta.save().unwrap();

        // Load
        let loaded = WorkspaceMetadata::load().unwrap();
        assert_eq!(loaded.applied_layers, meta.applied_layers);
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(
            loaded.files.get(Path::new(".claude/config.json")),
            Some(&"abc123".to_string())
        );
    }

    #[test]
    #[serial]
    fn test_workspace_metadata_load_not_found() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = WorkspaceMetadata::load();
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_workspace_metadata_serialization() {
        let mut meta = WorkspaceMetadata::new();
        meta.applied_layers = vec!["global".to_string()];
        meta.add_file(PathBuf::from("file.json"), "hash123".to_string());

        let json = serde_json::to_string_pretty(&meta).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("applied_layers"));
        assert!(json.contains("files"));
        assert!(json.contains("global"));
    }
}
