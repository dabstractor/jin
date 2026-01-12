# Product Requirement Prompt (PRP): Implement `jin mv` Command

---

## Goal

**Feature Goal**: Implement a `jin mv` command that allows users to rename/move staged files within Jin's layer system while preserving staging state and properly updating all Jin metadata.

**Deliverable**: A fully functional `jin mv` command that:
- Moves/renames files in the staging index
- Optionally moves files in the workspace
- Supports layer routing flags (--mode, --scope, --project, --global)
- Supports --dry-run and --force flags
- Properly updates .gitignore managed blocks
- Uses existing `StagedOperation::Rename` enum variant

**Success Definition**:
- Command successfully updates staging entries from source path to destination path
- File content hash is preserved during move (no re-reading of files)
- .gitignore managed block is updated (old path removed, new path added)
- Workspace files are moved when --force is used
- All validation follows existing patterns from `jin rm`
- Dry-run mode correctly previews changes without modifying state
- Error handling matches existing commands with helpful messages

## User Persona

**Target User**: Developer using Jin to manage configuration files across layers

**Use Case**: Developer wants to reorganize their configuration structure by renaming files or moving them to different directories, while preserving their staged state in Jin.

**User Journey**:
1. Developer has files staged for commit (e.g., `config.json`)
2. Developer wants to rename/move the file (e.g., to `settings/config.json`)
3. Developer runs `jin mv config.json settings/config.json`
4. Jin updates staging index to reflect new path
5. Developer commits with renamed/moved file in correct location

**Pain Points Addressed**:
- Currently, users must `jin rm` + `jin add` to rename/move files
- This loses staging metadata and requires re-reading files
- The manual approach is error-prone and doesn't preserve file history

## Why

- **User Efficiency**: Provides single-command operation for common file reorganization tasks
- **Staging State Preservation**: Maintains staging metadata without re-reading files or recomputing hashes
- **Consistency**: Aligns with Git's `git mv` command as a familiar pattern
- **Layer Management**: Enables reorganizing files within Jin's layer system
- **Completeness**: Completes the file operation command suite alongside `add` and `rm`

---

## All Needed Context

### Context Completeness Check

*"If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"*

**YES** - This PRP provides:
- Complete implementation pattern from `jin rm` (primary reference)
- Exact file locations and naming conventions
- Full CLI argument structure
- Staging system API with method signatures
- Layer routing logic
- Testing patterns and fixtures
- All validation requirements

### Documentation & References

```yaml
# MUST READ - Critical implementation files

# PRIMARY PATTERN REFERENCE
- file: src/commands/rm.rs
  why: Complete command implementation pattern to follow
  pattern: Execute function structure, validation flow, error handling
  critical: Shows exact staging integration, dry-run pattern, force handling
  notes: |
    - Lines 18-146: Main execute() function with all validation steps
    - Lines 148-178: unstage_file() helper showing staging operations
    - Lines 180-189: Confirmation prompt pattern
    - Lines 206-460: Test patterns for unit tests

# STAGING SYSTEM API
- file: src/staging/entry.rs
  why: StagedEntry and StagedOperation types for rename operations
  pattern: StagedEntry constructor patterns and StagedOperation::Rename variant
  critical: StagedOperation::Rename already exists but needs a constructor method
  notes: |
    - Lines 8-31: StagedEntry and StagedOperation struct definitions
    - Lines 33-60: StagedEntry::new() and StagedEntry::delete() constructors
    - Missing: StagedEntry::rename() constructor - needs to be added

- file: src/staging/index.rs
  why: StagingIndex API for loading/saving/managing staged entries
  pattern: HashMap-based entry management with load/save
  critical: Must use get(), remove(), add(), save() methods

- file: src/staging/router.rs
  why: Layer routing logic for --mode, --scope, --project, --global flags
  pattern: RoutingOptions struct and route_to_layer() function
  critical: Complete routing table with all flag combinations
  notes: |
    - Lines 5-16: RoutingOptions struct definition
    - Lines 18-63: route_to_layer() with routing table
    - Lines 65-82: validate_routing_options() for flag validation

- file: src/staging/gitignore.rs
  why: Functions for updating .gitignore managed blocks
  pattern: remove_from_managed_block() and ensure_in_managed_block()
  critical: Must call both for move operations (remove old, add new)

- file: src/staging/workspace.rs
  why: File system utilities for workspace operations
  pattern: read_file(), get_file_mode(), is_symlink() functions
  critical: Use get_file_mode() to preserve file permissions during move

# CLI STRUCTURE
- file: src/cli/args.rs
  why: Location to add MvArgs struct (after RmArgs, before DiffArgs)
  pattern: Follow RmArgs structure with identical fields
  notes: |
    - Lines 88-117: RmArgs structure (copy for MvArgs)
    - Add between lines 117-119 (after RmArgs, before DiffArgs)

- file: src/cli/mod.rs
  why: Location to add Mv variant to Commands enum
  pattern: Add after Rm, before Diff
  notes: |
    - Line 57: Rm(RmArgs) variant
    - Add new line: Mv(MvArgs),

- file: src/commands/mod.rs
  why: Module declaration and command registration
  pattern: Add mv module import and match arm
  notes: |
    - Add after line 27: pub mod mv;
    - Add after line 43: Commands::Mv(args) => mv::execute(args),

# LAYER SYSTEM
- file: src/lib.rs
  why: Public re-exports and module structure
  pattern: Re-exports JinError, Result, Layer types

- file: src/core/layer.rs (review only)
  why: Layer enum with all 9 layers and ref_path() method
  pattern: Layer enum variants for routing targets
  notes: |
    - GlobalBase, ModeBase, ModeScope, ModeScopeProject, ModeProject
    - ScopeBase, ProjectBase, UserLocal, WorkspaceActive

# TESTING PATTERNS
- file: src/commands/rm.rs (lines 206-460)
  why: Complete unit test suite for command patterns
  pattern: Test fixtures, assertion patterns, error case testing

- file: tests/common/fixtures.rs
  why: Integration test fixtures and setup helpers
  pattern: TestFixture, setup_test_repo(), jin_init(), jin() command builder
  notes: |
    - Lines 11-37: TestFixture struct for isolated test directories
    - Lines 78-82: setup_test_repo() creates initialized Jin project
    - Lines 165-169: jin() returns assert_cmd::Command for testing

- file: tests/common/assertions.rs
  why: Custom assertions for Jin-specific state verification
  pattern: assert_workspace_file*, assert_staging_contains*, assert_layer_ref_exists*
  notes: |
    - Lines 18-35: assert_workspace_file() for content verification
    - Lines 73-98: assert_staging_contains() for staging verification
    - Lines 127-158: assert_layer_ref_exists() for Git ref verification

# EXTERNAL RESEARCH (best practices)
- url: https://git-scm.com/docs/git-mv
  why: Git mv behavior patterns - rename is staged immediately
  critical: Git mv = filesystem mv + staging update (atomic conceptually)

- url: https://doc.rust-lang.org/std/fs/fn.rename.html
  why: Rust std::fs::rename() documentation
  gotcha: Only works on same filesystem; returns error code 18 for cross-device

- url: https://stackoverflow.com/questions/74976656/moving-files-from-place-to-place-in-rust
  why: Cross-filesystem move patterns using copy + delete fallback
  pattern: |
    match fs::rename(src, dst) {
      Ok(()) => Ok(()),
      Err(e) if e.raw_os_error() == Some(18) => {
        fs::copy(src, dst)?;
        fs::remove_file(src)
      }
      Err(e) => Err(e),
    }

- url: https://code.googlescape.org/git/+/v2.9.0-rc2/builtin/mv.c
  why: Git mv source code showing error handling and validation patterns
  critical: Validates source exists, destination doesn't, same file check
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── mod.rs          # Commands enum - add Mv(MvArgs) variant
│   └── args.rs         # Argument structs - add MvArgs struct
├── commands/
│   ├── mod.rs          # Module exports and execute() match - add mv module
│   ├── rm.rs           # PRIMARY REFERENCE - complete pattern to follow
│   └── mv.rs           # TO CREATE - new command implementation
├── staging/
│   ├── mod.rs          # Re-exports all staging types
│   ├── entry.rs        # StagedEntry, StagedOperation - add rename() constructor
│   ├── index.rs        # StagingIndex - get/remove/add/save API
│   ├── router.rs       # route_to_layer, validate_routing_options
│   ├── gitignore.rs    # remove_from_managed_block, ensure_in_managed_block
│   └── workspace.rs    # File utilities (read_file, get_file_mode, etc.)
├── core/
│   ├── error.rs        # JinError enum
│   └── layer.rs        # Layer enum
└── lib.rs              # Public re-exports

tests/
├── common/
│   ├── fixtures.rs     # Test fixtures and setup helpers
│   ├── assertions.rs   # Custom Jin-specific assertions
│   └── mod.rs          # Test common module
└── cli_mv.rs           # TO CREATE - integration tests for mv
```

### Desired Codebase Tree (files to add)

```bash
src/
├── cli/
│   └── args.rs         # MODIFY: Add MvArgs struct (lines ~118-140)
├── cli/
│   └── mod.rs          # MODIFY: Add Mv(MvArgs) to Commands enum (after Rm)
├── commands/
│   ├── mod.rs          # MODIFY: Add pub mod mv; and Mv match arm
│   └── mv.rs           # CREATE: New command implementation (~400 lines)
└── staging/
    └── entry.rs        # MODIFY: Add StagedEntry::rename() constructor

tests/
└── cli_mv.rs           # CREATE: Integration tests (~300 lines)
```

### Known Gotchas of Jin Codebase & Library Quirks

```rust
// CRITICAL: Staging paths are absolute PathBuf values
// When comparing paths from CLI args, convert to PathBuf and canonicalize
let path = PathBuf::from(path_str);
// StagingIndex.get() requires exact PathBuf match

// CRITICAL: Layer routing requires active mode for --mode flag
// Always call context.require_mode()? when mode is requested
// This returns JinError::NoActiveMode if no mode is set

// CRITICAL: RoutingOptions validation happens BEFORE route_to_layer
// Must call validate_routing_options(&options)? first
// Returns errors for invalid flag combinations:
//   - --global with any other flag
//   - --project without --mode

// CRITICAL: StagedEntry::Rename variant exists but has no constructor
// Must add StagedEntry::rename() method to entry.rs:
//   pub fn rename(old_path: PathBuf, new_path: PathBuf, target_layer: Layer, content_hash: String) -> Self

// CRITICAL: .gitignore managed block needs BOTH operations for move
// Must call remove_from_managed_block(&old_path) AND ensure_in_managed_block(&new_path)
// Order: remove old first, then add new

// CRITICAL: Workspace file moves require special handling
// Use std::fs::rename() for same-filesystem (atomic)
// Fall back to copy + remove for cross-filesystem (error 18)
// With --force flag: move actual files
// Without --force: only update staging (like git mv --cached)

// CRITICAL: File mode (permissions) must be preserved
// Use existing get_file_mode() from workspace.rs
// Store mode in StagedEntry (u32, typically 0o100644 or 0o100755)

// CRITICAL: Error collection pattern for partial success
// Collect errors but continue processing other files
// Only return error if ALL files fail
// Show summary of both success and failure counts

// CRITICAL: Dry-run mode must return early WITHOUT saving
// Check args.dry_run at start of file processing loop
// Print what would happen, then return Ok(())
// Do NOT call staging.save() in dry-run mode

// CRITICAL: Confirmation prompt pattern
// Only prompt if --force is NOT set AND workspace files exist
// Prompt message: "Type 'yes' to confirm:"
// Accept case-insensitive "yes" only

// CRITICAL: format_layer_name() helper needed for output
// Copy from rm.rs lines 191-204
// Maps Layer enum to display names (e.g., Layer::ProjectBase -> "project-base")

// CRITICAL: Test isolation with JIN_DIR environment variable
// Set custom JIN_DIR to avoid polluting user's actual ~/.jin
// Use tempfile::TempDir for all test directories
// TempDir must be kept in scope (stored in struct) to prevent cleanup

// CRITICAL: Integration test assertions use custom helpers
// assert_workspace_file(path, file, content)
// assert_workspace_file_exists(path, file)
// assert_workspace_file_not_exists(path, file)
// assert_staging_contains(path, file)
// assert_staging_not_contains(path, file)

// CRITICAL: Command must handle both source AND destination paths
// Args structure needs Vec<String> for paths
// Last argument is destination, rest are sources
// OR: pairs of arguments (src1, dst1, src2, dst2, ...)
// DECISION: Use pairs pattern for batch moves
```

---

## Implementation Blueprint

### Data Models and Structure

The `mv` command requires minimal new data structures - primarily adding to existing types:

```rust
// 1. Add to src/cli/args.rs (after RmArgs, before DiffArgs)
#[derive(Args, Debug)]
pub struct MvArgs {
    /// Source and destination file pairs (src1, dst1, src2, dst2, ...)
    pub files: Vec<String>,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Target project layer
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,

    /// Skip confirmation prompt for workspace moves
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Show what would be moved without doing it
    #[arg(long)]
    pub dry_run: bool,
}

// 2. Add to src/staging/entry.rs (in impl StagedEntry block)
impl StagedEntry {
    /// Create a new staged entry for renaming a file
    ///
    /// Preserves content_hash and mode from the original entry.
    /// The new_path is stored in the path field.
    pub fn rename(
        old_path: PathBuf,
        new_path: PathBuf,
        target_layer: Layer,
        content_hash: String,
        mode: u32,
    ) -> Self {
        Self {
            path: new_path,
            target_layer,
            content_hash,
            mode,
            operation: StagedOperation::Rename,
        }
    }

    /// Check if this entry is a rename operation
    pub fn is_rename(&self) -> bool {
        self.operation == StagedOperation::Rename
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/staging/entry.rs
  - IMPLEMENT: StagedEntry::rename() constructor method
  - IMPLEMENT: StagedEntry::is_rename() helper method
  - FOLLOW pattern: StagedEntry::delete() at lines 45-54
  - SIGNATURE: rename(old_path: PathBuf, new_path: PathBuf, target_layer: Layer, content_hash: String, mode: u32) -> Self
  - PLACEMENT: After StagedEntry::delete() method (after line 54)
  - NOTES: Store new_path in path field, set operation to StagedOperation::Rename

Task 2: MODIFY src/cli/args.rs
  - IMPLEMENT: MvArgs struct with clap derives
  - FOLLOW pattern: RmArgs at lines 88-117
  - NAMING: MvArgs struct, all snake_case fields
  - PLACEMENT: After RmArgs (after line 117), before DiffArgs
  - FIELDS: files: Vec<String>, mode: bool, scope: Option<String>, project: bool, global: bool, force: bool, dry_run: bool

Task 3: MODIFY src/cli/mod.rs
  - ADD: Mv(MvArgs) variant to Commands enum
  - FIND pattern: Rm(RmArgs) at line 57
  - ADD AFTER: Rm variant
  - ADD BEFORE: Diff variant
  - SYNTAX: Mv(MvArgs),

Task 4: MODIFY src/commands/mod.rs
  - ADD: pub mod mv; import statement
  - FIND pattern: pub mod rm; at line 27
  - ADD AFTER: rm module import
  - ADD: Commands::Mv(args) => mv::execute(args), match arm
  - FIND pattern: Commands::Rm(args) => rm::execute(args), at line 43
  - ADD AFTER: Rm match arm

Task 5: CREATE src/commands/mv.rs
  - IMPLEMENT: Complete mv command implementation
  - FOLLOW pattern: src/commands/rm.rs (complete file structure)
  - NAMING: execute(), move_file(), prompt_confirmation(), format_layer_name()
  - FUNCTIONS:
    - pub fn execute(args: MvArgs) -> Result<()> (main entry point)
    - fn move_file(src: &Path, dst: &Path, layer: Layer, staging: &mut StagingIndex, args: &MvArgs) -> Result<()>
    - fn prompt_confirmation(message: &str) -> Result<bool>
    - fn format_layer_name(layer: Layer) -> &'static str
  - DEPENDENCIES: Import from Task 1 (StagedEntry::rename)
  - PLACEMENT: src/commands/mv.rs (new file)
  - ESTIMATED LENGTH: ~400 lines (similar to rm.rs)

Task 6: CREATE tests/cli_mv.rs
  - IMPLEMENT: Integration tests for mv command
  - FOLLOW pattern: tests/cli_basic.rs for structure, rm.rs unit tests for specific cases
  - FIXTURES: use tests/common/fixtures.rs (setup_test_repo, jin, create_mode)
  - ASSERTIONS: use tests/common/assertions.rs (assert_workspace_file*, assert_staging*)
  - TEST CASES:
    - test_mv_single_file() (basic rename)
    - test_mv_with_layer_flags() (layer routing)
    - test_mv_dry_run() (preview mode)
    - test_mv_force() (workspace file move)
    - test_mv_source_not_staged() (error case)
    - test_mv_destination_exists() (error case)
    - test_mv_batch() (multiple file pairs)
  - PLACEMENT: tests/cli_mv.rs (new file)
  - ESTIMATED LENGTH: ~300 lines
```

### Implementation Patterns & Key Details

```rust
// ============================================================
// CRITICAL IMPLEMENTATION PATTERNS - Follow these exactly
// ============================================================

// -----------------------------------------------------------
// PATTERN 1: Main execute() function structure (from rm.rs)
// -----------------------------------------------------------
pub fn execute(args: MvArgs) -> Result<()> {
    // 1. VALIDATE: Check we have file pairs (must be even number)
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }
    if args.files.len() % 2 != 0 {
        return Err(JinError::Other(
            "Files must be specified in source/destination pairs".to_string()
        ));
    }

    // 2. LOAD CONTEXT: Project context for active mode/scope
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. BUILD ROUTING OPTIONS: From command args
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
    validate_routing_options(&options)?;

    // 4. DETERMINE TARGET LAYER: Using routing logic
    let target_layer = route_to_layer(&options, &context)?;

    // 5. OPEN JIN REPO: For consistency with other commands
    let _repo = JinRepo::open_or_create()?;

    // 6. LOAD STAGING INDEX: Get current staging state
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. DRY-RUN MODE: Preview and return early
    if args.dry_run {
        for chunk in args.files.chunks(2) {
            let src = PathBuf::from(&chunk[0]);
            let dst = PathBuf::from(&chunk[1]);
            if staging.get(&src).is_some() {
                let workspace_action = if args.force && src.exists() {
                    "and from workspace"
                } else {
                    "from staging only"
                };
                println!("Would move: {} -> {} ({})",
                    src.display(), dst.display(), workspace_action);
            } else {
                eprintln!("Warning: {} not in staging", src.display());
            }
        }
        return Ok(());
    }

    // 8. CONFIRMATION PROMPT: For workspace moves without --force
    let files_to_move_in_workspace: Vec<(PathBuf, PathBuf)> = args.files
        .chunks(2)
        .filter_map(|chunk| {
            let src = PathBuf::from(&chunk[0]);
            let dst = PathBuf::from(&chunk[1]);
            if staging.get(&src).is_some() && src.exists() && args.force {
                Some((src, dst))
            } else {
                None
            }
        })
        .collect();

    if !files_to_move_in_workspace.is_empty() && !args.force {
        let message = format!(
            "This will move {} file(s) in workspace. Type 'yes' to confirm:",
            files_to_move_in_workspace.len()
        );
        if !prompt_confirmation(&message)? {
            println!("Move cancelled");
            return Ok(());
        }
    }

    // 9. PROCESS FILES: With error collection for partial success
    let mut moved_count = 0;
    let mut errors = Vec::new();

    for chunk in args.files.chunks(2) {
        let src = PathBuf::from(&chunk[0]);
        let dst = PathBuf::from(&chunk[1]);
        match move_file(&src, &dst, target_layer, &mut staging, &args) {
            Ok(_) => moved_count += 1,
            Err(e) => errors.push(format!("{} -> {}: {}", src.display(), dst.display(), e)),
        }
    }

    // 10. SAVE STAGING: Persist all changes
    staging.save()?;

    // 11. PRINT SUMMARY: Show results
    if moved_count > 0 {
        println!("Moved {} file(s) in {} layer",
            moved_count, format_layer_name(target_layer));
    }

    // 12. HANDLE ERRORS: Partial success pattern
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if moved_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to move", errors.len()),
            });
        }
    }

    Ok(())
}

// -----------------------------------------------------------
// PATTERN 2: move_file() helper (staging operations)
// -----------------------------------------------------------
fn move_file(
    src: &Path,
    dst: &Path,
    layer: Layer,
    staging: &mut StagingIndex,
    args: &MvArgs,
) -> Result<()> {
    // 1. VALIDATE SOURCE: Check if file is in staging
    let existing_entry = staging
        .get(src)
        .ok_or_else(|| JinError::NotFound(
            format!("Source file not in staging: {}", src.display())
        ))?;

    // 2. VALIDATE DESTINATION: Check if destination already staged
    if staging.get(dst).is_some() {
        return Err(JinError::Other(
            format!("Destination already in staging: {}", dst.display())
        ));
    }

    // 3. PRESERVE METADATA: Get content hash and mode from existing entry
    let content_hash = existing_entry.content_hash.clone();
    let mode = existing_entry.mode;

    // 4. CREATE RENAME ENTRY: Using new constructor (Task 1)
    let rename_entry = StagedEntry::rename(
        src.to_path_buf(),
        dst.to_path_buf(),
        layer,
        content_hash,
        mode,
    );

    // 5. UPDATE STAGING INDEX: Remove old, add new
    staging.remove(src);
    staging.add(rename_entry);

    // 6. UPDATE GITIGNORE: Remove old path, add new path
    if let Err(e) = remove_from_managed_block(src) {
        eprintln!("Warning: Could not remove {} from .gitignore: {}", src.display(), e);
    }
    if let Err(e) = ensure_in_managed_block(dst) {
        eprintln!("Warning: Could not add {} to .gitignore: {}", dst.display(), e);
    }

    // 7. WORKSPACE MOVE: If --force is set and file exists
    if args.force && src.exists() {
        // Create parent directory if needed
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Atomic rename with cross-filesystem fallback
        match std::fs::rename(src, dst) {
            Ok(()) => Ok(()),
            Err(e) if e.raw_os_error() == Some(18) => {
                // Cross-device link: copy + delete fallback
                std::fs::copy(src, dst)?;
                std::fs::remove_file(src)
            }
            Err(e) => Err(e),
        }?;
    }

    Ok(())
}

// -----------------------------------------------------------
// PATTERN 3: Helper functions (copy from rm.rs)
// -----------------------------------------------------------
fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("yes"))
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

// -----------------------------------------------------------
// PATTERN 4: Unit tests (from rm.rs lines 206-460)
// -----------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execute_no_files() {
        let args = MvArgs {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_odd_number_of_files() {
        let args = MvArgs {
            files: vec!["src.txt".to_string(), "dst.txt".to_string(), "extra.txt".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let args = MvArgs {
            files: vec!["src.txt".to_string(), "dst.txt".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_format_layer_name() {
        assert_eq!(format_layer_name(Layer::GlobalBase), "global-base");
        assert_eq!(format_layer_name(Layer::ModeBase), "mode-base");
        assert_eq!(format_layer_name(Layer::ProjectBase), "project-base");
    }

    // Add more tests for move_file scenarios...
}
```

### Integration Points

```yaml
STAGING_INDEX:
  - file: .jin/staging/index.json
  - format: JSON array of StagedEntry objects
  - operations: load(), save(), get(path), remove(path), add(entry)
  - validation: Path must match exactly (absolute PathBuf)

GITIGNORE_MANAGED_BLOCK:
  - file: .gitignore in project root
  - markers: "# --- JIN MANAGED START ---" and "# --- JIN MANAGED END ---"
  - functions: remove_from_managed_block(path), ensure_in_managed_block(path)
  - requirements: Both must be called for move (remove old, add new)

WORKSPACE_FILES:
  - location: Project root directory
  - operations: std::fs::rename(src, dst) with fallback
  - conditions: Only move if args.force is true
  - validation: Check src.exists() before attempting move

LAYER_REFS:
  - location: ~/.jin/ (Git references)
  - pattern: refs/jin/layers/{layer_type}/{name}
  - routing: Handled by route_to_layer() from staging/router.rs
  - validation: validate_routing_options() checks flag combinations
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --bin jin               # Fast compilation check
cargo clippy --bin jin -- -D warnings  # Lint checks

# Auto-fix formatting issues
cargo fmt --all

# Expected: Zero errors, zero warnings. Fix all issues before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test mv command unit tests
cargo test mv::tests --lib

# Test specific functions
cargo test mv::tests::test_execute_no_files
cargo test mv::tests::test_execute_odd_number_of_files
cargo test mv::tests::test_move_file_not_staged

# Test StagedEntry::rename constructor
cargo test staging::entry::tests::test_staged_entry_rename

# Run all unit tests
cargo test --lib

# Expected: All tests pass. If failing, read output and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the binary
cargo build --release

# Test basic mv functionality
cd /tmp && mkdir test_mv && cd test_mv
jin init
echo "content" > config.json
jin add config.json
jin mv config.json settings/config.json
# Expected: Success message, staging updated

# Verify staging was updated
cat .jin/staging/index.json
# Expected: Contains "settings/config.json", NOT "config.json"

# Test dry-run mode
echo "content2" > data.json
jin add data.json
jin mv --dry-run data.json data2.json
# Expected: "Would move: data.json -> data2.json"
# Verify: data.json still exists, not renamed

# Test error case - source not staged
jin mv nonexistent.json target.json
# Expected: Error message "not in staging"

# Test layer routing
jin mode create testmode
jin mode use testmode
echo '{"key": "value"}' > mode-config.json
jin add mode-config.json
jin mv --mode mode-config.json mode-config-renamed.json
# Expected: File moved to mode layer

# Test force flag (workspace move)
echo "workspace content" > workspace.txt
jin add workspace.txt
jin mv --force workspace.txt workspace-moved.txt
# Expected: workspace.txt gone, workspace-moved.txt exists in workspace

# Cleanup
cd /tmp && rm -rf test_mv

# Expected: All manual tests pass with correct behavior.
```

### Level 4: Comprehensive Test Suite

```bash
# Run full integration test suite
cargo test --test cli_mv

# Run all integration tests
cargo test --test

# With output
cargo test --test cli_mv -- --nocapture

# Expected: All tests pass. Review any failing tests to fix edge cases.
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test mv::tests --lib`
- [ ] All integration tests pass: `cargo test --test cli_mv`
- [ ] No clippy warnings: `cargo clippy --bin jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --all --check`

### Feature Validation

- [ ] Single file move works: `jin mv src.txt dst.txt`
- [ ] Batch file moves work: `jin mv s1.txt d1.txt s2.txt d2.txt`
- [ ] Layer routing works: `jin mv --mode file.txt new.txt`
- [ ] Dry-run mode works: `jin mv --dry-run src.txt dst.txt`
- [ ] Force flag works: `jin mv --force src.txt dst.txt`
- [ ] Error cases handled: not staged, already exists, odd number of args
- [ ] .gitignore updated correctly: old removed, new added
- [ ] Workspace files moved with --force flag

### Code Quality Validation

- [ ] Follows rm.rs implementation pattern exactly
- [ ] File placement matches desired codebase tree
- [ ] All imports use correct module paths
- [ ] Error messages are clear and actionable
- [ ] Dry-run mode returns early without saving
- [ ] Confirmation prompt works without --force
- [ ] Partial success with error collection implemented

### Staging System Validation

- [ ] StagedEntry::rename() constructor added to entry.rs
- [ ] StagingIndex operations: get(), remove(), add(), save() all used correctly
- [ ] Content hash preserved from existing entry
- [ ] File mode (permissions) preserved from existing entry
- [ ] StagedOperation::Rename variant used in new entries

### CLI Integration Validation

- [ ] MvArgs struct added to args.rs with correct fields
- [ ] Mv(MvArgs) variant added to Commands enum in mod.rs
- [ ] Commands::Mv match arm added to commands/mod.rs
- [ ] Command accessible: `cargo run -- mv --help`

---

## Anti-Patterns to Avoid

- **Don't** create new routing logic - use existing `route_to_layer()` from staging/router.rs
- **Don't** skip validation - must check file count, staging state, destination conflicts
- **Don't** ignore cross-filesystem moves - handle error 18 with copy + delete fallback
- **Don't** forget to update .gitignore - must remove old AND add new path
- **Don't** use sync I/O where async is expected (not applicable - all commands are sync)
- **Don't** hardcode layer names - use `format_layer_name()` helper
- **Don't** skip dry-run mode - must check args.dry_run and return early
- **Don't** lose file metadata - preserve content_hash and mode from existing entry
- **Don't** modify rm.rs - create new mv.rs file instead
- **Don't** add new dependencies - use only existing imports from rm.rs
- **Don't** create new error types - use existing JinError variants
- **Don't** skip tests - unit tests in mv.rs, integration tests in cli_mv.rs

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Rationale**:
- Complete pattern reference from rm.rs (lines 1-460)
- All dependencies identified with exact line numbers
- Staging API fully documented with method signatures
- Test patterns and fixtures well-established
- External research provides edge case handling
- Only missing piece: StagedEntry::rename() constructor (clearly specified)

**Confidence Reduction Factors**:
- File pair parsing (chunks(2)) adds slight complexity vs rm
- Cross-filesystem move fallback needs testing
- Multiple validation points increase chance of missing edge case

**Validation**: The completed PRP enables an AI agent unfamiliar with Jin to implement the mv command successfully using only this document and codebase access.
