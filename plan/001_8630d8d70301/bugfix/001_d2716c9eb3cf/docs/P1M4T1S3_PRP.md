# PRP: Refactor test_create_mode_bound_scope to use Isolation Helpers

---

## Goal

**Feature Goal**: Refactor the `test_create_mode_bound_scope` test function to use proper isolation helpers (`setup_unit_test()`, `cleanup_test_mode()`, `create_test_mode_in_context()`) instead of the legacy `setup_test_env()` function, eliminating the need for the `#[serial]` attribute and enabling true parallel test execution.

**Deliverable**: A refactored test function in `src/commands/scope.rs` that uses `UnitTestContext` for complete isolation, can run in parallel with other tests, and properly verifies scope creation under a mode.

**Success Definition**:
- Test passes when run alone: `cargo test test_create_mode_bound_scope`
- Test passes when run with all tests: `cargo test`
- Test can run in parallel without `#[serial]` attribute
- Test uses `setup_unit_test()` for environment setup
- Test uses `create_test_mode_in_context()` for mode creation
- Test uses `cleanup_test_mode()` for cleanup (optional with proper isolation)
- Test uses absolute paths from `ctx.jin_dir` instead of relying on `JIN_DIR` environment variable

## User Persona (if applicable)

**Target User**: Developer working on the Jin CLI codebase, specifically maintaining and improving the test suite for the scope subcommand functionality.

**Use Case**: Ensuring that tests for mode-bound scope creation are properly isolated and can run in parallel without interference from other tests.

**User Journey**:
1. Developer runs `cargo test` to verify all tests pass
2. The refactored `test_create_mode_bound_scope` executes in parallel with other tests
3. Test creates isolated temporary environment, creates a test mode, creates a scope bound to that mode, and verifies the ref was created
4. Test automatically cleans up all artifacts via `Drop` trait
5. All tests pass without false failures from shared state

**Pain Points Addressed**:
- Current test uses `#[serial]` attribute which forces sequential execution, slowing down the test suite
- Current test uses `setup_test_env()` which modifies global `JIN_DIR` environment variable and current directory, causing cross-test interference
- Test may fail when run with other tests due to shared global state pollution

## Why

- **Test Reliability**: Tests that share global state (`JIN_DIR`, current directory, Git refs) are flaky and fail unpredictably when run in parallel
- **Test Performance**: The `#[serial]` attribute prevents parallel execution, increasing total test suite runtime
- **Code Consistency**: Other tests in the codebase already use `UnitTestContext` pattern for isolation; this refactor brings this test in line with established best practices
- **Maintainability**: Properly isolated tests are easier to debug, modify, and extend without introducing cross-test interference

## What

Refactor the `test_create_mode_bound_scope` test function in `src/commands/scope.rs` to:

1. Replace `setup_test_env()` with `setup_unit_test()` to get `UnitTestContext`
2. Replace `create_test_mode()` with `create_test_mode_in_context("testmode", &ctx)` to create mode in isolated environment
3. Optionally add `cleanup_test_mode("testmode", &ctx)` before test execution to ensure clean state
4. Use `JinRepo::open_or_create_at(&ctx.jin_dir)` instead of `JinRepo::open_or_create()` to use explicit path
5. Remove `#[serial]` attribute once confirmed working
6. Verify the ref exists using the isolated repository

### Success Criteria

- [ ] Test uses `setup_unit_test()` instead of `setup_test_env()`
- [ ] Test uses `create_test_mode_in_context()` with context parameter
- [ ] Test uses `JinRepo::open_or_create_at(&ctx.jin_dir)` for explicit path isolation
- [ ] Test no longer has `#[serial]` attribute
- [ ] Test passes when run alone: `cargo test test_create_mode_bound_scope`
- [ ] Test passes when run with all tests: `cargo test`
- [ ] Test can run in parallel with other tests without interference
- [ ] No changes to test logic/behavior - only isolation mechanism changes

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

Yes. This PRP provides:
- Exact file locations of the test to refactor and helper functions to use
- Full code examples of current implementation and target implementation
- Specific patterns to follow from existing isolated tests
- Validation commands to verify success
- Known gotchas and anti-patterns to avoid

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://doc.rust-lang.org/book/ch11-03-test-organization.html
  why: Understanding Rust test organization and isolation patterns
  critical: Each test should be independent and not rely on shared state

- url: https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html
  why: TempDir automatically deletes when dropped - core of isolation pattern
  critical: Must keep TempDir in scope for entire test duration to prevent premature cleanup

- url: https://docs.rs/serial_test/latest/serial_test/
  why: Understanding what #[serial] does and why we want to remove it
  critical: #[serial] forces sequential execution - we want parallel execution after proper isolation

- file: /home/dustin/projects/jin/src/commands/scope.rs
  why: Contains the test to refactor and the helper functions to use
  pattern: Look at test_cleanup_removes_all_refs (lines ~546-562) for the target pattern
  gotcha: The test_create_mode_bound_scope function is around line 476-486 (verify current line numbers)

- file: /home/dustin/projects/jin/src/test_utils.rs
  why: Contains setup_unit_test() function and UnitTestContext definition
  pattern: Lines 124-186 contain setup_unit_test() implementation
  gotcha: UnitTestContext implements Drop - automatically restores directory and environment on scope exit

- file: /home/dustin/projects/jin/plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/test_infrastructure_analysis.md
  why: Documents the isolation issues and recommended UnitTestContext pattern
  section: Sections on test_create_mode_bound_scope and isolation patterns
  critical: Explains why JIN_DIR environment variable and current directory changes cause flakiness

- file: /home/dustin/projects/jin/plan/001_8630d8d70301/docs/rust_test_isolation_best_practices.md
  why: Comprehensive Rust testing isolation patterns and best practices
  section: TempDir patterns, RAII cleanup, Git lock cleanup
  critical: Store TempDir in test structures to prevent premature cleanup

- docfile: /home/dustin/projects/jin/plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/P1M4T1S3/research/
  why: All research documents for this specific work item
  section: See all .md files in this directory for detailed research findings
```

### Current Codebase tree (relevant portions)

```bash
src/
├── commands/
│   ├── scope.rs              # Contains test_create_mode_bound_scope (target test to refactor)
│   │                         # Also contains create_test_mode_in_context() and cleanup_test_mode()
│   └── mode.rs               # Related mode command tests for reference
├── test_utils.rs             # Contains setup_unit_test() and UnitTestContext
├── git.rs                    # JinRepo with open_or_create_at() method
└── lib.rs                    # Main library exports

tests/
├── common/
│   ├── fixtures.rs           # TestFixture pattern for integration tests
│   └── git_helpers.rs        # cleanup_git_locks() function
├── mode_scope_workflow.rs    # Integration tests for mode/scope workflows
└── cli_basic.rs              # Basic CLI tests
```

### Desired Codebase tree with changes

```bash
# No new files - this is a refactor of existing test
# Changes to src/commands/scope.rs:
#   - test_create_mode_bound_scope is refactored to use UnitTestContext
#   - #[serial] attribute is removed
#   - setup_test_env() replaced with setup_unit_test()
#   - create_test_mode() replaced with create_test_mode_in_context()
#   - JinRepo::open_or_create() replaced with JinRepo::open_or_create_at(&ctx.jin_dir)
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: TempDir cleanup timing
// The TempDir in UnitTestContext is named _temp_dir (underscore prefix)
// This keeps it in scope for the entire test duration
// If it goes out of scope early, the directory gets deleted and tests fail

// CRITICAL: JIN_DIR environment variable vs explicit paths
// Legacy pattern: set JIN_DIR environment variable, call JinRepo::open_or_create()
// New pattern: use ctx.jin_dir absolute path, call JinRepo::open_or_create_at(&ctx.jin_dir)
// Using explicit paths is more reliable for parallel test execution

// CRITICAL: Ref path format for mode-bound scopes
// Mode ref: refs/jin/modes/{name}/_mode (note the _mode suffix)
// Scope ref: refs/jin/modes/{name}/scopes/{scope} (no underscore prefix on scope)
// The current test assertion uses: refs/jin/modes/testmode/scopes/testscope
// This assertion is CORRECT and should be preserved in the refactor

// CRITICAL: create_test_mode_in_context signature
// fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext)
// Must pass &ctx (borrowed reference) not ctx (owned)

// CRITICAL: JinRepo::open_or_create_at signature
// fn open_or_create_at(path: &Path) -> Result<JinRepo>
// Must pass &ctx.jin_dir (borrowed reference) not ctx.jin_dir (owned)

// CRITICAL: UnitTestContext field visibility
// pub project_path: PathBuf  (absolute path to test project directory)
// pub jin_dir: PathBuf       (absolute path to isolated JIN_DIR)
// Private fields: _temp_dir, _original_dir, _original_jin_dir

// CRITICAL: Drop trait implementation
// UnitTestContext implements Drop to automatically restore:
// - Original current directory
// - Original JIN_DIR environment variable
// - Delete temporary directory
// No manual cleanup needed - just let ctx go out of scope

// GOTCHA: Serial test attribute removal
// The #[serial] attribute from serial_test crate forces sequential execution
// After proper isolation with UnitTestContext, this can be removed
// Remove ONLY after confirming test passes in isolation first
```

## Implementation Blueprint

### Data models and structure

No new data models - this is a test refactor using existing structures:

```rust
// Existing structure from src/test_utils.rs
pub struct UnitTestContext {
    /// Temporary directory (must be kept in scope)
    _temp_dir: TempDir,
    /// Original directory (for restoration on drop)
    _original_dir: Option<PathBuf>,
    /// Original JIN_DIR value (for restoration on drop)
    _original_jin_dir: Option<String>,
    /// Absolute path to test project directory
    pub project_path: PathBuf,
    /// Absolute path to isolated JIN_DIR
    pub jin_dir: PathBuf,
}

// Existing function from src/test_utils.rs
pub fn setup_unit_test() -> UnitTestContext

// Existing helper from src/commands/scope.rs
fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext)

// Existing helper from src/commands/scope.rs
fn cleanup_test_mode(name: &str, ctx: &UnitTestContext)
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: READ the current test implementation
  - OPEN: src/commands/scope.rs
  - LOCATE: test_create_mode_bound_scope function (around line 476-486)
  - UNDERSTAND: Current test structure, what it tests, how it verifies success
  - VERIFY: The ref path assertion format (refs/jin/modes/testmode/scopes/testscope)

Task 2: READ the helper functions to understand their exact signatures
  - OPEN: src/commands/scope.rs tests module
  - FIND: create_test_mode_in_context() function definition
  - FIND: cleanup_test_mode() function definition
  - NOTE: Exact parameter types (borrowed vs owned)
  - OPEN: src/test_utils.rs
  - FIND: setup_unit_test() function definition
  - FIND: UnitTestContext struct definition

Task 3: READ an example test that already uses UnitTestContext
  - OPEN: src/commands/scope.rs tests module
  - FIND: test_cleanup_removes_all_refs function (lines ~546-562)
  - STUDY: The pattern of setup -> create -> cleanup pattern
  - NOTE: How ctx is passed as borrowed reference (&ctx)
  - NOTE: How JinRepo::open_or_create_at(&ctx.jin_dir) is used

Task 4: MODIFY test_create_mode_bound_scope function
  - REPLACE: let _temp = setup_test_env(); with let ctx = setup_unit_test();
  - REPLACE: create_test_mode("testmode"); with create_test_mode_in_context("testmode", &ctx);
  - ADD: cleanup_test_mode("testmode", &ctx); at start (optional, ensures clean state)
  - REPLACE: let repo = JinRepo::open_or_create().unwrap();
  - WITH: let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
  - PRESERVE: The assertion assert!(repo.ref_exists("refs/jin/modes/testmode/scopes/testscope"));
  - REMOVE: #[serial] attribute (or comment out initially, remove after confirmed working)

Task 5: VERIFY test compiles
  - RUN: cargo build --tests
  - CHECK: No compilation errors related to the refactor
  - VERIFY: All imports are present (setup_unit_test, UnitTestContext already imported)

Task 6: RUN test in isolation to verify it works
  - RUN: cargo test test_create_mode_bound_scope
  - VERIFY: Test passes
  - DEBUG: If fails, check that ctx is properly passed as borrowed reference

Task 7: REMOVE #[serial] attribute (if not already removed)
  - EDIT: Remove #[serial] attribute from test function
  - VERIFY: Test still compiles and passes

Task 8: RUN test in parallel with other tests to verify isolation
  - RUN: cargo test (full test suite)
  - VERIFY: test_create_mode_bound_scope passes
  - VERIFY: No other tests fail due to this change
  - RUN: cargo test --test-threads=8 (explicit parallel execution)
  - VERIFY: All tests pass in parallel

Task 9: VERIFY ref path assertion is correct
  - CONFIRM: The ref path "refs/jin/modes/testmode/scopes/testscope" is correct
  - NOTE: Mode ref is at refs/jin/modes/testmode/_mode (with _mode suffix)
  - NOTE: Scope ref is at refs/jin/modes/testmode/scopes/testscope (no underscore)
```

### Implementation Patterns & Key Details

```rust
// CURRENT IMPLEMENTATION (before refactor):
#[test]
#[serial]
fn test_create_mode_bound_scope() {
    let _temp = setup_test_env();           // Uses global JIN_DIR, changes current dir
    create_test_mode("testmode");           // Creates mode in default location

    let result = create("testscope", Some("testmode"));
    assert!(result.is_ok());

    // Verify ref was created
    let repo = JinRepo::open_or_create().unwrap();  // Uses JIN_DIR environment variable
    assert!(repo.ref_exists("refs/jin/modes/testmode/scopes/testscope"));
}

// TARGET IMPLEMENTATION (after refactor):
#[test]
fn test_create_mode_bound_scope() {
    // PATTERN: Use setup_unit_test() to get isolated context
    let ctx = setup_unit_test();

    // PATTERN: Optional cleanup at start ensures clean state
    // (Not strictly necessary with TempDir, but good defensive programming)
    cleanup_test_mode("testmode", &ctx);

    // PATTERN: Create mode in isolated context with explicit path
    create_test_mode_in_context("testmode", &ctx);

    // EXECUTE: The function under test
    let result = create("testscope", Some("testmode"));
    assert!(result.is_ok());

    // VERIFY: Use explicit path for repository access
    let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
    assert!(repo.ref_exists("refs/jin/modes/testmode/scopes/testscope"));

    // CLEANUP: Automatic via Drop when ctx goes out of scope
    // - Temporary directory deleted
    // - Original directory restored
    // - Original JIN_DIR environment variable restored
}

// EXAMPLE FROM CODEBASE (test_cleanup_removes_all_refs):
#[test]
fn test_cleanup_removes_all_refs() {
    let ctx = setup_unit_test();

    // Create a mode
    create_test_mode_in_context("cleanup_test", &ctx);

    // Verify refs exist
    let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
    assert!(repo.ref_exists("refs/jin/modes/cleanup_test/_mode"));

    // Run cleanup
    cleanup_test_mode("cleanup_test", &ctx);

    // Verify refs are gone
    assert!(!repo.ref_exists("refs/jin/modes/cleanup_test/_mode"));
}
```

### Integration Points

```yaml
TEST_DEPS:
  - modify: src/commands/scope.rs
  - pattern: "Change test_create_mode_bound_scope to use UnitTestContext"
  - preserve: "All other tests in the file remain unchanged"

IMPORTS:
  - verify: "use crate::test_utils::{setup_unit_test, UnitTestContext};" is present
  - verify: "use serial_test::serial;" can be removed if no other tests use it

FUNCTION_SIGNATURES:
  - setup_unit_test() -> UnitTestContext (no parameters, returns context)
  - create_test_mode_in_context(name: &str, ctx: &UnitTestContext) (borrowed ctx)
  - cleanup_test_mode(name: &str, ctx: &UnitTestContext) (borrowed ctx)
  - JinRepo::open_or_create_at(path: &Path) -> Result<JinRepo> (borrowed path)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after making code changes - fix before proceeding
cargo build --tests                    # Compile tests to check for syntax errors
cargo clippy --tests                   # Lint checks for test code

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
# Common errors:
# - "cannot find value `ctx` in this scope" -> You removed ctx variable, add it back
# - "expected struct `UnitTestContext`, found `TempDir`" -> You're using old pattern
# - "borrowed value does not live long enough" -> Check you're passing &ctx not ctx
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the refactored function in isolation
cargo test test_create_mode_bound_scope

# Expected output:
# Running target/debug/deps/jin-[hash]
# Running unittests src/commands/scope.rs (src/commands/scope.rs)
# Running tests/test_create_mode_bound_scope (target/debug/jin-[hash])
# test src::commands::scope::tests::test_create_mode_bound_scope ... ok
#
# test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

# If test fails, debug:
# - Check that ctx.jin_dir path is correct
# - Check that create_test_mode_in_context is called with &ctx
# - Check that JinRepo::open_or_create_at is called with &ctx.jin_dir
# - Verify the ref path string is correct

# Run all tests in the scope module to ensure no regression
cargo test --package jin --lib src::commands::scope::tests

# Expected: All tests in scope module pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Run full test suite to verify isolation works
cargo test

# Expected: All tests pass, including test_create_mode_bound_scope
# Key success indicator: No "test failed due to shared state" errors

# Run with explicit parallel execution to stress-test isolation
cargo test --test-threads=8

# Expected: All tests pass in parallel
# Success means: No cross-test interference from global state

# Run the test multiple times to check for flakiness
for i in {1..5}; do cargo test test_create_mode_bound_scope || exit 1; done

# Expected: All 5 runs pass without failure

# Specific verification: Test runs without #[serial] attribute
# Before refactor: Test has #[serial] attribute
# After refactor: Test has NO #[serial] attribute
# Verify: grep -A 1 "fn test_create_mode_bound_scope" src/commands/scope.rs
# Should show: "fn test_create_mode_bound_scope() {" without #[serial] above it
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Parallel execution stress test
cargo test -- --test-threads=16 --nocapture

# Verify no test interference by running tests in random order
cargo test -- --test-threads=1  # Sequential baseline
cargo test -- --test-threads=8  # Parallel execution

# Compare results - should be identical

# Verify no filesystem artifacts left behind
ls -la /tmp/ | grep jin || echo "No artifacts in /tmp"

# Verify test creates proper refs
# (This requires running the test with debug output or adding assertions)

# Performance validation: Ensure removing #[serial] actually improves performance
time cargo test -- --test-threads=1  # With serial execution
time cargo test -- --test-threads=8  # With parallel execution

# Expected: Parallel execution should be faster (lower elapsed time)

# Git lock verification - no locks should remain
find . -name "*.lock" -type f 2>/dev/null | grep -v target || echo "No git locks found"
```

## Final Validation Checklist

### Technical Validation

- [ ] Test compiles without errors: `cargo build --tests`
- [ ] No clippy warnings: `cargo clippy --tests`
- [ ] Test passes in isolation: `cargo test test_create_mode_bound_scope`
- [ ] Test passes with full suite: `cargo test`
- [ ] Test runs in parallel without `#[serial]`: `cargo test --test-threads=8`
- [ ] No filesystem artifacts left after test execution
- [ ] Ref path assertion is correct: `refs/jin/modes/testmode/scopes/testscope`

### Feature Validation

- [ ] Test logic unchanged - only isolation mechanism changed
- [ ] Test still verifies scope creation under mode works correctly
- [ ] Test still verifies the correct ref path exists
- [ ] Test can run multiple times without failure
- [ ] Test can run with other tests without interference

### Code Quality Validation

- [ ] Uses `setup_unit_test()` instead of `setup_test_env()`
- [ ] Uses `create_test_mode_in_context()` with `&ctx` parameter
- [ ] Uses `JinRepo::open_or_create_at(&ctx.jin_dir)` instead of `open_or_create()`
- [ ] `#[serial]` attribute removed from test function
- [ ] Test follows same pattern as other isolated tests (e.g., `test_cleanup_removes_all_refs`)
- [ ] Borrowed references (`&ctx`, `&ctx.jin_dir`) used correctly
- [ ] Context cleanup is automatic via `Drop` trait

### Documentation & Deployment

- [ ] No environment variables need to be documented (using explicit paths)
- [ ] Test is self-documenting with clear function and variable names
- [ ] No changes to production code (only test code)

---

## Anti-Patterns to Avoid

- **Don't keep `#[serial]` attribute** - The whole point of this refactor is to enable parallel execution by using proper isolation. Remove `#[serial]` after confirming test works.

- **Don't use `JIN_DIR` environment variable** - The new pattern uses explicit paths via `ctx.jin_dir`. Using the environment variable defeats the purpose of isolation.

- **Don't call `setup_test_env()`** - This is the legacy function we're replacing. Use `setup_unit_test()` instead.

- **Don't use `JinRepo::open_or_create()`** - This relies on `JIN_DIR` environment variable. Use `JinRepo::open_or_create_at(&ctx.jin_dir)` for explicit path isolation.

- **Don't pass `ctx` by value** - The helper functions expect `&ctx` (borrowed reference). Passing by value would move the context and break the `Drop` cleanup.

- **Don't let `ctx` go out of scope early** - The context must live for the entire test duration. Don't create it inside an inner scope.

- **Don't change the test logic** - Only change the isolation mechanism. The test should still verify the same behavior (scope creation under mode produces the correct ref).

- **Don't skip verification** - Always run the test in isolation first, then with the full suite, then explicitly in parallel to confirm proper isolation before considering the task complete.

---

## Confidence Score

**Confidence Score: 10/10** for one-pass implementation success

**Reasoning**:
1. All helper functions (`setup_unit_test`, `create_test_mode_in_context`, `cleanup_test_mode`) are already implemented and tested in previous subtasks (S1, S2)
2. Clear example exists in the same file (`test_cleanup_removes_all_refs`) showing the exact pattern to follow
3. The change is mechanical - replacing isolation mechanism without changing test logic
4. Comprehensive validation steps ensure success at each stage
5. All potential gotchas are documented with solutions
6. The refactored test has identical behavior to the original, just with better isolation

---

## Additional Research Documents

See `/home/dustin/projects/jin/plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/P1M4T1S3/research/` for:
- Current test implementation analysis
- Isolation helper function signatures and usage
- Test infrastructure analysis from architecture document
- Similar test patterns in the codebase
- Rust testing best practices for isolation
