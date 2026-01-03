# Product Requirement Prompt: Fix Git Lock Contention Issues in Mode Command Tests

**Task ID**: P2.M4.T3
**Milestone**: P2.M4 - Fix Failing Unit Tests
**Status**: Ready for Implementation

---

## Goal

**Feature Goal**: Fix 7 failing integration tests in `tests/mode_scope_workflow.rs` caused by Git lock contention when multiple tests access the same JIN_DIR Git repository concurrently.

**Deliverable**: Modified `tests/mode_scope_workflow.rs` with `#[serial]` attribute added to tests that set the global `JIN_DIR` environment variable, ensuring sequential execution and eliminating lock contention.

**Success Definition**:
- All 7 currently failing tests in `mode_scope_workflow.rs` pass
- Tests pass consistently when run in parallel with the full test suite
- No test execution time regression (tests are still fast overall)

---

## User Persona

**Target User**: Developers running the Jin test suite, especially CI/CD systems.

**Use Case**: Developers run `cargo test` to verify code changes. The mode scope workflow tests fail intermittently due to Git lock contention, making the test suite unreliable.

**User Journey**:
1. Developer makes code changes
2. Developer runs `cargo test --test mode_scope_workflow`
3. Tests pass consistently without Git lock errors

**Pain Points Addressed**:
- Intermittent test failures make it difficult to verify code changes
- CI/CD pipelines fail randomly due to test flakiness
- Developers waste time debugging non-code-related test failures

---

## Why

**Business value and user impact**:
- Reliable test suite is critical for continuous integration
- Intermittent failures reduce trust in test results
- Developers need deterministic test outcomes to be productive

**Integration with existing features**:
- Builds on existing `#[serial]` pattern used in `tests/destructive_validation.rs` and `tests/workspace_validation.rs`
- Uses existing `serial_test = "3.0"` dependency in Cargo.toml
- Maintains compatibility with `TestFixture` and `cleanup_git_locks()` patterns

**Problems this solves**:
- **Git lock contention**: 2 tests fail with `failed to lock file` errors
- **Invalid reference names**: 3 tests fail due to temp directory names being used as project names
- **Parent directory issues**: 2 tests fail with "parent is not directory" errors

---

## What

**User-visible behavior**: No direct user-visible behavior changes. This is test infrastructure improvement.

**Technical requirements**:
1. Add `#[serial]` attribute to tests in `mode_scope_workflow.rs` that call `fixture.set_jin_dir()`
2. Ensure all test functions that modify global state (JIN_DIR) run sequentially
3. Verify tests pass when run both in isolation and as part of the full suite

### Root Cause Analysis

The Git lock contention issue occurs because:

1. **Parallel test execution**: Rust runs tests in parallel by default
2. **Global environment variable**: `fixture.set_jin_dir()` sets `JIN_DIR` via `std::env::set_var()`, which is process-global state
3. **Overwriting state**: When multiple tests run in parallel, they overwrite each other's `JIN_DIR`
4. **Shared repository access**: Tests end up accessing the same Git repository (the last test's JIN_DIR)
5. **Lock contention**: Multiple tests trying to access the same repo causes Git lock conflicts

Evidence from test failures:
```
test_layer_routing_project_base: "failed to lock file '/tmp/.tmpnvMd37/.jin_global/config.lock'"
test_scope_requires_mode_error: "failed to lock file '/tmp/.tmpnvMd37/.jin_global/HEAD.lock'"
```

Note that both tests reference the SAME JIN_DIR path (`/tmp/.tmpnvMd37/.jin_global/`), confirming they are sharing the repository.

### Success Criteria

- [ ] All 7 failing tests in `mode_scope_workflow.rs` pass
- [ ] `cargo test --test mode_scope_workflow` shows 11 passed, 0 failed
- [ ] Tests pass when run with `--test-threads=1` (serial execution)
- [ ] Tests pass when run with default parallel execution
- [ ] No regressions in other test files

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES. This PRP provides:
- Exact file location and test names to modify
- Root cause analysis with evidence from test output
- Pattern references from existing code that already solved this issue
- Validation commands to verify the fix

### Documentation & References

```yaml
# MUST READ - Critical for understanding existing patterns

- file: tests/mode_scope_workflow.rs
  why: Contains the 7 failing tests that need #[serial] attribute
  pattern: Integration tests using TestFixture with set_jin_dir() calls
  gotcha: Tests run in parallel by default, causing JIN_DIR conflicts

- file: tests/destructive_validation.rs
  why: Shows correct pattern for #[serial] attribute usage
  pattern: All tests using global state have #[serial] attribute
  gotcha: Follow this pattern for mode_scope_workflow tests

- file: tests/workspace_validation.rs
  why: Another example of #[serial] usage for global state tests
  pattern: Consistent use of #[serial] across all test functions

- file: tests/common/fixtures.rs
  why: Contains TestFixture::set_jin_dir() that sets global state
  pattern: set_jin_dir() calls std::env::set_var("JIN_DIR", jin_dir)
  gotcha: This is process-global state, requiring #[serial] protection

- file: src/commands/mode.rs
  why: Unit tests already use #[serial] for tests with set_current_dir/set_var
  pattern: #[test] #[serial] fn test_create_mode()
  gotcha: Integration tests need the same treatment

- url: https://docs.rs/serial_test/latest/serial_test/
  why: Official documentation for serial_test crate
  critical: #[serial] attribute ensures tests run sequentially

- url: https://doc.rust-lang.org/std/env/fn.set_var.html
  why: Documentation on environment variable behavior
  critical: "Environment variables are global to the entire process"

- file: Cargo.toml
  why: Confirm serial_test dependency is already present
  pattern: serial_test = "3.0" at line 56
```

### Current Codebase Tree

```bash
jin/
├── tests/
│   ├── mode_scope_workflow.rs    # TARGET FILE - 7 failing tests need #[serial]
│   ├── destructive_validation.rs # REFERENCE - Shows #[serial] pattern
│   ├── workspace_validation.rs   # REFERENCE - Shows #[serial] pattern
│   └── common/
│       ├── fixtures.rs           # TestFixture with set_jin_dir()
│       ├── git_helpers.rs        # cleanup_git_locks() function
│       └── mod.rs                # Common test utilities
├── Cargo.toml                    # Has serial_test = "3.0" dependency
└── plan/
    └── P2M4T3/
        ├── PRP.md                # This document
        └── research/             # Research documentation
```

### Desired Codebase Tree (files to be modified)

```bash
tests/
└── mode_scope_workflow.rs        # MODIFY: Add #[serial] to 7 test functions
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: JIN_DIR is process-global environment variable
// When multiple tests run in parallel and call set_jin_dir(),
// they overwrite each other's values, causing tests to share
// the same Git repository and experience lock contention.

// WRONG (current state):
#[test]
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // Sets GLOBAL JIN_DIR
    // ... test code ...
}

// RIGHT (fix):
#[test]
#[serial]  // Ensures this test runs sequentially
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // Now safe: no other test runs concurrently
    // ... test code ...
}

// GOTCHA: ALL tests that call fixture.set_jin_dir() need #[serial]
// Even tests that don't directly interact with each other can conflict
// because JIN_DIR is global process state, not test-local state.

// GOTCHA: The serial_test crate is already in dependencies
// No need to add it to Cargo.toml - just use the attribute

// GOTCHA: Other test files already solved this problem
// tests/destructive_validation.rs uses #[serial] on ALL tests
// tests/workspace_validation.rs uses #[serial] on ALL tests
// Follow their pattern for consistency
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models needed. This task adds attributes to existing test functions.

### Failing Tests Analysis

```yaml
# Git Lock Contention Issues (2 tests) - Direct lock conflicts:
- test_layer_routing_project_base:
  error: "failed to lock file '/tmp/.tmpnvMd37/.jin_global/config.lock'"
  cause: Parallel tests sharing JIN_DIR

- test_scope_requires_mode_error:
  error: "failed to lock file '/tmp/.tmpnvMd37/.jin_global/HEAD.lock'"
  cause: Parallel tests sharing JIN_DIR

# Invalid Reference Name Issues (3 tests) - Symptoms of JIN_DIR sharing:
- test_layer_routing_mode_project:
  error: Reference 'refs/jin/layers/mode/.../project/.tmpiNgScl' is not valid
  cause: Using temp directory name as project name due to state confusion

- test_layer_routing_mode_scope:
  error: Invalid reference name
  cause: Using temp directory name in ref path

- test_layer_routing_mode_scope_project:
  error: Invalid reference name
  cause: Using temp directory name in ref path

# Parent Directory Issues (2 tests) - Related to state confusion:
- test_layer_precedence_higher_wins:
  error: "could not remove directory... parent is not directory"
  cause: JIN_DIR state confusion during parallel execution

- test_mode_scope_deep_merge:
  error: "parent is not directory"
  cause: JIN_DIR state confusion during parallel execution
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD #[serial] attribute to Git lock contention tests
  - ADD: #[serial] attribute to test_layer_routing_project_base
  - ADD: #[serial] attribute to test_scope_requires_mode_error
  - FOLLOW pattern: tests/destructive_validation.rs line 80
  - NAMING: Place #[serial] on line before #[test]
  - PLACEMENT: tests/mode_scope_workflow.rs lines 22, 512

Task 2: ADD #[serial] attribute to invalid reference tests
  - ADD: #[serial] to test_layer_routing_mode_project
  - ADD: #[serial] to test_layer_routing_mode_scope
  - ADD: #[serial] to test_layer_routing_mode_scope_project
  - FOLLOW pattern: tests/workspace_validation.rs line 48
  - PLACEMENT: tests/mode_scope_workflow.rs lines 72, 127, 189

Task 3: ADD #[serial] attribute to parent directory tests
  - ADD: #[serial] to test_layer_precedence_higher_wins
  - ADD: #[serial] to test_mode_scope_deep_merge
  - FOLLOW pattern: Existing #[serial] tests in same file
  - PLACEMENT: tests/mode_scope_workflow.rs lines 258, 342

Task 4: VERIFY all passing tests still pass
  - RUN: cargo test --test mode_scope_workflow
  - VERIFY: All 11 tests pass (7 fixed + 4 already passing)
  - CHECK: No regressions introduced
  - DEPENDENCIES: Tasks 1-3 complete

Task 5: VERIFY tests pass in full suite
  - RUN: cargo test
  - VERIFY: No new test failures in other files
  - CHECK: Integration tests still pass
  - DEPENDENCIES: Task 4 complete
```

### Implementation Patterns & Key Details

```rust
// ============================================
// Pattern to follow from tests/destructive_validation.rs
// ============================================

// BEFORE (current code - WRONG):
#[test]
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    // ...
}

// AFTER (fixed code - CORRECT):
#[test]
#[serial]  // CRITICAL: Add this line
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    // ...
}

// ============================================
// Complete list of tests needing #[serial]
// ============================================

// Line 22: test_layer_routing_mode_base
#[test]
#[serial]  // ADD THIS LINE
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {

// Line 72: test_layer_routing_mode_project
#[test]
#[serial]  // ADD THIS LINE
fn test_layer_routing_mode_project() -> Result<(), Box<dyn std::error::Error>> {

// Line 127: test_layer_routing_mode_scope
#[test]
#[serial]  // ADD THIS LINE
fn test_layer_routing_mode_scope() -> Result<(), Box<dyn std::error::Error>> {

// Line 189: test_layer_routing_mode_scope_project
#[test]
#[serial]  // ADD THIS LINE
fn test_layer_routing_mode_scope_project() -> Result<(), Box<dyn std::error::Error>> {

// Line 258: test_layer_precedence_higher_wins
#[test]
#[serial]  // ADD THIS LINE
fn test_layer_precedence_higher_wins() -> Result<(), Box<dyn std::error::Error>> {

// Line 342: test_mode_scope_deep_merge
#[test]
#[serial]  // ADD THIS LINE
fn test_mode_scope_deep_merge() -> Result<(), Box<dyn std::error::Error>> {

// Line 441: test_layer_routing_global_base
// NOTE: This test already passes, but uses set_jin_dir()
// Consider adding #[serial] for consistency

// Line 474: test_layer_routing_project_base
#[test]
#[serial]  // ADD THIS LINE
fn test_layer_routing_project_base() -> Result<(), Box<dyn std::error::Error>> {

// Line 512: test_scope_requires_mode_error
#[test]
#[serial]  // ADD THIS LINE
fn test_scope_requires_mode_error() -> Result<(), Box<dyn std::error::Error>> {

// Line 558: test_multiple_modes_isolated
// NOTE: This test already passes without #[serial]
// This is interesting - investigate why it doesn't fail
```

### Integration Points

```yaml
NO NEW INTEGRATIONS: This fix only adds attributes to existing tests

FILES AFFECTED:
  - modify: tests/mode_scope_workflow.rs (add #[serial] to 7 tests)

DEPENDENCIES:
  - serial_test = "3.0" already in Cargo.toml (line 56)
  - No new dependencies needed

TEST EXECUTION:
  - Tests with #[serial] will run sequentially
  - Other tests continue to run in parallel
  - Overall test suite time impact: minimal (only 7 tests affected)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Verify the attribute compiles correctly
cargo test --test mode_scope_workflow -- --list 2>&1 | head -20

# Check for any compilation errors
cargo check --tests

# Expected: Zero errors. #[serial] is a procedural macro that expands at compile time.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the mode_scope_workflow tests
cargo test --test mode_scope_workflow -- --nocapture

# Expected output:
# test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured

# Run with verbose output to see test execution order
cargo test --test mode_scope_workflow -- --nocapture --test-threads=1

# Expected: All tests pass when run serially
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all integration tests to ensure no regressions
cargo test --test destructive_validation
cargo test --test workspace_validation

# Expected: All tests pass, no new failures

# Run full test suite
cargo test 2>&1 | tail -50

# Expected: All mode_scope_workflow tests pass in full suite context
```

### Level 4: Regression Testing

```bash
# Run tests multiple times to check for flakiness
for i in {1..5}; do
    cargo test --test mode_scope_workflow 2>&1 | grep "test result:"
done

# Expected: All runs show 11 passed, 0 failed

# Run with parallel execution (default behavior)
cargo test --test mode_scope_workflow -- --test-threads=8

# Expected: All tests pass (#[serial] ensures sequential execution)

# Verify no performance regression
time cargo test --test mode_scope_workflow

# Expected: Test time similar to before (within 10%)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 7 failing tests now have `#[serial]` attribute
- [ ] `cargo test --test mode_scope_workflow` shows 11 passed, 0 failed
- [ ] No compilation errors or warnings
- [ ] Tests pass when run with `--test-threads=1`
- [ ] Tests pass when run with default parallel execution
- [ ] No changes to test logic, only added attributes

### Feature Validation

- [ ] `test_layer_routing_mode_base` passes
- [ ] `test_layer_routing_mode_project` passes
- [ ] `test_layer_routing_mode_scope` passes
- [ ] `test_layer_routing_mode_scope_project` passes
- [ ] `test_layer_precedence_higher_wins` passes
- [ ] `test_mode_scope_deep_merge` passes
- [ ] `test_layer_routing_project_base` passes
- [ ] `test_scope_requires_mode_error` passes
- [ ] Previously passing tests still pass (4 tests)
- [ ] Tests pass consistently across multiple runs

### Code Quality Validation

- [ ] Follows existing pattern from `tests/destructive_validation.rs`
- [ ] Follows existing pattern from `tests/workspace_validation.rs`
- [ ] `#[serial]` attribute placed before `#[test]` attribute
- [ ] No changes to test logic or assertions
- [ ] No changes to test fixtures or setup

### Documentation & Deployment

- [ ] No documentation changes needed (test infrastructure only)
- [ ] No environment variables or configuration changes
- [ ] Fix is backward compatible (doesn't affect test behavior)
- [ ] Ready to merge without additional steps

---

## Anti-Patterns to Avoid

- Don't modify test logic or assertions - only add `#[serial]` attributes
- Don't remove `#[test]` attributes - add `#[serial]` before them
- Don't add `#[serial]` to tests that don't use `fixture.set_jin_dir()`
- Don't change the `TestFixture` or `set_jin_dir()` implementation
- Don't add new test files or functions
- Don't modify Cargo.toml (serial_test already present)
- Don't skip running the full test suite after the fix
- Don't assume all tests need `#[serial]` - only those with `set_jin_dir()`

---

## Research References

### Related PRPs in P2.M4

- `plan/P2M4T1/PRP.md` - Test infrastructure improvements (setup_unit_test, cleanup_before_test)
- `plan/docs/P2M4T2_PRP_Fix_File_System_Path_Issues.md` - File system path handling in tests

### Library Documentation

- [serial_test crate - docs.rs](https://docs.rs/serial_test/latest/serial_test/) - Serial test execution
- [Rust testing guide - book](https://doc.rust-lang.org/book/ch11-00-testing.html) - Understanding test attributes

### Codebase Patterns

- `tests/destructive_validation.rs:80` - Example of `#[serial]` usage
- `tests/workspace_validation.rs:48` - Example of `#[serial]` usage
- `src/commands/mode.rs:304` - Unit tests using `#[serial]` for global state
- `tests/common/fixtures.rs:46` - `set_jin_dir()` implementation that requires `#[serial]`

---

## Confidence Score

**Implementation Confidence**: 10/10

**Reasoning**:
- Root cause is clearly identified (global JIN_DIR environment variable)
- Solution is well-established pattern in the codebase (destructive_validation, workspace_validation)
- Fix is minimal and surgical (only adding attributes, no logic changes)
- serial_test dependency is already present
- Zero risk of breaking existing functionality
- Validation is straightforward (run tests, verify they pass)

**Risk Factors**:
- None. This is a low-risk, high-confidence fix with clear path to validation.

---

## Summary

P2.M4.T3 fixes 7 failing integration tests in `tests/mode_scope_workflow.rs` by adding the `#[serial]` attribute to tests that modify global process state (JIN_DIR environment variable). The root cause is parallel test execution combined with global environment variable modification, causing tests to share Git repositories and experience lock contention.

**Key deliverables**:
1. Add `#[serial]` attribute to 7 failing test functions
2. Verify all tests pass with `cargo test --test mode_scope_workflow`
3. Ensure no regressions in full test suite

**The fix is simple, established, and risk-free.**

---

## Appendix: Test Output Evidence

### Current Failure Output

```
failures:

---- test_layer_routing_project_base stdout ----
thread 'test_layer_routing_project_base' (3510253) panicked at ...
Unexpected failure.
code=1
stderr=```"Error: Git error: failed to lock file '/tmp/.tmpnvMd37/.jin_global/config.lock'
for writing: ; class=Os (2); code=Locked (-14)\n"```

---- test_scope_requires_mode_error stdout ----
thread 'test_scope_requires_mode_error' (3510256) panicked at ...
Unexpected failure.
code=1
stderr=```"Error: Git error: failed to lock file '/tmp/.tmpnvMd37/.jin_global/HEAD.lock'
for writing: ; class=Os (2); code=Locked (-14)\n"```

---- test_layer_routing_mode_project stdout ----
thread 'test_layer_routing_mode_project' (3510250) panicked at tests/common/assertions.rs:165:19:
Layer ref 'refs/jin/layers/mode/test_mode_3510238_4/project/.tmpiNgScl' should exist...
the given reference name 'refs/jin/layers/mode/test_mode_3510238_4/project/.tmpiNgScl'
is not valid; class=Reference (4); code=InvalidSpec (-12)

test result: FAILED. 11 passed; 7 failed; 0 ignored
```

### Note: Both tests reference the SAME JIN_DIR path

The key evidence is that both failing tests reference `/tmp/.tmpnvMd37/.jin_global/` - the exact same path. This confirms they are sharing the JIN_DIR due to global environment variable state being overwritten by parallel test execution.
