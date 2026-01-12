# Transaction Patterns - Visual Diagrams

## 1. Two-Phase Commit Flow

```
TRANSACTION COORDINATOR
    │
    ├──────────────────────────────────────────────┐
    │                                               │
    ▼                                               ▼
PARTICIPANT A                               PARTICIPANT B
(Database Node)                             (Database Node)

┌─────────────────────────────────────────────────────────┐
│ PHASE 1: PREPARE                                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ Coordinator: "Can you commit operation X?"              │
│                 │                                       │
│                 ├──────────────────────┬────────────────┤
│                 ▼                      ▼                │
│            Check resources        Check resources       │
│            Acquire locks          Acquire locks         │
│            │                      │                     │
│            └──────────────────────┴─────────────┐       │
│                                                 ▼       │
│                                         YES or NO?       │
│            ┌────────────────────────────────────┘       │
│            │                 │                          │
│            ▼                 ▼                          │
│           YES               NO                          │
│        [Response]       [Response]                      │
│        [Durable]        [Durable]                       │
│            │                 │                          │
│            └────────────────────────────────────┬───────┤
│                                                 ▼       │
│                                       ANY NO RECEIVED?  │
│                                                │        │
│                           YES              NO │        │
│                            │                 │        │
│                          ┌─┴───┐        ┌────┴─┐      │
│                          ▼     ▼        ▼      ▼      │
│                       ABORT  ABORT   COMMIT COMMIT     │
│
├─────────────────────────────────────────────────────────┤
│ PHASE 2: COMMIT or ABORT                                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ Execute changes (guaranteed to succeed)                 │
│ OR                                                      │
│ Rollback and release locks                              │
│                                                         │
└─────────────────────────────────────────────────────────┘

KEY PRINCIPLE:
At Phase 2, if ALL said YES, changes are GUARANTEED to succeed
Durability via Write-Ahead Log (WAL) at each step
```

## 2. Write-Ahead Log (WAL) Transaction Flow

```
CLIENT APPLICATION
    │
    ├─ OPERATION 1
    │
    ├─ OPERATION 2
    │
    └─ OPERATION 3


FLOW:

    Operation 1
        │
        ▼
    ┌─────────────────┐
    │ Write to WAL    │  ← Must complete BEFORE executing
    │ (Append-only)   │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │ fsync() WAL     │  ← Durability point: data on disk
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │ Execute Op      │  ← Safe: crash here = recovery redoes
    │ (Change state)  │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │ Log completion  │
    │ of Op 1         │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │ fsync() WAL     │  ← Another durability point
    └────────┬────────┘
             │
    ┌────────┴────────┐
    │                 │
    ▼                 ▼
Operation 2      Operation 3
(same flow)      (same flow)


CRASH SCENARIOS:

Crash before fsync after "Write to WAL":
  → WAL entry lost (in kernel buffer)
  → No changes made to state
  → Safe: nothing persisted

Crash after fsync, before "Execute Op":
  → WAL entry durable
  → State not changed
  → Recovery: repeat operation

Crash after "Execute Op", before fsync completion:
  → State changed
  → WAL not yet durable (still in buffer)
  → Recovery: re-read state, determine what persisted

Crash after fsync completion marker:
  → Everything durable
  → Recovery: complete cleanup operations
```

## 3. git2::Transaction vs git update-ref --atomic

```
┌─────────────────────────────────────────────────────────┐
│ WRONG: git2::Transaction (NOT ATOMIC on failure)        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ let mut tx = repo.transaction()?;                       │
│ tx.lock_ref("refs/heads/main")?;                        │
│ tx.set_target(oid1)?;     ─────┐                        │
│ tx.lock_ref("refs/heads/feature")?;                     │
│ tx.set_target(oid2)?;     ─────┤ Updates applied       │
│ tx.lock_ref("refs/heads/bugfix")?;                      │
│ tx.set_target(oid3)?;  ✗ FAILS ┘ one by one            │
│ tx.commit()?;                                           │
│                                                         │
│ RESULT IF oid3 FAILS:                                   │
│   refs/heads/main  = oid1  ✓ (persisted)               │
│   refs/heads/feature = oid2 ✓ (persisted)              │
│   refs/heads/bugfix = oid_old (unchanged)              │
│                                                         │
│ ⚠️  PARTIAL STATE VISIBLE TO OTHER PROCESSES!          │
│
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ RIGHT: git update-ref --atomic --stdin (ATOMIC)         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ $ git update-ref --atomic --stdin << EOF               │
│ update refs/heads/main oid_old oid1                    │
│ update refs/heads/feature oid_old oid2                 │
│ update refs/heads/bugfix oid_old oid3                  │
│ EOF                                                    │
│                                                         │
│ LOCKING PHASE:                                          │
│   ├─ Try to lock refs/heads/main ✓                      │
│   ├─ Try to lock refs/heads/feature ✓                   │
│   └─ Try to lock refs/heads/bugfix ✓                    │
│                                                         │
│ ALL LOCKS ACQUIRED?                                     │
│   YES: Proceed to update                                │
│   NO:  ABORT (no changes applied)                       │
│                                                         │
│ RESULT ON SUCCESS:                                      │
│   refs/heads/main  = oid1                               │
│   refs/heads/feature = oid2                             │
│   refs/heads/bugfix = oid3                              │
│   All updates atomic ✓                                  │
│                                                         │
│ RESULT ON FAILURE:                                      │
│   refs/heads/main  = oid_old                            │
│   refs/heads/feature = oid_old                          │
│   refs/heads/bugfix = oid_old                           │
│   NO CHANGES APPLIED ✓                                  │
│                                                         │
│ ✅ NO PARTIAL STATE!                                     │
│
└─────────────────────────────────────────────────────────┘

KEY DIFFERENCE:

git2::Transaction
├─ Updates one at a time
├─ First failure stops processing
├─ Previous updates remain (partial state)
└─ NOT safe for multiple refs

git update-ref --atomic
├─ Locks all refs first
├─ If any lock fails: abort completely
├─ Updates all or none (no partial state)
└─ SAFE for multiple refs
```

## 4. Atomic File Write Pattern

```
┌──────────────────────────────────────────────────────────┐
│ ATOMIC FILE WRITE USING TEMP + RENAME                   │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ Step 1: Create Temporary File                            │
│ ────────────────────────────────────────                │
│   target.txt exists → {"old": "data"}                    │
│   target.txt.tmp created (empty)                         │
│                                                          │
│ Step 2: Write Data                                       │
│ ──────────────────                                       │
│   Write to target.txt.tmp → {"new": "data"}             │
│   File is private, no other process sees it              │
│                                                          │
│ Step 3: fsync (Durability)                               │
│ ────────────────────────────                             │
│   fsync(target.txt.tmp)                                  │
│   Ensures data on disk, not in kernel buffer             │
│                                                          │
│ Step 4: Atomic Rename (Linux/Mac)                        │
│ ──────────────────────────────────                       │
│   rename(target.txt.tmp, target.txt)                     │
│   This is atomic on POSIX systems:
│   - Either succeeds (old file replaced)
│   - Or fails entirely (old file unchanged)
│   - Never partial state
│                                                          │
│ Step 5: fsync Directory (Metadata)                       │
│ ─────────────────────────────────────                    │
│   fsync(directory)                                       │
│   Ensures rename operation is durable                    │
│                                                          │
│ RESULT:                                                  │
│   target.txt → {"new": "data"}                           │
│   target.txt.tmp → (deleted or doesn't exist)            │
│                                                          │
│ CRASH SCENARIOS:                                         │
│                                                          │
│ Before fsync (step 3):                                   │
│   └─ target.txt.tmp lost in kernel buffer                │
│   └─ target.txt unchanged                                │
│   └─ Safe: no partial state                              │
│                                                          │
│ Between fsync and rename (step 4):                        │
│   └─ target.txt.tmp on disk (durable)                    │
│   └─ target.txt unchanged                                │
│   └─ Safe: rename never happened                         │
│                                                          │
│ After rename, before dir fsync (step 5):                  │
│   └─ target.txt has new content                          │
│   └─ Rename durable on some filesystems                  │
│   └─ Safe: rename persisted (dir fsync best effort)      │
│                                                          │
│ After everything:                                        │
│   └─ Full durability                                     │
│   └─ Safe: transaction complete                          │
│
└──────────────────────────────────────────────────────────┘

RUST IMPLEMENTATION:

use atomicwrites::{AtomicFile, DisallowOverwrite};

let af = AtomicFile::new("target.txt", DisallowOverwrite);
af.write(|f| {
    f.write_all(b"new data")
})?;  // Automatic rename on success
     // Automatic cleanup on drop if not committed

use tempfile::NamedTempFile;

let mut tmp = NamedTempFile::new()?;
tmp.write_all(b"new data")?;
tmp.persist("target.txt")?;  // Atomic rename (may clobber)
```

## 5. Transaction State Machine

```
                    START
                      │
                      ▼
        ┌─────────────────────────────┐
        │   NOT_STARTED               │
        │ .transaction marker: NONE   │
        └──────────────┬──────────────┘
                       │
                       │ begin_transaction()
                       │ Write marker file
                       │ Write to WAL
                       ▼
        ┌─────────────────────────────┐
        │   IN_PROGRESS               │
        │ .transaction_in_progress    │
        │ Locked by process           │
        └────────────┬────────────────┘
                     │
        ┌────────────┼────────────────┐
        │            │                │
        │      ON CRASH              │
        │       (SIGKILL)             │
        │            │                │
        ▼            ▼                ▼
    [Recovery]  [STALE]        [Normal path]
        │            │                │
        │      (Process not           │
        │       running)              │
        │            │                │
        │      Trigger recovery       │
        │            │                │
        ▼            ▼                │
    ┌─────────────────────────────┐  │
    │ RECOVERY_NEEDED             │  │
    │ WAL has in-progress marker  │  │
    └──────────────┬──────────────┘  │
                   │                 │
              Analysis:              │
        ┌─────────────────┬────────┐ │
        │                 │        │ │
        ▼                 ▼        ▼ │
    Persisted      NOT         On     │
    (fsync mark)  Persisted   Disk    │
        │             │        │      │
        │             │        │      │
        ├─ REDO       ├─ UNDO  └──────┤
        │  COMMIT     │ ROLLBACK      │
        │             │               │
        └─────────────┴───────────────┘
                      │
                      │ Complete Recovery
                      │ Write cleanup marker
                      ▼
    ┌─────────────────────────────────┐
    │ COMMITTED or FAILED             │
    │ .transaction_committed or       │
    │ .transaction_failed             │
    └────────────────┬────────────────┘
                     │
                     │ Cleanup (idempotent)
                     │ Remove markers
                     ▼
        ┌─────────────────────────────┐
        │   COMPLETE                  │
        │ No markers on disk          │
        │ Ready for new transaction   │
        └─────────────────────────────┘
```

## 6. Recovery Detection on Startup

```
STARTUP SEQUENCE:

    Application starts
           │
           ▼
    ┌────────────────────┐
    │ Scan for markers   │
    │ in .jin/           │
    └────────┬───────────┘
             │
             ├─ .transaction_in_progress   ← Interrupted
             ├─ .transaction_committed     ← Cleanup pending
             ├─ .transaction_failed        ← Cleanup pending
             └─ .wal/                      ← Journal entries
             │
             ▼
    ┌─────────────────────────┐
    │ For each in_progress:   │
    │                         │
    │ Check marker age        │
    └────────┬────────────────┘
             │
             ├─ < 5 min old
             │  └─ Maybe process still running
             │     └─ Check via flock/pid
             │        └─ If locked: ignore
             │        └─ If unlocked: recover
             │
             └─ > 5 min old
                └─ Process definitely crashed
                └─ Proceed with recovery
             │
             ▼
    ┌──────────────────────────┐
    │ Read WAL entries for tx  │
    │                          │
    │ Check for:               │
    │ - CommitFsyncComplete?   │
    │ - Which refs persisted?  │
    │ - State on disk?         │
    └────────┬─────────────────┘
             │
             ├─ fsync marker found
             │  └─ All changes persisted
             │  └─ Safe to redo_commit()
             │
             ├─ fsync marker NOT found
             │  └─ Crashed before durability
             │  └─ Safe to rollback()
             │
             └─ Partial state detected
                └─ DANGEROUS
                └─ Manual intervention needed
             │
             ▼
    ┌──────────────────────────┐
    │ Execute recovery op      │
    │ (IDEMPOTENT)             │
    │                          │
    │ Already done?            │
    │   → Skip                 │
    │ Not done?                │
    │   → Execute              │
    │   → Mark complete        │
    └────────┬─────────────────┘
             │
             ▼
    ┌──────────────────────────┐
    │ Cleanup markers          │
    │ Remove .transaction_*    │
    │ Remove stale WAL files   │
    └────────┬─────────────────┘
             │
             ▼
    Application ready
    All transactions recovered
```

## 7. WAL Segment Lifecycle

```
SEGMENT LIFECYCLE:

Creation:
    empty disk space
           │
           ▼
    ┌────────────────┐
    │ wal-0001       │  (pre-allocated to size X)
    │ Magic: "okw"   │
    │ Version: 1     │
    │ Size: 0/10MB   │
    └────────┬───────┘
             │
             │ Operations written
             │
             ▼
    ┌────────────────┐
    │ wal-0001       │
    │ Entry 1: op1   │
    │ Entry 2: op2   │
    │ Entry 3: op3   │
    │ Size: 5/10MB   │
    └────────┬───────┘
             │
             │ Fill up
             │
             ▼
    ┌────────────────┐
    │ wal-0001       │
    │ Entry 1: op1   │
    │ ...            │
    │ Entry N: opN   │
    │ Size: 10/10MB  │ ← Full!
    └────────┬───────┘
             │
             │ Create new segment
             │ and checkpoint old one
             │
    ┌────────┴────────┐
    │                 │
    ▼                 ▼
wal-0002        wal-0001-cp
New writes         (checkpoint in progress)
continue to        marker on disk
wal-0002

    │                 │
    │                 │ Apply entries to
    │                 │ primary store
    │                 │
    │                 ▼
    │            wal-0001.ckpt
    │            (checkpoint done)
    │
    │                 │
    │                 │ Delete old
    │                 │
    │                 ▼
    │              (deleted)
    │
    │                 ▼
    ▼
wal-0002        (space reclaimed)
continues...
    │
    │ (eventually same process
    │  repeats for wal-0002)
    │
    └──────────────────────────

FILE STRUCTURE (per segment):

wal-0001:
┌─────────────────────────┐
│ Magic: "okw"            │ (3 bytes)
├─────────────────────────┤
│ Version: 1              │ (1 byte)
├─────────────────────────┤
│ Entry Marker: 1         │ (1 byte) ← Entry 1 starts
│ Entry 1 Data...         │
│ Entry Marker: 3         │ (1 byte) ← Entry 1 complete
├─────────────────────────┤
│ Entry Marker: 1         │ (1 byte) ← Entry 2 starts
│ Entry 2 Chunk: 2        │ (1 byte) ← Chunk marker
│ Entry 2 Data (part 1)   │
│ Entry 2 Chunk: 2        │ (1 byte) ← Another chunk
│ Entry 2 Data (part 2)   │
│ Entry Marker: 3         │ (1 byte) ← Entry 2 complete
├─────────────────────────┤
│ (empty - pre-allocated) │
│ (empty - pre-allocated) │
│ ... (up to 10MB)        │
└─────────────────────────┘

RECOVERY FROM PARTIAL ENTRY:

If process crashes while writing:
    │
    ├─ Before Entry Marker 3: Entry discarded
    │  └─ Safe: incomplete entry detected
    │
    └─ After Entry Marker 3: Entry included
       └─ Safe: entry is complete
```

---

These diagrams show:
1. How 2PC coordinates across systems
2. How WAL ensures durability
3. Why git2::Transaction fails where git --atomic succeeds
4. How atomic file writes work
5. The transaction state machine
6. How recovery detects interrupted transactions
7. How WAL segments are managed over time

Refer to these when implementing to ensure correct behavior at each step.
