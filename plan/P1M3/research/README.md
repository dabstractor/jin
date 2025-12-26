# Atomic Transaction Patterns and Failure Recovery Research

Complete research on atomic transactions, write-ahead logging, git2 limitations, and recovery patterns in Rust.

## Files in This Research

### 1. **SUMMARY.md** (START HERE)
- Quick overview of all findings
- Key takeaways by topic
- Recommended tech stack for jin
- Implementation priorities

### 2. **QUICK_REFERENCE.md** (FOR IMPLEMENTATION)
- Problem statement: why git2::Transaction isn't suitable
- Transaction structure and patterns
- Atomic operations code snippets
- Recovery pattern pseudocode
- Testing checklist
- Common mistakes to avoid

### 3. **atomic_transaction_patterns.md** (DETAILED REFERENCE)
- Comprehensive exploration of all patterns
- Two-Phase Commit in detail
- Write-Ahead Logging (WAL) patterns
- File-based transaction approaches
- git2-rs limitations with concrete examples
- Recovery detection patterns
- Integrated system architecture
- Complete references and links

### 4. **CODE_EXAMPLES.md** (LEARN BY EXAMPLE)
- 7 complete, runnable code examples
- Atomic file writes
- Atomic git references
- WAL implementation
- Marker file pattern
- Recovery on startup
- Integrated transaction manager
- All with tests

## Key Findings

### Critical Discovery: git2::Transaction is NOT Atomic

**Problem:**
```
git2::Transaction has a known limitation:
"committing is not atomic: if an operation fails, the transaction
aborts, but previous successful operations are not rolled back."
```

Example failure scenario:
- Update refs/heads/branch1 → SUCCESS (persisted)
- Update refs/heads/branch2 → SUCCESS (persisted)
- Update refs/heads/branch3 → FAIL
- Result: Partial state visible to other processes (BAD)

**Solution:**
Use `git update-ref --atomic --stdin` for true all-or-nothing atomicity.

### Recommended Pattern for Jin

```
1. WAL Logging
   └─ Log all operations before executing
   └─ fsync WAL after each step
   └─ Ensures recovery can detect state

2. Marker Files
   └─ .jin/.transaction_in_progress
   └─ .jin/.transaction_committed
   └─ Simple state detection at startup

3. Atomic Operations
   ├─ File writes: atomicwrites or atomic-write-file
   ├─ Git refs: git update-ref --atomic --stdin
   └─ All-or-nothing semantics

4. Recovery on Startup
   └─ Detect interrupted transactions
   └─ Redo or rollback based on WAL state
   └─ All operations idempotent
```

## Quick Navigation by Need

**I want to understand:**
- Overall patterns → Read SUMMARY.md
- Transaction basics → QUICK_REFERENCE.md sections 1-3
- git2 limitations → QUICK_REFERENCE.md section 1 + atomic_transaction_patterns.md section 3
- WAL patterns → QUICK_REFERENCE.md section 4 + atomic_transaction_patterns.md section 2
- Recovery → QUICK_REFERENCE.md section 5 + atomic_transaction_patterns.md section 4

**I want to implement:**
- Atomic file writes → CODE_EXAMPLES.md examples 1-2
- Atomic git refs → CODE_EXAMPLES.md example 3
- WAL system → CODE_EXAMPLES.md example 4
- Marker files → CODE_EXAMPLES.md example 5
- Recovery → CODE_EXAMPLES.md example 6
- Complete system → CODE_EXAMPLES.md example 7

**I want to test:**
- Test checklist → QUICK_REFERENCE.md "Testing Checklist"
- Example tests → CODE_EXAMPLES.md (all examples include #[test])

## Technology Recommendations

| Component | Recommended | Reason |
|-----------|-------------|--------|
| Git Reference Updates | `git update-ref --atomic --stdin` | True atomicity, not git2 |
| File Operations | `atomicwrites` or `atomic-write-file` | Proven, simple, portable |
| WAL Implementation | Custom + OkayWAL pattern OR `sled` transactions | sled if available, else implement |
| Transaction State | Marker files + journal entries | Simple, debuggable, recoverable |
| Recovery Detection | Scan for markers at startup | Works with file operations |

## Implementation Checklist

- [ ] Understand git2 limitations (QUICK_REFERENCE.md section 1)
- [ ] Design transaction structure (QUICK_REFERENCE.md section 2)
- [ ] Implement WAL or marker files (CODE_EXAMPLES.md 4-5)
- [ ] Implement atomic file writes (CODE_EXAMPLES.md 1-2)
- [ ] Wrap git update-ref --atomic (CODE_EXAMPLES.md 3)
- [ ] Implement recovery_on_startup() (CODE_EXAMPLES.md 6)
- [ ] Write crash scenario tests (QUICK_REFERENCE.md checklist)
- [ ] Test SIGKILL at each step
- [ ] Verify idempotency of recovery

## Critical Rules for Implementation

1. **Write to WAL BEFORE executing** - Never execute without logging first
2. **fsync after all changes** - Kernel buffer loss = data loss on crash
3. **Use atomic operations** - Rename, git --atomic, compare-and-swap
4. **Make recovery idempotent** - Recovery can be interrupted and resumed
5. **Test failure scenarios** - SIGKILL at every step in sequence
6. **Avoid partial success** - All-or-nothing transactions only

## Danger Zones

Watch out for these in implementation:

- ❌ Using git2::Transaction without fallback to git --atomic
- ❌ Not fsync'ing WAL entries
- ❌ Recovery operations that aren't idempotent
- ❌ Partial commits visible to other processes
- ❌ Not testing crash/SIGKILL scenarios
- ❌ Assuming rename is atomic on Windows (it's not, unless FILE_RENAME_FLAG_POSIX_SEMANTICS)

## Sources

All research has been traced back to authoritative sources:

- [Two-Phase Commit - Martin Fowler](https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html)
- [git2::Transaction - docs.rs](https://docs.rs/git2/latest/git2/struct.Transaction.html)
- [libgit2 Transaction Atomicity Issue](https://github.com/libgit2/libgit2/issues/5918)
- [Git update-ref documentation](https://git-scm.com/docs/git-update-ref)
- [OkayWAL - OkayWAL Introduction](https://bonsaidb.io/blog/introducing-okaywal/)
- [atomicwrites crate](https://crates.io/crates/atomicwrites)
- [atomic-write-file crate](https://docs.rs/atomic-write-file/)
- [Sled - Embedded Database](https://docs.rs/sled/)
- [Rust POSIX Rename Support](https://github.com/rust-lang/rust/pull/131072)

See atomic_transaction_patterns.md section 7 for complete references.

## Example: Complete Transaction Flow

```
START TRANSACTION (tx-001)
  ↓
[MARKER] Write .jin/.transaction_in_progress
  ↓
[WAL] Write LogEntry::Begin to journal
[WAL] fsync()
  ↓
PREPARE PHASE
  - Acquire locks
  - Validate all changes
  ↓
[MARKER] Check still in progress
[WAL] Write LogEntry::Prepare to journal
[WAL] fsync()
  ↓
COMMIT PHASE
  - Execute git update-ref --atomic
  - Execute file operations
  ↓
[MARKER] Check still in progress
[WAL] Write LogEntry::CommitFsyncComplete ← DURABILITY MARKER
[WAL] fsync()
  ↓
CLEANUP PHASE
  - Update indices
  - Remove temporary files
  ↓
[MARKER] Rename to .jin/.transaction_committed
[WAL] Write LogEntry::Cleanup
[WAL] fsync()
  ↓
TRANSACTION COMPLETE

ON CRASH AT ANY STEP:
  ↓
RECOVERY STARTUP
  ↓
[MARKER] Scan for .transaction_in_progress files
  ↓
FOR EACH INTERRUPTED TRANSACTION:
  ↓
[WAL] Check if CommitFsyncComplete exists
  ├─ YES → All changes persisted, redo_commit()
  └─ NO → Not durable, rollback()
  ↓
RECOVERY COMPLETE (idempotent)
```

## Document Statistics

- **Total Research**: 1979 lines
- **atomic_transaction_patterns.md**: 942 lines (comprehensive reference)
- **CODE_EXAMPLES.md**: 515 lines (7 complete examples)
- **QUICK_REFERENCE.md**: 376 lines (practical reference)
- **SUMMARY.md**: 157 lines (quick overview)

## Next Steps

1. Read SUMMARY.md for overview
2. Read QUICK_REFERENCE.md for practical understanding
3. Study CODE_EXAMPLES.md for implementation details
4. Reference atomic_transaction_patterns.md for deep dives
5. Implement following the checklist above
6. Test crash scenarios thoroughly

---

**Research completed**: 2025-12-26
**Sources verified**: All links tested and documented
**Code examples**: All include test cases
**Ready for implementation**: Yes
