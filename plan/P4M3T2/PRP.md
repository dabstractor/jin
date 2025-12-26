name: "P4.M3.T2: Scope Commands - Product Requirement Prompt"
description: |

---

## Goal

**Feature Goal**: Implement scope create/use/list/delete/show/unset CLI subcommands to manage language- and domain-specific configuration layers in the Jin multi-layer system.

**Deliverable**: Complete `src/commands/scope.rs` module with comprehensive test coverage, wired to CLI via `src/main.rs` and exported via `src/commands/mod.rs`.

**Success Definition**:
- All 6 scope commands (create, use, unset, delete, show, list) work correctly
- Commands follow the exact patterns established by mode commands (P4.M3.T1)
- Full test coverage matching mode command test patterns
- All validation passes: `cargo test`, `cargo clippy`, `cargo fmt --check`
- Scope Git refs are created/deleted correctly at `refs/jin/layers/scope/*` and `refs/jin/layers/mode/*/scope/*`

## User Persona

**Target User**: Developers using Jin for multi-layer AI/editor configuration management who need to organize configurations by programming language or domain.

**Use Case**: A developer works on a polyglot codebase and wants to maintain different AI assistant configurations for JavaScript vs. Rust code, or different coding standards for frontend vs. backend.

**User Journey**:
1. User creates a scope: `jin scope create language:javascript --mode=claude`
2. User activates the scope: `jin scope use language:javascript`
3. User adds language-specific files: `jin add .claude/js_commands.md --mode --scope=language:javascript`
4. User commits staged changes: `jin commit`
5. User deactivates when done: `jin scope unset`
6. User can list all available scopes: `jin scopes`

**Pain Points Addressed**:
- Managing language-specific AI prompts without proper scoping
- Sharing configurations across modes while maintaining separation
- Organizing domain-specific settings (e.g., `infra:docker`, `framework:react`)

## Why

- **Business value**: Enables fine-grained configuration management at the language/domain level, critical for polyglot development environments
- **Integration**: Scopes are core to Jin's 9-layer hierarchy (Layer 3: ModeScope, Layer 4: ModeScopeProject, Layer 6: ScopeBase)
- **Problems solved**:
  - Allows organizing AI/editor configs by language or domain context
  - Enables sharing configurations across modes (untethered scopes) or binding to specific modes
  - Provides precedence rules: mode-bound scope > untethered scope > mode base

## What

Implement six CLI subcommands that manage scope layers in Jin's configuration system:

### Commands to Implement

1. **`jin scope create <name> [--mode=<mode>]`** - Create a new scope
   - Creates Git ref at `refs/jin/layers/scope/<name>` (untethered) or `refs/jin/layers/mode/<mode>/scope/<name>` (mode-bound)
   - Validates scope name format (alphanumeric, hyphens, underscores, colons)
   - Fails if scope already exists
   - Creates empty initial commit for the scope

2. **`jin scope use <name>`** - Activate a scope
   - Updates `.jin/context` with active scope
   - Validates scope exists in Git refs
   - Preserves existing mode if set
   - Only one active scope at a time (overwrites previous)

3. **`jin scope unset`** - Deactivate current scope
   - Removes scope field from `.jin/context`
   - Preserves mode field if present
   - Idempotent (no error if no scope active)

4. **`jin scopes`** - List all available scopes
   - Lists both mode-bound and untethered scopes
   - Shows scope type (mode-bound or untethered)
   - Displays commit OID for each scope

5. **`jin scope delete <name>`** - Delete a scope
   - Deletes Git ref for the scope
   - Fails if scope is currently active
   - Fails if scope doesn't exist
   - Works for both mode-bound and untethered scopes

6. **`jin scope show`** - Display active scope
   - Shows current active scope from `.jin/context`
   - Indicates if no scope is active
   - Shows mode if also active

### Success Criteria

- [ ] `jin scope create language:javascript --mode=claude` creates mode-bound scope ref
- [ ] `jin scope create infra:docker` creates untethered scope ref (no --mode flag)
- [ ] `jin scope use language:javascript` sets active scope in context
- [ ] `jin scope unset` clears scope from context
- [ ] `jin scopes` lists all scopes with type indicators
- [ ] `jin scope delete language:javascript` removes scope ref (fails if active)
- [ ] `jin scope show` displays current active scope
- [ ] All commands follow mode command patterns exactly
- [ ] Comprehensive tests pass (matching mode command test coverage)

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**YES** - This PRP provides:
- Exact file paths and line numbers for all patterns to follow
- Complete mode command implementation as reference
- Full layer system and scope type definitions
- Git operations patterns for ref management
- Test patterns and validation commands
- All CLI wiring requirements

### Documentation & References

```yaml
# MUST READ - Critical implementation references

# Primary Pattern Reference - Follow this EXACTLY
- file: src/commands/mode.rs
  why: Complete implementation pattern for all scope commands (create/use/unset/delete/show/list)
  pattern: Command dispatcher, Git ref operations, context management, validation logic, test structure
  gotcha: Scope commands have additional complexity with mode-bound vs untethered scopes and listing both types

# Layer Type Definitions - Scope variants
- file: src/core/layer.rs
  why: Understanding Scope layer variants for Git ref paths and routing
  section: Lines 50-80 (ModeScope, ModeScopeProject, ScopeBase variants)
  gotcha: Scopes can be mode-bound (refs/jin/layers/mode/<mode>/scope/<scope>) or untethered (refs/jin/layers/scope/<scope>)

# Configuration Management - Context operations
- file: src/core/config.rs
  why: ProjectContext has `scope` field with set_scope() and has_scope() methods
  section: Lines 80-120 (ProjectContext struct and methods)
  gotcha: Use context.set_scope(Some(name)) and context.save() for persistence

# Error Types - Add ScopeNotFound
- file: src/core/error.rs
  why: Add ScopeNotFound error variant matching ModeNotFound pattern
  section: Lines 40-80 (JinError enum definition)
  gotcha: Must update exit_code() method to map ScopeNotFound to exit code 3

# CLI Arguments - ScopeCommand enum
- file: src/cli/args.rs
  why: ScopeCommand enum already defined with Create/Use/Unset/Delete/Show variants
  section: Lines 168-200 (ScopeCommand enum)
  gotcha: Create variant has `mode: Option<String>` for mode-binding

# Git Operations - Ref management
- file: src/git/repo.rs
  why: Methods for creating, finding, listing, and deleting Git references
  section: Lines 100-180 (reference management methods)
  gotcha: Use list_layer_refs_by_pattern() for listing scopes with wildcards

# Command Module Exports
- file: src/commands/mod.rs
  why: Add scope module exports following mode pattern
  section: Lines 1-16 (module declarations and exports)
  gotcha: Must export both execute() and execute_list() functions

# Main CLI Routing
- file: src/main.rs
  why: Wire scope commands to CLI handlers
  section: Lines 50-150 (command match arm routing)
  gotcha: Scope commands need individual match arms like Mode commands, not grouped

# PRD Specifications - Full requirements
- docfile: plan/docs/PRD.md
  why: Section 13.2 contains complete scope command specifications and examples
  section: Section 13.2: Scope
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                    # Project dependencies (clap, git2, serde, tempfile, thiserror)
├── plan/
│   └── docs/
│       └── PRD.md                # Section 13.2: Scope specifications
└── src/
    ├── cli/
    │   ├── args.rs               # ScopeCommand enum defined (lines 168-200)
    │   └── mod.rs
    ├── commands/
    │   ├── add.rs                # Layer routing pattern
    │   ├── commit.rs
    │   ├── init.rs
    │   ├── mode.rs               # PRIMARY REFERENCE - exact pattern to follow
    │   ├── mod.rs                # Add scope exports here
    │   └── status.rs             # Context display pattern
    ├── core/
    │   ├── config.rs             # ProjectContext with scope field
    │   ├── error.rs              # Add ScopeNotFound error
    │   ├── layer.rs              # Layer enum with scope variants
    │   └── mod.rs
    ├── git/
    │   ├── mod.rs
    │   └── repo.rs               # Git ref operations
    └── main.rs                   # CLI routing - add scope command handlers
```

### Desired Codebase Tree (files to add)

```bash
# NEW FILE TO CREATE
└── src/
    └── commands/
        └── scope.rs              # CREATE: Complete scope command implementation (500+ lines)
                                  # - execute() dispatcher function
                                  # - execute_list() for `jin scopes`
                                  # - execute_create() with mode binding
                                  # - execute_use() for activation
                                  # - execute_unset() for deactivation
                                  # - execute_delete() with safety checks
                                  # - execute_show() for display
                                  # - is_valid_scope_name() validator
                                  # - Full test suite (20+ tests)

# FILES TO MODIFY
└── src/
    ├── commands/
    │   └── mod.rs                # ADD: pub mod scope; + exports
    ├── core/
    │   └── error.rs              # ADD: ScopeNotFound { scope: String } variant
    └── main.rs                   # ADD: Scope command routing match arms
```

### Known Gotchas of our Codebase & Library Quirks

```rust
// CRITICAL: Scope name validation differs from mode names
// Modes: alphanumeric + hyphens + underscores only
// Scopes: alphanumeric + hyphens + underscores + COLONS (e.g., "language:javascript")
fn is_valid_scope_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
}

// CRITICAL: Scopes can be mode-bound OR untethered - need to track both
// Mode-bound: refs/jin/layers/mode/<mode>/scope/<scope>
// Untethered: refs/jin/layers/scope/<scope>
// When listing, need to query BOTH patterns and merge results

// CRITICAL: git2::Reference doesn't implement Clone - must use reference.target() to get OID
// See mode.rs line 84: let short_oid = &oid.to_string()[..8];

// CRITICAL: Context operations are idempotent - loading non-existent context returns Ok(Default)
// ProjectContext::load() never fails for missing files, only for parse errors

// CRITICAL: Test isolation - each test must use TempDir and set current directory
// See mode.rs lines 293-310 for DirGuard pattern

// CRITICAL: Local test config isolation - create .jin/config.yaml pointing to current directory
// This prevents tests from polluting the global ~/.jin repository
// See mode.rs lines 328-338

// GOTCHA: clap derive macros - use #[command(subcommand)] for enum variants
// ScopeCommand is already defined in args.rs - no need to modify

// GOTCHA: ExitCode - use ExitCode::SUCCESS and ExitCode::FAILURE from std::process::ExitCode
// main.rs already has error handling - just return Result<()>

// GOTCHA: String matching - use context.scope.as_ref() == Some(&name.to_string())
// Can't compare &String directly with &str without conversion
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - scope operations use existing types:
- `ScopeCommand` enum (already defined in `src/cli/args.rs`)
- `ProjectContext` with `scope: Option<String>` field
- `Layer` enum variants: `ModeScope`, `ModeScopeProject`, `ScopeBase`
- `JinError` enum (add `ScopeNotFound` variant)

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD ScopeNotFound error variant to src/core/error.rs
  IMPLEMENT: Add JinError::ScopeNotFound { scope: String } variant
  FOLLOW pattern: JinError::ModeNotFound { mode: String } (line ~50)
  UPDATE exit_code() method: Map ScopeNotFound to exit code 3
  NAMING: ScopeNotFound with scope field (not name)
  PLACEMENT: After ModeNotFound variant in JinError enum

Task 2: CREATE src/commands/scope.rs with module documentation
  IMPLEMENT: Complete scope command implementation (~500 lines)
  FOLLOW pattern: src/commands/mode.rs (exact structure and patterns)
  NAMING: Module name "scope", file name "scope.rs"
  FUNCTIONS:
    - execute(cmd: &ScopeCommand) -> Result<()> (dispatcher)
    - execute_list() -> Result<()> (jin scopes command)
    - execute_create(name: &str, mode: Option<&str>) -> Result<()>
    - execute_use(workspace_root: &Path, name: &str) -> Result<()>
    - execute_unset(workspace_root: &Path) -> Result<()>
    - execute_delete(workspace_root: &Path, name: &str) -> Result<()>
    - execute_show(workspace_root: &Path) -> Result<()>
    - is_valid_scope_name(name: &str) -> bool
  DEPENDENCIES: Import from Task 1 (ScopeNotFound error)
  PLACEMENT: src/commands/scope.rs (new file)

Task 3: IMPLEMENT execute_create with mode binding logic
  IMPLEMENT: Create scope with optional mode binding
  VALIDATE: Scope name format (allow colons, e.g., "language:javascript")
  CHECK: Scope doesn't already exist (both mode-bound and untethered)
  GIT_OPS:
    - If mode specified: ref_path = "refs/jin/layers/mode/{mode}/scope/{name}"
    - If no mode: ref_path = "refs/jin/layers/scope/{name}"
    - Create empty tree → initial commit → reference
  FOLLOW pattern: src/commands/mode.rs execute_create() (lines 101-151)
  GOTCHA: Must validate referenced mode exists when --mode flag is used

Task 4: IMPLEMENT execute_use for scope activation
  IMPLEMENT: Set active scope in ProjectContext
  VALIDATE: Scope exists (check both mode-bound and untethered locations)
  CONTEXT_OPS:
    - Load context from workspace_root
    - Set scope field: context.set_scope(Some(name.to_string()))
    - Save context to disk
  PRESERVE: Existing mode field in context
  FOLLOW pattern: src/commands/mode.rs execute_use() (lines 163-182)
  GOTCHA: Scope may exist in multiple locations - prefer mode-bound if current mode active

Task 5: IMPLEMENT execute_unset for scope deactivation
  IMPLEMENT: Clear scope field from ProjectContext
  CONTEXT_OPS: context.set_scope(None) → context.save()
  PRESERVE: Mode field if present
  IDEMPOTENT: No error if no scope is currently active
  FOLLOW pattern: src/commands/mode.rs execute_unset() (lines 193-200)

Task 6: IMPLEMENT execute_delete with safety checks
  IMPLEMENT: Delete Git ref for scope
  VALIDATE: Scope exists (check both locations)
  SAFETY_CHECK: Scope is not currently active in context
  GIT_OPS: repo.find_reference() → reference.delete()
  FOLLOW pattern: src/commands/mode.rs execute_delete() (lines 211-237)
  GOTCHA: Must check context.scope == Some(name) before deletion

Task 7: IMPLEMENT execute_show for active scope display
  IMPLEMENT: Display current active scope from context
  CONTEXT_OPS: Load context and read scope field
  OUTPUT: "Active scope: {name}" or "No active scope"
  BONUS: Show mode if also active (like mode show)
  FOLLOW pattern: src/commands/mode.rs execute_show() (lines 247-266)

Task 8: IMPLEMENT execute_list for listing all scopes
  IMPLEMENT: List all mode-bound and untethered scopes
  GIT_OPS:
    - List refs matching "refs/jin/layers/scope/*"
    - List refs matching "refs/jin/layers/mode/*/scope/*"
    - Merge and deduplicate results
  OUTPUT_FORMAT: "{name} [{type}] {short_oid}"
    - Mode-bound: "language:javascript [mode:claude] a1b2c3d4"
    - Untethered: "infra:docker [untethered] e5f6g7h8"
  FOLLOW pattern: src/commands/mode.rs execute_list() (lines 62-91)
  GOTCHA: Must extract mode name from path for mode-bound scopes

Task 9: ADD comprehensive test suite to src/commands/scope.rs
  IMPLEMENT: Full test coverage matching mode command tests
  TEST_CATEGORIES:
    - Creation tests (success, duplicate exists, validation)
    - Activation tests (success, not found, preserves mode)
    - Deactivation tests (success, idempotent, preserves mode)
    - Deletion tests (success, not found, active protection)
    - Display tests (show active, show none, show with mode)
    - List tests (empty list, multiple scopes, type indicators)
    - Name validation tests (valid, invalid, edge cases)
    - Command routing tests (all variants)
  FOLLOW pattern: src/commands/mode.rs tests module (lines 287-730)
  HELPERS: DirGuard, init_git_repo(), init_jin()
  PLACEMENT: #[cfg(test)] mod tests { ... } at end of scope.rs

Task 10: UPDATE src/commands/mod.rs with scope exports
  ADD: pub mod scope;
  ADD: pub use scope::execute as scope_execute;
  ADD: pub use scope::execute_list as scope_list_execute;
  FOLLOW pattern: Existing mode exports (lines 7, 14-15)
  PLACEMENT: After mode module declaration and exports

Task 11: WIRE scope commands in src/main.rs
  ADD: Match arms for all ScopeCommand variants
  ROUTING:
    - Commands::Scope(ScopeCommand::Create { name, mode }) → scope_execute()
    - Commands::Scope(ScopeCommand::Use { name }) → scope_execute()
    - Commands::Scope(ScopeCommand::Unset) → scope_execute()
    - Commands::Scope(ScopeCommand::Delete { name }) → scope_execute()
    - Commands::Scope(ScopeCommand::Show) → scope_execute()
    - Commands::Scopes → scope_list_execute()
  FOLLOW pattern: Mode command routing (lines ~70-120)
  ERROR_HANDLING: Match Ok(()) → ExitCode::SUCCESS, Err(e) → eprintln + ExitCode::FAILURE
```

### Implementation Patterns & Key Details

```rust
// ===== SCOPE NAME VALIDATION (allowing colons) =====
fn is_valid_scope_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
}

// ===== SCOPE CREATION WITH MODE BINDING =====
fn execute_create(name: &str, mode: Option<&str>) -> Result<()> {
    // 1. Validate scope name format
    if !is_valid_scope_name(name) {
        return Err(JinError::ValidationError {
            message: format!(
                "Invalid scope name: '{}'. Use alphanumeric, hyphens, underscores, colons.",
                name
            ),
        });
    }

    // 2. Load config and repo
    let config = JinConfig::load()?;
    let repo = JinRepo::open_or_create(&config.repository)?;

    // 3. Determine ref path based on mode binding
    let (ref_path, scope_type) = if let Some(mode_name) = mode {
        // Validate mode exists when binding
        let mode_ref = format!("refs/jin/layers/mode/{}", mode_name);
        if repo.find_reference(&mode_ref).is_err() {
            return Err(JinError::ModeNotFound {
                mode: mode_name.to_string(),
            });
        }
        (format!("refs/jin/layers/mode/{}/scope/{}", mode_name, name), "mode-bound")
    } else {
        (format!("refs/jin/layers/scope/{}", name), "untethered")
    };

    // 4. Check scope doesn't exist
    if repo.find_reference(&ref_path).is_ok() {
        return Err(JinError::RefExists {
            name: ref_path.clone(),
            layer: format!("scope/{}", name),
        });
    }

    // 5. Create empty tree and initial commit
    let empty_tree_oid = repo.create_empty_tree()?;
    let empty_tree = repo.find_tree(empty_tree_oid)?;
    let author = repo.signature("Jin", "jin@local")?;
    let initial_commit_oid = repo.create_commit(
        None,
        &author,
        &author,
        &format!("Initial commit for scope: {}", name),
        &empty_tree,
        &[],
    )?;

    // 6. Create the scope ref
    repo.create_reference(
        &ref_path,
        initial_commit_oid,
        false,
        &format!("Create scope: {}", name),
    )?;

    println!("Scope '{}' created [{}].", name, scope_type);
    Ok(())
}

// ===== SCOPE USE WITH EXISTENCE CHECK =====
fn execute_use(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check both mode-bound and untethered locations
    let untethered_ref = format!("refs/jin/layers/scope/{}", name);

    // Check if scope exists in either location
    let scope_exists = repo.find_reference(&untethered_ref).is_ok();

    // TODO: Also check mode-bound locations based on current active mode
    // For now, just check untethered

    if !scope_exists {
        return Err(JinError::ScopeNotFound {
            scope: name.to_string(),
        });
    }

    // Update context
    let mut context = ProjectContext::load(workspace_root)?;
    context.set_scope(Some(name.to_string()));
    context.save(workspace_root)?;

    println!("Scope '{}' is now active.", name);
    Ok(())
}

// ===== SCOPE LIST WITH TYPE INDICATORS =====
pub fn execute_list() -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // List untethered scopes
    let untethered_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/scope/*")?;

    // List mode-bound scopes
    let mode_bound_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*/scope/*")?;

    if untethered_refs.is_empty() && mode_bound_refs.is_empty() {
        println!("No scopes found.");
        return Ok(());
    }

    println!("Available scopes:");

    // Display untethered scopes
    for ref_name in untethered_refs {
        let scope_name = ref_name.strip_prefix("refs/jin/layers/scope/").unwrap_or(&ref_name);
        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:30} [untethered] {}", scope_name, short_oid);
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
            if let Ok(reference) = repo.find_reference(&ref_name) {
                if let Some(oid) = reference.target() {
                    let short_oid = &oid.to_string()[..8];
                    println!("  {:30} [mode:{}] {}", scope_name, mode_name, short_oid);
                }
            }
        }
    }

    Ok(())
}

// ===== TEST HELPER PATTERN =====
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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

    fn init_git_repo(dir: &Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    fn init_jin(dir: &Path) {
        let context = ProjectContext::default();
        context.save(dir).unwrap();

        let local_config = JinConfig {
            version: 1,
            repository: PathBuf::from("."),
            default_mode: None,
            default_scope: None,
        };
        let config_path = dir.join(".jin").join("config.yaml");
        let yaml_content = serde_yaml_ng::to_string(&local_config).unwrap();
        std::fs::write(config_path, yaml_content).unwrap();
    }
    // ... individual tests
}
```

### Integration Points

```yaml
ERROR_TYPES:
  add_to: src/core/error.rs
  variant: |
    #[error("Scope not found: {scope}")]
    ScopeNotFound { scope: String }
  update_exit_code: Add to exit_code() method -> 3

COMMAND_MODULE:
  add_to: src/commands/mod.rs
  additions: |
    pub mod scope;
    pub use scope::execute as scope_execute;
    pub use scope::execute_list as scope_list_execute;

CLI_ROUTING:
  add_to: src/main.rs
  pattern: |
    Commands::Scope(ScopeCommand::Create { name, mode }) => {
        match commands::scope_execute(&ScopeCommand::Create { name, mode }) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        }
    }
    Commands::Scope(ScopeCommand::Use { name }) => { ... }
    Commands::Scope(ScopeCommand::Unset) => { ... }
    Commands::Scope(ScopeCommand::Delete { name }) => { ... }
    Commands::Scope(ScopeCommand::Show) => { ... }
    Commands::Scopes => {
        match commands::scope_list_execute() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        }
    }

CONTEXT_INTEGRATION:
  existing_methods:
    - ProjectContext::load(workspace_root)
    - context.set_scope(Some(name))
    - context.set_scope(None)
    - context.has_scope()
    - context.save(workspace_root)

GIT_OPERATIONS:
  methods_from_repo:
    - JinRepo::open_or_create(&config.repository)
    - repo.find_reference(&ref_path)
    - repo.create_reference(&ref_path, oid, false, &msg)
    - repo.list_layer_refs_by_pattern("refs/jin/layers/scope/*")
    - repo.create_empty_tree()
    - repo.create_commit(None, &author, &committer, &msg, &tree, &[])
    - reference.delete() (via mut reference)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo fmt --                      # Auto-format all code
cargo clippy --all-targets -- -D warnings  # Lint with warnings as errors

# Check specific new file
cargo fmt -- src/commands/scope.rs
cargo clippy --bin jin 2>&1 | grep scope

# Expected: Zero formatting issues, zero clippy warnings
# Common issues to fix:
# - Unused imports (remove them)
# - Unnecessary .clone() calls (use references)
# - Missing error variants (add to error.rs)
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test scope module specifically
cargo test scopes --lib  # All scope-related tests
cargo test scope::tests::test_execute_create -- --exact  # Single test
cargo test scope::tests::test_execute -- --nocapture  # See println output

# Run all command tests
cargo test commands::  # Test all command modules

# Run with output for debugging
cargo test scopes -- --show-output

# Coverage (if cargo-llvm-cov is installed)
cargo llvm-cov --lib --coverage --lcov --output-path lcov.info

# Expected: All tests pass. Look for:
# - test_execute_create_creates_scope_ref
# - test_execute_create_fails_if_scope_exists
# - test_execute_create_validates_scope_name
# - test_execute_use_sets_active_scope
# - test_execute_use_fails_if_scope_not_found
# - test_execute_unset_clears_active_scope
# - test_execute_delete_deletes_scope_ref
# - test_execute_delete_fails_if_scope_is_active
# - test_execute_list_with_multiple_scopes
# - test_is_valid_scope_name (including colons)
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the binary
cargo build --release

# Verify binary was created
ls -lh target/release/jin

# Manual integration testing sequence
cd /tmp/test_scope_project && rm -rf .jin
git init

# Initialize Jin
target/release/jin init

# Create untethered scope
target/release/jin scope create language:javascript
# Expected: "Scope 'language:javascript' created [untethered]."

# Create mode-bound scope (mode must exist first)
target/release/jin mode create claude
target/release/jin scope create language:rust --mode=claude
# Expected: "Scope 'language:rust' created [mode-bound]."

# List scopes
target/release/jin scopes
# Expected output:
#   Available scopes:
#     language:javascript          [untethered] a1b2c3d4
#     language:rust                [mode:claude] e5f6g7h8

# Use a scope
target/release/jin scope use language:javascript
# Expected: "Scope 'language:javascript' is now active."

# Show active scope
target/release/jin scope show
# Expected: "Active scope: language:javascript"

# Add file to scope
echo "// JS config" > .claude/js.md
target/release/jin add .claude/js.md --scope=language:javascript

# Commit changes
target/release/jin commit

# Unset scope
target/release/jin scope unset
# Expected: "Scope deactivated."

# Try to delete active scope (should fail)
target/release/jin scope use language:javascript
target/release/jin scope delete language:javascript
# Expected: Error "Cannot delete active scope. Use 'jin scope unset' first."

# Unset and delete (should succeed)
target/release/jin scope unset
target/release/jin scope delete language:javascript
# Expected: "Scope 'language:javascript' deleted."

# Verify deletion
target/release/jin scopes
# Expected: Only language:rust remains

# Test validation
target/release/jin scope create "invalid name!"
# Expected: Error "Invalid scope name..."

# Test non-existent scope
target/release/jin scope use nonexistent
# Expected: Error "Scope not found: nonexistent"

# Clean up
cd -
rm -rf /tmp/test_scope_project
```

### Level 4: Domain-Specific Validation

```bash
# Scope routing validation - verify scope affects file routing
cd /tmp/test_routing && rm -rf .jin && git init
target/release/jin init

# Create and activate scope
target/release/jin scope create language:python
target/release/jin scope use language:python

# Add file with scope flag
echo "# Python config" > .python/config.md
target/release/jin add .python/config.md --scope=language:python

# Verify staging index contains scope routing
cat .jin/staging-index.yaml | grep -A 5 "language:python"

# Test mode-scope-project routing (3-layer combination)
target/release/jin mode create vscode
target/release/jin scope create backend --mode=vscode
target/release/jin add .vscode/settings.json --mode --scope=backend --project

# Verify layer precedence
target/release/jin status  # Should show active mode and scope

# Test scope name with colons (valid)
target/release/jin scope create "category:subcategory:item"
# Expected: Success

# Test edge cases
target/release/jin scope create ""  # Should fail - empty name
target/release/jin scope create "a"  # Should succeed - single char
target/release/jin scope create "very-long-scope-name-with-hyphens"  # Should succeed

# Performance test with many scopes
for i in {1..100}; do
  target/release/jin scope create "test-scope-$i" > /dev/null
done
target/release/jin scopes | wc -l  # Should list all 100 scopes

# Git ref validation
# Verify refs were created correctly
cd ~/.jin/repo  # or local .jin repo
git show-ref | grep "refs/jin/layers/scope/"
git show-ref | grep "refs/jin/layers/mode/.*/scope/"

# Context file validation
cat /tmp/test_routing/.jin/context
# Expected YAML format with scope field
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --lib`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] All scope command variants work: create, use, unset, delete, show, list
- [ ] ScopeNotFound error variant added to error.rs
- [ ] Exit code mapping correct for ScopeNotFound (exit code 3)

### Feature Validation

- [ ] `jin scope create <name>` creates untethered scope
- [ ] `jin scope create <name> --mode=<mode>` creates mode-bound scope
- [ ] `jin scope use <name>` activates scope in context
- [ ] `jin scope unset` deactivates scope
- [ ] `jin scopes` lists all scopes with type indicators
- [ ] `jin scope delete <name>` removes scope (fails if active)
- [ ] `jin scope show` displays active scope
- [ ] Scope names with colons work (e.g., "language:javascript")
- [ ] Mode validation works when creating mode-bound scopes
- [ ] Context preserves mode when scope changes
- [ ] Test coverage matches mode command tests

### Code Quality Validation

- [ ] Follows mode command patterns exactly
- [ ] File placement correct: src/commands/scope.rs
- [ ] Module exports added to src/commands/mod.rs
- [ ] CLI routing added to src/main.rs
- [ ] Error variant added to src/core/error.rs
- [ ] No unused imports or dead code
- [ ] Comprehensive inline documentation (/// comments)

### Documentation & Deployment

- [ ] Module documentation explains scope commands
- [ ] Function documentation covers parameters, errors, examples
- [ ] User-facing error messages are clear and actionable
- [ ] Test names are descriptive (test_execute_create_creates_scope_ref)
- [ ] Integration test comments explain test scenarios

---

## Anti-Patterns to Avoid

- ❌ Don't duplicate mode command logic - extract shared helpers if needed
- ❌ Don't forget to validate mode existence when creating mode-bound scopes
- ❌ Don't allow deleting the currently active scope (must unset first)
- ❌ Don't use sync functions where async is needed (not applicable here, but good practice)
- ❌ Don't hardcode ref paths - use format! macros with variables
- ❌ Don't catch all exceptions - use specific JinError variants
- ❌ Don't skip writing tests - mode commands have 400+ lines of tests for a reason
- ❌ Don't forget to update both mod.rs AND main.rs for wiring
- ❌ Don't use `name.to_string()` repeatedly - convert once at function start
- ❌ Don't ignore the difference between mode-bound and untethered scopes in list command
