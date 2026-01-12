# Product Requirement Prompt: Add Integration Test for Structured File Auto-Merge

## Goal

**Feature Goal**: Create a comprehensive integration test that verifies structured files (JSON, YAML, TOML, INI) automatically deep merge across multiple layers without creating conflict files (.jinmerge)

**Deliverable**: New integration test function `test_structured_file_auto_merge()` in `tests/conflict_workflow.rs` (or dedicated test file) that validates the bug fix from P1.M1.T1.S1-S2

**Success Definition**:
- Test creates ModeBase layer with JSON config, creates ProjectBase layer with different JSON config, runs `jin apply`, and verifies:
  - No `.jinmerge` file was created
  - Merged JSON contains both layers' keys with ProjectBase taking precedence for conflicting keys
  - Test fails before S1-S2 changes and passes after (regression prevention)

## Why

- **Regression Prevention**: The bug report shows structured files were incorrectly creating `.jinmerge` conflict files even when content could be deep merged. This test ensures that fix remains permanent.
- **Completeness**: P1.M1.T1.S1 removed pre-merge conflict checks for structured files, and P1.M1.T1.S2 verified layer precedence. This test validates the end-to-end behavior.
- **Documentation**: The test serves as executable documentation of expected behavior for structured file merging.
- **Integration Coverage**: Existing unit tests in `src/merge/deep.rs` verify merge logic, but no integration test validates the full workflow across layers.

## What

Create an integration test that:

1. Sets up a Jin repository with a mode and project
2. Adds a JSON configuration file to ModeBase layer (Layer 2)
3. Adds a different JSON configuration file to ProjectBase layer (Layer 7)
4. Runs `jin apply` to merge layers
5. Asserts no `.jinmerge` file was created (automatic merge)
6. Asserts the merged JSON contains keys from both layers with correct precedence

### Success Criteria

- [ ] Test creates ModeBase with `{"common": {"a": 1}, "mode": true}` or similar
- [ ] Test creates ProjectBase with `{"common": {"a": 1, "b": 2}, "project": false}` or similar
- [ ] Test verifies no `.jinmerge` file exists after `jin apply`
- [ ] Test verifies merged result equals `{"common": {"a": 1, "b": 2}, "mode": true, "project": false}`
- [ ] Test runs via `cargo test` and passes with S1-S2 changes
- [ ] Test is added to `tests/conflict_workflow.rs` or appropriate test file

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP includes:
- Exact file paths and line numbers for all patterns to follow
- Complete TestFixture API documentation
- Existing integration test patterns to copy
- Deep merge behavior specification
- Layer precedence rules
- Expected test output format

### Documentation & References

```yaml
# MUST READ - Test Fixture Pattern
- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: TestFixture struct and helper functions for creating isolated test environments
  pattern: TestFixture::new() -> Result<Self>, jin_init(), create_mode(), unique_test_id()
  gotcha: MUST call fixture.set_jin_dir() BEFORE any Jin operations; keep _tempdir in scope

# MUST READ - Custom Assertions
- file: /home/dustin/projects/jin/tests/common/assertions.rs
  why: Assertion helpers for verifying workspace files and layer refs
  pattern: assert_workspace_file_exists(), assert_workspace_file_not_exists()
  gotcha: JSON is pretty-printed by Jin, account for formatting in assertions

# MUST READ - Existing Conflict Test Pattern
- file: /home/dustin/projects/jin/tests/conflict_workflow.rs
  why: Examples of full workflow tests including mode creation, file addition, apply, and verification
  pattern: Lines 56-175 show complete conflict workflow test
  gotcha: Always use unique_test_id() for mode names to ensure parallel test safety

# MUST READ - Deep Merge Test Pattern
- file: /home/dustin/projects/jin/tests/mode_scope_workflow.rs
  why: Example of JSON deep merge verification in integration tests (lines 349-446)
  pattern: test_mode_scope_deep_merge() shows how to verify merged JSON values
  gotcha: Use string contains() assertions rather than exact match for pretty-printed JSON

# MUST READ - Deep Merge Implementation
- file: /home/dustin/projects/jin/src/merge/deep.rs
  why: Understanding RFC 7396 merge semantics and layer precedence behavior
  pattern: Lines 74-121 show core merge logic with overlay (higher layer) winning on conflicts
  gotcha: Null values delete keys (RFC 7396), objects merge recursively, arrays merge by key if keyed

# MUST READ - Layer Precedence
- file: /home/dustin/projects/jin/src/core/layer.rs
  why: Understanding layer numbering and precedence (ModeBase=2, ProjectBase=7)
  pattern: Lines 35-46 show precedence() method returning 1-9 values
  gotcha: Higher precedence number wins (7 > 2, so ProjectBase overrides ModeBase)

# MUST READ - Recent Bug Fix Context
- docfile: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/P1M1T1S3/research/deep_merge_logic.md
  why: Context on S1/S2 changes that removed pre-merge conflict checks for structured files
  section: "Recent Changes (S1/S2)"

# MUST READ - Layer System
- docfile: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/P1M1T1S3/research/layer_system.md
  why: Complete 9-layer hierarchy with storage paths and precedence
  section: "Jin 9-Layer Hierarchy"

# EXTERNAL - tempfile Crate
- url: https://docs.rs/tempfile/latest/tempfile/
  why: Understanding TempDir auto-cleanup behavior used by TestFixture
  critical: TempDir is deleted when dropped, so _tempdir field MUST be kept in scope

# EXTERNAL - Rust Integration Tests
- url: https://doc.rust-lang.org/book/ch11-03-test-organization.html
  why: Understanding tests/ directory structure and module imports
  critical: Integration tests go in tests/ directory, use `mod common;` to share utilities

# EXTERNAL - assert_cmd Crate
- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: Testing CLI commands with Command::new(env!("CARGO_BIN_EXE_jin"))
  critical: Use .current_dir(), .env(), .args(), .assert() pattern for jin commands
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin
├── Cargo.toml                 # Workspace configuration
├── src/
│   ├── merge/
│   │   ├── deep.rs            # Deep merge implementation (RFC 7396)
│   │   └── layer.rs           # Layer merge logic (S1/S2 changes)
│   └── core/
│       └── layer.rs           # Layer enum and precedence
└── tests/
    ├── common/
    │   ├── fixtures.rs        # TestFixture, jin_init, create_mode, unique_test_id
    │   ├── assertions.rs      # assert_workspace_file_exists, assert_layer_ref_exists
    │   └── mod.rs             # Module exports
    ├── conflict_workflow.rs   # Target location for new test
    ├── mode_scope_workflow.rs # Deep merge test example (lines 349-446)
    └── core_workflow.rs       # Basic workflow patterns
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin
└── tests/
    └── conflict_workflow.rs   # MODIFIED: Add test_structured_file_auto_merge()
                                # - New test function at end of file
                                # - Uses existing TestFixture pattern
                                # - Verifies no .jinmerge + correct merge result
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: fixture.set_jin_dir() MUST be called BEFORE any Jin operations
// If not called, Jin will use global ~/.jin instead of isolated test directory
fixture.set_jin_dir();

// CRITICAL: Keep TestFixture in scope for entire test
// TempDir auto-deletes when dropped, so fixture must not go out of scope early
let fixture = TestFixture::new()?;
// ... fixture stays in scope until test ends

// CRITICAL: Always pass .env("JIN_DIR", &jin_dir) to jin() commands
// Without this, commands use global state and break test isolation
jin().args(["add", "file.json", "--mode"])
    .current_dir(fixture.path())
    .env("JIN_DIR", jin_dir)  // REQUIRED for isolation
    .assert().success();

// GOTCHA: Jin pretty-prints JSON files
// When reading back JSON, it's reformatted with 2-space indentation
// Use string contains() assertions rather than exact match
assert!(content.contains(r#""key": "value""#));

// GOTCHA: ModeBase ref path uses /_ suffix
// ModeBase has child refs (ModeScope, ModeProject), so it uses /_ to avoid Git ref conflicts
"refs/jin/layers/mode/{mode_name}/_"  // Note trailing /_

// GOTCHA: ProjectBase ref path has NO /_ suffix
// ProjectBase has no children, so no suffix needed
"refs/jin/layers/project/{project_name}"

// CRITICAL: Use unique_test_id() for mode/scope names in tests
// Ensures parallel tests don't conflict with same mode names
let mode_name = format!("test_mode_{}", unique_test_id());

// GOTCHA: jin() requires CARGO_BIN_EXE_jin env var
// This is set automatically in integration tests via env! macro
Command::new(env!("CARGO_BIN_EXE_jin"))

// CRITICAL: Structured files NEVER create .jinmerge files
// After S1 fix, only text files can create .jinmerge
// JSON/YAML/TOML/INI always deep merge without conflicts

// CRITICAL: Layer precedence flows 1 (low) to 9 (high)
// ProjectBase (7) > ModeBase (2) when merging
// Overlay (higher layer) value wins on scalar conflicts
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - test uses existing TestFixture and asserts on file system state.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE test function skeleton in tests/conflict_workflow.rs
  - IMPLEMENT: test_structured_file_auto_merge() function with #[test] and #[serial] attributes
  - FOLLOW pattern: tests/conflict_workflow.rs:56-175 (test_full_workflow_conflict_to_resolution)
  - NAMING: Use snake_case for test function: test_structured_file_auto_merge
  - SIGNATURE: fn test_structured_file_auto_merge() -> Result<(), Box<dyn std::error::Error>>
  - PLACEMENT: Add at end of file (after line 900, before mod tests section if exists)

Task 2: IMPLEMENT test fixture setup
  - IMPLEMENT: TestFixture::new(), fixture.set_jin_dir(), jin_init(), mode creation
  - FOLLOW pattern: tests/conflict_workflow.rs:57-74 (fixture and mode setup)
  - NAMING: Use unique_test_id() for mode name: format!("test_mode_{}", unique_test_id())
  - PLACEMENT: At beginning of test function
  - DEPENDENCIES: Task 1 complete

Task 3: IMPLEMENT ModeBase layer file addition
  - IMPLEMENT: Create JSON file and add with --mode flag
  - FOLLOW pattern: tests/conflict_workflow.rs:76-92 (file addition to mode)
  - CONTENT: Use JSON with nested objects: r#"{"common": {"a": 1}, "mode": true}"#
  - NAMING: File name: config.json
  - PLACEMENT: After mode activation
  - DEPENDENCIES: Task 2 complete

Task 4: IMPLEMENT ProjectBase layer file addition
  - IMPLEMENT: Create JSON file and add with --project flag (no mode flag)
  - FOLLOW pattern: tests/conflict_workflow.rs:93-107 (mode layer file addition)
  - CONTENT: Use JSON with different structure: r#"{"common": {"a": 1, "b": 2}, "project": false}"#
  - NAMING: Same file name: config.json (different content for different layer)
  - PLACEMENT: After ModeBase commit
  - DEPENDENCIES: Task 3 complete

Task 5: IMPLEMENT apply command execution
  - IMPLEMENT: Run jin apply to merge layers
  - FOLLOW pattern: tests/conflict_workflow.rs:112-120 (apply with conflict)
  - ASSERT: Verify .success() - apply should succeed (no conflicts)
  - PLACEMENT: After both layer commits
  - DEPENDENCIES: Task 4 complete

Task 6: IMPLEMENT .jinmerge non-existence assertion
  - IMPLEMENT: Verify no .jinmerge file was created
  - FOLLOW pattern: tests/common/assertions.rs:63-71 (assert_workspace_file_not_exists)
  - ASSERT: config.json.jinmerge must NOT exist
  - PLACEMENT: After apply command
  - DEPENDENCIES: Task 5 complete

Task 7: IMPLEMENT merged result assertion
  - IMPLEMENT: Read merged file and verify content
  - FOLLOW pattern: tests/mode_scope_workflow.rs:417-443 (JSON content verification)
  - ASSERT: Verify both layers' keys present with correct precedence:
    - "common": {"a": 1, "b": 2}  # ProjectBase wins on nested object
    - "mode": true                # From ModeBase only
    - "project": false            # From ProjectBase only
  - PLACEMENT: After .jinmerge assertion
  - DEPENDENCIES: Task 6 complete

Task 8: VERIFY test runs successfully
  - IMPLEMENT: Run cargo test for the specific test
  - COMMAND: cargo test test_structured_file_auto_merge -- --nocapture
  - EXPECTED: Test passes with S1-S2 changes in place
  - VALIDATION: All assertions pass, no panics
  - DEPENDENCIES: Task 7 complete
```

### Implementation Patterns & Key Details

```rust
// ========== TEST FUNCTION SKELETON ==========

// Use #[serial] because test uses JIN_DIR environment variable
#[test]
#[serial]
fn test_structured_file_auto_merge() -> Result<(), Box<dyn std::error::Error>> {
    // Test body
}

// ========== FIXTURE SETUP PATTERN ==========
// From tests/conflict_workflow.rs:57-74

let fixture = setup_test_repo().unwrap();
let jin_dir = fixture.jin_dir.clone().unwrap();

// Create a mode for testing
let mode_name = format!("test_mode_{}", unique_test_id());
jin_cmd()
    .args(["mode", "create", &mode_name])
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// Activate the mode
jin_cmd()
    .args(["mode", "use", &mode_name])
    .current_dir(fixture.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// ========== FILE ADDITION TO MODEBASE ==========
// From tests/conflict_workflow.rs:76-92

let config_path = fixture.path().join("config.json");
fs::write(&config_path, r#"{"common": {"a": 1}, "mode": true}"#).unwrap();

jin_cmd()
    .args(["add", "config.json", "--mode"])
    .current_dir(fixture.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

jin_cmd()
    .args(["commit", "-m", "Add to mode"])
    .current_dir(fixture.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// ========== FILE ADDITION TO PROJECTBASE ==========
// Similar to above but use --project flag instead of --mode
// NOTE: --project means add to ProjectBase layer (Layer 7)

fs::write(&config_path, r#"{"common": {"a": 1, "b": 2}, "project": false}"#).unwrap();

jin_cmd()
    .args(["add", "config.json", "--project"])
    .current_dir(fixture.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

jin_cmd()
    .args(["commit", "-m", "Add to project"])
    .current_dir(fixture.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// ========== APPLY COMMAND ==========
// From tests/conflict_workflow.rs:112-120

// Remove from workspace first
fs::remove_file(&config_path).unwrap();

// Run apply - should NOT create .jinmerge (structured files auto-merge)
jin_cmd()
    .arg("apply")
    .current_dir(fixture.path())
    .env("JIN_DIR", &jin_dir)
    .assert()
    .success();

// GOTCHA: Do NOT check for "Operation paused" output - that's for conflicts
// Structured files should merge cleanly without pausing

// ========== VERIFY NO .JINMERGE ==========
// From tests/common/assertions.rs:63-71

let jinmerge_path = fixture.path().join("config.json.jinmerge");
assert!(!jinmerge_path.exists(), "No .jinmerge should exist for structured files");

// ========== VERIFY MERGED RESULT ==========
// From tests/mode_scope_workflow.rs:417-443

let content = fs::read_to_string(fixture.path().join("config.json"))?;

// Verify both layers' keys are present
assert!(content.contains(r#""common":"#), "Should have common key");
assert!(content.contains(r#""a": 1"#), "Should have a:1 from both");
assert!(content.contains(r#""b": 2"#), "Should have b:2 from ProjectBase");
assert!(content.contains(r#""mode": true"#), "Should have mode:true from ModeBase");
assert!(content.contains(r#""project": false"#), "Should have project:false from ProjectBase");

// GOTCHA: Use string contains() not exact match because Jin pretty-prints JSON
// The actual output will be formatted like:
// {
//   "common": {
//     "a": 1,
//     "b": 2
//   },
//   "mode": true,
//   "project": false
// }
```

### Integration Points

```yaml
NO NEW INTEGRATION POINTS - Test uses existing infrastructure:
  - tests/common/fixtures.rs: TestFixture::new(), setup_test_repo(), unique_test_id()
  - tests/common/assertions.rs: assert_workspace_file_exists(), assert_workspace_file_not_exists()
  - Existing jin binary: jin add, jin commit, jin apply commands
  - Existing layer system: ModeBase (--mode), ProjectBase (--project) layers

MODIFIED FILES:
  - tests/conflict_workflow.rs: Add new test function only
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after writing the test function - fix before proceeding
cargo fmt --check                      # Verify formatting
cargo check --tests                    # Check compilation
cargo clippy --tests                    # Lint checks

# Run on specific file
cargo check --test conflict_workflow    # Check specific test file

# Expected: Zero errors. Fix any issues before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the specific new test
cargo test test_structured_file_auto_merge -- --nocapture

# Run all conflict_workflow tests to ensure no regressions
cargo test --test conflict_workflow -- --nocapture

# Run all tests in tests/ directory
cargo test --tests

# Expected: New test passes. If failing, debug root cause.
# Common issues:
# - JIN_DIR not set (add .env("JIN_DIR", &jin_dir))
# - File path incorrect (use fixture.path().join("filename"))
# - JSON format mismatch (use contains() not exact match)
```

### Level 3: Integration Testing (System Validation)

```bash
# Run full test suite to ensure no regressions
cargo test --all

# Verify test behavior:
# 1. Test should PASS with S1-S2 changes in place
# 2. If you revert S1 changes (restore pre-merge check), test should FAIL
#    This confirms the test detects the bug it's meant to prevent

# Manual verification of behavior:
cd /tmp && mkdir test_jin && cd test_jin
jin init
jin mode create testmode
jin mode use testmode
echo '{"common": {"a": 1}, "mode": true}' > config.json
jin add config.json --mode
jin commit -m "Add to mode"
echo '{"common": {"a": 1, "b": 2}, "project": false}' > config.json
jin add config.json --project
jin commit -m "Add to project"
rm config.json
jin apply
# Expected: config.json exists with merged content, no .jinmerge file

# Expected: All validations pass, merge is automatic
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test with other structured file formats (YAML, TOML)
echo 'common:\n  a: 1\nmode: true' > config.yaml
jin add config.yaml --mode && jin commit -m "YAML mode"
echo 'common:\n  a: 1\n  b: 2\nproject: false' > config.yaml
jin add config.yaml --project && jin commit -m "YAML project"
rm config.yaml && jin apply
# Expected: YAML merges automatically like JSON

# Test edge cases:
# 1. Empty objects merge
# 2. Null values delete keys (RFC 7396)
# 3. Array merging with keyed items
# 4. Nested object deep merge (already covered by main test)

# Verify test is parallel-safe (can run with other tests)
cargo test --tests -- --test-threads=4
# Expected: All tests pass, no race conditions due to unique_test_id()
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --test conflict_workflow`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] No compilation errors: `cargo check --tests`
- [ ] No linting warnings: `cargo clippy --tests`

### Feature Validation

- [ ] Test creates ModeBase with `{"common": {"a": 1}, "mode": true}` content
- [ ] Test creates ProjectBase with `{"common": {"a": 1, "b": 2}, "project": false}` content
- [ ] Test verifies no `.jinmerge` file exists after `jin apply`
- [ ] Test verifies merged JSON contains both layers' keys
- [ ] Test verifies ProjectBase takes precedence on conflicting keys
- [ ] Test runs successfully with `cargo test test_structured_file_auto_merge`

### Code Quality Validation

- [ ] Follows existing test patterns from tests/conflict_workflow.rs
- [ ] Uses TestFixture pattern correctly (set_jin_dir, keep in scope)
- [ ] Uses unique_test_id() for mode name (parallel-safe)
- [ ] Uses #[serial] attribute (JIN_DIR environment variable usage)
- [ ] Uses .env("JIN_DIR", &jin_dir) on all jin() commands
- [ ] Uses string contains() assertions for pretty-printed JSON
- [ ] Test is placed in correct file (tests/conflict_workflow.rs)

### Documentation & Deployment

- [ ] Test function has clear name: test_structured_file_auto_merge
- [ ] Test comments explain the bug being prevented
- [ ] Test serves as documentation for expected merge behavior
- [ ] No new environment variables or dependencies required

---

## Anti-Patterns to Avoid

- ❌ Don't use exact string match for JSON - Jin pretty-prints, use `contains()`
- ❌ Don't forget `fixture.set_jin_dir()` before any Jin operations
- ❌ Don't use hardcoded mode names like "test_mode" - use `unique_test_id()`
- ❌ Don't skip `#[serial]` attribute - test uses JIN_DIR environment variable
- ❌ Don't forget `.env("JIN_DIR", &jin_dir)` on jin() commands
- ❌ Don't let TestFixture go out of scope early - keep until test ends
- ❌ Don't expect "Operation paused" output - that's only for text conflicts
- ❌ Don't verify .jinmerge file exists - the whole point is it should NOT exist
- ❌ Don't test with text files - this test is specifically for structured files
- ❌ Don't use `--mode --project` together - we want separate ModeBase and ProjectBase layers

---

## Test Implementation Checklist

Use this checklist during implementation to ensure all steps are completed:

### Setup Phase
- [ ] Create test function with `#[test]` and `#[serial]` attributes
- [ ] Create TestFixture and set JIN_DIR
- [ ] Initialize Jin repository with `jin_init()`
- [ ] Create unique mode name using `unique_test_id()`
- [ ] Create mode using `jin mode create`
- [ ] Activate mode using `jin mode use`

### ModeBase Layer Setup
- [ ] Create config.json with ModeBase content
- [ ] Add file with `jin add --mode`
- [ ] Commit with `jin commit -m "Add to mode"`

### ProjectBase Layer Setup
- [ ] Modify config.json with ProjectBase content
- [ ] Add file with `jin add --project`
- [ ] Commit with `jin commit -m "Add to project"`

### Apply Phase
- [ ] Remove config.json from workspace
- [ ] Run `jin apply`
- [ ] Verify command succeeds (no .success() assertion failures)

### Verification Phase
- [ ] Verify no .jinmerge file exists
- [ ] Read merged config.json content
- [ ] Verify `common.a` exists (from both layers)
- [ ] Verify `common.b` exists (from ProjectBase)
- [ ] Verify `mode: true` exists (from ModeBase)
- [ ] Verify `project: false` exists (from ProjectBase)

### Cleanup Phase
- [ ] Fixture auto-cleanup on drop (verify no manual cleanup needed)

---

## Confidence Score: 9/10

**Reasoning for 9/10**:
- All necessary file paths and line numbers provided
- Existing test patterns are well-documented and can be directly copied
- TestFixture pattern is well-established with clear gotchas documented
- Deep merge behavior is specified with RFC 7396 reference
- Layer precedence rules are clearly defined
- Validation commands are project-specific and executable

**Why not 10/10**:
- Minor uncertainty about exact test output format (string matching may need adjustment)
- No explicit instruction for where to place test in conflict_workflow.rs (end vs middle)
- Test may need adjustment based on actual Jin behavior during manual testing

**Validation**: The completed PRP should enable an AI agent unfamiliar with the codebase to implement this integration test successfully using only the PRP content and codebase access.
