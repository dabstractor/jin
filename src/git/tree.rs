//! Tree walking and reading operations
//!
//! This module provides [`TreeOps`], a trait for traversing and reading
//! Git tree contents in Jin's phantom repository.

use crate::core::Result;
use git2::{ObjectType, Oid, TreeEntry as Git2TreeEntry, TreeWalkMode, TreeWalkResult};
use std::path::Path;

use super::JinRepo;

/// Tree walking and reading operations.
///
/// This trait provides utilities for traversing Git trees and extracting
/// file contents from commits.
pub trait TreeOps {
    /// Walks a tree in pre-order (parent before children).
    ///
    /// The callback receives `(parent_path, entry)` where `parent_path` is
    /// the directory path leading to the entry, and `entry.name()` is the
    /// entry's own name.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::{JinRepo, ObjectOps, TreeOps, TreeWalkResult};
    ///
    /// let repo = JinRepo::open()?;
    /// // ... create some commits ...
    /// # let tree_oid = git2::Oid::zero();
    ///
    /// repo.walk_tree_pre(tree_oid, |path, entry| {
    ///     println!("{}{}", path, entry.name().unwrap_or(""));
    ///     TreeWalkResult::Ok
    /// })?;
    /// # Ok::<(), jin::JinError>(())
    /// ```
    fn walk_tree_pre<F>(&self, tree_oid: Oid, callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult;

    /// Walks a tree in post-order (children before parent).
    fn walk_tree_post<F>(&self, tree_oid: Oid, callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult;

    /// Gets a tree entry by path (e.g., "src/config.json").
    ///
    /// Returns the OID of the entry at the given path.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the path doesn't exist in the tree.
    fn get_tree_entry(&self, tree_oid: Oid, path: &Path) -> Result<Oid>;

    /// Reads blob content by OID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::{JinRepo, ObjectOps, TreeOps};
    ///
    /// let repo = JinRepo::open()?;
    /// let blob_oid = repo.create_blob(b"Hello!")?;
    /// let content = repo.read_blob_content(blob_oid)?;
    /// assert_eq!(content, b"Hello!");
    /// # Ok::<(), jin::JinError>(())
    /// ```
    fn read_blob_content(&self, blob_oid: Oid) -> Result<Vec<u8>>;

    /// Reads file content from a tree by path.
    ///
    /// This is a convenience method that combines `get_tree_entry` and
    /// `read_blob_content`.
    fn read_file_from_tree(&self, tree_oid: Oid, path: &Path) -> Result<Vec<u8>>;

    /// Lists all files in a tree recursively.
    ///
    /// Returns a vector of file paths relative to the tree root.
    /// Directories are not included, only files (blobs).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::{JinRepo, ObjectOps, TreeOps, TreeEntry};
    ///
    /// let repo = JinRepo::open()?;
    /// let blob1 = repo.create_blob(b"content1")?;
    /// let blob2 = repo.create_blob(b"content2")?;
    /// let tree = repo.create_tree_from_paths(&[
    ///     ("root.txt".to_string(), blob1),
    ///     ("src/main.rs".to_string(), blob2),
    /// ])?;
    ///
    /// let files = repo.list_tree_files(tree)?;
    /// assert!(files.contains(&"root.txt".to_string()));
    /// assert!(files.contains(&"src/main.rs".to_string()));
    /// # Ok::<(), jin::JinError>(())
    /// ```
    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>>;
}

impl TreeOps for JinRepo {
    fn walk_tree_pre<F>(&self, tree_oid: Oid, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult,
    {
        let tree = self.inner().find_tree(tree_oid)?;
        tree.walk(TreeWalkMode::PreOrder, |path, entry| callback(path, entry))?;
        Ok(())
    }

    fn walk_tree_post<F>(&self, tree_oid: Oid, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult,
    {
        let tree = self.inner().find_tree(tree_oid)?;
        tree.walk(TreeWalkMode::PostOrder, |path, entry| callback(path, entry))?;
        Ok(())
    }

    fn get_tree_entry(&self, tree_oid: Oid, path: &Path) -> Result<Oid> {
        let tree = self.inner().find_tree(tree_oid)?;
        let entry = tree.get_path(path)?;
        Ok(entry.id())
    }

    fn read_blob_content(&self, blob_oid: Oid) -> Result<Vec<u8>> {
        let blob = self.inner().find_blob(blob_oid)?;
        Ok(blob.content().to_vec())
    }

    fn read_file_from_tree(&self, tree_oid: Oid, path: &Path) -> Result<Vec<u8>> {
        let entry_oid = self.get_tree_entry(tree_oid, path)?;
        self.read_blob_content(entry_oid)
    }

    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>> {
        let mut files = Vec::new();

        self.walk_tree_pre(tree_oid, |parent_path, entry| {
            if let Some(name) = entry.name() {
                // Only include files (blobs), not directories
                if entry.kind() == Some(ObjectType::Blob) {
                    let full_path = if parent_path.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}{}", parent_path, name)
                    };
                    files.push(full_path);
                }
            }
            TreeWalkResult::Ok
        })?;

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::objects::ObjectOps;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, JinRepo) {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

    fn create_test_tree(repo: &JinRepo) -> Oid {
        let blob1 = repo.create_blob(b"content1").unwrap();
        let blob2 = repo.create_blob(b"content2").unwrap();
        let blob3 = repo.create_blob(b"content3").unwrap();

        repo.create_tree_from_paths(&[
            ("root.txt".to_string(), blob1),
            ("src/main.rs".to_string(), blob2),
            ("src/lib.rs".to_string(), blob3),
        ])
        .unwrap()
    }

    #[test]
    fn test_walk_tree_pre_order() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let mut entries = Vec::new();
        repo.walk_tree_pre(tree_oid, |path, entry| {
            if let Some(name) = entry.name() {
                entries.push(format!("{}{}", path, name));
            }
            TreeWalkResult::Ok
        })
        .unwrap();

        // Pre-order: parent directories before children
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|e| e == "root.txt"));
        assert!(entries.iter().any(|e| e == "src"));
        assert!(entries.iter().any(|e| e == "src/main.rs"));
    }

    #[test]
    fn test_walk_tree_post_order() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let mut entries = Vec::new();
        repo.walk_tree_post(tree_oid, |path, entry| {
            if let Some(name) = entry.name() {
                entries.push(format!("{}{}", path, name));
            }
            TreeWalkResult::Ok
        })
        .unwrap();

        // Post-order: children before parent directories
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_walk_tree_abort() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let mut count = 0;
        let result = repo.walk_tree_pre(tree_oid, |_path, _entry| {
            count += 1;
            if count >= 2 {
                TreeWalkResult::Abort
            } else {
                TreeWalkResult::Ok
            }
        });

        // Walk should complete (Abort just stops early, doesn't return error)
        assert!(result.is_ok());
        assert!(count >= 2);
    }

    #[test]
    fn test_get_tree_entry() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        // Get entry for root file
        let root_oid = repo
            .get_tree_entry(tree_oid, Path::new("root.txt"))
            .unwrap();
        let content = repo.read_blob_content(root_oid).unwrap();
        assert_eq!(content, b"content1");

        // Get entry for nested file
        let nested_oid = repo
            .get_tree_entry(tree_oid, Path::new("src/main.rs"))
            .unwrap();
        let content = repo.read_blob_content(nested_oid).unwrap();
        assert_eq!(content, b"content2");
    }

    #[test]
    fn test_get_tree_entry_not_found() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let result = repo.get_tree_entry(tree_oid, Path::new("nonexistent.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_blob_content() {
        let (_temp, repo) = create_test_repo();
        let content = b"Hello, Jin!";
        let blob_oid = repo.create_blob(content).unwrap();

        let read_content = repo.read_blob_content(blob_oid).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_file_from_tree() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let content = repo
            .read_file_from_tree(tree_oid, Path::new("src/lib.rs"))
            .unwrap();
        assert_eq!(content, b"content3");
    }

    #[test]
    fn test_list_tree_files() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let files = repo.list_tree_files(tree_oid).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.contains(&"root.txt".to_string()));
        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(files.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn test_list_tree_files_excludes_directories() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = create_test_tree(&repo);

        let files = repo.list_tree_files(tree_oid).unwrap();

        // "src" directory should not be in the list
        assert!(!files.contains(&"src".to_string()));
        assert!(!files.contains(&"src/".to_string()));
    }

    #[test]
    fn test_empty_tree() {
        let (_temp, repo) = create_test_repo();
        let tree_oid = repo.create_tree(&[]).unwrap();

        let files = repo.list_tree_files(tree_oid).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_deeply_nested_tree() {
        let (_temp, repo) = create_test_repo();
        let blob = repo.create_blob(b"deep").unwrap();
        let tree_oid = repo
            .create_tree_from_paths(&[("a/b/c/d/deep.txt".to_string(), blob)])
            .unwrap();

        // Read through the deep path
        let content = repo
            .read_file_from_tree(tree_oid, &PathBuf::from("a/b/c/d/deep.txt"))
            .unwrap();
        assert_eq!(content, b"deep");

        // List should include the deep file
        let files = repo.list_tree_files(tree_oid).unwrap();
        assert!(files.contains(&"a/b/c/d/deep.txt".to_string()));
    }
}
