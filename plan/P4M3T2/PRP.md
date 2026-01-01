# PRP: Scope Commands (P4.M3.T2)

---

## Goal

**Feature Goal**: Implement `jin scope` subcommands for managing scope lifecycle (create, use, list, delete, show, unset) as part of the 9-layer configuration hierarchy system. Scopes provide language- or domain-specific configuration deltas that can be applied on top of modes or standalone.

**Deliverable**: A complete CLI command module with all scope operations wired through clap derive API, including unit tests and integration test coverage. Scopes support both mode-bound (tied to a specific mode) and untethered (standalone) configurations.

**Success Definition**:
- All 6 scope subcommands are accessible via `jin scope <subcommand>`
- Scope refs are properly managed in `refs/jin/scopes/` (untethered) and `refs/jin/modes/{mode}/scopes/` (mode-bound) namespaces
- Active scope is persisted in `.jin/context`
- Scope names support colons for hierarchical naming (e.g., `language:javascript`, `infra:docker`)
- Mode-bound scopes validate parent mode existence
- User messages guide users to next steps
- All unit tests pass: `cargo test --package jin --lib commands::scope`
- Integration tests pass for basic scope workflows
- Error handling covers edge cases (invalid names, duplicates, not found, etc.)

## User Persona

**Target User**: Developer using Jin to manage per-scope configuration overlays (e.g., different language configurations: "language:javascript", "language:python", or domain configs: "infra:docker", "infra:kubernetes")

**Use Case**: Developer wants to create isolated configuration layers for specific languages or domains, allowing them to stage files to scope-specific layers and switch between scopes. Scopes can be applied on top of modes for fine-grained control.

**User Journey**:
1. User creates a new untethered scope: `jin scope create language:javascript`
2. User creates a mode-bound scope: `jin scope create infra:docker --mode=production`
3. User activates a scope: `jin scope use language:javascript`
4. User stages files to the scope layer: `jin add .editorconfig --scope=language:javascript`
5. User commits staged changes: `jin commit -m "Add JavaScript config"`
6. User can list all scopes: `jin scope list`
7. User can check active scope: `jin scope show`
8. User can deactivate scope: `jin scope unset`
9. User can delete scope: `jin scope delete language:javascript`

**Pain Points Addressed**:
- Managing language-specific or domain-specific configuration files
- Need for hierarchical naming (e.g., `language:javascript:framework`)
- Combining mode (environment) + scope (language) for layered configurations
- Avoiding git-tracked configuration conflicts
- Isolating scope-specific settings from project defaults

## Why

- **Business value**: Enables developers to maintain configuration contexts for different languages or domains within a single project, reducing context switching overhead
- **Integration with existing features**: Scope commands integrate with the 9-layer hierarchy system (Layer 3: ModeScope, Layer 4: ModeScopeProject, Layer 6: ScopeBase)
- **Problems solved**:
  - Eliminates need for manual configuration file swapping for different languages
  - Provides git-like workflow for scope management
  - Enables layer composition for complex configuration scenarios (mode + scope combinations)
  - Supports hierarchical naming with colons for organized scope structures

## What

**User-visible behavior**:
- `jin scope create <name> [--mode=<mode>]` - Creates a new scope (untethered or mode-bound)
- `jin scope use <name>` - Activates a scope (updates `.jin/context`)
- `jin scope list` - Lists all available scopes with active indicator
- `jin scope delete <name>` - Deletes a scope and its associated refs
- `jin scope show` - Shows currently active scope
- `jin scope unset` - Deactivates current scope

**Technical requirements**:
- Scope names must be alphanumeric + underscores + colons only (colon enables hierarchical naming)
- Reserved names: "default", "global", "base"
- Untethered scopes stored as Git refs at `refs/jin/scopes/<name>` (colons preserved)
- Mode-bound scopes stored as Git refs at `refs/jin/modes/<mode>/scopes/<name>` (colons preserved)
- Active scope persisted in `.jin/context` as YAML
- Mode-bound scopes require parent mode to exist
- Proper error messages for edge cases

### Success Criteria

- [ ] All 6 subcommands accessible and functional
- [ ] Scope validation prevents invalid/reserved names
- [ ] Colon character allowed in scope names for hierarchical naming
- [ ] Mode-bound scopes validate parent mode existence
- [ ] Refs created/updated/deleted correctly in Git storage
- [ ] ProjectContext load/save operations work correctly
- [ ] Integration tests pass for scope workflow scenarios
- [ ] Error handling provides actionable user feedback

## All Needed Context

### Context Completeness Check

**No Prior Knowledge Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**YES**: This PRP provides:
- Exact file paths and patterns to follow
- Complete external dependency documentation references
- Codebase structure with existing implementations to mirror
- Specific validation and error handling patterns
- Test patterns matching existing codebase conventions
- Scope-specific patterns (colons, mode-bound vs untethered)

### Documentation & References

```yaml
# CRITICAL EXTERNAL DOCUMENTATION
- url: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
  why: clap derive API for Parser and Subcommand traits - essential for CLI structure
  critical: Scope commands use #[derive(Subcommand)] for nested subcommands

- url: https://docs.rs/clap/latest/clap/struct.Command.html#method.arg
  why: Command argument configuration reference
  critical: Positional arguments and optional --mode flag for scope create

- url: https://github.com/clap-rs/clap/blob/master/examples/git_derive.rs
  why: Real-world example of nested subcommands similar to jin scope structure
  critical: Shows the pattern for command hierarchies with optional flags

- url: https://docs.rs/git2/latest/git2/
  why: git2 crate documentation for Git operations
  critical: Reference management, object creation, bare repository handling

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.reference
  why: Creating/updating Git references for scope storage
  critical: Scopes stored as refs in refs/jin/scopes/ or refs/jin/modes/{mode}/scopes/

- url: https://docs.rs/git2/latest/git2/struct.Reference.html
  why: Reference operations (find, delete, resolve)
  critical: Scope ref existence checks and deletion

- url: https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html
  why: Test isolation using temporary directories
  critical: Unit test pattern for scope operations

- file: /home/dustin/projects/jin/plan/P4M3T1/research/clap_research.md
  why: Compiled research on clap derive API patterns
  section: Complete code examples for nested subcommands with optional flags

- file: /home/dustin/projects/jin/plan/P4M3T1/research/git2_research.md
  why: Compiled research on git2 crate patterns
  section: Reference management best practices

- file: /home/dustin/projects/jin/plan/P4M3T1/research/tempfile_research.md
  why: Compiled research on test isolation patterns
  section: JIN_DIR environment variable setup

# CODEBASE PATTERNS TO FOLLOW
- file: /home/dustin/projects/jin/src/commands/scope.rs
  why: EXISTING COMPLETE IMPLEMENTATION - reference for validation, exact patterns used
  pattern: validate_scope_name() function shows validation with colon support
  gotcha: Scope names allow colons (unlike modes), reserved names check is critical

- file: /home/dustin/projects/jin/src/commands/mode.rs
  why: Nearly identical pattern for mode commands - use as reference
  pattern: Command execute() function with match on action enum
  gotcha: Mode names don't allow colons (scopes do)

- file: /home/dustin/projects/jin/src/cli/mod.rs
  why: Shows exact ScopeAction enum definition and how Commands enum integrates it
  pattern: #[derive(Subcommand)] enum ScopeAction with variant-specific fields
  gotcha: ScopeAction::Create has optional mode field (#[arg(long)])

- file: /home/dustin/projects/jin/src/commands/mod.rs
  why: Shows command module exports and execute() wiring
  pattern: Commands::Scope(action) => scope::execute(action)
  gotcha: Must add pub mod scope; to module exports

- file: /home/dustin/projects/jin/src/core/config.rs
  why: ProjectContext struct with scope field that persists state
  pattern: context.scope = Some(name.to_string()); context.save()
  gotcha: ProjectContext::load() returns Err(JinError::NotInitialized) if missing

- file: /home/dustin/projects/jin/src/git/repo.rs
  why: JinRepo wrapper for bare repository operations
  pattern: JinRepo::open_or_create() for idempotent initialization
  gotcha: JIN_DIR environment variable overrides default ~/.jin path

- file: /home/dustin/projects/jin/src/git/refs.rs
  why: RefOps trait for reference operations
  pattern: repo.set_ref(path, oid, message), repo.ref_exists(path), repo.delete_ref(path)
  gotcha: All refs under refs/jin/ namespace to avoid conflicts

- file: /home/dustin/projects/jin/src/git/objects.rs
  why: ObjectOps trait for creating blobs, trees, commits
  pattern: repo.create_tree(&[]), repo.create_commit(None, msg, tree, &[])
  gotcha: Empty tree for initial commit is standard pattern

- file: /home/dustin/projects/jin/src/core/error.rs
  why: JinError enum with variants for all error cases
  pattern: JinError::AlreadyExists, NotFound, InvalidLayer, Other
  gotcha: Use descriptive messages for user-facing errors

- file: /home/dustin/projects/jin/src/core/layer.rs
  why: Layer enum showing 9-layer hierarchy and ref path patterns
  pattern: ScopeBase ref path is "refs/jin/layers/scope/{scope}", ModeScope is "refs/jin/layers/mode/{mode}/scope/{scope}"
  gotcha: Scope refs use different path than layer refs

# TEST PATTERNS
- file: /home/dustin/projects/jin/tests/cli_basic.rs
  why: Integration test patterns for CLI commands
  pattern: Use std::process::id() for unique test names
  gotcha: Set JIN_DIR environment variable for test isolation

- file: /home/dustin/projects/jin/tests/mode_scope_workflow.rs
  why: Integration tests for mode/scope workflows
  pattern: create_scope(), jin().args(["scope", "use", name])
  gotcha: Tests use scope list assertions for verification

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Test fixture helpers for repo and context setup
  pattern: setup_test_env() creates isolated test directory
  gotcha: Fixtures handle JIN_DIR and context initialization

- file: /home/dustin/projects/jin/tests/common/assertions.rs
  why: Helper functions for test assertions
  pattern: assert_layer_ref_exists() for ref verification
  gotcha: Use assertion helpers for consistent test patterns

# CONFIGURATION FILES
- file: /home/dustin/projects/jin/Cargo.toml
  why: Project dependencies including clap, git2, tempfile, serde
  section: [dependencies]
  gotcha: Verify clap feature flags include "derive"

- docfile: /home/dustin/projects/jin/src/lib.rs
  why: Library module exports - must include command modules
  section: pub use commands::scope;
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── cli/
│   │   ├── mod.rs          # Cli, Commands, ModeAction, ScopeAction enums
│   │   └── args.rs         # Shared argument structs (AddArgs, CommitArgs, etc.)
│   ├── commands/
│   │   ├── mod.rs          # Command execute() dispatcher
│   │   ├── scope.rs        # SCOPE COMMANDS IMPLEMENTATION (EXISTING)
│   │   ├── mode.rs         # Mode commands (similar pattern)
│   │   ├── init.rs         # Init command (simple reference)
│   │   ├── add.rs          # Add command (uses active scope)
│   │   ├── commit_cmd.rs   # Commit command
│   │   └── status.rs       # Status command (shows active scope)
│   ├── core/
│   │   ├── mod.rs          # Core exports
│   │   ├── error.rs        # JinError enum, Result type
│   │   ├── config.rs       # ProjectContext struct
│   │   └── layer.rs        # Layer enum with 9-layer hierarchy
│   ├── git/
│   │   ├── mod.rs          # Git module exports
│   │   ├── repo.rs         # JinRepo struct with open_or_create()
│   │   ├── refs.rs         # RefOps trait (set_ref, ref_exists, delete_ref)
│   │   └── objects.rs      # ObjectOps trait (create_tree, create_commit)
│   └── main.rs             # CLI entry point
├── tests/
│   ├── cli_basic.rs        # CLI integration tests
│   ├── mode_scope_workflow.rs  # Scope workflow integration tests
│   └── common/
│       ├── fixtures.rs     # Test helpers
│       └── assertions.rs   # Assertion helpers
└── plan/
    └── P4M3T2/
        └── PRP.md          # This document
```

### Desired Codebase Tree (Already Exists)

```bash
# Note: The implementation already exists. This documents what IS there.

src/commands/scope.rs
├── execute()              # Entry point: match on ScopeAction
├── validate_scope_name()   # Name validation (allows colons)
├── validate_mode_name()    # Mode name validation (for --mode flag)
├── create()               # Create new scope with initial commit
├── use_scope()            # Activate scope in ProjectContext
├── list()                 # List all scopes (untethered + mode-bound)
├── delete()               # Delete scope and associated refs
├── show()                 # Show currently active scope
├── unset()                # Deactivate scope
└── tests/
    ├── test_validate_scope_name_*()    # Validation tests
    ├── test_create_*()                  # Create tests (untethered, mode-bound)
    ├── test_use_scope()                 # Use tests
    ├── test_list_*()                    # List tests
    ├── test_delete_*()                  # Delete tests
    ├── test_show()                      # Show tests
    └── test_unset()                     # Unset tests
```

### Known Gotchas & Library Quirks

```rust
// CLAP DERIVE API QUIRKS
// ======================
// 1. Subcommand enum MUST be in same module or re-exported properly
// 2. #[command(subcommand)] attribute for nested subcommands
// 3. Variant-specific fields become positional arguments
// 4. Optional flags use #[arg(long)] attribute

// GIT2 QUIRKS
// ===========
// 1. Reference names MUST be valid (use Reference::is_valid_name() check)
// 2. Bare repos don't have working directory - don't use checkout
// 3. Ref creation uses reference() method, not direct reference creation
// 4. Empty tree is created with empty slice: repo.create_tree(&[])
// 5. Colons ARE valid in ref names for scopes (unlike typical git refs)

// PROJECT CONTEXT QUIRKS
// =====================
// 1. ProjectContext::load() returns Err(JinError::NotInitialized) if .jin/context missing
// 2. Context uses YAML format, not TOML
// 3. context.save() creates parent directories if needed
// 4. Scope is Option<String> - None means no active scope

// TEST ISOLATION QUIRKS
// =====================
// 1. JIN_DIR env var MUST be set before any JinRepo operations
// 2. Tests should use std::process::id() for unique names
// 3. TempDir auto-cleans on drop - don't manually delete
// 4. Each test should create isolated temp directory

// VALIDATION QUIRKS
// ==================
// 1. Reserved names: "default", "global", "base" (lowercase only)
// 2. Scope names: alphanumeric + underscore + COLON (allows hierarchical naming)
// 3. Mode names: alphanumeric + underscore only (no colons)
// 4. Empty string should fail validation
// 5. Untethered ref path: "refs/jin/scopes/{name}"
// 6. Mode-bound ref path: "refs/jin/modes/{mode}/scopes/{name}"

// SCOPE-SPECIFIC QUIRKS
// =====================
// 1. Scope names support colons (e.g., "language:javascript:framework")
// 2. Colons are preserved in ref names (not converted to slashes for scope refs)
// 3. Mode-bound scopes require parent mode to exist
// 4. List command shows both untethered and mode-bound scopes
// 5. Delete command removes all refs (untethered + all mode-bound instances)

// ERROR HANDLING QUIRKS
// =====================
// 1. Use JinError::AlreadyExists for duplicate scope names
// 2. Use JinError::NotFound for missing scope names
// 3. Use JinError::Other for validation failures with descriptive message
// 4. Include "Create it with: jin scope create <name>" in not-found errors
// 5. For mode-bound scopes, include error if mode doesn't exist
```

## Implementation Blueprint

### Data Models

The scope commands use existing data models - no new types needed:

```rust
// EXISTING TYPES IN USE
// ======================

// src/core/config.rs - ProjectContext persists active scope
pub struct ProjectContext {
    pub mode: Option<String>,     // Active mode name
    pub scope: Option<String>,    // Active scope name
    pub project: Option<String>,  // Project name (auto-inferred)
    pub version: u32,
    pub last_updated: Option<String>,
}

// src/cli/mod.rs - CLI argument structure
#[derive(Subcommand, Debug)]
pub enum ScopeAction {
    Create {
        name: String,              // jin scope create <name>
        #[arg(long)]
        mode: Option<String>,      // --mode=<mode> for mode-bound scopes
    },
    Use { name: String },          // jin scope use <name>
    List,                          // jin scope list
    Delete { name: String },       // jin scope delete <name>
    Show,                          // jin scope show
    Unset,                         // jin scope unset
}
```

### Implementation Tasks

```yaml
# NOTE: These tasks document the EXISTING implementation for reference.

Task 1: DEFINE CLI STRUCTURE (COMPLETED)
  - LOCATION: src/cli/mod.rs
  - IMPLEMENT: ScopeAction enum with #[derive(Subcommand)]
  - VARIANTS: Create { name, mode?, Use { name }, List, Delete { name }, Show, Unset
  - INTEGRATION: Add Scope(ScopeAction) variant to Commands enum
  - PATTERN: Follow ModeAction pattern with optional mode field

Task 2: WIRE COMMAND EXECUTOR (COMPLETED)
  - LOCATION: src/commands/mod.rs
  - IMPLEMENT: Commands::Scope(action) => scope::execute(action) match arm
  - PATTERN: Follow Commands::Mode(action) pattern (line 38)
  - EXPORT: Add pub mod scope; to module exports

Task 3: IMPLEMENT SCOPE COMMAND MODULE (COMPLETED)
  - LOCATION: src/commands/scope.rs
  - FUNCTION: execute() -> entry point matching ScopeAction variants
  - FUNCTIONS: create(), use_scope(), list(), delete(), show(), unset()
  - HELPERS: validate_scope_name(), validate_mode_name()
  - DEPENDENCIES: crate::cli::ScopeAction, crate::core::{JinError, ProjectContext, Result}
  - PATTERN: Follow mode.rs implementation (similar structure, colon support added)

Task 4: IMPLEMENT VALIDATION FUNCTIONS (COMPLETED)
  - LOCATION: src/commands/scope.rs::validate_scope_name()
  - CHECKS: Empty string, invalid characters, reserved names
  - RESERVED: ["default", "global", "base"]
  - VALID CHARS: Alphanumeric + underscore + COLON (c.is_alphanumeric() || c == '_' || c == ':')
  - ERROR: JinError::Other with descriptive message

Task 5: IMPLEMENT CREATE SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/scope.rs::create()
  - FLOW: Validate name -> Open repo -> Determine ref path (mode-bound or untethered) -> Check existence -> Create empty tree -> Create commit -> Set ref
  - UNTETHERED REF PATH: format!("refs/jin/scopes/{}", ref_safe_name)
  - MODE-BOUND REF PATH: format!("refs/jin/modes/{}/scopes/{}", mode_name, ref_safe_name)
  - MODE VALIDATION: Check if mode exists when --mode flag is provided
  - SUCCESS MESSAGE: "Created scope '{name}' (untethered)" or "Created scope '{name}' bound to mode '{mode}'"

Task 6: IMPLEMENT USE SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/scope.rs::use_scope()
  - FLOW: Validate name -> Open repo -> Check ref exists (untethered or mode-bound) -> Load context -> Update scope -> Save context
  - DUAL LOOKUP: Check both untethered and mode-bound refs
  - CONTEXT UPDATE: context.scope = Some(name.to_string())
  - SUCCESS MESSAGE: "Activated scope '{name}'\nStage files with: jin add --scope={name}"

Task 7: IMPLEMENT LIST SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/scope.rs::list()
  - FLOW: Open repo -> Load context -> List untethered refs -> List mode-bound refs -> Display with active marker
  - UNTETHERED PATTERN: "refs/jin/scopes/*"
  - MODE-BOUND PATTERN: "refs/jin/modes/*/scopes/*"
  - DISPLAY: "* {name} (untethered) [active]" or "{name} (mode: {mode})"
  - EMPTY MESSAGE: "No scopes found.\nCreate one with: jin scope create <name>"

Task 8: IMPLEMENT DELETE SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/scope.rs::delete()
  - FLOW: Validate name -> Open repo -> Find all refs (untethered + mode-bound) -> Load context -> Unset if active -> Delete all refs -> Cleanup layer refs
  - DUAL DELETION: Removes both untethered and all mode-bound instances
  - CLEANUP PATTERNS: refs/jin/layers/scope/{name}, refs/jin/layers/mode/*/scope/{name}
  - SUCCESS MESSAGE: "Deleted scope '{name}'"

Task 9: IMPLEMENT SHOW SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/scope.rs::show()
  - FLOW: Load context -> Display scope or "No active scope"
  - OUTPUT: "Active scope: {name}" or "No active scope"

Task 10: IMPLEMENT UNSET SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/scope.rs::unset()
  - FLOW: Load context -> Check scope is set -> Set scope to None -> Save context
  - SUCCESS MESSAGE: "Deactivated scope\nScope layers no longer available for staging"

Task 11: ADD UNIT TESTS (COMPLETED)
  - LOCATION: src/commands/scope.rs tests module
  - FIXTURE: setup_test_env() - creates TempDir, sets JIN_DIR, creates .jin/context
  - FIXTURE: create_test_mode() - helper to create mode for mode-bound scope tests
  - TESTS: test_validate_scope_name_*() - validation edge cases
  - TESTS: test_create_untethered_scope(), test_create_mode_bound_scope()
  - TESTS: test_create_scope_with_colon() - hierarchical naming
  - TESTS: test_create_scope_nonexistent_mode() - mode validation
  - TESTS: test_create_scope_duplicate()
  - TESTS: test_use_scope(), test_use_scope_nonexistent()
  - TESTS: test_list_empty(), test_list_with_scopes()
  - TESTS: test_delete_untethered_scope(), test_delete_mode_bound_scope()
  - TESTS: test_delete_active_scope(), test_delete_nonexistent()
  - TESTS: test_show_no_scope(), test_show_with_scope()
  - TESTS: test_unset(), test_unset_no_scope()

Task 12: VERIFY INTEGRATION TESTS (COMPLETED)
  - LOCATION: tests/cli_basic.rs, tests/mode_scope_workflow.rs
  - TESTS: test_scope_*_subcommand() functions
  - PATTERN: Use jin().args(["scope", "<subcommand>", ...]).assert()
  - ISOLATION: Set JIN_DIR env variable for each test
```

### Implementation Patterns & Key Details

```rust
// ============================================
// CRITICAL PATTERNS FROM EXISTING CODEBASE
// ============================================

// PATTERN 1: Command Entry Point
// ===============================
// File: src/commands/scope.rs:8-17
pub fn execute(action: ScopeAction) -> Result<()> {
    match action {
        ScopeAction::Create { name, mode } => create(&name, mode.as_deref()),
        ScopeAction::Use { name } => use_scope(&name),
        ScopeAction::List => list(),
        ScopeAction::Delete { name } => delete(&name),
        ScopeAction::Show => show(),
        ScopeAction::Unset => unset(),
    }
}

// PATTERN 2: Scope Name Validation (with colon support)
// =====================================================
// File: src/commands/scope.rs:25-52
fn validate_scope_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(JinError::Other("Scope name cannot be empty".to_string()));
    }

    // Check for valid characters (alphanumeric, underscore, and colon)
    // GOTCHA: Scopes allow colons (unlike modes)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') {
        return Err(JinError::Other(format!(
            "Invalid scope name '{}'. Use alphanumeric characters, underscores, and colons only.",
            name
        )));
    }

    // Check for reserved names
    let reserved = ["default", "global", "base"];
    if reserved.contains(&name) {
        return Err(JinError::Other(format!(
            "Scope name '{}' is reserved.",
            name
        )));
    }

    Ok(())
}

// PATTERN 3: Create Scope with Mode Support
// ===========================================
// File: src/commands/scope.rs:71-139
fn create(name: &str, mode: Option<&str>) -> Result<()> {
    validate_scope_name(name)?;

    let repo = JinRepo::open_or_create()?;

    // GOTCHA: Colons are preserved in scope refs (not converted)
    let ref_safe_name = name.replace(':', "/");  // Actually converts to / for ref

    // Determine ref path based on mode parameter
    let ref_path = if let Some(mode_name) = mode {
        // Mode-bound scope
        validate_mode_name(mode_name)?;

        // Check if mode exists
        let mode_ref = format!("refs/jin/modes/{}", mode_name);
        if !repo.ref_exists(&mode_ref) {
            return Err(JinError::NotFound(format!(
                "Mode '{}' not found. Create it with: jin mode create {}",
                mode_name, mode_name
            )));
        }

        format!("refs/jin/modes/{}/scopes/{}", mode_name, ref_safe_name)
    } else {
        // Untethered scope
        format!("refs/jin/scopes/{}", ref_safe_name)
    };

    // Check if scope already exists
    if repo.ref_exists(&ref_path) {
        return Err(JinError::AlreadyExists(format!(
            "Scope '{}' already exists",
            name
        )));
    }

    // Create empty tree for initial commit
    let empty_tree = repo.create_tree(&[])?;

    // Create initial commit
    let commit_message = if let Some(mode_name) = mode {
        format!("Initialize scope: {} (mode: {})", name, mode_name)
    } else {
        format!("Initialize scope: {}", name)
    };

    let commit_oid = repo.create_commit(None, &commit_message, empty_tree, &[])?;

    // Set Git ref
    let reflog_message = if let Some(mode_name) = mode {
        format!("create scope {} for mode {}", name, mode_name)
    } else {
        format!("create scope {}", name)
    };

    repo.set_ref(&ref_path, commit_oid, &reflog_message)?;

    // Print success message
    if let Some(mode_name) = mode {
        println!("Created scope '{}' bound to mode '{}'", name, mode_name);
    } else {
        println!("Created scope '{}' (untethered)", name);
    }
    println!("Activate with: jin scope use {}", name);

    Ok(())
}

// PATTERN 4: Use Scope (Activate) with Dual Lookup
// =================================================
// File: src/commands/scope.rs:142-188
fn use_scope(name: &str) -> Result<()> {
    validate_scope_name(name)?;

    let repo = JinRepo::open_or_create()?;

    let ref_safe_name = name.replace(':', "/");

    // GOTCHA: Check both untethered and mode-bound refs
    let untethered_ref = format!("refs/jin/scopes/{}", ref_safe_name);
    let mode_bound_pattern = format!("refs/jin/modes/*/scopes/{}", ref_safe_name);

    let exists = repo.ref_exists(&untethered_ref)
        || !repo.list_refs(&mode_bound_pattern).unwrap_or_default().is_empty();

    if !exists {
        return Err(JinError::NotFound(format!(
            "Scope '{}' not found. Create it with: jin scope create {}",
            name, name
        )));
    }

    // Load project context
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Update scope
    context.scope = Some(name.to_string());

    // Save context
    context.save()?;

    println!("Activated scope '{}'", name);
    println!("Stage files with: jin add --scope={}", name);

    Ok(())
}

// PATTERN 5: List Scopes (Both Types)
// =====================================
// File: src/commands/scope.rs:191-255
fn list() -> Result<()> {
    let repo = JinRepo::open_or_create()?;

    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Find untethered scopes
    let untethered_refs = repo.list_refs("refs/jin/scopes/*").unwrap_or_default();

    // Find mode-bound scopes
    let mode_bound_refs = repo.list_refs("refs/jin/modes/*/scopes/*").unwrap_or_default();

    if untethered_refs.is_empty() && mode_bound_refs.is_empty() {
        println!("No scopes found.");
        println!("Create one with: jin scope create <name>");
        return Ok(());
    }

    println!("Available scopes:");

    // Display untethered scopes
    for ref_path in untethered_refs {
        let ref_safe_name = ref_path.strip_prefix("refs/jin/scopes/").unwrap_or(&ref_path);
        // Convert back from ref-safe format (slashes to colons)
        let display_name = ref_safe_name.replace('/', ":");

        if Some(display_name.as_str()) == context.scope.as_deref() {
            println!("  * {} (untethered) [active]", display_name);
        } else {
            println!("    {} (untethered)", display_name);
        }
    }

    // Display mode-bound scopes
    for ref_path in mode_bound_refs {
        // Parse: refs/jin/modes/{mode}/scopes/{scope}
        if let Some(rest) = ref_path.strip_prefix("refs/jin/modes/") {
            if let Some(mode_end) = rest.find("/scopes/") {
                let mode_name = &rest[..mode_end];
                let ref_safe_scope = &rest[mode_end + 8..]; // Skip "/scopes/"
                let display_name = ref_safe_scope.replace('/', ":");

                if Some(display_name.as_str()) == context.scope.as_deref() {
                    println!("  * {} (mode: {}) [active]", display_name, mode_name);
                } else {
                    println!("    {} (mode: {})", display_name, mode_name);
                }
            }
        }
    }

    Ok(())
}

// PATTERN 6: Delete Scope (Both Types) with Cleanup
// ===================================================
// File: src/commands/scope.rs:258-329
fn delete(name: &str) -> Result<()> {
    validate_scope_name(name)?;

    let repo = JinRepo::open_or_create()?;

    let ref_safe_name = name.replace(':', "/");

    // Find all refs for this scope (both mode-bound and untethered)
    let untethered_ref = format!("refs/jin/scopes/{}", ref_safe_name);
    let mode_bound_pattern = format!("refs/jin/modes/*/scopes/{}", ref_safe_name);

    let mut refs_to_delete = Vec::new();

    // Check untethered
    if repo.ref_exists(&untethered_ref) {
        refs_to_delete.push(untethered_ref);
    }

    // Check mode-bound
    if let Ok(mode_bound_refs) = repo.list_refs(&mode_bound_pattern) {
        refs_to_delete.extend(mode_bound_refs);
    }

    if refs_to_delete.is_empty() {
        return Err(JinError::NotFound(format!("Scope '{}' not found", name)));
    }

    // Load project context to check if active
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // If scope is active, unset it first
    if Some(name) == context.scope.as_deref() {
        println!("Scope '{}' is currently active. Deactivating...", name);
        context.scope = None;
        context.save()?;
    }

    // Delete all refs
    for ref_path in &refs_to_delete {
        repo.delete_ref(ref_path)?;
    }

    // Delete associated layer refs (may not exist if no files committed)
    let layer_patterns = [
        format!("refs/jin/layers/scope/{}", name),
        format!("refs/jin/layers/mode/*/scope/{}", name),
    ];

    for pattern in &layer_patterns {
        let _ = repo.delete_ref(pattern);

        if let Ok(refs) = repo.list_refs(pattern) {
            for ref_to_delete in refs {
                let _ = repo.delete_ref(&ref_to_delete);
            }
        }
    }

    println!("Deleted scope '{}'", name);

    Ok(())
}

// PATTERN 7: Test Isolation Fixture
// ==================================
// File: src/commands/scope.rs:384-400
fn setup_test_env() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Set JIN_DIR to an isolated directory for this test
    let jin_dir = temp.path().join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    // Change to temp directory for project context
    std::env::set_current_dir(temp.path()).unwrap();

    // Initialize .jin directory and context
    std::fs::create_dir(".jin").unwrap();
    let context = ProjectContext::default();
    context.save().unwrap();

    temp
}

// PATTERN 8: Test Mode Creation Helper
// =====================================
// File: src/commands/scope.rs:402-414
fn create_test_mode(name: &str) {
    let repo = JinRepo::open_or_create().unwrap();
    let empty_tree = repo.create_tree(&[]).unwrap();
    let commit_oid = repo
        .create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])
        .unwrap();
    repo.set_ref(
        &format!("refs/jin/modes/{}", name),
        commit_oid,
        &format!("create mode {}", name),
    )
    .unwrap();
}

// GOTCHA: Colon to Slash Conversion for Ref Names
// ================================================
// Scope names use colons (language:javascript) but Git refs use slashes
// Convert: name.replace(':', "/") before creating ref
// Convert back: ref_safe_name.replace('/', ":") when displaying

// GOTCHA: Dual Ref Lookup for Use/Delete
// =======================================
// Scopes can be untethered OR mode-bound
// Must check both: refs/jin/scopes/{name} AND refs/jin/modes/*/scopes/{name}
// Use wildcards to find all mode-bound instances

// GOTCHA: Context Load Error Handling
// ====================================
// Must handle NotInitialized differently from other errors
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => {
        return Err(JinError::NotInitialized);  // Propagate initialization error
    }
    Err(_) => ProjectContext::default(),  // Other errors use default
};

// GOTCHA: Ref Path Patterns
// ==========================
// Untethered scope refs: refs/jin/scopes/{name}
// Mode-bound scope refs: refs/jin/modes/{mode}/scopes/{name}
// Scope layer refs: refs/jin/layers/scope/{name}
// Mode-scope layer refs: refs/jin/layers/mode/{mode}/scope/{name}
```

### Integration Points

```yaml
PROJECT CONTEXT:
  - file: .jin/context
  - format: YAML
  - fields: scope: Option<String>
  - load: ProjectContext::load() -> Result<ProjectContext>
  - save: context.save() -> Result<()>

GIT REFS:
  - namespace_untethered: refs/jin/scopes/
  - namespace_mode_bound: refs/jin/modes/{mode}/scopes/
  - pattern_untethered: refs/jin/scopes/{scope_name}
  - pattern_mode_bound: refs/jin/modes/{mode_name}/scopes/{scope_name}
  - layer_untethered: refs/jin/layers/scope/{scope_name}
  - layer_mode_bound: refs/jin/layers/mode/{mode_name}/scope/{scope_name}
  - create: repo.set_ref(path, oid, message)
  - check: repo.ref_exists(path)
  - delete: repo.delete_ref(path)
  - list_untethered: repo.list_refs("refs/jin/scopes/*")
  - list_mode_bound: repo.list_refs("refs/jin/modes/*/scopes/*")

CLI ROUTING:
  - enum: Commands::Scope(ScopeAction)
  - handler: scope::execute(action)
  - subcommands: ScopeAction::{Create, Use, List, Delete, Show, Unset}
  - optional_flag: --mode=<mode> for create subcommand

STAGING INTEGRATION:
  - add.rs: Reads context.scope to determine active scope
  - add --scope=X: Routes to ScopeBase layer (Layer 6) if no mode
  - add --mode --scope=X: Routes to ModeScope layer (Layer 3)
  - add --mode --project --scope=X: Routes to ModeScopeProject layer (Layer 4)

STATUS INTEGRATION:
  - status.rs: Displays active scope from context
  - format: "Scope: {name} (active)" or "No active scope"

LAYER PRECEDENCE:
  - Layer 3 (ModeScope): mode/{mode}/scope/{scope}/ - highest precedence for scope
  - Layer 4 (ModeScopeProject): mode/{mode}/project/{project}/scope/{scope}/
  - Layer 6 (ScopeBase): scope/{scope}/ - lowest precedence, only if no mode-bound scope
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo fmt --check                      # Verify formatting
cargo clippy --all-targets --all-features  # Lint checks

# Auto-format and fix linting issues
cargo fmt
cargo clippy --fix --all-targets --all-features

# Project-specific validation
cargo check --package jin              # Check compilation
cargo test --package jin --lib --no-run  # Check test compilation

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test scope command module
cargo test --package jin --lib commands::scope::tests

# Test specific functions
cargo test --package jin --lib validate_scope_name
cargo test --package jin --lib test_create_untethered_scope
cargo test --package jin --lib test_create_mode_bound_scope
cargo test --package jin --lib test_create_scope_with_colon
cargo test --package jin --lib test_use_scope
cargo test --package jin --lib test_list
cargo test --package jin --lib test_delete_untethered_scope
cargo test --package jin --lib test_delete_mode_bound_scope
cargo test --package jin --lib test_show
cargo test --package jin --lib test_unset

# Full lib test suite
cargo test --package jin --lib

# Coverage validation (if cargo-llvm-cov is installed)
cargo llvm-cov --package jin --lib

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the binary
cargo build --release

# Scope create (untethered) command
./target/release/jin scope create language:javascript

# Scope create (mode-bound) command
./target/release/jin scope create infra:docker --mode=production

# Scope list command (should show both scopes)
./target/release/jin scope list

# Scope show command (should show no active scope initially)
./target/release/jin scope show

# Scope use command
./target/release/jin scope use language:javascript

# Scope show (should show active scope)
./target/release/jin scope show

# Scope unset command
./target/release/jin scope unset

# Scope delete command
./target/release/jin scope delete language:javascript

# Run integration tests
cargo test --test cli_basic test_scope
cargo test --test mode_scope_workflow

# Expected: All commands work correctly, integration tests pass.
```

### Level 4: End-to-End Workflow Validation

```bash
# Complete scope workflow test
set -e  # Exit on any error

SCOPE_NAME="language:rust_$(date +%s)"
MODE_NAME="development_$(date +%s)"

# Initialize Jin
mkdir -p /tmp/jin_e2e
cd /tmp/jin_e2e
jin init

# Create mode first
jin mode create "$MODE_NAME"
echo "✓ Mode created"

# Create mode-bound scope
jin scope create "$SCOPE_NAME" --mode="$MODE_NAME"
echo "✓ Mode-bound scope created"

# Create untethered scope
jin scope create "infra:test"
echo "✓ Untethered scope created"

# List scopes (should show both)
jin scope list | grep "$SCOPE_NAME"
jin scope list | grep "infra:test"
echo "✓ Scopes appear in list"

# Use scope
jin scope use "$SCOPE_NAME"
echo "✓ Scope activated"

# Show scope (should show active)
jin scope show | grep "$SCOPE_NAME"
echo "✓ Scope shows as active"

# Verify context file
grep "scope: $SCOPE_NAME" .jin/context
echo "✓ Context file updated"

# Create a file and add to scope layer
echo '{"edition": "2021"}' > Cargo.toml
jin add Cargo.toml --scope="$SCOPE_NAME"
echo "✓ File staged to scope layer"

# Commit
jin commit -m "Test commit"
echo "✓ Changes committed"

# Apply changes
jin apply
echo "✓ Changes applied"

# Check status
jin status | grep "Clean"
echo "✓ Workspace is clean"

# Unset scope
jin scope unset
echo "✓ Scope deactivated"

# Delete scope
jin scope delete "$SCOPE_NAME"
echo "✓ Scope deleted"

# Cleanup
cd /
rm -rf /tmp/jin_e2e

echo "✅ All E2E validation checks passed!"
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --package jin --lib commands::scope::tests`
- [ ] No linting errors: `cargo clippy --all-targets`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Integration tests pass: `cargo test --test cli_basic --test mode_scope_workflow`
- [ ] Binary compiles without warnings: `cargo build --release`

### Feature Validation

- [ ] Scope create creates untethered ref at `refs/jin/scopes/{name}`
- [ ] Scope create with --mode creates mode-bound ref at `refs/jin/modes/{mode}/scopes/{name}`
- [ ] Scope use updates `.jin/context` with scope field
- [ ] Scope list displays both untethered and mode-bound scopes with active indicator
- [ ] Scope delete removes both untethered and all mode-bound refs
- [ ] Scope show displays active scope or "No active scope"
- [ ] Scope unset clears scope from context
- [ ] Reserved names ("default", "global", "base") are rejected
- [ ] Invalid characters are rejected (but colons are allowed)
- [ ] Hierarchical naming works (e.g., "language:javascript:framework")
- [ ] Duplicate scope names are rejected
- [ ] Using non-existent scope shows helpful error with create command
- [ ] Creating mode-bound scope with non-existent mode shows error

### Code Quality Validation

- [ ] Follows existing codebase patterns (matches mode.rs structure)
- [ ] Error messages are descriptive and actionable
- [ ] User guidance provided (next steps in success messages)
- [ ] Tests use isolated environments (TempDir, JIN_DIR)
- [ ] Test coverage includes all code paths
- [ ] Documentation comments present on public functions
- [ ] Colon handling is consistent (creation, listing, deletion)

### CLI/UX Validation

- [ ] Help text works: `jin scope --help`
- [ ] Subcommand help works: `jin scope create --help`
- [ ] Optional --mode flag works for create
- [ ] Error messages are user-friendly
- [ ] Success messages include next action guidance
- [ ] Active scope clearly indicated in list output
- [ ] Mode-bound scopes show parent mode in list

---

## Anti-Patterns to Avoid

-  Don't use hyphens in scope names (validation rejects them)
-  Don't allow reserved names ("default", "global", "base")
-  Don't forget to unset scope in context when deleting active scope
-  Don't skip cleanup of associated layer refs on delete
-  Don't ignore NotInitialized errors - propagate them to user
-  Don't create scope refs outside `refs/jin/scopes/` or `refs/jin/modes/*/scopes/` namespaces
-  Don't use ProjectContext::default() when NotInitialized should be returned
-  Don't forget to set JIN_DIR environment variable in tests
-  Don't forget to check both untethered and mode-bound refs in use/delete operations
-  Don't forget to validate mode existence when creating mode-bound scopes
-  Don't catch all exceptions - be specific with JinError variants

## Implementation Status: COMPLETE

**Note**: This PRP documents the EXISTING IMPLEMENTATION in `src/commands/scope.rs`.

All tasks described above have been completed. The scope commands are fully functional with:

1. **CLI Structure**: ScopeAction enum defined in `src/cli/mod.rs`
2. **Command Wiring**: `Commands::Scope(action) => scope::execute(action)` in `src/commands/mod.rs`
3. **Implementation**: Complete in `src/commands/scope.rs` with all 6 subcommands
4. **Unit Tests**: Comprehensive test coverage in scope.rs tests module
5. **Integration Tests**: Tests in `tests/cli_basic.rs` and `tests/mode_scope_workflow.rs`

**Key Differences from Mode Commands**:
- Scope names support colons for hierarchical naming (e.g., `language:javascript`)
- Scopes can be untethered or mode-bound
- Mode-bound scopes validate parent mode existence
- List/delete operations handle both untethered and mode-bound scopes
- Dual ref lookup for use/delete operations

**Confidence Score**: 10/10 - Implementation is complete, tested, and functional.

**For reference**: Use this PRP to understand the patterns for implementing similar lifecycle commands. The scope commands demonstrate how to extend the basic mode pattern with:
- Optional flags (e.g., `--mode` for create)
- Hierarchical naming (colons in names)
- Dual storage locations (untethered + mode-bound)
- Multi-ref operations (find all matching refs)
