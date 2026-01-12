# Testing Git Operations and Repository State

## Overview

Testing git operations requires careful handling of repository initialization, file staging, commits, and state verification. The combination of `git2-rs` (Rust bindings for libgit2) and `tempfile`/`assert_fs` provides a robust foundation for integration testing git workflows.

## Using git2-rs

### Installation

```toml
[dependencies]
git2 = "0.20"

[dev-dependencies]
tempfile = "3"
assert_fs = "1"
```

### Documentation

- [git2-rs GitHub](https://github.com/rust-lang/git2-rs)
- [git2 crate documentation](https://docs.rs/git2/latest/)
- [Repository struct](https://docs.rs/git2/latest/git2/struct.Repository.html)

## Basic Repository Operations

### Initializing a Test Repository

```rust
use git2::Repository;
use tempfile::TempDir;
use std::path::Path;

#[test]
fn test_with_git_repo() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory
    let tmpdir = TempDir::new()?;
    let repo_path = tmpdir.path();

    // Initialize repository
    let repo = Repository::init(repo_path)?;

    // Verify it exists
    assert!(repo_path.join(".git").exists());

    Ok(())
}
```

### Creating Test Files and Commits

```rust
use git2::{Repository, Signature};
use std::fs;

#[test]
fn test_create_commit() -> Result<(), Box<dyn std::error::Error>> {
    let tmpdir = tempfile::TempDir::new()?;
    let repo = Repository::init(tmpdir.path())?;

    // Create a test file
    let file_path = tmpdir.path().join("test.txt");
    fs::write(&file_path, "hello world")?;

    // Add file to index
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("test.txt"))?;
    index.write()?;

    // Create a commit
    let signature = Signature::now("Test User", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],  // Empty parent list for first commit
    )?;

    Ok(())
}
```

## Common Testing Patterns

### Pattern 1: Multi-Step Workflow with Verification

```rust
use git2::{Repository, Signature};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_git_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let tmpdir = TempDir::new()?;
    let repo = Repository::init(tmpdir.path())?;

    // Step 1: Create and commit initial file
    fs::write(tmpdir.path().join("file1.txt"), "content1")?;
    commit_file(&repo, "file1.txt", "First commit")?;

    // Step 2: Modify and commit
    fs::write(tmpdir.path().join("file1.txt"), "updated content")?;
    commit_file(&repo, "file1.txt", "Second commit")?;

    // Step 3: Create and commit new file
    fs::write(tmpdir.path().join("file2.txt"), "content2")?;
    commit_file(&repo, "file2.txt", "Third commit")?;

    // Verify: Check commit history
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut commit_messages = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        if let Some(msg) = commit.message() {
            commit_messages.push(msg.to_string());
        }
    }

    assert_eq!(commit_messages.len(), 3);
    assert_eq!(commit_messages[0], "Third commit");  // Most recent
    assert_eq!(commit_messages[2], "First commit");  // Oldest

    Ok(())
}

fn commit_file(repo: &Repository, path: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new(path))?;
    index.write()?;

    let signature = Signature::now("Test User", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get HEAD commit as parent
    let head = repo.head()?;
    let parent_commit = repo.find_commit(head.target().unwrap())?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}
```

### Pattern 2: Testing Branch Operations

```rust
use git2::{Repository, Signature, BranchType};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_branch_creation() -> Result<(), Box<dyn std::error::Error>> {
    let tmpdir = TempDir::new()?;
    let repo = Repository::init(tmpdir.path())?;

    // Create initial commit on main
    fs::write(tmpdir.path().join("main.txt"), "main content")?;
    let oid = create_commit(&repo, "main.txt", "Initial")?;

    // Create and checkout branch
    let commit = repo.find_commit(oid)?;
    repo.branch("feature", &commit, false)?;

    // Verify branch exists
    let branches = repo.branches(Some(BranchType::Local))?;
    let branch_names: Vec<_> = branches
        .filter_map(|b| b.ok())
        .filter_map(|(b, _)| b.name().ok().flatten().map(|s| s.to_string()))
        .collect();

    assert!(branch_names.contains(&"main".to_string()));
    assert!(branch_names.contains(&"feature".to_string()));

    Ok(())
}

fn create_commit(repo: &Repository, path: &str, message: &str) -> Result<git2::Oid, Box<dyn std::error::Error>> {
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new(path))?;
    index.write()?;

    let signature = Signature::now("Test", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head = repo.head()?;
    let parent_commit = repo.find_commit(head.target().unwrap())?;

    let oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    Ok(oid)
}
```

### Pattern 3: Testing File Status

```rust
use git2::{Repository, Status};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_file_status() -> Result<(), Box<dyn std::error::Error>> {
    let tmpdir = TempDir::new()?;
    let repo = Repository::init(tmpdir.path())?;

    // Create and stage a file
    fs::write(tmpdir.path().join("tracked.txt"), "initial")?;
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("tracked.txt"))?;
    index.write()?;

    // Commit the file
    let signature = git2::Signature::now("Test", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &signature, &signature, "commit", &tree, &[])?;

    // Modify the committed file
    fs::write(tmpdir.path().join("tracked.txt"), "modified")?;

    // Create an untracked file
    fs::write(tmpdir.path().join("untracked.txt"), "new")?;

    // Check status
    let statuses = repo.statuses(None)?;

    for entry in statuses.iter() {
        match entry.path() {
            Some("tracked.txt") => {
                assert!(entry.status().contains(Status::WT_MODIFIED));
            }
            Some("untracked.txt") => {
                assert!(entry.status().contains(Status::WT_NEW));
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Pattern 4: Testing Remote Operations

```rust
use git2::{Repository, Signature};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_clone_repository() -> Result<(), Box<dyn std::error::Error>> {
    // Create source repository
    let source_tmpdir = TempDir::new()?;
    let source_repo = Repository::init_bare(source_tmpdir.path())?;

    // Create destination for clone
    let dest_tmpdir = TempDir::new()?;

    // Note: Cloning requires actual git or libgit2 with network support
    // For unit tests, manual setup is often better:
    let _clone_repo = Repository::init(dest_tmpdir.path())?;

    Ok(())
}
```

## Fixture Pattern for Git Testing

### Reusable Git Fixture Structure

```rust
use git2::{Repository, Signature};
use tempfile::TempDir;
use std::path::PathBuf;

pub struct GitFixture {
    _tempdir: TempDir,
    repo: Repository,
    path: PathBuf,
}

impl GitFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        let repo = Repository::init(&path)?;

        Ok(GitFixture {
            _tempdir: tempdir,
            repo,
            path,
        })
    }

    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn create_commit(&mut self, file: &str, content: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        let file_path = self.path.join(file);
        fs::write(&file_path, content)?;

        let mut index = self.repo.index()?;
        index.add_path(std::path::Path::new(file))?;
        index.write()?;

        let signature = Signature::now("Test", "test@example.com")?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let head = self.repo.head()?;
        let parent_commit = self.repo.find_commit(head.target().unwrap())?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }
}

#[test]
fn test_with_git_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let mut fixture = GitFixture::new()?;

    fixture.create_commit("file1.txt", "content1", "First")?;
    fixture.create_commit("file2.txt", "content2", "Second")?;

    let repo = fixture.repo();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let commit_count = revwalk.count();
    assert_eq!(commit_count, 2);

    Ok(())
}
```

## Integration with CLI Testing

### Testing CLI that Operates on Git Repositories

```rust
use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use git2::Repository;
use std::fs;

#[test]
fn test_git_cli_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Setup git repository
    let repo = Repository::init(temp.path())?;
    fs::write(temp.path().join("test.txt"), "content")?;

    // Create initial commit
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("test.txt"))?;
    index.write()?;

    let sig = git2::Signature::now("Test", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])?;

    // Run CLI command in repository
    let mut cmd = Command::cargo_bin("my-git-tool")?;
    cmd.current_dir(temp.path())
        .arg("log")
        .assert()
        .success();

    // Verify repository state
    temp.child("test.txt").assert(predicates::path::exists());

    temp.close()?;
    Ok(())
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Not Keeping TempDir in Scope

```rust
// WRONG: Tempdir dropped immediately
{
    let _repo = Repository::init(TempDir::new()?.path())?;
}  // TempDir dropped, directory deleted

// CORRECT: Keep tempdir in scope
let tmpdir = TempDir::new()?;
let repo = Repository::init(tmpdir.path())?;
```

### Pitfall 2: First Commit Parent Issues

```rust
// WRONG: First commit always has parent
let parent = repo.find_commit(oid)?;
repo.commit(Some("HEAD"), &sig, &sig, "msg", &tree, &[&parent])?;  // Fails for first commit

// CORRECT: Empty parent array for first commit
repo.commit(Some("HEAD"), &sig, &sig, "msg", &tree, &[])?;
```

### Pitfall 3: Not Flushing Index

```rust
// WRONG: Index changes not saved
let mut index = repo.index()?;
index.add_path(Path::new("file.txt"))?;
// Missing: index.write()?;

// CORRECT: Flush changes to disk
let mut index = repo.index()?;
index.add_path(Path::new("file.txt"))?;
index.write()?;
```

## Best Practices

| Practice | Reason |
|----------|--------|
| Use `TempDir` for all tests | Each test gets isolated repo |
| Keep `TempDir` in test struct | Prevents premature cleanup |
| Flush index after modifications | Ensures git2 sees changes |
| Test complete workflows | Catches integration issues |
| Combine with assert_cmd | Test actual CLI behavior |
| Use fixtures for reusability | Reduces test boilerplate |

## References

- [git2-rs GitHub Repository](https://github.com/rust-lang/git2-rs)
- [git2 Documentation](https://docs.rs/git2/latest/)
- [Repository Struct](https://docs.rs/git2/latest/git2/struct.Repository.html)
- [libgit2 Samples](https://libgit2.org/docs/guides/101-samples/)
- [git2-rs Examples](https://github.com/rust-lang/git2-rs/tree/master/examples)
