# Product Requirement Prompt (PRP): Context Command

## Goal

**Feature Goal**: Implement `jin context` command that displays the current active context (mode, scope, project, and last-updated timestamp) from `.jin/context`.

**Deliverable**: A working `jin context` command that loads and displays ProjectContext information in a clean, user-friendly format.

**Success Definition**:
- `jin context` command executes without errors
- Displays active mode if set
- Displays active scope if set
- Shows context file location
- Shows "No active context" message when nothing is set
- Follows existing command patterns from status/mode/scope commands
- Includes comprehensive unit tests

## User Persona

**Target User**: Developers working with Jin layer system who need to quickly see their current active context (mode/scope configuration).

**Use Case**: Developer wants to verify which mode and scope are currently active before running other Jin commands, to understand which layers will be affected by operations.

**User Journey**:
1. Developer runs `jin context`
2. Command loads `.jin/context` file
3. Display shows active mode (if any)
4. Display shows active scope (if any)
5. Display indicates context file location
6. If no context is set, shows clear "No active context" message

**Pain Points Addressed**:
- Avoids running `jin mode show` and `jin scope show` separately
- Provides single command to check current configuration state
- Helps verify context before running potentially destructive operations

## Why

- **Consolidated View**: Current system requires separate `jin mode show` and `jin scope show` commands to see full context
- **Quick Verification**: Users need to verify active context before operations like commit, reset, or apply
- **User Experience**: Matches common CLI patterns (git status, kubectl config current-context)
- **Completeness**: Part of P4.M5 Utility Commands milestone, filling inspection command gap

## What

The `jin context` command displays the current active context from `.jin/context`:

```bash
$ jin context
Active mode: claude
Active scope: language:rust
Context file: /home/user/project/.jin/context
```

Or when no context is set:
```bash
$ jin context
No active mode or scope
Context file: /home/user/project/.jin/context
```

### Success Criteria

- [ ] Command loads ProjectContext from `.jin/context`
- [ ] Displays active mode if set, otherwise "No active mode"
- [ ] Displays active scope if set, otherwise "No active scope"
- [ ] Shows context file location for debugging
- [ ] Returns error if Jin is not initialized
- [ ] Follows same patterns as status/mode/scope commands
- [ ] Includes unit tests for all scenarios

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**YES** - This PRP provides:
- Exact file structures and patterns to follow
- Complete ProjectContext API reference
- Specific code patterns from existing commands
- Validation commands that work in this codebase
- Error handling patterns to follow

### Documentation & References

```yaml
# MUST READ - Core types and patterns
- file: /home/dustin/projects/jin-glm-doover/src/core/config.rs
  why: Contains ProjectContext struct definition with all fields and methods
  pattern: ProjectContext has mode (Option<String>), scope (Option<String>), version (u8)
  critical: Context.load() returns default if file doesn't exist; context_path() gives file location
  section: lines 195-379 (ProjectContext struct and impl)

- file: /home/dustin/projects/jin-glm-doover/src/commands/status.rs
  why: Reference for context display pattern (display_active_context function)
  pattern: Shows mode/scope with simple println! statements
  gotcha: Uses ProjectContext::context_path() to check Jin initialization
  section: lines 41-89 (execute and display_active_context functions)

- file: /home/dustin/projects/jin-glm-doover/src/commands/mode.rs
  why: Reference for execute_show pattern (mode show command)
  pattern: Loads context, checks mode field, displays with println!
  gotcha: execute_show takes workspace_root: &Path, returns Result<()>
  section: lines 239-266 (execute_show function)

- file: /home/dustin/projects/jin-glm-doover/src/commands/scope.rs
  why: Reference for scope display pattern (scope show command)
  pattern: Similar to mode show, displays both mode and scope
  gotcha: Shows complementary context (mode when showing scope, and vice versa)
  section: lines 347-374 (execute_show function)

- file: /home/dustin/projects/jin-glm-doover/src/commands/mod.rs
  why: Shows module export pattern - need to add context module
  pattern: pub use context::execute as context_execute;
  critical: Must add pub mod context; and export line to wire command

- file: /home/dustin/projects/jin-glm-doover/src/main.rs
  why: Shows CLI command routing pattern
  pattern: Commands::Context => match commands::context_execute()
  section: lines 182-185 (Context command placeholder)

- file: /home/dustin/projects/jin-glm-doover/src/cli/args.rs
  why: Contains ContextCommand enum definition
  pattern: Commands::Context is a unit variant (no fields)
  section: line 89 (Commands::Context definition)

# External Documentation - Best Practices
- url: https://github.com/clap-rs/clap
  why: Rust CLI argument parsing - already using in project
  critical: Command is already defined in CLI args, just needs implementation

- url: https://github.com/console-rs/console
  why: Terminal styling and colors (optional enhancement)
  note: Not required but nice-to-have for color-coded output
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   ├── cli/
│   │   └── args.rs              # CLI definitions (ContextCommand defined at line 89)
│   ├── commands/
│   │   ├── mod.rs               # Module exports (needs context export added)
│   │   ├── status.rs            # Reference: display_active_context() pattern
│   │   ├── mode.rs              # Reference: execute_show() pattern
│   │   ├── scope.rs             # Reference: execute_show() pattern
│   │   ├── diff.rs              # Reference for load context pattern
│   │   └── log.rs               # Reference for detect_project_name pattern
│   ├── core/
│   │   ├── config.rs            # ProjectContext struct definition
│   │   └── error.rs             # JinError types
│   └── main.rs                  # CLI routing (line 182-185 has Context placeholder)
├── plan/
│   ├── P4M5T3/
│   │   └── PRP.md               # This file
│   └── docs/
│       └── PRD.md               # Product requirements
└── tests/
    └── ...
```

### Desired Codebase Tree (Files to Add)

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   ├── commands/
│   │   ├── context.rs           # NEW: Context command implementation
│   │   └── mod.rs               # MODIFY: Add context module export
└── src/main.rs                  # MODIFY: Wire Context command handler
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Jin initialization check pattern
// Must use ProjectContext::context_path() to check if .jin/context exists
// DO NOT use ProjectContext::load() - it returns default if file missing!
let context_path = ProjectContext::context_path(&workspace_root);
if !context_path.exists() {
    return Err(JinError::Message(
        "Jin is not initialized in this directory.\n\
         Run 'jin init' to initialize."
            .to_string(),
    ));
}

// CRITICAL: Error handling pattern
// All commands return Result<()>
// Use JinError::Message for user-facing errors
// Use ? operator for propagation

// PATTERN: Workspace root detection
let workspace_root = std::env::current_dir()?;

// PATTERN: Display format (from status.rs display_active_context)
// Use simple println! for output (no complex formatting)
// Match on Option for conditional display
if let Some(mode) = &context.mode {
    println!("Active mode: {}", mode);
}
if let Some(scope) = &context.scope {
    println!("Active scope: {}", scope);
}

// GOTCHA: Commands::Context is a unit variant (no arguments)
// No need to define a ContextCommand struct
// The execute() function takes no parameters: execute() -> Result<()>

// GOTCHA: mod.rs exports use specific naming
// Export pattern: pub use context::execute as context_execute;
// Function in context.rs must be: pub fn execute() -> Result<()>

// CRITICAL: Test helpers pattern
// Use DirGuard for directory management in tests
// Use init_jin() helper for Jin initialization
// See status.rs lines 147-208 for test helper patterns
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed. The `ProjectContext` struct already exists:

```rust
// From src/core/config.rs (lines 195-204)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectContext {
    /// Context format version (for future migration)
    pub version: u8,
    /// Active mode for this project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Active scope for this project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

// Key methods:
// - load(project_dir: &Path) -> Result<Self>
// - context_path(project_dir: &Path) -> PathBuf
// - has_mode() -> bool
// - has_scope() -> bool
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/context.rs
  - IMPLEMENT: execute() function as main entry point
  - IMPLEMENT: display_context() helper function for output formatting
  - FOLLOW pattern: src/commands/status.rs (display_active_context function)
  - IMPLEMENT: check_jin_initialized() helper following status.rs pattern
  - NAMING: pub fn execute() -> Result<()> (no parameters)
  - PLACEMENT: New file in src/commands/
  - ERROR HANDLING: Return JinError::Message for "Jin is not initialized"

Task 2: MODIFY src/commands/mod.rs
  - ADD: pub mod context;
  - ADD: pub use context::execute as context_execute;
  - FIND pattern: Existing module exports (lines 14-26)
  - PRESERVE: All existing exports
  - PLACEMENT: Add after log export, before status export (alphabetical order)

Task 3: MODIFY src/main.rs
  - REPLACE: Context command placeholder (lines 182-185)
  - IMPLEMENT: Commands::Context => match commands::context_execute()
  - FIND pattern: Status command wiring (lines 42-48) for reference
  - PRESERVE: All existing command handlers
  - PLACEMENT: Keep in same location, replace placeholder implementation

Task 4: CREATE tests in src/commands/context.rs
  - IMPLEMENT: test_execute_shows_mode_and_scope()
  - IMPLEMENT: test_execute_no_context_set()
  - IMPLEMENT: test_execute_not_initialized_error()
  - IMPLEMENT: test_execute_shows_only_mode()
  - IMPLEMENT: test_execute_shows_only_scope()
  - FOLLOW pattern: src/commands/status.rs tests (lines 147-343)
  - USE HELPERS: DirGuard, init_git_repo, init_jin
  - COVERAGE: All execute() code paths

Task 5: RUN validation
  - EXECUTE: cargo build (check for compilation)
  - EXECUTE: cargo test (verify all tests pass)
  - EXECUTE: cargo clippy (check for warnings)
  - EXECUTE: cargo fmt (check formatting)
  - MANUAL: Test jin context in actual project
```

### Implementation Patterns & Key Details

```rust
// ===== FILE: src/commands/context.rs =====

//! Context command implementation.
//!
//! This module implements the `jin context` command that displays
//! the current active context (mode, scope) from `.jin/context`.

use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use std::path::Path;

/// Execute the context command.
///
/// Displays the current active context including:
/// - Active mode (if set)
/// - Active scope (if set)
/// - Context file location
///
/// # Errors
///
/// Returns `JinError::Message` if Jin is not initialized.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::commands::context;
///
/// context::execute()?;
/// ```
pub fn execute() -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize."
                .to_string(),
        ));
    }

    // 3. Load and display context
    let context = ProjectContext::load(&workspace_root)?;
    display_context(&context, &context_path);

    Ok(())
}

/// Displays the active context information.
///
/// # Arguments
///
/// * `context` - The project context to display
/// * `context_path` - Path to the context file (for display)
fn display_context(context: &ProjectContext, context_path: &Path) {
    println!();

    // Display mode
    if let Some(mode) = &context.mode {
        println!("Active mode: {}", mode);
    } else {
        println!("No active mode");
    }

    // Display scope
    if let Some(scope) = &context.scope {
        println!("Active scope: {}", scope);
    } else {
        println!("No active scope");
    }

    // Display context file location
    println!();
    println!("Context file: {}", context_path.display());
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Save the current directory and restore it when dropped.
    struct DirGuard {
        original_dir: std::path::PathBuf,
    }

    impl DirGuard {
        fn new() -> std::io::Result<Self> {
            Ok(Self {
                original_dir: std::env::current_dir()?,
            })
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    /// Helper to initialize a Git repo
    fn init_git_repo(dir: &Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    /// Helper to initialize Jin in a directory
    fn init_jin(dir: &Path) {
        let jin_dir = dir.join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();

        let context = ProjectContext::default();
        context.save(dir).unwrap();

        let staging_index = crate::staging::index::StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_execute_shows_mode_and_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set mode and scope
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        // Execute should succeed
        execute().unwrap();

        // Verify context is still set
        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("claude".to_string()));
        assert_eq!(loaded.scope, Some("language:rust".to_string()));
    }

    #[test]
    fn test_execute_no_context_set() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Execute with default context (no mode/scope)
        execute().unwrap();

        // Verify no context is set
        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.mode.is_none());
        assert!(context.scope.is_none());
    }

    #[test]
    fn test_execute_not_initialized_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize Jin

        let result = execute();
        assert!(result.is_err());
        if let Err(JinError::Message(msg)) = result {
            assert!(msg.contains("Jin is not initialized"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_execute_shows_only_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set only mode
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("cursor".to_string()));
        context.save(project_dir).unwrap();

        execute().unwrap();

        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("cursor".to_string()));
        assert!(loaded.scope.is_none());
    }

    #[test]
    fn test_execute_shows_only_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set only scope
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_scope(Some("language:javascript".to_string()));
        context.save(project_dir).unwrap();

        execute().unwrap();

        let loaded = ProjectContext::load(project_dir).unwrap();
        assert!(loaded.mode.is_none());
        assert_eq!(loaded.scope, Some("language:javascript".to_string()));
    }
}

// ===== FILE: src/commands/mod.rs (MODIFICATION) =====

// Add these lines:
pub mod context;
pub use context::execute as context_execute;

// ===== FILE: src/main.rs (MODIFICATION) =====

// Replace lines 182-185:
Commands::Context => match commands::context_execute() {
    Ok(()) => ExitCode::SUCCESS,
    Err(e) => {
        eprintln!("Error: {}", e);
        ExitCode::FAILURE
    }
}
```

### Integration Points

```yaml
MAIN_RS:
  - file: src/main.rs
  - lines: 182-185
  - replace: Placeholder implementation with proper error handling

MOD_RS:
  - file: src/commands/mod.rs
  - location: After line 12 (before status module)
  - add: pub mod context;
  - add: pub use context::execute as context_execute;

NEW_FILE:
  - file: src/commands/context.rs
  - location: src/commands/
  - create: Complete implementation with tests
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo build                          # Check compilation
cargo clippy -- -D warnings          # Lint checks
cargo fmt --check                    # Format validation

# Expected: Zero errors, zero warnings
# If errors exist, READ output and fix before proceeding
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test context command specifically
cargo test --package jin-glm context
cargo test context::tests

# Full test suite for commands
cargo test --package jin-glm commands

# Expected: All tests pass
# If failing, debug root cause and fix implementation
```

### Level 3: Integration Testing (System Validation)

```bash
# Initialize a test project
cd /tmp
mkdir test-jin-context
cd test-jin-context
git init
jin init

# Test 1: No context set
jin context
# Expected output:
# No active mode
# No active scope
#
# Context file: /tmp/test-jin-context/.jin/context

# Test 2: Set mode only
jin mode create testmode
jin mode use testmode
jin context
# Expected output:
# Active mode: testmode
# No active scope
#
# Context file: /tmp/test-jin-context/.jin/context

# Test 3: Set scope only
jin mode unset
jin scope create test:scope
jin scope use test:scope
jin context
# Expected output:
# No active mode
# Active scope: test:scope
#
# Context file: /tmp/test-jin-context/.jin/context

# Test 4: Both set
jin mode create testmode2
jin mode use testmode2
jin context
# Expected output:
# Active mode: testmode2
# Active scope: test:scope
#
# Context file: /tmp/test-jin-context/.jin/context

# Test 5: Not in Jin project
cd /tmp
mkdir no-jin-here
cd no-jin-here
jin context
# Expected: Error "Jin is not initialized"

# Expected: All manual tests pass with expected output
```

### Level 4: Cross-Command Validation

```bash
# Verify context consistency across commands
cd /tmp/test-jin-context

# Set context
jin mode use testmode
jin scope use test:scope

# Verify other commands see same context
jin status | grep "Active mode: testmode"
jin status | grep "Active scope: test:scope"

jin mode show | grep "testmode"
jin scope show | grep "test:scope"

# Expected: All commands show consistent context
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin-glm`
- [ ] No linting errors: `cargo clippy -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Code compiles without warnings: `cargo build`

### Feature Validation

- [ ] `jin context` shows mode when set
- [ ] `jin context` shows scope when set
- [ ] `jin context` shows "No active mode" when no mode
- [ ] `jin context` shows "No active scope" when no scope
- [ ] `jin context` shows context file path
- [ ] `jin context` errors when not in Jin project
- [ ] Output format matches status command style
- [ ] Manual testing successful (see Level 3 tests)

### Code Quality Validation

- [ ] Follows existing codebase patterns (status/mode/scope)
- [ ] File placement matches desired structure
- [ ] Module exports added correctly
- [ ] Main.rs routing properly implemented
- [ ] Tests follow status.rs test patterns
- [ ] Error handling matches existing patterns

### Documentation & Deployment

- [ ] Module-level documentation present
- [ ] Function documentation present
- [ ] Example usage in comments
- [ ] Test documentation clear

## Anti-Patterns to Avoid

- [ ] Don't use std::process::exit() - let main.rs handle exit codes
- [ ] Don't create complex output formatting - use simple println!
- [ ] Don't add colors or styling unless requested (keep it simple)
- [ ] Don't use unwrap() in production code - use proper error handling with ?
- [ ] Don't check initialization with load() - use context_path().exists()
- [ ] Don't duplicate test helpers - use existing patterns from status.rs
- [ ] Don't add unnecessary dependencies - use existing imports
- [ ] Don't create new error types - use existing JinError::Message
- [ ] Don't implement subcommands for context - it's a simple display command
- [ ] Don't modify ProjectContext struct - it's already complete

---

## Summary

This PRP provides complete context for implementing the `jin context` command:

1. **Single file to create**: `src/commands/context.rs` with execute() function and display_context() helper
2. **Two files to modify**: `src/commands/mod.rs` (add exports) and `src/main.rs` (wire command)
3. **Clear patterns to follow**: status.rs for display, mode.rs for context loading
4. **Comprehensive tests**: 5 test cases covering all scenarios
5. **Validation commands**: cargo build/test/clippy/fmt for verification
6. **No new data structures**: Uses existing ProjectContext

**Confidence Score**: 9/10 for one-pass implementation success

The context command is straightforward - it loads and displays existing data from ProjectContext. All dependencies are in place, patterns are well-established, and the implementation follows a simple, proven pattern used throughout the codebase.
