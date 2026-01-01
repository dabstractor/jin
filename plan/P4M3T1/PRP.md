# PRP: Mode Commands (P4.M3.T1)

---

## Goal

**Feature Goal**: Implement `jin mode` subcommands for managing mode lifecycle (create, use, list, delete, show, unset) as part of the 9-layer configuration hierarchy system.

**Deliverable**: A complete CLI command module with all mode operations wired through clap derive API, including unit tests and integration test coverage.

**Success Definition**:
- All 6 mode subcommands are accessible via `jin mode <subcommand>`
- Mode refs are properly managed in `refs/jin/modes/` namespace
- Active mode is persisted in `.jin/context`
- User messages guide users to next steps
- All unit tests pass: `cargo test --package jin --lib commands::mode`
- Integration tests pass for basic mode workflows
- Error handling covers edge cases (invalid names, duplicates, not found, etc.)

## User Persona

**Target User**: Developer using Jin to manage per-mode configuration overlays (e.g., different AI assistants: "claude", "cursor", "copilot")

**Use Case**: Developer wants to create isolated configuration layers for different modes of development or tool usage, allowing them to stage files to mode-specific layers and switch between modes.

**User Journey**:
1. User creates a new mode: `jin mode create claude`
2. User activates the mode: `jin mode use claude`
3. User stages files to the mode layer: `jin add config.json --mode`
4. User commits staged changes: `jin commit -m "Add Claude config"`
5. User can list all modes: `jin mode list`
6. User can check active mode: `jin mode show`
7. User can deactivate mode: `jin mode unset`
8. User can delete mode: `jin mode delete claude`

**Pain Points Addressed**:
- Managing separate configuration files for different development contexts
- Need to quickly switch between different tool configurations
- Avoiding git-tracked configuration conflicts
- Isolating mode-specific settings from project defaults

## Why

- **Business value**: Enables developers to maintain multiple configuration contexts within a single project, reducing context switching overhead
- **Integration with existing features**: Mode commands integrate with the 9-layer hierarchy system (Layer 2: ModeBase, Layer 5: ModeProject, etc.)
- **Problems solved**:
  - Eliminates need for manual configuration file swapping
  - Provides git-like workflow for configuration management
  - Enables layer composition for complex configuration scenarios

## What

**User-visible behavior**:
- `jin mode create <name>` - Creates a new mode with empty initial commit
- `jin mode use <name>` - Activates a mode (updates `.jin/context`)
- `jin mode list` - Lists all available modes with active indicator
- `jin mode delete <name>` - Deletes a mode and its associated refs
- `jin mode show` - Shows currently active mode
- `jin mode unset` - Deactivates current mode

**Technical requirements**:
- Mode names must be alphanumeric + underscores only
- Reserved names: "default", "global", "base"
- Modes stored as Git refs at `refs/jin/modes/<name>`
- Active mode persisted in `.jin/context` as YAML
- Proper error messages for edge cases

### Success Criteria

- [ ] All 6 subcommands accessible and functional
- [ ] Mode validation prevents invalid/reserved names
- [ ] Refs created/updated/deleted correctly in Git storage
- [ ] ProjectContext load/save operations work correctly
- [ ] Integration tests pass for mode workflow scenarios
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

### Documentation & References

```yaml
# CRITICAL EXTERNAL DOCUMENTATION
- url: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
  why: clap derive API for Parser and Subcommand traits - essential for CLI structure
  critical: Mode commands use #[derive(Subcommand)] for nested subcommands

- url: https://docs.rs/clap/latest/clap/struct.Command.html#method.arg
  why: Command argument configuration reference
  critical: Positional arguments and flags for mode subcommands

- url: https://github.com/clap-rs/clap/blob/master/examples/git_derive.rs
  why: Real-world example of nested subcommands similar to jin mode structure
  critical: Shows the pattern for command hierarchies

- url: https://docs.rs/git2/latest/git2/
  why: git2 crate documentation for Git operations
  critical: Reference management, object creation, bare repository handling

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.reference
  why: Creating/updating Git references for mode storage
  critical: Modes stored as refs in refs/jin/modes/ namespace

- url: https://docs.rs/git2/latest/git2/struct.Reference.html
  why: Reference operations (find, delete, resolve)
  critical: Mode ref existence checks and deletion

- url: https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html
  why: Test isolation using temporary directories
  critical: Unit test pattern for mode operations

- file: /home/dustin/projects/jin/plan/P4M3T1/research/clap_research.md
  why: Compiled research on clap derive API patterns
  section: Complete code examples for nested subcommands

- file: /home/dustin/projects/jin/plan/P4M3T1/research/git2_research.md
  why: Compiled research on git2 crate patterns
  section: Reference management best practices

- file: /home/dustin/projects/jin/plan/P4M3T1/research/tempfile_research.md
  why: Compiled research on test isolation patterns
  section: JIN_DIR environment variable setup

# CODEBASE PATTERNS TO FOLLOW
- file: /home/dustin/projects/jin/src/commands/mode.rs
  why: EXISTING COMPLETE IMPLEMENTATION - reference for validation, exact patterns used
  pattern: validate_mode_name() function shows validation logic
  gotcha: Reserved names check is critical for avoiding conflicts

- file: /home/dustin/projects/jin/src/commands/scope.rs
  why: Nearly identical pattern for scope commands - use as reference
  pattern: Command execute() function with match on action enum
  gotcha: Scope handles colons in names (modes don't)

- file: /home/dustin/projects/jin/src/cli/mod.rs
  why: Shows exact ModeAction enum definition and how Commands enum integrates it
  pattern: #[derive(Subcommand)] enum ModeAction with variant-specific fields
  gotcha: Keep ModeAction inside cli/mod.rs, not in args.rs

- file: /home/dustin/projects/jin/src/commands/mod.rs
  why: Shows command module exports and execute() wiring
  pattern: Commands::Mode(action) => mode::execute(action)
  gotcha: Must add pub mod mode; to module exports

- file: /home/dustin/projects/jin/src/core/config.rs
  why: ProjectContext struct with mode field that persists state
  pattern: context.mode = Some(name.to_string()); context.save()
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
  pattern: ModeBase ref path is "refs/jin/layers/mode/{mode}"
  gotcha: Mode refs use different path than layer refs

# TEST PATTERNS
- file: /home/dustin/projects/jin/tests/cli_basic.rs
  why: Integration test patterns for CLI commands
  pattern: Use std::process::id() for unique test names
  gotcha: Set JIN_DIR environment variable for test isolation

- file: /home/dustin/projects/jin/tests/mode_scope_workflow.rs
  why: Integration tests for mode/scoped workflows
  pattern: create_mode(), jin().args(["mode", "use", name])
  gotcha: Tests use mode list assertions for verification

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
  section: pub use commands::mode;
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
│   │   ├── mode.rs         # MODE COMMANDS IMPLEMENTATION (EXISTING)
│   │   ├── scope.rs        # Scope commands (similar pattern)
│   │   ├── init.rs         # Init command (simple reference)
│   │   ├── add.rs          # Add command (uses active mode)
│   │   ├── commit_cmd.rs   # Commit command
│   │   └── status.rs       # Status command (shows active mode)
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
│   ├── mode_scope_workflow.rs  # Mode workflow integration tests
│   └── common/
│       ├── fixtures.rs     # Test helpers
│       └── assertions.rs   # Assertion helpers
└── plan/
    └── P4M3T1/
        └── PRP.md          # This document
```

### Desired Codebase Tree (Already Exists)

```bash
# Note: The implementation already exists. This documents what IS there.

src/commands/mode.rs
├── execute()              # Entry point: match on ModeAction
├── validate_mode_name()   # Name validation
├── create()               # Create new mode with initial commit
├── use_mode()             # Activate mode in ProjectContext
├── list()                 # List all modes with active indicator
├── delete()               # Delete mode and associated refs
├── show()                 # Show currently active mode
├── unset()                # Deactivate mode
└── tests/
    ├── test_validate_mode_name_*()    # Validation tests
    ├── test_create_mode()              # Create tests
    ├── test_use_mode()                 # Use tests
    ├── test_list_*()                   # List tests
    ├── test_delete_*()                 # Delete tests
    ├── test_show()                     # Show tests
    └── test_unset()                    # Unset tests
```

### Known Gotchas & Library Quirks

```rust
// CLAP DERIVE API QUIRKS
// ======================
// 1. Subcommand enum MUST be in same module or re-exported properly
// 2. #[command(subcommand)] attribute for nested subcommands
// 3. Variant-specific fields become positional arguments

// GIT2 QUIRKS
// ===========
// 1. Reference names MUST be valid (use Reference::is_valid_name() check)
// 2. Bare repos don't have working directory - don't use checkout
// 3. Ref creation uses reference() method, not direct reference creation
// 4. Empty tree is created with empty slice: repo.create_tree(&[])

// PROJECT CONTEXT QUIRKS
// =====================
// 1. ProjectContext::load() returns Err(JinError::NotInitialized) if .jin/context missing
// 2. Context uses YAML format, not TOML
// 3. context.save() creates parent directories if needed
// 4. Mode is Option<String> - None means no active mode

// TEST ISOLATION QUIRKS
// =====================
// 1. JIN_DIR env var MUST be set before any JinRepo operations
// 2. Tests should use std::process::id() for unique names
// 3. TempDir auto-cleans on drop - don't manually delete
// 4. Each test should create isolated temp directory

// VALIDATION QUIRKS
// ==================
// 1. Reserved names: "default", "global", "base" (lowercase only)
// 2. Mode names: alphanumeric + underscore only (no hyphens, colons, slashes)
// 3. Empty string should fail validation
// 4. Ref paths use "refs/jin/modes/{name}" pattern

// ERROR HANDLING QUIRKS
// =====================
// 1. Use JinError::AlreadyExists for duplicate mode names
// 2. Use JinError::NotFound for missing mode names
// 3. Use JinError::Other for validation failures with descriptive message
// 4. Include "Create it with: jin mode create <name>" in not-found errors
```

## Implementation Blueprint

### Data Models

The mode commands use existing data models - no new types needed:

```rust
// EXISTING TYPES IN USE
// ======================

// src/core/config.rs - ProjectContext persists active mode
pub struct ProjectContext {
    pub mode: Option<String>,     // Active mode name
    pub scope: Option<String>,    // Active scope name
    pub project: Option<String>,  // Project name (auto-inferred)
    pub version: u32,
    pub last_updated: Option<String>,
}

// src/cli/mod.rs - CLI argument structure
#[derive(Subcommand, Debug)]
pub enum ModeAction {
    Create { name: String },      // jin mode create <name>
    Use { name: String },         // jin mode use <name>
    List,                         // jin mode list
    Delete { name: String },      // jin mode delete <name>
    Show,                         // jin mode show
    Unset,                        // jin mode unset
}
```

### Implementation Tasks

```yaml
# NOTE: These tasks document the EXISTING implementation for reference.

Task 1: DEFINE CLI STRUCTURE (COMPLETED)
  - LOCATION: src/cli/mod.rs
  - IMPLEMENT: ModeAction enum with #[derive(Subcommand)]
  - VARIANTS: Create { name }, Use { name }, List, Delete { name }, Show, Unset
  - INTEGRATION: Add Mode(ModeAction) variant to Commands enum
  - PATTERN: Follow ScopeAction pattern (lines 138-165)

Task 2: WIRE COMMAND EXECUTOR (COMPLETED)
  - LOCATION: src/commands/mod.rs
  - IMPLEMENT: Commands::Mode(action) => mode::execute(action) match arm
  - PATTERN: Follow Commands::Scope(action) pattern (line 39)
  - EXPORT: Add pub mod mode; to module exports

Task 3: IMPLEMENT MODE COMMAND MODULE (COMPLETED)
  - LOCATION: src/commands/mode.rs
  - FUNCTION: execute() -> entry point matching ModeAction variants
  - FUNCTIONS: create(), use_mode(), list(), delete(), show(), unset()
  - HELPER: validate_mode_name() - validates and rejects reserved names
  - DEPENDENCIES: crate::cli::ModeAction, crate::core::{JinError, ProjectContext, Result}
  - PATTERN: Follow scope.rs implementation (identical structure)

Task 4: IMPLEMENT VALIDATION FUNCTION (COMPLETED)
  - LOCATION: src/commands/mode.rs::validate_mode_name()
  - CHECKS: Empty string, invalid characters, reserved names
  - RESERVED: ["default", "global", "base"]
  - VALID CHARS: Alphanumeric + underscore only (c.is_alphanumeric() || c == '_')
  - ERROR: JinError::Other with descriptive message

Task 5: IMPLEMENT CREATE SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/mode.rs::create()
  - FLOW: Validate name -> Open repo -> Check existence -> Create empty tree -> Create commit -> Set ref
  - REF PATH: format!("refs/jin/modes/{}", name)
  - SUCCESS MESSAGE: "Created mode '{name}'\nActivate with: jin mode use {name}"

Task 6: IMPLEMENT USE SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/mode.rs::use_mode()
  - FLOW: Validate name -> Open repo -> Check ref exists -> Load context -> Update mode -> Save context
  - CONTEXT UPDATE: context.mode = Some(name.to_string())
  - SUCCESS MESSAGE: "Activated mode '{name}'\nStage files with: jin add --mode"

Task 7: IMPLEMENT LIST SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/mode.rs::list()
  - FLOW: Open repo -> Load context -> List refs matching "refs/jin/modes/*" -> Display with active marker
  - PATTERN: "* {name} [active]" for active, "  {name}" for inactive
  - EMPTY MESSAGE: "No modes found.\nCreate one with: jin mode create <name>"

Task 8: IMPLEMENT DELETE SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/mode.rs::delete()
  - FLOW: Validate name -> Open repo -> Check ref exists -> Load context -> Unset if active -> Delete ref -> Cleanup layer refs
  - CLEANUP PATTERNS: refs/jin/layers/mode/{name}, refs/jin/modes/{name}/scopes/*
  - SUCCESS MESSAGE: "Deleted mode '{name}'"

Task 9: IMPLEMENT SHOW SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/mode.rs::show()
  - FLOW: Load context -> Display mode or "No active mode"
  - OUTPUT: "Active mode: {name}" or "No active mode"

Task 10: IMPLEMENT UNSET SUBCOMMAND (COMPLETED)
  - LOCATION: src/commands/mode.rs::unset()
  - FLOW: Load context -> Check mode is set -> Set mode to None -> Save context
  - SUCCESS MESSAGE: "Deactivated mode\nMode layer no longer available for staging"

Task 11: ADD UNIT TESTS (COMPLETED)
  - LOCATION: src/commands/mode.rs tests module
  - FIXTURE: setup_test_env() - creates TempDir, sets JIN_DIR, creates .jin/context
  - TESTS: test_validate_mode_name_*() - validation edge cases
  - TESTS: test_create_mode(), test_create_mode_duplicate()
  - TESTS: test_use_mode(), test_use_mode_nonexistent()
  - TESTS: test_list_empty(), test_list_with_modes()
  - TESTS: test_delete_mode(), test_delete_active_mode(), test_delete_nonexistent()
  - TESTS: test_show_no_mode(), test_show_with_mode()
  - TESTS: test_unset(), test_unset_no_mode()

Task 12: VERIFY INTEGRATION TESTS (COMPLETED)
  - LOCATION: tests/cli_basic.rs, tests/mode_scope_workflow.rs
  - TESTS: test_mode_*_subcommand() functions
  - PATTERN: Use jin().args(["mode", "<subcommand>", ...]).assert()
  - ISOLATION: Set JIN_DIR env variable for each test
```

### Implementation Patterns & Key Details

```rust
// ============================================
// CRITICAL PATTERNS FROM EXISTING CODEBASE
// ============================================

// PATTERN 1: Command Entry Point
// ===============================
// File: src/commands/mode.rs:8-17
pub fn execute(action: ModeAction) -> Result<()> {
    match action {
        ModeAction::Create { name } => create(&name),
        ModeAction::Use { name } => use_mode(&name),
        ModeAction::List => list(),
        ModeAction::Delete { name } => delete(&name),
        ModeAction::Show => show(),
        ModeAction::Unset => unset(),
    }
}

// PATTERN 2: Name Validation
// ===========================
// File: src/commands/mode.rs:25-49
fn validate_mode_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(JinError::Other("Mode name cannot be empty".to_string()));
    }

    // Check for valid characters (alphanumeric and underscore only)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(JinError::Other(format!(
            "Invalid mode name '{}'. Use alphanumeric characters and underscores only.",
            name
        )));
    }

    // Check for reserved names
    let reserved = ["default", "global", "base"];
    if reserved.contains(&name) {
        return Err(JinError::Other(format!(
            "Mode name '{}' is reserved.",
            name
        )));
    }

    Ok(())
}

// PATTERN 3: Create Mode with Initial Commit
// ===========================================
// File: src/commands/mode.rs:52-82
fn create(name: &str) -> Result<()> {
    validate_mode_name(name)?;

    let repo = JinRepo::open_or_create()?;

    // Check if mode already exists
    let ref_path = format!("refs/jin/modes/{}", name);
    if repo.ref_exists(&ref_path) {
        return Err(JinError::AlreadyExists(format!(
            "Mode '{}' already exists",
            name
        )));
    }

    // Create empty tree for initial commit
    let empty_tree = repo.create_tree(&[])?;

    // Create initial commit
    let commit_oid =
        repo.create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])?;

    // Set Git ref
    repo.set_ref(&ref_path, commit_oid, &format!("create mode {}", name))?;

    println!("Created mode '{}'", name);
    println!("Activate with: jin mode use {}", name);

    Ok(())
}

// PATTERN 4: Use Mode (Activate)
// ===============================
// File: src/commands/mode.rs:85-120
fn use_mode(name: &str) -> Result<()> {
    validate_mode_name(name)?;

    let repo = JinRepo::open_or_create()?;

    // Check if mode exists
    let ref_path = format!("refs/jin/modes/{}", name);
    if !repo.ref_exists(&ref_path) {
        return Err(JinError::NotFound(format!(
            "Mode '{}' not found. Create it with: jin mode create {}",
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

    // Update mode
    context.mode = Some(name.to_string());

    // Save context
    context.save()?;

    println!("Activated mode '{}'", name);
    println!("Stage files with: jin add --mode");

    Ok(())
}

// PATTERN 5: List Modes with Active Indicator
// ============================================
// File: src/commands/mode.rs:123-161
fn list() -> Result<()> {
    let repo = JinRepo::open_or_create()?;

    // Load project context to identify active mode
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Find all mode refs
    let mode_refs = repo.list_refs("refs/jin/modes/*")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        println!("Create one with: jin mode create <name>");
        return Ok(());
    }

    println!("Available modes:");

    // Extract names and display with active indicator
    for ref_path in mode_refs {
        let name = ref_path
            .strip_prefix("refs/jin/modes/")
            .unwrap_or(&ref_path);

        if Some(name) == context.mode.as_deref() {
            println!("  * {} [active]", name);
        } else {
            println!("    {}", name);
        }
    }

    Ok(())
}

// PATTERN 6: Delete Mode with Cleanup
// ====================================
// File: src/commands/mode.rs:164-218
fn delete(name: &str) -> Result<()> {
    validate_mode_name(name)?;

    let repo = JinRepo::open_or_create()?;

    // Check if mode exists
    let ref_path = format!("refs/jin/modes/{}", name);
    if !repo.ref_exists(&ref_path) {
        return Err(JinError::NotFound(format!("Mode '{}' not found", name)));
    }

    // Load project context to check if active
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // If mode is active, unset it first
    if Some(name) == context.mode.as_deref() {
        println!("Mode '{}' is currently active. Deactivating...", name);
        context.mode = None;
        context.save()?;
    }

    // Delete main mode ref
    repo.delete_ref(&ref_path)?;

    // Delete associated layer refs (may not exist if no files committed)
    // Silently ignore errors as these refs may not exist yet
    let layer_patterns = [
        format!("refs/jin/layers/mode/{}", name),
        format!("refs/jin/modes/{}/scopes/*", name),
    ];

    for pattern in &layer_patterns {
        // Try to delete, ignore errors
        let _ = repo.delete_ref(pattern);

        // Also try to list and delete individual refs matching pattern
        if let Ok(refs) = repo.list_refs(pattern) {
            for ref_to_delete in refs {
                let _ = repo.delete_ref(&ref_to_delete);
            }
        }
    }

    println!("Deleted mode '{}'", name);

    Ok(())
}

// PATTERN 7: Test Isolation Fixture
// ==================================
// File: src/commands/mode.rs:273-289
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

// GOTCHA: Empty Tree Creation
// ============================
// When creating initial commit for new mode, use empty slice
let empty_tree = repo.create_tree(&[])?;

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
// Mode refs: refs/jin/modes/{name}
// Mode layer refs: refs/jin/layers/mode/{name}
// Mode scope refs: refs/jin/modes/{name}/scopes/{scope}
```

### Integration Points

```yaml
PROJECT CONTEXT:
  - file: .jin/context
  - format: YAML
  - fields: mode: Option<String>
  - load: ProjectContext::load() -> Result<ProjectContext>
  - save: context.save() -> Result<()>

GIT REFS:
  - namespace: refs/jin/modes/
  - pattern: refs/jin/modes/{mode_name}
  - layer_pattern: refs/jin/layers/mode/{mode_name}
  - create: repo.set_ref(path, oid, message)
  - check: repo.ref_exists(path)
  - delete: repo.delete_ref(path)
  - list: repo.list_refs("refs/jin/modes/*")

CLI ROUTING:
  - enum: Commands::Mode(ModeAction)
  - handler: mode::execute(action)
  - subcommands: ModeAction::{Create, Use, List, Delete, Show, Unset}

STAGING INTEGRATION:
  - add.rs: Reads context.mode to determine active mode
  - add --mode: Routes to ModeBase layer (Layer 2)
  - add --mode --project: Routes to ModeProject layer (Layer 5)
  - add --mode --scope=X: Routes to ModeScope layer (Layer 3)

STATUS INTEGRATION:
  - status.rs: Displays active mode from context
  - format: "Mode: {name} (active)" or "No active mode"
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
# Test mode command module
cargo test --package jin --lib commands::mode::tests

# Test specific functions
cargo test --package jin --lib validate_mode_name
cargo test --package jin --lib test_create_mode
cargo test --package jin --lib test_use_mode
cargo test --package jin --lib test_list
cargo test --package jin --lib test_delete_mode
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

# Mode create command
./target/release/jin mode create test_mode_$$

# Mode list command
./target/release/jin mode list

# Mode show command (should show no active mode initially)
./target/release/jin mode show

# Mode use command
./target/release/jin mode use test_mode_$$

# Mode show (should show active mode)
./target/release/jin mode show

# Mode unset command
./target/release/jin mode unset

# Mode delete command
./target/release/jin mode delete test_mode_$$

# Run integration tests
cargo test --test cli_basic test_mode
cargo test --test mode_scope_workflow

# Expected: All commands work correctly, integration tests pass.
```

### Level 4: End-to-End Workflow Validation

```bash
# Complete mode workflow test
set -e  # Exit on any error

MODE_NAME="e2e_test_$(date +%s)"

# Initialize Jin
mkdir -p /tmp/jin_e2e
cd /tmp/jin_e2e
jin init

# Create mode
jin mode create "$MODE_NAME"
echo "✓ Mode created"

# List modes (should show new mode)
jin mode list | grep "$MODE_NAME"
echo "✓ Mode appears in list"

# Use mode
jin mode use "$MODE_NAME"
echo "✓ Mode activated"

# Show mode (should show active)
jin mode show | grep "$MODE_NAME"
echo "✓ Mode shows as active"

# Verify context file
grep "mode: $MODE_NAME" .jin/context
echo "✓ Context file updated"

# Create a file and add to mode layer
echo '{"test": true}' > config.json
jin add config.json --mode
echo "✓ File staged to mode layer"

# Commit
jin commit -m "Test commit"
echo "✓ Changes committed"

# Apply changes
jin apply
echo "✓ Changes applied"

# Check status
jin status | grep "Clean"
echo "✓ Workspace is clean"

# Unset mode
jin mode unset
echo "✓ Mode deactivated"

# Delete mode
jin mode delete "$MODE_NAME"
echo "✓ Mode deleted"

# Cleanup
cd /
rm -rf /tmp/jin_e2e

echo "✅ All E2E validation checks passed!"
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --package jin --lib commands::mode::tests`
- [ ] No linting errors: `cargo clippy --all-targets`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Integration tests pass: `cargo test --test cli_basic --test mode_scope_workflow`
- [ ] Binary compiles without warnings: `cargo build --release`

### Feature Validation

- [ ] Mode create creates ref at `refs/jin/modes/{name}`
- [ ] Mode use updates `.jin/context` with mode field
- [ ] Mode list displays all modes with active indicator
- [ ] Mode delete removes mode ref and associated layer refs
- [ ] Mode show displays active mode or "No active mode"
- [ ] Mode unset clears mode from context
- [ ] Reserved names ("default", "global", "base") are rejected
- [ ] Invalid characters are rejected
- [ ] Duplicate mode names are rejected
- [ ] Using non-existent mode shows helpful error with create command

### Code Quality Validation

- [ ] Follows existing codebase patterns (matches scope.rs structure)
- [ ] Error messages are descriptive and actionable
- [ ] User guidance provided (next steps in success messages)
- [ ] Tests use isolated environments (TempDir, JIN_DIR)
- [ ] Test coverage includes all code paths
- [ ] Documentation comments present on public functions

### CLI/UX Validation

- [ ] Help text works: `jin mode --help`
- [ ] Subcommand help works: `jin mode create --help`
- [ ] Error messages are user-friendly
- [ ] Success messages include next action guidance
- [ ] Active mode clearly indicated in list output

---

## Anti-Patterns to Avoid

- ❌ **Don't** use hyphens in mode names (validation rejects them)
- ❌ **Don't** allow reserved names ("default", "global", "base")
- ❌ **Don't** forget to unset mode in context when deleting active mode
- ❌ **Don't** skip cleanup of associated layer refs on delete
- ❌ **Don't** ignore NotInitialized errors - propagate them to user
- ❌ **Don't** create mode refs outside `refs/jin/modes/` namespace
- ❌ **Don't** use ProjectContext::default() when NotInitialized should be returned
- ❌ **Don't** forget to set JIN_DIR environment variable in tests
- ❌ **Don't** use sync functions in async context (N/A - this is sync code)
- ❌ **Don't** catch all exceptions - be specific with JinError variants

## Implementation Status: COMPLETE ✅

**Note**: This PRP documents the EXISTING IMPLEMENTATION in `src/commands/mode.rs`.

All tasks described above have been completed. The mode commands are fully functional with:

1. **CLI Structure**: ModeAction enum defined in `src/cli/mod.rs`
2. **Command Wiring**: `Commands::Mode(action) => mode::execute(action)` in `src/commands/mod.rs`
3. **Implementation**: Complete in `src/commands/mode.rs` with all 6 subcommands
4. **Unit Tests**: Comprehensive test coverage in mode.rs tests module
5. **Integration Tests**: Tests in `tests/cli_basic.rs` and `tests/mode_scope_workflow.rs`

**Confidence Score**: 10/10 - Implementation is complete, tested, and functional.

**For reference**: Use this PRP to understand the patterns for implementing similar commands (e.g., scope commands follow identical structure).
