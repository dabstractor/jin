# Testing CLI Applications with assert_cmd and predicates

## Overview

`assert_cmd` simplifies integration testing of command-line applications by providing an ergonomic way to:
- Find and configure your crate's binary
- Execute the binary with various arguments
- Assert on exit codes, stdout, and stderr

`predicates` provides powerful matching capabilities for output validation with comprehensive error messages.

## Crates

| Crate | Purpose | URL |
|-------|---------|-----|
| `assert_cmd` | CLI binary testing | [docs.rs/assert_cmd](https://docs.rs/assert_cmd/latest/assert_cmd/) |
| `predicates` | Predicate-based assertions | [crates.io](https://crates.io/crates/predicates) |
| `assert_fs` | Filesystem fixtures | [docs.rs/assert_fs](https://docs.rs/assert_fs) |
| `escargot` | Advanced binary configuration | [crates.io](https://crates.io/crates/escargot) |

## Installation

Add to `Cargo.toml`:

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.0"
tempfile = "3.0"
```

## Basic Usage Pattern

### Simple Success Test

```rust
use assert_cmd::Command;

#[test]
fn command_succeeds() {
    let mut cmd = Command::cargo_bin("my_app").unwrap();
    cmd.assert().success();
}
```

### Testing with Arguments

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("grrs").unwrap();
    cmd.arg("foobar").arg("test/file/doesnt/exist");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("could not read file"));

    Ok(())
}
```

### Capturing and Asserting Output

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_output() {
    Command::cargo_bin("my_app")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0"));
}
```

## Command Configuration

### Basic Configuration Methods

```rust
use assert_cmd::Command;

let mut cmd = Command::cargo_bin("my_app").unwrap();

// Add arguments
cmd.arg("argument1");
cmd.args(&["arg1", "arg2"]);

// Set working directory
cmd.current_dir("/tmp");

// Set environment variables
cmd.env("MY_VAR", "value");
cmd.env_remove("UNWANTED");
cmd.env_clear();

// Provide stdin
cmd.write_stdin("input data");

// Set timeout (in seconds)
// Note: This depends on implementation details
```

## Assertion Methods

### Exit Code Assertions

```rust
cmd.assert().success();                    // Exit code 0
cmd.assert().failure();                    // Non-zero exit code
cmd.assert().code(1);                      // Specific exit code
cmd.assert().interrupted();                // Killed by signal
```

### Output Assertions

```rust
use predicates::prelude::*;

// Stdout assertions
cmd.assert()
    .success()
    .stdout(predicate::str::contains("expected text"));

// Stderr assertions
cmd.assert()
    .failure()
    .stderr(predicate::str::contains("error message"));

// Exact match
cmd.assert()
    .success()
    .stdout("exact output\n");
```

### Output Result Methods

```rust
// Get the assertion result for custom handling
let result = cmd.assert();

// Access individual outputs
let stdout = String::from_utf8(result.get_output().stdout.clone()).unwrap();
let stderr = String::from_utf8(result.get_output().stderr.clone()).unwrap();
```

## Predicate Patterns

### String Predicates

```rust
use predicates::prelude::*;

// Contains substring
predicate::str::contains("substring")

// Starts with
predicate::str::contains("Start").from_utf8().unwrap()

// Regular expression matching
predicate::str::is_match("^pattern.*end$").unwrap()

// Exact equality
predicate::eq("exact string")
```

### Path Predicates

```rust
use predicates::prelude::*;

// File exists
predicate::path::exists()

// File missing
predicate::path::missing()

// Directory exists
predicate::path::is_dir()

// File exists (not directory)
predicate::path::is_file()
```

### Combining Predicates

```rust
use predicates::prelude::*;

// AND combination
let pred = predicate::str::contains("foo").and(predicate::str::contains("bar"));

// OR combination
let pred = predicate::str::contains("foo").or(predicate::str::contains("bar"));

// NOT combination
let pred = predicate::str::contains("foo").not();
```

## Complete Example: File Processing CLI

```rust
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::error::Error;

#[test]
fn find_content_in_file() -> Result<(), Box<dyn Error>> {
    // Setup: Create a temporary file with test content
    let file = assert_fs::NamedTempFile::new("sample.txt")?;
    file.write_str("A test\nActual content\nMore content\nAnother test")?;

    // Execute: Run the CLI with the test file
    let mut cmd = Command::cargo_bin("grrs")?;
    cmd.arg("test").arg(file.path());

    // Assert: Check output contains expected lines
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A test"))
        .stdout(predicate::str::contains("Another test"));

    Ok(())
}

#[test]
fn nonexistent_file() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("grrs")?;
    cmd.arg("pattern").arg("nonexistent.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("could not read file"));

    Ok(())
}

#[test]
fn empty_pattern() -> Result<(), Box<dyn Error>> {
    let file = assert_fs::NamedTempFile::new("test.txt")?;
    file.write_str("test content")?;

    let mut cmd = Command::cargo_bin("grrs")?;
    cmd.arg("").arg(file.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("pattern cannot be empty"));

    Ok(())
}
```

## Testing Multi-Step Workflows

### Sequential Command Execution

```rust
use assert_cmd::Command;
use assert_fs::TempDir;

#[test]
fn multi_step_workflow() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Step 1: Initialize
    Command::cargo_bin("my_tool")?
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Step 2: Add content
    Command::cargo_bin("my_tool")?
        .arg("add")
        .arg("file.txt")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Step 3: Verify state
    Command::cargo_bin("my_tool")?
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("file.txt"));

    temp_dir.close()?;
    Ok(())
}
```

## Common Patterns

### Environment Variable Testing

```rust
#[test]
fn respects_env_var() {
    Command::cargo_bin("my_app").unwrap()
        .env("MY_VAR", "custom_value")
        .arg("--config")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom_value"));
}
```

### Exit Code Verification

```rust
#[test]
fn error_exit_codes() {
    // Missing required argument
    Command::cargo_bin("my_app").unwrap()
        .assert()
        .failure()
        .code(1);

    // Invalid option
    Command::cargo_bin("my_app").unwrap()
        .arg("--invalid")
        .assert()
        .failure()
        .code(2);
}
```

### Stdin Input

```rust
#[test]
fn echo_test() {
    Command::cargo_bin("my_app").unwrap()
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout("hello world");
}
```

## Best Practices

### 1. **Test User-Visible Behavior**
- Focus on what users observe: output, exit codes, file creation
- Avoid testing implementation details

### 2. **Use Descriptive Test Names**
```rust
#[test]
fn given_valid_file_when_searching_then_returns_matching_lines() {
    // Test implementation
}
```

### 3. **Keep Tests Independent**
- Each test should be runnable in any order
- Clean up resources (assert_fs handles this automatically)
- Don't depend on output from previous tests

### 4. **Use Predicates for Flexibility**
- Use `predicate::str::contains()` for partial matches
- Use regex predicates when exact matching is too strict
- Avoid brittle exact string assertions

### 5. **Pair with assert_fs**
- Combine assert_cmd with assert_fs for complete workflow testing
- Use temporary directories to avoid test pollution
- Assert file creation and content

## Companion Crates

| Crate | Purpose |
|-------|---------|
| `assert_fs` | Filesystem fixtures and assertions |
| `dir-diff` | Comparing directory structures |
| `escargot` | Advanced binary configuration |
| `duct` | Running multiple processes |
| `tempfile` | Low-level temporary file management |

## References

- [assert_cmd - Rust](https://docs.rs/assert_cmd/latest/assert_cmd/)
- [assert_cmd - crates.io](https://crates.io/crates/assert_cmd)
- [GitHub: assert-rs/assert_cmd](https://github.com/assert-rs/assert_cmd)
- [Testing - Command Line Applications in Rust](https://rust-cli.github.io/book/tutorial/testing.html)
- [Testing Rust command-line apps - Alex Chan](https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert_cmd/)
- [Integration testing Cargo binaries with assert_cmd](https://egghead.io/lessons/rust-integration-testing-cargo-binaries-with-assert_cmd)
