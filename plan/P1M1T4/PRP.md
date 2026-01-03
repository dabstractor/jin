name: "P1.M1.T4: Update Status Command for Conflict State"
description: |

---

## Goal

**Feature Goal**: Modify the `jin status` command to detect and display when there's an in-progress apply operation with pending conflict resolutions, providing users with clear visibility into the conflict resolution workflow state.

**Deliverable**: Enhanced status command that:
1. Detects the `.jin/.paused_apply.yaml` state file created by the apply command
2. Displays a prominent "Merge conflicts" section showing:
   - Count of conflicted files
   - List of `.jinmerge` files pending resolution
   - Instruction to run `jin resolve <files>`
   - Timestamp when conflicts were detected

**Success Definition**:
- Running `jin status` when a paused apply operation exists shows a dedicated conflict section
- The conflict section appears between workspace state and staged changes sections
- Users see exactly which files have conflicts and how to resolve them
- When no conflicts exist, status displays normally (no changes to existing behavior)

## User Persona

**Target User**: Developer using Jin's layer system who encounters merge conflicts during `jin apply`

**Use Case**: Developer runs `jin apply`, conflicts are detected and .jinmerge files are created. They want to check the current state and see what conflicts need resolution.

**User Journey**:
1. Developer runs `jin apply`
2. Apply detects conflicts and creates .jinmerge files + `.jin/.paused_apply.yaml`
3. Developer runs `jin status` to see what's happening
4. Status shows: "Merge conflicts (N files):" followed by the list of .jinmerge files
5. Developer sees the instruction: "Resolve with: jin resolve <files>"
6. Developer resolves files and runs `jin resolve <file>` for each
7. Developer runs `jin status` again to verify all conflicts resolved

**Pain Points Addressed**:
- **Unclear state**: After apply pauses, users don't know what conflicts exist without checking files manually
- **No guidance**: Users don't know how to proceed with resolution
- **Hidden workflow**: The conflict resolution state is invisible without status integration

## Why

- **Workflow visibility**: The conflict resolution workflow (P1.M1.T1-T3) is complete but invisible to users. Status integration makes it discoverable.
- **Git-parity**: Git's `git status` shows unmerged files prominently. Jin should provide equivalent visibility.
- **User confidence**: Users need to see that their apply operation is paused, not failed, and understand next steps.
- **Integration with existing workflow**: Builds on the completed .jinmerge file format (T1), apply pause (T2), and resolve command (T3).

## What

Modify `jin status` to add a new "Merge conflicts" section when `.jin/.paused_apply.yaml` exists.

### Success Criteria

- [ ] Status detects `.jin/.paused_apply.yaml` file exists
- [ ] Status parses the state and displays conflict count
- [ ] Status lists all conflicted files (from `conflict_files` field)
- [ ] Status shows "Resolve with: jin resolve <files>" instruction
- [ ] Status displays timestamp from paused state
- [ ] Conflict section appears between workspace state and staged changes
- [ ] When no paused state exists, status displays normally
- [ ] Works with both empty and non-empty staging index
- [ ] Works with clean and dirty workspace states

### Expected Output Format

```
Jin status:

  Mode:  claude (active)
  Scope: (none)
  Project: (none)

Workspace state: Clean

Merge conflicts (3 files):
  .claude/config.json.jinmerge
  src/main.rs.jinmerge
  README.md.jinmerge
  Resolve with: jin resolve <files>
  Detected: 2024-01-03T14:30:00Z

Staged changes (0 files):
  (no staged changes)

Layer summary:
  ...
```

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: Yes. This PRP includes:
- Exact file paths and line numbers for all relevant code
- Complete data structures (`PausedApplyState`, `JinMergeConflict`)
- Exact integration point (line 86 in status.rs)
- Test patterns with executable examples
- Validation commands specific to this codebase

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: /home/dustin/projects/jin/src/commands/status.rs
  why: Main implementation file - exact integration point at line 86 (after workspace state, before staged changes)
  pattern: Simple println!-based output with conditional sections
  gotcha: Status uses println! not eprintln! for all output. Follow this pattern for consistency.

- file: /home/dustin/projects/jin/src/commands/apply.rs
  why: Contains PausedApplyState struct (lines 16-79) that defines the state file format
  pattern: Atomic write pattern with temp file + rename
  section: Lines 16-79 for PausedApplyState and PausedLayerConfig structures
  critical: State file is at `.jin/.paused_apply.yaml` with fields: timestamp, layer_config, conflict_files, applied_files, conflict_count

- file: /home/dustin/projects/jin/src/commands/resolve.rs
  why: Shows how to load and use PausedApplyState (lines 32-51)
  pattern: PausedApplyState::exists() and PausedApplyState::load() usage
  gotcha: Always check exists() before load() - load() returns error if file doesn't exist

- file: /home/dustin/projects/jin/src/merge/jinmerge.rs
  why: Contains JinMergeConflict structure and merge_path_for_file() method
  section: Lines 74-80 for JinMergeConflict struct, lines 137+ for merge_path_for_file()
  pattern: Static methods for file operations (exists, load, parse_from_file)
  gotcha: .jinmerge files are named with `.jinmerge` extension (e.g., `config.json.jinmerge`)

- file: /home/dustin/projects/jin/tests/cli_basic.rs
  why: Reference for status command integration test patterns
  pattern: Uses tempfile::TempDir for isolation, assert_cmd for CLI testing, predicates for output validation
  section: Look for tests starting with `test_status_`

- file: /home/dustin/projects/jin/plan/docs/implementation-gaps-analysis.md
  why: Contains task specification for P1.M1.T4 with detailed subtask breakdown
  section: Search for "P1.M1.T4" in the file

- file: /home/dustin/projects/jin/PRD.md
  why: Section 11.3 "Conflict Resolution" specifies the workflow requirements
  section: Search for "Conflict Resolution" or "11.3"
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── status.rs        # PRIMARY FILE - Add conflict section here (line ~86)
│   ├── apply.rs         # Reference: PausedApplyState struct
│   ├── resolve.rs       # Reference: State loading patterns
│   └── mod.rs           # Command dispatcher
├── merge/
│   ├── jinmerge.rs      # Reference: JinMergeConflict, merge_path_for_file()
│   └── mod.rs
├── core/
│   ├── error.rs         # Reference: JinError types
│   └── mod.rs           # ProjectContext import
├── staging/
│   ├── index.rs         # StagingIndex import
│   └── metadata.rs      # WorkspaceMetadata import
└── main.rs

tests/
├── cli_basic.rs         # Add integration test here
├── common/
│   └── mod.rs           # Test utilities
└── core_workflow.rs
```

### Desired Codebase Tree with Changes

```bash
# Modified files:
src/commands/status.rs   # ADD: import PausedApplyState, ADD: conflict section display logic
tests/cli_basic.rs       # ADD: test_status_shows_conflict_state() and related tests

# No new files needed - this is a modification to existing status command
```

### Known Gotchas of Codebase & Library Quirks

```rust
// CRITICAL: Status command uses println! not eprintln!
// All status output goes to stdout, not stderr. Follow this pattern.

// CRITICAL: Integration point is after workspace state section (line 86)
// The conflict section must appear between workspace state and staged changes.
// DO NOT modify the staged changes section logic.

// GOTCHA: PausedApplyState::load() returns Err if file doesn't exist
// Always check PausedApplyState::exists() before calling load()
// Pattern from resolve.rs lines 33-37:
//   if !PausedApplyState::exists() {
//       return Err(...);
//   }
//   let state = PausedApplyState::load()?;

// GOTCHA: .jinmerge files have .jinmerge extension
// Original file: config.json
// Merge file: config.json.jinmerge
// The conflict_files list in PausedApplyState contains original paths (not .jinmerge paths)
// Use JinMergeConflict::merge_path_for_file() to get the .jinmerge path

// GOTCHA: Timestamp formatting
// PausedApplyState.timestamp is chrono::DateTime<Utc>
// Use .to_rfc3339() or .format() for display

// PATTERN: Status sections are separated by blank println!()
// Each section ends with println!() for spacing

// PATTERN: File paths in status use .display()
// println!("  {}", path.display());

// PATTERN: Pluralization in status output
// if total == 1 { "" } else { "s" }
// Follow this for "file(s)" consistency

// TESTING: Use Jin repo clone pattern for setup
// See test_status_with_active_mode in cli_basic.rs for reference
```

## Implementation Blueprint

### Data Models and Structure

No new data structures needed. Use existing `PausedApplyState` from `apply.rs`:

```rust
// From /home/dustin/projects/jin/src/commands/apply.rs:16-29
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedApplyState {
    pub timestamp: DateTime<Utc>,
    pub layer_config: PausedLayerConfig,
    pub conflict_files: Vec<PathBuf>,  // Original paths, NOT .jinmerge paths
    pub applied_files: Vec<PathBuf>,
    pub conflict_count: usize,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/commands/status.rs - Add imports
  - ADD: use crate::commands::apply::PausedApplyState;
  - ADD: use crate::merge::jinmerge::JinMergeConflict;
  - LOCATION: Top of file, after line 4 (after use crate::git...)
  - PATTERN: Follow existing import grouping (core, git, staging, then commands)

Task 2: MODIFY src/commands/status.rs - Add conflict detection function
  - IMPLEMENT: fn check_for_conflicts() -> Option<PausedApplyState>
  - LOCATION: After WorkspaceState enum definition (around line 18)
  - LOGIC:
    - Check if PausedApplyState::exists()
    - If exists, load state and return Some(state)
    - If doesn't exist, return None
    - Handle load errors by returning None (graceful degradation)
  - NAMING: check_for_conflicts
  - SIGNATURE: fn check_for_conflicts() -> Option<PausedApplyState>

Task 3: MODIFY src/commands/status.rs - Add conflict display function
  - IMPLEMENT: fn show_conflict_state(state: &PausedApplyState) -> Result<()>
  - LOCATION: After check_for_conflicts function (around line 25)
  - LOGIC:
    - Print "Merge conflicts (N file(s)):" header
    - For each file in state.conflict_files:
      - Convert to .jinmerge path using JinMergeConflict::merge_path_for_file()
      - Print "  {path}.jinmerge"
    - Print "  Resolve with: jin resolve <files>"
    - Print timestamp: "  Detected: {timestamp}"
    - Print blank line for spacing
  - ERROR HANDLING: Return Ok(()) if display succeeds, JinError on failure
  - NAMING: show_conflict_state

Task 4: MODIFY src/commands/status.rs - Integrate into execute()
  - LOCATION: In execute() function, after workspace state section (after line 86)
  - ADD:
    - Call check_for_conflicts() to get Option<PausedApplyState>
    - If Some(state), call show_conflict_state(&state)
    - Preserve all existing code after (staged changes, layer summary)
  - INTEGRATION POINT: Line 86, after WorkspaceState match block, before "Show staged files" comment

Task 5: CREATE tests/cli_basic.rs - Add integration test for conflict state
  - IMPLEMENT: test_status_shows_conflict_state()
  - PATTERN: Follow test_status_with_active_mode structure (lines 50-82)
  - SETUP:
    - Create temp dir and initialize Jin
    - Create a mode and activate it
    - Create conflicting files in two layers
    - Run jin apply to trigger conflicts
  - ASSERTIONS:
    - Run jin status
    - stdout contains "Merge conflicts"
    - stdout contains ".jinmerge" files
    - stdout contains "Resolve with: jin resolve"
  - NAMING: test_status_shows_conflict_state

Task 6: CREATE tests/cli_basic.rs - Add test for no conflict state
  - IMPLEMENT: test_status_no_conflicts_normal_display()
  - SETUP: Initialize Jin, create mode, no apply operation
  - ASSERTIONS:
    - Run jin status
    - stdout does NOT contain "Merge conflicts"
    - Normal status sections appear unchanged
  - NAMING: test_status_no_conflicts_normal_display

Task 7: ADD unit test in src/commands/status.rs
  - IMPLEMENT: test_check_for_conflicts_no_state()
  - LOCATION: In existing #[cfg(test)] mod (around line 243)
  - LOGIC: Verify check_for_conflicts() returns None when no state file exists
  - PATTERN: Follow test_check_workspace_state_clean_no_metadata pattern

Task 8: ADD unit test in src/commands/status.rs
  - IMPLEMENT: test_check_for_conflicts_with_state()
  - LOGIC: Create PausedApplyState, save it, verify check_for_conflicts() returns Some
  - PATTERN: Use TempDir for isolation, follow existing test patterns
```

### Implementation Patterns & Key Details

```rust
// ========== Task 2: check_for_conflicts function ==========
// Place after WorkspaceState enum (around line 18)

/// Check for in-progress apply operation with conflicts
fn check_for_conflicts() -> Option<PausedApplyState> {
    // Follow pattern from resolve.rs:33-37
    if !PausedApplyState::exists() {
        return None;
    }

    // Graceful degradation: if load fails, return None
    match PausedApplyState::load() {
        Ok(state) => Some(state),
        Err(_) => None,  // Don't fail status if state is corrupted
    }
}

// ========== Task 3: show_conflict_state function ==========
// Place after check_for_conflicts (around line 25)

/// Display conflict state from paused apply operation
fn show_conflict_state(state: &PausedApplyState) -> Result<()> {
    // Follow pluralization pattern from line 71-73 in status.rs
    let count = state.conflict_count;
    println!(
        "Merge conflicts ({} file{}):",
        count,
        if count == 1 { "" } else { "s" }
    );

    // List each .jinmerge file
    // CRITICAL: conflict_files contains original paths, convert to .jinmerge paths
    for original_path in &state.conflict_files {
        let merge_path = JinMergeConflict::merge_path_for_file(original_path);
        println!("  {}", merge_path.display());
    }

    // Show resolve instruction
    println!("  Resolve with: jin resolve <files>");

    // Show timestamp - use RFC3339 format for ISO 8601
    println!("  Detected: {}", state.timestamp.to_rfc3339());

    // Blank line for spacing (follow status section pattern)
    println!();

    Ok(())
}

// ========== Task 4: Integration into execute() ==========
// Location: Line 86, after workspace state match block, before line 88 comment

// In execute() function, after line 86 (after WorkspaceState match block):

    // Check and display conflict state (NEW SECTION)
    if let Some(conflict_state) = check_for_conflicts() {
        show_conflict_state(&conflict_state)?;
    }

    // Show staged files (EXISTING - preserve this)
    let staged_count = staging.len();
    // ... rest of existing code unchanged ...

// ========== Test Pattern for Task 5 ==========

#[test]
fn test_status_shows_conflict_state() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Jin
    jin().arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", std::process::id());
    jin().args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin().args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create files that will conflict
    // ... setup for creating conflicting content ...

    // Run apply to trigger conflicts (creates paused state)
    jin().arg("apply")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();  // Apply now pauses instead of failing

    // Check status shows conflicts
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge conflicts"))
        .stdout(predicate::str::contains(".jinmerge"))
        .stdout(predicate::str::contains("Resolve with: jin resolve"));
}
```

### Integration Points

```yaml
IMPORTS:
  - file: src/commands/status.rs
  - add_at_line: 5 (after existing use statements)
  - add:
    - use crate::commands::apply::PausedApplyState;
    - use crate::merge::jinmerge::JinMergeConflict;

FUNCTIONS:
  - file: src/commands/status.rs
  - add_after: WorkspaceState enum (line 18)
  - add:
    - fn check_for_conflicts() -> Option<PausedApplyState>
    - fn show_conflict_state(state: &PausedApplyState) -> Result<()

EXECUTE_MODIFICATION:
  - file: src/commands/status.rs
  - at_line: 86 (after workspace state match block)
  - insert:
    """
    // Check and display conflict state
    if let Some(conflict_state) = check_for_conflicts() {
        show_conflict_state(&conflict_state)?;
    }
    """

TESTS:
  - file: tests/cli_basic.rs
  - add_at: End of file (after existing status tests)
  - add:
    - test_status_shows_conflict_state
    - test_status_no_conflicts_normal_display

UNIT_TESTS:
  - file: src/commands/status.rs
  - add_at: In #[cfg(test)] mod (after line 265)
  - add:
    - test_check_for_conflicts_no_state
    - test_check_for_conflicts_with_state
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --message-format=short 2>&1 | head -50

# Format check
cargo fmt --check

# Clippy for lints
cargo clippy -- -D warnings 2>&1 | head -50

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
# Focus on:
# - Missing imports
# - Type mismatches
# - Unused variables
# - Formatting issues
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run status unit tests specifically
cargo test --lib status -- --nocapture

# Run all command tests
cargo test --lib commands:: -- --nocapture

# Full unit test suite
cargo test --lib -- --nocapture

# Expected: All tests pass. If failing, debug root cause and fix implementation.
# Watch for:
# - test_check_for_conflicts_no_state should pass
# - test_check_for_conflicts_with_state should pass
# - test_execute_not_initialized should still pass (no regression)
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the CLI first
cargo build --release

# Test 1: Status with no conflicts (baseline)
cd /tmp && mkdir -p test_jin_status && cd test_jin_status
jin init
jin mode create testmode
jin mode use testmode
jin status
# Expected: Normal output, NO "Merge conflicts" section

# Test 2: Status with conflicts
# (Setup requires creating conflicting layers - see integration test)
# Run the integration test:
cargo test --test cli_basic test_status_shows_conflict_state -- --nocapture

# Test 3: Full workflow test
cargo test --test cli_basic test_status -- --nocapture

# Expected: All integration tests pass, conflict section displays correctly
```

### Level 4: End-to-End Workflow Validation

```bash
# Manual E2E test of complete conflict resolution workflow
cd /tmp && mkdir -p jin_e2e && cd jin_e2e

# Initialize
export JIN_DIR=$(pwd)/.jin_global
jin init

# Create mode
jin mode create conflict_test
jin mode use conflict_test

# Create conflicting files
mkdir -p .claude
echo '{"version": "1"}' > .claude/config.json
jin add .claude/config.json
jin commit -m "First version"

# Create another mode with conflicting content
jin mode create conflict_test2
echo '{"version": "2"}' > .claude/config.json
jin add .claude/config.json --mode
jin commit -m "Second version"

# Trigger conflict by applying both
jin mode use conflict_test
jin apply --force  # This should create .jinmerge files

# Check status shows conflicts
jin status | grep "Merge conflicts"
# Expected: "Merge conflicts (1 file(s)):"

# Resolve and verify status updates
echo '{"version": "resolved"}' > .claude/config.json.jinmerge
jin resolve .claude/config.json

# Check status no longer shows conflicts
jin status | grep "Merge conflicts"
# Expected: No output (grep returns non-zero)

# Expected: Full workflow completes successfully
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo check` passes with zero errors
- [ ] `cargo fmt --check` passes (no formatting issues)
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test --lib` passes all unit tests
- [ ] `cargo test --test cli_basic` passes integration tests

### Feature Validation

- [ ] `jin status` with no paused state displays normally (regression check)
- [ ] `jin status` with paused state shows "Merge conflicts" section
- [ ] Conflict section appears between workspace state and staged changes
- [ ] All conflict files from state are listed with .jinmerge extension
- [ ] "Resolve with: jin resolve <files>" instruction is shown
- [ ] Timestamp is displayed in ISO 8601 format
- [ ] File count has correct pluralization (file vs files)
- [ ] After resolving all conflicts, status no longer shows conflict section

### Code Quality Validation

- [ ] Follows existing codebase patterns (println! not eprintln!, .display() for paths)
- [ ] Error handling uses Result<T> pattern consistently
- [ ] Functions are placed in logical order (types -> helpers -> execute)
- [ ] No new dependencies added (uses existing PausedApplyState)
- [ ] Import statements follow existing grouping

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Public functions have doc comments (///)
- [ ] Integration tests serve as usage documentation
- [ ] No configuration changes required

### Edge Cases Handled

- [ ] Corrupted or malformed .paused_apply.yaml file (graceful degradation)
- [ ] Empty conflict_files list (shows "0 files")
- [ ] Very long file paths (display wraps correctly)
- [ ] Multiple conflicts (all listed)
- [ ] Timestamp in different timezone (RFC3339 handles this)

---

## Anti-Patterns to Avoid

- **Don't** use `eprintln!` for status output - use `println!` (all status goes to stdout)
- **Don't** modify the existing workspace state, staged changes, or layer summary logic
- **Don't** create new data structures - use existing `PausedApplyState`
- **Don't** hardcode the state file path - use `PausedApplyState::exists()` and `load()`
- **Don't** place the conflict section after staged changes - it must be between workspace state and staged changes
- **Don't** show the conflict section if no paused state exists
- **Don't** use unwrap() on state loading - handle errors gracefully with Option/Result
- **Don't** forget to convert original paths to .jinmerge paths using `merge_path_for_file()`
- **Don't** skip the blank line after conflict section - it separates sections
- **Don't** write integration tests that depend on external state - use TempDir for isolation
- **Don't** modify existing tests - add new ones alongside them

## Confidence Score

**8/10** for one-pass implementation success likelihood

**Reasoning**:
- **High confidence** because:
  - All required data structures exist (`PausedApplyState`, `JinMergeConflict`)
  - Clear integration point identified (line 86 in status.rs)
  - Existing patterns to follow (resolve.rs for state loading, status.rs for display)
  - Test patterns well-established in codebase
  - Simple, scoped change (no new files, minimal modification)

- **Slight risk** because:
  - Integration test setup for creating conflicts may be complex
  - Need to ensure graceful degradation if state file is corrupted
  - Exact placement of conflict section matters for UX

- **Mitigation**: This PRP provides exact line numbers, complete code examples, and test patterns to minimize ambiguity.
