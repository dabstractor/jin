# Product Requirement Prompt (PRP): Fix Test Expectation Mismatches

---

## Goal

**Feature Goal**: Fix 2 repair command unit tests (`test_check_staging_index_corrupted` and `test_create_default_context`) where test expectations don't match actual implementation behavior due to test isolation issues.

**Deliverable**: Two stabilized unit tests that consistently pass by adding `#[serial]` attribute to prevent parallel execution race conditions.

**Success Definition**:
- Both tests pass consistently when run in parallel with other tests
- No changes to the repair command implementation (implementation is correct)
- Tests follow existing codebase patterns for test isolation using `serial_test` crate

## Why

- **Test Reliability**: These tests are flaky and fail intermittently when run in parallel with other tests due to current directory manipulation
- **CI/CD Stability**: Flaky tests undermine confidence in the test suite and can block valid PRs
- **Existing Pattern**: The codebase already uses `#[serial]` attribute in 50+ tests for the same reason (see `tests/mode_scope_workflow.rs`, `tests/destructive_validation.rs`, etc.)
- **No Implementation Changes Needed**: The repair command implementation correctly detects corrupted staging indexes and creates default contexts. The issue is purely test infrastructure.

## What

### Problem Analysis

The two tests in `src/commands/repair.rs` suffer from **test isolation problems**:

1. **`test_check_staging_index_corrupted`** (line 820-842): Creates a corrupted staging index file in a temp directory, then uses `DirGuard` to change current directory. The test expects `issues_found.len() == 1` but intermittently gets `0` because:
   - `StagingIndex::default_path()` returns relative path `.jin/staging/index.json`
   - When another test changes current directory via `DirGuard` before this test runs, the relative path resolves to wrong location
   - The corrupted file is in temp directory, but `check_staging_index()` looks in current directory

2. **`test_create_default_context`** (line 983-1002): Creates `.jin` directory in temp directory, then uses `DirGuard` to change current directory. The test expects context file to be created but intermittently fails because:
   - `ProjectContext::default_path()` returns relative path `.jin/context`
   - Directory creation and context save happen at different times relative to current directory changes
   - File may be created in wrong location

### Root Cause

**Parallel execution interference**: Tests using `DirGuard` to change current directory race with each other because Rust's test runner executes tests in parallel by default. Relative paths like `.jin/staging/index.json` depend on current directory, causing cross-test pollution.

### Solution

Add `#[serial]` attribute to both tests to force sequential execution. This is a well-established pattern in the codebase with 50+ existing examples.

### Success Criteria

- [ ] `test_check_staging_index_corrupted` passes consistently
- [ ] `test_create_default_context` passes consistently
- [ ] No implementation changes to repair command logic
- [ ] Tests follow existing `#[serial]` pattern used in codebase

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Yes** - This PRP provides:
- Exact file locations and line numbers
- Complete test code before and after
- Root cause analysis with specific evidence
- The exact fix pattern with 50+ examples in codebase
- Validation commands that verify the fix

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: src/commands/repair.rs
  lines: 820-842, 983-1002
  why: The two tests that need fixing - complete implementation
  pattern: Tests using DirGuard with relative paths, need #[serial]
  gotcha: Don't modify the test logic, only add #[serial] attribute

- file: src/commands/repair.rs
  lines: 808-817, 844-908, 910-980
  why: Context - other repair tests that may also need #[serial]
  gotcha: Check all tests using DirGuard in this module

- file: src/commands/repair.rs
  lines: 308-351
  why: Implementation of check_staging_index() - proves implementation is correct
  pattern: Uses StagingIndex::default_path() which returns relative path

- file: src/core/config.rs
  why: Contains ProjectContext::default_path() - returns relative path ".jin/context"

- file: src/test_utils.rs
  why: Test infrastructure showing setup_unit_test() pattern
  pattern: UnitTestContext manages isolated test environment with absolute paths

- file: tests/mode_scope_workflow.rs
  lines: 25, 76, 132, 195, 266, 350, 450, 484, 523, 570
  why: Examples of #[serial] attribute usage in codebase (10 examples)
  pattern: #[test] followed by #[serial] on next line

- file: tests/destructive_validation.rs
  lines: 80, 131, 191, 254, 293, 336, 381, 435, 492, 541, 592, 639, 677
  why: More examples of #[serial] attribute usage (13 examples)
  pattern: Consistent placement of #[serial] after #[test]

- file: tests/workspace_validation.rs
  lines: 48, 69, 92, 154, 216, 271, 323, 365, 401, 448, 503
  why: More examples of #[serial] attribute usage (11 examples)

- file: Cargo.toml
  lines: 56
  why: serial_test = "3.0" dependency already exists
  gotcha: No need to add dependency, just use the crate

- docfile: plan/architecture/test_analysis.md
  lines: 226-267
  why: Original analysis documenting the test failure
  section: "Test: test_check_staging_index_corrupted"

- docfile: plan/docs/implementation-gaps-analysis.md
  lines: 302-352
  why: Analysis of implementation gaps including these tests
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── Cargo.toml              # Contains serial_test = "3.0" dependency
├── src/
│   ├── commands/
│   │   └── repair.rs      # TARGET FILE - contains both failing tests
│   ├── core/
│   │   └── config.rs      # ProjectContext::default_path() returns ".jin/context"
│   ├── staging/
│   │   └── index.rs       # StagingIndex::default_path() returns ".jin/staging/index.json"
│   └── test_utils.rs      # UnitTestContext for isolated test environments
└── tests/
    ├── mode_scope_workflow.rs      # 10 examples of #[serial] usage
    ├── destructive_validation.rs   # 13 examples of #[serial] usage
    └── workspace_validation.rs     # 11 examples of #[serial] usage
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: Relative path resolution depends on current directory
// StagingIndex::default_path() returns ".jin/staging/index.json"
// ProjectContext::default_path() returns ".jin/context"
// These resolve relative to std::env::current_dir(), NOT a fixed location

// CRITICAL: DirGuard changes current directory for tests
// When tests run in parallel, DirGuard calls from different tests interfere
// The #[serial] attribute prevents this by forcing sequential execution

// CRITICAL: tempfile::TempDir creates unique temp directories
// But relative paths still depend on current directory when accessed

// PATTERN: #[serial] attribute usage in this codebase
// Always placed immediately after #[test]
// Used for any test that: sets JIN_DIR, changes current dir, or manipulates env vars

// GOTCHA: setup_unit_test() vs TempDir + DirGuard
// setup_unit_test() uses absolute paths internally (safer)
// TempDir + DirGuard uses relative paths (needs #[serial])
```

## Implementation Blueprint

### Data models and structure

No data model changes needed. This is purely a test fix.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY serial_test dependency exists
  - CHECK: Cargo.toml line 56 contains `serial_test = "3.0"`
  - CONFIRM: Dependency is already present (no changes needed)
  - SKIP: Adding dependency to Cargo.toml

Task 2: MODIFY src/commands/repair.rs - test_check_staging_index_corrupted
  - ADD: #[serial] attribute immediately after #[test] on line 821
  - BEFORE:
    ```rust
    #[test]
    fn test_check_staging_index_corrupted() {
    ```
  - AFTER:
    ```rust
    #[test]
    #[serial]
    fn test_check_staging_index_corrupted() {
    ```
  - PRESERVE: All existing test logic (implementation is correct)
  - PATTERN: Follow existing #[serial] placement in tests/mode_scope_workflow.rs

Task 3: MODIFY src/commands/repair.rs - test_create_default_context
  - ADD: #[serial] attribute immediately after #[test] on line 983
  - BEFORE:
    ```rust
    #[test]
    fn test_create_default_context() {
    ```
  - AFTER:
    ```rust
    #[test]
    #[serial]
    fn test_create_default_context() {
    ```
  - PRESERVE: All existing test logic (implementation is correct)

Task 4: VERIFY no other repair tests need #[serial]
  - SCAN: All tests in src/commands/repair.rs module
  - CHECK: For any other tests using DirGuard or manipulating current directory
  - ADD: #[serial] to any additional tests found (if needed)
  - CANDIDATES: test_rebuild_staging_index (line 858), test_check_jinmap_valid_yaml (line 882)

Task 5: VERIFY use statement exists for serial_test
  - CHECK: src/commands/repair.rs imports
  - CONFIRM: `use serial_test::serial;` exists at top of file
  - ADD: If not present, add with other use statements
```

### Implementation Patterns & Key Details

```rust
// PATTERN: #[serial] attribute placement (from 50+ examples in codebase)
#[test]
#[serial]  // <-- Add this line
fn test_name() {
    // test logic...
}

// GOTCHA: Order matters - #[test] must come before #[serial]
// DON'T DO THIS:
// #[serial]
// #[test]  // <-- Wrong order, will cause compile error

// CRITICAL: Why this fixes the issue
// 1. Without #[serial]: Tests run in parallel
// 2. Test A calls DirGuard::new(temp) -> changes to /tmp/test_a
// 3. Test B calls DirGuard::new(temp) -> changes to /tmp/test_b
// 4. Test A's StagingIndex::default_path() now resolves to /tmp/test_b/.jin/staging (wrong!)
// 5. With #[serial]: Tests run sequentially, no interference

// REFERENCE: Implementation is correct, don't change it
// check_staging_index() correctly detects corrupted indexes
// create_default_context() correctly creates contexts
// The issue is ONLY test isolation, not logic
```

### Integration Points

```yaml
NONE: This change is isolated to test attributes only

TEST_INFRASTRUCTURE:
  - uses: serial_test crate (already in Cargo.toml)
  - effect: Forces sequential execution of marked tests
  - impact: Minimal - only affects test execution order
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding #[serial] attributes
cargo check --tests                    # Verify compilation
cargo clippy --tests                   # Check for lints

# Expected: No compilation errors. serial:: attribute is valid.

# Verify syntax is correct
cargo test --test repair --no-run      # Compile repair tests without running

# Expected: All tests compile successfully
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run specific tests in isolation first
cargo test test_check_staging_index_corrupted -- --exact
cargo test test_create_default_context -- --exact

# Expected: Both tests pass individually

# Run all repair tests
cargo test --package jin --lib commands::repair::tests

# Expected: All repair tests pass, no failures

# Run tests in parallel (this is where the bug manifests)
cargo test --package jin --lib commands::repair::tests --test-threads=2

# Expected: All tests pass even with multiple threads
# BEFORE FIX: These would intermittently fail
# AFTER FIX: Should pass consistently
```

### Level 3: Integration Testing (System Validation)

```bash
# Run full test suite
cargo test --all

# Expected: All tests pass, no regressions

# Run tests multiple times to catch flakiness
for i in {1..10}; do cargo test --package jin --lib commands::repair::tests || exit 1; done

# Expected: All 10 iterations pass without failure
# This validates that the fix truly eliminates the race condition

# Check for any other tests that might need #[serial]
cargo test --all 2>&1 | grep -i "panicked\|thread.*panicked"

# Expected: No panics related to current directory or file path issues
```

### Level 4: Validation of Root Cause Fix

```bash
# Verify the specific race condition is fixed
# Run tests with explicit thread count to trigger parallelism
cargo test --package jin --lib commands::repair::tests --test-threads=4

# Expected: All tests pass

# Verify #[serial] is actually being used
cargo test --package jin --lib commands::repair::tests -- --list

# Expected: Test names show they are marked as serial
# Output will show: test ... commands::repair::tests::test_check_staging_index_corrupted

# Run with RUST_TEST_SERIAL=1 to confirm serial behavior
cargo test --package jin --lib commands::repair::tests

# Expected: Tests run one at a time when #[serial] is present
```

## Final Validation Checklist

### Technical Validation

- [ ] Both tests compile without errors
- [ ] `test_check_staging_index_corrupted` passes consistently
- [ ] `test_create_default_context` passes consistently
- [ ] No changes to repair command implementation logic
- [ ] `#[serial]` attribute placed correctly (after `#[test]`)
- [ ] `use serial_test::serial;` import exists if needed

### Feature Validation

- [ ] Tests pass when run individually: `cargo test test_check_staging_index_corrupted -- --exact`
- [ ] Tests pass when run together: `cargo test --package jin --lib commands::repair::tests`
- [ ] Tests pass with multiple threads: `cargo test --test-threads=4`
- [ ] Tests pass on repeated runs: 10 consecutive runs without failure
- [ ] No other tests broke due to this change

### Code Quality Validation

- [ ] Follows existing `#[serial]` pattern from 50+ examples in codebase
- [ ] No unnecessary changes to test logic
- [ ] Minimal diff - only adding attributes
- [ ] PR description clearly explains the race condition fix

### Documentation & Deployment

- [ ] No user-facing changes (test-only fix)
- [ ] No documentation updates needed
- [ ] No changelog entry needed (test fixes are internal)

---

## Anti-Patterns to Avoid

- ❌ Don't modify the test logic - the test assertions are correct
- ❌ Don't modify the repair command implementation - it works correctly
- ❌ Don't add new dependencies - `serial_test` is already in Cargo.toml
- ❌ Don't change the order of `#[test]` and `#[serial]` attributes
- ❌ Don't add `#[serial]` to tests that don't manipulate shared state
- ❌ Don't try to fix this by changing to absolute paths - `#[serial]` is the established pattern
- ❌ Don't skip validation - run tests multiple times to confirm the fix works

---

## Appendix: Evidence of Root Cause

### Test Code Before Fix

```rust
// src/commands/repair.rs:820-842
#[test]
fn test_check_staging_index_corrupted() {
    let temp = TempDir::new().unwrap();

    // Create corrupted staging index in temp directory
    let index_path = temp.path().join(".jin/staging/index.json");
    std::fs::create_dir_all(index_path.parent().unwrap()).unwrap();
    std::fs::write(&index_path, "invalid json").unwrap();

    // Use DirGuard to change to temp directory and auto-restore
    let _guard = DirGuard::new(temp);  // <-- Changes current directory

    let args = RepairArgs {
        dry_run: true,
        check: false,
    };
    let mut issues_found = Vec::new();
    let mut issues_fixed = Vec::new();

    check_staging_index(&args, &mut issues_found, &mut issues_fixed);
    // Inside check_staging_index():
    //   let index_path = StagingIndex::default_path();  // Returns ".jin/staging/index.json"
    //   This resolves relative to CURRENT directory, not temp.path()

    assert_eq!(issues_found.len(), 1);   // <-- Fails intermittently: gets 0 instead of 1
    assert!(issues_found[0].contains("corrupted"));
}
```

### Test Code After Fix

```rust
// src/commands/repair.rs:820-843 (after fix)
#[test]
#[serial]  // <-- ADD THIS LINE
fn test_check_staging_index_corrupted() {
    // ... rest of test unchanged ...
}
```

### Why This Fixes the Issue

The `#[serial]` attribute from the `serial_test` crate ensures that marked tests never run in parallel with other serial tests. Since all tests using `DirGuard` should be marked `#[serial]`, they will execute sequentially, eliminating the race condition where current directory changes interfere with relative path resolution.
