# Rust Testing Best Practices for Integration Tests

## Overview
This document outlines best practices for Rust integration testing, focusing on file system operations, JSON testing, and CLI command testing patterns.

---

## 1. Rust Integration Test Patterns

### Test Directory Structure
Integration tests in Rust should be placed in the `tests/` directory at the crate root. Each file in this directory is compiled as a separate test binary.

**Key Points:**
- Integration tests access the crate's public API
- They have full access to the crate's modules
- Multiple test files can be organized by feature or component

**File Structure:**
```
my_project/
├── src/
│   └── lib.rs
├── tests/
│   ├── integration_tests.rs
│   ├── file_system_tests.rs
│   ├── cli_tests.rs
│   └── main.rs (optional, for shared setup)
```

### tempfile Crate Usage
The `tempfile` crate is essential for creating temporary files and directories in tests.

**Core Components:**
- [`NamedTempFile`](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html) - Temporary files with random names
- [`TempDir`](https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html) - Temporary directories
- Both implement `Drop` for automatic cleanup

**Basic Usage Pattern:**
```rust
use tempfile::{NamedTempFile, TempDir};
use std::fs;
use std::io::Write;

#[test]
fn test_with_temp_file() {
    // Create temporary file (automatically cleaned up)
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    // Write test data
    let content = "Hello, World!";
    fs::write(file_path, content).unwrap();

    // Verify content
    let read_content = fs::read_to_string(file_path).unwrap();
    assert_eq!(read_content, content);

    // File is automatically deleted when temp_file goes out of scope
}

#[test]
fn test_with_temp_dir() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    // Create subdirectory and files
    let sub_dir = dir_path.join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let test_file = sub_dir.join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Verify file exists
    assert!(test_file.exists());
}
```

**Advanced tempfile Patterns:**
```rust
use tempfile::{TempDir, Builder};
use std::path::Path;

#[test]
fn test_custom_temp_file() {
    // Create temp file with specific prefix
    let temp_file = Builder::new()
        .prefix("test_")
        .suffix(".txt")
        .tempfile()
        .unwrap();

    // Temp file can be kept for manual inspection if needed
    let path = temp_file.into_temp_path();
    // ... test logic ...
    // path persists after scope end
}

#[test]
fn test_nested_temp_structure() {
    let base_dir = TempDir::new().unwrap();
    let base_path = base_dir.path();

    // Create complex directory structure
    let nested = base_path.join("nested").join("structure");
    fs::create_dir_all(&nested).unwrap();

    // Create test files
    let config = nested.join("config.json");
    let data = nested.join("data.csv");

    fs::write(&config, r#"{"key": "value"}"#).unwrap();
    fs::write(&data, "1,2,3\n4,5,6").unwrap();

    // ... perform test operations ...
}
```

### Testing File System Operations

**Common Patterns:**
```rust
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_file_creation_and_deletion() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Test file creation
    assert!(!test_file.exists());
    fs::write(&test_file, "content").unwrap();
    assert!(test_file.exists());

    // Test file deletion
    fs::remove_file(&test_file).unwrap();
    assert!(!test_file.exists());
}

#[test]
fn test_directory_operations() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");

    // Create directory
    fs::create_dir(&test_dir).unwrap();
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());

    // Create file in directory
    let file = test_dir.join("file.txt");
    fs::write(&file, "content").unwrap();

    // List directory contents
    let entries: Vec<_> = fs::read_dir(&test_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0], file);
}
```

---

## 2. serde_json Testing Patterns

### Basic JSON Parsing and Validation
```rust
use serde_json::{json, Value};
use std::fs;

#[test]
fn test_json_parsing() {
    let json_content = r#"{
        "name": "test",
        "version": "1.0.0",
        "features": ["feature1", "feature2"],
        "config": {
            "debug": true,
            "port": 8080
        }
    }"#;

    let parsed: Value = serde_json::from_str(json_content).unwrap();

    // Basic assertions
    assert_eq!(parsed["name"], "test");
    assert_eq!(parsed["version"], "1.0.0");
    assert!(parsed["config"]["debug"].as_bool().unwrap());
}

#[test]
fn test_json_file_operations() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let file_path = temp_file.path();

    // Write JSON to file
    let test_json = json!({
        "key": "value",
        "number": 42,
        "array": [1, 2, 3]
    });

    let json_string = serde_json::to_string_pretty(&test_json).unwrap();
    fs::write(file_path, &json_string).unwrap();

    // Read and parse back from file
    let read_content = fs::read_to_string(file_path).unwrap();
    let parsed_json: Value = serde_json::from_str(&read_content).unwrap();

    // Verify structure
    assert_eq!(parsed_json["key"], "value");
    assert_eq!(parsed_json["number"], 42);
    assert_eq!(parsed_json["array"][0], 1);
}
```

### Structured JSON Testing
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    name: String,
    version: String,
    debug: bool,
    port: u16,
}

#[test]
fn test_struct_json_roundtrip() {
    let original = Config {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        debug: true,
        port: 8080,
    };

    // Serialize to JSON
    let json_string = serde_json::to_string(&original).unwrap();

    // Parse back from JSON
    let parsed: Config = serde_json::from_str(&json_string).unwrap();

    // Verify roundtrip
    assert_eq!(original, parsed);
}

#[test]
fn test_json_validation() {
    let test_cases = vec![
        (json!({"valid": true}), true),
        (json!({"valid": false}), false),
    ];

    for (json_value, expected) in test_cases {
        let is_valid = json_value["valid"].as_bool().unwrap();
        assert_eq!(is_valid, expected);
    }
}
```

### Deep JSON Comparison Patterns
```rust
use serde_json::json;

#[test]
fn test_deep_json_comparison() {
    let expected = json!({
        "users": [
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true
        }
    });

    let actual = json!({
        "users": [
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true
        }
    });

    // Helper function for deep comparison
    fn deep_equal_json(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len() &&
                a.iter().all(|(k, v)| b.contains_key(k) && deep_equal_json(v, &b[k]))
            }
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() &&
                a.iter().zip(b.iter()).all(|(x, y)| deep_equal_json(x, y))
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    assert!(deep_equal_json(&expected, &actual));
}
```

### JSON Error Handling Tests
```rust
#[test]
fn test_invalid_json_error_handling() {
    let invalid_json = r#"{"incomplete": json"#;

    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);

    assert!(result.is_err());

    // Match specific error types
    match result {
        Err(serde_json::Error { .. }) => {
            // Expected error
        }
        _ => panic!("Expected JSON parsing error"),
    }
}
```

---

## 3. CLI Command Testing Patterns

### Using std::process::Command
```rust
use std::process::Command;
use tempfile::TempDir;
use std::path::Path;

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "my_cli", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--help"));
    assert!(stdout.contains("Usage"));
}

#[test]
fn test_cli_with_file_input() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("input.txt");
    let output_file = temp_dir.path().join("output.txt");

    // Create input file
    std::fs::write(&test_file, "test content").unwrap();

    // Run CLI command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "my_cli", "--",
               "--input", test_file.to_str().unwrap(),
               "--output", output_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(output_file.exists());

    // Verify output
    let output_content = std::fs::read_to_string(&output_file).unwrap();
    assert_eq!(output_content, "processed: test content");
}
```

### Advanced CLI Testing Patterns
```rust
#[test]
fn test_cli_error_cases() {
    // Test missing required argument
    let output = Command::new("cargo")
        .args(&["run", "--bin", "my_cli", "--", "process"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing required argument"));

    // Test invalid file path
    let output = Command::new("cargo")
        .args(&["run", "--bin", "my_cli", "--",
               "--input", "/nonexistent/file.txt"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert!(!output.status.success());
}

#[test]
fn test_cli_with_environment_variables() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("config.json");

    std::fs::write(&test_file, r#{"debug": true}"#).unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--bin", "my_cli", "--",
               "--config", test_file.to_str().unwrap()])
        .env("LOG_LEVEL", "debug")
        .env("RUST_BACKTRACE", "1")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify environment was used
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("debug mode"));
}
```

### Interactive CLI Testing (for applications that require input)
```rust
use std::io::{self, Write};
use std::process::{Command, Stdio};

#[test]
fn test_cli_with_user_input() {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "interactive_cli"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    if let Some(stdin) = child.stdin.as_mut() {
        // Send user input
        writeln!(stdin, "option1").expect("Failed to write to stdin");
        writeln!(stdin, "user@example.com").expect("Failed to write to stdin");
        writeln!(stdin, "password123").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration saved successfully"));
}
```

### Integration Test with File System and CLI
```rust
#[test]
fn test_complete_cli_workflow() {
    // Setup test environment
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    let config_file = temp_dir.path().join("config.json");

    std::fs::create_dir_all(&input_dir).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();

    // Create input files
    std::fs::write(input_dir.join("file1.txt"), "content1").unwrap();
    std::fs::write(input_dir.join("file2.txt"), "content2").unwrap();

    // Create config
    let config = serde_json::json!({
        "input_dir": input_dir,
        "output_dir": output_dir,
        "process_mode": "batch"
    });
    std::fs::write(&config_file, config.to_string()).unwrap();

    // Run CLI
    let output = Command::new("cargo")
        .args(&["run", "--bin", "my_app", "--",
               "--config", config_file.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Verify results
    let result_file = output_dir.join("result.json");
    assert!(result_file.exists());

    let result_content = std::fs::read_to_string(&result_file).unwrap();
    let result: serde_json::Value = serde_json::from_str(&result_content).unwrap();

    assert_eq!(result["processed_files"], 2);
    assert!(result["success"].as_bool().unwrap());
}
```

---

## 4. Additional Testing Resources

### Useful Testing Crates
- [`pretty_assertions`](https://docs.rs/pretty_assertions/) - Better assertion error messages
- [`predicates`](https://docs.rs/predicates/) - Composable predicate testing
- [`mockall`](https://docs.rs/mockall/) - Mocking framework for Rust
- [`tokio-test`](https://docs.rs/tokio-test/) - Testing utilities for async code
- [`insta`](https://docs.rs/insta/) - Snapshot testing for Rust

### Best Practices Summary
1. **Always clean up temporary files** - Use `tempfile` crate
2. **Use appropriate assertion methods** - `assert_eq!`, `assert!`, `assert_ne!`
3. **Test both success and error cases** - Ensure robust error handling
4. **Keep tests isolated** - Each test should be independent
5. **Use descriptive test names** - Clearly communicate what each test validates
6. **Follow the AAA pattern** - Arrange, Act, Assert
7. **Mock external dependencies** - Use mocking frameworks when needed
8. **Integrate with CI/CD** - Ensure tests run in continuous integration

### Example Integration Test Structure
```rust
// tests/integration_tests.rs
use std::process::Command;
use tempfile::TempDir;
use serde_json::json;

mod common; // Common test utilities

#[test]
fn test_feature_integration() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    common::setup_test_environment(&temp_dir);

    // Act
    let output = Command::new("cargo")
        .args(&["run", "--", "--input", temp_dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    // Assert
    assert!(output.status.success());
    let result = common::parse_output(&output);
    assert_eq!(result["status"], "success");
}
```

This document provides a comprehensive overview of Rust testing best practices for integration tests, with practical examples for file system operations, JSON testing, and CLI command testing.