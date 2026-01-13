# Quick Reference: Rust Test Isolation

**One-page reference for common test isolation patterns**

---

## Essential Commands

```bash
# Run tests (parallel by default)
cargo test

# Run sequentially (debugging)
cargo test -- --test-threads=1

# Run with specific thread count
cargo test -- --test-threads=8

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --exact

# Environment variable
RUST_TEST_THREADS=1 cargo test
```

---

## Basic Test Template

```rust
use tempfile::TempDir;

#[test]
fn test_template() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let path = temp.path().join("file.txt");

    fs::write(&path, "data")?;
    assert!(path.exists());

    Ok(())
}
```

---

## Unique Identifier Generator

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn unique_test_id() -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test_{}_{}", std::process::id(), count)
}
```

---

## Fixture Template

```rust
pub struct TestFixture {
    _tempdir: TempDir,
    pub path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        Ok(TestFixture { _tempdir: tempdir, path })
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = cleanup_resources(&self.path);
    }
}
```

---

## Serial Test Template

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_with_global_state() {
    std::env::set_var("MY_VAR", "value");
    // Safe - no other test runs concurrently
}
```

---

## CLI Test Template

```rust
use assert_cmd::Command;

#[test]
fn test_cli() {
    Command::new(env!("CARGO_BIN_EXE_jin"))
        .arg("--help")
        .assert()
        .success();
}
```

---

## Common Anti-Patterns

| Anti-Pattern | Solution |
|--------------|----------|
| `PathBuf::from("/tmp/")` | `TempDir::new()?.path().join()` |
| `TempDir::new().unwrap().path()` | Store `TempDir` in variable first |
| `create_mode("test")` | `create_mode(&format!("test_{}", unique_test_id()))` |
| `std::env::set_var()` | Add `#[serial]` attribute |
| `let path = { TempDir::new().unwrap() }.path()` | Keep `TempDir` in scope |

---

## Verification Checklist

- [ ] Tests pass: `cargo test`
- [ ] Tests pass: `cargo test -- --test-threads=8`
- [ ] Tests pass: `cargo test -- --test-threads=1`
- [ ] No hardcoded paths
- [ ] `TempDir` kept in scope
- [ ] Unique identifiers used
- [ ] `#[serial]` on tests modifying environment

---

## Essential Crates

```toml
[dev-dependencies]
tempfile = "3.0"      # Temporary files/dirs
serial_test = "3.0"   # Serial execution
assert_cmd = "2.0"    # CLI testing
predicates = "3.0"    # Output assertions
```

---

## Key Documentation Links

- **Rust Book - Testing:** https://doc.rust-lang.org/book/ch11-00-testing.html
- **Cargo Test:** https://doc.rust-lang.org/cargo/commands/cargo-test.html
- **tempfile:** https://docs.rs/tempfile/latest/tempfile/
- **serial_test:** https://docs.rs/serial_test/latest/serial_test/

---

**Quick Reference for:**
- Writing isolated tests
- Debugging test failures
- Verifying parallel execution
