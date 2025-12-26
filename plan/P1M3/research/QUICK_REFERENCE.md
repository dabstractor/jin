# Quick Reference: Atomic Transaction Patterns

## Problem: git2::Transaction is NOT Atomic on Failure

```
❌ WRONG - Using git2::Transaction
let mut tx = repo.transaction()?;
tx.lock_ref("refs/heads/branch1")?;
tx.set_target(oid1)?;           // ✓ Persisted if crash
tx.lock_ref("refs/heads/branch2")?;
tx.set_target(oid2)?;           // ✓ Persisted if crash
tx.lock_ref("refs/heads/branch3")?;
tx.set_target(oid3)?;           // ✗ Fails
tx.commit()?;
// RESULT: branch1 and branch2 updated, branch3 not → PARTIAL STATE

✓ RIGHT - Using git update-ref --atomic --stdin
git update-ref --atomic --stdin << EOF
update refs/heads/branch1 oldoid1 newoid1
update refs/heads/branch2 oldoid2 newoid2
update refs/heads/branch3 oldoid3 newoid3
EOF
// RESULT: All succeed or ALL fail → ATOMIC
```

## Transaction Structure

### 1. WAL Entry (Before Any Action)
```rust
enum WALEntry {
    Begin { id: String, ops: Vec<Operation> },
    OpComplete { id: String, op_index: usize },
    FsyncComplete { id: String },  // ← Durability marker
    Commit { id: String },
    Cleanup { id: String },
}

// Write to WAL with fsync BEFORE committing
wal.write_entry(&WALEntry::Begin { ... })?;
wal.fsync()?;  // CRITICAL: Ensure durability
```

### 2. Marker Files (State Detection)
```
.jin/.transaction_in_progress   → Transaction active
.jin/.transaction_committed     → Transaction done (for cleanup)
.jin/.transaction_failed        → Transaction failed (for cleanup)

Content: { id, timestamp, operation_type, (optional) last_op_index }
```

### 3. Atomic Operations

#### File Write
```rust
use atomicwrites::{AtomicFile, DisallowOverwrite};

// Atomic write to disk
let af = AtomicFile::new("target.txt", DisallowOverwrite);
af.write(|f| {
    f.write_all(b"atomic content")
})?;
// File either has new content or old content, never partial
```

#### Git References
```rust
use std::process::{Command, Stdio};

// ALL-OR-NOTHING atomic batch update
let mut child = Command::new("git")
    .arg("-C").arg(repo_path)
    .arg("update-ref")
    .arg("--atomic")
    .arg("--stdin")
    .stdin(Stdio::piped())
    .spawn()?;

let mut stdin = child.stdin.take().unwrap();
stdin.write_all(b"update refs/heads/main old new\n")?;
// ... more updates
drop(stdin);

let status = child.wait()?;
assert!(status.success(), "Atomic update failed - NO changes applied");
```

## Recovery Pattern

### On Startup
```rust
fn recovery_on_startup() {
    // 1. Scan for incomplete transactions
    for marker in find_marker_files(".jin") {
        let tx_id = marker.transaction_id;

        // 2. Check if marker is stale (process crashed)
        if is_stale(&marker) && !process_is_running(marker.pid) {
            // 3. Determine state from WAL
            let state = determine_transaction_state(tx_id)?;

            match state {
                State::FsyncComplete => {
                    // All changes persisted before crash
                    // Safe to complete commit
                    redo_commit(tx_id)?;
                }
                State::PreFsync => {
                    // Crash before durability marker
                    // Safe to rollback
                    rollback(tx_id)?;
                }
                State::PartialCommit => {
                    // DANGER: Some refs persisted, others not
                    // Requires manual intervention
                    panic!("Inconsistent transaction state");
                }
            }

            // 4. Mark recovery complete (idempotent)
            write_cleanup_marker(tx_id)?;
        }
    }
}
```

## Write-Ahead Log (WAL) Pattern

### Basic Structure
```
1. Write operation to WAL
2. fsync WAL
3. Execute operation
4. Write completion to WAL
5. fsync WAL again
6. Mark transaction done
7. fsync final state

Key: EVERY state change must be written and fsync'd
     before the next operation begins
```

### Using OkayWAL
```rust
use okaywal::WriteAheadLog;

let wal = WriteAheadLog::recover("./wal", log_manager)?;

// Log the operation
wal.write_entry(&Operation::UpdateRef { ... })?;

// Ensure durability BEFORE executing
wal.fsync()?;

// Now safe to execute - even if crash, recovery knows
git_update_ref(...)?;

// Log completion
wal.write_entry(&Operation::Complete { ... })?;
wal.fsync()?;
```

## Idempotency Checklist

All recovery operations MUST be idempotent:
- [ ] Can call redo_commit() multiple times safely
- [ ] Can call rollback() multiple times safely
- [ ] Cleanup operations complete even if interrupted
- [ ] No data lost or corrupted by re-running recovery
- [ ] Recovery can be interrupted and resumed

### Example Idempotent Operation
```rust
fn redo_commit_idempotent(tx_id: &str) -> Result<()> {
    // Guard: check if already done
    if is_commit_already_complete(tx_id)? {
        return Ok(());  // Already done, return safely
    }

    // Execute commit (refs already persisted on disk)
    git_update_ref(...)?;

    // Guard: write completion marker
    write_commit_complete_marker(tx_id)?;

    // Guard: remove in-progress marker
    remove_in_progress_marker(tx_id)?;

    Ok(())
}
```

## Testing Checklist

Test failure at EVERY step:

```rust
#[test]
fn crash_before_wal_write() {
    // SIGKILL before: wal.write_entry()
    // Expected: Transaction never started, nothing persisted
    // Recovery: Should see nothing to recover
}

#[test]
fn crash_after_wal_write_before_fsync() {
    // SIGKILL between: wal.write_entry() and wal.fsync()
    // Expected: WAL entry not durable, will be lost
    // Recovery: Should rollback
}

#[test]
fn crash_after_wal_fsync_before_execute() {
    // SIGKILL between: wal.fsync() and git_update_ref()
    // Expected: WAL durable, refs not changed
    // Recovery: Should rollback
}

#[test]
fn crash_after_execute_before_fsync() {
    // SIGKILL between: git_update_ref() and wal.fsync()
    // Expected: Refs changed on disk, WAL not durable
    // Recovery: Should redo commit
}

#[test]
fn crash_during_recovery() {
    // SIGKILL while recovery_on_startup() running
    // Expected: Recovery can be interrupted and resumed
    // Recovery: Re-run recovery, same result
}
```

## Performance Tips

1. **Batch WAL writes** - Write multiple operations, single fsync
2. **Pre-allocate segments** - OkayWAL pre-allocates files
3. **Checkpoint regularly** - Reclaim WAL space after cleanup
4. **Use git --atomic --stdin** - Subprocess overhead is small vs atomicity
5. **Avoid WAL on hot path** - Recovery is async, don't block on it

## Key Metrics

- OkayWAL: ~1ms per 256-byte entry (fsync included)
- git update-ref: ~10-50ms for multiple refs (subprocess overhead)
- tempfile::persist: ~1ms for atomic rename
- Marker file detection: O(n) where n = active transactions

---

## Common Mistakes

❌ **Using git2::Transaction without fallback**
- It's not atomic on failure
- Use git update-ref --atomic instead

❌ **Not fsync'ing WAL entries**
- Kernel buffer loss = data loss on crash
- Always fsync before executing operations

❌ **Recovery operations not idempotent**
- If recovery interrupted, re-running corrupts data
- Every operation must be safe to repeat

❌ **Partial persists visible to other processes**
- If crash after some refs written, other processes see inconsistent state
- Use atomic operations (all-or-nothing)

❌ **Not testing crash scenarios**
- Works fine when nothing crashes
- Must test SIGKILL at every step

---

## Resources

| Need | Solution | Link |
|------|----------|------|
| True atomic git refs | git update-ref --atomic | [docs](https://git-scm.com/docs/git-update-ref) |
| Atomic file writes | atomicwrites or atomic-write-file | [crates.io](https://crates.io/crates/atomicwrites) |
| WAL implementation | OkayWAL | [github](https://github.com/khonsulabs/okaywal) |
| Transaction understanding | 2PC pattern | [Martin Fowler](https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html) |
| git2 limitations | Issue #5918 | [libgit2 github](https://github.com/libgit2/libgit2/issues/5918) |
