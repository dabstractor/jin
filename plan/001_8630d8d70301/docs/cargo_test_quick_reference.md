# Cargo Test Quick Reference Card

## Essential Commands (Copy & Paste)

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run specific test file
cargo test --test cli_basic

# Run with output visible
cargo test -- --nocapture

# Run sequentially (debugging)
cargo test -- --test-threads=1

# Run specific test with output
cargo test test_name -- --nocapture --test-threads=1

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo test

# Run ignored tests
cargo test -- --ignored

# Run exact test name
cargo test -- --exact test_name

# List all tests
cargo test -- --list

# Run only integration tests
cargo test --test '*'

# Run only unit tests
cargo test --lib
```

## Test Output Guide

### Success
```
test test_name ... ok
test result: ok. 10 passed; 0 failed
```

### Failure
```
test test_name ... FAILED
panicked at 'assertion failed: left != right', file.rs:42:9
```

### Summary Line Format
```
test result: <status>. <passed> passed; <failed> failed; <ignored> ignored; <measured> measured; <filtered> filtered out
```

## File I/O Pattern

```rust
use tempfile::TempDir;

#[test]
fn test_example() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");

    std::fs::write(&file_path, "content").unwrap();

    // Test code here
    // temp automatically cleaned up when dropped
}
```

## Critical Gotchas

1. **Keep TempDir in scope**
```rust
// WRONG
let path = TempDir::new().unwrap().path();  // Deleted immediately

// RIGHT
let temp = TempDir::new().unwrap();
let path = temp.path();  // Valid while temp exists
```

2. **Use unique names for parallel tests**
```rust
let unique_name = format!("test_{}", std::process::id());
```

3. **Set JIN_DIR for isolation**
```rust
std::env::set_var("JIN_DIR", temp.path().join(".jin_global"));
```

## Jin-Specific Patterns

```rust
// Use shared fixtures
use crate::common::fixtures;

let fixture = fixtures::setup_test_repo().unwrap();

// Use custom assertions
use crate::common::assertions;

assertions::assert_workspace_file(fixture.path(), "file.txt", "content");

// Use git helpers
use crate::common::git_helpers;

git_helpers::create_commit(fixture.path(), "message");
```

## Debugging Tests

```bash
# Full debugging output
RUST_BACKTRACE=full cargo test test_name -- --nocapture --test-threads=1 --exact

# Keep test files for inspection
KEEP_TEST_FILES=1 cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Test Attributes

```rust
#[test]              // Basic test
#[ignore]            // Skip unless --ignored
#[should_panic]      // Expects panic
#[serial]            // Run sequentially
```

## URLs

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Cargo Test Command](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [assert_cmd docs](https://docs.rs/assert_cmd/latest/assert_cmd/)
- [tempfile docs](https://docs.rs/tempfile/latest/tempfile/)

## Jin Project Files

- `/home/dustin/projects/jin/tests/common/fixtures.rs` - Test fixtures
- `/home/dustin/projects/jin/tests/common/assertions.rs` - Custom assertions
- `/home/dustin/projects/jin/tests/common/git_helpers.rs` - Git utilities
- `/home/dustin/projects/jin/plan/docs/cargo_test_execution_guide.md` - Full guide
