# git2-rs Git Reset Operations Research

## Overview
This document provides comprehensive research on implementing git reset operations using the git2-rs library in Rust. The git2-rs library is a binding to libgit2, providing idiomatic Rust interfaces for Git operations.

## 1. Core API Methods for Git Reset

### Repository Reset Methods

The `Repository` object in git2-rs provides several methods for performing reset operations:

#### 1.1 Main Reset Method: `reset()`

```rust
pub fn reset(&self, object: &Object, reset_type: ResetType, checkout_strategy: Option<CheckoutBuilder>) -> Result<(), Error>
```

- **Parameters:**
  - `object`: The git object (commit, tree, etc.) to reset to
  - `reset_type`: The type of reset operation (Soft, Mixed, Hard)
  - `checkout_strategy`: Optional checkout configuration for hard resets

#### 1.2 Convenience Methods

```rust
// Reset to a specific commit with different modes
pub fn reset_hard(&self, commit: &Object, strategy: Option<CheckoutBuilder>) -> Result<(), Error>
pub fn reset_soft(&self, commit: &Object) -> Result<(), Error>
pub fn reset_default(&self, commit: &Object) -> Result<(), Error>
```

### ResetType Enum

```rust
pub enum ResetType {
    Soft,    // Move HEAD, keep index and working tree
    Mixed,   // Move HEAD and reset index, keep working tree (default)
    Hard,    // Move HEAD, reset index and working tree
}
```

## 2. Complete Implementation Examples

### Example 1: Basic Reset to Specific Commit

```rust
use git2::{Repository, ObjectType, ResetType};

fn reset_to_commit(repo: &Repository, commitish: &str, reset_type: ResetType) -> Result<(), git2::Error> {
    // Parse the commit reference
    let obj = repo.revparse_single(commitish)?;
    let commit = obj.peel_to_commit()?;

    // Perform the reset
    repo.reset(&commit, reset_type, None)?;

    Ok(())
}

// Usage examples
fn main() -> Result<(), git2::Error> {
    let repo = Repository::open(".")?;

    // Soft reset - keeps changes in index
    reset_to_commit(&repo, "HEAD~1", ResetType::Soft)?;

    // Mixed reset (default) - keeps changes in working tree
    reset_to_commit(&repo, "HEAD~2", ResetType::Mixed)?;

    // Hard reset - discards all changes
    reset_to_commit(&repo, "HEAD~3", ResetType::Hard)?;

    Ok(())
}
```

### Example 2: Advanced Reset with Error Handling

```rust
use git2::{Repository, ObjectType, ResetType, Oid};
use std::path::Path;

pub struct GitResetter {
    repo: Repository,
}

impl GitResetter {
    pub fn new(repo_path: &Path) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        Ok(Self { repo })
    }

    pub fn reset_soft(&self, commitish: &str) -> Result<String, git2::Error> {
        let obj = self.repo.revparse_single(commitish)?;
        let commit = obj.peel_to_commit()?;

        self.repo.reset(&commit, ResetType::Soft, None)?;

        Ok(format!("Soft reset to commit: {}", commit.id()))
    }

    pub fn reset_mixed(&self, commitish: &str) -> Result<String, git2::Error> {
        let obj = self.repo.revparse_single(commitish)?;
        let commit = obj.peel_to_commit()?;

        self.repo.reset(&commit, ResetType::Mixed, None)?;

        Ok(format!("Mixed reset to commit: {}", commit.id()))
    }

    pub fn reset_hard(&self, commitish: &str) -> Result<String, git2::Error> {
        let obj = self.repo.revparse_single(commitish)?;
        let commit = obj.peel_to_commit()?;

        // For hard reset, use default checkout strategy
        self.repo.reset(&commit, ResetType::Hard, None)?;

        Ok(format!("Hard reset to commit: {}", commit.id()))
    }

    pub fn reset_to_oid(&self, oid: Oid, reset_type: ResetType) -> Result<String, git2::Error> {
        let commit = self.repo.find_commit(oid)?;
        self.repo.reset(&commit, reset_type, None)?;

        Ok(format!("Reset to commit: {}", oid))
    }
}

// Usage
fn main() -> Result<(), git2::Error> {
    let resetter = GitResetter::new(".")?;

    // Reset by commit reference
    let result = resetter.reset_soft("HEAD^")?;
    println!("{}", result);

    // Reset by commit OID
    let head = resetter.repo.head()?;
    let head_oid = head.target().unwrap();
    resetter.reset_to_oid(head_oid, ResetType::Hard)?;

    Ok(())
}
```

### Example 3: Branch Reset Operations

```rust
use git2::{Repository, ResetType, Reference};

fn reset_branch(repo: &Repository, branch_name: &str, commitish: &str) -> Result<(), git2::Error> {
    // Get the commit to reset to
    let obj = repo.revparse_single(commitish)?;
    let commit = obj.peel_to_commit()?;

    // Get the branch reference
    let branch = repo.find_reference(branch_name)?;

    // Set branch HEAD to the target commit
    branch.set_target(commit.id(), "reset branch")?;

    // Perform reset on the branch
    repo.reset(&commit, ResetType::Mixed, None)?;

    Ok(())
}

fn reset_current_branch(repo: &Repository, commitish: &str, reset_type: ResetType) -> Result<(), git2::Error> {
    // Get current HEAD reference
    let head = repo.head()?;
    let head_ref = head.name().unwrap();

    // Parse the target commit
    let obj = repo.revparse_single(commitish)?;
    let commit = obj.peel_to_commit()?;

    // Reset HEAD to the target commit
    repo.reset(&commit, reset_type, None)?;

    println!("Reset branch '{}' to commit: {}", head_ref, commit.id());
    Ok(())
}
```

## 3. Working with Git Objects

### Oid, Commit, and Reference Types

```rust
use git2::{Oid, Commit, Reference};

// Working with OIDs
fn commit_from_oid(repo: &Repository, oid: Oid) -> Result<Commit, git2::Error> {
    repo.find_commit(oid)
}

// Working with References
fn get_head_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let head = repo.head()?;
    let head_oid = head.target().unwrap();
    repo.find_commit(head_oid)
}

// Parse commit references
fn resolve_commitish(repo: &Repository, commitish: &str) -> Result<Commit, git2::Error> {
    let obj = repo.revparse_single(commitish)?;
    obj.peel_to_commit()
}
```

## 4. Reset Mode Details

### 4.1 -- Soft Reset

```rust
fn soft_reset_example(repo: &Repository) -> Result<(), git2::Error> {
    // Get target commit (e.g., one commit back)
    let obj = repo.revparse_single("HEAD~1")?;
    let commit = obj.peel_to_commit()?;

    // Soft reset - only moves HEAD
    repo.reset(&commit, ResetType::Soft, None)?;

    println!("Soft reset completed. Changes are still in index.");

    // Verify reset
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;
    assert_eq!(head_commit.id(), commit.id());

    Ok(())
}
```

### 4.2 -- Mixed Reset (Default)

```rust
fn mixed_reset_example(repo: &Repository) -> Result<(), git2::Error> {
    // Get target commit
    let obj = repo.revparse_single("origin/main")?;
    let commit = obj.peel_to_commit()?;

    // Mixed reset - moves HEAD and resets index
    repo.reset(&commit, ResetType::Mixed, None)?;

    println!("Mixed reset completed. Working tree unchanged, index reset.");

    // Check status to verify
    let status = repo.status(Some(git2::StatusOptions::new()))?;
    println!("Status after mixed reset: {} files modified", status.len());

    Ok(())
}
```

### 4.3 -- Hard Reset

```rust
use git2::CheckoutBuilder;

fn hard_reset_example(repo: &Repository) -> Result<(), git2::Error> {
    // Configure checkout strategy for hard reset
    let mut checkout = CheckoutBuilder::new();
    checkout.force(); // Overwrite untracked files

    // Get target commit
    let obj = repo.revparse_single("v1.0.0")?;
    let commit = obj.peel_to_commit()?;

    // Hard reset - moves HEAD, resets index and working tree
    repo.reset(&commit, ResetType::Hard, Some(&mut checkout))?;

    println!("Hard reset completed. All changes discarded.");

    // Verify no working directory changes
    let status = repo.status(Some(git2::StatusOptions::new()))?;
    assert_eq!(status.len(), 0);

    Ok(())
}
```

## 5. Error Handling and Validation

```rust
pub fn safe_reset(repo: &Repository, commitish: &str, reset_type: ResetType) -> Result<String, git2::Error> {
    // Validate commitish exists
    match repo.revparse_single(commitish) {
        Ok(obj) => {
            let commit = obj.peel_to_commit()?;

            // Check for uncommitted changes if doing hard reset
            if reset_type == ResetType::Hard {
                let status = repo.status(Some(git2::StatusOptions::new()))?;
                if !status.is_empty() {
                    return Err(git2::Error::from_str(
                        "Cannot perform hard reset with uncommitted changes"
                    ));
                }
            }

            // Perform reset
            repo.reset(&commit, reset_type, None)?;

            Ok(format!("Successfully reset to commit: {}", commit.id()))
        }
        Err(e) => Err(git2::Error::from_str(&format!("Invalid commitish: {}", e))),
    }
}
```

## 6. Practical Examples from Production Code

### Example: Git Status-like Reset Tool

```rust
use std::env;
use git2::{Repository, ResetType};

fn main() -> Result<(), git2::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <commitish> <soft|mixed|hard>", args[0]);
        std::process::exit(1);
    }

    let commitish = &args[1];
    let reset_type = match args[2].as_str() {
        "soft" => ResetType::Soft,
        "hard" => ResetType::Hard,
        "mixed" | _ => ResetType::Mixed,
    };

    let repo = Repository::open(".")?;
    let obj = repo.revparse_single(commitish)?;
    let commit = obj.peel_to_commit()?;

    // Show what will be reset
    println!("Resetting to commit: {}", commit.id());

    repo.reset(&commit, reset_type, None)?;

    println!("Reset completed successfully");

    Ok(())
}
```

## 7. Additional Resources and Sources

### Official Documentation
- [git2-rs Documentation on docs.rs](https://docs.rs/git2/latest/git2/)
- [git2-rs GitHub Repository](https://github.com/rust-lang/git2-rs)
- [libgit2 Documentation](https://libgit2.org/libgit2/) (C API that git2-rs wraps)

### API Reference Links
- [Repository::reset method](https://docs.rs/git2/latest/git2/struct.Repository.html#method.reset)
- [ResetType enum](https://docs.rs/git2/latest/git2/enum.ResetType.html)
- [CheckoutBuilder for hard resets](https://docs.rs/git2/latest/git2/struct.CheckoutBuilder.html)

### Example Repositories
- [git2-rs examples directory](https://github.com/rust-lang/git2-rs/tree/master/examples)
- [git2-rs reset example](https://github.com/rust-lang/git2-rs/blob/master/examples/reset.rs) (if available)

### Search Terms for Further Research
- `git2 reset soft mixed hard Rust`
- `git2-rs repository reset implementation`
- `libgit2 reset API documentation`
- `git checkout vs git reset in git2-rs`

## 8. Best Practices

1. **Always validate commit references** before performing resets
2. **Check for uncommitted changes** before hard resets
3. **Use appropriate error handling** for all git operations
4. **Consider the working directory state** when choosing reset type
5. **Make backups** before performing hard resets on important branches

## 9. Common Pitfalls

1. **Forgetting to peel objects to commits** before reset
2. **Not handling uncommitted files** in hard resets
3. **Using relative references** without proper validation
4. **Ignoring error codes** from git operations
5. **Not setting proper checkout strategies** for hard resets

This research provides a comprehensive foundation for implementing git reset operations in Rust using the git2-rs library.