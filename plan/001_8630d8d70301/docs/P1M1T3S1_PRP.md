# PRP: P1.M1.T3.S1 - Pass local flag in add command execute()

---

## Goal

**Feature Goal**: Ensure the `local` flag value from `AddArgs` is passed through to `RoutingOptions` in the `execute()` function, enabling `route_to_layer()` to receive the `--local` flag value.

**Deliverable**: Verified and documented implementation where `local: args.local` is present in the RoutingOptions construction within `src/commands/add.rs execute()` function.

**Success Definition**:
- Line 55 in `src/commands/add.rs` contains `local: args.local,`
- `cargo check` passes with zero errors
- `cargo test` passes all tests
- The `--local` flag value flows from CLI argument → AddArgs → RoutingOptions → route_to_layer()

---

## User Persona

**Target User**: CLI user who wants to add machine-specific configuration files to Layer 8 (UserLocal) using the `--local` flag

**Use Case**: User runs `jin add <file> --local` to add a file to Layer 8 storage at `~/.jin/local/`

**User Journey**:
1. User executes: `jin add .config/settings.json --local`
2. Clap parses `--local` flag into `AddArgs.local = true`
3. `execute()` function constructs `RoutingOptions` with `local: args.local`
4. `route_to_layer()` receives `options.local = true`
5. Routing logic returns `Layer::UserLocal`
6. File is staged for Layer 8 storage

**Pain Points Addressed**:
- Without this wiring, the `--local` flag would be parsed but never used
- User would see no behavior change when using `--local` flag
- Layer 8 would remain inaccessible via CLI

---

## Why

- **Completes the Wiring Chain**: P1.M1.T1.S1 added the CLI flag, P1.M1.T2.S1 added the field to RoutingOptions, P1.M1.T2.S3 added the routing logic. This subtask connects the flag to the routing logic.
- **Enables End-to-End Flow**: The `--local` flag value must travel from CLI parsing to routing decision
- **Follows Established Pattern**: The wiring pattern is identical to other flags (mode, scope, project, global)
- **Minimal Implementation**: Single line addition using an existing, well-understood pattern
- **Zero Risk**: The implementation is a straightforward field assignment with no logic changes

---

## What

### User-Visible Behavior

**After Implementation**:
```bash
# Add file to Layer 8 (UserLocal) - ~/.jin/local/
jin add .config/settings.json --local
# File is staged for Layer 8 storage

# The flag value flows through the entire chain:
# CLI → AddArgs.local → RoutingOptions.local → route_to_layer() → Layer::UserLocal
```

### Technical Requirements

1. **File to Modify**: `src/commands/add.rs` (line 55 in RoutingOptions construction)
2. **Code to Add**: `local: args.local,` following the exact pattern of other fields
3. **Placement**: In the `RoutingOptions` struct literal, after `global: args.global,` (line 54)
4. **Pattern**: Follow the existing field-by-field assignment pattern
5. **No Logic Changes**: This is purely data passing - no conditional logic or validation

### Success Criteria

- [ ] `local: args.local,` is present at line 55 in `src/commands/add.rs`
- [ ] `cargo check` passes with 0 errors
- [ ] `cargo test` passes all tests
- [ ] The pattern matches other fields (mode, scope, project, global)
- [ ] No unintended side effects or regressions

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for a single-line code addition that follows an existing pattern in the same function. The implementation is already complete in the codebase, and this PRP serves as verification and documentation._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Contract from P1.M1.T1.S1 (AddArgs local field - Complete)
- docfile: plan/P1M1T1S1/PRP.md
  why: AddArgs.local field already exists
  section: "Goal", "Data Models and Structure"
  critical: pub local: bool field is present in AddArgs struct at src/cli/args.rs:29
  output: "AddArgs struct has local field that Clap populates from --local flag"

# Contract from P1.M1.T2.S1 (RoutingOptions local field - Complete)
- docfile: plan/P1M1T2S1/PRP.md
  why: RoutingOptions.local field already exists
  section: "Goal", "Data Models and Structure"
  critical: pub local: bool field is present in RoutingOptions struct at src/staging/router.rs:17
  output: "RoutingOptions struct has local field to receive the flag value"

# Contract from P1.M1.T2.S3 (Routing logic - In Progress)
- docfile: plan/P1M1.T2S3/PRP.md
  why: Routing logic for --local flag is being implemented
  section: "Goal", "Implementation Blueprint"
  critical: route_to_layer() checks options.local and returns Layer::UserLocal
  output: "When options.local == true, routing returns Layer::UserLocal"

# Target File - execute() Function
- file: src/commands/add.rs (lines 34-129)
  why: The execute() function containing RoutingOptions construction
  pattern: Direct field-by-field assignment from args to options
  gotcha: scope field requires .clone() because it's Option<String>

# RoutingOptions Construction Site (Lines 49-56)
- file: src/commands/add.rs (lines 49-56)
  why: EXACT location where local field needs to be added
  pattern: |
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
        local: args.local,  // <-- THIS LINE (line 55)
    };
  critical: Follow exact pattern of other fields

# AddArgs Struct Definition
- file: src/cli/args.rs (lines 6-30)
  why: Source struct with local field from P1.M1.T1.S1
  pattern: |
    #[derive(Args, Debug)]
    pub struct AddArgs {
        pub files: Vec<String>,
        #[arg(long)]
        pub mode: bool,
        #[arg(long)]
        pub scope: Option<String>,
        #[arg(long)]
        pub project: bool,
        #[arg(long)]
        pub global: bool,
        #[arg(long)]
        pub local: bool,  // <-- Field exists from P1.M1.T1.S1
    }
  gotcha: local field is bool with #[arg(long)] attribute

# RoutingOptions Struct Definition
- file: src/staging/router.rs (lines 6-18)
  why: Destination struct with local field from P1.M1.T2.S1
  pattern: |
    #[derive(Debug, Default)]
    pub struct RoutingOptions {
        pub mode: bool,
        pub scope: Option<String>,
        pub project: bool,
        pub global: bool,
        pub local: bool,  // <-- Field exists from P1.M1.T2.S1
    }
  critical: Field names are identical between AddArgs and RoutingOptions

# route_to_layer() Function
- file: src/staging/router.rs (lines 31-70)
  why: Function that receives RoutingOptions and uses the local field
  pattern: |
    pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
        if options.local {
            return Ok(Layer::UserLocal);
        }
        // ... rest of routing logic
    }
  critical: Function checks options.local to determine routing

# Rust CLI Flag Passing Research
- docfile: plan/P1M1T3S1/research/rust_clap_patterns.md
  why: Comprehensive research on clap flag passing patterns
  section: "Current Codebase Patterns", "Common Patterns for Struct Initialization"
  critical: Direct field assignment is the current pattern used in this codebase
  note: 25+ curated links to clap documentation and community discussions

# System Architecture
- file: plan/docs/system_context.md
  why: Complete Layer 8 specification and routing table context
  section: "Layer Routing (jin add flags)"
  critical: Layer 8 = UserLocal, precedence 8, ~/.jin/local/

# Existing Implementation (Git History)
- commit: 2d85dd3315c2179a7f261e2a80191c117a24792f
  why: Proof that the implementation is already complete
  message: "feat: Add local field to RoutingOptions struct for Layer 8 routing support"
  critical: Line 55 already contains `local: args.local,`
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── cli/
│   │   └── args.rs                # AddArgs struct with local field (line 29)
│   ├── commands/
│   │   └── add.rs                # TARGET FILE - execute() function (lines 34-129)
│   │                              # RoutingOptions construction (lines 49-56)
│   │                              # local: args.local at line 55
│   ├── staging/
│   │   ├── mod.rs                 # Exports RoutingOptions and route_to_layer
│   │   └── router.rs              # RoutingOptions struct (line 17)
│   │                              # route_to_layer() function (lines 31-70)
│   └── core/
│       ├── layer.rs               # Layer::UserLocal enum variant
│       ├── config.rs              # ProjectContext type
│       └── error.rs               # Result type
└── plan/
    ├── P1M1T1S1/                   # Previous: Add local field to AddArgs
    ├── P1M1T2S1/                   # Previous: Add local field to RoutingOptions
    ├── P1M1T2S2/                   # Parallel: Add --local validation
    ├── P1M1T2S3/                   # Parallel: Add --local routing case
    ├── P1M1T3S1/                   # This work item
    │   └── research/
    │       └── rust_clap_patterns.md  # Rust clap research (25+ links)
    └── docs/
        └── system_context.md       # Layer 8 specification
```

### Desired Codebase Tree After This Subtask

```bash
# NO CHANGES - Implementation is already complete at line 55
jin/
├── src/
│   └── commands/
│       └── add.rs                # UNCHANGED - local: args.local already at line 55
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: This is a simple field assignment, not logic
// No conditional statements, no validation, no transformation
// Just passing the bool value from one struct to another

// PATTERN: Direct field assignment (current codebase pattern)
// All other fields use the same pattern:
// mode: args.mode,
// scope: args.scope.clone(),
// project: args.project,
// global: args.global,
// local: args.local,  // <-- Same pattern

// GOTCHA: scope field requires .clone()
// scope: args.scope.clone(),  // <-- Must clone Option<String>
// But local is bool (implements Copy), so no clone needed

// FIELD NAMES: Identical in both structs
// AddArgs.local and RoutingOptions.local
// No name transformation needed

// TYPE MATCH: Both are bool
// AddArgs.local: bool
// RoutingOptions.local: bool
// No type conversion needed

// PLACEMENT: After global field, before closing brace
// let options = RoutingOptions {
//     mode: args.mode,
//     scope: args.scope.clone(),
//     project: args.project,
//     global: args.global,
//     local: args.local,  // <-- HERE (line 55)
// };

// VALIDATION: No validation needed here
// validate_routing_options() is called AFTER construction (line 57)
// This function just passes the value through

// ROUTING: No routing logic here
// route_to_layer() is called AFTER validation (line 60)
// This function just prepares the options

// BOOL BEHAVIOR: bool implements Copy trait
// No need to clone like Option<String>
// The value is copied, not moved

// CLAP INTEGRATION: Clap already parsed the flag
// AddArgs.local is already populated by clap
// This function just uses the value

// TEST IMPACT: No new tests needed
// Existing tests already cover the execute() function
// The change is purely internal data flow

// PARALLEL EXECUTION: Safe with P1.M1.T2.S3
// P1.M1.T2.S3 adds routing logic in route_to_layer()
// This subtask passes the flag value to that function
// Both can work in parallel - no direct dependency

// ALREADY IMPLEMENTED: Commit 2d85dd3
// The line `local: args.local,` was added in commit 2d85dd3
// This PRP serves as verification and documentation

// CONSISTENCY: Must match pattern of other fields
// All fields use: field_name: args.field_name
// No transformation, no conditional logic, just assignment
```

---

## Implementation Blueprint

### Data Models and Structure

**Input Contract** (from P1.M1.T1.S1 - Complete):
```rust
// AddArgs struct in src/cli/args.rs (lines 6-30)
#[derive(Args, Debug)]
pub struct AddArgs {
    pub files: Vec<String>,
    #[arg(long)]
    pub mode: bool,
    #[arg(long)]
    pub scope: Option<String>,
    #[arg(long)]
    pub project: bool,
    #[arg(long)]
    pub global: bool,
    #[arg(long)]
    pub local: bool,  // <-- ADDED IN P1.M1.T1.S1
}
```

**Output Contract** (from P1.M1.T2.S1 - Complete):
```rust
// RoutingOptions struct in src/staging/router.rs (lines 6-18)
#[derive(Debug, Default)]
pub struct RoutingOptions {
    pub mode: bool,
    pub scope: Option<String>,
    pub project: bool,
    pub global: bool,
    pub local: bool,  // <-- ADDED IN P1.M1.T2.S1
}
```

**Current execute() Function** (lines 34-129):
```rust
pub fn execute(args: AddArgs) -> Result<()> {
    // 1. Validate files
    // 2. Load project context
    // 3. Build and validate routing options
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
        local: args.local,  // <-- LINE 55 - ALREADY PRESENT
    };
    validate_routing_options(&options)?;

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;

    // ... rest of function
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY EXISTING IMPLEMENTATION
  - FILE: src/commands/add.rs (line 55)
  - ACTION: Confirm `local: args.local,` is present
  - EXPECTED: Line 55 contains `local: args.local,`
  - STATUS: Already complete (commit 2d85dd3)
  - DEPENDENCIES: None

Task 2: VALIDATE WITH CARGO CHECK
  - COMMAND: cargo check
  - EXPECTED: Zero compilation errors
  - IF FAILS: Read error output, check for typos or syntax issues
  - DEPENDENCIES: Task 1 verified

Task 3: RUN TESTS
  - COMMAND: cargo test
  - EXPECTED: All tests pass
  - IF FAILS: Debug test failures, check for regressions
  - DEPENDENCIES: Task 2 complete (cargo check passes)

Task 4: VERIFY DATA FLOW
  - ACTION: Trace --local flag through the entire chain
  - CHAIN: CLI → AddArgs.local → RoutingOptions.local → route_to_layer()
  - EXPECTED: Each step passes the bool value correctly
  - DEPENDENCIES: Task 3 complete (tests pass)

Task 5: DOCUMENT VERIFICATION
  - ACTION: Update task status to Complete
  - DOCUMENT: Verification findings in PRP
  - DEPENDENCIES: Task 4 complete (data flow verified)
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE AT LINE 55 ==================
// Location: src/commands/add.rs, line 55

    local: args.local,

// ================== FULL CONTEXT (Lines 49-60) ==================

    // 3. Build and validate routing options
    let options = RoutingOptions {
        mode: args.mode,           // Line 51 - bool field (no clone needed)
        scope: args.scope.clone(), // Line 52 - Option<String> (clone required)
        project: args.project,     // Line 53 - bool field (no clone needed)
        global: args.global,       // Line 54 - bool field (no clone needed)
        local: args.local,         // Line 55 - bool field (no clone needed) <-- TARGET
    };
    validate_routing_options(&options)?;  // Line 57 - validates options

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;  // Line 60 - uses options

// ================== PATTERN EXPLANATION ==================
//
// All fields follow the same pattern: field_name: args.field_name
//
// bool fields (mode, project, global, local):
//   - No .clone() needed because bool implements Copy
//   - The value is copied from args to options
//
// Option<String> field (scope):
//   - .clone() required because String doesn't implement Copy
//   - We clone the Option wrapper, which copies the Some/None
//   - and clones the String inside if it's Some
//
// ================== WHY THIS WORKS ==================
//
// 1. AddArgs is parsed by clap from CLI arguments
//    When user runs: jin add file.txt --local
//    clap sets: args.local = true
//
// 2. execute() receives the populated AddArgs
//    pub fn execute(args: AddArgs) -> Result<()>
//
// 3. RoutingOptions is constructed from AddArgs
//    local: args.local  // copies the bool value
//
// 4. validate_routing_options() checks for invalid combinations
//    e.g., --local --mode is rejected
//
// 5. route_to_layer() receives the validated options
//    pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext)
//
// 6. Routing logic checks options.local
//    if options.local { return Ok(Layer::UserLocal); }
//
// 7. Layer is determined and file is staged
//    target_layer = Layer::UserLocal
//    file is staged for ~/.jin/local/

// ================== VERIFICATION CHECKLIST ==================
//
// To verify the implementation is correct:
//
// 1. Check line 55 exists:
//    grep "local: args.local" src/commands/add.rs
//
// 2. Check field types match:
//    AddArgs.local: bool
//    RoutingOptions.local: bool
//
// 3. Check placement is correct:
//    After global field (line 54)
//    Before closing brace (line 56)
//
// 4. Check compilation:
//    cargo check
//
// 5. Check tests:
//    cargo test
//
// 6. Check data flow:
//    - AddArgs.local is populated by clap
//    - RoutingOptions.local receives the value
//    - route_to_layer() uses the value
//
// 7. Check pattern consistency:
//    Matches mode, project, global fields
//    Same field names, same types, same pattern
```

### Integration Points

```yaml
INPUT:
  - file: src/cli/args.rs
    struct: AddArgs (line 29)
    field: pub local: bool
    source: Clap parses --local flag

PROCESSING:
  - file: src/commands/add.rs
    function: execute() (lines 34-129)
    action: Construct RoutingOptions (lines 50-56)
    line_55: local: args.local,

VALIDATION:
  - file: src/commands/add.rs
    function: validate_routing_options() (line 57)
    purpose: Validates no invalid flag combinations

ROUTING:
  - file: src/commands/add.rs
    function: route_to_layer() (line 60)
    input: &RoutingOptions with local field
    output: Result<Layer> (Layer::UserLocal when local==true)

DOWNSTREAM:
  - function: route_to_layer() in src/staging/router.rs
    logic: if options.local { return Ok(Layer::UserLocal); }
    dependency: P1.M1.T2.S3 (routing logic implementation)

NO CHANGES TO:
  - AddArgs struct (already has local field from P1.M1.T1.S1)
  - RoutingOptions struct (already has local field from P1.M1.T2.S1)
  - route_to_layer() function (will be modified in P1.M1.T2.S3)
  - validate_routing_options() function (will be modified in P1.M1.T2.S2)
  - Test files (no new tests needed for this simple wiring)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run verification - must pass
cargo check                           # Type checking - MUST pass with 0 errors

# Expected: Zero errors. If errors exist, READ output carefully.
# Common issues:
# - Missing comma after local: args.local
# - Wrong field name (local vs Local)
# - Wrong args reference (arg vs args)

# Format check (optional but recommended)
cargo fmt -- --check                  # Format check

# Expected: No formatting issues

# Clippy check (for code quality)
cargo clippy                          # Lint checking

# Expected: No warnings about the local field assignment
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test add command specifically
cargo test --lib commands::add::tests -v

# Expected: All tests pass, including execute() tests

# Test router module (receives the options)
cargo test --lib staging::router::tests -v

# Expected: All routing tests pass

# Full library test suite
cargo test --lib

# Expected: All tests pass (no regressions in other modules)
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual CLI test (after full implementation is complete)
# Note: This requires P1.M1.T2.S3 to be complete

# Create a test file
echo "test content" > /tmp/test_local.txt

# Test --local flag
jin add /tmp/test_local.txt --local

# Expected output:
# "Added /tmp/test_local.txt to Layer 8 (UserLocal)"

# Verify the file was staged correctly
# (This requires the full routing implementation from P1.M1.T2.S3)

# Test invalid flag combinations (validation from P1.M1.T2.S2)
jin add /tmp/test_local.txt --local --mode

# Expected: Error message about conflicting flags

# Cleanup
rm /tmp/test_local.txt
```

### Level 4: Data Flow Verification

```bash
# Verify the complete data flow chain

# Step 1: Check AddArgs has local field
grep -n "pub local: bool" src/cli/args.rs

# Expected: Line 29 shows "pub local: bool,"

# Step 2: Check RoutingOptions has local field
grep -n "pub local: bool" src/staging/router.rs

# Expected: Line 17 shows "pub local: bool,"

# Step 3: Check execute() passes local field
grep -n "local: args.local" src/commands/add.rs

# Expected: Line 55 shows "local: args.local,"

# Step 4: Check route_to_layer() uses local field
grep -A 2 "options.local" src/staging/router.rs | head -5

# Expected:
# if options.local {
#     return Ok(Layer::UserLocal);
# }

# Step 5: Verify compilation
cargo check

# Expected: Finished with 0 errors
```

---

## Final Validation Checklist

### Technical Validation

- [ ] Line 55 in `src/commands/add.rs` contains `local: args.local,`
- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo test --lib` passes all tests
- [ ] `cargo test --lib commands::add::tests` passes all add command tests
- [ ] Field name matches between `AddArgs.local` and `RoutingOptions.local`
- [ ] Field type matches (both are `bool`)
- [ ] Placement is correct (after `global`, before closing brace)
- [ ] Pattern matches other fields (`mode`, `project`, `global`)

### Feature Validation

- [ ] `--local` flag value flows from CLI to routing logic
- [ ] Data flow chain is complete: CLI → AddArgs → RoutingOptions → route_to_layer()
- [ ] No conditional logic added (simple field assignment)
- [ ] No validation added here (validation is in `validate_routing_options()`)
- [ ] No routing logic added here (routing is in `route_to_layer()`)
- [ ] Existing tests continue to pass
- [ ] No unintended side effects

### Code Quality Validation

- [ ] Code follows existing field-by-field assignment pattern
- [ ] No unnecessary `.clone()` on bool field (bool implements Copy)
- [ ] Consistent with other flag fields (mode, project, global)
- [ ] Clear and readable (direct field assignment)
- [ ] No magic values or transformations
- [ ] Self-documenting code (field names match)

### Documentation & Deployment

- [ ] Implementation is already complete (commit 2d85dd3)
- [ ] This PRP documents the implementation for verification
- [ ] No breaking changes to existing functionality
- [ ] Wiring is transparent to both user and caller
- [ ] Pattern is maintainable and follows Rust idioms

---

## Anti-Patterns to Avoid

- **Don't** add conditional logic in the execute() function (this is just data passing)
- **Don't** add `.clone()` to the bool field (bool implements Copy trait)
- **Don't** change field names between structs (names must match)
- **Don't** add validation here (use `validate_routing_options()`)
- **Don't** add routing logic here (use `route_to_layer()`)
- **Don't** forget the comma after `local: args.local,`
- **Don't** place the field before other existing fields (maintain order)
- **Don't** use `args.Local` or `args.LOCAL` (field name is `local`)
- **Don't** create a new function or method (use existing pattern)
- **Don't** add new tests for this simple wiring (existing tests cover it)

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Already Complete**: The implementation is already present at line 55 (commit 2d85dd3)
- **Extremely Simple**: Single line of code - one field assignment
- **Clear Pattern**: Exact copy of existing pattern for mode, project, global fields
- **Well-Understood**: Direct field assignment is fundamental Rust
- **No Logic Complexity**: Pure data passing with no transformations
- **No External Dependencies**: Uses only existing structs and fields
- **Testable**: Existing tests cover the execute() function
- **Exact Specification**: Line number, code, and pattern all specified
- **Clear Success Criteria**: Line 55 check + cargo check + cargo test
- **Isolated Change**: No ripple effects to other parts of codebase

**Implementation is equivalent to adding one line to a struct literal**:
```rust
local: args.local,
```

This is a straightforward field assignment that follows an established pattern in the same code block. The implementation risk is minimal because it uses the exact same pattern as the `mode`, `project`, and `global` fields, which are already proven to work correctly.

**Current Status**: ✅ COMPLETE (verified at line 55 in commit 2d85dd3)

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1T3S1/research/`

**Key File References**:
- `src/commands/add.rs` - execute() function with RoutingOptions construction (line 55)
- `src/cli/args.rs` - AddArgs struct with local field (line 29)
- `src/staging/router.rs` - RoutingOptions struct with local field (line 17)
- `plan/P1M1T1S1/PRP.md` - Previous work item (adds local field to AddArgs)
- `plan/P1M1T2S1/PRP.md` - Previous work item (adds local field to RoutingOptions)
- `plan/P1M1T2S2/PRP.md` - Parallel work item (adds validation for --local flag)
- `plan/P1M1T2S3/PRP.md` - Parallel work item (adds routing logic for --local flag)
- `plan/P1M1T3S1/research/rust_clap_patterns.md` - Rust clap research (25+ curated links)
- `plan/docs/system_context.md` - Layer 8 specification and routing table

**External References** (from research document):
- [Clap Derive Documentation](https://docs.rs/clap/latest/clap/_derive/index.html)
- [Clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [GitHub: Best practice to access arguments #5258](https://github.com/clap-rs/clap/discussions/5258)
- [StackOverflow: Boolean flag defaulted to true](https://stackoverflow.com/questions/77771008/how-do-i-create-a-rust-clap-derive-boolean-flag-that-is-defaulted-to-true-and-ca)
- [Rust Users Forum: Boolean arguments in clap](https://users.rust-lang.org/t/boolean-arguments-in-clap/125508)
- Plus 20+ more curated links in the research document
