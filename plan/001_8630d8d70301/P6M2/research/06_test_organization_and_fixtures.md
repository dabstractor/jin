# Test Organization and Fixture Patterns

## Overview

Well-organized tests are easier to maintain, understand, and extend. Proper fixture patterns enable code reuse and consistent test setup without duplication.

## Directory Structure

### Recommended Layout

```
project_root/
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── lib.rs              # Main library
│   ├── bin/
│   │   └── cli_tool.rs     # Binary entry point
│   ├── commands/
│   │   ├── init.rs
│   │   ├── process.rs
│   │   └── mod.rs
│   └── utils/
│       ├── helpers.rs
│       └── mod.rs
└── tests/
    ├── fixtures/           # Test data
    │   ├── valid_input/
    │   │   ├── simple.txt
    │   │   └── complex.toml
    │   ├── invalid_input/
    │   │   └── malformed.txt
    │   └── expected_output/
    │       └── result.txt
    ├── common/
    │   ├── mod.rs          # Shared test utilities
    │   ├── fixtures.rs     # Fixture definitions
    │   └── helpers.rs      # Helper functions
    ├── e2e/
    │   ├── init_test.rs
    │   ├── process_test.rs
    │   └── workflow_test.rs
    └── integration_test.rs # Legacy single file
```

### File Organization Rules

**Cargo Compilation Rules:**
1. Each `.rs` file in `tests/` root is compiled as a separate crate
2. Files in `tests/*/` subdirectories are NOT compiled as separate crates
3. `tests/common/mod.rs` or `tests/common.rs` won't appear in test output

**Using Subdirectories to Share Code:**

```
tests/
├── common/
│   └── mod.rs              # Won't be treated as a test
├── test_init.rs            # Test crate
├── test_process.rs         # Test crate
└── test_workflow.rs        # Test crate
```

Use subdirectories (`tests/common/`) instead of files (`tests/common.rs`) to hide shared modules.

## Shared Test Modules

### Pattern 1: Common Helper Module

```rust
// tests/common/mod.rs
use std::path::PathBuf;
use tempfile::TempDir;

pub fn get_fixture_path(fixture_name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(fixture_name)
}

pub fn copy_fixture(fixture_name: &str, dest: &std::path::Path) -> std::io::Result<()> {
    let src = get_fixture_path(fixture_name);
    std::fs::copy(&src, dest)?;
    Ok(())
}

pub fn assert_file_contains(path: &std::path::Path, expected: &str) {
    let content = std::fs::read_to_string(path)
        .expect("Failed to read file");
    assert!(
        content.contains(expected),
        "File {} does not contain '{}'\nContent: {}",
        path.display(),
        expected,
        content
    );
}
```

**Using the helper:**

```rust
// tests/test_process.rs
mod common;

#[test]
fn test_processes_fixture() -> std::io::Result<()> {
    let fixture = common::get_fixture_path("input.txt");
    assert!(fixture.exists());
    Ok(())
}
```

### Pattern 2: Reusable Fixture Struct

```rust
// tests/common/fixtures.rs
use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestEnvironment {
    _tempdir: TempDir,
    root: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> std::io::Result<Self> {
        let tempdir = TempDir::new()?;
        let root = tempdir.path().to_path_buf();

        Ok(TestEnvironment {
            _tempdir: tempdir,
            root,
        })
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub fn create_file(&self, name: &str, content: &str) -> std::io::Result<PathBuf> {
        let path = self.root.join(name);
        std::fs::write(&path, content)?;
        Ok(path)
    }

    pub fn create_dir(&self, name: &str) -> std::io::Result<PathBuf> {
        let path = self.root.join(name);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn read_file(&self, name: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.root.join(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_creates_environment() -> std::io::Result<()> {
        let env = TestEnvironment::new()?;

        env.create_file("test.txt", "content")?;
        let content = env.read_file("test.txt")?;

        assert_eq!(content, "content");
        Ok(())
    }
}
```

### Pattern 3: Setup and Teardown with Drop

```rust
// tests/common/fixtures.rs
use git2::Repository;
use tempfile::TempDir;
use std::path::PathBuf;

pub struct GitTestRepository {
    _tempdir: TempDir,
    repo: Repository,
}

impl GitTestRepository {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let repo = Repository::init(tempdir.path())?;

        println!("Created test repo at: {}", tempdir.path().display());

        Ok(GitTestRepository {
            _tempdir: tempdir,
            repo,
        })
    }

    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    pub fn path(&self) -> &std::path::Path {
        self._tempdir.path()
    }
}

impl Drop for GitTestRepository {
    fn drop(&mut self) {
        println!("Cleaning up test repo: {}", self._tempdir.path().display());
        // Automatic cleanup via tempdir drop
    }
}

#[test]
fn git_repo_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    let repo = GitTestRepository::new()?;
    let path = repo.path().to_path_buf();

    assert!(path.exists());
    // Cleanup happens here when repo is dropped

    Ok(())
}
```

### Pattern 4: Builder Pattern for Fixtures

```rust
// tests/common/fixtures.rs
use assert_fs::TempDir;
use std::path::PathBuf;

pub struct TestEnvironmentBuilder {
    files: Vec<(String, String)>,
    directories: Vec<String>,
}

impl TestEnvironmentBuilder {
    pub fn new() -> Self {
        TestEnvironmentBuilder {
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    pub fn with_file(mut self, name: &str, content: &str) -> Self {
        self.files.push((name.to_string(), content.to_string()));
        self
    }

    pub fn with_directory(mut self, name: &str) -> Self {
        self.directories.push(name.to_string());
        self
    }

    pub fn build(self) -> std::io::Result<TestEnvironment> {
        let temp = TempDir::new()?;

        // Create directories first
        for dir in self.directories {
            std::fs::create_dir_all(temp.path().join(&dir))?;
        }

        // Then create files
        for (name, content) in self.files {
            std::fs::write(temp.path().join(&name), content)?;
        }

        Ok(TestEnvironment {
            _temp: temp,
        })
    }
}

pub struct TestEnvironment {
    _temp: TempDir,
}

impl TestEnvironment {
    pub fn path(&self) -> &std::path::Path {
        self._temp.path()
    }
}

#[test]
fn builder_pattern_fixture() -> std::io::Result<()> {
    let env = TestEnvironmentBuilder::new()
        .with_directory("input")
        .with_directory("output")
        .with_file("input/test.txt", "test data")
        .with_file("config.toml", "[settings]")
        .build()?;

    assert!(env.path().join("input/test.txt").exists());
    assert!(env.path().join("config.toml").exists());

    Ok(())
}
```

## Fixture-Based Testing with rstest

### Installation

```toml
[dev-dependencies]
rstest = "0.21"
```

### Basic Fixture Pattern

```rust
use rstest::*;

#[fixture]
fn test_file() -> String {
    "test content".to_string()
}

#[rstest]
fn test_with_fixture(test_file: String) {
    assert_eq!(test_file, "test content");
}
```

### Parameterized Fixtures

```rust
use rstest::*;

#[fixture]
fn sample_data(#[default("Alice")] name: &str, #[default(25)] age: u8) -> (String, u8) {
    (name.to_string(), age)
}

#[rstest]
fn test_default(sample_data: (String, u8)) {
    assert_eq!(sample_data.0, "Alice");
    assert_eq!(sample_data.1, 25);
}

#[rstest]
fn test_custom(#[with("Bob", 30)] sample_data: (String, u8)) {
    assert_eq!(sample_data.0, "Bob");
    assert_eq!(sample_data.1, 30);
}
```

### Fixture Composition

```rust
use rstest::*;

#[fixture]
fn base_data() -> Vec<i32> {
    vec![1, 2, 3]
}

#[fixture]
fn extended_data(base_data: Vec<i32>) -> Vec<i32> {
    let mut data = base_data;
    data.push(4);
    data
}

#[rstest]
fn test_extended(extended_data: Vec<i32>) {
    assert_eq!(extended_data, vec![1, 2, 3, 4]);
}
```

### Table-Based Testing

```rust
use rstest::*;

#[rstest]
#[case(2, 2, 4)]
#[case(2, 3, 8)]
#[case(3, 2, 9)]
fn test_power(#[case] base: i32, #[case] exp: i32, #[case] expected: i32) {
    assert_eq!(base.pow(exp as u32), expected);
}
```

### Matrix Testing

```rust
use rstest::*;

#[rstest]
fn test_combinations(
    #[values(1, 2)] a: i32,
    #[values(10, 20)] b: i32,
) {
    // This generates 4 tests:
    // (1, 10), (1, 20), (2, 10), (2, 20)
    assert!(a * b > 0);
}
```

## Unit Tests in Library Code

### Convention: #[cfg(test)] Module

```rust
// src/lib.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    // Can test private functions
    fn internal_helper(x: i32) -> i32 {
        x * 2
    }

    #[test]
    fn test_internal() {
        assert_eq!(internal_helper(5), 10);
    }
}
```

### Testing Private Functions

```rust
// src/lib.rs
pub fn process(input: &str) -> String {
    filter_invalid_chars(input)
        .to_uppercase()
}

fn filter_invalid_chars(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_filter() {
        let result = filter_invalid_chars("Hello-World!");
        assert_eq!(result, "HelloWorld");
    }

    #[test]
    fn test_public_process() {
        let result = process("hello-world!");
        assert_eq!(result, "HELLOWORLD");
    }
}
```

## Integration Test Structure

### Single File Organization

```rust
// tests/integration_test.rs
use my_crate::*;
use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

mod common;

#[test]
fn test_basic_functionality() {
    let result = my_function("input");
    assert_eq!(result, "expected");
}

#[test]
fn test_cli_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    Command::cargo_bin("my_app")?
        .arg("--version")
        .assert()
        .success();

    temp.close()?;
    Ok(())
}
```

### Multi-File Organization

```
tests/
├── common/
│   ├── mod.rs
│   ├── fixtures.rs
│   └── helpers.rs
├── e2e/
│   ├── workflow_test.rs
│   └── error_handling_test.rs
└── integration_test.rs
```

## Best Practices

### 1. **One Concept Per Test File**

```rust
// tests/test_git_operations.rs - All git-related integration tests
// tests/test_file_processing.rs - All file processing integration tests
// tests/test_cli_interface.rs - All CLI integration tests
```

### 2. **Consistent Naming**

```rust
// Unit tests in src/
#[test]
fn add_returns_sum_of_two_numbers() { }

// Integration tests in tests/
#[test]
fn test_user_can_process_file_end_to_end() { }
```

### 3. **Fixture Reusability**

```rust
// Reuse fixtures across multiple test files
// tests/common/fixtures.rs defines TempEnvironment

// Use in different test files
// tests/test_init.rs uses TempEnvironment
// tests/test_process.rs uses TempEnvironment
// tests/test_workflow.rs uses TempEnvironment
```

### 4. **Clear Test Dependencies**

```rust
// Good: Tests are independent
#[test]
fn test_1() { /* doesn't depend on test_2 */ }

#[test]
fn test_2() { /* doesn't depend on test_1 */ }

// Bad: Tests have implicit dependencies
#[test]
fn test_1() {
    GLOBAL_STATE.set("value");
}

#[test]
fn test_2() {
    assert_eq!(GLOBAL_STATE.get(), "value");  // Depends on test_1
}
```

### 5. **Balance Between Files**

- Don't create one test file per test function (too fragmented)
- Don't put all tests in one file (hard to navigate)
- Group by feature or testing concern

## References

- [Test Organization - The Rust Book](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [How to organize your Rust tests - LogRocket](https://blog.logrocket.com/how-to-organize-rust-tests/)
- [Testing With Fixtures in Rust](https://dawchihliou.github.io/articles/testing-with-fixtures-in-rust)
- [rstest Documentation](https://docs.rs/rstest/)
- [Managing Test Fixtures and Setup/Teardown Logic - Sling Academy](https://www.slingacademy.com/article/managing-test-fixtures-and-setup-teardown-logic-in-rust/)
