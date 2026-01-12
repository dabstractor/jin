# Test Pattern Research

## TestFixture Pattern

**File**: `/home/dustin/projects/jin/tests/common/fixtures.rs`

### Core Structure

```rust
// Lines 16-50
pub struct TestFixture {
    _tempdir: TempDir,          // CRITICAL: Must be kept in scope
    pub path: PathBuf,          // Path to test directory
    pub jin_dir: Option<PathBuf>, // Isolated Jin directory for test isolation
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        let jin_dir = Some(path.join(".jin_global")); // Isolated Jin directory
        Ok(TestFixture {
            _tempdir: tempdir,
            path,
            jin_dir,
        })
    }

    // CRITICAL: Call this BEFORE any Jin operations
    pub fn set_jin_dir(&self) {
        if let Some(ref jin_dir) = self.jin_dir {
            std::env::set_var("JIN_DIR", jin_dir);
        }
    }
}
```

### Helper Functions

```rust
// Lines 118-130: Initialize Jin in a directory
pub fn jin_init(path: &Path, jin_dir: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Git repository first
    git2::Repository::init(path)?;

    let mut binding = jin();
    let mut cmd = binding.arg("init").current_dir(path);
    if let Some(jin_dir) = jin_dir {
        cmd = cmd.env("JIN_DIR", jin_dir);
    }
    cmd.assert().success();
    Ok(())
}

// Lines 183-210: Create a mode
pub fn create_mode(
    mode_name: &str,
    jin_dir: Option<&PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = jin();
    if let Some(jin_dir) = jin_dir {
        cmd.env("JIN_DIR", jin_dir);
    }
    let result = cmd.args(["mode", "create", mode_name]).assert();
    // ... validation
}

// Lines 265-269: Generate unique test IDs for parallel tests
pub fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", std::process::id(), count)
}
```

## Custom Assertions

**File**: `/home/dustin/projects/jin/tests/common/assertions.rs`

```rust
// Lines 18-35: Assert file content
pub fn assert_workspace_file(project_path: &Path, file: &str, expected_content: &str) {
    let file_path = project_path.join(file);
    assert!(file_path.exists(), "Workspace file {} should exist", file);

    let actual_content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read file: {}", e));

    assert_eq!(actual_content, expected_content, "Content mismatch");
}

// Lines 45-53: Assert file exists
pub fn assert_workspace_file_exists(project_path: &Path, file: &str) {
    let file_path = project_path.join(file);
    assert!(file_path.exists(), "Workspace file {} should exist", file);
}

// Lines 63-71: Assert file does not exist
pub fn assert_workspace_file_not_exists(project_path: &Path, file: &str) {
    let file_path = project_path.join(file);
    assert!(!file_path.exists(), "Workspace file {} should not exist", file);
}

// Lines 148-179: Assert layer ref exists
pub fn assert_layer_ref_exists(ref_path: &str, jin_repo_path: Option<&std::path::Path>) {
    let repo_path = match jin_repo_path {
        Some(path) => path.to_path_buf(),
        None => {
            // Fallback to environment variable or home directory
            if let Ok(jin_dir) = std::env::var("JIN_DIR") {
                std::path::PathBuf::from(jin_dir)
            } else {
                dirs::home_dir().expect("Failed to get home directory").join(".jin")
            }
        }
    };

    let repo = git2::Repository::open(&repo_path)
        .unwrap_or_else(|e| panic!("Failed to open Jin repository: {}", e));

    match repo.find_reference(ref_path) {
        Ok(_) => {}
        Err(e) => panic!("Layer ref '{}' should exist: {}", ref_path, e),
    };
}
```

## Integration Test Patterns

### File: `/home/dustin/projects/jin/tests/conflict_workflow.rs`

```rust
// Lines 56-175: Full conflict to resolution workflow
#[test]
fn test_full_workflow_conflict_to_resolution() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate mode
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
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

    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify and add to mode layer (creates conflict)
    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove from workspace
    fs::remove_file(&config_path).unwrap();

    // Run apply - should create .jinmerge
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // Verify .jinmerge file exists
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists());

    // Verify paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(paused_state_path.exists());

    // Resolve and verify cleanup
    // ...
}
```

### File: `/home/dustin/projects/jin/tests/mode_scope_workflow.rs`

```rust
// Lines 349-446: Deep merge test pattern
#[test]
#[serial]
fn test_mode_scope_deep_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fixture.set_jin_dir();
    jin_init(project_path, None)?;

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add base configuration in mode layer
    fs::write(
        project_path.join("settings.json"),
        r#"{"debug": false, "timeout": 30, "features": {"auth": true}}"#,
    )?;

    jin()
        .args(["add", "settings.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Base settings"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Override some values in higher layer
    fs::write(
        project_path.join("settings.json"),
        r#"{"debug": true, "features": {"logging": true}}"#,
    )?;

    jin()
        .args(["add", "settings.json", "--mode", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Project settings"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply and verify deep merge
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("settings.json"))?;

    // Verify merged values:
    assert!(content.contains(r#""debug": true"#));      // overridden
    assert!(content.contains(r#""timeout": 30"#));      // from base
    assert!(content.contains(r#""auth": true"#));       // from base
    assert!(content.contains(r#""logging": true"#));    // added

    Ok(())
}
```

## Key Test Patterns Summary

1. **Always use TestFixture** - Creates isolated temp directory
2. **Call fixture.set_jin_dir() first** - Set JIN_DIR before any Jin operations
3. **Use unique_test_id()** - For parallel-safe unique identifiers
4. **Always use .env("JIN_DIR", &jin_dir)** - Pass to all jin() commands
5. **Use #[serial] attribute** - For tests that modify global state
6. **Verify file existence with assertions** - Use custom assertion helpers
7. **JSON is pretty-printed** - Account for formatting in assertions
8. **Check both file content and metadata** - Verify refs, staging, workspace
