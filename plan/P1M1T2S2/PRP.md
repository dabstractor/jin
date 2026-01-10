# PRP: P1.M1.T2.S2 - Add --local Validation in route_to_layer()

---

## Goal

**Feature Goal**: Add validation logic to `validate_routing_options()` function in `src/staging/router.rs` that prevents `--local` flag from being combined with other layer-specific flags (`--mode`, `--scope`, `--project`, `--global`).

**Deliverable**: Modified `validate_routing_options()` function that:
1. Returns `JinError::Config` when `--local` is combined with any other layer flag
2. Follows the exact same validation pattern as the existing `--global` validation
3. Uses clear error message: `"Cannot combine --local with other layer flags"` or `"--local cannot be combined with other layer flags"`

**Success Definition**:
- `cargo check` passes with zero errors
- `cargo test` passes with new test cases for `--local` validation
- Invalid flag combinations like `--local --mode`, `--local --global`, etc. return proper `JinError::Config` errors
- Valid `--local` usage (alone) passes validation

---

## User Persona

**Target User**: CLI user who uses the `jin add --local` command

**Use Case**: User wants to add a file to Layer 8 (UserLocal) using `jin add <file> --local`

**User Journey**:
1. User runs `jin add .config --local` → validation passes, file routes to Layer 8
2. User accidentally runs `jin add .config --local --mode` → validation fails with clear error message
3. User understands that `--local` must be used alone (like `--global`)

**Pain Points Addressed**:
- Without validation, ambiguous routing behavior could occur if `--local` is combined with other flags
- Clear error messages guide users to correct flag usage
- Consistent with existing `--global` validation pattern users are already familiar with

---

## Why

- **Prevents Ambiguous Routing**: `--local` targets Layer 8 (UserLocal). Combining it with other layer flags creates routing ambiguity.
- **Consistent with `--global` Pattern**: The existing `--global` validation already prevents combining with other flags. `--local` should follow the same pattern.
- **Clear User Feedback**: Early validation with clear error messages is better than cryptic runtime errors later.
- **Layer 8 Independence**: Layer 8 (UserLocal) is designed to be independent of mode/scope/project context. It should not be combined with context-dependent flags.
- **Foundation for P1.M1.T2.S3**: This validation paves the way for the actual routing logic implementation in the next subtask.

---

## What

### User-Visible Behavior

**Valid Usage**:
```bash
jin add .config --local              # PASS - routes to Layer 8
jin add .config                      # PASS - routes to Layer 7 (default)
jin add .config --mode               # PASS - routes to Layer 2
```

**Invalid Usage** (should return error):
```bash
jin add .config --local --mode       # FAIL - incompatible flags
jin add .config --local --scope=foo  # FAIL - incompatible flags
jin add .config --local --project    # FAIL - incompatible flags
jin add .config --local --global     # FAIL - incompatible flags
```

**Error Message Format**:
```
Configuration error: Cannot combine --local with other layer flags
```
or
```
Configuration error: --local cannot be combined with other layer flags
```

### Technical Requirements

1. **Validation Location**: Add validation logic to `validate_routing_options()` function in `src/staging/router.rs`
2. **Validation Pattern**: Mirror the existing `--global` validation pattern (lines 69-74)
3. **Error Type**: Return `JinError::Config(String)` with descriptive message
4. **Validation Logic**: `if options.local && (options.mode || options.scope.is_some() || options.project || options.global)`
5. **Placement**: Add validation immediately after the `--global` validation (after line 74, before `--project` validation)
6. **No Other Changes**: Do NOT modify `route_to_layer()` function (that's P1.M1.T2.S3)

### Success Criteria

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo test --lib staging::router::tests` passes all tests including new ones
- [ ] `--local --mode` combination returns `JinError::Config`
- [ ] `--local --scope=<x>` combination returns `JinError::Config`
- [ ] `--local --project` combination returns `JinError::Config`
- [ ] `--local --global` combination returns `JinError::Config`
- [ ] `--local` alone passes validation
- [ ] Error message is clear and actionable

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for adding a validation rule to an existing function. The implementation is a 4-line addition that mirrors an existing validation pattern in the same function._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Contract from Previous Work Item
- docfile: plan/P1M1T2S1/PRP.md
  why: RoutingOptions will have pub local: bool field - this is the input to validate
  section: "Goal", "Data Models and Structure"
  critical: After P1.M1.T2.S1 completes, RoutingOptions.local will exist

# Target File to Modify
- file: src/staging/router.rs (lines 67-84)
  why: The validate_routing_options() function - add --local validation here
  pattern: Follow the --global validation pattern exactly

# Existing Validation Pattern (MIRROR THIS)
- file: src/staging/router.rs (lines 69-74)
  why: Exact pattern to follow for --local validation
  pattern: |
    // Can't use both --global and other layer flags
    if options.global && (options.mode || options.scope.is_some() || options.project) {
        return Err(JinError::Config(
            "Cannot combine --global with other layer flags".to_string(),
        ));
    }
  critical: Copy this pattern, replace global with local, add global to the exclusion list

# JinError Type Reference
- file: src/core/error.rs (lines 17-19)
  why: JinError::Config variant definition for error construction
  pattern: |
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
  gotcha: Use .to_string() on the message string to convert to owned String

# Layer 8 (UserLocal) Specification
- file: src/core/layer.rs (lines 27-28, 44, 93, 126)
  why: Understanding Layer 8 semantics - independent of mode/scope/project
  critical: Layer 8 (UserLocal) has precedence 8, stored at ~/.jin/local/, refs/jin/layers/local

# Fix Specification
- file: plan/docs/fix_specifications.md (lines 30-32)
  why: Validation rules defined in the fix spec
  section: "Validation Rules"
  critical: "--local cannot be combined with --mode, --scope, --project, or --global"

# Existing Test Pattern
- file: src/staging/router.rs (lines 195-203)
  why: Test pattern for validation - test_validate_global_conflict()
  pattern: |
    #[test]
    fn test_validate_global_conflict() {
        let options = RoutingOptions {
            global: true,
            mode: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }
  note: Create similar test for --local validation

# Import Statement
- file: src/staging/router.rs (line 3)
  why: JinError is imported from crate::core
  pattern: use crate::core::{JinError, Layer, ProjectContext, Result};
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── core/
│   │   ├── error.rs              # JinError::Config variant (line 17-19)
│   │   └── layer.rs              # Layer::UserLocal variant (line 28)
│   ├── staging/
│   │   ├── mod.rs                # Exports validate_routing_options (line 17)
│   │   └── router.rs             # TARGET FILE - validate_routing_options() (lines 67-84)
│   └── commands/
│       └── add.rs                # Calls validate_routing_options() (search for usage)
└── plan/
    ├── P1M1T2S1/
    │   └── PRP.md                # Previous work item (contract - adds local field)
    └── P1M1T2S2/
        ├── PRP.md                # This file
        └── research/             # Research artifacts directory
```

### Desired Codebase Tree After This Subtask

```bash
jin/
├── src/
│   └── staging/
│       └── router.rs             # MODIFIED: Add --local validation to validate_routing_options()
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: This is validation logic, NOT routing logic
// DO NOT modify route_to_layer() function in this subtask (that's P1.M1.T2.S3)
// ONLY modify validate_routing_options() function

// PATTERN: Mirror the --global validation exactly
// The --global validation checks: global && (mode || scope.is_some() || project)
// The --local validation should check: local && (mode || scope.is_some() || project || global)

// ERROR MESSAGE: Choose consistent phrasing
// Option A: "Cannot combine --local with other layer flags" (matches --global wording)
// Option B: "--local cannot be combined with other layer flags" (matches fix spec wording)
// Either is acceptable - stay consistent with existing error messages

// GOTCHA: scope is Option<String>, so use scope.is_some() not just scope
// if options.local && options.scope { ... }  // WRONG - won't compile
// if options.local && options.scope.is_some() { ... }  // CORRECT

// GOTCHA: global must be included in the exclusion list for --local
// Unlike --global, which is checked first and returns early,
// --local validation must also exclude --global because --local can't combine with anything

// PLACEMENT: Add --local validation AFTER --global validation
// Order matters because --global returns early (line 34 in route_to_layer)
// But in validate_routing_options, both validations run - no early return

// DERIVE: RoutingOptions already has #[derive(Debug, Default)] from P1.M1.T2.S1
// No changes needed to the struct definition in this subtask

// TEST: Add test cases mirroring test_validate_global_conflict
// Test all combinations: --local with --mode, --scope, --project, --global
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - this subtask adds validation logic to an existing function.

**Input Contract** (from P1.M1.T2.S1):
```rust
// After P1.M1.T2.S1 completes, RoutingOptions will have:
#[derive(Debug, Default)]
pub struct RoutingOptions {
    pub mode: bool,
    pub scope: Option<String>,
    pub project: bool,
    pub global: bool,
    pub local: bool,  // ADDED IN P1.M1.T2.S1
}
```

**Current Validation Function** (lines 67-84):
```rust
/// Validate routing options for consistency
pub fn validate_routing_options(options: &RoutingOptions) -> Result<()> {
    // Can't use both --global and other layer flags
    if options.global && (options.mode || options.scope.is_some() || options.project) {
        return Err(JinError::Config(
            "Cannot combine --global with other layer flags".to_string(),
        ));
    }

    // Can't use --project without --mode
    if options.project && !options.mode {
        return Err(JinError::Config(
            "--project requires --mode flag".to_string(),
        ));
    }

    Ok(())
}
```

**Modified Validation Function** (add lines after line 74):
```rust
/// Validate routing options for consistency
pub fn validate_routing_options(options: &RoutingOptions) -> Result<()> {
    // Can't use both --global and other layer flags
    if options.global && (options.mode || options.scope.is_some() || options.project) {
        return Err(JinError::Config(
            "Cannot combine --global with other layer flags".to_string(),
        ));
    }

    // Can't use --local with other layer flags
    if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
        return Err(JinError::Config(
            "Cannot combine --local with other layer flags".to_string(),
        ));
    }

    // Can't use --project without --mode
    if options.project && !options.mode {
        return Err(JinError::Config(
            "--project requires --mode flag".to_string(),
        ));
    }

    Ok(())
}
```

### Implementation Tasks

```yaml
Task 1: MODIFY src/staging/router.rs
  - FILE: src/staging/router.rs (lines 67-84)
  - FUNCTION: validate_routing_options()
  - ACTION: Add --local validation after --global validation
  - IMPLEMENT:
    * Add comment: // Can't use --local with other layer flags
    * Add if condition: if options.local && (options.mode || options.scope.is_some() || options.project || options.global)
    * Add error return: Err(JinError::Config("Cannot combine --local with other layer flags".to_string()))
    * Position: After line 74 (after --global validation), before line 76 (before --project validation)
  - PATTERN: Mirror lines 69-74 (--global validation)
  - NAMING: Use --local (with hyphens) in error message for consistency with CLI flag
  - DEPENDENCIES: P1.M1.T2.S1 must be complete (local field must exist)
  - FILES TO MODIFY: src/staging/router.rs (1 file)
  - FILES TO CREATE: None
  - PRESERVE: All existing validation logic unchanged

Task 2: ADD TEST CASES to src/staging/router.rs
  - FILE: src/staging/router.rs (tests module, lines 86-214)
  - ACTION: Add test functions for --local validation
  - IMPLEMENT:
    * test_validate_local_conflict_with_mode()
    * test_validate_local_conflict_with_scope()
    * test_validate_local_conflict_with_project()
    * test_validate_local_conflict_with_global()
    * test_validate_local_alone_passes()
  - PATTERN: Mirror test_validate_global_conflict() (lines 195-203)
  - POSITION: After existing validation tests, before end of tests module
  - DEPENDENCIES: Task 1 must be complete
  - FILES TO MODIFY: src/staging/router.rs (1 file - same file as Task 1)

Task 3: VALIDATE WITH CARGO CHECK
  - COMMAND: cargo check
  - EXPECTED: Zero compilation errors
  - IF FAILS: Read error output, fix typos or syntax issues
  - DEPENDENCIES: Task 1 and Task 2 complete

Task 4: RUN TESTS
  - COMMAND: cargo test --lib staging::router::tests
  - EXPECTED: All tests pass, including new --local validation tests
  - IF FAILS: Debug test failures, fix validation logic or test expectations
  - DEPENDENCIES: Task 3 complete (cargo check passes)
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE TO ADD ==================
// Location: src/staging/router.rs, after line 74, before line 76

    // Can't use --local with other layer flags
    if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
        return Err(JinError::Config(
            "Cannot combine --local with other layer flags".to_string(),
        ));
    }

// ================== CONTEXT FOR ADDITION ==================
// Insert this code block between:
// Line 74: closing brace of --global validation error
// Line 76: Comment for --project validation

// The function after modification should look like:

pub fn validate_routing_options(options: &RoutingOptions) -> Result<()> {
    // Can't use both --global and other layer flags
    if options.global && (options.mode || options.scope.is_some() || options.project) {
        return Err(JinError::Config(
            "Cannot combine --global with other layer flags".to_string(),
        ));
    }

    // Can't use --local with other layer flags  // <-- NEW BLOCK
    if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
        return Err(JinError::Config(
            "Cannot combine --local with other layer flags".to_string(),
        ));
    }  // <-- END NEW BLOCK

    // Can't use --project without --mode
    if options.project && !options.mode {
        return Err(JinError::Config(
            "--project requires --mode flag".to_string(),
        ));
    }

    Ok(())
}

// ================== VALIDATION LOGIC EXPLAINED ==================
// options.local: true when --local flag is set
// options.mode: true when --mode flag is set
// options.scope.is_some(): true when --scope=<value> flag is set
// options.project: true when --project flag is set
// options.global: true when --global flag is set

// The && operator: ALL conditions must be true for error
// The || operator: ANY flag combination triggers error
// If local=true AND (any other flag is true) -> return Error

// ================== TEST CASES TO ADD ==================
// Location: src/staging/router.rs, tests module (after line 213)

    #[test]
    fn test_validate_local_conflict_with_mode() {
        let options = RoutingOptions {
            local: true,
            mode: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_scope() {
        let options = RoutingOptions {
            local: true,
            scope: Some("language:javascript".to_string()),
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_project() {
        let options = RoutingOptions {
            local: true,
            project: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_global() {
        let options = RoutingOptions {
            local: true,
            global: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_alone_passes() {
        let options = RoutingOptions {
            local: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_ok());
    }

// ================== ERROR MESSAGE FORMAT ==================
// JinError::Config(String) produces output like:
// "Configuration error: Cannot combine --local with other layer flags"
//
// The #[error("Configuration error: {0}")] attribute in src/core/error.rs
// automatically prepends "Configuration error: " to the message

// ================== DIFFERENCES FROM --global VALIDATION ==================
// --global excludes: mode, scope, project (3 flags)
// --local excludes: mode, scope, project, global (4 flags)
//
// Reason: --global and --local are mutually exclusive
// Both are "standalone" layer flags that target independent layers
// --global -> Layer 1 (GlobalBase)
// --local -> Layer 8 (UserLocal)
```

### Integration Points

```yaml
MODIFICATIONS:
  - file: src/staging/router.rs
    function: validate_routing_options()
    lines: Insert after line 74
    scope: Add validation if-block, no other changes

NO CHANGES TO:
  - route_to_layer() function (that's P1.M1.T2.S3)
  - RoutingOptions struct (already modified in P1.M1.T2.S1)
  - src/core/error.rs (JinError::Config already exists)
  - src/core/layer.rs (Layer::UserLocal already exists)
  - Command files (validation is called from router, not commands)
  - src/staging/mod.rs (re-exports are automatic)

CALLERS OF validate_routing_options():
  - src/staging/router.rs (internal use)
  - Potentially called from command files (add.rs, mv.rs, rm.rs, import_cmd.rs)
  - No changes needed to callers - validation is transparent
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modification - must pass before proceeding
cargo check                           # Type checking - MUST pass with 0 errors

# Expected: Zero errors. If errors exist, READ output carefully.
# Common issues:
# - Missing semicolon after .to_string()
# - Missing closing brace
# - Wrong field name (local vs Local)
# - scope.is_some() typo

# Format check (optional but recommended)
cargo fmt -- --check                  # Format check

# Expected: No formatting issues

# Clippy check (for code quality)
cargo clippy                          # Lint checking

# Expected: No warnings. If warnings appear, evaluate and fix if warranted.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test router module specifically
cargo test --lib staging::router::tests -v

# Expected: All tests pass, including new --local validation tests

# Run specific new tests
cargo test test_validate_local -v

# Expected: All 5 new tests pass:
# - test_validate_local_conflict_with_mode
# - test_validate_local_conflict_with_scope
# - test_validate_local_conflict_with_project
# - test_validate_local_conflict_with_global
# - test_validate_local_alone_passes

# Full library test suite
cargo test --lib

# Expected: All tests pass (no regressions in other modules)

# Test with output
cargo test --lib -- --nocapture

# Expected: Clean test run with all passing
```

### Level 3: Manual Validation (Behavior Verification)

```bash
# Note: P1.M1.T3.S1 (passing local flag from command) is not yet complete
# So CLI testing won't work yet. But we can verify validation logic directly.

# Create a test script to verify validation behavior
cat > test_local_validation.rs << 'EOF'
use jin::staging::RoutingOptions;
use jin::staging::validate_routing_options;

fn main() {
    // Test 1: --local alone should pass
    let opts = RoutingOptions {
        local: true,
        ..Default::default()
    };
    match validate_routing_options(&opts) {
        Ok(()) => println!("Test 1 PASS: --local alone"),
        Err(e) => println!("Test 1 FAIL: {}", e),
    }

    // Test 2: --local --mode should fail
    let opts = RoutingOptions {
        local: true,
        mode: true,
        ..Default::default()
    };
    match validate_routing_options(&opts) {
        Ok(()) => println!("Test 2 FAIL: Should have rejected --local --mode"),
        Err(e) => println!("Test 2 PASS: {}", e),
    }

    // Test 3: --local --global should fail
    let opts = RoutingOptions {
        local: true,
        global: true,
        ..Default::default()
    };
    match validate_routing_options(&opts) {
        Ok(()) => println!("Test 3 FAIL: Should have rejected --local --global"),
        Err(e) => println!("Test 3 PASS: {}", e),
    }
}
EOF

# Compile and run (after cargo build)
rustc --edition 2021 -L target/debug/deps \
    --extern jin=target/debug/libjin.rlib \
    test_local_validation.rs
./test_local_validation

# Expected output:
# Test 1 PASS: --local alone
# Test 2 PASS: Configuration error: Cannot combine --local with other layer flags
# Test 3 PASS: Configuration error: Cannot combine --local with other layer flags

# Cleanup
rm test_local_validation.rs test_local_validation
```

### Level 4: Integration Testing (System Validation)

```bash
# Note: Full integration testing requires P1.M1.T3.S1 to be complete
# But we can verify the validation function is properly exported

# Check that validate_routing_options is accessible
cargo doc --open --no-deps

# Expected: Documentation opens showing validate_routing_options in staging module

# Verify no breaking changes to other tests
cargo test

# Expected: All existing tests still pass
# Focus on:
# - Router tests in src/staging/router.rs
# - Command tests in src/commands/
# - Any tests that call validate_routing_options
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy` produces no warnings (or only acceptable ones)
- [ ] `cargo test --lib staging::router::tests` passes all tests
- [ ] New test cases cover all --local conflict scenarios
- [ ] `--local` alone passes validation
- [ ] `--local --mode` fails validation
- [ ] `--local --scope=<x>` fails validation
- [ ] `--local --project` fails validation
- [ ] `--local --global` fails validation

### Feature Validation

- [ ] Validation logic matches `--global` pattern exactly
- [ ] Error message is clear and actionable
- [ ] All 4 exclusion flags are checked (mode, scope, project, global)
- [ ] No existing tests are broken by the change
- [ ] Function signature unchanged (takes `&RoutingOptions`, returns `Result<()>`)
- [ ] Placement is correct (after --global validation, before --project validation)

### Code Quality Validation

- [ ] Comment added explaining validation rule
- [ ] Code follows existing formatting and style
- [ ] Test function names follow `test_validate_*` pattern
- [ ] Test cases use `RoutingOptions` struct literal with `..Default::default()`
- [ ] All test assertions use `assert!(result.is_err())` or `assert!(result.is_ok())`

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable names
- [ ] Error message uses flag syntax (--local, not local)
- [ ] No breaking changes to existing functionality
- [ ] Validation is transparent to command callers

---

## Anti-Patterns to Avoid

- **Don't** modify `route_to_layer()` function in this subtask (that's P1.M1.T2.S3)
- **Don't** change the `JinError` type or add new error variants
- **Don't** modify command files (add.rs, mv.rs, rm.rs, import_cmd.rs)
- **Don't** skip adding test cases for the new validation
- **Don't** use `options.scope` instead of `options.scope.is_some()` - won't compile
- **Don't** forget to include `options.global` in the exclusion list
- **Don't** place validation before the `--global` validation - maintain order
- **Don't** use different error message format than existing validations
- **Don't** add `#[test]` attribute outside of the `#[cfg(test)]` module
- **Don't** forget the `.to_string()` call on the error message
- **Don't** make validation too permissive (must reject ANY combination with other flags)
- **Don't** make validation too strict (must allow `--local` alone)

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Extremely Simple**: 4-line addition to existing validation function
- **Clear Pattern**: Exact mirror of existing `--global` validation in same function
- **Well-Understood**: Rust if-else validation logic is fundamental
- **No Logic Complexity**: Pure validation, no state changes or side effects
- **No External Dependencies**: Uses existing `JinError::Config` type
- **Testable**: Validation logic is easily testable with unit tests
- **Exact Specification**: Error message, condition, placement all specified
- **Clear Success Criteria**: Unambiguous test cases define correctness
- **Isolated Change**: No ripple effects to other parts of codebase

**Implementation is equivalent to adding one if-block to a function**:
```rust
if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
    return Err(JinError::Config(
        "Cannot combine --local with other layer flags".to_string(),
    ));
}
```

This is a straightforward validation addition that prevents invalid flag combinations. The implementation risk is minimal because it follows an existing, proven pattern in the same function and leverages Rust's type system guarantees.

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1T2S2/research/`

**Key File References**:
- `src/staging/router.rs` - `validate_routing_options()` function (lines 67-84)
- `src/core/error.rs` - `JinError::Config` variant (lines 17-19)
- `src/core/layer.rs` - `Layer::UserLocal` enum variant (line 28)
- `plan/P1M1T2S1/PRP.md` - Previous work item contract (adds local field to RoutingOptions)
- `plan/docs/fix_specifications.md` - Fix specification with validation rules (lines 30-32)
