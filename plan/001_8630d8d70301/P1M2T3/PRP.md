# Product Requirement Prompt (PRP): Add Integration Tests for Push Enforcement

## Goal

**Feature Goal**: Create comprehensive integration tests for the fetch-before-push enforcement mechanism to ensure push operations correctly detect and handle remote repository state (behind, ahead, divergent, up-to-date).

**Deliverable**: Integration tests in `tests/sync_workflow.rs` that validate all push enforcement scenarios with proper coverage of edge cases.

**Success Definition**:
- All three sub-tasks (P1.M2.T3.S1, P1.M2.T3.S2, P1.M2.T3.S3) have corresponding passing tests
- Tests verify the exact behavior described in each sub-task
- Running `cargo test --test sync_workflow` passes all new tests
- Tests follow existing test patterns and use common fixtures

## User Persona

**Target User**: Developers and QA engineers verifying the push enforcement implementation

**Use Case**: Validate that the push command correctly prevents data loss by detecting when local repository is behind or divergent from remote, while still allowing legitimate pushes (ahead, up-to-date, force).

**User Journey**:
1. Developer runs integration tests to verify push enforcement
2. Tests simulate various Git state scenarios (behind, ahead, divergent)
3. Tests verify correct error messages and push behavior
4. Tests confirm `--force` flag properly bypasses safety checks

**Pain Points Addressed**:
- Ensures push enforcement works correctly before merging to main
- Provides regression protection for future changes to push logic
- Documents expected behavior through executable tests

## Why

- **Data Loss Prevention**: Push enforcement is critical for preventing accidental overwrite of remote changes
- **PRD Compliance**: P1.M2.T3 is a planned task marking the completion of Milestone 1.2
- **Test Coverage**: Existing tests in sync_workflow.rs may be incomplete or need enhancement
- **Validation**: Ensures the ref comparison logic (`compare_refs`) works correctly in real scenarios

## What

Integration tests covering these scenarios:

### Success Criteria

- [ ] **P1.M2.T3.S1**: Test push rejected when local is behind remote
  - Simulate remote having commits local doesn't have
  - Verify push fails with `JinError::BehindRemote`
  - Verify error message mentions "behind remote", "jin pull", and "--force"
- [ ] **P1.M2.T3.S2**: Test push succeeds when `--force` flag is set
  - Simulate behind scenario
  - Verify `--force` allows push despite being behind
  - Verify warning message about data loss is displayed
- [ ] **P1.M2.T3.S3**: Test push succeeds when up-to-date
  - Local and remote at same commit
  - Verify push succeeds (or reports "Nothing to push")
- [ ] **Additional Coverage**: Test push succeeds when ahead
  - Local has commits remote doesn't have
  - Verify push succeeds
- [ ] **Additional Coverage**: Test push rejected when divergent
  - Local and remote have different commits on same branch
  - Verify push fails with BehindRemote error

## All Needed Context

### Context Completeness Check

This PRP provides complete context for implementing integration tests for push enforcement. The implementation files already exist; this task is about validating them through tests.

### Documentation & References

```yaml
# MUST READ - Core implementation files

- file: /home/dustin/projects/jin/src/commands/push.rs
  why: Contains the push command implementation with fetch-before-push logic
  pattern: Look at execute() function flow (lines 17-103), detect_modified_layers() (lines 136-194)
  gotcha: The push command calls fetch first (line 32), then compares pre-fetch vs post-fetch refs

- file: /home/dustin/projects/jin/src/git/refs.rs
  why: Contains RefComparison enum and compare_refs() function used for push safety checks
  pattern: RefComparison::Ahead/Behind/Diverged/Equal (lines 16-28), compare_refs() algorithm (lines 164-177)
  gotcha: Uses git2's graph_ahead_behind() which returns (ahead_count, behind_count)

- file: /home/dustin/projects/jin/src/core/error.rs
  why: Contains JinError::BehindRemote error variant (lines 29-36)
  pattern: Error message format for push rejection
  gotcha: Error message includes remediation steps ("jin pull" and "--force")

# MUST READ - Test infrastructure

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Test fixture utilities for creating isolated test environments
  pattern: setup_jin_with_remote() for local+remote repos, create_mode(), unique_test_id()
  gotcha: Always call fixture.set_jin_dir() BEFORE any Jin operations for test isolation

- file: /home/dustin/projects/jin/tests/common/assertions.rs
  why: Custom assertion helpers for Jin-specific state verification
  pattern: assert_workspace_file_exists(), assert_layer_ref_exists(), assert_context_mode()
  gotcha: assert_layer_ref_exists() accepts optional jin_repo_path for test isolation

- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  why: Existing sync workflow tests that may need enhancement or serve as template
  pattern: Test structure using RemoteFixture, jin() command execution, Git state manipulation
  gotcha: Uses direct git2::Repository manipulation to simulate remote changes (see lines 348-376)

# MUST READ - Reference test patterns

- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  section: test_push_rejected_when_behind (lines 296-408)
  why: Example of testing behind scenario with direct remote manipulation
  pattern: Creates commit, pushes it, then directly updates remote ref via git2

- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  section: test_push_succeeds_with_force_when_behind (lines 410-509)
  why: Example of testing --force flag behavior
  pattern: Uses temp workspace to create remote state, then tests force push from local

- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  section: test_push_succeeds_when_ahead (lines 511-586)
  why: Example of testing ahead scenario (should succeed)
  pattern: Create first commit and push, create second commit without pushing, verify push succeeds
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── src/
│   ├── commands/
│   │   ├── push.rs          # Push command with fetch-before-push enforcement
│   │   ├── fetch.rs         # Fetch command called by push
│   │   └── mod.rs           # Command module exports
│   ├── git/
│   │   ├── refs.rs          # RefComparison enum and compare_refs()
│   │   ├── remote.rs        # Remote operations (build_push_options)
│   │   └── mod.rs           # Git module exports
│   ├── core/
│   │   ├── error.rs         # JinError::BehindRemote variant
│   │   └── mod.rs           # Core module exports
│   └── cli/
│       └── args.rs          # PushArgs struct with force field
└── tests/
    ├── sync_workflow.rs     # EXISTING: Sync workflow tests (may need enhancement)
    ├── common/
    │   ├── fixtures.rs      # Test fixtures (setup_jin_with_remote, TestFixture)
    │   ├── assertions.rs    # Custom assertions
    │   └── mod.rs           # Common module exports
    └── cli_basic.rs         # Basic CLI tests
```

### Desired Codebase Tree

```bash
# No new files - this task enhances existing tests in:
tests/sync_workflow.rs

# Tests to ADD or ENHANCE:
- test_push_rejected_when_behind()          # P1.M2.T3.S1 - may already exist, verify coverage
- test_push_succeeds_with_force_flag()      # P1.M2.T3.S2 - may already exist, verify coverage
- test_push_succeeds_when_up_to_date()      # P1.M2.T3.S3 - may need to be added
- test_push_succeeds_when_ahead()           # Additional coverage - verify exists
- test_push_rejected_when_divergent()       # Additional coverage - may need to be added
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Always set JIN_DIR for test isolation
// Without this, tests will interfere with each other
let jin_dir = fixture.jin_dir.clone().unwrap();
std::env::set_var("JIN_DIR", &jin_dir);

// CRITICAL: Use unique_test_id() for mode/scope names
// std::process::id() is NOT sufficient for parallel tests
let mode_name = format!("test_mode_{}", unique_test_id());

// CRITICAL: Git lock cleanup happens automatically on Drop
// But if tests crash, locks may remain - run with cleanup
fn cleanup_git_locks(repo_path: &Path) -> Result<()>

// GOTCHA: push.rs calls fetch BEFORE comparing refs
// The comparison is between pre-fetch local OIDs and post-fetch remote OIDs
// This is why capture_local_refs() exists (line 109)

// GOTCHA: graph_ahead_behind(a, b) returns (ahead, behind)
// NOT (behind, ahead) - first param is the "local" perspective
// (5, 0) means local is 5 commits ahead
// (0, 3) means local is 3 commits behind
// (2, 2) means diverged with 2 unique commits each

// GOTCHA: /local layers are NEVER pushed (line 115 in push.rs)
// Tests using modes/scopes will work; /local is explicitly excluded

// GOTCHA: Direct git2::Repository manipulation is needed to simulate remote changes
// You cannot use jin commands to update the remote directly in tests
// See sync_workflow.rs lines 348-376 for the pattern

// GOTCHA: predicates::str::contains() for partial string matching
// Use for error message validation since messages may have formatting
jin().arg("push").assert().failure()
    .stderr(predicate::str::contains("behind remote"))
    .stderr(predicate::str::contains("jin pull"))
    .stderr(predicate::str::contains("--force"));
```

## Implementation Blueprint

### Data Models and Structure

No new data models - using existing types:

```rust
// From src/git/refs.rs - already implemented
pub enum RefComparison {
    Ahead,    // Local ahead (fast-forward possible)
    Behind,   // Local behind (must pull first)
    Diverged, // Both have unique commits (merge required)
    Equal,    // Same commit
}

// From src/core/error.rs - already implemented
pub enum JinError {
    #[error("Push rejected: local layer '{layer}' is behind remote...")]
    BehindRemote { layer: String },
    // ... other variants
}

// From src/cli/args.rs - already implemented
pub struct PushArgs {
    #[arg(long)]
    pub force: bool,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY EXISTING TESTS in tests/sync_workflow.rs
  - CHECK: Does test_push_rejected_when_behind exist? Is it comprehensive?
  - CHECK: Does test_push_succeeds_with_force_when_behind exist? Does it test all aspects?
  - CHECK: Does test_push_succeeds_when_ahead exist?
  - IDENTIFY: Which tests from P1.M2.T3.S1/S2/S3 are missing or incomplete
  - DOCUMENT: What scenarios are not covered (e.g., divergent, up-to-date, multiple layers)
  - LOCATION: tests/sync_workflow.rs (lines 225-586 contain push tests)

Task 2: CREATE/ENHANCE test_push_rejected_when_behind (P1.M2.T3.S1)
  - IMPLEMENT: Test that push is rejected when local is behind remote
  - FOLLOW pattern: tests/sync_workflow.rs::test_push_rejected_when_behind (lines 296-408)
  - SETUP: Use setup_jin_with_remote() to get local + remote repos
  - SCENARIO:
    1. Create mode and local commit
    2. Push to remote
    3. Directly update remote ref via git2 to simulate remote change
    4. Create another local commit (now behind)
    5. Try to push without --force
  - VALIDATE:
    - Command fails (assert().failure())
    - Error contains "behind remote"
    - Error contains "jin pull"
    - Error contains "--force"
  - NAMING: test_push_rejected_when_behind
  - PLACEMENT: tests/sync_workflow.rs

Task 3: CREATE/ENHANCE test_push_succeeds_with_force_when_behind (P1.M2.T3.S2)
  - IMPLEMENT: Test that --force allows push when behind
  - FOLLOW pattern: tests/sync_workflow.rs::test_push_succeeds_with_force_when_behind (lines 410-509)
  - SETUP: Use temp workspace to populate remote, then test from local repo
  - SCENARIO:
    1. Create remote commit via temp workspace
    2. Create divergent local commit
    3. Push with --force flag
  - VALIDATE:
    - Command succeeds (assert().success())
    - Warning message displayed about data loss
    - Remote ref now points to local commit
  - NAMING: test_push_succeeds_with_force_when_behind
  - PLACEMENT: tests/sync_workflow.rs

Task 4: CREATE test_push_succeeds_when_up_to_date (P1.M2.T3.S3)
  - IMPLEMENT: Test that push succeeds (or reports nothing) when up-to-date
  - FOLLOW pattern: tests/sync_workflow.rs::test_push_succeeds_when_ahead
  - SETUP: Create local commit, push it, then try to push again
  - SCENARIO:
    1. Create mode and local commit
    2. Push to remote
    3. Try to push again (no new local commits)
  - VALIDATE:
    - Command succeeds (may report "Nothing to push")
    - No error occurs
    - Remote state unchanged
  - NAMING: test_push_succeeds_when_up_to_date
  - PLACEMENT: tests/sync_workflow.rs

Task 5: CREATE test_push_rejected_when_divergent (ADDITIONAL COVERAGE)
  - IMPLEMENT: Test that push is rejected when histories have diverged
  - FOLLOW pattern: Similar to test_push_rejected_when_behind but with divergent history
  - SETUP: Create same base commit, then different commits in local and remote
  - SCENARIO:
    1. Create base commit and push
    2. Create divergent commit in remote (via git2)
    3. Create divergent commit in local
    4. Try to push without --force
  - VALIDATE:
    - Command fails
    - Error contains "behind remote" or similar
    - Error mentions remediation steps
  - NAMING: test_push_rejected_when_divergent
  - PLACEMENT: tests/sync_workflow.rs

Task 6: VERIFY test_push_succeeds_when_ahead (ADDITIONAL COVERAGE)
  - CHECK: If test exists, verify it's comprehensive
  - SCENARIO: Local has commits remote doesn't have (normal push scenario)
  - VALIDATE: Push succeeds and remote is updated

Task 7: RUN ALL TESTS to verify coverage
  - EXECUTE: cargo test --test sync_workflow
  - VERIFY: All tests pass
  - CHECK: Test output shows all scenarios covered
  - VALIDATE: No test failures or panics

Task 8: ADD TEST DOCUMENTATION
  - ENSURE: Each test has clear doc comment explaining what it tests
  - ENSURE: Comments explain the Git state being simulated
  - FOLLOW: Rust documentation conventions (/// for public, // for internal)
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Test structure for push enforcement tests
#[test]
fn test_push_<scenario>() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup test environment with remote
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("test_mode_{}", unique_test_id());
    let jin_dir = remote_fixture.local_path.join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    // 2. Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 3. Create and activate mode
    create_mode(&mode_name, Some(&jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 4. Set up Git state (behind/ahead/divergent/up-to-date)
    // ... specific setup for scenario

    // 5. Execute push command
    jin()
        .args(["push"])  // or ["push", "--force"] for force tests
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()  // or .success() depending on expected outcome
        .stderr(predicate::str::contains("expected error text"));

    Ok(())
}

// PATTERN: Directly manipulating remote repository to simulate changes
// GOTCHA: You cannot use jin commands to update remote in tests
// Use git2::Repository to manipulate the bare remote directly

let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
let remote_ref_path = format!("refs/jin/layers/mode/{}", mode_name);

// Get current remote commit
let current_remote = remote_repo.find_reference(&remote_ref_path)?;
let current_oid = current_remote.target().unwrap();
let current_commit = remote_repo.find_commit(current_oid)?;

// Create new commit on top
let sig = remote_repo.signature()?;
let mut tree_builder = remote_repo.treebuilder(None)?;
let blob_oid = remote_repo.blob(b"new content")?;
tree_builder.insert("file.txt", blob_oid, 0o100644)?;
let tree_oid = tree_builder.write()?;
let tree = remote_repo.find_tree(tree_oid)?;

let new_commit_oid = remote_repo.commit(
    Some(&remote_ref_path),
    &sig,
    &sig,
    "Remote commit",
    &tree,
    &[&current_commit],
)?;

// PATTERN: Using temp workspace to populate remote
// Useful for creating complex remote states
let temp_workspace = TestFixture::new()?;
temp_workspace.set_jin_dir();
jin_init(temp_workspace.path())?;
// ... create commits in temp workspace ...
// Link and push to remote
jin()
    .args(["link", remote_fixture.remote_path.to_str().unwrap()])
    .current_dir(temp_workspace.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// PATTERN: Command assertion with predicates
use predicates::prelude::*;

jin()
    .arg("push")
    .current_dir(&remote_fixture.local_path)
    .env("JIN_DIR", &jin_dir)
    .assert()
    .failure()  // For expected failures
    .stderr(predicate::str::contains("behind remote"))
    .stderr(predicate::str::contains("jin pull"))
    .stderr(predicate::str::contains("--force"));

// PATTERN: Verifying remote state after push
let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
let ref_path = format!("refs/jin/layers/mode/{}", mode_name);
match remote_repo.find_reference(&ref_path) {
    Ok(reference) => {
        let oid = reference.target().expect("Ref should have target");
        let commit = remote_repo.find_commit(oid)?;
        assert!(commit.message().unwrap_or("").contains("expected message"));
    }
    Err(e) => panic!("Remote should have ref {}: {}", ref_path, e),
}
```

### Integration Points

```yaml
NO INTEGRATION POINTS:
  - This task is purely adding/enhancing tests
  - No changes to source code required
  - No new dependencies needed
  - No configuration changes required

TEST EXECUTION:
  - command: cargo test --test sync_workflow
  - expected: All tests pass
  - coverage: Should cover P1.M2.T3.S1, P1.M2.T3.S2, P1.M2.T3.S3
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after writing tests - fix before proceeding
cargo test --test sync_workflow --no-run

# Check for compilation errors
cargo check --tests

# Expected: Zero compilation errors. If errors exist, READ output and fix before proceeding.

# Run linter if configured
cargo clippy --tests

# Expected: No warnings. Fix any clippy warnings before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the specific sync_workflow tests
cargo test --test sync_workflow

# Run with output for debugging
cargo test --test sync_workflow -- --nocapture

# Run specific test
cargo test --test sync_workflow test_push_rejected_when_behind -- --exact

# Expected: All tests pass. If failing, debug root cause and fix test implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all tests to ensure no regression
cargo test --tests

# Run with concurrency to catch race conditions
cargo test --tests -- --test-threads=4

# Expected: All integration tests pass, no test isolation failures.

# Verify specific test scenarios manually if needed
# Create actual local and remote repos to verify behavior matches tests
```

### Level 4: Coverage Validation

```bash
# Install tarpaulin for coverage if available
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --tests --test sync_workflow --out Html

# Expected: Coverage of push enforcement logic should be high
# The tests should exercise: capture_local_refs, detect_modified_layers, compare_refs

# Alternative: Use cargo-llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --tests --test sync_workflow
```

## Final Validation Checklist

### Technical Validation

- [ ] All 3 sub-tasks (P1.M2.T3.S1, P1.M2.T3.S2, P1.M2.T3.S3) have corresponding tests
- [ ] All tests pass: `cargo test --test sync_workflow`
- [ ] No compilation errors: `cargo check --tests`
- [ ] No clippy warnings: `cargo clippy --tests`
- [ ] Tests use proper fixtures and isolation (JIN_DIR set, unique_test_id())
- [ ] Git state cleanup happens automatically (via Drop)

### Feature Validation

- [ ] P1.M2.T3.S1: Push rejected when behind - test verifies error message and rejection
- [ ] P1.M2.T3.S2: Push succeeds with --force - test verifies force flag bypasses check
- [ ] P1.M2.T3.S3: Push succeeds when up-to-date - test verifies no error
- [ ] Edge cases covered: divergent, ahead, multiple layers
- [ ] Error messages contain remediation steps ("jin pull", "--force")

### Code Quality Validation

- [ ] Tests follow existing patterns in sync_workflow.rs
- [ ] Test names clearly indicate scenario being tested
- [ ] Tests have doc comments explaining purpose
- [ ] No hardcoded values that could cause parallel test failures
- [ ] Each test is independent (can run in any order)

### Documentation & Deployment

- [ ] Tests document expected behavior through examples
- [ ] Test code is readable and maintainable
- [ ] Complex Git state manipulation is well-commented

## Anti-Patterns to Avoid

- **Don't** use global ~/.jin for tests - always use isolated JIN_DIR
- **Don't** skip test isolation - unique_test_id() is required for parallel tests
- **Don't** use jin commands to manipulate remote in tests - use git2 directly
- **Don't** create tests that depend on execution order - each test must be independent
- **Don't** ignore clippy warnings - fix them before considering tests complete
- **Don't** use std::process::id() for unique IDs - use unique_test_id() instead
- **Don't** forget to set JIN_DIR environment variable for test isolation
- **Don't** create tests that leave Git lock files - fixtures handle cleanup automatically
- **Don't** write tests without verifying the actual Git state - use assertions
- **Don't** assume existing tests are complete - verify and enhance as needed

## Appendix: Test Scenario Reference

### Git State Scenarios

| Scenario | Local OID | Remote OID | Expected Behavior | Test Name Pattern |
|----------|-----------|------------|-------------------|-------------------|
| Up-to-date | A | A | Success or "Nothing to push" | `test_push_succeeds_when_up_to_date` |
| Ahead | B (A→B) | A | Success (fast-forward) | `test_push_succeeds_when_ahead` |
| Behind | A | B (A→B) | Reject with error | `test_push_rejected_when_behind` |
| Divergent | C (A→C) | D (A→D) | Reject with error | `test_push_rejected_when_divergent` |
| Behind + Force | A | B | Success with warning | `test_push_succeeds_with_force_when_behind` |
| Divergent + Force | C | D | Success with warning | `test_push_succeeds_with_force_when_divergent` |

### Reference Comparison Algorithm

```rust
// From src/git/refs.rs:compare_refs()
let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;

match (ahead, behind) {
    (0, 0) => RefComparison::Equal,     // Same commit
    (_, 0) => RefComparison::Ahead,     // Local has unique commits
    (0, _) => RefComparison::Behind,    // Remote has unique commits
    (_, _) => RefComparison::Diverged,  // Both have unique commits
}
```

### Error Message Format

```
Push rejected: local layer 'refs/jin/layers/mode/test_mode' is behind remote.
The remote contains commits you don't have locally.
Run 'jin pull' to merge remote changes, or use '--force' to overwrite.
WARNING: --force may cause data loss!
```

---

**Confidence Score**: 9/10 for one-pass implementation success

This PRP provides comprehensive context including:
- Exact file locations and line numbers for all referenced code
- Complete implementation patterns with code examples
- All test scenarios mapped to PRD sub-tasks
- Specific validation commands and expected outcomes
- Detailed anti-patterns to avoid
- Git state simulation patterns with examples

The implementing agent has everything needed to successfully complete the integration tests for push enforcement.
