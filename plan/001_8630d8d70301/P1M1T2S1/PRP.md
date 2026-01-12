# PRP: P1.M1.T2.S1 - Add local Field to RoutingOptions Struct

---

## Goal

**Feature Goal**: Extend the `RoutingOptions` struct in `src/staging/router.rs` to include a `pub local: bool` field that mirrors the `local` field added to `AddArgs` in P1.M1.T1.S1.

**Deliverable**: Modified `RoutingOptions` struct with the new `local` field that:
1. Enables routing to Layer 8 (UserLocal) when `local == true`
2. Maintains consistency with the `AddArgs` struct from P1.M1.T1.S1
3. Preserves the `#[derive(Debug, Default)]` macros
4. Uses the same pattern as existing boolean fields (`mode`, `project`, `global`)

**Success Definition**:
- `cargo check` passes with zero errors
- `cargo build` succeeds with the modified struct
- `cargo test` continues to pass (no breaking changes to existing tests)
- All four command consumers (add, mv, rm, import_cmd) can construct RoutingOptions with the new field
- The struct's Default trait provides `local: false` as expected

---

## User Persona

**Target User**: Developer implementing the `--local` flag routing logic (this is an internal data structure change)

**Use Case**: The `RoutingOptions` struct is the bridge between CLI argument parsing and layer routing. After P1.M1.T1.S1 adds `local: bool` to `AddArgs`, this struct must have the corresponding field to receive that value.

**User Journey**:
1. P1.M1.T1.S1 completes: `AddArgs` has `local: bool` field
2. P1.M1.T2.S1 (this subtask): `RoutingOptions` gets `local: bool` field
3. P1.M1.T3.S1: `add.rs` command passes `args.local` to `RoutingOptions { local: args.local, ... }`
4. P1.M1.T2.S2: Validation logic prevents combining `--local` with other layer flags
5. P1.M1.T2.S3: Routing logic in `route_to_layer()` returns `Layer::UserLocal` when `local == true`

**Pain Points Addressed**:
- Without this field, the `local` flag from CLI cannot be passed to the routing system
- Asymmetric data structures (AddArgs has field, RoutingOptions doesn't) would cause compile errors

---

## Why

- **Data Structure Alignment**: P1.M1.T1.S1 adds `local: bool` to `AddArgs`. The `RoutingOptions` struct must mirror this to receive the flag value from command execution.
- **Enables Downstream Routing**: The `route_to_layer()` function consumes `RoutingOptions`. Without the `local` field, it cannot route to Layer 8.
- **Maintains Architectural Consistency**: All layer flags (`mode`, `project`, `global`) have corresponding fields in both structs. Adding `local` preserves this pattern.
- **Zero Breaking Changes**: Adding a bool field to a struct with `#[derive(Default)]` is backward compatible (defaults to `false`).
- **Foundation for Validation and Routing**: P1.M1.T2.S2 (validation) and P1.M1.T2.S3 (routing) depend on this field being present.

---

## What

### User-Visible Behavior

This subtask is **internal-only** - no direct user-visible behavior changes. The change enables downstream tasks to implement user-visible functionality.

### Technical Requirements

1. **Struct Modification**: Add `pub local: bool` field to `RoutingOptions` struct
2. **Field Position**: After `global` field (line 15) for consistency with AddArgs
3. **Doc Comment**: `/// Target user-local layer (Layer 8)`
4. **No Other Changes**: Do NOT modify `route_to_layer()` or `validate_routing_options()` (those are separate subtasks)

### Success Criteria

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (existing tests continue to work)
- [ ] `RoutingOptions::default().local == false` (Default trait behavior)
- [ ] `RoutingOptions { local: true, ..Default::default() }` compiles
- [ ] All command consumers can include `local: args.local` in struct initialization

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for adding a boolean field to an existing Rust struct with well-defined patterns. The implementation is a single-line addition that mirrors an existing pattern in the same struct._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Contract from Previous Work Item
- docfile: plan/P1M1T1S1/PRP.md
  why: Understand what AddArgs will produce - the local field that needs to flow through
  section: "Goal", "Implementation Blueprint", "Data Models and Structure"
  critical: AddArgs will have pub local: bool field that must map to RoutingOptions.local

# Target File to Modify
- file: src/staging/router.rs (lines 5-16)
  why: The RoutingOptions struct definition - this is the only file to modify
  pattern: Follow existing pattern for mode, project, global boolean fields
  gotcha: This is NOT a clap struct - no #[arg(long)] attributes needed

# Existing Boolean Field Pattern (follow this)
- file: src/staging/router.rs (lines 8-15)
  why: Exact pattern to follow for the new local field
  pattern: |
    /// Target mode layer
    pub mode: bool,

    /// Target scope
    pub scope: Option<String>,

    /// Target project layer
    pub project: bool,

    /// Target global layer
    pub global: bool,
  note: Add local field after global, same pattern

# Command Consumers - Build Pattern Reference
- file: src/commands/add.rs (lines 50-55)
  why: Shows how RoutingOptions is constructed from AddArgs
  pattern: |
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
  critical: After this subtask, will need to add `local: args.local,` line (that's P1.M1.T3.S1)

- file: src/commands/mv.rs (lines 54-59)
  why: Same construction pattern - confirms consistency
  note: All four commands (add, mv, rm, import_cmd) use this pattern

- file: src/commands/rm.rs (lines 49-54)
  why: Same construction pattern - confirms consistency

- file: src/commands/import_cmd.rs
  why: Same construction pattern - confirms consistency

# Derive Macro Pattern
- file: src/staging/router.rs (line 6)
  why: The #[derive(Debug, Default)] macros must be preserved
  pattern: #[derive(Debug, Default)]
  gotcha: bool fields automatically get Default::default() = false

# Layer 8 (UserLocal) Specification
- docfile: plan/P1M1T2S1/research/layer_8_specification.md
  why: Complete Layer 8 semantics for doc comment
  critical: Precedence 8, stored at ~/.jin/local/, refs/jin/layers/local

- file: src/core/layer.rs (line 28)
  why: Layer::UserLocal enum variant definition
  pattern: |
    /// Layer 8: Machine-only overlays (~/.jin/local/)
    UserLocal,

# RoutingOptions Usage Analysis
- docfile: plan/P1M1T2S1/research/routing_options_analysis.md
  why: Complete analysis of struct semantics and consumers
  section: "Field Semantics", "Consumers of RoutingOptions", "Pattern for Adding New Field"
  critical: All four commands build RoutingOptions with struct literal syntax

# Default Trait Behavior
- url: https://doc.rust-lang.org/std/default/trait.Default.html
  why: Understanding how bool fields get default values
  critical: bool: Default::default() returns false

# Rust Struct Best Practices (External)
- url: https://rust-lang.github.io/rust-clippy/master/index.html
  why: Clippy linting for struct field ordering and patterns
  section: "struct field ordering" warnings
  note: Keep bool fields together for memory efficiency (already done in this struct)
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   └── args.rs               # AddArgs with local field (P1.M1.T1.S1 output)
│   ├── core/
│   │   └── layer.rs              # Layer enum with UserLocal variant (line 28)
│   ├── staging/
│   │   ├── mod.rs                # Exports RoutingOptions (line 17)
│   │   └── router.rs             # TARGET FILE - RoutingOptions struct (lines 5-16)
│   └── commands/
│       ├── add.rs                # Builds RoutingOptions from AddArgs (lines 50-55)
│       ├── mv.rs                 # Builds RoutingOptions from MvArgs (lines 54-59)
│       ├── rm.rs                 # Builds RoutingOptions from RmArgs (lines 49-54)
│       └── import_cmd.rs         # Builds RoutingOptions from ImportArgs
└── plan/
    ├── P1M1T1S1/
    │   └── PRP.md                # Previous work item (contract)
    └── P1M1T2S1/
        ├── PRP.md                # This file
        └── research/             # Research artifacts
```

### Desired Codebase Tree After This Subtask

```bash
jin/
├── src/
│   └── staging/
│       └── router.rs             # MODIFIED: Add local: bool field to RoutingOptions
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: RoutingOptions is NOT a clap Args struct
// It is a plain data struct with no derive macros except Debug and Default
// DO NOT add #[arg(long)] or any clap-specific attributes

// PATTERN: Public boolean field with doc comment
/// Target user-local layer (Layer 8)
pub local: bool,

// GOTCHA: Field must be public (pub) for struct literal initialization
// All four commands use: RoutingOptions { mode: ..., local: ..., ... }
// If field is private, this will cause compile errors in command files

// DERIVE: #[derive(Debug, Default)] provides automatic defaults
// - Debug: Enables {:?} formatting
// - Default: Provides RoutingOptions::default() with all bools = false

// DEFAULT BEHAVIOR: bool fields default to false automatically
// RoutingOptions::default().local == false  // true

// FIELD ORDER: Place after global field for consistency
// Current: mode, scope, project, global
// New: mode, scope, project, global, local

// NOTE: This subtask ONLY adds the field to RoutingOptions struct
// DO NOT modify route_to_layer() function (that's P1.M1.T2.S3)
// DO NOT modify validate_routing_options() function (that's P1.M1.T2.S2)
// DO NOT modify command files to pass the flag (that's P1.M1.T3.S1)

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

**Target File**: `src/staging/router.rs` (lines 5-16)

**Current RoutingOptions Struct**:
```rust
/// Options for routing a file to a layer
#[derive(Debug, Default)]
pub struct RoutingOptions {
    /// Target mode layer
    pub mode: bool,
    /// Target scope
    pub scope: Option<String>,
    /// Target project layer
    pub project: bool,
    /// Target global layer
    pub global: bool,
}
```

**Modified RoutingOptions Struct** (add the `local` field after `global`):
```rust
/// Options for routing a file to a layer
#[derive(Debug, Default)]
pub struct RoutingOptions {
    /// Target mode layer
    pub mode: bool,
    /// Target scope
    pub scope: Option<String>,
    /// Target project layer
    pub project: bool,
    /// Target global layer
    pub global: bool,
    /// Target user-local layer (Layer 8)
    pub local: bool,
}
```

### Implementation Tasks

```yaml
Task 1: MODIFY src/staging/router.rs
  - FILE: src/staging/router.rs (lines 5-16)
  - ACTION: Add local field to RoutingOptions struct
  - IMPLEMENT:
    * Add doc comment: /// Target user-local layer (Layer 8)
    * Add field declaration: pub local: bool,
    * Position: After global field (line 15), before closing brace (line 16)
  - PATTERN: Follow exact same pattern as global field above
  - NAMING: local (snake_case), matches Layer::UserLocal semantics
  - ACCESS: pub (public) for struct literal initialization in commands
  - TYPE: bool (boolean flag, false when default)
  - DEPENDENCIES: None
  - FILES TO MODIFY: src/staging/router.rs (1 file)
  - FILES TO CREATE: None
  - PRESERVE: #[derive(Debug, Default)] macros unchanged
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE TO ADD ==================
// Location: src/staging/router.rs, after line 15 (after global field)

    /// Target user-local layer (Layer 8)
    pub local: bool,

// ================== CONTEXT FOR ADDITION ==================
// The struct currently ends at line 16 with closing brace
// Add the new field BEFORE the closing brace
// Position it immediately after the global field for consistency

// ================== VERIFICATION ==================
// After adding, the struct should look like this:

/// Options for routing a file to a layer
#[derive(Debug, Default)]
pub struct RoutingOptions {
    /// Target mode layer
    pub mode: bool,
    /// Target scope
    pub scope: Option<String>,
    /// Target project layer
    pub project: bool,
    /// Target global layer
    pub global: bool,
    /// Target user-local layer (Layer 8)
    pub local: bool,  // <-- NEW FIELD ADDED HERE
}

// ================== DEFAULT TRAIT BEHAVIOR ==================
// With #[derive(Default)] on a struct with bool fields:
// - RoutingOptions::default() creates struct with all bools = false
// - RoutingOptions::default().local == false  // true
// - Can use ..Default::default() in struct updates

// ================== STRUCT LITERAL USAGE ==================
// Commands will initialize like this (in P1.M1.T3.S1):
let options = RoutingOptions {
    mode: args.mode,
    scope: args.scope.clone(),
    project: args.project,
    global: args.global,
    local: args.local,  // <-- Will be added in P1.M1.T3.S1
};

// ================== FIELD SEMANTICS ==================
// local == true  -> Route to Layer 8 (UserLocal)
// local == false -> No effect on routing (handled by other fields)
// Independence: local does NOT require mode, scope, or project
```

### Integration Points

```yaml
MODIFICATIONS:
  - file: src/staging/router.rs
    change: Add local field to RoutingOptions struct
    lines: Insert after line 15 (before closing brace at line 16)
    scope: Single struct field addition only

NO CHANGES TO:
  - src/staging/mod.rs (re-exports are automatic, no changes needed)
  - src/commands/add.rs (will be updated in P1.M1.T3.S1)
  - src/commands/mv.rs (will be updated in P1.M1.T3.S1)
  - src/commands/rm.rs (will be updated in P1.M1.T3.S1)
  - src/commands/import_cmd.rs (will be updated in P1.M1.T3.S1)
  - route_to_layer() function (will be updated in P1.M1.T2.S3)
  - validate_routing_options() function (will be updated in P1.M1.T2.S2)
  - tests/ (no new tests in this subtask - existing tests should pass)
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
# - Field not declared as pub
# - Incorrect placement (should be after global, before closing brace)

# Format check (optional but recommended)
cargo fmt -- --check                  # Format check

# Expected: No formatting issues or auto-fix suggestions

# Clippy check (for code quality)
cargo clippy                          # Lint checking

# Expected: No warnings. If warnings appear, fix them.
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build with compilation successful

# Verify binary exists
./target/debug/jin --version          # Should show version without errors

# Expected: Binary executes and shows version
```

### Level 3: Default Trait Verification

```bash
# Test that Default trait works correctly
cargo test routing_options_default -- --nocapture

# Or create a quick test:
cat > test_default.rs << 'EOF'
use jin::staging::RoutingOptions;

fn main() {
    let opts = RoutingOptions::default();
    assert_eq!(opts.local, false);
    println!("Default local field: {}", opts.local);
}
EOF
rustc --edition 2021 -L target/debug/deps --extern jin=target/debug/libjin.rlib test_default.rs
./test_default

# Expected: "Default local field: false"
```

### Level 4: Struct Literal Compilation

```bash
# Test that struct literal syntax works
cargo test struct_literal_compilation -- --nocapture

# Or verify in command context (existing tests should compile):
cargo test --package jin --lib staging::router::tests

# Expected: All tests compile and pass
```

### Level 5: Existing Test Validation

```bash
# Run all existing tests to ensure no regressions
cargo test --lib

# Expected: All tests pass
# Focus areas:
# - Router tests in src/staging/router.rs (lines 84-212)
# - Command tests in src/commands/add.rs (lines 229-353)
# - Any integration tests

# Full test suite
cargo test

# Expected: All tests pass (no test modifications in this subtask)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo build` succeeds
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo test --lib` passes (all existing tests)
- [ ] `RoutingOptions::default().local == false`
- [ ] `RoutingOptions { local: true, ..Default::default() }.local == true`

### Feature Validation

- [ ] `RoutingOptions` struct has `local: bool` field
- [ ] Field is public (`pub local: bool`)
- [ ] Field has doc comment mentioning "Layer 8"
- [ ] Field is positioned after `global` field
- [ ] `#[derive(Debug, Default)]` macros preserved
- [ ] No clap attributes on the field (not a clap struct)
- [ ] All command consumers can add `local: args.local` to struct initialization

### Code Quality Validation

- [ ] Follows existing pattern for `global` field
- [ ] Doc comment formatting matches other fields
- [ ] Field name is `local` (snake_case)
- [ ] No unused field warnings (commands will use in P1.M1.T3.S1)
- [ ] Derive macros order is preserved: `Debug, Default`

### Documentation & Deployment

- [ ] Doc comment clearly explains Layer 8 purpose
- [ ] No breaking changes to existing functionality
- [ ] Default trait provides expected `false` value

---

## Anti-Patterns to Avoid

- **Don't** modify any other files in this subtask - only `src/staging/router.rs`
- **Don't** add `#[arg(long)]` or any clap-specific attributes - this is NOT a clap struct
- **Don't** modify `route_to_layer()` function (that's P1.M1.T2.S3)
- **Don't** modify `validate_routing_options()` function (that's P1.M1.T2.S2)
- **Don't** update command files to pass the flag (that's P1.M1.T3.S1)
- **Don't** write integration tests for this field (that's P1.M1.T4.S1)
- **Don't** remove or modify existing `#[derive(Debug, Default)]` macros
- **Don't** place field before `global` - maintain consistency
- **Don't** forget the comma after the field declaration
- **Don't** make the field private - must be `pub` for command access
- **Don't** use `Option<bool>` - just `bool` like the other flags
- **Don't** add validation logic in this subtask - that's P1.M1.T2.S2

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Extremely Simple**: Single field addition to existing struct
- **Clear Pattern**: Exact same pattern as existing `global` field
- **Well-Understood**: Rust struct field addition is fundamental language feature
- **No Logic Changes**: Pure data structure modification, no behavior changes
- **No Dependencies**: Standalone change with no external requirements
- **No Tests**: Existing tests validate no regressions; Default trait is automatically correct
- **Exact Specification**: Field name, type, doc comment, position all specified
- **Clear Validation**: Unambiguous success criteria
- **Backward Compatible**: Adding bool field to struct with Default derive breaks nothing

**Implementation is equivalent to adding one field to a struct**:
```rust
/// Target user-local layer (Layer 8)
pub local: bool,
```

This is a foundational subtask that creates the data structure hook for downstream functionality. The implementation risk is minimal because it follows an existing, proven pattern in the same struct and leverages Rust's type system guarantees.

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1T2S1/research/`

- `routing_options_analysis.md` - Complete analysis of RoutingOptions struct, field semantics, and consumers
- `layer_8_specification.md` - Layer 8 (UserLocal) complete specification and integration points

**Key File References**:
- `src/staging/router.rs` - RoutingOptions struct definition (lines 5-16)
- `src/commands/add.rs` - RoutingOptions construction pattern (lines 50-55)
- `src/core/layer.rs` - Layer::UserLocal enum variant (line 28)
- `plan/P1M1T1S1/PRP.md` - Previous work item contract (AddArgs with local field)
