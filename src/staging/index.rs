//! Staging index for Jin

use super::StagedEntry;
use crate::core::{JinError, Layer, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The staging index, tracking all staged files
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StagingIndex {
    /// Staged entries, keyed by path
    entries: HashMap<PathBuf, StagedEntry>,
    /// Version of the staging format
    #[serde(default = "default_version")]
    version: u32,
}

fn default_version() -> u32 {
    1
}

impl StagingIndex {
    /// Create a new empty staging index
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: 1,
        }
    }

    /// Load the staging index from disk
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(JinError::Io)?;
            serde_json::from_str(&content).map_err(|e| JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            })
        } else {
            Ok(Self::new())
        }
    }

    /// Save the staging index to disk
    ///
    /// Uses atomic write pattern: write to temp file, then rename.
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(JinError::Io)?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })?;

        // Atomic write pattern - use temp file in same directory
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content).map_err(JinError::Io)?;
        std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;

        Ok(())
    }

    /// Get the default path for the staging index
    pub fn default_path() -> PathBuf {
        PathBuf::from(".jin").join("staging").join("index.json")
    }

    /// Add an entry to the staging index
    pub fn add(&mut self, entry: StagedEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    /// Remove an entry from the staging index
    pub fn remove(&mut self, path: &Path) -> Option<StagedEntry> {
        self.entries.remove(path)
    }

    /// Get an entry by path
    pub fn get(&self, path: &Path) -> Option<&StagedEntry> {
        self.entries.get(path)
    }

    /// Get all entries
    pub fn entries(&self) -> impl Iterator<Item = &StagedEntry> {
        self.entries.values()
    }

    /// Get all staged paths
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.entries.keys()
    }

    /// Get entries for a specific layer
    pub fn entries_for_layer(&self, layer: Layer) -> Vec<&StagedEntry> {
        self.entries
            .values()
            .filter(|e| e.target_layer == layer)
            .collect()
    }

    /// Get all layers that have staged entries
    pub fn affected_layers(&self) -> Vec<Layer> {
        let mut layers: Vec<Layer> = self
            .entries
            .values()
            .map(|e| e.target_layer)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        layers.sort_by_key(|l| l.precedence());
        layers
    }

    /// Check if the staging index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of staged entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Clear all staged entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staging_index_new() {
        let index = StagingIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_staging_index_add_remove() {
        let mut index = StagingIndex::new();

        let entry = StagedEntry::new(
            PathBuf::from("test.json"),
            Layer::ProjectBase,
            "hash123".to_string(),
        );

        index.add(entry);
        assert_eq!(index.len(), 1);
        assert!(index.get(Path::new("test.json")).is_some());

        index.remove(Path::new("test.json"));
        assert!(index.is_empty());
    }

    #[test]
    fn test_staging_index_entries_for_layer() {
        let mut index = StagingIndex::new();

        index.add(StagedEntry::new(
            PathBuf::from("a.json"),
            Layer::ModeBase,
            "h1".to_string(),
        ));
        index.add(StagedEntry::new(
            PathBuf::from("b.json"),
            Layer::ProjectBase,
            "h2".to_string(),
        ));
        index.add(StagedEntry::new(
            PathBuf::from("c.json"),
            Layer::ModeBase,
            "h3".to_string(),
        ));

        let mode_entries = index.entries_for_layer(Layer::ModeBase);
        assert_eq!(mode_entries.len(), 2);

        let project_entries = index.entries_for_layer(Layer::ProjectBase);
        assert_eq!(project_entries.len(), 1);
    }

    #[test]
    fn test_affected_layers() {
        let mut index = StagingIndex::new();

        index.add(StagedEntry::new(
            PathBuf::from("a.json"),
            Layer::ModeBase,
            "h1".to_string(),
        ));
        index.add(StagedEntry::new(
            PathBuf::from("b.json"),
            Layer::ProjectBase,
            "h2".to_string(),
        ));

        let layers = index.affected_layers();
        assert_eq!(layers.len(), 2);
        // Should be sorted by precedence (ModeBase=2, ProjectBase=7)
        assert_eq!(layers[0], Layer::ModeBase);
        assert_eq!(layers[1], Layer::ProjectBase);
    }
}
