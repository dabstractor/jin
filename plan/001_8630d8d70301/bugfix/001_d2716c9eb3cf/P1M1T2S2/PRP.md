name: "PRP: Add Test for Array Key-Based Merging"
description: |

---

## Goal

**Feature Goal**: Create an integration test that verifies the deep_merge() function correctly merges arrays using key-based matching (default: ["id", "name"])

**Deliverable**: Integration test function `test_array_key_based_merge()` in `/home/dustin/projects/jin/tests/conflict_workflow.rs`

**Success Definition**:
- Test passes successfully with `cargo test test_array_key_based_merge`
- Test verifies that arrays with objects containing matching "id" or "name" fields are merged correctly
- Test confirms items with same id are merged (not duplicated) and new items are appended
- Test validates no `.jinmerge` conflict file is created for mergeable arrays
- Test confirms layer precedence works correctly (higher layer values override lower layer)

## User Persona (if applicable)

**Target User**: Developer working on the Jin CLI deep merge functionality

**Use Case**: Validating that the deep merge engine correctly handles array merging scenarios where objects contain key fields (id/name)

**User Journey**:
1. Developer adds array with keyed objects to Layer 2 (ModeBase)
2. Developer adds different array to Layer 7 (ProjectBase)
3. Jin applies deep merge when running `jin apply`
4. Arrays merge by key: matching items combined, new items appended
5. No conflict file created for mergeable content

**Pain Points Addressed**:
- Ensures array merging works correctly for complex configuration scenarios (e.g., lists of tasks, services, users)
- Prevents data loss when merging arrays with keyed objects
- Validates that Jin's layer precedence is respected in array merge scenarios

## Why

- **Business value**: Array merging is critical for real-world configuration management (lists of services, tasks, users, etc.)
- **Integration with existing features**: Complements the nested object deep merge test (P1.M1.T2.S1) and structured file auto-merge test (P1.M1.T1.S3)
- **Problems this solves**: Validates that the deep_merge() function's keyed array merging logic works correctly in integration scenarios

## What

Create an integration test that:
1. Sets up a ModeBase layer with an array containing objects with id/name fields
2. Sets up a ProjectBase layer with an array containing some matching keys and some new keys
3. Verifies arrays merge correctly (matching keys merge, new keys append)
4. Confirms no conflict file is created for mergeable arrays
5. Validates the merged result matches expected output

### Success Criteria

- [ ] Test function `test_array_key_based_merge` exists in `tests/conflict_workflow.rs`
- [ ] Test creates Layer 2 (ModeBase) with array `[{"id": 1, "name": "task1", "status": "pending"}]`
- [ ] Test creates Layer 7 (ProjectBase) with array `[{"id": 1, "priority": "high"}, {"id": 2, "name": "task2"}]`
- [ ] Test verifies merged result: `[{"id": 1, "name": "task1", "status": "pending", "priority": "high"}, {"id": 2, "name": "task2"}]`
- [ ] Test confirms no `.jinmerge` file is created
- [ ] Test validates layer precedence (Layer 7 values override Layer 2 for id=1)
- [ ] All assertions pass: `cargo test test_array_key_based_merge`

## All Needed Context

### Context Completeness Check

**Question**: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"

**Answer**: Yes - This PRP provides:
- Exact file locations and line numbers for reference code
- Complete test structure to follow from existing tests
- Specific deep_merge implementation details
- All imports and helper functions needed
- Expected test commands and validation criteria

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# EXISTING TEST PATTERN TO FOLLOW
- file: /home/dustin/projects/jin/tests/conflict_workflow.rs
  why: Contains test_nested_object_deep_merge (lines 1036-1151) - use as exact template
  pattern: Integration test structure for deep merge functionality
  gotcha: Must use jin_cmd() with .env("JIN_DIR", &jin_dir) for test isolation

# DEEP MERGE FUNCTION IMPLEMENTATION
- file: /home/dustin/projects/jin/src/merge/deep.rs
  why: Contains the deep_merge() and deep_merge_with_config() functions
  section: Lines 55-121 for main function, 134-169 for array merge logic
  gotcha: Default key fields are ["id", "name"] - checked in priority order

# MERGE ENGINE ANALYSIS
- docfile: /home/dustin/projects/jin/plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/merge_engine_analysis.md
  why: Explains array merging behavior and layer precedence
  gotcha: Arrays with matching keys are merged, non-matching items appended

# TEST UTILITIES
- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Contains setup_test_repo(), unique_test_id(), jin() helper functions
  section: Lines 137-142 for setup_test_repo, 253-269 for unique_test_id
  gotcha: Keep TestFixture in scope (don't let it drop prematurely)

# EXTERNAL RESEARCH - Best Practices
- url: https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md
  why: Gold standard for key-based array merging patterns
  critical: Order preservation - base order maintained, new items appended

- url: https://github.com/lodash/lodash/issues/1313
  why: Critical bug to avoid - empty arrays must replace, not be ignored
  critical: Test empty array scenarios explicitly

- docfile: /home/dustin/projects/jin/plan/001_8630d8d70301/docs/ARRAY_MERGE_TESTING_QUICK_REFERENCE.md
  why: Contains test patterns and edge cases for array merging
  section: Lines 142-263 for copy-paste test patterns
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── Cargo.toml                 # Project config with test dependencies
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── merge/
│   │   └── deep.rs            # DEEP MERGE IMPLEMENTATION (lines 55-121)
│   └── test_utils.rs          # Unit test utilities
├── tests/
│   ├── common/
│   │   ├── fixtures.rs        # TEST HELPERS (setup_test_repo, unique_test_id)
│   │   └── mod.rs
│   └── conflict_workflow.rs   # TARGET FILE - Add test here
└── plan/
    └── 001_8630d8d70301/
        └── bugfix/
            └── 001_d2716c9eb3cf/
                ├── architecture/
                │   └── merge_engine_analysis.md
                └── P1M1T2S2/
                    └── PRP.md  # THIS FILE
```

### Desired Codebase Tree (files to be added)

```bash
# No new files to create - add test function to existing file:

tests/conflict_workflow.rs
├── [existing tests...]
├── test_nested_object_deep_merge (line 1051) - REFERENCE PATTERN
└── test_array_key_based_merge (ADD AFTER line 1151) - NEW TEST
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Test isolation with JIN_DIR
// Always set .env("JIN_DIR", &jin_dir) for all jin_cmd() calls
// Tests share global state without proper isolation

// CRITICAL: TestFixture lifetime
// The _tempdir field MUST stay in scope or directory is deleted prematurely
// Don't reassign fixture variable

// CRITICAL: Layer setup order
// 1. Create mode: jin_cmd().args(["mode", "create", &mode_name])
// 2. Use mode: jin_cmd().args(["mode", "use", &mode_name])
// 3. Add file to ModeBase: jin_cmd().args(["add", "file.json", "--mode"])
// 4. Add file to ProjectBase: jin_cmd().args(["add", "file.json"]) (no flag)

// CRITICAL: Array key extraction behavior
// extract_array_keys() returns None if ANY item lacks key fields
// This causes entire array to be replaced instead of merged
// See /home/dustin/projects/jin/src/merge/deep.rs lines 177-203

// CRITICAL: Order preservation
// Base array order is maintained
// Overlay items matching base keys are merged IN PLACE
// New overlay items are APPENDED at the end
// See /home/dustin/projects/jin/src/merge/deep.rs lines 148-162

// CRITICAL: Empty array behavior
// Empty overlay array REPLACES base array entirely
// This is correct behavior (avoids Lodash bug)
// See /home/dustin/projects/jin/src/merge/deep.rs lines 138-141

// CRITICAL: serde_json::Value for assertions
// Parse JSON with serde_json::from_str()
// Access nested fields with merged["key"]["subkey"]
// Use assert_eq! for value comparisons
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed. Using existing:
- `serde_json::Value` for JSON parsing and assertions
- `TestFixture` from `tests/common/fixtures.rs` for test isolation

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: LOCATE insertion point in tests/conflict_workflow.rs
  - FIND: Line 1151 (end of test_nested_object_deep_merge)
  - INSERT: New test function after line 1151
  - PRESERVE: All existing tests and imports
  - PLACEMENT: tests/conflict_workflow.rs

Task 2: ADD test_array_key_based_merge() function
  - IMPLEMENT: Integration test for array key-based merging
  - FOLLOW pattern: tests/conflict_workflow.rs lines 1052-1151 (test_nested_object_deep_merge)
  - SIGNATURE: fn test_array_key_based_merge() -> Result<(), Box<dyn std::error::Error>>
  - NAMING: snake_case, descriptive test name
  - PLACEMENT: tests/conflict_workflow.rs (after line 1151)

Task 3: IMPLEMENT test setup
  - CREATE: TestFixture using setup_test_repo()
  - EXTRACT: jin_dir from fixture.jin_dir.clone().unwrap()
  - CREATE: Unique mode name using format!("test_mode_{}", unique_test_id())
  - FOLLOW pattern: Lines 1053-1069 from test_nested_object_deep_merge

Task 4: IMPLEMENT Layer 2 (ModeBase) setup
  - CREATE: config.json file path using fixture.path().join("config.json")
  - WRITE: JSON content with array: `[{"id": 1, "name": "task1", "status": "pending"}]`
  - ADD: File to ModeBase layer using jin_cmd().args(["add", "config.json", "--mode"])
  - COMMIT: Change with jin_cmd().args(["commit", "-m", "Add base array"])
  - FOLLOW pattern: Lines 1071-1090 from test_nested_object_deep_merge

Task 5: IMPLEMENT Layer 7 (ProjectBase) setup
  - OVERWRITE: config.json with new array: `[{"id": 1, "priority": "high"}, {"id": 2, "name": "task2"}]`
  - ADD: File to ProjectBase layer using jin_cmd().args(["add", "config.json"])
  - COMMIT: Change with jin_cmd().args(["commit", "-m", "Add overlay array"])
  - GOTCHA: No "--mode" flag for ProjectBase (it's the default layer)
  - FOLLOW pattern: Lines 1092-1111 from test_nested_object_deep_merge

Task 6: IMPLEMENT conflict verification
  - CHECK: No .jinmerge file created at config_path.with_extension(".jinmerge")
  - ASSERT: !merge_conflict_path.exists() with descriptive message
  - FOLLOW pattern: Lines 1113-1118 from test_nested_object_deep_merge

Task 7: IMPLEMENT merge execution
  - RUN: jin_cmd().arg("apply") to execute deep merge
  - VERIFY: Command succeeds with .assert().success()
  - FOLLOW pattern: Lines 1120-1126 from test_nested_object_deep_merge

Task 8: IMPLEMENT result verification
  - READ: Merged content using fs::read_to_string(&config_path)
  - PARSE: JSON using serde_json::from_str(&merged_content)?
  - ASSERT: Array length is 2 (both items present)
  - ASSERT: Item with id=1 has all fields (name, status, priority)
  - ASSERT: Item with id=2 exists with name="task2"
  - ASSERT: Layer 7 "priority" field is present (override works)
  - FOLLOW pattern: Lines 1128-1148 from test_nested_object_deep_merge
```

### Implementation Patterns & Key Details

```rust
// EXACT TEST STRUCTURE - Follow this pattern from test_nested_object_deep_merge

/// Test array key-based merging across layers
///
/// Scenario:
/// - Layer 2 (ModeBase): [{"id": 1, "name": "task1", "status": "pending"}]
/// - Layer 7 (ProjectBase): [{"id": 1, "priority": "high"}, {"id": 2, "name": "task2"}]
///
/// Expected result after deep merge:
/// - Item id=1: Merged with all fields {"id": 1, "name": "task1", "status": "pending", "priority": "high"}
/// - Item id=2: Appended from overlay {"id": 2, "name": "task2"}
/// - Total items: 2 (no duplicates)
///
/// This test verifies that the deep merge engine correctly handles arrays
/// with key-based matching (id/name fields) following RFC 7396 semantics
/// with Jin's layer precedence.
#[test]
fn test_array_key_based_merge() -> Result<(), Box<dyn std::error::Error>> {
    // PATTERN: Test setup with isolation (lines 1053-1055)
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // PATTERN: Create and activate mode (lines 1056-1069)
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

    // PATTERN: Layer 2 setup - ModeBase (lines 1071-1090)
    // GOTCHA: Use --mode flag for ModeBase layer
    let config_path = fixture.path().join("config.json");
    fs::write(
        &config_path,
        r#"[{"id": 1, "name": "task1", "status": "pending"}]"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add base array"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Layer 7 setup - ProjectBase (lines 1092-1111)
    // GOTCHA: No flag needed for ProjectBase (default layer)
    fs::write(
        &config_path,
        r#"[{"id": 1, "priority": "high"}, {"id": 2, "name": "task2"}]"#,
    )?;

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add overlay array"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Verify no conflict file (lines 1113-1118)
    let merge_conflict_path = config_path.with_extension(".jinmerge");
    assert!(
        !merge_conflict_path.exists(),
        "No conflict file should be created for mergeable arrays"
    );

    // PATTERN: Apply merge (lines 1120-1126)
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // PATTERN: Verify merged result (lines 1128-1148)
    let merged_content = fs::read_to_string(&config_path)?;
    let merged: serde_json::Value = serde_json::from_str(&merged_content)?;

    // CRITICAL ASSERTIONS for array merging

    // 1. Array length check (should be 2, not 3)
    let merged_array = merged.as_array()
        .expect("Merged value should be an array");
    assert_eq!(
        merged_array.len(), 2,
        "Array should have 2 items after merge (id=1 merged, id=2 appended)"
    );

    // 2. Find item with id=1 and verify all fields present
    let item_1 = merged_array.iter()
        .find(|v| v.get("id").and_then(|id| id.as_i64()) == Some(1))
        .expect("Item with id=1 should exist");

    assert_eq!(
        item_1.get("name").and_then(|n| n.as_str()), Some("task1"),
        "name should be preserved from ModeBase (Layer 2)"
    );
    assert_eq!(
        item_1.get("status").and_then(|s| s.as_str()), Some("pending"),
        "status should be preserved from ModeBase (Layer 2)"
    );
    assert_eq!(
        item_1.get("priority").and_then(|p| p.as_str()), Some("high"),
        "priority should be added from ProjectBase (Layer 7)"
    );

    // 3. Find item with id=2 and verify it was appended
    let item_2 = merged_array.iter()
        .find(|v| v.get("id").and_then(|id| id.as_i64()) == Some(2))
        .expect("Item with id=2 should exist (appended from overlay)");

    assert_eq!(
        item_2.get("name").and_then(|n| n.as_str()), Some("task2"),
        "name should be present for id=2"
    );

    // 4. Verify order: id=1 before id=2 (base order preserved)
    let ids: Vec<_> = merged_array.iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_i64()))
        .collect();
    assert_eq!(
        ids, vec![1, 2],
        "Order should be preserved: base items first, new items appended"
    );

    Ok(())
}
```

### Integration Points

```yaml
NO DATABASE CHANGES:
  - This is a test-only change
  - No migrations needed

NO CONFIG CHANGES:
  - No configuration modifications

NO ROUTE CHANGES:
  - No API changes

TEST ADDITION:
  - add to: tests/conflict_workflow.rs
  - location: After line 1151 (end of test_nested_object_deep_merge)
  - pattern: "Follow existing integration test structure"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding the test function
cargo check --tests                      # Check syntax without running tests
cargo clippy --tests                     # Lint check for test code

# Expected: No errors. If errors exist, READ output and fix syntax errors.
# Common issues: missing imports, incorrect macro usage, type mismatches
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the specific test function
cargo test test_array_key_based_merge -- --exact

# Run all conflict_workflow tests to ensure no regression
cargo test --test conflict_workflow

# Run with verbose output to see assertion details
cargo test test_array_key_based_merge -- --exact --nocapture

# Expected: Test passes. If failing, debug by:
# 1. Adding eprintln!() statements to inspect values
# 2. Checking that layers were set up correctly
# 3. Verifying JSON parsing is working
```

### Level 3: Integration Testing (System Validation)

```bash
# Ensure no regression in other merge tests
cargo test test_nested_object_deep_merge -- --exact
cargo test test_structured_file_auto_merge -- --exact

# Run full test suite for merge-related functionality
cargo test merge

# Expected: All merge tests pass. If other tests fail, check for:
# 1. Accidental modifications to existing code
# 2. Test isolation issues (JIN_DIR not set correctly)
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Manual verification of the merge behavior
# Create a temporary test scenario to verify array merging works as expected

# Test scenario: Create a local test repository
mkdir /tmp/jin_array_test && cd /tmp/jin_array_test
git init
jin init

# Create a mode
jin mode create test_mode
jin mode use test_mode

# Add array to ModeBase layer
echo '[{"id": 1, "name": "task1", "status": "pending"}]' > config.json
jin add config.json --mode
jin commit -m "Add base array"

# Add overlay array to ProjectBase layer
echo '[{"id": 1, "priority": "high"}, {"id": 2, "name": "task2"}]' > config.json
jin add config.json
jin commit -m "Add overlay array"

# Apply and verify
jin apply
cat config.json | jq .

# Expected output:
# [
#   {
#     "id": 1,
#     "name": "task1",
#     "status": "pending",
#     "priority": "high"
#   },
#   {
#     "id": 2,
#     "name": "task2"
#   }
# ]

# Verify no conflict file was created
ls config.json.jinmerge 2>/dev/null && echo "ERROR: Conflict file created" || echo "OK: No conflict file"

# Expected: OK: No conflict file
```

## Final Validation Checklist

### Technical Validation

- [ ] Test compiles without errors: `cargo check --tests`
- [ ] No linting warnings: `cargo clippy --tests`
- [ ] Test passes: `cargo test test_array_key_based_merge -- --exact`
- [ ] No regressions: `cargo test --test conflict_workflow`
- [ ] Test follows existing patterns (compare with test_nested_object_deep_merge)

### Feature Validation

- [ ] Array length is 2 (id=1 merged, id=2 appended, no duplicates)
- [ ] Item with id=1 has all fields from both layers
- [ ] Item with id=2 is present (appended from overlay)
- [ ] Order is preserved (id=1 before id=2)
- [ ] No conflict file created for mergeable arrays
- [ ] Layer precedence works (overlay values override base)

### Code Quality Validation

- [ ] Test function name follows snake_case convention
- [ ] Test documentation explains the scenario clearly
- [ ] Assertions have descriptive messages
- [ ] Test is isolated (uses unique_test_id for mode name)
- [ ] JIN_DIR environment variable is set correctly
- [ ] No code duplication (follows existing test patterns)

### Documentation & Deployment

- [ ] Test is self-documenting with clear scenario description
- [ ] Comments explain what each assertion verifies
- [ ] Test can be understood by someone unfamiliar with the codebase

---

## Anti-Patterns to Avoid

- ❌ Don't forget to set `JIN_DIR` environment variable (causes test pollution)
- ❌ Don't use integer literals for id fields without proper type handling (use i64 for serde_json)
- ❌ Don't assume array order without explicitly checking (order matters in this test)
- ❌ Don't skip the no-conflict-file check (critical for merge verification)
- ❌ Don't forget to use `--mode` flag for ModeBase layer (critical for layer setup)
- ❌ Don't drop the TestFixture prematurely (causes directory cleanup before test completes)
- ❌ Don't use `.get("key").unwrap()` without checking if key exists (causes panics)
- ❌ Don't modify existing tests (only add new test function)
- ❌ Don't hardcode mode names (use `unique_test_id()` for isolation)
- ❌ Don't skip running the full conflict_workflow test suite after changes

---

## Context Summary for Implementation

**What you're building**: An integration test that verifies array key-based merging works correctly.

**Where to add it**: `/home/dustin/projects/jin/tests/conflict_workflow.rs` after line 1151

**What to follow**: Use `test_nested_object_deep_merge` (lines 1036-1151) as the exact template

**Key differences from the template**:
- Test array merging instead of nested object merging
- Use array-specific assertions (length, finding items by id, order verification)
- JSON content uses arrays instead of nested objects

**Expected test commands**:
1. `cargo test test_array_key_based_merge -- --exact` - Run the new test
2. `cargo test --test conflict_workflow` - Run all conflict workflow tests
3. `cargo test merge` - Run all merge-related tests

**Confidence Score**: 9/10 for one-pass implementation success
