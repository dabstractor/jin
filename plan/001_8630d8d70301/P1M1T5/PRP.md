# Product Requirement Prompt (PRP): P1.M1.T5 - Integration Tests for Conflict Workflow

---

## Goal

**Feature Goal**: Create comprehensive integration tests for the .jinmerge conflict resolution workflow that validate end-to-end behavior from conflict detection through resolution.

**Deliverable**: A new test module `tests/conflict_workflow.rs` containing 10+ integration tests covering all aspects of the conflict resolution workflow.

**Success Definition**: All tests pass with `cargo nextest run --all-features`, covering:
- .jinmerge file creation during apply conflicts
- Paused state persistence and recovery
- Resolve command validation and workflow
- Status command conflict state display
- End-to-end workflow from conflict to resolution
- Error scenarios and edge cases

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test Result**: PASS - This PRP contains:
- Complete codebase structure with specific file paths
- Existing test patterns with code examples
- All module locations and their responsibilities
- Test framework configuration and commands
- Exact data structures and file formats
- Specific URLs for external documentation

### Documentation & References

```yaml
# MUST READ - Conflict workflow implementation
- file: src/merge/jinmerge.rs
  why: Complete .jinmerge file format implementation - data structures, serialization, parsing, validation
  pattern: JinMergeConflict struct, to_jinmerge_format(), parse_jinmerge_content()
  gotcha: Files named as "config.json.jinmerge" (append .jinmerge to original path)
  section: Lines 74-80 (JinMergeConflict), 54-68 (JinMergeRegion), 287-318 (serialization), 324-413 (parsing)

- file: src/commands/apply.rs
  why: Apply command conflict detection, .jinmerge file creation, paused state management
  pattern: handle_conflicts() function, PausedApplyState struct
  gotcha: Apply returns Ok(()) when conflicts detected (not error), writes to .jin/.paused_apply.yaml
  section: Lines 16-42 (PausedApplyState structures), 159-180 (conflict handling), 231-247 (handle_conflicts)

- file: src/commands/resolve.rs
  why: Resolve command validation logic, state updates, apply completion
  pattern: validate_no_conflict_markers(), complete_apply_operation()
  gotcha: Resolved content written atomically, state file deleted when all conflicts resolved
  section: Lines 33-37 (state loading), validation functions, complete_apply_operation()

- file: src/commands/status.rs
  why: Status command conflict state display implementation
  pattern: check_for_conflicts(), show_conflict_state()
  gotcha: Converts original paths to .jinmerge paths for display, graceful degradation if state corrupt
  section: Lines with check_for_conflicts(), show_conflict_state() implementation

- file: tests/cli_apply_conflict.rs
  why: Existing apply conflict tests - use as pattern for new tests
  pattern: setup_test_repo(), unique_test_id(), jin_cmd(), assert_cmd usage
  gotcha: Uses manual .jinmerge file creation in some tests (lines 89-116)
  section: Full file - reference for test structure, assertions, setup patterns

- file: tests/cli_resolve.rs
  why: Existing resolve command tests - use as pattern for new tests
  pattern: Manual paused state creation (lines 99-115), dry-run testing (lines 269-307)
  gotcha: Some tests manually create .jinmerge files to avoid complex layer setup
  section: Full file - reference for resolve testing patterns

- file: tests/common/fixtures.rs
  why: Test fixtures and helper functions
  pattern: TestFixture, setup_test_repo(), create_commit_in_repo(), unique_test_id()
  gotcha: JIN_DIR must be set for test isolation, TempDir must stay in scope
  section: Lines 16-51 (TestFixture), 114-119 (setup_test_repo), 149-200 (create_commit_in_repo), 302-306 (unique_test_id)

# External Documentation
- url: https://rust-cli.github.io/book/tutorial/testing.html
  why: Best practices for testing CLI applications in Rust
  critical: "Test observable behavior" and "Use integration tests for CLI interactions"

- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: assert_cmd crate documentation for CLI testing
  critical: Command::cargo_bin(), .assert(), .success(), .failure()

- url: https://docs.rs/tempfile/latest/tempfile/
  why: tempfile crate documentation for temporary directory management
  critical: TempDir::new(), automatic cleanup on drop

- url: https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory
  why: Rust test organization conventions
  critical: Integration tests go in tests/ directory at project root
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── src/
│   ├── merge/
│   │   ├── jinmerge.rs       # .jinmerge file format module
│   │   ├── layer.rs          # Layer merging logic
│   │   ├── deep.rs           # Deep merge (JSON/YAML)
│   │   ├── text.rs           # Text file merging
│   │   └── mod.rs            # Merge module exports
│   ├── commands/
│   │   ├── apply.rs          # Apply with conflict detection
│   │   ├── resolve.rs        # Resolve command
│   │   ├── status.rs         # Status with conflict display
│   │   └── mod.rs            # Commands module
│   └── ...
├── tests/
│   ├── common/
│   │   ├── fixtures.rs       # Test fixtures (TestFixture, setup_test_repo)
│   │   ├── assertions.rs     # Custom assertions
│   │   ├── git_helpers.rs    # Git utilities
│   │   └── mod.rs            # Common module
│   ├── cli_apply_conflict.rs # Existing apply conflict tests
│   ├── cli_resolve.rs        # Existing resolve tests
│   ├── cli_basic.rs          # Basic CLI tests
│   └── ... (other test files)
└── Cargo.toml                # Test dependencies: assert_cmd, predicates, tempfile, serial_test
```

### Desired Codebase Tree (Additions Only)

```bash
tests/
├── conflict_workflow.rs      # NEW: End-to-end conflict workflow tests
│   ├── test_full_workflow_conflict_to_resolution()
│   ├── test_status_shows_conflicts_during_pause()
│   ├── test_resolve_validates_conflict_markers()
│   ├── test_apply_creates_multiple_jinmerge_files()
│   ├── test_resolve_partial_updates_state()
│   ├── test_error_scenarios()
│   └── ... (10+ tests total)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Test isolation requires JIN_DIR to be set
// Always use setup_test_repo() which sets isolated JIN_DIR
// Failure leads to tests interfering with each other

// CRITICAL: TempDir must be stored in TestFixture._tempdir
// If _tempdir goes out of scope, directory is deleted immediately
// Keep TestFixture in scope throughout test

// CRITICAL: Git locks must be cleaned up before temp dir deletion
// TestFixture Drop impl handles this automatically
// Do NOT manually clean up temp directories

// CRITICAL: apply command returns Ok(()) on conflicts (not Err)
// Check for "Operation paused" in stdout, not .failure()
// Paused state file (.jin/.paused_apply.yaml) is created

// CRITICAL: .jinmerge files are named by appending .jinmerge
// Original: "config.json" -> .jinmerge: "config.json.jinmerge"
// Use JinMergeConflict::merge_path_for_file() for consistency

// CRITICAL: unique_test_id() uses atomic counter for true uniqueness
// std::process::id() alone is insufficient for parallel tests
// Always use unique_test_id() for resource naming

// CRITICAL: Conflict markers must be removed before resolve succeeds
// Markers: "<<<<<<<", "=======", ">>>>>>>"
// validate_no_conflict_markers() checks for these exact strings

// CRITICAL: cargo nextest is used for test execution (not cargo test)
// CI uses: cargo nextest run --all-features
// Validation command must use nextest

// CRITICAL: create_commit_in_repo configures Git user if not set
// Tests that create commits must use this helper
// Direct git2 operations may fail without user.email/user.name

// CRITICAL: Status command gracefully handles corrupted paused state
// Returns None if load fails, doesn't crash
// Tests should verify graceful degradation

// CRITICAL: resolve command validates file is in conflict_files list
// Cannot resolve arbitrary files - must be in paused state
// Tests must set up paused state with correct conflict_files

// CRITICAL: PausedApplyState stores original paths (not .jinmerge paths)
// Status converts to .jinmerge paths for display
// Tests must verify correct path conversion
```

---

## Implementation Blueprint

### Data Models and Structures

The tests use existing data structures from the implementation:

```rust
// From src/merge/jinmerge.rs - conflict file format
pub struct JinMergeConflict {
    pub file_path: PathBuf,      // Original file path
    pub conflicts: Vec<JinMergeRegion>,
}

pub struct JinMergeRegion {
    pub layer1_ref: String,
    pub layer1_content: String,
    pub layer2_ref: String,
    pub layer2_content: String,
    pub start_line: usize,
    pub end_line: usize,
}

// From src/commands/apply.rs - paused state
pub struct PausedApplyState {
    pub timestamp: DateTime<Utc>,
    pub layer_config: PausedLayerConfig,
    pub conflict_files: Vec<PathBuf>,  // Original paths
    pub applied_files: Vec<PathBuf>,
    pub conflict_count: usize,
}

// .jinmerge file format (text representation)
// Lines 1: "# Jin merge conflict. Resolve and run 'jin resolve <file>'"
// Conflict markers: "<<<<<<< layer_ref", "=======", ">>>>>>> layer_ref"
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/conflict_workflow.rs
  - IMPLEMENT: Integration test module for conflict workflow
  - FOLLOW pattern: tests/cli_apply_conflict.rs (module structure, test patterns)
  - NAMING: snake_case file name, test_* function names
  - DEPENDENCIES: Existing test framework (assert_cmd, predicates, tempfile)
  - PLACEMENT: tests/ directory at project root

Task 2: IMPLEMENT test_full_workflow_conflict_to_resolution()
  - IMPLEMENT: Complete end-to-end workflow test
  - FOLLOW pattern: tests/cli_apply_conflict.rs::test_apply_with_conflicts_creates_jinmerge_files (setup)
  - VERIFY: apply creates .jinmerge, resolve accepts valid resolution, status shows no conflicts
  - COVERAGE: All three commands (apply, resolve, status) in single test
  - PLACEMENT: First test in conflict_workflow.rs

Task 3: IMPLEMENT test_status_shows_conflicts_during_pause()
  - IMPLEMENT: Status command conflict state display test
  - FOLLOW pattern: tests/cli_apply_conflict.rs::test_apply_with_conflicts_creates_paused_state
  - VERIFY: Status shows conflict count, .jinmerge files, resolve instruction, timestamp
  - COVERAGE: PausedApplyState display formatting
  - PLACEMENT: conflict_workflow.rs

Task 4: IMPLEMENT test_resolve_validates_conflict_markers()
  - IMPLEMENT: Resolve marker validation test
  - FOLLOW pattern: tests/cli_resolve.rs::test_resolve_invalid_markers (lines 211-251)
  - VERIFY: Resolve fails when markers present, succeeds when markers removed
  - COVERAGE: validate_no_conflict_markers() function
  - PLACEMENT: conflict_workflow.rs

Task 5: IMPLEMENT test_apply_creates_multiple_jinmerge_files()
  - IMPLEMENT: Multiple conflict scenario test
  - FOLLOW pattern: tests/cli_apply_conflict.rs::test_apply_with_multiple_conflicts (lines 382-453)
  - VERIFY: All conflicting files get .jinmerge files, paused state tracks all conflicts
  - COVERAGE: Multiple file conflict handling
  - PLACEMENT: conflict_workflow.rs

Task 6: IMPLEMENT test_resolve_partial_updates_state()
  - IMPLEMENT: Partial resolution test
  - FOLLOW pattern: tests/cli_resolve.rs::test_resolve_partial_conflicts (lines 343-395)
  - VERIFY: Resolving subset of conflicts updates paused state correctly
  - COVERAGE: Partial conflict resolution workflow
  - PLACEMENT: conflict_workflow.rs

Task 7: IMPLEMENT test_resolve_all_completes_apply_operation()
  - IMPLEMENT: Full resolution completion test
  - FOLLOW pattern: tests/cli_resolve.rs::test_resolve_all_conflicts (lines 142-209)
  - VERIFY: Resolve --all completes apply operation, deletes paused state, updates workspace metadata
  - COVERAGE: Apply completion after full resolution
  - PLACEMENT: conflict_workflow.rs

Task 8: IMPLEMENT test_error_scenarios()
  - IMPLEMENT: Error handling tests (multiple test functions)
  - FOLLOW pattern: tests/cli_resolve.rs (error testing patterns)
  - VERIFY: resolve without paused state, resolve non-conflict file, resolve with markers, empty resolution
  - COVERAGE: All error code paths in resolve command
  - PLACEMENT: conflict_workflow.rs

Task 9: IMPLEMENT test_apply_non_conflicting_files_still_applied()
  - IMPLEMENT: Mixed conflict/success test
  - FOLLOW pattern: tests/cli_apply_conflict.rs::test_apply_with_conflicts_applies_non_conflicting_files (lines 194-298)
  - VERIFY: Non-conflicting files are applied even when conflicts exist
  - COVERAGE: Partial success scenario
  - PLACEMENT: conflict_workflow.rs

Task 10: IMPLEMENT test_dry_run_modes()
  - IMPLEMENT: Dry-run testing for apply and resolve
  - FOLLOW pattern: tests/cli_apply_conflict.rs::test_apply_dry_run_with_conflicts_shows_preview (lines 300-380)
  - VERIFY: Dry-run shows preview without writing files or state
  - COVERAGE: --dry-run flag behavior
  - PLACEMENT: conflict_workflow.rs
```

### Implementation Patterns & Key Details

```rust
// Pattern: Test fixture setup (from tests/common/fixtures.rs:114-119)
fn setup_test_repo() -> Result<TestFixture, Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // CRITICAL: Sets isolated JIN_DIR
    jin_init(fixture.path())?;
    Ok(fixture)
}

// Pattern: Creating conflict scenario for testing
// Follow tests/cli_apply_conflict.rs:22-112
#[test]
fn test_apply_creates_jinmerge() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // 1. Create mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd().args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 2. Activate mode
    jin_cmd().args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 3. Add file to global layer
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();
    jin_cmd().args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 4. Commit to global
    create_commit_in_repo(fixture.path(), "config.json", r#"{"port": 8080}"#, "Add to global")
        .unwrap();

    // 5. Modify and add to mode layer (creates conflict)
    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();
    jin_cmd().args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 6. Commit to mode
    create_commit_in_repo(fixture.path(), "config.json", r#"{"port": 9090}"#, "Add to mode")
        .unwrap();

    // 7. Remove from workspace
    fs::remove_file(&config_path).unwrap();

    // 8. Run apply - should create .jinmerge
    jin_cmd().arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // 9. Verify .jinmerge file
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists());
}

// Pattern: Manual paused state creation (from tests/cli_resolve.rs:99-115)
// Use when full layer setup is too complex
fn create_paused_state(fixture: &TestFixture, conflict_files: &[&str]) {
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();

    let conflict_list = conflict_files.iter()
        .map(|f| format!("  - {}", f))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(
        &paused_state_path,
        format!(r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: Some("test_mode")
  scope: None
  project: None
conflict_files:
{}
applied_files: []
conflict_count: {}
"#, conflict_list, conflict_files.len())
    ).unwrap();
}

// Pattern: Creating .jinmerge file for testing
// From tests/cli_resolve.rs:91-96
fn create_jinmerge_file(path: &Path, content: &str) {
    fs::write(
        path.with_extension("jinmerge"),
        format!("# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{}", content)
    ).unwrap();
}

// GOTCHA: Apply returns success even with conflicts
// Check stdout for pause message, not .failure()
jin_cmd().arg("apply")
    .assert()
    .success()  // NOT .failure()
    .stdout(predicate::str::contains("Operation paused"));

// GOTCHA: Verify .jinmerge path format
let jinmerge_path = fixture.path().join("config.json.jinmerge");  // NOT "config.jinmerge"
assert!(jinmerge_path.exists());

// GOTCHA: Status command graceful degradation
// Returns None if paused state corrupted, doesn't crash
// Test should verify this behavior
```

### Integration Points

```yaml
TEST_FRAMEWORK:
  - framework: "Rust built-in test + assert_cmd + predicates + tempfile"
  - runner: "cargo nextest run --all-features"
  - isolation: "TestFixture with JIN_DIR environment variable"
  - pattern: "Integration tests in tests/ directory"

TEST_HELPERS:
  - use: "tests/common/fixtures.rs"
  - functions: "setup_test_repo(), unique_test_id(), create_commit_in_repo()"
  - gotcha: "Always call setup_test_repo() for test isolation"

MOCK_DATA:
  - jinmerge files: "Manually create with fs::write() for simplicity"
  - paused state: "Manually create YAML to avoid complex layer setup"
  - conflicts: "Use same file in global + mode layers with different content"

ASSERTIONS:
  - file_exists: "assert!(path.exists())"
  - file_not_exists: "assert!(!path.exists())"
  - content_contains: "predicate::str::contains(\"text\")"
  - command_success: ".assert().success()"
  - command_failure: ".assert().failure()"
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after test file creation - fix before proceeding
cargo fmt --check tests/conflict_workflow.rs     # Check formatting
cargo clippy --tests -- -D warnings              # Lint tests

# Format and fix
cargo fmt tests/conflict_workflow.rs
cargo clippy --tests --fix --allow-dirty --allow-staged

# Expected: Zero warnings, formatted code. If errors exist, READ output and fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run individual test file
cargo nextest run --all-features conflict_workflow

# Run with verbose output
cargo nextest run --all-features conflict_workflow --verbose

# Run specific test
cargo nextest run --all-features test_full_workflow_conflict_to_resolution

# Expected: All tests in conflict_workflow.rs pass. If failing, debug root cause.
```

### Level 3: Integration Testing (System Validation)

```bash
# Full test suite for conflict-related tests
cargo nextest run --all-features cli_apply_conflict cli_resolve conflict_workflow

# Verify all conflict tests pass together
cargo nextest run --all-features --test-pattern 'conflict'

# Expected: All conflict-related tests pass, no cross-test interference.
```

### Level 4: End-to-End Validation

```bash
# Manual workflow verification (optional, for development)
# 1. Setup test repository
cd /tmp && mkdir test_conflict && cd test_conflict
git init
jin init --dir .jin_test
export JIN_DIR=$(pwd)/.jin_test

# 2. Create mode
jin mode create test_mode
jin mode use test_mode

# 3. Create conflict scenario
echo '{"port": 8080}' > config.json
jin add config.json --global
jin commit -m "Add to global"

echo '{"port": 9090}' > config.json
jin add config.json --mode
jin commit -m "Add to mode"

rm config.json

# 4. Run apply (should create conflict)
jin apply
ls config.json.jinmerge  # Should exist
cat .jin/.paused_apply.yaml  # Should exist

# 5. Check status
jin status  # Should show conflicts

# 6. Resolve conflict
echo '{"port": 9999}' > config.json.jinmerge
jin resolve config.json

# 7. Verify resolution
cat config.json  # Should show resolved content
ls config.json.jinmerge  # Should NOT exist
ls .jin/.paused_apply.yaml  # Should NOT exist

# Expected: Smooth workflow from conflict to resolution.
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo nextest run --all-features conflict_workflow`
- [ ] No linting errors: `cargo clippy --tests -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check tests/conflict_workflow.rs`
- [ ] Tests run in isolation without interference
- [ ] Test cleanup is automatic (TestFixture Drop impl)

### Feature Validation

- [ ] test_full_workflow_conflict_to_resolution: Complete workflow from apply to resolve
- [ ] test_status_shows_conflicts_during_pause: Status displays conflict state correctly
- [ ] test_resolve_validates_conflict_markers: Marker validation works
- [ ] test_apply_creates_multiple_jinmerge_files: Multiple conflicts handled
- [ ] test_resolve_partial_updates_state: Partial resolution updates state
- [ ] test_resolve_all_completes_apply_operation: Full resolution completes apply
- [ ] test_error_scenarios: All error cases covered
- [ ] test_apply_non_conflicting_files_still_applied: Mixed conflict/success works
- [ ] test_dry_run_modes: Dry-run doesn't modify state
- [ ] Additional edge case tests as needed

### Code Quality Validation

- [ ] Follows existing test patterns from cli_apply_conflict.rs and cli_resolve.rs
- [ ] Uses setup_test_repo() for all tests requiring Jin
- [ ] Uses unique_test_id() for all unique resource names
- [ ] Uses assert_cmd Command patterns consistently
- [ ] Uses predicates for output assertions
- [ ] Test names are descriptive (test_<action>_<scenario>)
- [ ] Comments explain complex test scenarios
- [ ] JIN_DIR isolation is maintained throughout

### Test Coverage

- [ ] .jinmerge file creation (apply conflicts)
- [ ] Paused state persistence (.jin/.paused_apply.yaml)
- [ ] Resolve validation (marker checking)
- [ ] Resolve workflow (apply to workspace, delete .jinmerge)
- [ ] Status display (conflict state)
- [ ] Partial resolution (state updates)
- [ ] Full resolution (apply completion)
- [ ] Error scenarios (no state, invalid file, markers present)
- [ ] Multiple conflicts
- [ ] Mixed conflict/success (non-conflicting files still applied)
- [ ] Dry-run modes (no side effects)

---

## Anti-Patterns to Avoid

- ❌ Don't skip test isolation - always use setup_test_repo()
- ❌ Don't use std::process::id() alone - use unique_test_id()
- ❌ Don't manually clean up temp directories - TestFixture handles this
- ❌ Don't expect apply to fail on conflicts - it returns success with pause message
- ❌ Don't forget to set JIN_DIR - tests will interfere with each other
- ❌ Don't create .jinmerge files with wrong naming - must append .jinmerge
- ❌ Don't use cargo test for validation - use cargo nextest
- ❌ Don't commit in tests without create_commit_in_repo() helper
- ❌ Don't let TempDir go out of scope - store in TestFixture
- ❌ Don't ignore Git lock cleanup - TestFixture Drop handles this
- ❌ Don't test resolve without creating paused state first
- ❌ Don't expect resolve to succeed with conflict markers present
- ❌ Don't forget to verify .jinmerge files are deleted after resolve
- ❌ Don't assume paused state is deleted on partial resolve
- ❌ Don't use hardcoded resource names - use unique_test_id()

---

## Additional Research References

### Stored Research Files

```yaml
- docfile: plan/P1M1T1/research/
  why: .jinmerge file format module implementation details
  contains: Data structures, serialization, parsing logic

- docfile: plan/P1M1T2/research/
  why: Apply command conflict detection and state management
  contains: Conflict collection, .jinmerge generation, paused state

- docfile: plan/P1M1T3/research/
  why: Resolve command implementation details
  contains: Validation logic, state updates, apply completion

- docfile: plan/P1M1T4/research/
  why: Status command conflict state display
  contains: Paused state detection, display formatting

- docfile: plan/docs/test_structure_analysis.md
  why: Comprehensive test patterns and conventions
  contains: Test organization, fixtures, assertions, cleanup

- docfile: plan/docs/test_isolation_issues.md
  why: Common test isolation pitfalls and solutions
  contains: JIN_DIR setup, Git lock cleanup, unique naming
```

---

## Confidence Score

**8.5/10** - High confidence in one-pass implementation success

**Justification**:
- Complete context with specific file paths and line numbers
- Existing test patterns fully documented with code examples
- All data structures and file formats specified
- Test framework and validation commands identified
- Known gotchas documented with workarounds
- Existing tests provide clear patterns to follow

**Remaining risk factors**:
- Complex layer setup for true conflict scenarios may require manual .jinmerge creation
- Some edge cases may need test refinement during implementation
- Parallel test execution may reveal isolation issues (addressed with unique_test_id())

---

## Success Metrics

**Quantitative**:
- 10+ test functions implemented
- 100% of success criteria in test list pass
- Zero clippy warnings
- All tests pass with cargo nextest

**Qualitative**:
- Tests follow existing patterns consistently
- Test names clearly indicate what is being tested
- Tests are isolated and can run in parallel
- Tests cover all conflict workflow scenarios
- Tests validate both success and error paths
