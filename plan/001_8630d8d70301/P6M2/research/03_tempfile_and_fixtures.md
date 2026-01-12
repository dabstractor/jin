# Temporary Files and Test Fixtures in Rust

## Overview

The `tempfile` crate provides secure, cross-platform temporary file and directory creation with automatic cleanup. The `assert_fs` crate builds on this with additional assertion capabilities for filesystem testing.

## Tempfile Crate

### Installation

```toml
[dev-dependencies]
tempfile = "3"
assert_fs = "1"
```

**Minimum Rust Version:** 1.63.0

### Key Features

- **Secure Creation:** OS-specific security practices
- **Automatic Cleanup:** Files deleted when dropped
- **Cross-Platform:** Works consistently across OS platforms
- **No Cleanup Failures:** Standard `tempfile()` doesn't fail on cleanup

### Basic Usage

```rust
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

// Create a temporary file (anonymous, no path access)
let mut tmpfile = tempfile::tempfile().unwrap();
std::io::Write::write_all(&mut tmpfile, b"Hello World!").unwrap();

// Create a named temporary file (allows path access)
let mut named_tmp = tempfile::NamedTempFile::new().unwrap();
let path = named_tmp.path().to_path_buf();
std::io::Write::write_all(&mut named_tmp, b"test content").unwrap();

// Create a temporary directory
let tmpdir = TempDir::new().unwrap();
let dir_path = tmpdir.path();
fs::write(dir_path.join("file.txt"), "content").ok();
```

## Choosing the Right Type

### Anonymous `tempfile()`

**Best for:** Single-use temporary files without path requirements

**Advantages:**
- Never fails on cleanup
- Doesn't rely on filesystem paths
- Simplest to use
- Most reliable

```rust
#[test]
fn process_and_write() {
    let mut tmp = tempfile::tempfile().unwrap();

    // Write processed data
    std::io::Write::write_all(&mut tmp, b"data").unwrap();

    // Auto-cleanup when dropped
} // tmpfile deleted here
```

### Named `NamedTempFile`

**Best for:** When you need the file path or want persistence for debugging

```rust
#[test]
fn with_path() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();

    // Can pass path to CLI tools
    assert!(path.exists());

    // Auto-cleanup
} // File deleted here
```

### Directory `TempDir`

**Best for:** Testing file operations, multiple files, directory structures

```rust
#[test]
fn directory_operations() {
    let dir = TempDir::new().unwrap();

    // Create files
    let file_path = dir.path().join("test.txt");
    std::fs::write(&file_path, "content").unwrap();

    // Can pass directory to CLI
    assert!(file_path.exists());

    // Explicit cleanup check
    let path = dir.path().to_path_buf();
    drop(dir);
    assert!(!path.exists()); // Verified deletion
}
```

## Storing TempDir in Test Structures

**Critical Pattern:** Store the `TempDir` handle to keep it in scope

```rust
struct TestFixture {
    _tempdir: TempDir,  // Underscore prefix indicates intentional non-use
    source: PathBuf,
    path: PathBuf,
}

impl TestFixture {
    fn new(source_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().join(source_name);

        // Copy fixture file
        std::fs::copy(
            format!("tests/fixtures/{}", source_name),
            &path,
        )?;

        Ok(TestFixture {
            _tempdir: tempdir,  // Keeps directory in scope
            source: PathBuf::from(source_name),
            path,
        })
    }
}

#[test]
fn use_fixture() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new("test_file.txt")?;

    // fixture._tempdir stays in scope until test ends
    // So the temp directory isn't deleted prematurely

    assert!(fixture.path.exists());
    Ok(())
}
```

## Assert_fs: Higher-Level Assertions

### Installation

```toml
[dev-dependencies]
assert_fs = "1"
predicates = "3"
```

### TempDir with Assertions

```rust
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn filesystem_assertions() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create temp directory
    let temp = assert_fs::TempDir::new()?;

    // Reference child files
    let input_file = temp.child("input.txt");

    // Assertions on paths that don't exist yet
    input_file.assert(predicate::path::missing());

    // Create the file
    input_file.write_str("content")?;

    // Verify it now exists
    input_file.assert(predicate::path::exists());

    // Explicit cleanup
    temp.close()?;
    Ok(())
}
```

### NamedTempFile Pattern

```rust
use assert_fs::prelude::*;

#[test]
fn find_content_in_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create named temp file
    let file = assert_fs::NamedTempFile::new("sample.txt")?;

    // Write test data
    file.write_str("A test\nActual content\nAnother test")?;

    // Can use file.path() in CLI commands
    let path = file.path();

    // File automatically deleted when dropped
    Ok(())
}
```

## Common Patterns

### Pattern 1: Fixture Setup and Cleanup

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn with_fixture() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Setup
    let tmpdir = TempDir::new()?;
    let fixture_path = tmpdir.path().join("input.txt");
    fs::write(&fixture_path, "initial content")?;

    // Test
    let result = process_file(&fixture_path);

    // Assertions
    assert_eq!(result, "processed");

    // Cleanup (automatic)
    Ok(())
}
```

### Pattern 2: Multiple Test Files

```rust
use assert_fs::TempDir;
use assert_fs::prelude::*;

#[test]
fn multi_file_test() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Create multiple files
    temp.child("file1.txt").write_str("content1")?;
    temp.child("file2.txt").write_str("content2")?;

    // Create subdirectory
    temp.child("subdir").create_dir_all()?;
    temp.child("subdir/nested.txt").write_str("nested")?;

    // Run command on directory
    let output = run_command(&temp.path());

    // Verify results
    temp.child("output.txt").assert(predicate::path::exists());

    temp.close()?;
    Ok(())
}
```

### Pattern 3: Conditional Persistence for Debugging

```rust
use assert_fs::TempDir;

#[test]
fn debug_preserves_files() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Do test work
    std::fs::write(temp.path().join("debug.txt"), "debug data")?;

    // Only keep temp files if test fails (for debugging)
    if std::env::var("KEEP_TEST_ARTIFACTS").is_ok() {
        // Convert to persistent, print path
        let persistent = temp.into_persistent();
        println!("Test files kept at: {}", persistent.path().display());
    } else {
        temp.close()?;  // Clean up
    }

    Ok(())
}
```

### Pattern 4: Using CARGO_MANIFEST_DIR for Fixtures

```rust
use tempfile::TempDir;
use std::fs;
use std::path::PathBuf;

fn get_fixture_path(fixture_name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(fixture_name)
}

#[test]
fn copy_fixture_to_temp() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let tmpdir = TempDir::new()?;
    let fixture_src = get_fixture_path("test_data.txt");
    let fixture_dst = tmpdir.path().join("test_data.txt");

    fs::copy(&fixture_src, &fixture_dst)?;

    // Use fixture_dst in test
    assert!(fixture_dst.exists());

    Ok(())
}
```

### Pattern 5: Parallel Test Safety

```rust
use tempfile::TempDir;

#[test]
fn test_1() {
    let tmpdir = TempDir::new().unwrap();  // Gets unique directory
    // ...
}

#[test]
fn test_2() {
    let tmpdir = TempDir::new().unwrap();  // Different directory
    // ...
}

// Both tests can run in parallel without conflicts
// Each has isolated filesystem state
```

## File System Isolation

### Why It Matters

- **Parallelization:** Tests can run concurrently without conflicts
- **Cleanup Guarantee:** Files deleted regardless of test outcome
- **No Side Effects:** Tests don't affect each other's state
- **Clean Test Bed:** Each test starts fresh

### Achieving Isolation

```rust
use assert_fs::TempDir;

#[test]
fn isolated_test_1() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;  // Unique path

    // All file operations happen in this isolated directory
    std::fs::write(temp.path().join("file.txt"), "data")?;

    // Cleanup happens automatically
    temp.close()?;
    Ok(())
}
```

## Best Practices

| Practice | Reason |
|----------|--------|
| Prefer `tempfile()` over `NamedTempFile` | More reliable cleanup |
| Store `TempDir` in test structs | Prevents premature cleanup |
| Use `assert_fs` for CLI testing | Better assertions and ergonomics |
| Copy fixtures to temp directories | Avoids modifying test data |
| Use `CARGO_MANIFEST_DIR` for fixture paths | Works in any environment |
| Clean up explicitly when debugging | Can inspect files if test fails |

## References

- [tempfile - Rust Documentation](https://docs.rs/tempfile/)
- [GitHub: Stebalien/tempfile](https://github.com/Stebalien/tempfile)
- [assert_fs - Rust Documentation](https://docs.rs/assert_fs)
- [GitHub: assert-rs/assert_fs](https://github.com/assert-rs/assert_fs)
- [Using assert_fs and predicates - egghead.io](https://egghead.io/lessons/rust-using-assert_fs-and-predicates-to-integration-test-with-a-real-temporary-file-system)
- [Testing in Rust: Temporary Files](http://www.andrewra.dev/2019/03/01/testing-in-rust-temporary-files/)
- [Advanced Rust Testing: Filesystem Isolation](https://rust-exercises.com/advanced-testing/05_filesystem_isolation/02_tempfile.html)
