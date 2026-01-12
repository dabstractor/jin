# Rust Cargo Test Execution Guide

## Overview

This guide provides comprehensive information on executing Rust tests with `cargo test`, specifically tailored for developers working on the Jin CLI project. It covers running specific tests, interpreting test output, common patterns, and environment setup.

## Table of Contents

1. [Running Specific Tests](#1-running-specific-tests)
2. [Interpreting Test Output](#2-interpreting-test-output)
3. [Test Output Formats](#3-test-output-formats)
4. [File I/O and Temporary Directories](#4-file-io-and-temporary-directories)
5. [Best Practices](#5-best-practices)
6. [Quick Reference Commands](#6-quick-reference-commands)
7. [Official Documentation URLs](#7-official-documentation-urls)

---

## 1. Running Specific Tests

### Basic Test Execution Patterns

```bash
# Run all tests in the project
cargo test

# Run a specific test by name
cargo test test_name

# Run tests from a specific test file
cargo test --test cli_basic

# Run a specific test from a specific file
cargo test --test cli_basic test_help

# Run tests matching a pattern
cargo test test_mode

# Run with verbose output (shows print! statements)
cargo test -- --nocapture

# Run tests sequentially (useful for debugging)
cargo test -- --test-threads=1

# Run with exact name matching
cargo test -- --exact test_help
```

### Integration Test Specific Commands

```bash
# Run only integration tests (skip unit tests)
cargo test --test '*'

# Run a specific integration test file
cargo test --test cli_basic

# Run a specific test function in an integration test
cargo test --test cli_basic test_help

# Run multiple specific tests
cargo test test_help test_version

# Run all tests in a module
cargo test cli_basic::

# Run tests in a specific module with pattern
cargo test cli_basic::test_mode
```

### Common Test Selection Patterns

```bash
# Run all tests starting with "test_"
cargo test test_

# Run all tests containing "mode"
cargo test mode

# Run all tests in multiple files
cargo test --test cli_basic --test cli_add_local

# Run tests and show output
cargo test test_init_subcommand -- --nocapture --test-threads=1

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Running Tests by Type

```bash
# Run only unit tests (in src/)
cargo test --lib

# Run only integration tests (in tests/)
cargo test --test '*'

# Run doc tests
cargo test --doc

# Run binary tests
cargo test --bin jin
```

---

## 2. Interpreting Test Output

### Success Indicators

```
running 3 tests
test test_help ... ok
test test_version ... ok
test test_init_subcommand ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Key Success Markers:**
- `test <name> ... ok` - Individual test passed
- `test result: ok` - All tests passed
- `<number> passed` - Count of passing tests

### Failure Indicators

```
running 1 test
test test_failure ... FAILED

failures:

---- test_failure stdout ----
thread 'test_failure' panicked at 'assertion failed: `(left == right)`
  left: `1`,
 right: `2`', tests/cli_basic.rs:50:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_failure

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

**Key Failure Markers:**
- `FAILED` after test name
- `panicked at` - Location and reason of panic
- `assertion failed` - Specific assertion that failed
- `test result: FAILED` - Overall test suite failed

### Common Output Sections

#### Doc Test Output
```
test src/lib.rs - (line 25) ... ok
test src/lib.rs - (line 42) ... ok
```

#### Unit Test Output
```
test tests::test_helper_function ... ok
test lib::tests::internal_test ... ok
```

#### Integration Test Output
```
test cli_basic::test_help ... ok
test cli_add_local::test_add_command ... ok
```

### Ignored and Filtered Tests

```
running 10 tests
test test_slow_feature ... ignored
test test_platform_specific ... ignored
test test_normal ... ok

test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 7 filtered out
```

**Understanding the Summary Line:**
```
test result: <status>. <passed> passed; <failed> failed; <ignored> ignored; <measured> measured; <filtered> filtered out
```

- **ignored**: Tests marked with `#[ignore]` attribute
- **measured**: Benchmark tests (using `#[bench]` - deprecated in favor of Criterion)
- **filtered out**: Tests not matching the filter pattern

### With `--nocapture` Flag

```bash
$ cargo test test_init_subcommand -- --nocapture

running 1 test
test test_init_subcommand ... ok

stdout output:
Initialized Jin in /tmp/tmp.XXXXX
Created .jin directory
Set up initial configuration

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 3. Test Output Formats

### Standard Success Format

```
   Compiling jin v0.1.0 (/path/to/jin)
    Finished dev [unoptimized + debuginfo] target(s) in 2.45s
     Running unittests src/lib.rs (target/debug/deps/jin-XXXXX)

running 15 tests
test tests::test_helper ... ok
test tests::test_another ... ok
...

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/cli_basic.rs (target/debug/deps/cli_basic-XXXXX)

running 45 tests
test test_help ... ok
test test_version ... ok
...

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

   Doc tests jin

running 3 tests
test src/lib.rs - (line 25) ... ok
...

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Failure Format with Context

```
running 1 test
test test_add_without_init ... FAILED

failures:

---- test_add_without_init stdout ----
thread 'test_add_without_init' panicked at 'assertion `left == right` failed:
  left: `"Jin not initialized"`,
 right: `"Success"`', tests/cli_basic.rs:385:10
stack backtrace:
   0: rust_begin_unwind
             at /rustc/library/std/src/panicking.rs:584:5
   1: core::panicking::panic_fmt
             at /rustc/library/core/src/panicking.rs:142:14
   2: core::panicking::assert_failed_inner
   3: core::panicking::assert_failed
   ...

note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    test_add_without_init

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

### What to Look For in Test Output

#### 1. **Compilation Status**
```
   Compiling jin v0.1.0 ...
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```
- If compilation fails, tests won't run
- Check compilation errors before test failures

#### 2. **Test Execution Phases**
```
     Running unittests src/lib.rs
     Running tests/cli_basic.rs
     Running tests/cli_add_local.rs
```
- Multiple test binaries run sequentially
- Each file compiles to its own test binary

#### 3. **Individual Test Results**
```
test test_name ... ok          # Passed
test test_name ... FAILED      # Failed
test test_name ... ignored     # Ignored
```

#### 4. **Summary Line**
```
test result: ok. 45 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```
- **ok**: All tests passed
- **FAILED**: At least one test failed
- **passed/failed**: Counts of each
- **ignored**: Tests marked with `#[ignore]`
- **filtered**: Tests excluded by pattern matching

#### 5. **Failure Details**
- **Location**: File and line number
- **Assertion type**: `assert_eq!`, `assert!`, `panic!`
- **Expected vs Actual**: For assertion failures
- **Backtrace**: Stack trace (enable with `RUST_BACKTRACE=1`)

### Color Coding (in terminals)

- **Green**: Passing tests
- **Red**: Failing tests
- **Yellow/Cyan**: Ignored tests (depends on terminal)
- **Bold**: Test names and important information

---

## 4. File I/O and Temporary Directories

### The `tempfile` Crate Pattern

The Jin project uses the `tempfile` crate for safe temporary file handling:

```rust
use tempfile::TempDir;

#[test]
fn test_with_temp_dir() {
    // Create temporary directory
    let temp = TempDir::new().unwrap();

    // Use the directory
    let test_file = temp.path().join("test.txt");
    std::fs::write(&test_file, "content").unwrap();

    // Directory is automatically deleted when temp goes out of scope
}
```

**Key Points:**
- `TempDir::new()` creates a unique temporary directory
- Directory is automatically deleted when the `TempDir` object is dropped
- Must keep `TempDir` in scope while test runs

### Fixture Pattern (Used in Jin)

```rust
// tests/common/fixtures.rs
pub struct TestFixture {
    _tempdir: TempDir,  // Underscore prefix = kept for cleanup, not directly used
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        Ok(TestFixture {
            _tempdir: tempdir,  // CRITICAL: Keep to prevent cleanup
            path,
            jin_dir: Some(path.join(".jin_global")),
        })
    }
}
```

**Critical Gotcha:**
```rust
// WRONG - TempDir cleaned up immediately
#[test]
fn test_wrong() {
    let path = TempDir::new().unwrap().path().to_path_buf();
    // path is now invalid - directory already deleted!
}

// RIGHT - Keep TempDir in scope
#[test]
fn test_right() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    // path is valid as long as temp is in scope
}
```

### Isolation with Environment Variables

```rust
#[test]
fn test_with_isolated_jin_dir() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Set environment variable for test isolation
    std::env::set_var("JIN_DIR", &jin_dir);

    // Run test commands
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}
```

### Cleanup Patterns

#### Automatic Cleanup (Recommended)
```rust
#[test]
fn test_auto_cleanup() {
    let temp = TempDir::new().unwrap();
    // Test code here
    // temp dropped automatically at end of function
}
```

#### Manual Cleanup (for debugging)
```rust
#[test]
fn test_manual_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    // Test code
    perform_test(temp.path())?;

    // Explicit close (optional)
    temp.close()?;
    Ok(())
}
```

#### Conditional Persistence (Debugging)
```rust
#[test]
fn test_persistent_debug() {
    let temp = if std::env::var("KEEP_TEST_FILES").is_ok() {
        TempDir::new().unwrap().into_path()
    } else {
        TempDir::new().unwrap().into_temp_path()
    };

    // Test code
    // Files persist if KEEP_TEST_FILES=1 is set
}
```

### Git Lock Cleanup Pattern

```rust
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

### File I/O Testing Patterns

#### Creating Test Files
```rust
#[test]
fn test_file_creation() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.json");

    std::fs::write(&file_path, r#"{"key": "value"}"#).unwrap();

    assert!(file_path.exists());
    assert_eq!(
        std::fs::read_to_string(&file_path).unwrap(),
        r#"{"key": "value"}"#
    );
}
```

#### Reading Test Files
```rust
#[test]
fn test_file_reading() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("data.txt");

    std::fs::write(&file_path, "test data").unwrap();

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "test data");
}
```

#### Directory Structure Testing
```rust
#[test]
fn test_directory_structure() {
    let temp = TempDir::new().unwrap();

    // Create nested directories
    let dir1 = temp.path().join(".jin/staging");
    std::fs::create_dir_all(&dir1).unwrap();

    let dir2 = temp.path().join(".jin/layers");
    std::fs::create_dir_all(&dir2).unwrap();

    assert!(dir1.exists());
    assert!(dir2.is_dir());
}
```

### Assertions for File I/O

```rust
// Custom assertion (as used in Jin)
pub fn assert_workspace_file(project_path: &Path, file: &str, expected_content: &str) {
    let file_path = project_path.join(file);
    assert!(
        file_path.exists(),
        "Workspace file {} should exist at {:?}",
        file,
        file_path
    );

    let actual_content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read file {:?}: {}", file_path, e));

    assert_eq!(
        actual_content, expected_content,
        "Workspace file {} content mismatch.\nExpected: {}\nActual: {}",
        file, expected_content, actual_content
    );
}
```

---

## 5. Best Practices

### Test Execution Best Practices

#### 1. Use Parallel Execution by Default
```rust
// Tests should be isolated and run in parallel
#[test]
fn test_isolated() {
    let temp = TempDir::new().unwrap();
    // Each test gets its own temp directory
}
```

#### 2. Use Serial Test for Global State
```rust
use serial_test::serial;

#[test]
#[serial]  // Runs sequentially with other #[serial] tests
fn test_with_global_state() {
    // Safe to use global state
}
```

#### 3. Unique Identifiers for Parallel Tests
```rust
// Jin's pattern for unique test IDs
pub fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", std::process::id(), count)
}

#[test]
fn test_with_unique_mode() {
    let mode_name = format!("test_mode_{}", unique_test_id());
    // Safe for parallel execution
}
```

#### 4. Descriptive Test Names
```rust
// Good
#[test]
fn test_init_creates_jin_directory_and_context_file() {
    // Clear what this tests
}

// Bad
#[test]
fn test_init_works() {
    // Vague
}
```

### Test Organization Best Practices

#### 1. Group Related Tests
```
tests/
├── cli_basic.rs         # Basic CLI commands
├── cli_add_local.rs     # Add command variations
├── cli_diff.rs          # Diff command
├── mode_scope_workflow.rs  # Mode and scope workflows
└── common/
    ├── fixtures.rs      # Test fixtures
    ├── assertions.rs    # Custom assertions
    └── git_helpers.rs   # Git utilities
```

#### 2. Use Shared Fixtures
```rust
// tests/common/mod.rs
pub mod fixtures;
pub mod assertions;
pub mod git_helpers;

// tests/cli_basic.rs
use crate::common::fixtures;

#[test]
fn test_using_fixture() {
    let fixture = fixtures::setup_test_repo().unwrap();
    // Use fixture
}
```

#### 3. Separate Unit and Integration Tests
```
src/
└── lib.rs              # Unit tests in #[cfg(test)] modules
tests/
└── integration.rs      # Integration tests
```

### Environment Setup for Tests

#### 1. Isolate Global State
```rust
#[test]
fn test_with_isolated_jin_dir() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Always set JIN_DIR for test isolation
    std::env::set_var("JIN_DIR", &jin_dir);

    // Test code
}
```

#### 2. Clean Environment Between Tests
```rust
#[test]
fn test_clean_environment() {
    // Save original value
    let original = std::env::var("JIN_DIR").ok();

    // Set test value
    std::env::set_var("JIN_DIR", "/tmp/test_jin");

    // Test code

    // Restore original
    match original {
        Some(val) => std::env::set_var("JIN_DIR", val),
        None => std::env::remove_var("JIN_DIR"),
    }
}
```

#### 3. Use Test-Specific Configuration
```rust
#[test]
fn test_with_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");

    // Write test-specific config
    std::fs::write(
        &config_path,
        r#"{"mode": "test", "scope": "test"}"#
    ).unwrap();

    // Use config in test
}
```

### Test Validation Best Practices

#### 1. Test Success AND Failure Paths
```rust
#[test]
fn test_init_success() {
    let temp = TempDir::new().unwrap();
    jin().arg("init").current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_init_in_existing_jin_directory_fails() {
    let temp = TempDir::new().unwrap();
    jin().arg("init").current_dir(temp.path())
        .assert()
        .success();

    // Second init should fail
    jin().arg("init").current_dir(temp.path())
        .assert()
        .failure();
}
```

#### 2. Use Predicates for Flexible Matching
```rust
use predicates::prelude::*;

#[test]
fn test_flexible_output() {
    jin().arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Mode:"))
        .stdout(predicate::str::contains("active"));
}
```

#### 3. Verify State, Not Just Output
```rust
#[test]
fn test_state_verification() {
    let temp = TempDir::new().unwrap();

    // Run command
    jin().arg("init").current_dir(temp.path())
        .assert()
        .success();

    // Verify actual state
    assert!(temp.path().join(".jin").exists());
    assert!(temp.path().join(".jin/context").exists());
}
```

### Debugging Tests

#### 1. Use `--nocapture` for Output
```bash
cargo test test_name -- --nocapture
```

#### 2. Use `--test-threads=1` for Sequential Execution
```bash
cargo test -- --test-threads=1
```

#### 3. Use `RUST_BACKTRACE` for Panic Details
```bash
RUST_BACKTRACE=1 cargo test
RUST_BACKTRACE=full cargo test
```

#### 4. Use `--verbose` for Compilation Details
```bash
cargo test --verbose
```

#### 5. Run Single Test
```bash
cargo test --test cli_basic test_help -- --nocapture --test-threads=1
```

### Performance Considerations

#### 1. Keep Tests Fast
```rust
// Good: Fast unit test
#[test]
fn test_parser() {
    let result = parse_config("key: value");
    assert_eq!(result.key, "value");
}

// Bad: Slow integration test for simple logic
#[test]
fn test_parser_slow() {
    let temp = TempDir::new().unwrap();
    jin().arg("parse").current_dir(temp.path())
        .assert();
}
```

#### 2. Use Mocks for Expensive Operations
```rust
// Mock filesystem for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_with_mock() {
        let mut mock = MockFileSystem::new();
        mock.expect_read()
            .returning(|| Ok("mocked content".to_string()));

        // Test with mock
    }
}
```

#### 3. Share Expensive Setup
```rust
use once_cell::sync::Lazy;

static EXPENSIVE_FIXTURE: Lazy<TestFixture> = Lazy::new(|| {
    TestFixture::new().unwrap()
});

#[test]
fn test_shared_fixture() {
    // Use shared fixture for multiple tests
}
```

---

## 6. Quick Reference Commands

### Essential Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run specific test file
cargo test --test cli_basic

# Run with output
cargo test -- --nocapture

# Run sequentially
cargo test -- --test-threads=1

# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run ignored tests
cargo test -- --ignored

# Run exact test name
cargo test -- --exact test_help

# Show test list
cargo test -- --list

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Common Patterns

```bash
# Run all CLI tests
cargo test --test 'cli_*'

# Run all workflow tests
cargo test workflow

# Run single test with full output
cargo test test_help -- --nocapture --test-threads=1 --exact

# Run tests in a specific file with pattern
cargo test --test cli_basic test_mode

# Run only integration tests
cargo test --test '*'

# Run only unit tests
cargo test --lib
```

### Debugging Commands

```bash
# Run with verbose compilation
cargo test --verbose

# Run with environment variable
JIN_DIR=/tmp/test cargo test

# Run with full backtrace
RUST_BACKTRACE=full cargo test

# Keep test files after run
KEEP_TEST_FILES=1 cargo test

# Run single test with maximum debugging
RUST_BACKTRACE=full cargo test test_name -- --nocapture --test-threads=1 --exact
```

### Filtering Commands

```bash
# Run tests matching pattern
cargo test mode

# Run tests not matching pattern
cargo test -- mode  # Note: negation not directly supported

# Run tests in specific module
cargo test cli_basic::

# Run tests in multiple files
cargo test --test cli_basic --test cli_add_local
```

### CI/CD Commands

```bash
# Run all tests with output on failure
cargo test --no-fail-fast

# Run tests with specific features
cargo test --features "test-feature"

# Run tests in release mode
cargo test --release

# Run all test types
cargo test --all-targets
```

---

## 7. Official Documentation URLs

### Core Rust Testing Documentation

1. **The Rust Programming Language - Testing**
   - URL: https://doc.rust-lang.org/book/ch11-00-testing.html
   - Covers: How to write tests, test organization, running tests

2. **The Rust Book - Test Organization**
   - URL: https://doc.rust-lang.org/book/ch11-03-test-organization.html
   - Covers: Unit tests vs integration tests, directory structure

3. **Rust by Example - Testing**
   - URL: https://doc.rust-lang.org/rust-by-example/testing.html
   - Covers: Test examples, assertions, custom failures

4. **Rust by Example - Integration Testing**
   - URL: https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
   - Covers: Integration test setup and patterns

5. **Cargo Documentation - cargo-test**
   - URL: https://doc.rust-lang.org/cargo/commands/cargo-test.html
   - Covers: All cargo test options and flags

6. **Rust Reference - Testing**
   - URL: https://doc.rust-lang.org/reference/attributes/testing.html
   - Covers: Test attributes, test harness, compile-time testing

### CLI Testing Frameworks

7. **assert_cmd Documentation**
   - URL: https://docs.rs/assert_cmd/latest/assert_cmd/
   - Covers: Testing CLI binaries, command assertions

8. **predicates Documentation**
   - URL: https://docs.rs/predicates/latest/predicates/
   - Covers: Predicate-based assertions for flexible matching

9. **assert_fs Documentation**
   - URL: https://docs.rs/assert_fs/latest/assert_fs/
   - Covers: Filesystem assertions for tests

10. **tempfile Documentation**
    - URL: https://docs.rs/tempfile/latest/tempfile/
    - Covers: Temporary file and directory management

11. **serial_test Documentation**
    - URL: https://docs.rs/serial_test/latest/serial_test/
    - Covers: Serial test execution for shared state

### Rust CLI Book

12. **Command Line Applications in Rust - Testing**
    - URL: https://rust-cli.github.io/book/tutorial/testing.html
    - Covers: Comprehensive CLI testing guide

13. **Command Line Applications in Rust - Output**
    - URL: https://rust-cli.github.io/book/information/output.html
    - Covers: Standard output, error output, exit codes

### Git Testing

14. **git2-rs Documentation**
    - URL: https://docs.rs/git2/latest/git2/
    - Covers: Git operations in Rust for testing

### Best Practices

15. **Rust Testing Best Practices (Blog)**
    - URL: https://blog.logrocket.com/how-to-organize-rust-tests/
    - Covers: Test organization strategies

16. **Integration Testing Rust Binaries (Blog)**
    - URL: https://www.unwoundstack.com/blog/integration-testing-rust-binaries.html
    - Covers: Real-world integration testing patterns

### Error Handling

17. **The Rust Book - Error Handling**
    - URL: https://doc.rust-lang.org/book/ch09-00-error-handling.html
    - Covers: Error handling patterns for tests

18. **thiserror Documentation**
    - URL: https://docs.rs/thiserror/latest/thiserror/
    - Covers: Derive macros for error types

### Project-Specific Resources

19. **Jin Project Test Patterns**
    - File: `/home/dustin/projects/jin/plan/P6M2/research/README.md`
    - Covers: Project-specific testing patterns and fixtures

20. **Jin Project Quick Reference**
    - File: `/home/dustin/projects/jin/plan/P6M2/research/QUICK_REFERENCE.md`
    - Covers: Quick reference for Jin testing

21. **Jin Common Test Fixtures**
    - File: `/home/dustin/projects/jin/tests/common/fixtures.rs`
    - Covers: Fixture implementation examples

22. **Jin Common Test Assertions**
    - File: `/home/dustin/projects/jin/tests/common/assertions.rs`
    - Covers: Custom assertion patterns

---

## Appendices

### A. Common Test Attributes

```rust
#[test]              // Basic test
#[ignore]            // Skip unless --ignored is used
#[should_panic]      // Test expects a panic
#[should_panic(expected = "specific message")]  // Panic with specific message
#[serial]            // Run sequentially (from serial_test crate)
```

### B. Common Assertion Macros

```rust
assert!(condition);                           // Basic assertion
assert_eq!(left, right);                      // Equality
assert_ne!(left, right);                      // Inequality
assert_matches!(expression, pattern);         // Pattern matching
assert_matches::assert_matches!(expr, pat);   // From assert_matches crate
```

### C. Test Output Colors

- Green: Passing tests
- Red: Failing tests
- Yellow/Cyan: Ignored tests
- Bold: Test names, important information

### D. Exit Code Conventions

```
0   = All tests passed
101 = Test executable failed to compile
1   = One or more tests failed
```

### E. Environment Variables for Testing

```bash
RUST_LOG=debug          # Enable debug logging
RUST_BACKTRACE=1        # Enable backtrace on panic
RUST_BACKTRACE=full     # Enable full backtrace
CARGO_TARGET_DIR=...    # Override target directory
KEEP_TEST_FILES=1       # Keep temp files (custom)
JIN_DIR=/path/to/dir    # Jin-specific (project)
```

---

## Summary

This guide provides comprehensive coverage of Rust testing patterns for `cargo test` execution, specifically tailored for the Jin CLI project. Key takeaways:

1. **Running Tests**: Use specific patterns to run individual tests, test files, or test suites
2. **Interpreting Output**: Understand success/failure indicators and what to look for
3. **File I/O**: Use `tempfile` crate with proper fixture patterns for safe temporary directory handling
4. **Best Practices**: Isolate tests, use parallel execution, test both success and failure paths
5. **Environment Setup**: Use environment variables for test isolation
6. **Official Resources**: Refer to official Rust documentation for authoritative information

The Jin project already implements many of these patterns in `/home/dustin/projects/jin/tests/common/` - use these as examples for writing new tests.

**Last Updated**: January 11, 2026
**Project**: Jin CLI
**Rust Edition**: 2021
**Cargo Version**: Compatible with 1.70+
