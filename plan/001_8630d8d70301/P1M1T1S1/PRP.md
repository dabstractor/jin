# PRP: P1.M1.T1.S1 - Add local Field to AddArgs Struct

---

## Goal

**Feature Goal**: Extend the `AddArgs` struct in `src/cli/args.rs` to include a `local: bool` field with clap attribute `#[arg(long)]` for parsing the `--local` CLI flag.

**Deliverable**: Modified `AddArgs` struct with the new `local` field that:
1. Parses `--local` flag from CLI arguments
2. Provides help text: "Add to user-local layer (Layer 8, machine-specific)"
3. Is positioned after the `global` field for consistency
4. Enables downstream tasks to access the flag value

**Success Definition**:
- `cargo check` passes with zero errors
- `cargo build` succeeds with the modified struct
- `./target/debug/jin add --help` shows the `--local` flag
- All existing tests continue to pass
- No changes to other files (this is a standalone struct modification)

---

## User Persona

**Target User**: Developer using Jin to manage machine-specific configuration files

**Use Case**: A developer needs to store configuration files that are specific to their machine (e.g., local paths, machine certificates, development environment overrides) and should override all other layer configurations.

**User Journey**:
1. Developer has a config file with machine-specific settings
2. Developer runs `jin add .env.local --local`
3. The flag is parsed and available for routing logic (implemented in downstream tasks)
4. File is routed to Layer 8 (UserLocal) for storage at `~/.jin/local/`

**Pain Points Addressed**:
- Currently inaccessible Layer 8 prevents users from storing machine-specific overrides
- No way to override all other layers except editing workspace directly (breaks merge system)
- Incomplete layer routing implementation (PRD specifies 9 layers, only 8 accessible)

---

## Why

- **Completes Layer System**: The PRD defines 9 layers but the CLI only provides access to 8. Layer 8 (UserLocal) is currently inaccessible via CLI.
- **Enables Machine-Specific Overrides**: Developers need a way to store configuration that applies only to their machine (e.g., local database paths, dev certificates).
- **Highest Precedence Layer**: UserLocal (Layer 8) has second-highest precedence, overriding all layers except WorkspaceActive. This is critical for personal development environments.
- **Foundational for Downstream Work**: This is the first subtask in P1.M1; all other tasks (P1.M1.T2, T3, T4) depend on this field being present.
- **Follows Existing Patterns**: The implementation mirrors the existing `--global`, `--mode`, `--project` flags for consistency.

---

## What

### User-Visible Behavior

After this change:
```bash
# The --local flag appears in help output
jin add --help
# Output includes:
#   --local     Add to user-local layer (Layer 8, machine-specific)

# The flag is parsed from CLI arguments
jin add .env.local --local
# args.local == true (will be used for routing in downstream tasks)

# Flag can be specified independently
jin add config.json --local
# args.local == true, args.mode == false, etc.

# Mutually exclusive with other layer flags (validated downstream)
jin add config.json --local --mode
# Will produce error in downstream tasks (not this subtask)
```

### Technical Requirements

1. **Struct Modification**: Add `local: bool` field to `AddArgs` struct
2. **Clap Attribute**: Use `#[arg(long)]` for flag parsing
3. **Help Text**: Doc comment with "Add to user-local layer (Layer 8, machine-specific)"
4. **Field Position**: After `global` field for consistency
5. **No Behavioral Changes**: This subtask only adds the field; routing logic is in P1.M1.T2

### Success Criteria

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo build` succeeds
- [ ] `jin add --help` shows `--local` flag with correct help text
- [ ] `cargo test` passes (existing tests, no new tests for this subtask)
- [ ] `local: bool` field is public and accessible
- [ ] Field is positioned after `global` field

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for adding a boolean field to an existing clap Args struct. The implementation is a single-line addition to an existing struct with well-defined patterns to follow._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Project Architecture
- file: plan/docs/system_context.md
  why: Complete 9-layer system architecture, layer routing table, storage paths
  section: "The 9-Layer System" and "Layer Routing (jin add flags)"
  critical: Layer 8 (UserLocal) specification - precedence 8, stored at ~/.jin/local/

# Target File to Modify
- file: src/cli/args.rs
  why: The AddArgs struct definition - this is the only file to modify
  pattern: Follow existing pattern for --global, --mode, --project flags
  gotcha: Field must be public (pub) for downstream access

# Existing Boolean Flag Pattern (follow this)
- file: src/cli/args.rs (lines 20-25)
  why: Exact pattern to follow for the new local field
  pattern: |
    /// Target global layer
    #[arg(long)]
    pub global: bool,

# Clap Documentation (External)
- url: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
  why: Derive API tutorial for argument parsing
  section: "Argument Attributes" and "Boolean Flags"
  critical: #[arg(long)] on bool field creates SetTrue action

- url: https://docs.rs/clap/latest/clap/_derive/index.html
  why: Complete derive API reference
  section: "arg attribute" for all available options
  critical: bool fields default to SetTrue action when #[arg(long)] is used

# Related Work Item Context
- docfile: plan/P1M1T1S1/research/related_work_items.md
  why: Understand how this field will be used downstream
  section: "Downstream Dependencies" and "Technical Implementation Context"
  critical: Validation rules, exclusivity with other flags, Layer 8 routing

# Clap Boolean Flag Research
- docfile: plan/P1M1T1S1/research/clap_boolean_flags.md
  why: Comprehensive research on clap boolean flags
  section: "Basic Boolean Flag Setup" and "Help Text Best Practices"
  critical: Gotchas, testing patterns, complete examples

# Test Patterns (Reference Only - No Tests in This Subtask)
- docfile: plan/P1M1T1S1/research/test_patterns.md
  why: Understanding how flags are tested (for downstream tasks)
  section: "Flag Testing Patterns" and "Boolean Flags (Presence/Absence)"
  note: Tests will be added in P1.M1.T4.S1, not this subtask

# Related Struct (Downstream Context - Reference Only)
- file: src/staging/router.rs (lines 5-16)
  why: The RoutingOptions struct that will receive local field in P1.M1.T2.S1
  pattern: Follow same structure: mode, scope, project, global, local
  note: DO NOT MODIFY in this subtask - reference only

# Layer Definition (Reference)
- file: src/core/layer.rs (lines 27-28)
  why: Layer::UserLocal enum variant definition
  pattern: |
    /// Layer 8: Machine-only overlays (~/.jin/local/)
    UserLocal,
  note: DO NOT MODIFY - reference for understanding target layer
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   ├── mod.rs                # Cli struct, Commands enum
│   │   └── args.rs               # TARGET FILE - AddArgs struct (line 6-26)
│   ├── core/
│   │   └── layer.rs              # Layer enum with UserLocal variant (line 28)
│   ├── staging/
│   │   └── router.rs             # RoutingOptions struct (line 5-16)
│   └── commands/
│       └── add.rs                # Uses AddArgs in execute() (line 50-55)
└── plan/
    └── P1M1T1S1/
        ├── PRP.md                # This file
        └── research/             # Research artifacts
```

### Desired Codebase Tree After This Subtask

```bash
jin/
├── src/
│   └── cli/
│       └── args.rs               # MODIFIED: Add local: bool field to AddArgs
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: clap v4.5 derive API - bool fields use SetTrue action by default
// When #[arg(long)] is used on a bool field, it automatically uses ArgAction::SetTrue
// --local flag absent -> local = false
// --local flag present -> local = true

// GOTCHA: Field must be public (pub) for access by commands/add.rs
// The execute() function accesses args.local directly
// Pattern: pub local: bool,

// PATTERN: Use doc comment for help text generation
// clap automatically uses the doc comment as help text
/// Add to user-local layer (Layer 8, machine-specific)
#[arg(long)]
pub local: bool,

// PATTERN: Position after global field for consistency
// Current order: files, mode, scope, project, global
// New order: files, mode, scope, project, global, local

// NOTE: This subtask ONLY adds the field to AddArgs struct
// DO NOT modify RoutingOptions struct (that's P1.M1.T2.S1)
// DO NOT add validation logic (that's P1.M1.T2.S2)
// DO NOT add routing case (that's P1.M1.T2.S3)
// DO NOT write tests (that's P1.M1.T4.S1)

// REFERENCE: Layer 8 (UserLocal) specification
// - Precedence: 8 (second highest, only WorkspaceActive is higher)
// - Storage: ~/.jin/local/
// - Git ref: refs/jin/layers/local
// - Purpose: Machine-specific configuration overrides
// - Independence: Does NOT require active mode or scope
```

---

## Implementation Blueprint

### Data Models and Structure

**Target File**: `src/cli/args.rs` (lines 6-26)

**Current AddArgs Struct**:
```rust
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
}
```

**Modified AddArgs Struct** (add the `local` field after `global`):
```rust
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
```

### Implementation Tasks

```yaml
Task 1: MODIFY src/cli/args.rs
  - FILE: src/cli/args.rs (lines 6-26)
  - ACTION: Add local field to AddArgs struct
  - IMPLEMENT:
    * Add doc comment: /// Target user-local layer (Layer 8, machine-specific)
    * Add clap attribute: #[arg(long)]
    * Add field declaration: pub local: bool,
    * Position: After global field (line 26)
  - PATTERN: Follow exact same pattern as global field above
  - NAMING: local (snake_case), matches --local flag name
  - ACCESS: pub (public) for downstream command access
  - TYPE: bool (boolean flag, false when absent, true when present)
  - DEPENDENCIES: None
  - FILES TO MODIFY: src/cli/args.rs (1 file)
  - FILES TO CREATE: None
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE TO ADD ==================
// Location: src/cli/args.rs, after line 25 (after global field)

    /// Target user-local layer (Layer 8, machine-specific)
    #[arg(long)]
    pub local: bool,

// ================== CONTEXT FOR ADDITION ==================
// The struct currently ends at line 26 with closing brace
// Add the new field BEFORE the closing brace
// Position it immediately after the global field for consistency

// ================== VERIFICATION ==================
// After adding, the struct should look like this:

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
    pub local: bool,  // <-- NEW FIELD ADDED HERE
}

// ================== CLAP BEHAVIOR ==================
// With #[arg(long)] on a bool field:
// - Default action: ArgAction::SetTrue (automatic for bool)
// - --local flag absent: args.local == false
// - --local flag present: args.local == true
// - Help text: Generated from doc comment automatically

// ================== HELP TEXT OUTPUT ==================
// Running: jin add --help
// Will show:
//   --local     Target user-local layer (Layer 8, machine-specific)
```

### Integration Points

```yaml
MODIFICATIONS:
  - file: src/cli/args.rs
    change: Add local field to AddArgs struct
    lines: Insert after line 25 (before closing brace)
    scope: Single struct modification only

NO CHANGES TO:
  - src/cli/mod.rs (no imports needed, args.rs is self-contained)
  - src/commands/add.rs (will be updated in P1.M1.T3.S1)
  - src/staging/router.rs (will be updated in P1.M1.T2.S1)
  - src/core/layer.rs (UserLocal already exists)
  - tests/ (no tests in this subtask)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modification - must pass before proceeding
cargo check                           # Type checking - MUST pass with 0 errors

# Expected: Zero errors. If errors exist, READ output carefully.
# Common issues:
# - Missing comma after field declaration
# - Incorrect attribute syntax
# - Field not declared as pub

# Format check (optional but recommended)
cargo fmt -- --check                  # Format check

# Expected: No formatting issues or auto-fix suggestions
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build with compilation successful

# Verify binary exists
./target/debug/jin --help             # Should show help without errors

# Expected: Binary executes and shows CLI help
```

### Level 3: Help Text Verification

```bash
# Verify --local flag appears in help output
./target/debug/jin add --help         # Should show --local flag

# Expected output includes:
#   --local     Target user-local layer (Layer 8, machine-specific)

# If help text doesn't appear:
# - Check doc comment formatting
# - Verify #[arg(long)] attribute
# - Ensure field is public (pub)
```

### Level 4: Existing Test Validation

```bash
# Run all existing tests to ensure no regressions
cargo test

# Expected: All tests pass (no test modifications in this subtask)
# Focus areas:
# - CLI argument parsing tests (tests/cli_basic.rs)
# - Add command tests (tests/cli_add*.rs if exists)
# - Unit tests in src/commands/add.rs
```

### Level 5: Manual Smoke Test

```bash
# Test flag is recognized (no functionality expected yet)
./target/debug/jin add test.json --local

# Expected: Either "Jin not initialized" error (expected, Jin not set up)
# OR other command execution errors
# BUT NOT: "unexpected argument '--local'" or "error: Found argument '--local' which wasn't expected"

# If you see "unexpected argument" error:
# - Check #[arg(long)] attribute is present
# - Verify field is in AddArgs struct
# - Run cargo check and cargo build again
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo build` succeeds
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test` passes (all existing tests)
- [ ] Binary executes: `./target/debug/jin --help` works

### Feature Validation

- [ ] `AddArgs` struct has `local: bool` field
- [ ] Field is public (`pub local: bool`)
- [ ] Field has `#[arg(long)]` attribute
- [ ] Field has doc comment with "Layer 8, machine-specific"
- [ ] Field is positioned after `global` field
- [ ] `jin add --help` shows `--local` flag
- [ ] `jin add test.json --local` doesn't produce "unexpected argument" error

### Code Quality Validation

- [ ] Follows existing pattern for `global` field
- [ ] Doc comment formatting matches other fields
- [ ] Field name is `local` (snake_case)
- [ ] No unused imports or dead code warnings
- [ ] Clippy produces no warnings: `cargo clippy`

### Documentation & Deployment

- [ ] Help text clearly explains Layer 8 purpose
- [ ] Help text mentions "machine-specific"
- [ ] No breaking changes to existing functionality

---

## Anti-Patterns to Avoid

- **Don't** modify any other files in this subtask - only `src/cli/args.rs`
- **Don't** add validation logic for `--local` flag (that's P1.M1.T2.S2)
- **Don't** modify `RoutingOptions` struct (that's P1.M1.T2.S1)
- **Don't** add routing case for `Layer::UserLocal` (that's P1.M1.T2.S3)
- **Don't** update `add.rs` command to pass the flag (that's P1.M1.T3.S1)
- **Don't** write integration tests (that's P1.M1.T4.S1)
- **Don't** add `#[arg(short = 'l')]` - only `--local` long form needed
- **Don't** use `default_value_t` - bool fields default to `false` automatically
- **Don't** use `ArgAction::SetTrue` explicitly - it's automatic for `bool` with `#[arg(long)]`
- **Don't** place field before `global` - maintain consistency
- **Don't** forget the comma after the field declaration
- **Don't** make the field private - must be `pub` for downstream access

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Extremely Simple**: Single field addition to existing struct
- **Clear Pattern**: Exact same pattern as existing `global` field
- **Well-Researched**: Comprehensive documentation of clap boolean flags
- **No Logic Changes**: Pure data structure modification, no behavior changes
- **No Dependencies**: Standalone change with no external requirements
- **No Tests**: Existing tests validate no regressions
- **Exact Specification**: Field name, type, attributes, position all specified
- **Clear Validation**: Unambiguous success criteria

**Implementation is equivalent to adding one field to a struct**:
```rust
/// Target user-local layer (Layer 8, machine-specific)
#[arg(long)]
pub local: bool,
```

This is a foundational subtask that creates the hook for downstream functionality. The implementation risk is minimal because it follows an existing, proven pattern in the same file.

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1T1S1/research/`

- `clap_boolean_flags.md` - Comprehensive clap v4.5 boolean flag research
- `test_patterns.md` - CLI testing patterns in the codebase
- `related_work_items.md` - Downstream dependencies and architectural context

**Key External References**:
- [clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [clap Derive API Reference](https://docs.rs/clap/latest/clap/_derive/index.html)
- [Jin System Architecture](plan/docs/system_context.md)
