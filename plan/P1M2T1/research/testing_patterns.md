# Testing Patterns and Best Practices for Rust Code Using git2-rs

## Overview

This document outlines comprehensive testing patterns and best practices for Rust projects that use the git2-rs library to interact with Git repositories. The focus is on creating isolated, reliable tests that can be run in parallel without interference.

## 1. Using tempfile Crate for Test Repositories

### Core Dependencies

```toml
# Cargo.toml
[dev-dependencies]
tempfile = "3.8"          # For temporary directories and files
pretty_assertions = "1.4"  # For better test failure messages
anyhow = "1.0"            # For error handling in tests
```

### Temporary Directory Patterns

```rust
use tempfile::TempDir;
use git2::Repository;
use std::path::Path;

fn create_temp_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let repo = Repository::init(&temp_dir).expect("Failed to initialize git repository");
    (temp_dir, repo)
}

#[test]
fn test_basic_operations() {
    let (_temp_dir, repo) = create_temp_repo();

    // Test basic repository operations
    assert!(repo.is_bare());
    assert!(!repo.is_empty().unwrap());
}
```

### Named Temporary Files for Specific Content

```rust
use tempfile::NamedTempFile;

#[test]
fn test_with_gitconfig() {
    let temp_dir = TempDir::new().unwrap();
    let gitconfig_path = temp_dir.path().join(".gitconfig");

    // Create a named temp file for git config
    let mut config_file = NamedTempFile::new_in(&temp_dir).unwrap();
    write!(config_file, "[user]\n    name = Test User\n    email = test@example.com").unwrap();

    // Use the config file in tests
    let repo = Repository::init(&temp_dir).unwrap();
    // ... rest of test
}
```

## 2. Creating Test Fixtures for Git Operations

### Repository Fixture Helpers

```rust
use git2::{Oid, Signature};
use std::fs;
use std::path::Path;

pub struct TestRepoFixture {
    pub temp_dir: TempDir,
    pub repo: Repository,
    pub author: Signature<'static>,
    pub committer: Signature<'static>,
}

impl TestRepoFixture {
    pub fn new() -> Self {
        let (temp_dir, repo) = create_temp_repo();
        let author = Signature::now("Test Author", "test@example.com").unwrap();
        let committer = Signature::now("Test Committer", "committer@example.com").unwrap();

        Self {
            temp_dir,
            repo,
            author,
            committer,
        }
    }

    pub fn create_initial_commit(&self) -> Oid {
        let mut index = self.repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();

        let tree = self.repo.find_tree(tree_id).unwrap();
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &self.author,
            &self.committer,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();

        commit_id
    }

    pub fn add_file(&self, filename: &str, content: &str) -> Oid {
        let file_path = self.temp_dir.path().join(filename);
        fs::write(&file_path, content).unwrap();

        let mut index = self.repo.index().unwrap();
        index.add_path(Path::new(filename)).unwrap();
        index.write_tree().unwrap();

        let tree = self.repo.find_tree(index.write_tree().unwrap()).unwrap();
        let parent = self.repo.head().unwrap().target().unwrap();

        self.repo.commit(
            Some("HEAD"),
            &self.author,
            &self.committer,
            &format!("Add {}", filename),
            &tree,
            &[&self.repo.find_commit(parent).unwrap()],
        ).unwrap()
    }
}
```

### Branch and Tag Fixtures

```rust
impl TestRepoFixture {
    pub fn create_branch(&self, branch_name: &str, target_commit: Oid) {
        self.repo.branch(
            branch_name,
            &self.repo.find_commit(target_commit).unwrap(),
            false,
        ).unwrap();
    }

    pub fn create_tag(&self, tag_name: &str, target_commit: Oid) {
        self.repo.tag(
            tag_name,
            &self.repo.find_commit(target_commit).unwrap(),
            &self.author,
            "Test tag",
            false,
        ).unwrap();
    }

    pub fn create_remote(&self, name: &str, url: &str) {
        self.repo.remote(name, url).unwrap();
    }
}
```

## 3. Testing Error Conditions in git2 Operations

### Error Testing Patterns

```rust
#[test]
fn test_repository_init_errors() {
    let temp_dir = TempDir::new().unwrap();

    // Test initializing in a non-empty directory
    fs::write(temp_dir.path().join("existing.txt"), "content").unwrap();

    let result = Repository::init(&temp_dir);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), git2::ErrorCode::GenericError);
}

#[test]
fn test_invalid_operations() {
    let (_temp_dir, repo) = create_temp_repo();

    // Test operations that should fail on empty repo
    let result = repo.head();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), git2::ErrorCode::InvalidSpec);

    // Test finding non-existent commit
    let result = repo.find_commit(git2::Oid::from_str("0000000000000000000000000000000000000000").unwrap());
    assert!(result.is_err());
}
```

### Custom Error Testing Helpers

```rust
pub fn assert_git_error<T>(result: Result<T, git2::Error>, expected_code: git2::ErrorCode) {
    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(e) => {
            assert_eq!(e.code(), expected_code, "Expected error code {:?}, got {:?}", expected_code, e.code());
            if let Some(message) = e.message() {
                println!("Error message: {}", message);
            }
        }
    }
}

#[test]
fn test_error_assertions() {
    let (_temp_dir, repo) = create_temp_repo();

    assert_git_error(repo.head(), git2::ErrorCode::InvalidSpec);
    assert_git_error(repo.find_commit(git2::Oid::from_str("invalid").unwrap()), git2::ErrorCode::Invalid);
}
```

## 4. Integration Test Patterns for Repository Wrappers

### Integration Test Structure

```rust
// tests/integration_tests.rs
mod fixtures;
mod error_tests;
mod operation_tests;

use fixtures::TestRepoFixture;

#[test]
fn test_full_workflow() {
    let fixture = TestRepoFixture::new();

    // Create initial commit
    let initial_commit = fixture.create_initial_commit();

    // Add a file
    let file_commit = fixture.add_file("test.txt", "Hello, World!");

    // Create branch
    fixture.create_branch("feature", file_commit);

    // Create tag
    fixture.create_tag("v1.0.0", file_commit);

    // Verify all operations succeeded
    assert!(fixture.repo.head().is_ok());
    assert!(fixture.repo.branches(None).unwrap().count() >= 2);
    assert!(fixture.repo.tag_names(None).unwrap().iter().any(|t| t == Some("v1.0.0")));
}
```

### Async Testing Patterns

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio::task;

    #[tokio::test]
    async fn test_concurrent_operations() {
        let fixture = TestRepoFixture::new();
        let repo = fixture.repo.clone();

        // Create multiple commits concurrently
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let repo = repo.clone();
                task::spawn(async move {
                    let content = format!("File {}", i);
                    let mut index = repo.index().unwrap();
                    let path = format!("file_{}.txt", i);
                    std::fs::write(repo.path().parent().unwrap().join(&path), &content).unwrap();
                    index.add_path(Path::new(&path)).unwrap();
                    index.write_tree().unwrap()
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles).await;
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
```

## 5. Snapshot Testing for Git State

### Setup for Snapshot Testing

```toml
[dev-dependencies]
insta = "1.34"  # For snapshot testing
```

### Snapshot Test Implementation

```rust
#[test]
fn test_repository_state_snapshot() {
    let fixture = TestRepoFixture::new();

    // Create repository state
    fixture.create_initial_commit();
    fixture.add_file("src/main.rs", "fn main() {}");
    fixture.add_file("README.md", "# Test Project");

    // Get repository state
    let state = get_repository_state(&fixture.repo);

    // Snapshot test
    insta::assert_snapshot!("repository_state", state);
}

fn get_repository_state(repo: &Repository) -> String {
    let mut output = String::new();

    // Get branches
    output.push_str("BRANCHES:\n");
    for branch in repo.branches(None).unwrap() {
        let branch = branch.unwrap();
        let name = branch.name().unwrap().unwrap_or("(unnamed)");
        let is_head = branch.is_head();
        output.push_str(&format!("- {} {}\n", name, if is_head { "(HEAD)" } else { "" }));
    }

    // Get tags
    output.push_str("\nTAGS:\n");
    for tag in repo.tag_names(None).unwrap() {
        output.push_str(&format!("- {}\n", tag.unwrap()));
    }

    // Get remotes
    output.push_str("\nREMOTES:\n");
    for remote in repo.remotes().unwrap() {
        let remote = remote.unwrap();
        output.push_str(&format!("- {}\n", remote));
    }

    output
}
```

### Binary Content Snapshot Testing

```rust
#[test]
fn test_commit_snapshot() {
    let fixture = TestRepoFixture::new();
    let commit_id = fixture.create_initial_commit();

    let commit = fixture.repo.find_commit(commit_id).unwrap();
    let commit_info = format!(
        "Commit: {}\nAuthor: {}\nDate: {}\nMessage: {}\n",
        commit.id(),
        commit.author().name().unwrap_or("(unknown)"),
        commit.time().to_string(),
        commit.message().unwrap_or("(no message)")
    );

    insta::assert_snapshot!("commit_info", commit_info);
}
```

## 6. Best Practices for Cleanup

### Automatic Cleanup

```rust
use std::sync::Mutex;

// Global test state cleanup
pub struct TestCleanup {
    temp_dirs: Mutex<Vec<tempfile::TempDir>>,
}

impl TestCleanup {
    pub fn new() -> Self {
        Self {
            temp_dirs: Mutex::new(Vec::new()),
        }
    }

    pub fn add_temp_dir(&self, temp_dir: tempfile::TempDir) {
        self.temp_dirs.lock().unwrap().push(temp_dir);
    }

    pub fn cleanup_all(&self) {
        let _ = self.temp_dirs.lock().unwrap().clear();
    }
}

static CLEANUP: TestCleanup = TestCleanup::new();

// Drop trait implementation for automatic cleanup
impl Drop for TestRepoFixture {
    fn drop(&mut self) {
        // Temporary directories are automatically cleaned up
        println!("Test repository cleaned up automatically");
    }
}
```

### Resource Management

```rust
// Test resource manager
pub struct TestResourceManager {
    temp_dirs: Vec<Weak<tempfile::TempDir>>,
}

impl TestResourceManager {
    pub fn new() -> Self {
        Self {
            temp_dirs: Vec::new(),
        }
    }

    pub fn create_temp_dir(&mut self) -> Arc<tempfile::TempDir> {
        let temp_dir = Arc::new(TempDir::new().unwrap());
        self.temp_dirs.push(Arc::downgrade(&temp_dir));
        temp_dir
    }

    pub fn cleanup(&mut self) {
        self.temp_dirs.clear();
    }
}

// Ensure proper cleanup even on test panics
#[test]
fn test_with_cleanup() {
    let mut manager = TestResourceManager::new();
    let temp_dir = manager.create_temp_dir();

    // Test code here
    let repo = Repository::init(&temp_dir).unwrap();

    // Cleanup happens when manager goes out of scope
}
```

## 7. Complete Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use git2::{Repository, Oid, Signature};

    fn setup_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(&temp_dir).unwrap();

        // Configure git user
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    #[test]
    fn test_commit_and_branch_operations() {
        let (_temp_dir, repo) = setup_test_repo();

        // Create initial commit
        let mut index = repo.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let author = Signature::now("Test User", "test@example.com").unwrap();
        let committer = Signature::now("Test User", "test@example.com").unwrap();

        let commit_id = repo.commit(
            Some("HEAD"),
            &author,
            &committer,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();

        // Verify commit exists
        let commit = repo.find_commit(commit_id).unwrap();
        assert_eq!(commit.message().unwrap(), "Initial commit");

        // Create and checkout branch
        repo.branch("main", &commit, false).unwrap();
        repo.set_head("refs/heads/main").unwrap();
    }

    #[test]
    fn test_error_handling() {
        let (_temp_dir, repo) = setup_test_repo();

        // Test finding non-existent commit
        let invalid_oid = Oid::from_str("0000000000000000000000000000000000000000").unwrap();
        let result = repo.find_commit(invalid_oid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), git2::ErrorCode::NotFound);
    }

    #[test]
    fn test_file_operations() {
        let (_temp_dir, repo) = setup_test_repo();

        // Add a file
        let file_path = repo.path().parent().unwrap().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        // Stage the file
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();

        // Commit the file
        let tree = repo.find_tree(tree_id).unwrap();
        let author = Signature::now("Test User", "test@example.com").unwrap();
        let committer = Signature::now("Test User", "test@example.com").unwrap();

        repo.commit(
            Some("HEAD"),
            &author,
            &committer,
            "Add test file",
            &tree,
            &[],
        ).unwrap();

        // Verify file exists in working directory
        assert!(std::fs::metadata(&file_path).is_ok());
    }
}
```

## 8. Testing Performance Considerations

```rust
#[test]
fn test_large_repositories() {
    let fixture = TestRepoFixture::new();

    // Create many commits to test performance
    for i in 0..1000 {
        let content = format!("File {}\n{}", i, "x".repeat(1024));
        fixture.add_file(&format!("file_{}.txt", i), &content);
    }

    // Test that operations still work on large repo
    let head = fixture.repo.head().unwrap();
    let commit = fixture.repo.find_commit(head.target().unwrap()).unwrap();
    assert!(commit.message().unwrap().starts_with("Add file"));
}

#[test]
fn test_concurrent_access() {
    let (_temp_dir, repo) = create_temp_repo();

    // Spawn multiple threads that access the repository
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let repo = repo.clone();
            std::thread::spawn(move || {
                let mut index = repo.index().unwrap();
                let _ = index.write_tree();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Conclusion

These testing patterns provide a comprehensive approach to testing git2-rs based applications:

1. **Isolation**: Use `tempfile` to create completely isolated test environments
2. **Fixtures**: Create reusable test fixtures that model common repository states
3. **Error Testing**: Systematically test error conditions and edge cases
4. **Integration Tests**: Test full workflows with proper async support
5. **Snapshots**: Use snapshot testing to verify repository state over time
6. **Cleanup**: Ensure proper resource management with automatic cleanup

By following these patterns, you can build a robust test suite that gives confidence in your Git-related code while maintaining fast, parallel test execution.