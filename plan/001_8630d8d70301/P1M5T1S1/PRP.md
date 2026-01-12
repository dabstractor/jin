# PRP: P1.M5.T1.S1 - Update --project help text in AddArgs

---

## Goal

**Feature Goal**: Update the help text for the `--project` flag in the `add` command to accurately clarify that it targets Layer 5 (ModeProject) and requires `--mode`, while distinguishing it from Layer 7 (ProjectBase) which is accessed via `jin add <file>` without any flags.

**Deliverable**: A single-line documentation change in `src/cli/args.rs` that updates the help attribute for the `project` field in the `AddArgs` struct.

**Success Definition**:
- `jin add --help` displays the updated help text for `--project`
- Help text clearly distinguishes between Layer 5 (ModeProject with --project + --mode) and Layer 7 (ProjectBase without flags)
- Help text accurately explains the `--mode` requirement for `--project`
- All existing tests pass
- The updated help text follows existing patterns in the codebase

---

## User Persona

**Target User**: Jin users who need to understand which layer the `--project` flag targets and when to use it versus using no flags at all.

**Use Case**: User runs `jin add --help` and wants to understand:
1. What layer does `--project` target?
2. Why does `--project` require `--mode`?
3. How do I access the project-base layer (Layer 7)?

**User Journey**:
1. User tries `jin add --project config.json` and gets error "--project requires --mode"
2. User runs `jin add --help` to understand the layer system
3. User sees clear help text explaining that `--project` is for Layer 5 (requires --mode)
4. User sees that Layer 7 (project-base) is accessed without flags
5. User understands the distinction and can choose the correct approach

**Pain Points Addressed**:
- **Before**: Help text only said "Target project layer" - users couldn't distinguish between Layer 5 and Layer 7
- **After**: Help text explicitly states "Layer 5 (requires --mode)" and contrasts with "Layer 7 (use without flags)"
- **Confusion resolved**: The identified_issues.md notes PRD confusion about --project accessing Layer 7 - this clarification fixes that

---

## Why

- **Problem**: The current help text for `--project` says "Target project layer" which is ambiguous. Jin has TWO project layers: Layer 5 (ModeProject) and Layer 7 (ProjectBase). Users cannot determine which layer `--project` targets or when to use it.

- **Documentation Confusion**: As noted in `plan/docs/identified_issues.md`, there was confusion in the PRD about `--project` accessing Layer 7. The help text must clarify that:
  - `--project` + `--mode` → Layer 5 (ModeProject)
  - No flags → Layer 7 (ProjectBase)

- **User Impact**: Users trying to add files to the project-base layer (Layer 7) might incorrectly use `--project` flag, then get the confusing error "--project requires --mode". Clear help text prevents this confusion.

- **Integration**: This is part of P1.M5 (Documentation and Clarification Updates). It improves UX without changing any behavior - only the documentation becomes clearer.

- **Code Quality**: Accurate documentation is critical for user experience. The layer system is complex; help text should reduce confusion, not add to it.

---

## What

### User-Visible Behavior

**Current Help Text** (confusing - doesn't specify which layer):
```bash
$ jin add --help
...
OPTIONS:
    --mode
            Target mode layer
    --scope <SCOPE>
            Target scope layer
    --project
            Target project layer
    --global
            Target global layer
    --local
            Target user-local layer (Layer 8, machine-specific)
...
```

**Desired Help Text** (clear - specifies Layer 5, requires --mode, distinguishes from Layer 7):
```bash
$ jin add --help
...
OPTIONS:
    --mode
            Target mode layer
    --scope <SCOPE>
            Target scope layer
    --project
            Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
    --global
            Target global layer
    --local
            Target user-local layer (Layer 8, machine-specific)
...
```

### Technical Requirements

1. **Modify `src/cli/args.rs`** (line 19):
   - Current: `/// Target project layer`
   - New: `/// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags`

2. **Preserve existing behavior**:
   - The `--project` flag behavior doesn't change (only the help text changes)
   - Flag attributes `#[arg(long)]` remain unchanged
   - Validation that `--project` requires `--mode` remains unchanged

3. **No new files or dependencies**: This is a documentation-only change

### Success Criteria

- [ ] `jin add --help` displays the updated help text for `--project`
- [ ] Help text explicitly mentions "Layer 5" and "requires --mode"
- [ ] Help text provides guidance for Layer 7: "use: jin add <file> without flags"
- [ ] All existing tests pass
- [ ] Help text format matches existing patterns in the codebase

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the exact line number to modify, the current help text, the desired help text, examples of similar help text patterns in the codebase, layer routing context from identified_issues.md, test patterns to verify the change, and external research references._

### Documentation & References

```yaml
# ISSUE IDENTIFICATION: Root cause of this documentation update
- file: /home/dustin/projects/jin/plan/docs/identified_issues.md
  why: Documents the confusion about --project accessing Layer 7
  section: "Issue #4: `--project` Flag Without Mode Fails Cryptically"
  critical: |
    ISSUE #4 EXCERPT:
    "Current State:
    - `jin add --project` fails with 'Configuration error: --project requires --mode flag'
    - This is correct behavior but test report suggests confusion about accessing Layer 7 directly

    Clarification:
    - Layer 7 (ProjectBase) IS accessible via `jin add <file>` (no flags) - this is the default
    - `--project` flag is for Layer 5 (ModeProject) and requires `--mode`
    - Documentation may need clarification"

    This issue is exactly what our help text update addresses.

# IMPLEMENTATION: Current AddArgs project field (MUST MODIFY)
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: This is the file to modify - line 19 contains the help text to update
  section: "Lines 6-30: AddArgs struct"
  current_code: |
    /// Target project layer
    #[arg(long)]
    pub project: bool,
  new_code: |
    /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
    #[arg(long)]
    pub project: bool,
  location: "Line 19 - the doc comment above the project field"
  gotcha: |
    CRITICAL: This is a doc comment (///), NOT a regular comment.
    Doc comments are what clap uses for help text.
    Must preserve the triple-slash format.

# PATTERN: Help text update example from P1.M4.T2.S1
- file: /home/dustin/projects/jin/plan/P1M4T2S1/PRP.md
  why: Shows the exact pattern for updating help text in clap Args structs
  section: "Implementation Blueprint - Implementation Patterns & Key Details"
  pattern: |
    Single-line doc comment update:
    BEFORE: /// Skip confirmation prompt for destructive operations
    AFTER:  /// Skip confirmation prompt and bypass detached state validation (use for recovery)
  critical: |
    The process is identical:
    1. Change only the doc comment (///)
    2. Preserve field attributes and definition
    3. No behavior changes, only documentation
    4. Manual verification with --help command

# PATTERN: Layer specification in --local help text
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: Shows how to specify layer numbers in help text
  section: "Line 27-29: local field"
  code: |
    /// Target user-local layer (Layer 8, machine-specific)
    #[arg(long)]
    pub local: bool,
  note: |
    The --local help text includes "(Layer 8, machine-specific)" which is the pattern
    we follow for specifying layer numbers. Our new text includes "(Layer 5, requires --mode)".

# REFERENCE: Layer routing table (for understanding)
- file: /home/dustin/projects/jin/plan/docs/system_context.md
  why: Complete 9-layer system architecture and routing rules
  section: "Layer Routing (jin add flags)" table
  critical: |
    LAYER ROUTING TABLE:
    --mode           → Layer 3 (ModeBase)
    --mode --scope   → Layer 4 (ModeScope)
    --mode --project → Layer 5 (ModeProject)  ← This is what --project targets
    --global         → Layer 6 (Global)
    (no flags)       → Layer 7 (ProjectBase)  ← This is the default, not --project
    --local          → Layer 8 (UserLocal)

    This confirms: --project flag is ONLY for Layer 5, and it requires --mode.

# CONTEXT: Validation that enforces --mode requirement
- file: /home/dustin/projects/jin/src/staging/router.rs
  why: Shows the validation logic that --project requires --mode
  section: "Validation logic in route_to_layer()"
  pattern: |
    When --project is used without --mode:
    → Error: "Configuration error: --project requires --mode flag"
  note: |
    The help text must explain this requirement so users understand WHY they get this error.

# PATTERN: Multi-part help text in codebase
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: Shows how to write help text with multiple parts
  section: "ResetArgs force field (line 87)"
  code: |
    /// Skip confirmation prompt and bypass detached state validation (use for recovery)
  note: |
    This shows that help text can have multiple clauses separated by punctuation.
    Our text follows this pattern: "Add to mode-project layer (Layer 5, requires --mode).
    For project-base layer (Layer 7), use: jin add <file> without flags"

# PATTERN: Examples in help text (command examples)
- file: /home/dustin/projects/jin/src/cli/mod.rs
  why: Shows how to include command examples in help text
  section: "Lines 111-119: Completion command with installation examples"
  pattern: |
    /// Generate shell completion scripts
    ///
    /// Outputs completion script to stdout...
    /// Installation:
    ///   Bash:       jin completion bash > ...
  note: |
    While we don't need multi-line help text, the pattern of including command
    examples (like "jin add <file> without flags") is valid and helpful.

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

- url: https://docs.rs/clap/latest/clap/struct.Arg.html#method.help
  why: Arg struct documentation for help attribute
  section: "help() method"
  critical: |
    "The help text for the arg. This is typically a short, one-line message."
    Our text is longer (~110 chars) but acceptable because it provides
    critical clarification to prevent user confusion.

# EXTERNAL RESEARCH: CLI help text best practices
- url: https://clig.dev/#help
  why: Command Line Interface Guidelines - help text best practices
  section: "Help Text"
  critical: |
    "Help text should be concise but complete. Use examples to clarify usage.
    When a flag has a common confusion point, address it directly."
    Our text addresses the Layer 5 vs Layer 7 confusion directly.

# EXTERNAL RESEARCH: Git flag help text comparison
- url: https://git-scm.com/docs/git-add
  why: Git add documentation for comparison
  section: "-u, --update flag"
  critical: |
    Git uses concise help like "Update the index" but sometimes adds context
    like "only matches tracked files". Our text provides similar context:
    "requires --mode" and "use: jin add <file> without flags".

# TEST: Existing add command help test (if exists)
- file: /home/dustin/projects/jin/tests/cli_add*.rs
  why: May contain tests for add command help output
  note: |
    Based on P1.M4.T2.S1 pattern, existing tests likely check for flag presence
    (--project appears in output) but not help text content.
    No test changes needed for this subtask.

# TEST: General help text testing patterns
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
  critical: |
    Pattern: Use predicate::str::contains() to check for specific text in help output.
    Could add test for new help text if desired, but not required for this subtask.

# CONTEXT: Similar help text updates in P1.M4.T2.S1
- file: /home/dustin/projects/jin/plan/P1M4T2S1/PRP.md
  why: Shows that help text updates are documentation-only changes
  section: "Goal" and "Implementation Blueprint"
  pattern: |
    Goal: "Update the help text for the --force flag to accurately reflect..."
    Deliverable: "A single-line documentation change in src/cli/args.rs"
  note: |
    This confirms our approach: single-line doc comment change, no behavior changes,
    existing tests pass, manual verification with --help command.
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   └── args.rs                    # MODIFY: Line 19 - project field help text
│   ├── staging/
│   │   └── router.rs                  # REFERENCE: Layer routing logic
│   └── commands/
│       └── add.rs                     # REFERENCE: Uses AddArgs
├── tests/
│   └── cli_*.rs                       # REFERENCE: Existing help tests
└── plan/
    ├── docs/
    │   ├── identified_issues.md       # ISSUE: Confusion about --project and Layer 7
    │   └── system_context.md          # REFERENCE: 9-layer system architecture
    └── P1M4T2S1/
        └── PRP.md                     # PATTERN: Similar help text update
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── src/
    └── cli/
        └── args.rs                   # MODIFIED: Line 19 help text updated
            # BEFORE:
            # /// Target project layer
            # #[arg(long)]
            # pub project: bool,
            #
            # AFTER:
            # /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
            # #[arg(long)]
            # pub project: bool,
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
// /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
// #[arg(long)]
// pub project: bool,

// INCORRECT:
// /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
//
// #[arg(long)]
// pub project: bool,

// GOTCHA: Help text length
// The new help text is ~110 characters. While Rust has no line length limit,
// very long help text may wrap in terminal output. However, this length
// is acceptable for the clarity it provides (resolves Layer 5 vs Layer 7 confusion).

// PATTERN: Two-sentence help text
// Most flags in the codebase use single-line help text.
// Our text uses a period to separate two clauses:
// 1. What --project does: "Add to mode-project layer (Layer 5, requires --mode)"
// 2. How to access Layer 7: "For project-base layer (Layer 7), use: jin add <file> without flags"
// This two-part structure is necessary to clarify the common confusion.

// CRITICAL: No behavior change
// This is documentation-only. The --project flag behavior doesn't change.
// Only the help text changes to accurately describe existing behavior.
// The validation that --project requires --mode remains unchanged.

// CRITICAL: Layer 7 is the DEFAULT, not --project
// This is the key confusion to address:
// - Layer 7 (ProjectBase) = DEFAULT behavior (no flags needed)
// - Layer 5 (ModeProject) = --project + --mode flags
// Users might think --project accesses Layer 7, but it doesn't.

// PATTERN: Including command examples in help text
// The phrase "use: jin add <file> without flags" is a mini-example.
// This follows the pattern seen in completion command help text.
// Examples in help text are valuable for clarifying usage.

// GOTCHA: Terminal width considerations
// At standard 80-character terminal width, this help text will wrap.
// This is acceptable because wrapping is preferable to user confusion.
// The alternative (shorter text) would leave the Layer 5 vs Layer 7 confusion unresolved.

// CRITICAL: Clap derive API behavior
// With clap's derive API, the doc comment automatically becomes the help text.
// No need to specify .help() method manually.
// Just changing the doc comment is sufficient.

// REFERENCE: Similar help text complexity in ResetArgs --force
// The reset --force help text was updated to:
// "Skip confirmation prompt and bypass detached state validation (use for recovery)"
// This is also ~85 characters and includes two clauses.
// Our --project text follows this pattern of comprehensive help text.
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is a documentation-only change:
- `AddArgs` struct structure remains unchanged
- Only the doc comment (help text) for the `project` field changes

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/cli/args.rs PROJECT FIELD HELP TEXT
  - FILE: src/cli/args.rs
  - LOCATION: Line 19 - the doc comment above the project field
  - CURRENT TEXT:
    ```rust
    /// Target project layer
    ```
  - NEW TEXT:
    ```rust
    /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
    ```
  - PRESERVE:
    - The triple-slash doc comment format (///)
    - The field attribute #[arg(long)]
    - The field definition pub project: bool
  - DEPENDENCIES: None (this is the only change needed)

Task 2: VERIFY CODE COMPILES
  - RUN: cargo check
  - EXPECTED: Zero errors, zero warnings
  - DEPENDENCIES: Task 1

Task 3: RUN EXISTING TESTS
  - RUN: cargo test
  - EXPECTED: All tests pass (no test changes needed)
  - DEPENDENCIES: Task 1
  - NOTE: Tests check for flag presence, not help text content

Task 4: MANUAL VERIFICATION (Recommended)
  - RUN: cargo build --release
  - RUN: ./target/release/jin add --help
  - VERIFY: Help text shows updated text for --project
  - DEPENDENCIES: Task 1, Task 2
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE CHANGE ==================

// FILE: src/cli/args.rs
// LOCATION: Line 19 (doc comment above project field)

// --- BEFORE (CURRENT CODE) ---
/// Target project layer
#[arg(long)]
pub project: bool,

// --- AFTER (UPDATED CODE) ---
/// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
#[arg(long)]
pub project: bool,

// ================== WHY THIS CHANGE ==================
//
// PROBLEM: Current help text "Target project layer" is ambiguous.
// Jin has TWO project layers:
// - Layer 5: ModeProject (requires --mode + --project flags)
// - Layer 7: ProjectBase (default, no flags needed)
//
// Users can't tell which layer --project targets, leading to confusion:
// 1. User tries: jin add --project config.json
// 2. Error: "--project requires --mode flag"
// 3. User is confused: "I just want to add to the project layer, why do I need --mode?"
// 4. Root cause: User thinks --project targets Layer 7 (the project-base layer)
// 5. Reality: --project targets Layer 5 (the mode-project layer), which requires --mode
//
// SOLUTION: Clarify in help text that:
// - --project is for Layer 5 (ModeProject)
// - It requires --mode
// - Layer 7 (ProjectBase) is accessed without flags

// ================== HELP TEXT STRUCTURE ==================
//
// The new help text has two parts separated by a period:
//
// Part 1: "Add to mode-project layer (Layer 5, requires --mode)"
//   - WHAT: Describes what --project does
//   - WHICH LAYER: Explicitly states "Layer 5"
//   - LAYER NAME: "mode-project layer" distinguishes from "project-base layer"
//   - REQUIREMENT: "requires --mode" explains why --mode is needed
//
// Part 2: "For project-base layer (Layer 7), use: jin add <file> without flags"
//   - ALTERNATIVE: Explains how to access Layer 7 (the common confusion point)
//   - COMMAND EXAMPLE: "jin add <file> without flags" shows the correct usage
//   - DISTINCTION: Clarifies that --project is NOT for Layer 7
//
// This structure follows CLI help text best practices:
// - Describe WHAT the flag does
// - Describe WHEN/WHY to use it (and when NOT to)
// - Provide examples to clarify usage
// - Address common confusion points directly

// ================== ALTERNATIVES CONSIDERED ==================
//
// Alternative 1 (more concise):
//   "Add to mode-project layer (Layer 5, requires --mode). Layer 7 is the default"
//   - Less clear: doesn't explain HOW to use Layer 7
//
// Alternative 2 (multi-line):
//   /// Add to mode-project layer (Layer 5, requires --mode).
//   ///
//   /// For project-base layer (Layer 7), use: jin add <file> without flags
//   - Unnecessary complexity: single-line with period separator is sufficient
//
// Alternative 3 (referencing documentation):
//   "Add to mode-project layer (Layer 5, requires --mode). See docs for layer details"
//   - Less helpful: requires user to look up documentation
//
// SELECTED: "Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags"
// - Clear: describes both layers
// - Concise: fits in help output
// - Actionable: provides command example
// - Directly addresses the confusion from identified_issues.md

// ================== CONSISTENCY WITH OTHER FLAGS ==================
//
// --local: "Target user-local layer (Layer 8, machine-specific)"
//   - Pattern: "Target X layer (Layer N, description)"
//   - Our text: "Add to mode-project layer (Layer 5, requires --mode)"
//   - Slight variation: "Add to" instead of "Target" (more action-oriented)
//
// --global: "Target global layer"
//   - Simple: No layer number or details
//   - Our text: More verbose because --project has special requirements
//
// --force (in reset): "Skip confirmation prompt and bypass detached state validation (use for recovery)"
//   - Pattern: Two clauses separated by "and", parenthetical guidance
//   - Our text: Similar pattern with period separator and example
//
// The --project text is longer than others but necessary to resolve the
// Layer 5 vs Layer 7 confusion documented in identified_issues.md.

// ================== USER EXPERIENCE IMPACT ==================
//
// BEFORE:
// User tries: jin add --project config.json
// Error: "--project requires --mode flag"
// User runs: jin add --help
// Sees: "--project    Target project layer"
// User thinks: "I AM targeting the project layer! Why do I need --mode?"
// User gives up or tries wrong things
//
// AFTER:
// User tries: jin add --project config.json
// Error: "--project requires --mode flag"
// User runs: jin add --help
// Sees: "--project    Add to mode-project layer (Layer 5, requires --mode).
//                 For project-base layer (Layer 7), use: jin add <file> without flags"
// User thinks: "Ah, --project is for Layer 5, and it requires --mode.
//               For Layer 7, I just use jin add without flags!"
// User runs: jin add config.json  (for Layer 7)
// OR
// User runs: jin add config.json --mode --project  (for Layer 5)
// User succeeds!

// ================== TERMINAL OUTPUT EXAMPLE ==================
//
// Before change:
// $ jin add --help
// Add files to the Jin layer system
//
// Usage:
//   jin add [OPTIONS] <FILES>...
//
// Arguments:
//   <FILES>...    Files to stage
//
// Options:
//       --mode
//               Target mode layer
//       --scope <SCOPE>
//               Target scope layer
//       --project
//               Target project layer
//       --global
//               Target global layer
//       --local
//               Target user-local layer (Layer 8, machine-specific)
//
// After change:
// $ jin add --help
// Add files to the Jin layer system
//
// Usage:
//   jin add [OPTIONS] <FILES>...
//
// Arguments:
//   <FILES>...    Files to stage
//
// Options:
//       --mode
//               Target mode layer
//       --scope <SCOPE>
//               Target scope layer
//       --project
//               Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
//       --global
//               Target global layer
//       --local
//               Target user-local layer (Layer 8, machine-specific)
```

### Integration Points

```yaml
HELP_TEXT_DISPLAY:
  - mechanism: Clap derive API uses doc comments as help text
  - trigger: User runs "jin add --help" or "jin add -h"
  - display: The updated help text appears in the --help output
  - location: Under OPTIONS section, next to --project

LAYER_ROUTING_CONTRACT:
  - defined_by: src/staging/router.rs route_to_layer() function
  - behavior: --project + --mode → Layer 5 (ModeProject)
  - behavior: no flags → Layer 7 (ProjectBase) [default]
  - this_change: Documentation only - makes help text match behavior
  - validation: --project without --mode produces error

EXISTING_TESTS:
  - pattern: Tests likely check for "--project" in help output
  - impact: Tests continue to pass (don't validate help text content)
  - no_changes: No test modifications needed

ISSUE_RESOLUTION:
  - issue: identified_issues.md Issue #4 - "--project Flag Without Mode Fails Cryptically"
  - resolution: Help text now explains Layer 5 vs Layer 7 distinction
  - impact: Reduces user confusion about --project behavior
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
./target/release/jin add --help

# Expected output includes:
# OPTIONS:
#     --project
#             Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags

# Verify the exact text:
./target/release/jin add --help | grep -A 2 "project"

# OR use grep to check for key phrases:
./target/release/jin add --help | grep "Layer 5"
./target/release/jin add --help | grep "requires --mode"
./target/release/jin add --help | grep "Layer 7"

# Expected: The updated help text is displayed with all key phrases
```

### Level 3: Test Execution (Component Validation)

```bash
# Run all add tests
cargo test add -- --nocapture

# Run any existing help tests
cargo test test_help -- --nocapture

# Run full test suite to ensure no regressions
cargo test

# Expected: All tests pass
# Note: Tests check for "--project" in output, which still appears
```

### Level 4: Manual Verification (User Experience Validation)

```bash
# Verify help output is readable and informative
./target/release/jin add --help

# Check that:
# 1. Help text is not truncated
# 2. Help text wraps reasonably (some wrapping expected at ~110 chars)
# 3. Both layers are mentioned (Layer 5 and Layer 7)
# 4. The --mode requirement is clear
# 5. The command example for Layer 7 is clear

# Verify the flag still works as expected (behavior unchanged)
cd $(mktemp -d)
export JIN_DIR=$(pwd)/.jin_global
git init
jin init

# Verify --project without --mode still fails with clear error
jin add config.json --project 2>&1 | grep "requires --mode"
# Expected: Error message appears (validation still works)

# Verify --project with --mode is accepted
jin mode create testmode
jin mode use testmode
jin add config.json --mode --project
# Expected: Command proceeds (behavior unchanged)

# Cleanup
cd -
rm -rf "$OLDPWD"
```

### Level 5: Cross-Reference Validation

```bash
# Verify help text matches actual behavior
# (Read the code to confirm --project does what the help text says)

# Check router.rs to confirm --project + --mode routing:
grep -A 5 "project" src/staging/router.rs | grep Layer::ModeProject

# Expected: Code shows --project + --mode → ModeProject layer
# This confirms the help text is accurate

# Check system_context.md to verify layer numbers:
grep -A 10 "Layer Routing" plan/docs/system_context.md | grep -E "Layer 5|Layer 7"

# Expected: Layer 5 = ModeProject, Layer 7 = ProjectBase
# This confirms the layer numbers in help text are accurate
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test add` all tests pass
- [ ] `cargo test` all tests pass (no regressions)
- [ ] Help text displays correctly: `./target/release/jin add --help`

### Feature Validation

- [ ] Help text change follows exact specification in Implementation Blueprint
- [ ] Help text at line 19 is updated to the new text
- [ ] Help text mentions "Layer 5" and "requires --mode"
- [ ] Help text mentions "Layer 7" and "without flags"
- [ ] Help text includes command example: "jin add <file> without flags"
- [ ] Help text accurately describes behavior (Layer 5 vs Layer 7 distinction)

### Code Quality Validation

- [ ] Doc comment format preserved (triple-slash ///)
- [ ] Field definition unchanged (only doc comment changed)
- [ ] No behavior changes (documentation-only change)
- [ ] Help text is clear and actionable
- [ ] Help text addresses the confusion from identified_issues.md

### Documentation & Deployment

- [ ] Help text resolves Issue #4 from identified_issues.md
- [ ] Help text follows existing patterns in codebase
- [ ] Help text provides command example for Layer 7 usage
- [ ] Ready for P1.M5.T2.S1 (layer routing reference table)

---

## Anti-Patterns to Avoid

- **Don't** change the `AddArgs` struct definition - only the doc comment
- **Don't** modify the `#[arg(long)]` attribute - it's correct
- **Don't** add new fields or imports - this is documentation-only
- **Don't** modify the routing logic in `router.rs` - behavior unchanged
- **Don't** create multi-line help text - single-line with period separator is sufficient
- **Don't** remove the reference to Layer 7 - that's the key clarification
- **Don't** make the text too short - the Layer 5 vs Layer 7 explanation is necessary
- **Don't** use regular comment (//) - must be doc comment (///)
- **Don't** add blank lines between comment and field - breaks clap derive
- **Don't** modify tests - existing tests will pass with new help text

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single-line change**: Only the doc comment text changes
- **Exact specification**: Current text and target text are both clearly specified
- **No behavior change**: Pure documentation update, no code logic changes
- **Low risk**: Documentation changes don't break compilation or tests
- **Clear context**: Research shows exact patterns to follow from P1.M4.T2.S1
- **Test verification**: Simple manual verification with `jin add --help`
- **Addresses documented issue**: Directly resolves Issue #4 from identified_issues.md

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
- Codebase analysis of args.rs, identified_issues.md, system_context.md
- Pattern analysis from P1.M4.T2.S1 (similar help text update)
- Pattern analysis from P1.M1.T1.S1 (AddArgs modifications)

**Related PRP** (Pattern reference):
- `plan/P1M4.T2.S1/PRP.md` - Shows the exact pattern for updating help text

**Code Files**:
- `src/cli/args.rs` - File to modify (line 19)
- `src/staging/router.rs` - Layer routing behavior (reference only)
- `plan/docs/identified_issues.md` - Issue #4 being addressed
- `plan/docs/system_context.md` - Layer system reference

**External Documentation**:
- [Clap Documentation](https://docs.rs/clap/latest/clap/_derive/index.html) - Doc comments
- [CLI Guidelines](https://clig.dev/) - Help text best practices
- [Git add docs](https://git-scm.com/docs/git-add) - Comparison reference
