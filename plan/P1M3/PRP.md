# PRP: P1.M3 - Transaction System

---

## Goal

**Feature Goal**: Implement an atomic multi-layer commit system with failure recovery that guarantees all-or-nothing semantics for Jin's multi-ref updates.

**Deliverable**: A complete transaction module (`src/git/transaction.rs`) providing:
1. `LayerTransaction` type for tracking multi-layer atomic commits
2. `TransactionState` and `TransactionLog` for two-phase commit with persistence
3. `RecoveryManager` for detecting and recovering interrupted transactions
4. Integration with `JinRepo` and existing git operations

**Success Definition**:
- All transaction tests pass: `cargo test git::transaction::`
- Multi-layer commits are truly atomic (all succeed or all fail)
- Interrupted transactions are detected on next `jin` command
- Recovery mechanism can rollback or resume incomplete transactions
- No partial commits are possible (PRD invariant)

---

## User Persona

**Target User**: Jin internals (commit pipeline, staging system)

**Use Case**: The transaction system is used by:
- `jin commit` to atomically update multiple layer refs
- Recovery detection on any `jin` command startup
- Future sync operations requiring atomic multi-ref updates

**User Journey**: Users don't interact directly with transactions - they experience atomic commits that either succeed completely or fail cleanly with no partial state.

**Pain Points Addressed**:
- Prevents corrupted Jin state from interrupted operations
- Ensures layer refs are always consistent
- Provides clear recovery path for failed transactions

---

## Why

- **PRD Invariant**: Section 6.2 states "`jin commit` is atomic across all affected layers. Partial commits are impossible."
- **Data Integrity**: Without atomic transactions, power failures or crashes during commit leave refs in inconsistent state
- **Recovery Requirement**: Section 15 requires "Auto-repair" and "Idempotent retries" for interrupted operations
- **Foundation for Sync**: Push/pull operations need atomic multi-ref updates for consistency

---

## What

### User-Visible Behavior

After this milestone:
```rust
// Atomic multi-layer commit (internal API)
let mut tx = LayerTransaction::begin(&repo)?;
tx.add_layer_update(Layer::ModeBase, mode, None, commit1)?;
tx.add_layer_update(Layer::ModeProject, mode, None, commit2)?;
tx.commit()?; // Either ALL refs update or NONE do

// Recovery on startup (automatic)
if let Some(incomplete) = RecoveryManager::detect(&repo)? {
    incomplete.rollback()?; // Clean up partial state
}
```

### Technical Requirements

1. **Two-Phase Commit**: Prepare phase writes intent, commit phase executes
2. **Transaction Log**: Persistent marker file at `.jin/.transaction_in_progress`
3. **Atomic File Operations**: Use POSIX atomic rename for state transitions
4. **Recovery Detection**: Check for incomplete transactions on any jin command
5. **Rollback Capability**: Undo partial updates using saved previous state

### Success Criteria

- [ ] `LayerTransaction::begin()` creates transaction log
- [ ] `LayerTransaction::add_layer_update()` queues ref updates
- [ ] `LayerTransaction::commit()` applies all updates atomically
- [ ] `LayerTransaction::abort()` rolls back cleanly
- [ ] `RecoveryManager::detect()` finds incomplete transactions
- [ ] `RecoveryManager::recover()` handles rollback or resume
- [ ] Transaction log deleted only after successful commit
- [ ] All operations are idempotent (safe to retry)
- [ ] Tests verify atomicity under simulated failures

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement Jin's transaction system, including exact type definitions, code patterns, failure scenarios, and recovery mechanisms._

### Documentation & References

```yaml
# MUST READ - Core Implementation Context

- file: src/git/transaction.rs
  why: Current JinTransaction wrapper (basic git2::Transaction)
  critical: |
    - git2::Transaction is NOT truly atomic
    - If one ref update fails, previous updates are NOT rolled back
    - This is explicitly documented in the module header
    - We need a higher-level transaction system on top

- file: plan/P1M2/research/KEY_TAKEAWAYS.md
  why: Best practices for atomic multi-ref operations
  critical: |
    - Use `git update-ref --stdin --atomic` for true atomicity
    - Alternatively: use two-phase commit with marker files
    - Transaction log provides recovery capability

- file: plan/P1M2/research/phantom_git_patterns.md
  why: Comprehensive research on Git transaction patterns
  sections:
    - "Section 3.2 Reference Transactions and Atomicity" - transaction states
    - "Section 5.2 Atomic Operations" - git update-ref patterns
    - "Appendix A: Quick Reference Commands" - atomic update syntax

- file: src/git/repo.rs
  why: JinRepo implementation for repository access
  pattern: |
    - JinRepo::open() / create() / open_or_create()
    - JinRepo::inner() returns &Repository
    - Create refs via set_ref() method

- file: src/git/refs.rs
  why: RefOps trait for reference operations
  pattern: |
    - RefOps::set_ref(name, oid, message)
    - RefOps::resolve_ref(name) -> Oid
    - RefOps::ref_exists(name) -> bool

- file: src/core/layer.rs
  why: Layer enum with ref_path() method
  pattern: |
    - Layer::ref_path(mode, scope, project) -> String
    - Layer::all_in_precedence_order() -> Vec<Layer>

- file: src/core/error.rs
  why: JinError::Transaction variant for transaction errors
  pattern: |
    - JinError::Transaction(String) for transaction failures

# EXTERNAL REFERENCES

- url: https://docs.rs/git2/latest/git2/struct.Transaction.html
  why: git2::Transaction API (current wrapper target)
  critical: |
    - Transaction::lock_ref() - lock before update
    - Transaction::set_target() - queue target change
    - Transaction::commit() - apply updates (NOT truly atomic)
    - Limitation: previous updates NOT rolled back on failure

- url: https://git-scm.com/docs/git-update-ref
  why: Atomic reference updates via plumbing command
  critical: |
    - --stdin --atomic flag for true atomic updates
    - start/prepare/commit/abort protocol
    - If any update fails, entire transaction aborts

- url: https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html
  why: Atomic file operations for transaction log
  critical: |
    - NamedTempFile::persist() uses atomic rename
    - Atomic rename is the key to crash-safe state transitions
    - tempfile crate already in dev-dependencies
```

### Current Codebase Tree (Relevant Files)

```bash
jin/
├── src/
│   ├── core/
│   │   ├── error.rs          # JinError::Transaction variant
│   │   └── layer.rs          # Layer::ref_path() for generating ref names
│   ├── git/
│   │   ├── mod.rs            # Exports JinTransaction
│   │   ├── repo.rs           # JinRepo wrapper
│   │   ├── refs.rs           # RefOps trait
│   │   ├── objects.rs        # ObjectOps for create_commit()
│   │   ├── tree.rs           # TreeOps for tree reading
│   │   └── transaction.rs    # Current basic JinTransaction (to be enhanced)
│   └── commit/
│       └── pipeline.rs       # CommitPipeline (consumer of transactions)
└── Cargo.toml                # tempfile in dev-dependencies
```

### Desired Codebase Tree After P1.M3

```bash
jin/
├── src/
│   ├── git/
│   │   ├── transaction.rs    # Enhanced with:
│   │   │   ├── JinTransaction        # Existing (kept for simple ops)
│   │   │   ├── LayerTransaction      # NEW: Multi-layer atomic commits
│   │   │   ├── TransactionState      # NEW: Enum (Pending/Prepared/Committed/Aborted)
│   │   │   ├── TransactionLog        # NEW: Persistent transaction state
│   │   │   ├── LayerUpdate           # NEW: Queued ref update
│   │   │   └── RecoveryManager       # NEW: Incomplete transaction handling
│   │   └── mod.rs            # Add exports for new types
│   └── commit/
│       └── pipeline.rs       # Updated to use LayerTransaction
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2::Transaction is NOT truly atomic
// From current src/git/transaction.rs:
// "If one ref update fails, previous successful updates are NOT rolled back"
// This is why we need LayerTransaction with two-phase commit

// CRITICAL: POSIX atomic rename guarantees
// std::fs::rename() is atomic on same filesystem
// Use this for state transitions: write temp file, rename to final location
// This is how transaction logs become crash-safe

// GOTCHA: Transaction log location
// Store at: .jin/.transaction_in_progress (project-local)
// NOT at ~/.jin/ because transactions are per-project

// GOTCHA: Recovery must happen before any other operation
// Check for incomplete transactions at the START of every jin command
// Before any ref modifications occur

// PATTERN: Two-phase commit states
// 1. BEGIN: Create transaction log with intended updates
// 2. PREPARE: Lock all refs, validate all updates possible
// 3. COMMIT: Apply updates, delete transaction log
// 4. ABORT: Release locks, optionally rollback, delete log

// PATTERN: Idempotent operations
// All operations should be safe to retry:
// - If commit succeeded, retry returns success (log already deleted)
// - If commit failed, retry attempts again
// - If interrupted, recovery handles cleanup

// PATTERN: Layer ref paths (from Layer::ref_path)
// refs/jin/layers/global
// refs/jin/layers/mode/{mode}
// refs/jin/layers/mode/{mode}/scope/{scope}
// refs/jin/layers/mode/{mode}/scope/{scope}/project/{project}
// refs/jin/layers/mode/{mode}/project/{project}
// refs/jin/layers/scope/{scope}
// refs/jin/layers/project/{project}
// refs/jin/layers/local
// refs/jin/layers/workspace
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/git/transaction.rs (ADDITIONS) ==================

use crate::core::{JinError, Layer, Result};
use git2::Oid;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// State of a LayerTransaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    /// Transaction created, updates being queued
    Pending,
    /// All updates validated, refs locked (ready to commit)
    Prepared,
    /// All updates applied successfully
    Committed,
    /// Transaction aborted, changes rolled back
    Aborted,
}

/// A queued update to a layer reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerUpdate {
    /// The layer being updated
    pub layer: Layer,
    /// Mode name (if applicable)
    pub mode: Option<String>,
    /// Scope name (if applicable)
    pub scope: Option<String>,
    /// Project name (if applicable)
    pub project: Option<String>,
    /// The full ref path (computed from layer + context)
    pub ref_path: String,
    /// Previous commit OID (for rollback, None if ref didn't exist)
    pub old_oid: Option<String>,
    /// New commit OID
    pub new_oid: String,
}

impl LayerUpdate {
    /// Create a new layer update
    pub fn new(
        layer: Layer,
        mode: Option<String>,
        scope: Option<String>,
        project: Option<String>,
        old_oid: Option<Oid>,
        new_oid: Oid,
    ) -> Self {
        let ref_path = layer.ref_path(
            mode.as_deref(),
            scope.as_deref(),
            project.as_deref(),
        );
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

/// Persistent transaction log for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    /// Version of the log format
    pub version: u32,
    /// Transaction ID (UUID or timestamp)
    pub id: String,
    /// Current state of the transaction
    pub state: TransactionState,
    /// Timestamp when transaction started
    pub started_at: String,
    /// Commit message for this transaction
    pub message: String,
    /// All queued layer updates
    pub updates: Vec<LayerUpdate>,
}

impl TransactionLog {
    /// Create a new transaction log
    pub fn new(message: impl Into<String>) -> Self {
        let id = chrono::Utc::now().format("%Y%m%d%H%M%S%f").to_string();
        Self {
            version: 1,
            id,
            state: TransactionState::Pending,
            started_at: chrono::Utc::now().to_rfc3339(),
            message: message.into(),
            updates: Vec::new(),
        }
    }

    /// Returns the default path for transaction log
    pub fn default_path() -> PathBuf {
        PathBuf::from(".jin").join(".transaction_in_progress")
    }

    /// Load transaction log from disk (if exists)
    pub fn load() -> Result<Option<Self>> {
        let path = Self::default_path();
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let log: Self = serde_json::from_str(&content).map_err(|e| {
            JinError::Transaction(format!("Failed to parse transaction log: {}", e))
        })?;
        Ok(Some(log))
    }

    /// Save transaction log to disk atomically
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
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

    /// Delete transaction log (called after successful commit)
    pub fn delete() -> Result<()> {
        let path = Self::default_path();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}


/// Multi-layer atomic transaction
///
/// Provides true atomic multi-ref updates using a two-phase commit pattern
/// with persistent transaction log for crash recovery.
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
/// // Queue updates
/// tx.add_layer_update(Layer::ModeBase, Some("claude"), None, None, commit1)?;
/// tx.add_layer_update(Layer::ModeProject, Some("claude"), None, Some("my-project"), commit2)?;
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
    /// Begin a new atomic transaction
    ///
    /// Creates a transaction log file to track state for crash recovery.
    pub fn begin(repo: &'repo super::JinRepo, message: impl Into<String>) -> Result<Self> {
        // Check for existing incomplete transaction
        if TransactionLog::load()?.is_some() {
            return Err(JinError::Transaction(
                "Incomplete transaction exists. Run recovery first.".to_string()
            ));
        }

        let log = TransactionLog::new(message);
        log.save()?;

        Ok(Self { repo, log })
    }

    /// Add a layer update to the transaction
    ///
    /// Records the current ref value (for rollback) and queues the update.
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

    /// Commit the transaction atomically
    ///
    /// Uses git2::Transaction for the actual ref updates, with our
    /// transaction log providing crash recovery capability.
    pub fn commit(mut self) -> Result<()> {
        if self.log.updates.is_empty() {
            TransactionLog::delete()?;
            return Ok(());
        }

        // Phase 1: Prepare - mark as prepared in log
        self.log.state = TransactionState::Prepared;
        self.log.save()?;

        // Phase 2: Execute ref updates via git2::Transaction
        let result = self.execute_updates();

        match result {
            Ok(()) => {
                // Phase 3: Complete - mark as committed and delete log
                self.log.state = TransactionState::Committed;
                TransactionLog::delete()?;
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                self.rollback()?;
                Err(e)
            }
        }
    }

    /// Execute the actual ref updates
    fn execute_updates(&self) -> Result<()> {
        use super::JinTransaction;

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

    /// Rollback the transaction, restoring previous ref values
    fn rollback(&mut self) -> Result<()> {
        use super::refs::RefOps;

        // Restore refs to previous values
        for update in &self.log.updates {
            if let Some(old_oid_str) = &update.old_oid {
                let old_oid = Oid::from_str(old_oid_str).map_err(|e| {
                    JinError::Transaction(format!("Invalid old OID {}: {}", old_oid_str, e))
                })?;
                // Best effort: try to restore, continue on error
                let _ = self.repo.set_ref(&update.ref_path, old_oid, "Rollback");
            } else {
                // Ref didn't exist before, delete it
                let _ = self.repo.delete_ref(&update.ref_path);
            }
        }

        self.log.state = TransactionState::Aborted;
        TransactionLog::delete()?;

        Ok(())
    }

    /// Abort the transaction without applying any changes
    pub fn abort(mut self) -> Result<()> {
        self.log.state = TransactionState::Aborted;
        TransactionLog::delete()?;
        Ok(())
    }
}


/// Recovery manager for incomplete transactions
///
/// Detects and handles transactions that were interrupted (e.g., by crash).
pub struct RecoveryManager;

impl RecoveryManager {
    /// Detect if there's an incomplete transaction
    ///
    /// Should be called at the start of any jin command.
    pub fn detect() -> Result<Option<IncompleteTransaction>> {
        match TransactionLog::load()? {
            Some(log) => Ok(Some(IncompleteTransaction { log })),
            None => Ok(None),
        }
    }

    /// Check for and automatically handle incomplete transactions
    ///
    /// Returns true if a recovery was performed.
    pub fn auto_recover(repo: &super::JinRepo) -> Result<bool> {
        match Self::detect()? {
            Some(incomplete) => {
                // Strategy: rollback incomplete transactions
                // (Alternative: could prompt user or attempt resume)
                incomplete.rollback(repo)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

/// An incomplete transaction detected during recovery
pub struct IncompleteTransaction {
    log: TransactionLog,
}

impl IncompleteTransaction {
    /// Get the transaction state
    pub fn state(&self) -> TransactionState {
        self.log.state
    }

    /// Get the transaction ID
    pub fn id(&self) -> &str {
        &self.log.id
    }

    /// Get when the transaction started
    pub fn started_at(&self) -> &str {
        &self.log.started_at
    }

    /// Get the number of updates in the transaction
    pub fn update_count(&self) -> usize {
        self.log.updates.len()
    }

    /// Rollback the incomplete transaction
    ///
    /// Restores all refs to their previous values.
    pub fn rollback(self, repo: &super::JinRepo) -> Result<()> {
        use super::refs::RefOps;

        for update in &self.log.updates {
            if let Some(old_oid_str) = &update.old_oid {
                let old_oid = Oid::from_str(old_oid_str).map_err(|e| {
                    JinError::Transaction(format!("Invalid old OID: {}", e))
                })?;
                // Best effort restore
                let _ = repo.set_ref(&update.ref_path, old_oid, "Recovery rollback");
            } else {
                // Ref didn't exist before, try to delete
                let _ = repo.delete_ref(&update.ref_path);
            }
        }

        TransactionLog::delete()?;
        Ok(())
    }

    /// Resume the incomplete transaction (attempt to complete it)
    ///
    /// Only valid if state is Prepared (updates were in progress).
    pub fn resume(self, repo: &super::JinRepo) -> Result<()> {
        if self.log.state != TransactionState::Prepared {
            return Err(JinError::Transaction(
                "Can only resume transactions in Prepared state".to_string()
            ));
        }

        use super::JinTransaction;

        let mut tx = JinTransaction::new(repo)?;

        for update in &self.log.updates {
            tx.lock_ref(&update.ref_path)?;
        }

        for update in &self.log.updates {
            let oid = Oid::from_str(&update.new_oid).map_err(|e| {
                JinError::Transaction(format!("Invalid OID: {}", e))
            })?;
            tx.set_target(&update.ref_path, oid, &self.log.message)?;
        }

        tx.commit()?;
        TransactionLog::delete()?;

        Ok(())
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD transaction types to src/git/transaction.rs
  - IMPLEMENT: TransactionState enum (Pending, Prepared, Committed, Aborted)
  - IMPLEMENT: LayerUpdate struct with layer, ref_path, old_oid, new_oid
  - PLACEMENT: After existing JinTransaction struct
  - DEPENDS ON: Nothing - can be done first
  - TESTS: Basic construction and serialization

Task 2: ADD TransactionLog to src/git/transaction.rs
  - IMPLEMENT: TransactionLog struct as specified in Data Models
  - METHODS: new(), default_path(), load(), save(), delete()
  - PATTERN: Use atomic rename (write temp, rename to final)
  - DEPENDS ON: Task 1 (uses TransactionState, LayerUpdate)
  - TESTS: Save/load roundtrip, atomic rename behavior

Task 3: ADD LayerTransaction to src/git/transaction.rs
  - IMPLEMENT: LayerTransaction struct with begin(), add_layer_update(), commit(), abort()
  - PRIVATE METHODS: execute_updates(), rollback()
  - INTEGRATION: Use existing JinTransaction for actual ref updates
  - DEPENDS ON: Task 2 (uses TransactionLog)
  - TESTS: Simple commit, multi-layer commit, abort behavior

Task 4: ADD RecoveryManager to src/git/transaction.rs
  - IMPLEMENT: RecoveryManager with detect(), auto_recover()
  - IMPLEMENT: IncompleteTransaction with state(), rollback(), resume()
  - PLACEMENT: After LayerTransaction
  - DEPENDS ON: Task 3 (uses TransactionLog types)
  - TESTS: Detection of incomplete transactions, rollback, resume

Task 5: UPDATE src/git/mod.rs exports
  - ADD: pub use transaction::{LayerTransaction, TransactionState, TransactionLog}
  - ADD: pub use transaction::{LayerUpdate, RecoveryManager, IncompleteTransaction}
  - PRESERVE: Existing JinTransaction export
  - DEPENDS ON: Tasks 1-4

Task 6: ADD comprehensive tests
  - FILE: Tests inline in transaction.rs (after existing tests)
  - TESTS:
    - test_transaction_log_save_load: Roundtrip serialization
    - test_transaction_log_atomic_write: Verify atomic rename
    - test_layer_transaction_empty: Empty commit behavior
    - test_layer_transaction_single_layer: Single layer update
    - test_layer_transaction_multi_layer: Multiple layer updates
    - test_layer_transaction_abort: Abort discards changes
    - test_layer_transaction_rollback_on_failure: Rollback restores old values
    - test_recovery_detect_incomplete: Detection of incomplete transaction
    - test_recovery_rollback: Rollback restores previous state
    - test_recovery_resume: Resume completes prepared transaction
    - test_concurrent_transaction_blocked: Can't start second transaction
  - PATTERN: Use tempdir for test repositories
  - DEPENDS ON: Tasks 1-5
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Atomic file write using rename
fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, path)?;  // Atomic on POSIX
    Ok(())
}

// PATTERN: Recovery check at command start
// This should be integrated into lib.rs run() function:
pub fn run(cli: Cli) -> anyhow::Result<()> {
    // Auto-recover incomplete transactions before any operation
    if let Ok(repo) = JinRepo::open() {
        if let Ok(true) = RecoveryManager::auto_recover(&repo) {
            eprintln!("Recovered from incomplete transaction");
        }
    }

    commands::execute(cli).map_err(Into::into)
}

// PATTERN: Two-phase commit timing
// 1. BEGIN: Transaction log created, state = Pending
// 2. add_layer_update(): Updates queued, log updated
// 3. commit() start: state = Prepared, log saved (POINT OF NO RETURN)
// 4. execute_updates(): git2::Transaction applies updates
// 5. commit() end: log deleted, state conceptually = Committed
//
// If crash occurs:
// - Before Prepared: Rollback (no refs modified)
// - After Prepared: Resume (complete the updates)

// PATTERN: Layer update construction
let update = LayerUpdate::new(
    Layer::ModeBase,
    Some("claude".to_string()),
    None,
    None,
    Some(old_commit),  // Captured for rollback
    new_commit,
);
// ref_path computed: "refs/jin/layers/mode/claude"

// PATTERN: Test setup with tempdir
#[cfg(test)]
fn create_test_repo() -> (TempDir, JinRepo) {
    let temp = TempDir::new().unwrap();
    // Set current directory so TransactionLog::default_path() works
    std::env::set_current_dir(temp.path()).unwrap();
    std::fs::create_dir_all(".jin").unwrap();

    let repo_path = temp.path().join(".jin-repo");
    let repo = JinRepo::create_at(&repo_path).unwrap();
    (temp, repo)
}

// GOTCHA: Transaction log path is relative to current directory
// TransactionLog::default_path() returns PathBuf::from(".jin/.transaction_in_progress")
// Tests need to set current directory or use absolute paths
```

### Integration Points

```yaml
FILESYSTEM:
  - Transaction log at: .jin/.transaction_in_progress (JSON format)
  - Temp file for atomic write: .jin/.transaction_in_progress.tmp
  - Both cleaned up after successful commit

GIT:
  - Uses existing JinTransaction for actual ref updates
  - Layer refs at: refs/jin/layers/* (via Layer::ref_path())
  - Each LayerUpdate records old_oid for rollback capability

DEPENDENCIES (from existing modules):
  - git/repo.rs: JinRepo for repository access
  - git/refs.rs: RefOps trait for ref operations
  - core/layer.rs: Layer::ref_path() for ref name generation
  - core/error.rs: JinError::Transaction for errors

FUTURE INTEGRATION:
  - commit/pipeline.rs: Will use LayerTransaction for atomic commits
  - lib.rs: Will call RecoveryManager::auto_recover() on startup
  - commands/*.rs: May use recovery status for user feedback
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build
```

### Level 3: Unit Tests (Component Validation)

```bash
# Run transaction module tests
cargo test git::transaction::         # All transaction tests
cargo test test_transaction_log       # Transaction log tests
cargo test test_layer_transaction     # LayerTransaction tests
cargo test test_recovery              # Recovery tests

# Run with output for debugging
cargo test git::transaction:: -- --nocapture

# Expected: All tests pass
```

### Level 4: Integration Testing

```bash
# Full test suite
cargo test

# Verify existing tests still pass
cargo test git::repo::
cargo test git::refs::
cargo test git::objects::

# Manual verification
cd $(mktemp -d)
git init
mkdir .jin

# Create a test scenario programmatically (via cargo test)

# Expected: All integration points work correctly
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo test git::transaction::` all tests pass
- [ ] `cargo test` all tests pass (including existing)

### Feature Validation

- [ ] TransactionState enum has all 4 states
- [ ] LayerUpdate correctly computes ref_path from Layer
- [ ] TransactionLog::save() uses atomic rename
- [ ] TransactionLog::load() correctly deserializes
- [ ] LayerTransaction::begin() creates transaction log
- [ ] LayerTransaction::add_layer_update() records old_oid
- [ ] LayerTransaction::commit() applies all updates atomically
- [ ] LayerTransaction::abort() cleans up without applying
- [ ] Rollback restores refs to previous values
- [ ] RecoveryManager::detect() finds incomplete transactions
- [ ] IncompleteTransaction::rollback() works correctly
- [ ] IncompleteTransaction::resume() completes prepared transactions
- [ ] Cannot start second transaction while one is active

### Code Quality Validation

- [ ] All public types have doc comments
- [ ] Error handling uses JinError::Transaction consistently
- [ ] No unwrap() in library code
- [ ] Tests use tempdir for isolation
- [ ] Follows existing code patterns (RefOps, ObjectOps style)

---

## Anti-Patterns to Avoid

- Don't rely solely on git2::Transaction for atomicity - it doesn't rollback on partial failure
- Don't store transaction log at ~/.jin/ - it should be per-project at .jin/
- Don't skip the atomic rename - use temp file then rename pattern
- Don't forget to capture old_oid - needed for rollback capability
- Don't delete transaction log before updates complete - that's the recovery mechanism
- Don't skip recovery check - incomplete transactions must be handled before any operation
- Don't use unwrap() in transaction code - errors must propagate properly
- Don't assume current directory - use proper path resolution

---

## Confidence Score

**Rating: 8/10** for one-pass implementation success

**Justification:**
- Clear two-phase commit pattern with well-defined states
- Atomic file operations via rename are well-understood
- Builds on existing JinTransaction infrastructure
- Transaction log format is straightforward JSON
- Recovery mechanisms have clear semantics
- Comprehensive test cases defined

**Remaining Risks:**
- git2::Transaction partial failure behavior may need careful testing
- Cross-platform atomic rename behavior (Windows vs POSIX)
- Current directory assumption for transaction log path
- Edge cases with concurrent processes (file locking not implemented)

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M3/research/`

Key research references:
- `plan/P1M2/research/phantom_git_patterns.md` - Git transaction patterns
- `plan/P1M2/research/KEY_TAKEAWAYS.md` - Atomic operation recommendations
- git2-rs Transaction documentation: https://docs.rs/git2/latest/git2/struct.Transaction.html
- git update-ref documentation: https://git-scm.com/docs/git-update-ref
- tempfile crate for atomic file operations: https://docs.rs/tempfile/latest/tempfile/

---

## Appendix: Transaction State Diagram

```
                    BEGIN
                      │
                      ▼
              ┌──────────────┐
              │   PENDING    │  Transaction log created
              └──────────────┘  add_layer_update() called
                      │
                      │ commit() called
                      ▼
              ┌──────────────┐
              │   PREPARED   │  State saved to log
              └──────────────┘  (Point of no return)
                      │
         ┌────────────┴────────────┐
         │                         │
    Success                    Failure
         │                         │
         ▼                         ▼
┌──────────────┐          ┌──────────────┐
│  COMMITTED   │          │   ABORTED    │
└──────────────┘          └──────────────┘
   Log deleted            Refs rolled back
                          Log deleted

Recovery:
- If PENDING: Rollback (no refs modified yet)
- If PREPARED: Resume (complete the updates) or Rollback
```
