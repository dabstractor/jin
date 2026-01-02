# Product Requirement Prompt: Layers Command

---

## Goal

**Feature Goal**: Implement a command that displays the current layer composition and merge order of the Jin project.

**Deliverable**: `jin layers` CLI command that shows all 9 layers in precedence order with their active status, file counts, and storage paths.

**Success Definition**:
- Command executes without errors in all initialization states (initialized, uninitialized)
- Shows current context (mode, scope, project) when active
- Displays all applicable layers in precedence order (1-9)
- Shows active status indicator (✓) for layers with commits
- Shows file count per layer when non-zero
- Provides summary statistics (active layers count, total files)
- Properly filters layers based on active mode/scope context

## User Persona

**Target User**: Developers using Jin for multi-layered configuration management

**Use Case**: User needs to understand which layers are active and contributing files to the current workspace context

**User Journey**:
1. User runs `jin layers` from project directory
2. Command displays current context (mode/scope/project if active)
3. Command lists all applicable layers in precedence order
4. User sees which layers have commits (✓) and file counts
5. User sees summary of active layers and total files

**Pain Points Addressed**:
- Visualizing which layers contribute to current workspace
- Understanding merge precedence order
- Identifying inactive vs active layers
- Debugging configuration composition issues

## Why

- **Visibility**: Users need to see which layers are active and contributing to their workspace
- **Debugging**: When configuration issues arise, understanding layer composition is critical
- **Onboarding**: New users benefit from seeing the 9-layer system in action
- **Transparency**: Shows exactly where files are stored and their precedence relationship

## What

### Command Behavior

```bash
# Basic usage
jin layers

# Output format:
Layer composition for current context:
  Mode:    claude
  Scope:   language:javascript

Merge order (lowest to highest precedence):
  ✓  1. global-base      [jin/global/] (42 files)
      2. mode-base       [jin/mode/claude/]
  ✓  3. mode-scope      [jin/mode/claude/scope/language:javascript/] (8 files)
      4. mode-scope-project [jin/mode/claude/scope/language:javascript/project/]
      5. mode-project    [jin/mode/claude/project/]
  ✓  6. scope-base      [jin/scope/language:javascript/] (15 files)
  ✓  7. project-base    [jin/project/ui-dashboard/] (23 files)
  ✓  8. user-local      [~/.jin/local/] (5 files)
  ✓  9. workspace-active [.jin/workspace/] (0 files)

Active layers: 6 of 9 layers have files
Total files in workspace: 93
```

### Success Criteria

- [ ] Command displays current context when mode/scope/project are set
- [ ] All 9 layers shown in correct precedence order
- [ ] Layers that require mode/scope are filtered when not active
- [ ] Active status indicator (✓) shows for layers with Git commits
- [ ] File counts displayed accurately per layer
- [ ] Storage paths display correctly with dynamic values
- [ ] Summary statistics accurate and informative
- [ ] Returns proper error when not initialized (JinError::NotInitialized)

## All Needed Context

### Context Completeness Check

_Before proceeding, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: YES - This PRP provides all necessary context including:
- Complete layer system specification
- Exact file patterns and naming conventions
- CLI command patterns to follow
- Code examples from similar commands
- Validation procedures specific to this codebase

### Documentation & References

```yaml
# MUST READ - Core Type Definitions
- file: src/core/layer.rs
  why: Defines Layer enum with all 9 layers, precedence(), ref_path(), storage_path(), requires_mode(), requires_scope() methods
  critical: The foundation of layer system - all layer operations depend on this
  pattern: Layer enum with derived Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize

- file: src/core/config.rs
  why: Defines ProjectContext struct with mode, scope, project fields
  critical: Used to load and filter layers based on active context
  pattern: ProjectContext::load() returns Result<ProjectContext>, defaults available

- file: src/git/repo.rs
  why: Defines JinRepo wrapper and Git operations
  critical: JinRepo::open_or_create() for repository access
  pattern: All commands use this pattern for repo access

- file: src/commands/context.rs
  why: Simple command example showing context loading pattern
  pattern: Match ProjectContext::load() with specific NotInitialized error handling

- file: src/commands/log.rs
  why: Complex command with layer filtering and Git reference operations
  pattern: Shows how to filter layers by mode/scope and access Git refs

- file: src/commands/list.rs
  why: Similar information display command with category-based output
  pattern: Clean formatting with sorted lists and summary statistics

# External Research
- url: https://docs.rs/comfy-table/latest/comfy_table/
  why: Rich table formatting crate for potential future enhancements
  section: "Getting Started" and "Column Constraints"

- url: https://docs.rs/git2/latest/git2/
  why: Git2 library documentation for tree walking operations
  section: "struct.Tree.html#method.walk"

- url: https://git-scm.com/docs/git-log
  why: Git's own visualization patterns for hierarchical data
  section: "--pretty=format" options for custom formatting
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs          # Command argument structs (LayersArgs not needed - no args)
│   └── mod.rs           # Commands enum - already has Layers variant
├── commands/
│   ├── mod.rs           # Command module exports and routing
│   ├── layers.rs        # THIS FILE - Layers command implementation
│   ├── context.rs       # Similar simple display command (pattern reference)
│   ├── log.rs           # Complex layer-aware command (pattern reference)
│   └── list.rs          # Listing command with formatting (pattern reference)
├── core/
│   ├── config.rs        # ProjectContext struct
│   ├── layer.rs         # Layer enum definition
│   ├── error.rs         # JinError enum (NotInitialized variant)
│   └── mod.rs           # Core module exports
└── git/
    ├── repo.rs          # JinRepo::open_or_create()
    └── mod.rs           # Git module exports

tests/
├── cli_basic.rs         # Basic CLI integration tests
└── common/
    ├── fixtures.rs      # Test helper functions
    └── mod.rs           # Common test utilities
```

### Desired Codebase Tree (No Changes Needed)

The command is already implemented. This PRP documents the existing implementation for maintenance and reference.

```bash
# No changes required - implementation complete
# This PRP serves as documentation of the implementation
```

### Known Gotchas of Codebase & Library Quirks

```rust
// CRITICAL: Context loading requires specific error handling pattern
// The NotInitialized error must be returned early, other errors use default context
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => {
        return Err(JinError::NotInitialized);  // MUST return early
    }
    Err(_) => ProjectContext::default(),  // Other errors use default
};

// CRITICAL: Layer filtering must check both mode and scope requirements
// A layer may require mode, scope, both, or neither
if layer.requires_mode() && context.mode.is_none() {
    continue;  // Skip this layer
}
if layer.requires_scope() && context.scope.is_none() {
    continue;  // Skip this layer
}

// CRITICAL: ref_path() and storage_path() require context parameters
// They are not static methods - need mode/scope/project passed in
let ref_path = layer.ref_path(
    context.mode.as_deref(),
    context.scope.as_deref(),
    context.project.as_deref(),
);

// GOTCHA: Git reference finding may fail
// Use is_ok() check rather than unwrap() to determine if layer has commits
let has_commits = git_repo.find_reference(&ref_path).is_ok();

// GOTCHA: Tree walking requires error handling in callback
// The callback returns TreeWalkResult, not Result
tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
    if entry.kind() == Some(git2::ObjectType::Blob) {
        count += 1;
    }
    git2::TreeWalkResult::Ok  // Not Result::Ok
})?;

// GOTCHA: println! format strings require proper spacing alignment
// Use {:<20} for left-align width 20, {:>2} for right-align width 2
println!(
    "  {} {:2}. {:<20} [{}]{}",
    status,           // Single character (✓ or space)
    layer.precedence(), // Right-align 2 chars
    layer.to_string(), // Left-align 20 chars
    storage_path,     // Dynamic width
    file_count_str    // Optional suffix
);
```

## Implementation Blueprint

### Data Models and Structure

The layers command uses existing types - no new models required:

```rust
// From src/core/layer.rs - Already defined
pub enum Layer {
    GlobalBase,           // Layer 1
    ModeBase,             // Layer 2
    ModeScope,            // Layer 3
    ModeScopeProject,     // Layer 4
    ModeProject,          // Layer 5
    ScopeBase,            // Layer 6
    ProjectBase,          // Layer 7
    UserLocal,            // Layer 8
    WorkspaceActive,      // Layer 9
}

impl Layer {
    pub fn all_in_precedence_order() -> Vec<Layer>;
    pub fn precedence(&self) -> u8;
    pub fn ref_path(&self, mode: Option<&str>, scope: Option<&str>, project: Option<&str>) -> String;
    pub fn storage_path(&self, mode: Option<&str>, scope: Option<&str>, project: Option<&str>) -> String;
    pub fn requires_mode(&self) -> bool;
    pub fn requires_scope(&self) -> bool;
}

// From src/core/config.rs - Already defined
pub struct ProjectContext {
    pub version: u32,
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}

impl ProjectContext {
    pub fn load() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
# Task 1: Verify CLI Registration (ALREADY DONE)
# File: src/cli/mod.rs
# - Ensure Layers variant exists in Commands enum
# - No arguments needed for this command

# Task 2: Verify Command Routing (ALREADY DONE)
# File: src/commands/mod.rs
# - Ensure layers::execute() is called when Commands::Layers matches

Task 3: IMPLEMENT src/commands/layers.rs - execute() function
  - LOAD: ProjectContext with proper error handling
  - PATTERN: Match on load result, early return on NotInitialized
  - OPEN: JinRepo using JinRepo::open_or_create()
  - DISPLAY: Header with context info
  - DISPLAY: Layer list with status indicators
  - CALCULATE: Summary statistics
  - RETURN: Ok(())

Task 4: IMPLEMENT src/commands/layers.rs - count_files_in_layer() helper
  - FIND: Git reference at ref_path
  - PEEL: Reference to commit, then to tree
  - WALK: Tree in PreOrder mode
  - COUNT: Only Blob objects (files, not trees)
  - RETURN: File count as usize

Task 5: CREATE src/commands/layers.rs - unit tests
  - TEST: execute() with default context (no mode/scope)
  - TEST: execute() with mode and scope set
  - TEST: execute() when not initialized (returns NotInitialized error)
  - TEST: count_files_in_layer() with empty layer (0 files)
  - FIXTURE: setup_test_env() for isolated test environment
  - PATTERN: Use tempfile crate for test isolation
```

### Implementation Patterns & Key Details

```rust
// MAIN FUNCTION PATTERN
pub fn execute() -> Result<()> {
    // PATTERN: Load context with specific error handling
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // PATTERN: Open repository
    let repo = JinRepo::open_or_create()?;
    let git_repo = repo.inner();

    // PATTERN: Display header with context
    println!("Layer composition for current context:");
    if context.mode.is_some() || context.scope.is_some() || context.project.is_some() {
        if let Some(mode) = &context.mode {
            println!("  Mode:    {}", mode);
        }
        if let Some(scope) = &context.scope {
            println!("  Scope:   {}", scope);
        }
        if let Some(project) = &context.project {
            println!("  Project: {}", project);
        }
    } else {
        println!("  (no active mode/scope/project)");
    }
    println!();

    // PATTERN: Display layers in precedence order
    println!("Merge order (lowest to highest precedence):");

    let all_layers = Layer::all_in_precedence_order();
    let mut active_count = 0;
    let mut total_files = 0;

    for layer in &all_layers {
        // GOTCHA: Filter layers that require mode/scope when not active
        if layer.requires_mode() && context.mode.is_none() {
            continue;
        }
        if layer.requires_scope() && context.scope.is_none() {
            continue;
        }

        // PATTERN: Get paths for this layer
        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // PATTERN: Check if layer has commits and count files
        let has_commits = git_repo.find_reference(&ref_path).is_ok();
        let file_count = if has_commits {
            active_count += 1;
            count_files_in_layer(git_repo, &ref_path).unwrap_or(0)
        } else {
            0
        };
        total_files += file_count;

        // PATTERN: Format storage path
        let storage_path = layer.storage_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // PATTERN: Display layer with aligned formatting
        let status = if has_commits { "✓" } else { " " };
        println!(
            "  {} {:2}. {:<20} [{}]{}",
            status,
            layer.precedence(),
            layer.to_string(),
            storage_path,
            if file_count > 0 {
                format!(" ({} files)", file_count)
            } else {
                String::new()
            }
        );
    }

    // PATTERN: Display summary
    println!();
    println!(
        "Active layers: {} of {} layers have files",
        active_count,
        all_layers.len()
    );
    println!("Total files in workspace: {}", total_files);

    Ok(())
}

// HELPER FUNCTION PATTERN
fn count_files_in_layer(repo: &git2::Repository, ref_path: &str) -> Result<usize> {
    // PATTERN: Navigate Git object graph
    let reference = repo.find_reference(ref_path)?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    // GOTCHA: Tree walk callback returns TreeWalkResult, not Result
    let mut count = 0;
    tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            count += 1;
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(count)
}

// TEST PATTERN
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp = TempDir::new().unwrap();

        // CRITICAL: Isolate test environment
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);
        std::env::set_current_dir(temp.path()).unwrap();

        // Initialize minimal context
        std::fs::create_dir(".jin").unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        temp
    }

    #[test]
    fn test_execute_default_context() {
        let _temp = setup_test_env();
        let result = execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Don't initialize .jin
        let result = execute();
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }
}
```

### Integration Points

```yaml
CLI_ENUM:
  - location: src/cli/mod.rs
  - pattern: |
      #[derive(Subcommand, Debug)]
      pub enum Commands {
          /// Show current layer composition
          Layers,
          // ... other commands
      }

COMMAND_ROUTING:
  - location: src/commands/mod.rs
  - pattern: |
      match cli.command {
          // ... other commands
          Commands::Layers => layers::execute(),
          // ... other commands
      }

LAYER_ENUM:
  - location: src/core/layer.rs
  - methods: all_in_precedence_order(), precedence(), ref_path(), storage_path(), requires_mode(), requires_scope()

CONTEXT_STRUCT:
  - location: src/core/config.rs
  - methods: ProjectContext::load(), context.mode, context.scope, context.project

GIT_OPERATIONS:
  - location: src/git/repo.rs
  - methods: JinRepo::open_or_create(), repo.inner()
  - git2 methods: find_reference(), peel_to_commit(), tree(), walk()
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating/modifying layers.rs
cargo check --bin jin                # Check for compilation errors
cargo clippy --bin jin -- -D warnings  # Lint checking

# Format check
cargo fmt --check                    # Verify formatting
cargo fmt                            # Auto-format if needed

# Expected: Zero errors, zero warnings. If errors exist, READ output and fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run layers command unit tests
cargo test layers                    # Run all layers tests
cargo test layers::tests::test_execute_default_context  # Run specific test
cargo test layers::tests::test_execute_not_initialized  # Test error case

# Run all command tests
cargo test --bin jin commands

# Expected: All tests pass. If failing, debug root cause and fix.
```

### Level 3: Integration Testing (System Validation)

```bash
# Initialize a test project
mkdir /tmp/jin-layers-test && cd /tmp/jin-layers-test
jin init

# Test basic output with no context
jin layers
# Expected: Shows all 9 layers, none active, 0 files

# Test with mode set
jin mode set claude
jin layers
# Expected: ModeBase and related layers shown, filtering applied

# Test with scope set
jin scope set language:javascript
jin layers
# Expected: ModeScope and ScopeBase layers shown

# Test with files added
echo "test" > test.txt
jin add test.txt
jin commit -m "Add test file"
jin layers
# Expected: At least one layer shows with ✓ and file count > 0

# Test not initialized error
cd /tmp
mkdir not-a-jin-project && cd not-a-jin-project
jin layers
# Expected: Error message "Not a jin project"

# Expected: All scenarios work correctly, proper output formatting, appropriate errors.
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test with all layers active (complex scenario)
# This would require a comprehensive test setup

# Visual validation of output alignment
jin layers | cat -A
# Check that spacing and alignment look correct

# Test edge cases
# - Very long mode/scope names
# - Very long file paths
# - Unicode characters in paths
jin mode set "very-long-mode-name-with-lots-of-characters"
jin scope set "scope:with:many:colons:and:segments"
jin layers

# Performance test with many files
# Create a project with 1000+ files across layers
for i in {1..1000}; do echo "content" > "file$i.txt"; done
jin add .
jin commit -m "Add many files"
jin layers
# Should complete quickly (< 1 second)

# Integration with other commands
# Verify layers output matches what other commands report
jin add newfile.txt
jin status  # Should show newfile.txt as staged
jin layers  # Should show file counts before commit
jin commit -m "Add newfile"
jin layers  # Should show updated file counts

# Expected: All edge cases handled gracefully, performance acceptable, integration consistent.
```

## Final Validation Checklist

### Technical Validation

- [ ] Command compiles without errors: `cargo check --bin jin`
- [ ] No clippy warnings: `cargo clippy --bin jin -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`
- [ ] All unit tests pass: `cargo test layers`
- [ ] No new dependencies added (uses existing)

### Feature Validation

- [ ] Shows current context when mode/scope/project set
- [ ] Filters layers correctly based on active mode/scope
- [ ] Displays all 9 layers in correct precedence order
- [ ] Status indicator (✓) shows for layers with commits
- [ ] File counts accurate when non-zero
- [ ] Storage paths display correctly with context substitution
- [ ] Summary statistics match actual counts
- [ ] Returns NotInitialized error when appropriate
- [ ] Works with default context (no mode/scope/project)
- [ ] Works with complex mode/scope/project combinations

### Code Quality Validation

- [ ] Follows existing command patterns (context, log, list)
- [ ] Error handling consistent with other commands
- [ ] No unwrap() calls that could panic
- [ ] Uses JinError::NotInitialized correctly
- [ ] Unit tests cover success and error paths
- [ ] Helper function (count_files_in_layer) tested
- [ ] Test isolation using tempfile

### Documentation & Deployment

- [ ] Function has doc comment: `/// Execute the layers command`
- [ ] Helper function has doc comment
- [ ] Tests have descriptive names
- [ ] Code is self-documenting with clear variable names

---

## Anti-Patterns to Avoid

- **Don't** use unwrap() on Git operations - use ? propagation
- **Don't** skip layer filtering based on mode/scope context
- **Don't** forget to handle NotInitialized error specifically
- **Don't** hardcode layer names or precedence values
- **Don't** use synchronous file operations in async contexts (not applicable here)
- **Don't** create new patterns when existing commands show the way
- **Don't** add table formatting dependencies without consensus
- **Don't** ignore the TreeWalkResult return type in tree walk callback
- **Don't** forget to update summary statistics when adding new output
- **Don't** assume all layers have commits - check with is_ok()

---

## Confidence Score

**8/10** - High confidence for one-pass implementation success

**Reasoning**:
- (+) Command is already fully implemented and tested
- (+) All context and patterns are well-documented in this PRP
- (+) Follows established codebase patterns consistently
- (+) No external dependencies or new patterns required
- (+) Validation procedures are specific and executable
- (-) Implementation is complete, so this PRP serves as documentation
- (-) Potential future enhancements could add complexity

**Validation**: The completed PRP enables an AI agent (or human developer) to understand, maintain, and enhance the `jin layers` command implementation using only this PRP content and codebase access.
