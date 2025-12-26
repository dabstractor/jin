# START HERE: Atomic Transaction Research

Welcome! You've found comprehensive research on atomic transactions and failure recovery in Rust.

## In 30 Seconds

**Problem:** git2::Transaction updates are not atomic on failure. Some refs update, others don't.

**Solution:** Use `git update-ref --atomic --stdin` for true all-or-nothing atomicity.

**Pattern:** WAL + Marker Files + Atomic Ops + Recovery

## What You'll Find Here

Six documents totaling 2,775 lines covering:
- Why git2 doesn't work for atomic transactions
- How to implement proper atomic transactions
- Write-Ahead Logging (WAL) for durability
- Recovery after crashes
- Complete working code examples
- Visual diagrams explaining everything

## Pick Your Path

### Path 1: "I just want the quick answer" (5 min)
1. Read this file
2. Skim QUICK_REFERENCE.md "Problem" section
3. Look at CODE_EXAMPLES.md example 3 (git update-ref)
4. You're done

### Path 2: "I need to implement this" (1 hour)
1. Read SUMMARY.md (overview of all topics)
2. Read QUICK_REFERENCE.md (practical patterns)
3. Study CODE_EXAMPLES.md (working code)
4. Reference DIAGRAMS.md when confused
5. Check atomic_transaction_patterns.md for deep questions

### Path 3: "I want to understand everything" (2 hours)
1. Read README.md (navigation guide)
2. Read SUMMARY.md (overview)
3. Study QUICK_REFERENCE.md (patterns)
4. Read CODE_EXAMPLES.md (implementations)
5. Study DIAGRAMS.md (visualizations)
6. Deep-dive into atomic_transaction_patterns.md (comprehensive)

## File Quick Map

```
README.md                              ← Navigation & checklist
    ↓
SUMMARY.md                             ← Quick overview
    ↓
QUICK_REFERENCE.md ← ← ← ← ← ← ← ← ← START HERE for implementation
    ↓
CODE_EXAMPLES.md (7 examples)          ← Copy-paste code
    ↓
DIAGRAMS.md (7 diagrams)               ← Visual understanding
    ↓
atomic_transaction_patterns.md         ← Deep reference
```

## The Problem (git2::Transaction)

```rust
// WRONG - Partial state on failure
let mut tx = repo.transaction()?;
tx.set_target("refs/heads/main", oid1)?;    // ✓ Persisted
tx.set_target("refs/heads/feature", oid2)?; // ✓ Persisted
tx.set_target("refs/heads/bugfix", oid3)?;  // ✗ Fails
tx.commit()?;
// RESULT: main and feature updated, bugfix not → PARTIAL STATE!
```

## The Solution (git update-ref --atomic)

```bash
# RIGHT - All or nothing atomicity
git update-ref --atomic --stdin << EOF
update refs/heads/main old new1
update refs/heads/feature old new2
update refs/heads/bugfix old new3
EOF
# RESULT: All succeed OR all fail, never partial
```

## The Complete Pattern

### 1. Log Before Executing
```
Write operation to WAL (journal)
    ↓
fsync WAL (make durable)
    ↓
Execute operation
    ↓
Mark completion in WAL
    ↓
fsync WAL again
```

### 2. State Detection
```
Marker files show transaction state:
.jin/.transaction_in_progress   → Still running
.jin/.transaction_committed     → Cleanup pending
.jin/.transaction_failed        → Cleanup pending
```

### 3. Recovery on Crash
```
Startup:
    ↓
Scan for marker files
    ↓
Check WAL for incomplete transactions
    ↓
If all changes persisted → redo_commit()
If not persisted → rollback()
    ↓
Application ready
```

## Key Code Patterns

### Atomic Git References
```rust
use std::process::{Command, Stdio};

Command::new("git")
    .arg("-C").arg(repo_path)
    .arg("update-ref")
    .arg("--atomic")
    .arg("--stdin")
    .stdin(Stdio::piped())
    .spawn()?
```

### Atomic File Writes
```rust
use atomicwrites::{AtomicFile, DisallowOverwrite};

let af = AtomicFile::new("target.txt", DisallowOverwrite);
af.write(|f| {
    f.write_all(b"data")
})?;
```

### Transaction Markers
```
.jin/.transaction_in_progress
├── id: "tx-001"
├── timestamp: 1703123456
├── operation: "update-refs"
└── pid: 12345
```

## Why This Matters

Without proper transactions, these scenarios are possible:

❌ **Scenario 1: Partial git refs**
- Process updates branch1 successfully
- Process crashes
- Process updates branch2 successfully
- Results: Inconsistent repository state
- Recovery: Can't tell what succeeded

❌ **Scenario 2: Orphaned files**
- File write partially complete
- Process crashes
- Results: Corrupted or incomplete files
- Recovery: Must clean up or detect

✅ **Scenario 3: With proper transactions**
- All changes logged and durable
- All git refs updated atomically
- All files written atomically
- Results: Consistent state even after crash
- Recovery: Detect incomplete tx, redo/rollback safely

## Testing Your Implementation

You MUST test crash scenarios:

```
Test at each step:
├─ Crash before WAL write
├─ Crash after WAL write, before fsync
├─ Crash after WAL fsync, before execute
├─ Crash after execute, before cleanup
└─ Crash during recovery

Use SIGKILL to simulate crashes (not just close)
```

## Implementation Checklist

- [ ] Replace git2::Transaction with git update-ref --atomic
- [ ] Add WAL or transaction journal
- [ ] Add marker files for state tracking
- [ ] Implement recovery_on_startup()
- [ ] Make all recovery operations idempotent
- [ ] Test SIGKILL at every step
- [ ] Verify no partial state visible
- [ ] Benchmark performance impact
- [ ] Document transaction flow

## Next Steps

1. **For quick implementation:** Go to CODE_EXAMPLES.md
2. **For understanding patterns:** Go to QUICK_REFERENCE.md
3. **For diagrams:** Go to DIAGRAMS.md
4. **For deep reference:** Go to atomic_transaction_patterns.md

## Critical Rules

1. **Write to WAL BEFORE executing** - Always log first
2. **fsync after logging** - Kernel buffer loss = data loss
3. **Use atomic operations** - All-or-nothing only
4. **Make recovery idempotent** - Can retry safely
5. **Test crash scenarios** - SIGKILL everywhere
6. **No partial success** - Either all or none

## Key Resources

- [Two-Phase Commit - Martin Fowler](https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html)
- [git update-ref documentation](https://git-scm.com/docs/git-update-ref)
- [git2::Transaction limitations](https://github.com/libgit2/libgit2/issues/5918)
- [atomicwrites crate](https://crates.io/crates/atomicwrites)
- [OkayWAL blog post](https://bonsaidb.io/blog/introducing-okaywal/)

## Questions?

- **"Why not use git2::Transaction?"** → See QUICK_REFERENCE.md section 1
- **"How do I write atomically?"** → See CODE_EXAMPLES.md examples 1-2
- **"How do I handle git refs?"** → See CODE_EXAMPLES.md example 3
- **"How do I recover from crashes?"** → See CODE_EXAMPLES.md example 6
- **"What does it look like when..."** → See DIAGRAMS.md

## Time Investment vs Benefit

- **Read time:** ~100 minutes (all documents)
- **Implementation time:** 1-2 weeks (depending on complexity)
- **Benefit:** No more corrupted repository state
- **ROI:** Prevents data loss, improves reliability

## You're Ready!

This research gives you:
✓ Understanding of atomic transactions
✓ Complete code examples (copy-paste ready)
✓ Visual diagrams (for understanding)
✓ Recovery patterns (for crash handling)
✓ Testing guidelines (for verification)
✓ Technology recommendations (for your stack)

**Choose your path above and start reading!**

---

All files are in: `/home/dustin/projects/jin/plan/P1M3/research/`

Questions? Check README.md for full navigation.

Happy implementing!
