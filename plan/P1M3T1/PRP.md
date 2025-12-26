# Product Requirement Prompt (PRP): Transaction Implementation (P1.M3.T1)

---

## Goal

**Feature Goal**: Implement the atomic transaction system for multi-layer Git commits that ensures all layer reference updates succeed or fail together, with automatic recovery from incomplete transactions.

**Deliverable**: A `src/git/transaction.rs` module with:
- `Transaction` struct managing multi-layer commit lifecycle
- `TransactionManager` for creating and tracking transactions
- Atomic multi-ref updates using git2's transaction API
- Staging reference management (`refs/jin/staging/<id>`)
- Recovery detection for orphaned/incomplete transactions
- RAII-style automatic cleanup via `Drop` trait
- Comprehensive unit tests using `tempfile`

**Success Definition**:
- `cargo build` compiles with zero errors
- All unit tests pass with isolated test repositories
- Multi-layer commits are atomic (all succeed or all fail)
- Orphaned transactions are detected and can be cleaned up
- Staging references are properly managed (created on begin, deleted on commit/cleanup)
- Integration with existing `JinRepo`, `JinError`, and `Layer` types

## User Persona

**Target User**: AI coding agent implementing Jin's transaction system foundation

**Use Case**: The agent needs to establish the transaction infrastructure that:
- Wraps git2's `Transaction` API for Jin-specific multi-layer operations
- Manages staging references as transaction markers
- Provides atomic commit/rollback semantics for layer ref updates
- Integrates with existing `JinRepo` wrapper and error types
- Enables recovery from incomplete transactions

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/git/transaction.rs` with `Transaction` and `TransactionManager` types
3. Implements transaction lifecycle (begin, prepare, commit, rollback)
4. Implements atomic multi-ref updates using git2 transaction API
5. Implements staging ref management for transaction tracking
6. Implements recovery detection for orphaned transactions
7. Adds comprehensive unit tests
8. Validates compilation and test success

**Pain Points Addressed**:
- No manual multi-ref coordination - git2 transaction API handles atomicity
- Consistent error handling with `JinError` integration
- Isolated test setup using `tempfile` patterns from existing codebase
- Clear separation between transaction preparation and commit phases
- Automatic cleanup via `Drop` prevents orphaned staging refs

## Why

- **Foundation for atomic commits**: Every multi-layer commit operation depends on transactions
- **Data consistency**: Prevents partial state where some layers are updated but others aren't
- **Recovery capability**: Enables cleanup from crashes/interruptions during commit
- **Integration point**: Bridges git2's transaction API with Jin's layer system
- **Problems this solves**:
  - Prevents inconsistent layer state from failed multi-ref updates
  - Provides recovery mechanism for incomplete transactions
  - Centralizes transaction logic for reuse by commit pipeline
  - Enables detection and cleanup of orphaned transactions

## What

Implement an atomic transaction system for multi-layer Git commits using git2's transaction API, with staging reference tracking and recovery detection.

### Success Criteria

- [ ] `src/git/transaction.rs` created with `Transaction` and `TransactionManager` types
- [ ] `Transaction::begin()` creates staging ref and initializes transaction state
- [ ] `Transaction::prepare()` locks all layer refs and validates current OIDs
- [ ] `Transaction::commit()` atomically updates all layer refs via git2 transaction
- [ ] `Transaction::rollback()` cleans up staging ref
- [ ] `Drop` implementation auto-rolls back uncommitted transactions
- [ ] `TransactionManager::detect_orphaned()` finds incomplete transactions
- [ ] `TransactionManager::recover()` cleans up orphaned transactions
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover all public methods with temp repos
- [ ] Module exported from `src/git/mod.rs`
- [ ] All tests pass: `cargo test --package jin --lib git::transaction`

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact `Transaction` and `TransactionManager` struct specifications with all methods
- Research documents with 100+ code examples for git2 transaction API
- Specific patterns from existing codebase to follow (JinRepo, JinError, Layer)
- Complete integration guide with existing types
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Transaction pattern specification and Git ref namespace
  section: Lines 147-156 for Atomic Transaction Pattern
  critical: "1. Begin transaction (create staging ref), 2. Build trees, 3. Create commits, 4. Prepare: lock refs, 5. Commit: atomically update refs, 6. Cleanup: remove staging ref"

- file: /home/dustin/projects/jin-glm-doover/plan/docs/implementation_status.md
  why: Current implementation status - P1.M2 complete, P1.M3 target
  section: Lines 1-21 for Foundation status
  critical: "P1.M3 Transaction System - Complete" (target status after implementation)

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Transaction error types already defined
  pattern: JinError::TransactionConflict, JinError::PrepareFailed, JinError::CommitFailed
  gotcha: Use PrepareFailed and CommitFailed with Box<JinError> for error chaining

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum's git_ref() method provides exact ref format for transactions
  section: Lines 215-279 for git_ref() implementation
  critical: ALWAYS call layer.git_ref() to get ref names for transaction locks

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: JinRepo wrapper with staging ref methods already exists
  section: Lines 438-543 for staging ref operations (create_staging_ref, delete_staging_ref, staging_ref_exists)
  critical: Use these methods for transaction marker management

- file: /home/dustin/projects/jin-glm-doover/src/git/mod.rs
  why: Module must export transaction module after creation
  pattern: Follow existing pattern: pub mod repo; pub use repo::JinRepo;
  gotcha: Need to add pub mod transaction; and pub use transaction::{Transaction, TransactionManager};

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify git2 dependency with required features
  section: Line 19 for git2 dependency
  critical: git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] }

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M3T1/research/git2_transaction_patterns.md
  why: Complete git2-rs transaction API documentation with 100+ code examples
  section: Core Transaction API (lines 14-75), Atomic Patterns (lines 76-209), Error Handling (lines 301-436)
  critical: Shows exact API usage: repo.transaction()?, tx.lock(), tx.set_target(), tx.commit(), tx.rollback()

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M3T1/research/rust_transaction_patterns.md
  why: Rust transaction design patterns (RAII, state machine, command pattern)
  section: RAII Pattern with Drop (lines 56-99), State Machine Pattern (lines 372-480), Command Pattern (lines 260-328)
  critical: RAII Drop implementation ensures cleanup on scope exit

# EXTERNAL - git2-rs Documentation

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.transaction
  why: Repository::transaction() creates reference transaction
  critical: Returns Result<Transaction<'_>, Error>

- url: https://docs.rs/git2/0.20/git2/struct.Transaction.html
  why: Complete Transaction API - all methods needed
  critical: lock(), set_target(), delete(), commit(), rollback()

- url: https://docs.rs/git2/0.20/git2/struct.Transaction.html#method.lock
  why: Lock references before modification
  critical: First parameter is &[&str] slice of ref names, second is force flag

- url: https://docs.rs/git2/0.20/git2/struct.Transaction.html#method.set_target
  why: Set a reference to point to a new OID
  critical: Parameters: refname: &str, id: Oid, msg: &str

- url: https://docs.rs/git2/0.20/git2/struct.Transaction.html#method.commit
  why: Commit transaction - applies all updates atomically
  critical: Must be called for changes to take effect

- url: https://docs.rs/git2/0.20/git2/struct.Transaction.html#method.rollback
  why: Rollback transaction - releases locks without applying changes
  critical: Call on error or intentional rollback
```

### Current Codebase Tree

```bash
# Run this command to verify current state
tree -L 3 -I 'target|Cargo.lock' /home/dustin/projects/jin-glm-doover

# Expected output:
# /home/dustin/projects/jin-glm-doover
# ├── Cargo.toml                      # Has git2 dependency with features
# ├── src/
# │   ├── main.rs
# │   ├── lib.rs
# │   ├── cli/mod.rs
# │   ├── commands/mod.rs
# │   ├── commit/mod.rs               # Commit pipeline (future integration point)
# │   ├── core/
# │   │   ├── mod.rs                 # Exports error, layer, config
# │   │   ├── error.rs               # Has JinError::TransactionConflict, PrepareFailed, CommitFailed
# │   │   ├── layer.rs               # Has Layer enum with git_ref() method
# │   │   └── config.rs
# │   └── git/
# │       ├── mod.rs                 # Currently exports repo, needs to add transaction
# │       └── repo.rs                # Has JinRepo with staging ref methods
# └── tests/
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── git/
│       ├── mod.rs                    # MODIFY: Add pub mod transaction; pub use transaction::*;
│       └── transaction.rs            # CREATE: Transaction and TransactionManager implementation
└── tests/
    └── git/
        └── transaction_test.rs       # CREATE: Unit tests for transaction system
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: git2 Transaction lifetimes
// Transaction has lifetime 'repo tied to Repository
// Cannot outlive the Repository it was created from
// Pattern: create and use Transaction within same scope

// CRITICAL: Lock before modify
// MUST call tx.lock() BEFORE any tx.set_target() calls
// Locking establishes which refs are part of the transaction
// Good:
//   let mut tx = repo.transaction()?;
//   tx.lock(&["refs/jin/layers/global"], false)?;
//   tx.set_target("refs/jin/layers/global", new_oid, "update")?;
// Bad:
//   let mut tx = repo.transaction()?;
//   tx.set_target("refs/jin/layers/global", new_oid, "update")?;
//   tx.lock(&["refs/jin/layers/global"], false)?;  // Too late!

// CRITICAL: Always commit or rollback
// Transaction MUST be either committed or rolled back
// Uncommitted transactions hold locks and can block other operations
// Use Drop trait to auto-rollback on scope exit

// CRITICAL: Staging ref creation uses force=false
// JinRepo::create_staging_ref() uses force=false to detect conflicts
// If staging ref exists, transaction is already in progress
// This is intentional - prevents concurrent transactions with same ID

// CRITICAL: Layer refs use Layer.git_ref() format
// ALWAYS use layer.git_ref() to get ref names for transaction locks
// NEVER hardcode "refs/jin/layers/" strings
// Good:
//   let ref_name = layer.git_ref()
//       .ok_or_else(|| JinError::InvalidLayer { name: format!("{:?}", layer) })?;
// Bad:
//   let ref_name = format!("refs/jin/layers/global");

// CRITICAL: JinRepo staging ref methods
// Use JinRepo methods for staging ref management:
//   create_staging_ref(transaction_id, oid) - creates marker
//   delete_staging_ref(transaction_id) - removes marker
//   staging_ref_exists(transaction_id) - checks if active
// These handle the "refs/jin/staging/" prefix automatically

// CRITICAL: Error conversion pattern
// Use ? operator for automatic git2::Error conversion via JinError::Git
// Use explicit conversion for context-specific errors:
//   JinError::PrepareFailed { source: Box::new(e.into()), files: affected_files }
//   JinError::CommitFailed { source: Box::new(e.into()), files: affected_files }

// GOTCHA: Transaction commits are atomic
// All set_target() calls in a transaction are applied atomically
// If commit() fails, NO changes are applied to any refs
// This is the key property for multi-layer atomicity

// GOTCHA: Rollback vs Drop
// tx.rollback() explicitly releases locks without applying changes
// Drop trait calling rollback is cleanup, not explicit rollback
// Both have same effect but different semantic meaning

// PATTERN: Follow existing thiserror patterns from error.rs
// JinError variants use thiserror derive macro
// Use #[from] for automatic git2::Error conversion
// Use Box<JinError> for error chaining in PrepareFailed/CommitFailed

// PATTERN: Follow JinRepo wrapper pattern
// Wrap git2 types while providing Jin-specific operations
// Use pub(crate) for internal access if needed
// Return Result<T> using JinError consistently
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
/// Transaction state tracking
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

/// A transaction for atomic multi-layer Git commits.
///
/// Wraps git2's Transaction API with Jin-specific semantics:
/// - Staging reference tracking for recovery
/// - Layer-aware reference management
/// - Automatic cleanup via Drop trait
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
/// use jin_glm::git::{JinRepo, TransactionManager};
/// use jin_glm::core::Layer;
///
/// let repo = JinRepo::open_or_create(path)?;
/// let tm = TransactionManager::new(repo);
///
/// // Begin transaction
/// let mut tx = tm.begin_transaction()?;
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
    id: String,
    /// Reference to the Jin repository
    repo: &'a JinRepo,
    /// The underlying git2 transaction
    tx: Option<git2::Transaction<'a>>,
    /// Current transaction state
    state: TransactionState,
    /// Layer updates to apply: (layer, new_oid, old_oid)
    updates: Vec<(Layer, git2::Oid, Option<git2::Oid>)>,
}

/// Manager for creating and recovering transactions.
///
/// Provides factory methods for creating transactions and
/// recovery utilities for detecting orphaned transactions.
pub struct TransactionManager {
    /// The Jin repository (owned for manager)
    repo: JinRepo,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/git/transaction.rs
  - IMPLEMENT: Module-level documentation and imports
  - PATTERN: Follow src/git/repo.rs module structure
  - IMPORTS:
    * use crate::core::error::{JinError, Result}
    * use crate::core::Layer
    * use crate::git::JinRepo
    * use git2::{Oid, Transaction}
    * use uuid::Uuid
    * use std::collections::HashMap
  - PLACEMENT: New file src/git/transaction.rs
  - NAMING: Transaction, TransactionManager, TransactionState

Task 2: IMPLEMENT TransactionState enum
  - IMPLEMENT: pub enum TransactionState with Started, Prepared, Committed, RolledBack variants
  - DERIVE: Debug, Clone, PartialEq
  - PATTERN: Follow Rust state machine pattern (see research/rust_transaction_patterns.md lines 372-432)
  - CODE TEMPLATE:
    #[derive(Debug, Clone, PartialEq)]
    pub enum TransactionState {
        Started,
        Prepared,
        Committed,
        RolledBack,
    }
  - PLACEMENT: At top of transaction.rs file
  - DEPENDENCIES: Task 1

Task 3: IMPLEMENT Transaction struct definition
  - IMPLEMENT: pub struct Transaction<'a> with id, repo, tx, state, updates fields
  - PATTERN: Use Option<git2::Transaction<'a>> for tx (None after commit/rollback)
  - CODE TEMPLATE:
    pub struct Transaction<'a> {
        pub(crate) id: String,
        repo: &'a JinRepo,
        tx: Option<git2::Transaction<'a>>,
        state: TransactionState,
        updates: Vec<(Layer, git2::Oid, Option<git2::Oid>)>,
    }
  - LIFETIME: Transaction borrows repo for its lifetime
  - PLACEMENT: After TransactionState enum
  - DEPENDENCIES: Task 2

Task 4: IMPLEMENT Transaction::begin() method
  - IMPLEMENT: pub fn begin(repo: &JinRepo) -> Result<Self>
  - PATTERN: Create staging ref, initialize empty transaction
  - CODE TEMPLATE:
    impl<'a> Transaction<'a> {
        pub fn begin(repo: &'a JinRepo) -> Result<Self> {
            let id = Uuid::new_v4().to_string();

            // Create staging ref as transaction marker
            let initial_oid = repo.find_commit(
                // Use HEAD or empty tree as initial point
            )?;

            repo.create_staging_ref(&id, initial_oid.id())?;

            Ok(Self {
                id,
                repo,
                tx: None,  // Will create on prepare
                state: TransactionState::Started,
                updates: Vec::new(),
            })
        }
    }
  - CRITICAL: create_staging_ref uses force=false (detects conflicts)
  - PLACEMENT: impl<'a> Transaction<'a> block
  - DEPENDENCIES: Task 3, JinRepo::create_staging_ref

Task 5: IMPLEMENT Transaction::add_layer_update()
  - IMPLEMENT: pub fn add_layer_update(&mut self, layer: Layer, new_oid: Oid) -> Result<()>
  - PATTERN: Get current OID, add to updates vector
  - CODE TEMPLATE:
    pub fn add_layer_update(&mut self, layer: Layer, new_oid: Oid) -> Result<()> {
        // Only versioned layers can be updated
        if !layer.is_versioned() {
            return Err(JinError::InvalidLayer {
                name: format!("{:?}", layer),
            });
        }

        // Get current OID for this layer
        let current_oid = self.repo.get_layer_ref(&layer)?
            .and_then(|r| r.target());

        self.updates.push((layer, new_oid, current_oid));
        Ok(())
    }
  - VALIDATION: Check layer.is_versioned() returns true
  - PLACEMENT: impl<'a> Transaction<'a> block
  - DEPENDENCIES: Task 4

Task 6: IMPLEMENT Transaction::prepare()
  - IMPLEMENT: pub fn prepare(&mut self) -> Result<()>
  - PATTERN: Lock all refs, validate state, transition to Prepared
  - CODE TEMPLATE:
    pub fn prepare(&mut self) -> Result<()> {
        if self.state != TransactionState::Started {
            return Err(JinError::Message(
                "Transaction already prepared or committed".to_string()
            ));
        }

        if self.updates.is_empty() {
            return Err(JinError::Message(
                "No layer updates to commit".to_string()
            ));
        }

        // Create git2 transaction
        let mut tx = self.repo.inner.transaction()?;

        // Collect all ref names to lock
        let ref_names: Vec<String> = self.updates.iter()
            .map(|(layer, _, _)| {
                layer.git_ref()
                    .ok_or_else(|| JinError::InvalidLayer {
                        name: format!("{:?}", layer),
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        let ref_name_strs: Vec<&str> = ref_names.iter().map(|s| s.as_str()).collect();

        // Lock all references
        tx.lock(&ref_name_strs, false)
            .map_err(|e| JinError::PrepareFailed {
                source: Box::new(e.into()),
                files: ref_names.clone(),
            })?;

        // Validate current OIDs match what we expect
        for (layer, _, expected_old) in &self.updates {
            if let &Some(expected) = expected_old {
                let ref_name = layer.git_ref()
                    .ok_or_else(|| JinError::InvalidLayer {
                        name: format!("{:?}", layer),
                    })?;

                match self.repo.inner.find_reference(&ref_name) {
                    Ok(current_ref) => {
                        let current = current_ref.target();
                        if current != *expected {
                            return Err(JinError::TransactionConflict {
                                conflict: format!(
                                    "Layer {:?} has been modified since transaction started",
                                    layer
                                ),
                            });
                        }
                    },
                    Err(e) if e.code() == git2::ErrorCode::NotFound => {
                        // Ref doesn't exist, that's OK if we expected None
                        if expected_old.is_some() {
                            return Err(JinError::TransactionConflict {
                                conflict: format!("Layer {:?} was deleted", layer),
                            });
                        }
                    },
                    Err(e) => return Err(e.into()),
                }
            }
        }

        self.tx = Some(tx);
        self.state = TransactionState::Prepared;
        Ok(())
    }
  - CRITICAL: Lock ALL refs before any modifications
  - ERROR HANDLING: Use PrepareFailed for any failures
  - PLACEMENT: impl<'a> Transaction<'a> block
  - DEPENDENCIES: Task 5

Task 7: IMPLEMENT Transaction::commit()
  - IMPLEMENT: pub fn commit(mut self) -> Result<()>
  - PATTERN: Apply all updates, commit transaction, clean up staging ref
  - CODE TEMPLATE:
    pub fn commit(mut self) -> Result<()> {
        if self.state != TransactionState::Prepared {
            return Err(JinError::Message(
                "Transaction must be prepared before commit".to_string()
            ));
        }

        let mut tx = self.tx.take()
            .ok_or_else(|| JinError::Message(
                "Transaction not prepared".to_string()
            ))?;

        // Apply all updates to the transaction
        for (layer, new_oid, _) in &self.updates {
            let ref_name = layer.git_ref()
                .ok_or_else(|| JinError::InvalidLayer {
                    name: format!("{:?}", layer),
                })?;

            tx.set_target(&ref_name, *new_oid, &format!("Jin transaction: {}", self.id))
                .map_err(|e| JinError::CommitFailed {
                    source: Box::new(e.into()),
                    files: vec![ref_name.clone()],
                })?;
        }

        // Commit the transaction (atomic update)
        tx.commit()
            .map_err(|e| JinError::CommitFailed {
                source: Box::new(e.into()),
                files: self.updates.iter()
                    .map(|(l, _, _)| l.git_ref().unwrap_or("unknown".to_string()))
                    .collect(),
            })?;

        // Clean up staging ref
        self.repo.delete_staging_ref(&self.id)?;

        self.state = TransactionState::Committed;
        Ok(())
    }
  - CRITICAL: tx.commit() is the atomic operation
  - CLEANUP: Always delete staging ref on success
  - PLACEMENT: impl<'a> Transaction<'a> block
  - DEPENDENCIES: Task 6

Task 8: IMPLEMENT Transaction::rollback()
  - IMPLEMENT: pub fn rollback(mut self) -> Result<()>
  - PATTERN: Drop transaction, clean up staging ref
  - CODE TEMPLATE:
    pub fn rollback(mut self) -> Result<()> {
        if let Some(mut tx) = self.tx.take() {
            // Explicit rollback releases locks
            tx.rollback()?;
        }

        // Clean up staging ref
        self.repo.delete_staging_ref(&self.id)?;

        self.state = TransactionState::RolledBack;
        Ok(())
    }
  - PLACEMENT: impl<'a> Transaction<'a> block
  - DEPENDENCIES: Task 4

Task 9: IMPLEMENT Transaction Drop trait
  - IMPLEMENT: impl Drop for Transaction<'_> with auto-rollback
  - PATTERN: RAII cleanup - rollback if not committed
  - CODE TEMPLATE:
    impl Drop for Transaction<'_> {
        fn drop(&mut self) {
            // Auto-rollback if transaction wasn't committed
            if self.state != TransactionState::Committed &&
               self.state != TransactionState::RolledBack {

                // Clean up staging ref
                let _ = self.repo.delete_staging_ref(&self.id);

                // Rollback transaction if exists
                if let Some(mut tx) = self.tx.take() {
                    let _ = tx.rollback();
                }
            }
        }
    }
  - CRITICAL: This is the RAII pattern - ensures cleanup
  - PLACEMENT: After Transaction impl block
  - DEPENDENCIES: Task 8

Task 10: IMPLEMENT TransactionManager struct
  - IMPLEMENT: pub struct TransactionManager with repo field
  - CODE TEMPLATE:
    pub struct TransactionManager {
        repo: JinRepo,
    }

    impl TransactionManager {
        pub fn new(repo: JinRepo) -> Self {
            Self { repo }
        }
    }
  - PLACEMENT: After Transaction Drop impl
  - DEPENDENCIES: Task 1

Task 11: IMPLEMENT TransactionManager::begin_transaction()
  - IMPLEMENT: pub fn begin_transaction(&self) -> Result<Transaction>
  - PATTERN: Factory method calling Transaction::begin()
  - CODE TEMPLATE:
    pub fn begin_transaction(&self) -> Result<Transaction> {
        Transaction::begin(&self.repo)
    }
  - PLACEMENT: impl TransactionManager block
  - DEPENDENCIES: Task 10, Task 4

Task 12: IMPLEMENT TransactionManager::detect_orphaned()
  - IMPLEMENT: pub fn detect_orphaned(&self) -> Result<Vec<String>>
  - PATTERN: Scan for staging refs, return list of transaction IDs
  - CODE TEMPLATE:
    pub fn detect_orphaned(&self) -> Result<Vec<String>> {
        let mut orphaned = Vec::new();

        // Look for staging refs
        for reference in self.repo.inner.references_glob("refs/jin/staging/*")? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                if name.starts_with("refs/jin/staging/") {
                    let transaction_id = name.strip_prefix("refs/jin/staging/")
                        .unwrap()
                        .to_string();
                    orphaned.push(transaction_id);
                }
            }
        }

        Ok(orphaned)
    }
  - PLACEMENT: impl TransactionManager block
  - DEPENDENCIES: Task 11

Task 13: IMPLEMENT TransactionManager::recover()
  - IMPLEMENT: pub fn recover(&self, transaction_id: &str) -> Result<()>
  - PATTERN: Clean up staging ref for orphaned transaction
  - CODE TEMPLATE:
    pub fn recover(&self, transaction_id: &str) -> Result<()> {
        // Simply delete the staging ref
        // In future, could attempt to complete transaction
        self.repo.delete_staging_ref(transaction_id)
    }
  - PLACEMENT: impl TransactionManager block
  - DEPENDENCIES: Task 12

Task 14: IMPLEMENT TransactionManager::recover_all()
  - IMPLEMENT: pub fn recover_all(&self) -> Result<usize>
  - PATTERN: Detect and recover all orphaned transactions
  - CODE TEMPLATE:
    pub fn recover_all(&self) -> Result<usize> {
        let orphaned = self.detect_orphaned()?;
        let mut recovered = 0;

        for tx_id in orphaned {
            self.recover(&tx_id)?;
            recovered += 1;
        }

        Ok(recovered)
    }
  - PLACEMENT: impl TransactionManager block
  - DEPENDENCIES: Task 13

Task 15: MODIFY src/git/mod.rs
  - ADD: pub mod transaction;
  - ADD: pub use transaction::{Transaction, TransactionManager, TransactionState};
  - PRESERVE: Existing module exports
  - FINAL FILE:
    pub mod repo;
    pub mod transaction;
    pub use repo::JinRepo;
    pub use transaction::{Transaction, TransactionManager, TransactionState};
  - PLACEMENT: src/git/mod.rs
  - DEPENDENCIES: Task 1 (transaction.rs must exist)

Task 16: CREATE tests/git/transaction_test.rs
  - IMPLEMENT: Unit tests for all transaction methods
  - FIXTURE: TestRepoFixture pattern from repo.rs tests (lines 968-1007)
  - TESTS:
    * test_transaction_begin_creates_staging_ref()
    * test_transaction_begin_fails_if_staging_ref_exists()
    * test_transaction_add_layer_update()
    * test_transaction_add_layer_update_fails_for_unversioned_layer()
    * test_transaction_prepare_locks_refs()
    * test_transaction_prepare_validates_current_oids()
    * test_transaction_prepare_fails_on_conflict()
    * test_transaction_commit_atomic_update()
    * test_transaction_commit_deletes_staging_ref()
    * test_transaction_rollback_releases_locks()
    * test_transaction_rollback_deletes_staging_ref()
    * test_transaction_drop_auto_rollback()
    * test_transaction_manager_detect_orphaned()
    * test_transaction_manager_recover()
    * test_transaction_manager_recover_all()
    * test_multi_layer_atomic_commit()
    * test_transaction_cannot_commit_twice()
  - FOLLOW: Pattern from repo.rs tests
  - USE: tempfile for temp repo directories
  - ASSERTIONS: Use Result<()> return for tests
  - PLACEMENT: tests/git/transaction_test.rs (create tests/git/ directory first)
  - DEPENDENCIES: Tasks 1-14
```

### Implementation Patterns & Key Details

```rust
// ===== STATE MACHINE PATTERN =====
// Transaction follows strict state transitions
// Started -> Prepared -> Committed
//                -> RolledBack
// Started -> RolledBack (early rollback)
//
// Invalid transitions return errors:
// - Cannot commit from Started state (must prepare first)
// - Cannot prepare from Prepared state (already prepared)
// - Cannot commit from Committed/RolledBack state

// ===== RAII DROP PATTERN =====
// Drop trait ensures cleanup on scope exit
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        // Only cleanup if not explicitly committed/rolled back
        if !matches!(self.state, TransactionState::Committed | TransactionState::RolledBack) {
            // Clean up staging ref (ignore errors in Drop)
            let _ = self.repo.delete_staging_ref(&self.id);

            // Rollback transaction if exists
            if let Some(mut tx) = self.tx.take() {
                let _ = tx.rollback();
            }
        }
    }
}

// ===== TRANSACTION LIFECYTE PATTERN =====
// 1. Begin: Create staging ref, initialize state
// 2. Add updates: Collect layer changes
// 3. Prepare: Lock refs, validate current state
// 4. Commit or Rollback: Apply changes or abort
// 5. Cleanup: Always remove staging ref

// ===== GIT2 TRANSACTION API PATTERN =====
// Create transaction from repository
let mut tx = repo.transaction()?;

// Lock ALL references before any modifications
let ref_names: Vec<&str> = updates.iter().map(|(name, _)| name.as_str()).collect();
tx.lock(&ref_names, false)?;

// Apply updates
for (refname, new_oid) in updates {
    tx.set_target(refname, new_oid, "message")?;
}

// Commit (atomic operation)
tx.commit()?;

// ===== ERROR HANDLING PATTERN =====
// Use PrepareFailed for preparation phase errors
.map_err(|e| JinError::PrepareFailed {
    source: Box::new(e.into()),
    files: ref_names.clone(),
})?;

// Use CommitFailed for commit phase errors
.map_err(|e| JinError::CommitFailed {
    source: Box::new(e.into()),
    files: affected_files,
})?;

// Use TransactionConflict for validation conflicts
Err(JinError::TransactionConflict {
    conflict: "Layer has been modified".to_string(),
})

// ===== STAGING REF PATTERN =====
// Staging refs follow pattern: refs/jin/staging/<transaction-id>
// Created on begin() with force=false (conflict detection)
// Deleted on commit(), rollback(), or Drop
// Used by detect_orphaned() to find incomplete transactions

// ===== LAYER INTEGRATION PATTERN =====
// Use Layer.git_ref() to get ref names
let ref_name = layer.git_ref()
    .ok_or_else(|| JinError::InvalidLayer {
        name: format!("{:?}", layer),
    })?;

// Validate layer is versioned before adding to transaction
if !layer.is_versioned() {
    return Err(JinError::InvalidLayer { ... });
}
```

### Integration Points

```yaml
JINREPO:
  - use: src/git/repo.rs
  - methods:
    * create_staging_ref(transaction_id, oid) - Task 4
    * delete_staging_ref(transaction_id) - Task 8, Drop
    * staging_ref_exists(transaction_id) - for testing
    * get_layer_ref(layer) - Task 5, Task 6
    * inner (public Repository access) - Task 6

ERROR_HANDLING:
  - use: src/core/error.rs
  - variants:
    * JinError::TransactionConflict - for OID conflicts
    * JinError::PrepareFailed { source, files } - preparation errors
    * JinError::CommitFailed { source, files } - commit errors
    * JinError::InvalidLayer - for UserLocal/WorkspaceActive

LAYER_SYSTEM:
  - use: src/core/layer.rs
  - methods:
    * layer.git_ref() - get ref name for locking/updating
    * layer.is_versioned() - validate before transaction

MODULE_EXPORTS:
  - modify: src/git/mod.rs
  - add: pub mod transaction; pub use transaction::{Transaction, TransactionManager, TransactionState};

TESTING:
  - create: tests/git/transaction_test.rs
  - use: tempfile crate (already in Cargo.toml)
  - pattern: TestRepoFixture from repo.rs

FUTURE_INTEGRATION:
  - P3.M2: Commit Pipeline will use Transaction for atomic commits
  - P4.M2: Commit command will use TransactionManager
  - P4.M5: Repair command will use TransactionManager::recover_all()
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating transaction.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "unused_imports" -> remove unused imports
# - "dead_code" -> public methods are used by tests, use #[allow(dead_code)] temporarily
# - Lifetime errors -> ensure Transaction lifetime is correctly bound to repo
# - Pattern matching errors -> ensure all TransactionState variants handled
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test transaction module specifically
cargo test --package jin --lib git::transaction --verbose

# Run all git module tests
cargo test --package jin --lib git:: --verbose

# Run with output
cargo test --package jin --lib git::transaction -- --nocapture

# Expected: All tests pass. Look for:
# - test_transaction_begin_creates_staging_ref
# - test_transaction_prepare_locks_refs
# - test_transaction_commit_atomic_update
# - test_transaction_drop_auto_rollback
# - test_multi_layer_atomic_commit
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual transaction operations with real git2
cargo test --package jin --test transaction_test --verbose

# Manual verification of atomic updates
# Create temp directory and test:
cd /tmp
mkdir test_jin_transaction
cd test_jin_transaction

# Run manual test Python script or use cargo test with output

# Verify atomic commit behavior:
# 1. Create transaction
# 2. Add multiple layer updates
# 3. Prepare transaction
# 4. Commit transaction
# 5. Verify all refs updated atomically

# Test rollback behavior:
# 1. Create transaction
# 2. Add updates
# 3. Prepare transaction
# 4. Rollback transaction
# 5. Verify no refs were modified

# Test orphaned transaction detection:
# 1. Create transaction
# 2. DON'T commit (simulate crash)
# 3. Run detect_orphaned()
# 4. Verify transaction ID is found

# Expected:
# - Atomic commits apply all updates or none
# - Rollback leaves refs unchanged
# - Orphaned transactions are detected
```

### Level 4: Domain-Specific Validation

```bash
# Verify multi-layer atomicity
cargo test --package jin test_multi_layer_atomic_commit -- --exact
# Asserts: All layers updated or none updated on commit failure

# Verify staging ref lifecycle
cargo test --package jin test_transaction_begin_creates_staging_ref -- --exact
# Asserts: Staging ref created on begin, deleted on commit/rollback

# Verify RAII cleanup
cargo test --package jin test_transaction_drop_auto_rollback -- --exact
# Asserts: Uncommitted transactions auto-rollback on drop

# Verify conflict detection
cargo test --package jin test_transaction_prepare_fails_on_conflict -- --exact
# Asserts: TransactionConflict error when refs modified

# Verify recovery functionality
cargo test --package jin test_transaction_manager_recover_all -- --exact
# Asserts: All orphaned transactions cleaned up

# Expected: All Jin-specific requirements met
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all public methods
- [ ] All structs have doc comments

### Feature Validation

- [ ] `Transaction::begin()` creates staging ref
- [ ] `Transaction::prepare()` locks refs and validates
- [ ] `Transaction::commit()` atomically updates all layer refs
- [ ] `Transaction::rollback()` releases locks without updates
- [ ] `Drop` auto-rolls back uncommitted transactions
- [ ] `TransactionManager::detect_orphaned()` finds incomplete transactions
- [ ] `TransactionManager::recover_all()` cleans up orphans
- [ ] Multi-layer commits are atomic (all or nothing)
- [ ] Integration with `JinRepo`, `JinError`, `Layer`

### Code Quality Validation

- [ ] Follows existing codebase patterns (repo.rs, error.rs structure)
- [ ] File placement matches desired tree structure
- [ ] Module exported from `src/git/mod.rs`
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods

### Documentation & Deployment

- [ ] Module-level doc comment explains transaction system purpose
- [ ] Each struct has doc comment explaining Jin-specific semantics
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (staging ref lifecycle, state transitions)

---

## Anti-Patterns to Avoid

- ❌ Don't hardcode "refs/jin/layers/" strings - use `layer.git_ref()`
- ❌ Don't forget to call `tx.lock()` BEFORE `tx.set_target()`
- ❌ Don't skip error conversion - use `JinError` consistently
- ❌ Don't commit transaction without preparing first
- ❌ Don't leave staging refs after commit/rollback
- ❌ Don't use `UserLocal` or `WorkspaceActive` layers in transactions
- ❌ Don't create transactions without dropping them (use RAII)
- ❌ Don't modify refs after commit but before cleanup
- ❌ Don't assume staging refs don't exist (check for conflicts)
- ❌ Don't ignore state validation (check state before operations)

---

## Appendix: Quick Reference

### Transaction API Summary

```rust
// Transaction (stateful, borrowed)
impl<'a> Transaction<'a> {
    pub fn begin(repo: &'a JinRepo) -> Result<Self>
    pub fn add_layer_update(&mut self, layer: Layer, new_oid: Oid) -> Result<()>
    pub fn prepare(&mut self) -> Result<()>
    pub fn commit(self) -> Result<()>
    pub fn rollback(self) -> Result<()>
}

// TransactionManager (factory + recovery)
impl TransactionManager {
    pub fn new(repo: JinRepo) -> Self
    pub fn begin_transaction(&self) -> Result<Transaction>
    pub fn detect_orphaned(&self) -> Result<Vec<String>>
    pub fn recover(&self, transaction_id: &str) -> Result<()>
    pub fn recover_all(&self) -> Result<usize>
}
```

### Transaction Lifecycle

```
[Start] -> begin() -> Started (staging ref created)
         -> add_layer_update() -> Started (collecting updates)
         -> prepare() -> Prepared (refs locked, validated)
         -> commit() -> Committed (atomic update, staging ref deleted)
         OR
         -> rollback() -> RolledBack (locks released, staging ref deleted)

[Error at any point] -> Drop -> RolledBack (auto-cleanup)
```

### Staging Ref Namespace

| Ref Pattern | Purpose |
|-------------|---------|
| `refs/jin/staging/<transaction-id>` | Transaction marker (created on begin, deleted on commit/rollback) |

### Error Mapping

| Operation | Error Condition | JinError Variant |
|-----------|----------------|------------------|
| `prepare()` | Layer modified since start | `TransactionConflict { conflict }` |
| `prepare()` | Lock acquisition fails | `PrepareFailed { source, files }` |
| `commit()` | Commit operation fails | `CommitFailed { source, files }` |
| `add_layer_update()` | Layer not versioned | `InvalidLayer { name }` |
| `begin()` | Staging ref already exists | `RefExists { name, layer }` |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
