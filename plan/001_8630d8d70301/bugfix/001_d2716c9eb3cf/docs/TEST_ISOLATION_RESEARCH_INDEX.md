# Test Isolation Research - Index

**Comprehensive research on Rust test isolation for parallel execution**
**Created:** 2026-01-12

---

## Research Overview

This research provides comprehensive guidance on Rust test isolation best practices, specifically focused on parallel test execution. The research is designed to help AI agents verify that tests are properly isolated and can run safely in parallel without race conditions, flakiness, or state leakage.

---

## Documents Created

### 1. **RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md** (1,268 lines)
**Full comprehensive research document**

**Contents:**
- How Rust's built-in test harness handles parallel execution
- The `--test-threads` flag and usage
- Common causes of test flakiness
- Best practices for test isolation
- Using `ctor` and related crates
- Managing temporary directories
- Verification checklist for AI agents
- Real-world examples from Jin project
- Common gotchas and anti-patterns
- Recommended resources with links

**Use when:** You need comprehensive understanding of test isolation concepts

**Link:** `/home/dustin/projects/jin/RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md`

---

### 2. **PRP_TEST_ISOLATION_VERIFICATION.md** (458 lines)
**Product Requirement Prompt for AI agents**

**Contents:**
- Quick reference commands
- Critical verification points
- Common anti-patterns to detect
- Test isolation best practices
- Diagnostic commands
- Verification checklist
- Example well-isolated test
- Quick decision tree

**Use when:** You're an AI agent verifying test isolation

**Link:** `/home/dustin/projects/jin/PRP_TEST_ISOLATION_VERIFICATION.md`

---

### 3. **RUST_TEST_ISOLATION_EXAMPLES.md** (797 lines)
**Code examples and patterns**

**Contents:**
- Basic tempfile patterns
- Fixture patterns
- Environment variable isolation
- Git operations testing
- CLI testing
- Parallel-safe resource creation
- Serial test patterns
- Advanced patterns

**Use when:** You need code examples for writing isolated tests

**Link:** `/home/dustin/projects/jin/RUST_TEST_ISOLATION_EXAMPLES.md`

---

### 4. **RESEARCH_SUMMARY.md** (487 lines)
**Executive summary of findings**

**Contents:**
- Research documents overview
- Key findings
- Official documentation links
- Real-world examples from Jin project
- Common gotchas and anti-patterns
- Verification checklist
- Actionable recommendations
- GitHub examples to study

**Use when:** You need a quick overview of the research

**Link:** `/home/dustin/projects/jin/RESEARCH_SUMMARY.md`

---

### 5. **QUICK_REFERENCE_TEST_ISOLATION.md** (169 lines)
**One-page quick reference**

**Contents:**
- Essential commands
- Basic test template
- Unique identifier generator
- Fixture template
- Serial test template
- CLI test template
- Common anti-patterns table
- Verification checklist
- Essential crates
- Key documentation links

**Use when:** You need quick reference during development

**Link:** `/home/dustin/projects/jin/QUICK_REFERENCE_TEST_ISOLATION.md`

---

## Key Findings Summary

### 1. Rust's Test Harness
- Runs tests in parallel by default
- Each test runs in a separate thread
- Default thread count = CPU core count
- Tests within the same file run concurrently

### 2. The `--test-threads` Flag
```bash
cargo test -- --test-threads=1     # Sequential
cargo test -- --test-threads=8     # 8 threads
RUST_TEST_THREADS=1 cargo test     # Environment variable
```

### 3. Common Flakiness Causes
- Hardcoded file paths
- Environment variable pollution
- Git lock conflicts
- Port conflicts
- Global mutable state
- Non-unique resource names

### 4. Best Practices
1. Use `tempfile` for automatic cleanup
2. Use RAII patterns for resource management
3. Generate unique identifiers for parallel tests
4. Use absolute paths
5. Isolate environment variables
6. Clean up locks in `Drop` implementations
7. Use `#[serial]` sparingly (performance impact)
8. Make each test independent

### 5. Essential Crates
```toml
[dev-dependencies]
tempfile = "3.0"      # Temporary files/dirs
serial_test = "3.0"   # Serial execution
assert_cmd = "2.0"    # CLI testing
predicates = "3.0"    # Output assertions
```

---

## Official Documentation Links

### Core Rust Documentation
- **The Rust Book - Testing:** https://doc.rust-lang.org/book/ch11-00-testing.html
- **Cargo Test:** https://doc.rust-lang.org/cargo/commands/cargo-test.html
- **Test Organization:** https://doc.rust-lang.org/book/ch11-03-test-organization.html
- **Rust By Example:** https://doc.rust-lang.org/rust-by-example/testing.html

### Crate Documentation
- **tempfile:** https://docs.rs/tempfile/latest/tempfile/
- **serial_test:** https://docs.rs/serial_test/latest/serial_test/
- **assert_cmd:** https://docs.rs/assert_cmd/latest/assert_cmd/
- **predicates:** https://docs.rs/predicates/latest/predicates/

### Community Resources
- **"How to Test in Rust":** https://matklad.github.io/2021/05/31/how-to-test.html
- **Rust Testing Best Practices:** https://jaketrent.com/post/rust-testing-best-practices/

---

## Real-World Examples

### From Jin Project (`/home/dustin/projects/jin`)

**Key Implementation Files:**
1. `/home/dustin/projects/jin/tests/common/fixtures.rs`
   - TestFixture with automatic cleanup
   - RemoteFixture for Git operations
   - Unique test ID generation

2. `/home/dustin/projects/jin/tests/common/git_helpers.rs`
   - Git lock cleanup utilities
   - GitTestEnv with automatic cleanup

3. `/home/dustin/projects/jin/src/test_utils.rs`
   - UnitTestContext for unit tests
   - Environment variable isolation

**Example Pattern:**
```rust
pub struct TestFixture {
    _tempdir: TempDir,
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = cleanup_git_locks(&self.path);
    }
}

#[test]
#[serial]
fn test_example() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();

    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(fixture.jin_dir.as_ref().unwrap()))?;

    Ok(())
}
```

---

## Verification Commands

### Basic Verification
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

### Code Quality Checks
```bash
# Check for hardcoded paths
grep -r '"/tmp/' tests/
grep -r '"~/' tests/

# Check for serial test usage
grep -r 'std::env::set_var' tests/ | grep -v '^\s*//'

# Count serial tests
grep -r '#\[serial\]' tests/ | wc -l
```

---

## Common Anti-Patterns

### Anti-Pattern 1: Premature TempDir Cleanup
```rust
// WRONG
let path = TempDir::new().unwrap().path().to_path_buf();

// CORRECT
let temp = TempDir::new().unwrap();
let path = temp.path().to_path_buf();
```

### Anti-Pattern 2: Missing Serial Attribute
```rust
// WRONG
#[test]
fn test_env() {
    std::env::set_var("MY_VAR", "value");
}

// CORRECT
#[test]
#[serial]
fn test_env() {
    std::env::set_var("MY_VAR", "value");
}
```

### Anti-Pattern 3: Hardcoded Paths
```rust
// WRONG
let path = PathBuf::from("/tmp/config.json");

// CORRECT
let temp = TempDir::new()?;
let path = temp.path().join("config.json");
```

### Anti-Pattern 4: Non-Unique Names
```rust
// WRONG
create_mode("test_mode").unwrap();

// CORRECT
let name = format!("test_mode_{}", unique_test_id());
create_mode(&name).unwrap();
```

---

## Which Document Should I Use?

### For AI Agents
→ **PRP_TEST_ISOLATION_VERIFICATION.md**
- Quick reference commands
- Verification checklists
- Decision trees
- Anti-pattern detection

### For Writing Tests
→ **RUST_TEST_ISOLATION_EXAMPLES.md**
- Code examples
- Copy-paste patterns
- Common scenarios
- Best practice implementations

### For Deep Understanding
→ **RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md**
- Comprehensive explanations
- Official documentation links
- Real-world examples
- Detailed best practices

### For Quick Reference
→ **QUICK_REFERENCE_TEST_ISOLATION.md**
- One-page reference
- Essential commands
- Common templates
- Anti-pattern table

### For Overview
→ **RESEARCH_SUMMARY.md**
- Executive summary
- Key findings
- All links
- Actionable recommendations

---

## Research Statistics

**Total Lines of Code/Documentation:** 3,179 lines
**Documents Created:** 5
**Code Examples:** 50+
**Anti-Patterns Identified:** 10+
**Best Practices Documented:** 20+
**Official Links:** 15+
**GitHub Examples:** 4 projects

---

## Project Context

**Project:** Jin CLI Tool
**Location:** `/home/dustin/projects/jin`
**Purpose:** Phantom Git layer system for developer configuration
**Test Framework:** Rust's built-in test harness
**Key Crates:** tempfile, serial_test, assert_cmd

**Project demonstrates:**
- Excellent test isolation patterns
- Proper use of fixtures
- Git lock cleanup
- Environment variable isolation
- Parallel-safe resource creation

---

## Next Steps

### For AI Agents
1. Review **PRP_TEST_ISOLATION_VERIFICATION.md**
2. Use verification checklists
3. Run diagnostic commands
4. Apply patterns from examples

### For Developers
1. Read **QUICK_REFERENCE_TEST_ISOLATION.md**
2. Study examples in **RUST_TEST_ISOLATION_EXAMPLES.md**
3. Reference **RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md** for deep dives
4. Apply patterns to your tests

### For Verification
1. Run tests with multiple thread counts
2. Check for anti-patterns
3. Verify proper cleanup
4. Ensure parallel safety

---

## Document Locations

All documents are located in: `/home/dustin/projects/jin/`

1. `RESEARCH_RUST_TEST_ISOLATION_PARALLEL_EXECUTION.md`
2. `PRP_TEST_ISOLATION_VERIFICATION.md`
3. `RUST_TEST_ISOLATION_EXAMPLES.md`
4. `RESEARCH_SUMMARY.md`
5. `QUICK_REFERENCE_TEST_ISOLATION.md`
6. `TEST_ISOLATION_RESEARCH_INDEX.md` (this file)

---

**Research Completed:** 2026-01-12
**Status:** Complete
**Purpose:** Product Requirement Prompt (PRP) for Test Isolation Verification
