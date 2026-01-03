//! Reference operations for Jin
//!
//! This module provides [`RefOps`], a trait for managing Git references
//! under the `refs/jin/layers/*` namespace.

use crate::core::{JinError, Result};
use git2::{Oid, Reference};

use super::JinRepo;

/// Result of comparing two Git references
///
/// Used to determine if a push operation is safe to execute.
/// Derived from git's graph_ahead_behind comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefComparison {
    /// Local commit is ahead of remote (fast-forward possible)
    Ahead,

    /// Local commit is behind remote (must pull first)
    Behind,

    /// Local and remote have diverged (merge required)
    Diverged,

    /// Local and remote point to the same commit
    Equal,
}

/// Trait for reference operations in Jin's phantom repository.
///
/// All references are stored under the `refs/jin/` namespace to avoid
/// conflicts with regular Git operations.
pub trait RefOps {
    /// Finds a reference by name.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the reference is not found.
    fn find_ref(&self, name: &str) -> Result<Reference<'_>>;

    /// Creates or updates a reference to point to an OID.
    ///
    /// # Arguments
    ///
    /// * `name` - The reference name (e.g., `refs/jin/layers/global`)
    /// * `oid` - The target object ID
    /// * `message` - The reflog message
    ///
    /// # Errors
    ///
    /// Returns `JinError::InvalidLayer` if the reference name is invalid.
    fn set_ref(&self, name: &str, oid: Oid, message: &str) -> Result<()>;

    /// Deletes a reference.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the reference is not found.
    fn delete_ref(&self, name: &str) -> Result<()>;

    /// Lists references matching a glob pattern.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A glob pattern (e.g., `refs/jin/layers/*`)
    ///
    /// # Returns
    ///
    /// A vector of reference names matching the pattern.
    fn list_refs(&self, pattern: &str) -> Result<Vec<String>>;

    /// Checks if a reference exists.
    fn ref_exists(&self, name: &str) -> bool;

    /// Gets the OID a reference points to (resolving symbolic refs).
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the reference cannot be resolved.
    fn resolve_ref(&self, name: &str) -> Result<Oid>;
}

impl RefOps for JinRepo {
    fn find_ref(&self, name: &str) -> Result<Reference<'_>> {
        Ok(self.inner().find_reference(name)?)
    }

    fn set_ref(&self, name: &str, oid: Oid, message: &str) -> Result<()> {
        // Validate reference name
        if !Reference::is_valid_name(name) {
            return Err(JinError::InvalidLayer(format!(
                "Invalid reference name: {}",
                name
            )));
        }

        // Create or update the reference
        self.inner().reference(name, oid, true, message)?;
        Ok(())
    }

    fn delete_ref(&self, name: &str) -> Result<()> {
        let mut reference = self.find_ref(name)?;
        reference.delete()?;
        Ok(())
    }

    fn list_refs(&self, pattern: &str) -> Result<Vec<String>> {
        let refs = self.inner().references_glob(pattern)?;
        let mut names = Vec::new();

        for ref_result in refs {
            let reference = ref_result?;
            if let Some(name) = reference.name() {
                names.push(name.to_string());
            }
        }

        Ok(names)
    }

    fn ref_exists(&self, name: &str) -> bool {
        self.inner().find_reference(name).is_ok()
    }

    fn resolve_ref(&self, name: &str) -> Result<Oid> {
        let reference = self.find_ref(name)?;
        let resolved = reference.resolve()?;
        resolved
            .target()
            .ok_or_else(|| JinError::Git(git2::Error::from_str("Reference has no target")))
    }
}

/// Compare local and remote Git references
///
/// Determines the relationship between two commits by analyzing
/// the commit graph. Returns whether local is ahead, behind,
/// diverged, or equal to remote.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `local_oid` - OID of local commit
/// * `remote_oid` - OID of remote commit
///
/// # Returns
///
/// `RefComparison` indicating the state relationship
///
/// # Errors
///
/// Returns `JinError::Git` if commit graph analysis fails
///
/// # Algorithm
///
/// Uses git2's `graph_ahead_behind` which counts commits
/// reachable from each ref but not the other:
/// - (0, 0) -> Same commit
/// - (n, 0) -> Local is ahead (n commits ahead, 0 behind)
/// - (0, n) -> Local is behind (0 ahead, n behind)
/// - (m, n) -> Diverged (both have unique commits)
pub fn compare_refs(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<RefComparison> {
    // CRITICAL: graph_ahead_behind returns (ahead_count, behind_count)
    // This is the core of git's ahead/behind detection
    // Note: API takes Oid values directly, not commit references
    let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;

    // Match on counts to determine state
    match (ahead, behind) {
        (0, 0) => Ok(RefComparison::Equal),
        (_, 0) => Ok(RefComparison::Ahead),
        (0, _) => Ok(RefComparison::Behind),
        (_, _) => Ok(RefComparison::Diverged),
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

    fn create_test_blob(repo: &JinRepo) -> Oid {
        repo.inner().blob(b"test content").unwrap()
    }

    fn create_test_commit(repo: &JinRepo) -> Oid {
        let blob_oid = create_test_blob(repo);

        // Create a tree with the blob
        let mut builder = repo.inner().treebuilder(None).unwrap();
        builder.insert("test.txt", blob_oid, 0o100644).unwrap();
        let tree_oid = builder.write().unwrap();
        let tree = repo.inner().find_tree(tree_oid).unwrap();

        // Create a signature
        let sig = git2::Signature::now("test", "test@test.com").unwrap();

        // Create the commit
        repo.inner()
            .commit(None, &sig, &sig, "test commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_set_and_find_ref() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        // Set a reference
        repo.set_ref("refs/jin/layers/test", commit_oid, "test ref")
            .unwrap();

        // Find the reference
        let reference = repo.find_ref("refs/jin/layers/test").unwrap();
        assert_eq!(reference.target().unwrap(), commit_oid);
    }

    #[test]
    fn test_delete_ref() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        // Create reference
        repo.set_ref("refs/jin/layers/delete_me", commit_oid, "to be deleted")
            .unwrap();
        assert!(repo.ref_exists("refs/jin/layers/delete_me"));

        // Delete reference
        repo.delete_ref("refs/jin/layers/delete_me").unwrap();
        assert!(!repo.ref_exists("refs/jin/layers/delete_me"));
    }

    #[test]
    fn test_list_refs_glob() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        // Create multiple references
        repo.set_ref("refs/jin/layers/layer1", commit_oid, "layer 1")
            .unwrap();
        repo.set_ref("refs/jin/layers/layer2", commit_oid, "layer 2")
            .unwrap();
        repo.set_ref("refs/jin/layers/mode/claude", commit_oid, "claude mode")
            .unwrap();

        // List all jin layer refs
        let refs = repo.list_refs("refs/jin/layers/*").unwrap();
        assert!(refs.len() >= 2);
        assert!(refs.iter().any(|r| r.contains("layer1")));
        assert!(refs.iter().any(|r| r.contains("layer2")));
    }

    #[test]
    fn test_ref_exists() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        assert!(!repo.ref_exists("refs/jin/layers/exists_test"));

        repo.set_ref("refs/jin/layers/exists_test", commit_oid, "test")
            .unwrap();

        assert!(repo.ref_exists("refs/jin/layers/exists_test"));
    }

    #[test]
    fn test_resolve_ref() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        repo.set_ref("refs/jin/layers/resolve_test", commit_oid, "test")
            .unwrap();

        let resolved = repo.resolve_ref("refs/jin/layers/resolve_test").unwrap();
        assert_eq!(resolved, commit_oid);
    }

    #[test]
    fn test_invalid_ref_name() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        // Invalid reference names should fail
        let result = repo.set_ref("invalid ref name with spaces", commit_oid, "test");
        assert!(result.is_err());
        if let Err(JinError::InvalidLayer(msg)) = result {
            assert!(msg.contains("Invalid reference name"));
        } else {
            panic!("Expected InvalidLayer error");
        }
    }

    #[test]
    fn test_ref_ops_trait_exists() {
        // Verify the trait compiles
        fn _takes_ref_ops<T: RefOps>(_: &T) {}
    }

    #[test]
    fn test_compare_refs_equal() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        // Same commit should be Equal
        let result = compare_refs(&repo, commit_oid, commit_oid);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RefComparison::Equal);
    }

    #[test]
    fn test_compare_refs_ahead() {
        let (_temp, repo) = create_test_repo();

        // Create first commit (base)
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let blob_oid = create_test_blob(&repo);
        let mut builder = repo.inner().treebuilder(None).unwrap();
        builder.insert("base.txt", blob_oid, 0o100644).unwrap();
        let tree_oid = builder.write().unwrap();
        let tree = repo.inner().find_tree(tree_oid).unwrap();
        let base_commit = repo
            .inner()
            .commit(None, &sig, &sig, "base", &tree, &[])
            .unwrap();
        let base_commit_obj = repo.inner().find_commit(base_commit).unwrap();

        // Create second commit on top of base (ahead)
        let mut builder2 = repo.inner().treebuilder(None).unwrap();
        builder2.insert("base.txt", blob_oid, 0o100644).unwrap();
        builder2
            .insert("new.txt", create_test_blob(&repo), 0o100644)
            .unwrap();
        let tree_oid2 = builder2.write().unwrap();
        let tree2 = repo.inner().find_tree(tree_oid2).unwrap();
        let ahead_commit = repo
            .inner()
            .commit(None, &sig, &sig, "ahead", &tree2, &[&base_commit_obj])
            .unwrap();

        // ahead_commit should be Ahead of base_commit
        let result = compare_refs(&repo, ahead_commit, base_commit);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RefComparison::Ahead);

        // And base_commit should be Behind ahead_commit
        let result = compare_refs(&repo, base_commit, ahead_commit);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RefComparison::Behind);
    }

    #[test]
    fn test_compare_refs_diverged() {
        let (_temp, repo) = create_test_repo();

        // Create base commit
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let blob_oid = create_test_blob(&repo);
        let mut builder = repo.inner().treebuilder(None).unwrap();
        builder.insert("base.txt", blob_oid, 0o100644).unwrap();
        let tree_oid = builder.write().unwrap();
        let tree = repo.inner().find_tree(tree_oid).unwrap();
        let base_commit = repo
            .inner()
            .commit(None, &sig, &sig, "base", &tree, &[])
            .unwrap();
        let base_commit_obj = repo.inner().find_commit(base_commit).unwrap();

        // Create diverged commit 1
        let mut builder1 = repo.inner().treebuilder(None).unwrap();
        builder1
            .insert("diverged1.txt", create_test_blob(&repo), 0o100644)
            .unwrap();
        let tree_oid1 = builder1.write().unwrap();
        let tree1 = repo.inner().find_tree(tree_oid1).unwrap();
        let diverged1 = repo
            .inner()
            .commit(None, &sig, &sig, "diverged1", &tree1, &[&base_commit_obj])
            .unwrap();

        // Create diverged commit 2 (same parent, different content)
        let mut builder2 = repo.inner().treebuilder(None).unwrap();
        builder2
            .insert("diverged2.txt", create_test_blob(&repo), 0o100644)
            .unwrap();
        let tree_oid2 = builder2.write().unwrap();
        let tree2 = repo.inner().find_tree(tree_oid2).unwrap();
        let diverged2 = repo
            .inner()
            .commit(None, &sig, &sig, "diverged2", &tree2, &[&base_commit_obj])
            .unwrap();

        // diverged1 and diverged2 should be Diverged
        let result = compare_refs(&repo, diverged1, diverged2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RefComparison::Diverged);
    }
}
