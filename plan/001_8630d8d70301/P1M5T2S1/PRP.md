# PRP: P1.M5.T2.S1 - Add after_help with Layer Routing Table to AddArgs

---

## Goal

**Feature Goal**: Add a `#[command(after_help = "...")]` attribute to the `AddArgs` struct that displays a formatted layer routing quick reference table when users run `jin add --help`.

**Deliverable**: A `#[command(after_help = "...")]` attribute added to the `AddArgs` struct in `src/cli/args.rs` containing a formatted layer routing table showing all flag combinations and their target layers.

**Success Definition**:
- `jin add --help` displays the layer routing table at the end of help output
- The table is readable and properly formatted for terminal display
- All 8 layer routing combinations are shown (from item description)
- The table shows: flag combination → target layer
- All existing tests pass
- The after_help text is clear and actionable

---

## User Persona

**Target User**: Jin users who need a quick reference for understanding which layer different flag combinations target when using `jin add`.

**Use Case**: User runs `jin add --help` and wants to see:
1. What layer does each flag combination target?
2. How do I access a specific layer?
3. What's the difference between using no flags vs using --mode, --scope, --project, --global, or --local?

**User Journey**:
1. User wants to add a file to a specific layer
2. User runs `jin add --help` to see layer options
3. User sees flag descriptions but wants a quick reference table
4. User sees the after_help layer routing table at the end
5. User quickly identifies the correct flag combination for their target layer
6. User runs `jin add <file> [flags]` with confidence

**Pain Points Addressed**:
- **Before**: Users had to memorize or look up layer routing in documentation
- **Before**: The relationship between flag combinations and target layers was not immediately visible in help
- **After**: Quick reference table appears directly in `jin add --help` output
- **After**: Users can see all routing options at a glance

---

## Why

- **Problem**: The layer system is complex with 9 layers and multiple flag combinations. Users currently have no quick reference in the CLI help output to understand which flags target which layers.

- **Documentation Gap**: While `plan/architecture/system_context.md` documents the layer routing table, this information is not accessible when users run `jin add --help`. Users must exit the CLI and look up documentation.

- **User Experience**: A quick reference table in the help output reduces cognitive load and helps users choose the correct flag combination without leaving the terminal.

- **Integration**: This is part of P1.M5 (Documentation and Clarification Updates). It improves UX without changing any behavior - only the help output becomes more comprehensive.

- **Precedent**: The `--local` flag (Layer 8) was recently implemented. A routing table helps users understand where this new flag fits in the layer system.

---

## What

### User-Visible Behavior

**Current Help Output**:
```bash
$ jin add --help
Add files to the Jin layer system

Usage:
  jin add [OPTIONS] <FILES>...

Arguments:
  <FILES>...    Files to stage

Options:
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

  -h, --help
          Print help information
```

**Desired Help Output** (with after_help table):
```bash
$ jin add --help
Add files to the Jin layer system

Usage:
  jin add [OPTIONS] <FILES>...

Arguments:
  <FILES>...    Files to stage

Options:
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

  -h, --help
          Print help information

LAYER ROUTING:
  Flags                  Target Layer
  ──────────────────────────────────────────────────────
  (no flags)             → Layer 7 (ProjectBase)
  --mode                 → Layer 2 (ModeBase)
  --mode --project       → Layer 5 (ModeProject)
  --scope=<X>            → Layer 6 (ScopeBase)
  --mode --scope=<X>     → Layer 3 (ModeScope)
  --mode --scope=<X> --project
                         → Layer 4 (ModeScopeProject)
  --global               → Layer 1 (GlobalBase)
  --local                → Layer 8 (UserLocal)
```

### Technical Requirements

1. **Add `#[command(after_help = "...")]` attribute** to `AddArgs` struct
2. **Use raw string syntax** (`r#"..."#`) for multi-line table formatting
3. **Include all 8 routing combinations** from the item description
4. **Preserve existing behavior** - no changes to struct fields or validation

### Success Criteria

- [ ] `jin add --help` displays the layer routing table after options
- [ ] Table is properly formatted and aligned
- [ ] All 8 routing combinations from item description are present
- [ ] All existing tests pass
- [ ] Format matches Rust/Clap conventions

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the exact code change location, the formatted table content, raw string syntax examples, Clap version compatibility, layer routing reference from system_context.md, and validation steps._

### Documentation & References

```yaml
# CONTRACT REFERENCE: Previous PRP (P1.M5.T1.S1) - Updates --project help text
- file: /home/dustin/projects/jin/plan/P1M5T1S1/PRP.md
  why: Defines the updated --project help text that will exist when this PRP starts
  context_scope: |
    The --project help text will be updated to:
    "Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags"
  note: |
    This PRP builds upon the clarified --project help text by adding the complete
    routing table. Users who see the --project clarification will now also see
    the full routing context in after_help.

# IMPLEMENTATION: AddArgs struct (MUST MODIFY)
- file: /home/dustin/projects/jin/src/cli/args.rs
  why: This is the file to modify - AddArgs struct at lines 5-30
  current_code: |
    /// Arguments for the `add` command
    #[derive(Args, Debug)]
    pub struct AddArgs {
        /// Files to stage
        pub files: Vec<String>,

        /// Target mode layer
        #[arg(long)]
        pub mode: bool,

        /// Target scope layer
        #[arg(long)]
        pub scope: Option<String>,

        /// Target project layer
        #[arg(long)]
        pub project: bool,

        /// Target global layer
        #[arg(long)]
        pub global: bool,

        /// Target user-local layer (Layer 8, machine-specific)
        #[arg(long)]
        pub local: bool,
    }
  modification: |
    Add #[command(after_help = "...")] attribute between #[derive(Args, Debug)] and struct definition
  location: "After line 6, before line 7"

# RESEARCH: Clap after_help attribute - Comprehensive guide
- file: /home/dustin/projects/jin/plan/P1M5T2S1/research/clap_after_help.md
  why: Complete research on Clap's after_help feature including syntax, examples, and gotchas
  critical: |
    KEY FINDINGS:
    - Clap 4.5 fully supports #[command(after_help = "...")] with derive API
    - Use raw strings r#"..."# for multi-line text
    - No existing after_help usage in codebase (new pattern)
    - Manual table formatting required (no built-in table support)
    - Examples from real projects: cargo-insta, czkawka

# REFERENCE: Layer routing table from system_context.md
- file: /home/dustin/projects/jin/plan/architecture/system_context.md
  why: Authoritative source for layer routing rules
  section: "Layer Routing (jin add flags) table"
  routing_table: |
    | Command                        | Target Layer     |
    |--------------------------------|------------------|
    | `jin add <file>`               | ProjectBase (7)  |
    | `jin add <file> --mode`        | ModeBase (2)     |
    | `jin add <file> --mode --project` | ModeProject (5) |
    | `jin add <file> --scope=<scope>`  | ScopeBase (6)   |
    | `jin add <file> --mode --scope=<scope>` | ModeScope (3) |
    | `jin add <file> --mode --scope=<scope> --project` | ModeScopeProject (4) |
    | `jin add <file> --global`      | GlobalBase (1)   |

# CONTRACT: Layer routing from item description
- source: Item description in task
  why: The authoritative specification for this work item
  required_combinations: |
    (no flags) → Layer 7
    --mode → Layer 2
    --mode --project → Layer 5
    --scope=X → Layer 6
    --mode --scope=X → Layer 3
    --mode --scope=X --project → Layer 4
    --global → Layer 1
    --local → Layer 8
  critical: |
    These 8 combinations MUST be included in the after_help table.
    Note: system_context.md table is missing --local (Layer 8) because it was
    written before --local was implemented. Use item description as source of truth.

# PATTERN: Command-level attributes on Cli struct
- file: /home/dustin/projects/jin/src/cli/mod.rs
  why: Shows existing pattern for using #[command(...)] attributes
  section: "Lines 1-20: Cli struct definition"
  pattern: |
    #[derive(Parser, Debug)]
    #[command(name = "jin")]
    #[command(author, version, about = "Phantom Git layer system...")]
    #[command(propagate_version = true)]
    pub struct Cli {
  note: |
    This shows the codebase already uses multi-line #[command(...)] blocks.
    Follow this pattern for after_help attribute.

# CLAP: Version and dependency information
- file: /home/dustin/projects/jin/Cargo.toml
  why: Confirms Clap version for API compatibility
  section: "Line 24: clap dependency"
  version: "clap = { version = \"4.5\", features = [\"derive\", \"cargo\"] }"
  compatible: true

# EXTERNAL RESEARCH: Clap derive API documentation
- url: https://docs.rs/clap/latest/clap/_derive/index.html
  why: Official documentation for derive macro attributes
  section: "Raw Attribute Documentation"
  critical: |
    "This allows users to access the raw behavior of an attribute via <attr>(<value>) syntax"
    Confirms that any Command builder method can be used as derive attribute.

- url: https://docs.rs/clap/latest/clap/builder/struct.Command.html#method.after_help
  why: Official documentation for after_help method
  section: "after_help() method"
  critical: |
    "Longer explanation to appear after the options when displaying the help information from --help or -h"

- url: https://github.com/clap-rs/clap/discussions/4090
  why: GitHub discussion confirming derive attribute usage
  critical: |
    Confirms that any builder method can be used as derive attribute.
    "So you can do #[command(after_help = \"\")]"

- url: https://github.com/clap-rs/clap/discussions/5203
  why: Additional confirmation of after_help usage
  quote: "The derive reference clarifies that any builder method may be used as a derive attribute"

# EXTERNAL RESEARCH: Rust raw string literals
- url: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals
  why: Official Rust documentation for raw string syntax
  critical: |
    Raw string literals: r#"..."# - Allows including special characters without escaping
    Essential for multi-line help text with \n characters and complex formatting

# CONTEXT: Validation layer routing logic
- file: /home/dustin/projects/jin/src/staging/router.rs
  why: Reference for understanding actual routing behavior
  section: "route_to_layer() function"
  note: |
    The router implements the logic that maps flag combinations to layers.
    This PRP only affects help text, not the routing logic itself.

# TEST: Existing help test patterns
- file: /home/dustin/projects/jin/tests/cli_basic.rs
  why: Shows how to test help output
  pattern: |
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
    Can add test for after_help content if desired, but not required for this subtask.
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   ├── mod.rs                    # REFERENCE: Command attribute patterns
│   │   └── args.rs                   # MODIFY: AddArgs at lines 5-30
│   ├── staging/
│   │   └── router.rs                 # REFERENCE: Layer routing logic
│   └── commands/
│       └── add.rs                    # REFERENCE: Uses AddArgs
├── tests/
│   └── cli_*.rs                      # REFERENCE: Help test patterns
└── plan/
    ├── architecture/
    │   └── system_context.md         # REFERENCE: Layer routing table
    ├── P1M5T1S1/
    │   └── PRP.md                    # CONTRACT: Updated --project help text
    └── P1M5T2S1/
        ├── PRP.md                    # THIS FILE
        └── research/
            └── clap_after_help.md    # RESEARCH: Comprehensive Clap after_help guide
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── src/
    └── cli/
        └── args.rs                   # MODIFIED: AddArgs with after_help attribute
            # BEFORE:
            # /// Arguments for the `add` command
            # #[derive(Args, Debug)]
            # pub struct AddArgs { ... }
            #
            # AFTER:
            # /// Arguments for the `add` command
            # #[derive(Args, Debug)]
            # #[command(after_help = "...")]
            # pub struct AddArgs { ... }
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: New pattern for codebase
// No existing after_help usage found. This is the first use in the codebase.
// Follow existing #[command(...)] attribute pattern from src/cli/mod.rs

// GOTCHA: Raw string delimiter selection
// Use r#"..."# for most cases. If text contains "#", use r##"..."## or more hashes.
// The proposed table doesn't contain # symbols, so r#"..."# is sufficient.

// CORRECT:
// #[command(after_help = r#"LAYER ROUTING:
//   Flags                  Target Layer
//   --mode                 → Layer 2 (ModeBase)"#)]

// INCORRECT (escaping hell):
// #[command(after_help = "LAYER ROUTING:\n  Flags                  Target Layer\n  --mode                 → Layer 2 (ModeBase)")]

// GOTCHA: Table formatting
// Clap doesn't provide automatic table formatting. Must manually space columns.
// Use monospace-friendly alignment (spaces, not tabs).
// Test in terminal to verify alignment looks correct.

// PATTERN: Multi-line attribute blocks
// Follow existing pattern from src/cli/mod.rs:
// #[command(
//     name = "jin"
// )]
// Not:
// #[command(name = "jin")]  // Single line is OK but multi-line is more readable

// CRITICAL: Table content from item description
// The item description specifies these 8 combinations:
// 1. (no flags) → Layer 7
// 2. --mode → Layer 2
// 3. --mode --project → Layer 5
// 4. --scope=X → Layer 6
// 5. --mode --scope=X → Layer 3
// 6. --mode --scope=X --project → Layer 4
// 7. --global → Layer 1
// 8. --local → Layer 8
//
// Note: system_context.md is missing --local because it predates --local implementation.
// Use item description as authoritative source.

// GOTCHA: Layer names from system_context.md
// Layer 1: GlobalBase
// Layer 2: ModeBase
// Layer 3: ModeScope
// Layer 4: ModeScopeProject
// Layer 5: ModeProject
// Layer 6: ScopeBase
// Layer 7: ProjectBase
// Layer 8: UserLocal
// Include these names in parentheses after layer numbers for clarity.

// CRITICAL: No behavior change
// This is documentation-only. The struct fields don't change.
// Only the #[command(after_help = "...")] attribute is added.

// PATTERN: Header in after_help
// Start with "LAYER ROUTING:" header to separate from options.
// Then column headers "Flags" and "Target Layer".
// Then separator line for readability.

// GOTCHA: Terminal width considerations
// The table should fit within standard 80-character terminals.
// Current design: ~65 characters wide (fits comfortably).
// If adding more content, consider terminal wrapping.

// CRITICAL: Arrow symbol for routing
// Use "→" (Unicode arrow) for visual clarity.
// This is the same symbol used in system_context.md documentation.
// Ensure file is UTF-8 encoded (already is).

// PATTERN: Alignment in table
// Column 1 (Flags): Left-aligned, variable width
// Column 2 (Target Layer): Left-aligned with arrow
// Use spaces to align. Example:
//   "Flags                  Target Layer"  (header)
//   "--mode                 → Layer 2"     (row)
//   "--scope=<X>            → Layer 6"     (row)

// GOTCHA: --scope format in table
// The item description uses "--scope=X" format.
// In actual usage, it's "--scope=<value>" or "--scope value".
// Use "--scope=<X>" in table for clarity (X = placeholder).

// CRITICAL: Attribute placement
// The #[command(after_help = "...")] MUST be:
// 1. AFTER #[derive(Args, Debug)]
// 2. BEFORE the struct definition
// 3. Can be on same line or separate line

// CORRECT PLACEMENT:
// #[derive(Args, Debug)]
// #[command(after_help = "...")]
// pub struct AddArgs {

// INCORRECT PLACEMENT:
// #[derive(Args, Debug)]
// pub struct AddArgs {
//     #[command(after_help = "...")]  // WRONG: Goes on struct, not field

// CRITICAL: Doc comment order
// The doc comment "/// Arguments for the `add` command" should remain
// at the top, before #[derive(Args, Debug)].

// CORRECT ORDER:
// /// Arguments for the `add` command
// #[derive(Args, Debug)]
// #[command(after_help = "...")]
// pub struct AddArgs {
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is a documentation-only change:
- `AddArgs` struct fields remain unchanged
- Only the `#[command(after_help = "...")]` attribute is added

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD #[command(after_help = "...")] ATTRIBUTE TO AddArgs
  - FILE: src/cli/args.rs
  - LOCATION: After line 6 (after #[derive(Args, Debug)], before pub struct AddArgs)
  - ADD: #[command(after_help = r#"LAYER ROUTING:\n...")] attribute
  - PRESERVE:
    - The doc comment "/// Arguments for the `add` command"
    - The #[derive(Args, Debug)] attribute
    - All struct fields and their attributes
  - TABLE CONTENT: (from item description - 8 combinations)
    LAYER ROUTING:
      Flags                  Target Layer
      ──────────────────────────────────────────────────────
      (no flags)             → Layer 7 (ProjectBase)
      --mode                 → Layer 2 (ModeBase)
      --mode --project       → Layer 5 (ModeProject)
      --scope=<X>            → Layer 6 (ScopeBase)
      --mode --scope=<X>     → Layer 3 (ModeScope)
      --mode --scope=<X> --project
                             → Layer 4 (ModeScopeProject)
      --global               → Layer 1 (GlobalBase)
      --local                → Layer 8 (UserLocal)
  - SYNTAX: Raw string r#"..."# for multi-line support
  - DEPENDENCIES: None (standalone change)

Task 2: VERIFY CODE COMPILES
  - RUN: cargo check
  - EXPECTED: Zero errors, zero warnings
  - DEPENDENCIES: Task 1

Task 3: RUN EXISTING TESTS
  - RUN: cargo test
  - EXPECTED: All tests pass
  - DEPENDENCIES: Task 1
  - NOTE: No test changes needed

Task 4: MANUAL VERIFICATION (Recommended)
  - RUN: cargo build --release
  - RUN: ./target/release/jin add --help
  - VERIFY: Layer routing table appears after options
  - VERIFY: Table is properly formatted and aligned
  - DEPENDENCIES: Task 1, Task 2
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE CHANGE ==================

// FILE: src/cli/args.rs
// LOCATION: Lines 5-7 (AddArgs struct definition)

// --- BEFORE (CURRENT CODE) ---
/// Arguments for the `add` command
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Files to stage
    pub files: Vec<String>,
    // ... rest of fields
}

// --- AFTER (UPDATED CODE) ---
/// Arguments for the `add` command
#[derive(Args, Debug)]
#[command(after_help = r#"LAYER ROUTING:
  Flags                  Target Layer
  ──────────────────────────────────────────────────────
  (no flags)             → Layer 7 (ProjectBase)
  --mode                 → Layer 2 (ModeBase)
  --mode --project       → Layer 5 (ModeProject)
  --scope=<X>            → Layer 6 (ScopeBase)
  --mode --scope=<X>     → Layer 3 (ModeScope)
  --mode --scope=<X> --project
                         → Layer 4 (ModeScopeProject)
  --global               → Layer 1 (GlobalBase)
  --local                → Layer 8 (UserLocal)
"#)]
pub struct AddArgs {
    /// Files to stage
    pub files: Vec<String>,
    // ... rest of fields unchanged
}

// ================== FORMATTING DECISIONS ==================
//
// Header: "LAYER ROUTING:" - Clear section header
// Column headers: "Flags" and "Target Layer" - Aligned with spaces
// Separator line: ─ characters for visual separation
// Arrow: → (Unicode) for visual clarity
//
// Column widths:
// - "Flags" column: Left-aligned, 22 characters max width
// - "Target Layer" column: Left-aligned after arrow
//
// Row alignment:
// - Short flag combos (--mode) on single line
// - Long flag combos (--mode --scope=<X> --project) split across two lines

// ================== WHY THIS FORMAT ==================
//
// 1. COMPATIBILITY: Uses raw string r#"..."# for clean multi-line text
// 2. READABILITY: Table format is scannable and familiar
// 3. COMPLETENESS: Includes all 8 combinations from item description
// 4. CONSISTENCY: Matches layer names from system_context.md
// 5. CLARITY: Layer numbers + names for unambiguous identification
// 6. ALIGNMENT: Monospace-friendly spacing for terminal display
// 7. VISUAL: Separator line and arrow symbol for quick scanning

// ================== ALTERNATIVE FORMATS CONSIDERED ==================
//
// Alternative 1 (list format, no table):
//   "LAYER ROUTING:
//    (no flags) → Layer 7 (ProjectBase)
//    --mode → Layer 2 (ModeBase)
//    ..."
//   - Less structured
//   - Harder to scan
//
// Alternative 2 (more compact):
//   "LAYER ROUTING:
//    (no flags) → L7, --mode → L2, --mode --project → L5
//    --scope=<X> → L6, --mode --scope=<X> → L3
//    --mode --scope=<X> --project → L4, --global → L1, --local → L8"
//   - Harder to read
//   - Missing layer names
//
// Alternative 3 (using \n instead of multi-line):
//   #[command(after_help = "LAYER ROUTING:\n  Flags                  Target Layer\n  ...")]
//   - Harder to maintain
//   - Less readable in code
//
// SELECTED: Multi-line table format with full layer names
// - Most readable for users
// - Most maintainable for developers
// - Consistent with documentation style

// ================== TERMINAL OUTPUT EXAMPLE ==================
//
// The help output will look like this:
//
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
//   -h, --help
//           Print help information
//
// LAYER ROUTING:
//   Flags                  Target Layer
//   ──────────────────────────────────────────────────────
//   (no flags)             → Layer 7 (ProjectBase)
//   --mode                 → Layer 2 (ModeBase)
//   --mode --project       → Layer 5 (ModeProject)
//   --scope=<X>            → Layer 6 (ScopeBase)
//   --mode --scope=<X>     → Layer 3 (ModeScope)
//   --mode --scope=<X> --project
//                          → Layer 4 (ModeScopeProject)
//   --global               → Layer 1 (GlobalBase)
//   --local                → Layer 8 (UserLocal)

// ================== ALIGNMENT DETAILS ==================
//
// Column 1 (Flags) starts at position 2 (indentation)
// Column 2 (Target Layer) starts at position 24 (after 22 chars for flags)
//
// Row by row:
//   "  Flags                  Target Layer"           (header)
//   "  ──────────────────────────────────────────────────────"  (separator)
//   "  (no flags)             → Layer 7 (ProjectBase)"
//   "  --mode                 → Layer 2 (ModeBase)"
//   "  --mode --project       → Layer 5 (ModeProject)"
//   "  --scope=<X>            → Layer 6 (ScopeBase)"
//   "  --mode --scope=<X>     → Layer 3 (ModeScope)"
//   "  --mode --scope=<X> --project"
//   "                         → Layer 4 (ModeScopeProject)"
//   "  --global               → Layer 1 (GlobalBase)"
//   "  --local                → Layer 8 (UserLocal)"

// ================== INTEGRATION WITH OTHER CHANGES ==================
//
// This PRP runs IN PARALLEL with P1.M5.T1.S1 (Update --project help text).
// When both are complete, users will see:
//
// 1. In OPTIONS:
//    --project
//            Add to mode-project layer (Layer 5, requires --mode).
//            For project-base layer (Layer 7), use: jin add <file> without flags
//
// 2. In LAYER ROUTING (after_help):
//    --mode --project       → Layer 5 (ModeProject)
//    (no flags)             → Layer 7 (ProjectBase)
//
// The two pieces complement each other:
// - Option help explains --project specifically
// - Routing table shows the complete picture
```

### Integration Points

```yaml
CLAP_DERIVE_API:
  - mechanism: #[command(after_help = "...")] attribute
  - trigger: User runs "jin add --help" or "jin add -h"
  - display: The table appears after all options and before any subcommands
  - compatibility: Clap 4.5 with derive feature

HELP_TEXT_DISPLAY_ORDER:
  - section 1: Command description ("Add files to the Jin layer system")
  - section 2: Usage line ("jin add [OPTIONS] <FILES>...")
  - section 3: Arguments ("<FILES>...    Files to stage")
  - section 4: Options (all the flags with their help text)
  - section 5: Help flag ("-h, --help    Print help information")
  - section 6: AFTER_HELP (the new LAYER ROUTING table) ← ADD HERE

EXISTING_TESTS:
  - pattern: Tests likely check for specific flags in help output
  - impact: Tests continue to pass (don't validate after_help content)
  - no_changes: No test modifications needed

DOCUMENTATION_CONSISTENCY:
  - source: plan/architecture/system_context.md
  - mapping: Table content matches system_context.md layer definitions
  - addition: --local (Layer 8) added - not in original system_context.md table
  - reason: --local was implemented after system_context.md was written

P1M5T1S1_CONTRACT:
  - parallel: Both P1.M5.T1.S1 and P1.M5.T2.S1 improve help output
  - complement: P1.M5.T1.S1 clarifies --project, P1.M5.T2.S1 shows full routing
  - integration: Users get both specific flag help and global routing context
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding the after_help attribute
cargo check                              # Type checking - MUST pass
cargo fmt -- --check                     # Format check - should pass

# Expected: Zero errors, zero warnings
# Adding a #[command(...)] attribute is a straightforward change
```

### Level 2: Help Text Verification

```bash
# Build the CLI to test help output
cargo build --release

# Verify the help text displays correctly
./target/release/jin add --help

# Expected output includes the LAYER ROUTING table at the end
# The table should be properly aligned and readable

# Verify specific content appears:
./target/release/jin add --help | grep "LAYER ROUTING"
./target/release/jin add --help | grep "Layer 7 (ProjectBase)"
./target/release/jin add --help | grep "Layer 8 (UserLocal)"
./target/release/jin add --help | grep "→"

# Expected: All grep commands find the text
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
# Note: Tests check for flag presence, not after_help content
```

### Level 4: Manual Verification (User Experience Validation)

```bash
# Verify the complete help output is readable
./target/release/jin add --help | less

# Check that:
# 1. LAYER ROUTING header appears
# 2. Table columns are aligned
# 3. All 8 flag combinations are present
# 4. Layer numbers match system_context.md
# 5. Layer names are included
# 6. Table fits within terminal width (no awkward wrapping)

# Verify arrow character renders correctly
# (Should show as →, not as escaped or broken character)

# Verify indentation looks correct (2 spaces for table content)
```

### Level 5: Cross-Reference Validation

```bash
# Verify routing matches actual router behavior
# (Read router.rs to confirm the table is accurate)

# Check a few key routing rules:
grep -A 10 "route_to_layer" src/staging/router.rs | grep -E "ModeBase|ProjectBase"

# Expected: Code confirms the routing behavior shown in the table
# - No flags → ProjectBase
# - --mode → ModeBase
# - --mode --project → ModeProject
# etc.

# Verify layer names match Layer enum
grep -A 20 "pub enum Layer" src/core/layer.rs

# Expected: Layer names in table match enum variants:
# GlobalBase, ModeBase, ModeScope, ModeScopeProject,
# ModeProject, ScopeBase, ProjectBase, UserLocal
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

- [ ] `#[command(after_help = "...")]` attribute added to AddArgs
- [ ] Attribute is placed correctly (after derive, before struct)
- [ ] Raw string syntax `r#"..."#` used
- [ ] "LAYER ROUTING:" header appears in help output
- [ ] All 8 routing combinations from item description are present
- [ ] Layer numbers and names match system_context.md
- [ ] Table is properly formatted and aligned
- [ ] Arrow character (→) renders correctly

### Code Quality Validation

- [ ] Follows existing codebase patterns (#[command(...)] blocks)
- [ ] Doc comment preserved at top of struct
- [ ] Struct fields unchanged (documentation-only change)
- [ ] Table is readable and scannable
- [ ] Format is consistent with documentation style

### Documentation & Deployment

- [ ] Layer routing matches actual behavior in router.rs
- [ ] Layer names match Layer enum variants
- [ ] --local (Layer 8) included (not in original system_context.md)
- [ ] Table complements --project help text from P1.M5.T1.S1
- [ ] Ready for deployment (help improvement only, no behavior change)

---

## Anti-Patterns to Avoid

- **Don't** modify any struct fields - this is documentation-only
- **Don't** change the routing logic - only help text changes
- **Don't** use regular strings with `\n` - use raw strings `r#"..."#`
- **Don't** put the attribute on individual fields - it goes on the struct
- **Don't** remove any existing attributes or doc comments
- **Don't** make the table too wide - keep it under 80 characters if possible
- **Don't** forget to include all 8 combinations from the item description
- **Don't** mix up layer numbers - verify against system_context.md
- **Don't** use tabs for alignment - use spaces
- **Don't** skip manual verification - table alignment needs visual check

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single attribute addition**: Only one `#[command(after_help = "...")]` attribute to add
- **Exact specification**: Table content fully specified in item description
- **No behavior change**: Pure documentation update, no code logic changes
- **Low risk**: Adding help text doesn't break compilation or tests
- **Clear context**: Comprehensive research on Clap after_help usage
- **Test verification**: Simple manual verification with `jin add --help`
- **Complementary**: Works with P1.M5.T1.S1 to provide complete help context

**Zero Risk Factors**:
- No logic changes - only help text addition
- No new dependencies or imports
- Change is reversible (can remove attribute if needed)
- Existing tests verify the build works
- No test modifications needed

**Current Status**: Ready for implementation - all context gathered, exact change specified, verification steps defined

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from this PRP creation):
- Codebase analysis of args.rs, system_context.md
- Clap after_help comprehensive research stored at plan/P1M5T2S1/research/clap_after_help.md
- Pattern analysis from existing command attributes in cli/mod.rs

**Related PRP** (Parallel execution):
- `plan/P1M5T1S1/PRP.md` - Updates --project help text (complementary work)

**Code Files**:
- `src/cli/args.rs` - File to modify (AddArgs struct)
- `src/staging/router.rs` - Layer routing behavior (reference only)
- `src/core/layer.rs` - Layer enum definitions (reference only)
- `plan/architecture/system_context.md` - Layer system reference

**External Documentation**:
- [Clap Derive API](https://docs.rs/clap/latest/clap/_derive/index.html) - Derive attribute syntax
- [Clap Command.after_help](https://docs.rs/clap/latest/clap/builder/struct.Command.html#method.after_help) - after_help method
- [Clap Discussion #4090](https://github.com/clap-rs/clap/discussions/4090) - Derive attributes confirmation
- [Rust Raw Strings](https://doc.rust-lang.org/reference/tokens.html#raw-string-literals) - Raw string syntax

**Research Directory**:
- `plan/P1M5T2S1/research/clap_after_help.md` - Comprehensive Clap after_help research
