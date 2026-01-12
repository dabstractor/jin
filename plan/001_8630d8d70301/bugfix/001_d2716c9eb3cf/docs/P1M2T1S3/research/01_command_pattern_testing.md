# Command Pattern for Testing CLI Apps in Rust

## Overview
The command pattern is the standard approach for testing Rust CLI applications, allowing you to spawn subprocesses, capture output, and assert on behavior.

## Key Resources

### Official Documentation
- **assert_cmd crate**: https://docs.rs/assert_cmd/latest/assert_cmd/
  - Primary crate for CLI testing in Rust ecosystem
  - Provides `Command` type for spawning and testing CLIs
  - Part of the assert_cmd ecosystem maintained by Rust CLI working group

### Related Crates
- **predicates**: https://docs.rs/predicates/latest/predicates/
  - Used for boolean-valued assertions on command output
  - Provides composable predicates for stdout/stderr validation

## Command Pattern Implementation

### Basic Pattern (Used in jin Project)

```rust
use assert_cmd::Command;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_help() {
    jin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Phantom Git layer system"));
}
```

### Key Components

1. **Command Creation**
   ```rust
   Command::new(env!("CARGO_BIN_EXE_jin"))  // Recommended: uses compiled binary
   Command::cargo_bin("jin")                 // Alternative: finds binary in target dir
   ```

2. **Chaining Arguments**
   ```rust
   jin()
       .args(["add", "config.json", "--mode"])  // Multiple args
       .arg("--verbose")                         // Single arg
   ```

3. **Setting Working Directory and Environment**
   ```rust
   jin()
       .current_dir(temp.path())              // Set working directory
       .env("JIN_DIR", temp.path().join(".jin"))  // Set environment variable
   ```

4. **Assertions**
   ```rust
   jin()
       .arg("status")
       .assert()
       .success()                              // Exit code 0
       .failure()                              // Non-zero exit code
       .stdout(predicate::str::contains("text"))
       .stderr(predicate::str::contains("error"))
   ```

## Best Practices

### 1. Isolation Pattern
```rust
#[test]
fn test_with_isolation() {
    let temp = tempfile::TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}
```

### 2. Unique Identifiers for Parallel Tests
```rust
#[test]
fn test_mode_creation() {
    let unique_name = format!("test_mode_{}", std::process::id());
    jin()
        .args(["mode", "create", &unique_name])
        .assert()
        .success();
}
```

### 3. Serial Tests for Shared Resources
```rust
#[test]
#[serial]
fn test_shared_global_state() {
    // Tests that modify global state
}
```

## Advanced Patterns

### Testing Exit Codes
```rust
jin()
    .arg("invalid")
    .assert()
    .failure()
    .code(1);  // Specific exit code
```

### Testing Stdin
```rust
jin()
    .write_stdin("input data")
    .arg("process")
    .assert()
    .success();
```

### Getting Output for Further Inspection
```rust
let output = jin()
    .arg("status")
    .assert()
    .get_output();

let stdout_str = String::from_utf8_lossy(&output.stdout);
let stderr_str = String::from_utf8_lossy(&output.stderr);
```

## Testing Multi-Command Workflows

The jin project demonstrates sophisticated CLI testing with complex workflows:

```rust
#[test]
fn test_full_workflow() {
    let fixture = setup_test_repo().unwrap();

    // Step 1: Initialize
    jin().arg("init")
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    // Step 2: Add file
    jin().args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    // Step 3: Commit
    jin().args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();

    // Step 4: Apply
    jin().arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success();
}
```

## Common Pitfalls

1. **Not using `env!("CARGO_BIN_EXE_jin")`**: May fail in some test scenarios
2. **Forgetting `current_dir()`**: Tests run in crate root by default
3. **Not isolating environment variables**: Tests may interfere with each other
4. **Not cleaning up temp directories**: Can cause test pollution

## References

- assert_cmd repository: https://github.com/assert-rs/assert_cmd
- Rust CLI working group: https://cli.rs/
- Testing chapter in Rust Book: https://doc.rust-lang.org/book/ch11-00-testing.html
