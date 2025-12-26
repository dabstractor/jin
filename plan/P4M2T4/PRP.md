name: "PRP for P4.M2.T4: Status Command"
description: |

---

## Goal

**Feature Goal**: Implement `jin status` command that displays workspace and staging state, including active mode/scope context, staged files, and layer commit information.

**Deliverable**: A fully functional `jin status` command that:
- Displays active mode/scope from `.jin/context`
- Shows all staged files grouped by layer
- Lists layers with commits (from Git refs)
- Indicates workspace cleanliness
- Follows git status output conventions

**Success Definition**:
- Running `jin status` displays organized status information
- Shows all staged entries from `.jin/staging/index.json`
- Lists active context (mode/scope) if set
- Displays layer refs from Git repository
- Returns appropriate exit codes
- Output format is consistent with existing commands

## User Persona

**Target User**: Developer using Jin for multi-layer configuration management

**Use Case**: Developer wants to see current state of their Jin workspace before committing or making changes

**User Journey**:
1. Developer runs `jin status` in their project directory
2. Command displays active context (mode/scope if set)
3. Command shows staged files grouped by layer
4. Command displays layers with their commit status
5. Developer understands what's staged and what's ready to commit

**Pain Points Addressed**:
- Without status command, developers must manually inspect `.jin/staging/index.json`
- No visibility into which files are staged for which layers
- No easy way to see active mode/scope context
- No visibility into layer commit state

## Why

- **Core functionality**: Status command is fundamental to any version control system
- **Developer workflow**: Users need visibility into workspace state before committing
- **Debugging aid**: Helps understand what's staged and where it will be committed
- **Completeness**: Completes the core add/commit/status workflow
- **Low complexity**: Uses existing infrastructure (StagingIndex, ProjectContext, JinRepo)

## What

### User-Visible Behavior

```
$ jin status

Active mode: claude
Active scope: language:javascript

Staged files (3):
  mode/claude:
    config/ai/settings.json
  mode/claude/scope/language:javascript:
    .eslintrc.js
    tsconfig.json

Layers with commits:
  global: <oid>
  mode/claude: <oid>
  mode/claude/scope/language:javascript: <oid>

Workspace: clean
```

Or when nothing is staged:
```
$ jin status

No active mode or scope

No files staged

No layers with commits

Workspace: clean
```

### Success Criteria

- [ ] Displays active mode from `.jin/context` if set
- [ ] Displays active scope from `.jin/context` if set
- [ ] Lists all staged files grouped by layer
- [ ] Shows layer refs from Git repository
- [ ] Returns ExitCode::SUCCESS on success
- [ ] Returns appropriate error for missing Jin initialization
- [ ] Follows output format patterns from init/add/commit commands

## All Needed Context

### Context Completeness Check

This PRP passes the "No Prior Knowledge" test - an AI agent unfamiliar with the codebase has:
- Exact file paths and line numbers for all relevant code
- Complete source code for reference implementations
- Specific API signatures and return types
- Error handling patterns
- Output format conventions
- Testing patterns to follow

### Documentation & References

```yaml
# MUST READ - Critical implementation files

- file: src/commands/init.rs
  why: Reference for command structure, idempotency pattern, output formatting
  pattern: Module docstring, execute() function signature, println! usage
  gotcha: Check for existing Jin initialization using ProjectContext::context_path()

- file: src/commands/add.rs
  why: Reference for loading ProjectContext and StagingIndex, layer grouping output
  pattern: ProjectContext::load(), StagingIndex::load_from_disk(), summary output grouping
  gotcha: Use unwrap_or_else(|_| StagingIndex::new()) for missing staging index

- file: src/commands/commit.rs
  why: Reference for JinRepo usage, layer ref listing, project name detection
  pattern: JinRepo::open_or_create(), detect_project_name(), list_layer_refs()
  gotcha: Use git2::Repository::discover() to check for Git repo

- file: src/commands/mod.rs
  why: Module export pattern for new status command
  pattern: pub mod status; pub use status::execute as status_execute;
  placement: Add status module and export alongside init, add, commit

- file: src/cli/args.rs:274-276
  why: StatusCommand struct definition (already exists as unit struct)
  pattern: #[derive(clap::Args)] pub struct StatusCommand;
  gotcha: No fields to add, struct is complete

- file: src/main.rs:39-42
  why: Current status command routing (placeholder)
  pattern: Commands::Status(cmd) => match commands::status_execute(&cmd) { ... }
  action: Replace placeholder with actual command handler call

- file: src/staging/index.rs:45-57
  why: StagingIndex structure and query methods
  pattern: all_entries(), entries_by_layer(), len(), is_empty()
  gotcha: Entries have relative paths (after normalization by commit command)

- file: src/staging/entry.rs:131-148
  why: StagedEntry structure with path, layer, status fields
  pattern: entry.path, entry.layer, entry.is_staged()
  gotcha: Path is relative to workspace root

- file: src/core/config.rs:164-188
  why: ProjectContext structure for mode/scope
  pattern: ProjectContext::load(), context.mode, context.scope
  gotcha: Returns default (mode/scope = None) if file doesn't exist

- file: src/git/repo.rs:375-405
  why: JinRepo methods for listing layer refs
  pattern: list_layer_refs() -> Result<Vec<(Layer, Oid)>>
  gotcha: Only returns versioned layers (1-7), UserLocal/WorkspaceActive not included

# EXTERNAL REFERENCES

- url: https://git-scm.com/docs/git-status
  why: Git status output format conventions
  critical: Section ordering (context -> staged -> layers -> workspace), indentation patterns
```

### Current Codebase Tree

```bash
jin-glm-doover/
├── src/
│   ├── cli/
│   │   └── args.rs              # CLI argument definitions (StatusCommand at line 274)
│   ├── commands/
│   │   ├── mod.rs               # Command module exports
│   │   ├── init.rs              # Init command (reference pattern)
│   │   ├── add.rs               # Add command (reference pattern)
│   │   └── commit.rs            # Commit command (reference pattern)
│   ├── core/
│   │   ├── config.rs            # ProjectContext, JinConfig definitions
│   │   ├── error.rs             # JinError enum, Result type alias
│   │   └── layer.rs             # Layer enum definition
│   ├── git/
│   │   └── repo.rs              # JinRepo with list_layer_refs() method
│   ├── staging/
│   │   ├── index.rs             # StagingIndex with all_entries(), entries_by_layer()
│   │   └── entry.rs             # StagedEntry with path, layer, status
│   └── main.rs                  # CLI entry point with command routing
└── Cargo.toml
```

### Desired Codebase Tree (Files to Add)

```bash
jin-glm-doover/
├── src/
│   ├── commands/
│   │   ├── mod.rs               # MODIFY: Add `pub mod status;` and export
│   │   └── status.rs            # CREATE: New status command implementation
│   └── main.rs                  # MODIFY: Update status command routing
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: StagingIndex::load_from_disk() returns default if file doesn't exist
// Use: let staging = StagingIndex::load_from_disk(&workspace_root).unwrap_or_else(|_| StagingIndex::new());

// CRITICAL: ProjectContext::load() returns default if .jin/context doesn't exist
// Always check mode/scope are Some() before displaying

// CRITICAL: JinRepo::list_layer_refs() only returns versioned layers (1-7)
// UserLocal and WorkspaceActive are NEVER in the list

// CRITICAL: Git repo may not exist (Jin can work without Git)
// Use git2::Repository::discover() and handle error gracefully

// CRITICAL: Workspace root detection
// Use std::env::current_dir() consistently (like other commands)

// CRITICAL: Path handling in StagedEntry
// After commit normalizes to relative, status displays relative paths

// CRITICAL: Error handling pattern from init.rs
// Check for Jin initialization first: ProjectContext::context_path(&workspace_root).exists()

// PATTERN: Output format from other commands
// - Use println! for user output (not eprintln! unless error)
// - Group related output with blank lines
// - Use indentation (two spaces) for related items
// - End with success message

// PATTERN: Test structure from other commands
// - Use tempfile::TempDir for test isolation
// - Create DirGuard for current directory restoration
// - Test both success and error cases
// - Use descriptive test names
```

## Implementation Blueprint

### Data Models (No New Models Required)

This implementation uses existing data structures:
- `StagingIndex`: Load from `.jin/staging/index.json`
- `StagedEntry`: Iterate via `all_entries()` and `entries_by_layer()`
- `ProjectContext`: Load from `.jin/context` for mode/scope
- `JinRepo`: Query for layer refs via `list_layer_refs()`
- `Layer`: Display formatting already implemented via Display trait

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: CREATE src/commands/status.rs
  - IMPLEMENT: Status command module with execute() function
  - SIGNATURE: pub fn execute(_cmd: &StatusCommand) -> Result<()>
  - FOLLOW pattern: src/commands/init.rs (module structure, documentation, error handling)
  - NAMING: Module docstring, function docs with examples
  - PLACEMENT: New file in src/commands/

Task 2: IMPLEMENT status::execute() main logic
  - LOAD: workspace_root = std::env::current_dir()?
  - CHECK: Jin initialization (ProjectContext::context_path exists)
  - LOAD: ProjectContext for mode/scope display
  - LOAD: StagingIndex for staged files
  - LOAD: Git repository (optional - may not exist)
  - LOAD: JinRepo for layer refs (if Git exists)
  - HANDLE: All errors with appropriate JinError variants

Task 3: IMPLEMENT display_active_context()
  - DISPLAY: "Active mode: {mode}" if mode is Some()
  - DISPLAY: "Active scope: {scope}" if scope is Some()
  - DISPLAY: "No active mode or scope" if both are None
  - PATTERN: Conditional display based on Option<> values

Task 4: IMPLEMENT display_staged_files()
  - GROUP: Staged entries by layer using staging.entries_by_layer()
  - DISPLAY: "Staged files ({count}):" header
  - DISPLAY: Layer name as group header
  - DISPLAY: Indented file paths under each layer
  - PATTERN: Two-space indentation, blank line after section

Task 5: IMPLEMENT display_layer_refs()
  - QUERY: repo.list_layer_refs()? for all (Layer, Oid) pairs
  - DISPLAY: "Layers with commits:" header
  - DISPLAY: "{layer}: {oid}" for each layer
  - DISPLAY: "No layers with commits" if empty
  - PATTERN: Format layer using Display trait

Task 6: IMPLEMENT workspace cleanliness check
  - DETERMINE: clean if staging.is_empty()
  - DISPLAY: "Workspace: clean" or "Workspace: dirty"
  - PATTERN: Simple status message

Task 7: MODIFY src/commands/mod.rs
  - ADD: pub mod status;
  - ADD: pub use status::execute as status_execute;
  - PRESERVE: Existing module exports
  - PATTERN: Follow existing add/commit/init export pattern

Task 8: MODIFY src/main.rs
  - FIND: Commands::Status(cmd) match arm (lines 39-42)
  - REPLACE: Placeholder with commands::status_execute(&cmd) call
  - PRESERVE: Error handling pattern (Ok -> SUCCESS, Err -> FAILURE with eprintln!)
  - PATTERN: Match existing command routing pattern

Task 9: CREATE test suite in status.rs
  - IMPLEMENT: test_status_shows_active_context()
  - IMPLEMENT: test_status_shows_staged_files_by_layer()
  - IMPLEMENT: test_status_shows_layer_refs()
  - IMPLEMENT: test_status_no_jin_initialized_error()
  - IMPLEMENT: test_status_empty_staging()
  - PATTERN: Follow test structure from init.rs (tempdir, DirGuard, helper functions)

Task 10: RUN validation
  - COMPILE: cargo build
  - TEST: cargo test --package jin-glm --lib commands::status
  - MANUAL: cargo run -- status (in test project)
  - VERIFY: Output format matches specification
```

### Implementation Patterns & Key Details

```rust
// ===== MODULE STRUCTURE (from init.rs) =====
//! Status command implementation.
//!
//! This module implements the `jin status` command that displays
//! workspace and staging state including active context, staged files,
//! and layer commit information.

use crate::cli::args::StatusCommand;
use crate::core::config::ProjectContext;
use crate::core::error::Result;
use crate::git::JinRepo;
use crate::staging::index::StagingIndex;
use std::path::Path;

// ===== MAIN EXECUTE FUNCTION =====
/// Execute the status command.
///
/// Displays comprehensive workspace status including:
/// - Active mode/scope from `.jin/context`
/// - Staged files grouped by layer
/// - Layer refs from Git repository
/// - Workspace cleanliness state
///
/// # Arguments
///
/// * `_cmd` - The status command (currently has no fields)
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if not in a Git repository.
/// Returns `JinError::Io` if context/staging files cannot be read.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::StatusCommand;
/// use jin_glm::commands::status;
///
/// let cmd = StatusCommand;
/// status::execute(&cmd)?;
/// ```
pub fn execute(_cmd: &StatusCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize.".to_string(),
        ));
    }

    // 3. Load and display active context
    let context = ProjectContext::load(&workspace_root)?;
    display_active_context(&context);

    // 4. Load and display staged files
    let staging = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());
    display_staged_files(&staging);

    // 5. Load and display layer refs (if Git exists)
    if let Ok(_git_repo) = git2::Repository::discover(&workspace_root) {
        if let Ok(repo) = JinRepo::open_or_create(&workspace_root) {
            display_layer_refs(&repo)?;
        }
    }

    // 6. Display workspace status
    display_workspace_status(&staging);

    Ok(())
}

// ===== DISPLAY HELPERS =====

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

fn display_staged_files(staging: &StagingIndex) {
    println!();

    if staging.is_empty() {
        println!("No files staged");
        return;
    }

    // Group by layer
    let mut layers: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for entry in staging.all_entries() {
        let layer_name = format!("{}", entry.layer);
        let path = entry.path.display().to_string();
        layers.entry(layer_name).or_default().push(path);
    }

    println!("Staged files ({}):", staging.len());
    for (layer, files) in layers {
        println!("  {}:", layer);
        for file in files {
            println!("    {}", file);
        }
    }
}

fn display_layer_refs(repo: &JinRepo) -> Result<()> {
    println!();

    let refs = repo.list_layer_refs()?;

    if refs.is_empty() {
        println!("No layers with commits");
        return Ok(());
    }

    println!("Layers with commits:");
    for (layer, oid) in refs {
        println!("  {}: {}", layer, oid);
    }

    Ok(())
}

fn display_workspace_status(staging: &StagingIndex) {
    println!();
    if staging.is_empty() {
        println!("Workspace: clean");
    } else {
        println!("Workspace: dirty");
    }
}

// ===== GOTCHAS TO HANDLE =====

// 1. StagingIndex may not exist - use unwrap_or_else
let staging = StagingIndex::load_from_disk(&workspace_root)
    .unwrap_or_else(|_| StagingIndex::new());

// 2. Git repo may not exist - handle gracefully
if let Ok(_git_repo) = git2::Repository::discover(&workspace_root) {
    // Only query JinRepo if Git exists
}

// 3. JinRepo::open_or_create may fail - wrap in if let Ok()
if let Ok(repo) = JinRepo::open_or_create(&workspace_root) {
    display_layer_refs(&repo)?;
}

// 4. Check Jin initialization FIRST (like init.rs does)
let context_path = ProjectContext::context_path(&workspace_root);
if !context_path.exists() {
    return Err(JinError::Message("Jin is not initialized...".to_string()));
}

// 5. Output formatting: use println! not eprintln! for status
// eprintln! is ONLY for errors (see main.rs pattern)
```

### Integration Points

```yaml
COMMAND_MODULE:
  - file: src/commands/mod.rs
  - add: pub mod status;
  - add: pub use status::execute as status_execute;
  - pattern: Match existing module exports

MAIN_ROUTING:
  - file: src/main.rs
  - section: Commands::Status(cmd) match arm (~lines 39-42)
  - action: Replace placeholder with commands::status_execute(&cmd)
  - pattern: Match existing command routing with Ok/Err handling

STAGING_INDEX:
  - load: StagingIndex::load_from_disk(&workspace_root)
  - handle: Missing file returns Ok(default), use unwrap_or_else
  - methods: all_entries(), entries_by_layer(), len(), is_empty()

PROJECT_CONTEXT:
  - load: ProjectContext::load(&workspace_root)
  - handle: Missing file returns Ok(default)
  - fields: context.mode (Option<String>), context.scope (Option<String>)

JINREPO:
  - open: JinRepo::open_or_create(&workspace_root)
  - query: list_layer_refs() -> Result<Vec<(Layer, Oid)>>
  - optional: Only query if git2::Repository::discover() succeeds
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating src/commands/status.rs
cargo check --package jin-glm 2>&1 | head -50

# Fix any compilation errors before proceeding
# Expected: Compiles successfully with warnings about unused code (until integrated)
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run status command tests
cargo test --package jin-glm --lib commands::status -- --nocapture

# Expected: All tests pass
# Test coverage should include:
# - Active context display (mode, scope, both, neither)
# - Staged files grouping by layer
# - Layer refs display
# - Empty staging state
# - Missing Jin initialization error
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the project
cargo build --release

# Initialize a test project
cd /tmp
mkdir test-jin-status
cd test-jin-status
git init
cargo run -- init

# Test: No staging, clean workspace
cargo run -- status
# Expected: Shows "No files staged", "No layers with commits", "Workspace: clean"

# Test: Add files and stage them
echo "test" > config.json
cargo run -- add config.json --project
cargo run -- status
# Expected: Shows "Active mode: None", "Staged files (1):", "  project/test-jin-status:", "    config.json"

# Test: Commit and check status
cargo run -- commit -m "Add config"
cargo run -- status
# Expected: Shows "No files staged", but shows "Layers with commits:", "  project/test-jin-status: <oid>"

# Test: Set mode and check status
# (Manually edit .jin/context to set mode: claude)
cargo run -- status
# Expected: Shows "Active mode: claude"
```

### Level 4: Manual Validation

```bash
# Verify output format matches specification
cargo run -- status | tee /tmp/status-output.txt

# Check for:
# - [ ] Active mode/scope line (if set)
# - [ ] Staged files count
# - [ ] Files grouped by layer with proper indentation
# - [ ] Layer refs listed (if any exist)
# - [ ] Workspace cleanliness status

# Test error handling
cd /tmp
mkdir no-jin-here
cd no-jin-here
cargo run -- status
# Expected: Error message "Jin is not initialized..."
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo check --package jin-glm`
- [ ] All tests pass: `cargo test --package jin-glm --lib`
- [ ] No linting warnings: `cargo clippy --package jin-glm`
- [ ] Manual testing successful (see Level 3 commands)
- [ ] Error cases handled gracefully:
  - [ ] Not initialized (no .jin/context)
  - [ ] Missing staging index (creates empty)
  - [ ] No Git repository (skips layer refs)
  - [ ] Empty staging state

### Feature Validation

- [ ] Shows active mode when set
- [ ] Shows active scope when set
- [ ] Shows "No active mode or scope" when neither set
- [ ] Lists staged files grouped by layer
- [ ] Shows file count in staged section
- [ ] Shows layer refs with commit OIDs
- [ ] Indicates workspace cleanliness (clean/dirty)
- [ ] Returns ExitCode::SUCCESS on success
- [ ] Returns ExitCode::FAILURE on error

### Code Quality Validation

- [ ] Follows existing command patterns (init, add, commit)
- [ ] Module docstring present
- [ ] Function docstring with examples present
- [ ] Error handling uses appropriate JinError variants
- [ ] Output uses println! (not eprintln! for status)
- [ ] Proper indentation (two spaces for grouped items)
- [ ] Blank lines between sections for readability

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Comments explain "why" not "what"
- [ ] Test names are descriptive
- [ ] Integration with main.rs complete
- [ ] Module export in mod.rs complete

---

## Anti-Patterns to Avoid

- [ ] Don't create new data structures - use existing StagingIndex, ProjectContext, JinRepo
- [ ] Don't hardcode layer names - use Layer::Display trait
- [ ] Don't assume Git exists - check with Repository::discover()
- [ ] Don't panic on errors - use Result<> and JinError
- [ ] Don't use eprintln! for status output (only for errors)
- [ ] Don't skip Jin initialization check
- [ ] Don't forget to handle empty staging/empty refs
- [ ] Don't create unnecessary helper functions (keep it simple)
- [ ] Don't add CLI flags - StatusCommand is a unit struct with no fields
- [ ] Don't implement workspace file scanning - only show staged files

## Confidence Score

**8/10** - High confidence for one-pass implementation success

**Reasoning**:
- All dependencies are already implemented and tested
- Clear reference patterns from init/add/commit commands
- Simple data flow (load -> display)
- No complex algorithms or state mutations
- Well-defined output format
- Only ~150 lines of code estimated

**Risks**:
- Git repository interaction (mitigated: optional, graceful handling)
- Layer display formatting (mitigated: Display trait already implemented)
- Test setup complexity (mitigated: clear patterns from existing tests)
