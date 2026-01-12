# Rust Test Patterns Research - Jin Codebase

## 1. Testing Framework

The Jin codebase uses Rust's built-in testing framework with these dev-dependencies:

```toml
[dev-dependencies]
assert_cmd = "2.0"       # For command testing
predicates = "3.0"       # For output assertions
tempfile = "3.0"         # For temporary directories
serial_test = "3.0"      # For parallel test execution
```

**Test runner**: Standard `cargo test` command

## 2. Integration Test Organization

### Directory Structure
```
tests/
├── common/                 # Shared test utilities
│   ├── mod.rs             # Module exports
│   ├── assertions.rs      # Custom assertion helpers
│   ├── fixtures.rs        # Test fixtures and setup
│   └── git_helpers.rs     # Git-specific utilities
├── cli_basic.rs           # Basic CLI command tests
├── cli_diff.rs            # Diff command tests
├── cli_import.rs          # Import command tests
├── cli_list.rs            # List command tests
├── cli_mv.rs              # Move command tests
├── cli_reset.rs           # Reset command tests
├── core_workflow.rs       # End-to-end workflow tests
├── atomic_operations.rs   # Atomic operation tests
├── mode_scope_workflow.rs # Mode/scope workflow tests
├── sync_workflow.rs       # Synchronization tests
└── error_scenarios.rs     # Error handling tests
```

## 3. Command-Line Testing Patterns

### Pattern 1: Basic Command Testing
```rust
use assert_cmd::Command;

#[test]
fn test_help() {
    jin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Phantom Git layer system"));
}
```

### Pattern 2: Environment Variable Isolation
```rust
#[test]
fn test_status_subcommand() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}
```

## 4. Test Setup Patterns

### Test Fixture Pattern
```rust
// From fixtures.rs
pub struct TestFixture {
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
```

## 5. Fixture Locations and Utilities

### Test Utilities Location
- **Assertions**: `/tests/common/assertions.rs`
- **Fixtures**: `/tests/common/fixtures.rs`
- **Git Helpers**: `/tests/common/git_helpers.rs`

### Key Fixtures
1. **TestFixture**: Creates isolated temporary directory with Jin
2. **RemoteFixture**: Creates local + remote repository setup
3. **GitTestEnv**: Wrapper with automatic lock file cleanup

### Helper Functions
- `jin_init()`: Initialize Jin in a directory
- `create_commit_in_repo()`: Create Git commits with files
- `create_mode()`: Create Jin modes
- `create_scope()`: Create Jin scopes
- `unique_test_id()`: Generate unique test identifiers

## 6. Running Tests

### Basic Test Commands
```bash
# Run all tests
cargo test

# Run specific test file
cargo test cli_basic

# Run test with filter
cargo test test_help

# Run tests without output capture
cargo test -- --nocapture

# Run integration tests only
cargo test --test
```

### Parallel Test Execution
The codebase uses `serial_test` crate for tests that must run sequentially:
```rust
use serial_test::serial;

#[serial]
#[test]
fn test_state_dependent_operation() {
    // This test will run sequentially
}
```

## 7. Test Isolation Patterns

### Environment Variable Isolation
```rust
let temp = TempDir::new().unwrap();
let jin_dir = temp.path().join(".jin_global");

// Set JIN_DIR for isolation
std::env::set_var("JIN_DIR", &jin_dir);

// All commands use the isolated directory
jin()
    .arg("init")
    .current_dir(temp.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();
```

### Git Lock File Cleanup
```rust
// From git_helpers.rs
pub fn cleanup_git_locks(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo_path.join(".git");

    // Clean various Git lock files
    let lock_files = &["index.lock", "HEAD.lock", "config.lock"];

    for lock_file in lock_files {
        let lock_path = git_dir.join(lock_file);
        if lock_path.exists() {
            fs::remove_file(&lock_path)?;
        }
    }

    Ok(())
}
```

## Key Testing Principles Observed

1. **Complete Isolation**: Each test uses unique temporary directories
2. **Environment Control**: JIN_DIR is always set to prevent test interference
3. **Git Lock Cleanup**: Automatic cleanup of stale Git lock files
4. **Custom Assertions**: Domain-specific assertions for Jin operations
5. **Comprehensive Coverage**: Tests cover happy paths, error conditions, and edge cases
6. **Atomic Operations**: Tests verify rollback behavior on failure
7. **Parallel Safety**: Tests are designed to run in parallel when possible
