# Rust Test Isolation Code Examples

**Collection of idiomatic Rust test isolation patterns**
**Purpose:** Reference examples for writing properly isolated tests

---

## Table of Contents

1. [Basic Tempfile Patterns](#1-basic-tempfile-patterns)
2. [Fixture Patterns](#2-fixture-patterns)
3. [Environment Variable Isolation](#3-environment-variable-isolation)
4. [Git Operations Testing](#4-git-operations-testing)
5. [CLI Testing](#5-cli-testing)
6. [Parallel-Safe Resource Creation](#6-parallel-safe-resource-creation)
7. [Serial Test Patterns](#7-serial-test-patterns)
8. [Advanced Patterns](#8-advanced-patterns)

---

## 1. Basic Tempfile Patterns

### 1.1 Single File Test

```rust
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;

#[test]
fn test_with_temp_file() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary file
    let temp_file = NamedTempFile::new()?;

    // Write to it
    writeln!(temp_file.as_file_mut(), "test data")?;

    // Read it back
    let contents = fs::read_to_string(temp_file.path())?;
    assert_eq!(contents, "test data\n");

    // File automatically deleted when temp_file is dropped
    Ok(())
}
```

### 1.2 Temporary Directory Test

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_with_temp_dir() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path();

    // Create files in it
    fs::write(dir_path.join("file1.txt"), "data1")?;
    fs::write(dir_path.join("file2.txt"), "data2")?;

    // Verify
    assert!(dir_path.join("file1.txt").exists());
    assert!(dir_path.join("file2.txt").exists());

    // Directory and contents automatically deleted
    Ok(())
}
```

### 1.3 Nested Directory Structure

```rust
#[test]
fn test_with_nested_dirs() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let base = temp_dir.path();

    // Create nested structure
    let config_dir = base.join("config");
    let data_dir = base.join("data");

    fs::create_dir(&config_dir)?;
    fs::create_dir(&data_dir)?;

    fs::write(config_dir.join("settings.toml"), "key = value")?;
    fs::write(data_dir.join("data.json"), "{}")?;

    // Verify structure
    assert!(config_dir.is_dir());
    assert!(data_dir.is_dir());
    assert!(config_dir.join("settings.toml").exists());

    Ok(())
}
```

---

## 2. Fixture Patterns

### 2.1 Basic Fixture

```rust
use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestFixture {
    // CRITICAL: Underscore prefix indicates intentionally unused
    // We only keep it to prevent premature cleanup
    _tempdir: TempDir,
    pub path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();

        Ok(TestFixture {
            _tempdir: tempdir,
            path,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[test]
fn test_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let test_file = fixture.path().join("test.txt");

    fs::write(&test_file, "data")?;
    assert!(test_file.exists());

    Ok(())
    // fixture._tempdir dropped here, directory deleted
}
```

### 2.2 Fixture with Pre-populated Data

```rust
pub struct ConfigTestFixture {
    _tempdir: TempDir,
    pub path: PathBuf,
}

impl ConfigTestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();

        // Create standard structure
        let config_dir = path.join("config");
        fs::create_dir(&config_dir)?;

        // Create default config
        fs::write(
            config_dir.join("settings.toml"),
            r#"
[general]
verbose = true
debug = false
            "#
        )?;

        Ok(ConfigTestFixture {
            _tempdir: tempdir,
            path,
        })
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join("config/settings.toml")
    }
}

#[test]
fn test_with_config_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = ConfigTestFixture::new()?;

    assert!(fixture.config_path().exists());
    let contents = fs::read_to_string(fixture.config_path())?;
    assert!(contents.contains("verbose = true"));

    Ok(())
}
```

### 2.3 Fixture with Custom Cleanup

```rust
pub struct GitTestFixture {
    _tempdir: TempDir,
    pub repo_path: PathBuf,
}

impl GitTestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let repo_path = tempdir.path().to_path_buf();

        // Initialize Git repository
        git2::Repository::init(&repo_path)?;

        Ok(GitTestFixture {
            _tempdir: tempdir,
            repo_path,
        })
    }
}

impl Drop for GitTestFixture {
    fn drop(&mut self) {
        // CRITICAL: Clean up Git locks BEFORE temp dir deletion
        cleanup_git_locks(&self.repo_path);
    }
}

fn cleanup_git_locks(repo_path: &Path) {
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        return;
    }

    // Remove common lock files
    let locks = vec!["index.lock", "HEAD.lock", "config.lock"];
    for lock in locks {
        let _ = std::fs::remove_file(git_dir.join(lock));
    }
}

#[test]
fn test_git_operations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = GitTestFixture::new()?;

    // Perform Git operations
    let repo = git2::Repository::open(&fixture.repo_path)?;
    assert!(repo.is_empty());

    Ok(())
    // cleanup_git_locks called automatically via Drop
}
```

---

## 3. Environment Variable Isolation

### 3.1 Manual Environment Cleanup

```rust
#[test]
fn test_with_manual_env_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    // Save original value
    let original = std::env::var("MY_CONFIG").ok();

    // Set test value
    std::env::set_var("MY_CONFIG", "test_value");

    // Perform test
    let value = std::env::var("MY_CONFIG")?;
    assert_eq!(value, "test_value");

    // Restore original value
    match original {
        Some(val) => std::env::set_var("MY_CONFIG", val),
        None => std::env::remove_var("MY_CONFIG"),
    }

    Ok(())
}
```

### 3.2 Environment Fixture

```rust
use std::collections::HashMap;

pub struct EnvFixture {
    originals: HashMap<String, Option<String>>,
}

impl EnvFixture {
    pub fn new(vars: &[&str]) -> Self {
        let mut originals = HashMap::new();
        for var in vars {
            originals.insert(var.to_string(), std::env::var(var).ok());
        }
        EnvFixture { originals }
    }

    pub fn set(&self, key: &str, value: &str) {
        std::env::set_var(key, value);
    }
}

impl Drop for EnvFixture {
    fn drop(&mut self) {
        for (key, value) in &self.originals {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}

#[test]
fn test_with_env_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EnvFixture::new(&["MY_CONFIG", "HOME"]);

    fixture.set("MY_CONFIG", "test_value");

    let value = std::env::var("MY_CONFIG")?;
    assert_eq!(value, "test_value");

    Ok(())
    // Environment automatically restored
}
```

### 3.3 Serial Test for Environment

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_env_with_serial() {
    // Safe to modify environment because no other test runs concurrently
    std::env::set_var("MY_CONFIG", "test_value");
    assert_eq!(std::env::var("MY_CONFIG").unwrap(), "test_value");
}
```

---

## 4. Git Operations Testing

### 4.1 Basic Git Test

```rust
#[test]
fn test_git_init() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    // Initialize repository
    let repo = git2::Repository::init(repo_path)?;
    assert!(repo.is_empty());

    // Verify .git directory exists
    assert!(repo_path.join(".git").exists());

    Ok(())
}
```

### 4.2 Git Commit Test

```rust
#[test]
fn test_git_commit() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    // Initialize repository
    let repo = git2::Repository::init(repo_path)?;

    // Create file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "content")?;

    // Stage file
    let mut index = repo.index()?;
    index.add_path(Path::new("test.txt"))?;
    index.write()?;

    // Commit
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = repo.signature()?;
    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    )?;

    // Verify commit exists
    let commit = repo.find_commit(oid)?;
    assert_eq!(commit.message().unwrap(), "Initial commit");

    Ok(())
}
```

### 4.3 Git Test with Lock Cleanup

```rust
#[test]
#[serial]
fn test_git_with_lock_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    // CRITICAL: Clean up locks from previous runs
    cleanup_git_locks(repo_path);

    // Initialize and perform operations
    let repo = git2::Repository::init(repo_path)?;
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "content")?;

    let mut index = repo.index()?;
    index.add_path(Path::new("test.txt"))?;
    index.write()?;

    // Lock cleanup happens automatically in Drop
    Ok(())
}
```

---

## 5. CLI Testing

### 5.1 Basic CLI Test

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
```

### 5.2 CLI with Isolated Environment

```rust
#[test]
fn test_cli_with_temp_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let jin_dir = temp.path().join(".jin_global");

    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    assert!(jin_dir.exists());

    Ok(())
}
```

### 5.3 CLI with File Verification

```rust
#[test]
fn test_cli_creates_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let output_file = temp.path().join("output.txt");

    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("generate")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    assert!(output_file.exists());
    assert!(fs::metadata(&output_file)?.len() > 0);

    Ok(())
}
```

---

## 6. Parallel-Safe Resource Creation

### 6.1 Unique Identifier Generator

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn unique_test_id() -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test_{}_{}", std::process::id(), count)
}

#[test]
fn test_with_unique_name() {
    let mode_name = format!("mode_{}", unique_test_id());
    create_mode(&mode_name).unwrap();

    // No conflicts with other tests running in parallel
}
```

### 6.2 Port Allocation

```rust
use std::sync::atomic::{AtomicU16, Ordering};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(8000);

fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[test]
fn test_server() {
    let port = get_test_port();
    let server = start_server(port);

    assert!(server.is_running());
}
```

### 6.3 Parallel-Safe Mode Creation

```rust
#[test]
fn test_parallel_mode_creation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let jin_dir = temp.path().join(".jin_global");
    fs::create_dir(&jin_dir)?;

    // Unique mode name prevents conflicts
    let mode_name = format!("test_mode_{}", unique_test_id());

    Command::new(env!("CARGO_BIN_EXE_jin"))
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify mode exists
    let mode_dir = jin_dir.join("modes").join(&mode_name);
    assert!(mode_dir.exists());

    Ok(())
}
```

---

## 7. Serial Test Patterns

### 7.1 Basic Serial Test

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_serial_one() {
    // Only this test runs (no parallel tests)
}

#[test]
#[serial]
fn test_serial_two() {
    // Waits for test_serial_one to complete
}
```

### 7.2 Named Serial Groups

```rust
#[test]
#[serial(database)]
fn test_db_setup() {
    // Runs serially with other "database" tests
}

#[test]
#[serial(database)]
fn test_db_query() {
    // Runs serially with test_db_setup
}

#[test]
#[serial(filesystem)]
fn test_fs_operations() {
    // Can run in parallel with "database" tests
}
```

### 7.3 Serial Test for Global State

```rust
use std::sync::Mutex;

lazy_static! {
    static ref GLOBAL_STATE: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

#[test]
#[serial]
fn test_modifies_global_state() {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.push("test".to_string());
    assert_eq!(state.len(), 1);
}
```

---

## 8. Advanced Patterns

### 8.1 Remote Repository Fixture

```rust
pub struct RemoteFixture {
    _tempdir: TempDir,
    pub local_path: PathBuf,
    pub remote_path: PathBuf,
}

impl RemoteFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let local_path = tempdir.path().join("local");
        let remote_path = tempdir.path().join("remote");

        fs::create_dir(&local_path)?;
        fs::create_dir(&remote_path)?;

        // Initialize local repo
        git2::Repository::init(&local_path)?;

        // Initialize bare remote
        git2::Repository::init_bare(&remote_path)?;

        Ok(RemoteFixture {
            _tempdir: tempdir,
            local_path,
            remote_path,
        })
    }
}

#[test]
fn test_with_remote() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = RemoteFixture::new()?;

    // Can test push/pull operations
    assert!(fixture.local_path.exists());
    assert!(fixture.remote_path.exists());

    Ok(())
}
```

### 8.2 Test with Timeout

```rust
use std::time::Duration;

#[test]
fn test_with_timeout() {
    let start = std::time::Instant::now();

    let result = std::thread::spawn(move || {
        // Long-running operation
        std::thread::sleep(Duration::from_millis(100));
        42
    })
    .join();

    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(result.unwrap(), 42);
}
```

### 8.3 Retry Logic for Flaky Tests

```rust
fn retry_test<F>(mut test_fn: F, max_attempts: u32) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<(), Box<dyn std::error::Error>>,
{
    for attempt in 1..=max_attempts {
        match test_fn() {
            Ok(()) => return Ok(()),
            Err(e) if attempt < max_attempts => {
                eprintln!("Attempt {} failed, retrying...", attempt);
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

#[test]
fn test_with_retry() -> Result<(), Box<dyn std::error::Error>> {
    retry_test(|| {
        let temp = TempDir::new()?;
        // Test logic that might be flaky
        assert!(temp.path().exists());
        Ok(())
    }, 3)
}
```

### 8.4 Snapshot Testing Pattern

```rust
#[test]
fn test_snapshot_output() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::similar(
            "No staged changes\n\
             No uncommitted changes\n"
        ));

    Ok(())
}
```

### 8.5 Property-Based Testing Pattern

```rust
#[test]
fn test_property_roundtrip() {
    // Test that serialization is lossless
    let test_cases = vec![
        ("simple", "value"),
        ("with spaces", "multiple words"),
        ("unicode", "日本語"),
    ];

    for (key, value) in test_cases {
        let serialized = serialize(key, value);
        let (deser_key, deser_value) = deserialize(&serialized).unwrap();
        assert_eq!((key, value), (deser_key, deser_value));
    }
}
```

---

## Quick Reference: Best Practice Patterns

### Always Do

✅ Use `tempfile` for temporary resources
✅ Keep `TempDir` in scope
✅ Generate unique identifiers for parallel tests
✅ Use absolute paths
✅ Clean up resources in `Drop`
✅ Use `#[serial]` when modifying global state
✅ Isolate environment variables

### Never Do

❌ Use hardcoded paths like `/tmp/`
❌ Drop `TempDir` early
❌ Assume test execution order
❌ Share mutable state without synchronization
❌ Use static names for parallel resources
❌ Modify environment without `#[serial]`
❌ Ignore cleanup in error paths

---

**Document Version:** 1.0
**Last Updated:** 2026-01-12
**For:** Reference examples for writing isolated tests
