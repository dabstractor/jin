# Product Requirement Prompt: Fix Test Infrastructure Issues

**Task ID**: P2.M4.T1
**Milestone**: P2.M4 - Fix Failing Unit Tests
**Status**: Ready for Implementation

---

## Goal

**Feature Goal**: Improve test infrastructure to eliminate common failure patterns across 12 failing unit tests by creating unified test setup, adding pre-test Git lock cleanup, and enforcing absolute path usage.

**Deliverable**: A unified test setup module in `tests/common/` with:
1. A common `setup_unit_test()` function that replaces duplicated setup code
2. Pre-test Git lock cleanup integrated into all unit tests
3. Absolute path resolution for all test file operations

**Success Definition**:
- All unit tests can use the shared `setup_unit_test()` function
- Pre-test Git lock cleanup prevents lock contention failures
- Tests use absolute paths consistently, eliminating "file not found" errors
- Test isolation is maintained (no state leakage between tests)

---

## User Persona

**Target User**: Developer implementing test fixes for P2.M4.T2, P2.M4.T3, and P2.M4.T4

**Use Case**: The developer needs a reliable test infrastructure foundation before fixing individual test failures. Without this foundation, individual test fixes would be fragile and prone to regression.

**User Journey**:
1. Developer creates `setup_unit_test()` in `tests/common/mod.rs`
2. Developer adds pre-test cleanup to `tests/common/git_helpers.rs`
3. Developer updates unit tests to use the new setup
4. Developer validates that tests pass consistently

**Pain Points Addressed**:
- Eliminates duplicated `setup_test_env()` functions across 10+ command files
- Prevents Git lock contention from causing intermittent failures
- Removes path resolution issues causing "file not found" errors

---

## Why

**Business value and user impact**:
- Enables reliable test execution for continuous integration
- Reduces developer time spent debugging flaky tests
- Provides stable foundation for subsequent test fixes (P2.M4.T2-T4)

**Integration with existing features**:
- Builds on existing `TestFixture` and `RemoteFixture` patterns in `tests/common/fixtures.rs`
- Extends `cleanup_git_locks()` function in `tests/common/git_helpers.rs`
- Maintains compatibility with `serial_test` crate for global state tests

**Problems this solves**:
- **Duplicated setup code**: 10+ files have nearly identical `setup_test_env()` functions
- **Git lock contention**: Stale `.git/index.lock` files cause 4 mode command tests to fail
- **Path resolution issues**: Relative paths combined with `std::env::set_current_dir()` cause 6+ file system tests to fail

---

## What

**User-visible behavior**: No direct user-visible behavior changes. This is internal test infrastructure improvement.

**Technical requirements**:
1. Create a unified `setup_unit_test()` function in `tests/common/mod.rs`
2. Add `cleanup_before_test()` function in `tests/common/git_helpers.rs`
3. Update all unit test setup functions to use the new shared setup
4. Ensure all file operations use absolute paths derived from the test context

### Success Criteria

- [ ] `setup_unit_test()` function created in `tests/common/mod.rs`
- [ ] `cleanup_before_test()` function created in `tests/common/git_helpers.rs`
- [ ] All 10+ unit test files updated to use `setup_unit_test()`
- [ ] All tests use absolute paths for file operations
- [ ] All existing tests continue to pass
- [ ] No new test failures introduced

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES. This PRP provides:
- Exact file paths and patterns to follow
- Specific code examples from the existing codebase
- Complete research on Rust testing best practices
- Git lock cleanup patterns with code examples
- Dependency-ordered implementation tasks

---

### Documentation & References

```yaml
# MUST READ - Critical for understanding existing patterns

- file: tests/common/fixtures.rs
  why: Contains TestFixture pattern to follow for unit test setup
  pattern: TestFixture struct with _tempdir, path, jin_dir fields; Drop implementation for cleanup
  gotcha: _tempdir must be kept in scope to prevent premature cleanup

- file: tests/common/git_helpers.rs
  why: Contains existing cleanup_git_locks() function to extend
  pattern: Recursive lock file cleanup under .git/ directory
  gotcha: Must clean up locks in both project .git and JIN_DIR .git

- file: tests/common/assertions.rs
  why: Custom assertions that may be useful in unit tests
  pattern: assert_workspace_file, assert_staging_contains, etc.

- file: src/commands/mode.rs
  why: Example of duplicated setup_test_env() pattern to replace
  pattern: setup_test_env() function using TempDir and JIN_DIR

- file: src/commands/diff.rs
  why: Example of test that needs proper directory creation
  pattern: test_execute_staged_empty creates .jin directory manually

- docfile: plan/architecture/test_analysis.md
  why: Comprehensive analysis of all 12 failing tests
  section: Section 1 (Test Fixture Patterns), Section 3 (Common Failure Patterns)
  critical: Documents exact failure patterns for each test category

- docfile: plan/P2M4T1/research/rust_testing_best_practices.md
  why: Industry best practices for Rust test infrastructure
  section: Section 6 (Recommendations for Jin Project), Section 7 (Useful Testing Utilities)

- docfile: plan/P2M4T1/research/git_lock_handling.md
  why: Comprehensive Git lock file handling patterns
  section: Section 3 (Pre-test Cleanup Patterns), Section 4 (git2 Crate Lock Handling)
  critical: All Git lock file types and cleanup strategies

- docfile: plan/P2M4T1/research/test_fixture_patterns.md
  why: Test fixture library comparison and patterns
  section: Section 1 (rstest Crate), Section 5 (Recommendations for Jin Project)

- url: https://doc.rust-lang.org/book/ch11-00-testing.html
  why: Official Rust testing guide
  critical: Understanding of #[cfg(test)] module pattern

- url: https://docs.rs/tempfile/latest/tempfile/
  why: TempDir lifecycle and best practices
  critical: RAII pattern for automatic cleanup

- url: https://docs.rs/serial_test/latest/serial_test/
  why: Serial test execution for global state tests
  critical: #[serial] attribute usage for tests using set_current_dir
```

---

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── commands/
│   │   ├── mode.rs          # Unit tests with setup_test_env()
│   │   ├── diff.rs          # Unit tests with setup_test_env()
│   │   ├── mv.rs            # Unit tests with setup_test_env()
│   │   ├── repair.rs        # Unit tests with setup_test_env()
│   │   ├── reset.rs         # Unit tests with setup_test_env()
│   │   ├── rm.rs            # Unit tests with setup_test_env()
│   │   ├── add.rs           # Unit tests with setup_test_env()
│   │   ├── export.rs        # Unit tests with setup_test_env()
│   │   ├── layers.rs        # Unit tests with setup_test_env()
│   │   └── scope.rs         # Unit tests with setup_test_env()
│   └── ...
├── tests/
│   ├── common/
│   │   ├── mod.rs           # Module exports (add setup_unit_test here)
│   │   ├── fixtures.rs      # TestFixture, RemoteFixture patterns
│   │   ├── git_helpers.rs   # cleanup_git_locks() (extend with cleanup_before_test)
│   │   └── assertions.rs    # Custom assertions
│   └── *.rs                 # Integration tests (not modified in this task)
├── Cargo.toml               # Test dependencies: tempfile, serial_test
└── plan/
    ├── architecture/
    │   └── test_analysis.md # Detailed test failure analysis
    └── P2M4T1/
        ├── PRP.md           # This document
        └── research/
            ├── rust_testing_best_practices.md
            ├── git_lock_handling.md
            └── test_fixture_patterns.md
```

---

### Desired Codebase Tree (files to be modified)

```bash
tests/
├── common/
│   ├── mod.rs               # NEW: setup_unit_test() function
│   ├── fixtures.rs          # UNCHANGED
│   ├── git_helpers.rs       # NEW: cleanup_before_test() function
│   └── assertions.rs        # UNCHANGED
```

All unit test files in `src/commands/*.rs` will import and use the new shared setup.

---

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: JIN_DIR environment variable is process-global state
// Tests must use #[serial] attribute when using set_current_dir() or set_var()
// Example from src/commands/mode.rs:
#[test]
#[serial]
fn test_create_mode() {
    let _temp = setup_test_env();  // Sets JIN_DIR and current_dir
    // ... test code ...
}

// CRITICAL: TempDir must be kept in scope throughout test
// Early dropping causes premature directory deletion
// BAD:
// let temp_dir = setup_test_env();
// {
//     let _temp = temp_dir;
// } // temp_dir dropped here, directory deleted
// use temp_dir.path(); // PANIC: directory no longer exists

// GOOD:
// let _temp = setup_test_env();  // Underscore prefix prevents accidental drop
// // _temp lives until end of function

// CRITICAL: Git lock files must be cleaned BEFORE test runs, not just after
// The existing cleanup_git_locks() is only in Drop implementations
// New cleanup_before_test() must be called at START of setup_unit_test()

// CRITICAL: File paths must be absolute to avoid current_dir issues
// BAD: std::fs::write(".jin/context", "mode: test")  // Relative path
// GOOD: let context_path = project_path.join(".jin/context");
//       std::fs::write(&context_path, "mode: test")  // Absolute path

// CRITICAL: .jin/staging/index.json must exist for diff command tests
// Some tests assume this file exists but don't create it
// Fix: Create empty staging index in setup_unit_test()

// CRITICAL: ProjectContext must be saved before operations that read it
// Always call context.save() after modifying context in tests
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models needed. This task creates utility functions, not data structures.

---

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/common/git_helpers.rs - cleanup_before_test()
  - IMPLEMENT: cleanup_before_test(jin_dir: &Path) function
  - FOLLOW pattern: Existing cleanup_git_locks() function
  - ADD: Check for JIN_DIR environment variable and clean locks there too
  - NAMING: cleanup_before_test (parallel to cleanup_git_locks)
  - LOCATION: tests/common/git_helpers.rs (after cleanup_git_locks function)
  - DEPENDENCIES: None (standalone utility function)

Task 2: CREATE tests/common/mod.rs - setup_unit_test()
  - IMPLEMENT: setup_unit_test() -> UnitTestContext function
  - FOLLOW pattern: setup_test_env() from src/commands/mode.rs (lines 456-470)
  - UNIFY: All variations of setup_test_env() from different command files
  - ADD: Call cleanup_before_test() at start to clean stale locks
  - RETURN: UnitTestContext struct with temp_dir, project_path, jin_dir fields
  - NAMING: setup_unit_test() (clearer than setup_test_env)
  - LOCATION: tests/common/mod.rs (after existing exports)
  - DEPENDENCIES: Task 1 (needs cleanup_before_test)

Task 3: MODIFY src/commands/mode.rs
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - UPDATE: All test functions to use new setup
  - PRESERVE: #[serial] attributes on tests using set_current_dir
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)

Task 4: MODIFY src/commands/diff.rs
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - FIX: test_execute_staged_empty to create staging index
  - UPDATE: All test functions to use new setup
  - PRESERVE: Test assertions and logic
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)

Task 5: MODIFY src/commands/mv.rs
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - FIX: Use absolute paths from context for file operations
  - UPDATE: All test functions to use new setup
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)

Task 6: MODIFY src/commands/reset.rs
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - FIX: Create layer refs before testing reset
  - UPDATE: All test functions to use new setup
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)

Task 7: MODIFY src/commands/rm.rs
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - FIX: Set up staging state before testing dry-run
  - UPDATE: All test functions to use new setup
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)

Task 8: MODIFY src/commands/repair.rs
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - UPDATE: All test functions to use new setup
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)

Task 9: MODIFY remaining command test files
  - INTEGRATE: Import setup_unit_test from tests::common
  - REPLACE: Local setup_test_env() with setup_unit_test()
  - FILES: add.rs, export.rs, layers.rs, scope.rs
  - UPDATE: All test functions to use new setup
  - DEPENDENCIES: Task 2 (needs setup_unit_test function)
```

---

### Implementation Patterns & Key Details

```rust
// ============================================
// Task 1: cleanup_before_test() implementation
// ============================================
// File: tests/common/git_helpers.rs
// Add after cleanup_git_locks() function (around line 95)

/// Clean up Git locks BEFORE running a test
///
/// This function should be called at the START of test setup to ensure
/// no stale locks from previous test runs cause failures.
///
/// # Arguments
/// * `jin_dir` - Path to JIN_DIR (may be None, using default ~/.jin)
///
/// # Gotchas
/// - Must clean BOTH project .git AND JIN_DIR .git
/// - Silently ignores errors (directories may not exist yet)
/// - Call this BEFORE any JinRepo operations
pub fn cleanup_before_test(jin_dir: Option<&Path>) {
    // CRITICAL: Clean up JIN_DIR locks first (most common source of contention)
    if let Some(jin_dir) = jin_dir {
        let _ = cleanup_git_locks(jin_dir);
    }

    // Clean up current directory's .git locks (if we're in a git repo)
    if let Ok(current_dir) = std::env::current_dir() {
        let _ = cleanup_git_locks(&current_dir);
    }
}

// ============================================
// Task 2: UnitTestContext and setup_unit_test()
// ============================================
// File: tests/common/mod.rs
// Add after existing module exports (around line 9)

use std::path::PathBuf;
use tempfile::TempDir;
use crate::common::git_helpers::cleanup_before_test;

/// Test context for unit tests
///
/// Provides all paths and context needed for unit tests.
/// CRITICAL: Keep _temp_dir in scope to prevent premature cleanup.
pub struct UnitTestContext {
    /// Temporary directory (must be kept in scope)
    _temp_dir: TempDir,
    /// Absolute path to test project directory
    pub project_path: PathBuf,
    /// Absolute path to isolated JIN_DIR
    pub jin_dir: PathBuf,
}

impl UnitTestContext {
    /// Get the absolute path to .jin directory
    pub fn jin_path(&self) -> PathBuf {
        self.project_path.join(".jin")
    }

    /// Get the absolute path to .jin/context file
    pub fn context_path(&self) -> PathBuf {
        self.jin_path().join("context")
    }

    /// Get the absolute path to .jin/staging/index.json
    pub fn staging_index_path(&self) -> PathBuf {
        self.jin_path().join("staging/index.json")
    }
}

/// Unified test setup for unit tests
///
/// This function replaces the duplicated setup_test_env() functions
/// across all command test files.
///
/// # Returns
/// * UnitTestContext with all paths and temporary directory
///
/// # Gotchas
/// - Call cleanup_before_test() FIRST to remove stale locks
/// - Keep the returned UnitTestContext in scope (use let _ctx = ...)
/// - All paths are absolute, avoiding current_dir() issues
/// - Creates .jin directory structure for tests that need it
///
/// # Example
/// ```rust
/// #[test]
/// #[serial]  // Required because we set JIN_DIR
/// fn test_something() {
///     let ctx = setup_unit_test();
///     // Use ctx.project_path, ctx.jin_dir for absolute paths
///     let context_path = ctx.context_path();
///     std::fs::write(&context_path, "mode: test").unwrap();
/// }
/// ```
pub fn setup_unit_test() -> UnitTestContext {
    use crate::core::config::ProjectContext;
    use crate::git::repo::JinRepo;

    // CRITICAL: Clean up locks BEFORE creating new test environment
    cleanup_before_test(None);

    // Create temporary directory for isolated test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    let jin_dir = temp_dir.path().join(".jin_global");

    // CRITICAL: Set JIN_DIR before any Jin operations
    std::env::set_var("JIN_DIR", &jin_dir);

    // CRITICAL: Set current directory for tests that expect it
    // (Tests must use #[serial] attribute to prevent conflicts)
    std::env::set_current_dir(&project_path)
        .expect("Failed to set current directory");

    // Initialize Jin repository
    let _ = JinRepo::open_or_create();

    // Create .jin directory structure
    std::fs::create_dir_all(".jin").expect("Failed to create .jin directory");

    // Create and save default context
    let context = ProjectContext::default();
    context.save().expect("Failed to save context");

    // Create empty staging index (many tests expect this to exist)
    let staging_dir = project_path.join(".jin/staging");
    std::fs::create_dir_all(&staging_dir).expect("Failed to create staging directory");
    std::fs::write(staging_dir.join("index.json"), "{}")
        .expect("Failed to create staging index");

    UnitTestContext {
        _temp_dir: temp_dir,
        project_path,
        jin_dir,
    }
}

// ============================================
// Task 3-9: Updating unit test files
// ============================================
// Pattern for each src/commands/*.rs file:

// 1. Remove the local setup_test_env() function (usually in tests module)

// 2. Add import at top of tests module:
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::setup_unit_test;  // NEW: shared setup

    // 3. Update each test function:
    #[test]
    #[serial]  // PRESERVE: Required for set_current_dir/set_var
    fn test_example() {
        // OLD: let _temp = setup_test_env();
        // NEW:
        let ctx = setup_unit_test();

        // Use ctx.project_path for absolute paths
        let file_path = ctx.project_path.join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        // ... rest of test ...
    }
}
```

---

### Integration Points

```yaml
TESTS_COMMON_MOD:
  - add: setup_unit_test() function (Task 2)
  - add: UnitTestContext struct (Task 2)
  - import: use crate::common::git_helpers::cleanup_before_test

TESTS_COMMON_GIT_HELPERS:
  - add: cleanup_before_test() function (Task 1)

UNIT_TEST_FILES (10+ files):
  - remove: Local setup_test_env() function
  - add: Import tests::common::setup_unit_test
  - modify: All test functions to use setup_unit_test()
  - preserve: #[serial] attributes on global state tests

CARGO_TOML:
  - no changes needed (tempfile and serial_test already present)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification
cargo fmt --check              # Check formatting
cargo clippy -- -D warnings    # Lint checks

# Expected: Zero warnings or errors. Fix any issues before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test each command module as it's updated
cargo test --package jin --lib -- commands::mode
cargo test --package jin --lib -- commands::diff
cargo test --package jin --lib -- commands::mv
cargo test --package jin --lib -- commands::reset
cargo test --package jin --lib -- commands::rm
cargo test --package jin --lib -- commands::repair

# Full unit test suite
cargo test --lib

# Expected: All tests pass. Some may still fail due to test-specific issues
# (those will be fixed in P2.M4.T2-T4), but no new failures should appear.
```

### Level 3: Integration Testing (System Validation)

```bash
# Ensure integration tests still work
cargo test --test integration_tests

# Check for any test isolation issues
cargo test --lib -- --test-threads=1

# Expected: All integration tests pass. No lock contention errors.
```

### Level 4: Regression Testing

```bash
# Run full test suite
cargo test

# Validate specific test categories that should improve:
cargo test --lib -- -- mode  # Should have fewer lock errors
cargo test --lib -- -- diff  # Should have fewer file-not-found errors

# Expected:
# - Mode tests may still fail but with different errors (not lock-related)
# - Diff tests may still fail but with different errors (not file-not-found)
# - No regression in currently passing tests
```

---

## Final Validation Checklist

### Technical Validation

- [ ] Task 1: cleanup_before_test() function created in tests/common/git_helpers.rs
- [ ] Task 2: setup_unit_test() function created in tests/common/mod.rs
- [ ] Task 3: src/commands/mode.rs updated to use setup_unit_test()
- [ ] Task 4: src/commands/diff.rs updated to use setup_unit_test()
- [ ] Task 5: src/commands/mv.rs updated to use setup_unit_test()
- [ ] Task 6: src/commands/reset.rs updated to use setup_unit_test()
- [ ] Task 7: src/commands/rm.rs updated to use setup_unit_test()
- [ ] Task 8: src/commands/repair.rs updated to use setup_unit_test()
- [ ] Task 9: Remaining command files updated (add, export, layers, scope)
- [ ] All unit tests compile without errors
- [ ] No clippy warnings introduced
- [ ] Code is properly formatted

### Feature Validation

- [ ] All tests use shared setup_unit_test() function
- [ ] Pre-test Git lock cleanup is active for all tests
- [ ] All file operations use absolute paths from UnitTestContext
- [ ] #[serial] attributes preserved on global state tests
- [ ] TempDir lifecycle properly managed (no early drops)
- [ ] No new test failures introduced

### Code Quality Validation

- [ ] Follows existing codebase patterns (TestFixture, RemoteFixture)
- [ ] UnitTestContext struct provides useful helper methods
- [ ] Documentation comments explain gotchas
- [ ] Function naming matches existing conventions
- [ ] No duplicated code across test files

### Foundation for Subsequent Tasks

- [ ] P2.M4.T2 (Fix File System Path Issues) can build on this foundation
- [ ] P2.M4.T3 (Fix Git Lock Contention) can build on this foundation
- [ ] P2.M4.T4 (Fix Test Expectation Mismatches) can build on this foundation

---

## Anti-Patterns to Avoid

- ❌ Don't remove #[serial] attributes from tests that use set_current_dir() or set_var()
- ❌ Don't drop TempDir early (must keep UnitTestContext in scope)
- ❌ Don't use relative paths for file operations (use ctx.project_path.join())
- ❌ Don't call cleanup_git_locks() in tests (use cleanup_before_test() via setup_unit_test())
- ❌ Don't create multiple setup functions (use setup_unit_test() everywhere)
- ❌ Don't modify integration test files (this task is for unit tests only)
- ❌ Don't skip creating .jin/staging/index.json (diff tests need it)
- ❌ Don't set JIN_DIR to None (always provide a path)
- ❌ Don't forget to save context after modifying it
- ❌ Don't run tests in parallel without #[serial] on global state tests

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success likelihood

**Validation**: The completed PRP enables an AI agent unfamiliar with the codebase to:
1. Understand the test infrastructure patterns from existing code
2. Create the shared setup function following established patterns
3. Update all unit tests to use the shared setup
4. Validate that no regressions are introduced

**Dependencies on Subsequent Tasks**:
- P2.M4.T2 (Fix File System Path Issues) depends on completion of this task
- P2.M4.T3 (Fix Git Lock Contention) depends on completion of this task
- P2.M4.T4 (Fix Test Expectation Mismatches) can proceed independently

**Estimated Implementation Time**: 2-4 hours for a developer familiar with Rust
