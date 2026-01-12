# Testing Multi-Layer Git Operations and Version Control Patterns

## Overview

Testing Git operations requires special considerations due to:
- Repository state management
- Lock files and concurrent access
- Branch and reference manipulation
- Remote repository interactions
- Multi-layer abstraction (as in jin's case)

## Key Patterns from jin Project

### 1. Git Repository Initialization in Tests

**File**: `/home/dustin/projects/jin/tests/common/fixtures.rs`

```rust
use git2::Repository;

pub fn jin_init(path: &Path, jin_dir: Option<&PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Git repository in project directory first
    // This is needed for tests that use create_commit_in_repo
    git2::Repository::init(path)?;

    let mut binding = jin();
    let mut cmd = binding.arg("init").current_dir(path);
    if let Some(jin_dir) = jin_dir {
        cmd = cmd.env("JIN_DIR", jin_dir);
    }
    cmd.assert().success();
    Ok(())
}
```

**Key Insight**: Initialize Git repo before testing higher-level operations.

### 2. Remote Repository Setup

```rust
pub fn setup_jin_with_remote() -> Result<RemoteFixture, Box<dyn std::error::Error>> {
    let fixture = RemoteFixture::new()?;

    // Initialize Jin in local directory with isolated JIN_DIR
    jin_init(&fixture.local_path, fixture.jin_dir.as_ref())?;

    // Initialize bare Git repository as remote
    git2::Repository::init_bare(&fixture.remote_path)?;

    Ok(fixture)
}
```

**Pattern**: Use bare repositories for remotes in tests.

### 3. Git Lock Cleanup

**Critical for preventing test failures**

```rust
pub fn cleanup_git_locks(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo_path.join(".git");

    if !git_dir.exists() {
        return Ok(());
    }

    // Clean common lock files
    let lock_files = vec![
        ("index.lock", false),
        ("config.lock", false),
        ("HEAD.lock", false),
        ("packed-refs.lock", false),
    ];

    for (lock_file, _) in lock_files {
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

    Ok(())
}
```

**Why This Matters**: Git operations that fail or are interrupted can leave lock files that cause subsequent tests to fail with "index lock exists" errors.

### 4. Automatic Cleanup with Drop

```rust
impl Drop for TestFixture {
    fn drop(&mut self) {
        // CRITICAL: Clean up Git locks before temp dir is deleted
        let _ = cleanup_git_locks(&self.path);

        // Also clean up Jin directory locks if it exists
        if let Some(ref jin_dir) = self.jin_dir {
            let _ = cleanup_git_locks(jin_dir);
        }
    }
}
```

**Pattern**: Always clean up Git locks in Drop implementation.

## Testing Multi-Layer Git Operations

The jin project implements a "phantom Git layer system" where files are stored in different Git repositories (layers). Testing this requires:

### 1. Layer Isolation Testing

```rust
#[test]
fn test_add_local_routes_to_layer_8() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();

    let test_file = fixture.path().join(".config/settings.json");
    fs::create_dir_all(test_file.parent().unwrap())?;
    fs::write(&test_file, r#"{"theme": "dark"}"#)?;

    // ACT: Add file with --local flag
    jin()
        .args(["add", ".config/settings.json", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged"))
        .stdout(predicate::str::contains("user-local"));

    // ASSERT: File is staged in Layer 8 (UserLocal)
    let staging_index = fixture.jin_dir.as_ref().unwrap().join("staging/index.json");
    assert!(staging_index.exists());
    let staging_content = fs::read_to_string(&staging_index)?;
    assert!(staging_content.contains("UserLocal"));

    Ok(())
}
```

### 2. Cross-Layer Conflict Testing

```rust
#[test]
fn test_structured_file_auto_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add config.json to ModeBase layer (Layer 2)
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"common": {"a": 1}, "mode": true}"#)?;

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

    // Modify config.json and add to ModeProject layer (Layer 5)
    fs::write(
        &config_path,
        r#"{"common": {"a": 1, "b": 2}, "project": false}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode", "--project"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to project"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove from workspace to test apply behavior
    fs::remove_file(&config_path)?;

    // Run apply - should NOT create .jinmerge (structured files auto-merge)
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify no .jinmerge file was created
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(
        !jinmerge_path.exists(),
        "No .jinmerge should exist for structured files that can deep merge"
    );

    // Verify merged content contains both layers' keys
    let content = fs::read_to_string(&config_path)?;
    assert!(content.contains(r#""a": 1"#));
    assert!(content.contains(r#""b": 2"#));
    assert!(content.contains(r#""mode": true"#));
    assert!(content.contains(r#""project": false"#));

    Ok(())
}
```

## Testing Git Workflows

### 1. Multi-Command Workflow Testing

```rust
#[test]
fn test_full_workflow_conflict_to_resolution() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // 1. Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 2. Activate the mode
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 3. Add file to global layer
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

    // 4. Modify and add to mode layer (creates conflict)
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

    // 5. Remove from workspace
    fs::remove_file(&config_path).unwrap();

    // 6. Run apply - should create .jinmerge
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // 7. Verify .jinmerge file
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists());

    // 8. Verify paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(paused_state_path.exists());

    // 9. Check status shows conflicts
    jin_cmd()
        .arg("status")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge conflicts"));

    // 10. Resolve the conflict
    fs::write(
        &jinmerge_path,
        r#"{"port": 9999}"#
    )
    .unwrap();

    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // 11-15. Verification steps...
}
```

### 2. Nested Object Deep Merge Testing

```rust
#[test]
fn test_nested_object_deep_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add config.json to ModeBase layer (Layer 2) with 3-level nested structure
    let config_path = fixture.path().join("config.json");
    fs::write(
        &config_path,
        r#"{"config": {"database": {"host": "localhost", "port": 5432}}, "app": "base"}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add base config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Overwrite config.json and add to ProjectBase layer (Layer 7)
    fs::write(
        &config_path,
        r#"{"config": {"database": {"port": 5433, "ssl": true}}, "app": "override"}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add override config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Apply merge to workspace
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify merged result matches expected output
    let merged_content = fs::read_to_string(&config_path)?;
    let merged: serde_json::Value = serde_json::from_str(&merged_content)?;

    // Verify nested merge behavior
    assert_eq!(
        merged["config"]["database"]["host"], "localhost",
        "host should be preserved from ModeBase"
    );
    assert_eq!(
        merged["config"]["database"]["port"], 5433,
        "port should be overridden by ProjectBase"
    );
    assert_eq!(
        merged["config"]["database"]["ssl"], true,
        "ssl should be added from ProjectBase"
    );
    assert_eq!(
        merged["app"], "override",
        "app should be overridden by ProjectBase"
    );

    Ok(())
}
```

## Best Practices for Git Operation Testing

1. **Always initialize Git repos in temp directories**
2. **Clean up lock files in Drop implementations**
3. **Use bare repositories for remote testing**
4. **Test both success and failure paths**
5. **Verify repository state after operations**
6. **Use unique identifiers for branches/modes/scopes**
7. **Test concurrent access scenarios with serial_test**
8. **Validate file contents, not just existence**
9. **Test edge cases (empty repos, conflicts, etc.)**
10. **Use descriptive test names for workflows**

## Common Git Testing Scenarios

### 1. Repository State Verification
```rust
let repo = git2::Repository::open(&path)?;
let head = repo.head()?;
assert_eq!(head.shorthand(), Some("main"));
```

### 2. Commit Verification
```rust
let repo = git2::Repository::open(&path)?;
let head_commit = repo.head()?.peel_to_commit()?;
assert_eq!(head_commit.message(), Some("Add config"));
```

### 3. Branch Testing
```rust
let repo = git2::Repository::open(&path)?;
let branches = repo.branches(Some(git2::BranchType::Local))?;
assert!(branches.count() > 0);
```

### 4. Remote Testing
```rust
let repo = git2::Repository::open(&path)?;
let remotes = repo.remotes()?;
assert!(remotes.len() > 0);
```

## Additional Resources

- git2-rs documentation: https://docs.rs/git2/
- Git internals: https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain
- Testing Git operations: https://jwiegley.github.io/git2/
- Git lock file mechanism: https://git-scm.com/docs/gitglossary#Documentation/gitglossary.txt-aiddeflockfilealockfile
