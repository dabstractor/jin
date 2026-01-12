# Research Summary: Atomic Transaction Patterns in Rust

## Document Location
`/home/dustin/projects/jin/plan/P1M3/research/atomic_transaction_patterns.md`

## Key Findings

### 1. Two-Phase Commit (2PC)
- **Best for:** Distributed transactions across multiple systems
- **Key Pattern:** Write-Ahead Log (WAL) for durability
- **Recovery:** Replay log on startup to detect incomplete transactions
- **Rust Implementations:**
  - [dilawarm/two-phase-commit](https://github.com/dilawarm/two-phase-commit)
  - [amirheidarikhoram/2pc-rust-impl](https://github.com/amirheidarikhoram/2pc-rust-impl)
  - [augrim crate](https://docs.rs/augrim)

### 2. Write-Ahead Logging (WAL)
- **OkayWAL** - General purpose, segment-based, v0.2 (experimental)
  - Performance: ~1ms per 256-byte entry
  - Pre-allocated segments reduce allocation overhead
  - Automatic checkpointing for space reclamation
  - Source: [bonsaidb.io/blog/introducing-okaywal/](https://bonsaidb.io/blog/introducing-okaywal/)

- **Alternative WAL Libraries:**
  - wral - Append-only with rotation
  - simple_wal - Scans log on startup to detect interrupts
  - walcraft - Concurrent environments with in-memory buffer

### 3. Atomic File Operations
- **POSIX rename is atomic** - On Unix, rename(temp, target) is guaranteed atomic
- **Crates:**
  - [atomicwrites](https://crates.io/crates/atomicwrites) - Simple, two modes
  - [atomic-write-file](https://docs.rs/atomic-write-file/) - Directory-aware, all platforms
  - [tempfile](https://docs.rs/tempfile/) - General purpose with persist()

- **Pattern:**
  1. Write to temporary file in same directory
  2. fsync temporary file
  3. Atomic rename to target
  4. fsync directory (metadata durability)
  5. On crash: temp file left behind, target unchanged (safe)

- **Windows (10+):** Now supports POSIX semantics via FILE_RENAME_FLAG_POSIX_SEMANTICS

### 4. git2-rs Limitations - CRITICAL FINDING

**Problem:** git2 transactions are NOT truly atomic on failure

```
"committing is not atomic: if an operation fails, the transaction
aborts, but previous successful operations are not rolled back."
```

**Failure Scenario Example:**
- Update refs/heads/branch1 ✓
- Update refs/heads/branch2 ✓
- Update refs/heads/branch3 ✗ (fails)
- **Result:** Partial state visible to other processes (BAD)

**Also:** No API to determine which updates succeeded/failed

**Solution:** Use `git update-ref --atomic --stdin`
- True all-or-nothing semantics
- All updates succeed or NONE are applied
- Reference: [git-scm.com/docs/git-update-ref](https://git-scm.com/docs/git-update-ref)

**Rust Implementation:**
```rust
git update-ref --atomic --stdin << EOF
create refs/heads/new-branch abc123...
update refs/heads/main old new
delete refs/heads/old-branch
EOF
```

### 5. Recovery Detection Patterns

#### A. Marker File Approach
```
States:
├── .jin/.transaction_in_progress   → In progress
├── .jin/.transaction_committed     → Committed
└── .jin/.transaction_failed        → Failed
```

**Detection Logic:**
1. Scan for marker files at startup
2. Check if process still holds lock (using flock)
3. If stale (older than threshold), transaction is interrupted
4. Resume or rollback based on transaction state

#### B. Journal Entry Approach
Log each operation with markers:
```
BEGIN → REF_UPDATE → REF_UPDATE → COMMIT_FSYNC_COMPLETE → CLEANUP
```

On recovery:
- If COMMIT_FSYNC_COMPLETE exists: refs persisted, redo commit
- If missing: didn't reach durability, rollback
- Idempotent: can retry recovery safely

### 6. Integrated Example Architecture

```
Application
    ↓
TransactionManager
    ├── WAL logging (OkayWAL)
    ├── File operations (atomic rename)
    ├── Git refs (git update-ref --atomic)
    └── Recovery on startup
    ↓
Recovery Detection
    ├── Marker files or journal
    ├── Detect crashed state
    └── Resume/rollback safely
```

## Critical Rules for Implementation

1. **Write to WAL BEFORE executing** - Never execute without logging first
2. **fsync after all changes** - Durability requires explicit fsync, not just close
3. **Use atomic operations** - Rename, git update-ref, compare-and-swap
4. **Make recovery idempotent** - Recovery can be interrupted and resumed
5. **Test failure scenarios** - Crash at every point in sequence
6. **Avoid partial success** - All-or-nothing transactions only

## Recommended Tech Stack for Jin

Based on research findings:

| Component | Recommendation | Reason |
|-----------|---|---|
| **Transaction Coordination** | git update-ref --atomic --stdin | True atomicity (not git2) |
| **Durability Log** | OkayWAL or custom WAL | fsync batching, recovery |
| **File Operations** | atomicwrites or atomic-write-file | Simple, tested, portable |
| **Recovery Detection** | Marker files + journal | Simple, visible, debuggable |
| **Idempotency** | Journaling all operations | Safe restart on failure |

## Implementation Priorities

1. **Highest:** Implement proper git transaction wrapper using `git update-ref --atomic --stdin`
2. **High:** Add WAL/journal for operation tracking
3. **High:** Implement marker file detection at startup
4. **Medium:** Add recovery procedures (resume/rollback)
5. **Medium:** Test crash scenarios (SIGKILL at each step)
6. **Low:** Optimize WAL with checkpointing

## Key Sources

- [Two-Phase Commit - Martin Fowler](https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html)
- [git2::Transaction docs](https://docs.rs/git2/latest/git2/struct.Transaction.html)
- [OkayWAL blog](https://bonsaidb.io/blog/introducing-okaywal/)
- [Git update-ref documentation](https://git-scm.com/docs/git-update-ref)
- [atomicwrites crate](https://crates.io/crates/atomicwrites)
- [libgit2 Issue #5918](https://github.com/libgit2/libgit2/issues/5918)
