# PRP: P1.M4.T3.S1 - Create test for reset --hard --force in detached state

---

## Goal

**Feature Goal**: Create a comprehensive integration test that validates the `jin reset --hard --force` command successfully bypasses workspace attachment validation and recovers from a detached state, while `jin reset --hard` (without `--force`) correctly fails with a detached workspace error.

**Deliverable**: A new test function `test_reset_hard_force_in_detached_state` added to `tests/cli_reset.rs` that exercises the complete workflow of creating a detached state, verifying error without `--force`, and verifying success with `--force`.

**Success Definition**:
- Test creates a valid Jin workflow (init, add, commit, apply) to establish workspace metadata
- Test manually modifies a workspace file to create a detached state
- Test verifies `jin reset --hard` fails with `JinError::DetachedWorkspace` error
- Test verifies `jin reset --hard --force` succeeds and removes the modified file
- Test verifies workspace state after force reset
- Test follows existing test patterns in `tests/cli_reset.rs`
- All existing tests continue to pass
- Test is isolated using `TempDir` and unique test identifiers

---

## User Persona

**Target User**: Quality assurance engineers and developers who need to verify that the `--force` flag provides a reliable recovery path from detached workspace states.

**Use Case**: After implementing P1.M4.T1.S1 (which adds the `--force` bypass logic), we need test coverage to ensure the recovery path works correctly and doesn't regress in future changes.

**User Journey**:
1. Developer reads the test to understand the expected behavior of `--force` in detached states
2. Developer runs the test to verify the implementation works
3. Test serves as documentation of the expected behavior
4. Future changes that break the behavior will be caught by the test

**Pain Points Addressed**:
- **Missing test coverage**: P1.M4.T1.S1 implements the `--force` bypass, but there's no test for it
- **Behavior ambiguity**: Without a test, the exact expected behavior of `--force` in detached states is unclear
- **Regression risk**: Future changes could inadvertently break the `--force` bypass without detection

---

## Why

- **Problem**: P1.M4.T1.S1 implements the logic change that makes `--force` skip workspace attachment validation. P1.M4.T2.S1 updates the help text. However, there is no automated test verifying this behavior works correctly.

- **User Impact**: Without test coverage, the `--force` recovery path could break in future changes, leaving users without a way to recover from detached states.

- **Integration**: This test completes P1.M4.T3 by providing verification that the implementation from P1.M4.T1.S1 works as specified. It documents the expected behavior and prevents regressions.

- **Code Quality**: Test coverage is critical for ensuring reliability. The destructive_validation.rs tests verify that validation happens (for `--hard` without `--force`), but there's no complementary test for the `--force` bypass path.

---

## What

### User-Visible Behavior

The test should verify the following behavior:

**Scenario 1: Without `--force` (should fail)**
```bash
# After creating detached state
$ jin reset --hard
Error: Workspace is in a detached state
Details: Workspace files have been modified outside of Jin operations
Recovery hint: Run 'jin apply' to restore from active context
```

**Scenario 2: With `--force` (should succeed)**
```bash
# After creating detached state
$ jin reset --hard --force
Discarded 1 file(s) from staging and workspace
# Files are successfully deleted
```

### Technical Requirements

1. **Add test to `tests/cli_reset.rs`**:
   - Create function `test_reset_hard_force_in_detached_state()`
   - Use `TempDir` for isolation (same as other reset tests)
   - Set up complete workflow: init → add file → commit → apply
   - Manually modify a workspace file to create detached state
   - Verify `jin reset --hard` fails (using CLI, not direct function call)
   - Verify `jin reset --hard --force` succeeds
   - Verify file is deleted after force reset

2. **Follow existing test patterns**:
   - Use `jin()` command builder from cli_reset.rs
   - Use `predicate::str::contains()` for assertions
   - Use `TempDir` for test isolation
   - Use unique identifiers (e.g., `std::process::id()`) to avoid conflicts

3. **Test structure**:
   ```rust
   #[test]
   fn test_reset_hard_force_in_detached_state() {
       // Setup: init, add file, commit, apply
       // Create detached state by modifying workspace file
       // Verify: reset --hard fails
       // Verify: reset --hard --force succeeds
       // Verify: file is deleted
   }
   ```

4. **No new dependencies**: Use existing test utilities

### Success Criteria

- [ ] Test creates a valid Jin workflow (init, add, commit, apply)
- [ ] Test creates a detached state by manually modifying a workspace file
- [ ] `jin reset --hard` fails with "detached" or "modified" error message
- [ ] `jin reset --hard --force` succeeds with "Discarded" output
- [ ] File is removed from workspace after force reset
- [ ] Test follows patterns from existing tests in cli_reset.rs
- [ ] `cargo test test_reset_hard_force_in_detached_state` passes
- [ ] All other tests continue to pass

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including exact code patterns to follow, the full test workflow, the error messages to expect, the file structure of the codebase, the existing test utilities available, specific line references for related code, and external research references._

### Documentation & References

```yaml
# CONTRACT FROM P1.M4.T1.S1 - Behavior to test
- file: /home/dustin/projects/jin/plan/P1M4T1S1/PRP.md
  why: P1.M4.T1.S1 implements the logic that --force should skip validation
  section: "Goal Section - Feature Goal" and "What - User-Visible Behavior"
  critical: |
    P1.M4.T1.S1 modifies reset.rs so that --force skips workspace validation.
    The key change: validate_workspace_attached() is only called when !args.force
    This test verifies that behavior works correctly in practice.
    Expected behavior matrix:
      - reset --hard: Validates, fails if detached
      - reset --hard --force: Skips validation, succeeds even if detached

# CONTRACT FROM P1.M4.T2.S1 - Help text context
- file: /home/dustin/projects/jin/plan/P1M4T2S1/PRP.md
  why: P1.M4.T2.S1 updates help text to reflect the --force behavior
  section: "Goal Section - Feature Goal"
  critical: |
    The help text now says "Skip confirmation prompt and bypass detached state validation (use for recovery)"
    This confirms the expected behavior we need to test.

# IMPLEMENTATION: reset.rs validation logic (what we're testing)
- file: /home/dustin/projects/jin/src/commands/reset.rs
  why: This is the code being tested - shows the exact logic flow
  section: "Lines 58-68: Validation skip with --force"
  code: |
    if mode == ResetMode::Hard {
        if !args.force {
            let repo = JinRepo::open()?;
            validate_workspace_attached(&context, &repo)?;
        }
        // If --force, skip validation and proceed to load staging
    }
  critical: |
    This is the key code: when --force is set, validation is skipped.
    The test verifies this behavior by:
    1. Creating a detached state
    2. Calling reset --hard (without --force) -> should fail validation
    3. Calling reset --hard --force -> should skip validation and succeed

# PATTERN: Existing reset test structure in cli_reset.rs
- file: /home/dustin/projects/jin/tests/cli_reset.rs
  why: Shows the exact pattern to follow for reset tests
  section: "Lines 125-159: test_reset_hard_mode_with_force()"
  pattern: |
    #[test]
    fn test_reset_hard_mode_with_force() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path();
        let jin_dir = temp.path().join(".jin_global");

        // Initialize
        jin()
            .arg("init")
            .current_dir(project_path)
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();

        // Create and stage a file
        fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
        jin()
            .args(["add", "config.json"])
            .current_dir(project_path)
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();

        // Reset hard mode with force (should skip confirmation)
        jin()
            .args(["reset", "--hard", "--force"])
            .current_dir(project_path)
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success()
            .stdout(predicate::str::contains("Discarded"));

        // Verify file removed from workspace
        assert!(!project_path.join("config.json").exists());
    }
  note: |
    This test shows the standard pattern but doesn't test detached state.
    Our new test extends this pattern to include detached state verification.

# PATTERN: Test utilities in cli_reset.rs
- file: /home/dustin/projects/jin/tests/cli_reset.rs
  why: Shows the helper functions and imports used
  section: "Lines 1-11: Imports and jin() helper"
  code: |
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    fn jin() -> Command {
        Command::new(env!("CARGO_BIN_EXE_jin"))
    }
  critical: |
    These are the imports and helper to use in the new test.
    - assert_cmd::Command for CLI testing
    - predicates::prelude::* for assertions
    - std::fs for file manipulation
    - tempfile::TempDir for test isolation

# PATTERN: How to create a mode and use it (for workflow setup)
- file: /home/dustin/projects/jin/tests/cli_reset.rs
  why: Shows how to create and activate a mode for the test workflow
  section: "Lines 175-188: Mode creation and activation"
  code: |
    let mode_name = format!("test_mode_{}", std::process::id());
    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
  note: |
    Using std::process::id() ensures unique test names to avoid conflicts.

# REFERENCE: Destructive validation tests (for context)
- file: /home/dustin/projects/jin/tests/destructive_validation.rs
  why: Shows how detached states are created and tested
  section: "Lines 79-128: test_reset_hard_rejected_when_files_modified()"
  pattern: |
    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally to create detached state
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt reset --hard, should be rejected
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        // ...
        force: true, // Skip confirmation for test
    });
  gotcha: |
    These tests call execute() directly with force: true but still expect validation to fail.
    This is BEFORE P1.M4.T1.S1 fix. After P1.M4.T1.S1, force: true should skip validation.
    Our test uses the CLI (jin() command) to test the actual user-facing behavior.

# REFERENCE: Workspace metadata structure (for understanding)
- file: /home/dustin/projects/jin/src/staging/metadata.rs
  why: Shows what WorkspaceMetadata contains (for understanding how to create detached state)
  section: "Lines 17-25: WorkspaceMetadata struct"
  code: |
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkspaceMetadata {
        pub timestamp: String,
        pub applied_layers: Vec<String>,
        pub files: HashMap<PathBuf, String>,  // file -> hash
    }
  note: |
    When we modify a tracked file, the hash in metadata no longer matches,
    creating a detached state. The test leverages this for simplicity.

# VALIDATION: Error message to expect
- file: /home/dustin/projects/jin/src/core/error.rs
  why: Shows the DetachedWorkspace error structure
  section: "JinError::DetachedWorkspace variant"
  pattern: |
    DetachedWorkspace {
        workspace_commit: Option<String>,
        expected_layer_ref: String,
        details: String,  // Contains "modified" or "Workspace files"
        recovery_hint: String,
    }
  note: |
    Test should check stderr for "detached" or "modified" to verify error.

# VALIDATION: Success message to expect
- file: /home/dustin/projects/jin/src/commands/reset.rs
  why: Shows the success message printed by reset --hard
  section: "Line 119: Success message"
  code: |
    println!("Discarded {} file(s) from staging and workspace", count);
  note: |
    Test should check stdout for "Discarded" to verify success.

# WORKSPACE: How workspace metadata is stored
- file: /home/dustin/projects/jin/src/staging/metadata.rs
  why: Shows where workspace metadata is saved
  section: "WorkspaceMetadata::load() and save() methods"
  path: ".jin/workspace/last_applied.json"
  note: |
    The test doesn't need to read this file directly - modifying a tracked file
    automatically creates the detached state when validation runs.

# EXTERNAL RESEARCH: assert_cmd documentation
- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: Official assert_cmd documentation for CLI testing patterns
  section: "Command builder and assertion methods"
  critical: |
    - Command::new(env!("CARGO_BIN_EXE_jin")) - get binary path
    - .args(["reset", "--hard", "--force"]) - pass multiple arguments
    - .current_dir(path) - set working directory
    - .env("JIN_DIR", path) - set environment variable
    - .assert().success() - expect success
    - .assert().failure() - expect failure
    - .stdout(predicate::str::contains("text")) - check stdout
    - .stderr(predicate::str::contains("text")) - check stderr

# EXTERNAL RESEARCH: predicates crate documentation
- url: https://docs.rs/predicates/latest/predicates/
  why: Predicates for string matching in assertions
  section: "predicate::str::contains()"
  critical: |
    predicate::str::contains("text") creates a predicate that checks if
    the output contains the specified substring.

# EXTERNAL RESEARCH: tempfile crate documentation
- url: https://docs.rs/tempfile/latest/tempfile/
  why: Temporary directory management for tests
  section: "TempDir::new()"
  critical: |
    TempDir::new() creates a temporary directory that is automatically
    deleted when the TempDir goes out of scope.

# EXTERNAL RESEARCH: Rust testing best practices
- url: https://rust-cli.github.io/book/tutorial/testing.html
  why: Rust CLI Book - testing chapter
  section: "Integration testing with assert_cmd"
  critical: |
    - Use TempDir for test isolation
    - Use unique identifiers for parallel-safe tests
    - Clean up happens automatically when TempDir is dropped
    - Use predicates for flexible output matching

# CONTEXT: Current codebase tree
- file: /home/dustin/projects/jin/tests/cli_reset.rs
  why: This is the file to modify - add the new test here
  location: "Add new test after line 307 (after test_reset_help)"
  note: |
    The test should be added to the existing cli_reset.rs file,
    following the same structure and patterns as existing tests.

# CONTEXT: Related test file (destructive_validation.rs)
- file: /home/dustin/projects/jin/tests/destructive_validation.rs
  why: Shows detached state testing patterns
  note: |
    This file uses direct function calls and TestFixture.
    Our test uses CLI commands (jin() builder) for more realistic testing.
    The patterns here are useful for understanding but we follow cli_reset.rs style.

# GOTCHA: Important note about destructive_validation.rs tests
- file: /home/dustin/projects/jin/tests/destructive_validation.rs
  section: "Lines 101-110: test_reset_hard_rejected_when_files_modified"
  warning: |
    These tests use force: true and expect validation to FAIL.
    This is the OLD behavior BEFORE P1.M4.T1.S1.
    After P1.M4.T1.S1, force: true should SKIP validation.
    Those tests may need updating after P1.M4.T1.S1 is implemented.
    Our test focuses on CLI-level testing of the NEW behavior.

# WORKFLOW: Complete Jin workflow for test setup
- steps:
  1. jin init - Initialize Jin in project
  2. Create and modify a file
  3. jin add <file> - Stage the file
  4. jin commit -m "message" - Commit staged changes
  5. jin apply - Apply committed changes to workspace
  6. Manually modify the file to create detached state
  7. jin reset --hard - Should fail (detached state)
  8. jin reset --hard --force - Should succeed (bypass validation)
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── commands/
│   │   └── reset.rs                   # CODE BEING TESTED (lines 58-68: validation skip)
│   ├── staging/
│   │   ├── workspace.rs              # validate_workspace_attached() function
│   │   └── metadata.rs               # WorkspaceMetadata structure
│   └── core/
│       └── error.rs                  # JinError::DetachedWorkspace variant
├── tests/
│   ├── cli_reset.rs                  # FILE TO MODIFY: Add new test here
│   │   ├── test_reset_hard_mode_with_force()  # Similar pattern (lines 126-159)
│   │   └── test_reset_help()         # Add new test after this (line 307)
│   └── destructive_validation.rs     # Detached state patterns (for reference)
└── plan/
    ├── P1M4T1S1/PRP.md               # CONTRACT: Logic implementation to test
    └── P1M4T2S1/PRP.md               # CONTRACT: Help text update
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── tests/
    └── cli_reset.rs                  # MODIFIED: New test added
        # Added after line 307:
        #
        # #[test]
        # fn test_reset_hard_force_in_detached_state() {
        #     // Test implementation...
        # }
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: The test must use jin() CLI command builder, NOT direct function calls
// This tests the actual user-facing behavior, not just the implementation.
// See cli_reset.rs patterns vs destructive_validation.rs patterns.

// GOTCHA: Creating a detached state requires a COMPLETE workflow
// Just doing "jin init" and "jin add" is NOT enough.
// Must do: init → add → commit → apply → modify file → detached state
// This ensures workspace metadata exists with file hashes.

// GOTCHA: File modification must change CONTENT, not just metadata
// Modifying file content changes the Git blob hash.
// The validation detects mismatched hashes in metadata.
// fs::write(file_path, "different content") creates detached state.

// GOTCHA: Test must use unique identifiers for mode names
// Using format!("test_mode_{}", std::process::id()) ensures uniqueness.
// This allows tests to run in parallel without conflicts.

// GOTCHA: The test should NOT use #[serial] attribute
// Unlike destructive_validation.rs tests, cli_reset.rs tests don't set
// std::env::set_current_dir, so they can run in parallel.
// TempDir provides sufficient isolation.

// GOTCHA: Error message checking
// The DetachedWorkspace error may say "detached", "modified", or "Workspace files".
// Use flexible checking: predicate::str::contains("detached").or("modified")
// Or check for any of the possible error indicators.

// GOTCHA: Success message checking
// The success message is "Discarded N file(s) from staging and workspace".
// Check for "Discarded" to confirm success.
// The file count may vary, so don't check for exact message.

// PATTERN: Using jin() command builder
// Correct:
//   jin()
//       .args(["reset", "--hard", "--force"])
//       .current_dir(project_path)
//       .env("JIN_DIR", &jin_dir)
//       .assert()
//       .success();
//
// Incorrect:
//   jin::commands::reset::execute(jin::cli::ResetArgs { ... });
//   (This is used in destructive_validation.rs, not cli_reset.rs)

// PATTERN: File existence verification
// Correct:
//   assert!(!project_path.join("config.json").exists());
//
// Incorrect:
//   assert_eq!(project_path.join("config.json").exists(), false);
//   (Less idiomatic)

// GOTCHA: Workflow setup is critical
// The test MUST create a proper Jin workflow:
// 1. jin init
// 2. echo '{"test": true}' > config.json
// 3. jin add config.json
// 4. jin commit -m "Add config"
// 5. jin apply
// 6. echo '{"modified": true}' > config.json  # Creates detached state
// 7. jin reset --hard  # Should fail
// 8. jin reset --hard --force  # Should succeed
//
// If any step is missing, the test may not work as expected.

// CRITICAL: Understanding the contract from P1.M4.T1.S1
// The fix changes validation from ALWAYS validating (for Hard mode) to:
//   - Validating ONLY when !args.force
//   - Skipping validation when args.force is true
//
// Our test verifies:
// 1. Without --force: validation runs, detects detached state, returns error
// 2. With --force: validation skipped, operation proceeds, files deleted

// GOTCHA: Relationship with destructive_validation.rs tests
// The tests in destructive_validation.rs use force: true and expect errors.
// This is because they were written BEFORE P1.M4.T1.S1.
// After P1.M4.T1.S1, those tests might fail (force: true skips validation).
// Our test focuses on the NEW, correct behavior via CLI.

// PATTERN: Test isolation using TempDir
// let temp = TempDir::new().unwrap();
// let project_path = temp.path();
// let jin_dir = temp.path().join(".jin_global");
//
// TempDir is automatically cleaned up when dropped.
// No manual cleanup needed.

// CRITICAL: No new dependencies needed
// Use existing imports from cli_reset.rs:
// - use assert_cmd::Command;
// - use predicates::prelude::*;
// - use std::fs;
// - use tempfile::TempDir;
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This test uses existing structures:
- `ResetArgs` from `src/cli/args.rs`
- `JinError::DetachedWorkspace` from `src/core/error.rs`
- `WorkspaceMetadata` from `src/staging/metadata.rs`

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD NEW TEST FUNCTION TO tests/cli_reset.rs
  - FILE: tests/cli_reset.rs
  - LOCATION: After line 307 (after test_reset_help function)
  - FUNCTION NAME: test_reset_hard_force_in_detached_state
  - STRUCTURE:
    1. Setup TempDir and paths
    2. Initialize Jin (jin init)
    3. Create file and stage it (jin add)
    4. Commit changes (jin commit)
    5. Apply to workspace (jin apply)
    6. Modify file manually to create detached state
    7. Test reset --hard fails (assert failure, check stderr)
    8. Test reset --hard --force succeeds (assert success, check stdout)
    9. Verify file is deleted
  - DEPENDENCIES: None (new test, independent of other tests)

Task 2: VERIFY TEST COMPILES
  - RUN: cargo test --no-run
  - EXPECTED: Test compiles without errors
  - DEPENDENCIES: Task 1

Task 3: RUN NEW TEST
  - RUN: cargo test test_reset_hard_force_in_detached_state
  - EXPECTED: Test passes
  - DEPENDENCIES: Task 1

Task 4: RUN ALL RESET TESTS
  - RUN: cargo test reset
  - EXPECTED: All reset tests pass
  - DEPENDENCIES: Task 1

Task 5: RUN FULL TEST SUITE
  - RUN: cargo test
  - EXPECTED: All tests pass (no regressions)
  - DEPENDENCIES: Task 1
```

### Implementation Patterns & Key Details

```rust
// ================== COMPLETE TEST IMPLEMENTATION ==================
//
// FILE: tests/cli_reset.rs
// LOCATION: Add after line 307 (after test_reset_help function)
//
// This test follows the exact pattern from test_reset_hard_mode_with_force
// but adds detached state verification.

// --- TEST CODE TO ADD ---

#[test]
fn test_reset_hard_force_in_detached_state() {
    // ================== SETUP ==================
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // ================== STEP 1: INITIALIZE ==================
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 2: CREATE AND STAGE FILE ==================
    fs::write(
        project_path.join("config.json"),
        r#"{"original": true}"#
    ).unwrap();

    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 3: COMMIT CHANGES ==================
    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 4: APPLY TO WORKSPACE ==================
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 5: CREATE DETACHED STATE ==================
    // Modify the file externally - this creates detached state
    // because the file hash no longer matches metadata
    fs::write(
        project_path.join("config.json"),
        r#"{"modified": true}"#
    ).unwrap();

    // ================== STEP 6: VERIFY reset --hard FAILS ==================
    // Without --force, should fail with detached state error
    jin()
        .args(["reset", "--hard"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("detached")
            .or(predicate::str::contains("modified"))
            .or(predicate::str::contains("Workspace files"))
        );

    // ================== STEP 7: VERIFY reset --hard --force SUCCEEDS ==================
    // With --force, should skip validation and succeed
    jin()
        .args(["reset", "--hard", "--force"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Discarded"));

    // ================== STEP 8: VERIFY FILE IS DELETED ==================
    // After hard reset, the file should be removed from workspace
    assert!(!project_path.join("config.json").exists());
}

// ================== TEST BREAKDOWN ==================
//
// The test has 8 clear steps:
//
// STEP 1: Initialize Jin repository
//   - Creates .jin directory structure
//   - Sets up initial context
//
// STEP 2: Create and stage a file
//   - Creates config.json with original content
//   - Stages file with jin add
//
// STEP 3: Commit staged changes
//   - Creates a Jin commit
//   - Stores file hash in commit
//
// STEP 4: Apply to workspace
//   - Creates WorkspaceMetadata with file hashes
//   - Applies files to workspace
//   - This is critical - without apply, there's no metadata to validate
//
// STEP 5: Create detached state
//   - Modify file content externally
//   - New content = new hash
//   - Metadata still has old hash = mismatch = detached state
//
// STEP 6: Verify error without --force
//   - reset --hard validates workspace
//   - Detects hash mismatch
//   - Returns DetachedWorkspace error
//   - Test checks stderr for error indicators
//
// STEP 7: Verify success with --force
//   - reset --hard --force skips validation
//   - Proceeds to delete staged files
//   - Returns success with "Discarded" message
//   - Test checks stdout for success indicator
//
// STEP 8: Verify file deletion
//   - Hard reset removes files from workspace
//   - File should not exist after reset
//
// ================== WHY THIS TEST WORKS ==================
//
// The test leverages the workspace validation mechanism:
//
// 1. After "jin apply", workspace metadata contains:
//    { "config.json": "abc123hash" }
//
// 2. After manual modification, file has:
//    { "config.json": "def456hash" }  (different hash)
//
// 3. Validation detects mismatch:
//    detect_file_mismatch() returns modified files
//    validate_workspace_attached() returns DetachedWorkspace error
//
// 4. With --force:
//    Validation is skipped (see reset.rs lines 62-68)
//    Files are deleted directly
//    Success message is printed
//
// ================== ASSERTION STRATEGY ==================
//
// For the failure case (reset --hard):
//   .assert().failure() - expects non-zero exit code
//   .stderr(predicate::str::contains("detached")
//       .or(predicate::str::contains("modified"))
//       .or(predicate::str::contains("Workspace files")))
//   - Flexible matching for error message
//   - Error message may vary based on validation details
//
// For the success case (reset --hard --force):
//   .assert().success() - expects zero exit code
//   .stdout(predicate::str::contains("Discarded"))
//   - Checks for success message
//   - Message format: "Discarded N file(s) from staging and workspace"
//
// For file verification:
//   assert!(!project_path.join("config.json").exists());
//   - Verifies file was deleted
//   - Hard reset removes files from workspace

// ================== COMPARISON TO SIMILAR TESTS ==================
//
// test_reset_hard_mode_with_force (lines 126-159):
//   - Similar structure (TempDir, jin init, add file, reset)
//   - Does NOT create detached state
//   - Does NOT test error case
//   - Only tests success case
//
// Our test:
//   - Same basic structure
//   - ADDS: commit and apply steps (to create metadata)
//   - ADDS: manual file modification (to create detached state)
//   - ADDS: error verification (reset --hard fails)
//   - ADDS: success verification (reset --hard --force succeeds)

// ================== KEY DIFFERENCES FROM DESTRUCTIVE_VALIDATION TESTS ==================
//
// destructive_validation.rs tests:
//   - Use TestFixture from common/fixtures.rs
//   - Use #[serial] attribute (modify environment)
//   - Call execute() directly with ResetArgs
//   - Use force: true to skip confirmation BUT expect validation to fail
//   - This is OLD behavior (before P1.M4.T1.S1)
//
// Our test:
//   - Use TempDir directly (simpler)
//   - No #[serial] needed (don't modify environment)
//   - Use jin() command builder (CLI level)
//   - Test both error and success cases
//   - This is NEW behavior (after P1.M4.T1.S1)
```

### Integration Points

```yaml
WORKSPACE_VALIDATION:
  - function: validate_workspace_attached
  - file: src/staging/workspace.rs
  - lines: 325-399
  - behavior: Detects file mismatches and returns DetachedWorkspace error
  - bypassed_by: --force flag (see reset.rs lines 62-68)

RESET_COMMAND:
  - function: execute
  - file: src/commands/reset.rs
  - lines: 58-68
  - behavior: Validates workspace unless --force is set
  - tested_by: This new test

ERROR_TYPE:
  - enum: JinError::DetachedWorkspace
  - file: src/core/error.rs
  - fields: details (contains "modified" or "detached")
  - returned_by: validate_workspace_attached when file mismatch detected

WORKSPACE_METADATA:
  - struct: WorkspaceMetadata
  - file: .jin/workspace/last_applied.json
  - field: files: HashMap<PathBuf, String>  (file -> hash)
  - created_by: jin apply command
  - used_by: validate_workspace_attached

TEST_FILE:
  - file: tests/cli_reset.rs
  - modification: Add new test function after line 307
  - follows: Patterns from test_reset_hard_mode_with_force
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding the test - fix before proceeding
cargo check --tests                    # Type checking - MUST pass
cargo fmt -- --check tests/            # Format check - should pass

# Expected: Zero errors, zero warnings
# If errors exist, READ output and fix before proceeding
```

### Level 2: Test Compilation

```bash
# Verify test compiles
cargo test --no-run test_reset_hard_force_in_detached_state

# Expected: Test compiles successfully
# If compilation fails, check syntax and imports
```

### Level 3: Run New Test

```bash
# Run the new test specifically
cargo test test_reset_hard_force_in_detached_state -- --nocapture

# Expected: Test passes with output showing:
# - "jin init" succeeds
# - "jin add" succeeds
# - "jin commit" succeeds
# - "jin apply" succeeds
# - "jin reset --hard" fails (detached error)
# - "jin reset --hard --force" succeeds
# - File is deleted
```

### Level 4: Run All Reset Tests

```bash
# Run all reset tests to ensure no regressions
cargo test reset -- --nocapture

# Expected: All tests pass
# Key tests to verify:
# - test_reset_hard_mode_with_force
# - test_reset_hard_force_in_detached_state (new)
# - test_reset_help
```

### Level 5: Full Test Suite

```bash
# Run full test suite to ensure no regressions
cargo test

# Expected: All tests pass
# Focus areas: reset, destructive_validation

# Verify tests are repeatable (run multiple times)
cargo test test_reset_hard_force_in_detached_state -- --test-threads=1
```

### Level 6: Manual Verification (Optional)

```bash
# Manual verification (in temporary directory)
cd $(mktemp -d)
export JIN_DIR=$(pwd)/.jin_global

# Run through the test steps manually
git init
jin init

echo '{"original": true}' > config.json
jin add config.json
jin commit -m "Add config"
jin apply

# Modify file to create detached state
echo '{"modified": true}' > config.json

# Should fail
jin reset --hard
# Expected: Error about detached state

# Should succeed
jin reset --hard --force
# Expected: "Discarded 1 file(s) from staging and workspace"

# Verify file deleted
ls config.json
# Expected: No such file or directory

# Cleanup
cd -
rm -rf "$OLDPWD"
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check --tests` completes with 0 errors
- [ ] `cargo fmt -- --check tests/` shows no formatting issues
- [ ] `cargo test test_reset_hard_force_in_detached_state` passes
- [ ] `cargo test reset` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] Test follows exact structure specified in Implementation Blueprint
- [ ] Test creates complete Jin workflow (init, add, commit, apply)
- [ ] Test creates detached state by modifying workspace file
- [ ] `jin reset --hard` fails with detached error message
- [ ] `jin reset --hard --force` succeeds with "Discarded" message
- [ ] File is removed from workspace after force reset
- [ ] Test uses jin() command builder (not direct function calls)

### Code Quality Validation

- [ ] Test follows patterns from test_reset_hard_mode_with_force
- [ ] Test has clear comments explaining each step
- [ ] Test uses TempDir for isolation
- [ ] Test uses unique identifiers (no hardcoded test names)
- [ ] No new dependencies or imports added

### Documentation & Deployment

- [ ] Test serves as documentation of expected --force behavior
- [ ] Test name clearly indicates what it tests
- [ ] Test is readable and maintainable
- [ ] Ready for merge after P1.M4.T1.S1 and P1.M4.T2.S1 are complete

---

## Anti-Patterns to Avoid

- **Don't** skip the commit and apply steps - they're required to create workspace metadata
- **Don't** use direct function calls (execute()) - use jin() command builder for CLI-level testing
- **Don't** add #[serial] attribute - TempDir provides sufficient isolation
- **Don't** check for exact error message - use flexible predicate matching
- **Don't** create mode/scope for this test - simple project workflow is sufficient
- **Don't** modify metadata file directly - modifying tracked file content is simpler
- **Don't** verify exact "Discarded N file(s)" message - just check for "Discarded"
- **Don't** add new imports - use existing imports from cli_reset.rs
- **Don't** place test in wrong file - add to tests/cli_reset.rs, not destructive_validation.rs
- **Don't** forget to verify file deletion - this confirms hard reset worked
- **Don't** test the wrong behavior - reset --hard should FAIL, reset --hard --force should SUCCEED
- **Don't** make the test too complex - keep it focused on the specific behavior

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single file change**: Only adding one test function to tests/cli_reset.rs
- **Complete specification**: Full test implementation provided with detailed comments
- **Established patterns**: Following exact structure from test_reset_hard_mode_with_force
- **Clear dependencies**: No new imports or utilities needed
- **Well-researched**: All test patterns, error messages, and behaviors documented
- **Comprehensive context**: Reset implementation, validation logic, and workspace mechanics all explained

**Zero Risk Factors**:
- Test is additive - no existing code modified
- Uses existing test utilities and patterns
- Isolated from other tests via TempDir
- No new dependencies required
- Test structure is straightforward and well-understood

**Current Status**: Ready for implementation - all context gathered, full test code provided, validation steps defined

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from this PRP creation):
- `plan/P1M4T3S1/research/` - Directory for all research findings
  - Agent research files:
    - afe2faa (reset test patterns analysis)
    - aa144fa (test utilities analysis)
    - abec55c (workspace/detached state mechanics)
    - a1c25e5 (Rust CLI testing patterns)

**Related PRPs** (Contracts to fulfill):
- `plan/P1M4T1S1/PRP.md` - Implements the --force validation skip logic to test
- `plan/P1M4T2S1/PRP.md` - Updates help text for --force flag

**Code Files**:
- `tests/cli_reset.rs` - File to modify (add test after line 307)
- `src/commands/reset.rs` - Code being tested (validation skip logic)
- `src/staging/workspace.rs` - Validation function implementation
- `tests/destructive_validation.rs` - Related test patterns (for reference)

**External Documentation**:
- [assert_cmd Documentation](https://docs.rs/assert_cmd) - CLI testing patterns
- [predicates Documentation](https://docs.rs/predicates) - String matching predicates
- [tempfile Documentation](https://docs.rs/tempfile) - Temporary directory management
- [Rust CLI Book - Testing](https://rust-cli.github.io/book/tutorial/testing.html) - CLI testing best practices
