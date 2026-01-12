# PRP: P4.M2 - Core Commands

---

## Goal

**Feature Goal**: Implement the four core commands (`jin init`, `jin add`, `jin commit`, `jin status`) that enable the complete Jin workflow: initializing a project, staging files, committing changes atomically across layers, and viewing workspace state.

**Deliverable**:
1. `src/commands/init.rs` - Initialize Jin in a project (create `.jin/` directory, bare repo, context file)
2. `src/commands/add.rs` - **ALREADY IMPLEMENTED** - Stage files to appropriate layers
3. `src/commands/commit_cmd.rs` - Commit staged files atomically using CommitPipeline
4. `src/commands/status.rs` - Show workspace state, active context, and staged changes

**Success Definition**:
- `jin init` creates `.jin/` directory with context file and bare repository at `~/.jin/`
- `jin add <files>` stages files (already working, no changes needed)
- `jin commit -m "msg"` creates atomic commits across all affected layers
- `jin status` displays current mode/scope/project, staged files, and active layers
- All commands pass integration tests
- `cargo test` passes with zero errors
- Commands follow existing error handling and CLI patterns

---

## User Persona

**Target User**: Developer managing tool-specific configuration files across multiple contexts (modes, scopes, projects)

**Use Case**: A developer wants to separate their IDE settings for different programming languages (modes) and projects (scopes) without polluting the main Git repository.

**User Journey**:
1. Developer runs `jin init` in their project directory
2. Developer stages config files with `jin add .vscode/settings.json --mode`
3. Developer runs `jin status` to verify what will be committed
4. Developer commits with `jin commit -m "Add VS Code Python settings"`
5. Changes are atomically committed to the appropriate layer in Jin's bare repository
6. Developer can switch modes and merge different configurations

**Pain Points Addressed**:
- **No manual .jin directory creation** - `jin init` handles setup
- **Clear visibility** - `jin status` shows exactly what will be committed
- **Atomic multi-layer commits** - `jin commit` ensures consistency across layers
- **Git-like workflow** - Familiar init/add/commit/status pattern

---

## Why

- **Foundation Commands**: These four commands form the minimum viable workflow for Jin
- **User Expectations**: Developers expect Git-like semantics (init to start, add to stage, commit to persist, status to inspect)
- **PRD Requirement**: Section 18.1 specifies these as P4.M2 deliverables
- **Dependency for Later Features**: Mode/scope commands (P4.M3+) depend on these working correctly
- **Already Partial**: `add` is complete (326 lines), provides reference pattern for others

---

## What

### User-Visible Behavior

#### `jin init`
```bash
$ jin init
Initialized Jin in /home/user/project
  Created: .jin/context
  Repository: ~/.jin/

# Creates:
# - .jin/ directory in current project
# - .jin/context file (YAML) with default values
# - ~/.jin/ bare repository (if not exists)
# - Configures git to ignore .jin/ directory
```

#### `jin add <files>` (ALREADY IMPLEMENTED)
```bash
$ jin add config.json
Staged 1 file(s) to project-base layer

$ jin add .vscode/ --mode
Staged 3 file(s) to mode-base layer

# Behavior:
# - Validates files (not symlink, not Git-tracked)
# - Creates blobs in Jin repository
# - Adds to staging index (.jin/staging/index.json)
# - Updates .gitignore managed block
```

#### `jin commit -m "message"`
```bash
$ jin commit -m "Add Python IDE settings"
[mode-base abc1234] Add Python IDE settings
 3 files changed

# Behavior:
# - Groups staged files by target layer
# - Creates trees and commits per-layer
# - Atomically updates all layer refs
# - Clears staging index
# - Supports --dry-run flag

$ jin commit -m "test" --dry-run
[Dry run] Would create commits:
  mode-base: 3 files
  project-base: 1 file
```

#### `jin status`
```bash
$ jin status
Active Context:
  Mode:    python
  Scope:   (none)
  Project: my-app

Staged files (mode-base):
  new file: .vscode/settings.json
  new file: .vscode/launch.json
  new file: .vscode/tasks.json

Staged files (project-base):
  new file: config.json

# Shows:
# - Active mode/scope/project from .jin/context
# - Staged files grouped by target layer
# - Operation type (new file, modified, deleted)
```

### Success Criteria

- [ ] `jin init` creates `.jin/context` with valid YAML structure
- [ ] `jin init` creates `~/.jin/` bare repository (or uses existing)
- [ ] `jin init` errors if already initialized
- [ ] `jin add` continues working (no changes needed)
- [ ] `jin commit` creates commits for all affected layers
- [ ] `jin commit` updates refs atomically via LayerTransaction
- [ ] `jin commit` clears staging index after success
- [ ] `jin commit --dry-run` previews without creating commits
- [ ] `jin status` loads and displays active context
- [ ] `jin status` shows staged files grouped by layer
- [ ] `jin status` handles missing context gracefully
- [ ] All commands return proper exit codes (0 = success, non-zero = error)

---

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement these commands successfully?"_

**Yes** - This PRP includes:
- Complete API references from existing modules (staging, commit, git, core)
- Exact patterns from `src/commands/add.rs` (326 lines, fully implemented)
- Git2-rs documentation URLs for Repository::init(), status operations, commit creation
- Error handling patterns from JinError enum
- CLI patterns from clap derive usage
- Integration test examples from tests/cli_basic.rs

### Documentation & References

```yaml
# MUST READ - Existing Jin Module APIs

- file: src/commands/add.rs:1-326
  why: Complete reference implementation for command structure
  pattern: |
    - Load ProjectContext for active mode/scope
    - Build and validate routing options
    - Determine target layer with route_to_layer()
    - Open JinRepo with open_or_create()
    - Load StagingIndex with load() or new()
    - Process files: validate, create blobs, add to staging
    - Save staging index
    - Print summary
  gotcha: |
    - MUST check ProjectContext::load() for NotInitialized error
    - MUST validate file is not symlink (is_symlink) and not Git-tracked (is_git_tracked)
    - MUST use ensure_in_managed_block() to update .gitignore

- file: src/staging/index.rs:1-150
  why: StagingIndex API for managing staged files
  pattern: |
    StagingIndex::new() - Create empty
    StagingIndex::load() - Load from .jin/staging/index.json
    staging.add(entry) - Add StagedEntry
    staging.save() - Persist to JSON
    staging.entries_for_layer(layer) - Filter by layer
    staging.affected_layers() - Get layers with staged files (sorted by precedence)
    staging.clear() - Remove all entries
    staging.len() / is_empty() - Check status
  gotcha: |
    - StagingIndex stores at .jin/staging/index.json
    - affected_layers() returns layers sorted by precedence (1-9)
    - load() returns Err if file doesn't exist, use unwrap_or_else(StagingIndex::new)

- file: src/commit/pipeline.rs:1-250
  why: CommitPipeline orchestrates atomic multi-layer commits
  pattern: |
    let mut pipeline = CommitPipeline::new(staging);
    let config = CommitConfig {
        message: "commit message".to_string(),
        author_name: None,  // Uses git config
        author_email: None,
        dry_run: false,
    };
    let result = pipeline.execute(&config)?;
    // Returns CommitResult with commit hashes per layer
  gotcha: |
    - CommitPipeline takes ownership of StagingIndex
    - Config with dry_run=true prints summary without committing
    - execute() uses LayerTransaction for atomic ref updates
    - Automatically clears staging index on success
    - Returns error if staging is empty

- file: src/git/repo.rs:1-200
  why: JinRepo wrapper for bare repository operations
  pattern: |
    JinRepo::open_or_create()? - Open ~/.jin/ or create it
    JinRepo::create_at(path)? - Create bare repo at specific path
    repo.create_blob(content)? - Create blob object
    repo.create_tree_from_paths(files)? - Build tree with nested dirs
    repo.create_commit(Some("HEAD"), msg, tree, parents)? - Create commit
  gotcha: |
    - JinRepo wraps git2::Repository, use .inner() for low-level access
    - Always creates BARE repository (no working directory)
    - Default path is ~/.jin/ (not .jin/ in project)
    - Use Repository::init_bare() from git2-rs, MUST create dir first

- file: src/core/config.rs:1-150
  why: ProjectContext tracks active mode/scope/project
  pattern: |
    ProjectContext::load()? - Load from .jin/context
    ctx.save()? - Save to .jin/context as YAML
    ctx.require_mode()? - Get mode or error
    ctx.require_scope()? - Get scope or error
    ProjectContext::default() - Empty context
  gotcha: |
    - load() returns NotInitialized if .jin/context doesn't exist
    - save() creates .jin/ directory automatically
    - YAML format with version field for schema evolution
    - project field can be auto-inferred from Git remote

- file: src/core/layer.rs:1-200
  why: Layer enum and ref path mapping
  pattern: |
    Layer::GlobalBase, Layer::ModeBase, etc.
    layer.ref_path(mode, scope, project) - Generate Git ref path
    layer.storage_path(mode, scope, project) - Generate dir path
    layer.precedence() - Get 1-9 priority number
  gotcha: |
    - Layers 2-5 require mode (ModeBase, ModeScope, etc.)
    - Layer 6 requires scope (ScopeBase)
    - Layer 7 requires project (ProjectBase)
    - Refs stored under refs/jin/layers/ namespace

- file: src/core/error.rs:1-100
  why: JinError enum for consistent error handling
  pattern: |
    JinError::NotInitialized - .jin/ doesn't exist
    JinError::NoActiveContext { context_type: "mode" } - Missing mode
    JinError::StagingFailed { path, reason } - Staging operation failed
    JinError::Transaction(msg) - Transaction error
    JinError::Other(msg) - General error
  critical: |
    - Use thiserror #[error(...)] for Display implementation
    - Auto-convert from std::io::Error and git2::Error with #[from]
    - Return Result<T> (alias for Result<T, JinError>)

# EXTERNAL REFERENCES - Git2-rs Documentation

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.init_bare
  why: How to create bare repository for Jin's storage
  critical: |
    - Repository::init_bare(path) creates bare repo
    - MUST call std::fs::create_dir_all(path) first!
    - Returns Result<Repository, Error>
    - Bare repo has no working directory, only .git/ contents
  section: Repository::init_bare

- url: https://docs.rs/git2/latest/git2/struct.Statuses.html
  why: How to query Git status (for advanced jin status features)
  critical: |
    - Use StatusOptions to configure what to show
    - Status bitflags distinguish INDEX_* vs WT_* changes
    - StatusEntry provides head_to_index() and index_to_workdir() diffs
  section: Statuses

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.commit
  why: Creating commits with git2-rs
  critical: |
    - repo.commit(Some("HEAD"), author, committer, message, tree, parents)?
    - Returns Oid of new commit
    - Atomically updates the ref if update_ref is Some
    - For initial commit, parents is empty slice
  section: Repository::commit

- url: https://docs.rs/git2/latest/git2/struct.Signature.html
  why: Creating author/committer signatures
  critical: |
    - repo.signature()? reads from git config (user.name, user.email)
    - Signature::now(name, email)? for manual creation
    - Returns error if git config is incomplete
  section: Signature

# CLI PATTERNS

- file: tests/cli_basic.rs:1-300
  why: Integration test patterns for all commands
  pattern: |
    - Use assert_cmd::Command to run binary
    - Check .success() for exit code 0
    - Check .stdout().contains("expected text")
    - Use tempfile::TempDir for isolated tests
  gotcha: |
    - Tests run jin binary, not library functions directly
    - Must build binary first: cargo build
    - Use predicates crate for complex assertions
```

### Current Codebase Tree

```bash
jin/
├── Cargo.toml                    # Dependencies: clap, git2, serde, thiserror, anyhow
├── src/
│   ├── main.rs                   # Calls jin::run(cli)
│   ├── lib.rs                    # Exports cli, commands, core modules
│   ├── cli/
│   │   ├── mod.rs                # Cli, Commands, ModeAction, ScopeAction
│   │   └── args.rs               # AddArgs, CommitArgs, etc.
│   ├── commands/
│   │   ├── mod.rs                # execute(cli) dispatcher
│   │   ├── init.rs               # STUB - need to implement
│   │   ├── add.rs                # COMPLETE - 326 lines, reference pattern
│   │   ├── commit_cmd.rs         # STUB - need to implement
│   │   ├── status.rs             # STUB - need to implement
│   │   └── [other commands]      # Future milestones
│   ├── core/
│   │   ├── config.rs             # ProjectContext, JinConfig
│   │   ├── layer.rs              # Layer enum, ref_path(), precedence()
│   │   ├── error.rs              # JinError enum
│   │   └── mod.rs                # Re-exports
│   ├── git/
│   │   ├── repo.rs               # JinRepo wrapper
│   │   ├── refs.rs               # RefOps trait
│   │   ├── objects.rs            # ObjectOps trait (create_blob, create_tree, create_commit)
│   │   ├── transaction.rs        # LayerTransaction (two-phase commit)
│   │   └── tree.rs               # Tree building utilities
│   ├── staging/
│   │   ├── index.rs              # StagingIndex (HashMap<PathBuf, StagedEntry>)
│   │   ├── entry.rs              # StagedEntry, StagedOperation
│   │   ├── router.rs             # route_to_layer(), RoutingOptions
│   │   ├── workspace.rs          # read_file(), is_symlink(), is_git_tracked()
│   │   ├── gitignore.rs          # ensure_in_managed_block()
│   │   └── mod.rs                # Re-exports
│   ├── commit/
│   │   ├── pipeline.rs           # CommitPipeline, CommitConfig, CommitResult
│   │   └── mod.rs                # Re-exports
│   └── merge/
│       └── [merge modules]       # For P2, not needed in P4.M2
├── tests/
│   └── cli_basic.rs              # Integration tests for all commands
└── plan/
    ├── P4M1/PRP.md               # CLI framework (complete)
    └── P4M2/PRP.md               # This document
```

### Desired Codebase Tree (After P4.M2)

```bash
# No new files - only updating existing stubs
src/commands/
├── init.rs          # ~80 lines: Create .jin/, context file, bare repo
├── add.rs           # UNCHANGED - already complete
├── commit_cmd.rs    # ~120 lines: Use CommitPipeline for atomic commits
└── status.rs        # ~150 lines: Display context + staged files
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: Repository::init_bare requires directory to exist
// ============================================================
// WRONG:
let repo = Repository::init_bare("/path/to/repo")?;  // FAILS if dir doesn't exist!

// CORRECT:
std::fs::create_dir_all("/path/to/repo")?;
let repo = Repository::init_bare("/path/to/repo")?;

// ============================================================
// CRITICAL: Check for NotInitialized before loading context
// ============================================================
// PATTERN from add.rs:41-47
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => {
        return Err(JinError::NotInitialized);  // Propagate to user
    }
    Err(_) => ProjectContext::default(),  // Other errors, use default
};

// ============================================================
// CRITICAL: ProjectContext.save() creates .jin/ automatically
// ============================================================
// No need to manually create .jin/ before save()
let mut ctx = ProjectContext::default();
ctx.save()?;  // Creates .jin/ directory if missing

// ============================================================
// PATTERN: CommitPipeline takes ownership of StagingIndex
// ============================================================
let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());
let mut pipeline = CommitPipeline::new(staging);  // staging moved
// Can't use staging anymore!

// ============================================================
// PATTERN: StagingIndex::affected_layers() returns sorted Vec
// ============================================================
// Returns layers in precedence order (GlobalBase=1 to WorkspaceActive=9)
let layers = staging.affected_layers();
for layer in layers {
    // Guaranteed to be in order
}

// ============================================================
// GOTCHA: git2 Signature requires valid git config
// ============================================================
// If user.name or user.email missing from git config:
let sig = repo.signature()?;  // ERROR: config value 'user.name' not found

// FALLBACK:
let sig = repo.signature()
    .or_else(|_| Signature::now("Jin", "jin@local"))?;

// ============================================================
// PATTERN: Error handling with JinError
// ============================================================
// Auto-conversion from std::io::Error and git2::Error
fn example() -> Result<()> {
    std::fs::read_to_string("file")?;  // io::Error -> JinError::Io
    Repository::open("path")?;          // git2::Error -> JinError::Git
    Ok(())
}

// Custom errors
return Err(JinError::Other("description".to_string()));
return Err(JinError::NotInitialized);
return Err(JinError::NoActiveContext {
    context_type: "mode".to_string(),
});

// ============================================================
// PATTERN: Dry-run in CommitPipeline
// ============================================================
let config = CommitConfig {
    message: args.message,
    dry_run: args.dry_run,  // If true, prints summary but doesn't commit
    ..Default::default()
};
let result = pipeline.execute(&config)?;
if result.is_some() {
    println!("Committed {} layers", result.unwrap().committed_layers.len());
}

// ============================================================
// GOTCHA: StagingIndex persistence
// ============================================================
// Must explicitly call save() after modifications
let mut staging = StagingIndex::load()?;
staging.add(entry);
staging.save()?;  // REQUIRED - not automatic!

// ============================================================
// PATTERN: Layer ref path construction
// ============================================================
// From add.rs, requires mode/scope/project from context
let layer = Layer::ModeBase;
let mode = context.require_mode()?;  // Or None for non-mode layers
let ref_path = layer.ref_path(Some(mode), None, None);
// Returns: "refs/jin/layers/mode/python"
```

---

## Implementation Blueprint

### Data Models (Already Complete)

All necessary data structures exist:

```rust
// src/core/config.rs
pub struct ProjectContext {
    pub version: u32,
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}

// src/staging/index.rs
pub struct StagingIndex {
    entries: HashMap<PathBuf, StagedEntry>,
    version: u32,
}

// src/commit/pipeline.rs
pub struct CommitConfig {
    pub message: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub dry_run: bool,
}

pub struct CommitResult {
    pub committed_layers: Vec<Layer>,
    pub file_count: usize,
    pub commit_hashes: Vec<(Layer, String)>,
}
```

### Implementation Tasks (Dependency-Ordered)

```yaml
Task 1: IMPLEMENT src/commands/init.rs
  GOAL: Initialize Jin in current project

  STEPS:
    1. Check if already initialized (.jin/ exists)
       - Return AlreadyExists error if .jin/context exists

    2. Create bare repository at ~/.jin/ (if doesn't exist)
       - Use dirs::home_dir() to get home path
       - Path: home_dir().join(".jin")
       - Call std::fs::create_dir_all() first (CRITICAL!)
       - Call git2::Repository::init_bare(path)?
       - Or use JinRepo::create_at(path)?

    3. Create .jin/context file in project
       - Use ProjectContext::default()
       - Set version = 1
       - Set mode, scope, project = None
       - Call ctx.save()? (creates .jin/ automatically)

    4. Print confirmation
       - "Initialized Jin in <current_dir>"
       - "  Created: .jin/context"
       - "  Repository: ~/.jin/"

  PATTERN: Follow simple validation -> create -> confirm structure

  FOLLOW:
    - src/git/repo.rs:50-80 for JinRepo::create_at()
    - src/core/config.rs:60-90 for ProjectContext::save()

  ERROR HANDLING:
    - Return JinError::AlreadyExists if .jin/context exists
    - Propagate git2::Error via ? operator (auto-converts)
    - Propagate std::io::Error via ? operator (auto-converts)

  NAMING: Use snake_case for functions, UPPER_CASE for constants

  PLACEMENT: src/commands/init.rs

Task 2: VERIFY src/commands/add.rs (NO CHANGES NEEDED)
  STATUS: COMPLETE - 326 lines fully implemented

  REFERENCE PATTERNS TO FOLLOW:
    - ProjectContext::load() with NotInitialized check (lines 41-47)
    - RoutingOptions construction and validation (lines 50-56)
    - JinRepo::open_or_create() usage (line 62)
    - StagingIndex::load() with fallback to new() (line 65)
    - File processing loop with error collection (lines 71-101)
    - staging.save() after modifications (line 104)
    - Summary printing with file count (lines 107-113)
    - Error handling with error list (lines 115-125)

  DO NOT MODIFY: This file is the reference implementation

Task 3: IMPLEMENT src/commands/commit_cmd.rs
  GOAL: Commit staged files atomically across layers

  STEPS:
    1. Load staging index
       - let staging = StagingIndex::load()?
       - Return error if load fails (no staging to commit)

    2. Check if staging is empty
       - if staging.is_empty() { return Err(Other("nothing to commit")) }

    3. Load project context (for layer ref resolution)
       - let context = ProjectContext::load()?
       - Needed for layer.ref_path(mode, scope, project)

    4. Open Jin repository
       - let repo = JinRepo::open_or_create()?

    5. Create CommitPipeline
       - let mut pipeline = CommitPipeline::new(staging)
       - Note: staging ownership moved to pipeline

    6. Build CommitConfig
       - message: args.message
       - author_name: None (use git config)
       - author_email: None (use git config)
       - dry_run: args.dry_run

    7. Execute pipeline
       - let result = pipeline.execute(&config)?
       - Returns CommitResult with commit_hashes

    8. Print result
       - For dry_run: "[Dry run] Would create commits: <layer>: N files"
       - For real: "[<layer> <short_hash>] <message>\n N files changed"

  PATTERN: Load -> Validate -> Pipeline -> Execute -> Report

  FOLLOW:
    - src/commit/pipeline.rs:80-200 for CommitPipeline::execute()
    - src/commands/add.rs:34-128 for error handling pattern

  ERROR HANDLING:
    - Staging empty -> "nothing to commit"
    - No active mode when needed -> propagate NoActiveContext
    - Commit creation fails -> propagate git2::Error
    - Transaction fails -> propagate Transaction error

  DRY-RUN:
    - CommitConfig.dry_run = true
    - Pipeline prints summary and returns None
    - Don't create commits or update refs

  NAMING: commit_cmd.rs (not commit.rs, to avoid conflict with commit/ module)

  PLACEMENT: src/commands/commit_cmd.rs

Task 4: IMPLEMENT src/commands/status.rs
  GOAL: Display workspace state and staged files

  STEPS:
    1. Load project context
       - let context = ProjectContext::load()?
       - If NotInitialized, print "Not in a Jin repository" and return Ok(())

    2. Load staging index
       - let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new())
       - Use fallback to new() if doesn't exist (no staged files)

    3. Print active context
       - "Active Context:"
       - "  Mode:    <mode or (none)>"
       - "  Scope:   <scope or (none)>"
       - "  Project: <project or (none)>"

    4. Get affected layers
       - let layers = staging.affected_layers()
       - Returns Vec<Layer> sorted by precedence

    5. For each layer, print staged files
       - "Staged files (<layer-name>):"
       - "  new file: <path>"
       - Group by layer, show operation type (new file, modified, deleted)

    6. If no staged files
       - Print "nothing to commit, working tree clean"

  PATTERN: Load -> Display context -> Display staged files

  FOLLOW:
    - src/core/config.rs:30-50 for ProjectContext fields
    - src/staging/index.rs:120-140 for affected_layers() and entries_for_layer()
    - src/commands/add.rs:191-203 for format_layer_name() helper

  ERROR HANDLING:
    - NotInitialized -> Graceful message, not an error (return Ok())
    - Staging load fails -> Treat as empty staging

  OUTPUT FORMAT:
    ```
    Active Context:
      Mode:    python
      Scope:   (none)
      Project: my-app

    Staged files (mode-base):
      new file: .vscode/settings.json
      new file: .vscode/launch.json

    Staged files (project-base):
      new file: config.json
    ```

  NAMING: Snake_case for functions

  PLACEMENT: src/commands/status.rs

Task 5: UPDATE tests/cli_basic.rs (ADD NEW TESTS)
  GOAL: Add integration tests for init, commit, status

  ADD TESTS:
    1. test_init_command()
       - Run jin init in temp dir
       - Check .jin/context created
       - Check output contains "Initialized Jin"
       - Try jin init again, should error (already initialized)

    2. test_commit_command()
       - Run jin init
       - Create test file, jin add it
       - Run jin commit -m "test"
       - Check output contains commit hash
       - Verify staging cleared (jin status should show nothing)

    3. test_commit_dry_run()
       - Setup with init + add
       - Run jin commit -m "test" --dry-run
       - Check output contains "[Dry run]"
       - Verify staging NOT cleared (jin status should still show files)

    4. test_status_command()
       - Run jin init
       - Run jin status (should show empty)
       - Add files
       - Run jin status (should show staged files)
       - Commit files
       - Run jin status (should be clean)

    5. test_status_not_initialized()
       - Run jin status in non-initialized dir
       - Should print "Not in a Jin repository" (not error)

  FOLLOW:
    - tests/cli_basic.rs:1-300 for existing test patterns
    - Use assert_cmd::Command for running binary
    - Use tempfile::TempDir for isolation
    - Use predicates::str::contains() for output assertions

  PLACEMENT: tests/cli_basic.rs (append to existing file)
```

### Implementation Patterns & Key Details

```rust
// ============================================================
// PATTERN: init command implementation
// ============================================================
use crate::core::{JinError, ProjectContext, Result};
use crate::git::JinRepo;
use std::path::PathBuf;

pub fn execute() -> Result<()> {
    // 1. Check if already initialized
    if PathBuf::from(".jin/context").exists() {
        return Err(JinError::AlreadyExists(
            "Jin already initialized in this directory".to_string()
        ));
    }

    // 2. Create bare repository at ~/.jin/ (if needed)
    let repo = JinRepo::open_or_create()?;  // Handles creation automatically

    // 3. Create .jin/context in project
    let context = ProjectContext::default();
    context.save()?;  // Creates .jin/ directory

    // 4. Print confirmation
    let current_dir = std::env::current_dir()?;
    println!("Initialized Jin in {}", current_dir.display());
    println!("  Created: .jin/context");
    println!("  Repository: {}", repo.path().display());

    Ok(())
}

// ============================================================
// PATTERN: commit command implementation
// ============================================================
use crate::cli::CommitArgs;
use crate::commit::{CommitConfig, CommitPipeline};
use crate::core::{JinError, ProjectContext, Result};
use crate::git::JinRepo;
use crate::staging::StagingIndex;

pub fn execute(args: CommitArgs) -> Result<()> {
    // 1. Load staging index
    let staging = StagingIndex::load()
        .map_err(|_| JinError::Other("no staging index found".to_string()))?;

    // 2. Check if empty
    if staging.is_empty() {
        return Err(JinError::Other("nothing to commit".to_string()));
    }

    // 3. Load context (needed for layer ref paths)
    let _context = ProjectContext::load()?;

    // 4. Open repository
    let _repo = JinRepo::open_or_create()?;

    // 5. Create pipeline
    let mut pipeline = CommitPipeline::new(staging);

    // 6. Build config
    let config = CommitConfig {
        message: args.message.clone(),
        author_name: None,
        author_email: None,
        dry_run: args.dry_run,
    };

    // 7. Execute
    let result = pipeline.execute(&config)?;

    // 8. Print result
    if args.dry_run {
        println!("[Dry run] Would create commits:");
        if let Some(res) = result {
            for (layer, _hash) in res.commit_hashes {
                println!("  {:?}: N files", layer);
            }
        }
    } else if let Some(res) = result {
        for (layer, hash) in &res.commit_hashes {
            let short_hash = &hash[0..7];
            println!("[{:?} {}] {}", layer, short_hash, args.message);
        }
        println!(" {} files changed", res.file_count);
    }

    Ok(())
}

// ============================================================
// PATTERN: status command implementation
// ============================================================
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::staging::StagingIndex;

pub fn execute() -> Result<()> {
    // 1. Load context (graceful if not initialized)
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            println!("Not in a Jin repository");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // 2. Load staging (fallback to empty)
    let staging = StagingIndex::load()
        .unwrap_or_else(|_| StagingIndex::new());

    // 3. Print active context
    println!("Active Context:");
    println!("  Mode:    {}", context.mode.as_deref().unwrap_or("(none)"));
    println!("  Scope:   {}", context.scope.as_deref().unwrap_or("(none)"));
    println!("  Project: {}", context.project.as_deref().unwrap_or("(none)"));
    println!();

    // 4. Print staged files by layer
    let layers = staging.affected_layers();
    if layers.is_empty() {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    for layer in layers {
        let entries = staging.entries_for_layer(layer);
        println!("Staged files ({}):", format_layer_name(layer));
        for entry in entries {
            let op = match entry.operation {
                StagedOperation::AddOrModify => "new file:",
                StagedOperation::Delete => "deleted:",
                _ => "modified:",
            };
            println!("  {} {}", op, entry.path.display());
        }
        println!();
    }

    Ok(())
}

fn format_layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::GlobalBase => "global-base",
        Layer::ModeBase => "mode-base",
        Layer::ModeScope => "mode-scope",
        Layer::ModeScopeProject => "mode-scope-project",
        Layer::ModeProject => "mode-project",
        Layer::ScopeBase => "scope-base",
        Layer::ProjectBase => "project-base",
        Layer::UserLocal => "user-local",
        Layer::WorkspaceActive => "workspace-active",
    }
}
```

### Integration Points

```yaml
CORE MODULES (read-only):
  - src/core/config.rs: ProjectContext::load(), save(), default()
  - src/core/layer.rs: Layer enum, ref_path(), precedence()
  - src/core/error.rs: JinError variants, Result<T> alias

GIT OPERATIONS (read-only):
  - src/git/repo.rs: JinRepo::open_or_create(), create_at()
  - src/git/objects.rs: ObjectOps trait (create_blob, create_tree, create_commit)
  - src/git/refs.rs: RefOps trait (set_ref, resolve_ref)
  - src/git/transaction.rs: LayerTransaction for atomic updates

STAGING SYSTEM (read-only):
  - src/staging/index.rs: StagingIndex::load(), save(), add(), entries_for_layer()
  - src/staging/entry.rs: StagedEntry, StagedOperation
  - src/staging/router.rs: route_to_layer(), RoutingOptions

COMMIT SYSTEM (read-only):
  - src/commit/pipeline.rs: CommitPipeline::new(), execute()
  - CommitConfig, CommitResult structs

CLI FRAMEWORK (no changes):
  - src/cli/mod.rs: Commands enum routes to execute()
  - src/cli/args.rs: CommitArgs struct with message and dry_run
  - src/commands/mod.rs: Dispatcher calls init::execute(), etc.

EXTERNAL DEPENDENCIES:
  - git2 = "0.19": Repository, Signature, Oid, Error
  - serde = "1.0": Serialize/Deserialize for ProjectContext
  - serde_yaml = "0.9": YAML serialization for .jin/context
  - dirs = "5.0": home_dir() for ~/.jin/ path
  - anyhow = "1.0": Error handling in main()
  - thiserror = "2.0": JinError derive macros
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Type checking
cargo check
# Expected: Zero errors

# Format check
cargo fmt -- --check
# Expected: All files formatted

# Lint check
cargo clippy -- -D warnings
# Expected: Zero warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run all unit tests
cargo test --lib
# Expected: All tests pass

# Run specific command tests
cargo test --lib commands::init::tests
cargo test --lib commands::commit_cmd::tests
cargo test --lib commands::status::tests
# Expected: Each module's tests pass
```

### Level 3: Integration Tests (System Validation)

```bash
# Build binary first
cargo build
# Expected: Clean build

# Run CLI integration tests
cargo test --test cli_basic
# Expected: All integration tests pass

# Specific test validation
cargo test --test cli_basic test_init_command
cargo test --test cli_basic test_commit_command
cargo test --test cli_basic test_commit_dry_run
cargo test --test cli_basic test_status_command
cargo test --test cli_basic test_status_not_initialized
# Expected: Each test passes independently
```

### Level 4: Manual End-to-End Validation

```bash
# Create test project
mkdir /tmp/jin-test && cd /tmp/jin-test
git init  # Jin requires a Git project

# Test init
jin init
# Expected:
# - Output: "Initialized Jin in /tmp/jin-test"
# - File exists: .jin/context
# - Dir exists: ~/.jin/
# - Running jin init again should error

# Test add (already implemented)
echo '{"test": true}' > config.json
jin add config.json
# Expected: "Staged 1 file(s) to project-base layer"

# Test status before commit
jin status
# Expected:
# - Shows "Active Context" with mode/scope/project
# - Shows "Staged files (project-base): new file: config.json"

# Test commit
jin commit -m "Add config file"
# Expected:
# - Output like: "[ProjectBase abc1234] Add config file"
# - Shows "1 files changed"

# Test status after commit
jin status
# Expected: "nothing to commit, working tree clean"

# Test dry-run
jin add README.md
jin commit -m "Test dry run" --dry-run
# Expected:
# - Output contains "[Dry run]"
# - jin status should still show staged files (not committed)

# Test commit after dry-run
jin commit -m "Actually commit"
# Expected:
# - Creates real commit
# - jin status shows clean

# Verify repository structure
ls -la ~/.jin/
# Expected: Bare Git repository structure (objects/, refs/, HEAD, config)

ls -la .jin/
# Expected: context file exists

cat .jin/context
# Expected: Valid YAML with version, mode, scope, project fields
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo test --lib` all unit tests pass
- [ ] `cargo test --test cli_basic` all integration tests pass
- [ ] Binary builds successfully: `cargo build`

### Feature Validation (jin init)

- [ ] Creates `.jin/context` file with valid YAML
- [ ] Creates `~/.jin/` bare repository (if doesn't exist)
- [ ] Prints confirmation message with paths
- [ ] Returns error if already initialized (.jin/context exists)
- [ ] Exit code 0 on success, non-zero on error

### Feature Validation (jin add)

- [ ] No changes needed - already complete
- [ ] Continues to work as reference implementation
- [ ] All existing tests still pass

### Feature Validation (jin commit)

- [ ] Loads staging index successfully
- [ ] Returns error if staging is empty
- [ ] Creates commits for all affected layers
- [ ] Updates refs atomically via LayerTransaction
- [ ] Clears staging index after successful commit
- [ ] Prints commit summary with layer and hash
- [ ] Exit code 0 on success, non-zero on error

### Feature Validation (jin commit --dry-run)

- [ ] Prints summary of what would be committed
- [ ] Does NOT create actual commits
- [ ] Does NOT update refs
- [ ] Does NOT clear staging index
- [ ] Output clearly marked as "[Dry run]"

### Feature Validation (jin status)

- [ ] Loads and displays active context (mode/scope/project)
- [ ] Shows staged files grouped by layer
- [ ] Shows operation type (new file, modified, deleted)
- [ ] Prints "nothing to commit" when staging is empty
- [ ] Handles not-initialized gracefully (prints message, exits 0)
- [ ] Handles missing staging index (treats as empty)

### Integration Validation

- [ ] Full workflow works: init -> add -> status -> commit -> status
- [ ] Multiple layers work: add to mode-base, add to project-base, commit both
- [ ] Error recovery: failed commit leaves staging intact
- [ ] Concurrent safety: LayerTransaction prevents corruption

### Code Quality Validation

- [ ] Follows existing patterns from src/commands/add.rs
- [ ] Consistent error handling with JinError
- [ ] Proper use of Result<T> return types
- [ ] No unwrap() calls (use ? operator or map_err)
- [ ] Clear variable names (context, staging, pipeline, config)
- [ ] Functions under 50 lines where possible
- [ ] Comments explain "why", not "what"

### Documentation Validation

- [ ] Module-level doc comments (//!) for each command file
- [ ] Function doc comments for public functions
- [ ] Error cases documented in doc comments
- [ ] Integration test comments explain what they validate

---

## Anti-Patterns to Avoid

- ❌ **Don't call Repository::init_bare() without creating directory first** - Will panic
- ❌ **Don't call ProjectContext::load() without handling NotInitialized** - User-facing error
- ❌ **Don't forget to save() StagingIndex after modifications** - Changes lost
- ❌ **Don't use unwrap() in production code** - Use ? operator or proper error handling
- ❌ **Don't create commits without LayerTransaction** - Breaks atomicity guarantee
- ❌ **Don't clear staging on dry-run** - User expects preview, not side effects
- ❌ **Don't print "error:" prefix yourself** - JinError Display implementation handles it
- ❌ **Don't create new error types** - Use existing JinError variants
- ❌ **Don't bypass CommitPipeline** - It handles all the complex logic correctly
- ❌ **Don't hardcode ~/.jin/ path** - Use JinRepo::default_path() or dirs::home_dir()

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success likelihood

**Justification:**

**Strengths:**
- ✅ `jin add` is complete (326 lines) - perfect reference pattern
- ✅ All supporting modules (staging, commit, git, core) are complete and tested
- ✅ CommitPipeline handles all complex commit logic
- ✅ LayerTransaction provides atomic ref updates
- ✅ CLI framework is complete (P4.M1)
- ✅ Clear implementation tasks with exact patterns to follow
- ✅ Comprehensive git2-rs documentation URLs provided
- ✅ All error cases identified with specific JinError variants
- ✅ Integration test patterns exist in tests/cli_basic.rs

**Potential Challenges:**
- ⚠️ git2::Repository::signature() requires valid git config - need fallback
- ⚠️ Repository::init_bare() directory creation requirement - clearly documented
- ⚠️ CommitPipeline takes ownership of StagingIndex - pattern shown in PRP

**Mitigations:**
- All gotchas documented in "Known Gotchas & Library Quirks" section
- Reference implementation (add.rs) demonstrates every pattern needed
- Validation loop catches issues early (syntax, unit tests, integration tests, manual E2E)
- External documentation links provided with specific sections to read

**Missing Points (-1):**
- Signature fallback pattern not explicitly shown in implementation blueprint
- Could benefit from more detailed CommitPipeline::execute() walkthrough
- Status output formatting could be more specific (but format_layer_name() provided)

---

## Implementation Order Recommendation

**Day 1: Foundation**
1. Implement `jin init` (simplest - ~80 lines)
2. Test init thoroughly (integration test)
3. Verify `jin add` still works (should be unchanged)

**Day 2: Core Workflow**
4. Implement `jin status` (read-only operations - ~150 lines)
5. Test with existing staged files from add
6. Implement `jin commit` (~120 lines)
7. Test full workflow: init -> add -> status -> commit -> status

**Day 3: Polish & Edge Cases**
8. Implement `jin commit --dry-run`
9. Add comprehensive integration tests
10. Manual E2E testing with multiple layers
11. Error case validation

---

## Success Metrics

**Primary Metrics:**
- [ ] All 4 commands (init, add, commit, status) work end-to-end
- [ ] `cargo test` passes with 0 failures
- [ ] Integration tests cover all success and error paths
- [ ] Manual testing succeeds for multi-layer commits

**Quality Metrics:**
- [ ] Code follows patterns from src/commands/add.rs
- [ ] Error messages are actionable and user-friendly
- [ ] No clippy warnings
- [ ] All public functions have doc comments

**User Experience Metrics:**
- [ ] `jin init` completes in <1 second
- [ ] `jin commit` creates commits atomically (all-or-nothing)
- [ ] `jin status` loads in <200ms for typical staging index
- [ ] Error messages guide users to solutions

---

## Appendix: File Line Counts

Estimated implementation sizes based on reference patterns:

| File | Current Lines | After P4.M2 | Change |
|------|---------------|-------------|--------|
| `src/commands/init.rs` | 13 (stub) | ~80 | +67 |
| `src/commands/add.rs` | 326 (complete) | 326 | 0 |
| `src/commands/commit_cmd.rs` | 14 (stub) | ~120 | +106 |
| `src/commands/status.rs` | 13 (stub) | ~150 | +137 |
| `tests/cli_basic.rs` | 316 | ~500 | +184 |
| **Total** | **682** | **~1,176** | **+494** |

---

## Appendix: Command Responsibility Matrix

| Command | Initializes | Reads Staging | Writes Staging | Reads Context | Writes Context | Creates Commits |
|---------|-------------|---------------|----------------|---------------|----------------|-----------------|
| `init` | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ |
| `add` | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ |
| `commit` | ❌ | ✅ | ✅ (clear) | ✅ | ❌ | ✅ |
| `status` | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ |

---

## Appendix: Error Handling Matrix

| Scenario | Error Type | User Message | Exit Code |
|----------|------------|--------------|-----------|
| `jin init` when already initialized | `AlreadyExists` | "Jin already initialized in this directory" | 1 |
| `jin add/commit/status` before init | `NotInitialized` | "Not in a Jin repository. Run 'jin init' first." | 1 |
| `jin commit` with empty staging | `Other` | "nothing to commit" | 1 |
| `jin commit` with no active mode (mode layer) | `NoActiveContext` | "No active mode. Run 'jin mode use <name>' first." | 1 |
| `jin add` with Git-tracked file | `GitTracked` | "File is tracked by Git. Use 'jin import' instead." | 1 |
| git2 operation fails | `Git` | Forwarded from git2::Error | 1 |
| Filesystem operation fails | `Io` | Forwarded from std::io::Error | 1 |
| Success | (none) | Normal output | 0 |
