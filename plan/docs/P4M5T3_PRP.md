# PRP: Context Command (P4.M5.T3)

---

## Goal

**Feature Goal**: Implement a CLI command that displays the current active Jin context (mode, scope, project) to users.

**Deliverable**: A `jin context` command that reads and displays the active context from `.jin/context` file.

**Success Definition**:
- Running `jin context` displays the current mode, scope, project, and last updated timestamp
- Command returns appropriate error when Jin is not initialized
- Display handles missing values gracefully (shows "(none)" for unset mode/scope)
- All existing unit tests pass
- Integration test covers both success and failure cases

## User Persona

**Target User**: Developer using Jin for multi-layer configuration management

**Use Case**: User wants to quickly verify which mode and scope are currently active before performing layer operations (add, commit, apply).

**User Journey**:
1. User has initialized Jin in their project
2. User has activated modes/scopes via `jin mode use` and `jin scope use`
3. User runs `jin context` to verify current active configuration
4. User sees formatted output showing active mode, scope, project, and last updated time
5. User proceeds with confidence to perform layer operations

**Pain Points Addressed**:
- Eliminates confusion about which mode/scope is currently active
- Provides visibility into Jin's state before operations that depend on layer composition
- Helps debug issues when layer operations don't behave as expected

## Why

- **User Visibility**: Users need to know their active context to understand how layers will be composed and applied
- **Debugging**: When layer operations produce unexpected results, context display helps diagnose the issue
- **Workflow Validation**: Before running `jin apply` or `jin add`, users can verify they have the right mode/scope active
- **Integration Point**: Other commands (add, apply) read context internally; this provides user-facing visibility into the same data

## What

**User-visible behavior**:

```bash
# Display current context
$ jin context
Current Jin context:

  Active mode:   claude
  Active scope:  language:javascript
  Project:       ui-dashboard
  Last updated:  2025-12-27T10:30:00Z

# When mode/scope not set
$ jin context
Current Jin context:

  Active mode:   (none)
  Active scope:  (none)
  Project:       (auto-inferred)
```

**Technical requirements**:
- Read `.jin/context` file using `ProjectContext::load()`
- Display formatted output with mode, scope, project, and last_updated fields
- Return `JinError::NotInitialized` if `.jin/context` doesn't exist
- Handle missing optional fields gracefully with "(none)" fallback
- No command-line arguments (simple display-only command)

### Success Criteria

- [ ] Command displays all four context fields (mode, scope, project, last_updated)
- [ ] Missing mode/scope displays "(none)" instead of empty string
- [ ] Uninitialized project returns `JinError::NotInitialized` with helpful message
- [ ] Command integrates with CLI router in `commands/mod.rs`
- [ ] Unit tests pass: `cargo test context`
- [ ] Integration test passes: `cargo test test_context_subcommand`

---

## All Needed Context

### Context Completeness Check

**No Prior Knowledge Test**: If someone knew nothing about this codebase, would they have everything needed?

**Answer**: **YES** - The context command is already fully implemented. This PRP validates the existing implementation and ensures comprehensive testing.

### Documentation & References

```yaml
# MUST READ - Core implementation files
- file: src/commands/context.rs
  why: The complete implementation of the context command - this file IS the implementation
  pattern: Load context, display formatted output, handle errors
  gotcha: Implementation already exists and is complete

- file: src/core/config.rs:78-154
  why: ProjectContext struct definition with load(), save(), and helper methods
  pattern: Context storage in YAML format at .jin/context
  section: ProjectContext struct and impl block

- file: src/cli/mod.rs:62-63
  why: CLI enum definition for Context command
  pattern: Simple command with no arguments (Context variant)
  gotcha: Command already wired in CLI

- file: src/commands/mod.rs:44
  why: Command router wiring for Context command
  pattern: Commands::Context => context::execute()
  gotcha: Already integrated into command router

# EXTERNAL RESEARCH - Context command patterns in CLI tools
- url: https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_current-context/
  why: kubectl context display pattern - shows current context name only
  critical: Simplicity over verbosity - just show the essential info

- url: https://docs.docker.com/engine/manage-resources/contexts/
  why: Docker context management - separate list, use, inspect commands
  critical: Separation of display vs modification commands

- url: https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html
  why: AWS profile configuration - shows current via config list
  critical: Configuration stored in simple file formats

# IMPLEMENTATION REFERENCE - Similar commands
- file: src/commands/layers.rs
  why: Similar simple display command showing layer composition
  pattern: Load context, format output, handle missing values

- file: src/commands/list.rs
  why: Similar display-only command with no arguments
  pattern: Read repository state, display formatted output

# TESTING REFERENCE
- file: tests/cli_basic.rs:458-464
  why: Existing integration test for context command (failure case only)
  pattern: Use assert_cmd for CLI testing, predicates for output validation
  critical: Need to add success case test

- file: src/commands/context.rs:42-94
  why: Existing unit tests showing test setup patterns
  pattern: Use tempfile::TempDir, set JIN_DIR env var, create .jin directory
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin
├── src/
│   ├── cli/
│   │   ├── mod.rs           # Commands enum with Context variant (line 62-63)
│   │   └── args.rs          # Command argument structs (not used for Context)
│   ├── commands/
│   │   ├── mod.rs           # Command router with Context wiring (line 44)
│   │   ├── context.rs       # COMPLETE: context command implementation
│   │   ├── mode.rs          # Reference: mode command with subcommands
│   │   ├── scope.rs         # Reference: scope command with subcommands
│   │   ├── layers.rs        # Reference: similar display command
│   │   └── list.rs          # Reference: display-only command
│   ├── core/
│   │   ├── config.rs        # ProjectContext struct (lines 78-154)
│   │   └── error.rs         # JinError::NotInitialized variant
│   └── lib.rs               # Main entry point
└── tests/
    ├── cli_basic.rs         # Integration tests (line 458: test_context_subcommand)
    └── common/
        ├── fixtures.rs      # Test helpers (jin(), setup_test_repo())
        └── assertions.rs    # Custom assertions (assert_context_mode, etc.)
```

### Desired Codebase Tree

```bash
# No changes needed - implementation is complete

# The following files already exist and are complete:
src/commands/context.rs        # COMPLETE: context command implementation
src/core/config.rs              # COMPLETE: ProjectContext with load/save
src/cli/mod.rs                  # COMPLETE: Context command in CLI enum
src/commands/mod.rs             # COMPLETE: context::execute() wired in router

# Testing status:
src/commands/context.rs         # Has unit tests (test_execute_default_context, etc.)
tests/cli_basic.rs              # Has one integration test (failure case only)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Context file format and location
// .jin/context is YAML format, not TOML
// Location: .jin/context in project root, not ~/.jin/config.toml
// Use ProjectContext::load() which returns JinError::NotInitialized if missing

// CRITICAL: Error handling pattern
// Always check for JinError::NotInitialized specifically
// Other parsing errors should fall back to ProjectContext::default()
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
    Err(_) => ProjectContext::default(),  // Fallback for parse errors
};

// CRITICAL: Display format uses specific spacing
// Field names are left-aligned to width of 15 chars total
// "  Active mode:   {value}"  - 2 leading spaces, label padded to 15, then value
// Use .unwrap_or("(none)") for missing optional values

// CRITICAL: last_updated is optional
// Only display if Some(value), otherwise skip the line entirely
// if let Some(last_updated) = context.last_updated.as_deref() {
//     println!("  Last updated:  {}", last_updated);
// }

// CRITICAL: No arguments for this command
// Context is a simple command variant, not Context(Args)
// In commands/mod.rs: Commands::Context => context::execute() (no args passed)
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// Existing data models - NO CHANGES NEEDED

// src/core/config.rs - ProjectContext struct
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    pub version: u32,
    pub mode: Option<String>,        // Active mode name
    pub scope: Option<String>,       // Active scope name
    pub project: Option<String>,     // Project name (auto-inferred)
    pub last_updated: Option<String>, // ISO 8601 timestamp
}

// Methods used:
// ProjectContext::load() -> Result<Self>
//   - Reads .jin/context file
//   - Returns Err(JinError::NotInitialized) if file missing
//   - Deserializes YAML content
```

### Implementation Tasks

**IMPORTANT**: The context command is **ALREADY FULLY IMPLEMENTED**. This section validates the existing implementation and identifies any gaps.

```yaml
Task 1: VALIDATE existing implementation (src/commands/context.rs)
  - VERIFY: execute() function loads context correctly
  - VERIFY: Display format matches specification (15-char alignment, "(none)" fallback)
  - VERIFY: Error handling for NotInitialized case
  - VERIFY: Graceful handling of parse errors (fallback to default)
  - STATUS: COMPLETE - implementation exists and is correct

Task 2: VALIDATE CLI integration (src/cli/mod.rs)
  - VERIFY: Commands::Context variant exists (line 62-63)
  - VERIFY: No arguments attached (simple command variant)
  - VERIFY: Command router wiring (src/commands/mod.rs line 44)
  - STATUS: COMPLETE - wiring is correct

Task 3: REVIEW existing unit tests (src/commands/context.rs:42-94)
  - VERIFY: test_execute_default_context() tests basic functionality
  - VERIFY: test_execute_with_mode_and_scope() tests populated context
  - VERIFY: test_execute_not_initialized() tests error case
  - RUN: cargo test context::tests
  - STATUS: COMPLETE - unit tests exist and cover main cases

Task 4: ADD integration test for success case (tests/cli_basic.rs)
  - IMPLEMENT: test_context_subcommand_success() function
  - FOLLOW: pattern from test_init_subcommand (tempfile, JIN_DIR, current_dir)
  - TEST: Full initialization + mode/scope activation + context display
  - ASSERT: stdout contains "Current Jin context"
  - ASSERT: stdout contains mode name when set
  - ADD: After line 464 in cli_basic.rs
  - PLACEMENT: tests/cli_basic.rs

Task 5: VALIDATE test coverage
  - RUN: cargo test --lib context (unit tests)
  - RUN: cargo test cli_basic (integration tests)
  - VERIFY: All tests pass
  - CHECK: Coverage includes success, failure, and edge cases
```

### Implementation Patterns & Key Details

```rust
// ========== EXISTING IMPLEMENTATION (src/commands/context.rs) ==========

// Pattern 1: Context loading with error handling
pub fn execute() -> Result<()> {
    // Load project context with specific error handling
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),  // Parse errors -> default
    };

    // ... display logic
}

// Pattern 2: Formatted display with 15-char label alignment
println!("Current Jin context:");
println!();
println!(
    "  Active mode:   {}",
    context.mode.as_deref().unwrap_or("(none)")
);
// Label "Active mode:" + spaces = 15 chars total, then value

// Pattern 3: Optional field conditional display
if let Some(last_updated) = context.last_updated.as_deref() {
    println!("  Last updated:  {}", last_updated);
}

// ========== CLI INTEGRATION PATTERN ==========

// In src/cli/mod.rs (line 62-63):
/// Show/set active context
Context,  // Simple variant, no arguments

// In src/commands/mod.rs (line 44):
Commands::Context => context::execute(),

// ========== TESTING PATTERN ==========

// Unit test setup pattern (src/commands/context.rs:46-62):
fn setup_test_env() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Isolate global config
    let jin_dir = temp.path().join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    // Change to temp directory
    std::env::set_current_dir(temp.path()).unwrap();

    // Initialize .jin directory and context
    std::fs::create_dir(".jin").unwrap();
    let context = ProjectContext::default();
    context.save().unwrap();

    temp
}

// Integration test pattern (tests/cli_basic.rs:30-44):
fn test_init_subcommand() {
    let temp = TempDir::new().unwrap();

    // Isolate global config
    let jin_dir = temp.path().join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    jin()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Jin"));
}
```

### Integration Points

```yaml
NO NEW INTEGRATIONS NEEDED - Existing wiring is complete

COMMAND ROUTING:
  - location: src/commands/mod.rs:44
  - pattern: Commands::Context => context::execute()
  - status: COMPLETE

CLI DEFINITION:
  - location: src/cli/mod.rs:62-63
  - pattern: Simple command variant with no arguments
  - status: COMPLETE

CONTEXT STORAGE:
  - location: .jin/context (YAML format)
  - read via: ProjectContext::load()
  - status: COMPLETE

ERROR HANDLING:
  - error type: JinError::NotInitialized
  - message: "Jin not initialized in this project"
  - location: src/core/error.rs
  - status: COMPLETE
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after any code changes to verify correctness
cargo check                           # Fast compilation check
cargo clippy --all-targets            # Lint checking
cargo fmt --check                     # Format validation

# Fix any issues before proceeding
cargo fmt                             # Auto-format code

# Expected: Zero errors, zero warnings. If issues exist, fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test context command unit tests
cargo test context::tests --lib

# Test all core functionality
cargo test --lib

# Run with output for debugging
cargo test context::tests --lib -- --nocapture

# Expected: All tests pass. If failing, debug the specific test case.

# Specific tests to verify:
# - test_execute_default_context: passes
# - test_execute_with_mode_and_scope: passes
# - test_execute_not_initialized: passes
```

### Level 3: Integration Testing (System Validation)

```bash
# Test the complete CLI command
cargo test test_context_subcommand

# Test all CLI basic tests
cargo test cli_basic

# Manual testing with real binary
cargo build --release
./target/release/jin context
# Expected output when initialized:
# Current Jin context:
#
#   Active mode:   (none)
#   Active scope:  (none)
#   Project:       (auto-inferred)

# Test error case (uninitialized directory)
cd /tmp && mkdir test_jin && cd test_jin
./target/release/jin context
# Expected: Error "Jin not initialized in this project"

# Test with mode/scope set
./target/release/jin init
./target/release/jin mode use testmode
./target/release/jin scope use testscope
./target/release/jin context
# Expected: Shows "Active mode: testmode" and "Active scope: testscope"

# Expected: All manual tests produce expected output.
```

### Level 4: Domain-Specific Validation

```bash
# Context Display Validation:

# 1. Verify format alignment (15-char labels)
./target/release/jin context | cat -A
# Should show consistent spacing

# 2. Verify "(none)" fallback for unset values
# (When mode/scope not activated)
./target/release/jin context | grep "(none)"

# 3. Verify last_updated only shows when set
# (Should only appear if context was previously modified)

# 4. Test context persistence
./target/release/jin mode use testmode
./target/release/jin context | grep "testmode"
# Should show the activated mode

# 5. Test error message clarity
cd /tmp && mkdir err_test && cd err_test && ../../target/release/jin context 2>&1
# Should show clear "Jin not initialized" message

# Integration with other commands:

# 6. Verify context read by other commands
./target/release/jin add --mode file.txt
# Should work when mode is active (verify via context first)

# Expected: All domain-specific validations pass.
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All Level 1 validation passes: `cargo check`, `cargo clippy`, `cargo fmt --check`
- [ ] All Level 2 unit tests pass: `cargo test context::tests --lib`
- [ ] All Level 3 integration tests pass: `cargo test cli_basic`
- [ ] All Level 4 domain validations pass (manual testing)
- [ ] No compiler warnings
- [ ] No clippy warnings

### Feature Validation

- [ ] Command displays all four context fields (mode, scope, project, last_updated)
- [ ] Missing mode/scope displays "(none)" instead of empty string
- [ ] Uninitialized project returns `JinError::NotInitialized`
- [ ] Display format has proper label alignment (15 characters)
- [ ] last_updated line only appears when value is Some
- [ ] Command works when called after `jin init`
- [ ] Command shows correct values after `jin mode use` and `jin scope use`

### Code Quality Validation

- [ ] Follows existing codebase patterns (context loading, error handling)
- [ ] Consistent with similar commands (layers, list)
- [ ] Error messages are clear and actionable
- [ ] No hardcoded values that should be configurable
- [ ] Tests cover success, failure, and edge cases

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable names
- [ ] CLI help text is descriptive (`jin context --help`)
- [ ] Error messages guide users to correct actions
- [ ] Integration with existing CLI is seamless

---

## Anti-Patterns to Avoid

- ❌ **Don't add arguments to Context command** - It's intentionally simple; use `jin mode use` and `jin scope use` to set context
- ❌ **Don't hardcode field widths** - Use consistent 15-char alignment but make it maintainable
- ❌ **Don't suppress parse errors** - Fallback to `ProjectContext::default()` for parse errors, but `NotInitialized` should propagate
- ❌ **Don't use empty string for missing values** - Use "(none)" for clarity
- ❌ **Don't show last_updated when None** - Skip the line entirely, don't show empty value
- ❌ **Don't duplicate mode/scope command logic** - Context is display-only; setting is done by dedicated commands
- ❌ **Don't skip testing** - Both unit and integration tests are required

---

## Implementation Status

**Current State**: ✅ **COMPLETE**

The context command is fully implemented and integrated:

1. **Implementation**: `src/commands/context.rs` - Complete
2. **CLI Definition**: `src/cli/mod.rs:62-63` - Complete
3. **Command Router**: `src/commands/mod.rs:44` - Complete
4. **Unit Tests**: `src/commands/context.rs:42-94` - Complete
5. **Integration Test**: `tests/cli_basic.rs:458-464` - Partial (only failure case)

**Recommended Actions**:

1. ✅ **Verify existing implementation** - Code review confirms it's correct
2. ➕ **Add success case integration test** - Extend `cli_basic.rs` with test for initialized project
3. ✅ **Run full test suite** - Ensure no regressions

**Confidence Score**: 10/10 for one-pass implementation success

The implementation already exists and follows all established patterns. The PRP validates correctness and identifies a minor gap (success case integration test) that can be filled if desired.
