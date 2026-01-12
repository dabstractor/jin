# Rust CLI Testing Research for --local Flag Tests

## Core Testing Crates & Documentation

### assert_cmd - Primary CLI Testing Crate
- **Documentation**: https://docs.rs/assert_cmd/latest/assert_cmd/
- **Key methods**:
  - `Command::new(env!("CARGO_BIN_EXE_jin"))` - Get command for binary
  - `.arg()` / `.args()` - Add arguments
  - `.current_dir()` - Set working directory
  - `.env()` - Set environment variables
  - `.assert()` - Execute and get Assert
  - `.success()` / `.failure()` - Check exit code
  - `.stdout()` / `.stderr()` - Assert on output

### predicates - Output Assertion Library
- **Documentation**: https://docs.rs/predicates/latest/predicates/
- **Key predicates**:
  - `predicate::str::contains("text")` - Substring matching
  - `predicate::str::is_match(regex)` - Regex matching
  - `predicate::eq("text")` - Exact equality
  - `.or()` / `.and()` - Compose predicates

### tempfile - Temporary Directory Management
- **Documentation**: https://docs.rs/tempfile/latest/tempfile/
- **Key types**:
  - `TempDir::new()` - Create temporary directory
  - `NamedTempFile::new()` - Create temporary file
  - Automatic cleanup on drop

## Test Patterns from Codebase

### Basic Test Structure
```rust
use assert_cmd::Command;
use predicates::prelude::*;

fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_example() {
    jin()
        .arg("add")
        .arg("file.txt")
        .arg("--local")
        .assert()
        .success()
        .stdout(predicate::str::contains("success message"));
}
```

### Test with Isolated Environment
```rust
use tempfile::TempDir;

#[test]
fn test_with_isolation() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}
```

### Using Common Fixtures
```rust
mod common;
use common::fixtures::*;

#[test]
fn test_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();

    // Initialize Jin
    jin_init(fixture.path(), fixture.jin_dir.as_ref())?;

    // Run test commands
    jin()
        .args(["add", "config.json", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    Ok(())
}
```

## Testing Conflicting Flags Pattern
```rust
#[test]
fn test_conflicting_flags() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .args(["add", "file.txt", "--local", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot combine --local with other layer flags"));
}
```

## Multi-Step Workflow Test Pattern
```rust
#[test]
fn test_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Create test file
    let test_file = fixture.path().join("config.json");
    fs::write(&test_file, "{\"key\": \"value\"}")?;

    // Add with --local
    jin()
        .args(["add", "config.json", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    // Verify staged
    assert_staging_contains(fixture.path(), "config.json");

    // Commit
    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    Ok(())
}
```

## External Resources

### Rust CLI Book - Testing Chapter
- **URL**: https://rust-cli.github.io/book/tutorial/testing.html
- **Key sections**:
  - "Making code testable" - Extracting logic for testability
  - "Testing CLI applications by running them" - Integration test setup
  - "Generating test files" - Using assert_fs for temporary file testing

### Blog: Testing Rust CLI Apps with assert_cmd
- **URL**: https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert-cmd/
- **Key sections**:
  - "Testing error cases" - Exit codes and stderr validation
  - "Comparing output to a regular expression" - Using predicates
  - "Creating focused helper functions" - Test organization

### Integration Testing Rust Binaries
- **URL**: https://www.unwoundstack.com/blog/integration-testing-rust-binaries.html
- **Key sections**:
  - "The Solution" - Custom test harness patterns
  - "Code Arrangement" - Workspace organization

## Best Practices

1. **Always use isolated environments** - Set JIN_DIR to test-specific path
2. **Use TestFixture from tests/common/fixtures.rs** - Handles cleanup automatically
3. **Test both success and failure cases** - Verify error messages are correct
4. **Use predicates for output assertions** - More readable than string comparison
5. **Test flag conflicts** - Ensure validation logic works
6. **Use Result return type** - Allows use of ? operator for cleaner test code

## Gotchas

1. **TempDir must be stored** - If TempDir is dropped, directory is deleted
2. **JIN_DIR must be set before operations** - Set environment variable before jin commands
3. **Git locks need cleanup** - TestFixture handles this automatically on drop
4. **File paths must be relative** - When passing to jin commands, use relative paths
5. **std::process::id() is not unique enough** - Use unique_test_id() from fixtures for truly unique IDs
