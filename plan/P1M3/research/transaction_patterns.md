# Transaction Patterns Research for P1.M3

## Executive Summary

This research explores patterns for implementing atomic multi-layer commits with failure recovery in Jin. The key finding is that `git2::Transaction` is **not truly atomic** - partial failures don't rollback previous updates. Jin requires a higher-level transaction system using a two-phase commit pattern with persistent transaction logs.

---

## 1. git2::Transaction Limitations

### Current Implementation Analysis

From `src/git/transaction.rs`:
```rust
/// Note that `git2::Transaction` is **NOT truly atomic**. If one ref update fails,
/// previous successful updates are NOT rolled back. For critical operations that
/// require true atomicity, consider external locking mechanisms.
```

### What git2::Transaction Provides

```rust
// Lock refs for update
tx.lock_ref("refs/jin/layers/mode/claude")?;
tx.lock_ref("refs/jin/layers/project/my-app")?;

// Queue target changes
tx.set_target("refs/jin/layers/mode/claude", commit1, "message")?;
tx.set_target("refs/jin/layers/project/my-app", commit2, "message")?;

// Apply updates
tx.commit()?;
```

**Guarantees:**
- Refs are locked before update (prevents concurrent modification)
- Reflog entries are created
- Updates are applied in sequence

**Limitations:**
- If update 2 fails after update 1 succeeds, update 1 is **not** rolled back
- No persistent state for crash recovery
- No built-in retry mechanism

### Alternative: git update-ref --atomic

Git provides true atomic updates via plumbing command:

```bash
git update-ref --stdin --atomic << EOF
start
update refs/jin/layers/mode/claude abc123 def456
update refs/jin/layers/project/my-app ghi789 jkl012
prepare
commit
EOF
```

However, calling external git commands from Rust adds complexity and loses type safety.

---

## 2. Two-Phase Commit Pattern

### Overview

A two-phase commit (2PC) ensures atomicity by:
1. **Prepare Phase**: Record intent, validate all operations possible
2. **Commit Phase**: Execute operations, mark as complete

### Jin Transaction States

```
PENDING → PREPARED → COMMITTED
              ↓
           ABORTED
```

- **PENDING**: Transaction created, updates being queued
- **PREPARED**: All updates validated, ready to commit (point of no return)
- **COMMITTED**: All updates applied successfully
- **ABORTED**: Transaction rolled back

### Transaction Log Design

```json
{
  "version": 1,
  "id": "20251226143022123456",
  "state": "prepared",
  "started_at": "2025-12-26T14:30:22.123456Z",
  "message": "Update mode and project layers",
  "updates": [
    {
      "layer": "mode_base",
      "mode": "claude",
      "scope": null,
      "project": null,
      "ref_path": "refs/jin/layers/mode/claude",
      "old_oid": "abc123def456...",
      "new_oid": "789012345678..."
    }
  ]
}
```

---

## 3. Atomic File Operations

### POSIX Atomic Rename

On POSIX systems, `rename()` is atomic within the same filesystem:
- Either the rename completes entirely, or it doesn't happen at all
- If system crashes during rename, only one file exists (old or new)

### Pattern: Write-Rename

```rust
fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;  // Write to temp file
    std::fs::rename(&temp_path, path)?;     // Atomic rename
    Ok(())
}
```

### Rust Crate: tempfile

The `tempfile` crate provides `NamedTempFile::persist()` which uses atomic rename:

```rust
use tempfile::NamedTempFile;

let mut file = NamedTempFile::new()?;
write!(file, "{}", content)?;
file.persist(final_path)?;  // Atomic rename
```

---

## 4. Recovery Detection

### Strategy: Check on Startup

Every jin command should check for incomplete transactions before proceeding:

```rust
pub fn run(cli: Cli) -> Result<()> {
    // Check for incomplete transaction
    if let Some(incomplete) = RecoveryManager::detect()? {
        // Options:
        // 1. Auto-rollback (safest)
        // 2. Auto-resume (if prepared)
        // 3. Prompt user
        incomplete.rollback()?;
    }

    commands::execute(cli)
}
```

### Recovery Decisions by State

| State | Action | Rationale |
|-------|--------|-----------|
| PENDING | Rollback | No refs modified yet |
| PREPARED | Resume or Rollback | Updates were in progress |
| COMMITTED | Clean up log | Updates succeeded, log is stale |
| ABORTED | Clean up log | Already rolled back |

### Handling PREPARED State

When transaction is PREPARED, updates may be partially applied:
- **Resume**: Re-run all updates (must be idempotent)
- **Rollback**: Restore old_oid for each update

Recommendation: Default to rollback for safety, allow resume as option.

---

## 5. Idempotent Operations

### Why Idempotency Matters

If a transaction is interrupted and resumed, operations may run twice:
```
First attempt: update ref A, update ref B (crash!)
Second attempt: update ref A, update ref B (success)
```

Ref update with same OID is idempotent - safe to retry.

### Ensuring Idempotency

```rust
fn set_ref_idempotent(name: &str, oid: Oid, message: &str) -> Result<()> {
    // Check if ref already has target value
    if self.ref_exists(name) && self.resolve_ref(name)? == oid {
        return Ok(());  // Already at target, no-op
    }
    self.repo.reference(name, oid, true, message)?;
    Ok(())
}
```

---

## 6. Comparison with Alternatives

### Option A: git2::Transaction (Current)

**Pros:**
- Simple API
- Locks refs during update

**Cons:**
- Not truly atomic
- No crash recovery

### Option B: External git update-ref

**Pros:**
- True atomic semantics
- Well-tested

**Cons:**
- External process overhead
- Loses type safety
- Parsing complexity

### Option C: Two-Phase Commit with Log (Recommended)

**Pros:**
- True atomicity via rollback
- Crash recovery via persistent log
- Stays within Rust/git2 ecosystem
- Full control over behavior

**Cons:**
- More complex implementation
- Additional file I/O

---

## 7. Implementation Recommendations

### Transaction Log Location

```
.jin/.transaction_in_progress
```

Per-project, not global (~/.jin/), because:
- Transactions are project-specific
- Multiple projects shouldn't interfere
- Matches Jin's per-project .jin/ directory pattern

### Error Handling Strategy

```rust
match tx.commit() {
    Ok(()) => println!("Committed successfully"),
    Err(e) => {
        eprintln!("Transaction failed: {}", e);
        // Rollback already attempted internally
        // User sees clean state
    }
}
```

### Concurrency Protection

```rust
pub fn begin(repo: &JinRepo, message: &str) -> Result<LayerTransaction> {
    // Check for existing transaction
    if TransactionLog::load()?.is_some() {
        return Err(JinError::Transaction(
            "Another transaction in progress".to_string()
        ));
    }
    // Proceed...
}
```

Note: File-based locking doesn't protect against race conditions between check and create. For full safety, use file locking (flock) or atomic create.

---

## 8. References

- git2-rs Transaction: https://docs.rs/git2/latest/git2/struct.Transaction.html
- git update-ref: https://git-scm.com/docs/git-update-ref
- Two-Phase Commit: https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html
- tempfile crate: https://docs.rs/tempfile/latest/tempfile/
- Plan P1M2 research: plan/P1M2/research/phantom_git_patterns.md

---

Generated: December 26, 2025
For Jin Transaction System (P1.M3)
