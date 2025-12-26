//! Atomic transaction system for multi-layer Git commits.
//!
//! This module provides transaction semantics for updating multiple Jin layer
//! references atomically using git2's transaction API. All layer reference updates
//! succeed or fail together, with automatic recovery from incomplete transactions.
//!
//! # Transaction Lifecycle
//!
//! 1. **Begin**: Create staging ref, initialize transaction state
//! 2. **Add Updates**: Collect layer changes to apply
//! 3. **Prepare**: Lock all layer refs, validate current OIDs
//! 4. **Commit**: Atomically update all refs via git2 transaction
//! 5. **Rollback**: Release locks without applying changes
//! 6. **Cleanup**: Remove staging ref (automatic on commit/rollback/Drop)
//!
//! # Staging References
//!
//! Transactions use staging references as markers:
//! - Pattern: `refs/jin/staging/<transaction-id>`
//! - Created on `begin()` with `force=false` (conflict detection)
//! - Deleted on `commit()`, `rollback()`, or `Drop`
//! - Used by `detect_orphaned()` to find incomplete transactions
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::git::{JinRepo, TransactionManager};
//! use jin_glm::core::Layer;
//!
//! let repo = JinRepo::open_or_create(path)?;
//! let tm = TransactionManager::new(repo);
//!
//! // Begin transaction
//! let mut tx = tm.begin_transaction()?;
//!
//! // Add layer updates
//! tx.add_layer_update(Layer::GlobalBase, new_tree_id)?;
//! tx.add_layer_update(Layer::ModeBase { mode: "dev".into() }, new_tree_id)?;
//!
//! // Prepare (locks refs)
//! tx.prepare()?;
//!
//! // Commit (atomic update)
//! tx.commit()?;
//! ```
//!
//! # RAII Cleanup
//!
//! Uncommitted transactions are automatically rolled back when dropped:
//!
//! ```ignore
//! {
//!     let mut tx = tm.begin_transaction()?;
//!     tx.add_layer_update(layer, oid)?;
//!     tx.prepare()?;
//!     // Scope ends - tx.drop() automatically rolls back
//! }
//! // Staging ref cleaned up, locks released
//! ```

use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use git2::Oid;
use uuid::Uuid;

// ===== Transaction State =====

/// Transaction state tracking.
///
/// The transaction follows a strict state machine:
/// - `Started` → `Prepared` → `Committed`
/// - `Started` → `RolledBack` (early rollback)
/// - `Prepared` → `RolledBack` (rollback after prepare)
///
/// Invalid transitions return errors.
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    /// Transaction has been started but not prepared
    Started,
    /// Transaction has been prepared (refs locked, validated)
    Prepared,
    /// Transaction has been committed
    Committed,
    /// Transaction has been rolled back
    RolledBack,
}

// ===== Transaction =====

/// A transaction for atomic multi-layer Git commits.
///
/// Wraps git2's `Transaction` API with Jin-specific semantics:
/// - Staging reference tracking for recovery
/// - Layer-aware reference management
/// - Automatic cleanup via `Drop` trait
///
/// # Lifecycle
///
/// 1. **Begin**: Create staging ref, initialize transaction
/// 2. **Prepare**: Lock all layer refs, validate current state
/// 3. **Commit**: Atomically update all refs
/// 4. **Rollback**: Release locks, clean up staging ref
///
/// # Examples
///
/// ```ignore
/// use jin_glm::git::{JinRepo, Transaction};
/// use jin_glm::core::Layer;
///
/// let repo = JinRepo::open_or_create(path)?;
///
/// // Begin transaction
/// let mut tx = Transaction::begin(&repo)?;
///
/// // Add layer updates
/// tx.add_layer_update(Layer::GlobalBase, new_tree_id)?;
/// tx.add_layer_update(Layer::ProjectBase { project: "myapp".into() }, new_tree_id)?;
///
/// // Prepare (locks refs)
/// tx.prepare()?;
///
/// // Commit (atomic update)
/// tx.commit()?;
/// ```
pub struct Transaction<'a> {
    /// Unique identifier for this transaction
    pub(crate) id: String,
    /// Reference to the Jin repository
    repo: &'a JinRepo,
    /// The underlying git2 transaction (None after commit/rollback)
    tx: Option<git2::Transaction<'a>>,
    /// Current transaction state
    state: TransactionState,
    /// Layer updates to apply: (layer, new_oid, old_oid)
    updates: Vec<(Layer, Oid, Option<Oid>)>,
}

impl<'a> Transaction<'a> {
    /// Begins a new transaction.
    ///
    /// Creates a staging reference as a transaction marker and initializes
    /// the transaction state. The staging ref uses `force=false` to detect
    /// conflicting transactions with the same ID.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Jin repository to create a transaction in
    ///
    /// # Returns
    ///
    /// A new `Transaction` in `Started` state
    ///
    /// # Errors
    ///
    /// - `JinError::RefExists` if a staging ref with this ID already exists
    /// - `JinError::Git` for underlying git2 errors
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let repo = JinRepo::open_or_create(path)?;
    /// let tx = Transaction::begin(&repo)?;
    /// ```
    pub fn begin(repo: &'a JinRepo) -> Result<Self> {
        let id = Uuid::new_v4().to_string();

        // Create staging ref as transaction marker
        // Use an empty tree as initial point (will be updated on commit)
        let initial_oid = repo.inner.treebuilder(None)?.write()?;

        repo.create_staging_ref(&id, initial_oid)?;

        Ok(Self {
            id,
            repo,
            tx: None, // Will create on prepare
            state: TransactionState::Started,
            updates: Vec::new(),
        })
    }

    /// Returns the transaction ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the current transaction state.
    pub fn state(&self) -> &TransactionState {
        &self.state
    }

    /// Adds a layer update to the transaction.
    ///
    /// Only versioned layers (1-7) can be updated. UserLocal and WorkspaceActive
    /// layers are not versioned and will return an error.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to update
    /// * `new_oid` - The new OID to point the layer reference to
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update was added to the transaction
    ///
    /// # Errors
    ///
    /// - `JinError::InvalidLayer` if the layer is not versioned
    ///
    /// # Examples
    ///
    /// ```ignore
    /// tx.add_layer_update(Layer::GlobalBase, new_tree_id)?;
    /// tx.add_layer_update(Layer::ModeBase { mode: "dev".into() }, new_tree_id)?;
    /// ```
    pub fn add_layer_update(&mut self, layer: Layer, new_oid: Oid) -> Result<()> {
        // Only versioned layers can be updated
        if !layer.is_versioned() {
            return Err(JinError::InvalidLayer {
                name: format!("{:?}", layer),
            });
        }

        // Get current OID for this layer
        let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
            name: format!("{:?}", layer),
        })?;

        let current_oid = self
            .repo
            .inner
            .find_reference(&ref_name)
            .ok()
            .and_then(|r| r.target());

        self.updates.push((layer, new_oid, current_oid));
        Ok(())
    }

    /// Prepares the transaction for commit.
    ///
    /// Locks all layer references and validates that the current OIDs match
    /// what was captured when adding updates. This ensures no concurrent
    /// modifications have occurred.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the transaction is prepared and locked
    ///
    /// # Errors
    ///
    /// - `JinError::Message` if transaction is already prepared or no updates
    /// - `JinError::PrepareFailed` if locking fails
    /// - `JinError::TransactionConflict` if a layer has been modified
    ///
    /// # Examples
    ///
    /// ```ignore
    /// tx.prepare()?;
    /// ```
    pub fn prepare(&mut self) -> Result<()> {
        if self.state != TransactionState::Started {
            return Err(JinError::Message(
                "Transaction already prepared or committed".to_string(),
            ));
        }

        if self.updates.is_empty() {
            return Err(JinError::Message("No layer updates to commit".to_string()));
        }

        // Create git2 transaction
        let mut tx = self.repo.inner.transaction()?;

        // Collect all ref names to lock
        let ref_names: Vec<String> = self
            .updates
            .iter()
            .map(|(layer, _, _)| {
                layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
                    name: format!("{:?}", layer),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let ref_name_strs: Vec<&str> = ref_names.iter().map(|s| s.as_str()).collect();

        // Lock all references (use lock_ref for each ref individually)
        for ref_name in &ref_name_strs {
            tx.lock_ref(ref_name).map_err(|e| JinError::PrepareFailed {
                source: Box::new(e.into()),
                files: ref_names.clone(),
            })?;
        }

        // Validate current OIDs match what we expect
        for (layer, _, expected_old) in &self.updates {
            if let &Some(expected) = expected_old {
                let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
                    name: format!("{:?}", layer),
                })?;

                match self.repo.inner.find_reference(&ref_name) {
                    Ok(current_ref) => {
                        let current = current_ref.target();
                        if current != Some(expected) {
                            return Err(JinError::TransactionConflict {
                                conflict: format!(
                                    "Layer {:?} has been modified since transaction started",
                                    layer
                                ),
                            });
                        }
                    }
                    Err(e) if e.code() == git2::ErrorCode::NotFound => {
                        // Ref doesn't exist, that's OK if we expected None
                        if expected_old.is_some() {
                            return Err(JinError::TransactionConflict {
                                conflict: format!("Layer {:?} was deleted", layer),
                            });
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }

        self.tx = Some(tx);
        self.state = TransactionState::Prepared;
        Ok(())
    }

    /// Commits the transaction.
    ///
    /// Applies all layer updates atomically using the git2 transaction API.
    /// All updates are applied or none are applied - this is the key property
    /// for multi-layer atomicity.
    ///
    /// After successful commit, the staging ref is cleaned up.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all updates were applied atomically
    ///
    /// # Errors
    ///
    /// - `JinError::Message` if transaction is not prepared
    /// - `JinError::CommitFailed` if the commit operation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// tx.prepare()?;
    /// tx.commit()?;
    /// ```
    pub fn commit(mut self) -> Result<()> {
        if self.state != TransactionState::Prepared {
            return Err(JinError::Message(
                "Transaction must be prepared before commit".to_string(),
            ));
        }

        let mut tx = self
            .tx
            .take()
            .ok_or_else(|| JinError::Message("Transaction not prepared".to_string()))?;

        // Get signature for reflog
        let signature = self
            .repo
            .inner
            .signature()
            .unwrap_or_else(|_| git2::Signature::now("Jin", "jin@local").unwrap());

        // Apply all updates to the transaction
        for (layer, new_oid, _) in &self.updates {
            let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

            tx.set_target(
                &ref_name,
                *new_oid,
                Some(&signature),
                &format!("Jin transaction: {}", self.id),
            )
            .map_err(|e| JinError::CommitFailed {
                source: Box::new(e.into()),
                files: vec![ref_name.clone()],
            })?;
        }

        // Commit the transaction (atomic update)
        tx.commit().map_err(|e| JinError::CommitFailed {
            source: Box::new(e.into()),
            files: self
                .updates
                .iter()
                .map(|(l, _, _)| l.git_ref().unwrap_or_else(|| "unknown".to_string()))
                .collect(),
        })?;

        // Clean up staging ref
        let _ = self.repo.delete_staging_ref(&self.id);

        self.state = TransactionState::Committed;
        Ok(())
    }

    /// Rolls back the transaction.
    ///
    /// Releases all locks without applying any changes and cleans up the
    /// staging ref. This is the explicit rollback method - automatic cleanup
    /// also happens via the `Drop` trait.
    ///
    /// # Returns
    ///
    /// `Ok(())` if rollback was successful
    ///
    /// # Examples
    ///
    /// ```ignore
    /// tx.prepare()?;
    /// // ... decide not to commit ...
    /// tx.rollback()?;
    /// ```
    pub fn rollback(mut self) -> Result<()> {
        // Drop the transaction to release locks (git2 auto-rollback on drop)
        let _ = self.tx.take();

        // Clean up staging ref
        let _ = self.repo.delete_staging_ref(&self.id);

        self.state = TransactionState::RolledBack;
        Ok(())
    }
}

// ===== Drop Implementation (RAII) =====

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        // Auto-rollback if transaction wasn't committed or explicitly rolled back
        if self.state != TransactionState::Committed && self.state != TransactionState::RolledBack {
            // Clean up staging ref (ignore errors in Drop)
            let _ = self.repo.delete_staging_ref(&self.id);

            // Drop the transaction to release locks (git2 auto-rollback on drop)
            let _ = self.tx.take();
        }
    }
}

// ===== TransactionManager =====

/// Manager for creating and recovering transactions.
///
/// Provides factory methods for creating transactions and recovery utilities
/// for detecting orphaned transactions.
///
/// # Examples
///
/// ```ignore
/// let repo = JinRepo::open_or_create(path)?;
/// let tm = TransactionManager::new(repo);
///
/// // Create a transaction
/// let mut tx = tm.begin_transaction()?;
///
/// // Detect orphaned transactions
/// let orphaned = tm.detect_orphaned()?;
/// for tx_id in orphaned {
///     tm.recover(&tx_id)?;
/// }
/// ```
#[derive(Debug)]
pub struct TransactionManager<'a> {
    /// The Jin repository (borrowed)
    repo: &'a JinRepo,
}

impl<'a> TransactionManager<'a> {
    /// Creates a new transaction manager.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Jin repository to manage transactions for
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let repo = JinRepo::open_or_create(path)?;
    /// let tm = TransactionManager::new(&repo);
    /// ```
    pub fn new(repo: &'a JinRepo) -> Self {
        Self { repo }
    }

    /// Begins a new transaction.
    ///
    /// Factory method that creates a new transaction with a unique ID.
    ///
    /// # Returns
    ///
    /// A new `Transaction` in `Started` state
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut tx = tm.begin_transaction()?;
    /// ```
    pub fn begin_transaction(&self) -> Result<Transaction<'a>> {
        Transaction::begin(self.repo)
    }

    /// Detects orphaned/incomplete transactions.
    ///
    /// Scans for staging references to find transactions that were started
    /// but never committed or rolled back (e.g., due to crashes).
    ///
    /// # Returns
    ///
    /// A vector of transaction IDs for orphaned transactions
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let orphaned = tm.detect_orphaned()?;
    /// for tx_id in orphaned {
    ///     println!("Orphaned transaction: {}", tx_id);
    /// }
    /// ```
    pub fn detect_orphaned(&self) -> Result<Vec<String>> {
        let mut orphaned = Vec::new();

        // Look for staging refs
        for reference in self
            .repo
            .inner
            .references_glob("refs/jin/staging/*")?
            .flatten()
        {
            if let Some(name) = reference.name() {
                if name.starts_with("refs/jin/staging/") {
                    let transaction_id =
                        name.strip_prefix("refs/jin/staging/").unwrap().to_string();
                    orphaned.push(transaction_id);
                }
            }
        }

        Ok(orphaned)
    }

    /// Recovers a specific orphaned transaction.
    ///
    /// Cleans up the staging reference for an orphaned transaction.
    /// In the future, this could attempt to complete the transaction.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The ID of the transaction to recover
    ///
    /// # Returns
    ///
    /// `Ok(())` if recovery was successful
    ///
    /// # Examples
    ///
    /// ```ignore
    /// tm.recover("orphaned-tx-id")?;
    /// ```
    pub fn recover(&self, transaction_id: &str) -> Result<()> {
        // Simply delete the staging ref
        // In future, could attempt to complete transaction
        self.repo.delete_staging_ref(transaction_id)
    }

    /// Recovers all orphaned transactions.
    ///
    /// Detects and cleans up all orphaned transactions in the repository.
    ///
    /// # Returns
    ///
    /// The number of transactions recovered
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let count = tm.recover_all()?;
    /// println!("Recovered {} transactions", count);
    /// ```
    pub fn recover_all(&self) -> Result<usize> {
        let orphaned = self.detect_orphaned()?;
        let mut recovered = 0;

        for tx_id in orphaned {
            self.recover(&tx_id)?;
            recovered += 1;
        }

        Ok(recovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_state_transitions() {
        // Test state equality
        assert_eq!(TransactionState::Started, TransactionState::Started);
        assert_ne!(TransactionState::Started, TransactionState::Prepared);
    }
}
