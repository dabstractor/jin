# PRP: Add Integration Tests for Workspace Validation

## Goal

**Feature Goal**: Verify that integration tests for workspace validation are comprehensive and passing

**Deliverable**: Validated test suite for detached workspace detection covering all three detachment conditions

**Success Definition**:
- All existing tests in `tests/workspace_validation.rs`, `tests/destructive_validation.rs`, and `tests/repair_check.rs` pass
- Test coverage is comprehensive for all three detachment conditions
- Tests follow established patterns from the codebase

## Why

- **Quality Assurance**: Integration tests prevent regressions in workspace validation logic
- **Documentation**: Tests serve as executable documentation of expected behavior
- **Confidence**: Comprehensive test coverage ensures safe refactoring of validation logic
- **Compliance**: Completes P1.M3.T5 requirements for PRD compliance

## What

### Current State

Tests have already been written in three files:
- `tests/workspace_validation.rs` - Core validation function tests
- `tests/destructive_validation.rs` - Reset/apply validation integration tests
- `tests/repair_check.rs` - Repair --check command tests

### Success Criteria

- [ ] All tests pass with `cargo test --test workspace_validation`
- [ ] All tests pass with `cargo test --test destructive_validation`
- [ ] All tests pass with `cargo test --test repair_check`
- [ ] Full test suite passes with `cargo test`
- [ ] Coverage includes all three detachment conditions:
  1. Files modified/deleted externally
  2. Missing layer refs
  3. Invalid active context

## All Needed Context

### Context Completeness Check

_The tests already exist. This PRP provides context for verification and potential expansion._

### Documentation & References

```yaml
# MUST READ - Testing patterns from codebase
- file: /home/dustin/projects/jin/tests/workspace_validation.rs
  why: Example of core validation function testing - tests validate_workspace_attached() directly
  pattern: Uses TestFixture, direct function calls, error matching for DetachedWorkspace
  gotcha: Tests use std::env::set_current_dir - must run with serial_test or ensure isolation

- file: /home/dustin/projects/jin/tests/destructive_validation.rs
  why: Example of CLI integration testing - tests reset --hard and apply --force validation
  pattern: Uses TestFixture, calls command execute functions directly, validates error types
  gotcha: Uses force: true to skip confirmation prompts

- file: /home/dustin/projects/jin/tests/repair_check.rs
  why: Example of CLI binary testing - tests repair --check command via assert_cmd
  pattern: Uses jin_cmd() helper, predicates for output matching, --dry-run flag testing
  gotcha: Tests both --check only and full repair workflows

- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  why: Reference for complex integration testing patterns - push/fetch/sync workflows
  pattern: Uses setup_jin_with_remote(), creates commits, validates Git state
  gotcha: Shows how to test multi-step workflows

- file: /home/dustin/projects/jin/tests/conflict_workflow.rs
  why: Reference for end-to-end workflow testing - complete conflict resolution flow
  pattern: Tests full user journey from conflict to resolution, state persistence
  gotcha: Shows how to test stateful operations across multiple commands

# Test utilities and fixtures
- file: /home/dustin/projects/jin/tests/common/mod.rs
  why: Test module organization - see how tests/common is structured

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: TestFixture and RemoteFixture - reusable test environment setup
  pattern: TestFixture::new() creates isolated temp directory with JIN_DIR isolation
  gotcha: Always call fixture.set_jin_dir() before running commands

- file: /home/dustin/projects/jin/tests/common/git_helpers.rs
  why: GitTestEnv - automatic Git lock cleanup for tests
  pattern: cleanup_git_locks() removes .git/*.lock files
  gotcha: Git lock contention is common cause of test flakiness

- file: /home/dustin/projects/jin/tests/common/assertions.rs
  why: Custom assertion helpers - assert_workspace_file, assert_staging_contains, etc.
  pattern: Helper functions for common Jin-specific assertions
  gotcha: Use these instead of raw assertions for better error messages

# Core implementation being tested
- file: /home/dustin/projects/jin/src/staging/workspace.rs
  why: validate_workspace_attached() implementation (lines 325-399)
  critical: Three detection functions: detect_file_mismatch, detect_missing_commits, detect_invalid_context
  section: Helper functions detect_file_mismatch() (170-200), detect_missing_commits() (218-241), detect_invalid_context() (259-282)

- file: /home/dustin/projects/jin/src/core/error.rs
  why: DetachedWorkspace error definition (lines 38-54, 173-204)
  pattern: Error struct with workspace_commit, expected_layer_ref, details, recovery_hint
  gotcha: Error Display impl formats multi-line message with details and recovery hint

- file: /home/dustin/projects/jin/src/commands/reset.rs
  why: reset --hard validation integration (lines 58-64)
  pattern: Validates BEFORE confirmation prompt to prevent unnecessary prompts
  gotcha: Only validates for hard mode, not soft/mixed

- file: /home/dustin/projects/jin/src/commands/apply.rs
  why: apply --force validation integration (lines 114-120)
  pattern: Only validates when --force flag is set
  gotcha: Normal apply (without --force) does not validate

- file: /home/dustin/projects/jin/src/commands/repair.rs
  why: repair --check implementation (lines 678-738)
  pattern: check_workspace_attachment() function, early return when --check is set
  gotcha: --check exits immediately after workspace check, doesn't run other checks

# External resources
- url: https://rust-cli.github.io/book/tutorial/testing.html
  why: Best practices for CLI testing - assert_cmd usage, test organization
  critical: Focus on observable behaviors, not internal implementation details

- url: https://docs.rs/assert_cmd
  why: assert_cmd API reference - Command::cargo_bin(), assert(), success(), failure()
  critical: Understanding .stdout(), .stderr(), .code() assertion methods

- url: https://docs.rs/predicates
  why: predicates crate - predicate::str::contains() for output matching
  critical: Complex boolean logic with .and(), .or(), .not() combinators
```

### Current Codebase Tree (relevant sections)

```bash
tests/
├── common/
│   ├── mod.rs              # Module declarations
│   ├── assertions.rs       # Custom Jin-specific assertions
│   ├── fixtures.rs         # TestFixture, RemoteFixture, setup helpers
│   └── git_helpers.rs      # Git lock cleanup helpers
├── workspace_validation.rs # Core validation function tests
├── destructive_validation.rs # reset/apply validation tests
├── repair_check.rs         # repair --check tests
├── sync_workflow.rs        # Reference: sync workflow tests
└── conflict_workflow.rs    # Reference: conflict resolution tests
```

### Existing Test Files Summary

| File | Tests | Purpose | Status |
|------|-------|---------|--------|
| `workspace_validation.rs` | 14 tests | Direct `validate_workspace_attached()` testing | Written |
| `destructive_validation.rs` | 13 tests | reset --hard and apply --force validation | Written |
| `repair_check.rs` | 8 tests | repair --check command behavior | Written |

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Git lock file contention causes test flakiness
// Solution: Use tests/common/git_helpers.rs for automatic cleanup
// Or run tests with: cargo test -- --test-threads=1

// CRITICAL: std::env::set_current_dir affects all threads
// Solution: Always use fixture.set_jin_dir() for Jin isolation
// Never change CWD in parallel tests

// GOTCHA: DetachedWorkspace error has specific fields
// JinError::DetachedWorkspace { workspace_commit, expected_layer_ref, details, recovery_hint }
// Match on all fields when asserting error types

// GOTCHA: reset --hard uses force: true to skip confirmation
// When testing, set force: true in ResetArgs to avoid interactive prompt

// GOTCHA: apply --force only validates when force flag is set
// Normal apply (force: false) does NOT call validate_workspace_attached()

// GOTCHA: repair --check exits early after workspace check
// It does NOT run the other 7 repair checks when --check is set

// PATTERN: Fresh workspace (no metadata) passes validation
// validate_workspace_attached() returns Ok(()) when WorkspaceMetadata doesn't exist
```

## Implementation Blueprint

### Test Structure Overview

The workspace validation tests are organized into three files:

1. **`workspace_validation.rs`** - Unit-style integration tests
   - Tests `validate_workspace_attached()` function directly
   - Tests all three detachment conditions
   - Tests validation order (file mismatch checked first)
   - Tests recovery hints

2. **`destructive_validation.rs`** - CLI command integration tests
   - Tests `reset --hard` rejection when detached
   - Tests `apply --force` rejection when detached
   - Tests that non-destructive operations skip validation
   - Tests error messages and recovery hints

3. **`repair_check.rs`** - Binary-level CLI tests
   - Tests `repair --check` command output
   - Tests --check early exit behavior
   - Tests --check with --dry-run flag

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY existing tests compile and pass
  - RUN: cargo test --test workspace_validation
  - RUN: cargo test --test destructive_validation
  - RUN: cargo test --test repair_check
  - VERIFY: All tests pass without errors
  - IF FAILING: Debug and fix any compilation or test failures

Task 2: ANALYZE test coverage gaps
  - REVIEW: Tests in workspace_validation.rs (14 tests)
  - REVIEW: Tests in destructive_validation.rs (13 tests)
  - REVIEW: Tests in repair_check.rs (8 tests)
  - IDENTIFY: Any missing edge cases for three detachment conditions
  - DOCUMENT: Coverage gaps if found

Task 3: ADD missing test cases if needed
  - IF: Missing edge case for file modification
  - ADD: Test for external file modification after apply
  - IF: Missing edge case for layer ref deletion
  - ADD: Test for partial layer ref deletion (some exist, some don't)
  - IF: Missing edge case for context invalidation
  - ADD: Test for mode/scope deletion after activation
  - FOLLOW: Existing test patterns in respective files

Task 4: VERIFY test isolation and parallel safety
  - CHECK: Each test uses unique TestFixture
  - CHECK: No shared state between tests
  - CHECK: Git lock cleanup is working
  - RUN: cargo test -- --test-threads=1 (baseline)
  - RUN: cargo test (with parallel execution)
  - VERIFY: Tests pass in both modes

Task 5: RUN full test suite validation
  - RUN: cargo test --all
  - VERIFY: No regressions in other tests
  - CHECK: Test output shows expected pass/fail counts
  - VALIDATE: No unexpected test failures or panics

Task 6: DOCUMENT test results
  - CREATE: plan/P1M3T5/research/test_results.md
  - RECORD: Test pass/fail status
  - RECORD: Any issues found and resolved
  - RECORD: Test coverage summary
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Test fixture setup (from workspace_validation.rs)

#[test]
fn test_validation_detects_modified_files() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();  // CRITICAL: Set JIN_DIR for isolation
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Setup: Create file and metadata
    let file_path = "test_config.txt";
    let original_content = b"original content";
    fs::write(fixture.path().join(file_path), original_content).unwrap();

    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let oid = repo.inner().blob(original_content).unwrap();
    let hash = oid.to_string();

    // Create metadata with file hash
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.add_file(PathBuf::from(file_path), hash.clone());
    metadata.save().unwrap();

    // Verify: Validation passes initially
    let context = jin::core::config::ProjectContext::default();
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(result.is_ok());

    // Trigger: Modify file externally
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Verify: Validation fails with DetachedWorkspace error
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(result.is_err());

    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            assert!(details.contains("modified") || details.contains("Workspace files"));
        }
        _ => panic!("Expected DetachedWorkspace error"),
    }
}

// PATTERN: CLI command testing (from destructive_validation.rs)

#[test]
fn test_reset_hard_rejected_when_files_modified() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    init_jin_project(&jin_dir).unwrap();

    // Setup tracked file
    let file_path = "config.txt";
    setup_tracked_file(&fixture, file_path, b"original content").unwrap();

    // Modify file externally
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt reset --hard, should be rejected
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: false,
        hard: true,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: true,  // CRITICAL: Skip confirmation for tests
    });

    assert!(result.is_err());

    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            assert!(details.contains("modified"));
        }
        _ => panic!("Expected DetachedWorkspace error"),
    }
}

// PATTERN: Binary CLI testing (from repair_check.rs)

fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_repair_check_success_when_attached() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run repair --check
    let result = jin_cmd()
        .args(["repair", "--check"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    result
        .success()
        .stdout(predicate::str::contains("Checking workspace attachment"))
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("Workspace is properly attached"));
}
```

### Test Coverage Matrix

| Detachment Condition | workspace_validation.rs | destructive_validation.rs | repair_check.rs |
|---------------------|-------------------------|---------------------------|-----------------|
| Files modified/deleted | ✓ (5 tests) | ✓ (6 tests) | ✓ (1 test) |
| Missing layer refs | ✓ (4 tests) | ✓ (4 tests) | - |
| Invalid active context | ✓ (3 tests) | ✓ (2 tests) | - |
| Fresh workspace (no metadata) | ✓ (2 tests) | ✓ (2 tests) | ✓ (1 test) |
| Recovery hints | ✓ (1 test) | ✓ (2 tests) | - |
| Non-destructive operations skip validation | - | ✓ (2 tests) | - |

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run individual test files
cargo test --test workspace_validation
cargo test --test destructive_validation
cargo test --test repair_check

# Expected: All tests compile and pass
# If errors exist: READ output and fix before proceeding

# Check for warnings
cargo test --all -- -D warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run all integration tests
cargo test --test '*_validation' --test 'repair_check' --verbose

# Run specific test patterns
cargo test test_validation_detects_modified_files
cargo test test_reset_hard_rejected_when_files_modified
cargo test test_repair_check_success_when_attached

# Expected: All targeted tests pass
# If failing: Debug root cause and fix
```

### Level 3: Integration Testing (System Validation)

```bash
# Run full test suite
cargo test --all

# Run tests in single-threaded mode (for debugging)
cargo test -- --test-threads=1

# Run tests with output
cargo test --all -- --nocapture --test-threads=1

# Expected: All tests pass, no deadlocks, no timeouts
```

### Level 4: Coverage & Quality Validation

```bash
# Check test coverage (if cargo-tarpaulin is installed)
cargo tarpaulin --test workspace_validation --test destructive_validation --test repair_check --out Html

# Run clippy on test files
cargo clippy --tests -- -D warnings

# Format check
cargo fmt -- --check tests/workspace_validation.rs
cargo fmt -- --check tests/destructive_validation.rs
cargo fmt -- --check tests/repair_check.rs

# Expected: High coverage (>80%), no clippy warnings, properly formatted
```

## Final Validation Checklist

### Technical Validation

- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy --tests`
- [ ] Properly formatted: `cargo fmt -- --check tests/`
- [ ] Tests pass in parallel mode (default)
- [ ] Tests pass in single-threaded mode: `cargo test -- --test-threads=1`

### Feature Validation

- [ ] All three detachment conditions have test coverage
- [ ] Fresh workspace (no metadata) passes validation
- [ ] Destructive operations (reset --hard, apply --force) reject detached workspace
- [ ] Non-destructive operations skip validation
- [ ] repair --check detects and reports detached state
- [ ] Error messages include recovery hints

### Code Quality Validation

- [ ] Tests follow existing patterns from codebase
- [ ] Each test uses isolated TestFixture
- [ ] No shared state between tests
- [ ] Proper cleanup of temporary directories
- [ ] Git lock cleanup is working (no lock file errors)

### Documentation & Deployment

- [ ] Test results documented in plan/P1M3T5/research/test_results.md
- [ ] Any gaps or issues are recorded
- [ ] Tests are ready for CI/CD integration

## Anti-Patterns to Avoid

- ❌ Don't add tests that duplicate existing coverage
- ❌ Don't change working directory in tests without restoring it
- ❌ Don't use shared fixtures across tests (causes flakiness)
- ❌ Don't ignore Git lock cleanup (causes test failures)
- ❌ Don't test implementation details - focus on observable behavior
- ❌ Don't skip confirmation prompts incorrectly (use force: true where needed)
- ❌ Don't forget to set JIN_DIR environment variable for isolation
- ❌ Don't use std::env::set_current_dir in parallel tests
