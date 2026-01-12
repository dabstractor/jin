# Integration Test Patterns Research - Agent Output

## Integration Test Patterns in Jin

Based on the agent research of existing integration tests in the Jin codebase:

### 1. **Test Fixture Pattern**

The codebase uses a sophisticated fixture system for test isolation:

```rust
// From tests/common/fixtures.rs
pub struct TestFixture {
    _tempdir: TempDir,  // Must be kept in scope
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

pub fn setup_test_repo() -> Result<TestFixture, Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let jin_dir = fixture.jin_dir.as_ref().unwrap();
    jin_init(fixture.path(), Some(jin_dir))?;
    Ok(fixture)
}
```

**Key Features:**
- Uses `tempfile::TempDir` for automatic cleanup
- Isolated `.jin_global` directory per test via `JIN_DIR` environment variable
- Git lock cleanup in `Drop` implementation

### 2. **Assertion Patterns**

The codebase has comprehensive assertion helpers:

```rust
// From tests/common/assertions.rs
// File existence checks
pub fn assert_workspace_file_exists(project_path: &Path, file: &str) {
    let file_path = project_path.join(file);
    assert!(file_path.exists(), "Workspace file {} should exist at {:?}", file, file_path);
}

// Staging index verification
pub fn assert_staging_contains(project_path: &Path, file: &str) {
    let staging_index_path = project_path.join(".jin/staging/index.json");
    let staging_content = fs::read_to_string(&staging_index_path).unwrap();
    assert!(staging_content.contains(file), "Staging index should contain '{}'", file);
}

// Context verification
pub fn assert_context_mode(project_path: &Path, expected_mode: &str) {
    let context_path = project_path.join(".jin/context");
    let context_content = fs::read_to_string(&context_path).unwrap();
    assert!(context_content.contains(&format!("mode: {}", expected_mode)));
}
```

### 3. **Command Output/Error Message Testing**

Multiple patterns for testing command success/failure:

```rust
// Success pattern
jin()
    .arg("init")
    .current_dir(project_path)
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success()
    .stdout(predicate::str::contains("Initialized Jin"));

// Failure pattern
jin()
    .arg("status")
    .current_dir(temp.path())
    .env("JIN_DIR", temp.path().join(".jin_global"))
    .assert()
    .failure()
    .stderr(predicate::str::contains("Jin not initialized"));

// Flexible acceptance (for commands that may succeed or already exist)
let result = jin().args(["mode", "create", "testmode"]).assert();
let output = result.get_output();
let stdout_str = String::from_utf8_lossy(&output.stdout);
let stderr_str = String::from_utf8_lossy(&output.stderr);
assert!(
    stdout_str.contains("testmode") || stderr_str.contains("already exists"),
    "Expected mode creation or already exists error"
);
```

### 4. **File System State Verification**

Tests verify metadata files and directory structures:

```rust
// Check .jin directory exists
pub fn assert_jin_initialized(project_path: &Path) {
    let jin_dir = project_path.join(".jin");
    assert!(jin_dir.exists() && jin_dir.is_dir());
}

// Verify paused state cleanup (from tests/cli_resolve.rs)
let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
// ... after resolve command ...
assert!(!paused_state_path.exists(), "Paused state should be cleared after resolve");
```

### 5. **Sequential Testing with #[serial]**

Tests that modify environment variables use the `serial` attribute:

```rust
// From tests/mode_scope_workflow.rs
#[test]
#[serial]  // Required for tests that set JIN_DIR
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // CRITICAL: Set before any Jin operations
    // ... test logic ...
}
```

### 6. **Multi-Command Workflow Tests**

Tests sequence multiple commands together:

```rust
// From tests/core_workflow.rs
fn test_complete_workflow_init_to_apply() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Step 1: Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Step 2: Create and use mode
    let mode_name = format!("workflow_test_{}", std::process::id());
    jin().args(["mode", "create", &mode_name]).assert().success();
    jin().args(["mode", "use", &mode_name]).current_dir(project_path).assert().success();

    // Step 3: Create and add file
    fs::write(project_path.join("config.json"), "{}")?;
    jin().args(["add", "config.json", "--mode"]).current_dir(project_path).assert().success();

    // Step 4: Commit
    jin().args(["commit", "-m", "Add config"]).current_dir(project_path).assert().success();

    // Step 5: Apply
    jin().arg("apply").current_dir(project_path).assert().success();

    Ok(())
}
```

### 7. **"No Error" Scenario Testing**

Tests verify operations don't fail unexpectedly:

```rust
// From tests/core_workflow.rs
fn test_init_already_initialized() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Try to init again - should handle gracefully
    let result = jin().arg("init").current_dir(project_path).env("JIN_DIR", jin_dir).assert();

    // Accept success or warning about existing init
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success() || stderr_str.contains("already") || stdout_str.contains("already"),
        "Init should handle already initialized directory gracefully"
    );

    Ok(())
}
```

### 8. **Metadata Clear Verification Pattern**

While no specific tests for metadata auto-clear on mode switch exist yet, the pattern would involve:

```rust
// Pattern for verifying metadata cleared
fn test_metadata_cleared_on_mode_switch() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Setup initial mode with metadata
    let mode1 = "test_mode_1";
    create_mode(mode1, Some(&fixture.jin_dir.unwrap()))?;
    jin().args(["mode", "use", mode1]).current_dir(fixture.path()).assert().success();

    // Create some metadata files
    fs::write(fixture.path().join(".jin/metadata.json"), "{}")?;

    // Switch mode
    let mode2 = "test_mode_2";
    create_mode(mode2, Some(&fixture.jin_dir.unwrap()))?;
    jin().args(["mode", "use", mode2]).current_dir(fixture.path()).assert().success();

    // Verify metadata cleared
    assert!(!fixture.path().join(".jin/metadata.json").exists());

    Ok(())
}
```

### Best Practices Observed:

1. **Test Isolation**: Each test gets its own temporary directory with isolated `.jin_global`
2. **Environment Management**: Always set `JIN_DIR` before any Jin operations
3. **Comprehensive Assertions**: Custom helpers for Jin-specific state verification
4. **Flexible Success Criteria**: Accept both success and certain error conditions where appropriate
5. **Sequential Execution**: Use `#[serial]` for tests modifying global state
6. **Resource Cleanup**: Automatic cleanup via `Drop` implementations
7. **Error Testing**: Extensive testing of error conditions and recovery paths
