# CLI Test Patterns in Jin Codebase

## Overview

This document analyzes the existing CLI test patterns in the Jin codebase, specifically for testing argument parsing and flags. The tests use `assert_cmd` and `predicates` crates for integration testing.

## Key Testing Patterns

### 1. Basic Test Structure

#### Command Setup
```rust
use assert_cmd::Command;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}
```

#### Environment Isolation
```rust
use tempfile::TempDir;

#[test]
fn test_example() {
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Set JIN_DIR to isolated directory
    std::env::set_var("JIN_DIR", &jin_dir);

    // Run tests in isolated environment
    jin()
        .arg("command")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

### 2. Flag Testing Patterns

#### Boolean Flags ( Presence/Absence )
```rust
// Test that --force flag is recognized
#[test]
fn test_link_force_flag() {
    let result = jin()
        .args(["link", "https://github.com/example/repo.git", "--force"])
        .assert();

    // Should not complain about unknown flag
    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr_str.contains("unexpected argument '--force'"),
        "Force flag should be recognized"
    );
}
```

#### Value Flags ( Key-Value Pairs )
```rust
// Test with --mode flag
jin()
    .args(["add", "config.json", "--mode"])
    .current_dir(temp.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// Test with --scope=value format
jin()
    .args(["add", "config.json", "--scope=language:javascript"])
    .current_dir(temp.path())
    .assert()
    .failure();
```

#### Multiple Flags
```rust
// Test multiple flags: --hard and --force
jin()
    .args(["reset", "--hard", "--force"])
    .current_dir(project_path)
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success()
    .stdout(predicate::str::contains("Discarded"));
```

### 3. Specific Flag Tests Found

#### --mode Flag Tests
```rust
// From cli_basic.rs
#[test]
fn test_add_with_mode_flag() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("config.json");
    std::fs::write(&test_file, "{}").unwrap();

    jin()
        .current_dir(temp.path())
        .args(["add", "config.json", "--mode"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

// From cli_reset.rs
#[test]
fn test_reset_layer_targeting_mode() {
    // ... setup ...
    jin()
        .args(["reset", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));
}
```

#### --global Flag Tests
```rust
// From cli_reset.rs
#[test]
fn test_reset_global_layer() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // ... setup ...
    // Add file to global layer
    jin()
        .args(["add", "global.json", "--global"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset global layer
    jin()
        .args(["reset", "--global"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));
}
```

#### --project Flag Tests
```rust
// From cli_reset.rs
#[test]
fn test_reset_invalid_layer_combination() {
    // ... setup ...
    // Try to reset --project without --mode
    jin()
        .args(["reset", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--project requires --mode"));
}
```

#### --staged Flag Tests
```rust
// From cli_diff.rs
#[test]
fn test_diff_staged_empty() {
    // ... setup ...
    jin()
        .arg("diff")
        .arg("--staged")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("No staged changes"));
}

#[test]
fn test_diff_staged_with_files() {
    // ... setup ...
    // Stage the file to mode-base
    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Check staged diff
    jin()
        .arg("diff")
        .arg("--staged")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(contains("config.json"))
        .stdout(contains("mode-base"));
}
```

### 4. Flag Combination Testing

#### Mutually Exclusive Flags
```rust
// From cli_reset.rs
#[test]
fn test_reset_invalid_layer_combination() {
    // ... setup ...
    // Try to reset --project without --mode
    jin()
        .args(["reset", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--project requires --mode"));
}
```

#### Dependent Flags
```rust
// Multiple flags that work together
jin()
    .args(["link", "https://github.com/example/repo.git", "--force"])
    .assert()
    .success();
```

### 5. Error Handling for Flags

#### Invalid Flag Values
```rust
// From cli_completion.rs
#[test]
fn test_completion_invalid_shell() {
    jin()
        .args(["completion", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("possible values"));
}
```

#### Missing Required Flag Arguments
```rust
#[test]
fn test_completion_no_shell() {
    jin()
        .args(["completion"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("<SHELL>"));
}
```

### 6. Best Practices Observed

1. **Environment Isolation**: Each test creates a temporary directory and sets `JIN_DIR` to ensure test isolation.

2. **Error Message Testing**: Tests don't just check for failure/success, but also verify specific error messages.

3. **Flag Recognition Tests**: Some tests verify that flags are properly recognized by the CLI (not just their functionality).

4. **Combinatorial Testing**: Tests verify flag combinations and interactions.

5. **Help Flag Testing**: Many tests include `--help` flag verification to document available options.

6. **Output Pattern Matching**: Tests use `predicate::str::contains()` to verify expected output patterns.

### 7. Common Test Patterns

#### Help Flag Testing Pattern
```rust
#[test]
fn test_command_help() {
    jin()
        .args(["command", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("command description"))
        .stdout(predicate::str::contains("--flag"));
}
```

#### Error Pattern Testing
```rust
#[test]
fn test_command_error() {
    jin()
        .arg("command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("expected error message"));
}
```

#### Success Pattern Testing
```rust
#[test]
fn test_command_success() {
    jin()
        .args(["command", "--flag", "value"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("expected success message"));
}
```

## Testing Strategy Recommendations

1. **Test Flag Parsing**: Verify flags are recognized before testing functionality
2. **Test Error Cases**: Include tests for invalid flag combinations and values
3. **Test Help Output**: Verify --help shows all flags and their descriptions
4. **Isolate Tests**: Use temporary directories and environment variable isolation
5. **Check Output**: Use predicates to verify specific output patterns
6. **Test Dependencies**: Verify that required flags produce appropriate errors when missing