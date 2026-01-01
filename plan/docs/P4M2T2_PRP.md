# PRP: P4.M2.T2 - Add Command

---

## Goal

**Feature Goal**: Implement `jin add` command that stages files to appropriate layers based on command-line flags and active context.

**Deliverable**: Complete `jin add` command implementation with layer routing, file validation, staging integration, and comprehensive test coverage.

**Success Definition**:
- Files can be staged to all 7 target layers using appropriate flags
- Files are validated (existence, not symlink, not Git-tracked) before staging
- Staged entries persist to staging index with Git blob hashes
- Automatic .gitignore management prevents Git from tracking staged files
- Full test coverage of routing logic, validation, and error handling

## User Persona

**Target User**: Developer using Jin to manage multi-layer configuration files.

**Use Case**: Developer wants to stage configuration files to specific layers (mode, scope, project, global) before committing them.

**User Journey**:
1. Developer creates/modifies configuration files in workspace
2. Developer runs `jin add config.json` (or with layer flags)
3. Jin validates files and creates Git blobs for content
4. Files are added to staging index with target layer metadata
5. Files are automatically added to .gitignore managed block
6. Developer runs `jin commit` to persist staged changes

**Pain Points Addressed**:
- No manual layer management needed
- Clear error messages for invalid operations
- Prevents accidental Git tracking of Jin-managed files
- Flexible layer targeting with simple flag combinations

## Why

- **Core Workflow Prerequisite**: Add command is the first step in the init → add → commit → apply workflow
- **Layer Abstraction**: Users work with files, Jin handles layer routing automatically
- **Safety First**: File validation prevents staging invalid or dangerous files (symlinks)
- **Git Integration**: Automatic .gitignore management keeps Jin and Git separate
- **Partial Success**: Individual file failures don't block entire operation

## What

### Command Signature

```bash
jin add <files> [--mode] [--scope=<scope>] [--project] [--global]
```

### Arguments

| Argument | Type | Description |
|----------|------|-------------|
| `files` | Vec<String> | List of files or directories to stage (required) |
| `--mode` | bool | Target mode layer (requires active mode in context) |
| `--scope=<scope>` | Option<String> | Target scope layer (can be combined with --mode) |
| `--project` | bool | Target project layer (requires --mode flag) |
| `--global` | bool | Target global layer (mutually exclusive with --mode) |

### Layer Routing Table

| Command Flags | Target Layer | Storage Path |
|---------------|--------------|--------------|
| (no flags) | ProjectBase (7) | `refs/jin/layers/project/<project>/` |
| `--mode` | ModeBase (2) | `refs/jin/layers/mode/<mode>/` |
| `--mode --project` | ModeProject (5) | `refs/jin/layers/mode/<mode>/project/<project>/` |
| `--scope=<scope>` | ScopeBase (6) | `refs/jin/layers/scope/<scope>/` |
| `--mode --scope=<scope>` | ModeScope (3) | `refs/jin/layers/mode/<mode>/scope/<scope>/` |
| `--mode --scope=<scope> --project` | ModeScopeProject (4) | `refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>/` |
| `--global` | GlobalBase (1) | `refs/jin/layers/global/` |

### File Validation Rules

1. **Existence**: File must exist at specified path
2. **Type**: Must be a regular file (directories are expanded recursively)
3. **Symlinks**: Not supported (security risk)
4. **Git Tracking**: File must not be tracked by project's Git repository
5. **Content**: Read as text and hashed into Git blob

### Success Criteria

- [x] Validates all input files before staging
- [x] Creates Git blobs for all valid files
- [x] Updates staging index with staged entries
- [x] Adds files to .gitignore managed block
- [x] Reports number of files staged successfully
- [x] Handles partial failures (some files succeed, others fail)
- [x] Returns proper exit codes (0 = success, non-zero = error)
- [x] Shows appropriate error messages for each failure type

## All Needed Context

### Context Completeness Check

**Status**: PASS - This PRP provides complete context for implementing the add command. The implementation already exists and is documented below for reference and validation.

### Documentation & References

```yaml
# MUST READ - Critical implementation files

- file: src/commands/add.rs
  why: Complete add command implementation (330 lines) - use as primary reference
  pattern: Command structure, error handling, staging workflow
  gotcha: "Directory handling: use walk_directory() to expand, don't process directly"

- file: src/cli/args.rs
  why: AddArgs struct definition - exact argument structure for CLI parsing
  pattern: Clap derive Args macro usage with bool flags
  section: Lines 5-26

- file: src/staging/router.rs
  why: Layer routing logic and validation - determines target layer from flags
  pattern: route_to_layer() function and RoutingOptions struct
  gotcha: "Context.require_mode() is called when --mode flag is used"

- file: src/staging/entry.rs
  why: StagedEntry and StagedOperation type definitions
  pattern: Struct fields for staging metadata (path, target_layer, content_hash, mode, operation)

- file: src/staging/index.rs
  why: StagingIndex API for loading/saving/persisting staged changes
  pattern: load(), save(), add(), get() methods
  gotcha: "Use unwrap_or_else(|_| StagingIndex::new()) for load to handle missing index"

- file: src/staging/workspace.rs
  why: File validation utilities (is_symlink, is_git_tracked, read_file, get_file_mode)
  pattern: Individual validation functions with Result<> returns
  gotcha: "Git-tracked check requires opening project repo - may fail in non-Git dirs"

- file: src/staging/gitignore.rs
  why: Automatic .gitignore management for staged files
  pattern: ensure_in_managed_block() function
  gotcha: "Only warns on failure - doesn't return error to allow staging to continue"

- file: src/core/layer.rs
  why: Layer enum definition with all 9 variants
  pattern: Layer::GlobalBase, Layer::ModeBase, etc.
  gotcha: "format_layer_name() helper converts Layer to display string (snake_case)"

- file: src/core/error.rs
  why: JinError enum variants for error handling
  pattern: NotFound, Symlink, GitTracked, Config, NotInitialized, StagingFailed
  gotcha: "Use specific error variants for better user messages"

- file: src/git/mod.rs
  why: JinRepo and ObjectOps traits for Git blob creation
  pattern: JinRepo::open_or_create(), create_blob()
  gotcha: "Jin repo is at ~/.jin/ by default (JIN_DIR env var override)"

- file: src/commands/mod.rs
  why: Command registration and dispatch pattern
  pattern: Commands::Add(args) => add::execute(args)
  gotcha: "All commands export execute() function as entry point"

- file: tests/common/fixtures.rs
  why: Test helper utilities for integration testing
  pattern: TestFixture, setup_test_repo(), jin_init()
  gotcha: "Must set JIN_DIR env var to isolated directory in tests"

- file: tests/cli_basic.rs
  why: Integration test patterns for CLI commands
  pattern: jin().arg("add").assert().success()
  gotcha: "Use predicates::str::contains for output assertions"

- file: tests/core_workflow.rs
  why: End-to-end workflow test examples
  pattern: test_add_files_to_mode_layer()
  gotcha: "Tests run full init → mode → add → commit → apply workflow"

- file: PRD.md
  why: Product requirements for layer system and command behavior
  section: Section 9.1 - "jin add: Stage files to appropriate layers"
  gotcha: "Layer precedence: Global(1) < ModeBase(2) < ModeScope(3) < ModeScopeProject(4) < ModeProject(5) < ScopeBase(6) < ProjectBase(7) < UserLocal(8) < WorkspaceActive(9)"

- docfile: plan/docs/LAYER_SYSTEM.md
  why: Detailed layer system architecture and routing rules
  section: Complete layer reference with storage paths
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs              # AddArgs struct (lines 5-26)
│   └── mod.rs               # Command enum with Add variant
├── commands/
│   ├── add.rs               # COMPLETE IMPLEMENTATION (330 lines)
│   ├── mod.rs               # Command dispatcher
│   ├── init.rs              # Reference for command patterns
│   └── status.rs            # Reference for status display
├── staging/
│   ├── entry.rs             # StagedEntry, StagedOperation types
│   ├── gitignore.rs         # .gitignore management
│   ├── index.rs             # StagingIndex persistence
│   ├── mod.rs               # Module exports
│   ├── router.rs            # Layer routing logic
│   └── workspace.rs         # File validation utilities
├── core/
│   ├── error.rs             # JinError variants
│   ├── layer.rs             # Layer enum (9 variants)
│   └── mod.rs               # Context types
└── git/
    ├── mod.rs               # JinRepo, ObjectOps traits
    └── repo.rs              # Repository operations

tests/
├── cli_basic.rs             # Basic CLI integration tests
├── core_workflow.rs         # Full workflow tests
├── common/
│   ├── fixtures.rs          # Test helpers
│   ├── assertions.rs        # Custom assertions
│   └── mod.rs               # Test module exports
```

### Desired Codebase State

**Status**: ALREADY IMPLEMENTED - No new files needed.

The `jin add` command is **fully implemented** in `src/commands/add.rs` with:
- 330 lines of production code
- Complete layer routing via `src/staging/router.rs`
- File validation with security checks (symlinks, Git-tracked)
- Staging integration with persistence
- Automatic .gitignore management
- 7 unit tests covering validation and routing
- Integration tests in `tests/cli_basic.rs` and `tests/core_workflow.rs`

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: ProjectContext::load() returns different errors
// - NotInitialized: Jin not set up (return error immediately)
// - Other errors: Use ProjectContext::default() as fallback
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
    Err(_) => ProjectContext::default(),
};

// CRITICAL: validate_routing_options() must be called BEFORE route_to_layer()
// Order matters: validation checks for invalid flag combinations first
validate_routing_options(&options)?;
let target_layer = route_to_layer(&options, &context)?;

// CRITICAL: walk_directory() expands directories recursively
// Don't process directories directly - always expand first
let files_to_stage = if path.is_dir() {
    walk_directory(&path)?
} else {
    vec![path.clone()]
};

// CRITICAL: ensure_in_managed_block() only warns on failure
// Staging should continue even if .gitignore update fails
if let Err(e) = ensure_in_managed_block(&file_path) {
    eprintln!("Warning: Could not update .gitignore: {}", e);
}

// CRITICAL: is_git_tracked() requires project Git repository
// May fail gracefully in directories without Git
if is_git_tracked(path)? {
    return Err(JinError::GitTracked { path: path.display().to_string() });
}

// CRITICAL: Jin repo location is configurable via JIN_DIR env var
// Tests MUST set JIN_DIR to isolated directory
std::env::set_var("JIN_DIR", temp.path().join(".jin_global"));

// CRITICAL: Symlinks are rejected for security reasons
// Never follow symlinks when reading file content
if is_symlink(path)? {
    return Err(JinError::Symlink { path: path.display().to_string() });
}

// CRITICAL: Mode flag requires active mode in context
// Context has require_mode() method that checks this
if options.mode {
    context.require_mode()?;  // Returns error if no active mode
}

// CRITICAL: StagingIndex load may fail if file doesn't exist
// Use unwrap_or_else to create new index in that case
let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

// CRITICAL: Each file stage operation is independent
// Partial success is allowed - collect errors and report at end
for file_path in files_to_stage {
    match stage_file(&file_path, target_layer, &repo, &mut staging) {
        Ok(_) => staged_count += 1,
        Err(e) => errors.push(format!("{}: {}", file_path.display(), e)),
    }
}
```

## Implementation Blueprint

### Data Models and Structures

The add command uses existing types from the codebase:

```rust
// CLI Arguments (src/cli/args.rs)
pub struct AddArgs {
    pub files: Vec<String>,      // Files/directories to stage
    pub mode: bool,               // Target mode layer
    pub scope: Option<String>,    // Target scope
    pub project: bool,            // Target project layer
    pub global: bool,             // Target global layer
}

// Layer Routing (src/staging/router.rs)
pub struct RoutingOptions {
    pub mode: bool,
    pub scope: Option<String>,
    pub project: bool,
    pub global: bool,
}

// Staging Entry (src/staging/entry.rs)
pub struct StagedEntry {
    pub path: PathBuf,            // Workspace file path
    pub target_layer: Layer,      // Destination layer
    pub content_hash: String,     // Git blob OID (hex string)
    pub mode: u32,                // File mode (0o100644 or 0o100755)
    pub operation: StagedOperation, // AddOrModify, Delete, or Rename
}

pub enum StagedOperation {
    AddOrModify,
    Delete,
    Rename,
}

// Layer Enum (src/core/layer.rs)
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
```

### Implementation Tasks

**Status**: ALL TASKS COMPLETE - Implementation exists in `src/commands/add.rs`

The following documents the existing implementation structure:

```yaml
# Task 1: DEFINE AddArgs in src/cli/args.rs
  status: COMPLETE (lines 5-26)
  implementation:
    - AddArgs struct with 5 fields (files, mode, scope, project, global)
    - Uses clap::Args derive macro for CLI parsing
    - All flags are optional (Vec for files, bool for flags, Option for scope)
  file: src/cli/args.rs
  lines: 5-26

# Task 2: IMPLEMENT layer routing in src/staging/router.rs
  status: COMPLETE (full file)
  implementation:
    - route_to_layer() function determines target layer from flags
    - validate_routing_options() checks for invalid combinations
    - RoutingOptions struct mirrors AddArgs structure
    - Full test coverage (10 tests) for all routing scenarios
  file: src/staging/router.rs
  key_functions:
    - route_to_layer(options, context) -> Result<Layer>
    - validate_routing_options(options) -> Result<()>

# Task 3: IMPLEMENT file validation in src/staging/workspace.rs
  status: COMPLETE (module implemented)
  implementation:
    - read_file() reads file content with error handling
    - is_symlink() checks for symbolic links (security)
    - is_git_tracked() checks if file is tracked by Git
    - get_file_mode() returns file permissions (0o100644/0o100755)
    - walk_directory() recursively expands directories
  file: src/staging/workspace.rs
  key_functions:
    - read_file(path) -> Result<String>
    - is_symlink(path) -> Result<bool>
    - is_git_tracked(path) -> Result<bool>
    - get_file_mode(path) -> u32
    - walk_directory(path) -> Result<Vec<PathBuf>>

# Task 4: IMPLEMENT .gitignore management in src/staging/gitignore.rs
  status: COMPLETE (module implemented)
  implementation:
    - ensure_in_managed_block() adds file to .gitignore
    - Parses existing .gitignore to find managed block
    - Creates managed block if it doesn't exist
    - Only warns on failure (non-blocking)
  file: src/staging/gitignore.rs
  key_functions:
    - ensure_in_managed_block(path) -> Result<()>

# Task 5: IMPLEMENT main add command in src/commands/add.rs
  status: COMPLETE (330 lines)
  implementation:
    - execute() function handles complete workflow
    - Validates files, creates blobs, updates staging, manages .gitignore
    - Handles partial success (reports errors for individual files)
    - Returns proper exit codes
    - 7 unit tests covering validation, routing, error cases
  file: src/commands/add.rs
  key_functions:
    - execute(args: AddArgs) -> Result<()>
    - stage_file(path, layer, repo, staging) -> Result<()>
    - validate_file(path) -> Result<()>
    - format_layer_name(layer) -> &'static str

# Task 6: REGISTER command in src/commands/mod.rs
  status: COMPLETE
  implementation:
    - Commands::Add(AddArgs) variant in Commands enum
    - Command dispatcher: Commands::Add(args) => add::execute(args)
  file: src/commands/mod.rs

# Task 7: ADD integration tests
  status: COMPLETE
  implementation:
    - tests/cli_basic.rs: test_add_subcommand(), test_add_with_mode_flag()
    - tests/core_workflow.rs: test_add_files_to_mode_layer()
    - Tests verify CLI parsing, layer routing, staging integration
  files:
    - tests/cli_basic.rs
    - tests/core_workflow.rs
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Command execution workflow (src/commands/add.rs:34-128)
pub fn execute(args: AddArgs) -> Result<()> {
    // 1. Validate input
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Load context with fallback for non-init errors
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. Validate routing options BEFORE using them
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
    validate_routing_options(&options)?;

    // 4. Determine target layer (may fail if no active mode)
    let target_layer = route_to_layer(&options, &context)?;

    // 5. Open Jin repository (creates if doesn't exist)
    let repo = JinRepo::open_or_create()?;

    // 6. Load or create staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. Process files with partial success handling
    let mut staged_count = 0;
    let mut errors = Vec::new();

    for path_str in &args.files {
        let path = PathBuf::from(path_str);

        // Expand directories recursively
        let files_to_stage = if path.is_dir() {
            match walk_directory(&path) {
                Ok(files) => files,
                Err(e) => {
                    errors.push(format!("{}: {}", path.display(), e));
                    continue;
                }
            }
        } else {
            vec![path.clone()]
        };

        // Stage each file independently
        for file_path in files_to_stage {
            match stage_file(&file_path, target_layer, &repo, &mut staging) {
                Ok(_) => {
                    // Non-blocking .gitignore update
                    if let Err(e) = ensure_in_managed_block(&file_path) {
                        eprintln!("Warning: Could not update .gitignore: {}", e);
                    }
                    staged_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", file_path.display(), e));
                }
            }
        }
    }

    // 8. Persist staging index
    staging.save()?;

    // 9. Report results
    if staged_count > 0 {
        println!("Staged {} file(s) to {} layer", staged_count, format_layer_name(target_layer));
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if staged_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to stage", errors.len()),
            });
        }
    }

    Ok(())
}

// PATTERN: Single file staging (src/commands/add.rs:131-157)
fn stage_file(path: &Path, layer: Layer, repo: &JinRepo, staging: &mut StagingIndex) -> Result<()> {
    // 1. Validate file (exists, not symlink, not git-tracked)
    validate_file(path)?;

    // 2. Read content from workspace
    let content = read_file(path)?;

    // 3. Create Git blob in Jin repository
    let oid = repo.create_blob(&content)?;

    // 4. Get file mode (executable or regular)
    let mode = get_file_mode(path);

    // 5. Create staged entry
    let entry = StagedEntry {
        path: path.to_path_buf(),
        target_layer: layer,
        content_hash: oid.to_string(),
        mode,
        operation: StagedOperation::AddOrModify,
    };

    // 6. Add to staging index
    staging.add(entry);

    Ok(())
}

// PATTERN: File validation (src/commands/add.rs:159-189)
fn validate_file(path: &Path) -> Result<()> {
    // Check existence
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // Check not a directory (should have been expanded)
    if path.is_dir() {
        return Err(JinError::Other(format!("{} is a directory", path.display())));
    }

    // Security check: no symlinks
    if is_symlink(path)? {
        return Err(JinError::Symlink { path: path.display().to_string() });
    }

    // Check not tracked by project Git
    if is_git_tracked(path)? {
        return Err(JinError::GitTracked { path: path.display().to_string() });
    }

    Ok(())
}

// PATTERN: Layer routing (src/staging/router.rs:29-63)
pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
    // Global flag takes precedence (mutually exclusive)
    if options.global {
        return Ok(Layer::GlobalBase);
    }

    // Mode flag requires active mode
    if options.mode {
        context.require_mode()?;  // Fails if no active mode

        if options.scope.is_some() {
            if options.project {
                // Mode + Scope + Project
                Ok(Layer::ModeScopeProject)
            } else {
                // Mode + Scope
                Ok(Layer::ModeScope)
            }
        } else if options.project {
            // Mode + Project
            Ok(Layer::ModeProject)
        } else {
            // Mode only
            Ok(Layer::ModeBase)
        }
    } else if options.scope.is_some() {
        // Untethered scope (no mode)
        Ok(Layer::ScopeBase)
    } else {
        // Default: project base
        Ok(Layer::ProjectBase)
    }
}

// PATTERN: Routing validation (src/staging/router.rs:66-82)
pub fn validate_routing_options(options: &RoutingOptions) -> Result<()> {
    // Global flag is mutually exclusive with other layer flags
    if options.global && (options.mode || options.scope.is_some() || options.project) {
        return Err(JinError::Config(
            "Cannot combine --global with other layer flags".to_string(),
        ));
    }

    // Project flag requires mode flag
    if options.project && !options.mode {
        return Err(JinError::Config(
            "--project requires --mode flag".to_string(),
        ));
    }

    Ok(())
}

// GOTCHA: Context loading handles NotInitialized specially
// Other errors use default context to allow operation
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
    Err(_) => ProjectContext::default(),
};

// GOTCHA: StagingIndex may not exist on first run
// Use unwrap_or_else to create new index in that case
let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

// GOTCHA: .gitignore updates are non-blocking
// Warn but don't fail staging if .gitignore update fails
if let Err(e) = ensure_in_managed_block(&file_path) {
    eprintln!("Warning: Could not update .gitignore: {}", e);
}

// GOTCHA: Mode flag requires active mode in context
// Use context.require_mode() for this check
if options.mode {
    context.require_mode()?;  // Returns NoActiveMode error if not set
}
```

### Integration Points

```yaml
STAGING_SYSTEM:
  load: "StagingIndex::load() from ~/.jin/staging/index.json"
  save: "staging.save() persists to same location"
  add: "staging.add(StagedEntry) adds or replaces entry"

GIT_REPOSITORY:
  open: "JinRepo::open_or_create() at ~/.jin/ (JIN_DIR env var)"
  blob: "repo.create_blob(content) returns git2::Oid"
  location: "Bare repository with refs under refs/jin/layers/"

PROJECT_CONTEXT:
  load: "ProjectContext::load() from .jin/context"
  fallback: "ProjectContext::default() for non-init errors"
  mode: "context.mode: Option<String> - active mode name"
  scope: "context.scope: Option<String> - active scope name"

GITIGNORE_MANAGEMENT:
  ensure: "ensure_in_managed_block(path) adds to .gitignore"
  block: "# === BEGIN JIN MANAGED === ... # === END JIN MANAGED ==="
  nonblocking: "Only warns on failure"

ROUTING_OPTIONS:
  validate: "validate_routing_options() checks flag combinations"
  route: "route_to_layer() returns Layer based on flags + context"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after any code changes - fix before proceeding
cargo fmt --check                    # Format check
cargo clippy -- -D warnings          # Lint with warnings as errors
cargo check                          # Type check without building

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# For specific files:
cargo fmt --check src/commands/add.rs
cargo clippy --bin jin -- -D warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test add command unit tests
cargo test add::tests --verbose

# Test routing logic
cargo test staging::router::tests --verbose

# Test staging operations
cargo test staging::index::tests --verbose
cargo test staging::workspace::tests --verbose

# Full test suite
cargo test --verbose

# Expected: All tests pass. If failing, debug root cause and fix.

# Test coverage (optional)
cargo llvm-cov --lcov --output-path lcov.info
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual testing - basic workflow
cd /tmp && mkdir test_jin && cd test_jin
jin init                                    # Initialize Jin
echo '{"key": "value"}' > config.json       # Create test file
jin add config.json                         # Stage to project layer
jin status                                  # Verify staging
jin commit -m "Add config"                  # Commit staged changes

# Test layer routing
jin mode create claude                      # Create mode
jin mode set claude                         # Activate mode
jin add config.json --mode                  # Stage to mode layer
jin status                                  # Verify mode layer staging

# Test directory staging
mkdir -p configs/subdir
echo "data" > configs/a.txt
echo "data" > configs/subdir/b.txt
jin add configs/                            # Stage directory recursively
jin status                                  # Verify all files staged

# Test error cases
jin add nonexistent.txt                     # Should fail (not found)
ln -s config.json link.txt                  # Create symlink
jin add link.txt                            # Should fail (symlink)

# Test .gitignore management
cat .gitignore                              # Should have managed block with config.json

# Expected: All operations work correctly with appropriate error messages.
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test all layer routing combinations
jin add config.json                         # Default: ProjectBase (7)
jin add config.json --mode                  # ModeBase (2)
jin add config.json --mode --project        # ModeProject (5)
jin add config.json --scope=language:js     # ScopeBase (6)
jin add config.json --mode --scope=lang:js  # ModeScope (3)
jin add config.json --mode --scope=lang:js --project  # ModeScopeProject (4)
jin add config.json --global                # GlobalBase (1)

# Test invalid flag combinations (should fail)
jin add config.json --mode --global         # Error: mutually exclusive
jin add config.json --project               # Error: --project requires --mode

# Test partial success scenarios
echo "a" > a.txt && echo "b" > b.txt
ln -s a.txt link.txt
jin add a.txt b.txt link.txt nonexistent.txt
# Should stage a.txt and b.txt, report errors for link.txt and nonexistent.txt

# Test idempotency (adding same file twice)
jin add config.json
jin add config.json                         # Should update, not duplicate
jin status                                  # Should show single entry

# Test with different file types
echo "script" > script.sh && chmod +x script.sh
jin add script.sh                           # Should detect executable mode (0o100755)

# Test large directories
mkdir -p large_dir && seq 1 100 | xargs -I{} touch large_dir/file{}.txt
jin add large_dir/                           # Should stage all 100 files

# Expected: All validation scenarios pass with correct behavior.
```

## Final Validation Checklist

### Technical Validation

- [x] All 4 validation levels completed successfully
- [x] All tests pass: `cargo test --verbose`
- [x] No linting errors: `cargo clippy -- -D warnings`
- [x] No formatting issues: `cargo fmt --check`
- [x] No type errors: `cargo check`

### Feature Validation

- [x] Files can be staged to all 7 target layers using appropriate flags
- [x] File validation works (existence, symlink, Git-tracked checks)
- [x] Directory expansion works recursively
- [x] Staging index persists correctly
- [x] .gitignore management adds files to managed block
- [x] Partial success handling works (some files succeed, others fail)
- [x] Layer routing follows the routing table correctly
- [x] Invalid flag combinations return appropriate errors
- [x] Active mode requirement for --mode flag is enforced

### Code Quality Validation

- [x] Follows existing codebase patterns (execute function, Result<> return)
- [x] Error handling uses specific JinError variants
- [x] File structure matches conventions (src/commands/add.rs)
- [x] Anti-patterns avoided (no hardcoded values, proper error handling)
- [x] Dependencies properly imported from staging, core, git modules
- [x] Code is well-documented with doc comments
- [x] Tests follow established patterns (TestFixture, predicates)

### Documentation & Deployment

- [x] Doc comments explain command purpose and behavior
- [x] Error messages are clear and actionable
- [x] Success output shows layer name and file count
- [x] No environment variables needed (JIN_DIR is optional override)

---

## Anti-Patterns to Avoid

- ❌ Don't process directories directly - always use `walk_directory()` to expand
- ❌ Don't follow symlinks - reject them for security reasons
- ❌ Don't hardcode layer paths - use `Layer::ref_path()` and `Layer::storage_path()`
- ❌ Don't call `route_to_layer()` before `validate_routing_options()`
- ❌ Don't fail on .gitignore errors - warn but continue staging
- ❌ Don't use `ProjectContext::load()?` directly - handle NotInitialized specially
- ❌ Don't assume staging index exists - use `unwrap_or_else(|_| StagingIndex::new())`
- ❌ Don't stop on first file error - collect all errors and report at end
- ❌ Don't forget to expand directories - users expect recursive staging
- ❌ Don't mix up --project flag logic - requires --mode flag
- ❌ Don't use synchronous file operations in async contexts (not applicable here)
- ❌ Don't catch all exceptions with generic catch - use specific JinError variants

---

## Implementation Status

**Status**: COMPLETE - Production Ready

The `jin add` command is fully implemented with:
- Complete functionality in `src/commands/add.rs` (330 lines)
- Full layer routing support via `src/staging/router.rs`
- Comprehensive file validation (security, Git tracking)
- Staging integration with persistence
- Automatic .gitignore management
- 7 unit tests covering core functionality
- Integration tests in `tests/cli_basic.rs` and `tests/core_workflow.rs`
- Error handling for all failure scenarios

**Confidence Score**: 10/10

The implementation is production-ready and has been validated through comprehensive testing.

---

## Research Artifacts

Research conducted for this PRP is stored in:
- `plan/P4M2T2/research/` - Additional research notes and findings

## References

- PRD Section 9.1: `jin add` command specification
- Layer System Documentation: `plan/docs/LAYER_SYSTEM.md`
- Implementation: `src/commands/add.rs`
- Routing Logic: `src/staging/router.rs`
- Test Patterns: `tests/common/fixtures.rs`
