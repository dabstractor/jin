# Product Requirement Prompt (PRP): Add Test for Nested Object Deep Merge

---

## Goal

**Feature Goal**: Add integration test verifying recursive deep merge behavior for nested JSON objects across multiple layers (Layer 2: ModeBase and Layer 7: ProjectBase)

**Deliverable**: A new integration test function `test_nested_object_deep_merge()` in `tests/conflict_workflow.rs` that validates the deep merge engine correctly handles nested object merging at multiple levels of depth.

**Success Definition**: Test passes, confirming that:
1. Nested objects merge recursively (not replace)
2. Values at the same nesting level override (layer precedence: higher layer wins)
3. Values at deeper nesting levels merge when parent keys differ
4. No `.jinmerge` conflict files are created (clean merge)
5. Result matches RFC 7396 JSON Merge Patch semantics with Jin's layer precedence

## User Persona

**Target User**: Jin CLI developer/maintainer validating merge engine behavior

**Use Case**: As a developer, I need to verify that the deep merge engine correctly handles nested JSON structures when merging configuration files across different layers. This ensures that users can have base configurations in ModeBase layer that are selectively overridden/extended in ProjectBase layer without manual conflict resolution.

**User Journey**:
1. Developer runs `cargo test test_nested_object_deep_merge`
2. Test creates nested JSON config in ModeBase (Layer 2) with 3-level nesting
3. Test creates overlapping nested JSON config in ProjectBase (Layer 7)
4. Test applies merge and verifies recursive merge behavior
5. Test assertions confirm expected merged result without conflicts

**Pain Points Addressed**:
- Lack of test coverage for nested object deep merge > 2 levels deep
- Uncertainty whether layer precedence works correctly at deep nesting levels
- Risk of regression when modifying merge engine logic

## Why

- **Business value**: Ensures configuration management works correctly for complex nested configs (database credentials, feature flags with nested settings, multi-level environment configs)
- **Integration with existing features**: Validates PRD §11.1 "Structured Merge Rules" and §11.2 "Merge Priority" after the bug fix in P1.M1.T1 (removing pre-merge conflict check)
- **Problems this solves**: Provides confidence that the deep merge engine handles real-world nested configurations without requiring manual conflict resolution

## What

Add an integration test that creates two layers with deeply nested JSON configurations and verifies they merge correctly.

### Test Data

**Layer 2 (ModeBase) - base configuration:**
```json
{
  "config": {
    "database": {
      "host": "localhost",
      "port": 5432
    }
  },
  "app": "base"
}
```

**Layer 7 (ProjectBase) - override configuration:**
```json
{
  "config": {
    "database": {
      "port": 5433,
      "ssl": true
    }
  },
  "app": "override"
}
```

**Expected merged result:**
```json
{
  "config": {
    "database": {
      "host": "localhost",
      "port": 5433,
      "ssl": true
    }
  },
  "app": "override"
}
```

### Success Criteria

- [ ] Test creates ModeBase layer (Layer 2) with nested JSON config
- [ ] Test creates ProjectBase layer (Layer 7) with overlapping nested JSON config
- [ ] Apply operation produces merged result without conflicts
- [ ] No `.jinmerge` file created (clean auto-merge)
- [ ] `config.database.host` preserved from ModeBase (localhost)
- [ ] `config.database.port` overridden by ProjectBase (5433)
- [ ] `config.database.ssl` added from ProjectBase (true)
- [ ] `app` overridden by ProjectBase ("override")
- [ ] Test passes when run with `cargo test test_nested_object_deep_merge`

## All Needed Context

### Context Completeness Check

This PRP passes the "No Prior Knowledge" test - a developer unfamiliar with the codebase can implement this test using only the content below plus standard Rust tooling.

### Documentation & References

```yaml
# MUST READ - Core merge engine implementation
- file: src/merge/deep.rs
  why: Contains deep_merge() function with recursive object merge logic
  pattern: Lines 84-101 show (MergeValue::Object, MergeValue::Object) match arm
  critical: Null deletion semantics, recursive merge, layer precedence

- file: src/merge/value.rs
  why: MergeValue enum definition - understand data structures
  pattern: Lines 10-27 define MergeValue enum with Object variant
  gotcha: Uses IndexMap (not HashMap) to preserve key order

- file: src/core/layer.rs
  why: Layer enum with precedence values and ref_path() method
  pattern: Lines 1-80 define Layer enum (1=GlobalBase to 9=WorkspaceActive)
  gotcha: Layer 2 = ModeBase, Layer 7 = ProjectBase

# MUST READ - Test patterns to follow
- file: tests/conflict_workflow.rs
  why: Contains test_structured_file_auto_merge() - the pattern to follow
  pattern: Lines 915-1010 show complete test workflow
  critical: Uses setup_test_repo(), jin_cmd(), layer flags (--mode, --mode --project)

- file: tests/mode_scope_workflow.rs
  why: Contains test_mode_scope_deep_merge() - another deep merge test
  pattern: Lines 349-446 show deep merge test with nested features object
  gotcha: JSON is pretty-printed when written to workspace (use formatted comparison)

- file: tests/common/fixtures.rs
  why: TestFixture, setup_test_repo(), create_mode(), jin() helpers
  pattern: Lines 137-142 for setup_test_repo(), lines 183-210 for create_mode()
  critical: Always pass .env("JIN_DIR", &jin_dir) for test isolation

- file: tests/common/assertions.rs
  why: assert_workspace_file() for verifying merged content
  pattern: Lines 18-35 show assertion helper that checks file existence and content
  gotcha: Expect pretty-printed JSON (2-space indentation)

# MUST READ - Architecture documentation
- docfile: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/merge_engine_analysis.md
  why: Describes the merge engine flow, deep merge semantics, and layer precedence
  section: "Deep Merge Implementation" (lines 59-94)
  critical: Confirms deep merge is correct and follows RFC 7396 semantics

# EXTERNAL - Rust testing best practices
- url: https://doc.rust-lang.org/book/ch11-00-testing.html
  why: How to write integration tests in Rust
  critical: Integration tests go in tests/ directory, use #[test] attribute

- url: https://docs.rs/serde_json/latest/serde_json/macro.json.html
  why: Using json! macro for readable test data construction
  critical: json! macro creates serde_json::Value, but we write raw JSON strings for layer content
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── Cargo.toml
├── src/
│   ├── merge/
│   │   ├── deep.rs           # deep_merge() function (CORRECT - no changes needed)
│   │   ├── layer.rs          # merge_layers(), layer precedence logic
│   │   └── value.rs          # MergeValue enum definition
│   ├── core/
│   │   └── layer.rs          # Layer enum (1-9 precedence levels)
│   └── test_utils.rs         # Unit test context helpers
├── tests/
│   ├── common/
│   │   ├── mod.rs            # Test utilities module
│   │   ├── fixtures.rs       # TestFixture, setup_test_repo(), create_mode()
│   │   └── assertions.rs     # assert_workspace_file(), assert_layer_ref_exists()
│   ├── conflict_workflow.rs  # [TARGET FILE - add test here]
│   ├── mode_scope_workflow.rs
│   └── pull_merge.rs
└── plan/
    └── 001_8630d8d70301/
        └── bugfix/
            └── 001_d2716c9eb3cf/
                ├── architecture/
                │   └── merge_engine_analysis.md
                └── P1M1T2S1/
                    └── PRP.md  # This file
```

### Desired Codebase Tree (New Test)

```bash
tests/
├── conflict_workflow.rs  # MODIFIED - add test_nested_object_deep_merge() function
│   └── [New Test Function]
│       └── test_nested_object_deep_merge()  # Validates nested object recursive merge
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: TestFixture must stay in scope
// When TempDir inside TestFixture is dropped, the directory is deleted immediately
// WRONG:
// let test = setup_test_repo().unwrap();
// drop(test);  // Directory deleted!
// jin().args([...])  // Will fail!

// RIGHT: Keep fixture in scope
let fixture = setup_test_repo().unwrap();
// ... use fixture.path() throughout test

// CRITICAL: Always set JIN_DIR for test isolation
// WRONG:
// jin().args(["init"])  // Uses global ~/.jin - not isolated!

// RIGHT:
// fixture.set_jin_dir();  // Sets environment variable
// OR
// jin().args(["init"]).env("JIN_DIR", &jin_dir)

// CRITICAL: Layer flag combinations determine which layer file goes to
// --mode         -> ModeBase (Layer 2)
// --mode --project -> ModeProject (Layer 5)
// (no flags)     -> ProjectBase (Layer 7)
// --global       -> GlobalBase (Layer 1)

// CRITICAL: JSON is pretty-printed when written to workspace
// Jin uses 2-space indentation for JSON output
// Use pretty-printed JSON in assertions:
r#"{
  "config": {
    "database": {
      "host": "localhost",
      "port": 5433,
      "ssl": true
    }
  },
  "app": "override"
}"#

// CRITICAL: serial_test attribute for tests using JIN_DIR
// Tests that use environment variables must run sequentially
use serial_test::serial;

#[test]
#[serial]
fn test_nested_object_deep_merge() { ... }

// CRITICAL: Layer 7 is ProjectBase (NOT --mode --project which is Layer 5)
// To add to ProjectBase (Layer 7), use NO flags (default layer)
// Reference: src/core/layer.rs line 40-47
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - test uses existing structures:

```rust
// Existing MergeValue enum in src/merge/value.rs
pub enum MergeValue {
    Object(IndexMap<String, MergeValue>),  // Ordered key-value pairs
    // ... other variants
}

// Existing Layer enum in src/core/layer.rs
pub enum Layer {
    ModeBase,      // Layer 2
    ProjectBase,   // Layer 7
    // ... other layers
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: ADD test_nested_object_deep_merge() function to tests/conflict_workflow.rs
  - IMPLEMENT: Integration test function with #[test] and #[serial] attributes
  - FOLLOW pattern: test_structured_file_auto_merge() (lines 915-1010 in same file)
  - NAMING: test_nested_object_deep_merge - descriptive name indicating nested object behavior
  - PLACEMENT: After existing test functions in conflict_workflow.rs

Task 2: SETUP test fixture and mode initialization
  - IMPLEMENT: Create fixture using setup_test_repo(), create mode
  - FOLLOW pattern: Lines 916-933 in test_structured_file_auto_merge()
  - ISOLATION: Use unique_test_id() for mode name to ensure parallel test safety
  - DEPENDENCIES: Import setup_test_repo, create_mode from common::fixtures

Task 3: CREATE Layer 2 (ModeBase) configuration with nested JSON
  - IMPLEMENT: Write config.json file with 3-level nested structure
  - CONTENT: {"config": {"database": {"host": "localhost", "port": 5432}}, "app": "base"}
  - ADD to layer: jin_cmd().args(["add", "config.json", "--mode"])
  - COMMIT: jin_cmd().args(["commit", "-m", "Add base config"])

Task 4: CREATE Layer 7 (ProjectBase) configuration with overlapping nested JSON
  - IMPLEMENT: Overwrite config.json with overlapping nested structure
  - CONTENT: {"config": {"database": {"port": 5433, "ssl": true}}, "app": "override"}
  - ADD to layer: jin_cmd().args(["add", "config.json"]) (NO flags = ProjectBase)
  - COMMIT: jin_cmd().args(["commit", "-m", "Add override config"])

Task 5: VERIFY no conflict file created (clean merge expected)
  - IMPLEMENT: Check that .jinmerge file does NOT exist after apply
  - FOLLOW pattern: Lines 992-995 in test_structured_file_auto_merge()
  - ASSERTION: assert!(!config_path.with_extension(".jinmerge").exists())

Task 6: RUN jin apply to merge layers into workspace
  - IMPLEMENT: Execute jin apply command
  - FOLLOW pattern: Line 996 in test_structured_file_auto_merge()
  - COMMAND: jin_cmd().arg("apply").current_dir(fixture.path()).env("JIN_DIR", &jin_dir)

Task 7: VERIFY merged result matches expected output
  - IMPLEMENT: Read merged config.json and assert content matches expected
  - FOLLOW pattern: Lines 997-1005 (use assert_workspace_file or manual assertion)
  - ASSERTIONS:
    - config.database.host == "localhost" (from ModeBase)
    - config.database.port == 5433 (overridden by ProjectBase)
    - config.database.ssl == true (added by ProjectBase)
    - app == "override" (overridden by ProjectBase)
  - FORMAT: Use pretty-printed JSON (2-space indentation)

Task 8: ADD test documentation comments
  - IMPLEMENT: Add doc comment explaining test scenario and expected behavior
  - FOLLOW pattern: Lines 912-914 in test_structured_file_auto_merge()
  - CONTENT: Describe layers, nested structure, merge semantics being tested
```

### Implementation Patterns & Key Details

```rust
// Pattern: Test function structure with serial attribute
use serial_test::serial;

/// Test nested object deep merge across layers
///
/// Scenario:
/// - Layer 2 (ModeBase): {"config": {"database": {"host": "localhost", "port": 5432}}, "app": "base"}
/// - Layer 7 (ProjectBase): {"config": {"database": {"port": 5433, "ssl": true}}, "app": "override"}
///
/// Expected result after deep merge:
/// - config.database.host: "localhost" (preserved from ModeBase)
/// - config.database.port: 5433 (overridden by ProjectBase)
/// - config.database.ssl: true (added from ProjectBase)
/// - app: "override" (overridden by ProjectBase)
#[test]
#[serial]
fn test_nested_object_deep_merge() -> Result<(), Box<dyn std::error::Error>> {
    // PATTERN: Setup fixture and mode (follow conflict_workflow.rs:916-933)
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Add to Layer 2 (ModeBase) with --mode flag
    let config_path = fixture.path().join("config.json");
    fs::write(
        &config_path,
        r#"{"config": {"database": {"host": "localhost", "port": 5432}}, "app": "base"}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add base config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Add to Layer 7 (ProjectBase) with NO flags
    // GOTCHA: ProjectBase is default layer, no --mode flag needed
    fs::write(
        &config_path,
        r#"{"config": {"database": {"port": 5433, "ssl": true}}, "app": "override"}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add override config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Verify clean merge (no conflict file)
    let merge_conflict_path = config_path.with_extension(".jinmerge");
    assert!(
        !merge_conflict_path.exists(),
        "No conflict file should be created for mergeable nested objects"
    );

    // PATTERN: Apply merge to workspace
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Verify merged result (use pretty-printed JSON for comparison)
    let merged_content = fs::read_to_string(&config_path)?;
    let merged: serde_json::Value = serde_json::from_str(&merged_content)?;

    // Verify nested merge behavior
    assert_eq!(
        merged["config"]["database"]["host"],
        "localhost",
        "host should be preserved from ModeBase"
    );
    assert_eq!(
        merged["config"]["database"]["port"],
        5433,
        "port should be overridden by ProjectBase"
    );
    assert_eq!(
        merged["config"]["database"]["ssl"],
        true,
        "ssl should be added from ProjectBase"
    );
    assert_eq!(
        merged["app"],
        "override",
        "app should be overridden by ProjectBase"
    );

    Ok(())
}

// GOTCHA: Import statements needed at top of conflict_workflow.rs
// These are already present in the file, but confirming:
use std::fs;
mod common;
use common::fixtures::*;
use serial_test::serial;
```

### Integration Points

```yaml
TEST_FILE:
  - modify: tests/conflict_workflow.rs
  - location: Add after test_structured_file_auto_merge() function (after line ~1010)
  - pattern: Follow existing test function structure

FIXTURES:
  - import: setup_test_repo() from common::fixtures
  - import: create_mode() from common::fixtures
  - import: unique_test_id() from common::fixtures

ASSERTIONS:
  - use: fs::read_to_string() to read merged JSON
  - use: serde_json::from_str() to parse JSON for structured assertions
  - use: assert!() or assert_eq!() for value verification

LAYER_SYSTEM:
  - Layer 2 (ModeBase): Created with --mode flag
  - Layer 7 (ProjectBase): Default layer (no flags)
  - Reference: src/core/layer.rs for layer enum definition

MERGE_ENGINE:
  - deep_merge(): src/merge/deep.rs (lines 74-121)
  - Object merge case: lines 84-101 (recursive merge logic)
  - merge_layers(): src/merge/layer.rs (orchestrates layer merging)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding the test function - fix any errors before proceeding
cargo fmt -- --check tests/conflict_workflow.rs     # Check formatting
cargo clippy --tests 2>&1 | grep conflict_workflow  # Check for linting issues

# Auto-fix formatting if needed
cargo fmt

# Expected: Zero formatting/linting errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Run the New Test (Component Validation)

```bash
# Run just the new test function
cargo test test_nested_object_deep_merge -- --nocapture

# Run all conflict_workflow tests to ensure no regression
cargo test --test conflict_workflow -- --nocapture

# Expected: New test passes. If failing, debug with:
# - Check test output for assertion failures
# - Verify layer content was added correctly (check git refs)
# - Manually inspect merged JSON in temp directory (test keeps it alive briefly)
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all tests to ensure no regression elsewhere in codebase
cargo test --all

# Run specific test modules that touch merge functionality
cargo test --test conflict_workflow
cargo test --test mode_scope_workflow
cargo test --test pull_merge

# Expected: All tests pass. The new test should not break any existing tests.
# If other tests fail, investigate whether the new test is causing interference
# (unlikely with proper test isolation via unique_test_id() and serial_test)
```

### Level 4: Manual Verification (Domain-Specific Validation)

```bash
# Manual test to verify merge behavior manually (optional but recommended)

# Create a test project
cd /tmp && rm -rf jin_test && mkdir jin_test && cd jin_test
export JIN_DIR=/tmp/jin_test_jin
jin init
git init

# Create and activate a mode
jin mode create testmode
jin mode use testmode

# Create base config (ModeBase)
cat > config.json << 'EOF'
{
  "config": {
    "database": {
      "host": "localhost",
      "port": 5432
    }
  },
  "app": "base"
}
EOF
jin add config.json --mode
jin commit -m "Base config"

# Create override config (ProjectBase)
cat > config.json << 'EOF'
{
  "config": {
    "database": {
      "port": 5433,
      "ssl": true
    }
  },
  "app": "override"
}
EOF
jin add config.json
jin commit -m "Override config"

# Apply and verify
jin apply
cat config.json

# Expected: Pretty-printed JSON with merged nested database object
# - host: "localhost" (from ModeBase)
# - port: 5433 (overridden)
# - ssl: true (added)
# - app: "override" (overridden)
```

## Final Validation Checklist

### Technical Validation

- [ ] Test compiles without errors: `cargo build --tests`
- [ ] Test passes when run individually: `cargo test test_nested_object_deep_merge`
- [ ] Test passes in full suite: `cargo test --test conflict_workflow`
- [ ] All tests pass: `cargo test --all`
- [ ] No linting errors: `cargo clippy --tests`
- [ ] Code is formatted: `cargo fmt --check`

### Feature Validation

- [ ] Test creates Layer 2 (ModeBase) with nested JSON config
- [ ] Test creates Layer 7 (ProjectBase) with overlapping nested JSON config
- [ ] No `.jinmerge` conflict file created (clean merge)
- [ ] `config.database.host` preserved from ModeBase
- [ ] `config.database.port` overridden by ProjectBase (5433)
- [ ] `config.database.ssl` added from ProjectBase (true)
- [ ] `app` overridden by ProjectBase ("override")
- [ ] All success criteria met

### Code Quality Validation

- [ ] Follows existing test pattern from `test_structured_file_auto_merge()`
- [ ] Uses `#[serial]` attribute for JIN_DIR isolation
- [ ] Uses `unique_test_id()` for mode naming
- [ ] Has clear documentation comment explaining test scenario
- [ ] Assertion messages are descriptive on failure
- [ ] Test function name is descriptive: `test_nested_object_deep_merge`

### Documentation & Deployment

- [ ] Test is self-documenting with clear doc comment
- [ ] Test failure messages help identify what went wrong
- [ ] No environment-specific assumptions (uses unique_test_id)

---

## Anti-Patterns to Avoid

- ❌ Don't skip the `#[serial]` attribute - tests using JIN_DIR must run sequentially
- ❌ Don't forget to call `fixture.set_jin_dir()` or `.env("JIN_DIR", &jin_dir)` - tests will interfere
- ❌ Don't drop the TestFixture early - directory gets deleted while still needed
- ❌ Don't use hardcoded mode names like "test_mode" - use `unique_test_id()` for parallel safety
- ❌ Don't compare raw JSON strings for complex objects - parse and use structured assertions
- ❌ Don't assume JSON is compact - Jin pretty-prints with 2-space indentation
- ❌ Don't use `--mode --project` flags for Layer 7 - ProjectBase is the default layer (no flags)
- ❌ Don't forget to commit after `jin add` - file won't be in layer without commit
- ❌ Don't ignore the possibility of `.jinmerge` file - explicitly assert it doesn't exist
- ❌ Don't test layer content with git commands directly - use `jin apply` to test the actual merge behavior

---

## Appendix: Reference Test (test_structured_file_auto_merge)

For reference, here's the existing test that demonstrates the pattern to follow:

```rust
/// Test structured file auto-merge without conflicts
///
/// Scenario:
/// - Layer 2 (ModeBase): {"common": {"a": 1}, "mode": true}
/// - Layer 5 (ModeProject): {"common": {"a": 1, "b": 2}, "project": false}
/// Expected: Deep merge combines objects, no .jinmerge file created
#[test]
fn test_structured_file_auto_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add config.json to ModeBase layer (Layer 2)
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"common": {"a": 1}, "mode": true}"#)?;

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

    // Modify config.json and add to ModeProject layer (Layer 5)
    fs::write(
        &config_path,
        r#"{"common": {"a": 1, "b": 2}, "project": false}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode", "--project"])
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

    // Remove from workspace to test apply behavior
    fs::remove_file(&config_path)?;

    // Verify no conflict file
    let merge_conflict_path = config_path.with_extension(".jinmerge");
    assert!(
        !merge_conflict_path.exists(),
        "No conflict file should be created for mergeable structured files"
    );

    // Apply and verify merged result
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    let content = fs::read_to_string(&config_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;

    assert_eq!(parsed["common"]["a"], 1, "common.a should be preserved");
    assert_eq!(parsed["common"]["b"], 2, "common.b should be added");
    assert_eq!(parsed["mode"], true, "mode should be preserved");
    assert_eq!(parsed["project"], false, "project should be added");

    Ok(())
}
```

---

## Confidence Score

**Score: 9/10** for one-pass implementation success

**Rationale**:
- Comprehensive context provided (file locations, patterns, gotchas)
- Reference test provided as exact template to follow
- All validation commands verified to work in this codebase
- Clear dependency-ordered implementation tasks
- Known gotchas documented to prevent common mistakes

**Remaining risk**: Small chance of environment-specific issues (Git version, Rust toolchain) but these are unlikely to affect test implementation.

---

## Sources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [serde_json json! macro](https://docs.rs/serde_json/latest/serde_json/macro.json.html)
- [RFC 7396 - JSON Merge Patch](https://tools.ietf.org/html/rfc7396)
- Project architecture: `plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/merge_engine_analysis.md`
- Reference test: `tests/conflict_workflow.rs:test_structured_file_auto_merge`
- Merge implementation: `src/merge/deep.rs:deep_merge()`
- Layer definition: `src/core/layer.rs:Layer`
