# Rust Test Isolation Best Practices for Parallel Test Execution

**Research Document for Product Requirement Prompt (PRP)**
**Purpose:** Guide AI agents to verify test isolation works correctly in Rust projects
**Last Updated:** 2026-01-12
**Rust Version:** 1.92.0

---

## Executive Summary

This research document provides comprehensive guidance on Rust test isolation best practices, specifically focused on parallel test execution. It is designed to help AI agents verify that tests are properly isolated and can run safely in parallel without race conditions, flakiness, or state leakage.

**Key Findings:**
- Rust's test harness runs tests in parallel by default using multiple threads
- The `--test-threads` flag controls parallelism and is critical for debugging
- Test isolation requires careful management of: filesystem resources, environment variables, global state, and temporary directories
- The `serial_test` crate is essential for tests that cannot be made parallel-safe
- The `tempfile` crate provides RAII-based automatic cleanup for test resources

---

## Table of Contents

1. [Rust's Built-in Test Harness](#1-rusts-built-in-test-harness)
2. [The `--test-threads` Flag](#2-the---test-threads-flag)
3. [Common Causes of Test Flakiness](#3-common-causes-of-test-flakiness)
4. [Best Practices for Test Isolation](#4-best-practices-for-test-isolation)
5. [Using `ctor` and Related Crates](#5-using-ctor-and-related-crates)
6. [Managing Temporary Directories](#6-managing-temporary-directories)
7. [Verification Checklist for AI Agents](#7-verification-checklist-for-ai-agents)
8. [Real-World Examples](#8-real-world-examples)
9. [Common Gotchas and Anti-Patterns](#9-common-gotchas-and-anti-patterns)
10. [Recommended Resources](#10-recommended-resources)

---

## 1. Rust's Built-in Test Harness

### 1.1 How Parallel Execution Works

**Official Documentation:**
- **The Rust Book - Testing:** https://doc.rust-lang.org/book/ch11-00-testing.html
- **Cargo Test Reference:** https://doc.rust-lang.org/cargo/commands/cargo-test.html

**Default Behavior:**
```bash
cargo test
# Runs tests in parallel using multiple threads
# Default: Number of threads = CPU core count
```

**Key Characteristics:**
1. **Each test runs in a separate thread** - Tests within the same test binary run concurrently
2. **No shared memory by default** - Each test has its own stack
3. **Shared filesystem access** - Tests can conflict when writing to the same paths
4. **Shared environment variables** - `std::env::set_var()` affects all concurrent tests
5. **Deterministic ordering not guaranteed** - Tests may run in any order

### 1.2 Test Execution Model

```
┌─────────────────────────────────────────────────────────────┐
│  cargo test                                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Test Harness (lib/tests compiled as separate crates)│   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐             │   │
│  │  │Thread 1 │  │Thread 2 │  │Thread N │  ...        │   │
│  │  │test_a() │  │test_b() │  │test_c() │             │   │
│  │  └─────────┘  └─────────┘  └─────────┘             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Critical Implications:**
- Tests **MUST NOT** assume they run in any particular order
- Tests **MUST NOT** share mutable state without synchronization
- Tests **MUST** clean up resources (or use RAII patterns)
- Tests **SHOULD** use unique temporary directories

### 1.3 Test Organization

```
project/
├── src/
│   ├── lib.rs              # Contains #[cfg(test)] modules
│   └── some_module.rs      # Can have inline tests
├── tests/                  # Integration tests (each file = separate crate)
│   ├── common/             # Shared test utilities
│   │   ├── mod.rs
│   │   ├── fixtures.rs
│   │   └── helpers.rs
│   ├── test_module_a.rs    # Runs in parallel with test_module_b
│   └── test_module_b.rs
└── Cargo.toml
```

**Best Practice:** Each test file in `tests/` is compiled as a separate crate, providing natural isolation. However, tests within the same file run in parallel.

---

## 2. The `--test-threads` Flag

### 2.1 Controlling Parallelism

**Syntax:**
```bash
# Run tests sequentially (single thread)
cargo test -- --test-threads=1

# Run with specific number of threads
cargo test -- --test-threads=4

# Use all CPU cores (default)
cargo test -- --test-threads=num_cpus

# Set via environment variable
export RUST_TEST_THREADS=1
cargo test
```

**When to Use:**
1. **Debugging flaky tests** - Sequential execution reveals race conditions
2. **Tests with shared state** - When refactoring to isolation isn't feasible
3. **Resource-constrained environments** - CI/CD with limited memory/CPU
4. **Tests requiring exclusive access** - File locks, ports, external services

### 2.2 Verification Pattern for AI Agents

**Step 1: Run with default parallelism**
```bash
cargo test --all
# Expected: All tests pass without race conditions
```

**Step 2: Run with increased parallelism**
```bash
cargo test -- --test-threads=8
# Expected: Still passes, no new failures
```

**Step 3: Run sequentially**
```bash
cargo test -- --test-threads=1
# Expected: All tests pass, same results as parallel
```

**Step 4: Run multiple times**
```bash
for i in {1..10}; do cargo test -- --test-threads=8 || exit 1; done
# Expected: 100% pass rate (no flakiness)
```

### 2.3 Interpreting Results

| Result | Interpretation | Action Required |
|--------|---------------|-----------------|
| Passes in parallel, fails sequentially | Test ordering bug (rare) | Fix test interdependencies |
| Passes sequentially, fails in parallel | Race condition | Fix isolation issues |
| Passes in both, but flaky intermittently | Timing/resource issue | Add proper cleanup or delays |
| Fails in both | Test bug, not isolation issue | Debug test logic |

---

## 3. Common Causes of Test Flakiness

### 3.1 Filesystem Race Conditions

**Symptom:** Tests pass when run alone but fail when run in parallel.

**Cause:** Multiple tests writing to the same file paths.

**Example (BAD):**
```rust
#[test]
fn test_config_write() {
    // HARDCODED PATH - CAUSES RACE CONDITIONS
    let path = PathBuf::from("/tmp/test_config.json");
    fs::write(&path, "{}").unwrap();
    // Test fails if another test is also writing here
}
```

**Solution (GOOD):**
```rust
#[test]
fn test_config_write() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("test_config.json");
    fs::write(&path, "{}").unwrap();
    // Each test gets unique directory
}
```

### 3.2 Environment Variable Pollution

**Symptom:** Tests fail with unexpected values from other tests.

**Cause:** Tests modifying `std::env` without cleanup.

**Example (BAD):**
```rust
#[test]
fn test_with_env() {
    std::env::set_var("MY_CONFIG", "test_value");
    // Variable persists to next test
}
```

**Solution (GOOD):**
```rust
#[test]
#[serial]  // Requires serial_test crate
fn test_with_env() {
    std::env::set_var("MY_CONFIG", "test_value");
    // No other test runs concurrently
}

// OR use proper cleanup
#[test]
fn test_with_env_cleanup() {
    let original = std::env::var("MY_CONFIG").ok();
    std::env::set_var("MY_CONFIG", "test_value");
    // ... test logic ...
    match original {
        Some(val) => std::env::set_var("MY_CONFIG", val),
        None => std::env::remove_var("MY_CONFIG"),
    }
}
```

### 3.3 Git Lock Conflicts

**Symptom:** Tests fail with "Failed to remove .git/index.lock" errors.

**Cause:** Concurrent Git operations in same repository.

**Example (from Jin project):**
```rust
// CRITICAL: Clean up Git locks before test
fn cleanup_git_locks(repo_path: &Path) {
    let git_dir = repo_path.join(".git");
    let locks = vec!["index.lock", "HEAD.lock", "config.lock"];
    for lock in locks {
        let _ = fs::remove_file(git_dir.join(lock));
    }
}

#[test]
#[serial]
fn test_git_operations() {
    let temp = TempDir::new().unwrap();
    cleanup_git_locks(temp.path());  // Prevent stale locks
    // Perform Git operations...
}
```

### 3.4 Port Conflicts

**Symptom:** "Address already in use" errors in network tests.

**Cause:** Multiple tests trying to bind to the same port.

**Solution:**
```rust
use std::sync::atomic::{AtomicU16, Ordering};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(8000);

fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[test]
fn test_server() {
    let port = get_test_port();  // Unique port per test
    start_server(port);
}
```

### 3.5 Global Mutable State

**Symptom:** Tests fail with unexpected state, values change between runs.

**Cause:** Static variables modified without synchronization.

**Example (BAD):**
```rust
static mut GLOBAL_COUNTER: u32 = 0;

#[test]
fn test_global() {
    unsafe { GLOBAL_COUNTER = 0; }  // Race condition!
}
```

**Solution (GOOD):**
```rust
use std::sync::Mutex;

lazy_static! {
    static ref GLOBAL_COUNTER: Mutex<u32> = Mutex::new(0);
}

#[test]
fn test_global_safe() {
    let mut counter = GLOBAL_COUNTER.lock().unwrap();
    *counter = 0;  // Thread-safe
}
```

---

## 4. Best Practices for Test Isolation

### 4.1 Use `tempfile` for Automatic Cleanup

**Installation:**
```toml
[dev-dependencies]
tempfile = "3.0"
```

**Pattern:**
```rust
use tempfile::TempDir;

#[test]
fn test_with_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "data")?;
    assert!(file_path.exists());

    // TempDir automatically deleted when dropped
    Ok(())
}
```

**Critical Gotcha:**
```rust
// WRONG - TempDir dropped before test completes
#[test]
fn test_premature_cleanup() {
    let path = {
        let temp = TempDir::new().unwrap();
        temp.path().to_path_buf()  // TempDir dropped here!
    };
    fs::write(path.join("test.txt"), "data").unwrap();  // FAILS
}

// CORRECT - Keep TempDir in scope
#[test]
fn test_proper_cleanup() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    fs::write(path.join("test.txt"), "data").unwrap();  // OK
}  // TempDir dropped here
```

### 4.2 Use RAII for Resource Management

**Pattern from Jin project:**
```rust
pub struct TestFixture {
    _tempdir: TempDir,  // Underscore = "intentionally unused"
    pub path: PathBuf,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Custom cleanup before temp dir deletion
        let _ = cleanup_git_locks(&self.path);
    }
}

#[test]
fn test_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    // Use fixture.path for all operations
    // Automatic cleanup when fixture goes out of scope
    Ok(())
}
```

### 4.3 Generate Unique Identifiers

**Pattern:**
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
    // No conflicts with other tests
}
```

### 4.4 Use Absolute Paths

**Problem:** Relative paths break when `current_dir()` changes.

**Solution:**
```rust
#[test]
fn test_absolute_paths() {
    let temp = TempDir::new().unwrap();
    let absolute_path = temp.path().canonicalize().unwrap();

    // Always use absolute_path for operations
    fs::write(absolute_path.join("file.txt"), "data").unwrap();
}
```

### 4.5 Isolate Environment Variables

**Pattern with fixtures:**
```rust
pub struct EnvFixture {
    _original_vars: HashMap<String, Option<String>>,
}

impl EnvFixture {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        for key in &["JIN_DIR", "HOME", "CONFIG"] {
            vars.insert(key.to_string(), std::env::var(key).ok());
        }
        EnvFixture { _original_vars: vars }
    }

    pub fn set(&self, key: &str, value: &str) {
        std::env::set_var(key, value);
    }
}

impl Drop for EnvFixture {
    fn drop(&mut self) {
        for (key, value) in &self._original_vars {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}
```

---

## 5. Using `ctor` and Related Crates

### 5.1 The `ctor` Crate

**Purpose:** Execute code before/after test runs.

**Documentation:** https://docs.rs/ctor/latest/ctor/

**When to Use:**
- Global test setup (e.g., start test database)
- Global test teardown (e.g., cleanup resources)
- Initialize test logging

**Installation:**
```toml
[dev-dependencies]
ctor = "0.2"
```

**Example:**
```rust
#[ctor]
fn global_setup() {
    // Runs once before all tests
    env_logger::init();
}

#[dtor]
fn global_teardown() {
    // Runs once after all tests complete
    cleanup_test_database();
}
```

**Warning:** Avoid `ctor` for most tests. It creates global state and makes tests harder to isolate. Prefer per-test fixtures.

### 5.2 The `serial_test` Crate

**Purpose:** Force specific tests to run sequentially.

**Documentation:** https://docs.rs/serial_test/latest/serial_test/

**Installation:**
```toml
[dev-dependencies]
serial_test = "3.0"
```

**Basic Usage:**
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_serial_one() {
    // Runs exclusively, no parallel tests
}

#[test]
#[serial]
fn test_serial_two() {
    // Waits for test_serial_one to complete
}
```

**Named Serial Groups:**
```rust
#[test]
#[serial(file_system)]
fn test_fs_one() {
    // Runs serially with other "file_system" tests
}

#[test]
#[serial(database)]
fn test_db_one() {
    // Runs serially with "database" tests
    // But in parallel with "file_system" tests
}
```

**Best Practices:**
1. Use sparingly - slows down test suite
2. Document WHY serial execution is needed
3. Prefer making tests stateless instead
4. Group related tests with meaningful names

### 5.3 The `once_cell` and `lazy_static` Crates

**Purpose:** Thread-safe one-time initialization.

**Example:**
```rust
use once_cell::sync::Lazy;

static TEST_CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::load_from_test_data()
});

#[test]
fn test_with_config() {
    // TEST_CONFIG initialized once, shared safely
    assert_eq!(TEST_CONFIG.value, "expected");
}
```

---

## 6. Managing Temporary Directories

### 6.1 The `tempfile` Crate Patterns

**NamedTempFile:**
```rust
use tempfile::NamedTempFile;

#[test]
fn test_with_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    writeln!(temp_file.as_file_mut(), "test data")?;

    let contents = fs::read_to_string(temp_file.path())?;
    assert_eq!(contents, "test data\n");

    Ok(()
    // File automatically deleted here
}
```

**TempDir:**
```rust
use tempfile::TempDir;

#[test]
fn test_with_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path();

    fs::create_dir(dir_path.join("subdir"))?;
    fs::write(dir_path.join("subdir/file.txt"), "data")?;

    assert!(dir_path.join("subdir/file.txt").exists());

    Ok(())
    // Directory and contents deleted here
}
```

**Persisting Files:**
```rust
#[test]
fn test_persist() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = NamedTempFile::new()?;
    writeln!(temp_file.as_file_mut(), "permanent data")?;

    let permanent_path = "./data/output.txt";
    temp_file.persist(permanent_path)?;

    assert!(Path::new(permanent_path).exists());
    // File is no longer auto-deleted

    Ok(())
}
```

### 6.2 Custom Fixture Patterns

**Basic Fixture:**
```rust
pub struct TestFixture {
    _tempdir: TempDir,
    pub path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();

        // Setup initial state
        fs::create_dir(path.join("config"))?;

        Ok(TestFixture { _tempdir: tempdir, path })
    }
}
```

**Advanced Fixture with Cleanup:**
```rust
pub struct GitTestFixture {
    _tempdir: TempDir,
    pub repo_path: PathBuf,
}

impl GitTestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let repo_path = tempdir.path().to_path_buf();

        git2::Repository::init(&repo_path)?;

        Ok(GitTestFixture { _tempdir: tempdir, repo_path })
    }
}

impl Drop for GitTestFixture {
    fn drop(&mut self) {
        // CRITICAL: Clean up locks before directory deletion
        cleanup_git_locks(&self.repo_path);
    }
}
```

---

## 7. Verification Checklist for AI Agents

### 7.1 Pre-Test Verification

**Check 1: Dependencies**
```bash
# Verify required dev-dependencies
grep -E "tempfile|serial_test|assert_cmd" Cargo.toml
```

**Expected:**
```toml
[dev-dependencies]
tempfile = "3.0"
serial_test = "3.0"  # If tests use #[serial]
assert_cmd = "2.0"   # If testing CLI
```

**Check 2: Test Structure**
```bash
# Verify test file organization
ls -la tests/
ls -la tests/common/
```

**Expected:**
```
tests/
├── common/
│   ├── mod.rs
│   ├── fixtures.rs
│   └── helpers.rs
└── test_*.rs
```

### 7.2 Parallel Execution Verification

**Step 1: Baseline Test Run**
```bash
cargo test --all 2>&1 | tee test_output.txt
```

**Verify:**
- [ ] All tests pass
- [ ] Output shows "test result: ok"
- [ ] No "panicked at" messages
- [ ] Pass rate is 100%

**Step 2: High-Parallelism Test**
```bash
cargo test -- --test-threads=8 2>&1 | tee test_parallel.txt
```

**Verify:**
- [ ] All tests pass (same as baseline)
- [ ] No new failures
- [ ] No deadlock or timeout

**Step 3: Sequential Execution**
```bash
cargo test -- --test-threads=1 2>&1 | tee test_sequential.txt
```

**Verify:**
- [ ] All tests pass
- [ ] Same results as parallel run

**Step 4: Repeated Execution (Flakiness Check)**
```bash
for i in {1..10}; do
    echo "Run $i:"
    cargo test -- --test-threads=8 || exit 1
done
```

**Verify:**
- [ ] 100% success rate across 10 runs
- [ ] No intermittent failures

### 7.3 Isolation Verification

**Check 1: No Hardcoded Paths**
```bash
# Scan for problematic patterns
grep -r '"/tmp/' tests/
grep -r '"~/' tests/
grep -r 'PathBuf::from(".*/' tests/
```

**Expected:** No matches (all paths should use `tempfile`)

**Check 2: Proper Serial Usage**
```bash
# Check for tests that modify global state
grep -r 'std::env::set_var' tests/ | grep -v '^\s*//'
```

**Expected:** Each test with `set_var` should have `#[serial]` attribute

**Check 3: TempDir Scope**
```bash
# Check for TempDir not stored in struct
grep -A 10 'TempDir::new()' tests/ | grep -B 5 'to_path_buf()'
```

**Warning:** If found, verify TempDir is kept in scope

### 7.4 Cleanup Verification

**Check 1: Drop Implementations**
```bash
# Check for custom Drop implementations
grep -r 'impl Drop' tests/common/
```

**Expected:** Fixtures have `Drop` for cleanup

**Check 2: Lock Cleanup**
```bash
# Check for Git lock cleanup in test helpers
grep -r 'cleanup_git_locks' tests/common/
```

**Expected:** Cleanup function exists and is called in `Drop`

**Check 3: No Stale Files**
```bash
# Run tests and check for leftover files
before=$(find /tmp -name "*.lock" 2>/dev/null | wc -l)
cargo test -- --test-threads=8
after=$(find /tmp -name "*.lock" 2>/dev/null | wc -l)

if [ $before -ne $after ]; then
    echo "WARNING: Lock files leaked during tests"
fi
```

### 7.5 Performance Verification

**Measure Test Duration:**
```bash
# Parallel execution
time cargo test -- --test-threads=8

# Sequential execution
time cargo test -- --test-threads=1
```

**Expected:**
- Parallel should be significantly faster (2-4x on multi-core)
- Too many `#[serial]` tests will reduce parallelism benefit

**Check Serial Test Count:**
```bash
grep -r '#\[serial\]' tests/ | wc -l
```

**Guideline:** Keep serial tests to minimum (< 10% of total tests)

---

## 8. Real-World Examples

### 8.1 Example from Jin Project

**File:** `/home/dustin/projects/jin/tests/common/fixtures.rs`

```rust
/// Test fixture that maintains isolated directory
pub struct TestFixture {
    /// CRITICAL: TempDir must be stored to prevent premature cleanup
    _tempdir: TempDir,
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        let jin_dir = Some(path.join(".jin_global"));

        Ok(TestFixture {
            _tempdir: tempdir,
            path,
            jin_dir,
        })
    }

    pub fn set_jin_dir(&self) {
        if let Some(ref jin_dir) = self.jin_dir {
            std::env::set_var("JIN_DIR", jin_dir);
        }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // CRITICAL: Clean up Git locks before temp dir is deleted
        let _ = crate::common::git_helpers::cleanup_git_locks(&self.path);
        if let Some(ref jin_dir) = self.jin_dir {
            let _ = crate::common::git_helpers::cleanup_git_locks(jin_dir);
        }
    }
}
```

**Usage in Test:**
```rust
#[test]
#[serial]  // Required because we set JIN_DIR
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    // ... test logic ...

    Ok(())
    // Automatic cleanup when fixture is dropped
}
```

### 8.2 CLI Testing with `assert_cmd`

**Example:**
```rust
use assert_cmd::Command;

#[test]
fn test_cli_help() {
    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_cli_with_isolated_env() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let jin_dir = temp.path().join(".jin_global");

    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    Ok(())
}
```

### 8.3 Git Operations Testing

**Example:**
```rust
#[test]
#[serial]
fn test_git_commit_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = GitTestFixture::new()?;

    // Create file and commit
    let file_path = fixture.repo_path.join("test.txt");
    fs::write(&file_path, "content")?;

    // Add to Git
    let repo = git2::Repository::open(&fixture.repo_path)?;
    let mut index = repo.index()?;
    index.add_path(Path::new("test.txt"))?;
    index.commit()?;

    // Verify commit exists
    assert!(repo.head().is_ok());

    Ok(())
    // Git locks automatically cleaned up in Drop
}
```

---

## 9. Common Gotchas and Anti-Patterns

### 9.1 Gotcha: Premature TempDir Cleanup

**Anti-Pattern:**
```rust
#[test]
fn test_wrong() {
    let path = TempDir::new().unwrap().path().to_path_buf();
    // TempDir dropped here, directory deleted
    fs::write(&path.join("file.txt"), "data").unwrap();  // FAILS
}
```

**Solution:**
```rust
#[test]
fn test_correct() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    fs::write(&path.join("file.txt"), "data").unwrap();  // OK
}  // TempDir dropped here
```

### 9.2 Gotcha: Test Ordering Assumptions

**Anti-Pattern:**
```rust
#[test]
fn test_init() {
    initialize_global_state();
}

#[test]
fn test_use_state() {
    use_global_state();  // May fail if run before test_init
}
```

**Solution:**
```rust
#[test]
fn test_with_setup() {
    let state = setup_state();
    use_state(&state);
}
```

### 9.3 Gotcha: Ignoring Test Output

**Problem:** Missing debug information when tests fail.

**Solution:**
```bash
# Run with output to see println! statements
cargo test -- --nocapture

# Show output even for passing tests
cargo test -- --show-output

# Run specific test with output
cargo test test_name -- --nocapture --exact
```

### 9.4 Gotcha: Async Testing Without Await

**Anti-Pattern:**
```rust
#[tokio::test]
async fn test_async() {
    some_async_function().await;
    // Missing assertions
}
```

**Solution:**
```rust
#[tokio::test]
async fn test_async() {
    let result = some_async_function().await;
    assert_eq!(result, expected);
}
```

### 9.5 Gotcha: Non-Unique Resource Names

**Anti-Pattern:**
```rust
#[test]
fn test_create_mode() {
    create_mode("test_mode").unwrap();
    // Race condition if multiple tests run
}
```

**Solution:**
```rust
#[test]
fn test_create_mode() {
    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name).unwrap();
}
```

---

## 10. Recommended Resources

### 10.1 Official Rust Documentation

**Core Documentation:**
- **The Rust Book - Chapter 11: Testing**
  https://doc.rust-lang.org/book/ch11-00-testing.html
- **Cargo Test Commands**
  https://doc.rust-lang.org/cargo/commands/cargo-test.html
- **Rust By Example - Testing**
  https://doc.rust-lang.org/rust-by-example/testing.html
- **The Rust Reference - Testing**
  https://doc.rust-lang.org/reference/testing.html

**Key Sections:**
- **Integration Tests:** https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
- **Test Organization:** https://doc.rust-lang.org/book/ch11-03-test-organization.html
- **Running Tests:** https://doc.rust-lang.org/book/ch11-02-running-tests.html

### 10.2 Crate Documentation

**Essential Testing Crates:**
- **tempfile:** https://docs.rs/tempfile/latest/tempfile/
- **serial_test:** https://docs.rs/serial_test/latest/serial_test/
- **assert_cmd:** https://docs.rs/assert_cmd/latest/assert_cmd/
- **predicates:** https://docs.rs/predicates/latest/predicates/

**Advanced Testing Crates:**
- **proptest** (Property-Based Testing): https://altsysrq.github.io/proptest-book/
- **quickcheck** (Property-Based Testing): https://docs.rs/quickcheck/latest/quickcheck/
- **insta** (Snapshot Testing): https://insta.rs/docs/
- **mockall** (Mocking): https://docs.rs/mockall/latest/mockall/

### 10.3 Community Resources

**Blog Posts and Articles:**
- **"How to Test in Rust" by Aleksey Kladov**: https://matklad.github.io/2021/05/31/how-to-test.html
- **"Rust Testing Best Practices"**: https://jaketrent.com/post/rust-testing-best-practices/
- **"Writing Great Tests in Rust"**: https://blog.yoshuawuyts.com/testing/

**Conference Talks:**
- **RustConf 2024 - Testing Techniques**: Search for RustConf testing videos
- **"Zero to Production in Rust"** - Chapter on testing
- **"Testing in Production with Rust"** - Real-world testing strategies

### 10.4 GitHub Examples

**Well-Tested Rust Projects:**
- **Rust Standard Library:** https://github.com/rust-lang/rust/tree/master/library
- **Tokio:** https://github.com/tokio-rs/tokio
- **Serde:** https://github.com/serde-rs/serde
- **Clap:** https://github.com/clap-rs/clap

### 10.5 Books

**Recommended Reading:**
- **"Rust for Rustaceans" by Jon Gjengset** - Chapter on testing
- **"Programming Rust" by Blandy, Orendorff, Tindall** - Testing best practices
- **"Zero to Production in Rust" by Luca Palmieri** - Real-world testing

---

## Appendix: Quick Reference

### A.1 Essential Commands

```bash
# Run all tests
cargo test

# Run with specific thread count
cargo test -- --test-threads=8
cargo test -- --test-threads=1

# Run with output
cargo test -- --nocapture
cargo test -- --show-output

# Run specific test
cargo test test_name
cargo test test_name -- --exact

# Run test module
cargo test --test module_name

# Run ignored tests
cargo test -- --ignored

# Show test output in real-time
cargo test -- --format=pretty

# Set threads via environment
RUST_TEST_THREADS=1 cargo test
```

### A.2 Common Test Attributes

```rust
#[test]              // Standard test
#[test]              // Test that should panic
fn test_panic() {
    panic!("Expected panic");
}

#[test]
#[should_panic(expected = "Expected panic")]  // Panic with specific message
fn test_panic_with_message() {
    panic!("Expected panic");
}

#[test]
#[ignore]           // Skip this test by default
fn test_slow() {
    // Expensive test
}

#[test]
#[serial]           // Run sequentially (requires serial_test)
fn test_exclusive() {
    // No parallel tests run concurrently
}
```

### A.3 Test Fixture Template

```rust
use tempfile::TempDir;
use std::path::PathBuf;
use std::fs;

pub struct TestFixture {
    _tempdir: TempDir,  // Must keep in scope
    pub path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();

        // Setup initial state
        fs::create_dir(path.join("config"))?;

        Ok(TestFixture { _tempdir: tempdir, path })
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Cleanup before temp dir deletion
        let _ = cleanup_resources(&self.path);
    }
}

#[test]
fn test_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    // Use fixture.path for all operations
    Ok(())
}
```

---

## Summary

**Key Principles for Test Isolation:**

1. **Each test must be independent** - No assumptions about execution order
2. **Use `tempfile` for resources** - Automatic cleanup via RAII
3. **Use `#[serial]` sparingly** - Only when truly necessary
4. **Generate unique names** - Prevent conflicts in parallel tests
5. **Use absolute paths** - Avoid `current_dir()` issues
6. **Clean up locks** - Especially for Git operations
7. **Isolate environment** - Use fixtures with proper setup/teardown
8. **Verify with multiple thread counts** - Ensure tests pass both in parallel and sequentially

**Verification Workflow:**

1. Run tests with default parallelism (`cargo test`)
2. Run tests with high parallelism (`--test-threads=8`)
3. Run tests sequentially (`--test-threads=1`)
4. Run tests repeatedly (catch flakiness)
5. Compare results - all should have 100% pass rate

**Common Pitfalls:**

- Premature TempDir cleanup
- Hardcoded file paths
- Environment variable pollution
- Git lock conflicts
- Test ordering assumptions
- Missing assertions
- Non-unique resource names

---

**Document Version:** 1.0
**Last Updated:** 2026-01-12
**Maintained For:** Product Requirement Prompt (PRP) - Test Isolation Verification
