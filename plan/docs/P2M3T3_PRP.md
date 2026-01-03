name: "P2.M3.T3: Integration Tests for Active Context Notifications in Fetch"
description: |

---

## Goal

**Feature Goal**: Verify the fetch command correctly highlights active context updates and separates other updates through comprehensive integration tests.

**Deliverable**: Two new integration test functions in `tests/sync_workflow.rs`:
1. `test_fetch_highlights_active_mode_updates` - Validates active mode updates are prominently displayed
2. `test_fetch_separates_active_and_other_updates` - Validates proper separation of update categories

**Success Definition**: Tests pass consistently, validating that:
- Active mode/scope updates appear in "Updates for your active context" section
- Other mode/scope updates appear in "Other updates" section
- Section headers correctly indicate active mode/scope
- Tests use proper fixtures and isolation patterns

## User Persona (if applicable)

**Target User**: Development team ensuring fetch command behavior correctness

**Use Case**: Validate that the active context filtering implementation (P2.M3.T2) works as expected

**User Journey**:
1. Developer runs `cargo test test_fetch_highlights_active_mode_updates`
2. Test creates remote repository with multiple mode updates
3. Test sets active mode locally
4. Test executes fetch and verifies output structure
5. Test confirms active mode updates are highlighted

**Pain Points Addressed**:
- Ensures user-visible behavior matches specification
- Prevents regressions in active context highlighting
- Validates edge cases (default context, multiple modes)

## Why

- **Feature Validation**: P2.M3.T2 implemented active context filtering - tests verify it works
- **Regression Prevention**: Future changes won't break user-facing notification behavior
- **Documentation**: Tests serve as executable specification of expected output format
- **Edge Case Coverage**: Validates behavior with default context, multiple modes, scopes

## What

Integration tests for fetch command's active context notification feature.

### Success Criteria

- [ ] `test_fetch_highlights_active_mode_updates` passes
- [ ] `test_fetch_separates_active_and_other_updates` passes
- [ ] Tests follow existing fixture patterns from `tests/common/fixtures.rs`
- [ ] Tests use proper JIN_DIR isolation
- [ ] Tests validate both success case (active context found) and default context case

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: YES. This PRP includes:
- Exact fetch command implementation details
- Existing test patterns and fixtures
- Complete ProjectContext structure
- External testing best practices with specific URLs
- Code snippets showing expected behavior

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://rust-cli.github.io/book/tutorial/testing.html
  why: Official Rust CLI testing guide using assert_cmd
  critical: Use assert_cmd for CLI command execution, predicates for output validation

- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: assert_cmd crate API documentation
  critical: Command::cargo_bin(), assert(), success(), failure(), stdout(), stderr()

- url: https://docs.rs/predicates/latest/predicates/
  why: predicates crate for readable output assertions
  critical: predicate::str::contains() for substring matching

- file: /home/dustin/projects/jin/src/commands/fetch.rs
  why: Fetch command implementation with active context filtering logic
  pattern: is_ref_relevant_to_context() function (lines 214-247) shows filtering rules
  gotcha: Context is loaded with graceful fallback - defaults to ProjectContext::default() if not initialized

- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  why: Existing fetch tests (test_fetch_loads_context, test_fetch_highlights_active_mode_updates, test_fetch_separates_active_and_other_updates)
  pattern: RemoteFixture setup, JIN_DIR isolation, create_mode helper usage
  gotcha: Notice tests use setup_jin_with_remote() for both local and remote repos

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Reusable test fixtures and helper functions
  pattern: setup_jin_with_remote(), create_mode(), unique_test_id()
  gotcha: Always use isolated JIN_DIR to prevent test interference

- file: /home/dustin/projects/jin/src/core/config.rs
  why: ProjectContext structure definition (lines 88-107)
  pattern: ProjectContext has mode, scope, project, version, last_updated fields
  gotcha: Context loads from .jin/context, use ProjectContext::load() with graceful error handling

- file: /home/dustin/projects/jin/tests/common/mod.rs
  why: Common test imports and jin() command builder
  pattern: use assert_cmd::Command; let cmd = jin();
  gotcha: jin() returns Command::new(env!("CARGO_BIN_EXE_jin"))
```

### Current Codebase tree (run `tree` in the root of the project) to get an overview of the codebase

```bash
jin/
├── src/
│   ├── commands/
│   │   ├── fetch.rs          # Fetch command with active context filtering (P2.M3.T2)
│   │   └── ...
│   ├── core/
│   │   ├── config.rs         # ProjectContext definition
│   │   └── error.rs          # JinError types
│   └── ...
├── tests/
│   ├── sync_workflow.rs      # Target file - add tests here
│   ├── common/
│   │   ├── fixtures.rs       # Test fixtures (setup_jin_with_remote, create_mode)
│   │   ├── assertions.rs     # Custom assertions
│   │   ├── git_helpers.rs    # Git lock cleanup
│   │   └── mod.rs            # Common imports
│   └── ...
└── plan/
    └── P2M3T3/
        ├── research/         # Store external research here
        └── PRP.md            # This file
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
# NO NEW FILES - Tests are added to existing tests/sync_workflow.rs

jin/
└── tests/
    └── sync_workflow.rs      # MODIFIED: Add two new test functions at end
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: Always use isolated JIN_DIR for each test
// Without isolation, tests interfere via shared ~/.jin directory
let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();
jin()
    .arg("fetch")
    .env("JIN_DIR", jin_dir)  // MUST ALWAYS SET
    .assert()
    .success();

// CRITICAL: RemoteFixture uses TempDir that auto-cleans
// Keep remote_fixture in scope to prevent premature cleanup
let remote_fixture = setup_jin_with_remote()?;
// If remote_fixture goes out of scope, temp dirs are deleted

// CRITICAL: ProjectContext load returns JinError::NotInitialized if .jin/context missing
// Fetch command handles this gracefully: ProjectContext::default() is used
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => ProjectContext::default(),
    Err(e) => return Err(e),
};

// GOTCHA: Mode refs use path format: "refs/jin/layers/mode/{mode_name}"
// Scope refs use: "refs/jin/layers/mode/{mode}/scope/{scope}" or "refs/jin/layers/scope/{scope}"
// Filtering logic in is_ref_relevant_to_context() matches these patterns

// GOTCHA: Active context filtering rules:
// - mode/{active_mode} matches -> active context
// - mode/{active_mode}/scope/{active_scope} matches -> active context
// - scope/{active_scope} matches ONLY when mode is None -> active context
// - global ALWAYS matches -> active context
// - project/* NEVER matches -> other updates
// - mode/{other_mode} -> other updates

// PATTERN: Existing tests already validate P2.M3.T2 behavior
// tests/sync_workflow.rs already contains test_fetch_highlights_active_mode_updates (line 1083-1192)
// and test_fetch_separates_active_and_other_updates (line 1201-1313)
// These tests WERE IMPLEMENTED as part of P2.M3.T2 but are listed as "Planned" in plan status
```

## Implementation Blueprint

### Data models and structure

No new data models needed. Tests use existing:
- `ProjectContext` from `src/core/config.rs`
- `TestFixture` and `RemoteFixture` from `tests/common/fixtures.rs`
- `UpdateInfo` internal struct from `src/commands/fetch.rs`

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY existing tests in tests/sync_workflow.rs
  - CHECK: Lines 1083-1192 for test_fetch_highlights_active_mode_updates
  - CHECK: Lines 1201-1313 for test_fetch_separates_active_and_other_updates
  - VERIFY: Tests compile and pass with current codebase
  - ACTION: If tests exist and pass, mark task complete (they may have been implemented)
  - IF MISSING: Proceed to Task 2

Task 2: (CONDITIONAL) Add test_fetch_highlights_active_mode_updates to tests/sync_workflow.rs
  - IMPLEMENT: Test function following existing pattern from test_fetch_loads_context (line 1009)
  - FOLLOW pattern: test_fetch_loads_context structure (RemoteFixture, temp_workspace for remote commits)
  - NAMING: test_fetch_highlights_active_mode_updates
  - PLACEMENT: After test_fetch_with_default_context (around line 1400)
  - DEPENDENCIES: tests/common/fixtures.rs helpers

Task 3: (CONDITIONAL) Add test_fetch_separates_active_and_other_updates to tests/sync_workflow.rs
  - IMPLEMENT: Test function with two-mode scenario (active + other)
  - FOLLOW pattern: test_fetch_highlights_active_mode_updates structure
  - NAMING: test_fetch_separates_active_and_other_updates
  - PLACEMENT: After test_fetch_highlights_active_mode_updates
  - VALIDATION: Both section headers present, correct mode in each section

Task 4: (CONDITIONAL) Add test_fetch_with_default_context to tests/sync_workflow.rs
  - IMPLEMENT: Test with no active mode/scope set
  - VALIDATION: No "Updates for your active context" section, all in "Other updates"
  - PLACEMENT: After test_fetch_separates_active_and_other_updates
```

### Implementation Patterns & Key Details

```rust
// Pattern: Fetch test with active mode highlighting
#[test]
fn test_fetch_highlights_active_mode_updates() -> Result<(), Box<dyn std::error::Error>> {
    // PATTERN: Use RemoteFixture for local + bare remote repos
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // PATTERN: Unique test ID prevents conflicts between parallel tests
    let mode_name = format!("active_test_{}", unique_test_id());
    let other_mode = format!("other_mode_{}", unique_test_id());

    // PATTERN: Create temp workspace to populate remote (don't have push command yet)
    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // PATTERN: Create and commit in active mode on remote
    create_mode(&mode_name, Some(temp_jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("active_file.txt"), "active mode content")?;
    jin()
        .args(["add", "active_file.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .args(["commit", "-m", "Add active mode file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // PATTERN: Create and commit in other mode on remote
    create_mode(&other_mode, Some(temp_jin_dir))?;
    // ... similar steps for other mode ...

    // PATTERN: Push temp workspace to bare remote
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();
    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // PATTERN: Link local repo to remote and set active mode
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    create_mode(&mode_name, Some(jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // PATTERN: Execute fetch and validate output
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Updates for your active context"))
        .stdout(predicate::str::contains(format!("mode: {}", mode_name)));

    Ok(())
}

// Pattern: Test with section separation validation
#[test]
fn test_fetch_separates_active_and_other_updates() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup similar to above ...

    // PATTERN: Capture output for complex validation
    let result = jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    // PATTERN: Validate section structure
    assert!(stdout.contains("Updates for your active context"));
    assert!(stdout.contains("Other updates"));

    // PATTERN: Validate content in correct sections
    assert!(stdout.contains(&format!("mode/{}", active_mode)));
    assert!(stdout.contains(&format!("mode/{}", other_mode)));

    Ok(())
}

// Pattern: Default context test (no active mode)
#[test]
fn test_fetch_with_default_context() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup ...

    // PATTERN: Remove context file to force default context
    let context_path = remote_fixture.local_path.join(".jin").join("context");
    fs::remove_file(&context_path).ok();

    // PATTERN: Validate absence of active context section
    let result = jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    // CRITICAL: With default context, NO active context section
    assert!(!stdout.contains("Updates for your active context"));
    assert!(stdout.contains("Other updates"));

    Ok(())
}
```

### Integration Points

```yaml
FETCH_COMMAND:
  - uses: src/commands/fetch.rs::execute()
  - validates: Active context filtering (is_ref_relevant_to_context)
  - output: Two-section format ("Updates for your active context", "Other updates")

PROJECT_CONTEXT:
  - loads: From .jin/context (or default if missing)
  - fields: mode, scope, project (all Option<String>)

TEST_FIXTURES:
  - setup_jin_with_remote(): Creates local repo + bare remote
  - create_mode(): Creates a mode in Jin repository
  - unique_test_id(): Generates unique test identifier
  - jin(): Returns Command::new(env!("CARGO_BIN_EXE_jin"))

DEPENDENCIES:
  - dev-dependencies in Cargo.toml: assert_cmd = "2.0", predicates = "3.0"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding tests - fix before proceeding
cargo fmt -- tests/sync_workflow.rs              # Auto-format
cargo clippy --tests -- -D warnings 2>&1 | grep sync_workflow  # Check for lints

# Expected: Zero clippy warnings. Fix any issues before proceeding.

# Quick syntax check
cargo check --tests 2>&1 | grep -A5 sync_workflow

# Expected: "Finished" with no errors.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run just the new fetch tests
cargo test test_fetch_highlights_active_mode_updates --test sync_workflow -- --exact
cargo test test_fetch_separates_active_and_other_updates --test sync_workflow -- --exact
cargo test test_fetch_with_default_context --test sync_workflow -- --exact

# Run all fetch-related tests
cargo test test_fetch --test sync_workflow -- --nocapture

# Expected: All tests pass with output showing:
# - "Updates for your active context" section with active mode
# - "Other updates" section with other modes
# - Proper separation between sections

# Run with output to see actual fetch output
cargo test test_fetch_highlights_active_mode_updates --test sync_workflow -- --exact --nocapture
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all sync workflow tests to ensure no regressions
cargo test --test sync_workflow

# Expected: All sync_workflow tests pass, including existing tests:
# - test_link_to_local_remote
# - test_fetch_updates_refs
# - test_fetch_loads_context
# - test_fetch_highlights_active_mode_updates (NEW)
# - test_fetch_separates_active_and_other_updates (NEW)
# - test_fetch_with_default_context (NEW)
# - And all other existing tests

# Run all integration tests
cargo test --test '*'

# Expected: Zero test failures across all test files
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Manual verification of fetch output formatting
# Create a real scenario and inspect output
cargo run -- fetch

# Inspect that output sections are visually distinct
# "Updates for your active context (mode: X):" should appear first
# "Other updates:" should appear second (if there are other updates)

# Test edge cases manually
# 1. No updates at all (empty remote)
cargo run -- fetch  # Should show nothing or "No updates available"

# 2. Only active context updates (no other modes)
#    Set active mode, fetch when only that mode has updates
#    Should show only "Updates for your active context" section

# 3. Only other updates (default context, no active mode)
#    Remove .jin/context, fetch when remote has updates
#    Should show only "Other updates" section

# 4. Multiple active context updates (mode + scope)
#    Set both mode and scope, fetch when both have updates
#    Both should appear in active context section
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] New tests compile: `cargo check --tests`
- [ ] New tests pass: `cargo test test_fetch --test sync_workflow`
- [ ] No regressions: All existing tests still pass
- [ ] Code formatted: `cargo fmt -- tests/sync_workflow.rs`
- [ ] No clippy warnings: `cargo clippy --tests`

### Feature Validation

- [ ] `test_fetch_highlights_active_mode_updates` validates active mode in prominent section
- [ ] `test_fetch_separates_active_and_other_updates` validates proper section separation
- [ ] `test_fetch_with_default_context` validates behavior with no active context
- [ ] Tests follow existing fixture patterns (RemoteFixture, JIN_DIR isolation)
- [ ] Tests use unique_test_id() to prevent conflicts

### Code Quality Validation

- [ ] Tests follow naming convention: test_fetch_*
- [ ] Tests use Result<(), Box<dyn std::error::Error>> return type
- [ ] Tests use predicate::str::contains() for readable assertions
- [ ] Tests use jin() command builder from common/mod.rs
- [ ] Tests use helpers from common/fixtures.rs (setup_jin_with_remote, create_mode)

### Documentation & Deployment

- [ ] Test functions include doc comments explaining what they validate
- [ ] Test names clearly indicate scenario being tested
- [ ] No hardcoded values that could cause test flakiness (use unique_test_id())

---

## Anti-Patterns to Avoid

- ❌ Don't create new test files - add to existing tests/sync_workflow.rs
- ❌ Don't skip JIN_DIR isolation - always set .env("JIN_DIR", jin_dir)
- ❌ Don't use hardcoded mode names - use unique_test_id() to prevent conflicts
- ❌ Don't let RemoteFixture go out of scope prematurely - keep it in scope
- ❌ Don't forget to push from temp workspace to bare remote
- ❌ Don't create mode locally without linking to remote first
- ❌ Don't use complex regex when simple predicate::str::contains() works
- ❌ Don't test internal implementation details - test user-visible output
