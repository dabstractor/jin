# Research Summary: Rust Test Isolation for Parallel Execution

**Research Conducted:** 2026-01-12
**Purpose:** Product Requirement Prompt (PRP) for AI agent verification
**Project:** Jin CLI Tool - `/home/dustin/projects/jin`

---

## Research Documents Created

This research produced three comprehensive documents:

### 1. **RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md** (Comprehensive Guide)
   - Full research document with deep dives
   - Official documentation links
   - Detailed explanations of concepts
   - Real-world examples from the Jin project

### 2. **PRP_TEST_ISOLATION_VERIFICATION.md** (AI Agent Guide)
   - Quick reference for AI agents
   - Verification checklists
   - Diagnostic commands
   - Decision trees
   - Anti-pattern detection

### 3. **RUST_TEST_ISOLATION_EXAMPLES.md** (Code Examples)
   - Idiomatic Rust test patterns
   - Copy-paste examples
   - Fixture implementations
   - Common scenarios

---

## Key Findings

### 1. How Rust's Test Harness Handles Parallel Execution

**Default Behavior:**
- Tests run in parallel using multiple threads
- Each test runs in a separate thread
- Default thread count = CPU core count
- Tests within the same file run concurrently

**Critical Implications:**
- Tests MUST NOT assume execution order
- Tests MUST NOT share mutable state without synchronization
- Tests MUST clean up resources
- Tests SHOULD use unique temporary directories

### 2. The `--test-threads` Flag

**Usage:**
```bash
cargo test -- --test-threads=1     # Sequential execution
cargo test -- --test-threads=8     # 8 parallel threads
RUST_TEST_THREADS=1 cargo test     # Environment variable
```

**When to Use:**
- Debugging flaky tests
- Tests with shared state
- Resource-constrained environments
- Tests requiring exclusive access

### 3. Common Causes of Test Flakiness

| Cause | Symptom | Solution |
|-------|---------|----------|
| Hardcoded paths | Race conditions | Use `tempfile` |
| Environment pollution | Unexpected values | Use `#[serial]` or cleanup |
| Git lock conflicts | "Failed to remove lock" | Clean up locks in `Drop` |
| Port conflicts | "Address already in use" | Allocate unique ports |
| Global mutable state | Unexpected values | Use synchronization or `#[serial]` |
| Non-unique names | Resource conflicts | Generate unique IDs |

### 4. Best Practices for Test Isolation

**Core Principles:**
1. Use `tempfile` for automatic cleanup
2. Use RAII patterns for resource management
3. Generate unique identifiers for parallel tests
4. Use absolute paths to avoid `current_dir()` issues
5. Isolate environment variables with fixtures
6. Clean up locks (especially Git locks)
7. Use `#[serial]` sparingly (performance impact)
8. Make each test independent

### 5. The `ctor` Crate

**Purpose:** Execute code before/after test runs

**When to Use:**
- Global test setup (e.g., start test database)
- Global test teardown
- Initialize test logging

**Warning:** Avoid for most tests - creates global state

**Alternative:** Prefer per-test fixtures

### 6. Managing Temporary Directories

**Pattern:**
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

**Critical Gotcha:**
```rust
// WRONG - TempDir dropped prematurely
let path = TempDir::new().unwrap().path().to_path_buf();
fs::write(&path.join("file.txt"), "data").unwrap();  // FAILS

// CORRECT - Keep TempDir in scope
let temp = TempDir::new().unwrap();
let path = temp.path().to_path_buf();
fs::write(&path.join("file.txt"), "data").unwrap();  // OK
```

---

## Official Documentation Links

### Core Rust Documentation

1. **The Rust Book - Chapter 11: Testing**
   https://doc.rust-lang.org/book/ch11-00-testing.html

2. **Cargo Test Commands**
   https://doc.rust-lang.org/cargo/commands/cargo-test.html

3. **Test Organization**
   https://doc.rust-lang.org/book/ch11-03-test-organization.html

4. **Rust By Example - Testing**
   https://doc.rust-lang.org/rust-by-example/testing.html

5. **The Rust Reference - Testing**
   https://doc.rust-lang.org/reference/testing.html

### Crate Documentation

1. **tempfile**
   https://docs.rs/tempfile/latest/tempfile/
   - Temporary files and directories
   - Automatic cleanup via RAII

2. **serial_test**
   https://docs.rs/serial_test/latest/serial_test/
   - Sequential test execution
   - Named serial groups

3. **assert_cmd**
   https://docs.rs/assert_cmd/latest/assert_cmd/
   - CLI testing utilities

4. **predicates**
   https://docs.rs/predicates/latest/predicates/
   - Output assertion helpers

### Community Resources

1. **"How to Test in Rust" by Aleksey Kladov**
   https://matklad.github.io/2021/05/31/how-to-test.html

2. **Rust Testing Best Practices**
   https://jaketrent.com/post/rust-testing-best-practices/

3. **Rust API Guidelines - Testing**
   https://rust-lang.github.io/api-guidelines/testing.html

### Advanced Testing

1. **proptest** (Property-Based Testing)
   https://altsysrq.github.io/proptest-book/

2. **insta** (Snapshot Testing)
   https://insta.rs/docs/

3. **mockall** (Mocking Framework)
   https://docs.rs/mockall/latest/mockall/

---

## Real-World Examples from Jin Project

### Location
**Project:** `/home/dustin/projects/jin`
**Test Directory:** `/home/dustin/projects/jin/tests/`

### Key Implementation Files

1. **`/home/dustin/projects/jin/tests/common/fixtures.rs`**
   - TestFixture implementation with automatic cleanup
   - RemoteFixture for Git operations
   - Unique test ID generation

2. **`/home/dustin/projects/jin/tests/common/git_helpers.rs`**
   - Git lock cleanup utilities
   - GitTestEnv with automatic cleanup

3. **`/home/dustin/projects/jin/src/test_utils.rs`**
   - UnitTestContext for unit tests
   - Environment variable isolation
   - Automatic restoration on drop

### Example Pattern from Jin

```rust
pub struct TestFixture {
    _tempdir: TempDir,  // Must keep in scope
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // CRITICAL: Clean up Git locks before temp dir deletion
        let _ = cleanup_git_locks(&self.path);
        if let Some(ref jin_dir) = self.jin_dir {
            let _ = cleanup_git_locks(jin_dir);
        }
    }
}

#[test]
#[serial]  // Required because we set JIN_DIR
fn test_layer_routing() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(fixture.jin_dir.as_ref().unwrap()))?;

    // ... test logic ...

    Ok(())
}
```

---

## Common Gotchas and Anti-Patterns

### Gotcha 1: Premature TempDir Cleanup

**Anti-Pattern:**
```rust
let path = TempDir::new().unwrap().path().to_path_buf();
// TempDir dropped here, directory deleted
```

**Solution:**
```rust
let temp = TempDir::new().unwrap();
let path = temp.path().to_path_buf();
// Keep temp in scope
```

### Gotcha 2: Missing Serial Attribute

**Anti-Pattern:**
```rust
#[test]
fn test_env() {
    std::env::set_var("MY_VAR", "value");  // Race condition!
}
```

**Solution:**
```rust
#[test]
#[serial]
fn test_env() {
    std::env::set_var("MY_VAR", "value");  // Safe
}
```

### Gotcha 3: Hardcoded Paths

**Anti-Pattern:**
```rust
let path = PathBuf::from("/tmp/config.json");
```

**Solution:**
```rust
let temp = TempDir::new()?;
let path = temp.path().join("config.json");
```

### Gotcha 4: Non-Unique Resource Names

**Anti-Pattern:**
```rust
create_mode("test_mode").unwrap();  // Conflicts!
```

**Solution:**
```rust
let name = format!("test_mode_{}", unique_test_id());
create_mode(&name).unwrap();
```

---

## Verification Checklist for AI Agents

### Pre-Test Verification

- [ ] Required dev-dependencies present (`tempfile`, `serial_test`)
- [ ] Test structure follows best practices
- [ ] No hardcoded paths in tests
- [ ] Tests using `std::env::set_var` have `#[serial]` attribute
- [ ] TempDir kept in scope (not dropped prematurely)

### Execution Verification

```bash
# Run tests with default parallelism
cargo test --all

# Run with high parallelism
cargo test -- --test-threads=8

# Run sequentially
cargo test -- --test-threads=1

# Run repeatedly (flakiness check)
for i in {1..10}; do
    cargo test -- --test-threads=8 || exit 1
done
```

**Success Criteria:**
- [ ] All tests pass with default parallelism
- [ ] All tests pass with `--test-threads=8`
- [ ] All tests pass with `--test-threads=1`
- [ ] 100% pass rate across 10 repeated runs
- [ ] Same results in parallel and sequential modes

### Code Quality Verification

- [ ] Fixtures have `Drop` implementations
- [ ] Unique identifiers used for resource names
- [ ] Absolute paths used for file operations
- [ ] No test ordering assumptions
- [ ] Proper error handling

### Performance Verification

```bash
# Count serial tests (should be < 10% of total)
grep -r '#\[serial\]' tests/ | wc -l
```

---

## Actionable Recommendations

### For Writing New Tests

1. **Always use `tempfile`** for temporary resources
2. **Generate unique identifiers** for parallel-safe resource creation
3. **Use fixtures** for complex setup/teardown
4. **Implement `Drop`** for custom cleanup
5. **Use `#[serial]` sparingly** - only when necessary
6. **Use absolute paths** to avoid `current_dir()` issues
7. **Clean up Git locks** in test fixtures

### For Verifying Test Isolation

1. **Run tests with multiple thread counts**
2. **Run tests repeatedly** to catch flakiness
3. **Check for hardcoded paths**
4. **Verify serial test usage** is appropriate
5. **Measure performance impact** of serial tests
6. **Compare parallel vs sequential** results

### For Debugging Test Failures

1. **Run with `--test-threads=1`** to isolate race conditions
2. **Use `--nocapture`** to see test output
3. **Use `--exact`** to run specific tests
4. **Check for stale locks** from previous runs
5. **Verify unique identifiers** are being used

---

## GitHub Examples of Well-Isolated Tests

### Recommended Projects to Study

1. **Rust Standard Library**
   https://github.com/rust-lang/rust/tree/master/library
   - Comprehensive test coverage
   - Excellent isolation patterns

2. **Tokio (Async Runtime)**
   https://github.com/tokio-rs/tokio
   - Complex async testing
   - Resource management patterns

3. **Serde (Serialization)**
   https://github.com/serde-rs/serde
   - Property-based testing examples
   - Data-driven test patterns

4. **Clap (CLI Parser)**
   https://github.com/clap-rs/clap
   - CLI testing patterns
   - Integration test organization

---

## Recommended Dev Dependencies

```toml
[dev-dependencies]
# Core testing utilities
tempfile = "3.0"        # Temporary files/directories
serial_test = "3.0"     # Sequential test execution

# CLI testing
assert_cmd = "2.0"      # CLI command testing
predicates = "3.0"      # Output assertions

# Advanced testing
proptest = "1.4"        # Property-based testing
quickcheck = "1.0"      # Property-based testing
insta = "1.34"          # Snapshot testing
mockall = "0.12"        # Mocking framework

# Async testing
tokio-test = "0.4"      # Tokio test utilities
```

---

## Summary

**Research Output:**
- 3 comprehensive documents created
- Real-world examples from Jin project analyzed
- Official documentation links compiled
- Best practices documented with code examples
- Anti-patterns identified with solutions
- Verification checklists provided

**Key Insights:**
1. Rust tests run in parallel by default - isolation is critical
2. The `--test-threads` flag is essential for debugging
3. `tempfile` crate provides RAII-based automatic cleanup
4. `serial_test` crate enables sequential execution when needed
5. Unique identifiers prevent resource conflicts in parallel tests
6. Git lock cleanup is critical for tests involving Git operations
7. Environment variable isolation requires special handling

**Next Steps for AI Agents:**
1. Review the three research documents
2. Use verification checklists to assess test isolation
3. Apply code examples to improve test isolation
4. Use diagnostic commands to verify parallel execution
5. Reference official documentation for deeper understanding

---

**Documents Created:**
- `/home/dustin/projects/jin/RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md` (Full research)
- `/home/dustin/projects/jin/PRP_TEST_ISOLATION_VERIFICATION.md` (AI agent guide)
- `/home/dustin/projects/jin/RUST_TEST_ISOLATION_EXAMPLES.md` (Code examples)
- `/home/dustin/projects/jin/RESEARCH_SUMMARY.md` (This document)

**Research Date:** 2026-01-12
**Status:** Complete
