# PRP: Test Isolation Verification for AI Agents

**Product Requirement Prompt for Verifying Rust Test Isolation**
**Purpose:** Guide AI agents to verify tests are properly isolated for parallel execution

---

## Quick Reference for AI Agents

### Core Verification Commands

```bash
# 1. Baseline test run (parallel)
cargo test --all

# 2. High-parallelism test
cargo test -- --test-threads=8

# 3. Sequential execution (debugging)
cargo test -- --test-threads=1

# 4. Repeated execution (flakiness check)
for i in {1..10}; do cargo test -- --test-threads=8 || exit 1; done

# 5. Environment variable override
RUST_TEST_THREADS=1 cargo test
```

### Success Criteria

✅ **All tests pass** with default parallelism
✅ **All tests pass** with `--test-threads=8` (high parallelism)
✅ **All tests pass** with `--test-threads=1` (sequential)
✅ **100% pass rate** across 10 repeated runs
✅ **Same results** in parallel and sequential modes

---

## Critical Verification Points

### 1. Check for Proper Test Dependencies

**Command:**
```bash
grep -E "tempfile|serial_test|assert_cmd" Cargo.toml
```

**Expected Output:**
```toml
[dev-dependencies]
tempfile = "3.0"
serial_test = "3.0"
assert_cmd = "2.0"
```

### 2. Check for Hardcoded Paths (BAD)

**Command:**
```bash
grep -r '"/tmp/' tests/
grep -r '"~/' tests/
```

**Expected:** No matches

**Correct Pattern:**
```rust
let temp = TempDir::new()?;
let path = temp.path().join("file.txt");
```

### 3. Verify Serial Test Usage

**Check tests that modify environment:**
```bash
grep -r 'std::env::set_var' tests/ | grep -v '^\s*//'
```

**Expected:** Each test with `set_var` should have `#[serial]` attribute

**Example:**
```rust
#[test]
#[serial]  // REQUIRED when modifying environment
fn test_with_env() {
    std::env::set_var("MY_VAR", "value");
}
```

### 4. Verify TempDir Scope

**Check for potential premature cleanup:**
```bash
grep -B 2 -A 5 'TempDir::new()' tests/*.rs
```

**Warning Pattern:**
```rust
// WRONG - TempDir dropped immediately
let path = TempDir::new().unwrap().path().to_path_buf();
```

**Correct Pattern:**
```rust
// CORRECT - TempDir kept in scope
let temp = TempDir::new().unwrap();
let path = temp.path().to_path_buf();
// ... use path ...
```

### 5. Check for Unique Identifiers

**Pattern to look for:**
```rust
// GOOD - Unique names for parallel tests
let mode_name = format!("test_mode_{}", unique_test_id());
```

**Anti-Pattern:**
```rust
// BAD - Static names cause race conditions
create_mode("test_mode").unwrap();
```

### 6. Verify Drop Implementations

**Check fixtures have cleanup:**
```bash
grep -A 10 'impl Drop' tests/common/*.rs
```

**Expected:** Fixtures should clean up locks/resources

**Example:**
```rust
impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = cleanup_git_locks(&self.path);
    }
}
```

---

## Common Anti-Patterns to Detect

### Anti-Pattern 1: Premature TempDir Cleanup

❌ **Bad:**
```rust
#[test]
fn test_wrong() {
    let path = TempDir::new().unwrap().path().to_path_buf();
    fs::write(&path.join("file.txt"), "data").unwrap();  // FAILS
}
```

✅ **Good:**
```rust
#[test]
fn test_correct() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();
    fs::write(&path.join("file.txt"), "data").unwrap();  // OK
}
```

### Anti-Pattern 2: Missing Serial Attribute

❌ **Bad:**
```rust
#[test]
fn test_env() {
    std::env::set_var("MY_VAR", "value");  // Race condition!
}
```

✅ **Good:**
```rust
#[test]
#[serial]
fn test_env() {
    std::env::set_var("MY_VAR", "value");  // Safe
}
```

### Anti-Pattern 3: Hardcoded Paths

❌ **Bad:**
```rust
#[test]
fn test_config() {
    let path = PathBuf::from("/tmp/config.json");
    fs::write(&path, "{}").unwrap();  // Race condition!
}
```

✅ **Good:**
```rust
#[test]
fn test_config() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("config.json");
    fs::write(&path, "{}").unwrap();  // Isolated
}
```

### Anti-Pattern 4: Non-Unique Resource Names

❌ **Bad:**
```rust
#[test]
fn test_create() {
    create_mode("test_mode").unwrap();  // Conflicts!
}
```

✅ **Good:**
```rust
#[test]
fn test_create() {
    let name = format!("test_mode_{}", unique_test_id());
    create_mode(&name).unwrap();  // Unique
}
```

---

## Test Isolation Best Practices

### 1. Use `tempfile` for All Temporary Resources

```rust
use tempfile::TempDir;

#[test]
fn test_with_temp_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "data")?;
    assert!(file_path.exists());

    Ok(())
    // TempDir automatically deleted here
}
```

### 2. Use RAII for Resource Management

```rust
pub struct TestFixture {
    _tempdir: TempDir,  // Underscore = intentionally unused
    pub path: PathBuf,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = cleanup_resources(&self.path);
    }
}

#[test]
fn test_with_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    // Use fixture.path
    Ok(())
}
```

### 3. Generate Unique Identifiers

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn unique_test_id() -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("test_{}_{}", std::process::id(), count)
}
```

### 4. Use Absolute Paths

```rust
#[test]
fn test_absolute_paths() {
    let temp = TempDir::new().unwrap();
    let absolute_path = temp.path().canonicalize().unwrap();
    fs::write(absolute_path.join("file.txt"), "data").unwrap();
}
```

---

## Diagnostic Commands

### Check for Test Flakiness

```bash
# Run tests 10 times
for i in {1..10}; do
    echo "Run $i:"
    cargo test -- --test-threads=8 || exit 1
done
```

### Compare Parallel vs Sequential

```bash
# Parallel
cargo test 2>&1 | tee parallel.txt

# Sequential
cargo test -- --test-threads=1 2>&1 | tee sequential.txt

# Compare
diff parallel.txt sequential.txt
```

### Measure Performance Impact

```bash
# Time parallel execution
time cargo test -- --test-threads=8

# Time sequential execution
time cargo test -- --test-threads=1

# Parallel should be 2-4x faster
```

### Count Serial Tests

```bash
# Too many serial tests reduce parallelism benefit
grep -r '#\[serial\]' tests/ | wc -l
```

**Guideline:** Keep serial tests < 10% of total tests

---

## Official Documentation Links

### Core Rust Testing Documentation

- **The Rust Book - Testing:** https://doc.rust-lang.org/book/ch11-00-testing.html
- **Cargo Test Commands:** https://doc.rust-lang.org/cargo/commands/cargo-test.html
- **Test Organization:** https://doc.rust-lang.org/book/ch11-03-test-organization.html

### Crate Documentation

- **tempfile:** https://docs.rs/tempfile/latest/tempfile/
- **serial_test:** https://docs.rs/serial_test/latest/serial_test/
- **assert_cmd:** https://docs.rs/assert_cmd/latest/assert_cmd/

### Community Resources

- **"How to Test in Rust":** https://matklad.github.io/2021/05/31/how-to-test.html
- **Rust Testing Best Practices:** https://jaketrent.com/post/rust-testing-best-practices/

---

## Verification Checklist

### Pre-Test Checks

- [ ] Required dev-dependencies present (`tempfile`, `serial_test`)
- [ ] Test structure follows best practices (`tests/common/` for utilities)
- [ ] No hardcoded paths in tests
- [ ] Tests using `std::env::set_var` have `#[serial]` attribute
- [ ] TempDir kept in scope (not dropped prematurely)

### Execution Checks

- [ ] All tests pass with default parallelism
- [ ] All tests pass with `--test-threads=8`
- [ ] All tests pass with `--test-threads=1`
- [ ] Same results in parallel and sequential modes
- [ ] 100% pass rate across 10 repeated runs

### Code Quality Checks

- [ ] Fixtures have `Drop` implementations for cleanup
- [ ] Unique identifiers used for resource names
- [ ] Absolute paths used for file operations
- [ ] No test ordering assumptions
- [ ] Proper error handling (no `.unwrap()` panic risks)

### Performance Checks

- [ ] Parallel execution is significantly faster
- [ ] Serial tests are minimal (< 10% of total)
- [ ] No unnecessary `#[serial]` attributes

---

## Example: Well-Isolated Test

```rust
use tempfile::TempDir;
use serial_test::serial;

#[test]
#[serial]  // Required because we modify environment
fn test_well_isolated() -> Result<(), Box<dyn std::error::Error>> {
    // Setup: Use temporary directory
    let temp = TempDir::new()?;
    let test_path = temp.path();

    // Setup: Use unique identifier
    let resource_name = format!("test_{}", unique_test_id());

    // Setup: Isolate environment
    let original = std::env::var("MY_VAR").ok();
    std::env::set_var("MY_VAR", "test_value");

    // Test logic here
    // ...

    // Cleanup: Restore environment (or use Drop impl)
    match original {
        Some(val) => std::env::set_var("MY_VAR", val),
        None => std::env::remove_var("MY_VAR"),
    }

    // Automatic cleanup: temp deleted when dropped
    Ok(())
}
```

---

## Quick Decision Tree

```
Test fails?
├─ Fails in sequential mode?
│  └─ No → Isolation bug (race condition)
│     └─ Add unique identifiers or use #[serial]
│
├─ Fails in parallel mode only?
│  └─ Yes → Race condition
│     └─ Check: Shared state? Filesystem? Environment?
│     └─ Fix: Use tempfile, unique names, or #[serial]
│
└─ Intermittent failures?
   └─ Flakiness
      └─ Add proper cleanup, check locks, use unique names
```

---

**Document Version:** 1.0
**Last Updated:** 2026-01-12
**For:** AI Agent Verification of Test Isolation
