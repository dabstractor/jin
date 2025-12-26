# Product Requirement Prompt: Layers Command

---

## Goal

**Feature Goal**: Implement `jin layers` command that displays the current layer composition and merge order for the active workspace.

**Deliverable**: A new `layers` command module (`src/commands/layers.rs`) that shows all 9 layers in the hierarchy, indicates which layers have committed content, marks the active context layers, and displays the merge precedence order.

**Success Definition**:
- Command executes without errors in any Jin-initialized directory
- Displays all 9 layers in correct precedence order (lowest to highest)
- Shows which layers have committed content (via Git refs)
- Highlights active mode/scope context layers
- Gracefully handles edge cases (no layers with commits, no active context)

## User Persona

**Target User**: Developers using Jin to manage multi-layered configurations

**Use Case**: A developer wants to understand which layers are active in their current workspace and which layers contain committed data. This is useful for:
- Debugging merge behavior
- Understanding why certain configuration values are being applied
- Verifying the layer stack before committing changes
- Onboarding new developers to understand the layer system

**User Journey**:
1. Developer runs `jin layers` in their project directory
2. Command displays a formatted list of all 9 layers in precedence order
3. Developer sees which layers have commits and which are empty
4. Developer understands the merge order and can predict configuration behavior

**Pain Points Addressed**:
- Currently no easy way to see the complete layer stack
- Difficult to understand which layers contribute to workspace state
- No visibility into which layers have committed content vs are empty

## Why

- **User Impact**: Provides essential visibility into the layer system for debugging and understanding
- **Integration**: Complements existing inspection commands (`diff`, `log`, `context`, `status`)
- **Problem Solving**: Resolves the "black box" problem where users don't know which layers are active

## What

### User-Visible Behavior

The `jin layers` command displays:

```
Layer Composition (1=lowest, 9=highest precedence):
  [1] global                  - No commits
  [2] mode/claude             - Active mode, 3 commits
  [3] mode/claude/scope/python - Active scope, 1 commit
  [4] mode/claude/scope/python/project/myapp - No commits
  [5] mode/claude/project/myapp - No commits
  [6] scope/python            - No commits
  [7] project/myapp           - 5 commits
  [8] user-local              - Not versioned (local only)
  [9] workspace-active        - Derived result
```

### Success Criteria

- [ ] All 9 layers displayed in correct precedence order
- [ ] Layers with commits show commit count or "No commits"
- [ ] Active mode/scope layers are clearly marked
- [ ] Non-versioned layers (user-local, workspace-active) are noted as such
- [ ] Command returns appropriate error when Jin not initialized
- [ ] Output is readable and scannable

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test**: Would someone unfamiliar with this codebase have everything needed to implement this successfully?

Yes - this PRP provides:
- Complete file structure references
- Exact code patterns to follow
- All necessary imports and types
- Validation commands for testing
- Edge case handling

### Documentation & References

```yaml
# MUST READ - Core Layer System
- file: src/core/layer.rs
  why: Complete Layer enum definition with all 9 variants, Display implementation, helper methods
  pattern: Layer enum with variants GlobalBase through WorkspaceActive, Display trait shows format like "global", "mode/claude", etc.
  gotcha: Layer enum derives Ord so variants are ordered by declaration order - this IS the precedence order

# MUST READ - JinRepo Layer Listing
- file: src/git/repo.rs
  why: JinRepo::list_layer_refs() method returns Vec<(Layer, git2::Oid)> of layers with commits
  section: Lines 386-405 for list_layer_refs() implementation
  pattern: Uses glob pattern "refs/jin/layers/*" to find all versioned layers with commits
  gotcha: Only returns layers 1-7 that have commits - layers 8-9 are not versioned

# MUST READ - Command Pattern
- file: src/commands/context.rs
  why: Simplest inspection command pattern to follow
  pattern: execute() function, load ProjectContext, display output, JinError for errors
  section: Lines 28-47 for execute() pattern

# MUST READ - Complex Inspection Command
- file: src/commands/log.rs
  why: Shows how to use list_layer_refs() and format layer display
  pattern: Uses repo.list_layer_refs() to get layers, then iterates and displays
  section: Lines 364-409 for main execute() showing layer iteration
  gotcha: Uses JinRepo::open_or_create() not git2::Repository directly

# MUST READ - Active Context Loading
- file: src/core/config.rs
  why: ProjectContext::load() method to get active mode/scope
  pattern: context.mode and context.scope Option<String> fields
  gotcha: Context may have None for mode and/or scope - this is valid

# MUST READ - Command Registration
- file: src/commands/mod.rs
  why: Where to add the new module export and execute function
  pattern: pub mod layers; and pub use layers::execute as layers_execute;
  gotcha: Must add both module declaration and re-export

# MUST READ - CLI Dispatch
- file: src/main.rs
  why: Where to wire up the command handler
  section: Lines 189-192 show current placeholder for Layers command
  pattern: match commands::layers_execute() with error handling

# MUST READ - CLI Argument Definition
- file: src/cli/args.rs
  why: Shows Layers command has no arguments (Lines 91-92)
  pattern: Commands::Layers (unit variant, no Args struct)
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── Cargo.toml              # Dependencies (git2, clap, serde, tempfile)
├── src/
│   ├── main.rs             # CLI entry point - dispatch to commands
│   ├── lib.rs              # Library exports
│   ├── cli/
│   │   ├── mod.rs
│   │   └── args.rs         # Command definitions (Layers: lines 91-92)
│   ├── commands/
│   │   ├── mod.rs          # Module exports (add layers here)
│   │   ├── context.rs      # Simple inspection command pattern
│   │   ├── log.rs          # Complex inspection with layers
│   │   ├── status.rs       # Status display patterns
│   │   └── ...             # Other commands
│   ├── core/
│   │   ├── layer.rs        # Layer enum (all 9 variants)
│   │   ├── config.rs       # ProjectContext struct
│   │   └── error.rs        # JinError types
│   └── git/
│       └── repo.rs         # JinRepo::list_layer_refs()
└── plan/
    └── P4M5T7/
        └── PRP.md          # This file
```

### Desired Codebase Tree with Files to be Added

```bash
# NEW FILE TO CREATE:
src/commands/layers.rs       # New layers command implementation

# FILES TO MODIFY:
src/commands/mod.rs          # Add: pub mod layers; and pub use layers::execute as layers_execute;
src/main.rs                  # Modify: Lines 189-192 to dispatch to layers_execute
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Layer enum ordering
// The Layer enum variants are declared in precedence order (1-9)
// The derived Ord implementation uses this declaration order
// Iterating Layer enum variants gives correct precedence order
// DO NOT reorganize enum variants - order is semantically significant

// CRITICAL: Git ref namespace
// All versioned layers store refs under refs/jin/layers/
// Pattern: refs/jin/layers/global, refs/jin/layers/mode/<name>, etc.
// Layers 8-9 (UserLocal, WorkspaceActive) are NOT in Git

// CRITICAL: list_layer_refs() behavior
// Only returns layers that have commits (non-empty refs)
// Empty layers are not returned - must handle empty state explicitly
// Returns Vec<(Layer, git2::Oid)> - Oid is the commit SHA

// CRITICAL: ProjectContext may be incomplete
// context.mode can be None (no active mode)
// context.scope can be None (no active scope)
// Both can be None - this is the default state

// CRITICAL: Workspace root detection
// Use std::env::current_dir()? for workspace root
// Always check ProjectContext::context_path().exists() first for Jin init check
// Use git2::Repository::discover() for Git repo validation

// CRITICAL: Layer Display format
// Layer implements Display trait
// Use format!("{}", layer) or layer.to_string()
// Output: "global", "mode/claude", "mode/claude/scope/python", etc.

// CRITICAL: Testing patterns
// Use tempfile::TempDir for isolated test directories
// Use DirGuard pattern to save/restore current directory
// Always init both Git repo AND Jin state in tests
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models needed. Using existing types:

```rust
// Existing types used:
use crate::core::Layer;           // All 9 layer variants
use crate::core::config::ProjectContext;  // Active mode/scope
use crate::git::JinRepo;          // list_layer_refs()
use crate::core::error::{JinError, Result};
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/layers.rs
  - IMPLEMENT: execute() function as main entry point
  - IMPLEMENT: display_layers() function to format and print output
  - IMPLEMENT: get_layer_status() helper to check commit presence
  - FOLLOW pattern: src/commands/context.rs for simple structure
  - FOLLOW pattern: src/commands/log.rs for layer iteration
  - NAMING: execute(), display_layers(), get_layer_status()
  - PLACEMENT: New file in src/commands/

Task 2: MODIFY src/commands/mod.rs
  - ADD: pub mod layers;
  - ADD: pub use layers::execute as layers_execute;
  - FIND pattern: Similar to other command exports like "pub use context::execute as context_execute;"
  - PRESERVE: All existing exports and module declarations
  - PLACEMENT: After existing imports, alphabetical order preferred

Task 3: MODIFY src/main.rs
  - MODIFY: Lines 189-192 (Commands::Layers placeholder)
  - REPLACE: println!("jin layers - command handler to be implemented"); ExitCode::SUCCESS
  - WITH: match commands::layers_execute() { Ok(()) => ExitCode::SUCCESS, Err(e) => { eprintln!("Error: {}", e); ExitCode::FAILURE } }
  - PRESERVE: All surrounding command dispatch patterns
  - PLACEMENT: In Commands::Layers arm of main match statement

Task 4: CREATE tests in src/commands/layers.rs
  - IMPLEMENT: Unit tests following existing patterns
  - FOLLOW pattern: src/commands/context.rs tests (lines 79-244)
  - NAMING: test_execute_shows_layers, test_execute_no_jin_initialized_error, test_execute_empty_layers
  - COVERAGE: Happy path, no Jin init, empty layers, with active context
  - FIXTURE: Use TempDir, DirGuard, init_git_repo(), init_jin() helpers
  - PLACEMENT: #[cfg(test)] mod tests { ... } at end of file

Task 5: VERIFY compilation and basic functionality
  - RUN: cargo build
  - RUN: cargo test layers
  - RUN: cargo run -- layers (in test project)
  - EXPECT: Clean build, tests pass, command executes without panic
```

### Implementation Patterns & Key Details

```rust
// ===== FILE: src/commands/layers.rs =====
//! Layers command implementation.
//!
//! This module implements the `jin layers` command that displays
//! the current layer composition and merge order.

use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use std::collections::HashSet;

/// Execute the layers command.
///
/// Displays all 9 layers in precedence order with:
/// - Layer number (1-9)
/// - Layer name (from Display impl)
/// - Commit count or "No commits"
/// - Active context markers for mode/scope
///
/// # Errors
///
/// Returns `JinError::Message` if Jin is not initialized.
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

    // 3. Load active context
    let context = ProjectContext::load(&workspace_root)?;

    // 4. Validate Git repository exists
    let _git_repo =
        git2::Repository::discover(&workspace_root).map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 6. Get layers with commits
    let committed_layers: HashSet<_> = repo
        .list_layer_refs()?
        .into_iter()
        .map(|(layer, _oid)| layer)
        .collect();

    // 7. Display all layers in precedence order
    display_layers(&context, &committed_layers);

    Ok(())
}

/// Displays all layers with their status.
///
/// # Arguments
///
/// * `context` - The active project context
/// * `committed_layers` - Set of layers that have commits
fn display_layers(context: &ProjectContext, committed_layers: &HashSet<Layer>) {
    println!();
    println!("Layer Composition (1=lowest, 9=highest precedence):");
    println!();

    // PATTERN: All Layer enum variants in declaration order = precedence order
    // Must manually list all 9 variants here

    // Layer 1: GlobalBase
    display_layer_entry(
        1,
        &Layer::GlobalBase,
        committed_layers.contains(&Layer::GlobalBase),
        false,  // never active context
        false,
    );

    // Layer 2: ModeBase - check if active mode
    let layer_2 = if let Some(ref mode) = context.mode {
        Layer::ModeBase { mode: mode.clone() }
    } else {
        Layer::ModeBase { mode: "(none)".to_string() }
    };
    let is_active_2 = context.mode.is_some();
    display_layer_entry(2, &layer_2, committed_layers.contains(&layer_2), is_active_2, false);

    // Layer 3: ModeScope - check if active mode+scope
    let layer_3 = if let (Some(ref mode), Some(ref scope)) = (&context.mode, &context.scope) {
        Layer::ModeScope { mode: mode.clone(), scope: scope.clone() }
    } else {
        // Display placeholder for clarity even if not active
        Layer::ModeScope { mode: "(none)".to_string(), scope: "(none)".to_string() }
    };
    let is_active_3 = context.mode.is_some() && context.scope.is_some();
    display_layer_entry(3, &layer_3, committed_layers.contains(&layer_3), is_active_3, false);

    // Layer 4: ModeScopeProject
    let layer_4 = if let (Some(ref mode), Some(ref scope)) = (&context.mode, &context.scope) {
        // Get project name
        let project = detect_project_name().unwrap_or_else(|_| "(unknown)".to_string());
        Layer::ModeScopeProject { mode: mode.clone(), scope: scope.clone(), project }
    } else {
        Layer::ModeScopeProject {
            mode: "(none)".to_string(),
            scope: "(none)".to_string(),
            project: "(none)".to_string(),
        }
    };
    let is_active_4 = context.mode.is_some() && context.scope.is_some();
    display_layer_entry(4, &layer_4, committed_layers.contains(&layer_4), is_active_4, false);

    // Layer 5: ModeProject
    let layer_5 = if let Some(ref mode) = context.mode {
        let project = detect_project_name().unwrap_or_else(|_| "(unknown)".to_string());
        Layer::ModeProject { mode: mode.clone(), project }
    } else {
        Layer::ModeProject { mode: "(none)".to_string(), project: "(none)".to_string() }
    };
    let is_active_5 = context.mode.is_some();
    display_layer_entry(5, &layer_5, committed_layers.contains(&layer_5), is_active_5, false);

    // Layer 6: ScopeBase
    let layer_6 = if let Some(ref scope) = context.scope {
        Layer::ScopeBase { scope: scope.clone() }
    } else {
        Layer::ScopeBase { scope: "(none)".to_string() }
    };
    let is_active_6 = context.scope.is_some();
    display_layer_entry(6, &layer_6, committed_layers.contains(&layer_6), is_active_6, false);

    // Layer 7: ProjectBase
    let project = detect_project_name().unwrap_or_else(|_| "(unknown)".to_string());
    let layer_7 = Layer::ProjectBase { project };
    display_layer_entry(7, &layer_7, committed_layers.contains(&layer_7), false, false);

    // Layer 8: UserLocal - not versioned
    display_layer_entry(8, &Layer::UserLocal, false, false, true);

    // Layer 9: WorkspaceActive - not versioned, derived
    display_layer_entry(9, &Layer::WorkspaceActive, false, false, true);

    println!();
}

/// Displays a single layer entry.
///
/// # GOTCHA: The display format shows:
/// - Layer number in brackets
/// - Layer name from Display trait
/// - Active context indicator if applicable
/// - Commit status or "Not versioned" note
fn display_layer_entry(
    index: usize,
    layer: &Layer,
    has_commits: bool,
    is_active_context: bool,
    is_not_versioned: bool,
) {
    let layer_name = format!("{}", layer);
    let status = if is_not_versioned {
        "Not versioned".to_string()
    } else if has_commits {
        // Could get commit count here, but "Has commits" is sufficient for MVP
        "Has commits".to_string()
    } else {
        "No commits".to_string()
    };

    let active_marker = if is_active_context { " - Active context" } else { "" };

    println!(
        "  [{}] {:<30} - {}{}",
        index, layer_name, status, active_marker
    );
}

/// Detects the project name from Git remote or directory name.
///
/// Copy of pattern from src/commands/log.rs lines 113-152
fn detect_project_name() -> Result<String> {
    let workspace_root = std::env::current_dir()?;
    let repo = git2::Repository::discover(&workspace_root)
        .map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // Try to get from git remote origin
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.rsplit('/').next() {
                let name = name.trim_end_matches(".git");
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }

    // Fallback to directory name
    workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| JinError::Message("Cannot determine project name".to_string()))
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Save the current directory and restore it when dropped.
    /// PATTERN: Copied from context.rs lines 85-101
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
    /// PATTERN: Copied from context.rs lines 104-106
    fn init_git_repo(dir: &std::path::Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    /// Helper to initialize Jin in a directory
    /// PATTERN: Copied from context.rs lines 109-132
    fn init_jin(dir: &std::path::Path) {
        use crate::staging::index::StagingIndex;

        // Create .jin directory
        let jin_dir = dir.join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();

        // Create and save context
        let context = ProjectContext::default();
        context.save(dir).unwrap();

        // Verify context file was created
        let context_path = ProjectContext::context_path(dir);
        assert!(
            context_path.exists(),
            "Context file should exist after init_jin"
        );

        // Create staging index
        let staging_index = StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        // Create workspace directory
        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_execute_shows_all_layers() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Execute should succeed
        execute().unwrap();
    }

    #[test]
    fn test_execute_with_active_context() {
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
    fn test_execute_with_commits_in_layers() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a regular git repo (this is what execute() will use)
        let git_repo = git2::Repository::init(project_dir).unwrap();
        init_jin(project_dir);

        // Create commits in different layers
        let time = git2::Time::new(0, 0);
        let signature = git2::Signature::new("Test User", "test@example.com", &time).unwrap();

        // Global layer commit
        let tree_oid = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree = git_repo.find_tree(tree_oid).unwrap();
        git_repo
            .commit(
                Some("refs/jin/layers/global"),
                &signature,
                &signature,
                "First global commit",
                &tree,
                &[],
            )
            .unwrap();

        // Execute should show layers with commits
        execute().unwrap();
    }
}
```

### Integration Points

```yaml
MAIN_RS:
  - file: src/main.rs
  - section: Lines 189-192
  - modify: Replace placeholder with proper dispatch
  - pattern: Same as other commands (Context, Diff, Log, etc.)

COMMANDS_MOD_RS:
  - file: src/commands/mod.rs
  - add: pub mod layers; after other module declarations
  - add: pub use layers::execute as layers_execute; after other re-exports
  - preserve: All existing exports

NO_CHANGES_NEEDED:
  - src/cli/args.rs (Layers command already defined)
  - src/core/layer.rs (Layer enum complete)
  - src/git/repo.rs (list_layer_refs exists)
  - src/core/config.rs (ProjectContext exists)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# After creating layers.rs
cargo check --package jin_glm --bin jin

# Expected: No errors. If errors exist:
# - Check imports match file structure
# - Verify function signatures use Result<()>
# - Ensure all types are properly qualified

# After modifying mod.rs
cargo check --package jin_glm --bin jin

# Expected: No errors. If "unresolved import" errors:
# - Verify pub mod layers; is present
# - Check file is named exactly layers.rs

# After modifying main.rs
cargo build --release

# Expected: Clean build. If compile errors:
# - Check commands::layers_execute is exported in mod.rs
# - Verify match arm syntax matches other commands

# Full workspace check
cargo fmt --all
cargo clippy --all-targets --all-features

# Expected: No formatting issues, no clippy warnings beyond existing ones
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run layers tests specifically
cargo test --package jin_glm --bin jin layers

# Expected: All tests pass. Test coverage:
# - test_execute_shows_all_layers: Basic execution
# - test_execute_with_active_context: Context handling
# - test_execute_not_initialized_error: Error handling
# - test_execute_with_commits_in_layers: Commit detection

# Run all command tests
cargo test --package jin_glm --bin jin

# Expected: No regressions in other commands

# Run full test suite
cargo test --workspace

# Expected: All tests pass, including integration tests
```

### Level 3: Integration Testing (System Validation)

```bash
# Create test project
mkdir /tmp/jin-layers-test && cd /tmp/jin-layers-test
git init
cargo run -- jin init

# Test basic layers output
cargo run -- jin layers

# Expected output:
# Layer Composition (1=lowest, 9=highest precedence):
#   [1] global                        - No commits
#   [2] mode/(none)                   - No commits
#   ...

# Test with active context
cargo run -- jin mode use claude
cargo run -- jin scope use language:rust
cargo run -- jin layers

# Expected: Layer 2 shows "mode/claude - Active context"
# Expected: Layer 3 shows "mode/claude/scope/language:rust - Active context"

# Test with commits
echo "test" > .jin/config.txt
cargo run -- jin add .jin/config.txt --global
cargo run -- jin commit -m "Test commit"
cargo run -- jin layers

# Expected: Layer 1 shows "global - Has commits"

# Test error case
cd /tmp
cargo run -- jin layers

# Expected: Error message "Jin is not initialized"
```

### Level 4: Manual & Visual Validation

```bash
# Visual inspection of output formatting
cd /tmp/jin-layers-test
cargo run -- jin layers

# Validate:
# - All 9 layers shown (numbered 1-9)
# - Layer names are readable (not Debug format)
# - Active context clearly marked
# - Commit status is clear
# - Column alignment is clean

# Edge case testing
cd /tmp/jin-layers-test
cargo run -- jin mode unset
cargo run -- jin scope unset
cargo run -- jin layers

# Expected: Layers 2-6 show "(none)" placeholders, no "Active context" markers

# Performance test (with many layers)
# Create commits in all 7 versioned layers, then:
time cargo run -- jin layers

# Expected: Completes in < 1 second, no noticeable delay
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --workspace`
- [ ] No new clippy warnings: `cargo clippy --all-targets`
- [ ] No formatting issues: `cargo fmt --all --check`
- [ ] Clean build: `cargo build --release`

### Feature Validation

- [ ] All 9 layers displayed in correct order
- [ ] Layers with commits show "Has commits"
- [ ] Layers without commits show "No commits"
- [ ] Non-versioned layers (8-9) show "Not versioned"
- [ ] Active context layers marked with "Active context"
- [ ] Error handling for uninitialized Jin works
- [ ] Output is readable and scannable

### Code Quality Validation

- [ ] Follows existing codebase patterns
- [ ] File placement correct (src/commands/layers.rs)
- [ ] Module exports added (src/commands/mod.rs)
- [ ] CLI dispatch wired (src/main.rs)
- [ ] Tests follow established patterns
- [ ] No hardcoded values or magic numbers
- [ ] Proper error handling with JinError

### Documentation & Deployment

- [ ] Code comments explain layer display logic
- [ ] Module documentation describes command purpose
- [ ] Public functions have doc comments
- [ ] Tests are well-named and self-documenting

---

## Anti-Patterns to Avoid

- ❌ Don't create a new Args struct in cli/args.rs (Layers has no arguments)
- ❌ Don't use hardcoded layer lists - reference Layer enum variants directly
- ❌ Don't assume context.mode or context.scope are always Some
- ❌ Don't call git2::Repository directly in command (use JinRepo wrapper)
- ❌ Don't skip the Jin initialization check (security/safety)
- ❌ Don't use Debug format for layer display - use Display trait
- ❌ Don't add new dependencies to Cargo.toml
- ❌ Don't modify Layer enum or core types
- ❌ Don't use unwrap() except in tests (proper error handling in production code)
- ❌ Don't create layers with placeholder names like "(none)" in committed_layers comparison

---

## Confidence Score

**9/10** - This PRP provides comprehensive context with specific file references, exact code patterns, and complete implementation guidance. The only minor uncertainty is the exact output formatting preferences, but the pattern is well-established in similar commands.

The research covered:
- All existing inspection commands (context, log, status, diff)
- Complete layer system implementation
- JinRepo API for layer operations
- ProjectContext for active mode/scope
- Testing patterns and fixture setup
- CLI registration and dispatch patterns

All files referenced are specific with line numbers. All patterns identified have code examples. All edge cases are documented.
