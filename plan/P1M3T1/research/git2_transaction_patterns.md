# git2-rs Atomic Transaction Patterns and Multi-Ref Operations Research

## Table of Contents
1. [git2-rs Transaction API Overview](#git2-rs-transaction-api-overview)
2. [Atomic Transaction Patterns](#atomic-transaction-patterns)
3. [Multi-Ref Operations](#multi-ref-operations)
4. [Error Handling and Rollback](#error-handling-and-rollback)
5. [Staging References and Recovery](#staging-references-and-recovery)
6. [Best Practices](#best-practices)
7. [Common Pitfalls](#common-pitfalls)
8. [Code Examples](#code-examples)
9. [Advanced Patterns](#advanced-patterns)

## git2-rs Transaction API Overview

### Core Transaction Types

git2-rs provides two main types of transactions:

#### 1. Repository Transaction (`repository::transaction`)
```rust
impl Repository {
    /// Create a new reference transaction
    pub fn transaction(&self) -> Result<Transaction<'_>, Error>

    /// Create a new reference transaction with flags
    pub fn transaction_flags(&self, flags: TransactionFlags) -> Result<Transaction<'_>, Error>
}
```

#### 2. Reference Transaction (`Transaction`)
```rust
pub struct Transaction<'repo> {
    // Internal implementation
}

impl<'repo> Transaction<'repo> {
    // Lock references for the transaction
    pub fn lock(&mut self, refs: &[&str], force: bool) -> Result<(), Error>

    // Set a reference target
    pub fn set_target(
        &mut self,
        refname: &str,
        id: Oid,
        msg: &str
    ) -> Result<(), Error>

    // Delete a reference
    pub fn delete(&mut self, refname: &str, msg: &str) -> Result<(), Error>

    // Set the reference log message
    pub fn set_reflog(&mut self, refname: &str, msg: &str) -> Result<(), Error>

    // Commit the transaction
    pub fn commit(&mut self) -> Result<(), Error>

    // Roll back the transaction
    pub fn rollback(&mut self) -> Result<(), Error>
}
```

### Transaction Flags

```rust
bitflags! {
    pub struct TransactionFlags: u32 {
        const NONE = 0;
        const REFSPEC_ATOMIFY = 1 << 0;
        const REFSPEC_DELETE = 1 << 1;
        const REFSPEC_LITERAL = 1 << 2;
    }
}
```

## Atomic Transaction Patterns

### Basic Atomic Transaction

```rust
use git2::{Repository, Transaction};

fn atomic_update(repo: &Repository, updates: Vec<(&str, git2::Oid)>) -> Result<(), git2::Error> {
    // Start a new transaction
    let mut tx = repo.transaction()?;

    // Lock all references we want to modify
    let ref_names: Vec<&str> = updates.iter().map(|(name, _)| *name).collect();
    tx.lock(&ref_names, false)?;

    // Apply all updates
    for (refname, new_oid) in updates {
        tx.set_target(refname, new_oid, "Atomic update")?;
    }

    // Commit all changes atomically
    tx.commit()?;

    Ok(())
}
```

### Transaction with Validation

```rust
fn validated_atomic_update(
    repo: &Repository,
    updates: Vec<(String, git2::Oid)>,
) -> Result<(), git2::Error> {
    let mut tx = repo.transaction()?;

    // Extract reference names
    let ref_names: Vec<&str> = updates.iter().map(|(name, _)| name.as_str()).collect();

    // Lock references
    tx.lock(&ref_names, false)?;

    // Validate each update before applying
    for (refname, new_oid) in &updates {
        // Check if reference exists and get current state
        match repo.find_reference(refname) {
            Ok(current_ref) => {
                // Verify current target matches expectations if needed
                if current_ref.target() != Some(*new_oid) {
                    // Proceed with update
                    tx.set_target(refname, *new_oid, "Validated atomic update")?;
                }
            },
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                // Reference doesn't exist, create it
                tx.set_target(refname, *new_oid, "Create new ref")?;
            },
            Err(e) => return Err(e),
        }
    }

    tx.commit()
}
```

### Nested Transaction Pattern

```rust
use std::collections::HashMap;

struct NestedTransaction<'a> {
    repo: &'a Repository,
    operations: Vec<TransactionOperation>,
    committed: bool,
}

enum TransactionOperation {
    Update(String, git2::Oid),
    Delete(String),
}

impl<'a> NestedTransaction<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        NestedTransaction {
            repo,
            operations: Vec::new(),
            committed: false,
        }
    }

    pub fn update_ref(&mut self, refname: String, oid: git2::Oid) {
        self.operations.push(TransactionOperation::Update(refname, oid));
    }

    pub fn delete_ref(&mut self, refname: String) {
        self.operations.push(TransactionOperation::Delete(refname));
    }

    pub fn execute(&mut self) -> Result<(), git2::Error> {
        if self.committed {
            return Err(git2::Error::from_str("Transaction already committed"));
        }

        let mut tx = self.repo.transaction()?;

        // Extract all refnames
        let ref_names: Vec<&str> = self.operations.iter()
            .map(|op| match op {
                TransactionOperation::Update(refname, _) => refname.as_str(),
                TransactionOperation::Delete(refname) => refname.as_str(),
            })
            .collect();

        // Lock all references
        tx.lock(&ref_names, false)?;

        // Execute operations
        for op in &self.operations {
            match op {
                TransactionOperation::Update(refname, oid) => {
                    tx.set_target(refname, *oid, "Nested transaction update")?;
                },
                TransactionOperation::Delete(refname) => {
                    tx.delete(refname, "Nested transaction delete")?;
                },
            }
        }

        tx.commit()?;
        self.committed = true;
        Ok(())
    }
}
```

## Multi-Ref Operations

### Batch Reference Update

```rust
fn batch_update_refs(
    repo: &Repository,
    updates: HashMap<String, git2::Oid>,
    message: &str,
) -> Result<(), git2::Error> {
    let mut tx = repo.transaction()?;

    // Convert to vector for locking
    let ref_names: Vec<&str> = updates.keys().map(|s| s.as_str()).collect();

    // Lock all references
    tx.lock(&ref_names, false)?;

    // Apply all updates
    for (refname, new_oid) in updates {
        tx.set_target(&refname, new_oid, message)?;
    }

    tx.commit()
}
```

### Conditional Multi-Ref Update

```rust
fn conditional_multi_ref_update(
    repo: &Repository,
    updates: Vec<(String, git2::Oid, Option<git2::Oid>)>, // (refname, new_oid, expected_old_oid)
) -> Result<(), git2::Error> {
    let mut tx = repo.transaction()?;

    let ref_names: Vec<&str> = updates.iter().map(|(name, _, _)| name.as_str()).collect();
    tx.lock(&ref_names, false)?;

    for (refname, new_oid, expected_old) in updates {
        if let Some(expected) = expected_old {
            // Check current value matches expectation
            let current_ref = repo.find_reference(&refname)?;
            if current_ref.target() != Some(expected) {
                tx.rollback()?;
                return Err(git2::Error::from_str(
                    "Reference value mismatch, transaction rolled back"
                ));
            }
        }

        tx.set_target(&refname, new_oid, "Conditional multi-ref update")?;
    }

    tx.commit()
}
```

### Branch and Tag Updates

```rust
fn update_branches_and_tags(
    repo: &Repository,
    branch_updates: Vec<(&str, git2::Oid)>,
    tag_updates: Vec<(&str, git2::Oid)>,
) -> Result<(), git2::Error> {
    let mut tx = repo.transaction()?;

    // Combine all references to lock
    let all_refs: Vec<&str> = branch_updates.iter()
        .chain(tag_updates.iter())
        .map(|(name, _)| *name)
        .collect();

    tx.lock(&all_refs, false)?;

    // Update branches
    for (branch, oid) in branch_updates {
        tx.set_target(branch, oid, "Update branch")?;
    }

    // Update tags
    for (tag, oid) in tag_updates {
        tx.set_target(tag, oid, "Update tag")?;
    }

    tx.commit()
}
```

## Error Handling and Rollback

### Transaction Error Handling

```rust
fn safe_transaction<T>(
    repo: &Repository,
    operation: impl FnOnce(&mut git2::Transaction) -> Result<T, git2::Error>,
) -> Result<T, git2::Error> {
    let mut tx = repo.transaction()?;

    match operation(&mut tx) {
        Ok(result) => {
            tx.commit()?;
            Ok(result)
        },
        Err(e) => {
            // Attempt to rollback
            let _ = tx.rollback();
            Err(e)
        }
    }
}

// Usage example
fn update_with_error_handling(repo: &Repository) -> Result<()> {
    safe_transaction(repo, |tx| {
        // Perform operations
        tx.set_target("refs/heads/main", new_commit_id, "Update main")?;
        tx.set_target("refs/heads/feature", feature_commit_id, "Update feature")?;

        // Simulate an error
        if some_condition {
            return Err(git2::Error::from_str("Validation failed"));
        }

        Ok(())
    })
}
```

### Rollback on Drop

```rust
struct AutoRollbackTransaction<'a> {
    transaction: Option<git2::Transaction<'a>>,
    repo: &'a Repository,
    should_rollback: bool,
}

impl<'a> AutoRollbackTransaction<'a> {
    pub fn new(repo: &'a Repository) -> Result<Self, git2::Error> {
        Ok(Self {
            transaction: Some(repo.transaction()?),
            repo,
            should_rollback: true,
        })
    }

    pub fn commit(mut self) -> Result<(), git2::Error> {
        if let Some(ref mut tx) = self.transaction {
            tx.commit()?;
            self.should_rollback = false;
        }
        Ok(())
    }

    pub fn set_target(&mut self, refname: &str, id: git2::Oid, msg: &str) -> Result<(), git2::Error> {
        if let Some(ref mut tx) = self.transaction {
            tx.set_target(refname, id, msg)
        } else {
            Err(git2::Error::from_str("Transaction already committed"))
        }
    }
}

impl<'a> Drop for AutoRollbackTransaction<'a> {
    fn drop(&mut self) {
        if self.should_rollback {
            if let Some(ref mut tx) = self.transaction.take() {
                let _ = tx.rollback();
            }
        }
    }
}
```

### Retryable Transaction Pattern

```rust
fn retryable_transaction<F, T>(
    repo: &Repository,
    max_retries: u32,
    operation: F,
) -> Result<T, git2::Error>
where
    F: FnMut() -> Result<T, git2::Error>,
{
    let mut attempt = 0;

    loop {
        match safe_transaction(repo, |tx| {
            // Clone the operation to retry it
            let mut op_clone = operation.clone();
            op_clone()
        }) {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                if attempt >= max_retries {
                    return Err(e);
                }

                // Check if error is retryable
                if !is_retryable_error(&e) {
                    return Err(e);
                }

                // Exponential backoff
                let delay = std::time::Duration::from_millis(2u64.pow(attempt.min(6)) * 100);
                std::thread::sleep(delay);
            }
        }
    }
}

fn is_retryable_error(err: &git2::Error) -> bool {
    matches!(
        err.code(),
        git2::ErrorCode::Locked |
        git2::ErrorCode::Modified |
        git2::ErrorCode::Conflict
    )
}
```

## Staging References and Recovery

### Staging Reference Pattern

```rust
use uuid::Uuid;

struct TransactionManager {
    repo: git2::Repository,
}

impl TransactionManager {
    pub fn new(repo: git2::Repository) -> Self {
        Self { repo }
    }

    pub fn begin_transaction(&self) -> Result<TransactionHandle, git2::Error> {
        let transaction_id = Uuid::new_v4().to_string();
        Ok(TransactionHandle::new(&self.repo, transaction_id)?)
    }

    pub fn recover_orphaned_transactions(&self) -> Result<Vec<String>, git2::Error> {
        let mut orphaned = Vec::new();

        // Look for staging refs
        for reference in self.repo.references_glob("refs/jin/staging/*")? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                if name.starts_with("refs/jin/staging/") {
                    let transaction_id = name.strip_prefix("refs/jin/staging/").unwrap();
                    orphaned.push(transaction_id.to_string());
                }
            }
        }

        Ok(orphaned)
    }
}

struct TransactionHandle {
    repo: git2::Repository,
    transaction_id: String,
    committed: bool,
}

impl TransactionHandle {
    pub fn new(repo: &git2::Repository, transaction_id: String) -> Result<Self, git2::Error> {
        // Create staging reference
        let staging_ref_name = format!("refs/jin/staging/{}", transaction_id);
        let staging_oid = repo.find_reference("HEAD")?.target().unwrap_or_else(|| {
            // Fallback to empty tree if no HEAD
            repo.find_tree(repo.treebuilder(None)?.write()?).unwrap().id()
        });

        repo.reference(
            &staging_ref_name,
            staging_oid,
            false, // Don't overwrite if exists
            &format!("Staging for transaction: {}", transaction_id),
        )?;

        Ok(Self {
            repo: repo.clone(),
            transaction_id,
            committed: false,
        })
    }

    pub fn stage_update(&mut self, refname: &str, new_oid: git2::Oid) -> Result<(), git2::Error> {
        let staging_ref_name = format!("refs/jin/staging/{}", self.transaction_id);

        // Store the update in the staging area
        // This is a simplified example - in practice you might store more metadata
        let metadata = format!("{}:{}", refname, new_oid);
        let blob = self.repo.blob(metadata.as_bytes())?;

        let mut builder = self.repo.treebuilder(None)?;
        builder.insert("updates", blob, git2::FileMode::Blob.into())?;

        let tree_id = builder.write()?;
        let tree = self.repo.find_tree(tree_id)?;

        let author = self.repo.signature("Transaction Manager", "tm@example.com")?;

        self.repo.commit(
            Some(&staging_ref_name),
            &author,
            &author,
            &format!("Stage updates for transaction {}", self.transaction_id),
            &tree,
            &[self.repo.find_reference(&staging_ref_name)?.target().map(|oid| {
                self.repo.find_commit(oid).unwrap()
            })].iter().filter_map(|o| o.as_ref()).collect::<Vec<_>>(),
        )?;

        Ok(())
    }

    pub fn commit(self, updates: Vec<(String, git2::Oid)>) -> Result<(), git2::Error> {
        if self.committed {
            return Err(git2::Error::from_str("Transaction already committed"));
        }

        let mut tx = self.repo.transaction()?;

        // Lock all target references
        let ref_names: Vec<&str> = updates.iter().map(|(name, _)| name.as_str()).collect();
        tx.lock(&ref_names, false)?;

        // Apply all updates
        for (refname, new_oid) in updates {
            tx.set_target(&refname, new_oid, &format!("From transaction: {}", self.transaction_id))?;
        }

        tx.commit()?;

        // Clean up staging reference
        let staging_ref_name = format!("refs/jin/staging/{}", self.transaction_id);
        self.repo.find_reference(&staging_ref_name)?.delete()?;

        Ok(())
    }
}

impl Drop for TransactionHandle {
    fn drop(&mut self) {
        if !self.committed {
            // Clean up staging reference on drop (rollback)
            let staging_ref_name = format!("refs/jin/staging/{}", self.transaction_id);
            if let Ok(mut reference) = self.repo.find_reference(&staging_ref_name) {
                let _ = reference.delete();
            }
        }
    }
}
```

### Transaction Recovery System

```rust
struct RecoveryManager {
    repo: git2::Repository,
}

impl RecoveryManager {
    pub fn new(repo: git2::Repository) -> Self {
        Self { repo }
    }

    pub fn recover_transactions(&self) -> Result<u32, git2::Error> {
        let mut recovered = 0;

        // Find all staging references
        for reference in self.repo.references_glob("refs/jin/staging/*")? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                if name.starts_with("refs/jin/staging/") {
                    let transaction_id = name.strip_prefix("refs/jin/staging/").unwrap();

                    if self.try_recover_transaction(transaction_id)? {
                        recovered += 1;
                    }
                }
            }
        }

        Ok(recovered)
    }

    fn try_recover_transaction(&self, transaction_id: &str) -> Result<bool, git2::Error> {
        let staging_ref_name = format!("refs/jin/staging/{}", transaction_id);

        // Check if transaction was completed but not cleaned up
        let commit = match self.repo.find_reference(&staging_ref_name) {
            Ok(ref_ref) => match ref_ref.target() {
                Some(oid) => match self.repo.find_commit(oid) {
                    Ok(commit) => Some(commit),
                    Err(_) => None,
                },
                None => None,
            },
            Err(_) => return Ok(false), // Staging ref doesn't exist
        };

        if let Some(commit) = commit {
            let message = commit.message().unwrap_or("");

            if message.contains("From transaction:") {
                // This was a completed transaction - safe to clean up
                self.repo.find_reference(&staging_ref_name)?.delete()?;
                return Ok(true);
            } else if message.starts_with("Stage updates for transaction") {
                // Incomplete transaction - try to complete or clean up
                return self.recover_incomplete_transaction(&staging_ref_name, &commit);
            }
        }

        Ok(false)
    }

    fn recover_incomplete_transaction(
        &self,
        staging_ref_name: &str,
        commit: &git2::Commit,
    ) -> Result<bool, git2::Error> {
        // Parse updates from commit message or tree
        // This is simplified - in practice you'd store more structured data
        let message = commit.message().unwrap_or("");

        // Try to apply the transaction if still valid
        // This requires additional validation logic

        // For now, just clean up
        self.repo.find_reference(staging_ref_name)?.delete()?;
        Ok(true)
    }

    pub fn cleanup_old_transactions(&self, older_than_days: u64) -> Result<u32, git2::Error> {
        let mut cleaned = 0;
        let cutoff = std::time::SystemTime::now()
            - std::time::Duration::from_secs(older_than_days * 24 * 3600);

        for reference in self.repo.references_glob("refs/jin/staging/*")? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                if name.starts_with("refs/jin/staging/") {
                    let commit = match reference.target().and_then(|oid| self.repo.find_commit(oid).ok()) {
                        Some(commit) => commit,
                        None => continue,
                    };

                    let commit_time = commit.time();

                    // Convert git2 time to SystemTime
                    let commit_system_time = std::time::UNIX_EPOCH
                        + std::time::Duration::from_secs(commit_time.seconds() as u64);

                    if commit_system_time < cutoff {
                        // Old transaction - safe to clean up
                        reference.delete()?;
                        cleaned += 1;
                    }
                }
            }
        }

        Ok(cleaned)
    }
}
```

## Best Practices

### 1. Always Lock All References First
```rust
// Good: Lock all references before any modifications
let mut tx = repo.transaction()?;
let ref_names = vec!["refs/heads/main", "refs/heads/feature"];
tx.lock(&ref_names, false)?;
tx.set_target("refs/heads/main", new_main_id, "")?;
tx.set_target("refs/heads/feature", new_feature_id, "")?;
tx.commit()?;

// Bad: Modifying references without proper locking
// This can lead to race conditions
```

### 2. Validate Before Transaction
```rust
fn validate_updates(repo: &Repository, updates: &[(String, git2::Oid)]) -> Result<(), git2::Error> {
    for (refname, expected_oid) in updates {
        if let Ok(current_ref) = repo.find_reference(refname) {
            if current_ref.target() != Some(*expected_oid) {
                return Err(git2::Error::from_str(
                    format!("Validation failed for {}", refname).as_str()
                ));
            }
        }
    }
    Ok(())
}
```

### 3. Use Descriptive Messages
```rust
// Good: Clear, descriptive messages
tx.set_target(
    "refs/heads/feature",
    new_commit_id,
    "chore: Add feature flags and configuration system"
)?;

// Bad: Vague or empty messages
tx.set_target("refs/heads/feature", new_commit_id, "")?;
```

### 4. Handle Rollbacks Gracefully
```rust
fn execute_operation(repo: &Repository) -> Result<()> {
    let mut tx = repo.transaction()?;

    // Set up rollback handlers
    let rollback_guard = scopeguard::guard((), |_| {
        // Ensure rollback is called on error
        let _ = tx.rollback();
    });

    // Perform operations
    tx.set_target("refs/heads/main", new_id, "Update")?;

    // Drop the guard to prevent rollback on success
    drop(rollback_guard);

    tx.commit()?;
    Ok(())
}
```

### 5. Monitor Transaction State
```rust
fn transaction_with_monitoring(repo: &Repository) -> Result<(), git2::Error> {
    let start_time = std::time::Instant::now();
    let mut tx = repo.transaction()?;

    try {
        // Perform operations
        let operation_count = 0; // Track number of operations

        // Log transaction progress
        println!("Transaction in progress with {} operations", operation_count);

        tx.commit()?;

        let duration = start_time.elapsed();
        println!("Transaction completed in {:?}", duration);

        Ok(())
    } catch {
        let duration = start_time.elapsed();
        eprintln!("Transaction failed after {:?}", duration);
        Err(e)
    }
}
```

## Common Pitfalls

### 1. Race Conditions
```rust
// Pitfall: Checking and updating references separately
fn bad_atomic_update(repo: &Repository, refname: &str, new_oid: git2::Oid) -> Result<(), git2::Error> {
    // Race condition: another process could modify the reference between check and update
    let current_ref = repo.find_reference(refname)?;
    if current_ref.target() != Some(expected_oid) {
        return Err(git2::Error::from_str("Reference changed"));
    }
    repo.reference(refname, new_oid, true, "Update")?;
    Ok(())
}

// Solution: Use transactions
fn good_atomic_update(repo: &Repository, refname: &str, new_oid: git2::Oid) -> Result<(), git2::Error> {
    let mut tx = repo.transaction()?;
    tx.lock(&[refname], false)?;
    tx.set_target(refname, new_oid, "Update")?;
    tx.commit()?;
    Ok(())
}
```

### 2. Deadlocks
```rust
// Pitfall: Locking references in inconsistent order
fn bad_multi_repo_update(
    repo1: &Repository,
    repo2: &Repository,
    ref1: &str,
    ref2: &str,
    new_oid1: git2::Oid,
    new_oid2: git2::Oid,
) -> Result<(), git2::Error> {
    // Deadlock risk if repos are locked in different orders by different calls
    let mut tx1 = repo1.transaction()?;
    let mut tx2 = repo2.transaction()?;

    tx1.lock(&[ref1], false)?;
    tx2.lock(&[ref2], false)?; // Potential deadlock

    tx1.set_target(ref1, new_oid1, "")?;
    tx2.set_target(ref2, new_oid2, "")?;

    tx1.commit()?;
    tx2.commit()?;
    Ok(())
}

// Solution: Use consistent locking order
fn good_multi_repo_update(
    repos: &[&Repository],
    updates: &[(&str, git2::Oid)],
) -> Result<(), git2::Error> {
    // Always lock repositories in the same order (by path)
    let mut locked_txs: Vec<(usize, git2::Transaction)> = Vec::new();

    for (i, repo) in repos.iter().enumerate() {
        let mut tx = repo.transaction()?;

        let ref_names: Vec<&str> = updates
            .iter()
            .filter(|(refname, _)| {
                // Determine which refs belong to which repo
                refname.starts_with("refs/heads/repo1") && i == 0 ||
                refname.starts_with("refs/heads/repo2") && i == 1
            })
            .map(|(name, _)| *name)
            .collect();

        if !ref_names.is_empty() {
            tx.lock(&ref_names, false)?;
            locked_txs.push((i, tx));
        }
    }

    // Apply updates
    for (i, tx) in &mut locked_txs {
        for (refname, new_oid) in updates {
            if (refname.starts_with("refs/heads/repo1") && *i == 0) ||
               (refname.starts_with("refs/heads/repo2") && *i == 1) {
                tx.set_target(refname, *new_oid, "")?;
            }
        }
    }

    // Commit all transactions
    for (_, tx) in locked_txs {
        tx.commit()?;
    }

    Ok(())
}
```

### 3. Memory Leaks
```rust
// Pitfall: Not cleaning up staging references
fn bad_transaction_with_staging(repo: &Repository) -> Result<(), git2::Error> {
    let staging_ref = "refs/staging/tmp";

    // Create staging ref
    repo.reference(staging_ref, some_oid, true, "Temporary")?;

    // ... perform operations ...

    // Forget to clean up staging ref!
    // This leaves garbage in the repository

    Ok(())
}

// Solution: Use RAII
fn good_transaction_with_staging(repo: &Repository) -> Result<(), git2::Error> {
    let staging_ref = "refs/staging/tmp";

    // Create staging ref
    let mut staging = repo.reference(staging_ref, some_oid, true, "Temporary")?;

    // ... perform operations ...

    // Clean up when staging goes out of scope
    staging.delete()?;

    Ok(())
}
```

### 4. Transaction Timeout
```rust
// Pitfall: Long-running transactions without timeout
fn bad_long_transaction(repo: &Repository) -> Result<(), git2::Error> {
    let mut tx = repo.transaction()?;

    // Lock references for a long time
    tx.lock(&["refs/heads/main"], false)?;

    // Perform long operation
    long_running_operation()?;

    tx.commit()?;
    Ok(())
}

// Solution: Add timeout
use std::time::Instant;

fn good_timed_transaction(repo: &Repository, timeout_secs: u64) -> Result<(), git2::Error> {
    let start = Instant::now();
    let mut tx = repo.transaction()?;

    tx.lock(&["refs/heads/main"], false)?;

    while start.elapsed().as_secs() < timeout_secs {
        if check_condition() {
            tx.commit()?;
            return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    tx.rollback()?;
    Err(git2::Error::from_str("Transaction timeout"))
}
```

## Code Examples

### Complete Transaction Manager

```rust
use git2::{Repository, Oid, Transaction};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TransactionManager {
    repo: Repository,
}

#[derive(Debug)]
pub struct TransactionUpdate {
    pub refname: String,
    pub new_oid: Oid,
    pub old_oid: Option<Oid>, // None for new refs
    pub message: String,
}

#[derive(Debug)]
pub struct Transaction {
    pub id: String,
    pub created_at: SystemTime,
    pub updates: Vec<TransactionUpdate>,
}

impl TransactionManager {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub fn begin_transaction(&self) -> Result<Transaction, git2::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = SystemTime::now();
        let updates = Vec::new();

        Ok(Transaction { id, created_at, updates })
    }

    pub fn add_update(
        &mut self,
        transaction: &mut Transaction,
        refname: String,
        new_oid: Oid,
        message: String,
    ) -> Result<(), git2::Error> {
        // Get current oid if ref exists
        let old_oid = self.repo.find_reference(&refname)
            .ok()
            .and_then(|r| r.target());

        transaction.updates.push(TransactionUpdate {
            refname,
            new_oid,
            old_oid,
            message,
        });

        Ok(())
    }

    pub fn commit_transaction(&self, transaction: Transaction) -> Result<(), git2::Error> {
        if transaction.updates.is_empty() {
            return Ok(()); // No-op
        }

        let mut tx = self.repo.transaction()?;

        // Lock all references
        let ref_names: Vec<&str> = transaction.updates
            .iter()
            .map(|u| u.refname.as_str())
            .collect();

        tx.lock(&ref_names, false)?;

        // Apply all updates
        for update in &transaction.updates {
            tx.set_target(&update.refname, update.new_oid, &update.message)?;
        }

        tx.commit()?;

        // Log transaction completion
        println!(
            "Transaction {} completed with {} updates",
            transaction.id,
            transaction.updates.len()
        );

        Ok(())
    }

    pub fn rollback_transaction(&self, transaction: Transaction) -> Result<(), git2::Error> {
        println!("Rolling back transaction {}", transaction.id);
        Ok(())
    }

    pub fn verify_transaction(&self, transaction: &Transaction) -> Result<(), git2::Error> {
        for update in &transaction.updates {
            if let Ok(current_ref) = self.repo.find_reference(&update.refname) {
                let current_oid = current_ref.target();
                if current_oid != Some(update.old_oid) {
                    return Err(git2::Error::from_str(
                        format!(
                            "Reference {} has changed (expected: {:?}, actual: {:?})",
                            update.refname, update.old_oid, current_oid
                        ).as_str()
                    ));
                }
            } else if update.old_oid.is_some() {
                return Err(git2::Error::from_str(
                    format!("Reference {} no longer exists", update.refname).as_str()
                ));
            }
        }
        Ok(())
    }

    pub fn list_transactions(&self) -> Result<Vec<Transaction>, git2::Error> {
        // This would scan for transaction metadata
        // For now, return empty vector
        Ok(Vec::new())
    }
}

// Usage example
fn usage_example() -> Result<(), git2::Error> {
    let repo = Repository::open(".")?;
    let mut tm = TransactionManager::new(repo);

    // Start transaction
    let mut transaction = tm.begin_transaction()?;

    // Add updates
    tm.add_update(
        &mut transaction,
        "refs/heads/main".to_string(),
        some_commit_oid(),
        "Update main branch".to_string(),
    )?;

    tm.add_update(
        &mut transaction,
        "refs/tags/v1.0.0".to_string(),
        some_tag_oid(),
        "Create v1.0.0 tag".to_string(),
    )?;

    // Verify before committing
    tm.verify_transaction(&transaction)?;

    // Commit or rollback
    match tm.commit_transaction(transaction) {
        Ok(_) => println!("Transaction committed successfully"),
        Err(e) => {
            println!("Transaction failed: {}", e);
            // Handle error
        }
    }

    Ok(())
}
```

### High-Level Atomic Operations

```rust
pub trait AtomicGitOperations {
    fn atomic_branch_update(
        &self,
        branch: &str,
        new_commit: Oid,
        old_commit: Option<Oid>,
        message: &str,
    ) -> Result<(), git2::Error>;

    fn atomic_tag_update(
        &self,
        tag: &str,
        target_commit: Oid,
        message: &str,
    ) -> Result<(), git2::Error>;

    fn atomic_multiple_updates(
        &self,
        updates: Vec<(&str, Oid, Option<Oid>)>,
        message: &str,
    ) -> Result<(), git2::Error>;
}

impl AtomicGitOperations for Repository {
    fn atomic_branch_update(
        &self,
        branch: &str,
        new_commit: Oid,
        old_commit: Option<Oid>,
        message: &str,
    ) -> Result<(), git2::Error> {
        let mut tx = self.transaction()?;
        let branch_ref = format!("refs/heads/{}", branch);

        // Validate if old_commit is specified
        if let Some(expected_old) = old_commit {
            let current_ref = self.find_reference(&branch_ref)?;
            if current_ref.target() != Some(expected_old) {
                return Err(git2::Error::from_str("Branch has been modified"));
            }
        }

        tx.lock(&[&branch_ref], false)?;
        tx.set_target(&branch_ref, new_commit, message)?;
        tx.commit()
    }

    fn atomic_tag_update(
        &self,
        tag: &str,
        target_commit: Oid,
        message: &str,
    ) -> Result<(), git2::Error> {
        let mut tx = self.transaction()?;
        let tag_ref = format!("refs/tags/{}", tag);

        tx.lock(&[&tag_ref], true)?; // Allow overwriting tags
        tx.set_target(&tag_ref, target_commit, message)?;
        tx.commit()
    }

    fn atomic_multiple_updates(
        &self,
        updates: Vec<(&str, Oid, Option<Oid>)>,
        message: &str,
    ) -> Result<(), git2::Error> {
        let mut tx = self.transaction()?;

        // Validate all updates first
        for (refname, new_oid, expected_old) in &updates {
            if let Some(expected) = expected_old {
                let current_ref = self.find_reference(refname)?;
                if current_ref.target() != Some(*expected) {
                    return Err(git2::Error::from_str(
                        format!("Reference {} has changed", refname).as_str()
                    ));
                }
            }
        }

        // Lock all references
        let ref_names: Vec<&str> = updates.iter().map(|(name, _, _)| *name).collect();
        tx.lock(&ref_names, false)?;

        // Apply updates
        for (refname, new_oid, _) in updates {
            tx.set_target(refname, new_oid, message)?;
        }

        tx.commit()
    }
}
```

## Advanced Patterns

### Distributed Transactions

```rust
use std::sync::Arc;

pub struct DistributedTransactionManager {
    repositories: Arc<Vec<Repository>>,
}

impl DistributedTransactionManager {
    pub fn new(repositories: Vec<Repository>) -> Self {
        Self {
            repositories: Arc::new(repositories),
        }
    }

    pub fn execute_distributed_transaction(
        &self,
        operations: Vec<(usize, String, Oid, String)>, // (repo_idx, refname, oid, message)
    ) -> Result<(), DistributedTransactionError> {
        // Two-phase commit protocol
        let mut phase1_results = Vec::new();

        // Phase 1: Prepare all transactions
        for (repo_idx, refname, new_oid, message) in &operations {
            let repo = &self.repositories[*repo_idx];

            match self.prepare_repo_transaction(repo, refname, new_oid, &message) {
                Ok(()) => phase1_results.push((repo_idx, refname, Ok(()))),
                Err(e) => {
                    // Rollback any prepared transactions
                    self.rollback_prepared(&phase1_results);
                    return Err(DistributedTransactionError::PreparationFailed {
                        repo_idx: *repo_idx,
                        refname: refname.clone(),
                        source: e,
                    });
                }
            }
        }

        // Phase 2: Commit all transactions
        for (repo_idx, refname, _) in &phase1_results {
            let repo = &self.repositories[*repo_idx];

            match self.commit_repo_transaction(repo, refname) {
                Ok(()) => {},
                Err(e) => {
                    // This is more serious - manual intervention may be required
                    return Err(DistributedTransactionError::CommitFailed {
                        repo_idx: *repo_idx,
                        refname: refname.clone(),
                        source: e,
                    });
                }
            }
        }

        Ok(())
    }

    fn prepare_repo_transaction(
        &self,
        repo: &Repository,
        refname: &str,
        new_oid: Oid,
        message: &str,
    ) -> Result<(), git2::Error> {
        let mut tx = repo.transaction()?;

        // Check if reference exists and get current state
        let current_oid = repo.find_reference(refname)
            .ok()
            .and_then(|r| r.target());

        // Lock the reference
        tx.lock(&[refname], false)?;

        // Store preparation info (simplified)
        let prep_info = format!("PREPARE:{}:{}", refname, new_oid);
        let blob = repo.blob(prep_info.as_bytes())?;

        // Note: In a real implementation, you'd store this properly
        println!("Prepared update for {} in repo", refname);

        Ok(())
    }

    fn commit_repo_transaction(
        &self,
        repo: &Repository,
        refname: &str,
    ) -> Result<(), git2::Error> {
        let mut tx = repo.transaction()?;
        tx.lock(&[refname], false)?;

        // Get the prepared info (simplified)
        // In a real implementation, you'd retrieve this
        let new_oid = repo.find_reference(refname)?.target().unwrap_or_else(|| {
            // Fallback or error
            panic!("Missing target after prepare");
        });

        tx.set_target(refname, new_oid, "Distributed transaction commit")?;
        tx.commit()?;

        Ok(())
    }

    fn rollback_prepared(&self, prepared: &[(usize, &String, Result<(), git2::Error>)]) {
        for (repo_idx, refname, _) in prepared {
            let repo = &self.repositories[*repo_idx];
            let _ = self.rollback_repo_transaction(repo, refname);
        }
    }

    fn rollback_repo_transaction(
        &self,
        repo: &Repository,
        refname: &str,
    ) -> Result<(), git2::Error> {
        let mut tx = repo.transaction()?;
        tx.lock(&[refname], false)?;

        // Rollback logic depends on stored state
        // This is simplified
        tx.rollback()?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum DistributedTransactionError {
    PreparationFailed {
        repo_idx: usize,
        refname: String,
        source: git2::Error,
    },
    CommitFailed {
        repo_idx: usize,
        refname: String,
        source: git2::Error,
    },
}

impl std::fmt::Display for DistributedTransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributedTransactionError::PreparationFailed { repo_idx, refname, source } => {
                write!(
                    f,
                    "Preparation failed for repo {} ref {}: {}",
                    repo_idx, refname, source
                )
            },
            DistributedTransactionError::CommitFailed { repo_idx, refname, source } => {
                write!(
                    f,
                    "Commit failed for repo {} ref {}: {}",
                    repo_idx, refname, source
                )
            },
        }
    }
}

impl std::error::Error for DistributedTransactionError {}
```

### Async Transaction Support

```rust
use tokio::sync::Mutex;

pub struct AsyncTransactionManager {
    repo: Arc<Mutex<Repository>>,
}

impl AsyncTransactionManager {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo: Arc::new(Mutex::new(repo)),
        }
    }

    pub async fn async_atomic_update(
        &self,
        refname: String,
        new_oid: Oid,
        message: String,
    ) -> Result<(), git2::Error> {
        let mut repo_guard = self.repo.lock().await;
        let repo = &mut *repo_guard;

        // Run blocking operation in thread pool
        tokio::task::spawn_blocking(move || {
            let mut tx = repo.transaction()?;
            tx.lock(&[&refname], false)?;
            tx.set_target(&refname, new_oid, &message)?;
            tx.commit()?;
            Ok::<_, git2::Error>(())
        })
        .await?
    }

    pub async fn async_transaction<F, T>(
        &self,
        operation: F,
    ) -> Result<T, git2::Error>
    where
        F: FnOnce(&mut Repository) -> Result<T, git2::Error> + Send + 'static,
        T: Send + 'static,
    {
        let mut repo_guard = self.repo.lock().await;
        let repo = &mut *repo_guard;

        tokio::task::spawn_blocking(move || {
            // Create transaction
            let mut tx = repo.transaction()?;

            // Execute operation with transaction
            let result = operation(repo)?;

            // Commit transaction
            tx.commit()?;

            Ok(result)
        })
        .await?
        .map_err(|e| {
            // Join error
            git2::Error::from_str(&format!("Task failed: {}", e))
        })?
        .map_err(|e| {
            // Operation error
            e
        })
    }
}
```

### Monitoring and Metrics

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct TransactionMetrics {
    pub total_transactions: AtomicU64,
    pub successful_transactions: AtomicU64,
    pub failed_transactions: AtomicU64,
    pub total_duration: AtomicU64,
    pub active_transactions: AtomicU64,
}

impl TransactionMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            total_transactions: AtomicU64::new(0),
            successful_transactions: AtomicU64::new(0),
            failed_transactions: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
            active_transactions: AtomicU64::new(0),
        })
    }

    pub fn record_transaction_start(&self) -> Instant {
        self.active_transactions.fetch_add(1, Ordering::Relaxed);
        self.total_transactions.fetch_add(1, Ordering::Relaxed);
        Instant::now()
    }

    pub fn record_transaction_success(&self, start_time: Instant) {
        self.successful_transactions.fetch_add(1, Ordering::Relaxed);
        self.record_transaction_end(start_time);
    }

    pub fn record_transaction_failure(&self, start_time: Instant) {
        self.failed_transactions.fetch_add(1, Ordering::Relaxed);
        self.record_transaction_end(start_time);
    }

    fn record_transaction_end(&self, start_time: Instant) {
        let duration = start_time.elapsed().as_millis() as u64;
        self.total_duration.fetch_add(duration, Ordering::Relaxed);
        self.active_transactions.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> TransactionStats {
        let total = self.total_transactions.load(Ordering::Relaxed);
        let successful = self.successful_transactions.load(Ordering::Relaxed);
        let failed = self.failed_transactions.load(Ordering::Relaxed);
        let total_duration = self.total_duration.load(Ordering::Relaxed);
        let active = self.active_transactions.load(Ordering::Relaxed);

        TransactionStats {
            total_transactions: total,
            successful_transactions: successful,
            failed_transactions: failed,
            success_rate: if total > 0 { successful as f64 / total as f64 } else { 0.0 },
            average_duration_ms: if successful > 0 { total_duration / successful } else { 0 },
            active_transactions: active,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub success_rate: f64,
    pub average_duration_ms: u64,
    pub active_transactions: u64,
}

pub struct MonitoredTransactionManager {
    metrics: Arc<TransactionMetrics>,
}

impl MonitoredTransactionManager {
    pub fn new(metrics: Arc<TransactionMetrics>) -> Self {
        Self { metrics }
    }

    pub fn execute_with_monitoring<F, T>(&self, operation: F) -> Result<T, git2::Error>
    where
        F: FnOnce() -> Result<T, git2::Error>,
    {
        let start_time = self.metrics.record_transaction_start();

        match operation() {
            Ok(result) => {
                self.metrics.record_transaction_success(start_time);
                Ok(result)
            },
            Err(e) => {
                self.metrics.record_transaction_failure(start_time);
                Err(e)
            }
        }
    }

    pub fn get_stats(&self) -> TransactionStats {
        self.metrics.get_stats()
    }
}

// Usage example
fn monitored_transaction_example() -> Result<(), git2::Error> {
    let metrics = TransactionMetrics::new();
    let monitor = MonitoredTransactionManager::new(metrics.clone());

    let repo = Repository::open(".")?;

    monitor.execute_with_monitoring(|| {
        let mut tx = repo.transaction()?;
        tx.lock(&["refs/heads/main"], false)?;
        tx.set_target("refs/heads/main", new_commit_oid(), "Update")?;
        tx.commit()
    })?;

    let stats = metrics.get_stats();
    println!("Transaction stats: {:?}", stats);

    Ok(())
}
```

## Conclusion

This research document provides a comprehensive overview of atomic transaction patterns in git2-rs, covering:

1. **Core Transaction APIs** - Understanding the basic `Transaction` type and its methods
2. **Atomic Patterns** - Single and multi-reference update patterns
3. **Error Handling** - Robust error handling with automatic rollback
4. **Staging and Recovery** - Techniques for handling incomplete transactions
5. **Best Practices** - Proven patterns for reliable transactions
6. **Common Pitfalls** - Avoiding race conditions, deadlocks, and memory leaks
7. **Advanced Patterns** - Including distributed transactions and async support

The key principles for successful Git transactions are:

- **Always lock references before modifying them**
- **Validate state before committing changes**
- **Handle rollbacks gracefully**
- **Use timeouts for long-running operations**
- **Monitor transaction health**
- **Recover from incomplete transactions**

These patterns provide a foundation for building reliable, concurrent Git operations in Rust using git2-rs.

---

*Note: This research document is based on git2-rs version 0.18+ and libgit2 best practices. Always refer to the latest official documentation for API updates.*