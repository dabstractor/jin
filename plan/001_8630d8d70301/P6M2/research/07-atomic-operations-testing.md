# Testing Transactional and Atomic Operations

## Overview
Atomic operations ensure that complex multi-step procedures either complete entirely or fail cleanly, leaving no partial state. This document covers patterns for testing atomicity guarantees in Git workflows and filesystem operations.

## Core Concepts

### Atomicity Guarantees

**Atomic operation**: An operation that either completes fully or doesn't execute at all.

```
Requirements:
├─ All-or-nothing: No partial execution
├─ Isolation: Concurrent operations don't interfere
├─ Durability: Changes survive process crash
└─ Consistency: State remains valid
```

### Transaction Boundaries

```
Transactional operation structure:
┌──────────────────────────────────────┐
│ BEGIN TRANSACTION                    │
├──────────────────────────────────────┤
│ Step 1: Update reference             │
│ Step 2: Create object                │
│ Step 3: Update index                 │
│ Step 4: Verify consistency           │
├──────────────────────────────────────┤
│ COMMIT (all succeed) or ROLLBACK     │
└──────────────────────────────────────┘
```

## Git Atomicity Patterns

### 1. Ref Atomicity

**Pattern**: References (branches, tags) updates are atomic.

**Test strategy**:
```bash
@test "ref update is atomic" {
    initialize_repo "$TEST_REPO"

    # Create initial commit
    echo "v1" > version.txt
    git -C "$TEST_REPO" add version.txt
    git -C "$TEST_REPO" commit -m "v1"
    local initial_commit=$(git -C "$TEST_REPO" rev-parse HEAD)

    # Attempt to update ref while verifying atomicity
    # Either the ref points to new commit or old commit - never partial state
    git -C "$TEST_REPO" tag v1.0 $initial_commit

    # Verify tag exists atomically
    local tag_commit=$(git -C "$TEST_REPO" rev-parse v1.0)
    [ "$tag_commit" == "$initial_commit" ]

    # No in-between state should be visible
}
```

### 2. Commit Atomicity

**Pattern**: Commits are written atomically to object database.

**Test strategy**:
```go
func TestCommitAtomicity(t *testing.T) {
    // Setup repo
    repo := setupTestRepo(t)

    // Attempt to create commit
    // Either full commit object exists or nothing
    commitHash, err := repo.CreateCommit(CommitOptions{
        Message: "Test commit",
        Files: map[string]string{
            "file1.txt": "content1",
            "file2.txt": "content2",
        },
    })

    if err != nil {
        // If failed, verify no partial state
        _, err := repo.GetObject(commitHash)
        if err == nil {
            t.Fatal("Partial commit object exists after error")
        }
    } else {
        // If succeeded, all objects exist
        commit, err := repo.GetObject(commitHash)
        if err != nil {
            t.Fatal("Commit object missing after successful creation")
        }

        // Verify all referenced objects exist
        for _, file := range commit.Files {
            if _, err := repo.GetObject(file.Hash); err != nil {
                t.Fatal("Blob object missing for file:", file)
            }
        }
    }
}
```

### 3. Push Atomicity (All-or-Nothing Update)

**Pattern**: Multiple ref updates in single push are atomic.

**Test strategy**:
```bash
@test "push updates all refs atomically or none" {
    # Setup: local and remote repos
    initialize_repos "$LOCAL" "$REMOTE"

    # Create multiple branches with changes
    git -C "$LOCAL" checkout -b feature1
    echo "feature1" > feature1.txt
    git -C "$LOCAL" add feature1.txt
    git -C "$LOCAL" commit -m "Feature 1"

    git -C "$LOCAL" checkout -b feature2
    echo "feature2" > feature2.txt
    git -C "$LOCAL" add feature2.txt
    git -C "$LOCAL" commit -m "Feature 2"

    # Push both refs
    # If one fails (e.g., permission), other shouldn't be updated
    run git -C "$LOCAL" push origin feature1 feature2

    # Verify both succeed or both fail
    remote_feature1=$(git -C "$REMOTE" rev-parse feature1 2>/dev/null || echo "MISSING")
    remote_feature2=$(git -C "$REMOTE" rev-parse feature2 2>/dev/null || echo "MISSING")

    # Either both exist or both missing
    if [ "$remote_feature1" == "MISSING" ]; then
        [ "$remote_feature2" == "MISSING" ] || {
            fail "Partial push: feature1 missing but feature2 exists"
        }
    else
        [ "$remote_feature2" != "MISSING" ] || {
            fail "Partial push: feature1 exists but feature2 missing"
        }
    fi
}
```

## Rollback Testing Patterns

### 1. Savepoint-Based Rollback

**Pattern**: Named savepoints enable partial rollback.

```python
class TransactionManager:
    def __init__(self, repo_path):
        self.repo = Repo(repo_path)
        self.savepoints = {}

    def create_savepoint(self, name):
        """Create named savepoint"""
        self.savepoints[name] = self.repo.head.commit

    def rollback_to_savepoint(self, name):
        """Rollback to savepoint"""
        if name not in self.savepoints:
            raise ValueError(f"Savepoint {name} not found")

        target_commit = self.savepoints[name]
        self.repo.head.reset(target_commit, index=True, working_tree=True)

def test_partial_rollback():
    tm = TransactionManager(test_repo)

    # Create initial savepoint
    tm.create_savepoint("initial")

    # Make changes
    add_file(test_repo, "file1.txt", "content1")
    commit(test_repo, "Add file1")

    tm.create_savepoint("after_file1")

    add_file(test_repo, "file2.txt", "content2")
    commit(test_repo, "Add file2")

    # Rollback to middle savepoint
    tm.rollback_to_savepoint("after_file1")

    # Verify state
    assert file_exists(test_repo, "file1.txt")
    assert not file_exists(test_repo, "file2.txt")
```

### 2. Nested Transaction Pattern

**Pattern**: Inner transactions can rollback without affecting outer.

```go
type Transaction struct {
    repo       *Repository
    startCommit string
}

type Savepoint struct {
    transaction *Transaction
    commit      string
}

func (tx *Transaction) CreateSavepoint() *Savepoint {
    return &Savepoint{
        transaction: tx,
        commit:      tx.repo.CurrentCommit(),
    }
}

func (sp *Savepoint) Rollback() error {
    return sp.transaction.repo.Reset(sp.commit)
}

func TestNestedTransactionRollback(t *testing.T) {
    repo := setupTestRepo(t)
    tx := newTransaction(repo)

    // Outer operation
    addFile(repo, "outer1.txt", "data")
    commit(repo, "Outer commit 1")

    // Create savepoint
    sp := tx.CreateSavepoint()

    // Inner operation
    addFile(repo, "inner1.txt", "data")
    commit(repo, "Inner commit 1")

    // Rollback inner (should not affect outer)
    sp.Rollback()

    // Verify outer commit still exists
    commits, _ := repo.Log()
    if len(commits) != 1 {
        t.Fatal("Outer commit was lost")
    }

    // Verify inner file doesn't exist
    if _, err := repo.ReadFile("inner1.txt"); err == nil {
        t.Fatal("Inner file still exists after rollback")
    }
}
```

## Failure Handling in Transactions

### 1. Crash Safety

**Pattern**: Verify atomicity survives process crashes.

```bash
@test "transaction survives process crash" {
    # Setup a transaction-like operation with checkpoints
    create_test_operation_script > "$TEST_OP_SCRIPT"

    # Start operation and kill mid-execution
    "$TEST_OP_SCRIPT" &
    OP_PID=$!

    sleep 0.1  # Let it progress

    kill -9 $OP_PID 2>/dev/null || true
    wait $OP_PID 2>/dev/null || true

    # Verify state is consistent (either fully done or fully undone)
    run check_operation_state "$TEST_REPO"
    [ $status -eq 0 ]

    # State should be valid git repo
    run git -C "$TEST_REPO" fsck
    [ $status -eq 0 ]
}
```

**Script with checkpoints**:
```bash
#!/bin/bash
# Transactional operation with recovery

CHECKPOINT_FILE=".operation_checkpoint"
OPERATION_LOCK=".operation.lock"

commit_checkpoint() {
    local step=$1
    echo "$step" > "$CHECKPOINT_FILE"
}

cleanup_on_error() {
    rm -f "$OPERATION_LOCK"
    # Either complete or rollback - don't leave partial state
}

trap cleanup_on_error EXIT

# Acquire lock
if [ -f "$OPERATION_LOCK" ]; then
    # Recover from previous attempt
    LAST_STEP=$(cat "$CHECKPOINT_FILE" 2>/dev/null)
    echo "Recovering from step $LAST_STEP"
fi

# Step 1
echo "Executing step 1..."
do_something || exit 1
commit_checkpoint "step1"

# Step 2
echo "Executing step 2..."
do_something_else || exit 1
commit_checkpoint "step2"

# Commit (atomically)
echo "Committing transaction..."
git add .
git commit -m "Atomic transaction" || exit 1

# Success - clean up
rm -f "$CHECKPOINT_FILE" "$OPERATION_LOCK"
```

### 2. Concurrent Transaction Isolation

**Pattern**: Verify transactions don't interfere.

```python
import concurrent.futures
import tempfile
import time

def test_concurrent_transaction_isolation():
    with tempfile.TemporaryDirectory() as tmpdir:
        repo1 = initialize_repo(f"{tmpdir}/repo1")
        repo2 = initialize_repo(f"{tmpdir}/repo2")

        results = []

        def transaction_1():
            with repo1.transaction() as tx:
                add_file(repo1, "tx1_file.txt", "content")
                time.sleep(0.1)  # Simulate long operation
                commit(repo1, "TX1 commit")
                results.append(("tx1", True))

        def transaction_2():
            with repo2.transaction() as tx:
                add_file(repo2, "tx2_file.txt", "content")
                time.sleep(0.1)  # Simulate long operation
                commit(repo2, "TX2 commit")
                results.append(("tx2", True))

        # Run transactions concurrently
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
            f1 = executor.submit(transaction_1)
            f2 = executor.submit(transaction_2)
            f1.result()
            f2.result()

        # Verify both succeeded and don't interfere
        assert len(results) == 2
        assert file_exists(repo1, "tx1_file.txt")
        assert not file_exists(repo1, "tx2_file.txt")
        assert file_exists(repo2, "tx2_file.txt")
        assert not file_exists(repo2, "tx1_file.txt")
```

## Testing Atomicity Guarantees

### 1. Consistency Verification

**Pattern**: Verify invariants before and after transaction.

```rust
fn verify_repo_consistency(repo: &Repository) -> Result<(), ConsistencyError> {
    // All objects referenced by refs should exist
    for (ref_name, commit) in repo.refs() {
        verify_object_tree(repo, &commit)?;
    }

    // Index should match HEAD
    let head_tree = repo.head_tree()?;
    let index_tree = repo.index_tree()?;
    if head_tree != index_tree {
        return Err(ConsistencyError::IndexMismatch);
    }

    // Working directory state should be consistent
    for file in repo.list_tracked_files() {
        if !file_exists(file) {
            return Err(ConsistencyError::MissingFile(file.to_string()));
        }
    }

    Ok(())
}

#[test]
fn test_transaction_maintains_consistency() {
    let repo = setup_test_repo();

    // Verify initial consistency
    assert!(verify_repo_consistency(&repo).is_ok());

    // Execute transaction
    let result = repo.transaction(|tx| {
        tx.add_file("file.txt", "content")?;
        tx.commit("Test commit")?;
        Ok(())
    });

    // Verify consistency after transaction
    assert!(result.is_ok());
    assert!(verify_repo_consistency(&repo).is_ok());
}

#[test]
fn test_failed_transaction_maintains_consistency() {
    let repo = setup_test_repo();

    // Execute transaction that will fail
    let _ = repo.transaction(|tx| {
        tx.add_file("file.txt", "content")?;
        // Intentional failure
        Err("Simulated error".into())
    });

    // Verify consistency is maintained even after failure
    assert!(verify_repo_consistency(&repo).is_ok());
}
```

### 2. Durability Verification

**Pattern**: Verify changes survive shutdown.

```bash
@test "transaction changes survive shutdown" {
    # Start operation in background
    my_cli sync "$TEST_REPO" &
    OP_PID=$!

    # Wait for operation to complete or fail
    wait $OP_PID
    EXIT_CODE=$?

    # Shutdown everything
    wait

    # Restart and verify state is as operation left it
    if [ $EXIT_CODE -eq 0 ]; then
        # Committed - changes should exist
        run git -C "$TEST_REPO" log --oneline
        [ $(echo "$output" | wc -l) -ge 1 ]
    else
        # Failed - changes should be rolled back
        # Repo should be valid but unchanged
        run git -C "$TEST_REPO" fsck
        [ $status -eq 0 ]
    fi
}
```

## Locking and Mutual Exclusion

### 1. Lock-Based Mutual Exclusion

**Pattern**: Test that locks prevent concurrent modifications.

```go
type RepositoryLock struct {
    lockFile string
}

func (l *RepositoryLock) Acquire() error {
    // Acquire exclusive lock
    f, err := os.OpenFile(l.lockFile, os.O_CREATE|os.O_EXCL, 0600)
    if err != nil {
        return ErrLocked
    }
    f.Close()
    return nil
}

func TestMutualExclusionWithLock(t *testing.T) {
    tmpdir := t.TempDir()
    lockFile := filepath.Join(tmpdir, ".lock")

    // First locker acquires lock
    lock1 := &RepositoryLock{lockFile}
    err := lock1.Acquire()
    assert.NoError(t, err)

    // Second locker should fail
    lock2 := &RepositoryLock{lockFile}
    err = lock2.Acquire()
    assert.Equal(t, err, ErrLocked)

    // Release lock
    os.Remove(lockFile)

    // Second locker can now acquire
    err = lock2.Acquire()
    assert.NoError(t, err)
}
```

### 2. Deadlock Detection

**Pattern**: Verify no circular dependencies in locks.

```bash
@test "detects and breaks deadlocks" {
    # Create scenario where two operations could deadlock
    # (e.g., both trying to acquire locks in different order)

    my_cli op1 &
    OP1_PID=$!
    my_cli op2 &
    OP2_PID=$!

    # Set timeout - operations should complete or timeout with error
    sleep 5

    if kill -0 $OP1_PID 2>/dev/null; then
        # Still running - might be deadlocked
        kill -9 $OP1_PID 2>/dev/null
        fail "Operation 1 appears deadlocked"
    fi

    if kill -0 $OP2_PID 2>/dev/null; then
        kill -9 $OP2_PID 2>/dev/null
        fail "Operation 2 appears deadlocked"
    fi
}
```

## Best Practices

1. **Test both success and failure paths**: Atomicity matters when things fail
2. **Verify consistency before and after**: Not just operation success
3. **Test crash safety**: Process death mid-transaction
4. **Test concurrent transactions**: Multiple operations simultaneously
5. **Verify no partial state**: Either complete or not at all
6. **Use savepoints for recovery**: Enable resuming failed operations
7. **Document transaction boundaries**: Clear what's atomic
8. **Test cleanup on failure**: No orphaned resources
9. **Verify durability**: Changes survive restarts
10. **Monitor for deadlocks**: Set timeouts, test detection

## References

- [Django: Database Transactions](https://docs.djangoproject.com/en/5.1/topics/db/transactions/)
- [SQLite: Atomic Commit](https://sqlite.org/atomiccommit.html)
- [GeeksforGeeks: Atomic Transactions in OS](https://www.geeksforgeeks.org/operating-systems/atomic-transactions-in-os/)
- [Git: Internal Objects Model](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [Testing Atomic Operations in Rust](https://www.rust-lang.org/what/wg-libs/arc-atomic/)
