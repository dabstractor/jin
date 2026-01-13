# Product Requirement Prompt (PRP): P1.M4.T1.S4 - Verify Test Isolation for Parallel Execution

## Goal

**Feature Goal**: Verify that the refactored `test_create_mode_bound_scope` test (from subtask S3) passes consistently when run in parallel with other tests, confirming proper test isolation.

**Deliverable**: Verification results demonstrating that the test passes consistently across multiple execution scenarios (parallel, sequential, repeated runs) with 100% success rate.

**Success Definition**:
- The test `commands::scope::tests::test_create_mode_bound_scope` passes with `--test-threads=2` (parallel execution)
- The test passes in full test suite runs (`cargo test`)
- The test passes across 10 consecutive runs with no flakiness
- No remaining shared state issues detected (e.g., static variables, global singletons)

## User Persona

**Target User**: Development team members and CI/CD systems that run the test suite.

**Use Case**: Running the test suite in parallel for faster feedback during development and in continuous integration pipelines.

**User Journey**:
1. Developer makes changes to the codebase
2. Developer runs `cargo test` to verify changes
3. All tests run in parallel by default (faster feedback)
4. Developer receives reliable test results without flakiness

**Pain Points Addressed**:
- Flaky tests that fail intermittently when run with other tests
- Time-consuming sequential test execution required for reliable results
- Uncertainty whether test failures indicate real issues or isolation problems

## Why

- **Parallel Testing Performance**: Rust's test harness runs tests in parallel by default for faster execution. Proper isolation enables this performance benefit.
- **CI/CD Efficiency**: CI systems can complete test runs faster when tests run in parallel, reducing feedback time.
- **Test Reliability**: Flaky tests erode trust in the test suite. Proper isolation ensures consistent results.
- **Developer Experience**: Developers need reliable, fast test feedback during development.

## What

**User-visible behavior**: When running `cargo test`, all tests including `test_create_mode_bound_scope` pass consistently, regardless of whether they run in parallel or sequentially.

### Success Criteria

- [ ] Test passes with parallel execution (`--test-threads=2`)
- [ ] Test passes in full test suite (`cargo test`)
- [ ] Test passes across 10 consecutive runs (no flakiness)
- [ ] No shared state issues detected (investigation if failures occur)
- [ ] Optional: Remove `#[serial]` attribute if test remains stable

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes. This PRP provides the exact test location, isolation helpers used, verification commands, expected outcomes, and debugging procedures.

### Documentation & References

```yaml
# MUST READ - Critical context for verification

- file: /home/dustin/projects/jin/src/commands/scope.rs
  why: Contains the refactored test_create_mode_bound_scope test that uses isolation helpers
  section: Lines 634-664 (test_create_mode_bound_scope function)
  critical: The test now uses setup_unit_test() and create_test_mode_in_context() for isolation

- file: /home/dustin/projects/jin/src/commands/scope.rs
  why: Contains the isolation helper functions created in S1-S2
  section: Lines 478-544 (create_test_mode_in_context and cleanup_test_mode)
  pattern: These functions use absolute paths (ctx.jin_dir) instead of environment variables

- file: /home/dustin/projects/jin/src/test_utils.rs
  why: Contains the UnitTestContext pattern that provides automatic cleanup and isolation
  section: Lines 40-96 (UnitTestContext struct and setup_unit_test function)
  critical: The Drop implementation restores original environment, ensuring no cross-test interference

- file: /home/dustin/projects/jin/plan/001_8630d8d70301/bug_hunt_tasks.json
  why: Contains the contract definition and context for this subtask
  section: Lines 242-252 (P1.M4.T1.S4 definition)
  critical: Defines the expected verification approach and success criteria

- url: https://doc.rust-lang.org/cargo/commands/cargo-test.html
  why: Official Cargo test documentation for --test-threads flag
  section: "--test-threads flag controls parallelism"
  critical: Default is number of CPU cores; --test-threads=1 forces sequential execution

- url: https://docs.rs/serial_test/latest/serial_test/
  why: Documentation for serial_test crate used in the codebase
  section: "The #[serial] attribute forces sequential execution"
  gotcha: Has performance impact; should only be used when absolutely necessary
```

### Current Codebase Tree (Relevant Sections)

```bash
/home/dustin/projects/jin/
├── src/
│   ├── commands/
│   │   └── scope.rs              # Contains test_create_mode_bound_scope (lines 634-664)
│   └── test_utils.rs             # Contains UnitTestContext and setup_unit_test()
│
├── tests/
│   └── common/                   # Shared test utilities
│       ├── fixtures.rs           # TestFixture pattern for integration tests
│       └── git_helpers.rs        # Git lock cleanup utilities
│
├── Cargo.toml                    # Test dependencies: serial_test, tempfile, assert_cmd
└── plan/
    └── 001_8630d8d70301/
        └── bugfix/
            └── 001_d2716c9eb3cf/
                └── P1M4T1S4/
                    └── PRP.md    # This document
```

### Desired Codebase Tree After Verification

```bash
# No code changes expected in this subtask - only verification
# If verification succeeds and test is stable, consider removing #[serial] attribute

# Potential change (if test proves stable):
# src/commands/scope.rs line 635: Remove #[serial] attribute from test_create_mode_bound_scope
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: JIN_DIR environment variable is still set in the test (line 650)
// Even with isolation via absolute paths, the create() function uses JinRepo::open_or_create()
// which relies on JIN_DIR. This is why the test still sets JIN_DIR temporarily.

// GOTCHA: The #[serial] attribute is still present on the test (line 635)
// This was left as a safety precaution. After successful verification, it can be removed.

// CRITICAL: TempDir cleanup happens via RAII when UnitTestContext is dropped
// Keep the ctx variable in scope until the test completes.

// CRITICAL: Git lock files can cause conflicts between parallel tests
// The setup_unit_test() function calls cleanup_before_test() to remove stale locks.

// GOTCHA: Some tests in scope.rs still use the old setup_test_env() pattern
// Only test_create_mode_bound_scope was refactored in S3.
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed for this verification task. The existing test structure uses:

```rust
// From src/commands/scope.rs lines 478-544
fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext)
fn cleanup_test_mode(name: &str, ctx: &UnitTestContext)

// From src/test_utils.rs lines 48-96
pub struct UnitTestContext {
    _temp_dir: TempDir,
    _original_dir: Option<PathBuf>,
    _original_jin_dir: Option<String>,
    pub project_path: PathBuf,
    pub jin_dir: PathBuf,
}
```

### Implementation Tasks (Verification Only)

This subtask is a **verification task**, not an implementation task. The code changes were completed in S3. This task focuses on:

```yaml
Verification Task 1: Run test with parallel execution
  - COMMAND: cargo test --test-threads=2 commands::scope::tests::test_create_mode_bound_scope
  - EXPECTED: Test passes with no failures
  - GOTCHA: --test-threads=2 forces parallelism, exposing isolation issues
  - IF_FAILS: Proceed to Verification Task 5 (debugging)

Verification Task 2: Run test in full test suite
  - COMMAND: cargo test
  - EXPECTED: All tests pass, including test_create_mode_bound_scope
  - GOTCHA: Full suite runs many tests in parallel, maximum stress test
  - IF_FAILS: Proceed to Verification Task 5 (debugging)

Verification Task 3: Run test repeatedly for flakiness detection
  - COMMAND: for i in {1..10}; do cargo test commands::scope::tests::test_create_mode_bound_scope || exit 1; done
  - EXPECTED: 100% pass rate (10/10 iterations)
  - GOTCHA: Flaky tests may pass sometimes but not always
  - IF_FAILS: Proceed to Verification Task 5 (debugging)

Verification Task 4: (Optional) Remove #[serial] attribute
  - CONDITION: Only if Tasks 1-3 all pass consistently
  - ACTION: Remove #[serial] from line 635 in src/commands/scope.rs
  - VERIFY: Rerun Tasks 1-3 to confirm stability without #[serial]
  - ROLLBACK: If tests fail, restore #[serial] attribute

Verification Task 5: Debug isolation issues (if tests fail)
  - CHECK1: Look for static variables or global state in scope.rs
  - CHECK2: Verify absolute paths are used everywhere (ctx.jin_dir)
  - CHECK3: Check for environment variable pollution
  - CHECK4: Run with RUST_BACKTRACE=1 for detailed failure info
  - CHECK5: Add debug output to identify shared state access
```

### Verification Patterns & Key Details

```bash
# Pattern: Cargo test execution with controlled parallelism

# 1. Parallel execution (default: CPU core count)
cargo test
# Expected: All tests run in parallel threads

# 2. Limited parallelism
cargo test -- --test-threads=2
# Expected: Maximum 2 tests run concurrently

# 3. Sequential execution
cargo test -- --test-threads=1
# Expected: Tests run one at a time (no parallelism)

# 4. Single test execution
cargo test commands::scope::tests::test_create_mode_bound_scope
# Expected: Only this test runs, but may run with other tests in parallel

# 5. Verbose output
cargo test -- --show-output
# Expected: Shows println! output from tests

# 6. No fail-fast
cargo test -- --no-fail-fast
# Expected: All tests run even if some fail
```

```rust
// Pattern: Test isolation verification

// The refactored test should demonstrate these isolation properties:

// 1. Filesystem isolation: Each test gets unique temp directory
let ctx = setup_unit_test();  // Creates /tmp/.tmpXXXXXXX/.jin_global

// 2. Path isolation: Uses absolute paths, not current_dir
let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();

// 3. Environment isolation: Original JIN_DIR restored on drop
// UnitTestContext Drop implementation handles this automatically

// 4. Git ref isolation: Each test creates unique mode/scope refs
create_test_mode_in_context("testmode", &ctx);  // refs/jin/modes/testmode/_mode
create("testscope", Some("testmode"));          // refs/jin/modes/testmode/scopes/testscope

// 5. Cleanup isolation: TempDir deleted when ctx goes out of scope
// Automatic via Drop, no manual cleanup needed
```

### Integration Points

No new integration points. This verification task validates existing integrations:

```yaml
EXISTING_INTEGRATION:
  - test_framework: Rust's built-in test harness
  - parallelism: Cargo's --test-threads flag
  - isolation: UnitTestContext from src/test_utils.rs
  - helpers: create_test_mode_in_context, cleanup_test_mode

VERIFICATION_TARGET:
  - test: commands::scope::tests::test_create_mode_bound_scope
  - location: src/commands/scope.rs:634-664
  - attribute: #[serial] (line 635) - may be removed after verification
```

## Validation Loop

### Level 1: Syntax & Style (Not Applicable)

This is a verification task. No code changes are expected (unless removing `#[serial]`).

If removing `#[serial]` after successful verification:
```bash
# Run after removing #[serial] attribute
cargo test --check                    # Check for compilation errors
cargo clippy -- -D warnings          # Linting check
```

### Level 2: Unit Test Verification (Primary Validation)

```bash
# Verification 2.1: Run target test in parallel
cargo test --test-threads=2 commands::scope::tests::test_create_mode_bound_scope

# Expected output:
#   test commands::scope::tests::test_create_mode_bound_scope ... ok
#   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

# Verification 2.2: Run all scope tests in parallel
cargo test --test-threads=4 commands::scope

# Expected: All scope tests pass, including test_create_mode_bound_scope
# Typical scope test count: ~20 tests

# Verification 2.3: Run scope tests sequentially
cargo test --test-threads=1 commands::scope

# Expected: Same tests pass in sequential mode (confirms isolation works)

# Verification 2.4: Compare parallel vs sequential results
# Run both and compare pass/fail counts
cargo test --test-threads=4 commands::scope -- -q | tee parallel_results.txt
cargo test --test-threads=1 commands::scope -- -q | tee sequential_results.txt
diff parallel_results.txt sequential_results.txt

# Expected: No differences (same tests pass in both modes)
```

### Level 3: Integration Testing (Full Test Suite)

```bash
# Verification 3.1: Full test suite with default parallelism
cargo test --all

# Expected: All tests pass (typically ~650 tests in the Jin project)
# Look for: "test result: ok. X passed; 0 failed"

# Verification 3.2: Full test suite with limited parallelism
cargo test --all -- --test-threads=2

# Expected: All tests pass even with forced parallelism
# This is the key test for isolation verification

# Verification 3.3: Full test suite sequentially (baseline)
cargo test --all -- --test-threads=1

# Expected: All tests pass in sequential mode
# This confirms tests are fundamentally correct

# Verification 3.4: Check for specific test in full suite output
cargo test --all 2>&1 | grep -A2 "test_create_mode_bound_scope"

# Expected output:
#   test commands::scope::tests::test_create_mode_bound_scope ... ok
```

### Level 4: Flakiness and Stability Testing

```bash
# Verification 4.1: Single test repeated 10 times
for i in {1..10}; do
    echo "Run $i:"
    cargo test commands::scope::tests::test_create_mode_bound_scope -- -q || exit 1
done

# Expected: 10/10 runs pass (100% success rate)

# Verification 4.2: Full test suite repeated 3 times
for i in {1..3}; do
    echo "Full suite run $i:"
    cargo test --all -- -q || exit 1
done

# Expected: 3/3 runs pass with full suite

# Verification 4.3: Parallel stress test
# Run with high thread count to maximize parallelism
cargo test --all -- --test-threads=8

# Expected: All tests pass even under high parallelism

# Verification 4.4: Check for intermittent failures
# Run 20 iterations and count failures
FAILURES=0
for i in {1..20}; do
    cargo test commands::scope::tests::test_create_mode_bound_scope -- -q > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        ((FAILURES++))
    fi
done
echo "Failures: $FAILURES out of 20 runs"

# Expected: Failures: 0 out of 20 runs
# If failures > 0: Test is flaky, investigate root cause
```

### Level 5: Debug and Diagnostic Commands (If Tests Fail)

```bash
# Diagnostic 5.1: Enable backtrace for detailed error info
RUST_BACKTRACE=1 cargo test commands::scope::tests::test_create_mode_bound_scope

# Expected: Full stack trace on failure

# Diagnostic 5.2: Run test with output capture disabled
cargo test commands::scope::tests::test_create_mode_bound_scope -- --nocapture

# Expected: All println! output visible, helps debug issues

# Diagnostic 5.3: Run test with logging
RUST_LOG=debug cargo test commands::scope::tests::test_create_mode_bound_scope

# Expected: Debug log output (if using env_logger)

# Diagnostic 5.4: Check for Git lock files (common issue in parallel tests)
find /tmp -name "*.lock" -path "*/.git/*" 2>/dev/null | head -20

# Expected: No lock files in test directories
# If found: Lock cleanup may not be working properly

# Diagnostic 5.5: Check JIN_DIR pollution
# Add debug output to test:
# println!("JIN_DIR: {:?}", std::env::var("JIN_DIR"));
# println!("jin_dir: {:?}", ctx.jin_dir);

# Expected: JIN_DIR matches ctx.jin_dir (both isolated)
```

## Final Validation Checklist

### Technical Validation

- [ ] Test passes with `--test-threads=2` (parallel execution)
- [ ] Test passes in full test suite (`cargo test --all`)
- [ ] Test passes across 10 consecutive runs (no flakiness)
- [ ] Test passes in sequential mode (`--test-threads=1`) for baseline
- [ ] No difference in results between parallel and sequential execution

### Feature Validation

- [ ] No shared state issues detected (no static/global variable conflicts)
- [ ] No Git lock file conflicts between tests
- [ ] No environment variable pollution between tests
- [ ] Test is deterministic (same result every run)

### Code Quality Validation

- [ ] If stable, consider removing `#[serial]` attribute
- [ ] Test uses absolute paths from `ctx.jin_dir` (not environment variables)
- [ ] Test cleanup is automatic via `Drop` trait
- [ ] Test follows established patterns from `test_cleanup_removes_all_refs`

### Documentation & Deployment

- [ ] Verification results documented (pass/fail for each scenario)
- [ ] If `#[serial]` removed, commit message explains verification
- [ ] Any issues found are documented for future reference

---

## Anti-Patterns to Avoid

- ❌ **Don't remove `#[serial]` without verification**: Only remove after Tests 1-3 all pass
- ❌ **Don't ignore intermittent failures**: Flakiness indicates real isolation issues
- ❌ **Don't assume isolation works**: Verify with explicit parallel execution (`--test-threads=2`)
- ❌ **Don't skip repeated testing**: Run multiple iterations to catch flakiness
- ❌ **Don't ignore full suite failures**: The test must pass with ALL other tests
- ❌ **Don't modify test logic**: This is a verification task, not an implementation task
- ❌ **Don't change other tests**: Focus only on `test_create_mode_bound_scope`
- ❌ **Don't rely on `#[serial]` as permanent fix**: It's a performance penalty

## Success Metrics

**Confidence Score**: 9/10 for one-pass verification success

**Rationale**: The isolation helpers (S1-S2) follow established patterns from the codebase. The refactored test (S3) uses absolute paths and automatic cleanup. This verification task confirms the expected behavior.

**Validation**: If all validation checks pass, the test is properly isolated and can run in parallel. The `#[serial]` attribute can be removed for better performance.
