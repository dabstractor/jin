# PRP: P3.M1 - Staging System with Layer Routing

---

## Goal

**Feature Goal**: Implement a complete file staging system that reads workspace files, creates Git blob objects, routes files to the correct layer based on CLI flags and active context, and persists the staging index for the commit pipeline.

**Deliverable**:
1. Complete `src/commands/add.rs` implementation for `jin add` command
2. Enhanced `src/staging/index.rs` with blob creation and workspace integration
3. New `src/staging/workspace.rs` for reading files from the working directory
4. Integration with `.gitignore` management (managed block)
5. Comprehensive unit and integration tests for all staging operations

**Success Definition**:
- `jin add <file>` stages files to the correct layer based on flags
- `jin add --mode` routes to Mode Base layer (requires active mode)
- `jin add --mode --project` routes to Mode → Project layer
- `jin add --scope=<scope>` routes to Scope Base layer
- All staged files have valid Git blob OIDs in the Jin repository
- Staging index persists to `.jin/staging/index.json`
- All tests pass: `cargo test staging:: && cargo test commands::add::`
- `cargo check && cargo clippy && cargo test` all pass with zero errors/warnings

---

## User Persona

**Target User**: Developer using Jin to manage tool-specific configuration files

**Use Case**: A developer wants to stage their `.claude/config.json` file to be committed to a specific layer. They run `jin add .claude/config.json --mode` to stage it to the active mode's base layer.

**User Journey**:
1. Developer modifies `.claude/config.json` in their workspace
2. Developer runs `jin add .claude/config.json --mode`
3. Jin validates the file exists and is not Git-tracked
4. Jin reads file content and creates a Git blob in the Jin repository
5. Jin routes the file to the correct layer (ModeBase) based on flags + active context
6. Jin adds the file path to `.gitignore` managed block if not already present
7. Jin updates the staging index and persists to disk
8. Developer can run `jin status` to see staged files (future milestone)
9. Developer runs `jin commit -m "message"` to commit staged files (P3.M2)

**Pain Points Addressed**:
- Deterministic layer routing based on explicit flags
- Automatic `.gitignore` management prevents accidental Git commits
- Content-addressed storage ensures file integrity
- Clear error messages when routing is invalid

---

## Why

- **PRD Requirement**: Section 6.1 defines staging & committing as core API contract
- **PRD Requirement**: Section 9.1 defines the complete routing table for `jin add`
- **Foundation for Commit Pipeline**: P3.M2 depends on complete staging system
- **User-Facing Command**: `jin add` is the primary way users interact with Jin
- **Atomic Commits**: Staging enables grouping changes before committing (PRD Section 6.2)
- **9-Layer System**: Correct routing is essential for the layer hierarchy (PRD Section 4.1)

---

## What

### User-Visible Behavior

After this milestone:

```bash
# Default: routes to Project Base (Layer 7)
jin add .claude/config.json
# → Stages to refs/jin/layers/project/<project>/

# Mode flag: routes to Mode Base (Layer 2)
jin mode use claude
jin add .claude/config.json --mode
# → Stages to refs/jin/layers/mode/claude/

# Mode + Project: routes to Mode → Project (Layer 5)
jin add .claude/config.json --mode --project
# → Stages to refs/jin/layers/mode/claude/project/<project>/

# Mode + Scope: routes to Mode → Scope (Layer 3)
jin add .claude/config.json --mode --scope=language:javascript
# → Stages to refs/jin/layers/mode/claude/scope/language:javascript/

# Untethered scope: routes to Scope Base (Layer 6)
jin add .editorconfig --scope=language:javascript
# → Stages to refs/jin/layers/scope/language:javascript/

# Global: routes to Global Base (Layer 1)
jin add .tool-defaults.json --global
# → Stages to refs/jin/layers/global/

# Multiple files
jin add .claude/ CLAUDE.md --mode
# → Stages all files in directory + CLAUDE.md to mode layer

# Error cases
jin add .claude/config.json --mode  # Without active mode
# → ERROR: No active mode

jin add --project  # Without --mode
# → ERROR: --project requires --mode flag

jin add tracked-by-git.rs
# → ERROR: File is tracked by Git. Use `jin import` instead.
```

### Technical Requirements

1. **File Reading**: Read file content from workspace (working directory)
2. **Blob Creation**: Create Git blob objects in Jin repository using `git2`
3. **Layer Routing**: Apply routing rules from PRD Section 9.1
4. **Context Integration**: Read active mode/scope from `.jin/context`
5. **Staging Persistence**: Save staging index to `.jin/staging/index.json`
6. **Gitignore Management**: Add staged files to `.gitignore` managed block
7. **Validation**: Reject Git-tracked files, symlinks, missing files, invalid flag combinations

### Success Criteria

- [ ] `jin add <file>` creates blob and stages to Project Base
- [ ] `jin add <file> --mode` stages to Mode Base (requires active mode)
- [ ] `jin add <file> --mode --project` stages to Mode → Project
- [ ] `jin add <file> --mode --scope=<scope>` stages to Mode → Scope
- [ ] `jin add <file> --mode --scope=<scope> --project` stages to Mode → Scope → Project
- [ ] `jin add <file> --scope=<scope>` stages to Scope Base (untethered)
- [ ] `jin add <file> --global` stages to Global Base
- [ ] Multiple files and directories are supported
- [ ] Files are added to `.gitignore` managed block
- [ ] Error if file is Git-tracked
- [ ] Error if file doesn't exist
- [ ] Error if symlink detected
- [ ] Error if --mode without active mode
- [ ] Error if --project without --mode
- [ ] Staging index persists across command invocations
- [ ] Content hash matches file content

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement the staging system. An AI agent with access to this PRP and the codebase can implement the feature in one pass._

### Documentation & References

```yaml
# MUST READ - Existing Implementation (COMPLETE)

- file: src/staging/entry.rs
  why: StagedEntry struct - already complete, no changes needed
  lines: 89
  pattern: |
    StagedEntry { path, target_layer, content_hash, mode, operation }
    StagedOperation enum: AddOrModify, Delete, Rename
    StagedEntry::new(path, layer, hash) for add/modify
    StagedEntry::delete(path, layer) for deletions
  gotcha: content_hash is hex string, mode is u32 (0o100644 for regular files)

- file: src/staging/index.rs
  why: StagingIndex with HashMap storage - needs enhancement
  lines: 204
  pattern: |
    StagingIndex { entries: HashMap<PathBuf, StagedEntry>, version }
    load()/save() for persistence to .jin/staging/index.json
    add(entry), remove(path), get(path), entries(), entries_for_layer(layer)
    affected_layers() returns sorted layers by precedence
  gotcha: Uses HashMap - does NOT preserve insertion order. Consider IndexMap if order matters.
  enhancement_needed: |
    - Add create_blob_for_file(path) to create Git blob and return OID
    - Add stage_file(path, layer, options) high-level API
    - Add unstage_file(path) to remove from staging

- file: src/staging/router.rs
  why: Layer routing logic - already complete
  lines: 213
  pattern: |
    RoutingOptions { mode, scope, project, global }
    route_to_layer(options, context) -> Result<Layer>
    validate_routing_options(options) -> Result<()>
  gotcha: |
    - mode=true requires context.mode.is_some()
    - project=true requires mode=true
    - global conflicts with other flags
  critical: This is the PRD Section 9.1 routing table implementation

- file: src/staging/mod.rs
  why: Module exports
  lines: 13
  pattern: |
    pub use entry::StagedEntry;
    pub use index::StagingIndex;
    pub use router::route_to_layer;

# MUST READ - Command Implementation (TODO)

- file: src/commands/add.rs
  why: jin add command - currently a stub, needs full implementation
  lines: 14
  pattern: |
    pub fn execute(args: AddArgs) -> Result<()>
  critical: This is the main implementation target for this milestone

- file: src/cli/args.rs
  why: CLI argument definitions
  pattern: Look for AddArgs struct with files, mode, scope, project, global fields
  gotcha: Use clap derive macros for argument parsing

# MUST READ - Core Types

- file: src/core/layer.rs
  why: Layer enum and precedence - already complete
  lines: 284
  pattern: |
    Layer enum: GlobalBase(1), ModeBase(2), ModeScope(3), ModeScopeProject(4),
                ModeProject(5), ScopeBase(6), ProjectBase(7), UserLocal(8), WorkspaceActive(9)
    precedence() -> u8
    ref_path(mode, scope, project) -> String for Git ref names
    requires_mode(), requires_scope(), is_project_specific()
  critical: Use Layer::ref_path() to get the Git ref path for commits

- file: src/core/config.rs
  why: ProjectContext for active mode/scope
  lines: 231
  pattern: |
    ProjectContext { version, mode, scope, project, last_updated }
    ProjectContext::load() -> Result<Self>
    require_mode() -> Result<&str> - errors if no active mode
    require_scope() -> Result<&str> - errors if no active scope
  gotcha: load() returns NotInitialized error if .jin/context doesn't exist

- file: src/core/error.rs
  why: JinError types
  lines: 102
  pattern: |
    JinError::Io, Git, Config, Parse, MergeConflict, Transaction
    JinError::InvalidLayer, NoActiveContext { context_type }
    JinError::NotFound, AlreadyExists, NotInitialized, Other
  enhancement_needed: Add JinError::GitTracked { path } for files tracked by main Git

# MUST READ - Git Integration

- file: src/git/repo.rs
  why: JinRepo wrapper for Jin's bare repository
  lines: 279
  pattern: |
    JinRepo::open() - open existing Jin repo at ~/.jin/
    JinRepo::open_or_create() - preferred for most operations
    inner() -> &Repository for git2 operations
  gotcha: Jin repo is BARE - no working directory

- file: src/git/objects.rs
  why: Blob creation - ObjectOps trait
  lines: 451
  pattern: |
    ObjectOps trait implemented on JinRepo:
    create_blob(content: &[u8]) -> Result<Oid>
    create_blob_from_path(path: &Path) -> Result<Oid>
    find_blob(oid) -> Result<Blob>
  critical: Use create_blob_from_path() to hash workspace files directly

# EXTERNAL REFERENCES

- url: https://git-scm.com/docs/index-format
  why: Git index format documentation
  critical: Jin uses JSON instead of binary format for readability

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.blob
  why: git2 blob creation API
  critical: repo.blob(content) creates blob and returns Oid

- url: https://docs.rs/indexmap/latest/indexmap/
  why: Ordered HashMap for deterministic staging order
  critical: Consider using IndexMap instead of HashMap for entries
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── cli/
│   │   ├── args.rs           # CLI arguments (AddArgs defined here)
│   │   └── mod.rs            # CLI module
│   ├── commands/
│   │   ├── add.rs            # jin add - TO BE IMPLEMENTED (14 lines, stub)
│   │   ├── commit_cmd.rs     # jin commit - uses staging (future integration)
│   │   ├── status.rs         # jin status - reads staging (future integration)
│   │   └── mod.rs            # Command dispatch
│   ├── core/
│   │   ├── config.rs         # ProjectContext for active mode/scope
│   │   ├── error.rs          # JinError types
│   │   ├── layer.rs          # Layer enum with 9 layers
│   │   └── mod.rs            # Core exports
│   ├── git/
│   │   ├── objects.rs        # ObjectOps: create_blob(), create_tree()
│   │   ├── refs.rs           # RefOps: resolve_ref(), ref_exists()
│   │   ├── repo.rs           # JinRepo wrapper
│   │   ├── transaction.rs    # LayerTransaction for atomic commits
│   │   ├── tree.rs           # TreeOps for reading trees
│   │   └── mod.rs            # Git exports
│   ├── staging/
│   │   ├── entry.rs          # StagedEntry struct (complete)
│   │   ├── index.rs          # StagingIndex (needs enhancement)
│   │   ├── router.rs         # route_to_layer() (complete)
│   │   └── mod.rs            # Staging exports
│   ├── commit/
│   │   ├── pipeline.rs       # CommitPipeline (uses StagingIndex)
│   │   └── mod.rs            # Commit module
│   └── lib.rs                # Library exports
├── Cargo.toml                # Dependencies (git2, serde, thiserror, etc.)
└── tests/
    └── integration/
        └── cli_basic.rs      # CLI integration tests
```

### Desired Codebase Tree After P3.M1

```bash
jin/
├── src/
│   ├── commands/
│   │   └── add.rs            # COMPLETED (~150 lines):
│   │       ├── execute(args) - main entry point
│   │       ├── stage_files(paths, layer, repo, index) - batch staging
│   │       ├── validate_file(path) - exists, not symlink, not git-tracked
│   │       └── comprehensive tests
│   ├── core/
│   │   └── error.rs          # ENHANCED:
│   │       ├── JinError::GitTracked { path } - for git-tracked files
│   │       ├── JinError::Symlink { path } - for symlink rejection
│   │       └── JinError::StagingFailed { path, reason }
│   ├── staging/
│   │   ├── entry.rs          # (unchanged - already complete)
│   │   ├── index.rs          # ENHANCED (~280 lines):
│   │   │   ├── stage_file(path, layer, repo) - high-level staging API
│   │   │   ├── unstage_file(path) - remove from staging
│   │   │   ├── get_blob_oid(path) - get content hash for file
│   │   │   └── additional helper methods
│   │   ├── router.rs         # (unchanged - already complete)
│   │   ├── workspace.rs      # NEW (~100 lines):
│   │   │   ├── read_file(path) - read workspace file content
│   │   │   ├── file_exists(path) - check file exists
│   │   │   ├── is_symlink(path) - detect symlinks
│   │   │   ├── is_git_tracked(path) - check main Git repo
│   │   │   └── get_file_mode(path) - get executable bit
│   │   ├── gitignore.rs      # NEW (~80 lines):
│   │   │   ├── ensure_in_managed_block(path) - add to .gitignore
│   │   │   ├── MANAGED_START/END markers
│   │   │   └── read/write gitignore file
│   │   └── mod.rs            # Updated exports
│   └── lib.rs                # (unchanged)
└── plan/
    └── P3M1/
        ├── PRP.md            # This file
        └── research/         # Research documents
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: Jin repository is BARE - no working directory
// ============================================================
// When creating blobs, we read from the WORKSPACE (project working dir)
// but store in the JIN REPO (bare repo at ~/.jin/)
//
// CORRECT:
// let content = std::fs::read(workspace_path)?;  // Read from workspace
// let oid = jin_repo.create_blob(&content)?;     // Store in Jin's bare repo
//
// WRONG:
// let oid = jin_repo.create_blob_from_path(path)?;  // This reads from Jin repo path!

// ============================================================
// CRITICAL: Check Git tracking in PROJECT repo, not Jin repo
// ============================================================
// The project has its own .git/ directory
// Jin has a separate bare repo at ~/.jin/
//
// To check if file is tracked by project's Git:
// let project_repo = git2::Repository::discover(".")?;  // Opens project repo
// let index = project_repo.index()?;
// let is_tracked = index.get_path(Path::new(file), 0).is_some();

// ============================================================
// CRITICAL: .gitignore managed block format
// ============================================================
// PRD Section 8.1 specifies exact format:
// # --- JIN MANAGED START ---
// .claude/
// .vscode/settings.json
// # --- JIN MANAGED END ---
//
// Rules:
// - Never edit outside this block
// - Auto-deduplicate entries
// - Remove entries when files are removed from Jin

// ============================================================
// GOTCHA: PathBuf comparison is case-sensitive on all platforms
// ============================================================
// Jin treats paths as case-sensitive even on Windows/macOS
// This matches Git's behavior for cross-platform consistency

// ============================================================
// GOTCHA: Staging directory pattern
// ============================================================
// When staging a directory like ".claude/", Jin should:
// 1. Walk the directory recursively
// 2. Stage each file individually (not the directory itself)
// 3. Directories become implicit via tree structure in commit

// ============================================================
// GOTCHA: File mode detection
// ============================================================
// On Unix: Check executable bit via std::os::unix::fs::PermissionsExt
// On Windows: Default to 0o100644 (regular file)
//
// #[cfg(unix)]
// fn get_file_mode(path: &Path) -> u32 {
//     use std::os::unix::fs::PermissionsExt;
//     let meta = std::fs::metadata(path).ok();
//     match meta {
//         Some(m) if m.permissions().mode() & 0o111 != 0 => 0o100755,
//         _ => 0o100644,
//     }
// }
// #[cfg(not(unix))]
// fn get_file_mode(_path: &Path) -> u32 { 0o100644 }

// ============================================================
// PATTERN: Content hash as hex string
// ============================================================
// StagedEntry.content_hash stores OID as 40-character hex string
// let oid: Oid = repo.create_blob(&content)?;
// let hash: String = oid.to_string();  // 40 hex chars
//
// To convert back:
// let oid = Oid::from_str(&hash)?;
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/staging/workspace.rs (NEW) ==================

use crate::core::{JinError, Result};
use std::path::{Path, PathBuf};

/// Read a file from the workspace (project working directory)
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            JinError::NotFound(path.display().to_string())
        } else {
            JinError::Io(e)
        }
    })
}

/// Check if a path is a symlink
pub fn is_symlink(path: &Path) -> Result<bool> {
    let meta = std::fs::symlink_metadata(path)?;
    Ok(meta.file_type().is_symlink())
}

/// Check if a file is tracked by the project's Git repository
pub fn is_git_tracked(path: &Path) -> Result<bool> {
    // Implementation uses git2::Repository::discover()
}

/// Get file mode (executable or regular)
pub fn get_file_mode(path: &Path) -> u32 {
    // Platform-specific implementation
}

/// Walk a directory recursively and return all file paths
pub fn walk_directory(path: &Path) -> Result<Vec<PathBuf>> {
    // Implementation using std::fs::read_dir recursively
}

// ================== src/staging/gitignore.rs (NEW) ==================

const MANAGED_START: &str = "# --- JIN MANAGED START ---";
const MANAGED_END: &str = "# --- JIN MANAGED END ---";

/// Ensure a path is in the .gitignore managed block
pub fn ensure_in_managed_block(path: &Path) -> Result<()> {
    // Implementation reads .gitignore, finds/creates managed block, adds path
}

/// Remove a path from the .gitignore managed block
pub fn remove_from_managed_block(path: &Path) -> Result<()> {
    // Implementation for when files are removed from Jin
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD new error variants to src/core/error.rs
  - ADD: JinError::GitTracked { path: String } for git-tracked files
  - ADD: JinError::Symlink { path: String } for symlink detection
  - ADD: JinError::StagingFailed { path: String, reason: String }
  - FOLLOW pattern: Existing error variants in file
  - PLACEMENT: In JinError enum
  - TESTS: Add tests for error display messages

Task 2: CREATE src/staging/workspace.rs
  - CREATE: New file for workspace file operations
  - IMPLEMENT: read_file(path: &Path) -> Result<Vec<u8>>
  - IMPLEMENT: is_symlink(path: &Path) -> Result<bool>
  - IMPLEMENT: is_git_tracked(path: &Path) -> Result<bool>
  - IMPLEMENT: get_file_mode(path: &Path) -> u32
  - IMPLEMENT: walk_directory(path: &Path) -> Result<Vec<PathBuf>>
  - IMPORT: crate::core::{JinError, Result}, std::path::{Path, PathBuf}
  - TESTS: Test each function with edge cases

Task 3: CREATE src/staging/gitignore.rs
  - CREATE: New file for .gitignore managed block
  - DEFINE: MANAGED_START, MANAGED_END constants
  - IMPLEMENT: ensure_in_managed_block(path: &Path) -> Result<()>
  - IMPLEMENT: remove_from_managed_block(path: &Path) -> Result<()>
  - IMPLEMENT: read_gitignore() -> Result<String>
  - IMPLEMENT: write_gitignore(content: &str) -> Result<()>
  - HANDLE: .gitignore doesn't exist (create it)
  - HANDLE: Managed block doesn't exist (create it)
  - HANDLE: Duplicate entries (deduplicate)
  - TESTS: Test create, update, remove operations

Task 4: UPDATE src/staging/mod.rs exports
  - ADD: pub mod workspace;
  - ADD: pub mod gitignore;
  - ADD: pub use workspace::{read_file, is_symlink, is_git_tracked, walk_directory};
  - ADD: pub use gitignore::ensure_in_managed_block;
  - PRESERVE: Existing exports

Task 5: ENHANCE src/staging/index.rs
  - ADD: use crate::git::{JinRepo, ObjectOps};
  - ADD: stage_file(path, layer, repo) method
    - Read file content using workspace::read_file
    - Create blob using repo.create_blob(&content)
    - Get file mode using workspace::get_file_mode
    - Create StagedEntry with hash = oid.to_string()
    - Add to entries HashMap
  - ADD: unstage_file(path) method
    - Remove entry from HashMap
  - ADD: get_staged_paths() -> Vec<&PathBuf> method
  - ENSURE: save() is called after modifications
  - TESTS: Test stage, unstage, persistence

Task 6: IMPLEMENT src/commands/add.rs
  - IMPORT: All required modules
  - IMPLEMENT: execute(args: AddArgs) -> Result<()>
    - Load ProjectContext for active mode/scope
    - Build RoutingOptions from CLI args
    - Validate routing options
    - Determine target layer using route_to_layer()
    - Open Jin repository
    - Load staging index
    - For each file path in args.files:
      - Expand directories using walk_directory()
      - Validate file (exists, not symlink, not git-tracked)
      - Stage file to index
      - Add to .gitignore managed block
    - Save staging index
    - Print summary
  - HANDLE: Empty file list (error)
  - HANDLE: All validation errors with clear messages
  - TESTS: Integration tests for all routing scenarios

Task 7: ADD AddArgs to src/cli/args.rs (if not complete)
  - VERIFY: AddArgs struct exists with:
    - files: Vec<PathBuf> - files to stage
    - mode: bool - use --mode flag
    - scope: Option<String> - use --scope=<value> flag
    - project: bool - use --project flag
    - global: bool - use --global flag
  - ADD: If missing, add using clap derive macros
  - FOLLOW pattern: Existing CLI args in file

Task 8: ADD comprehensive tests for staging system
  - FILE: Tests in each module's #[cfg(test)] section
  - TEST: workspace.rs - file reading, symlink detection, git tracking
  - TEST: gitignore.rs - managed block creation, updates, edge cases
  - TEST: index.rs - staging, unstaging, persistence
  - TEST: add.rs - full workflow with different flag combinations
  - USE: tempfile crate for isolated test directories
  - MOCK: Create test git repositories for git-tracking tests
```

### Implementation Patterns & Key Details

```rust
// ================== src/commands/add.rs IMPLEMENTATION ==================

use crate::cli::AddArgs;
use crate::core::{Layer, ProjectContext, Result, JinError};
use crate::git::{JinRepo, ObjectOps};
use crate::staging::{
    StagedEntry, StagingIndex, RoutingOptions,
    route_to_layer, validate_routing_options,
    read_file, is_symlink, is_git_tracked, walk_directory,
    ensure_in_managed_block,
};
use std::path::PathBuf;

/// Execute the jin add command
pub fn execute(args: AddArgs) -> Result<()> {
    // 1. Validate we have files to stage
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Load project context for active mode/scope
    let context = ProjectContext::load().unwrap_or_default();

    // 3. Build and validate routing options
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
    validate_routing_options(&options)?;

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // 6. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. Process each file
    let mut staged_count = 0;
    for path in &args.files {
        // Expand directories
        let files_to_stage = if path.is_dir() {
            walk_directory(path)?
        } else {
            vec![path.clone()]
        };

        for file_path in files_to_stage {
            // Validate file
            validate_file(&file_path)?;

            // Read content and create blob
            let content = read_file(&file_path)?;
            let oid = repo.create_blob(&content)?;
            let mode = get_file_mode(&file_path);

            // Create staged entry
            let entry = StagedEntry {
                path: file_path.clone(),
                target_layer,
                content_hash: oid.to_string(),
                mode,
                operation: crate::staging::StagedOperation::AddOrModify,
            };

            // Add to staging index
            staging.add(entry);

            // Add to .gitignore managed block
            ensure_in_managed_block(&file_path)?;

            staged_count += 1;
        }
    }

    // 8. Save staging index
    staging.save()?;

    // 9. Print summary
    println!(
        "Staged {} file(s) to {} layer",
        staged_count,
        target_layer
    );

    Ok(())
}

/// Validate a file for staging
fn validate_file(path: &PathBuf) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // Check not a symlink (PRD Section 19.3)
    if is_symlink(path)? {
        return Err(JinError::Symlink {
            path: path.display().to_string(),
        });
    }

    // Check not tracked by project's Git (PRD Section 9.2)
    if is_git_tracked(path)? {
        return Err(JinError::GitTracked {
            path: path.display().to_string(),
        });
    }

    Ok(())
}

#[cfg(unix)]
fn get_file_mode(path: &PathBuf) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) if meta.permissions().mode() & 0o111 != 0 => 0o100755,
        _ => 0o100644,
    }
}

#[cfg(not(unix))]
fn get_file_mode(_path: &PathBuf) -> u32 {
    0o100644
}

// ================== src/staging/workspace.rs IMPLEMENTATION ==================

use crate::core::{JinError, Result};
use std::path::{Path, PathBuf};

/// Read a file from the workspace
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            JinError::NotFound(path.display().to_string())
        } else {
            JinError::Io(e)
        }
    })
}

/// Check if a path is a symlink
pub fn is_symlink(path: &Path) -> Result<bool> {
    let meta = std::fs::symlink_metadata(path)?;
    Ok(meta.file_type().is_symlink())
}

/// Check if a file is tracked by the project's Git repository
pub fn is_git_tracked(path: &Path) -> Result<bool> {
    // Try to discover project's Git repository
    let repo = match git2::Repository::discover(".") {
        Ok(r) => r,
        Err(_) => return Ok(false), // No Git repo = not tracked
    };

    // Get the index (staging area) of project's Git
    let index = repo.index().map_err(JinError::Git)?;

    // Normalize path relative to repo workdir
    let workdir = repo.workdir().unwrap_or_else(|| Path::new("."));
    let rel_path = path.strip_prefix(workdir).unwrap_or(path);

    // Check if file is in the index
    Ok(index.get_path(rel_path, 0).is_some())
}

/// Walk a directory recursively and return all file paths
pub fn walk_directory(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory_recursive(path, &mut files)?;
    Ok(files)
}

fn walk_directory_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_directory_recursive(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

// ================== src/staging/gitignore.rs IMPLEMENTATION ==================

use crate::core::Result;
use std::path::Path;

const MANAGED_START: &str = "# --- JIN MANAGED START ---";
const MANAGED_END: &str = "# --- JIN MANAGED END ---";
const GITIGNORE_PATH: &str = ".gitignore";

/// Ensure a path is in the .gitignore managed block
pub fn ensure_in_managed_block(path: &Path) -> Result<()> {
    let content = read_gitignore();
    let path_str = path.display().to_string();

    // Parse existing content
    let (before, managed, after) = parse_managed_block(&content);

    // Check if already present
    if managed.contains(&path_str) {
        return Ok(());
    }

    // Add to managed block
    let mut new_managed = managed;
    new_managed.push(path_str);
    new_managed.sort();
    new_managed.dedup();

    // Rebuild content
    let new_content = build_gitignore(&before, &new_managed, &after);
    write_gitignore(&new_content)?;

    Ok(())
}

fn read_gitignore() -> String {
    std::fs::read_to_string(GITIGNORE_PATH).unwrap_or_default()
}

fn write_gitignore(content: &str) -> Result<()> {
    std::fs::write(GITIGNORE_PATH, content)?;
    Ok(())
}

fn parse_managed_block(content: &str) -> (String, Vec<String>, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut before = Vec::new();
    let mut managed = Vec::new();
    let mut after = Vec::new();
    let mut in_block = false;
    let mut after_block = false;

    for line in lines {
        if line == MANAGED_START {
            in_block = true;
            continue;
        }
        if line == MANAGED_END {
            in_block = false;
            after_block = true;
            continue;
        }

        if in_block {
            if !line.trim().is_empty() && !line.starts_with('#') {
                managed.push(line.to_string());
            }
        } else if after_block {
            after.push(line.to_string());
        } else {
            before.push(line.to_string());
        }
    }

    (before.join("\n"), managed, after.join("\n"))
}

fn build_gitignore(before: &str, managed: &[String], after: &str) -> String {
    let mut result = String::new();

    if !before.is_empty() {
        result.push_str(before);
        result.push('\n');
    }

    result.push_str(MANAGED_START);
    result.push('\n');
    for entry in managed {
        result.push_str(entry);
        result.push('\n');
    }
    result.push_str(MANAGED_END);
    result.push('\n');

    if !after.is_empty() {
        result.push_str(after);
        if !after.ends_with('\n') {
            result.push('\n');
        }
    }

    result
}
```

### Integration Points

```yaml
DEPENDENCIES (already in Cargo.toml):
  - git2 = { version = "0.19", features = ["vendored-libgit2"] }
  - serde = { version = "1.0", features = ["derive"] }
  - serde_json = "1.0"
  - thiserror = "2.0"
  - clap = { version = "4.5", features = ["derive", "cargo"] }

NO NEW DEPENDENCIES NEEDED

IMPORTS NEEDED in add.rs:
  - use crate::core::{Layer, ProjectContext, Result, JinError};
  - use crate::git::{JinRepo, ObjectOps};
  - use crate::staging::*;

STAGING MODULE:
  - entry.rs: StagedEntry struct (complete)
  - index.rs: StagingIndex (enhanced this milestone)
  - router.rs: route_to_layer(), validate_routing_options() (complete)
  - workspace.rs: File reading utilities (new this milestone)
  - gitignore.rs: .gitignore management (new this milestone)

GIT MODULE:
  - repo.rs: JinRepo wrapper
  - objects.rs: ObjectOps for create_blob()
  - No changes needed to git module

CORE MODULE:
  - error.rs: Add new error variants
  - config.rs: ProjectContext for active mode/scope
  - layer.rs: Layer enum and precedence

CLI MODULE:
  - args.rs: AddArgs struct (verify complete)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build with no warnings
```

### Level 3: Unit Tests (Component Validation)

```bash
# Run staging module tests
cargo test staging::                  # All staging tests

# Run specific test files
cargo test staging::workspace::       # Workspace file operations
cargo test staging::gitignore::       # Gitignore management
cargo test staging::index::           # Staging index operations
cargo test staging::entry::           # Staged entry (existing)
cargo test staging::router::          # Layer routing (existing)

# Run command tests
cargo test commands::add::            # jin add command tests

# Run with output for debugging
cargo test staging:: -- --nocapture

# Expected: All tests pass
```

### Level 4: Integration Testing

```bash
# Full test suite
cargo test

# Verify existing tests still pass (regression)
cargo test core::                     # Core module tests
cargo test git::                      # Git module tests

# Manual testing in temp directory
cd /tmp && mkdir jin-test && cd jin-test
git init  # Create project repo
jin init  # Initialize Jin

# Test basic staging
echo '{"test": true}' > .claude/config.json
jin mode create claude
jin mode use claude
jin add .claude/config.json --mode
cat .jin/staging/index.json  # Verify staging index

# Test routing variations
jin add .editorconfig --scope=language:javascript
jin add README.jin.md --global

# Verify .gitignore
cat .gitignore  # Should have managed block

# Cleanup
cd /tmp && rm -rf jin-test

# Expected: All manual tests produce correct output
```

### Level 5: Full Validation

```bash
# Complete validation pipeline
cargo fmt -- --check && \
cargo clippy -- -D warnings && \
cargo build && \
cargo test

# Expected: All commands succeed with zero errors
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo test staging::` all tests pass
- [ ] `cargo test commands::add::` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] `jin add <file>` stages to Project Base layer
- [ ] `jin add <file> --mode` stages to Mode Base layer
- [ ] `jin add <file> --mode --project` stages to Mode → Project layer
- [ ] `jin add <file> --mode --scope=<s>` stages to Mode → Scope layer
- [ ] `jin add <file> --mode --scope=<s> --project` stages to Mode → Scope → Project layer
- [ ] `jin add <file> --scope=<s>` stages to Scope Base layer (untethered)
- [ ] `jin add <file> --global` stages to Global Base layer
- [ ] Multiple files can be staged in one command
- [ ] Directories are recursively staged
- [ ] Staging index persists to `.jin/staging/index.json`
- [ ] Content hash (blob OID) is correct for file content
- [ ] File mode (executable bit) is correctly detected

### Error Handling Validation

- [ ] Error if file doesn't exist
- [ ] Error if path is a symlink
- [ ] Error if file is tracked by project's Git
- [ ] Error if --mode without active mode
- [ ] Error if --project without --mode
- [ ] Error if --global with other layer flags
- [ ] Error if no files specified
- [ ] All error messages are clear and actionable

### .gitignore Validation

- [ ] Staged files are added to `.gitignore` managed block
- [ ] Managed block has correct format (# --- JIN MANAGED START/END ---)
- [ ] Duplicate entries are deduplicated
- [ ] Existing .gitignore content outside block is preserved
- [ ] .gitignore is created if it doesn't exist

### Code Quality Validation

- [ ] All new methods have doc comments
- [ ] Error handling uses JinError types consistently
- [ ] No unwrap() in library code (only in tests)
- [ ] Platform-specific code uses cfg attributes
- [ ] Tests cover all edge cases
- [ ] Follows existing code patterns in staging module

---

## Anti-Patterns to Avoid

- ❌ Don't read files from Jin repo path - read from workspace (project working dir)
- ❌ Don't check Git tracking in Jin repo - check in project's Git repo
- ❌ Don't store blob OID as raw bytes - store as 40-char hex string
- ❌ Don't modify .gitignore outside the managed block
- ❌ Don't stage directories as entries - stage individual files
- ❌ Don't use unwrap() in library code - propagate errors with ?
- ❌ Don't forget to call staging.save() after modifications
- ❌ Don't skip validation - check all preconditions before staging
- ❌ Don't create blobs without reading content first (bare repo has no workdir)
- ❌ Don't assume Unix - handle Windows paths and permissions

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Staging module structure already exists with complete entry.rs and router.rs
- Layer routing logic is fully implemented and tested
- Git integration via ObjectOps trait is complete
- Clear PRD requirements (Section 9.1 routing table)
- All dependencies already in Cargo.toml
- Existing test patterns to follow
- Error types already defined (just need additions)
- ProjectContext for active mode/scope is complete
- Research provides comprehensive patterns for implementation

**Remaining Risks:**
- Git tracking detection edge cases (submodules, nested repos)
- Cross-platform file mode handling (Windows has no executable bit)
- .gitignore parsing edge cases (comments, patterns)

---

## Research Artifacts

Research has been completed covering:

| Topic | Key Insights |
|-------|--------------|
| **Git Staging Patterns** | Content-addressable storage, blob hashing, index persistence |
| **Layer Routing** | Kubernetes contexts, Terraform workspaces, Cobra+Viper patterns |
| **Rust Implementation** | git2 blob creation, serde JSON, thiserror, tempfile testing |
| **.gitignore Management** | Managed block pattern, atomic writes, deduplication |

Key external references:
- Git Index Format: https://git-scm.com/docs/index-format
- git2 Documentation: https://docs.rs/git2
- PRD Section 8.1: .gitignore managed block specification
- PRD Section 9.1: Layer routing table

---

## Appendix: Routing Decision Matrix

| Command | Target Layer | Ref Path |
|---------|--------------|----------|
| `jin add <file>` | Project Base (7) | `refs/jin/layers/project/<project>/` |
| `jin add <file> --global` | Global Base (1) | `refs/jin/layers/global/` |
| `jin add <file> --mode` | Mode Base (2) | `refs/jin/layers/mode/<mode>/` |
| `jin add <file> --mode --project` | Mode → Project (5) | `refs/jin/layers/mode/<mode>/project/<project>/` |
| `jin add <file> --scope=<s>` | Scope Base (6) | `refs/jin/layers/scope/<scope>/` |
| `jin add <file> --mode --scope=<s>` | Mode → Scope (3) | `refs/jin/layers/mode/<mode>/scope/<scope>/` |
| `jin add <file> --mode --scope=<s> --project` | Mode → Scope → Project (4) | `refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>/` |

---

## Appendix: Test Case Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== Workspace Tests ==========

    #[test]
    fn test_read_file_success() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, b"content").unwrap();

        let content = read_file(&file).unwrap();
        assert_eq!(content, b"content");
    }

    #[test]
    fn test_read_file_not_found() {
        let result = read_file(Path::new("/nonexistent/file.txt"));
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_is_symlink() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();

        assert!(!is_symlink(&file).unwrap());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let link = temp.path().join("link.txt");
            symlink(&file, &link).unwrap();
            assert!(is_symlink(&link).unwrap());
        }
    }

    // ========== Gitignore Tests ==========

    #[test]
    fn test_ensure_in_managed_block_creates_block() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        ensure_in_managed_block(Path::new(".claude/")).unwrap();

        let content = std::fs::read_to_string(".gitignore").unwrap();
        assert!(content.contains(MANAGED_START));
        assert!(content.contains(".claude/"));
        assert!(content.contains(MANAGED_END));
    }

    #[test]
    fn test_ensure_in_managed_block_preserves_existing() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        std::fs::write(".gitignore", "node_modules/\n").unwrap();
        ensure_in_managed_block(Path::new(".claude/")).unwrap();

        let content = std::fs::read_to_string(".gitignore").unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains(".claude/"));
    }

    // ========== Routing Tests ==========

    #[test]
    fn test_route_default_to_project_base() {
        let options = RoutingOptions::default();
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ProjectBase);
    }

    #[test]
    fn test_route_mode_requires_active_mode() {
        let options = RoutingOptions { mode: true, ..Default::default() };
        let context = ProjectContext::default(); // No active mode
        let result = route_to_layer(&options, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_route_mode_with_active_mode() {
        let options = RoutingOptions { mode: true, ..Default::default() };
        let context = ProjectContext {
            mode: Some("claude".to_string()),
            ..Default::default()
        };
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ModeBase);
    }

    // ========== Staging Integration Tests ==========

    #[test]
    fn test_stage_file_creates_blob() {
        let temp = TempDir::new().unwrap();
        let jin_repo_path = temp.path().join(".jin-repo");
        let repo = JinRepo::create_at(&jin_repo_path).unwrap();

        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{\"key\": \"value\"}").unwrap();

        let content = std::fs::read(&file).unwrap();
        let oid = repo.create_blob(&content).unwrap();

        // Verify blob exists
        let blob = repo.find_blob(oid).unwrap();
        assert_eq!(blob.content(), b"{\"key\": \"value\"}");
    }

    #[test]
    fn test_staging_index_persistence() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        std::fs::create_dir_all(".jin/staging").unwrap();

        let mut index = StagingIndex::new();
        index.add(StagedEntry::new(
            PathBuf::from("test.json"),
            Layer::ProjectBase,
            "abc123".to_string(),
        ));
        index.save().unwrap();

        let loaded = StagingIndex::load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.get(Path::new("test.json")).is_some());
    }
}
```
