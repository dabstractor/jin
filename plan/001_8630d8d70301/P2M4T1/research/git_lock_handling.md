# Git Lock File Handling and Contention Resolution in Test Environments

## Table of Contents
1. [Git Lock File Types and Purposes](#git-lock-file-types-and-purposes)
2. [Lock File Cleanup Strategies](#lock-file-cleanup-strategies)
3. [Pre-test Cleanup Patterns](#pre-test-cleanup-patterns)
4. [git2 Crate Lock Handling](#git2-crate-lock-handling)
5. [Testing with Git Repositories](#testing-with-git-repositories)
6. [Code Examples](#code-examples)
7. [Common Failure Modes and Solutions](#common-failure-modes-and-solutions)
8. [Recommended Practices](#recommended-practices)

## Git Lock File Types and Purposes

Git uses lock files to ensure atomic operations and prevent concurrent modifications that could corrupt repository data. When Git needs to modify a critical file, it follows this pattern:
1. Create a `.lock` file
2. Write the new contents to the lock file
3. Rename the lock file to replace the original file

This ensures **atomic writes** and prevents multiple Git processes from modifying the same file simultaneously.

### Common Git Lock File Types

| Lock File | Purpose | Location | Operations That Create It |
|-----------|---------|----------|---------------------------|
| **index.lock** | Locks the staging area/index during modifications | `.git/index.lock` | `git add`, `git reset`, `git commit` |
| **config.lock** | Locks the configuration file during config changes | `.git/config.lock` | `git config` |
| **refs.lock** | Locks references during branch/tag operations | `.git/refs/heads/<branch>.lock` | `git branch`, `git tag`, `git checkout` |
| **packed-refs.lock** | Locks the packed references file | `.git/packed-refs.lock` | `git pack-refs`, garbage collection |
| **HEAD.lock** | Locks the HEAD reference | `.git/HEAD.lock` | `git checkout`, `git merge`, `git rebase` |
| **FETCH_HEAD.lock** | Locks during fetch operations | `.git/FETCH_HEAD.lock` | `git fetch` |
| **MERGE_HEAD.lock** | Locks during merge operations | `.git/MERGE_HEAD.lock` | `git merge` |
| **NOTES_LOCK** | Locks during git notes operations | `.git/NOTES_LOCK` | `git notes` |

### Key Documentation Sources

- [Git API Lockfile Documentation](https://git-scm.com/docs/api-lockfile)
- [Understanding Git's index.lock](https://www.pluralsight.com/resources/blog/guides/understanding-and-using-gits-indexlock-file)
- [Azure Repos: Git index.lock](https://learn.microsoft.com/en-us/azure/devops/repos/git/git-index-lock?view=azure-devops)

## Lock File Cleanup Strategies

### 1. Manual Deletion (Use with Caution)

```bash
# Remove specific lock files
rm .git/index.lock
rm .git/config.lock
rm .git/refs/heads/main.lock

# Remove all lock files (not recommended)
find .git -name "*.lock" -type f -delete
```

**Risk**: Deleting while another process is active can corrupt the repository. Only use when you're certain no Git operations are running.

### 2. Git's Built-in Cleanup

```bash
# Let Git clean up after itself if possible
git gc  # Garbage collection may remove stale locks

# Use git maintenance if available (Git 2.32+)
git maintenance run
```

### 3. Safe Cleanup with Process Verification

```bash
# First, check for running Git processes
ps aux | grep git

# If no processes are running, remove locks
if [ -z "$(ps aux | grep git)" ]; then
    find .git -name "*.lock" -type f -delete
fi
```

### 4. Automated Cleanup in CI/CD

```yaml
# GitHub Actions example
- name: Clean up Git lock files
  run: |
    # Check for stale locks and remove if safe
    if [ -f .git/index.lock ] && ! pgrep -f "git.*$(pwd)"; then
      rm .git/index.lock
    fi
    # Repeat for other lock types as needed
```

## Pre-test Cleanup Patterns

### Pattern 1: Fresh Repository per Test

Create a completely new repository for each test to ensure no stale locks exist:

```rust
use tempfile::TempDir;
use git2::Repository;

fn setup_test_repo() -> (Repository, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    (repo, temp_dir)  // temp_dir cleans up automatically on drop
}
```

### Pattern 2: Pre-test Lock Removal

Remove any existing locks before starting tests:

```rust
use std::path::Path;
use std::fs;

fn ensure_clean_repo(repo_path: &Path) {
    // Clean up common lock files
    let lock_files = [
        "index.lock",
        "config.lock",
        "packed-refs.lock",
    ];

    for lock_file in lock_files {
        let lock_path = repo_path.join(".git").join(lock_file);
        if lock_path.exists() {
            fs::remove_file(lock_path).unwrap_or_else(|_| {
                eprintln!("Warning: Could not remove lock file: {}", lock_file);
            });
        }
    }
}
```

### Pattern 3: Sequential Test Execution

Ensure tests run sequentially to avoid concurrent access:

```toml
# Cargo.toml configuration
[[test]]
name = "git_integration_tests"
required-features = ["git"]
harness = false  # Allows custom test setup/teardown
```

```rust
#[test]
fn sequential_git_tests() {
    // Ensure only one test runs at a time
    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = TEST_MUTEX.lock().unwrap();

    // Test logic here
    test_git_operations();
}
```

## git2 Crate Lock Handling

The git2 crate (Rust bindings to libgit2) has its own approach to handling locks and concurrent access.

### Error Handling for Lock Scenarios

```rust
use git2::{Repository, Error};

fn safe_git_operation(repo_path: &str) -> Result<(), Error> {
    let repo = match Repository::open(repo_path) {
        Ok(repo) => repo,
        Err(e) if e.message().contains("lock file") => {
            // Handle lock file contention
            eprintln!("Lock contention detected, retrying...");
            std::thread::sleep(std::time::Duration::from_millis(100));
            return safe_git_operation(repo_path);  // Retry
        }
        Err(e) => return Err(e),
    };

    // Continue with git operations
    Ok(())
}
```

### Retry Mechanism for Contention

```rust
use std::time::Duration;
use std::thread;

fn with_retry<T>(mut f: impl FnMut() -> Result<T, git2::Error>, max_retries: u32) -> Result<T, git2::Error> {
    let mut last_error = None;

    for attempt in 0..max_retries {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if e.message().contains("lock") && attempt < max_retries - 1 => {
                last_error = Some(e);
                thread::sleep(Duration::from_millis(50 * (attempt + 1) as u64));
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_error.unwrap())
}

// Usage
with_retry(|| {
    let repo = Repository::open(".")?;
    // Perform git operations
    Ok(())
}, 3)?;
```

### Repository Wrapper with Cleanup

```rust
use std::path::PathBuf;
use tempfile::TempDir;
use git2::Repository;

struct ManagedRepo {
    repo: Repository,
    _temp_dir: Option<TempDir>,  // Optional auto-cleanup
    path: PathBuf,
}

impl ManagedRepo {
    fn new_temp() -> Result<Self, git2::Error> {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path().to_path_buf();
        let repo = Repository::init(&path)?;

        Ok(Self {
            repo,
            _temp_dir: Some(temp_dir),
            path,
        })
    }

    fn from_path(path: &PathBuf) -> Result<Self, git2::Error> {
        // Clean up any existing locks
        Self::cleanup_locks(path);

        let repo = Repository::open(path)?;

        Ok(Self {
            repo,
            _temp_dir: None,
            path: path.clone(),
        })
    }

    fn cleanup_locks(path: &PathBuf) {
        let git_dir = path.join(".git");
        let lock_files = ["index.lock", "config.lock", "packed-refs.lock"];

        for lock_file in &lock_files {
            let lock_path = git_dir.join(lock_file);
            if lock_path.exists() {
                std::fs::remove_file(&lock_path).ok();
            }
        }
    }
}

impl Drop for ManagedRepo {
    fn drop(&mut self) {
        // Additional cleanup if needed
    }
}
```

## Testing with Git Repositories

### Best Practices for Git Testing

1. **Isolation**: Each test should work with its own repository
2. **Cleanup**: Ensure all temporary repositories are cleaned up
3. **Determinism**: Use deterministic commit SHAs and timestamps
4. **Error Handling**: Handle lock contention gracefully
5. **Parallelism**: Be careful with parallel test execution

### Using Test Fixtures

```rust
use git2::Repository;
use tempfile::TempDir;
use std::path::Path;

fn create_test_repo_with_commits() -> (Repository, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Create initial commit
    let mut index = repo.index().unwrap();
    let test_file = temp_dir.path().join("README.md");
    std::fs::write(&test_file, "# Test Repository").unwrap();

    index.add_path(Path::new("README.md")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = repo.signature().unwrap();

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    ).unwrap();

    (repo, temp_dir)
}

#[test]
fn test_git_operations() {
    let (repo, _temp_dir) = create_test_repo_with_commits();

    // Test operations
    let head = repo.head().unwrap();
    assert!(head.name().unwrap().ends_with("main"));
}
```

### Testing Concurrent Access

```rust
use std::sync::Arc;
use std::thread;
use git2::Repository;

#[test]
fn test_concurrent_git_operations() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();
    let repo_path_clone = repo_path.clone();

    // Clone the repository for each thread
    let repo1 = Arc::new(Repository::init(&repo_path).unwrap());
    let repo2 = Arc::new(Repository::init(&repo_path_clone).unwrap());

    let handle1 = thread::spawn(move || {
        // Thread 1: Create commits
        for i in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            // Simulate git operation with retry
            with_retry(|| {
                let mut index = repo1.index()?;
                let file_path = Path::new(&format!("file{}.txt", i));
                std::fs::write(file_path, &format!("Content {}", i))?;
                index.add_path(file_path)?;
                index.write_tree()?;
                Ok(())
            }, 3).unwrap();
        }
    });

    let handle2 = thread::spawn(move || {
        // Thread 2: Read operations
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(15));
            let _ = repo2.head();  // Read operation
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
```

## Code Examples

### Example 1: Complete Test Setup with Cleanup

```rust
use git2::{Repository, Oid};
use tempfile::TempDir;
use std::path::Path;

struct GitTestHarness {
    repo: Repository,
    temp_dir: TempDir,
}

impl GitTestHarness {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo = Repository::init(temp_dir.path())
            .expect("Failed to initialize git repository");

        // Create initial commit
        Self::create_initial_commit(&repo, temp_dir.path());

        Self { repo, temp_dir }
    }

    fn create_initial_commit(repo: &Repository, workdir: &Path) {
        let mut index = repo.index().expect("Failed to get index");

        // Create a simple file
        let readme_path = workdir.join("README.md");
        std::fs::write(&readme_path, "# Test Repository").unwrap();
        index.add_path(Path::new("README.md")).unwrap();

        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        let sig = repo.signature().expect("Failed to get signature");

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        ).expect("Failed to make commit");
    }

    fn create_branch(&self, name: &str) -> Result<Oid, git2::Error> {
        let head = self.repo.head()?;
        let head_commit = head.peel_to_commit()?;

        self.repo.branch(name, &head_commit, false)?;
        Ok(head_commit.id())
    }

    fn add_commit(&self, message: &str) -> Result<Oid, git2::Error> {
        let mut index = self.repo.index()?;
        let file_path = self.temp_dir.path().join("new_file.txt");
        std::fs::write(&file_path, format!("Content at {}", std::time::SystemTime::now().elapsed().unwrap().as_secs()))?;
        index.add_path(Path::new("new_file.txt"))?;

        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        let sig = self.repo.signature()?;
        let head = self.repo.head()?;
        let parent = head.peel_to_commit()?;

        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent],
        )
    }
}

#[test]
fn test_branch_operations() {
    let harness = GitTestHarness::new();

    // Create a new branch
    let _ = harness.create_branch("feature-branch");

    // Add commits on main
    harness.add_commit("Second commit").unwrap();

    // Test repository state
    let main_head = harness.repo.head().unwrap();
    assert!(main_head.name().unwrap().ends_with("main"));
}

#[test]
fn test_commit_history() {
    let harness = GitTestHarness::new();

    // Add multiple commits
    for i in 1..=3 {
        harness.add_commit(&format!("Commit {}", i)).unwrap();
    }

    // Verify commit count
    let head = harness.repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let mut count = 0;
    let mut current = commit;

    while let Ok(parent) = current.parent(0) {
        count += 1;
        current = parent;
    }
    count += 1;  // Count the initial commit

    assert_eq!(count, 4);  // Initial + 3 additional commits
}
```

### Example 2: Lock Retry Utility

```rust
use git2::Error;
use std::time::Duration;
use std::thread;

/// Retry a git operation that might encounter lock contention
pub fn with_lock_retry<T, F: FnMut() -> Result<T, Error>>(
    mut operation: F,
    max_retries: u32,
    base_delay_ms: u64,
) -> Result<T, Error> {
    let mut last_error = None;
    let mut delay = Duration::from_millis(base_delay_ms);

    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Check if this is a lock-related error
                if e.message().contains("lock") ||
                   e.message().contains("index.lock") ||
                   e.message().contains("index already locked") {
                    if attempt < max_retries - 1 {
                        last_error = Some(e);
                        thread::sleep(delay);
                        delay *= 2;  // Exponential backoff
                        continue;
                    }
                }
                return Err(e);
            }
        }
    }

    Err(last_error.unwrap())
}

// Example usage
#[test]
fn test_retry_on_lock_contention() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Simulate concurrent operations
    let result = with_lock_retry(|| {
        // This might fail due to lock contention
        let mut index = repo.index()?;
        index.add_path(Path::new("test.txt"))?;
        index.write_tree()
    }, 5, 10);

    assert!(result.is_ok());
}
```

### Example 3: CI/CD Integration

```yaml
# .github/workflows/git-tests.yml
name: Git Integration Tests

on: [push, pull_request]

jobs:
  git-tests:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Set up Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Cache cargo registry
      uses: actions/cache@v2
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Clean up any existing git locks
      run: |
        # This script removes any stray lock files that might cause test failures
        if [ -d .git ]; then
          find .git -name "*.lock" -type f -delete 2>/dev/null || true
          echo "Cleaned up existing lock files"
        fi

    - name: Run tests
      run: cargo test -- --test-threads=1  # Run git tests sequentially

    - name: Test with parallel execution
      run: cargo test --lib  # Parallel tests for library code

    - name: Integration tests
      run: |
        # Integration tests that need their own repos
        cargo test integration_tests -- --test-threads=1
```

## Common Failure Modes and Solutions

### 1. Stale index.lock Error

**Symptom**:
```
fatal: Unable to create '.git/index.lock': File exists.
```

**Causes**:
- Previous git process crashed or was interrupted
- Another Git process is running in the same repository
- File system permissions prevent lock cleanup

**Solutions**:
```bash
# Manual cleanup (verify no Git processes are running first)
rm .git/index.lock

# In code
use std::fs;
use std::path::Path;

fn clear_stale_locks(repo_path: &Path) {
    let lock_path = repo_path.join(".git/index.lock");
    if lock_path.exists() {
        // Double-check no Git processes are running
        if !std::process::Command::new("pgrep")
            .args(["-f", "git"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            fs::remove_file(&lock_path).ok();
        }
    }
}
```

### 2. Reference Lock Contention

**Symptom**:
```
error: cannot lock ref 'refs/heads/main': unable to resolve reference 'refs/heads/main'
```

**Causes**:
- Multiple processes trying to update the same reference
- Git operations interrupted during branch/tag operations

**Solutions**:
```rust
use git2::{Reference, Repository};

fn safe_update_reference(repo: &Repository, ref_name: &str, target: Oid) -> Result<(), git2::Error> {
    with_lock_retry(|| {
        // Try to delete existing reference first
        if let Ok(existing) = repo.find_reference(ref_name) {
            existing.delete()?;
        }

        // Create new reference
        repo.reference(ref_name, target, true, "Safe update")?;
        Ok(())
    }, 3, 10)
}
```

### 3. Config.lock Permission Issues

**Symptom**:
```
error: could not lock config file .git/config: Permission denied
```

**Causes**:
- File permissions on .git/config
- Multiple processes trying to modify configuration
- Antivirus or backup software interfering

**Solutions**:
```bash
# Check permissions
ls -la .git/config

# Fix permissions if needed
chmod 644 .git/config

# In CI/CD, ensure proper file permissions
chmod -R 755 .git
```

### 4. Packed-refs.lock Issues in Large Repositories

**Symptom**:
```
fatal: unable to create packed-refs.lock: File exists
```

**Causes**:
- Frequent garbage collection or pack-refs operations
- Large repositories with many references
- Concurrent pack-refs operations

**Solutions**:
```rust
use std::time::Duration;

fn safe_pack_refs(repo: &Repository) -> Result<(), git2::Error> {
    // Wait a bit if there's contention
    with_lock_retry(|| {
        repo.pack_refs(git2::PackRefsOptions::new())
    }, 5, 100)
}
```

### 5. Merge/Rebase Lock Issues

**Symptom**:
```
error: could not lock refs/HEAD
```

**Causes**:
- Multiple merge/rebase operations
- Corrupted HEAD reference
- Interrupted rebase/merge operations

**Solutions**:
```bash
# Clean up merge/restate
git merge --abort 2>/dev/null || true
git rebase --abort 2>/dev/null || true

# Reset HEAD to a known good state
git reset --hard HEAD
```

## Recommended Practices

### 1. Test Environment Setup

- Use `tempfile` crate for temporary repositories
- Implement RAII cleanup patterns
- Create fresh repositories for each test when possible
- Use sequential execution for integration tests

### 2. Lock Handling

- Always handle lock-related errors gracefully
- Implement retry mechanisms with exponential backoff
- Check for stale locks before starting operations
- Use proper file synchronization primitives

### 3. CI/CD Considerations

- Clean up stale locks before running tests
- Use containerization for isolated test environments
- Monitor for lock-related failures in test logs
- Implement proper timeout mechanisms

### 4. Performance Optimization

- Batch Git operations to minimize lock contention
- Use shallow clones for large repositories in tests
- Cache repository state when possible
- Consider using in-memory Git implementations for unit tests

### 5. Monitoring and Debugging

- Log lock-related errors with context
- Keep test artifacts for debugging (when needed)
- Monitor test execution time for lock-related delays
- Use verbose Git output for debugging lock issues

### 6. Advanced Patterns

```rust
// Repository pool for reuse
use std::sync::Arc;
use std::collections::HashMap;

pub struct RepoPool {
    repos: HashMap<String, Arc<Repository>>,
}

impl RepoPool {
    pub fn get_repo(&mut self, path: &str) -> Result<Arc<Repository>, git2::Error> {
        if !self.repos.contains_key(path) {
            let repo = Repository::open(path)?;
            self.repos.insert(path.to_string(), Arc::new(repo));
        }
        Ok(self.repos[path].clone())
    }

    pub fn cleanup_locks(&self) {
        for repo in self.repos.values() {
            let repo_path = repo.path().parent().unwrap();
            Self::clean_locks(repo_path);
        }
    }

    fn clean_locks(repo_path: &Path) {
        // Implementation for cleaning locks
    }
}
```

This comprehensive guide provides patterns and solutions for handling Git lock files in test environments, with specific focus on the git2 crate in Rust. The examples and strategies can be adapted to various testing scenarios and CI/CD pipelines.