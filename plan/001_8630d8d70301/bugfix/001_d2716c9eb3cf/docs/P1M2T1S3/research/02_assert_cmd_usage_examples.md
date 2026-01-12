# assert_cmd Crate Usage and Examples

## Current Usage in jin Project

The jin project extensively uses `assert_cmd` version 2.0 for CLI integration testing. Below are the key patterns and examples extracted from the codebase.

## Dependencies

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"
serial_test = "3.0"
```

## Basic Setup Patterns

### 1. Binary Invocation Helper

**File**: `/home/dustin/projects/jin/tests/cli_basic.rs`

```rust
use assert_cmd::Command;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}
```

**Key Insight**: Using `env!("CARGO_BIN_EXE_jin")` ensures tests use the correct binary path regardless of build configuration.

### 2. Isolated Test Environment

**File**: `/home/dustin/projects/jin/tests/common/fixtures.rs`

```rust
pub struct TestFixture {
    _tempdir: TempDir,  // CRITICAL: Must keep in scope
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

**Key Insight**: The `_tempdir` field with leading underscore prevents accidental drops while ensuring cleanup when TestFixture is dropped.

## Common Test Patterns

### Pattern 1: Single Command Testing

```rust
#[test]
fn test_help() {
    jin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Phantom Git layer system"));
}
```

### Pattern 2: Multiple Arguments

```rust
#[test]
fn test_add_with_mode_flag() {
    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
}
```

### Pattern 3: Failure Testing

```rust
#[test]
fn test_invalid_subcommand() {
    jin()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

### Pattern 4: Environment Variable Isolation

```rust
#[test]
fn test_with_isolated_jin_dir() {
    let temp = tempfile::TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}
```

## Advanced Patterns

### 1. Conditional Assertions

```rust
#[test]
fn test_mode_list_subcommand() {
    jin()
        .args(["mode", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Available modes")
                .or(predicate::str::contains("No modes found"))
        );
}
```

### 2. Output Inspection

```rust
#[test]
fn test_link_valid_https_url() {
    let result = jin()
        .args(["link", "https://github.com/nonexistent/repo.git"])
        .assert();

    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout_str.contains("Testing connection")
            || stderr_str.contains("Cannot access remote repository")
            || stderr_str.contains("Repository not found")
            || stderr_str.contains("Already exists"),
        "Expected connectivity test, error, or already exists"
    );
}
```

### 3. Complex Workflow Testing

**File**: `/home/dustin/projects/jin/tests/conflict_workflow.rs`

```rust
#[test]
fn test_full_workflow_conflict_to_resolution() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add file to global layer
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit
    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Apply - should create conflict
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // Verify files created
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists());

    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(paused_state_path.exists());

    // Resolve conflict
    fs::write(
        &jinmerge_path,
        r#"{"port": 9999}"#
    ).unwrap();

    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // Verify cleanup
    assert!(!jinmerge_path.exists());
    assert!(!paused_state_path.exists());
}
```

## Testing with Predicates

The `predicates` crate provides powerful assertion compositions:

```rust
use predicates::prelude::*;

// String contains
.stdout(predicate::str::contains("expected text"))

// String does NOT contain
.stdout(predicate::str::contains("error").not())

// Logical OR
.stdout(predicate::str::contains("yes").or(predicate::str::contains("no")))

// Logical AND
.stdout(predicate::str::contains("success")
    .and(predicate::str::contains("completed")))
```

## Serial Testing

For tests that modify shared global state:

```rust
#[test]
#[serial]
fn test_mode_create_subcommand() {
    let result = jin()
        .args(["mode", "create", "testmode"])
        .assert();

    // Accept either success or already exists
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout_str.contains("testmode") || stderr_str.contains("already exists"),
        "Expected mode creation or already exists error"
    );
}
```

## Best Practices Observed in jin

1. **Always isolate with temp directories**: Prevent test interference
2. **Use unique identifiers**: `format!("test_{}", std::process::id())`
3. **Chain assertions**: `.success().stdout(...).stderr(...)`
4. **Use predicates for flexibility**: `.or()` for alternative acceptable outcomes
5. **Test both success and failure paths**: Explicit failure assertions
6. **Use descriptive test names**: `test_add_local_routes_to_layer_8`
7. **Add comments for test sections**: `// STEP 1:`, `// ASSERT:`, etc.

## Common Assertions Summary

| Assertion | Purpose |
|-----------|---------|
| `.success()` | Exit code 0 |
| `.failure()` | Non-zero exit code |
| `.code(n)` | Specific exit code |
| `.stdout(predicate)` | Validate stdout |
| `.stderr(predicate)` | Validate stderr |
| `.get_output()` | Get raw Output for inspection |

## Additional Resources

- assert_cmd API documentation: https://docs.rs/assert_cmd/
- predicates crate: https://docs.rs/predicates/
- serial_test crate: https://docs.rs/serial_test/
