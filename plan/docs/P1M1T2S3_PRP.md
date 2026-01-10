# PRP: P1.M1.T2.S3 - Add --local Routing Case in route_to_layer()

---

## Goal

**Feature Goal**: Add routing logic to `route_to_layer()` function in `src/staging/router.rs` that returns `Layer::UserLocal` when the `--local` flag is set.

**Deliverable**: Modified `route_to_layer()` function that:
1. Checks `options.local` flag immediately after `options.global` check
2. Returns `Ok(Layer::UserLocal)` when `local == true`
3. Follows the same early-return pattern as the `--global` routing case
4. Includes comprehensive test coverage for the new routing behavior

**Success Definition**:
- `cargo check` passes with zero errors
- `cargo test --lib staging::router::tests` passes all tests including new `--local` routing tests
- `route_to_layer()` returns `Layer::UserLocal` when `options.local == true`
- All existing tests continue to pass (no regressions)
- The routing follows the same pattern as `--global` (early return, no context needed)

---

## User Persona

**Target User**: CLI user who wants to add machine-specific configuration files to Layer 8 (UserLocal)

**Use Case**: User runs `jin add <file> --local` to add a file to Layer 8, which is stored at `~/.jin/local/` and has precedence 8 (second highest, only below WorkspaceActive)

**User Journey**:
1. User wants to add a machine-specific config (e.g., local IDE settings)
2. User runs `jin add .config/settings.json --local`
3. Jin validates the flag combination (P1.M1.T2.S2 ensures no conflicts)
4. `route_to_layer()` determines the target layer as `Layer::UserLocal`
5. File is staged for Layer 8 storage
6. On apply, file ends up in `~/.jin/local/` with proper precedence

**Pain Points Addressed**:
- Layer 8 (UserLocal) is currently inaccessible via CLI - no `--local` flag routing exists
- Users cannot add machine-specific configuration overrides without direct Git manipulation
- Inconsistency: All other layers are accessible via CLI flags, but Layer 8 is not

---

## Why

- **Completes --local Flag Implementation**: P1.M1.T1.S1 added the CLI flag, P1.M1.T2.S1 added the field to RoutingOptions, P1.M1.T2.S2 added validation. This subtask completes the routing logic.
- **Makes Layer 8 Accessible**: Users can finally route files to Layer 8 (UserLocal) via `jin add <file> --local`
- **Maintains Architectural Consistency**: Follows the exact same pattern as `--global` routing (early return, no context needed, independent layer)
- **Simple Implementation**: The routing logic is a 3-line addition using an established pattern in the same function
- **Enables Downstream Tasks**: P1.M1.T3.S1 (wiring the flag through add command) depends on this routing being in place
- **Low Risk**: Early return pattern is isolated, well-tested, and follows existing conventions

---

## What

### User-Visible Behavior

**Valid Usage**:
```bash
# Add file to Layer 8 (UserLocal) - ~/.jin/local/
jin add .config/settings.json --local

# Layer 8 has precedence 8, overrides layers 1-7
# Stored at refs/jin/layers/local in Git
# Independent of mode/scope/project context
```

**Routing Behavior**:
| Command | Target Layer | Storage Path | Precedence |
|---------|--------------|--------------|------------|
| `jin add <file> --local` | UserLocal (8) | `~/.jin/local/` | 8 |

**Invalid Flag Combinations** (rejected by validation from P1.M1.T2.S2):
```bash
jin add <file> --local --mode       # FAIL - validation error
jin add <file> --local --scope=foo  # FAIL - validation error
jin add <file> --local --project    # FAIL - validation error
jin add <file> --local --global     # FAIL - validation error
```

### Technical Requirements

1. **Function to Modify**: `route_to_layer()` in `src/staging/router.rs` (lines 31-65)
2. **Routing Logic**: Add `if options.local { return Ok(Layer::UserLocal); }` after `--global` check
3. **Placement**: After line 35 (after `--global` routing), before line 37 (before mode checks)
4. **Pattern**: Follow exact same pattern as `--global` routing (early return, no context needed)
5. **No Context Required**: Layer 8 (UserLocal) is independent - doesn't need mode/scope/project
6. **Tests Required**: Add `test_route_local()` function following existing test pattern

### Success Criteria

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo test --lib staging::router::tests` passes all tests
- [ ] `test_route_local()` test passes - routes to Layer::UserLocal
- [ ] Existing tests continue to pass (no regressions)
- [ ] `--local` flag routes to correct layer with correct precedence
- [ ] Code follows existing early-return pattern
- [ ] Test follows existing test pattern (`..Default::default()`, exact layer assertion)

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for adding a routing case to an existing function. The implementation is a 3-line addition that mirrors an existing routing pattern in the same function. All dependencies, patterns, and test structures are documented._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Contract from P1.M1.T2.S2 (Validation - assumed complete)
- docfile: plan/P1M1T2S2/PRP.md
  why: Validation logic is already in place - --local flag conflicts are caught
  section: "Goal", "Implementation Blueprint"
  critical: validate_routing_options() already prevents --local + other flag combinations
  output: "Validation ensures --local is never combined with other flags when route_to_layer() is called"

# Contract from P1.M1.T2.S1 (Data Structure - assumed complete)
- docfile: plan/P1M1T2S1/PRP.md
  why: RoutingOptions.local field already exists
  section: "Data Models and Structure"
  critical: pub local: bool field is present in RoutingOptions struct

# Target File to Modify
- file: src/staging/router.rs (lines 31-65)
  why: The route_to_layer() function - add --local routing case here
  pattern: Follow the --global routing pattern exactly (lines 32-35)
  gotcha: Add routing after --global, before mode checks

# Existing Routing Pattern (MIRROR THIS)
- file: src/staging/router.rs (lines 32-35)
  why: Exact pattern to follow for --local routing
  pattern: |
    // Global flag takes precedence
    if options.global {
        return Ok(Layer::GlobalBase);
    }
  critical: Use same early-return pattern, just different Layer variant

# Layer Enum - UserLocal Variant
- file: src/core/layer.rs (lines 27-28, 44, 93, 126)
  why: Layer::UserLocal enum variant specification
  pattern: |
    /// Layer 8: Machine-only overlays (~/.jin/local/)
    UserLocal,
  critical: Precedence 8, stored at ~/.jin/local/, refs/jin/layers/local

# System Architecture - Layer 8 Specification
- file: plan/docs/system_context.md (lines 13-24, 143-144)
  why: Complete Layer 8 semantics and routing table context
  section: "The 9-Layer System", "Layer Routing (jin add flags)"
  critical: Layer 8 = UserLocal, precedence 8, ~/.jin/local/, refs/jin/layers/local

# Test Pattern Reference
- docfile: plan/P1M1T2S3/research/test_patterns.md
  why: Exact test patterns used in this codebase
  section: "Test Structure Components", "Successful Route Tests Pattern"
  critical: Use ..Default::default(), exact layer assertion, unwrap() for success

# Existing Test for Global Routing
- file: src/staging/router.rs (lines 120-129)
  why: Test pattern to follow for --local routing test
  pattern: |
    #[test]
    fn test_route_global() {
        let options = RoutingOptions {
            global: true,
            ..Default::default()
        };
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::GlobalBase);
    }
  note: Create similar test for --local routing

# Rust Routing Patterns Research
- docfile: plan/P1M1T2S3/research/rust_routing_patterns.md
  why: Rust best practices for if-else routing chains
  section: "Pattern for Adding --local Routing", "Best Practices Identified"
  critical: Use early return, no context needed, same as --global pattern

# Fix Specification
- file: plan/docs/fix_specifications.md
  why: Original fix specification that defined this work
  section: Search for "--local flag" or "Layer 8"
  critical: Defines the complete --local flag implementation
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── core/
│   │   ├── error.rs              # JinError type (not modified in this subtask)
│   │   ├── config.rs             # ProjectContext type (not modified in this subtask)
│   │   └── layer.rs              # Layer enum with UserLocal variant (line 28)
│   ├── staging/
│   │   ├── mod.rs                # Exports route_to_layer (line 17)
│   │   └── router.rs             # TARGET FILE - route_to_layer() (lines 31-65)
│   └── commands/
│       └── add.rs                # Will use route_to_layer() in P1.M1.T3.S1
└── plan/
    ├── P1M1T2S1/
    │   └── PRP.md                # Previous work item - adds local field to RoutingOptions
    ├── P1M1T2S2/
    │   └── PRP.md                # Parallel work item - adds validation for --local flag
    ├── P1M1T2S3/
    │   ├── PRP.md                # This file
    │   └── research/             # Research artifacts
    │       ├── test_patterns.md          # Test patterns from codebase analysis
    │       └── rust_routing_patterns.md   # Rust best practices for routing
    └── docs/
        ├── system_context.md     # Layer 8 specification and routing table
        └── fix_specifications.md # Original fix specification
```

### Desired Codebase Tree After This Subtask

```bash
jin/
├── src/
│   └── staging/
│       └── router.rs             # MODIFIED: Add --local routing case to route_to_layer()
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: This is routing logic, NOT validation logic
// Validation is already handled by P1.M1.T2.S2 (validate_routing_options function)
// route_to_layer() assumes valid input - no need to check for flag conflicts

// PATTERN: Use early return like --global routing
// The --global check returns early, and --local should do the same
// Both flags target independent layers that don't need context

// PLACEMENT: Add --local routing AFTER --global, BEFORE mode checks
// Order matters for precedence:
// 1. --global (Layer 1) - highest precedence standalone flag
// 2. --local (Layer 8) - second highest precedence standalone flag
// 3. --mode (Layer 2-5) - requires context
// 4. --scope (Layer 3, 4, 6) - may require context
// 5. Default (Layer 7) - ProjectBase

// NO CONTEXT NEEDED: Layer 8 (UserLocal) is independent
// Unlike --mode which requires context.require_mode()
// --local can be used without any active mode/scope/project
// Just return Ok(Layer::UserLocal) - no context checks needed

// LAYER VARIANT: Use Layer::UserLocal (not Layer::Local or Layer::LocalUser)
// The enum variant is named UserLocal in src/core/layer.rs:28

// PRECEDENCE: Layer 8 has precedence 8
// This means it overrides layers 1-7
// Only WorkspaceActive (Layer 9) has higher precedence

// STORAGE PATH: Layer 8 uses ~/.jin/local/ (not jin/local/)
// Note the tilde (~) for home directory
// This is different from other layers which use jin/ prefix

// GIT REF: Layer 8 uses refs/jin/layers/local
// No /_ suffix needed (no child refs under UserLocal)

// INDEPENDENCE: Layer 8 doesn't require_mode(), doesn't require_scope(), isn't is_project_specific()
// It's a standalone layer like GlobalBase (Layer 1)

// TEST PATTERN: Follow test_route_global() exactly
// Use RoutingOptions { local: true, ..Default::default() }
// Use ProjectContext::default() (no mode/scope needed)
// Assert exact layer: assert_eq!(layer, Layer::UserLocal)

// DERIVE: No changes to derives or macros needed
// Layer enum already derives everything needed
// RoutingOptions already has local field from P1.M1.T2.S1

// DEPENDENCIES: P1.M1.T2.S2 must be complete
// But since we're running in parallel, assume validation is in place
// The routing logic doesn't depend on validation - it assumes valid input
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - this subtask adds routing logic to an existing function.

**Input Contract** (from P1.M1.T2.S1 - assumed complete):
```rust
// RoutingOptions already has local field
#[derive(Debug, Default)]
pub struct RoutingOptions {
    pub mode: bool,
    pub scope: Option<String>,
    pub project: bool,
    pub global: bool,
    pub local: bool,  // ADDED IN P1.M1.T2.S1
}
```

**Current route_to_layer() Function** (lines 31-65):
```rust
pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
    // Global flag takes precedence
    if options.global {
        return Ok(Layer::GlobalBase);
    }

    // Check mode flag
    if options.mode {
        // Require active mode
        context.require_mode()?;

        if let Some(ref _scope) = options.scope {
            // Mode + Scope
            if options.project {
                // Mode + Scope + Project
                Ok(Layer::ModeScopeProject)
            } else {
                // Mode + Scope
                Ok(Layer::ModeScope)
            }
        } else if options.project {
            // Mode + Project
            Ok(Layer::ModeProject)
        } else {
            // Mode only
            Ok(Layer::ModeBase)
        }
    } else if let Some(ref _scope) = options.scope {
        // Scope without mode (untethered scope)
        Ok(Layer::ScopeBase)
    } else {
        // Default: Project Base
        Ok(Layer::ProjectBase)
    }
}
```

**Modified route_to_layer() Function** (add lines after line 35):
```rust
pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
    // Global flag takes precedence
    if options.global {
        return Ok(Layer::GlobalBase);
    }

    // Local flag routes to UserLocal layer
    if options.local {
        return Ok(Layer::UserLocal);
    }

    // Check mode flag
    if options.mode {
        // Require active mode
        context.require_mode()?;

        if let Some(ref _scope) = options.scope {
            // Mode + Scope
            if options.project {
                // Mode + Scope + Project
                Ok(Layer::ModeScopeProject)
            } else {
                // Mode + Scope
                Ok(Layer::ModeScope)
            }
        } else if options.project {
            // Mode + Project
            Ok(Layer::ModeProject)
        } else {
            // Mode only
            Ok(Layer::ModeBase)
        }
    } else if let Some(ref _scope) = options.scope {
        // Scope without mode (untethered scope)
        Ok(Layer::ScopeBase)
    } else {
        // Default: Project Base
        Ok(Layer::ProjectBase)
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/staging/router.rs
  - FILE: src/staging/router.rs (lines 31-65)
  - FUNCTION: route_to_layer()
  - ACTION: Add --local routing case after --global check
  - IMPLEMENT:
    * Add comment: // Local flag routes to UserLocal layer
    * Add if condition: if options.local
    * Add return: return Ok(Layer::UserLocal);
    * Position: After line 35 (after --global routing), before line 37 (before mode checks)
  - PATTERN: Mirror lines 32-35 (--global routing)
  - NAMING: Layer::UserLocal (exact enum variant name)
  - NO_CONTEXT_NEEDED: Unlike --mode, --local doesn't need context checks
  - DEPENDENCIES: P1.M1.T2.S1 must be complete (local field must exist)
  - PRESERVE: All existing routing logic unchanged
  - FILES TO MODIFY: src/staging/router.rs (1 file)
  - FILES TO CREATE: None

Task 2: ADD TEST CASE to src/staging/router.rs
  - FILE: src/staging/router.rs (tests module, lines 93-275)
  - ACTION: Add test_route_local() function
  - IMPLEMENT:
    * Test function: test_route_local()
    * RoutingOptions: { local: true, ..Default::default() }
    * ProjectContext: ProjectContext::default() (no mode/scope needed)
    * Call: route_to_layer(&options, &context).unwrap()
    * Assert: assert_eq!(layer, Layer::UserLocal)
  - PATTERN: Mirror test_route_global() (lines 120-129)
  - POSITION: After test_route_global(), before test_route_mode()
  - DEPENDENCIES: Task 1 must be complete
  - FILES TO MODIFY: src/staging/router.rs (1 file - same file as Task 1)

Task 3: VALIDATE WITH CARGO CHECK
  - COMMAND: cargo check
  - EXPECTED: Zero compilation errors
  - IF FAILS: Read error output, fix typos or syntax issues
  - DEPENDENCIES: Task 1 and Task 2 complete

Task 4: RUN TESTS
  - COMMAND: cargo test --lib staging::router::tests -v
  - EXPECTED: All tests pass, including test_route_local()
  - IF FAILS: Debug test failures, fix routing logic or test expectations
  - DEPENDENCIES: Task 3 complete (cargo check passes)

Task 5: RUN FULL TEST SUITE
  - COMMAND: cargo test --lib
  - EXPECTED: All tests pass (no regressions in other modules)
  - IF FAILS: Check for unintended side effects
  - DEPENDENCIES: Task 4 complete (router tests pass)
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT CODE TO ADD ==================
// Location: src/staging/router.rs, after line 35, before line 37

    // Local flag routes to UserLocal layer
    if options.local {
        return Ok(Layer::UserLocal);
    }

// ================== CONTEXT FOR ADDITION ==================
// Insert this code block between:
// Line 35: closing brace/semicolon of --global routing
// Line 37: Comment for mode flag check

// The function after modification should look like:

pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
    // Global flag takes precedence
    if options.global {
        return Ok(Layer::GlobalBase);
    }

    // Local flag routes to UserLocal layer  // <-- NEW BLOCK
    if options.local {
        return Ok(Layer::UserLocal);
    }  // <-- END NEW BLOCK

    // Check mode flag
    if options.mode {
        // ... rest of function unchanged ...
    }

    // ... rest of function unchanged ...
}

// ================== ROUTING PRECEDENCE EXPLAINED ==================
// 1. --global (Layer 1)  -> checked first, returns early
// 2. --local  (Layer 8)  -> checked second, returns early  <-- NEW
// 3. --mode   (Layer 2-5) -> checked third, may return or continue
// 4. --scope  (Layer 3,4,6) -> checked in mode block or separately
// 5. Default  (Layer 7)  -> final else clause

// Note: Validation (P1.M1.T2.S2) ensures only ONE of these flags is set
// So in practice, only ONE of these early returns will ever execute

// ================== LAYER 8 SPECIFICATIONS ==================
// Layer::UserLocal
// - Precedence: 8 (second highest, only below WorkspaceActive)
// - Storage: ~/.jin/local/
// - Git ref: refs/jin/layers/local
// - Context required: NONE (unlike --mode which requires active mode)
// - Can combine with: NOTHING (validation prevents combinations)
// - Independence: Fully independent of mode/scope/project

// ================== TEST CASE TO ADD ==================
// Location: src/staging/router.rs, tests module (after line 129)

    #[test]
    fn test_route_local() {
        let options = RoutingOptions {
            local: true,
            ..Default::default()
        };
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::UserLocal);
    }

// ================== TEST PATTERN EXPLAINED ==================
// RoutingOptions:
//   - local: true (the flag we're testing)
//   - All other fields: Default (false for bools, None for options)
//
// ProjectContext:
//   - Use default (no mode, no scope, no project)
//   --local doesn't need context - Layer 8 is independent
//
// unwrap():
//   - Expects success (Ok result)
//   - Should not panic because --local is a valid flag
//
// assert_eq!():
//   - Exact match to Layer::UserLocal variant
//   - No partial matches - must be exact

// ================== DIFFERENCES FROM --global ROUTING ==================
// --global routing:
//   - Layer: GlobalBase (Layer 1)
//   - Precedence: 1 (lowest)
//   - Storage: jin/global/
//
// --local routing (NEW):
//   - Layer: UserLocal (Layer 8)
//   - Precedence: 8 (second highest)
//   - Storage: ~/.jin/local/
//
// Similarities:
//   - Both use early return pattern
//   - Both don't need context
//   - Both are independent layers
//   - Both are mutually exclusive with other flags (validation)

// ================== WHY NO CONTEXT NEEDED ==================
// --mode requires context:
//   context.require_mode()?;  // Fails if no active mode
//
// --local doesn't need context:
//   return Ok(Layer::UserLocal);  // Works anytime, no validation needed
//
// Reason: Layer 8 (UserLocal) is designed to be independent
// - It doesn't require an active mode
// - It doesn't require a scope
// - It doesn't require a project
// - It's a standalone layer for machine-specific configs
```

### Integration Points

```yaml
MODIFICATIONS:
  - file: src/staging/router.rs
    function: route_to_layer()
    lines: Insert after line 35, before line 37
    scope: Add routing if-block, no other changes

NO CHANGES TO:
  - validate_routing_options() function (already modified in P1.M1.T2.S2)
  - RoutingOptions struct (already modified in P1.M1.T2.S1)
  - Layer enum (Layer::UserLocal already exists in src/core/layer.rs)
  - ProjectContext type (not needed for --local routing)
  - src/core/error.rs (no new error types needed)
  - Command files (will use route_to_layer() in P1.M1.T3.S1)
  - src/staging/mod.rs (re-exports are automatic)

CALLERS OF route_to_layer():
  - src/staging/router.rs (internal use in tests)
  - Potentially called from command files (add.rs, mv.rs, rm.rs, import_cmd.rs)
  - No changes needed to callers - routing is transparent

DOWNSTREAM DEPENDENCIES:
  - P1.M1.T3.S1 (pass local flag in add command execute())
    - Depends on this routing being in place
    - Will call route_to_layer() with local field set
    - Expects Layer::UserLocal to be returned

PARALLEL EXECUTION CONTEXT:
  - P1.M1.T2.S2 (validation) is running in parallel
  - This subtask assumes P1.M1.T2.S2 will be complete
  - The routing logic doesn't depend on validation - it assumes valid input
  - If validation is not yet complete, routing logic still works correctly
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modification - must pass before proceeding
cargo check                           # Type checking - MUST pass with 0 errors

# Expected: Zero errors. If errors exist, READ output carefully.
# Common issues:
# - Missing semicolon after return statement
# - Wrong Layer variant name (Local vs UserLocal)
# - Missing closing brace

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

# Expected: All tests pass, including new test_route_local()

# Run specific new test
cargo test test_route_local -v

# Expected: Test passes with output like:
# test src/staging/router.rs::tests::test_route_local ... ok

# Full library test suite
cargo test --lib

# Expected: All tests pass (no regressions in other modules)

# Test with output
cargo test --lib -- --nocapture

# Expected: Clean test run with all passing
```

### Level 3: Integration Testing (System Validation)

```bash
# Note: P1.M1.T3.S1 (passing local flag from command) is not yet complete
# So CLI testing won't work yet. But we can verify routing logic directly.

# Verify the function signature and export
cargo doc --open --no-deps

# Expected: Documentation opens showing route_to_layer in staging module

# Check that all existing tests still pass
cargo test

# Expected: All existing tests pass
# Focus on:
# - Router tests in src/staging/router.rs
# - Any tests that call route_to_layer

# Manual verification of routing logic (optional)
cat > test_local_routing.rs << 'EOF'
use jin::staging::{RoutingOptions, route_to_layer};
use jin::core::{Layer, ProjectContext};

fn main() {
    // Test: --local routes to UserLocal
    let opts = RoutingOptions {
        local: true,
        ..Default::default()
    };
    let ctx = ProjectContext::default();
    match route_to_layer(&opts, &ctx) {
        Ok(Layer::UserLocal) => println!("PASS: --local routes to UserLocal"),
        Ok(other) => println!("FAIL: Got {:?}", other),
        Err(e) => println!("FAIL: {}", e),
    }
}
EOF

# Compile and run (after cargo build)
rustc --edition 2021 -L target/debug/deps \
    --extern jin=target/debug/libjin.rlib \
    test_local_routing.rs
./test_local_routing

# Expected output:
# PASS: --local routes to UserLocal

# Cleanup
rm test_local_routing.rs test_local_routing
```

### Level 4: Domain-Specific Validation

```bash
# Verify Layer 8 properties
cat > verify_layer8.rs << 'EOF'
use jin::core::Layer;

fn main() {
    let layer = Layer::UserLocal;

    // Verify precedence
    assert_eq!(layer.precedence(), 8, "Layer 8 should have precedence 8");

    // Verify storage path
    let path = layer.storage_path(None, None, None);
    assert_eq!(path, "~/.jin/local/", "Layer 8 should use ~/.jin/local/");

    // Verify ref path
    let ref_path = layer.ref_path(None, None, None);
    assert_eq!(ref_path, "refs/jin/layers/local", "Layer 8 should use refs/jin/layers/local");

    // Verify independence
    assert!(!layer.requires_mode(), "Layer 8 should not require mode");
    assert!(!layer.requires_scope(), "Layer 8 should not require scope");
    assert!(!layer.is_project_specific(), "Layer 8 should not be project-specific");

    println!("All Layer 8 properties verified!");
}
EOF

# Compile and run
rustc --edition 2021 -L target/debug/deps \
    --extern jin=target/debug/libjin.rlib \
    verify_layer8.rs
./verify_layer8

# Expected output:
# All Layer 8 properties verified!

# Cleanup
rm verify_layer8.rs verify_layer8
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy` produces no warnings (or only acceptable ones)
- [ ] `cargo test --lib staging::router::tests` passes all tests
- [ ] `test_route_local()` test passes
- [ ] All existing tests still pass (no regressions)
- [ ] Function signature unchanged (takes `&RoutingOptions`, `&ProjectContext`, returns `Result<Layer>`)

### Feature Validation

- [ ] Routing logic matches `--global` pattern exactly
- [ ] Returns `Layer::UserLocal` when `options.local == true`
- [ ] Placement is correct (after `--global`, before mode checks)
- [ ] No context checks needed (unlike `--mode` routing)
- [ ] Early return pattern is used
- [ ] Test follows existing test pattern (`..Default::default()`, exact layer assertion)

### Code Quality Validation

- [ ] Comment added explaining routing rule
- [ ] Code follows existing formatting and style
- [ ] Test function name follows `test_route_*` pattern
- [ ] Test uses `RoutingOptions` struct literal with `..Default::default()`
- [ ] Test assertion uses `assert_eq!(layer, Layer::UserLocal)`
- [ ] No unintended side effects on other routing logic

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable names
- [ ] Comment explains Layer 8 routing
- [ ] No breaking changes to existing functionality
- [ ] Routing is transparent to callers

---

## Anti-Patterns to Avoid

- **Don't** modify `validate_routing_options()` function (that's P1.M1.T2.S2)
- **Don't** modify `RoutingOptions` struct (already modified in P1.M1.T2.S1)
- **Don't** add context checks like `context.require_mode()` for `--local` routing
- **Don't** use `Layer::Local` or `Layer::LocalUser` - correct variant is `Layer::UserLocal`
- **Don't** place routing before `--global` check - maintain precedence order
- **Don't** use nested conditionals - use early return like `--global` pattern
- **Don't** forget to add test case for the new routing behavior
- **Don't** use `assert!(result.is_ok())` - use `unwrap()` and assert exact layer
- **Don't** modify command files (add.rs, mv.rs, rm.rs, import_cmd.rs) in this subtask
- **Don't** add new error types or error handling - routing is straightforward

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Extremely Simple**: 3-line addition to existing routing function
- **Clear Pattern**: Exact mirror of existing `--global` routing in same function
- **Well-Understood**: Rust early-return pattern is fundamental
- **No Logic Complexity**: Pure routing, no state changes or complex conditions
- **No External Dependencies**: Uses existing `Layer::UserLocal` enum variant
- **Testable**: Routing logic is easily testable with unit tests
- **Exact Specification**: Placement, logic, test pattern all specified
- **Clear Success Criteria**: Unambiguous test defines correctness
- **Isolated Change**: No ripple effects to other parts of codebase
- **Parallel Execution Safe**: Doesn't depend on parallel task P1.M1.T2.S2

**Implementation is equivalent to adding one if-block to a function**:
```rust
// Local flag routes to UserLocal layer
if options.local {
    return Ok(Layer::UserLocal);
}
```

This is a straightforward routing addition that follows an established pattern in the same function. The implementation risk is minimal because it uses the exact same early-return pattern as the `--global` routing case, which is already proven to work correctly.

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1T2S3/research/`

**Key File References**:
- `src/staging/router.rs` - `route_to_layer()` function (lines 31-65)
- `src/core/layer.rs` - `Layer::UserLocal` enum variant (line 28)
- `plan/P1M1T2S1/PRP.md` - Previous work item (adds local field to RoutingOptions)
- `plan/P1M1T2S2/PRP.md` - Parallel work item (adds validation for --local flag)
- `plan/P1M1T2S3/research/test_patterns.md` - Test patterns from codebase analysis
- `plan/P1M1T2S3/research/rust_routing_patterns.md` - Rust best practices for routing
- `plan/docs/system_context.md` - Layer 8 specification and routing table

**External References**:
- [RFC 2497 - if-let-chains](https://rust-lang.github.io/rfcs/2497-if-let-chains.html)
- [Stop Writing Ugly If-Else Chains](https://medium.com/@bhesaniyavatsal/stop-writing-ugly-if-else-chains-this-one-rust-feature-will-change-how-you-code-forever-10e9f93e41c4)
- [Using match Ergonomically](https://dev.to/sgchris/using-match-ergonomics-avoid-the-if-else-chains-19dm)
