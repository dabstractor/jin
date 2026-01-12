# Research: Test Patterns for Metadata Operations

## Test Framework

**Dependencies**:
- `assert_cmd` - Command testing framework
- `predicates` - String and output matching
- `tempfile` - Temporary directory management
- `serial_test` - Sequential test execution (for tests using `std::env::set_current_dir`)
- `git2` - Git operations

## Test File Locations

### Integration Tests (`tests/`)
- `mode_scope_workflow.rs` - Mode/scope operations
- `workspace_validation.rs` - Workspace validation
- `destructive_validation.rs` - Destructive operations
- `cli_basic.rs`, `cli_reset.rs` - CLI command tests
- `core_workflow.rs` - Core workflow tests

### Unit Tests (in source files)
- Most modules have `#[cfg(test)] mod tests` blocks

### Test Utilities
- `tests/common/mod.rs` - Common test setup
- `tests/common/fixtures.rs` - Test fixtures and helpers
- `tests/common/assertions.rs` - Custom assertions
- `tests/common/git_helpers.rs` - Git utilities
- `src/test_utils.rs` - Unit test utilities

## Test Structure Pattern

```rust
#[test]
#[serial]
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    // Test mode switching and file operations
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();
}
```

## Metadata Testing Patterns

### Creating Test Metadata
```rust
let mut metadata = WorkspaceMetadata::new();
metadata.applied_layers = vec!["global".to_string(), "mode/production".to_string()];
metadata.add_file(PathBuf::from("config.json"), "abc123".to_string());
metadata.save()?;
```

### Loading and Verifying
```rust
let loaded = WorkspaceMetadata::load().unwrap();
assert_eq!(loaded.applied_layers, vec!["global", "mode/production"]);
assert_eq!(loaded.files.len(), 1);
```

## Test Utilities

### TestFixture
```rust
pub struct TestFixture {
    _tempdir: TempDir,
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl TestFixture {
    pub fn new() -> Result<Self> { /* ... */ }
    pub fn path(&self) -> &Path { /* ... */ }
    pub fn set_jin_dir(&self) { /* ... */ }
}
```

### Custom Assertions
```rust
assert_workspace_file(project_path, "config.json", expected_content);
assert_staging_contains(project_path, "config.json");
assert_layer_ref_exists("refs/jin/layers/mode/dev", Some(jin_dir));
assert_context_mode(project_path, "production");
assert_context_scope(project_path, "env:prod");
```

### Test Setup Helpers
```rust
jin_init(path, None)?;
create_mode(mode_name, Some(jin_dir))?;
create_scope(scope_name, Some(jin_dir))?;
create_commit_in_repo(repo_path, file, content, msg)?;
```

## Mode/Scope Testing Pattern

```rust
// Create mode/scope
let mode_name = format!("test_mode_{}", unique_test_id());
create_mode(&mode_name, Some(jin_dir))?;

// Use mode/scope
jin()
    .args(["mode", "use", &mode_name])
    .current_dir(project_path)
    .env("JIN_DIR", jin_dir)
    .assert()
    .success();

// Verify context
assert_context_mode(project_path, &mode_name);
```

## Key Testing Insights

1. **Test Isolation**: Each test gets a temporary directory with unique JIN_DIR
2. **Sequential Execution**: Tests modifying environment use `#[serial]`
3. **Git Lock Cleanup**: Automatic cleanup prevents test interference
4. **Custom Assertions**: Rich assertion helpers for Jin-specific states
5. **Fixture Pattern**: Reusable test setup components
6. **Error Pattern Testing**: Tests verify specific error types and messages

## Test for Scope Metadata Clearing

Based on the mode switching test pattern, scope tests should follow:

```rust
#[test]
#[serial]
fn test_scope_switch_clears_metadata() -> Result<()> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    // Create scopes
    let scope1 = format!("scope_{}", unique_test_id());
    let scope2 = format!("scope_{}", unique_test_id());
    create_scope(&scope1, Some(jin_dir))?;
    create_scope(&scope2, Some(jin_dir))?;

    // Apply first scope
    create_test_file(project_path, "config.txt")?;
    jin_add(project_path, &["--scope", &scope1, "config.txt"])?;
    jin_commit(project_path, "Add config")?;
    jin_apply(project_path)?;

    // Switch to second scope - should clear metadata
    jin()
        .args(["scope", "use", &scope2])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(contains("Cleared workspace metadata"));

    // Verify metadata was cleared
    let metadata_path = jin_dir.join("workspace").join("last_applied.json");
    assert!(!metadata_path.exists());

    Ok(())
}
```
