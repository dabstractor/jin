# Quick Start: Rust Integration Testing for CLI Applications

A rapid reference guide to get started with integration testing for Rust CLI applications.

## 5-Minute Setup

### 1. Add Dependencies

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.0"
tempfile = "3.0"
git2 = "0.20"
```

### 2. Create Directory Structure

```
tests/
├── common/
│   ├── mod.rs
│   └── fixtures.rs
├── test_1.rs
├── test_2.rs
└── test_3.rs
```

### 3. Add Shared Module

```rust
// tests/common/mod.rs
pub mod fixtures;

// tests/common/fixtures.rs
use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestEnv {
    _temp: TempDir,
    root: PathBuf,
}

impl TestEnv {
    pub fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().to_path_buf();
        Ok(TestEnv { _temp: temp, root })
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }
}
```

### 4. Write First Integration Test

```rust
// tests/test_basic.rs
use assert_cmd::Command;
mod common;

#[test]
fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("my_app")?
        .arg("--help")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_with_temp_dir() -> Result<(), Box<dyn std::error::Error>> {
    use assert_fs::prelude::*;

    let temp = assert_fs::TempDir::new()?;
    temp.child("test.txt").write_str("content")?;

    Command::cargo_bin("my_app")?
        .arg("process")
        .arg(temp.path().join("test.txt"))
        .assert()
        .success();

    temp.close()?;
    Ok(())
}
```

### 5. Run Tests

```bash
cargo test
```

## Common Test Templates

### Template 1: Simple Command Test

```rust
#[test]
fn test_command_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("my_app")?
        .arg("--version")
        .assert()
        .success();

    Ok(())
}
```

### Template 2: File Processing Test

```rust
#[test]
fn test_file_processing() -> Result<(), Box<dyn std::error::Error>> {
    use assert_fs::prelude::*;
    use predicates::prelude::*;

    let temp = assert_fs::TempDir::new()?;
    let input = temp.child("input.txt");
    input.write_str("test data")?;

    Command::cargo_bin("my_app")?
        .arg("process")
        .arg(input.path())
        .current_dir(temp.path())
        .assert()
        .success();

    let output = temp.child("output.txt");
    output.assert(predicate::path::exists());

    temp.close()?;
    Ok(())
}
```

### Template 3: Error Handling Test

```rust
#[test]
fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    use predicates::prelude::*;

    Command::cargo_bin("my_app")?
        .arg("process")
        .arg("nonexistent.txt")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));

    Ok(())
}
```

### Template 4: Multi-Step Workflow Test

```rust
#[test]
fn test_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Step 1: Initialize
    Command::cargo_bin("my_app")?
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    // Step 2: Process
    Command::cargo_bin("my_app")?
        .arg("process")
        .current_dir(temp.path())
        .assert()
        .success();

    // Step 3: Verify
    Command::cargo_bin("my_app")?
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("complete"));

    temp.close()?;
    Ok(())
}
```

### Template 5: Git Repository Test

```rust
#[test]
fn test_git_operations() -> Result<(), Box<dyn std::error::Error>> {
    use git2::Repository;

    let temp = tempfile::TempDir::new()?;
    let repo = Repository::init(temp.path())?;

    // Create file
    std::fs::write(temp.path().join("file.txt"), "content")?;

    // Commit
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("file.txt"))?;
    index.write()?;

    let sig = git2::Signature::now("Test", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])?;

    // Verify
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    assert_eq!(revwalk.count(), 1);

    Ok(())
}
```

## Common Predicates

```rust
use predicates::prelude::*;

// String predicates
predicate::str::contains("substring")
predicate::str::is_match("regex")
predicate::eq("exact match")

// Path predicates
predicate::path::exists()
predicate::path::missing()
predicate::path::is_file()
predicate::path::is_dir()

// Combining predicates
pred.and(other)
pred.or(other)
pred.not()
```

## Common Assertions

```rust
use assert_cmd::Command;
use predicates::prelude::*;

// Exit codes
cmd.assert().success();
cmd.assert().failure();
cmd.assert().code(1);

// Output
cmd.assert().stdout(predicate::str::contains("text"));
cmd.assert().stderr(predicate::str::contains("error"));

// Exact output
cmd.assert().stdout("exact output\n");
```

## Command Configuration

```rust
use assert_cmd::Command;

let mut cmd = Command::cargo_bin("app")?;

// Arguments
cmd.arg("arg1");
cmd.args(&["arg1", "arg2"]);

// Environment
cmd.env("VAR", "value");
cmd.env_remove("VAR");
cmd.env_clear();

// Working directory
cmd.current_dir("/tmp");

// Input
cmd.write_stdin("input data");
```

## Debugging Tips

### Keep Test Files for Inspection

```rust
#[test]
fn test_with_debug() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Do work

    if std::env::var("DEBUG").is_ok() {
        let persistent = temp.into_persistent();
        println!("Test files at: {}", persistent.path().display());
    } else {
        temp.close()?;
    }

    Ok(())
}
```

Run with: `DEBUG=1 cargo test test_with_debug -- --nocapture`

### Run Tests Sequentially

```bash
cargo test -- --test-threads=1
```

### Show Test Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_name
```

## Project-Specific Patterns for Jin

### Testing CLI Commands

```rust
use assert_cmd::Command;
use assert_fs::TempDir;

#[test]
fn test_jin_init() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::cargo_bin("jin")?
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    temp.close()?;
    Ok(())
}
```

### Testing Git Integration

```rust
use git2::Repository;
use assert_cmd::Command;

#[test]
fn test_jin_with_repo() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::TempDir::new()?;
    Repository::init(temp.path())?;

    Command::cargo_bin("jin")?
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success();

    Ok(())
}
```

### Testing Multi-Command Workflows

```rust
#[test]
fn test_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Initialize
    Command::cargo_bin("jin")?
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    // Add
    Command::cargo_bin("jin")?
        .arg("add")
        .arg("feature")
        .current_dir(temp.path())
        .assert()
        .success();

    // Status
    Command::cargo_bin("jin")?
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("feature"));

    temp.close()?;
    Ok(())
}
```

## File Structure for Jin Project

```
jin/
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── commands/
│   │   ├── init.rs
│   │   ├── apply.rs
│   │   └── mod.rs
│   └── git/
│       └── mod.rs
└── tests/
    ├── common/
    │   ├── mod.rs
    │   └── fixtures.rs
    ├── cli/
    │   ├── init_test.rs
    │   ├── apply_test.rs
    │   └── workflow_test.rs
    └── integration_test.rs
```

## Quick Checklist

- [ ] Add dev dependencies to Cargo.toml
- [ ] Create `tests/` directory if not present
- [ ] Create `tests/common/mod.rs` and `tests/common/fixtures.rs`
- [ ] Write first simple test in `tests/test_basic.rs`
- [ ] Run `cargo test` to verify setup
- [ ] Create fixture struct if needed
- [ ] Add file processing tests
- [ ] Add git-related tests
- [ ] Add multi-step workflow tests
- [ ] Document test organization in code

## Resources

Quick reference documents:
- **Full Fundamentals:** `01_integration_testing_fundamentals.md`
- **CLI Testing:** `02_assert_cmd_and_predicates.md`
- **Files & Fixtures:** `03_tempfile_and_fixtures.md`
- **Git Testing:** `04_git_integration_testing.md`
- **E2E Workflows:** `05_e2e_workflow_testing.md`
- **Organization:** `06_test_organization_and_fixtures.md`

See README.md for complete references and documentation links.

## Next Steps

1. Copy the 5-Minute Setup code
2. Choose relevant test templates
3. Run `cargo test` to validate
4. Refer to full documentation for specific patterns
5. Build test coverage incrementally
