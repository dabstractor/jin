//! Staged entry type for Jin

use crate::core::Layer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a file staged for commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedEntry {
    /// Path to the file in the workspace
    pub path: PathBuf,
    /// Target layer for this entry
    pub target_layer: Layer,
    /// Content hash (Git blob OID as hex string)
    pub content_hash: String,
    /// File mode (e.g., 0o100644 for regular file)
    pub mode: u32,
    /// Operation type
    pub operation: StagedOperation,
}

/// Type of staging operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagedOperation {
    /// Add or modify a file
    AddOrModify,
    /// Delete a file
    Delete,
    /// Rename a file
    Rename,
}

impl StagedEntry {
    /// Create a new staged entry for adding/modifying a file
    pub fn new(path: PathBuf, target_layer: Layer, content_hash: String) -> Self {
        Self {
            path,
            target_layer,
            content_hash,
            mode: 0o100644,
            operation: StagedOperation::AddOrModify,
        }
    }

    /// Create a new staged entry for deletion
    pub fn delete(path: PathBuf, target_layer: Layer) -> Self {
        Self {
            path,
            target_layer,
            content_hash: String::new(),
            mode: 0,
            operation: StagedOperation::Delete,
        }
    }

    /// Check if this entry is a deletion
    pub fn is_delete(&self) -> bool {
        self.operation == StagedOperation::Delete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staged_entry_new() {
        let entry = StagedEntry::new(
            PathBuf::from(".claude/config.json"),
            Layer::ModeBase,
            "abc123".to_string(),
        );

        assert_eq!(entry.path, PathBuf::from(".claude/config.json"));
        assert_eq!(entry.target_layer, Layer::ModeBase);
        assert_eq!(entry.content_hash, "abc123");
        assert_eq!(entry.operation, StagedOperation::AddOrModify);
        assert!(!entry.is_delete());
    }

    #[test]
    fn test_staged_entry_delete() {
        let entry = StagedEntry::delete(PathBuf::from(".claude/old.json"), Layer::ProjectBase);

        assert!(entry.is_delete());
        assert!(entry.content_hash.is_empty());
    }
}
