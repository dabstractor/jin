//! Atomic transaction support for Jin
//!
//! This module provides [`JinTransaction`], a wrapper around `git2::Transaction`
//! for atomic multi-reference updates.
//!
//! # Warning
//!
//! Note that `git2::Transaction` is **NOT truly atomic**. If one ref update fails,
//! previous successful updates are NOT rolled back. For critical operations that
//! require true atomicity, consider external locking mechanisms.

use crate::core::Result;
use git2::{Oid, Signature};

use super::JinRepo;

/// Transaction wrapper for atomic reference updates.
///
/// Provides a more ergonomic interface over `git2::Transaction` with
/// Jin-specific conventions.
///
/// # Warning
///
/// Despite the name, `git2::Transaction` is NOT truly atomic. If a ref update
/// fails after other refs have been updated, those previous updates are NOT
/// rolled back. Use with caution for critical operations.
///
/// # Example
///
/// ```no_run
/// use jin::git::{JinRepo, JinTransaction, ObjectOps, TreeEntry};
///
/// let repo = JinRepo::open()?;
/// let blob = repo.create_blob(b"content")?;
/// let tree = repo.create_tree(&[TreeEntry::blob("file.txt", blob)])?;
/// let commit = repo.create_commit(None, "test", tree, &[])?;
///
/// let mut tx = JinTransaction::new(&repo)?;
/// tx.lock_ref("refs/jin/layers/test1")?;
/// tx.lock_ref("refs/jin/layers/test2")?;
/// tx.set_target("refs/jin/layers/test1", commit, "update layer 1")?;
/// tx.set_target("refs/jin/layers/test2", commit, "update layer 2")?;
/// tx.commit()?;
/// # Ok::<(), jin::JinError>(())
/// ```
pub struct JinTransaction<'repo> {
    inner: git2::Transaction<'repo>,
}

impl<'repo> JinTransaction<'repo> {
    /// Creates a new transaction from a JinRepo.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the transaction cannot be created.
    pub fn new(repo: &'repo JinRepo) -> Result<Self> {
        let inner = repo.inner().transaction()?;
        Ok(Self { inner })
    }

    /// Locks a reference for update within this transaction.
    ///
    /// A reference must be locked before it can be updated via `set_target`.
    /// Locking prevents other processes from modifying the ref until the
    /// transaction completes.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the reference is already locked or cannot
    /// be locked.
    pub fn lock_ref(&mut self, refname: &str) -> Result<()> {
        self.inner.lock_ref(refname)?;
        Ok(())
    }

    /// Sets the target of a locked reference.
    ///
    /// The reference must have been locked via `lock_ref` before calling
    /// this method.
    ///
    /// # Arguments
    ///
    /// * `refname` - The reference name (must be locked)
    /// * `target` - The new target OID
    /// * `message` - The reflog message
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the reference was not locked.
    pub fn set_target(&mut self, refname: &str, target: Oid, message: &str) -> Result<()> {
        // Note: signature is optional in git2. If None, it reads from config.
        // We pass None to let git2 handle signature lookup.
        let sig: Option<&Signature> = None;
        self.inner.set_target(refname, target, sig, message)?;
        Ok(())
    }

    /// Removes a locked reference.
    ///
    /// The reference must have been locked via `lock_ref` before calling
    /// this method.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the reference was not locked or cannot
    /// be removed.
    pub fn remove(&mut self, refname: &str) -> Result<()> {
        self.inner.remove(refname)?;
        Ok(())
    }

    /// Commits the transaction, applying all queued updates.
    ///
    /// # Warning
    ///
    /// This is **NOT truly atomic**. If a ref update fails after other refs
    /// have been updated, those previous updates are NOT rolled back.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if any ref update fails. Note that partial
    /// updates may have already been applied.
    pub fn commit(self) -> Result<()> {
        self.inner.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::objects::{ObjectOps, TreeEntry};
    use crate::git::refs::RefOps;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, JinRepo) {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

    fn create_test_commit(repo: &JinRepo) -> Oid {
        let blob = repo.create_blob(b"content").unwrap();
        let tree = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob)])
            .unwrap();
        repo.create_commit(None, "test commit", tree, &[]).unwrap()
    }

    #[test]
    fn test_transaction_new() {
        let (_temp, repo) = create_test_repo();
        let tx = JinTransaction::new(&repo);
        assert!(tx.is_ok());
    }

    #[test]
    fn test_transaction_lock_and_set() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        let mut tx = JinTransaction::new(&repo).unwrap();
        tx.lock_ref("refs/jin/layers/tx_test").unwrap();
        tx.set_target("refs/jin/layers/tx_test", commit_oid, "transaction test")
            .unwrap();
        tx.commit().unwrap();

        // Verify the ref was created
        assert!(repo.ref_exists("refs/jin/layers/tx_test"));
        let resolved = repo.resolve_ref("refs/jin/layers/tx_test").unwrap();
        assert_eq!(resolved, commit_oid);
    }

    #[test]
    fn test_transaction_multiple_refs() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        let mut tx = JinTransaction::new(&repo).unwrap();
        tx.lock_ref("refs/jin/layers/multi1").unwrap();
        tx.lock_ref("refs/jin/layers/multi2").unwrap();
        tx.set_target("refs/jin/layers/multi1", commit_oid, "multi test 1")
            .unwrap();
        tx.set_target("refs/jin/layers/multi2", commit_oid, "multi test 2")
            .unwrap();
        tx.commit().unwrap();

        // Verify both refs were created
        assert!(repo.ref_exists("refs/jin/layers/multi1"));
        assert!(repo.ref_exists("refs/jin/layers/multi2"));
    }

    #[test]
    fn test_transaction_remove_ref() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        // First create a ref
        repo.set_ref("refs/jin/layers/to_remove", commit_oid, "to be removed")
            .unwrap();
        assert!(repo.ref_exists("refs/jin/layers/to_remove"));

        // Remove it via transaction
        let mut tx = JinTransaction::new(&repo).unwrap();
        tx.lock_ref("refs/jin/layers/to_remove").unwrap();
        tx.remove("refs/jin/layers/to_remove").unwrap();
        tx.commit().unwrap();

        // Verify the ref was removed
        assert!(!repo.ref_exists("refs/jin/layers/to_remove"));
    }

    #[test]
    fn test_transaction_without_lock_fails() {
        let (_temp, repo) = create_test_repo();
        let commit_oid = create_test_commit(&repo);

        let mut tx = JinTransaction::new(&repo).unwrap();
        // Try to set target without locking first
        let result = tx.set_target("refs/jin/layers/no_lock", commit_oid, "should fail");
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_empty_commit() {
        let (_temp, repo) = create_test_repo();

        // Committing an empty transaction should work
        let tx = JinTransaction::new(&repo).unwrap();
        let result = tx.commit();
        assert!(result.is_ok());
    }
}
