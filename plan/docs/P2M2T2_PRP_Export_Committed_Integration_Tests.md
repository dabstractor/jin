# PRP: P2.M2.T2 - Integration Tests for Export Committed Files

---

## Goal

**Feature Goal**: Create integration tests that validate the export command can export files committed to Jin layers without requiring them to be in the staging index, verifying JinMap lookups and layer content extraction.

**Deliverable**: A new test module `tests/export_committed.rs` containing integration tests covering:
1. Export of committed files without staging (happy path)
2. Export rejection for untracked files
3. Export from different layer types (global, mode, project, scope)
4. Edge cases (file modified locally, JinMap missing, layer ref missing)

**Success Definition**:
- All tests pass with `cargo nextest run --all-features export_committed`
- Tests validate JinMap lookup functionality
- Tests verify layer content extraction works correctly
- Tests verify export rejects files not in JinMap
- Tests are isolated (use TempDir, JIN_DIR environment variable)
- Tests follow existing patterns from tests/repair_check.rs and tests/common/fixtures.rs

---

## User Persona

**Target User**: Jin developer verifying the P2.M2.T1 implementation (JinMap export validation) works correctly.

**Use Case**: Developer has implemented the feature to export committed files using JinMap lookups and needs to verify the implementation through integration tests.

**User Journey**:
1. Developer implements P2.M2.T1 (JinMap export validation)
2. Developer runs integration tests to verify behavior
3. Tests confirm export works for committed files
4. Tests confirm export rejects untracked files
5. Tests provide examples of expected behavior

**Pain Points Addressed**:
- **Manual testing is slow**: Automated tests verify export functionality quickly
- **Incomplete coverage**: Systematic test scenarios ensure all code paths tested
- **Regression bugs**: Tests catch breaking changes to JinMap export logic
- **Unclear behavior**: Tests document expected export behavior for committed files

---

## Why

- **PRD Requirement**: Task P2.M2.T2 requires integration tests for exporting committed files
- **P2.M2.T1 Verification**: The P2.M2.T1 implementation added JinMap export validation that needs testing
- **Workflow completeness**: Export should work for both staged and committed files
- **Integration with existing feature**: JinMap already tracks layer-to-file mappings; tests verify export uses it correctly

---

## What

### User-Visible Behavior

Integration tests validate the following export scenarios:

**Scenario 1: Export committed file without staging**
```bash
jin init
echo '{"port": 8080}' > config.json
jin add config.json
jin commit -m "Add config"
# File now committed, not staged
jin export config.json
# Expected: Success - file exported via JinMap lookup
```

**Scenario 2: Export rejects untracked files**
```bash
jin init
echo '{"port": 8080}' > config.json
# File not added or committed
jin export config.json
# Expected: Error - "not Jin-tracked"
```

**Scenario 3: Export from mode layer**
```bash
jin init
jin mode create dev
jin mode use dev
echo '{"debug": true}' > config.json
jin add config.json --mode
jin commit -m "Add debug config"
jin export config.json
# Expected: Success - file exported from mode layer
```

### Success Criteria

- [ ] Export succeeds for files in JinMap but not staging
- [ ] Export fails for files not in JinMap with clear error message
- [ ] Export works for all layer types (global, mode, project, scope)
- [ ] Tests verify JinMap lookup is used
- [ ] Tests verify layer content extraction works
- [ ] Tests follow existing patterns (TestFixture, jin_cmd(), assert_cmd)

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test Result**: PASS - This PRP contains:
- Complete P2.M2.T1 implementation summary with specific file paths
- Existing test patterns from tests/repair_check.rs and tests/common/fixtures.rs
- Integration test framework patterns from plan/P6M2/research/
- All module locations and their responsibilities
- Test framework configuration and commands
- Specific code patterns for test setup and assertions
- External documentation references

### Documentation & References

```yaml
# MUST READ - P2.M2.T1 Implementation to Test
- file: src/commands/export.rs
  why: Complete export implementation with JinMap validation - test this code
  pattern: validate_jin_tracked() function (lines 136-195), export_file() (lines 103-134)
  gotcha: Export checks staging first (fast path), then JinMap for committed files
  section: Lines 152-194 for JinMap validation logic

- file: src/core/jinmap.rs
  why: JinMap data structures and load/save functionality
  pattern: JinMap::load() (lines 94-108), contains_file() (lines 220-225), get_layer_files() (lines 210-213)
  gotcha: load() returns JinMap::default() if file doesn't exist (first-run pattern)
  section: Lines 37-91 for JinMap struct, 94-108 for load()

- file: src/git/tree.rs
  why: TreeOps trait for reading file contents from Git trees
  pattern: read_file_from_tree() (lines 133-136), list_tree_files() (lines 138-157)
  gotcha: Path is relative to tree root, use Path::new(file_path)
  section: Lines 133-157 for tree operations

# MUST READ - Test Patterns to Follow
- file: tests/repair_check.rs
  why: Integration test pattern using TestFixture, jin_cmd(), assert_cmd
  pattern: test_repair_check_success_when_attached() (lines 22-46) for basic structure
  gotcha: Uses fixture.set_jin_dir() to set JIN_DIR for isolation
  section: Full file - reference for test structure, assertions, setup patterns

- file: tests/common/fixtures.rs
  why: Test fixtures and helper functions - TestFixture, setup_test_repo(), unique_test_id()
  pattern: TestFixture::new() (lines 27-36), set_jin_dir() (lines 43-50)
  gotcha: TempDir must be stored in TestFixture._tempdir or it's deleted immediately
  section: Lines 16-51 for TestFixture, 114-130 for jin_init()

- file: tests/common/git_helpers.rs
  why: Git utilities for cleanup
  pattern: cleanup_git_locks() function
  gotcha: TestFixture Drop calls this automatically
  section: Full file for git lock cleanup patterns

- file: tests/cli_apply_conflict.rs
  why: CLI testing patterns with assert_cmd
  pattern: Uses jin_cmd() helper from fixtures.rs, assert_cmd assertions
  gotcha: Set JIN_DIR env var for test isolation
  section: Lines referencing jin_cmd(), fixture.set_jin_dir()

# External Documentation
- file: plan/P6M2/research/05_e2e_workflow_testing.md
  why: End-to-end multi-step workflow test patterns
  critical: Sequential command testing, state verification patterns
  section: Lines 47-91 for workflow testing patterns

- file: plan/P6M2/research/02_assert_cmd_and_predicates.md
  why: assert_cmd and predicates crate usage
  critical: Command::cargo_bin(), .assert(), .success(), predicate::str::contains()
  section: Lines 1-100 for assert_cmd patterns

- file: plan/P6M2/research/03_tempfile_and_fixtures.md
  why: TempDir usage and fixture organization
  critical: TempDir::new() for isolation, auto-cleanup on drop
  section: Lines 78-120 for TestFixture pattern

- file: plan/P2M2T1/PRP.md
  why: The PRP for P2.M2.T1 implementation - what we're testing
  critical: Understanding the feature being tested
  section: Full file for complete implementation details
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── src/
│   ├── commands/
│   │   ├── export.rs          # IMPLEMENTATION UNDER TEST - JinMap export validation
│   │   └── ...
│   ├── core/
│   │   ├── jinmap.rs          # USED BY EXPORT - JinMap data structures
│   │   ├── config.rs          # USED BY EXPORT - ProjectContext::load()
│   │   └── ...
│   ├── git/
│   │   ├── tree.rs            # USED BY EXPORT - TreeOps for layer content extraction
│   │   ├── refs.rs            # USED BY EXPORT - RefOps for resolving layer refs
│   │   └── ...
│   └── ...
├── tests/
│   ├── common/
│   │   ├── fixtures.rs        # USE: TestFixture, jin_init(), setup_test_repo()
│   │   ├── git_helpers.rs     # USE: cleanup_git_locks()
│   │   └── mod.rs             # Common module exports
│   ├── cli_basic.rs           # REFERENCE: Basic CLI test patterns
│   ├── repair_check.rs        # REFERENCE: Integration test structure
│   ├── cli_apply_conflict.rs  # REFERENCE: jin_cmd() usage patterns
│   └── ... (other test files)
└── Cargo.toml                  # dev-dependencies: assert_cmd, predicates, tempfile
```

### Desired Codebase Tree (Additions Only)

```bash
tests/
├── export_committed.rs        # NEW: Integration tests for exporting committed files
│   ├── mod tests              # Test module declaration
│   ├── use common::fixtures   # Import TestFixture
│   ├── test_export_committed_file_without_staging()
│   ├── test_export_rejects_untracked_files()
│   ├── test_export_from_mode_layer()
│   ├── test_export_from_project_layer()
│   ├── test_export_file_modified_locally()
│   └── ... (additional test cases)
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Test isolation requires JIN_DIR to be set
// Always use fixture.set_jin_dir() BEFORE any Jin operations
// Failure leads to tests interfering with each other
let fixture = TestFixture::new()?;
fixture.set_jin_dir();  // CRITICAL: Sets isolated JIN_DIR

// CRITICAL: TempDir must be stored in TestFixture._tempdir
// If _tempdir goes out of scope, directory is deleted immediately
pub struct TestFixture {
    _tempdir: TempDir,  // Underscore prefix, but MUST be stored
    pub path: PathBuf,
}

// CRITICAL: Git locks must be cleaned up before temp dir deletion
// TestFixture Drop impl handles this automatically via git_helpers::cleanup_git_locks()
// Do NOT manually clean up temp directories

// CRITICAL: Export validation checks staging FIRST (fast path)
// Then checks JinMap for committed files
// Tests must ensure file is NOT in staging to test JinMap path
// Use: jin commit to remove from staging after committing

// CRITICAL: JinMap.load() returns Default if file doesn't exist
// This is first-run pattern, not an error
// Don't treat missing JinMap as error in tests

// CRITICAL: JinMap stores relative paths (file names only)
// contains_file() checks for filename match
// get_layer_files() returns Vec<String> of file names

// CRITICAL: export_file() only removes from staging if present
// Committed files aren't in staging, so they stay in Jin after export
// This is correct behavior - export doesn't modify Jin state for committed files

// CRITICAL: unique_test_id() uses atomic counter for true uniqueness
// std::process::id() alone is insufficient for parallel tests
// Always use unique_test_id() for resource naming

// CRITICAL: cargo nextest is used for test execution (not cargo test)
// CI uses: cargo nextest run --all-features
// Validation command must use nextest

// CRITICAL: Tests must initialize Git repo before using create_commit_in_repo
// jin_init() already does this via git2::Repository::init()

// CRITICAL: Export writes to Git index via `git add` command
// Tests must verify file is in Git index after export
// Use: git ls-files or check .git/index existence

// CRITICAL: .jin directory structure
// - .jin/.jinmap - JinMap YAML file
// - .jin/context - ProjectContext YAML file
// - .jin/layers/refs/jin/layers/* - Layer Git refs

// CRITICAL: Layer ref path format
// - Global: refs/jin/layers/global
// - Mode: refs/jin/layers/mode/{mode_name}
// - Project: refs/jin/layers/project/{project_name}
// - Scope: refs/jin/layers/scope/{scope_name}
```

---

## Implementation Blueprint

### Data Models and Structures

The tests use existing data structures from the implementation:

```rust
// From src/core/jinmap.rs - JinMap structure
pub struct JinMap {
    pub version: u32,
    pub mappings: HashMap<String, Vec<String>>,  // layer_ref -> file paths
    pub meta: JinMapMeta,
}

// JinMap lookup methods used by export:
// - load() -> Result<JinMap>
// - contains_file(path: &str) -> bool
// - get_layer_files(layer_ref: &str) -> Option<&[String]>
// - layer_refs() -> Vec<&String>

// From src/commands/export.rs - Export validation logic
fn validate_jin_tracked(path: &Path, staging: &StagingIndex, repo: &JinRepo) -> Result<()> {
    // 1. File existence check
    // 2. Staging index check (fast path)
    // 3. JinMap check for committed files
    // 4. Layer tree verification
}

// Layer tree extraction process:
// 1. Load JinMap
// 2. Find file in JinMap.layer_refs()
// 3. Resolve layer ref to commit OID
// 4. Get tree from commit
// 5. Read file from tree using TreeOps::read_file_from_tree()

// From src/git/tree.rs - Tree operations
pub trait TreeOps {
    fn read_file_from_tree(&self, tree_id: Oid, path: &Path) -> Result<Vec<u8>>;
    fn list_tree_files(&self, tree_id: Oid) -> Result<Vec<String>>;
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/export_committed.rs
  - IMPLEMENT: Integration test module for export committed files
  - FOLLOW pattern: tests/repair_check.rs (module structure, test patterns)
  - NAMING: snake_case file name, test_* function names
  - DEPENDENCIES: Existing test framework (assert_cmd, predicates, tempfile)
  - PLACEMENT: tests/ directory at project root
  - CODE:
      mod common;
      use common::fixtures::TestFixture;
      use assert_cmd::Command;
      use predicates::prelude::*;

      /// Get a Command for the jin binary
      fn jin_cmd() -> Command {
          Command::new(env!("CARGO_BIN_EXE_jin"))
      }

Task 2: IMPLEMENT test_export_committed_file_without_staging()
  - IMPLEMENT: Happy path test for exporting committed file
  - FOLLOW pattern: tests/repair_check.rs::test_repair_check_success_when_attached
  - SETUP:
    1. Create TestFixture with isolated JIN_DIR
    2. Initialize Jin repo
    3. Create test file
    4. Add file to staging
    5. Commit file (removes from staging)
    6. Export file (should succeed via JinMap)
  - VERIFY: Export succeeds, file in Git index
  - COVERAGE: Main happy path for P2.M2.T1 feature
  - PLACEMENT: First test in export_committed.rs

Task 3: IMPLEMENT test_export_rejects_untracked_files()
  - IMPLEMENT: Error path test for untracked files
  - FOLLOW pattern: tests/cli_apply_conflict.rs error testing
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Create test file (not added or committed)
    3. Attempt export
  - VERIFY: Export fails with "not Jin-tracked" error
  - COVERAGE: Error path for files not in Jin
  - PLACEMENT: export_committed.rs

Task 4: IMPLEMENT test_export_from_mode_layer()
  - IMPLEMENT: Export file from mode layer
  - FOLLOW pattern: tests/cli_apply_conflict.rs::test_apply_creates_jinmerge (mode setup)
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Create mode: jin mode create testmode
    3. Activate mode: jin mode use testmode
    4. Add file to mode layer: jin add config.json --mode
    5. Commit file
    6. Export file
  - VERIFY: Export succeeds from mode layer
  - COVERAGE: Mode layer JinMap lookup
  - PLACEMENT: export_committed.rs

Task 5: IMPLEMENT test_export_from_project_layer()
  - IMPLEMENT: Export file from project layer
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Set project in context
    3. Add file to project layer: jin add config.json --project
    4. Commit file
    5. Export file
  - VERIFY: Export succeeds from project layer
  - COVERAGE: Project layer JinMap lookup
  - PLACEMENT: export_committed.rs

Task 6: IMPLEMENT test_export_file_modified_locally()
  - IMPLEMENT: Export when local file differs from committed version
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Add and commit file
    3. Modify file locally
    4. Export file
  - VERIFY: Export succeeds (exports local version, not committed version)
  - COVERAGE: Export handles locally modified files
  - PLACEMENT: export_committed.rs

Task 7: IMPLEMENT test_export_with_missing_jinmap()
  - IMPLEMENT: Export when JinMap file is missing
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Add and commit file
    3. Delete .jin/.jinmap file
    4. Attempt export
  - VERIFY: Export fails (JinMap missing = no committed files)
  - COVERAGE: Graceful JinMap handling
  - PLACEMENT: export_committed.rs

Task 8: IMPLEMENT test_export_with_missing_layer_ref()
  - IMPLEMENT: Export when layer ref is missing (corrupted state)
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Add and commit file
    3. Manually delete layer ref from .jin/layers/refs/jin/layers/
    4. Attempt export
  - VERIFY: Export fails with appropriate error
  - COVERAGE: Corrupted state handling
  - PLACEMENT: export_committed.rs

Task 9: IMPLEMENT test_export_still_works_for_staged_files()
  - IMPLEMENT: Verify existing staged file export still works
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Add file to staging (don't commit)
    3. Export file
  - VERIFY: Export succeeds (staging fast path)
  - COVERAGE: Regression test for existing functionality
  - PLACEMENT: export_committed.rs

Task 10: IMPLEMENT test_export_multiple_files_from_different_layers()
  - IMPLEMENT: Export files from multiple layers
  - SETUP:
    1. Create TestFixture with Jin repo
    2. Create mode
    3. Add file1 to global, commit
    4. Add file2 to mode, commit
    5. Export both files
  - VERIFY: Both exports succeed
  - COVERAGE: Multi-layer scenario
  - PLACEMENT: export_committed.rs
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Basic test structure with TestFixture
// From tests/repair_check.rs:22-46
#[test]
fn test_export_committed_file_without_staging() {
    // Create isolated test environment
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    // CRITICAL: Set JIN_DIR before any Jin operations
    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Initialize Git repo (required for export)
    git2::Repository::init(fixture.path()).unwrap();

    // Create test file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    // Add and commit (file now in JinMap, not staging)
    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Export should succeed via JinMap lookup
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    // Verify file is in Git index
    let repo = git2::Repository::open(fixture.path()).unwrap();
    let index = repo.index().unwrap();
    assert!(index.get_path(Path::new("config.json"), 0).is_some());
}

// Pattern 2: Mode layer export test
#[test]
fn test_export_from_mode_layer() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize repos
    jin::git::JinRepo::create_at(&jin_dir).unwrap();
    git2::Repository::init(fixture.path()).unwrap();

    // Create mode
    jin_cmd()
        .args(["mode", "create", "testmode"])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate mode
    jin_cmd()
        .args(["mode", "use", "testmode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and commit file to mode layer
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"debug": true}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add debug config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Export should succeed from mode layer
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
}

// Pattern 3: Error path test - untracked file
#[test]
fn test_export_rejects_untracked_files() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize repos
    jin::git::JinRepo::create_at(&jin_dir).unwrap();
    git2::Repository::init(fixture.path()).unwrap();

    // Create file (not added or committed)
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    // Export should fail
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not Jin-tracked"));
}

// Pattern 4: Verify JinMap lookup is used
#[test]
fn test_export_uses_jinmap_lookup() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize repos
    jin::git::JinRepo::create_at(&jin_dir).unwrap();
    git2::Repository::init(fixture.path()).unwrap();

    // Create, add, commit file
    let file_path = fixture.path().join("config.json");
    fs::write(&file_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify file is in JinMap
    let jinmap_path = fixture.path().join(".jin").join(".jinmap");
    assert!(jinmap_path.exists());

    let jinmap_content = fs::read_to_string(&jinmap_path).unwrap();
    assert!(jinmap_content.contains("config.json"));

    // Export should succeed using JinMap
    jin_cmd()
        .arg("export")
        .arg("config.json")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify file was NOT removed from Jin (committed files stay in Jin)
    let jinmap_after = fs::read_to_string(&jinmap_path).unwrap();
    assert!(jinmap_after.contains("config.json"));
}

// GOTCHA: Use unique_test_id() for any unique naming
// From tests/common/fixtures.rs:335-339
fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", std::process::id(), count)
}

// GOTCHA: Verify file in Git index after export
let repo = git2::Repository::open(fixture.path()).unwrap();
let index = repo.index().unwrap();
let entry = index.get_path(Path::new("config.json"), 0);
assert!(entry.is_some(), "File should be in Git index after export");
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
  - functions: "TestFixture::new(), set_jin_dir(), jin_init()"
  - gotcha: "Always call set_jin_dir() before Jin operations"

TEST_ASSERTIONS:
  - command_success: ".assert().success()"
  - command_failure: ".assert().failure()"
  - stdout_contains: ".stdout(predicate::str::contains(\"text\"))"
  - stderr_contains: ".stderr(predicate::str::contains(\"error\"))"
  - file_exists: "assert!(path.exists())"

IMPLEMENTATION_UNDER_TEST:
  - file: "src/commands/export.rs"
  - functions: "validate_jin_tracked(), export_file(), execute()"
  - feature: "JinMap lookup for committed files (P2.M2.T1)"
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after test file creation - fix before proceeding
cargo fmt --check tests/export_committed.rs      # Check formatting
cargo clippy --tests -- -D warnings             # Lint tests

# Format and fix
cargo fmt tests/export_committed.rs
cargo clippy --tests --fix --allow-dirty --allow-staged

# Expected: Zero warnings, formatted code. If errors exist, READ output and fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run individual test file
cargo nextest run --all-features export_committed

# Run with verbose output
cargo nextest run --all-features export_committed --verbose

# Run specific test
cargo nextest run --all-features test_export_committed_file_without_staging

# Expected: All tests in export_committed.rs pass. If failing, debug root cause.
```

### Level 3: Integration Testing (System Validation)

```bash
# Full test suite for export-related tests
cargo nextest run --all-features export_committed cli_basic

# Verify all export tests pass together
cargo nextest run --all-features --test-pattern 'export'

# Expected: All export-related tests pass, no cross-test interference.
```

### Level 4: Cross-Validation with Existing Tests

```bash
# Run all integration tests to ensure no regressions
cargo nextest run --all-features --tests

# Expected: All integration tests pass, including new export_committed tests.
```

### Level 5: Manual Verification (Optional)

```bash
# Manual workflow verification (for development confidence)
# 1. Setup test repository
cd /tmp && mkdir test_export && cd test_export
git init
jin init --dir .jin_test
export JIN_DIR=$(pwd)/.jin_test

# 2. Test committed file export
echo '{"port": 8080}' > config.json
jin add config.json
jin commit -m "Add config"
jin export config.json
# Expected: Success - file exported to Git

# 3. Verify Git has the file
git status
git ls-files
# Expected: config.json should be in Git index

# 4. Test untracked file rejection
echo '{"port": 9090}' > untracked.json
jin export untracked.json
# Expected: Error - "not Jin-tracked"

# Cleanup
cd -
rm -rf /tmp/test_export
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo nextest run --all-features export_committed`
- [ ] No linting errors: `cargo clippy --tests`
- [ ] No formatting issues: `cargo fmt --check tests/export_committed.rs`
- [ ] Tests run in isolation without interference
- [ ] Test cleanup is automatic (TestFixture Drop impl)

### Feature Validation

- [ ] test_export_committed_file_without_staging: Happy path works
- [ ] test_export_rejects_untracked_files: Error handling works
- [ ] test_export_from_mode_layer: Mode layer export works
- [ ] test_export_from_project_layer: Project layer export works
- [ ] test_export_file_modified_locally: Modified file handling works
- [ ] test_export_with_missing_jinmap: Missing JinMap handled
- [ ] test_export_with_missing_layer_ref: Corrupted state handled
- [ ] test_export_still_works_for_staged_files: Existing functionality preserved
- [ ] test_export_multiple_files_from_different_layers: Multi-layer works
- [ ] Additional edge case tests as needed

### Code Quality Validation

- [ ] Follows existing test patterns from repair_check.rs and cli_apply_conflict.rs
- [ ] Uses TestFixture::new() and set_jin_dir() for all tests
- [ ] Uses jin_cmd() helper consistently
- [ ] Uses assert_cmd Command patterns consistently
- [ ] Uses predicates for output assertions
- [ ] Test names are descriptive (test_<action>_<scenario>)
- [ ] Comments explain complex test scenarios
- [ ] JIN_DIR isolation is maintained throughout

### Test Coverage

- [ ] JinMap lookup is tested
- [ ] Layer content extraction is tested
- [ ] Export from global layer works
- [ ] Export from mode layer works
- [ ] Export from project layer works
- [ ] Export rejects untracked files
- [ ] Export handles locally modified files
- [ ] Export handles missing JinMap
- [ ] Export handles missing layer refs
- [ ] Existing staged file export still works

---

## Anti-Patterns to Avoid

- ❌ Don't skip test isolation - always use TestFixture::new() and set_jin_dir()
- ❌ Don't manually clean up temp directories - TestFixture handles this
- ❌ Don't expect export to fail for committed files - it should succeed
- ❌ Don't forget to set JIN_DIR - tests will interfere with each other
- ❌ Don't skip Jin initialization - required for export tests
- ❌ Don't skip Git initialization - export writes to Git index
- ❌ Don't use hardcoded mode/project names - use unique_test_id()
- ❌ Don't test export without committing first - file won't be in JinMap
- ❌ Don't let TempDir go out of scope - store in TestFixture
- ❌ Don't forget to verify file is in Git index after export
- ❌ Don't ignore the fact that committed files stay in Jin after export
- ❌ Don't skip testing the error paths (untracked files, missing JinMap, etc.)
- ❌ Don't use cargo test for validation - use cargo nextest
- ❌ Don't create files without adding them via jin add
- ❌ Don't test export on files that are still in staging (commit them first)

---

## Additional Research References

### Implementation Being Tested

```yaml
- docfile: plan/P2M2T1/PRP.md
  why: Complete implementation details for P2.M2.T1
  contains: validate_jin_tracked() changes, JinMap usage, layer extraction

- file: src/commands/export.rs
  why: The implementation code being tested
  section: Lines 136-195 for JinMap validation logic
```

### Test Pattern References

```yaml
- file: plan/P6M2/research/05_e2e_workflow_testing.md
  why: End-to-end workflow test patterns
  section: Lines 47-91 for multi-step workflow testing

- file: plan/P6M2/research/02_assert_cmd_and_predicates.md
  why: CLI testing patterns with assert_cmd
  section: Lines 1-100 for assertion patterns

- file: plan/P6M2/research/03_tempfile_and_fixtures.md
  why: TempDir and fixture usage patterns
  section: Lines 78-120 for TestFixture pattern
```

### External Documentation

```yaml
- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: assert_cmd crate documentation for CLI testing
  critical: Command::new(), .arg(), .assert(), .success(), .failure()

- url: https://docs.rs/predicates/latest/predicates/
  why: predicates crate documentation for output assertions
  critical: predicate::str::contains(), path::exists()

- url: https://docs.rs/tempfile/latest/tempfile/
  why: tempfile crate documentation for temporary directory management
  critical: TempDir::new(), automatic cleanup on drop

- url: https://doc.rust-lang.org/book/ch11-03-test-organization.html
  why: Rust test organization conventions
  critical: Integration tests go in tests/ directory at project root
```

---

## Confidence Score

**8/10** - High confidence in one-pass implementation success

**Justification**:
- Complete P2.M2.T1 implementation context provided
- Existing test patterns fully documented with code examples
- All test fixtures and helpers identified
- Test framework and validation commands specified
- Known gotchas documented with workarounds
- Existing tests provide clear patterns to follow
- Comprehensive test scenarios defined

**Remaining risk factors**:
- Export command behavior for committed files may have edge cases not covered
- Layer ref resolution may have additional error cases
- Some test scenarios may need adjustment during implementation
- Parallel test execution may reveal isolation issues (addressed with JIN_DIR)

---

## Success Metrics

**Quantitative**:
- 8-10 test functions implemented
- 100% of success criteria in test list pass
- Zero clippy warnings
- All tests pass with cargo nextest

**Qualitative**:
- Tests follow existing patterns consistently
- Test names clearly indicate what is being tested
- Tests are isolated and can run in parallel
- Tests cover both success and error paths
- Tests validate P2.M2.T1 implementation thoroughly

---

**PRP Created**: January 3, 2026
**Research Completed**: Comprehensive (P2.M2.T1 PRP, existing test patterns, P6M2 research)
**Implementation Readiness**: High (8/10 confidence)
**Expected Implementation Time**: 2-3 hours for experienced Rust developer
