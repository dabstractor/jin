# PRP: List Command (P4.M5.T8)

## Goal

**Feature Goal**: Implement `jin list` command that displays all available modes, scopes, and projects in a unified view with their Git commit OIDs.

**Deliverable**: A fully functional `jin list` command module at `src/commands/list.rs` that consolidates information from existing mode/scope listing functionality and adds project listing.

**Success Definition**:
- Running `jin list` displays active context (mode/scope) followed by sections for available modes, scopes, and projects
- Each section shows names with short OIDs (8 characters)
- Empty sections show "No X found" messages
- Command integrates properly with CLI dispatch and error handling
- All tests pass with `cargo test`

## User Persona

**Target User**: Developer using Jin to manage AI/editor-specific configurations

**Use Case**: Developer wants to see all available modes, scopes, and projects at a glance to understand what configurations exist in their Jin repository

**User Journey**:
1. User runs `jin list` in a Jin-initialized project
2. Command shows active mode/scope if any
3. Command lists all available modes with their OIDs
4. Command lists all scopes (untethered and mode-bound) with their OIDs and parent mode
5. Command lists all projects with their OIDs

**Pain Points Addressed**:
- Currently need to run `jin modes` and `jin scopes` separately
- No way to see projects (no dedicated command exists)
- Inconsistent display across separate commands

## Why

- **Consolidation**: Single command replaces need for multiple list commands (`jin modes`, `jin scopes`)
- **Discovery**: Users can discover all available configurations in one place
- **Project visibility**: Projects are currently invisible (no dedicated list command)
- **Consistency**: Matches `jin status` pattern of showing multiple related information sections

## What

### User-Visible Behavior

```
$ jin list

Active mode: claude
Active scope: language:rust

Available modes:
  claude               a1b2c3d4
  cursor               e5f6g7h8
  zed                  i9j0k1l2

Available scopes:
  language:rust        [untethered] m3n4o5p6
  language:javascript  [untethered] q7r8s9t0
  backend              [mode:claude] u1v2w3x4

Available projects:
  jin-glm-doover       y5z6a7b8
```

### Success Criteria

- [ ] `jin list` displays active mode/scope from `.jin/context`
- [ ] Lists all modes from `refs/jin/layers/mode/*`
- [ ] Lists all scopes from both `refs/jin/layers/scope/*` and `refs/jin/layers/mode/*/scope/*`
- [ ] Lists all projects from `refs/jin/layers/project/*`
- [ ] Shows short OIDs (8 characters) for each entry
- [ ] Shows mode binding for mode-bound scopes
- [ ] Returns appropriate error if not in a Jin-initialized directory
- [ ] All existing tests continue to pass
- [ ] New tests cover the list command functionality

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

Yes - this PRP provides:
- Exact file patterns to follow with line references
- Complete command structure templates
- Existing implementation patterns to replicate
- Module registration patterns
- Test patterns to follow

### Documentation & References

```yaml
# MUST READ - Critical files for implementation

- file: src/commands/status.rs
  why: Primary pattern for multi-section display commands
  pattern: Shows how to structure display_active_context(), display_* sections, error handling
  gotcha: Uses println!() with specific formatting (20-char width for names, 8-char OIDs)
  line_reference: Lines 41-145 for execute() structure, display functions

- file: src/commands/mode.rs
  why: Contains execute_list() implementation for modes
  pattern: Lists refs by pattern, extracts names, displays with OIDs
  gotcha: Uses strip_prefix() to extract names from ref paths
  line_reference: Lines 62-91 for execute_list()

- file: src/commands/scope.rs
  why: Contains execute_list() implementation for scopes with unescaping
  pattern: Handles both untethered and mode-bound scopes, unescapes colons
  gotcha: Scope names with colons are URL-encoded (%3A) in Git refs
  line_reference: Lines 67-122 for execute_list(), lines 407-424 for escape/unescape functions

- file: src/cli/args.rs
  why: Defines ListCommand struct (unit struct, no fields)
  pattern: Simple unit struct pattern like StatusCommand, Context
  line_reference: Line 94 for Commands::List variant

- file: src/commands/mod.rs
  why: Module export pattern
  pattern: Add pub mod list; and pub use list::execute as list_execute;
  line_reference: Lines 1-37 for module structure and exports

- file: src/main.rs
  why: Command dispatch pattern in main()
  pattern: match commands::list_execute() with error handling
  line_reference: Lines 196-199 for current placeholder, follow pattern of other commands

- file: src/core/config.rs
  why: ProjectContext and JinConfig types
  pattern: Load context, access mode/scope fields, save context
  gotcha: ProjectContext::load() returns Result, handle errors properly

- file: src/git/mod.rs
  why: JinRepo trait for Git operations
  pattern: open(), open_or_create(), list_layer_refs_by_pattern(), find_reference()
  gotcha: list_layer_refs_by_pattern() returns Vec<String> of full ref paths
```

### Current Codebase Tree

```bash
src/
├── cli/
│   └── args.rs              # CLI definitions (ListCommand at line 94)
├── commands/
│   ├── mod.rs               # Module exports (add list module here)
│   ├── status.rs            # Multi-section display pattern
│   ├── mode.rs              # Mode listing (execute_list)
│   ├── scope.rs             # Scope listing (execute_list)
│   └── [other commands]
├── core/
│   ├── config.rs            # ProjectContext, JinConfig
│   ├── error.rs             # JinError types
│   └── layer.rs             # Layer enum
└── main.rs                  # Command dispatch (line 196-199 placeholder)
```

### Desired Codebase Tree with Files to be Added

```bash
src/commands/
├── list.rs                  # NEW: List command implementation
│   ├── execute()            # Main entry point
│   ├── display_active_context()
│   ├── display_available_modes()
│   ├── display_available_scopes()
│   ├── display_available_projects()
│   └── tests module         # Comprehensive tests
```

### Known Gotchas of Our Codebase

```rust
// CRITICAL: Scope names with colons are URL-encoded in Git refs
// "language:rust" becomes "language%3Arust" in refs/jin/layers/scope/
// Must use unescape_from_ref() from scope.rs to display correctly

// CRITICAL: Mode-bound scopes have path pattern: refs/jin/layers/mode/{mode}/scope/{scope}
// Must parse this path to extract both mode and scope names

// CRITICAL: Use consistent formatting from existing commands:
// - {:20} for name alignment (modes)
// - {:30} for name alignment (scopes - longer names with colons)
// - First 8 characters of OID (&oid.to_string()[..8])

// CRITICAL: Always check Jin initialization first
// Pattern from status.rs lines 46-53:
// let context_path = ProjectContext::context_path(&workspace_root);
// if !context_path.exists() {
//     return Err(JinError::Message("Jin is not initialized..."));
// }

// CRITICAL: Empty state handling - each section should handle empty results
// Pattern: if refs.is_empty() { println!("No X found."); return Ok(()); }

// CRITICAL: Test isolation - use local .jin/config.yaml in test helpers
// This prevents tests from interfering with user's global Jin repository
```

## Implementation Blueprint

### Data Models and Structure

No new data models required. Using existing types:
- `ProjectContext` from `core::config` for active context
- `JinRepo` from `git` for Git operations
- `JinConfig` from `core::config` for repository path
- `JinError` from `core::error` for error handling

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/list.rs
  - IMPLEMENT: Module docstring, imports, execute() function
  - FOLLOW pattern: src/commands/status.rs (lines 1-145)
  - NAMING: execute() takes &ListCommand, returns Result<()>
  - PLACEMENT: New file in src/commands/

Task 2: IMPLEMENT display_active_context() in list.rs
  - FOLLOW pattern: src/commands/status.rs (lines 78-89)
  - DISPLAY: Active mode and scope from ProjectContext
  - HANDLE: "No active mode or scope" when both are None

Task 3: IMPLEMENT display_available_modes() in list.rs
  - FOLLOW pattern: src/commands/mode.rs (lines 62-91)
  - LIST: refs/jin/layers/mode/* pattern
  - FORMAT: {:20} for name, 8-char OID
  - HANDLE: "No modes found." for empty result

Task 4: IMPLEMENT display_available_scopes() in list.rs
  - FOLLOW pattern: src/commands/scope.rs (lines 67-122)
  - LIST: Both refs/jin/layers/scope/* and refs/jin/layers/mode/*/scope/*
  - UNESCAPE: Use scope::unescape_from_ref() for colon handling
  - FORMAT: {:30} for name, [untethered] or [mode:X] label, 8-char OID
  - HANDLE: "No scopes found." for empty result

Task 5: IMPLEMENT display_available_projects() in list.rs
  - LIST: refs/jin/layers/project/* pattern
  - FORMAT: Same as modes ({:20} for name, 8-char OID)
  - HANDLE: "No projects found." for empty result
  - GOTCHA: Projects may not exist in many repos (empty case is common)

Task 6: MODIFY src/commands/mod.rs
  - ADD: pub mod list;
  - ADD: pub use list::execute as list_execute;
  - LOCATION: After line 18 (alphabetical order with other commands)
  - PRESERVE: All existing exports

Task 7: MODIFY src/main.rs
  - REPLACE: Lines 196-199 placeholder with proper dispatch
  - PATTERN: Follow Commands::Status pattern (lines 42-48)
  - INTEGRATE: match commands::list_execute() with error handling

Task 8: ADD tests to list.rs
  - FOLLOW pattern: src/commands/status.rs (lines 147-343)
  - TEST: init_jin helper, DirGuard pattern
  - COVER: execute_list success, empty states, no Jin initialized error
  - MOCK: Create test modes/scopes/projects to verify listing
```

### Implementation Patterns & Key Details

```rust
// ===== Module structure (follow status.rs) =====
//! List command implementation.
//!
//! This module implements the `jin list` command that displays
//! all available modes, scopes, and projects with their commit OIDs.

use crate::cli::args::ListCommand;
use crate::core::config::{JinConfig, ProjectContext};
use crate::core::error::Result;
use crate::git::JinRepo;
use crate::commands::scope; // For unescape_from_ref

// ===== Main execute function pattern =====
pub fn execute(_cmd: &ListCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization (CRITICAL - must be first)
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(crate::core::error::JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize."
                .to_string(),
        ));
    }

    // 3. Load and display active context
    let context = ProjectContext::load(&workspace_root)?;
    display_active_context(&context);

    // 4. Open Jin repository and display listings
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    display_available_modes(&repo)?;
    display_available_scopes(&repo)?;
    display_available_projects(&repo)?;

    Ok(())
}

// ===== Display function patterns =====

fn display_active_context(context: &ProjectContext) {
    println!();
    if let Some(mode) = &context.mode {
        println!("Active mode: {}", mode);
    }
    if let Some(scope) = &context.scope {
        println!("Active scope: {}", scope);
    }
    if context.mode.is_none() && context.scope.is_none() {
        println!("No active mode or scope");
    }
}

fn display_available_modes(repo: &JinRepo) -> Result<()> {
    println!();

    let mode_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        return Ok(());
    }

    println!("Available modes:");
    for ref_name in mode_refs {
        let mode_name = ref_name
            .strip_prefix("refs/jin/layers/mode/")
            .unwrap_or(&ref_name);

        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:20} {}", mode_name, short_oid);
            }
        }
    }

    Ok(())
}

fn display_available_scopes(repo: &JinRepo) -> Result<()> {
    println!();

    // Find all untethered scope refs
    let untethered_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/scope/*")?;

    // Find all mode-bound scope refs
    let mode_bound_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*/scope/*")?;

    if untethered_refs.is_empty() && mode_bound_refs.is_empty() {
        println!("No scopes found.");
        return Ok(());
    }

    println!("Available scopes:");

    // Display untethered scopes
    for ref_name in untethered_refs {
        let scope_name = ref_name
            .strip_prefix("refs/jin/layers/scope/")
            .unwrap_or(&ref_name);

        // Unescape the scope name for display (handle colons)
        let display_name = scope::unescape_from_ref(scope_name);

        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:30} [untethered] {}", display_name, short_oid);
            }
        }
    }

    // Display mode-bound scopes
    for ref_name in mode_bound_refs {
        // Extract mode and scope from path: refs/jin/layers/mode/{mode}/scope/{scope}
        let parts: Vec<&str> = ref_name.split('/').collect();
        if parts.len() >= 7 {
            let mode_name = parts[4];
            let scope_name = parts[6];

            // Unescape the scope name for display
            let display_name = scope::unescape_from_ref(scope_name);

            if let Ok(reference) = repo.find_reference(&ref_name) {
                if let Some(oid) = reference.target() {
                    let short_oid = &oid.to_string()[..8];
                    println!("  {:30} [mode:{}] {}", display_name, mode_name, short_oid);
                }
            }
        }
    }

    Ok(())
}

fn display_available_projects(repo: &JinRepo) -> Result<()> {
    println!();

    let project_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/project/*")?;

    if project_refs.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    println!("Available projects:");
    for ref_name in project_refs {
        let project_name = ref_name
            .strip_prefix("refs/jin/layers/project/")
            .unwrap_or(&ref_name);

        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:20} {}", project_name, short_oid);
            }
        }
    }

    Ok(())
}

// ===== GOTCHA: Accessing scope::unescape_from_ref =====
// The function is NOT public in scope.rs, so we need to either:
// 1. Make it public in scope.rs (add pub to fn unescape_from_ref)
// 2. Copy the function to list.rs
// 3. Create a helper module for shared utilities

// Recommendation: Make unescape_from_ref and escape_for_ref public in scope.rs
// Change line 395 in scope.rs from: fn escape_for_ref
// To: pub fn escape_for_ref
// Change line 411 in scope.rs from: fn unescape_from_ref
// To: pub fn unescape_from_ref
```

### Integration Points

```yaml
MODIFY src/commands/mod.rs:
  - add to: After line 18 (alphabetical order)
  - pattern: |
      pub mod list;
      pub use list::execute as list_execute;

MODIFY src/main.rs:
  - replace lines: 196-199
  - pattern: |
      Commands::List => match commands::list_execute() {
          Ok(()) => ExitCode::SUCCESS,
          Err(e) => {
              eprintln!("Error: {}", e);
              ExitCode::FAILURE
          }
      }

MODIFY src/commands/scope.rs (if needed):
  - make functions public: lines 395, 411
  - add pub keyword to escape_for_ref and unescape_from_ref
  - reason: Allow list.rs to use the unescaping logic
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --package jin-glm                    # Check compilation
cargo clippy --package jin-glm -- -D warnings   # Lint with warnings as errors

# Project-wide validation after all changes
cargo check
cargo clippy -- -D warnings

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the list command specifically
cargo test --package jin-glm list -- --nocapture

# Test all commands to ensure no regression
cargo test --package jin-glm commands

# Full test suite
cargo test

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual testing in a test repository
cd /tmp && mkdir test-jin-list && cd test-jin-list
git init
jin init

# Create test modes
jin mode create claude
jin mode create cursor
jin mode use claude

# Create test scopes
jin scope create language:rust
jin scope create backend --mode=claude
jin scope use language:rust

# Run list command
jin list

# Expected output:
# Active mode: claude
# Active scope: language:rust
#
# Available modes:
#   claude               <8-char-oid>
#   cursor               <8-char-oid>
#
# Available scopes:
#   language:rust        [untethered] <8-char-oid>
#   backend              [mode:claude] <8-char-oid>
#
# No projects found.

# Test empty state
cd /tmp && mkdir test-jin-list-empty && cd test-jin-list-empty
git init
jin init
jin list

# Expected: No active mode or scope, No modes found, No scopes found, No projects found
```

### Level 4: CLI Output Validation

```bash
# Verify formatting alignment
jin list | cat -A

# Check that columns align correctly
# Verify no trailing whitespace
# Confirm consistent spacing

# Test error handling
cd /tmp
mkdir test-no-jin && cd test-no-jin
jin list

# Expected: "Jin is not initialized in this directory."
```

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` passes with zero errors
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo test` passes all tests (including new list tests)
- [ ] `cargo run -- list` produces expected output in test repository
- [ ] Error message displays when not in Jin-initialized directory

### Feature Validation

- [ ] Active context displays (mode and/or scope)
- [ ] "No active mode or scope" displays when both are None
- [ ] All modes listed with correct formatting
- [ ] All scopes listed with [untethered] or [mode:X] labels
- [ ] All projects listed (or "No projects found")
- [ ] Short OIDs (8 characters) displayed correctly
- [ ] Scope names with colons display unescaped (language:rust not language%3Arust)

### Code Quality Validation

- [ ] Module docstring matches pattern of other commands
- [ ] Function docstrings follow existing patterns
- [ ] Error handling matches status.rs pattern
- [ ] Display formatting matches mode.rs/scope.rs patterns
- [ ] Test coverage includes: success case, empty states, error case

### Integration Validation

- [ ] `src/commands/mod.rs` exports list module correctly
- [ ] `src/main.rs` dispatches List command to list_execute
- [ ] No regression in existing `jin modes` or `jin scopes` commands
- [ ] Consistent output format across all three sections

---

## Anti-Patterns to Avoid

- **Don't** create separate functions for untethered vs mode-bound scopes - handle both in display_available_scopes()
- **Don't** use inconsistent OID formatting - always 8 characters
- **Don't** skip the Jin initialization check - it must be first in execute()
- **Don't** forget to unescape scope names with colons - use scope::unescape_from_ref()
- **Don't** use inconsistent spacing/formatting from existing commands - follow {:20} and {:30} patterns
- **Don't** create a new test helper pattern - follow status.rs DirGuard and init_jin patterns
- **Don't** forget to handle empty states for all three sections (modes, scopes, projects)
- **Don't** create a complex dependency on table formatting crates - use simple println! formatting
- **Don't** modify existing execute_list() functions in mode.rs or scope.rs - they work independently
- **Don't** add unnecessary flags or options to ListCommand - keep it as a simple unit struct

---

## Confidence Score

**8/10** for one-pass implementation success likelihood

**Reasoning**:
- Clear existing patterns to follow (status.rs, mode.rs, scope.rs)
- All necessary context provided with specific file references
- Simple command with no new data structures
- Potential issues:
  - May need to make scope::unescape_from_ref public (minor change)
  - Project listing is uncharted territory but follows same pattern as modes
  - Test isolation requires careful setup (pattern exists to follow)

**Mitigation for 2-point deduction**:
- Explicit instructions for making unescape_from_ref public if needed
- Complete test pattern from status.rs to follow
- Manual integration testing steps provided
