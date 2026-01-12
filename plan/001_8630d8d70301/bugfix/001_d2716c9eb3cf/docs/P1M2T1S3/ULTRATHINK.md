# ULTRATHINK Plan: PRP for P1.M2.T1.S3 - Integration Test for jin log Command

## PRP Section-by-Section Writing Plan

### 1. Goal Section
- **Feature Goal**: Add integration test verifying jin log displays commits from all layers
- **Deliverable**: New test function `test_log_shows_all_layer_commits()` in `tests/cli_log.rs`
- **Success Definition**: Test creates mode+scope, commits to ModeBase and ModeScope layers, runs jin log, and verifies both commits appear in output

### 2. User Persona (N/A for infrastructure test)
- Skip - this is a test infrastructure task, no user persona

### 3. Why Section
- Business value: Ensures the bug fix (S1-S2) is verified by automated tests
- Prevents regression of the jin log bug
- Validates that dynamic ref discovery works for all layer types

### 4. What / Success Criteria
- Test creates mode and scope
- Test commits to ModeBase layer (without --scope flag)
- Test commits to ModeScope layer (with --scope flag)
- Test runs `jin log` and captures output
- Test verifies BOTH commit messages appear in output
- Test fails before S1-S2, passes after S1-S2

### 5. All Needed Context

#### Documentation & References
```yaml
# Implementation files
- file: /home/dustin/projects/jin/plan/001_8630d8d70301/bug_hunt_tasks.json
  why: Contains the exact bug report reproduction case to follow
  critical: Lines 126-134 describe the test requirements precisely

- file: /home/dustin/projects/jin/src/commands/log.rs
  why: Current jin log implementation to understand output format
  pattern: Lines 36-77 show dynamic ref discovery, lines 145-156 show output format

- file: /home/dustin/projects/jin/src/core/layer.rs
  why: Layer enum with ref_path patterns and parse_layer_from_ref_path
  pattern: Lines 183-222 for parse_layer_from_ref_path, lines 57-96 for ref_path patterns

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: TestFixture pattern for isolated test environments
  pattern: Lines 26-50 for TestFixture::new(), lines 167-169 for jin() helper

- file: /home/dustin/projects/jin/tests/cli_list.rs
  why: Example test file pattern for CLI command tests
  pattern: Overall structure for tests/cli_*.rs files

# External research
- docfile: /home/dustin/projects/jin/plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/P1M2T1S3/research/02_assert_cmd_usage_examples.md
  why: assert_cmd patterns for output capture

- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: assert_cmd crate documentation for command testing

- url: https://docs.rs/predicates/latest/predicates/
  why: predicates crate for output assertions
```

#### Codebase Structure (Current)
```bash
tests/
├── cli_basic.rs          # Basic tests, has simple log test
├── cli_list.rs           # List command tests (pattern to follow)
├── cli_diff.rs           # Diff command tests
├── core_workflow.rs      # Core workflow tests
├── mode_scope_workflow.rs # Mode/scope workflow tests
└── common/
    ├── fixtures.rs       # TestFixture, jin(), jin_init() helpers
    ├── assertions.rs     # Shared assertion helpers
    └── git_helpers.rs    # Git helper functions
```

#### Desired Codebase Structure (After)
```bash
tests/
├── cli_log.rs            # NEW: Log command integration tests
│   └── test_log_shows_all_layer_commits()
└── [existing files unchanged]
```

#### Known Gotchas
```rust
// CRITICAL: TestFixture must be kept in scope or TempDir deletes immediately
// Pattern: let fixture = TestFixture::new()?; keep fixture reference alive

// CRITICAL: Set JIN_DIR environment variable BEFORE any jin operations
// Pattern: fixture.set_jin_dir(); jin().env("JIN_DIR", jin_dir)

// CRITICAL: To commit to ModeScope layer, use --scope= flag when adding
// Pattern: jin().args(["add", "file.txt", "--mode", "--scope=lang:rust"])

// CRITICAL: ModeScope requires both mode and scope to be active
// Order: create mode, create scope, mode use, scope use

// CRITICAL: Ref paths use /_ suffix for layers with children
// ModeBase: refs/jin/layers/mode/{name}/_
// ModeScope: refs/jin/layers/mode/{mode}/scope/{scope}/_

// GOTCHA: log output includes layer name in parentheses after commit hash
// Format: "commit abc1234 (mode-base)" for display
```

### 6. Implementation Blueprint

#### Data Models
- No new data models (test code)

#### Implementation Tasks
```yaml
Task 1: CREATE tests/cli_log.rs
  - IMPLEMENT: Integration test file for jin log command
  - FOLLOW pattern: tests/cli_list.rs (file structure, imports, test organization)
  - NAMING: cli_log.rs following command-based naming convention
  - PLACEMENT: tests/ directory

Task 2: IMPLEMENT test_log_shows_all_layer_commits()
  - IMPLEMENT: Test function that reproduces bug report scenario
  - FOLLOW pattern: tests/cli_list.rs test functions (TestFixture usage, jin() helper)
  - NAMING: test_log_shows_all_layer_commits
  - DEPENDENCIES: Requires S1 (parse_layer_from_ref_path) and S2 (dynamic ref listing) complete
  - LOGIC:
    1. Create TestFixture with isolated JIN_DIR
    2. Initialize Jin repository with jin_init()
    3. Create a mode using create_mode() helper
    4. Create a scope using create_scope() helper
    5. Activate mode with jin mode use
    6. Activate scope with jin scope use
    7. Create and commit file to ModeBase layer (no --scope flag)
    8. Create and commit file to ModeScope layer (with --scope flag)
    9. Run jin log and capture output
    10. Assert both commit messages appear in output
```

#### Implementation Patterns & Key Details
```rust
// Test structure following tests/cli_list.rs pattern
use predicates::prelude::*;

mod common;
use common::fixtures::*;

#[test]
fn test_log_shows_all_layer_commits() {
    // Setup: TestFixture with isolated JIN_DIR
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();
    jin_init(fixture.path(), Some(jin_dir)).unwrap();

    // Step 1: Create and activate mode
    let mode_name = format!("testmode_{}", unique_test_id());
    jin()
        .args(["mode", "create", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 2: Create and activate scope
    let scope_name = format!("scope_{}", unique_test_id());
    jin()
        .args(["scope", "create", &scope_name, "--mode", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "use", &scope_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 3: Commit to ModeBase layer (file without --scope)
    let mode_file = fixture.path().join("mode.json");
    fs::write(&mode_file, "{\"mode\": \"base\"}").unwrap();

    jin()
        .args(["add", "mode.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode base commit"])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 4: Commit to ModeScope layer (file with --scope)
    let scope_file = fixture.path().join("scope.json");
    fs::write(&scope_file, "{\"scope\": \"test\"}").unwrap();

    jin()
        .args(["add", "scope.json", "--mode", "--scope", &scope_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode scope commit"])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 5: Run jin log and verify both commits appear
    jin()
        .arg("log")
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Mode base commit"))
        .stdout(predicate::str::contains("Mode scope commit"));
}
```

#### Integration Points
```yaml
NONE: This is a test file only, no integration points to modify
```

### 7. Validation Loop

#### Level 1: Syntax & Style
```bash
cargo check --tests
cargo clippy --tests -W clippy::all
```

#### Level 2: Unit Tests
```bash
cargo test --test cli_log
```

#### Level 3: Integration Testing
```bash
# Test should FAIL before S1-S2 (ModeScope commit not shown)
# Test should PASS after S1-S2 (both commits shown)

cargo test test_log_shows_all_layer_commits
```

#### Level 4: Creative & Domain-Specific
```bash
# Verify test captures the exact bug scenario
# Manual verification: run test before and after S1-S2
```

### 8. Final Validation Checklist
- Test file created at tests/cli_log.rs
- Test follows patterns from tests/cli_list.rs
- Test uses TestFixture with isolated JIN_DIR
- Test creates mode and scope
- Test commits to both ModeBase and ModeScope layers
- Test runs jin log and captures output
- Test verifies both commit messages appear
- Test should fail before S1-S2 and pass after
- No existing tests need modification
