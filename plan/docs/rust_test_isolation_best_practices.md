# Rust Test Isolation Best Practices

## 1. Test Isolation Patterns

Rust's testing framework provides inherent isolation between tests through several mechanisms:

### Unit Tests vs Integration Tests
- **Unit tests** are compiled with the library code and can access private interfaces
- **Integration tests** ([docs](https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory)) are compiled as separate crates/processes
- Integration tests provide better isolation for tests that use external resources

### Process-Level Isolation
- Integration tests run as separate binaries/processes by default
- This provides strong isolation between tests

## 2. Tempfile Usage Patterns

### Basic tempfile crate usage:

```rust
use tempfile::tempdir;
use tempfile::NamedTempFile;
use tempfile::tempfile;

// Create an anonymous temporary file (auto-deleted by OS)
let mut file = tempfile()?;
writeln!(file, "test data")?;

// Create a named temporary file with path access
let named_file = NamedTempFile::new()?;
let path = named_file.path();

// Create a temporary directory
let temp_dir = tempdir()?;
let file_path = temp_dir.path().join("test.txt");
std::fs::write(&file_path, "content")?;
// Directory auto-deletes when temp_dir goes out of scope
```

### Key tempfile features:
- **`tempfile()`**: Creates anonymous temporary files
- **`NamedTempFile::new()`**: Creates named temporary files accessible by path
- **`tempdir()`**: Creates temporary directories with recursive deletion on drop
- **Automatic cleanup**: All types implement `Drop`

## 3. Test Fixtures with Cleanup

### Pattern 1: Fixture Struct with Drop Cleanup
```rust
use tempfile::TempDir;
use std::path::PathBuf;

struct TestFixture {
    temp_dir: TempDir,
    source_path: PathBuf,
    file_path: PathBuf,
}

impl TestFixture {
    fn new(fixture_filename: &str) -> Self {
        let root_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR not set");

        let temp_dir = TempDir::new().unwrap();

        TestFixture {
            temp_dir,
            source_path: PathBuf::from(root_dir),
            file_path: temp_dir.path().join(fixture_filename),
        }
    }
}
```

### Pattern 2: Panic-Resistant Setup/Teardown
```rust
use std::panic;

fn run_test_with_setup_teardown<T>(test: T)
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    setup();

    let result = panic::catch_unwind(|| {
        test();
    });

    teardown();

    assert!(result.is_ok(), "Test panicked");
}
```

## 4. Unique Naming Strategies

### Pattern 1: Atomic Counter for Unique IDs
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

fn unique_test_id() -> usize {
    static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
    ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[test]
fn test_with_unique_id() {
    let id = unique_test_id();
    let temp_file = format!("test_{}.tmp", id);
}
```

### Pattern 2: Thread-Local Storage
```rust
use std::cell::RefCell;

thread_local! {
    static THREAD_ID: RefCell<usize> = RefCell::new(0);
}

fn thread_unique_id() -> usize {
    THREAD_ID.with(|cell| {
        *cell.borrow_mut() += 1;
        *cell.borrow()
    })
}
```

### Pattern 3: Random String Generation
```rust
use rand::Rng;

fn generate_unique_name(prefix: &str) -> String {
    let random_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    format!("{}_{}", prefix, random_part)
}
```

## 5. Common Pitfalls to Avoid

### 1. Early Drop Problem
```rust
// WRONG: TempDir dropped prematurely
let temp_dir = tempdir().unwrap();
Command::new("process")
    .current_dir(temp_dir) // TempDir moved here
    .status()?;

// RIGHT: Keep TempDir in scope
let temp_dir = tempdir().unwrap();
Command::new("process")
    .current_dir(&temp_dir) // Pass by reference
    .status()?;
```

### 2. Race Conditions with Current Directory
```rust
// WRONG: Tests interfere with each other via current directory
#[test]
fn test_1() {
    env::set_current_dir("/tmp").unwrap();
}

// RIGHT: Use absolute paths
#[test]
fn test_2() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
}
```

## 6. Best Practices to Follow

### 1. Use One Temp Directory per Test
```rust
#[test]
fn test_isolated_file_operations() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
}
```

### 2. Integration Test Organization
```
tests/
├── integration_test.rs
├── common/
│   └── mod.rs  # Shared test utilities
├── fixtures/   # Test data files
```

## Key Resources

1. **[tempfile crate documentation](https://docs.rs/tempfile/)** - Official API reference
2. **[Test Organization - Rust Book](https://doc.rust-lang.org/book/ch11-03-test-organization.html)** - Official guide
3. **[Testing in Rust: Temporary Files](http://www.andrewra.dev/2019/03/01/testing-in-rust-temporary-files/)** - Practical guide
