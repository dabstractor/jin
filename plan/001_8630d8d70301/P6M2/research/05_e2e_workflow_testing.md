# End-to-End Workflow Testing for CLI Applications

## Overview

End-to-end (E2E) testing validates complete user workflows from start to finish. For CLI applications, this means orchestrating multiple command invocations, verifying state changes, and ensuring all components work together correctly.

## Three Main Testing Approaches

### 1. Standard Library Process Module

Using `std::process::Command` with minimal abstraction:

```rust
use std::process::Command;

#[test]
fn test_basic_workflow() {
    let output = Command::new("my_app")
        .arg("--version")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("1.0.0"));
}
```

**Advantages:**
- No external dependencies
- Direct control over process execution
- Suitable for simple tests

**Disadvantages:**
- Verbose error handling
- Less ergonomic assertions
- Manual file cleanup

### 2. Testing Frameworks (assert_cmd)

Using specialized testing libraries:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_with_assert_cmd() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("my_app")?
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0"));

    Ok(())
}
```

**Advantages:**
- Clean assertion syntax
- Better error messages
- Integrated with Rust ecosystem
- Works with temporary files

**Disadvantages:**
- Additional dependency
- Learning curve

**Best Choice for Most CLI Testing**

### 3. Mocking Dependencies

Isolating application logic from external systems:

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    ExternalService {}
    impl Service for ExternalService {
        fn call(&self, input: &str) -> Result<String, Error>;
    }
}

#[test]
fn test_with_mocking() {
    let mut mock_service = MockExternalService::new();
    mock_service.expect_call()
        .with(eq("input"))
        .times(1)
        .returning(|_| Ok("output".to_string()));

    // Run code that uses mock_service
}
```

**Advantages:**
- Test in isolation from external services
- Deterministic test behavior
- No network required

**Disadvantages:**
- Doesn't test actual integrations
- Mocks can diverge from reality

## Multi-Step Workflow Patterns

### Pattern 1: Sequential Commands with State Verification

```rust
use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn test_file_processing_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let input_file = temp.child("input.txt");
    input_file.write_str("data to process")?;

    // Step 1: Process file
    Command::cargo_bin("my_app")?
        .arg("process")
        .arg(input_file.path())
        .current_dir(temp.path())
        .assert()
        .success();

    // Step 2: Verify output file created
    let output_file = temp.child("output.txt");
    output_file.assert(predicate::path::exists());

    // Step 3: Verify output content
    output_file.assert(predicate::str::contains("processed"));

    // Step 4: Run post-processing
    Command::cargo_bin("my_app")?
        .arg("validate")
        .arg(output_file.path())
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid"));

    temp.close()?;
    Ok(())
}
```

### Pattern 2: Complex State Machine Workflow

```rust
use assert_cmd::Command;
use assert_fs::TempDir;
use assert_fs::prelude::*;

#[test]
fn test_multi_step_state_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Step 1: Initialize
    Command::cargo_bin("my_app")?
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    temp.child("config.toml").assert(predicates::path::exists());

    // Step 2: Add resource
    Command::cargo_bin("my_app")?
        .arg("add")
        .arg("resource1")
        .current_dir(temp.path())
        .assert()
        .success();

    // Step 3: List resources
    Command::cargo_bin("my_app")?
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("resource1"));

    // Step 4: Remove resource
    Command::cargo_bin("my_app")?
        .arg("remove")
        .arg("resource1")
        .current_dir(temp.path())
        .assert()
        .success();

    // Step 5: Verify removal
    Command::cargo_bin("my_app")?
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("resource1").not());

    temp.close()?;
    Ok(())
}
```

### Pattern 3: Testing Error Recovery

```rust
use assert_cmd::Command;
use assert_fs::TempDir;

#[test]
fn test_error_recovery_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Step 1: Attempt invalid operation
    Command::cargo_bin("my_app")?
        .arg("process")
        .arg("nonexistent.txt")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("File not found"));

    // Step 2: Create valid file
    temp.child("valid.txt").write_str("data")?;

    // Step 3: Retry with valid file (should succeed)
    Command::cargo_bin("my_app")?
        .arg("process")
        .arg("valid.txt")
        .current_dir(temp.path())
        .assert()
        .success();

    temp.close()?;
    Ok(())
}
```

### Pattern 4: Git-Based Workflow

```rust
use assert_cmd::Command;
use assert_fs::TempDir;
use git2::Repository;
use std::fs;

#[test]
fn test_git_cli_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Step 1: Initialize git repo
    Repository::init(temp.path())?;

    // Step 2: Create files and commit
    temp.child("file1.txt").write_str("content1")?;

    let repo = Repository::open(temp.path())?;
    create_commit(&repo, temp.path(), "file1.txt", "Initial commit")?;

    // Step 3: Run CLI command to list commits
    Command::cargo_bin("my_git_tool")?
        .arg("log")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Initial commit"));

    // Step 4: Create new file and commit
    temp.child("file2.txt").write_str("content2")?;
    create_commit(&repo, temp.path(), "file2.txt", "Add file2")?;

    // Step 5: Verify log shows both commits
    Command::cargo_bin("my_git_tool")?
        .arg("log")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Initial commit"))
        .stdout(predicates::str::contains("Add file2"));

    temp.close()?;
    Ok(())
}

fn create_commit(repo: &Repository, path: &std::path::Path, file: &str, msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new(file))?;
    index.write()?;

    let sig = git2::Signature::now("Test", "test@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head = repo.head()?;
    let parent = repo.find_commit(head.target().unwrap())?;

    repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&parent])?;
    Ok(())
}
```

### Pattern 5: Testing with Environment Variables

```rust
use assert_cmd::Command;

#[test]
fn test_workflow_with_env_vars() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Run with default environment
    Command::cargo_bin("my_app")?
        .arg("status")
        .assert()
        .success();

    // Step 2: Run with custom configuration
    Command::cargo_bin("my_app")?
        .env("MY_APP_DEBUG", "1")
        .env("MY_APP_CONFIG", "/custom/path")
        .arg("status")
        .assert()
        .success()
        .stdout(predicates::str::contains("DEBUG"));

    // Step 3: Clear environment variables
    Command::cargo_bin("my_app")?
        .env_clear()
        .arg("status")
        .assert()
        .failure();  // May fail without required env vars

    Ok(())
}
```

## Best Practices for E2E Testing

### 1. **Independence and Repeatability**

Each test should be completely independent:

```rust
#[test]
fn workflow_1() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;  // Fresh temp dir
    // Test operations
    temp.close()?;
    Ok(())
}

#[test]
fn workflow_2() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;  // Different temp dir
    // Test operations
    temp.close()?;
    Ok(())
}

// Tests can run in parallel, in any order, multiple times with same result
```

**Why:**
- Tests can run in parallel
- No state pollution between tests
- Tests are idempotent

### 2. **Comprehensive Coverage**

Test a wide variety of scenarios:

```rust
#[test]
fn normal_workflow() { /* ... */ }

#[test]
fn edge_case_empty_input() { /* ... */ }

#[test]
fn error_handling_invalid_input() { /* ... */ }

#[test]
fn error_recovery_after_failure() { /* ... */ }

#[test]
fn large_data_processing() { /* ... */ }
```

### 3. **Meaningful Assertions**

Use specific, expressive assertions:

```rust
// POOR: Generic assertion
assert!(output.contains(""));  // What are we checking?

// GOOD: Specific, documented assertion
cmd.assert()
    .success()
    .stdout(predicate::str::contains("processed 100 files"));
```

### 4. **Test Data Isolation**

Keep test data separate from production:

```rust
// Organize fixtures clearly
tests/
├── fixtures/
│   ├── valid_input/
│   ├── invalid_input/
│   └── edge_cases/
└── e2e_tests.rs

#[test]
fn test_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/valid_input");

    let temp = TempDir::new()?;
    fs::copy(&fixture_path, temp.path().join("input"))?;

    // Use temp copy of fixture
    Ok(())
}
```

### 5. **CI Integration**

Design tests to work in CI environments:

```rust
#[test]
#[ignore]  // Run with: cargo test -- --include-ignored
fn test_slow_network_operation() -> Result<(), Box<dyn std::error::Error>> {
    // Long-running test that might timeout in CI
    Ok(())
}

#[test]
fn test_basic_fast_operation() {
    // Always runs, completes quickly
}
```

## Debugging Workflow Tests

### Temporary File Persistence

```rust
use assert_fs::TempDir;
use std::env;

#[test]
fn test_preserve_on_failure() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Do test work
    std::fs::write(temp.path().join("debug.txt"), "debug data")?;

    // Preserve files for debugging
    if env::var("KEEP_TEST_FILES").is_ok() {
        let persistent = temp.into_persistent();
        println!("Test files: {}", persistent.path().display());
    } else {
        temp.close()?;
    }

    Ok(())
}
```

Run with: `KEEP_TEST_FILES=1 cargo test test_preserve_on_failure`

### Verbose Logging

```rust
#[test]
fn test_with_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for test
    let _ = env_logger::builder().is_test(true).try_init();

    log::info!("Starting test workflow");

    let temp = TempDir::new()?;
    log::debug!("Created temp dir: {}", temp.path().display());

    // Run test
    log::info!("Test completed successfully");

    temp.close()?;
    Ok(())
}
```

Run with: `RUST_LOG=debug cargo test test_with_logging -- --nocapture`

## Common Pitfalls

| Pitfall | Solution |
|---------|----------|
| Tests don't clean up | Use TempDir, assert_fs handles it automatically |
| Tests interfere with each other | Use separate TempDir for each test |
| Flaky tests from timing | Use synchronous commands, avoid sleeps |
| Tests too slow | Focus on critical workflows, use units for edge cases |
| Hard to debug failures | Keep files on debug flag, use verbose logging |

## References

- [Testing - Command Line Applications in Rust](https://rust-cli.github.io/book/tutorial/testing.html)
- [Approaches for E2E Testing - Sling Academy](https://www.slingacademy.com/article/approaches-for-end-to-end-testing-in-rust-cli-applications/)
- [End-to-End Testing for Rust Web Services - LogRocket](https://blog.logrocket.com/end-to-end-testing-for-rust-web-services/)
- [How to Test Rust Command Line Applications - Neil Henning](https://www.neilhenning.dev/posts/rust-lit/)
