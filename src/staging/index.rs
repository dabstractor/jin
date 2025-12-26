//! Staging index for managing staged entries.
//!
//! This module defines the `StagingIndex` structure that manages all staged
//! files for a Jin workspace. It provides CRUD operations, layer-based
//! querying, and persistence for staging state.

use crate::core::{
    error::{JinError, Result},
    Layer,
};
use crate::staging::entry::StagedEntry;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Staging index managing all staged entries.
///
/// The staging index maintains two data structures:
/// - `entries`: Primary index mapping path -> entry (ordered by insertion)
/// - `by_layer`: Secondary index mapping layer -> list of paths
///
/// This dual-index design enables both O(1) lookups by path and efficient
/// layer-based queries for commit operations.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::staging::{StagingIndex, StagedEntry};
/// use jin_glm::core::Layer;
/// use std::path::PathBuf;
///
/// let mut index = StagingIndex::new();
///
/// let entry = StagedEntry::new(
///     PathBuf::from("config.json"),
///     Layer::ProjectBase { project: "myapp".to_string() },
///     b"{\"key\": \"value\"}"
/// )?;
///
/// index.add_entry(entry)?;
/// assert_eq!(index.len(), 1);
/// ```
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StagingIndex {
    /// Primary index: path -> entry (ordered by insertion)
    #[serde(
        serialize_with = "serialize_entries",
        deserialize_with = "deserialize_entries"
    )]
    entries: IndexMap<PathBuf, StagedEntry>,
    /// Secondary index: layer -> paths (for layer-based queries)
    /// Note: by_layer is reconstructed from entries during deserialization
    #[serde(skip)]
    by_layer: HashMap<Layer, Vec<PathBuf>>,
}

// ===== SERIALIZATION HELPERS =====

/// Serialize IndexMap with PathBuf keys as strings
fn serialize_entries<S>(
    entries: &IndexMap<PathBuf, StagedEntry>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(entries.len()))?;
    for (k, v) in entries {
        let key_str = k.to_string_lossy().to_string();
        map.serialize_entry(&key_str, v)?;
    }
    map.end()
}

/// Deserialize IndexMap with PathBuf keys from strings
fn deserialize_entries<'de, D>(
    deserializer: D,
) -> std::result::Result<IndexMap<PathBuf, StagedEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    use std::collections::BTreeMap;

    // Deserialize into BTreeMap<String, StagedEntry> first
    let map: BTreeMap<String, StagedEntry> = BTreeMap::deserialize(deserializer)?;
    let mut entries = IndexMap::new();
    for (key_str, entry) in map {
        entries.insert(PathBuf::from(key_str), entry);
    }
    Ok(entries)
}

impl StagingIndex {
    /// Creates a new empty staging index.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let index = StagingIndex::new();
    /// assert!(index.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces an entry in the staging index.
    ///
    /// If an entry with the same path already exists, it is removed from
    /// the layer index before the new entry is added. This ensures that
    /// moving a file between layers works correctly.
    ///
    /// # Arguments
    ///
    /// * `entry` - The staged entry to add
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut index = StagingIndex::new();
    /// index.add_entry(entry)?;
    /// ```
    pub fn add_entry(&mut self, entry: StagedEntry) -> Result<()> {
        let path = entry.path.clone();
        let layer = entry.layer.clone();

        // Remove from old layer if exists
        if let Some(old_entry) = self.entries.shift_remove(&path) {
            self.remove_from_layer_index(&old_entry);
        }

        // Add to entries and layer index
        self.entries.insert(path.clone(), entry);
        self.by_layer.entry(layer).or_default().push(path);

        Ok(())
    }

    /// Removes an entry from the staging index.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the entry to remove
    ///
    /// # Returns
    ///
    /// Returns `Some(StagedEntry)` if an entry was removed, or `None` if
    /// no entry exists at the given path.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(entry) = index.remove_entry(&path) {
    ///     println!("Removed: {:?}", entry.path);
    /// }
    /// ```
    pub fn remove_entry(&mut self, path: &Path) -> Option<StagedEntry> {
        if let Some(entry) = self.entries.shift_remove(path) {
            self.remove_from_layer_index(&entry);
            Some(entry)
        } else {
            None
        }
    }

    /// Gets a reference to an entry by path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to look up
    ///
    /// # Returns
    ///
    /// Returns `Some(&StagedEntry)` if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(entry) = index.get_entry(&path) {
    ///     println!("Layer: {:?}", entry.layer);
    /// }
    /// ```
    pub fn get_entry(&self, path: &Path) -> Option<&StagedEntry> {
        self.entries.get(path)
    }

    /// Gets a mutable reference to an entry by path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to look up
    ///
    /// # Returns
    ///
    /// Returns `Some(&mut StagedEntry)` if found, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(entry) = index.get_entry_mut(&path) {
    ///     entry.stage();
    /// }
    /// ```
    pub fn get_entry_mut(&mut self, path: &Path) -> Option<&mut StagedEntry> {
        self.entries.get_mut(path)
    }

    /// Returns all entries for a specific layer.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to filter by
    ///
    /// # Returns
    ///
    /// A vector of references to entries in the specified layer.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let project_entries = index.entries_by_layer(&Layer::ProjectBase {
    ///     project: "myapp".to_string()
    /// });
    /// ```
    pub fn entries_by_layer(&self, layer: &Layer) -> Vec<&StagedEntry> {
        self.by_layer
            .get(layer)
            .map(|paths| paths.iter().filter_map(|p| self.entries.get(p)).collect())
            .unwrap_or_default()
    }

    /// Returns all entries in the index.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for entry in index.all_entries() {
    ///     println!("{:?}", entry.path);
    /// }
    /// ```
    pub fn all_entries(&self) -> Vec<&StagedEntry> {
        self.entries.values().collect()
    }

    /// Returns the number of entries in the index.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// println!("Staged files: {}", index.len());
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the index is empty.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if index.is_empty() {
    ///     println!("No files staged");
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes an entry from the layer secondary index.
    ///
    /// This is a helper method used internally to maintain consistency
    /// between the primary and secondary indexes.
    fn remove_from_layer_index(&mut self, entry: &StagedEntry) {
        if let Some(paths) = self.by_layer.get_mut(&entry.layer) {
            if let Some(pos) = paths.iter().position(|p| p == &entry.path) {
                paths.remove(pos);
            }
            if paths.is_empty() {
                self.by_layer.remove(&entry.layer);
            }
        }
    }

    /// Saves the staging index to disk.
    ///
    /// Creates the `.jin/staging` directory if it doesn't exist and writes
    /// the index to `index.json` in JSON format.
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err(JinError)` on failure.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// index.save_to_disk(&PathBuf::from("/my/project"))?;
    /// ```
    pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()> {
        let staging_dir = workspace_root.join(".jin/staging");
        std::fs::create_dir_all(&staging_dir)?;

        let index_file = staging_dir.join("index.json");
        let file = std::fs::File::create(&index_file)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| JinError::Message(format!("Failed to serialize index: {}", e)))?;

        Ok(())
    }

    /// Loads the staging index from disk.
    ///
    /// Returns an empty index if the file doesn't exist (first add).
    ///
    /// # Arguments
    ///
    /// * `workspace_root` - Path to the workspace root directory
    ///
    /// # Returns
    ///
    /// Returns `Ok(StagingIndex)` on success, or `Err(JinError)` on failure.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let index = StagingIndex::load_from_disk(&PathBuf::from("/my/project"))?;
    /// ```
    pub fn load_from_disk(workspace_root: &Path) -> Result<Self> {
        let index_file = workspace_root.join(".jin/staging/index.json");

        if !index_file.exists() {
            return Ok(Self::default());
        }

        let file = std::fs::File::open(&index_file)?;
        let reader = BufReader::new(file);

        let mut index: StagingIndex = serde_json::from_reader(reader)
            .map_err(|e| JinError::Message(format!("Failed to deserialize index: {}", e)))?;

        // Rebuild the by_layer secondary index
        index.rebuild_layer_index();

        Ok(index)
    }

    /// Clears all entries from the index.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// index.clear();
    /// assert!(index.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.by_layer.clear();
    }

    /// Rebuilds the layer secondary index from entries.
    ///
    /// This is used internally after deserialization to rebuild the
    /// by_layer index that is skipped during serialization.
    fn rebuild_layer_index(&mut self) {
        self.by_layer.clear();
        for (path, entry) in &self.entries {
            let layer = entry.layer.clone();
            let path = path.clone();
            self.by_layer.entry(layer).or_default().push(path);
        }
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Helper to create a test entry
    fn create_test_entry(path: &str, layer: Layer, content: &[u8]) -> StagedEntry {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(path);
        fs::write(&file_path, content).unwrap();
        StagedEntry::new(file_path, layer, content).unwrap()
    }

    // ===== Constructor Tests =====

    #[test]
    fn test_staging_index_new() {
        let index = StagingIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_staging_index_default() {
        let index = StagingIndex::default();
        assert!(index.is_empty());
    }

    // ===== CRUD Operation Tests =====

    #[test]
    fn test_staging_index_add_entry() {
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let entry = create_test_entry("config.json", layer, b"test");

        index.add_entry(entry).unwrap();
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_staging_index_add_replace() {
        let mut index = StagingIndex::new();
        let layer1 = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let layer2 = Layer::ModeBase {
            mode: "claude".to_string(),
        };

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.json");
        fs::write(&file_path, b"content1").unwrap();

        let entry1 = StagedEntry::new(file_path.clone(), layer1, b"content1").unwrap();
        let entry2 = StagedEntry::new(file_path, layer2, b"content2").unwrap();
        let lookup_path = entry1.path.clone();

        index.add_entry(entry1).unwrap();
        assert_eq!(index.len(), 1);
        assert_eq!(
            index.get_entry(&lookup_path).unwrap().layer,
            Layer::ProjectBase {
                project: "myapp".to_string()
            }
        );

        index.add_entry(entry2).unwrap();
        assert_eq!(index.len(), 1); // Still 1, replaced
        assert_eq!(
            index.get_entry(&lookup_path).unwrap().layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_staging_index_remove_entry() {
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let entry = create_test_entry("config.json", layer, b"test");
        let path = entry.path.clone();

        index.add_entry(entry).unwrap();
        assert_eq!(index.len(), 1);

        let removed = index.remove_entry(&path);
        assert!(removed.is_some());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_staging_index_remove_nonexistent() {
        let mut index = StagingIndex::new();
        let removed = index.remove_entry(Path::new("nonexistent.json"));
        assert!(removed.is_none());
    }

    #[test]
    fn test_staging_index_get_entry() {
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let entry = create_test_entry("config.json", layer.clone(), b"test");
        let path = entry.path.clone();

        index.add_entry(entry).unwrap();

        let retrieved = index.get_entry(&path);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().layer, layer);
    }

    #[test]
    fn test_staging_index_get_entry_mut() {
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let entry = create_test_entry("config.json", layer, b"test");
        let path = entry.path.clone();

        index.add_entry(entry).unwrap();
        assert!(!index.get_entry(&path).unwrap().is_staged());

        if let Some(entry_mut) = index.get_entry_mut(&path) {
            entry_mut.stage();
        }
        assert!(index.get_entry(&path).unwrap().is_staged());
    }

    #[test]
    fn test_staging_index_entries_by_layer() {
        let mut index = StagingIndex::new();
        let layer1 = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let layer2 = Layer::ModeBase {
            mode: "claude".to_string(),
        };

        let entry1 = create_test_entry("config1.json", layer1.clone(), b"test1");
        let entry2 = create_test_entry("config2.json", layer1.clone(), b"test2");
        let entry3 = create_test_entry("config3.json", layer2.clone(), b"test3");

        index.add_entry(entry1).unwrap();
        index.add_entry(entry2).unwrap();
        index.add_entry(entry3).unwrap();

        let project_entries = index.entries_by_layer(&layer1);
        assert_eq!(project_entries.len(), 2);

        let mode_entries = index.entries_by_layer(&layer2);
        assert_eq!(mode_entries.len(), 1);
    }

    #[test]
    fn test_staging_index_entries_by_layer_empty() {
        let index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let entries = index.entries_by_layer(&layer);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_staging_index_multiple_layers() {
        let mut index = StagingIndex::new();
        let layers = vec![
            Layer::GlobalBase,
            Layer::ModeBase {
                mode: "claude".to_string(),
            },
            Layer::ProjectBase {
                project: "myapp".to_string(),
            },
        ];

        for (i, layer) in layers.iter().enumerate() {
            let entry = create_test_entry(&format!("file{}.json", i), layer.clone(), b"test");
            index.add_entry(entry).unwrap();
        }

        assert_eq!(index.len(), 3);

        // Verify each layer has one entry
        for layer in &layers {
            let entries = index.entries_by_layer(layer);
            assert_eq!(entries.len(), 1);
        }
    }

    #[test]
    fn test_staging_index_clear() {
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        let entry = create_test_entry("config.json", layer, b"test");

        index.add_entry(entry).unwrap();
        assert_eq!(index.len(), 1);

        index.clear();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_staging_index_all_entries() {
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };

        for i in 0..3 {
            let entry = create_test_entry(&format!("file{}.json", i), layer.clone(), b"test");
            index.add_entry(entry).unwrap();
        }

        let all = index.all_entries();
        assert_eq!(all.len(), 3);
    }

    // ===== Persistence Tests =====

    #[test]
    fn test_staging_index_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let mut index = StagingIndex::new();
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };

        let temp_file = temp_dir.path().join("config.json");
        fs::write(&temp_file, b"test content").unwrap();
        let entry = StagedEntry::new(temp_file, layer, b"test content").unwrap();
        index.add_entry(entry).unwrap();

        // Save
        index.save_to_disk(temp_dir.path()).unwrap();

        // Load
        let loaded = StagingIndex::load_from_disk(temp_dir.path()).unwrap();
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn test_staging_index_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let index = StagingIndex::load_from_disk(temp_dir.path()).unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test_staging_index_save_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let index = StagingIndex::new();

        index.save_to_disk(temp_dir.path()).unwrap();

        let staging_dir = temp_dir.path().join(".jin/staging");
        assert!(staging_dir.exists());
        assert!(staging_dir.is_dir());

        let index_file = staging_dir.join("index.json");
        assert!(index_file.exists());
    }
}
