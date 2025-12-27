//! Git object creation utilities
//!
//! This module provides [`ObjectOps`], a trait for creating Git objects
//! (blobs, trees, commits) in Jin's phantom repository.

use crate::core::Result;
use git2::{Blob, Commit, Oid, Signature, Tree};
use std::collections::HashMap;
use std::path::Path;

use super::JinRepo;

/// File modes for tree entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryMode {
    /// Regular file (0o100644)
    Blob,
    /// Executable file (0o100755)
    BlobExecutable,
    /// Symbolic link (0o120000)
    Link,
    /// Directory/subtree (0o040000)
    Tree,
}

impl EntryMode {
    /// Returns the raw mode value for Git.
    pub fn as_i32(self) -> i32 {
        match self {
            EntryMode::Blob => 0o100644,
            EntryMode::BlobExecutable => 0o100755,
            EntryMode::Link => 0o120000,
            EntryMode::Tree => 0o040000,
        }
    }

    /// Creates an `EntryMode` from a raw mode value.
    pub fn from_i32(mode: i32) -> Option<Self> {
        match mode {
            0o100644 => Some(EntryMode::Blob),
            0o100755 => Some(EntryMode::BlobExecutable),
            0o120000 => Some(EntryMode::Link),
            0o040000 => Some(EntryMode::Tree),
            _ => None,
        }
    }
}

/// Tree entry for building trees.
#[derive(Debug, Clone)]
pub struct TreeEntry<'a> {
    /// The name of the entry (filename or directory name)
    pub name: &'a str,
    /// The OID of the blob or subtree
    pub oid: Oid,
    /// The entry mode
    pub mode: EntryMode,
}

impl<'a> TreeEntry<'a> {
    /// Creates a new tree entry for a regular file.
    pub fn blob(name: &'a str, oid: Oid) -> Self {
        Self {
            name,
            oid,
            mode: EntryMode::Blob,
        }
    }

    /// Creates a new tree entry for an executable file.
    pub fn blob_executable(name: &'a str, oid: Oid) -> Self {
        Self {
            name,
            oid,
            mode: EntryMode::BlobExecutable,
        }
    }

    /// Creates a new tree entry for a subtree.
    pub fn tree(name: &'a str, oid: Oid) -> Self {
        Self {
            name,
            oid,
            mode: EntryMode::Tree,
        }
    }
}

/// Object creation operations.
pub trait ObjectOps {
    /// Creates a blob from byte content.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::{JinRepo, ObjectOps};
    ///
    /// let repo = JinRepo::open()?;
    /// let oid = repo.create_blob(b"Hello, Jin!")?;
    /// # Ok::<(), jin::JinError>(())
    /// ```
    fn create_blob(&self, content: &[u8]) -> Result<Oid>;

    /// Creates a blob from a file path.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the file cannot be read.
    fn create_blob_from_path(&self, path: &Path) -> Result<Oid>;

    /// Creates a tree from entries.
    ///
    /// # Note
    ///
    /// TreeBuilder is single-level only. For nested directories, build inner
    /// trees first, then reference them in outer tree with `EntryMode::Tree`.
    fn create_tree(&self, entries: &[TreeEntry]) -> Result<Oid>;

    /// Creates a tree from file paths, handling nested directories.
    ///
    /// # Arguments
    ///
    /// * `files` - A slice of (path, oid) tuples where path can contain `/`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::{JinRepo, ObjectOps};
    ///
    /// let repo = JinRepo::open()?;
    /// let blob1 = repo.create_blob(b"content1")?;
    /// let blob2 = repo.create_blob(b"content2")?;
    ///
    /// let tree_oid = repo.create_tree_from_paths(&[
    ///     ("config.json".to_string(), blob1),
    ///     ("src/main.rs".to_string(), blob2),
    /// ])?;
    /// # Ok::<(), jin::JinError>(())
    /// ```
    fn create_tree_from_paths(&self, files: &[(String, Oid)]) -> Result<Oid>;

    /// Creates a commit.
    ///
    /// # Arguments
    ///
    /// * `update_ref` - Optional reference to update (e.g., `refs/jin/layers/global`)
    /// * `message` - The commit message
    /// * `tree_oid` - The tree to commit
    /// * `parents` - Parent commit OIDs
    fn create_commit(
        &self,
        update_ref: Option<&str>,
        message: &str,
        tree_oid: Oid,
        parents: &[Oid],
    ) -> Result<Oid>;

    /// Finds a blob by OID.
    fn find_blob(&self, oid: Oid) -> Result<Blob<'_>>;

    /// Finds a tree by OID.
    fn find_tree(&self, oid: Oid) -> Result<Tree<'_>>;

    /// Finds a commit by OID.
    fn find_commit(&self, oid: Oid) -> Result<Commit<'_>>;
}

impl ObjectOps for JinRepo {
    fn create_blob(&self, content: &[u8]) -> Result<Oid> {
        Ok(self.inner().blob(content)?)
    }

    fn create_blob_from_path(&self, path: &Path) -> Result<Oid> {
        Ok(self.inner().blob_path(path)?)
    }

    fn create_tree(&self, entries: &[TreeEntry]) -> Result<Oid> {
        let mut builder = self.inner().treebuilder(None)?;

        for entry in entries {
            builder.insert(entry.name, entry.oid, entry.mode.as_i32())?;
        }

        Ok(builder.write()?)
    }

    fn create_tree_from_paths(&self, files: &[(String, Oid)]) -> Result<Oid> {
        // Collect data with owned strings for nested tree building
        struct OwnedEntry {
            name: String,
            oid: Oid,
            mode: EntryMode,
        }

        let mut root_entries: Vec<OwnedEntry> = Vec::new();
        let mut subdirs: HashMap<String, Vec<(String, Oid)>> = HashMap::new();

        for (path, oid) in files {
            if let Some(sep_pos) = path.find('/') {
                let dir = path[..sep_pos].to_string();
                let rest = path[sep_pos + 1..].to_string();
                subdirs.entry(dir).or_default().push((rest, *oid));
            } else {
                root_entries.push(OwnedEntry {
                    name: path.clone(),
                    oid: *oid,
                    mode: EntryMode::Blob,
                });
            }
        }

        // Recursively create subdirectory trees
        for (dir_name, dir_files) in subdirs {
            let subtree_oid = self.create_tree_from_paths(&dir_files)?;
            root_entries.push(OwnedEntry {
                name: dir_name,
                oid: subtree_oid,
                mode: EntryMode::Tree,
            });
        }

        // Build the tree
        let mut builder = self.inner().treebuilder(None)?;
        for entry in &root_entries {
            builder.insert(&entry.name, entry.oid, entry.mode.as_i32())?;
        }

        Ok(builder.write()?)
    }

    fn create_commit(
        &self,
        update_ref: Option<&str>,
        message: &str,
        tree_oid: Oid,
        parents: &[Oid],
    ) -> Result<Oid> {
        let tree = self.find_tree(tree_oid)?;

        // Get signature from git config or use defaults
        let signature = self.inner().signature().unwrap_or_else(|_| {
            Signature::now("jin", "jin@local").expect("Failed to create signature")
        });

        // Resolve parent OIDs to Commit objects
        let parent_commits: Vec<Commit> = parents
            .iter()
            .map(|oid| self.find_commit(*oid))
            .collect::<Result<Vec<_>>>()?;

        let parent_refs: Vec<&Commit> = parent_commits.iter().collect();

        let oid = self.inner().commit(
            update_ref,
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;

        Ok(oid)
    }

    fn find_blob(&self, oid: Oid) -> Result<Blob<'_>> {
        Ok(self.inner().find_blob(oid)?)
    }

    fn find_tree(&self, oid: Oid) -> Result<Tree<'_>> {
        Ok(self.inner().find_tree(oid)?)
    }

    fn find_commit(&self, oid: Oid) -> Result<Commit<'_>> {
        Ok(self.inner().find_commit(oid)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, JinRepo) {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

    #[test]
    fn test_create_blob() {
        let (_temp, repo) = create_test_repo();
        let content = b"Hello, Jin!";
        let oid = repo.create_blob(content).unwrap();

        let blob = repo.find_blob(oid).unwrap();
        assert_eq!(blob.content(), content);
    }

    #[test]
    fn test_create_tree_simple() {
        let (_temp, repo) = create_test_repo();

        let blob_oid = repo.create_blob(b"content").unwrap();
        let tree_oid = repo
            .create_tree(&[TreeEntry::blob("test.txt", blob_oid)])
            .unwrap();

        let tree = repo.find_tree(tree_oid).unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.get(0).unwrap().name().unwrap(), "test.txt");
    }

    #[test]
    fn test_create_tree_nested() {
        let (_temp, repo) = create_test_repo();

        let blob1 = repo.create_blob(b"content1").unwrap();
        let blob2 = repo.create_blob(b"content2").unwrap();

        let tree_oid = repo
            .create_tree_from_paths(&[
                ("root.txt".to_string(), blob1),
                ("subdir/nested.txt".to_string(), blob2),
            ])
            .unwrap();

        let tree = repo.find_tree(tree_oid).unwrap();
        assert_eq!(tree.len(), 2);

        // Verify root.txt exists
        assert!(tree.get_name("root.txt").is_some());

        // Verify subdir exists as a tree
        let subdir_entry = tree.get_name("subdir").unwrap();
        assert_eq!(subdir_entry.kind(), Some(git2::ObjectType::Tree));
    }

    #[test]
    fn test_create_commit_initial() {
        let (_temp, repo) = create_test_repo();

        let blob_oid = repo.create_blob(b"content").unwrap();
        let tree_oid = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob_oid)])
            .unwrap();

        let commit_oid = repo
            .create_commit(None, "Initial commit", tree_oid, &[])
            .unwrap();

        let commit = repo.find_commit(commit_oid).unwrap();
        assert_eq!(commit.message().unwrap(), "Initial commit");
        assert_eq!(commit.parent_count(), 0);
    }

    #[test]
    fn test_create_commit_with_parent() {
        let (_temp, repo) = create_test_repo();

        // Create first commit
        let blob1 = repo.create_blob(b"content1").unwrap();
        let tree1 = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob1)])
            .unwrap();
        let commit1 = repo
            .create_commit(None, "First commit", tree1, &[])
            .unwrap();

        // Create second commit with first as parent
        let blob2 = repo.create_blob(b"content2").unwrap();
        let tree2 = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob2)])
            .unwrap();
        let commit2 = repo
            .create_commit(None, "Second commit", tree2, &[commit1])
            .unwrap();

        let commit = repo.find_commit(commit2).unwrap();
        assert_eq!(commit.message().unwrap(), "Second commit");
        assert_eq!(commit.parent_count(), 1);
        assert_eq!(commit.parent_id(0).unwrap(), commit1);
    }

    #[test]
    fn test_find_objects() {
        let (_temp, repo) = create_test_repo();

        let blob_oid = repo.create_blob(b"content").unwrap();
        let tree_oid = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob_oid)])
            .unwrap();
        let commit_oid = repo
            .create_commit(None, "Test commit", tree_oid, &[])
            .unwrap();

        // All objects should be findable
        assert!(repo.find_blob(blob_oid).is_ok());
        assert!(repo.find_tree(tree_oid).is_ok());
        assert!(repo.find_commit(commit_oid).is_ok());
    }

    #[test]
    fn test_entry_mode() {
        assert_eq!(EntryMode::Blob.as_i32(), 0o100644);
        assert_eq!(EntryMode::BlobExecutable.as_i32(), 0o100755);
        assert_eq!(EntryMode::Link.as_i32(), 0o120000);
        assert_eq!(EntryMode::Tree.as_i32(), 0o040000);

        assert_eq!(EntryMode::from_i32(0o100644), Some(EntryMode::Blob));
        assert_eq!(
            EntryMode::from_i32(0o100755),
            Some(EntryMode::BlobExecutable)
        );
        assert_eq!(EntryMode::from_i32(0o120000), Some(EntryMode::Link));
        assert_eq!(EntryMode::from_i32(0o040000), Some(EntryMode::Tree));
        assert_eq!(EntryMode::from_i32(12345), None);
    }

    #[test]
    fn test_tree_entry_constructors() {
        let oid = Oid::from_str("0000000000000000000000000000000000000001").unwrap();

        let blob_entry = TreeEntry::blob("file.txt", oid);
        assert_eq!(blob_entry.mode, EntryMode::Blob);

        let exec_entry = TreeEntry::blob_executable("script.sh", oid);
        assert_eq!(exec_entry.mode, EntryMode::BlobExecutable);

        let tree_entry = TreeEntry::tree("subdir", oid);
        assert_eq!(tree_entry.mode, EntryMode::Tree);
    }

    #[test]
    fn test_deeply_nested_paths() {
        let (_temp, repo) = create_test_repo();

        let blob = repo.create_blob(b"deep content").unwrap();
        let tree_oid = repo
            .create_tree_from_paths(&[("a/b/c/deep.txt".to_string(), blob)])
            .unwrap();

        let tree = repo.find_tree(tree_oid).unwrap();
        assert_eq!(tree.len(), 1);

        // Verify the structure exists
        let a_entry = tree.get_name("a").unwrap();
        assert_eq!(a_entry.kind(), Some(git2::ObjectType::Tree));
    }
}
