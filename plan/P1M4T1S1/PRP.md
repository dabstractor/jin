# PRP: P1.M4.T1.S1 - Update reset --hard validation logic

---

## Goal

**Feature Goal**: Modify the `jin reset --hard --force` command to bypass workspace attachment validation, enabling recovery from detached workspace states without requiring manual metadata cleanup.

**Deliverable**: A single-line logic change in `src/commands/reset.rs` that reorders the validation and confirmation check to make `--force` skip both operations.

**Success Definition**:
- `jin reset --hard --force` proceeds successfully even when workspace is in detached state
- `jin reset --hard` (without --force) still validates workspace attachment and shows error if detached
- All existing tests pass
- The fix follows existing patterns in the codebase for `--force` flag behavior

---

## User Persona

**Target User**: Jin users who encounter a "detached workspace" state and need to recover by performing a hard reset without having to manually resolve the detached state first.

**Use Case**: User has switched modes/scopes or performed other operations that caused workspace metadata to reference non-existent layers. The user wants to force a hard reset to discard staged changes and recover without running `jin apply` first.

**User Journey**:
1. User has a detached workspace (e.g., after deleting a mode that was in use)
2. User runs `jin reset --hard` → Gets "Workspace is in a detached state" error
3. User needs to recover, so runs `jin reset --hard --force` → Reset succeeds, files removed
4. User can now continue working or apply new configuration

**Pain Points Addressed**:
- **Before**: No way to recover from detached state using `jin reset` - must manually delete metadata or use `jin apply --force`
- **After**: `jin reset --hard --force` provides a clear recovery path
- **Consistency**: Matches `--force` behavior in other commands (`apply`, `rm`, `mv`)

---

## Why

- **Problem**: Currently, `jin reset --hard` always validates workspace attachment before allowing the operation, even when `--force` is specified. This prevents recovery from detached states.

- **User Impact**: Users in a detached state cannot use `jin reset --hard --force` to recover. They must either manually delete the workspace metadata file or use `jin apply --force` (which has different semantics).

- **Integration**: This fix completes P1.M4.T1 by enabling the recovery path documented in `plan/docs/fix_specifications.md`. It aligns with the established pattern where `--force` skips both validation and confirmation.

- **Code Quality**: The fix is minimal (one line change) and follows the existing pattern used in `src/commands/apply.rs` (lines 107-111) where `--force` skips validation checks.

---

## What

### User-Visible Behavior

**Current Behavior (Broken)**:
```bash
$ jin reset --hard --force
# Error: Workspace is in a detached state
# (validation happens BEFORE checking --force flag)
```

**Desired Behavior (After Fix)**:
```bash
$ jin reset --hard --force
# Success: Discarded N file(s) from staging and workspace
# (--force skips BOTH validation AND confirmation)
```

**Behavior Matrix**:

| Command | Validates Workspace? | Prompts for Confirmation? |
|---------|---------------------|--------------------------|
| `jin reset --hard` | YES (error if detached) | YES (requires "yes") |
| `jin reset --hard --force` | **NO** (skip validation) | **NO** (skip confirmation) |

### Technical Requirements

1. **Modify `src/commands/reset.rs`** (lines 58-64):
   - Current: Validation happens outside the `--force` check
   - New: Validation happens ONLY when `!args.force`
   - Change the condition to nest validation inside the force check

2. **Preserve existing behavior**:
   - `jin reset --soft` and `--mixed` continue to skip validation (no change)
   - `jin reset --hard` without `--force` still validates and prompts
   - `jin reset --hard --force` skips both validation and prompt

3. **No new files or dependencies**: This is a logic-only change

### Success Criteria

- [ ] `jin reset --hard --force` succeeds in detached state
- [ ] `jin reset --hard` (without --force) still fails in detached state with proper error message
- [ ] `cargo test` passes (all existing tests pass)
- [ ] Code follows existing `if !args.force { validate()?; }` pattern from codebase
- [ ] Help text for `--force` flag remains accurate (may be updated in P1.M4.T2.S1)

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the exact line numbers to modify, the current code pattern, the desired pattern to follow, the validation function implementation, test patterns to follow, and external research references._

### Documentation & References

```yaml
# MUST READ - Fix Specification
- file: /home/dustin/projects/jin/plan/docs/fix_specifications.md
  why: Contains the exact specification for P1.M4.T1.S1 fix
  section: "Fix 4: Reset in Detached State"
  critical: |
    Specifies that --force should skip workspace attachment validation.
    Current: validate_workspace_attached() happens before --force check
    New: validate_workspace_attached() happens inside "if !args.force" block

# IMPLEMENTATION: Current reset.rs code (MUST MODIFY)
- file: /home/dustin/projects/jin/src/commands/reset.rs
  why: This is the file to modify - lines 58-64 contain the validation logic
  section: "Lines 38-120: execute() function"
  current_pattern: |
    // 3.5. Validate workspace is attached before destructive operation
    if mode == ResetMode::Hard {
        let repo = JinRepo::open()?;
        validate_workspace_attached(&context, &repo)?;  // LINE 63: CURRENT LOCATION
    }

    // 4. Load staging
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 6. Confirmation for --hard mode
    if mode == ResetMode::Hard {
        if !args.force {  // LINE 79: Force check for confirmation only
            // ... prompt logic ...
        }
    }
  desired_pattern: |
    // 3.5. Validate workspace is attached before destructive operation (unless --force)
    if mode == ResetMode::Hard {
        if !args.force {  // NEW: Check --force FIRST
            let repo = JinRepo::open()?;
            validate_workspace_attached(&context, &repo)?;
        }
        // If --force, skip validation and proceed to load staging
    }

    // 4. Load staging (unchanged)
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 6. Confirmation for --hard mode (can be consolidated with above block)
    if mode == ResetMode::Hard {
        if !args.force {
            // ... prompt logic ...
        }
    }
  gotcha: |
    CRITICAL: The key insight is that validation (line 63) happens BEFORE
    the confirmation force check (line 79). We need to move validation
    INSIDE the "if !args.force" block so --force skips BOTH.

# REFERENCE: validate_workspace_attached function implementation
- file: /home/dustin/projects/jin/src/staging/workspace.rs
  why: Shows exactly what validation is being skipped
  section: "Lines 325-399: validate_workspace_attached() function"
  critical: |
    This function validates three conditions:
    1. File mismatch - returns JinError::DetachedWorkspace
    2. Missing commits/refs - returns JinError::DetachedWorkspace
    3. Invalid context - returns JinError::DetachedWorkspace

    When --force is used, we skip ALL these validations to allow recovery.

# PATTERN: Similar --force validation skip in apply.rs
- file: /home/dustin/projects/jin/src/commands/apply.rs
  why: Shows the EXACT pattern to follow for conditional validation with --force
  section: "Lines 107-120: Workspace dirty check and attachment validation"
  pattern: |
    // 2. Check workspace dirty (unless --force)
    if !args.force && check_workspace_dirty()? {
        return Err(JinError::Other(...));
    }

    // 2.5. Validate workspace state (only with --force)
    let repo = if args.force {
        let r = JinRepo::open()?;
        validate_workspace_attached(&context, &r)?;
        r
    } else {
        JinRepo::open()?
    };
  critical: |
    This shows "if !args.force { validate()?; }" pattern used elsewhere.
    The reset command should follow the same pattern for consistency.

# PATTERN: Similar --force confirmation skip in rm.rs
- file: /home/dustin/projects/jin/src/commands/rm.rs
  why: Shows the pattern of skipping confirmation with --force
  section: "Lines 98-107: Confirmation prompt with --force check"
  pattern: |
    if !files_to_remove_from_workspace.is_empty() && !args.force {
        let message = format!("This will remove {} file(s)...");
        if !prompt_confirmation(&message)? {
            println!("Removal cancelled");
            return Ok(());
        }
    }
  note: |
    Shows the standard "if !args.force" pattern for skipping confirmation.

# PATTERN: Similar --force confirmation skip in mv.rs
- file: /home/dustin/projects/jin/src/commands/mv.rs
  why: Shows another example of the --force confirmation skip pattern
  section: "Lines 113-122: Confirmation prompt with --force check"
  pattern: |
    if !files_to_move_in_workspace.is_empty() && !args.force {
        // prompt logic
    }

# ARGUMENTS: ResetArgs struct definition
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: Shows the --force flag definition and help text
  section: "Lines 56-90: ResetArgs struct"
  pattern: |
    #[derive(Args, Debug)]
    pub struct ResetArgs {
        #[arg(long)]
        pub soft: bool,

        #[arg(long)]
        pub mixed: bool,

        #[arg(long)]
        pub hard: bool,

        #[arg(long)]
        pub mode: bool,

        #[arg(long)]
        pub scope: Option<String>,

        #[arg(long)]
        pub project: bool,

        #[arg(long)]
        pub global: bool,

        /// Skip confirmation prompt for destructive operations
        #[arg(long, short = 'f')]
        pub force: bool,
    }
  note: |
    Current help text says "Skip confirmation prompt" - this will be
    updated in P1.M4.T2.S1 to reflect that it also skips validation.

# TESTS: Existing reset tests (REFERENCE ONLY)
- file: /home/dustin/projects/jin/tests/cli_reset.rs
  why: Shows existing test patterns for reset functionality
  section: "All test functions"
  pattern: |
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_reset_hard_mode_with_confirmation() {
        // Tests --hard without --force prompts
    }

    #[test]
    fn test_reset_hard_mode_with_force() {
        // Tests --hard with --force skips confirmation
    }
  note: |
    These tests already exist and should continue to pass after the fix.
    No new tests are added in THIS subtask (P1.M4.T1.S1) - tests come in P1.M4.T3.S1.

# TESTS: Destructive validation tests (REFERENCE)
- file: /home/dustin/projects/jin/tests/destructive_validation.rs
  why: Shows test patterns for detached state validation
  section: "Test functions for reset --hard validation"
  pattern: |
    #[test]
    #[serial]
    fn test_reset_hard_rejected_when_files_modified() {
        // Creates detached state
        // Runs reset --hard
        // Asserts DetachedWorkspace error
    }

    #[test]
    #[serial]
    fn test_reset_hard_allows_fresh_workspace() {
        // Tests that fresh workspaces (no metadata) pass validation
    }
  critical: |
    These tests verify the current behavior where --hard validates.
    After our fix, --hard --force should skip this validation.
    New test for --hard --force will be added in P1.M4.T3.S1.

# EXTERNAL RESEARCH: Rust CLI --force best practices
- url: https://clig.dev/
  why: Command Line Interface Guidelines - best practices for --force flag
  section: "Destructive operations section"
  critical: |
    "--force should skip BOTH validation and confirmation for destructive
    operations. This is the standard pattern across CLI tools."

- url: https://docs.rs/clap/latest/clap/builder/struct.Arg.html
  why: Official clap documentation for argument definitions
  section: "Force flag documentation"
  critical: |
    "The only time a user should be required to use a flag is if the
    operation is destructive in nature, and the user is essentially
    proving to you, 'Yes, I know what I'm doing.'"

- url: https://www.gnu.org/software/coreutils/manual/html_node/rm-invocation.html
  why: POSIX standard for `rm -f` - the classic skip-both pattern
  section: "rm -f documentation"
  critical: |
    "rm -f: ignore nonexistent files and arguments, never prompt.
    This overrides any previous -i or --interactive option."
    Shows that --force should take precedence and skip ALL checks.

- url: https://git-scm.com/docs/git-rm
  why: Git's implementation of --force flag for destructive operations
  section: "--force flag documentation"
  critical: |
    "Override the upstream checks if the file exists and has staged
    changes that are different from the upstream."
    Shows --force used to bypass safety validations.

# ERROR: DetachedWorkspace error type
- file: /home/dustin/projects/jin/src/core/error.rs
  why: Defines the error type that validate_workspace_attached returns
  section: "JinError::DetachedWorkspace variant"
  pattern: |
    pub enum JinError {
        DetachedWorkspace {
            workspace_commit: Option<String>,
            expected_layer_ref: String,
            details: String,
            recovery_hint: String,
        },
        // ... other variants
    }
  note: |
    When workspace is detached, this error is returned with details about
    what's wrong and how to recover. With --force, we skip returning this.

# UNIT TEST: Existing reset unit test
- file: /home/dustin/projects/jin/src/commands/reset.rs
  why: Shows existing unit test patterns in reset.rs
  section: "Lines 392-428: test_reset_hard_with_force() test"
  pattern: |
    #[test]
    #[serial]
    fn test_reset_hard_with_force() {
        // Setup: Create test file and stage it
        let ctx = crate::test_utils::setup_unit_test();
        let test_file = project_path.join("test.json");
        std::fs::write(&test_file, r#"{"test": true}"#).unwrap();

        // Stage the file
        let mut staging = StagingIndex::new();
        let entry = StagedEntry { /* ... */ };
        staging.add(entry);
        staging.save().unwrap();

        // Reset hard with force flag (should not prompt)
        let args = ResetArgs {
            hard: true,
            force: true,
            // ... other fields
        };
        let result = execute(args);
        assert!(result.is_ok());

        // Verify file was deleted
        assert!(!test_file.exists());
    }
  note: |
    This test already tests --hard --force, but doesn't test the detached
    state scenario. New test for detached state will be added in P1.M4.T3.S1.
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   └── args.rs                    # REFERENCE: ResetArgs definition (lines 56-90)
│   ├── commands/
│   │   ├── reset.rs                   # MODIFY: Lines 58-64 for validation fix
│   │   ├── apply.rs                   # REFERENCE: Similar --force pattern (lines 107-120)
│   │   ├── rm.rs                      # REFERENCE: Confirmation skip pattern (lines 98-107)
│   │   └── mv.rs                      # REFERENCE: Confirmation skip pattern (lines 113-122)
│   ├── staging/
│   │   ├── workspace.rs              # REFERENCE: validate_workspace_attached() (lines 325-399)
│   │   └── metadata.rs               # REFERENCE: WorkspaceMetadata structure
│   └── core/
│       └── error.rs                  # REFERENCE: JinError::DetachedWorkspace variant
├── tests/
│   ├── cli_reset.rs                  # REFERENCE: Existing reset tests
│   └── destructive_validation.rs     # REFERENCE: Detached state test patterns
└── plan/
    └── docs/
        └── fix_specifications.md     # REFERENCE: P1.M4.T1.S1 fix specification
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── src/
    └── commands/
        └── reset.rs                   # MODIFIED: Lines 58-64 moved validation inside --force check
            # BEFORE (current):
            # if mode == ResetMode::Hard {
            #     let repo = JinRepo::open()?;
            #     validate_workspace_attached(&context, &repo)?;
            # }
            #
            # AFTER (fixed):
            # if mode == ResetMode::Hard {
            #     if !args.force {
            #         let repo = JinRepo::open()?;
            #         validate_workspace_attached(&context, &repo)?;
            #     }
            # }
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Current code structure has validation BEFORE confirmation force check
// Lines 58-64: Validation happens unconditionally for --hard mode
// Lines 76-90: Confirmation happens conditionally (checks !args.force)
//
// The fix requires nesting validation inside the "if !args.force" block
// so that --force skips BOTH validation AND confirmation.

// GOTCHA: Validation requires JinRepo::open()
// validate_workspace_attached needs a &JinRepo parameter
// Current code opens repo at line 62: let repo = JinRepo::open()?;
// This needs to move inside the "if !args.force" block

// GOTCHA: Don't confuse with apply.rs pattern
// apply.rs does validation ONLY when --force is set (line 114-120)
// reset.rs should skip validation when --force is set (inverse)
// Pattern to follow: if !args.force { validate()?; }

// PATTERN: Standard --force validation skip
// From apply.rs, rm.rs, mv.rs - all use: if !args.force { validate()?; }
// This is the pattern to follow for consistency

// GOTCHA: ResetMode::Soft and ResetMode::Mixed skip validation
// Only ResetMode::Hard validates workspace attachment
// This is correct - soft/mixed are non-destructive

// GOTCHA: Error message format for DetachedWorkspace
// When validation fails, user sees:
// "Workspace is in a detached state."
// Followed by details and recovery hint
// With --force, this error is never returned

// CRITICAL: The fix is a logic reordering, not new code
// Current flow:
//   1. Determine mode
//   2. Load context
//   3. Determine layer
//   4. [VALIDATE for Hard mode] <- CURRENT: Always happens
//   5. Load staging
//   6. [CONFIRM for Hard mode] <- CURRENT: Skips if --force
//   7. Perform reset
//
// New flow:
//   1. Determine mode
//   2. Load context
//   3. Determine layer
//   4. [VALIDATE for Hard mode] <- NEW: Only if !--force
//   5. Load staging
//   6. [CONFIRM for Hard mode] <- Same as before: Skips if --force
//   7. Perform reset

// GOTCHA: Consider consolidating validation and confirmation blocks
// Both are "if mode == ResetMode::Hard && !args.force"
// Could be combined, but keeping separate is fine for clarity
// Future PRP (P1.M4.T2.S1) may update help text to clarify

// CRITICAL: No new dependencies needed
// This is a pure logic change using existing functions and types

// GOTCHA: Test implications
// Existing tests for --hard --force (test_reset_hard_with_force) should
// continue to pass because they don't create a detached state
// New test for detached state recovery will be added in P1.M4.T3.S1

// PATTERN: Error handling in Jin
// All validate functions return Result<()>
// Errors are propagated up with ? operator
// validate_workspace_attached returns JinError::DetachedWorkspace

// GOTCHA: WorkspaceMetadata::load() behavior
// Returns Err(JinError::NotFound) if no metadata exists
// validate_workspace_attached treats this as Ok(()) - fresh workspace
// This means --hard --force works for fresh workspaces too (no-op)
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is a logic-only change using existing structures:
- `ResetArgs` from `src/cli/args.rs` (already has `force: bool` field)
- `ResetMode` enum from `src/commands/reset.rs`
- `JinError::DetachedWorkspace` from `src/core/error.rs`

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/commands/reset.rs VALIDATION LOGIC
  - FILE: src/commands/reset.rs
  - LOCATION: Lines 58-64 (execute() function, validation block)
  - CURRENT CODE:
    ```rust
    // 3.5. Validate workspace is attached before destructive operation
    // CRITICAL: Only validate for Hard mode (destructive)
    // CRITICAL: Validation happens BEFORE confirmation prompt - don't prompt if operation will be rejected
    if mode == ResetMode::Hard {
        let repo = JinRepo::open()?;
        validate_workspace_attached(&context, &repo)?;
    }
    ```
  - NEW CODE:
    ```rust
    // 3.5. Validate workspace is attached before destructive operation (unless --force)
    // CRITICAL: Only validate for Hard mode (destructive) AND when --force is not set
    // CRITICAL: Validation happens BEFORE confirmation prompt - don't prompt if operation will be rejected
    // CRITICAL: When --force is set, skip both validation AND confirmation
    if mode == ResetMode::Hard {
        if !args.force {
            let repo = JinRepo::open()?;
            validate_workspace_attached(&context, &repo)?;
        }
        // If --force, skip validation and proceed to load staging
    }
    ```
  - DEPENDENCIES: None (this is the only change needed)

Task 2: VERIFY CODE COMPILES
  - RUN: cargo check
  - EXPECTED: Zero errors, zero warnings
  - DEPENDENCIES: Task 1

Task 3: RUN EXISTING TESTS
  - RUN: cargo test
  - EXPECTED: All tests pass (no test changes needed for this subtask)
  - DEPENDENCIES: Task 1
  - NOTE: Tests for --hard --force in detached state will be added in P1.M4.T3.S1

Task 4: MANUAL VERIFICATION (Optional)
  - CREATE: Test scenario with detached workspace
  - RUN: jin reset --hard --force
  - VERIFY: Command succeeds without error
  - DEPENDENCIES: Task 1
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE CHANGE ==================

// FILE: src/commands/reset.rs
// LOCATION: Lines 58-64 in execute() function

// --- BEFORE (CURRENT CODE) ---
// 3.5. Validate workspace is attached before destructive operation
// CRITICAL: Only validate for Hard mode (destructive)
// CRITICAL: Validation happens BEFORE confirmation prompt - don't prompt if operation will be rejected
if mode == ResetMode::Hard {
    let repo = JinRepo::open()?;
    validate_workspace_attached(&context, &repo)?;
}

// --- AFTER (FIXED CODE) ---
// 3.5. Validate workspace is attached before destructive operation (unless --force)
// CRITICAL: Only validate for Hard mode (destructive) AND when --force is not set
// CRITICAL: Validation happens BEFORE confirmation prompt - don't prompt if operation will be rejected
// CRITICAL: When --force is set, skip both validation AND confirmation
if mode == ResetMode::Hard {
    if !args.force {
        let repo = JinRepo::open()?;
        validate_workspace_attached(&context, &repo)?;
    }
    // If --force, skip validation and proceed to load staging
}

// ================== PATTERN EXPLANATION ==================
//
// This change follows the established pattern in the codebase:
//
// From src/commands/apply.rs (lines 107-111):
//   if !args.force && check_workspace_dirty()? {
//       return Err(JinError::Other(...));
//   }
//
// From src/commands/rm.rs (lines 98-107):
//   if !files_to_remove_from_workspace.is_empty() && !args.force {
//       if !prompt_confirmation(&message)? {
//           return Ok(());
//       }
//   }
//
// The pattern is: if !args.force { validate()?; }
//
// This ensures --force skips the validation/confirmation.

// ================== WHY THIS WORKS ==================
//
// The confirmation prompt at lines 76-90 already checks !args.force:
//
//   if mode == ResetMode::Hard {
//       if !args.force {
//           // ... prompt confirmation ...
//       }
//   }
//
// By nesting validation inside the same !args.force check, we ensure
// that --force skips BOTH operations:
//
//   WITHOUT --force:
//     1. Validate workspace (error if detached)
//     2. Load staging
//     3. Prompt for confirmation
//     4. Perform reset
//
//   WITH --force:
//     1. Skip validation
//     2. Load staging
//     3. Skip confirmation
//     4. Perform reset
//
// This provides the recovery path needed for detached workspaces.

// ================== ALTERNATIVE CONSIDERED ==================
//
// We could consolidate the validation and confirmation into a single block:
//
//   if mode == ResetMode::Hard && !args.force {
//       let repo = JinRepo::open()?;
//       validate_workspace_attached(&context, &repo)?;
//
//       // ... prompt confirmation ...
//   }
//
// However, keeping them separate maintains clearer separation of concerns:
// - Validation is about safety (preventing data loss from detached state)
// - Confirmation is about user intent (making sure they want to delete)
//
// The current structure with two separate "if mode == ResetMode::Hard" blocks
// is clearer and matches the existing code structure.

// ================== VALIDATION FLOW ==================
//
// validate_workspace_attached() checks three conditions:
//
// 1. File Mismatch: Workspace files modified outside Jin operations
//    → Error: "Workspace files have been modified outside of Jin operations"
//
// 2. Missing Commits/Refs: Layer refs in metadata no longer exist
//    → Error: "Workspace metadata references layers that no longer exist"
//
// 3. Invalid Context: Active mode/scope references deleted entities
//    → Error: "Active context references a mode or scope that no longer exists"
//
// With --force, we skip ALL these checks to allow recovery.
//
// The user is explicitly saying "I know what I'm doing - force this operation."
//
// This matches the POSIX convention of `rm -f` which:
// - Skips confirmation prompts
// - Suppresses error messages
// - Allows operations that would otherwise be rejected

// ================== TESTING STRATEGY ==================
//
// Existing tests (should continue to pass):
// - test_reset_hard_with_force: Tests --hard --force in normal state
//
// New tests (will be added in P1.M4.T3.S1):
// - test_reset_hard_rejected_in_detached_state
// - test_reset_hard_force_succeeds_in_detached_state
//
// The key distinction:
// - --hard (without --force): Should FAIL in detached state
// - --hard --force: Should SUCCEED in detached state

// ================== CONSISTENCY WITH OTHER COMMANDS ==================
//
// apply --force:
//   - Skips workspace dirty check
//   - Performs workspace attachment validation
//   (Different semantics: --force enables validation for safety)
//
// rm --force:
//   - Skips confirmation prompt for workspace deletion
//   (Same pattern: --force skips user interaction)
//
// mv --force:
//   - Skips confirmation prompt for workspace moves
//   (Same pattern: --force skips user interaction)
//
// reset --hard --force (after fix):
//   - Skips workspace attachment validation
//   - Skips confirmation prompt
//   (Same pattern: --force skips safety checks and prompts)
//
// The consistency is: --force means "I know what I'm doing, skip the checks"
```

### Integration Points

```yaml
VALIDATION_FUNCTION:
  - function: validate_workspace_attached
  - file: src/staging/workspace.rs
  - lines: 325-399
  - behavior: Returns JinError::DetachedWorkspace if validation fails
  - called_from: src/commands/reset.rs line 63 (moving into !args.force block)

ERROR_TYPE:
  - enum: JinError::DetachedWorkspace
  - file: src/core/error.rs
  - fields: workspace_commit, expected_layer_ref, details, recovery_hint
  - when_returned: When validate_workspace_attached detects detached state
  - after_fix: Never returned when --force is set

CONFIRMATION_PROMPT:
  - function: prompt_confirmation
  - file: src/commands/reset.rs
  - lines: 217-225
  - called_from: Line 85 in confirmation block
  - behavior: Prompts user for "yes" input
  - after_fix: Skipped when --force is set (no change)

ARGUMENTS:
  - struct: ResetArgs
  - file: src/cli/args.rs
  - lines: 56-90
  - field: force: bool
  - flag: --force, -f
  - help_text: "Skip confirmation prompt for destructive operations"
  - note: Help text will be updated in P1.M4.T2.S1 to reflect validation skip

TEST_FILES:
  - file: tests/cli_reset.rs
  - existing_tests: test_reset_hard_with_force
  - new_tests (P1.M4.T3.S1): test_reset_hard_force_in_detached_state

  - file: tests/destructive_validation.rs
  - existing_tests: test_reset_hard_rejected_when_*
  - new_tests (P1.M4.T3.S1): test_reset_hard_force_bypasses_validation
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after making the code change - fix before proceeding
cargo check                              # Type checking - MUST pass
cargo fmt -- --check                     # Format check - should pass

# Expected: Zero errors, zero warnings
# If errors exist, READ output and fix before proceeding
```

### Level 2: Unit Test Compilation

```bash
# Verify tests compile
cargo test --no-run

# Expected: Tests compile successfully
# If compilation fails, check syntax and imports
```

### Level 3: Test Execution (Component Validation)

```bash
# Run all reset tests
cargo test reset -- --nocapture

# Run specific test that uses --force
cargo test test_reset_hard_with_force -- --nocapture

# Run all destructive validation tests
cargo test destructive_validation -- --nocapture

# Expected: All tests pass
# Key test: test_reset_hard_with_force should continue to pass
```

### Level 4: Full Test Suite (System Validation)

```bash
# Run full test suite to ensure no regressions
cargo test

# Expected: All tests pass
# Focus areas: reset, destructive_validation, commands

# Verify tests are serial-safe (run multiple times)
cargo test reset -- --test-threads=1
cargo test test_reset_hard_with_force -- --test-threads=1
```

### Level 5: Manual Verification (Optional - For Developer Confidence)

```bash
# Manual verification (in temporary directory)
cd $(mktemp -d)
export JIN_DIR=$(pwd)/.jin_global
git init
jin init

# Create mode and apply configuration
jin mode create test_mode
jin mode use test_mode
echo '{"test": true}' > config.json
jin add --mode config.json
jin commit -m "Test config"
jin apply

# Create detached state by deleting the mode
jin mode delete test_mode

# Try reset --hard (should fail)
jin reset --hard
# Expected: Error "Workspace is in a detached state"

# Try reset --hard --force (should succeed)
jin reset --hard --force
# Expected: Success "Discarded N file(s) from staging and workspace"

# Cleanup
cd -
rm -rf "$OLDPWD"

# Expected: --force bypasses validation and succeeds
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test reset` all tests pass
- [ ] `cargo test destructive_validation` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] Code change follows exact pattern specified in Implementation Blueprint
- [ ] Validation at line 63 is now inside "if !args.force" block
- [ ] `jin reset --hard --force` would skip validation (logic verified by inspection)
- [ ] `jin reset --hard` (without --force) still validates (logic verified by inspection)
- [ ] Change follows existing `if !args.force { validate()?; }` pattern from codebase

### Code Quality Validation

- [ ] Comment blocks updated to reflect new behavior
- [ ] Code structure matches existing patterns in reset.rs
- [ ] No unnecessary code changes (only the validation nesting)
- [ ] Import statements unchanged (no new dependencies)

### Documentation & Deployment

- [ ] Code comments explain the --force skip behavior
- [ ] Implementation matches fix specification in plan/docs/fix_specifications.md
- [ ] Ready for P1.M4.T2.S1 (help text update) and P1.M4.T3.S1 (test addition)

---

## Anti-Patterns to Avoid

- **Don't** change the confirmation prompt logic (lines 76-90) - it's already correct
- **Don't** add new functions or imports - this is a pure logic reordering
- **Don't** modify the ResetArgs struct - the force field already exists
- **Don't** change validate_workspace_attached function - it's correct as-is
- **Don't** add new tests in this subtask - tests come in P1.M4.T3.S1
- **Don't** combine validation and confirmation into one block - keep separate for clarity
- **Don't** use `args.force` alone - must be `!args.force` to match existing pattern
- **Don't** forget the JinRepo::open() call - validation needs the repo parameter
- **Don't** skip running existing tests - they should all still pass
- **Don't** over-engineer the fix - one line change (adding the if !args.force block)

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single-line change**: Only nesting validation inside `if !args.force` block
- **Exact specification**: Fix specification clearly defines the change
- **Established pattern**: Same `if !args.force { validate()?; }` pattern used in apply.rs, rm.rs, mv.rs
- **Isolated change**: No new functions, imports, or dependencies
- **Clear before/after**: Current code and target code are both specified
- **Existing tests pass**: No test changes needed for this subtask
- **Comprehensive research**: Validation function, test patterns, and external research all gathered

**Zero Risk Factors**:
- Logic change is minimal and localized
- No new dependencies or data structures
- Change follows established codebase patterns
- Existing tests provide safety net
- Fix is reversible (can nest/un-nest the validation)

**Current Status**: Ready for implementation - all context gathered, pattern identified, exact code change specified

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from this PRP creation):
- `plan/P1M4T1S1/research/` - Directory for all research findings
  - Agent research files stored here for reference

**Background Documentation**:
- `src/commands/reset.rs` - File to modify (lines 58-64)
- `src/staging/workspace.rs` - validate_workspace_attached function (lines 325-399)
- `src/cli/args.rs` - ResetArgs struct definition (lines 56-90)
- `src/commands/apply.rs` - Similar --force pattern (lines 107-120)
- `src/commands/rm.rs` - Confirmation skip pattern (lines 98-107)
- `src/commands/mv.rs` - Confirmation skip pattern (lines 113-122)
- `tests/cli_reset.rs` - Existing reset tests
- `tests/destructive_validation.rs` - Detached state test patterns

**Pattern References**:
- `if !args.force { validate()?; }` - Standard validation skip pattern
- `validate_workspace_attached()` - Workspace attachment validation
- `JinError::DetachedWorkspace` - Error type returned on validation failure

**Related Work Items**:
- `plan/P1M4T2S1/PRP.md` - Will update --force help text (future)
- `plan/P1M4T3S1/PRP.md` - Will add detached state tests (future)

**External Research**:
- [Command Line Interface Guidelines](https://clig.dev/) - --force flag best practices
- [clap Documentation](https://docs.rs/clap/latest/clap/builder/struct.Arg.html) - Argument definitions
- [POSIX rm -f](https://www.gnu.org/software/coreutils/manual/html_node/rm-invocation.html) - Classic skip-both pattern
- [git rm --force](https://git-scm.com/docs/git-rm) - Git's --force implementation
