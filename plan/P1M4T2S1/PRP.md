# PRP: P1.M4.T2.S1 - Update --force help text in ResetArgs

---

## Goal

**Feature Goal**: Update the help text for the `--force` flag in the `reset` command to accurately reflect its enhanced behavior of skipping both confirmation prompts AND workspace attachment validation.

**Deliverable**: A single-line documentation change in `src/cli/args.rs` that updates the help attribute for the `force` field in the `ResetArgs` struct.

**Success Definition**:
- `jin reset --help` displays the updated help text for `--force`
- Help text accurately describes both behaviors: skipping confirmation and bypassing validation
- Help text provides guidance on when to use the flag (recovery scenarios)
- All existing tests pass
- The updated help text follows existing patterns in the codebase

---

## User Persona

**Target User**: Jin users who need to understand what the `--force` flag does when running `jin reset --hard`.

**Use Case**: User encounters a "detached workspace" state and needs to understand that `--force` can be used to bypass the validation and recover, but wants to know what the flag actually does before using it.

**User Journey**:
1. User runs `jin reset --hard` and gets an error about detached state
2. User wants to understand their options, so runs `jin reset --help`
3. User sees the `--force` flag help text explaining it skips both confirmation AND validation for recovery
4. User understands the trade-offs and can make an informed decision

**Pain Points Addressed**:
- **Before**: Help text only mentioned "Skip confirmation prompt" - users wouldn't know that `--force` also bypasses validation (the key new feature from P1.M4.T1.S1)
- **After**: Help text clearly explains both behaviors and provides context that it's for recovery scenarios

---

## Why

- **Problem**: The current help text for `--force` in `reset` says "Skip confirmation prompt for destructive operations". After P1.M4.T1.S1, the flag now also skips workspace attachment validation. Users won't know this new behavior exists.

- **User Impact**: Users in a detached state won't realize that `jin reset --hard --force` provides a recovery path, because the help text doesn't mention the validation bypass behavior.

- **Integration**: This completes P1.M4.T2 by updating documentation to match the new behavior implemented in P1.M4.T1.S1. The help text should accurately reflect what the code now does.

- **Code Quality**: Accurate documentation is critical for user experience. Users should be able to understand flag behavior from help text alone.

---

## What

### User-Visible Behavior

**Current Help Text** (inaccurate - only mentions confirmation):
```bash
$ jin reset --help
...
OPTIONS:
    -f, --force
            Skip confirmation prompt for destructive operations
...
```

**Desired Help Text** (accurate - mentions both confirmation AND validation):
```bash
$ jin reset --help
...
OPTIONS:
    -f, --force
            Skip confirmation prompt and bypass detached state validation (use for recovery)
...
```

### Technical Requirements

1. **Modify `src/cli/args.rs`** (line 87):
   - Current: `/// Skip confirmation prompt for destructive operations`
   - New: `/// Skip confirmation prompt and bypass detached state validation (use for recovery)`

2. **Preserve existing behavior**:
   - The `--force` flag behavior doesn't change (only the help text changes)
   - Flag attributes `#[arg(long, short = 'f')]` remain unchanged

3. **No new files or dependencies**: This is a documentation-only change

### Success Criteria

- [ ] `jin reset --help` displays the updated help text for `--force`
- [ ] Help text mentions both "confirmation prompt" and "detached state validation"
- [ ] Help text includes "(use for recovery)" guidance
- [ ] All existing tests pass (including `test_reset_help`)
- [ ] Help text format matches existing patterns in the codebase

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the exact line number to modify, the current help text, the desired help text, examples of similar help text patterns in the codebase, test patterns to verify the change, and external research references._

### Documentation & References

```yaml
# CONTRACT FROM P1.M4.T1.S1 - Must reflect this behavior
- file: /home/dustin/projects/jin/plan/P1M4T1S1/PRP.md
  why: P1.M4.T1.S1 implements the logic change that this help text must describe
  section: "Goal Section - Feature Goal"
  critical: |
    P1.M4.T1.S1 modifies the logic so that --force skips BOTH validation AND confirmation.
    The help text must accurately describe this new behavior.
    The key behavior: "jin reset --hard --force bypasses workspace attachment validation"

# IMPLEMENTATION: Current ResetArgs force field (MUST MODIFY)
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: This is the file to modify - line 87 contains the help text to update
  section: "Lines 56-90: ResetArgs struct"
  current_code: |
    /// Skip confirmation prompt for destructive operations
    #[arg(long, short = 'f')]
    pub force: bool,
  new_code: |
    /// Skip confirmation prompt and bypass detached state validation (use for recovery)
    #[arg(long, short = 'f')]
    pub force: bool,
  location: "Line 87 - the doc comment above the force field"
  gotcha: |
    CRITICAL: This is a doc comment (///), NOT a regular comment.
    Doc comments are what clap uses for help text.
    Must preserve the triple-slash format.

# REFERENCE: Other --force flag help text in codebase
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: Shows patterns used for --force help text in other commands
  section: "Various Args structs"
  examples: |
    ApplyArgs (line 47-49):
      /// Force apply even if workspace is dirty

    RmArgs (line 118-120):
      /// Skip confirmation prompt for workspace deletion

    MvArgs (line 153-155):
      /// Skip confirmation prompt for workspace moves

    ResolveArgs (line 247-249):
      /// Skip confirmation prompts
  note: |
    These examples show that --force flags are typically described as either:
    1. "Skip confirmation prompt" (for destructive operations)
    2. "Force even if..." (for validation bypass)
    Our new text combines both approaches.

# REFERENCE: Multi-line help text example
- file: /home/dustin/projects/jin/src/cli/mod.rs
  why: Shows how to write comprehensive help text with examples
  section: "Lines 111-119: Completion command"
  pattern: |
    /// Generate shell completion scripts
    ///
    /// Outputs completion script to stdout...
    /// Installation:
    ///   Bash:       jin completion bash > ...
  note: |
    Multi-line help text is possible, but single-line is preferred for simple flags.
    Our updated help text should remain single-line for consistency.

# TEST: Existing reset help test
- file: /home/dustin/projects/jin/tests/cli_reset.rs
  why: Shows the existing test that verifies reset help output
  section: "Lines 294-307: test_reset_help()"
  current_test: |
    #[test]
    fn test_reset_help() {
        jin()
            .args(["reset", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Reset staged or committed changes"))
            .stdout(predicate::str::contains("--soft"))
            .stdout(predicate::str::contains("--mixed"))
            .stdout(predicate::str::contains("--hard"))
            .stdout(predicate::str::contains("--global"))
            .stdout(predicate::str::contains("--force"));
    }
  note: |
    This test checks that --help contains "--force" (the flag name).
    The test doesn't validate the specific help text content, so it will pass
    with our updated help text. No test changes needed for this subtask.

# PATTERN: Help text testing in codebase
- file: /home/dustin/projects/jin/tests/cli_basic.rs
  why: Shows how to test help text content if needed
  section: "Various test functions"
  patterns: |
    #[test]
    fn test_help() {
        jin()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Phantom Git layer system"));
    }

    #[test]
    fn test_link_help() {
        jin()
            .args(["link", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Remote repository URL"))
            .stdout(predicate::str::contains("--force"));
    }
  critical: |
    Pattern: Use predicate::str::contains() to check for specific text in help output.
    Could add test for new help text if desired, but not required for this subtask.

# CLAP: Version and API reference
- file: /home/dustin/projects/jin/Cargo.toml
  why: Confirms clap version for documentation reference
  section: "Line 24: clap dependency"
  version: "clap = { version = \"4.5\", features = [\"derive\", \"cargo\"] }"
  note: |
    Using clap v4.5 with derive API. Doc comments (///) are the standard way
    to specify help text with clap's derive API.

# EXTERNAL RESEARCH: Clap help text documentation
- url: https://docs.rs/clap/latest/clap/_derive/index.html
  why: Official clap documentation for help text with derive API
  section: "Doc Comments"
  critical: |
    "When using the derive API, doc comments are automatically used for
    help text. The first line of the doc comment becomes the short help,
    and the full doc comment becomes the long help."
    Confirms that /// doc comments are the correct approach.

- url: https://docs.rs/clap/latest/clap/struct.Arg.html
  why: Arg struct documentation for help attribute
  section: "help() method"
  critical: |
    "The help text for the arg. This is typically a short, one-line message."
    Confirms that single-line help text is standard practice.

# EXTERNAL RESEARCH: CLI help text best practices
- url: https://clig.dev/
  why: Command Line Interface Guidelines - help text best practices
  section: "Help Text"
  critical: |
    "Help text should be concise but complete. Use parenthetical notes for
    usage context. Mention what the flag does AND when to use it."
    Our text "(use for recovery)" follows this pattern.

- url: https://github.com/clap-rs/clap/tree/master/examples
  why: Official clap examples showing help text patterns
  section: "derive examples"
  critical: |
    Examples show that doc comments are the standard way to specify help.
    Help text typically focuses on WHAT the flag does, sometimes with context.

# EXTERNAL RESEARCH: Git --force help text comparison
- url: https://git-scm.com/docs/git-reset
  why: Git's reset command documentation for comparison
  section: "--force flag"
  critical: |
    Git reset doesn't have a --force flag, but git push --force says:
    "Force push (overwrite remote)" - clear, concise, explains behavior.
    Our text "Skip confirmation prompt and bypass detached state validation"
    follows the same pattern of clear, actionable description.

- url: https://git-scm.com/docs/git-rm
  why: Git rm --force documentation
  section: "-f, --force"
  critical: |
    "Override the upstream checks" - explains what validation is bypassed.
    Our text "bypass detached state validation" follows this pattern.

# CONTEXT: What is "detached state validation"?
- file: /home/dustin/projects/jin/src/staging/workspace.rs
  why: Provides context for what "detached state validation" means
  section: "Lines 325-399: validate_workspace_attached()"
  critical: |
    This function validates that:
    1. Workspace files haven't been modified outside Jin
    2. Workspace metadata references layers that still exist
    3. Active mode/scope references still exist

    When validation fails, user sees "Workspace is in a detached state" error.
    With --force, this validation is skipped to allow recovery.

# CONTEXT: Related error message users might see
- file: /home/dustin/projects/jin/src/core/error.rs
  why: Shows the error message that users see when validation fails
  section: "JinError::DetachedWorkspace variant"
  pattern: |
    DetachedWorkspace {
        workspace_commit: Option<String>,
        expected_layer_ref: String,
        details: String,
        recovery_hint: String,
    }
  note: |
    When validation fails, users see "Workspace is in a detached state."
    The --force flag provides the recovery path mentioned in the error.
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   └── args.rs                    # MODIFY: Line 87 - force field help text
│   ├── commands/
│   │   └── reset.rs                   # REFERENCE: Logic changed in P1.M4.T1.S1
│   ├── staging/
│   │   └── workspace.rs              # REFERENCE: validate_workspace_attached()
│   └── core/
│       └── error.rs                  # REFERENCE: DetachedWorkspace error type
├── tests/
│   └── cli_reset.rs                  # REFERENCE: test_reset_help() test
└── plan/
    ├── P1M4T1S1/
    │   └── PRP.md                    # CONTRACT: Behavior implemented in previous subtask
    └── docs/
        └── fix_specifications.md     # REFERENCE: Fix specification
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── src/
    └── cli/
        └── args.rs                   # MODIFIED: Line 87 help text updated
            # BEFORE:
            # /// Skip confirmation prompt for destructive operations
            # #[arg(long, short = 'f')]
            # pub force: bool,
            #
            # AFTER:
            # /// Skip confirmation prompt and bypass detached state validation (use for recovery)
            # #[arg(long, short = 'f')]
            # pub force: bool,
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: This is a doc comment (///), not a regular comment (//)
// Clap derives help text from doc comments above struct fields.
// Must preserve the triple-slash format.

// GOTCHA: Help text positioning
// The doc comment MUST be immediately before the field definition.
// No blank lines allowed between comment and field.

// CORRECT:
// /// Skip confirmation prompt and bypass detached state validation (use for recovery)
// #[arg(long, short = 'f')]
// pub force: bool,

// INCORRECT:
// /// Skip confirmation prompt and bypass detached state validation (use for recovery)
//
// #[arg(long, short = 'f')]
// pub force: bool,

// GOTCHA: Line length considerations
// While Rust has no line length limit, very long help text may wrap poorly
// in terminal output. However, the new text (~80 chars) is reasonable.

// PATTERN: Single-line help text
// Most flags in the codebase use single-line help text.
// Multi-line help text is used for commands with extensive documentation
// (e.g., completion command with installation instructions).
// For --force flag, single-line is appropriate.

// PATTERN: Help text structure
// Common patterns observed:
// 1. Action verb phrase: "Skip confirmation prompt"
// 2. AND conjunction: "and bypass detached state validation"
// 3. Parenthetical context: "(use for recovery)"
// This structure provides WHAT, AND WHAT, and WHEN/WHY.

// GOTCHA: Test doesn't validate help text content
// The existing test_reset_help only checks that "--force" appears in output.
// It doesn't validate the specific help text.
// Manual verification (jin reset --help) is recommended.

// CRITICAL: No behavior change
// This is documentation-only. The --force flag behavior doesn't change.
// Only the help text changes to accurately describe existing behavior.

// CRITICAL: Previous PRP (P1.M4.T1.S1) implemented the behavior
// This PRP (P1.M4.T2.S1) updates documentation to match that behavior.
// The help text must describe what the code NOW does (after P1.M4.T1.S1).

// GOTCHA: Clap derive API behavior
// With clap's derive API, the doc comment automatically becomes the help text.
// No need to specify .help() method manually.
// Just changing the doc comment is sufficient.

// PATTERN: Parenthetical guidance in help text
// While not common in flag help text in this codebase, it's a valid pattern.
// Similar patterns found in error messages and other documentation.
// The "(use for recovery)" provides clear guidance on when to use the flag.
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is a documentation-only change:
- `ResetArgs` struct structure remains unchanged
- Only the doc comment (help text) for the `force` field changes

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/cli/args.rs FORCE FIELD HELP TEXT
  - FILE: src/cli/args.rs
  - LOCATION: Line 87 - the doc comment above the force field
  - CURRENT TEXT:
    ```rust
    /// Skip confirmation prompt for destructive operations
    ```
  - NEW TEXT:
    ```rust
    /// Skip confirmation prompt and bypass detached state validation (use for recovery)
    ```
  - PRESERVE:
    - The triple-slash doc comment format (///)
    - The field attribute #[arg(long, short = 'f')]
    - The field definition pub force: bool
  - DEPENDENCIES: None (this is the only change needed)

Task 2: VERIFY CODE COMPILES
  - RUN: cargo check
  - EXPECTED: Zero errors, zero warnings
  - DEPENDENCIES: Task 1

Task 3: RUN EXISTING TESTS
  - RUN: cargo test
  - EXPECTED: All tests pass (no test changes needed)
  - DEPENDENCIES: Task 1
  - NOTE: test_reset_help verifies --help output, doesn't validate specific help text

Task 4: MANUAL VERIFICATION (Recommended)
  - RUN: cargo build --release
  - RUN: ./target/release/jin reset --help
  - VERIFY: Help text shows updated text for --force
  - DEPENDENCIES: Task 1, Task 2
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE CHANGE ==================

// FILE: src/cli/args.rs
// LOCATION: Line 87 (doc comment above force field)

// --- BEFORE (CURRENT CODE) ---
/// Skip confirmation prompt for destructive operations
#[arg(long, short = 'f')]
pub force: bool,

// --- AFTER (UPDATED CODE) ---
/// Skip confirmation prompt and bypass detached state validation (use for recovery)
#[arg(long, short = 'f')]
pub force: bool,

// ================== WHY THIS CHANGE ==================
//
// P1.M4.T1.S1 modified the reset command logic so that --force skips BOTH:
// 1. Confirmation prompt (existing behavior)
// 2. Workspace attachment validation (NEW behavior)
//
// The old help text only mentioned #1.
// The new help text mentions BOTH #1 and #2.
//
// This ensures users understand that --force provides a recovery path
// when the workspace is in a detached state.

// ================== HELP TEXT STRUCTURE ==================
//
// The new help text has three parts:
//
// Part 1: "Skip confirmation prompt"
//   - What: Describes the first behavior (existing)
//   - Clear action: "Skip"
//   - Specific target: "confirmation prompt"
//
// Part 2: "and bypass detached state validation"
//   - What: Describes the second behavior (NEW from P1.M4.T1.S1)
//   - Clear action: "bypass"
//   - Specific target: "detached state validation"
//
// Part 3: "(use for recovery)"
//   - Why: Provides usage context
//   - Parenthetical: Separates guidance from description
//   - Actionable: Tells user when to use this flag
//
// This structure follows CLI help text best practices:
// - Describe WHAT the flag does
// - Describe WHEN/WHY to use it
// - Be concise but complete

// ================== ALTERNATIVES CONSIDERED ==================
//
// Alternative 1 (more concise):
//   "Skip confirmation and validation (recovery mode)"
//   - Too vague: doesn't specify WHAT validation
//
// Alternative 2 (more verbose):
//   "Skip confirmation prompt for destructive operations and bypass
//    workspace attachment validation for recovery purposes"
//   - Too long: wraps poorly in terminal output
//
// Alternative 3 (multi-line):
//   /// Skip confirmation prompt and bypass detached state validation
//   ///
//   /// Use this flag to recover from a detached workspace state.
//   - Unnecessary complexity: single-line is sufficient
//
// SELECTED: "Skip confirmation prompt and bypass detached state validation (use for recovery)"
// - Clear: describes both behaviors
// - Concise: fits on one line
// - Actionable: provides usage context

// ================== CONSISTENCY WITH OTHER FLAGS ==================
//
// Apply --force: "Force apply even if workspace is dirty"
//   - Describes bypass condition: "even if workspace is dirty"
//
// Reset --force (old): "Skip confirmation prompt for destructive operations"
//   - Describes what is skipped: "confirmation prompt"
//
// Reset --force (new): "Skip confirmation prompt and bypass detached state validation (use for recovery)"
//   - Describes both: "confirmation prompt" AND "detached state validation"
//   - Adds context: "(use for recovery)"
//
// The new text is longer but more accurate and informative.
// Users need to know about the validation bypass (the key new feature).

// ================== USER EXPERIENCE CONSIDERATIONS ==================
//
// BEFORE: User sees "Workspace is in a detached state" error
//         User runs: jin reset --help
//         User sees: "Skip confirmation prompt for destructive operations"
//         User thinks: "That doesn't help me - I'm not being prompted, I'm getting an error"
//         User gives up or searches for solutions
//
// AFTER: User sees "Workspace is in a detached state" error
//        User runs: jin reset --help
//        User sees: "Skip confirmation prompt and bypass detached state validation (use for recovery)"
//        User thinks: "Ah, --force bypasses validation! I can use this to recover"
//        User runs: jin reset --hard --force
//        User recovers successfully
//
// The improved help text directly addresses the user's problem.

// ================== TERMINAL OUTPUT EXAMPLE ==================
//
// Before change:
// $ jin reset --help
// Reset staged or committed changes
//
// Usage:
//   jin reset [OPTIONS]
//
// Options:
//   -f, --force
//           Skip confirmation prompt for destructive operations
//
// After change:
// $ jin reset --help
// Reset staged or committed changes
//
// Usage:
//   jin reset [OPTIONS]
//
// Options:
//   -f, --force
//           Skip confirmation prompt and bypass detached state validation (use for recovery)
```

### Integration Points

```yaml
HELP_TEXT_DISPLAY:
  - mechanism: Clap derive API uses doc comments as help text
  - trigger: User runs "jin reset --help" or "jin reset -h"
  - display: The updated help text appears in the --help output
  - location: Under OPTIONS section, next to -f, --force

BEHAVIOR_CONTRACT:
  - defined_by: P1.M4.T1.S1 PRP
  - behavior: --force skips both confirmation AND validation
  - this_change: Documentation only - makes help text match behavior
  - verification: Manual test with "jin reset --hard --force" in detached state

EXISTING_TEST:
  - test: test_reset_help in tests/cli_reset.rs
  - behavior: Verifies --help output contains "--force" (flag name)
  - impact: Test continues to pass (doesn't validate help text content)
  - no_changes: No test modifications needed
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after making the documentation change
cargo check                              # Type checking - MUST pass
cargo fmt -- --check                     # Format check - should pass

# Expected: Zero errors, zero warnings
# Documentation changes typically don't cause compilation errors
```

### Level 2: Help Text Verification

```bash
# Build the CLI to test help output
cargo build --release

# Verify the help text displays correctly
./target/release/jin reset --help

# Expected output includes:
# OPTIONS:
#     -f, --force
#             Skip confirmation prompt and bypass detached state validation (use for recovery)

# Verify the exact text:
./target/release/jin reset --help | grep -A 1 "force"

# OR use assert_cmd-style test (similar to existing test pattern):
cargo run -- reset --help | grep "bypass detached state validation"

# Expected: The updated help text is displayed
```

### Level 3: Test Execution (Component Validation)

```bash
# Run all reset tests
cargo test reset -- --nocapture

# Run the help test specifically
cargo test test_reset_help -- --nocapture

# Run full test suite to ensure no regressions
cargo test

# Expected: All tests pass
# Note: test_reset_help checks for "--force" in output, which still appears
```

### Level 4: Manual Verification (User Experience Validation)

```bash
# Verify help output is readable and informative
./target/release/jin reset --help

# Check that:
# 1. Help text is not truncated
# 2. Help text is not wrapped poorly
# 3. Both behaviors are mentioned (confirmation AND validation)
# 4. Usage context is provided (recovery)

# Verify the flag still works as expected
# (This tests the behavior from P1.M4.T1.S1, not the documentation)
cd $(mktemp -d)
export JIN_DIR=$(pwd)/.jin_global
git init
jin init

# Create a scenario where --force is needed
# ... (setup steps from P1.M4.T1.S1 manual verification)

# Verify --force flag works
jin reset --hard --force

# Expected: Command succeeds (behavior from P1.M4.T1.S1)
# The help text update doesn't change behavior, only documents it
```

### Level 5: Cross-Reference Validation

```bash
# Verify help text matches actual behavior
# (Read the code to confirm --force does what the help text says)

# Check reset.rs to confirm --force skips validation:
grep -A 5 "if !args.force" src/commands/reset.rs

# Expected: Code shows validation inside "if !args.force" block
# This confirms the help text is accurate

# Check workspace.rs to understand what validation is skipped:
grep -A 20 "fn validate_workspace_attached" src/staging/workspace.rs

# Expected: Function shows what "detached state validation" means
# This confirms the terminology in help text is accurate
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test reset` all tests pass
- [ ] `cargo test` all tests pass (no regressions)
- [ ] Help text displays correctly: `./target/release/jin reset --help`

### Feature Validation

- [ ] Help text change follows exact specification in Implementation Blueprint
- [ ] Help text at line 87 is updated to the new text
- [ ] Help text mentions both "confirmation prompt" AND "detached state validation"
- [ ] Help text includes "(use for recovery)" guidance
- [ ] Help text accurately describes behavior implemented in P1.M4.T1.S1

### Code Quality Validation

- [ ] Doc comment format preserved (triple-slash ///)
- [ ] Field definition unchanged (only doc comment changed)
- [ ] No behavior changes (documentation-only change)
- [ ] Help text length is reasonable (~80 chars)

### Documentation & Deployment

- [ ] Help text is clear and actionable
- [ ] Help text provides usage context
- [ ] Help text follows existing patterns in codebase
- [ ] Ready for P1.M4.T3.S1 (test addition for detached state behavior)

---

## Anti-Patterns to Avoid

- **Don't** change the `ResetArgs` struct definition - only the doc comment
- **Don't** modify the `#[arg(long, short = 'f')]` attribute - it's correct
- **Don't** add new fields or imports - this is documentation-only
- **Don't** modify the `reset.rs` logic - behavior changed in P1.M4.T1.S1
- **Don't** create multi-line help text - single-line is appropriate here
- **Don't** remove "confirmation prompt" from help text - both behaviors must be mentioned
- **Don't** forget the "(use for recovery)" context - provides critical usage guidance
- **Don't** use regular comment (//) - must be doc comment (///)
- **Don't** add blank lines between comment and field - breaks clap derive
- **Don't** modify tests - test_reset_help will pass with new help text

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single-line change**: Only the doc comment text changes
- **Exact specification**: Current text and target text are both clearly specified
- **No behavior change**: Pure documentation update, no code logic changes
- **Low risk**: Documentation changes don't break compilation or tests
- **Clear context**: Research shows exact patterns to follow
- **Test verification**: Simple manual verification with `jin reset --help`

**Zero Risk Factors**:
- No logic changes - only doc comment text
- No new dependencies or imports
- Change is reversible (can revert to old text)
- Existing tests verify the build works
- No test modifications needed

**Current Status**: Ready for implementation - all context gathered, exact change specified, verification steps defined

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from this PRP creation):
- `plan/P1M4T2S1/research/` - Directory for all research findings
  - Agent research files: a228124 (ResetArgs location), a9f8961 (clap patterns), a6a24b7 (test patterns)

**Related PRP** (Contract to fulfill):
- `plan/P1M4T1S1/PRP.md` - Implements the behavior this help text must describe

**Code Files**:
- `src/cli/args.rs` - File to modify (line 87)
- `src/commands/reset.rs` - Behavior implementation (reference only)
- `tests/cli_reset.rs` - Help test (reference only)

**External Documentation**:
- [Clap Documentation](https://docs.rs/clap/latest/clap/_derive/index.html) - Doc comments
- [CLI Guidelines](https://clig.dev/) - Help text best practices
- [Git reset docs](https://git-scm.com/docs/git-reset) - Comparison reference
