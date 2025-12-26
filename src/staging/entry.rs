//! Staging entry types for Jin.
//!
//! This module defines the file status tracking and staged entry structures
//! used by Jin's staging system. Each staged file is represented by a
//! `StagedEntry` with layer, metadata, and content hash information.

use crate::core::{error::Result, Layer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

// Import the bitflags macro
use bitflags::bitflags;

// ===== FILE STATUS FLAGS =====

bitflags! {
    /// File status flags for staged entries.
    ///
    /// Uses bitflags for efficient status representation where multiple flags
    /// can be combined (e.g., STAGED | MODIFIED).
    ///
    /// # Status Values
    ///
    /// - `CLEAN`: File has not been modified
    /// - `MODIFIED`: File has been modified but not staged
    /// - `STAGED`: File is staged for commit
    /// - `REMOVED`: File has been removed (staged deletion)
    /// - `NEW`: File is new (not in previous commit)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use jin_glm::staging::FileStatus;
    ///
    /// let mut status = FileStatus::MODIFIED;
    /// status |= FileStatus::STAGED;  // Now modified AND staged
    ///
    /// if status.contains(FileStatus::STAGED) {
    ///     println!("File is staged");
    /// }
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct FileStatus: u8 {
        /// File has not been modified
        const CLEAN = 0b00000001;
        /// File has been modified but not staged
        const MODIFIED = 0b00000010;
        /// File is staged for commit
        const STAGED = 0b00000100;
        /// File has been removed (staged deletion)
        const REMOVED = 0b00001000;
        /// File is new (not in previous commit)
        const NEW = 0b00010000;
    }
}

impl FileStatus {
    /// Returns true if the file is staged.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let status = FileStatus::STAGED | FileStatus::MODIFIED;
    /// assert!(status.is_staged());
    /// ```
    pub fn is_staged(self) -> bool {
        self.intersects(Self::STAGED)
    }

    /// Returns true if the file is modified.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let status = FileStatus::MODIFIED;
    /// assert!(status.is_modified());
    /// ```
    pub fn is_modified(self) -> bool {
        self.intersects(Self::MODIFIED)
    }

    /// Returns true if the file is removed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let status = FileStatus::REMOVED;
    /// assert!(status.is_removed());
    /// ```
    pub fn is_removed(self) -> bool {
        self.intersects(Self::REMOVED)
    }
}

// ===== STAGED ENTRY =====

/// A single staged file entry with layer and metadata.
///
/// Represents a file that has been staged for commit to a specific layer.
/// Each entry contains the file path, target layer, content hash, and
/// metadata for tracking staging state.
///
/// # Fields
///
/// - `path`: Path relative to workspace root
/// - `layer`: Target layer for this entry
/// - `content_hash`: SHA-256 hash of file content
/// - `status`: File status flags (STAGED, MODIFIED, etc.)
/// - `staged_at`: When file was staged (None if not staged)
/// - `size`: File size in bytes
/// - `modified_at`: Last modification time from filesystem
///
/// # Examples
///
/// ```ignore
/// use jin_glm::staging::StagedEntry;
/// use jin_glm::core::Layer;
/// use std::path::PathBuf;
///
/// let path = PathBuf::from("config.json");
/// let layer = Layer::ProjectBase { project: "myapp".to_string() };
/// let content = b"{\"key\": \"value\"}";
///
/// let entry = StagedEntry::new(path.clone(), layer, content)?;
/// assert!(entry.is_modified());
///
/// entry.stage();
/// assert!(entry.is_staged());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedEntry {
    /// Path relative to workspace root
    pub path: PathBuf,
    /// Target layer for this entry
    pub layer: Layer,
    /// SHA-256 hash of file content
    pub content_hash: Vec<u8>,
    /// File status flags
    pub status: FileStatus,
    /// When file was staged (None for unstaged)
    #[serde(default)]
    pub staged_at: Option<SystemTime>,
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub modified_at: SystemTime,
}

// ===== STAGED ENTRY METHODS =====

impl StagedEntry {
    /// Creates a new staged entry from a file path and content.
    ///
    /// Computes the SHA-256 content hash, reads file metadata, and
    /// initializes the entry with MODIFIED status.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file (relative to workspace root)
    /// * `layer` - Target layer for this entry
    /// * `content` - File content bytes
    ///
    /// # Returns
    ///
    /// Returns `Ok(StagedEntry)` if the file metadata can be read,
    /// or `Err(JinError)` if the file doesn't exist or metadata fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use jin_glm::staging::StagedEntry;
    /// use jin_glm::core::Layer;
    /// use std::path::PathBuf;
    ///
    /// let path = PathBuf::from("config.json");
    /// let layer = Layer::ProjectBase { project: "myapp".to_string() };
    /// let content = b"test content";
    ///
    /// let entry = StagedEntry::new(path, layer, content)?;
    /// ```
    pub fn new(path: PathBuf, layer: Layer, content: &[u8]) -> Result<Self> {
        use sha2::{Digest, Sha256};

        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let hash = Sha256::digest(content);

        Ok(Self {
            path,
            layer,
            content_hash: hash.to_vec(),
            status: FileStatus::MODIFIED,
            staged_at: None,
            size: metadata.len(),
            modified_at: modified,
        })
    }

    /// Returns true if this entry is staged for commit.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if !entry.is_staged() {
    ///     entry.stage();
    /// }
    /// ```
    pub fn is_staged(&self) -> bool {
        self.status.is_staged()
    }

    /// Returns true if this entry is modified.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if entry.is_modified() {
    ///     println!("File has changes");
    /// }
    /// ```
    pub fn is_modified(&self) -> bool {
        self.status.is_modified()
    }

    /// Returns true if this entry is marked for removal.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if entry.is_removed() {
    ///     println!("File will be deleted");
    /// }
    /// ```
    pub fn is_removed(&self) -> bool {
        self.status.is_removed()
    }

    /// Marks this entry as staged for commit.
    ///
    /// Sets the STAGED flag and records the current time.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// entry.stage();
    /// assert!(entry.is_staged());
    /// assert!(entry.staged_at.is_some());
    /// ```
    pub fn stage(&mut self) {
        self.status |= FileStatus::STAGED;
        self.staged_at = Some(SystemTime::now());
    }

    /// Unstages this entry, removing the STAGED flag.
    ///
    /// Clears the STAGED flag and resets staged_at to None.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// entry.unstage();
    /// assert!(!entry.is_staged());
    /// assert!(entry.staged_at.is_none());
    /// ```
    pub fn unstage(&mut self) {
        self.status.remove(FileStatus::STAGED);
        self.staged_at = None;
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== FileStatus Tests =====

    #[test]
    fn test_file_status_combinations() {
        // Individual flags
        assert_eq!(FileStatus::CLEAN.bits(), 0b00000001);
        assert_eq!(FileStatus::MODIFIED.bits(), 0b00000010);
        assert_eq!(FileStatus::STAGED.bits(), 0b00000100);
        assert_eq!(FileStatus::REMOVED.bits(), 0b00001000);
        assert_eq!(FileStatus::NEW.bits(), 0b00010000);
    }

    #[test]
    fn test_file_status_is_staged() {
        // STAGED flag is set
        assert!(FileStatus::STAGED.is_staged());

        // STAGED combined with other flags
        let status = FileStatus::STAGED | FileStatus::MODIFIED;
        assert!(status.is_staged());

        // STAGED flag not set
        assert!(!FileStatus::MODIFIED.is_staged());
        assert!(!FileStatus::CLEAN.is_staged());
    }

    #[test]
    fn test_file_status_is_modified() {
        // MODIFIED flag is set
        assert!(FileStatus::MODIFIED.is_modified());

        // MODIFIED combined with other flags
        let status = FileStatus::MODIFIED | FileStatus::STAGED;
        assert!(status.is_modified());

        // MODIFIED flag not set
        assert!(!FileStatus::CLEAN.is_modified());
        assert!(!FileStatus::STAGED.is_modified());
    }

    #[test]
    fn test_file_status_is_removed() {
        // REMOVED flag is set
        assert!(FileStatus::REMOVED.is_removed());

        // REMOVED combined with other flags
        let status = FileStatus::REMOVED | FileStatus::STAGED;
        assert!(status.is_removed());

        // REMOVED flag not set
        assert!(!FileStatus::CLEAN.is_removed());
        assert!(!FileStatus::MODIFIED.is_removed());
    }

    #[test]
    fn test_file_status_bitflags_or() {
        let status = FileStatus::MODIFIED | FileStatus::STAGED;
        assert!(status.is_modified());
        assert!(status.is_staged());
    }

    #[test]
    fn test_file_status_remove() {
        let mut status = FileStatus::MODIFIED | FileStatus::STAGED;
        assert!(status.is_staged());

        status.remove(FileStatus::STAGED);
        assert!(!status.is_staged());
        assert!(status.is_modified());
    }
}
