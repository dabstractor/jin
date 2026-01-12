# Product Requirement Prompt (PRP): Import Command

---

## Goal

**Feature Goal**: Implement `jin import` command that enables users to migrate Git-tracked files into Jin's layer-based management system while preserving file metadata and providing atomic rollback capability.

**Deliverable**: Complete `jin import` command with CLI wiring, Git integration, staging system integration, gitignore management, and comprehensive testing.

**Success Definition**:
- Users can import Git-tracked files into Jin staging with a single command
- Files are removed from Git index and added to Jin's bare repository
- Import operations are atomic with automatic rollback on failure
- Files are added to .gitignore managed block to prevent re-tracking by Git
- Command supports batch operations with per-file error handling
- Comprehensive integration tests cover all scenarios

## User Persona

**Target User**: Developer who has been tracking configuration files in Git and wants to migrate them to Jin's layer-based management system.

**Use Case**: Developer has existing Git-tracked configuration files (e.g., `.vscode/settings.json`, `.dev/config.json`) and wants to bring them under Jin's management to leverage layer-specific configurations.

**User Journey**:
1. Developer identifies Git-tracked files to migrate: `git ls-files | grep config`
2. Developer runs import command: `jin import .vscode/settings.json .dev/config.json`
3. Jin validates files are Git-tracked
4. Jin removes files from Git index (keeps in workspace)
5. Jin stages files to appropriate layer
6. Jin updates .gitignore to prevent Git from re-tracking
7. Developer commits staged files: `jin commit -m "Migrate configs to Jin"`

**Pain Points Addressed**:
- **Manual Migration**: Eliminates need to manually `git rm --cached` and edit .gitignore
- **Metadata Loss**: Preserves executable bits and file modes during migration
- **Safety**: Atomic rollback prevents partial migrations that corrupt state
- **Discoverability**: Clear error messages guide users to use `jin add` for non-Git files

## Why

- **Migration Path**: Provides smooth onboarding for existing Git-tracked files to Jin's layer system
- **Symmetry with Export**: Import and export are inverse operations, enabling bidirectional Git/Jin workflows
- **Developer Experience**: Single command replaces multi-step Git manipulation process
- **Layer System Integration**: Imported files become full participants in layer-based configuration management

## What

`jin import` moves files from Git tracking to Jin's staging system:

```bash
# Import specific files
jin import config.json settings.yaml

# Import with force flag (skip modified check)
jin import --force config.json

# Import entire directory (auto-expanded)
jin import .vscode/
```

### Success Criteria

- [ ] Files validated as Git-tracked before import (using `git ls-files`)
- [ ] Files removed from Git index using `git rm --cached`
- [ ] File content read and stored as Git blobs in Jin's bare repository
- [ ] Files staged to Jin's staging index with target layer and metadata
- [ ] Files added to .gitignore managed block
- [ ] Atomic rollback: on error, all previously removed files re-added to Git
- [ ] Per-file error handling: one file failure doesn't stop batch processing
- [ ] Clear error messages guide users (e.g., "Use `jin add` instead" for non-Git files)
- [ ] Integration tests cover: success paths, error paths, rollback, batch operations

---

## All Needed Context

### Context Completeness Check

**Passes "No Prior Knowledge" Test**: This PRP provides complete context including:
- Exact command patterns from existing similar commands (export, add, diff)
- Full Git plumbing command references with specific URLs
- Layer system architecture and routing patterns
- Staging system integration points
- Test framework and helper utilities
- File structure and naming conventions
- Specific gotchas and anti-patterns to avoid

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# === External Documentation ===

- url: https://git-scm.com/docs/git-hash-object
  why: Git plumbing command for creating blob objects from file content
  critical: Use `-w` flag to actually write objects to database, not just compute hash

- url: https://git-scm.com/docs/git-update-index
  why: Git plumbing command for manipulating the index directly
  critical: Use `--remove --cached` pattern to remove files from index while keeping workspace copy

- url: https://git-scm.com/docs/git-ls-files
  why: Git plumbing command for listing tracked files in the index
  critical: Use `--cached` to show only files in Git's index (not working directory)

- url: https://git-scm.com/docs/git-diff-index
  why: Git plumbing command for checking if files differ from HEAD
  critical: Exit code 1 indicates file has modifications, 0 means clean

# === Codebase Documentation ===

- docfile: plan/P4M5/research/context_import_export_research.md
  why: Comprehensive 42 KB research document covering import/export workflows, Git integration patterns
  section: Full document - contains 8-step import workflow, atomic operations, Git integration strategy

- docfile: plan/P4M5/PRP.md
  why: Product Requirements Document for all utility commands (diff, log, context, import, export)
  section: Import/Export sections - defines behavior and success criteria

- docfile: plan/P4M5/research/CONTEXT_IMPORT_EXPORT_SUMMARY.md
  why: Quick reference with key implementation steps for import/export
  section: Implementation Steps section - contains concise workflow summary

- docfile: plan/docs/LAYER_SYSTEM.md
  why: Explains the 9-layer hierarchy and how import fits into the system
  section: Layer Precedence and Routing sections

- docfile: plan/docs/COMMANDS.md
  why: Complete command specification with examples for all Jin commands
  section: Utility section - contains command patterns and examples

# === Implementation Patterns ===

- file: src/commands/export.rs
  why: The symmetric command to import - shows reverse operations for rollback
  pattern: Atomic operations with rollback, staging removal, Git index manipulation
  gotcha: Export uses `git add` while import uses `git rm --cached`

- file: src/commands/add.rs
  why: Shows the staging pattern that import should follow after Git removal
  pattern: File validation, directory expansion, layer routing, staging index operations
  gotcha: Add validates files are NOT Git-tracked, import validates they ARE Git-tracked

- file: src/commands/import_cmd.rs
  why: The existing implementation that needs completion/enhancement
  pattern: Command structure, validation logic, rollback mechanism
  gotcha: Core logic exists but needs layer flags and comprehensive testing

- file: src/commands/diff.rs
  why: Shows layer comparison and content reading patterns
  pattern: Layer resolution, Git tree operations, output formatting

- file: src/commands/log.rs
  why: Shows Git history traversal and commit reading patterns
  pattern: Revwalk usage, commit parsing, layer reference resolution

- file: src/cli/mod.rs
  why: Main CLI structure using clap derive API
  pattern: Commands enum, ImportArgs wiring (line 65-66)
  gotcha: ImportArgs only has `files: Vec<String>` and `force: bool` - no layer flags

- file: src/cli/args.rs
  why: Contains ImportArgs struct definition (lines 114-123)
  pattern: Simple Vec<String> for files, bool for force flag
  gotcha: ImportArgs lacks layer routing flags that AddArgs has (mode, scope, project, global)

- file: src/core/layer.rs
  why: Defines the Layer enum and routing logic
  pattern: 9-layer hierarchy, ref_path() method, storage_path() method
  gotcha: Each layer has specific naming rules for refs and storage

- file: src/git/repo.rs
  why: JinRepo wrapper around git2::Repository
  pattern: open_or_create(), inner() accessor, namespace isolation (refs/jin/)
  gotcha: Always use open_or_create() for safety, not open()

- file: src/git/tree.rs
  why: TreeOps trait for reading files from Git trees
  pattern: read_file_from_tree(), walk_tree_pre(), get_tree_entry()
  gotcha: Tree operations require tree OID, not commit OID

- file: src/git/objects.rs
  why: ObjectOps trait for creating Git objects
  pattern: create_blob(), create_tree(), create_commit()
  gotcha: create_blob() returns git2::Oid which must be converted to String for storage

- file: src/staging/index.rs
  why: StagingIndex manages staged files before commit
  pattern: load(), add(), remove(), save(), HashMap-based storage
  gotcha: Always call save() after modifications to persist to disk

- file: src/staging/workspace.rs
  why: Workspace file operations utilities
  pattern: read_file(), is_git_tracked(), get_file_mode(), walk_directory()
  gotcha: is_git_tracked() uses `git ls-files` command internally

# === Testing Patterns ===

- file: tests/cli_diff.rs
  why: Integration test example for a utility command
  pattern: TestFixture usage, jin() command helper, assertion patterns

- file: tests/cli_reset.rs
  why: Integration test example for a workspace modification command
  pattern: Setup, execute, assert cycle for command testing

- file: tests/common/fixtures.rs
  why: Test helper utilities and fixtures
  pattern: TestFixture, jin_init(), setup_test_repo(), create_commit_in_repo()
  gotcha: Set JIN_DIR environment variable to .jin_global for test isolation

- file: tests/common/assertions.rs
  why: Custom assertions for Jin-specific state
  pattern: assert_workspace_file(), assert_staging_contains(), assert_jin_initialized()
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── Cargo.toml                    # Project dependencies (clap, git2, tempfile, assert_cmd, predicates)
├── src/
│   ├── main.rs                   # CLI entry point, runs commands
│   ├── cli/
│   │   ├── mod.rs                # Commands enum (Import variant at line 66)
│   │   └── args.rs               # ImportArgs struct (lines 114-123)
│   ├── commands/
│   │   ├── import_cmd.rs         # EXISTING IMPLEMENTATION - needs completion
│   │   ├── export.rs             # Symmetric command - shows rollback pattern
│   │   ├── add.rs                # Shows staging pattern after Git removal
│   │   ├── diff.rs               # Shows layer operations
│   │   └── ...
│   ├── core/
│   │   ├── layer.rs              # Layer enum and routing
│   │   └── mod.rs                # Re-exports, JinError, Result types
│   ├── git/
│   │   ├── repo.rs               # JinRepo wrapper
│   │   ├── tree.rs               # TreeOps trait
│   │   ├── objects.rs            # ObjectOps trait
│   │   └── refs.rs               # RefOps trait
│   ├── staging/
│   │   ├── index.rs              # StagingIndex
│   │   └── workspace.rs          # File utilities (is_git_tracked, read_file, etc.)
│   └── lib.rs                    # Module declarations
└── tests/
    ├── cli_diff.rs               # Integration test example
    ├── cli_reset.rs              # Integration test example
    ├── common/
    │   ├── fixtures.rs           # Test helpers
    │   └── assertions.rs         # Custom assertions
    └── ...
```

### Desired Codebase Tree (Changes Highlighted)

```bash
# No new files needed - import_cmd.rs already exists
# Changes needed:
# 1. src/commands/import_cmd.rs  - Complete/enhance existing implementation
# 2. tests/cli_import.rs         - NEW: Integration tests for import command
```

### Known Gotchas of This Codebase

```rust
// CRITICAL: ImportArgs does NOT have layer routing flags
// Unlike AddArgs (which has mode, scope, project, global flags),
// ImportArgs only has files: Vec<String> and force: bool
// This means import currently defaults to ProjectBase layer only
// Consider adding layer flags to match AddArgs functionality

// CRITICAL: Git command execution pattern
// Use std::process::Command for git rm --cached
// The git2 crate doesn't support "remove from index but keep workspace" natively
// Command::new("git").arg("rm").arg("--cached").arg(path)

// CRITICAL: is_git_tracked() returns Result<bool>
// Must use ? operator to propagate errors
// if !is_git_tracked(path)? { return Err(...) }

// CRITICAL: Rollback must be ALL or NOTHING
// If file 3 fails, rollback files 1 and 2 by re-adding to Git
// Use Command::new("git").arg("add").arg(file) for rollback

// CRITICAL: StagingIndex must be saved after modifications
// staging.add(entry);
// staging.save()?; // REQUIRED or changes lost

// CRITICAL: .gitignore operations can fail gracefully
// ensure_in_managed_block() returns Result<()>
// Log warning but don't fail import if .gitignore update fails

// CRITICAL: Directory expansion happens before processing
// If path.is_dir(), call walk_directory() to get all files
// Then process each file individually with atomic rollback

// CRITICAL: File mode preservation
// get_file_mode() returns u32 (0o100644 for regular, 0o100755 for executable)
// Store this in StagedEntry.mode to preserve executable bit

// CRITICAL: Error handling - be specific with JinError variants
// NotFound for missing files
// Symlink for symlink files
// Other for Git command failures with descriptive messages

// CRITICAL: Test isolation requires JIN_DIR environment variable
// Set env("JIN_DIR", temp_dir.join(".jin_global")) in all tests
// Prevents test interference with user's actual Jin installation
```

---

## Implementation Blueprint

### Data Models and Structure

**No new models needed** - Import uses existing types:

```rust
// From src/core/layer.rs - already exists
pub enum Layer {
    GlobalBase,        // Layer 1
    ModeBase,         // Layer 2
    ModeScope,        // Layer 3
    ModeScopeProject, // Layer 4
    ModeProject,       // Layer 5
    ScopeBase,        // Layer 6
    ProjectBase,      // Layer 7 (default for import)
    UserLocal,        // Layer 8
    WorkspaceActive,  // Layer 9
}

// From src/staging/index.rs - already exists
pub struct StagedEntry {
    pub path: PathBuf,           // File path relative to project root
    pub target_layer: Layer,     // Which layer this file belongs to
    pub content_hash: String,    // Git blob OID as string
    pub mode: u32,               // File mode (0o100644 or 0o100755)
    pub operation: StagedOperation, // AddOrModify, Remove, etc.
}

// From src/cli/args.rs - already exists
pub struct ImportArgs {
    pub files: Vec<String,  // Files/directories to import
    pub force: bool,        // Skip modification check
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: ENHANCE src/cli/args.rs - ImportArgs
  - ADD: Layer routing flags to ImportArgs (optional enhancement)
  - FOLLOW pattern: AddArgs struct (mode, scope, project, global flags)
  - NAMING: Match AddArgs naming conventions exactly
  - PLACEMENT: After ExportArgs, before RepairArgs (around line 124)
  - RATIONALE: Currently import only targets ProjectBase - flags enable layer control

Task 2: COMPLETE src/commands/import_cmd.rs - Main Execute Function
  - IMPLEMENT: Full execute() function with batch processing and atomic rollback
  - FOLLOW pattern: export.rs for rollback logic, add.rs for staging logic
  - NAMING: execute(args: ImportArgs) -> Result<()>
  - DEPENDENCIES: ImportArgs from Task 1 (if enhanced), StagingIndex, JinRepo
  - KEY STEPS:
    1. Validate files specified
    2. Load ProjectContext and JinRepo
    3. Load StagingIndex
    4. Process each file with atomic rollback
    5. Save staging index
    6. Print summary
  - PLACEMENT: src/commands/import_cmd.rs (existing file)

Task 3: IMPLEMENT src/commands/import_cmd.rs - import_file() Helper
  - IMPLEMENT: Single file import logic with validation and Git removal
  - FOLLOW pattern: export.rs export_file() function for structure
  - NAMING: import_file(path, layer, repo, staging, git_removed_list, force) -> Result<()>
  - KEY STEPS:
    1. Validate file (exists, not symlink, is Git-tracked, not modified unless --force)
    2. Remove from Git index using git rm --cached
    3. Read content from workspace
    4. Create blob in Jin repo
    5. Add to staging index
    6. Update .gitignore managed block
  - ROLLBACK: Track in git_removed_list for rollback on failure
  - PLACEMENT: In import_cmd.rs, after execute()

Task 4: IMPLEMENT src/commands/import_cmd.rs - validate_import_file()
  - IMPLEMENT: Comprehensive file validation before import
  - FOLLOW pattern: add.rs validation logic (inverted - checking IS tracked)
  - NAMING: validate_import_file(path: &Path, force: bool) -> Result<()>
  - VALIDATIONS:
    1. File exists (JinError::NotFound if not)
    2. Not a directory (should be expanded before calling)
    3. Not a symlink (JinError::Symlink)
    4. IS Git-tracked (use is_git_tracked() from workspace.rs)
    5. Not modified in Git (unless --force flag, use is_git_modified())
  - ERROR MESSAGES: Clear guidance ("Use `jin add` instead" for non-Git files)
  - PLACEMENT: In import_cmd.rs, after import_file()

Task 5: IMPLEMENT src/commands/import_cmd.rs - Git Helper Functions
  - IMPLEMENT: remove_from_git(), is_git_modified(), rollback_git_removals()
  - FOLLOW pattern: export.rs Git command execution pattern
  - NAMING:
    - remove_from_git(path: &Path) -> Result<()>
    - is_git_modified(path: &Path) -> Result<bool>
    - rollback_git_removals(files: &[PathBuf])
  - remove_from_git():
    - Use Command::new("git").arg("rm").arg("--cached").arg(path)
    - Check output.status.success()
    - Return JinError::Other with stderr on failure
  - is_git_modified():
    - Use Command::new("git").arg("diff-index").arg("--quiet").arg("HEAD").arg(path)
    - Return Ok(!output.status.success()) (exit code 1 = modified)
  - rollback_git_removals():
    - Iterate through files and run Command::new("git").arg("add").arg(file)
    - Log each rollback operation to stderr
    - Continue on error (best-effort rollback)
  - PLACEMENT: In import_cmd.rs, after validate_import_file()

Task 6: IMPLEMENT src/commands/import_cmd.rs - Unit Tests
  - IMPLEMENT: Comprehensive #[cfg(test)] mod tests
  - FOLLOW pattern: import_cmd.rs existing test structure
  - TEST CASES:
    1. test_execute_no_files - Empty file list returns error
    2. test_validate_not_found - Non-existent file returns NotFound
    3. test_validate_is_directory - Directory returns error
    4. test_validate_symlink - Symlink returns Symlink error
    5. test_validate_not_tracked - Non-Git file suggests jin add
    6. test_validate_modified - Modified file errors without --force
    7. test_validate_modified_with_force - Modified file OK with --force
    8. test_format_layer_name - Verify layer name formatting
  - FIXTURES: Use tempfile::TempDir for test isolation
  - PLACEMENT: End of import_cmd.rs file

Task 7: CREATE tests/cli_import.rs - Integration Tests
  - IMPLEMENT: Full integration tests with real Git repositories
  - FOLLOW pattern: tests/cli_diff.rs and tests/cli_reset.rs
  - TEST CASES:
    1. test_import_single_file - Import one Git-tracked file
    2. test_import_multiple_files - Batch import multiple files
    3. test_import_directory - Import directory (auto-expand)
    4. test_import_not_tracked - Error for non-Git file with helpful message
    5. test_import_modified_file - Error for modified file without --force
    6. test_import_modified_with_force - Success with --force flag
    7. test_import_not_initialized - Error when Jin not initialized
    8. test_import_rollback - Verify rollback on failure
    9. test_import_symlink - Error for symlink files
    10. test_import_gitignore_update - Verify .gitignore is updated
  - HELPERS: Use tests/common/fixtures.rs (TestFixture, jin_init(), etc.)
  - ASSERTIONS: Use tests/common/assertions.rs (assert_staging_contains(), etc.)
  - SETUP: Each test creates temp Git repo, initializes Jin, creates test files
  - PLACEMENT: tests/cli_import.rs (new file)

Task 8: UPDATE src/main.rs - Ensure Command Wiring
  - VERIFY: Import command is wired to execute function
  - FIND pattern: Other command executions in main.rs match/case
  - ENSURE: Commands::Import variant calls import_cmd::execute()
  - CODE SHOULD BE: Commands::Import(args) => import_cmd::execute(args)?
  - PLACEMENT: src/main.rs, main() function
```

### Implementation Patterns & Key Details

```rust
// === PATTERN 1: Command Execute Structure ===
// From src/commands/export.rs and add.rs

pub fn execute(args: ImportArgs) -> Result<()> {
    // 1. Input validation
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Load context and repository
    let context = ProjectContext::load().unwrap_or_default();
    let repo = JinRepo::open_or_create()?;
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 3. Determine target layer (using default routing or enhanced flags)
    let target_layer = route_to_layer(&RoutingOptions::default(), &context)?;

    // 4. Batch processing with atomic rollback
    let mut imported_count = 0;
    let mut errors = Vec::new();
    let mut git_removed_files = Vec::new(); // Track for rollback

    for path_str in &args.files {
        let path = PathBuf::from(path_str);

        // Expand directories
        let files_to_import = if path.is_dir() {
            walk_directory(&path)?
        } else {
            vec![path.clone()]
        };

        for file_path in files_to_import {
            match import_file(&file_path, target_layer, &repo, &mut staging,
                            &mut git_removed_files, args.force) {
                Ok(_) => imported_count += 1,
                Err(e) => {
                    // CRITICAL: Rollback all previous operations
                    if !git_removed_files.is_empty() {
                        rollback_git_removals(&git_removed_files);
                    }
                    errors.push(format!("{}: {}", file_path.display(), e));
                    break; // Stop processing on first error
                }
            }
        }
    }

    // 5. Save staging index
    if imported_count > 0 {
        staging.save()?;
    }

    // 6. Print summary
    if imported_count > 0 {
        println!("Imported {} file(s) to {} layer",
                imported_count, format_layer_name(target_layer));
    }

    // 7. Handle errors
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if imported_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to import", errors.len()),
            });
        }
    }

    Ok(())
}

// === PATTERN 2: Single File Import ===
// Combines Git operations with Jin staging

fn import_file(
    path: &Path,
    layer: Layer,
    repo: &JinRepo,
    staging: &mut StagingIndex,
    git_removed_files: &mut Vec<PathBuf>,
    force: bool,
) -> Result<()> {
    // 1. Validate file
    validate_import_file(path, force)?;

    // 2. Remove from Git index (keeps in workspace)
    remove_from_git(path)?;
    git_removed_files.push(path.to_path_buf()); // Track for rollback

    // 3. Read content from workspace
    let content = read_file(path)?;

    // 4. Create blob in Jin's bare repository
    let oid = repo.create_blob(&content)?;

    // 5. Get file mode (preserve executable bit)
    let mode = get_file_mode(path);

    // 6. Create staged entry
    let entry = StagedEntry {
        path: path.to_path_buf(),
        target_layer: layer,
        content_hash: oid.to_string(),
        mode,
        operation: StagedOperation::AddOrModify,
    };

    // 7. Add to staging index
    staging.add(entry);

    // 8. Update .gitignore (warn but don't fail)
    if let Err(e) = ensure_in_managed_block(path) {
        eprintln!("Warning: Could not update .gitignore: {}", e);
    }

    Ok(())
}

// === PATTERN 3: File Validation ===
// Inverted from add.rs - validates IS Git-tracked

fn validate_import_file(path: &Path, force: bool) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // Check not a directory
    if path.is_dir() {
        return Err(JinError::Other(format!(
            "{} is a directory, not a file",
            path.display()
        )));
    }

    // Check not a symlink
    if is_symlink(path)? {
        return Err(JinError::Symlink {
            path: path.display().to_string(),
        });
    }

    // CRITICAL: Check IS tracked by Git (opposite of add.rs)
    if !is_git_tracked(path)? {
        return Err(JinError::Other(format!(
            "{} is not tracked by Git. Use `jin add` instead.",
            path.display()
        )));
    }

    // Check for uncommitted changes (unless --force)
    if !force && is_git_modified(path)? {
        return Err(JinError::Other(format!(
            "{} has uncommitted changes in Git. Use --force to import anyway.",
            path.display()
        )));
    }

    Ok(())
}

// === PATTERN 4: Git Command Execution ===
// Using std::process::Command for git rm --cached

fn remove_from_git(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("rm")
        .arg("--cached")
        .arg(path)
        .output()
        .map_err(|e| JinError::Other(format!("Failed to execute git rm: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JinError::Other(format!(
            "git rm --cached failed for {}: {}",
            path.display(),
            stderr
        )));
    }

    Ok(())
}

fn is_git_modified(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("diff-index")
        .arg("--quiet")
        .arg("HEAD")
        .arg("--")
        .arg(path)
        .output()
        .map_err(|e| JinError::Other(format!("Failed to execute git diff-index: {}", e)))?;

    // Exit code 0 = no changes, 1 = has changes
    Ok(!output.status.success())
}

// === PATTERN 5: Atomic Rollback ===
// Re-add all removed files to Git index

fn rollback_git_removals(files: &[PathBuf]) {
    eprintln!("Error occurred, rolling back changes...");

    for file in files {
        let output = Command::new("git").arg("add").arg(file).output();

        match output {
            Ok(output) if output.status.success() => {
                eprintln!("Rolled back: {}", file.display());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Failed to rollback {}: {}", file.display(), stderr);
            }
            Err(e) => {
                eprintln!("Failed to rollback {}: {}", file.display(), e);
            }
        }
    }
}

// === PATTERN 6: Layer Name Formatting ===
// For user-facing output

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

// === GOTCHA: Git2 Crate Limitation ===
// git2 doesn't support "remove from index but keep workspace"
// Must use external git command for rm --cached
// But use git2 for all other operations (blob creation, tree reading, etc.)
```

### Integration Points

```yaml
STAGING_INDEX:
  - file: .jin/staging/index.json
  - operations: load(), add(), save()
  - entry: StagedEntry with path, layer, content_hash, mode, operation

GIT_INDEX:
  - command: git rm --cached <path>
  - effect: Removes from Git tracking, keeps in workspace
  - rollback: git add <path> (on error)

GITIGNORE:
  - file: .gitignore
  - function: ensure_in_managed_block() from staging/workspace.rs
  - effect: Adds file to "# Managed by Jin" section
  - failure: Warn but don't fail import

LAYER_ROUTING:
  - default: ProjectBase (Layer 7)
  - function: route_to_layer() from staging/workspace.rs
  - enhancement: Add --mode, --scope, --project, --global flags to ImportArgs

JIN_REPOSITORY:
  - location: ~/.jin/ (bare repository)
  - operation: create_blob() stores content as Git object
  - return: git2::Oid converted to String for storage
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after completing import_cmd.rs modifications
cargo check --bin jin 2>&1 | head -50

# Expected: No compilation errors. Fix before proceeding.

# Auto-format and fix linting
cargo fmt
cargo clippy --bin jin -- -D warnings 2>&1 | head -50

# Expected: Zero warnings. If warnings exist, READ output and fix.

# Project-wide validation
cargo fmt --verbose
cargo clippy -- -D warnings

# Expected: All files formatted, no clippy warnings.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run unit tests in import_cmd.rs
cargo test --lib import_cmd 2>&1

# Expected: All unit tests pass.
# Test cases:
# - test_execute_no_files
# - test_validate_not_found
# - test_validate_is_directory
# - test_validate_symlink
# - test_validate_not_tracked
# - test_validate_modified
# - test_validate_modified_with_force
# - test_format_layer_name

# Full unit test suite for commands module
cargo test --lib commands:: 2>&1

# Expected: All command tests pass.

# With output
cargo test --lib import_cmd -- --nocapture 2>&1

# Expected: Tests pass with visible output for debugging.
```

### Level 3: Integration Testing (System Validation)

```bash
# Run integration tests for import command
cargo test --test cli_import 2>&1

# Expected: All integration tests pass.
# Test cases:
# - test_import_single_file
# - test_import_multiple_files
# - test_import_directory
# - test_import_not_tracked
# - test_import_modified_file
# - test_import_modified_with_force
# - test_import_not_initialized
# - test_import_rollback
# - test_import_symlink
# - test_import_gitignore_update

# All integration tests
cargo test --test cli_import -- --nocapture 2>&1

# Expected: Comprehensive test coverage with visible output.

# Manual testing with real Git repository
cd /tmp && mkdir test-import && cd test-import
git init
echo "test" > config.json
git add config.json
git commit -m "Initial"

# Initialize Jin
cargo run -- init

# Import file
cargo run -- import config.json

# Expected: "Imported 1 file(s) to project-base layer"

# Verify file removed from Git index
git ls-files

# Expected: config.json NOT listed (removed from Git)

# Verify file in Jin staging
cat .jin/staging/index.json | jq .

# Expected: config.json listed in staging index

# Verify .gitignore updated
cat .gitignore

# Expected: config.json in "# Managed by Jin" section

# Test rollback behavior
echo "test2" > config2.json
git add config2.json
git commit -m "Add config2"
echo "modified" >> config2.json

# Import without --force should fail
cargo run -- import config2.json 2>&1

# Expected: Error about uncommitted changes, config2.json NOT in Jin staging
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test atomic rollback with partial failure
cd /tmp/test-rollback
git init
echo "f1" > file1.json
echo "f2" > file2.json
echo "f3" > file3.json
git add .
git commit -m "Initial"

# Initialize Jin
cargo run -- init

# Create scenario where file3 will fail (make it a symlink)
ln -s /nonexistent file3.json

# Attempt batch import
cargo run -- import file1.json file2.json file3.json 2>&1

# Expected:
# - file1.json and file2.json rolled back (re-added to Git)
# - Error message about file3.json being a symlink
# - Verify with git ls-files (all three should still be tracked)

# Test layer routing (if enhancement implemented)
cargo run -- import --mode config-mode.json
cargo run -- import --global config-global.json
cargo run -- import --scope env:dev config-scope.json

# Verify files staged to correct layers via .jin/staging/index.json

# Performance test with large batch
mkdir /tmp/large-import && cd /tmp/large-import
git init
for i in {1..100}; do echo "content $i" > file$i.json; done
git add .
git commit -m "Initial"

cargo run -- init
time cargo run -- import *.json

# Expected: All 100 files imported successfully
# Measure: Should complete in reasonable time (< 10 seconds)

# Test .gitignore edge cases
# - Multiple .gitignore files in subdirectories
# - Existing .gitignore with custom patterns
# - Conflicting managed blocks

# Test binary file import
dd if=/dev/urandom of=binary.bin bs=1024 count=100
git add binary.bin
git commit -m "Add binary"
cargo run -- import binary.bin

# Expected: Binary file imported correctly, content hash matches
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --lib import_cmd`
- [ ] All integration tests pass: `cargo test --test cli_import`
- [ ] No linting errors: `cargo clippy -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Manual testing successful: Git ls-files shows removal, Jin staging shows addition

### Feature Validation

- [ ] Git-tracked files successfully imported to Jin staging
- [ ] Files removed from Git index but remain in workspace
- [ ] Files added to .gitignore managed block
- [ ] Atomic rollback works: failure restores previous files to Git
- [ ] Error messages guide users (e.g., "Use `jin add` instead")
- [ ] Batch operations process multiple files correctly
- [ ] Directory expansion works for importing directories
- [ ] --force flag allows importing modified files
- [ ] All success criteria from "What" section met

### Code Quality Validation

- [ ] Follows existing command patterns (export.rs, add.rs)
- [ ] File placement matches desired codebase tree
- [ ] Anti-patterns avoided (see below)
- [ ] Dependencies properly imported (JinRepo, StagingIndex, etc.)
- [ ] Error handling uses specific JinError variants
- [ ] Code is self-documenting with clear function names

### Documentation & Deployment

- [ ] Function-level documentation comments present
- [ ] Module-level documentation explains command purpose
- [ ] Error messages are clear and actionable
- [ ] Integration tests serve as usage examples

---

## Anti-Patterns to Avoid

- ❌ **Don't use git2 for `git rm --cached`**: git2 doesn't support this operation - use `Command::new("git")`
- ❌ **Don't skip rollback on error**: Atomic operations require all-or-nothing semantics
- ❌ **Don't fail on .gitignore errors**: Log warning but continue with import
- ❌ **Don't validate files are NOT Git-tracked**: Import validates they ARE tracked (opposite of add)
- ❌ **Don't forget to save StagingIndex**: Changes lost without `staging.save()?`
- ❌ **Don't process directories without expansion**: Call `walk_directory()` first
- ❌ **Don't ignore file mode**: Use `get_file_mode()` to preserve executable bit
- ❌ **Don't use generic error messages**: Guide users ("Use `jin add` instead")
- ❌ **Don't skip is_git_modified() check**: Require --force for modified files
- ❌ **Don't forget test isolation**: Set `JIN_DIR` environment variable in all tests

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Rationale**:
- Comprehensive research across codebase, external documentation, and similar commands
- Existing import_cmd.rs provides solid foundation - completion not from-scratch
- Clear patterns from export.rs and add.rs to follow
- Specific gotchas identified with solutions provided
- Complete test coverage defined with examples
- Atomic rollback pattern well-researched and documented

**Remaining Risk**:
- Layer routing flags for ImportArgs (optional enhancement) not critical for MVP
- Test complexity for atomic rollback scenarios requires careful fixture setup
- Git command error handling across different Git versions may need adjustment

**Validation**: The completed implementation should enable an AI agent unfamiliar with the codebase to implement the import command successfully using only this PRP content and codebase access.
