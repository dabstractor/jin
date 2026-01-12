# Best Practices for Isolated Test Environments in Rust

## Overview

Isolated test environments are critical for reliable integration testing, especially for CLI applications that modify filesystem state, environment variables, or have dependencies on external systems.

## Key Strategies

### 1. Temporary Directory Isolation

**Best Practice**: Use the `tempfile` crate for automatic cleanup

```rust
use tempfile::TempDir;

#[test]
fn test_with_temp_dir() {
    let temp = TempDir::new().unwrap();

    // Use temp.path() for all file operations
    let config_path = temp.path().join("config.json");
    fs::write(&config_path, "{}").unwrap();

    // Directory automatically cleaned up when temp is dropped
}
```

**Critical Gotcha**: The `TempDir` must remain in scope or it will be deleted prematurely.

### 2. Environment Variable Isolation

**Pattern from jin project**:

```rust
pub struct TestFixture {
    _tempdir: TempDir,
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl TestFixture {
    pub fn set_jin_dir(&self) {
        if let Some(ref jin_dir) = self.jin_dir {
            std::env::set_var("JIN_DIR", jin_dir);
        }
    }
}

#[test]
fn test_with_isolated_env() {
    let fixture = TestFixture::new().unwrap();
    fixture.set_jin_dir();  // Sets isolated JIN_DIR

    // All commands use the isolated environment
    jin()
        .arg("init")
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();
}
```

### 3. Git Repository Isolation

**Pattern for testing Git operations**:

```rust
impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();

        // Initialize Git repo in temp directory
        git2::Repository::init(&path)?;

        Ok(TestFixture {
            _tempdir: tempdir,
            path,
            jin_dir: Some(path.join(".jin_global")),
        })
    }
}
```

### 4. Automatic Cleanup with Drop

**Critical pattern for preventing test pollution**:

```rust
impl Drop for TestFixture {
    fn drop(&mut self) {
        // CRITICAL: Clean up Git locks before temp dir is deleted
        let _ = cleanup_git_locks(&self.path);

        // Also clean up Jin directory locks
        if let Some(ref jin_dir) = self.jin_dir {
            let _ = cleanup_git_locks(jin_dir);
        }
    }
}
```

This ensures cleanup happens even if test fails, preventing "stale lock" errors in subsequent runs.

## Git Lock Cleanup Pattern

**File**: `/home/dustin/projects/jin/tests/common/git_helpers.rs`

```rust
pub fn cleanup_git_locks(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo_path.join(".git");

    if !git_dir.exists() {
        return Ok(());
    }

    // Clean common lock files
    let lock_files = &[
        "index.lock",
        "HEAD.lock",
        "config.lock",
        "packed-refs.lock",
        "refs/heads/main.lock",
        "refs/heads/master.lock",
    ];

    for lock_file in lock_files {
        let lock_path = git_dir.join(lock_file);
        if lock_path.exists() {
            let _ = fs::remove_file(&lock_path); // Ignore errors
        }
    }

    // Recursively clean all .lock files under refs
    let refs_dir = git_dir.join("refs");
    if refs_dir.exists() {
        clean_lock_files_recursive(&refs_dir)?;
    }

    Ok(())}
```

## Remote Repository Testing

For testing remote Git operations:

```rust
pub struct RemoteFixture {
    pub _tempdir: TempDir,
    pub local_path: PathBuf,
    pub remote_path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl RemoteFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let local_path = tempdir.path().join("local");
        let remote_path = tempdir.path().join("remote");

        fs::create_dir(&local_path)?;
        fs::create_dir(&remote_path)?;

        // Initialize bare remote repository
        git2::Repository::init_bare(&remote_path)?;

        Ok(RemoteFixture {
            _tempdir: tempdir,
            local_path,
            remote_path,
            jin_dir: Some(tempdir.path().join(".jin_global")),
        })
    }
}
```

## Unique Identifiers for Parallel Tests

**Problem**: `std::process::id()` alone is insufficient for parallel tests.

**Solution**: Atomic counter

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", std::process::id(), count)
}

#[test]
fn test_with_unique_mode() {
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin()
        .args(["mode", "create", &mode_name])
        .assert()
        .success();
}
```

## Serial Testing for Shared Resources

When tests MUST modify global state:

```rust
#[test]
#[serial]  // From serial_test crate
fn test_modifies_global_state() {
    // Only one test with #[serial] runs at a time
}
```

## Working Directory Management

**Pattern**: Always set current_dir in tests

```rust
#[test]
fn test_current_dir() {
    let temp = TempDir::new().unwrap();

    jin()
        .arg("init")
        .current_dir(temp.path())  // Critical: sets working directory
        .env("JIN_DIR", temp.path().join(".jin"))
        .assert()
        .success();
}
```

## Best Practices Summary

1. **Always use temp directories**: Never use actual user directories
2. **Isolate environment variables**: Use unique paths per test
3. **Implement Drop cleanup**: Especially for Git locks
4. **Use unique identifiers**: Combine process ID with atomic counter
5. **Set working directory explicitly**: Don't rely on default
6. **Clean up before temp deletion**: Git locks must be removed first
7. **Use serial_test when necessary**: For truly shared resources
8. **Keep fixtures in scope**: TempDir deletes immediately when dropped

## Common Pitfalls

1. **Early TempDir drop**: Fixture must own the TempDir
2. **Forgotten Git locks**: Causes "index lock exists" errors
3. **Shared environment variables**: Tests interfere with each other
4. **Non-unique names**: Parallel tests overwrite each other's data
5. **Missing current_dir**: Commands run in wrong directory

## Testing Checklist

- [ ] Using temp directory for all file operations
- [ ] Isolated environment variables (JIN_DIR, HOME, etc.)
- [ ] Git repository initialized in temp directory
- [ ] Unique identifiers for names (modes, scopes, etc.)
- [ ] Drop implementation for cleanup
- [ ] current_dir() set on all commands
- [ ] Git lock cleanup in place
- [ ] Serial test attribute for shared state
- [ ] Test creates and destroys its own data

## Additional Resources

- tempfile crate: https://docs.rs/tempfile/
- Git lock file documentation: https://git-scm.com/docs/gitglossary#Documentation/gitglossary.txt-aiddeflockfilealockfile
- Rust testing best practices: https://doc.rust-lang.org/book/ch11-03-test-organization.html
