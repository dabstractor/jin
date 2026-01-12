# Atomic Transaction Patterns and Failure Recovery in Rust

## Research Overview

This document synthesizes patterns and best practices for implementing atomic transactions with failure recovery in Rust, with specific focus on two-phase commit, write-ahead logging, git2 limitations, and recovery detection mechanisms.

---

## 1. Two-Phase Commit (2PC) Patterns in Rust

### 1.1 Overview and Phases

Two-Phase Commit is a distributed algorithm for coordinating atomic transactions across multiple participants. Reference: [Two-Phase Commit - Martin Fowler](https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html)

**Phase 1 (Prepare):**
- Coordinator asks each participant if it can execute the transaction
- Participants acquire necessary resources (locks, disk space, etc.)
- Participants respond YES only if they guarantee they can commit in Phase 2
- If ANY participant says NO, the entire transaction is aborted

**Phase 2 (Commit):**
- If all participants agreed in Phase 1, coordinator instructs execution
- At this point, successful completion is guaranteed (participants have resources locked)
- Failures roll back the transaction and release all acquired locks

### 1.2 Transaction Journal/Log Patterns

The critical pattern for 2PC is durability through Write-Ahead Logging (WAL):

```
Key Principle: "It is crucial for each participant to ensure the durability
of their decisions using patterns like Write-Ahead Log. This means that even
if a node crashes and subsequently restarts, it should be capable of
completing the protocol without any issues."
```

**Transaction Journal Structure:**

```
Transaction Journal Entry:
├── transaction_id: unique identifier
├── phase: "PREPARE", "COMMIT", or "ABORT"
├── timestamp: when decision was made
├── participant_responses: [
│   {
│       participant_id: "node-1",
│       decision: "YES"|"NO",
│       timestamp: when response was written
│   }
├── fsync_marker: "DURABLE" (only after fsync)
└── status: "PENDING", "COMMITTED", "ABORTED"
```

### 1.3 Recovery on Restart

Recovery algorithm:

```
1. Read transaction journal from stable storage
2. For each incomplete transaction (missing COMMITTED or ABORTED marker):
   - Check if transaction is partially committed on disk
   - If all participants have committed: complete the commit (redo)
   - If any participant failed: roll back changes (undo)
   - Write final status to journal with fsync
3. Resume normal operations
```

### 1.4 Rust Implementation Resources

Available implementations:

- **[dilawarm/two-phase-commit](https://github.com/dilawarm/two-phase-commit)** - Full Rust & Go implementation with Orchestrator pattern
- **[amirheidarikhoram/2pc-rust-impl](https://github.com/amirheidarikhoram/2pc-rust-impl)** - Naive 2PC with TCP coordination and state management
- **[augrim crate](https://docs.rs/augrim)** - Two-Phase Commit atomic commitment protocol crate
- **Academic assignments** from University of Texas using `ipc-channel` for multi-process coordination

Key characteristics:
- Use `tokio` for async coordination
- Implement message channels for participant coordination
- Maintain transaction state in concurrent-safe structures (Arc<Mutex<>>)

---

## 2. File-Based Transaction Approaches

### 2.1 Write-Ahead Logging (WAL) Patterns

WAL is the foundation of durable transaction systems. Reference: [Introduction to OkayWAL](https://bonsaidb.io/blog/introducing-okaywal/)

**Basic WAL Flow:**

```
1. Client requests transaction
2. WRITE operation to WAL (atomic append)
3. Once fsync completes, operation is durable
4. Apply change to primary data store
5. Optional: checkpoint to reclaim WAL space
```

**Benefits:**
- Fast writes (append-only is optimal for disk)
- Durability guaranteed after fsync
- Simple recovery (replay log on startup)
- Efficient checkpointing (batch updates to primary store)

### 2.2 OkayWAL - Production WAL Implementation

OkayWAL is a general-purpose WAL for Rust: [GitHub - khonsulabs/okaywal](https://github.com/khonsulabs/okaywal)

**Architecture:**

```
Segment Files (wal-{id}):
├── Magic code: "okw"
├── Version information
├── Entry markers:
│   ├── 1: Entry start
│   ├── 2: Chunk markers
│   └── 3: Entry completion

Pre-allocation:
├── Configurable segment size (e.g., 10MB)
├── Reduces allocation overhead
├── Faster writes than allocating on-demand

Checkpointing:
├── Background thread monitors segment size
├── Invokes LogManager to persist checkpoint data
├── Transitions to "-cp" suffix during checkpointing
├── Prevents new readers, existing readers complete
```

**Key OkayWAL APIs:**

```rust
// Create or recover WAL
let wal = WriteAheadLog::recover(directory, log_manager)?;

// Write entries with automatic batching
wal.write_entry(serialized_data)?;

// Ensure durability
wal.fsync()?;

// Checkpoint and cleanup
wal.checkpoint()?;
```

**Characteristics:**
- Multi-threaded atomic writes with optimized fsync batching
- Random access to previously written data
- Automatic checkpoint management
- Recovery with basic data versioning
- Performance: ~1ms for 256-byte entries across 1-16 threads

**Status:** v0.2, not yet production-ready pending Nebari integration

### 2.3 Alternative WAL Implementations

**[wral crate](https://docs.rs/wral/)** - Write-Ahead-Logging:
- Append-only journal file
- Monotonically increasing sequence numbers
- Configurable journal rotation
- Optional state persistence per batch

**[simple_wal crate](https://docs.rs/simple_wal/)** - Simple approach:
- Full log scan on startup to detect interrupted writes
- Automatic cleanup of partial entries
- Recommended to compact log periodically

**[walcraft](https://github.com/RustyFarmer101/walcraft)** - Concurrent WAL:
- In-memory buffer with append-only logs
- Multiple log files with automatic cleanup
- Optimized for concurrent environments

### 2.4 Atomic File Rename Patterns (POSIX Semantics)

POSIX `rename()` is atomic on Unix systems. Reference: [atomicwrites crate](https://github.com/untitaker/rust-atomicwrites)

**Basic Atomic Write Pattern:**

```
1. Create temporary file in same directory as target
2. Write all data to temporary file
3. Call fsync on temporary file (ensure durability)
4. Call rename(temp_file, target_file) (atomic on POSIX)
   - If target exists, atomically replaces it
   - If crash occurs between fsync and rename:
     * Temporary file left on disk (can be cleaned up)
     * Target file unchanged (safe)
5. fsync on directory (ensure metadata durability)
```

**Implementation Crates:**

**[atomicwrites](https://crates.io/crates/atomicwrites):**

```rust
use atomicwrites::{AtomicFile, DisallowOverwrite};

let af = AtomicFile::new("target.txt", DisallowOverwrite);
af.write(|f| {
    f.write_all(b"atomic content")
})?;
```

Two modes:
- `AllowOverwrite`: Uses rename (atomically replaces existing file)
- `DisallowOverwrite`: Uses link + unlink (fails if target exists)

**[atomic-write-file](https://docs.rs/atomic-write-file/latest/atomic_write_file/):**

```rust
use atomic_write_file::AtomicWriteFile;

let mut file = AtomicWriteFile::open("target.txt")?;
file.write_all(b"data")?;
file.commit()?;  // Atomic rename
```

Advanced features:
- Uses directory file descriptors (openat, linkat, renameat)
- Handles directory renames/remounts during operation
- Supports all major platforms (Unix, Windows, WASI)
- Automatically removes temp files on drop if not committed

**[tempfile crate](https://docs.rs/tempfile/):**

```rust
use tempfile::NamedTempFile;

let mut tmp = NamedTempFile::new()?;
tmp.write_all(b"data")?;
tmp.flush()?;
tmp.persist("target.txt")?;  // Atomic rename, replaces existing
```

Important caveat: "Neither the file contents nor the containing directory are synchronized, so the update may not yet have reached the disk when persist returns."

**Windows Considerations:**

Previously, atomic writes on Windows were extremely racy due to lack of POSIX semantics. Windows 10 1601+ introduced:
- `FileRenameInfoEx` with `FILE_RENAME_FLAG_POSIX_SEMANTICS`
- Allows atomic rename with target file open for sharing
- Fallback to `FileRenameInfo` if not available
- Reference: [Rust PR #131072](https://github.com/rust-lang/rust/pull/131072)

**Recovery on Crash:**

```
Scenario: Process crashes between fsync(temp) and rename(temp, target)

Result:
├── Temporary file exists on disk
├── Target file unchanged (safe)
└── Recovery: Delete temporary files matching pattern (*.tmp, .jin/*)

Detection:
├── Scan destination directory for temp files on startup
├── Delete any found temp files
├── Safe because they're not yet the target
```

---

## 3. git2-rs Limitations for Transactions

### 3.1 git2 Transaction API

Reference: [git2::Transaction - docs.rs](https://docs.rs/git2/latest/git2/struct.Transaction.html)

**API Methods:**

```rust
// Create transaction
let mut tx = repo.transaction()?;

// Lock and update references
tx.lock_ref("refs/heads/main")?;
tx.set_target(oid)?;
tx.set_symbolic_target("refs/remotes/origin/main")?;
tx.set_reflog("User: operation description")?;
tx.remove()?;

// Commit all changes
tx.commit()?;
```

### 3.2 Critical Atomicity Limitation

**Problem:** Git2 transactions are NOT truly atomic on failure.

```
From git2 documentation:
"committing is not atomic: if an operation fails, the transaction
aborts, but previous successful operations are not rolled back."
```

**Failure Scenario:**

```
Transaction with 3 ref updates:
1. Update refs/heads/branch1 ✓
2. Update refs/heads/branch2 ✓
3. Update refs/heads/branch3 ✗ (fails)

Result:
├── branch1: UPDATED (persisted to disk)
├── branch2: UPDATED (persisted to disk)
├── branch3: UNCHANGED
└── Partial state visible to other processes!
```

**Reflog Atomicity Issue:**

When combining reflog with target updates:
```
"Atomicity is not guaranteed: if the transaction fails to modify
refname, the reflog may still have been committed to disk. If this
is combined with setting the target, that update won't be written
to the log."
```

**Design Gap:**

There's no API to determine which reference updates succeeded/failed:
- Can't call reference-transaction hook with actual succeeded updates
- Can't determine recovery state unambiguously

### 3.3 Alternative: git update-ref --atomic --stdin

Reference: [Git Documentation - git-update-ref](https://git-scm.com/docs/git-update-ref)

**Advantages over git2:**

```
$ git update-ref --atomic --stdin << EOF
create refs/heads/new-branch abc123def...
update refs/heads/main old-oid new-oid
delete refs/heads/old-branch
EOF
```

**True Atomicity:**

```
"With --stdin, update-ref reads instructions from standard input
and performs all modifications together. If all refs can be locked
with matching old-oids simultaneously, all modifications are
performed; otherwise, NO modifications are performed."
```

**All-or-Nothing Behavior:**

```
Transaction: 3 ref updates
├── If all locks acquired: all committed atomically
├── If any lock fails: no changes (full rollback)
└── Single atomic operation at filesystem level
```

**Rust Implementation:**

```rust
use std::process::{Command, Stdio};
use std::io::Write;

fn atomic_git_refs(repo_path: &str, updates: &[&str]) -> Result<()> {
    let mut child = Command::new("git")
        .arg("-C").arg(repo_path)
        .arg("update-ref")
        .arg("--atomic")
        .arg("--stdin")
        .stdin(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        for update in updates {
            stdin.write_all(update.as_bytes())?;
            stdin.write_all(b"\n")?;
        }
    }

    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err("Atomic update failed - no changes applied".into())
    }
}
```

**Recent Development (Git 2.52+):**

New `--batch-updates` flag for partial success mode:
- Allows individual updates to fail
- Transaction continues despite failures
- Reference: [Karthik Nayak - Batch Updates Patch](https://public-inbox.org/git/20250327-245-partially-atomic-ref-updates-v5-8-4db2a3e34404@gmail.com/)

---

## 4. Recovery Detection Patterns

### 4.1 Detecting Interrupted Operations

**Challenge:** After crash/SIGKILL, determine transaction state:
- Committed successfully?
- In progress (partially committed)?
- Never started?

**Detection Strategies:**

#### A. Marker File Approach (Simple)

```
Transaction States:
├── NOT STARTED
│   └── No marker file exists
│
├── IN PROGRESS
│   └── Marker file exists: .jin/.transaction_in_progress
│       ├── Contains: transaction_id, timestamp, operation_type
│       └── File locked by active process (flock)
│
├── COMMITTED
│   └── Marker renamed to: .jin/.transaction_committed
│       └── Contains: transaction_id, commit_timestamp, final_state
│
└── FAILED/ROLLED_BACK
    └── Marker renamed to: .jin/.transaction_failed
        └── Contains: transaction_id, error_info, rollback_timestamp
```

**Implementation:**

```rust
use std::fs;
use std::path::Path;

const MARKER_DIR: &str = ".jin";
const IN_PROGRESS_MARKER: &str = ".jin/.transaction_in_progress";
const COMMITTED_MARKER: &str = ".jin/.transaction_committed";
const FAILED_MARKER: &str = ".jin/.transaction_failed";

struct TransactionMarker {
    id: String,
    timestamp: u64,
    operation: String,
}

fn start_transaction(id: &str, operation: &str) -> Result<()> {
    // Create marker directory
    fs::create_dir_all(MARKER_DIR)?;

    let marker = TransactionMarker {
        id: id.to_string(),
        timestamp: current_timestamp(),
        operation: operation.to_string(),
    };

    // Write to temp file first
    let temp_path = format!("{}.tmp", IN_PROGRESS_MARKER);
    fs::write(&temp_path, serde_json::to_string(&marker)?)?;

    // Atomic rename
    fs::rename(&temp_path, IN_PROGRESS_MARKER)?;

    Ok(())
}

fn detect_interrupted_transactions() -> Result<Vec<String>> {
    let mut interrupted = Vec::new();

    // Check for in-progress markers from crashed processes
    if Path::new(IN_PROGRESS_MARKER).exists() {
        if let Ok(content) = fs::read_to_string(IN_PROGRESS_MARKER) {
            if let Ok(marker) = serde_json::from_str::<TransactionMarker>(&content) {
                // Check if marker is stale (crashed before completion)
                if is_stale(&marker) {
                    interrupted.push(marker.id);
                }
            }
        }
    }

    Ok(interrupted)
}

fn complete_transaction(id: &str, success: bool) -> Result<()> {
    let target_marker = if success {
        COMMITTED_MARKER
    } else {
        FAILED_MARKER
    };

    // Atomic rename - moves marker to final state
    fs::rename(IN_PROGRESS_MARKER, target_marker)?;

    Ok(())
}
```

#### B. Journal Entry Approach (Robust)

```
Transaction Journal Format:
├── Entry 1: {id: "tx1", ts: 100, op: "BEGIN", phase: "PREPARE"}
├── Entry 2: {id: "tx1", ts: 101, op: "REF_UPDATE", ref: "main", oid: "abc123"}
├── Entry 3: {id: "tx1", ts: 102, op: "COMMIT"}
├── Entry 4: {id: "tx1", ts: 103, op: "COMMIT_FSYNC_COMPLETE"} ← Durability marker
└── Entry 5: {id: "tx1", ts: 104, op: "CLEANUP"}

Recovery Algorithm:
├── For each transaction without COMMIT_FSYNC_COMPLETE:
│   ├── If has REF_UPDATE entries: transaction in-flight
│   ├── Check which refs actually persisted on disk
│   ├── If all persisted: redo commit (safe)
│   ├── If partial: rollback using undo log
│   └── If none: rollback unused
└── For each transaction with COMMIT_FSYNC_COMPLETE but no CLEANUP:
    └── Complete cleanup (idempotent)
```

**Implementation with OkayWAL:**

```rust
use okaywal::{WriteAheadLog, LogManager};

#[derive(Serialize, Deserialize)]
enum TransactionLogEntry {
    Begin { id: String, timestamp: u64 },
    RefUpdate { id: String, ref_name: String, old_oid: String, new_oid: String },
    Prepare { id: String, responses: Vec<(String, bool)> },
    CommitFsyncComplete { id: String },
    Cleanup { id: String },
    Abort { id: String, reason: String },
}

fn recovery_on_startup(wal_dir: &str) -> Result<()> {
    let log_manager = MyLogManager::new();
    let wal = WriteAheadLog::recover(wal_dir, log_manager)?;

    // Read all entries since last checkpoint
    let entries = wal.read_since_checkpoint()?;

    let mut incomplete_txs = HashMap::new();
    for entry in entries {
        match entry {
            TransactionLogEntry::Begin { id, timestamp } => {
                incomplete_txs.insert(id.clone(), (vec![], false));
            }
            TransactionLogEntry::RefUpdate { id, ref_name, old_oid, new_oid } => {
                if let Some((refs, _)) = incomplete_txs.get_mut(&id) {
                    refs.push((ref_name, old_oid, new_oid));
                }
            }
            TransactionLogEntry::CommitFsyncComplete { id } => {
                if let Some((_, fsync_complete)) = incomplete_txs.get_mut(&id) {
                    *fsync_complete = true;
                }
            }
            TransactionLogEntry::Cleanup { id } => {
                incomplete_txs.remove(&id);
            }
            _ => {}
        }
    }

    // Process incomplete transactions
    for (tx_id, (refs, fsync_complete)) in incomplete_txs {
        if fsync_complete {
            // All changes were written to disk before crash
            // Safe to redo the commit
            redo_commit(&refs)?;
            wal.write_entry(&TransactionLogEntry::Cleanup { id: tx_id })?;
        } else {
            // Crash before durability marker - rollback
            undo_refs(&refs)?;
            wal.write_entry(&TransactionLogEntry::Abort {
                id: tx_id,
                reason: "Crashed before fsync".to_string()
            })?;
        }
    }

    wal.fsync()?;
    Ok(())
}
```

### 4.2 Marker File Cleanup Patterns

**Challenge:** Prevent marker files from accumulating on disk.

**Solution 1: Periodic Cleanup**

```rust
fn cleanup_old_markers() -> Result<()> {
    let now = current_timestamp();
    let max_age_secs = 24 * 60 * 60; // 24 hours

    for entry in fs::read_dir(".jin")? {
        let path = entry?.path();
        if path.ends_with(".transaction_failed") || path.ends_with(".transaction_committed") {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = modified.elapsed() {
                        if age.as_secs() > max_age_secs {
                            fs::remove_file(&path)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
```

**Solution 2: Checkpoint-Based Cleanup**

```rust
fn compact_transaction_log() -> Result<()> {
    // Read all log entries
    let entries = read_all_transaction_log()?;

    // Filter out completed transactions
    let active_entries: Vec<_> = entries
        .iter()
        .filter(|e| !is_completed(e))
        .collect();

    // Write compacted log to temp file
    let temp_log = write_new_log(&active_entries)?;

    // Atomic rename
    fs::rename(temp_log, TRANSACTION_LOG)?;

    Ok(())
}
```

### 4.3 Resuming Interrupted Transactions

**Idempotency Requirement:**

```
Critical Principle: All operations in recovery MUST be idempotent.
If recovery is interrupted before completion, re-running it must
produce the same result, not corrupt data further.
```

**Safe Recovery Pattern:**

```rust
fn resume_interrupted_transaction(tx_id: &str) -> Result<()> {
    // Step 1: Determine current state (idempotent read)
    let state = detect_transaction_state(tx_id)?;

    match state {
        TransactionState::InProgress {
            refs,
            journal_entry
        } => {
            // Check if refs were already persisted
            let persisted = check_persisted_refs(&refs)?;

            if persisted.len() == refs.len() {
                // All updates persisted - safe to commit
                complete_commit(tx_id)?;
            } else if persisted.is_empty() {
                // No updates persisted - safe to rollback
                rollback(tx_id)?;
            } else {
                // Partial update - dangerous, manual intervention
                report_inconsistent_state(tx_id, &persisted)?;
            }
        }

        TransactionState::Committed { .. } => {
            // Already committed - ensure cleanup is complete (idempotent)
            cleanup_commit(tx_id)?;
        }

        TransactionState::RolledBack { .. } => {
            // Already rolled back - ensure cleanup is complete (idempotent)
            cleanup_rollback(tx_id)?;
        }

        TransactionState::NotFound => {
            // Transaction never started - no action needed
            Ok(())
        }
    }
}

// Idempotent operation: can be called multiple times safely
fn complete_commit(tx_id: &str) -> Result<()> {
    // 1. Check if already complete (idempotent guard)
    if is_commit_complete(tx_id)? {
        return Ok(());
    }

    // 2. Update git refs (already persisted, but update indices)
    update_commit_metadata(tx_id)?;

    // 3. Write completion marker
    write_commit_complete_marker(tx_id)?;

    // 4. Remove in-progress marker
    remove_in_progress_marker(tx_id)?;

    Ok(())
}
```

---

## 5. Integrated Transaction System Example

### Architecture

```
Application Layer
    ↓
TransactionManager (high-level API)
    ├── create_transaction()
    ├── execute_operation()
    ├── commit_transaction()
    └── recover_incomplete()
    ↓
WAL (Write-Ahead Log via OkayWAL)
    ├── Log all operations before execution
    ├── Ensure durability with fsync
    └── Support recovery on startup
    ↓
File Operations (Atomic Rename)
    ├── Temp file writes
    ├── Atomic rename on completion
    └── Cleanup on crash
    ↓
Git References (via git update-ref --atomic)
    └── All-or-nothing batch updates
```

### Implementation Sketch

```rust
use okaywal::{WriteAheadLog, LogManager};
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
enum TransactionOp {
    UpdateRef { ref_name: String, old_oid: String, new_oid: String },
    WriteFile { path: String, contents: Vec<u8> },
    DeleteFile { path: String },
}

#[derive(Serialize, Deserialize)]
enum WALEntry {
    TxBegin { id: String, ops: Vec<TransactionOp> },
    OpComplete { id: String, op_index: usize },
    TxFsyncComplete { id: String },
    TxCommit { id: String },
    TxCleanup { id: String },
}

struct TransactionManager {
    wal: WriteAheadLog,
    git_repo: git2::Repository,
}

impl TransactionManager {
    fn new(repo_path: &str) -> Result<Self> {
        let wal = WriteAheadLog::recover(".jin/wal", MyLogManager)?;
        let git_repo = git2::Repository::open(repo_path)?;

        // Recover any interrupted transactions on startup
        Self::recover_incomplete(&wal, &git_repo)?;

        Ok(TransactionManager { wal, git_repo })
    }

    fn begin_transaction(&mut self, id: &str, ops: Vec<TransactionOp>) -> Result<()> {
        // 1. Write to WAL before doing anything
        self.wal.write_entry(&WALEntry::TxBegin {
            id: id.to_string(),
            ops: ops.clone()
        })?;

        // 2. Execute operations with progress markers
        for (i, op) in ops.iter().enumerate() {
            self.execute_operation(op)?;
            self.wal.write_entry(&WALEntry::OpComplete {
                id: id.to_string(),
                op_index: i
            })?;
        }

        // 3. Ensure durability marker before committing
        self.wal.fsync()?;
        self.wal.write_entry(&WALEntry::TxFsyncComplete {
            id: id.to_string()
        })?;

        // 4. Commit using atomic git command
        self.atomic_git_commit(id)?;

        // 5. Mark complete in WAL
        self.wal.write_entry(&WALEntry::TxCommit {
            id: id.to_string()
        })?;

        // 6. Cleanup
        self.wal.write_entry(&WALEntry::TxCleanup {
            id: id.to_string()
        })?;
        self.wal.fsync()?;

        Ok(())
    }

    fn execute_operation(&mut self, op: &TransactionOp) -> Result<()> {
        match op {
            TransactionOp::UpdateRef { ref_name, old_oid, new_oid } => {
                // Prepare for atomic git update-ref
                Ok(())
            }
            TransactionOp::WriteFile { path, contents } => {
                // Atomic write using temp file + rename
                let temp_path = format!("{}.tmp", path);
                fs::write(&temp_path, contents)?;
                fs::rename(&temp_path, path)?;
                Ok(())
            }
            TransactionOp::DeleteFile { path } => {
                fs::remove_file(path)?;
                Ok(())
            }
        }
    }

    fn atomic_git_commit(&self, tx_id: &str) -> Result<()> {
        // Execute git update-ref --atomic --stdin
        // All ref updates succeed or all fail
        Ok(())
    }

    fn recover_incomplete(wal: &WriteAheadLog, repo: &git2::Repository) -> Result<()> {
        // Read WAL and detect incomplete transactions
        // For each incomplete transaction:
        //   1. Check which operations persisted
        //   2. If all persisted: redo commit
        //   3. If partial: rollback
        //   4. Update WAL to mark recovery complete
        Ok(())
    }
}
```

---

## 6. Summary of Key Patterns

### Best Practices

| Pattern | Use Case | Pros | Cons | Example |
|---------|----------|------|------|---------|
| **Two-Phase Commit** | Distributed transactions across multiple systems | True ACID across systems | Complex protocol, latency | Multi-service coordination |
| **Write-Ahead Log** | Durability for single-system transactions | Simple recovery, good performance | Requires cleanup | OkayWAL, sled |
| **Atomic File Rename** | Single file state transitions | Simple, OS-atomic, portable | Single file only | Config files, state files |
| **git update-ref --atomic** | Multiple git reference updates | True all-or-nothing atomicity | subprocess overhead | Batch branch updates |
| **Marker Files** | Transaction state detection | Simple, visible in filesystem | Race conditions | .jin/.transaction_in_progress |

### Critical Rules

1. **Write to WAL before executing** - Never execute without logging first
2. **Fsync after all changes** - Durability requires explicit fsync, not just close
3. **Use atomic operations** - Rename, git update-ref, compare-and-swap
4. **Make recovery idempotent** - Recovery can be interrupted and resumed safely
5. **Test failure scenarios** - Crash before fsync, after fsync, during cleanup
6. **Avoid partial success** - Either all changes succeed or all rollback

### Libraries and Resources

**WAL Implementations:**
- [OkayWAL](https://github.com/khonsulabs/okaywal) - General purpose WAL
- [wral](https://docs.rs/wral/) - Write-ahead logging with rotation
- [sled](https://docs.rs/sled/) - Embedded DB with transactions

**Atomic File Operations:**
- [atomicwrites](https://crates.io/crates/atomicwrites) - Simple atomic writes
- [atomic-write-file](https://docs.rs/atomic-write-file/) - Advanced, directory-aware
- [tempfile](https://docs.rs/tempfile/) - Multi-purpose temp files with persist

**Git Integration:**
- [git2](https://docs.rs/git2/) - Rust bindings to libgit2
- `git` CLI subprocess with `--atomic --stdin` for true atomicity

**Transaction Management:**
- [sled::transaction()](https://docs.rs/sled/latest/sled/) - Serializable ACID transactions
- [sqlx](https://docs.rs/sqlx/) - SQL transactions with auto-rollback
- Custom implementations using WAL patterns

---

## 7. References

### Core Patterns
- [Two-Phase Commit - Martin Fowler](https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html)
- [Introducing OkayWAL](https://bonsaidb.io/blog/introducing-okaywal/)
- [Design and Reliability of a User Space Write-Ahead Log in Rust](https://www.researchgate.net/publication/393784221_Design_and_Reliability_of_a_User_Space_Write-Ahead_Log_in_Rust)

### Git and Transactions
- [git2::Transaction Documentation](https://docs.rs/git2/latest/git2/struct.Transaction.html)
- [libgit2 Transaction API](https://libgit2.org/docs/reference/main/transaction/git_transaction_commit.html)
- [Git update-ref Documentation](https://git-scm.com/docs/git-update-ref)
- [libgit2 Issue #5918 - Transaction Atomicity](https://github.com/libgit2/libgit2/issues/5918)

### File Operations
- [atomicwrites GitHub](https://github.com/untitaker/rust-atomicwrites)
- [atomic-write-file Documentation](https://docs.rs/atomic-write-file/latest/atomic_write_file/)
- [tempfile Documentation](https://docs.rs/tempfile/latest/tempfile/)
- [Rust POSIX Rename Support PR #131072](https://github.com/rust-lang/rust/pull/131072)

### WAL Libraries
- [OkayWAL GitHub](https://github.com/khonsulabs/okaywal)
- [wral - Write-Ahead Logging](https://docs.rs/wral/)
- [simple_wal - Simple WAL](https://docs.rs/simple_wal/)
- [walcraft - Concurrent WAL](https://github.com/RustyFarmer101/walcraft)

### Transaction Systems
- [Sled Documentation](https://docs.rs/sled/)
- [Sled GitHub](https://github.com/spacejam/sled)
- [augrim - 2PC Protocol](https://docs.rs/augrim)

### Implementations
- [dilawarm/two-phase-commit](https://github.com/dilawarm/two-phase-commit) - Full 2PC implementation
- [amirheidarikhoram/2pc-rust-impl](https://github.com/amirheidarikhoram/2pc-rust-impl) - 2PC with TCP
