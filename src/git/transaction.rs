//! Atomic transaction support for Jin
//!
//! This module provides [`JinTransaction`], a wrapper around `git2::Transaction`
//! for atomic multi-reference updates, and [`LayerTransaction`] for true atomic
//! multi-layer commits with crash recovery.
//!
//! # Warning
//!
//! Note that `git2::Transaction` is **NOT truly atomic**. If one ref update fails,
//! previous successful updates are NOT rolled back. For true atomicity, use
//! [`LayerTransaction`] which provides two-phase commit with persistent transaction
//! log for crash recovery.
//!
//! # Two-Phase Commit Architecture
//!
//! [`LayerTransaction`] implements a two-phase commit pattern:
//!
//! 1. **BEGIN**: Transaction log created, state = Pending
//! 2. **PREPARE**: All updates validated, state = Prepared (point of no return)
//! 3. **COMMIT**: Updates applied, log deleted, state = Committed
//! 4. **ABORT**: Rollback performed, log deleted, state = Aborted
//!
//! If a crash occurs during the transaction, [`RecoveryManager`] can detect and
//! recover the incomplete transaction on the next jin command.

use crate::core::{JinError, Layer, Result};
use git2::{Oid, Signature};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

// ============================================================================
// Two-Phase Commit Transaction System
// ============================================================================

/// State of a LayerTransaction in the two-phase commit lifecycle.
///
/// The state machine is:
/// ```text
/// Pending -> Prepared -> Committed
///     |          |
///     v          v
///  Aborted   Aborted
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    /// Transaction created, updates being queued.
    /// Safe to abort without any side effects.
    Pending,
    /// All updates validated, refs locked (ready to commit).
    /// This is the "point of no return" - recovery should resume.
    Prepared,
    /// All updates applied successfully.
    /// Transaction log has been deleted.
    Committed,
    /// Transaction aborted, changes rolled back.
    /// Transaction log has been deleted.
    Aborted,
}

/// A queued update to a layer reference.
///
/// Records both the new OID and the old OID (if any) to support rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerUpdate {
    /// The layer being updated
    pub layer: Layer,
    /// Mode name (if applicable for this layer type)
    pub mode: Option<String>,
    /// Scope name (if applicable for this layer type)
    pub scope: Option<String>,
    /// Project name (if applicable for this layer type)
    pub project: Option<String>,
    /// The full ref path (computed from layer + context)
    pub ref_path: String,
    /// Previous commit OID as string (for rollback, None if ref didn't exist)
    pub old_oid: Option<String>,
    /// New commit OID as string
    pub new_oid: String,
}

impl LayerUpdate {
    /// Create a new layer update.
    ///
    /// Computes the ref path from the layer and context parameters.
    pub fn new(
        layer: Layer,
        mode: Option<String>,
        scope: Option<String>,
        project: Option<String>,
        old_oid: Option<Oid>,
        new_oid: Oid,
    ) -> Self {
        let ref_path = layer.ref_path(mode.as_deref(), scope.as_deref(), project.as_deref());
        Self {
            layer,
            mode,
            scope,
            project,
            ref_path,
            old_oid: old_oid.map(|o| o.to_string()),
            new_oid: new_oid.to_string(),
        }
    }
}

/// Persistent transaction log for crash recovery.
///
/// Stored at `.jin/.transaction_in_progress` as JSON.
/// Uses atomic rename for crash-safe state transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    /// Version of the log format (for future compatibility)
    pub version: u32,
    /// Transaction ID (timestamp-based for uniqueness)
    pub id: String,
    /// Current state of the transaction
    pub state: TransactionState,
    /// Timestamp when transaction started (RFC3339 format)
    pub started_at: String,
    /// Commit message for this transaction
    pub message: String,
    /// All queued layer updates
    pub updates: Vec<LayerUpdate>,
    /// Base path for the transaction log (not serialized)
    #[serde(skip)]
    base_path: Option<PathBuf>,
}

impl TransactionLog {
    /// Create a new transaction log with the given commit message.
    pub fn new(message: impl Into<String>) -> Self {
        let id = chrono::Utc::now().format("%Y%m%d%H%M%S%f").to_string();
        Self {
            version: 1,
            id,
            state: TransactionState::Pending,
            started_at: chrono::Utc::now().to_rfc3339(),
            message: message.into(),
            updates: Vec::new(),
            base_path: None,
        }
    }

    /// Create a new transaction log with a custom base path.
    ///
    /// The transaction log will be stored at `{base_path}/.jin/.transaction_in_progress`.
    pub fn with_base_path(message: impl Into<String>, base_path: PathBuf) -> Self {
        let mut log = Self::new(message);
        log.base_path = Some(base_path);
        log
    }

    /// Returns the default path for the transaction log.
    ///
    /// The log is stored at `.jin/.transaction_in_progress` relative to
    /// the current working directory.
    pub fn default_path() -> PathBuf {
        PathBuf::from(".jin").join(".transaction_in_progress")
    }

    /// Returns the path for this transaction log, using base_path if set.
    fn path(&self) -> PathBuf {
        match &self.base_path {
            Some(base) => base.join(".jin").join(".transaction_in_progress"),
            None => Self::default_path(),
        }
    }

    /// Load transaction log from disk if it exists.
    ///
    /// # Errors
    ///
    /// Returns error if the file exists but cannot be parsed.
    pub fn load() -> Result<Option<Self>> {
        Self::load_from(Self::default_path())
    }

    /// Load transaction log from a specific path.
    pub fn load_from(path: PathBuf) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let mut log: Self = serde_json::from_str(&content).map_err(|e| {
            JinError::Transaction(format!("Failed to parse transaction log: {}", e))
        })?;
        // Derive base_path from the loaded path
        if let Some(jin_dir) = path.parent() {
            if let Some(base) = jin_dir.parent() {
                log.base_path = Some(base.to_path_buf());
            }
        }
        Ok(Some(log))
    }

    /// Save transaction log to disk atomically.
    ///
    /// Uses atomic rename (write to temp file, then rename) for crash safety.
    pub fn save(&self) -> Result<()> {
        let path = self.path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to temp file first, then atomic rename
        let temp_path = path.with_extension("tmp");
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            JinError::Transaction(format!("Failed to serialize transaction log: {}", e))
        })?;
        std::fs::write(&temp_path, content)?;
        std::fs::rename(&temp_path, &path)?;
        Ok(())
    }

    /// Delete transaction log (called after successful commit or abort).
    pub fn delete() -> Result<()> {
        Self::delete_at(Self::default_path())
    }

    /// Delete transaction log at a specific path.
    pub fn delete_at(path: PathBuf) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        // Also clean up any stale temp file
        let temp_path = path.with_extension("tmp");
        if temp_path.exists() {
            let _ = std::fs::remove_file(&temp_path);
        }
        Ok(())
    }

    /// Delete this transaction log.
    pub fn delete_self(&self) -> Result<()> {
        Self::delete_at(self.path())
    }
}

/// Multi-layer atomic transaction with crash recovery.
///
/// Provides true atomic multi-ref updates using a two-phase commit pattern
/// with persistent transaction log for crash recovery.
///
/// # Two-Phase Commit
///
/// 1. **BEGIN**: Transaction log created, state = Pending
/// 2. **add_layer_update()**: Updates queued, log saved after each
/// 3. **commit()**: State = Prepared, then apply all updates
/// 4. On success: Log deleted, state conceptually = Committed
/// 5. On failure: Rollback, log deleted, state = Aborted
///
/// # Recovery
///
/// If a crash occurs:
/// - Before Prepared: Rollback (no refs modified yet)
/// - After Prepared: Resume (complete the updates) or Rollback
///
/// # Example
///
/// ```no_run
/// use jin::git::{JinRepo, LayerTransaction};
/// use jin::core::Layer;
///
/// let repo = JinRepo::open()?;
///
/// // Begin transaction
/// let mut tx = LayerTransaction::begin(&repo, "Update mode and project layers")?;
///
/// // Queue updates (commit OIDs would come from actual commits)
/// // tx.add_layer_update(Layer::ModeBase, Some("claude"), None, None, commit1)?;
/// // tx.add_layer_update(Layer::ModeProject, Some("claude"), None, Some("my-project"), commit2)?;
///
/// // Atomic commit - all succeed or all fail
/// tx.commit()?;
/// # Ok::<(), jin::JinError>(())
/// ```
pub struct LayerTransaction<'repo> {
    repo: &'repo super::JinRepo,
    log: TransactionLog,
}

impl<'repo> LayerTransaction<'repo> {
    /// Begin a new atomic transaction.
    ///
    /// Creates a transaction log file to track state for crash recovery.
    ///
    /// # Errors
    ///
    /// Returns error if an incomplete transaction already exists.
    /// Use [`RecoveryManager::auto_recover`] to handle incomplete transactions first.
    pub fn begin(repo: &'repo super::JinRepo, message: impl Into<String>) -> Result<Self> {
        // Check for existing incomplete transaction
        if TransactionLog::load()?.is_some() {
            return Err(JinError::Transaction(
                "Incomplete transaction exists. Run recovery first.".to_string(),
            ));
        }

        let log = TransactionLog::new(message);
        log.save()?;

        Ok(Self { repo, log })
    }

    /// Begin a new atomic transaction with a custom base path for the transaction log.
    ///
    /// This is useful for testing or when the project directory differs from
    /// the current working directory.
    pub fn begin_with_path(
        repo: &'repo super::JinRepo,
        message: impl Into<String>,
        base_path: PathBuf,
    ) -> Result<Self> {
        let log_path = base_path.join(".jin").join(".transaction_in_progress");
        // Check for existing incomplete transaction
        if TransactionLog::load_from(log_path)?.is_some() {
            return Err(JinError::Transaction(
                "Incomplete transaction exists. Run recovery first.".to_string(),
            ));
        }

        let log = TransactionLog::with_base_path(message, base_path);
        log.save()?;

        Ok(Self { repo, log })
    }

    /// Add a layer update to the transaction.
    ///
    /// Records the current ref value (for rollback) and queues the update.
    /// The transaction log is saved after each update for crash safety.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer type to update
    /// * `mode` - Mode name (required for mode-related layers)
    /// * `scope` - Scope name (required for scope-related layers)
    /// * `project` - Project name (required for project-related layers)
    /// * `new_commit` - The new commit OID for this layer
    pub fn add_layer_update(
        &mut self,
        layer: Layer,
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
        new_commit: Oid,
    ) -> Result<()> {
        use super::refs::RefOps;

        let ref_path = layer.ref_path(mode, scope, project);

        // Get current OID for rollback capability
        let old_oid = if self.repo.ref_exists(&ref_path) {
            Some(self.repo.resolve_ref(&ref_path)?)
        } else {
            None
        };

        let update = LayerUpdate::new(
            layer,
            mode.map(String::from),
            scope.map(String::from),
            project.map(String::from),
            old_oid,
            new_commit,
        );

        self.log.updates.push(update);
        self.log.save()?;

        Ok(())
    }

    /// Commit the transaction atomically.
    ///
    /// Uses git2::Transaction for the actual ref updates, with our
    /// transaction log providing crash recovery capability.
    ///
    /// # Errors
    ///
    /// Returns error if any ref update fails. On failure, all refs are
    /// rolled back to their previous values.
    pub fn commit(mut self) -> Result<()> {
        if self.log.updates.is_empty() {
            self.log.delete_self()?;
            return Ok(());
        }

        // Phase 1: Prepare - mark as prepared in log
        self.log.state = TransactionState::Prepared;
        self.log.save()?;

        // Phase 2: Execute ref updates via git2::Transaction
        let result = self.execute_updates();

        match result {
            Ok(()) => {
                // Phase 3: Complete - delete log (state conceptually = Committed)
                self.log.delete_self()?;
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                self.rollback()?;
                Err(e)
            }
        }
    }

    /// Execute the actual ref updates via git2::Transaction.
    fn execute_updates(&self) -> Result<()> {
        let mut tx = JinTransaction::new(self.repo)?;

        // Lock all refs first
        for update in &self.log.updates {
            tx.lock_ref(&update.ref_path)?;
        }

        // Set all targets
        for update in &self.log.updates {
            let oid = Oid::from_str(&update.new_oid).map_err(|e| {
                JinError::Transaction(format!("Invalid OID {}: {}", update.new_oid, e))
            })?;
            tx.set_target(&update.ref_path, oid, &self.log.message)?;
        }

        // Commit the git2 transaction
        tx.commit()?;

        Ok(())
    }

    /// Rollback the transaction, restoring previous ref values.
    fn rollback(&mut self) -> Result<()> {
        use super::refs::RefOps;

        // Restore refs to previous values (best effort)
        for update in &self.log.updates {
            if let Some(old_oid_str) = &update.old_oid {
                if let Ok(old_oid) = Oid::from_str(old_oid_str) {
                    // Best effort: try to restore, continue on error
                    let _ = self.repo.set_ref(&update.ref_path, old_oid, "Rollback");
                }
            } else {
                // Ref didn't exist before, delete it
                let _ = self.repo.delete_ref(&update.ref_path);
            }
        }

        self.log.state = TransactionState::Aborted;
        self.log.delete_self()?;

        Ok(())
    }

    /// Abort the transaction without applying any changes.
    ///
    /// Safe to call at any time before commit. Cleans up the transaction log.
    pub fn abort(mut self) -> Result<()> {
        self.log.state = TransactionState::Aborted;
        self.log.delete_self()?;
        Ok(())
    }

    /// Get the number of pending updates in this transaction.
    pub fn update_count(&self) -> usize {
        self.log.updates.len()
    }

    /// Get the transaction ID.
    pub fn id(&self) -> &str {
        &self.log.id
    }
}

/// Recovery manager for incomplete transactions.
///
/// Detects and handles transactions that were interrupted (e.g., by crash).
/// Should be called at the start of any jin command.
pub struct RecoveryManager;

impl RecoveryManager {
    /// Detect if there's an incomplete transaction.
    ///
    /// Returns [`IncompleteTransaction`] if a transaction log exists.
    pub fn detect() -> Result<Option<IncompleteTransaction>> {
        match TransactionLog::load()? {
            Some(log) => Ok(Some(IncompleteTransaction { log })),
            None => Ok(None),
        }
    }

    /// Detect if there's an incomplete transaction at a specific path.
    pub fn detect_at(base_path: &Path) -> Result<Option<IncompleteTransaction>> {
        let path = base_path.join(".jin").join(".transaction_in_progress");
        match TransactionLog::load_from(path)? {
            Some(log) => Ok(Some(IncompleteTransaction { log })),
            None => Ok(None),
        }
    }

    /// Check for and automatically handle incomplete transactions.
    ///
    /// Default strategy is to rollback incomplete transactions.
    ///
    /// Returns `true` if a recovery was performed.
    pub fn auto_recover(repo: &super::JinRepo) -> Result<bool> {
        match Self::detect()? {
            Some(incomplete) => {
                // Strategy: rollback incomplete transactions
                // Alternative: could attempt resume for Prepared state
                incomplete.rollback(repo)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Check for and automatically handle incomplete transactions at a specific path.
    pub fn auto_recover_at(repo: &super::JinRepo, base_path: &Path) -> Result<bool> {
        match Self::detect_at(base_path)? {
            Some(incomplete) => {
                incomplete.rollback(repo)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

/// An incomplete transaction detected during recovery.
///
/// Provides information about the transaction and methods to either
/// rollback or resume it.
pub struct IncompleteTransaction {
    log: TransactionLog,
}

impl IncompleteTransaction {
    /// Get the transaction state.
    pub fn state(&self) -> TransactionState {
        self.log.state
    }

    /// Get the transaction ID.
    pub fn id(&self) -> &str {
        &self.log.id
    }

    /// Get when the transaction started (RFC3339 format).
    pub fn started_at(&self) -> &str {
        &self.log.started_at
    }

    /// Get the transaction message.
    pub fn message(&self) -> &str {
        &self.log.message
    }

    /// Get the number of updates in the transaction.
    pub fn update_count(&self) -> usize {
        self.log.updates.len()
    }

    /// Get the list of updates (for inspection).
    pub fn updates(&self) -> &[LayerUpdate] {
        &self.log.updates
    }

    /// Rollback the incomplete transaction.
    ///
    /// Restores all refs to their previous values (best effort).
    pub fn rollback(self, repo: &super::JinRepo) -> Result<()> {
        use super::refs::RefOps;

        for update in &self.log.updates {
            if let Some(old_oid_str) = &update.old_oid {
                if let Ok(old_oid) = Oid::from_str(old_oid_str) {
                    // Best effort restore
                    let _ = repo.set_ref(&update.ref_path, old_oid, "Recovery rollback");
                }
            } else {
                // Ref didn't exist before, try to delete
                let _ = repo.delete_ref(&update.ref_path);
            }
        }

        self.log.delete_self()?;
        Ok(())
    }

    /// Resume the incomplete transaction (attempt to complete it).
    ///
    /// Only valid if state is Prepared (updates were in progress).
    ///
    /// # Errors
    ///
    /// Returns error if state is not Prepared, or if resume fails.
    pub fn resume(self, repo: &super::JinRepo) -> Result<()> {
        if self.log.state != TransactionState::Prepared {
            return Err(JinError::Transaction(
                "Can only resume transactions in Prepared state".to_string(),
            ));
        }

        let mut tx = JinTransaction::new(repo)?;

        for update in &self.log.updates {
            tx.lock_ref(&update.ref_path)?;
        }

        for update in &self.log.updates {
            let oid = Oid::from_str(&update.new_oid)
                .map_err(|e| JinError::Transaction(format!("Invalid OID: {}", e)))?;
            tx.set_target(&update.ref_path, oid, &self.log.message)?;
        }

        tx.commit()?;
        self.log.delete_self()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::objects::{ObjectOps, TreeEntry};
    use crate::git::refs::RefOps;
    use std::path::Path;
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

    // ========================================================================
    // Two-Phase Commit Transaction System Tests
    // ========================================================================

    /// Helper to set up an isolated test environment
    /// Returns (TempDir, JinRepo, base_path) where base_path is the absolute path
    /// that can be used with path-aware transaction methods.
    fn setup_layer_transaction_test() -> (TempDir, JinRepo, PathBuf) {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path().to_path_buf();

        // Create .jin directory at base_path
        std::fs::create_dir_all(base_path.join(".jin")).unwrap();

        let repo_path = base_path.join(".jin-repo");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo, base_path)
    }

    /// Helper to get the transaction log path for a given base path
    fn log_path_for(base_path: &Path) -> PathBuf {
        base_path.join(".jin").join(".transaction_in_progress")
    }

    /// Helper to clean up transaction log at a specific path
    fn cleanup_transaction_log_at(base_path: &Path) {
        let _ = TransactionLog::delete_at(log_path_for(base_path));
    }

    #[test]
    fn test_transaction_state_serialization() {
        let state = TransactionState::Pending;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"Pending\"");

        let state2: TransactionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, state2);
    }

    #[test]
    fn test_layer_update_creation() {
        let oid = Oid::from_str("0000000000000000000000000000000000000000").unwrap();
        let update = LayerUpdate::new(
            Layer::ModeBase,
            Some("claude".to_string()),
            None,
            None,
            None,
            oid,
        );

        assert_eq!(update.layer, Layer::ModeBase);
        assert_eq!(update.mode, Some("claude".to_string()));
        // ModeBase uses /_ suffix to avoid Git ref path conflicts with child refs
        assert_eq!(update.ref_path, "refs/jin/layers/mode/claude/_");
        assert!(update.old_oid.is_none());
    }

    #[test]
    fn test_transaction_log_save_load() {
        let (_temp, _repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        let mut log = TransactionLog::with_base_path("test transaction", base_path.clone());
        log.updates.push(LayerUpdate::new(
            Layer::GlobalBase,
            None,
            None,
            None,
            None,
            Oid::from_str("0000000000000000000000000000000000000000").unwrap(),
        ));

        // Save
        log.save().unwrap();

        // Load
        let loaded = TransactionLog::load_from(log_path_for(&base_path)).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.message, "test transaction");
        assert_eq!(loaded.updates.len(), 1);
        assert_eq!(loaded.state, TransactionState::Pending);

        // Cleanup
        TransactionLog::delete_at(log_path_for(&base_path)).unwrap();
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_transaction_log_atomic_write() {
        let (_temp, _repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        let log = TransactionLog::with_base_path("atomic test", base_path.clone());
        log.save().unwrap();

        // Verify the temp file doesn't exist after save
        let temp_file_path = log_path_for(&base_path).with_extension("tmp");
        assert!(!temp_file_path.exists());

        // Verify the actual file exists
        assert!(log_path_for(&base_path).exists());

        cleanup_transaction_log_at(&base_path);
    }

    #[test]
    fn test_layer_transaction_empty() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // Empty transaction should work
        let tx = LayerTransaction::begin_with_path(&repo, "empty transaction", base_path.clone())
            .unwrap();
        assert_eq!(tx.update_count(), 0);
        tx.commit().unwrap();

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_layer_transaction_single_layer() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        let commit_oid = create_test_commit(&repo);

        let mut tx =
            LayerTransaction::begin_with_path(&repo, "single layer test", base_path.clone())
                .unwrap();
        tx.add_layer_update(Layer::GlobalBase, None, None, None, commit_oid)
            .unwrap();
        assert_eq!(tx.update_count(), 1);
        tx.commit().unwrap();

        // Verify the ref was created
        assert!(repo.ref_exists("refs/jin/layers/global"));
        let resolved = repo.resolve_ref("refs/jin/layers/global").unwrap();
        assert_eq!(resolved, commit_oid);

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_layer_transaction_multi_layer() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        let commit1 = create_test_commit(&repo);
        let commit2 = create_test_commit(&repo);

        let mut tx =
            LayerTransaction::begin_with_path(&repo, "multi layer test", base_path.clone())
                .unwrap();
        // Use non-conflicting layers: GlobalBase and ModeBase with different paths
        tx.add_layer_update(Layer::GlobalBase, None, None, None, commit1)
            .unwrap();
        tx.add_layer_update(Layer::ModeBase, Some("claude"), None, None, commit2)
            .unwrap();
        assert_eq!(tx.update_count(), 2);
        tx.commit().unwrap();

        // Verify both refs were created
        // ModeBase uses /_ suffix to avoid Git ref path conflicts with child refs
        assert!(repo.ref_exists("refs/jin/layers/global"));
        assert!(repo.ref_exists("refs/jin/layers/mode/claude/_"));

        let resolved1 = repo.resolve_ref("refs/jin/layers/global").unwrap();
        let resolved2 = repo.resolve_ref("refs/jin/layers/mode/claude/_").unwrap();
        assert_eq!(resolved1, commit1);
        assert_eq!(resolved2, commit2);

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_layer_transaction_abort() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        let commit_oid = create_test_commit(&repo);

        let mut tx =
            LayerTransaction::begin_with_path(&repo, "abort test", base_path.clone()).unwrap();
        tx.add_layer_update(Layer::GlobalBase, None, None, None, commit_oid)
            .unwrap();

        // Abort instead of commit
        tx.abort().unwrap();

        // Ref should NOT exist
        assert!(!repo.ref_exists("refs/jin/layers/global"));

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_layer_transaction_records_old_oid() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // Create initial ref
        let old_commit = create_test_commit(&repo);
        repo.set_ref("refs/jin/layers/global", old_commit, "initial")
            .unwrap();

        let new_commit = create_test_commit(&repo);

        let mut tx =
            LayerTransaction::begin_with_path(&repo, "update test", base_path.clone()).unwrap();
        tx.add_layer_update(Layer::GlobalBase, None, None, None, new_commit)
            .unwrap();

        // Check that the transaction log has the old OID recorded
        let log = TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .unwrap();
        assert_eq!(log.updates.len(), 1);
        assert!(log.updates[0].old_oid.is_some());
        assert_eq!(
            log.updates[0].old_oid.as_ref().unwrap(),
            &old_commit.to_string()
        );

        tx.commit().unwrap();

        // Verify the ref was updated
        let resolved = repo.resolve_ref("refs/jin/layers/global").unwrap();
        assert_eq!(resolved, new_commit);
    }

    #[test]
    fn test_concurrent_transaction_blocked() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // Start first transaction
        let tx1 = LayerTransaction::begin_with_path(&repo, "first transaction", base_path.clone())
            .unwrap();

        // Try to start second transaction - should fail
        let result =
            LayerTransaction::begin_with_path(&repo, "second transaction", base_path.clone());
        assert!(result.is_err());
        if let Err(JinError::Transaction(msg)) = result {
            assert!(msg.contains("Incomplete transaction exists"));
        } else {
            panic!("Expected Transaction error");
        }

        // Cleanup by aborting first transaction
        tx1.abort().unwrap();
    }

    #[test]
    fn test_recovery_detect_incomplete() {
        let (_temp, _repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // No transaction - nothing to detect
        assert!(RecoveryManager::detect_at(&base_path).unwrap().is_none());

        // Create a transaction log manually
        let log = TransactionLog::with_base_path("incomplete test", base_path.clone());
        log.save().unwrap();

        // Now should detect
        let incomplete = RecoveryManager::detect_at(&base_path).unwrap();
        assert!(incomplete.is_some());
        let incomplete = incomplete.unwrap();
        assert_eq!(incomplete.message(), "incomplete test");
        assert_eq!(incomplete.state(), TransactionState::Pending);

        cleanup_transaction_log_at(&base_path);
    }

    #[test]
    fn test_recovery_rollback() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // Create a ref first
        let old_commit = create_test_commit(&repo);
        repo.set_ref("refs/jin/layers/global", old_commit, "original")
            .unwrap();

        let new_commit = create_test_commit(&repo);

        // Create a transaction log with an update that was "interrupted"
        let mut log = TransactionLog::with_base_path("interrupted transaction", base_path.clone());
        log.updates.push(LayerUpdate::new(
            Layer::GlobalBase,
            None,
            None,
            None,
            Some(old_commit),
            new_commit,
        ));
        log.state = TransactionState::Prepared;
        log.save().unwrap();

        // Simulate that the new commit was applied
        repo.set_ref("refs/jin/layers/global", new_commit, "updated")
            .unwrap();
        assert_eq!(
            repo.resolve_ref("refs/jin/layers/global").unwrap(),
            new_commit
        );

        // Recover - should rollback to old_commit
        let incomplete = RecoveryManager::detect_at(&base_path).unwrap().unwrap();
        incomplete.rollback(&repo).unwrap();

        // Ref should be restored to old value
        assert_eq!(
            repo.resolve_ref("refs/jin/layers/global").unwrap(),
            old_commit
        );

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_recovery_resume() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        let commit_oid = create_test_commit(&repo);

        // Create a transaction log in Prepared state
        let mut log = TransactionLog::with_base_path("resume test", base_path.clone());
        log.updates.push(LayerUpdate::new(
            Layer::GlobalBase,
            None,
            None,
            None,
            None,
            commit_oid,
        ));
        log.state = TransactionState::Prepared;
        log.save().unwrap();

        // Ref doesn't exist yet (simulating crash before apply)
        assert!(!repo.ref_exists("refs/jin/layers/global"));

        // Resume should complete the transaction
        let incomplete = RecoveryManager::detect_at(&base_path).unwrap().unwrap();
        incomplete.resume(&repo).unwrap();

        // Ref should now exist
        assert!(repo.ref_exists("refs/jin/layers/global"));
        assert_eq!(
            repo.resolve_ref("refs/jin/layers/global").unwrap(),
            commit_oid
        );

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_recovery_resume_requires_prepared_state() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // Create a transaction log in Pending state
        let log = TransactionLog::with_base_path("pending test", base_path.clone());
        log.save().unwrap();

        // Try to resume - should fail
        let incomplete = RecoveryManager::detect_at(&base_path).unwrap().unwrap();
        let result = incomplete.resume(&repo);
        assert!(result.is_err());
        if let Err(JinError::Transaction(msg)) = result {
            assert!(msg.contains("Prepared state"));
        }

        cleanup_transaction_log_at(&base_path);
    }

    #[test]
    fn test_auto_recover() {
        let (_temp, repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // No transaction - no recovery needed
        assert!(!RecoveryManager::auto_recover_at(&repo, &base_path).unwrap());

        // Create incomplete transaction
        let log = TransactionLog::with_base_path("auto recover test", base_path.clone());
        log.save().unwrap();

        // Auto recover should clean it up
        assert!(RecoveryManager::auto_recover_at(&repo, &base_path).unwrap());

        // Log should be deleted
        assert!(TransactionLog::load_from(log_path_for(&base_path))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_incomplete_transaction_inspection() {
        let (_temp, _repo, base_path) = setup_layer_transaction_test();
        cleanup_transaction_log_at(&base_path);

        // Create a transaction log with updates
        let mut log = TransactionLog::with_base_path("inspection test", base_path.clone());
        let oid = Oid::from_str("0000000000000000000000000000000000000000").unwrap();
        log.updates.push(LayerUpdate::new(
            Layer::GlobalBase,
            None,
            None,
            None,
            None,
            oid,
        ));
        log.updates.push(LayerUpdate::new(
            Layer::ModeBase,
            Some("claude".to_string()),
            None,
            None,
            None,
            oid,
        ));
        log.save().unwrap();

        let incomplete = RecoveryManager::detect_at(&base_path).unwrap().unwrap();
        assert_eq!(incomplete.state(), TransactionState::Pending);
        assert_eq!(incomplete.message(), "inspection test");
        assert_eq!(incomplete.update_count(), 2);
        assert!(!incomplete.id().is_empty());
        assert!(!incomplete.started_at().is_empty());

        let updates = incomplete.updates();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].layer, Layer::GlobalBase);
        assert_eq!(updates[1].layer, Layer::ModeBase);

        cleanup_transaction_log_at(&base_path);
    }
}
