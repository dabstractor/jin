# PRP: P1.M1.T4.S1 - Create test file for --local flag functionality

---

## Goal

**Feature Goal**: Create comprehensive integration tests for the `--local` flag functionality that validates the complete workflow from CLI argument parsing through Layer 8 (UserLocal) routing, including flag conflict validation.

**Deliverable**: Test file `tests/cli_add_local.rs` with four passing tests that verify:
1. `--local` routes files to Layer 8 (UserLocal)
2. `--local` rejects `--mode` flag combination
3. `--local` rejects `--global` flag combination
4. Complete `jin add --local` → `jin commit` → `jin apply` workflow

**Success Definition**:
- All four tests in `tests/cli_add_local.rs` pass
- `cargo test --test cli_add_local` completes successfully
- Tests properly validate the `--local` flag implementation from P1.M1.T3.S1
- Test coverage matches existing test patterns in `tests/cli_basic.rs`

---

## User Persona

**Target User**: Jin CLI users who want to add machine-specific configuration files to Layer 8 (UserLocal) using the `--local` flag

**Use Case**: User runs `jin add .config/settings.json --local` to add a file to Layer 8 storage at `~/.jin/local/`

**User Journey**:
1. User executes: `jin add .config/settings.json --local`
2. Clap parses `--local` flag into `AddArgs.local = true`
3. `execute()` function constructs `RoutingOptions` with `local: args.local`
4. `route_to_layer()` receives `options.local = true`
5. Routing logic returns `Layer::UserLocal`
6. File is staged for Layer 8 storage
7. User commits and applies to complete the workflow

**Pain Points Addressed**:
- Without tests, the `--local` flag behavior is not verified
- Flag conflict validation may have edge cases that aren't caught
- Complete workflow may have integration issues between components
- Regression risk when modifying related code

---

## Why

- **Validates Implementation**: Tests verify that the `--local` flag implementation from P1.M1.T3.S1 works correctly
- **Prevents Regressions**: Future changes to routing, validation, or CLI code won't break `--local` functionality
- **Documents Expected Behavior**: Tests serve as executable documentation of how `--local` should work
- **Catches Integration Issues**: Multi-step workflow test catches issues between components
- **Follows Existing Patterns**: Tests match the established testing patterns in `tests/cli_basic.rs`
- **Provides Coverage**: Four tests cover routing, validation, and workflow scenarios

---

## What

### User-Visible Behavior

**After Implementation**:
```bash
# Test 1: --local routes to Layer 8
cargo test test_add_local_routes_to_layer_8

# Test 2: --local rejects --mode combination
cargo test test_add_local_rejects_mode_flag

# Test 3: --local rejects --global combination
cargo test test_add_local_rejects_global_flag

# Test 4: Complete workflow test
cargo test test_add_local_commit_apply_workflow
```

### Technical Requirements

1. **File to Create**: `tests/cli_add_local.rs`
2. **Test Framework**: `assert_cmd` + `predicates` + `tempfile`
3. **Test Fixtures**: Use `TestFixture` from `tests/common/fixtures.rs`
4. **Test Assertions**: Use custom assertions from `tests/common/assertions.rs`
5. **Test Isolation**: Set `JIN_DIR` environment variable for each test

### Success Criteria

- [ ] Test file `tests/cli_add_local.rs` created with all four tests
- [ ] `test_add_local_routes_to_layer_8` verifies `--local` routing to UserLocal
- [ ] `test_add_local_rejects_mode_flag` verifies `--local --mode` error
- [ ] `test_add_local_rejects_global_flag` verifies `--local --global` error
- [ ] `test_add_local_commit_apply_workflow` verifies complete workflow
- [ ] All tests pass: `cargo test --test cli_add_local`
- [ ] Tests follow existing patterns from `tests/cli_basic.rs`

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including test patterns, fixture usage, assertion helpers, and the implementation being tested. The executing AI agent has everything needed to create the test file._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Contract from P1.M1.T3.S1 (Pass local flag - Complete)
- docfile: plan/P1M1T3S1/PRP.md
  why: Defines the --local flag implementation being tested
  section: "Goal", "What", "Implementation Blueprint"
  critical: local: args.local is passed at src/commands/add.rs line 55
  output: "--local flag flows from CLI → AddArgs → RoutingOptions → route_to_layer()"

# Contract from P1.M1.T2.S2 (Validation logic - Complete)
- docfile: plan/P1M1T2S2/PRP.md
  why: Defines the validation logic for --local flag conflicts
  section: "Goal", "Validation"
  critical: validate_routing_options() rejects --local with other flags
  output: "Error: 'Cannot combine --local with other layer flags'"

# Contract from P1.M1.T2.S3 (Routing logic - Complete)
- docfile: plan/P1M1T2S3/PRP.md
  why: Defines the routing logic for --local flag
  section: "Goal", "Implementation Blueprint"
  critical: route_to_layer() returns Layer::UserLocal when options.local == true
  output: "--local routes to Layer 8 (UserLocal, ~/.jin/local/)"

# Test Pattern Reference File
- file: tests/cli_basic.rs
  why: Primary reference for CLI testing patterns in this codebase
  pattern: |
    use assert_cmd::Command;
    use predicates::prelude::*;

    fn jin() -> Command {
        Command::new(env!("CARGO_BIN_EXE_jin"))
    }

    #[test]
    fn test_add_with_mode_flag() {
        let temp = tempfile::tempdir().unwrap();
        let test_file = temp.path().join("config.json");
        std::fs::write(&test_file, "{}").unwrap();

        jin()
            .current_dir(temp.path())
            .args(["add", "config.json", "--mode"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Jin not initialized"));
    }
  gotcha: Tests must use temp directories and set JIN_DIR for isolation

# Common Test Fixtures
- file: tests/common/fixtures.rs
  why: Provides TestFixture, jin_init(), setup_test_repo(), jin() helper
  pattern: |
    pub struct TestFixture {
        _tempdir: TempDir,
        pub path: PathBuf,
        pub jin_dir: Option<PathBuf>,
    }

    pub fn setup_test_repo() -> Result<TestFixture, Box<dyn std::error::Error>> {
        let fixture = TestFixture::new()?;
        let jin_dir = fixture.jin_dir.as_ref().unwrap();
        jin_init(fixture.path(), Some(jin_dir))?;
        Ok(fixture)
    }

    pub fn jin() -> Command {
        Command::new(env!("CARGO_BIN_EXE_jin"))
    }
  gotcha: TestFixture._tempdir must be stored to prevent premature cleanup
  critical: Always call fixture.set_jin_dir() before any Jin operations

# Custom Test Assertions
- file: tests/common/assertions.rs
  why: Provides Jin-specific assertion helpers
  pattern: |
    pub fn assert_staging_contains(project_path: &Path, file: &str)
    pub fn assert_staging_not_contains(project_path: &Path, file: &str)
    pub fn assert_layer_ref_exists(ref_path: &str, jin_repo_path: Option<&Path>)
  gotcha: Staging index is at .jin/staging/index.json

# Routing Implementation (being tested)
- file: src/staging/router.rs (lines 31-70)
  why: Function that routes --local to UserLocal layer
  pattern: |
    pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
        if options.global {
            return Ok(Layer::GlobalBase);
        }
        if options.local {
            return Ok(Layer::UserLocal);
        }
        // ... rest of routing logic
    }
  critical: --local has precedence after --global, returns Layer::UserLocal

# Validation Implementation (being tested)
- file: src/staging/router.rs (lines 73-96)
  why: Function that validates --local flag conflicts
  pattern: |
    pub fn validate_routing_options(options: &RoutingOptions) -> Result<()> {
        if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
            return Err(JinError::Config(
                "Cannot combine --local with other layer flags".to_string(),
            ));
        }
        // ... other validations
    }
  critical: Error message is "Cannot combine --local with other layer flags"

# Add Command Implementation (being tested)
- file: src/commands/add.rs (lines 49-57)
  why: Function that constructs RoutingOptions with local field
  pattern: |
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
        local: args.local,
    };
    validate_routing_options(&options)?;
  critical: local field is passed from args.local

# Layer Definition
- file: src/core/layer.rs (lines 27-28, 93, 44, 126)
  why: Defines UserLocal layer and storage path
  pattern: |
    pub enum Layer {
        // ...
        UserLocal,  // Layer 8: Machine-only overlays (~/.jin/local/)
    }
  critical: UserLocal stores at ~/.jin/local/, has precedence level 8

# External Research: assert_cmd Documentation
- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: Official documentation for CLI testing crate
  section: "struct.Command", "struct.Assert"
  critical: Command::new(env!("CARGO_BIN_EXE_jin")), .assert(), .success(), .failure()

# External Research: predicates crate
- url: https://docs.rs/predicates/latest/predicates/
  why: Output assertion library
  section: "predicate::str"
  critical: predicate::str::contains(), predicate::str::is_match()

# External Research: Rust CLI Book Testing
- url: https://rust-cli.github.io/book/tutorial/testing.html
  why: Best practices for testing CLI applications
  section: "Testing CLI applications by running them"
  critical: Integration tests go in tests/ directory, use assert_cmd

# Research Artifacts
- docfile: plan/P1M1T4S1/research/rust_cli_testing.md
  why: Comprehensive research on Rust CLI testing patterns
  section: "Test Patterns from Codebase", "Testing Conflicting Flags Pattern"
  critical: Code examples for flag conflict testing, workflow testing
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   └── args.rs                # AddArgs struct with local field (line 29)
│   ├── commands/
│   │   └── add.rs                # execute() passes local field (line 55)
│   ├── staging/
│   │   ├── mod.rs                # Exports routing functions
│   │   └── router.rs             # route_to_layer(), validate_routing_options()
│   └── core/
│       ├── layer.rs              # Layer::UserLocal enum variant
│       ├── config.rs             # ProjectContext type
│       └── error.rs              # JinError type
├── tests/
│   ├── cli_basic.rs              # PRIMARY TEST PATTERN REFERENCE
│   ├── cli_reset.rs
│   ├── cli_diff.rs
│   ├── cli_import.rs
│   ├── cli_resolve.rs
│   ├── cli_mv.rs
│   ├── cli_apply_conflict.rs
│   ├── cli_list.rs
│   ├── common/
│   │   ├── mod.rs                # Common test utilities
│   │   ├── fixtures.rs           # TestFixture, jin_init(), jin() helpers
│   │   ├── git_helpers.rs        # Git lock cleanup
│   │   └── assertions.rs         # Custom Jin assertions
│   └── ...other test files
└── plan/
    ├── P1M1T1S1/PRP.md           # AddArgs.local field (Complete)
    ├── P1M1T2S2/PRP.md           # Validation logic (Complete)
    ├── P1M1T2S3/PRP.md           # Routing logic (Complete)
    ├── P1M1T3S1/PRP.md           # Wiring implementation (Complete)
    └── P1M1T4S1/
        ├── PRP.md                # THIS PRP
        └── research/
            └── rust_cli_testing.md # Rust CLI testing research
```

### Desired Codebase Tree After This Subtask

```bash
jin/
├── tests/
│   ├── cli_add_local.rs          # NEW FILE - --local flag tests
│   │   ├── test_add_local_routes_to_layer_8()
│   │   ├── test_add_local_rejects_mode_flag()
│   │   ├── test_add_local_rejects_global_flag()
│   │   └── test_add_local_commit_apply_workflow()
│   └── ...
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: TempDir must be stored to prevent premature cleanup
// When TempDir is dropped, the directory is deleted immediately
// let temp = TempDir::new().unwrap(); // temp must be kept in scope

// CRITICAL: JIN_DIR must be set BEFORE any Jin operations
// fixture.set_jin_dir(); // Call this before jin() commands

// CRITICAL: TestFixture._tempdir is private and named with underscore
// This signals it must be kept in scope but shouldn't be accessed directly

// CRITICAL: Git locks need cleanup before temp dir deletion
// TestFixture handles this automatically in Drop impl

// CRITICAL: File paths passed to jin commands should be relative
// .args(["add", "config.json", "--local"]) // Not absolute path

// CRITICAL: unique_test_id() is better than std::process::id()
// Static atomic counter ensures uniqueness across parallel tests

// CRITICAL: Error message for flag conflicts is exact
// "Cannot combine --local with other layer flags"

// CRITICAL: --local returns Layer::UserLocal
// Storage path is ~/.jin/local/

// CRITICAL: predicates must be used for output assertions
// .stderr(predicate::str::contains("error message"))

// CRITICAL: Tests use Result return type for ? operator
// #[test] fn test_name() -> Result<(), Box<dyn std::error::Error>>

// CRITICAL: jin_init() also initializes a Git repository
// This is needed for create_commit_in_repo tests

// GOTCHA: After create_commit_in_repo, file is unstaged in Git
// This is intentional - jin add rejects Git-staged files

// GOTCHA: assert_staging_contains() checks .jin/staging/index.json
// Not the Git index, but Jin's staging index

// GOTCHA: setup_test_repo() already calls jin_init()
// Don't call jin_init() again when using setup_test_repo()
```

---

## Implementation Blueprint

### Data Models and Structure

**Input Contract** (from P1.M1.T3.S1 - Complete):
```rust
// AddArgs struct in src/cli/args.rs (lines 6-30)
#[derive(Args, Debug)]
pub struct AddArgs {
    pub files: Vec<String>,
    #[arg(long)]
    pub mode: bool,
    #[arg(long)]
    pub scope: Option<String>,
    #[arg(long)]
    pub project: bool,
    #[arg(long)]
    pub global: bool,
    #[arg(long)]
    pub local: bool,  // <-- Field being tested
}
```

**Implementation Being Tested** (from P1.M1.T3.S1, P1.M1.T2.S2, P1.M1.T2.S3):
```rust
// execute() in src/commands/add.rs (lines 49-60)
let options = RoutingOptions {
    mode: args.mode,
    scope: args.scope.clone(),
    project: args.project,
    global: args.global,
    local: args.local,  // <-- Being tested
};
validate_routing_options(&options)?;  // <-- Validation being tested
let target_layer = route_to_layer(&options, &context)?;  // <-- Routing being tested

// route_to_layer() in src/staging/router.rs (lines 35-38)
if options.local {
    return Ok(Layer::UserLocal);  // <-- Expected routing
}

// validate_routing_options() in src/staging/router.rs (lines 82-87)
if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
    return Err(JinError::Config(
        "Cannot combine --local with other layer flags".to_string(),  // <-- Expected error
    ));
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/cli_add_local.rs
  - IMPLEMENT: Test file with module declarations and imports
  - FOLLOW pattern: tests/cli_basic.rs (file structure, imports)
  - NAMING: cli_add_local.rs (snake_case, cli_ prefix)
  - PLACEMENT: tests/ directory (integration test location)
  - DEPENDENCIES: None (first task)

Task 2: IMPLEMENT test_add_local_routes_to_layer_8
  - CREATE: Test function that verifies --local routes to UserLocal
  - FOLLOW pattern: test_add_with_mode_flag from cli_basic.rs
  - SETUP: Use setup_test_repo() from fixtures
  - ACTIONS: Create test file, run jin add with --local, verify staging
  - ASSERTIONS: Use assert_staging_contains() from assertions.rs
  - NAMING: test_add_local_routes_to_layer_8
  - PLACEMENT: In tests/cli_add_local.rs
  - DEPENDENCIES: Task 1 (file exists)

Task 3: IMPLEMENT test_add_local_rejects_mode_flag
  - CREATE: Test function that verifies --local --mode error
  - FOLLOW pattern: test_add_with_mode_flag from cli_basic.rs
  - SETUP: Use TempDir::new() and JIN_DIR isolation
  - ACTIONS: Run jin add with --local --mode, expect failure
  - ASSERTIONS: Use predicate::str::contains("Cannot combine --local with other layer flags")
  - NAMING: test_add_local_rejects_mode_flag
  - PLACEMENT: In tests/cli_add_local.rs
  - DEPENDENCIES: Task 1 (file exists)

Task 4: IMPLEMENT test_add_local_rejects_global_flag
  - CREATE: Test function that verifies --local --global error
  - FOLLOW pattern: test_add_local_rejects_mode_flag
  - SETUP: Use TempDir::new() and JIN_DIR isolation
  - ACTIONS: Run jin add with --local --global, expect failure
  - ASSERTIONS: Use predicate::str::contains("Cannot combine --local with other layer flags")
  - NAMING: test_add_local_rejects_global_flag
  - PLACEMENT: In tests/cli_add_local.rs
  - DEPENDENCIES: Task 1 (file exists)

Task 5: IMPLEMENT test_add_local_commit_apply_workflow
  - CREATE: Test function for complete add → commit → apply workflow
  - FOLLOW pattern: test_status_dirty_workflow from cli_basic.rs
  - SETUP: Use setup_test_repo() from fixtures
  - ACTIONS: Create file, jin add --local, jin commit, verify staging
  - ASSERTIONS: Use assert_staging_contains(), verify commit success
  - NAMING: test_add_local_commit_apply_workflow
  - PLACEMENT: In tests/cli_add_local.rs
  - DEPENDENCIES: Task 1 (file exists)

Task 6: RUN CARGO TEST
  - COMMAND: cargo test --test cli_add_local
  - EXPECTED: All four tests pass
  - IF FAILS: Read test output, debug assertions, check error messages
  - DEPENDENCIES: Tasks 2-5 complete (all tests implemented)
```

### Implementation Patterns & Key Details

```rust
// ================== FILE STRUCTURE ==================
// Location: tests/cli_add_local.rs

//! Integration tests for jin add --local command
//!
//! Tests the --local flag functionality for adding files to Layer 8 (UserLocal).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

// ================== TEST 1: ROUTING TO USERLOCAL ==================

#[test]
fn test_add_local_routes_to_layer_8() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create isolated test environment with Jin initialized
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();  // CRITICAL: Call before any Jin operations

    // Create test file
    let test_file = fixture.path().join(".config/settings.json");
    fs::create_dir_all(test_file.parent().unwrap())?;
    fs::write(&test_file, r#"{"theme": "dark"}"#)?;

    // ACT: Add file with --local flag
    jin()
        .args(["add", ".config/settings.json", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added"))
        .stdout(predicate::str::contains(".config/settings.json"));

    // ASSERT: File is staged in Layer 8 (UserLocal)
    assert_staging_contains(fixture.path(), ".config/settings.json");

    // ASSERT: Verify the staging index shows UserLocal layer
    let staging_index = fixture.path().join(".jin/staging/index.json");
    let staging_content = fs::read_to_string(&staging_index)?;
    assert!(
        staging_content.contains("UserLocal") || staging_content.contains("user_local") || staging_content.contains("local"),
        "Staging index should contain UserLocal layer reference. Content:\n{}",
        staging_content
    );

    Ok(())
}

// ================== TEST 2: REJECTS --MODE FLAG ==================

#[test]
fn test_add_local_rejects_mode_flag() -> Result<(), Box<dyn std::error::Error>> {
    use tempfile::TempDir;

    // SETUP: Create temp directory with isolated JIN_DIR
    let temp = TempDir::new()?;
    let jin_dir = temp.path().join(".jin_global");

    // ACT: Try to add with both --local and --mode flags
    jin()
        .args(["add", "config.json", "--local", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot combine --local with other layer flags"
        ));

    Ok(())
}

// ================== TEST 3: REJECTS --GLOBAL FLAG ==================

#[test]
fn test_add_local_rejects_global_flag() -> Result<(), Box<dyn std::error::Error>> {
    use tempfile::TempDir;

    // SETUP: Create temp directory with isolated JIN_DIR
    let temp = TempDir::new()?;
    let jin_dir = temp.path().join(".jin_global");

    // ACT: Try to add with both --local and --global flags
    jin()
        .args(["add", "config.json", "--local", "--global"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot combine --local with other layer flags"
        ));

    Ok(())
}

// ================== TEST 4: COMPLETE WORKFLOW ==================

#[test]
fn test_add_local_commit_apply_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create isolated test environment with Jin initialized
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();

    // Create test file
    let test_file = fixture.path().join(".local/config.toml");
    fs::create_dir_all(test_file.parent().unwrap())?;
    fs::write(&test_file, r#"[settings]
theme = "dark"
editor = "vim"
"#)?;

    // STEP 1: Add file with --local flag
    jin()
        .args(["add", ".local/config.toml", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added"))
        .stdout(predicate::str::contains(".local/config.toml"));

    // ASSERT: File is staged
    assert_staging_contains(fixture.path(), ".local/config.toml");

    // STEP 2: Commit the staged file
    jin()
        .args(["commit", "-m", "Add local config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("commit"))
        .stdout(predicate::str::contains("1 file"));

    // ASSERT: File is no longer staged after commit
    assert_staging_not_contains(fixture.path(), ".local/config.toml");

    // STEP 3: Verify file was committed to UserLocal layer
    // Check that the layer ref exists in Jin repository
    let jin_repo = fixture.jin_dir.as_ref().unwrap();
    let repo = git2::Repository::open(jin_repo)?;

    // Verify the commit exists
    let head = repo.head()?;
    let commit = repo.find_commit(head.target().unwrap())?;
    let msg = commit.message().unwrap();
    assert!(
        msg.contains("Add local config"),
        "Commit message should contain 'Add local config'. Got: {}",
        msg
    );

    Ok(())
}

// ================== PATTERN EXPLANATION ==================
//
// Test Structure:
// 1. SETUP: Create test fixture or temp directory
// 2. ACT: Run jin commands
// 3. ASSERT: Verify expected state changes
//
// Test Isolation:
// - setup_test_repo() creates TestFixture with Jin initialized
// - fixture.set_jin_dir() sets JIN_DIR environment variable
// - All jin() commands must include .env("JIN_DIR", ...)
//
// Test Naming:
// - test_<function>_<scenario> pattern
// - test_add_local_routes_to_layer_8
// - test_add_local_rejects_mode_flag
//
// Assertions:
// - .assert().success() for expected success
// - .assert().failure() for expected failure
// - predicate::str::contains() for output validation
// - assert_staging_contains() for Jin-specific assertions
//
// File Operations:
// - fs::write() to create test files
// - fs::create_dir_all() for nested directories
// - fixture.path() to get temp directory path
//
// Gotchas:
// - TestFixture._tempdir must stay in scope (private field)
// - JIN_DIR must be set before first Jin operation
// - File paths in jin commands should be relative
// - Use ? operator for Result propagation
```

### Integration Points

```yaml
TEST DEPENDENCIES:
  - crate: assert_cmd
    use: Command::new(env!("CARGO_BIN_EXE_jin"))
    version: "2.0"

  - crate: predicates
    use: predicate::str::contains()
    version: "3.0"

  - crate: tempfile
    use: TempDir::new()
    version: "3.0"

TEST FIXTURES:
  - module: tests/common
    use: TestFixture, setup_test_repo(), jin()
    file: tests/common/fixtures.rs

  - module: tests/common::assertions
    use: assert_staging_contains(), assert_staging_not_contains()
    file: tests/common/assertions.rs

IMPLEMENTATION_BEING_TESTED:
  - file: src/commands/add.rs
    function: execute()
    line_55: local: args.local,

  - file: src/staging/router.rs
    function: route_to_layer()
    lines_35-38: if options.local { return Ok(Layer::UserLocal); }

  - file: src/staging/router.rs
    function: validate_routing_options()
    lines_82-87: --local conflict validation

TEST_OUTPUT:
  - file: tests/cli_add_local.rs
    tests: 4 test functions
    coverage: Routing, validation, workflow scenarios
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after file creation - fix before proceeding
cargo test --test cli_add_local --no-run     # Compile tests without running
cargo clippy --test cli_add_local             # Lint checking

# Project-wide validation
cargo test --no-run                           # Compile all tests
cargo fmt -- --check                          # Format check

# Expected: Zero compilation errors. If errors exist, READ output and fix.
# Common issues:
# - Missing imports (use std::fs;, use tempfile::TempDir)
# - Wrong predicate path (predicate::str::contains)
# - Missing semicolons in test functions
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the new tests
cargo test --test cli_add_local -v            # Run with verbose output

# Run specific test
cargo test --test cli_add_local test_add_local_routes_to_layer_8 -v

# Full test suite for CLI tests
cargo test --test cli_basic -v                # Verify no regressions

# Expected: All tests pass. If failing, debug test failures.
# Common issues:
# - Wrong error message in predicate
# - File path issues (relative vs absolute)
# - JIN_DIR not set correctly
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all CLI tests together
cargo test --test cli_* -v                    # Run all CLI integration tests

# Run with output capture
cargo test --test cli_add_local -- --nocapture

# Test with different configurations
cargo test --test cli_add_local -- --test-threads=1  # Serial execution

# Expected: All integration tests pass, no test interference
# Common issues:
# - Test pollution from shared state
# - Parallel test execution issues
# - Temporary directory cleanup failures
```

### Level 4: Coverage & Quality Validation

```bash
# Verify test coverage (if tarpaulin is available)
cargo tarpaulin --test cli_add_local --out Html

# Run tests with documentation
cargo test --test cli_add_local -- --show-output

# Verify no unintended side effects
cargo test --lib                              # Run library tests
cargo test                                    # Run ALL tests

# Expected: All tests pass, coverage meets requirements
# Common issues:
# - Tests don't actually verify the implementation
# - Missing edge case coverage
# - Tests are too tightly coupled to implementation details
```

---

## Final Validation Checklist

### Technical Validation

- [ ] File `tests/cli_add_local.rs` created with correct imports
- [ ] `cargo test --test cli_add_local` compiles with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy --test cli_add_local` produces no warnings
- [ ] All 4 tests pass individually
- [ ] All 4 tests pass together
- [ ] No regressions in existing tests

### Feature Validation

- [ ] `test_add_local_routes_to_layer_8` verifies --local routing to UserLocal
- [ ] `test_add_local_rejects_mode_flag` verifies --local --mode error
- [ ] `test_add_local_rejects_global_flag` verifies --local --global error
- [ ] `test_add_local_commit_apply_workflow` verifies complete workflow
- [ ] Tests match existing patterns from `tests/cli_basic.rs`
- [ ] Tests use TestFixture and custom assertions correctly

### Code Quality Validation

- [ ] Tests follow naming convention (test_<function>_<scenario>)
- [ ] Tests use Result return type for ? operator
- [ ] Tests have clear SETUP/ACT/ASSERT structure
- [ ] Test isolation is properly maintained (JIN_DIR set)
- [ ] Error messages in predicates match implementation
- [ ] File operations are safe (create_dir_all before write)

### Documentation & Completeness

- [ ] Tests serve as executable documentation of --local behavior
- [ ] Comments explain what is being tested and why
- [ ] Edge cases are covered (flag conflicts, workflow)
- [ ] Tests are maintainable and not overly brittle
- [ ] Tests will catch regressions in future changes

---

## Anti-Patterns to Avoid

- **Don't** use hardcoded paths like `/tmp/` - use `tempfile::TempDir` instead
- **Don't** forget to call `fixture.set_jin_dir()` before Jin operations
- **Don't** use absolute file paths in `jin()` commands - use relative paths
- **Don't** let TempDir go out of scope before test completes
- **Don't** use `std::process::id()` for unique IDs - use `unique_test_id()`
- **Don't** skip testing flag conflicts - validation is critical
- **Don't** test exact error message position - use `contains()` predicates
- **Don't** forget to use `?` operator for Result propagation
- **Don't** use global state or shared mutable state in tests
- **Don't** create tests that depend on execution order
- **Don't** ignore Git lock cleanup - TestFixture handles this
- **Don't** test implementation details only - test user-visible behavior

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification**:
- **Clear Specification**: Four specific tests with defined behaviors
- **Existing Patterns**: Excellent reference in `tests/cli_basic.rs`
- **Test Fixtures**: `TestFixture` and custom assertions available
- **Implementation Complete**: The code being tested is already implemented
- **Well-Researched**: Comprehensive research on testing patterns
- **Specific Assertions**: Error messages and behaviors are known
- **Isolation Support**: JIN_DIR environment variable for test isolation
- **Documentation**: External resources with code examples

**Minor Risk Factors**:
- **Layer Verification**: Verifying "UserLocal" in staging index may require checking exact format
- **Git Operations**: Workflow test involves Git commit verification which may have edge cases
- **Test Pollution**: Need to ensure JIN_DIR isolation prevents test interference

**Mitigation**:
- Use flexible predicates (`contains()` rather than exact match)
- Follow existing test patterns exactly
- Use TestFixture which handles cleanup automatically
- Run tests with `--test-threads=1` if needed for debugging

**Current Status**: Ready for implementation - all context and patterns are clear

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1T4S1/research/`

**Key File References**:
- `tests/cli_basic.rs` - PRIMARY TEST PATTERN REFERENCE (1054 lines)
- `tests/common/fixtures.rs` - TestFixture, setup_test_repo(), jin() helpers
- `tests/common/assertions.rs` - Custom Jin assertions (assert_staging_contains, etc.)
- `src/commands/add.rs` - execute() function being tested (line 55)
- `src/staging/router.rs` - route_to_layer(), validate_routing_options() being tested
- `plan/P1M1T1S1/PRP.md` - AddArgs.local field (Complete)
- `plan/P1M1T2S2/PRP.md` - Validation logic (Complete)
- `plan/P1M1T2S3/PRP.md` - Routing logic (Complete)
- `plan/P1M1T3S1/PRP.md` - Wiring implementation (Complete)
- `plan/P1M1T4S1/research/rust_cli_testing.md` - Rust CLI testing research

**External References** (from research):
- [assert_cmd Documentation](https://docs.rs/assert_cmd/latest/assert_cmd/)
- [predicates crate](https://docs.rs/predicates/latest/predicates/)
- [Rust CLI Book - Testing Chapter](https://rust-cli.github.io/book/tutorial/testing.html)
- [Testing Rust CLI Apps with assert_cmd](https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert-cmd/)
- [Integration Testing Rust Binaries](https://www.unwoundstack.com/blog/integration-testing-rust-binaries.html)
