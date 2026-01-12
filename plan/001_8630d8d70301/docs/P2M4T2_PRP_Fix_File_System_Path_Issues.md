# Product Requirement Prompt: Fix File System Path Issues in Tests

---

## Goal

**Feature Goal**: Fix 25 unit tests failing due to file system "No such file or directory" errors caused by relative path usage in `setup_unit_test()` before the current directory is set.

**Deliverable**: Fixed `src/test_utils.rs::setup_unit_test()` function that uses absolute paths for all directory and file creation operations.

**Success Definition**: All 25 currently failing unit tests pass when running `cargo test --lib`.

## User Persona

**Target User**: Developers running the Jin test suite.

**Use Case**: Developers need a reliable test suite that passes consistently when running unit tests in isolation or as part of the full test suite.

**User Journey**:
1. Developer makes code changes
2. Developer runs `cargo test` to verify changes
3. Tests pass consistently without "No such file or directory" errors

**Pain Points Addressed**: Currently 25 unit tests fail with file not found errors, making it difficult to verify code changes.

## Why

- Test reliability is critical for maintaining code quality
- The failing tests block development on other features
- The root cause is a simple path resolution issue in shared test infrastructure
- Fixing this will unblock P2.M4.T3 and P2.M4.T4 tasks

## What

Fix the `setup_unit_test()` function in `src/test_utils.rs` to use absolute paths when creating the `.jin` directory structure instead of relative paths. The issue occurs because:

1. The function sets `JIN_DIR` environment variable (line 140)
2. It creates `.jin` directory using relative path `".jin"` (line 153)
3. It THEN sets the current directory (line 147)
4. Between steps 2-3, the relative path `".jin"` resolves to the original working directory (which may have been cleaned up by a previous test)
5. This causes `create_dir_all(".jin")` to fail with "No such file or directory"

### Success Criteria

- [ ] All 25 failing unit tests pass: `cargo test --lib`
- [ ] No changes to test logic, only path handling
- [ ] Tests pass when run in isolation and in parallel
- [ ] Integration tests continue to pass

### Failing Tests (25 total)

```
commands::add::tests::test_stage_file_creates_blob
commands::add::tests::test_validate_file_success
commands::context::tests::test_execute_default_context
commands::export::tests::test_add_to_git_no_git_repo
commands::export::tests::test_add_to_git_success
commands::export::tests::test_execute_file_not_jin_tracked
commands::mode::tests::test_create_mode
commands::mode::tests::test_create_mode_duplicate
commands::mode::tests::test_delete_active_mode
commands::mode::tests::test_show_no_mode
commands::mode::tests::test_use_mode
commands::mv::tests::test_execute_dry_run
commands::mv::tests::test_move_file_not_staged
commands::repair::tests::test_check_staging_index_corrupted
commands::repair::tests::test_check_staging_index_missing
commands::repair::tests::test_check_workspace_metadata_missing
commands::repair::tests::test_create_default_context
commands::repair::tests::test_execute_dry_run
commands::repair::tests::test_execute_no_issues
commands::repair::tests::test_rebuild_staging_index
commands::repair::tests::test_rebuild_workspace_metadata
commands::reset::tests::test_reset_hard_with_force
commands::rm::tests::test_execute_dry_run
commands::scope::tests::test_delete_mode_bound_scope
commands::scope::tests::test_delete_untethered_scope
```

## All Needed Context

### Context Completeness Check

Before writing this PRP, validated: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?" - **YES**

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: src/test_utils.rs
  why: Contains the setup_unit_test() function that needs to be fixed
  pattern: Unit test setup with TempDir, JIN_DIR isolation, directory creation
  gotcha: Lines 153-162 use relative paths before current directory is set

- file: tests/common/mod.rs
  why: Integration test version of setup_unit_test() - already fixed correctly
  pattern: Uses absolute paths from project_path for directory creation
  gotcha: Shows the correct pattern to follow (already fixed in P2.M4.T1)

- file: src/commands/diff.rs
  why: Example of passing test using setup_unit_test()
  pattern: Lines 476-488 show correct test usage pattern

- file: src/commands/mv.rs
  why: Example of failing test (test_execute_dry_run, test_move_file_not_staged)
  pattern: Lines 458-501 show test patterns

- file: src/commands/reset.rs
  why: Example of failing test (test_reset_hard_with_force)
  pattern: Lines 390-426 show test patterns

- file: src/commands/rm.rs
  why: Example of failing test (test_execute_dry_run)
  pattern: Lines 359-395 show test patterns

- url: https://doc.rust-lang.org/stable/std/fs/fn.create_dir_all.html
  why: Standard library documentation for recursively creating directories
  critical: create_dir_all requires a valid path - relative paths resolve before current_dir() changes

- url: https://docs.rs/tempfile/latest/tempfile/
  why: TempDir crate documentation used in test setup
  critical: TempDir::path() returns absolute path that should be used for all operations

- url: https://doc.rust-lang.org/std/env/fn.current_dir.html
  why: Understanding current directory behavior
  critical: Current directory affects relative path resolution - must be set BEFORE using relative paths
```

### Current Codebase tree (relevant sections)

```bash
jin/
├── src/
│   ├── test_utils.rs              # MAIN FIX TARGET - setup_unit_test()
│   ├── commands/
│   │   ├── diff.rs                # Tests: test_execute_staged_empty (PASSING)
│   │   ├── mv.rs                  # Tests: test_execute_dry_run, test_move_file_not_staged (FAILING)
│   │   ├── reset.rs               # Tests: test_reset_hard_with_force (FAILING)
│   │   ├── rm.rs                  # Tests: test_execute_dry_run (FAILING)
│   │   ├── mode.rs                # Multiple tests FAILING
│   │   ├── repair.rs              # Multiple tests FAILING
│   │   └── ...
│   └── ...
└── tests/
    └── common/
        └── mod.rs                  # CORRECT PATTERN - already uses absolute paths
```

### Desired Codebase tree with files to be added

No new files - only modification of `src/test_utils.rs`.

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: setup_unit_test() must use absolute paths from project_path
// The function MUST create directories BEFORE setting current directory
// because:
// 1. Tests run in parallel (serial_test attribute helps but isn't enough)
// 2. Previous tests may have cleaned up temp directories
// 3. Relative paths like ".jin" will resolve to non-existent locations

// WRONG (current code):
std::env::set_current_dir(&project_path).expect("Failed to set current directory");
std::fs::create_dir_all(".jin").expect("Failed to create .jin directory");  // Uses relative path

// RIGHT (fix):
let jin_path = project_path.join(".jin");
std::fs::create_dir_all(&jin_path).expect("Failed to create .jin directory");  // Uses absolute path
std::env::set_current_dir(&project_path).expect("Failed to set current directory");

// GOTCHA: The integration tests version (tests/common/mod.rs) already fixed this correctly
// Follow that pattern for consistency
```

## Implementation Blueprint

### Root Cause Analysis

The issue is in `src/test_utils.rs` lines 147-162:

```rust
// Line 147: Sets current directory
std::env::set_current_dir(&project_path).expect("Failed to set current directory");

// Lines 150-162: Creates directory structure using RELATIVE paths
let _ = JinRepo::open_or_create();

std::fs::create_dir_all(".jin").expect("Failed to create .jin directory");  // RELATIVE PATH!

let context = ProjectContext::default();
context.save().expect("Failed to save context");  // Fails because .jin doesn't exist

let staging_dir = project_path.join(".jin/staging");  // Uses absolute path - CORRECT
std::fs::create_dir_all(&staging_dir).expect("Failed to create staging directory");
std::fs::write(staging_dir.join("index.json"), "{}").expect("Failed to create staging index");
```

The problem: When multiple tests run in parallel, the `current_dir()` may not be set to a valid location when `setup_unit_test()` is called. The relative path `.jin` resolves to the current directory, which may have been cleaned up by a previous test.

### Implementation Tasks

```yaml
Task 1: MODIFY src/test_utils.rs setup_unit_test() function
  - CHANGE line 153: Replace relative path ".jin" with absolute path
  - BEFORE: std::fs::create_dir_all(".jin").expect("Failed to create .jin directory");
  - AFTER: let jin_path = project_path.join(".jin");
          std::fs::create_dir_all(&jin_path).expect("Failed to create .jin directory");
  - FOLLOW pattern: tests/common/mod.rs lines 92-101 (already fixed correctly)
  - NAMING: Use `jin_path` variable name to match integration test pattern
  - PLACEMENT: Lines 152-153 in src/test_utils.rs

Task 2: UPDATE context.save() call if needed
  - VERIFY context.save() creates parent directories if needed
  - If context.save() fails due to missing parent, add explicit directory creation
  - CHECK src/core/config.rs for ProjectContext::save() implementation
  - PATTERN: Look for std::fs::create_dir_all(parent) pattern in save() methods

Task 3: VERIFY UnitTestContext helper methods still work
  - CHECK jin_path(), context_path(), staging_index_path() methods
  - These methods use self.project_path internally, so they should still work
  - VERIFY no hardcoded relative paths in these methods

Task 4: ADD jin_path helper if it doesn't exist
  - CHECK if UnitTestContext has jin_path() method
  - If not, add it following the pattern from tests/common/mod.rs
  - IMPLEMENTATION: pub fn jin_path(&self) -> PathBuf { self.project_path.join(".jin") }
```

### Implementation Patterns & Key Details

```rust
// Show critical patterns and gotchas

// Pattern from tests/common/mod.rs (CORRECT - follow this):
pub fn setup_unit_test() -> UnitTestContext {
    // ... setup code ...

    // Create .jin directory structure using ABSOLUTE paths
    let jin_path = project_path.join(".jin");
    std::fs::create_dir_all(&jin_path).expect("Failed to create .jin directory");

    // Create and save default context
    let context = ProjectContext::default();
    context.save().expect("Failed to save context");

    // Create empty staging index using ABSOLUTE paths
    let staging_dir = project_path.join(".jin/staging");
    std::fs::create_dir_all(&staging_dir).expect("Failed to create staging directory");
    std::fs::write(staging_dir.join("index.json"), "{}").expect("Failed to create staging index");

    // ...
}

// CRITICAL: The key difference is using project_path.join(".jin") instead of ".jin"
// This ensures the path is absolute and doesn't depend on current_dir()

// GOTCHA: ProjectContext::save() may need parent directory creation
// Check src/core/config.rs for the save() implementation:
// pub fn save(&self) -> Result<()> {
//     let path = Self::default_path()?;  // Gets path, may not create parent
//     if let Some(parent) = path.parent() {
//         std::fs::create_dir_all(parent)?;  // This should handle it
//     }
//     // ...
// }
```

### Integration Points

```yaml
NO NEW INTEGRATIONS: This fix only touches test infrastructure

FILES AFFECTED:
  - modify: src/test_utils.rs (setup_unit_test() function only)

TEST AFFECTED:
  - All 25 failing unit tests should pass after fix
  - No integration tests affected
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after making the change - fix before proceeding
cargo check --lib                    # Check compilation
cargo clippy --lib                   # Linting checks

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the previously failing tests
cargo test --lib 2>&1 | grep -E "(FAILED|passed|failed)"

# Test specific failing tests individually
cargo test --lib test_execute_staged_empty -- --nocapture
cargo test --lib test_reset_hard_with_force -- --nocapture
cargo test --lib test_execute_dry_run -- --nocapture

# Run all unit tests
cargo test --lib

# Expected: All 25 previously failing tests should now pass
# Expected output: "test result: ok. 530 passed; 0 failed"
```

### Level 3: Integration Testing (System Validation)

```bash
# Run integration tests to ensure no regression
cargo test --test integration 2>&1 | tail -20

# Run full test suite
cargo test 2>&1 | grep -A5 "test result:"

# Expected: Integration tests still pass, no new failures
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Run tests in parallel to verify thread safety
cargo test --lib -- --test-threads=8 2>&1 | grep "test result:"

# Run tests multiple times to check for flakiness
for i in {1..5}; do cargo test --lib 2>&1 | grep -E "(passed|failed)"; done

# Check specific command tests that were failing
cargo test --lib commands::diff::tests::test_execute_staged_empty -- --nocapture
cargo test --lib commands::mv::tests::test_execute_dry_run -- --nocapture
cargo test --lib commands::reset::tests::test_reset_hard_with_force -- --nocapture
cargo test --lib commands::rm::tests::test_execute_dry_run -- --nocapture

# Expected: All tests pass consistently across multiple runs
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --lib` shows 530 passed, 0 failed
- [ ] No compilation errors: `cargo check --lib` passes
- [ ] No linting errors: `cargo clippy --lib` passes

### Feature Validation

- [ ] All 25 previously failing tests now pass
- [ ] Tests pass when run in isolation
- [ ] Tests pass when run in parallel
- [ ] Tests pass consistently across multiple runs
- [ ] Integration tests still pass (no regression)

### Code Quality Validation

- [ ] Follows existing pattern from tests/common/mod.rs
- [ ] Uses absolute paths from project_path
- [ ] No hardcoded relative paths in directory creation
- [ ] UnitTestContext helper methods still work correctly
- [ ] No changes to test logic, only path handling

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable names
- [ ] Comments explain the absolute path usage if not obvious
- [ ] No environment variables or configuration changes needed

---

## Anti-Patterns to Avoid

- Don't change the current directory setting logic - it's correct
- Don't modify individual test files - fix is in setup_unit_test() only
- Don't use conditional logic based on test names - fix the root cause
- Don't add sleep() or timing workarounds - use proper path handling
- Don't skip the failing tests - they should all pass after the fix
- Don't change the test logic - only the path handling in setup

---

## Additional Research Notes

### Test Failure Pattern Analysis

All 25 failing tests exhibit the same failure pattern:

```
thread '<test_name>' panicked at src/test_utils.rs:153:37:
Failed to create .jin directory: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

Or:

```
thread '<test_name>' panicked at src/test_utils.rs:157:20:
Failed to save context: Io(Os { code: 2, kind: NotFound, message: "No such file or directory" })
```

This confirms the issue is in the shared `setup_unit_test()` function, not in individual tests.

### Parallel Execution Issues

Tests run with the `#[serial]` attribute should execute sequentially, but the underlying issue is that `current_dir()` can point to a cleaned-up temporary directory from a previous test run. Using absolute paths from `TempDir::path()` eliminates this dependency on `current_dir()`.

### Reference to Similar Fix

The integration test version in `tests/common/mod.rs` (lines 92-101) already uses the correct pattern with absolute paths. This was likely fixed in a previous task. The unit test version in `src/test_utils.rs` needs the same fix applied.

---

## Confidence Score

**9/10** - This is a straightforward fix with a clear root cause, established pattern to follow, and comprehensive validation approach. The only uncertainty is whether `ProjectContext::save()` properly creates parent directories, but this can be verified during implementation.
