name: "P4.M3.T1: Mode Commands Implementation"
description: |

---

## Goal

**Feature Goal**: Implement mode management commands (create/use/list/delete/show/unset) enabling developers to create, activate, and manage AI/editor-specific configuration layers.

**Deliverable**: Working `jin mode <subcommand>` commands with full Git ref management, context file integration, and user-friendly output.

**Success Definition**:
- `jin mode create <name>` creates a mode Git ref
- `jin mode use <name>` sets active mode in `.jin/context`
- `jin mode unset` clears active mode from context
- `jin mode delete <name>` removes mode ref and data
- `jin mode show` displays current active mode
- `jin modes` lists all available modes with their commit OIDs
- All commands handle edge cases and provide clear error messages

## User Persona

**Target User**: Developer working with multiple AI tools (Claude, Cursor, Zed, etc.) who needs to maintain separate configuration layers for each tool.

**Use Case**: Developer wants to switch between "claude" mode for AI-assisted Rust development and "cursor" mode for TypeScript work, maintaining different configuration files for each environment.

**User Journey**:
1. Developer creates a new mode: `jin mode create claude`
2. Developer activates the mode: `jin mode use claude`
3. Developer adds config files: `jin add .clangd --mode`
4. Developer commits mode-specific configs: `jin commit -m "Add clangd config"`
5. Developer switches modes: `jin mode use cursor`
6. Developer views available modes: `jin modes`
7. Developer checks active mode: `jin mode show`
8. Developer removes unused mode: `jin mode delete old-mode`

**Pain Points Addressed**:
- Manual configuration switching between tools
- Risk of contaminating project Git with tool-specific files
- Difficulty maintaining separate configs for different AI tools
- No clear visibility into which mode is active

## Why

- **Business value**: Enables developers to maintain clean separation between tool configurations while keeping everything versioned
- **Integration with existing features**: Modes integrate with the 9-layer hierarchy, scope commands, and workspace merge system
- **Problems solved**:
  - Eliminates manual config file swapping when switching AI tools
  - Provides git-tracked history of mode-specific configuration changes
  - Prevents tool-specific files from polluting the main project repository

## What

### User-Visible Behavior

**Mode Create (`jin mode create <name>`)**
- Creates a new mode by initializing a Git ref at `refs/jin/layers/mode/<name>`
- Creates an empty initial commit for the mode
- Fails if mode already exists
- No workspace merge occurs on creation
- Output: `Mode '<name>' created.`

**Mode Use (`jin mode use <name>`)**
- Sets the active mode in `.jin/context` YAML file
- Creates `.jin/context` if it doesn't exist
- Preserves existing scope if set
- Fails if mode doesn't exist (check Git ref)
- Output: `Mode '<name>' is now active.`

**Mode Unset (`jin mode unset`)**
- Removes mode field from `.jin/context`
- Preserves scope field if present
- Keeps `.jin/context` file with remaining data
- No error if no mode is set
- Output: `Mode deactivated.`

**Mode Delete (`jin mode delete <name>`)**
- Deletes the Git ref at `refs/jin/layers/mode/<name>`
- Fails if mode doesn't exist
- Fails if mode is currently active (require `jin mode unset` first)
- Output: `Mode '<name>' deleted.`

**Mode Show (`jin mode show`)**
- Displays current active mode from `.jin/context`
- Shows scope if also active
- Output: `Active mode: <name>` or `No active mode`

**Modes List (`jin modes`)**
- Lists all mode Git refs in the repository
- Shows mode name and commit OID (short)
- Output format:
  ```
  Available modes:
    claude      abc1234
    cursor      def5678
    zed         ghi9012
  ```
- Or: `No modes found.`

### Success Criteria

- [ ] All mode subcommands execute without panics
- [ ] Mode refs are created/deleted correctly in Git
- [ ] `.jin/context` file is created/updated properly
- [ ] Error messages are clear and actionable
- [ ] Commands handle missing refs gracefully
- [ ] Delete command refuses to delete active mode
- [ ] List command shows all available modes

## All Needed Context

### Context Completeness Check

This PRP provides complete context for implementation. An AI agent unfamiliar with the codebase can implement mode commands successfully using only this PRP and codebase access.

### Documentation & References

```yaml
# MUST READ - CLI Command Pattern Reference
- file: src/commands/add.rs
  why: Complete command execution pattern with context loading, layer routing, staging integration, error handling, and comprehensive tests
  pattern: Command execute() function signature, workspace root detection, context loading, validation, staging operations, user output
  gotcha: Always use std::env::current_dir() for workspace root, not hardcoded paths

- file: src/commands/init.rs
  why: Simple command pattern with idempotent operations and directory creation
  pattern: Basic command structure with Result<()> return, directory creation with create_dir_all
  gotcha: Use create_dir_all for idempotent directory creation

- file: src/commands/status.rs
  why: Display command pattern for showing context and state information
  pattern: Context loading, display formatting, conditional output
  gotcha: Handle missing context gracefully with default/empty states

# MUST READ - CLI Argument Definitions
- file: src/cli/args.rs
  why: Complete ModeCommand enum definition with all subcommands (Create, Use, Unset, Delete, Show)
  pattern: clap derive Subcommand macro, nested enum structure
  section: Lines 132-163 for ModeCommand enum definition
  gotcha: Modes is a separate Commands variant, not part of ModeCommand enum

# MUST READ - Context and Configuration
- file: src/core/config.rs
  why: ProjectContext struct with load(), save(), set_mode(), set_mode(None) for unset operations
  pattern: YAML serialization with serde_yaml_ng, file I/O with error handling
  section: Lines 200-363 for ProjectContext implementation
  gotcha: save() creates parent directories automatically, set_mode(None) clears the mode field

# MUST READ - Layer System
- file: src/core/layer.rs
  why: Layer enum with ModeBase variant, storage_path() and git_ref() methods for mode layer routing
  pattern: Layer::ModeBase { mode: String } variant, git_ref() returns Some("refs/jin/layers/mode/<name>")
  section: Lines 42-120 for Layer enum definition, lines 216-280 for git_ref() implementation
  gotcha: Mode Git ref format is "refs/jin/layers/mode/<name>" (not "refs/modes/<name>")

# MUST READ - Git Repository Operations
- file: src/git/repo.rs
  why: JinRepo wrapper with reference_exists(), create_ref(), delete_ref(), find_jin_refs() operations
  pattern: Ref operations using git2::Repository, error handling with JinError
  gotcha: Always check ref_exists before create, use find_jin_refs("refs/jin/layers/mode/") for listing

# MUST READ - Main Command Dispatch
- file: src/main.rs
  why: Command routing pattern showing how ModeCommand variants are dispatched to execute functions
  pattern: match on Commands::Mode(ModeCommand::*), call execute() with proper error handling
  gotcha: ModeCommand variants are nested inside Commands::Mode(), need pattern matching to extract

# MUST READ - Command Module Export Pattern
- file: src/commands/mod.rs
  why: Shows how to export new command execute functions for use in main.rs
  pattern: pub use module::execute as module_execute naming convention
  gotcha: Must add mod mode; and pub use mode::execute as mode_execute; after creating mode.rs
```

### Current Codebase Tree

```bash
jin-glm-doover/
├── Cargo.toml
├── src/
│   ├── main.rs              # Command dispatch - needs mode routing added
│   ├── cli/
│   │   ├── mod.rs           # CLI exports
│   │   └── args.rs          # ModeCommand enum already defined
│   ├── commands/
│   │   ├── mod.rs           # Command exports - needs mode added
│   │   ├── init.rs          # Simple command pattern
│   │   ├── add.rs           # Complex command pattern (reference)
│   │   ├── commit.rs        # Commit command
│   │   └── status.rs        # Display command pattern (reference)
│   ├── core/
│   │   ├── config.rs        # ProjectContext for mode storage
│   │   ├── layer.rs         # Layer::ModeBase for mode layer
│   │   └── error.rs         # JinError types
│   ├── git/
│   │   └── repo.rs          # JinRepo ref operations
│   └── ...
└── plan/
    └── P4M3T1/
        └── PRP.md           # This file
```

### Desired Codebase Tree (After Implementation)

```bash
jin-glm-doover/
├── src/
│   ├── main.rs              # ADD: ModeCommand routing
│   ├── commands/
│   │   ├── mod.rs           # ADD: mode module export
│   │   └── mode.rs          # NEW: Mode command implementations
│   └── ...
└── tests/
    └── mode_commands/       # NEW: Integration tests (optional)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Git ref format for modes is "refs/jin/layers/mode/<name>"
// NOT "refs/modes/<name>" or "refs/jin/modes/<name>"
// Example: refs/jin/layers/mode/claude

// CRITICAL: ModeCommand is nested inside Commands enum
// Pattern match required: Commands::Mode(ModeCommand::Create { name }) => { ... }
// NOT: Commands::ModeCreate { name } => { ... }

// CRITICAL: ProjectContext::save() creates .jin directory automatically
// No need to manually create parent directories before calling save()

// CRITICAL: ProjectContext::load() returns default context if file doesn't exist
// Always check context.mode.is_some() before using mode value

// CRITICAL: set_mode(None) clears the mode field (for unset operation)
// context.set_mode(None); then context.save(&workspace_root)?;

// CRITICAL: find_jin_refs() returns refs matching the prefix pattern
// Use "refs/jin/layers/mode/" to find all mode refs
// Strip the prefix to get mode names

// CRITICAL: git2::Oid::to_string() returns full 40-character hex
// Use oid.to_string()[..8].to_string() for short display (first 8 chars)

// CRITICAL: Always validate mode exists before operations
// Check with JinRepo::reference_exists("refs/jin/layers/mode/<name>") first

// CRITICAL: Delete should fail if mode is currently active
// Load context, check context.mode.as_ref() == Some(&mode_name) before deleting

// CRITICAL: Empty mode refs must have at least one commit
// Use JinRepo::create_empty_commit() or create initial tree/blob for new refs

// CRITICAL: Use std::env::current_dir()? for workspace root
// Don't hardcode "." or use PathBuf::from(".")
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - using existing types:
- `ProjectContext` for active mode storage (src/core/config.rs)
- `Layer::ModeBase { mode: String }` for layer representation
- `ModeCommand` enum already defined (src/cli/args.rs)

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: CREATE src/commands/mode.rs
  - IMPLEMENT: execute() function with ModeCommand pattern matching
  - IMPLEMENT: execute_create() for mode creation
  - IMPLEMENT: execute_use() for mode activation
  - IMPLEMENT: execute_unset() for mode deactivation
  - IMPLEMENT: execute_delete() for mode removal
  - IMPLEMENT: execute_show() for displaying active mode
  - IMPLEMENT: execute_list() (separate, called from main for Modes command)
  - FOLLOW pattern: src/commands/add.rs (function structure, error handling, output)
  - NAMING: execute() takes &ModeCommand, returns Result<()>
  - PLACEMENT: New file at src/commands/mode.rs

Task 2: MODIFY src/commands/mod.rs
  - ADD: mod mode; declaration
  - ADD: pub use mode::execute as mode_execute;
  - ADD: pub use mode::execute_list as mode_list_execute;
  - FOLLOW pattern: Existing exports (add_execute, commit_execute, etc.)
  - PRESERVE: All existing exports

Task 3: MODIFY src/main.rs
  - ADD: Commands::Mode(ModeCommand::Create { name }) => mode_execute(&ModeCommand::Create { name })
  - ADD: Commands::Mode(ModeCommand::Use { name }) => mode_execute(&ModeCommand::Use { name })
  - ADD: Commands::Mode(ModeCommand::Unset) => mode_execute(&ModeCommand::Unset)
  - ADD: Commands::Mode(ModeCommand::Delete { name }) => mode_execute(&ModeCommand::Delete { name })
  - ADD: Commands::Mode(ModeCommand::Show) => mode_execute(&ModeCommand::Show)
  - ADD: Commands::Modes => mode_list_execute()
  - FIND pattern: Existing command dispatch in main() function
  - PRESERVE: All existing command routing

Task 4: IMPLEMENT execute_create() in mode.rs
  - LOAD: JinConfig::load() to get repository path
  - CHECK: mode doesn't already exist (JinRepo::reference_exists)
  - CREATE: empty Git ref at refs/jin/layers/mode/<name>
  - INITIALIZE: create initial empty commit/tree for the ref
  - RETURN: Ok(()) on success, JinError::ModeAlreadyExists if exists
  - OUTPUT: println!("Mode '{}' created.", name)

Task 5: IMPLEMENT execute_use() in mode.rs
  - GET: workspace_root = std::env::current_dir()?
  - LOAD: context = ProjectContext::load(&workspace_root)
  - CHECK: mode ref exists in Git (JinRepo::reference_exists)
  - SET: context.set_mode(Some(name.to_string()))
  - SAVE: context.save(&workspace_root)?
  - OUTPUT: println!("Mode '{}' is now active.", name)

Task 6: IMPLEMENT execute_unset() in mode.rs
  - GET: workspace_root = std::env::current_dir()?
  - LOAD: context = ProjectContext::load(&workspace_root)
  - CLEAR: context.set_mode(None)
  - SAVE: context.save(&workspace_root)?
  - OUTPUT: println!("Mode deactivated.")

Task 7: IMPLEMENT execute_delete() in mode.rs
  - GET: workspace_root = std::env::current_dir()?
  - LOAD: context = ProjectContext::load(&workspace_root)
  - CHECK: mode is not currently active (context.mode.as_ref() != Some(&name))
  - CHECK: mode ref exists (JinRepo::reference_exists)
  - DELETE: Git ref using JinRepo::delete_ref
  - OUTPUT: println!("Mode '{}' deleted.", name)

Task 8: IMPLEMENT execute_show() in mode.rs
  - GET: workspace_root = std::env::current_dir()?
  - LOAD: context = ProjectContext::load(&workspace_root)
  - MATCH: on context.mode {
      Some(mode) => println!("Active mode: {}", mode),
      None => println!("No active mode"),
    }
  - OPTIONAL: Also show scope if context.has_scope()

Task 9: IMPLEMENT execute_list() in mode.rs (separate pub function)
  - LOAD: JinConfig to get repository path
  - FIND: all refs matching "refs/jin/layers/mode/" prefix
  - EXTRACT: mode names by stripping prefix
  - GET: commit OIDs for each ref
  - OUTPUT: Formatted table with mode name and short OID
  - HANDLE: empty list case with "No modes found."

Task 10: CREATE tests in mode.rs
  - IMPLEMENT: unit tests for each command function
  - IMPLEMENT: integration tests with temp directories
  - FOLLOW pattern: src/commands/add.rs tests (DirGuard, temp dir, Git repo init)
  - COVER: happy path, error cases (mode exists, not found, active mode delete)
```

### Implementation Patterns & Key Details

```rust
// ===== Command Execution Pattern =====

/// Execute mode subcommand.
///
/// Routes to the appropriate handler based on the subcommand variant.
/// Uses workspace root detection and context loading following established patterns.
pub fn execute(cmd: &ModeCommand) -> Result<()> {
    let workspace_root = std::env::current_dir()?;

    match cmd {
        ModeCommand::Create { name } => execute_create(name),
        ModeCommand::Use { name } => execute_use(&workspace_root, name),
        ModeCommand::Unset => execute_unset(&workspace_root),
        ModeCommand::Delete { name } => execute_delete(&workspace_root, name),
        ModeCommand::Show => execute_show(&workspace_root),
    }
}

// ===== Mode Create Pattern =====

fn execute_create(name: &str) -> Result<()> {
    // Validate mode name format (alphanumeric, hyphens, underscores)
    if !is_valid_mode_name(name) {
        return Err(JinError::ValidationError {
            message: format!("Invalid mode name: '{}'. Use alphanumeric, hyphens, underscores.", name),
        });
    }

    // Load Jin config to get repository path
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check mode doesn't exist
    let ref_path = format!("refs/jin/layers/mode/{}", name);
    if repo.reference_exists(&ref_path)? {
        return Err(JinError::ModeAlreadyExists { mode: name.to_string() });
    }

    // Create empty ref with initial commit
    repo.create_ref(&ref_path)?;

    println!("Mode '{}' created.", name);
    Ok(())
}

// ===== Mode Use Pattern =====

fn execute_use(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check mode exists
    let ref_path = format!("refs/jin/layers/mode/{}", name);
    if !repo.reference_exists(&ref_path)? {
        return Err(JinError::ModeNotFound { mode: name.to_string() });
    }

    // Load and update context
    let mut context = ProjectContext::load(workspace_root);
    context.set_mode(Some(name.to_string()));
    context.save(workspace_root)?;

    println!("Mode '{}' is now active.", name);
    Ok(())
}

// ===== Mode Unset Pattern =====

fn execute_unset(workspace_root: &Path) -> Result<()> {
    let mut context = ProjectContext::load(workspace_root);
    context.set_mode(None);
    context.save(workspace_root)?;

    println!("Mode deactivated.");
    Ok(())
}

// ===== Mode Delete Pattern =====

fn execute_delete(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check mode exists
    let ref_path = format!("refs/jin/layers/mode/{}", name);
    if !repo.reference_exists(&ref_path)? {
        return Err(JinError::ModeNotFound { mode: name.to_string() });
    }

    // Check mode is not active
    let context = ProjectContext::load(workspace_root);
    if context.mode.as_ref() == Some(&name.to_string()) {
        return Err(JinError::Message(
            "Cannot delete active mode. Use 'jin mode unset' first.".to_string()
        ));
    }

    // Delete the ref
    repo.delete_ref(&ref_path)?;

    println!("Mode '{}' deleted.", name);
    Ok(())
}

// ===== Mode Show Pattern =====

fn execute_show(workspace_root: &Path) -> Result<()> {
    let context = ProjectContext::load(workspace_root);

    match &context.mode {
        Some(mode) => {
            println!("Active mode: {}", mode);
            if context.has_scope() {
                println!("Active scope: {}", context.scope.as_ref().unwrap());
            }
        }
        None => {
            println!("No active mode");
            if context.has_scope() {
                println!("Active scope: {}", context.scope.as_ref().unwrap());
            }
        }
    }

    Ok(())
}

// ===== Modes List Pattern =====

pub fn execute_list() -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Find all mode refs
    let mode_refs = repo.find_jin_refs("refs/jin/layers/mode/")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        return Ok(());
    }

    println!("Available modes:");
    for (ref_name, oid) in mode_refs {
        // Extract mode name from ref path
        let mode_name = ref_name.strip_prefix("refs/jin/layers/mode/")
            .unwrap_or(&ref_name);
        let short_oid = &oid.to_string()[..8];
        println!("  {:20} {}", mode_name, short_oid);
    }

    Ok(())
}

// ===== Validation Helper =====

fn is_valid_mode_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
```

### Integration Points

```yaml
MAIN_RS_DISPATCH:
  add to: src/main.rs
  pattern: |
    Commands::Mode(ModeCommand::Create { name }) => {
        match commands::mode_execute(&ModeCommand::Create { name }) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => { eprintln!("Error: {}", e); ExitCode::FAILURE }
        }
    }
    (repeat for all ModeCommand variants)

COMMANDS_MOD:
  add to: src/commands/mod.rs
  pattern: |
    pub mod mode;
    pub use mode::execute as mode_execute;
    pub use mode::execute_list as mode_list_execute;

GIT_REF_OPERATIONS:
  - create_ref: Creates new Git ref with initial empty commit
  - delete_ref: Removes Git ref from repository
  - reference_exists: Checks if ref exists before operations
  - find_jin_refs: Lists all refs matching prefix

CONTEXT_FILE:
  - path: .jin/context (relative to workspace root)
  - format: YAML with version, mode, scope fields
  - operations: ProjectContext::load(), .save(), .set_mode()

ERROR_TYPES_TO_ADD:
  - JinError::ModeAlreadyExists { mode: String }
  - JinError::ModeNotFound { mode: String }
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --color=always                    # Check compilation
cargo clippy --color=always                   # Lint checking
cargo fmt --check                              # Format check

# Project-wide validation after all changes
cargo check --all-targets
cargo clippy --all-targets
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
# Common issues: missing imports, unused variables, type mismatches
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test mode command functions
cargo test mode::tests -- --nocapture          # Run mode.rs tests
cargo test commands::mode -- --nocapture       # Run mode command tests

# Full command tests
cargo test commands:: -- --nocapture           # Run all command tests

# Coverage validation (if cargo-tarpaulin installed)
cargo tarpaulin --out-dir target/tarpaulin --workspace

# Expected: All tests pass. If failing, debug root cause and fix implementation.
# Test patterns to verify: create/use/unset/delete/show/list all work correctly
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual integration tests in a temporary directory
cd /tmp && mkdir test-mode && cd test-mode
git init
cargo run -- init

# Test mode create
cargo run -- mode create claude
cargo run -- mode create cursor
cargo run -- mode create zed

# Test mode use
cargo run -- mode use claude
cat .jin/context  # Should show mode: claude

# Test mode show
cargo run -- mode show  # Should display "Active mode: claude"

# Test modes list
cargo run -- modes  # Should list all 3 modes

# Test mode unset
cargo run -- mode unset
cat .jin/context  # Should show mode field cleared

# Test mode delete (after unset)
cargo run -- mode delete zed
cargo run -- modes  # Should show only claude and cursor

# Test error cases
cargo run -- mode create claude  # Should fail (already exists)
cargo run -- mode use nonexistent  # Should fail (not found)
cargo run -- mode delete claude  # Should fail (still active after use)

# Expected: All operations work as specified, error messages are clear
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Git ref validation
cd ~/.jin/repo  # Or path from jin config
git show-ref --verify refs/jin/layers/mode/claude  # Should succeed after create
git show-ref | grep "refs/jin/layers/mode/"  # Should list all mode refs

# Context file validation
cat .jin/context  # Verify YAML format matches spec
# Expected output:
# version: 1
# mode: claude
# scope: language:rust  # if set

# Mode layer integration (future scope validation)
cargo run -- add .clangd --mode
cargo run -- commit -m "Add clangd"
# Should create files in jin/mode/claude/ layer

# Idempotency tests
cargo run -- mode create test-mode  # Should succeed
cargo run -- mode create test-mode  # Should fail (already exists)
cargo run -- mode unset  # Should succeed even if no mode set

# Expected: All Git refs are valid, context file is proper YAML,
# mode files route to correct layer paths
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --all`
- [ ] No compilation errors: `cargo check --all-targets`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check`

### Feature Validation

- [ ] All success criteria from "What" section met
- [ ] Manual testing successful (Level 3 commands all work)
- [ ] Error cases handled gracefully:
  - [ ] Create fails if mode exists
  - [ ] Use fails if mode doesn't exist
  - [ ] Delete fails if mode doesn't exist
  - [ ] Delete fails if mode is active
- [ ] Integration points work as specified
- [ ] `.jin/context` file format is correct

### Code Quality Validation

- [ ] Follows existing command patterns (add.rs, status.rs)
- [ ] File placement matches desired codebase tree
- [ ] Function signatures match pattern (execute(&Command) -> Result<()>)
- [ ] Error handling uses JinError types appropriately
- [ ] User output is clear and concise
- [ ] Code includes appropriate documentation comments

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Public functions have doc comments
- [ ] Error messages are user-friendly
- [ ] No hardcoded paths (use std::env::current_dir())

---

## Anti-Patterns to Avoid

- ❌ Don't create new error types when existing JinError variants work
- ❌ Don't hardcode workspace paths - always use `std::env::current_dir()?`
- ❌ Don't skip checking if mode exists before operations
- ❌ Don't allow deleting the currently active mode
- ❌ Don't forget to update src/commands/mod.rs exports
- ❌ Don't forget to add command routing in src/main.rs
- ❌ Don't use Git ref format other than "refs/jin/layers/mode/<name>"
- ❌ Don't manually create .jin directory - ProjectContext::save() handles it
- ❌ Don't panic on errors - always return Result<> with JinError
- ❌ Don't create mode refs without initial commits (empty refs cause issues)
