---
name: "P1.M1.T3: Implement Resolve Command"
description: |

---

## Goal

**Feature Goal**: Create the `jin resolve` command that completes the conflict resolution workflow by validating user-resolved `.jinmerge` files and resuming the interrupted apply operation.

**Deliverable**: A new CLI command `jin resolve <file>` that:
1. Validates that `.jinmerge` conflicts have been manually resolved by the user
2. Applies resolved content to the workspace
3. Cleans up `.jinmerge` files and updates state
4. Automatically continues the apply operation when all conflicts are resolved

**Success Definition**:
- Users can run `jin resolve <file>` after manually editing `.jinmerge` files
- The command validates that conflict markers have been removed
- Resolved files are written to the workspace atomically
- `.jinmerge` files are cleaned up after successful resolution
- The paused apply operation automatically completes when all conflicts are resolved
- All validation gates pass (unit tests, integration tests, linting, type checking)

## User Persona

**Target User**: Developer using Jin to manage layered configuration files who encounters merge conflicts during `jin apply`.

**Use Case**: A developer runs `jin apply` which encounters conflicts and creates `.jinmerge` files. They manually edit these files to resolve the conflicts (choosing between layer contents or creating custom resolutions), then run `jin resolve <file>` to apply their resolutions and complete the operation.

**User Journey**:
1. User runs `jin apply` which detects conflicts and exits with `.jinmerge` files
2. User edits `.jinmerge` files manually (removes conflict markers, keeps desired content)
3. User runs `jin resolve config.json` to apply their resolution
4. System validates the resolution (no conflict markers remain)
5. Resolved content is written to `config.json`
6. `.jinmerge` file is cleaned up
7. If more conflicts exist, user continues; otherwise apply completes automatically

**Pain Points Addressed**:
- Manual conflict resolution workflow was incomplete - users had no way to signal resolution
- No validation that conflicts were properly resolved before continuing
- Risk of partial state if conflicts resolved but apply not completed
- Unclear next steps after manual conflict resolution

## Why

- **Enables complete conflict resolution workflow**: Task P1.M1.T2 implemented `.jinmerge` file creation and apply pausing, but users had no way to resume the operation after resolving conflicts
- **Validates user actions**: Prevents silent failures from incomplete conflict resolutions
- **Completes the interrupted apply**: Automatically continues the operation rather than requiring manual `jin apply` re-invocation
- **Integration with existing conflict workflow**: Builds on P1.M1.T1 (.jinmerge format) and P1.M1.T2 (apply pausing)

## What

Users run `jin resolve <file>` after manually resolving `.jinmerge` conflicts:

```bash
# After jin apply creates .jinmerge files
$ cat config.json.jinmerge
# Jin merge conflict. Resolve and run 'jin resolve <file>'
<<<<<<< mode/claude/scope:javascript/
{"target": "es6"}
=======
{"target": "es2020"}
>>>>>>> mode/claude/project/ui-dashboard/

# User edits to resolve (keep one side or custom)
$ cat config.json.jinmerge
{"target": "es2020"}

# Run resolve to apply
$ jin resolve config.json
Resolved 1 conflict for config.json
Use 'jin resolve' for remaining conflicts or 'jin status' for details

# After all conflicts resolved, apply completes automatically
```

### Command Interface

```bash
jin resolve <file>              # Resolve specific file
jin resolve --all               # Resolve all conflicts at once
jin resolve --force             # Skip validation prompts
jin resolve --dry-run           # Show what would be resolved
```

### Success Criteria

- [ ] `jin resolve <file>` validates `.jinmerge` file has no conflict markers
- [ ] Resolved content is written to original file atomically
- [ ] `.jinmerge` file is deleted after successful resolution
- [ ] Paused state is updated (or deleted if all conflicts resolved)
- [ ] When all conflicts resolved, apply operation completes automatically
- [ ] Error handling for invalid resolutions, missing files, stale state
- [ ] All tests pass (unit and integration)
- [ ] Follows existing CLI patterns and conventions

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validated: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**YES** - This PRP provides:
- Exact file paths for all related code (from completed tasks)
- Complete `.jinmerge` file format specification
- Exact conflict state file format and location
- CLI command registration patterns with code examples
- Existing command patterns to follow (apply, status, reset)
- Test patterns and validation commands
- External research on resolve/retry patterns

### Documentation & References

```yaml
# MUST READ - Core dependencies from completed tasks

# .jinmerge format module (P1.M1.T1 - COMPLETE)
- file: src/merge/jinmerge.rs
  why: Defines JinMergeConflict, JinMergeRegion structs and all file I/O operations
  pattern: Use JinMergeConflict::parse_from_file() to read .jinmerge, JinMergeConflict::is_jinmerge_file() for detection
  gotcha: Files use .jinmerge extension, always start with JINMERGE_HEADER, use 7-char Git markers
  sections:
    - JinMergeConflict struct (lines 74-80)
    - parse_from_file() function (lines 202-205)
    - is_jinmerge_file() function (lines 245-260)
    - merge_path_for_file() function (lines 277-281)

# Apply command with conflict state (P1.M1.T2 - COMPLETE)
- file: src/commands/apply.rs
  why: Contains PausedApplyState struct, state persistence logic, and conflict handling
  pattern: Follow handle_conflicts() pattern for state management, use atomic writes
  gotcha: State stored at .jin/.paused_apply.yaml, uses YAML format with atomic rename
  sections:
    - PausedApplyState struct (lines 17-29)
    - PausedLayerConfig struct (lines 32-42)
    - handle_conflicts() function (lines 207-246)
    - get_conflicting_layer_contents() (lines 257-311)

# CLI command registration pattern
- file: src/cli/mod.rs
  why: Shows how to add Resolve variant to Commands enum
  pattern: Add Resolve(ResolveArgs) variant, add resolve module to commands/mod.rs
  sections:
    - Commands enum definition (lines 28-122)
    - ModeAction enum pattern for subcommands (lines 124-148)

# CLI arguments pattern
- file: src/cli/args.rs
  why: Defines argument structures for all commands
  pattern: #[derive(Args, Debug)], use #[arg(long)] for flags, #[arg(conflicts_with)] for mutual exclusion
  sections:
    - ApplyArgs pattern (lines 40-50) - similar flags to resolve
    - ResetArgs pattern (lines 52-86) - subcommand flags

# Command execution dispatch
- file: src/commands/mod.rs
  why: Central command execution - must add resolve::execute() match arm
  pattern: Commands::Resolve(args) => resolve::execute(args)
  sections:
    - Module declarations (lines 8-31)
    - execute() function match arms (lines 34-63)

# Status command (for reference on checking paused state)
- file: src/commands/status.rs
  why: May already check for paused apply state - can reuse pattern
  pattern: Check PausedApplyState::exists() to detect in-progress operation

# Test patterns
- file: tests/cli_apply_conflict.rs
  why: Integration tests for conflict workflow - extend for resolve testing
  pattern: Use TestFixture for isolation, assert_cmd::Command for CLI testing
  sections:
    - Test fixture setup pattern
    - File state assertions (assert_workspace_file_*)

# Test common utilities
- file: tests/common/mod.rs
  why: Shared test utilities - fixtures, assertions, git helpers
  pattern: Use TestFixture::new()? for isolated test environment
  sections:
    - fixtures.rs - TestFixture struct
    - assertions.rs - Custom assertions for Jin state
    - git_helpers.rs - Git operations in tests

# External research (stored in research/ subdir)
- docfile: plan/P1M1T3/research/external_resolve_patterns.md
  why: Best practices from Git, database migration tools for resume/retry workflows
  critical: Two-phase pattern (resolve then continue), state persistence, validation before completion

# Project architecture docs
- docfile: plan/docs/COMMANDS.md
  why: Documents all Jin commands - ensure resolve follows conventions

- docfile: plan/docs/MERGE_QUICK_REFERENCE.md
  why: Quick reference for merge system and conflict handling

- docfile: plan/docs/WORKFLOWS.md
  why: Documents Jin workflows including conflict resolution
```

### Current Codebase Tree (relevant sections)

```bash
src/
├── cli/
│   ├── mod.rs          # Add Resolve(ResolveArgs) to Commands enum
│   └── args.rs         # Add ResolveArgs struct
├── commands/
│   ├── mod.rs          # Add pub mod resolve; and match arm
│   ├── apply.rs        # PausedApplyState, handle_conflicts() - COMPLETE
│   └── resolve.rs      # CREATE - new resolve command
├── merge/
│   ├── mod.rs          # Already exports JinMergeConflict
│   └── jinmerge.rs     # Use JinMergeConflict::parse_from_file() - COMPLETE
├── core/
│   ├── error.rs        # JinError types for error handling
│   └── mod.rs          # ProjectContext, Result type
├── git/
│   └── repo.rs         # JinRepo for repository operations
└── staging/
    └── metadata.rs     # WorkspaceMetadata for tracking applied files

tests/
├── common/
│   ├── mod.rs          # Test utilities
│   ├── fixtures.rs     # TestFixture
│   └── assertions.rs   # Custom assertions
└── cli_resolve.rs      # CREATE - integration tests for resolve
```

### Desired Codebase Tree with Files to be Added

```bash
# NEW FILES TO CREATE
src/commands/resolve.rs       # Main resolve command implementation
tests/cli_resolve.rs          # Integration tests for resolve command

# FILES TO MODIFY
src/cli/mod.rs                # Add Resolve(ResolveArgs) variant
src/cli/args.rs               # Add ResolveArgs struct
src/commands/mod.rs           # Add pub mod resolve; and match arm
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: .jinmerge file format
// Files MUST start with JINMERGE_HEADER comment
// Markers are EXACTLY 7 characters (Git-compatible)
// Layer labels are FULL ref paths, not generic "ours/theirs"
const JINMERGE_HEADER: &str = "# Jin merge conflict. Resolve and run 'jin resolve <file>'";

// CRITICAL: PausedApplyState location
// State is ALWAYS at .jin/.paused_apply.yaml (hardcoded)
// Uses YAML format with atomic write (.tmp then rename)
// Contains both conflict_files AND applied_files
let state_path = PathBuf::from(".jin/.paused_apply.yaml");

// GOTCHA: Conflict files are stored as ORIGINAL paths, not .jinmerge paths
// When iterating state.conflict_files, these are the workspace file paths
// Must call JinMergeConflict::merge_path_for_file() to get .jinmerge path
for conflict_path in state.conflict_files {
    let merge_path = JinMergeConflict::merge_path_for_file(&conflict_path);
    // merge_path is "config.json.jinmerge"
    // conflict_path is "config.json"
}

// CRITICAL: Atomic write pattern for file modifications
// ALWAYS write to temp file first, then rename to prevent corruption
let temp_path = path.with_extension("jin-tmp");
std::fs::write(&temp_path, content)?;
std::fs::rename(&temp_path, path)?;

// CRITICAL: Test isolation with JIN_DIR
// All integration tests MUST set JIN_DIR environment variable
// Use fixture.set_jin_dir() from TestFixture
// Run with --test-threads=1 to prevent interference
jin_cmd().env("JIN_DIR", &jin_dir).assert()...

// CRITICAL: clap v4 derive API patterns
// Use #[arg(long)] for flags, not #[clap(long)]
// Use conflicts_with for mutual exclusion
// Subcommands use separate enum like ModeAction
#[derive(Args, Debug)]
pub struct ResolveArgs {
    #[arg(long, conflicts_with = "theirs")]
    pub ours: bool,
}

// GOTCHA: Workspace metadata tracking
// After resolving files, must update WorkspaceMetadata
// metadata.add_file(path, hash) tracks applied files
// metadata.save() persists changes
let mut metadata = WorkspaceMetadata::new()?;
metadata.add_file(path.clone(), new_hash)?;
metadata.save()?;

// GOTCHA: Layer content retrieval for validation
// If re-merging to validate, iterate layers in REVERSE (highest precedence first)
// get_conflicting_layer_contents() shows this pattern (apply.rs:264-294)
for layer in config.layers.iter().rev() {
    // highest precedence layers checked first
}
```

## Implementation Blueprint

### Data Models and Structures

```rust
// Resolve command uses existing structures from completed tasks:

// From src/commands/apply.rs (P1.M1.T2 - already complete)
struct PausedApplyState {
    timestamp: DateTime<Utc>,
    layer_config: PausedLayerConfig,
    conflict_files: Vec<PathBuf>,      // Original file paths
    applied_files: Vec<PathBuf>,        // Successfully applied files
    conflict_count: usize,
}

struct PausedLayerConfig {
    layers: Vec<String>,
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
}

// From src/merge/jinmerge.rs (P1.M1.T1 - already complete)
struct JinMergeConflict {
    file_path: PathBuf,
    conflicts: Vec<JinMergeRegion>,
}

struct JinMergeRegion {
    layer1_ref: String,
    layer1_content: String,
    layer2_ref: String,
    layer2_content: String,
    start_line: usize,
    end_line: usize,
}

// NEW: Resolve command arguments (src/cli/args.rs)
#[derive(Args, Debug)]
pub struct ResolveArgs {
    /// File(s) to resolve (optional, resolves all if not specified)
    pub files: Vec<String>,

    /// Resolve all remaining conflicts
    #[arg(long, short = 'a')]
    pub all: bool,

    /// Skip confirmation prompts
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Show what would be resolved without doing it
    #[arg(long)]
    pub dry_run: bool,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD ResolveArgs struct to src/cli/args.rs
  - IMPLEMENT: ResolveArgs with files, all, force, dry_run fields
  - FOLLOW pattern: ApplyArgs (lines 40-50) for similar flag structure
  - NAMING: ResolveArgs struct name, snake_case field names
  - VALIDATION: files is Vec<String> (empty means --all), conflicts_with if adding mutual exclusions
  - PLACEMENT: After PushArgs (around line 220)

Task 2: ADD Resolve variant to Commands enum in src/cli/mod.rs
  - IMPLEMENT: Resolve(ResolveArgs) variant in Commands enum
  - FOLLOW pattern: Apply(ApplyArgs) (line 57) for positioning
  - NAMING: Resolve variant name, same as existing command pattern
  - PLACEMENT: After Apply variant (around line 57)
  - DEPENDENCIES: ResolveArgs from Task 1

Task 3: CREATE src/commands/resolve.rs
  - IMPLEMENT: execute() function with ResolveArgs parameter
  - FOLLOW pattern: src/commands/apply.rs for command structure (load context, validate, execute)
  - NAMING: pub fn execute(args: ResolveArgs) -> Result<()>
  - DEPENDENCIES: ResolveArgs from Task 1
  - PLACEMENT: New file in src/commands/
  - STRUCTURE:
    1. Load PausedApplyState from .jin/.paused_apply.yaml
    2. Validate state exists and is not stale
    3. Determine which files to resolve (from args.files or all)
    4. For each file:
       a. Parse .jinmerge file using JinMergeConflict::parse_from_file()
       b. Validate no conflict markers remain
       c. Read resolved content from .jinmerge file
       d. Write resolved content to workspace file (atomic)
       e. Delete .jinmerge file
    5. Update PausedApplyState (remove resolved file from conflict_files)
    6. If no more conflicts, complete apply operation automatically
    7. Save updated state or delete if complete

Task 4: ADD resolve module to src/commands/mod.rs
  - IMPLEMENT: pub mod resolve; declaration
  - FOLLOW pattern: pub mod apply; (line 9)
  - NAMING: resolve (lowercase module name)
  - PLACEMENT: After apply module declaration (around line 9)

Task 5: ADD resolve execution match arm in src/commands/mod.rs
  - IMPLEMENT: Commands::Resolve(args) => resolve::execute(args)
  - FOLLOW pattern: Commands::Apply(args) => apply::execute(args) (line 44)
  - NAMING: resolve::execute(args)
  - PLACEMENT: In execute() function match statement (around line 44)
  - DEPENDENCIES: resolve module from Task 4

Task 6: IMPLEMENT helper functions in src/commands/resolve.rs
  - IMPLEMENT: load_paused_state() - loads and validates .jin/.paused_apply.yaml
  - IMPLEMENT: validate_resolution() - checks .jinmerge file has no conflict markers
  - IMPLEMENT: apply_resolved_file() - atomically writes resolved content to workspace
  - IMPLEMENT: complete_apply_operation() - finishes apply when all conflicts resolved
  - FOLLOW pattern: apply.rs helper functions (lines 207-468)
  - NAMING: fn helper_name() -> Result<Type>
  - ERROR HANDLING: Use JinError::Other for validation failures
  - PLACEMENT: In resolve.rs, before execute()

Task 7: CREATE tests/cli_resolve.rs
  - IMPLEMENT: test_resolve_simple_conflict() - basic resolve workflow
  - IMPLEMENT: test_resolve_all_conflicts() - --all flag behavior
  - IMPLEMENT: test_resolve_invalid_markers() - error on remaining conflict markers
  - IMPLEMENT: test_resolve_no_paused_state() - error when no .paused_apply.yaml
  - IMPLEMENT: test_resolve_completes_apply() - auto-complete when all done
  - IMPLEMENT: test_resolve_dry_run() - --dry-run shows without doing
  - FOLLOW pattern: tests/cli_apply_conflict.rs for test structure
  - NAMING: #[test] fn test_scenario_description()
  - ISOLATION: Use TestFixture::new()? and fixture.set_jin_dir()
  - COVERAGE: All error conditions, happy paths, edge cases
  - PLACEMENT: New file in tests/

Task 8: ADD PausedApplyState::load() method to src/commands/apply.rs
  - IMPLEMENT: static method to load and deserialize PausedApplyState
  - FOLLOW pattern: Existing save() method (lines 45-57)
  - NAMING: fn load() -> Result<Self>
  - ERROR HANDLING: Return JinError::Other if file missing or invalid
  - PLACEMENT: In apply.rs, in impl PausedApplyState block (around line 63)
  - GOTCHA: Resolve command needs to load state - add this method for reusability

Task 9: UPDATE status command to show paused apply state
  - MODIFY: src/commands/status.rs to check PausedApplyState::exists()
  - IMPLEMENT: Display conflict state if paused operation detected
  - FOLLOW pattern: Existing status output format
  - NAMING: Show "Paused apply operation: N conflicts remaining"
  - PLACEMENT: In status::execute() function
  - GOTCHA: This is P1.M1.T4 but integrates with resolve - may be done separately
```

### Implementation Patterns & Key Details

```rust
// ============================================
// RESOLVE COMMAND EXECUTION FLOW
// ============================================

// Main entry point - src/commands/resolve.rs
pub fn execute(args: ResolveArgs) -> Result<()> {
    // 1. Check for paused state (follow apply.rs pattern)
    if !PausedApplyState::exists() {
        return Err(JinError::Other(
            "No paused apply operation found. Run 'jin apply' first.".to_string()
        ));
    }

    // 2. Load and validate paused state
    let state = PausedApplyState::load()?;

    // 3. Validate state is not stale (optional timeout check)
    let max_age = chrono::Duration::hours(24);
    if Utc::now() - state.timestamp > max_age {
        eprintln!("Warning: Paused operation is over 24 hours old.");
        if !args.force {
            return Err(JinError::Other(
                "Stale paused state. Use --force to proceed.".to_string()
            ));
        }
    }

    // 4. Determine files to resolve
    let files_to_resolve = if args.files.is_empty() || args.all {
        // Resolve all conflicts
        state.conflict_files.clone()
    } else {
        // Validate specified files are in conflict list
        for file in &args.files {
            let path = PathBuf::from(file);
            if !state.conflict_files.contains(&path) {
                return Err(JinError::Other(format!(
                    "File '{}' is not in conflict state. Use 'jin status' for details.",
                    file
                )));
            }
        }
        args.files.iter().map(|f| PathBuf::from(f)).collect()
    };

    // 5. Dry-run mode
    if args.dry_run {
        println!("Would resolve {} files:", files_to_resolve.len());
        for file in &files_to_resolve {
            println!("  - {}", file.display());
        }
        return Ok(());
    }

    // 6. Resolve each file
    let mut resolved_count = 0;
    let mut errors = Vec::new();

    for conflict_path in files_to_resolve {
        match resolve_single_file(&conflict_path, &state) {
            Ok(_) => resolved_count += 1,
            Err(e) => errors.push(format!("{}: {}", conflict_path.display(), e)),
        }
    }

    // 7. Report results
    if resolved_count > 0 {
        println!("Resolved {} file(s)", resolved_count);
    }

    if !errors.is_empty() {
        eprintln!("Errors resolving {} file(s):", errors.len());
        for error in &errors {
            eprintln!("  - {}", error);
        }
        if resolved_count == 0 {
            return Err(JinError::Other("Failed to resolve any files".to_string()));
        }
    }

    // 8. Check if all conflicts resolved
    let remaining_conflicts = state.conflict_files.len() - resolved_count;
    if remaining_conflicts == 0 {
        // Complete the apply operation automatically
        complete_apply_operation(&state)?;
        println!("All conflicts resolved. Apply operation completed.");
    } else {
        println!("Remaining conflicts: {}", remaining_conflicts);
        println!("Use 'jin resolve --all' to resolve remaining conflicts.");
    }

    Ok(())
}

// ============================================
// HELPER: Resolve single file
// ============================================

fn resolve_single_file(
    conflict_path: &PathBuf,
    state: &PausedApplyState,
) -> Result<()> {
    // 1. Locate .jinmerge file
    let merge_path = JinMergeConflict::merge_path_for_file(conflict_path);
    if !merge_path.exists() {
        return Err(JinError::Other(format!(
            "No .jinmerge file found for {}. Did you delete it?",
            conflict_path.display()
        )));
    }

    // 2. Parse .jinmerge file
    let merge_conflict = JinMergeConflict::parse_from_file(&merge_path)?;

    // 3. Validate no conflict markers remain
    validate_no_conflict_markers(&merge_conflict)?;

    // 4. Read resolved content from .jinmerge file
    let resolved_content = std::fs::read_to_string(&merge_path)?;

    // 5. Write resolved content to workspace file (atomic)
    apply_resolved_file(conflict_path, &resolved_content)?;

    // 6. Delete .jinmerge file
    std::fs::remove_file(&merge_path)
        .map_err(|e| JinError::Other(format!("Failed to delete .jinmerge file: {}", e)))?;

    // 7. Update state (remove from conflict_files)
    update_paused_state(conflict_path)?;

    Ok(())
}

// ============================================
// HELPER: Validate no conflict markers remain
// ============================================

fn validate_no_conflict_markers(merge_conflict: &JinMergeConflict) -> Result<()> {
    // Check if the .jinmerge file still contains conflict markers
    // User should have removed markers and kept only the resolved content

    let merge_path = JinMergeConflict::merge_path_for_file(&merge_conflict.file_path);
    let content = std::fs::read_to_string(&merge_path)?;

    // Check for conflict markers
    if content.contains("<<<<<<<") || content.contains("=======") || content.contains(">>>>>>>") {
        return Err(JinError::Other(
            "Conflict markers still present. Please resolve all conflicts before running 'jin resolve'.".to_string()
        ));
    }

    // Check for header
    if !content.starts_with(JINMERGE_HEADER) {
        return Err(JinError::Other(
            "Invalid .jinmerge file format (missing header).".to_string()
        ));
    }

    Ok(())
}

// ============================================
// HELPER: Apply resolved file atomically
// ============================================

fn apply_resolved_file(file_path: &PathBuf, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(JinError::Io)?;
    }

    // Atomic write pattern (follow apply.rs:349-354)
    let temp_path = file_path.with_extension("jin-tmp");
    std::fs::write(&temp_path, content)
        .map_err(JinError::Io)?;

    std::fs::rename(&temp_path, file_path)
        .map_err(JinError::Io)?;

    Ok(())
}

// ============================================
// HELPER: Update paused state after resolution
// ============================================

fn update_paused_state(resolved_path: &PathBuf) -> Result<()> {
    let mut state = PausedApplyState::load()?;

    // Remove resolved file from conflict list
    state.conflict_files.retain(|p| p != resolved_path);
    state.conflict_count = state.conflict_files.len();

    // If no more conflicts, delete state file
    if state.conflict_files.is_empty() {
        let state_path = PathBuf::from(".jin/.paused_apply.yaml");
        std::fs::remove_file(&state_path)
            .map_err(|e| JinError::Other(format!("Failed to remove paused state: {}", e)))?;
    } else {
        // Save updated state
        state.save()?;
    }

    Ok(())
}

// ============================================
// HELPER: Complete apply operation
// ============================================

fn complete_apply_operation(state: &PausedApplyState) -> Result<()> {
    // All conflicts resolved - complete the apply operation
    // This mimics the final steps of apply.rs (lines 167-192)

    // 1. Update workspace metadata
    let mut metadata = WorkspaceMetadata::new()?;
    metadata.applied_layers = state.layer_config.layers.clone();

    // Add resolved files to metadata
    let repo = JinRepo::open()?;
    for conflict_path in &state.conflict_files {
        // Now all files are resolved
        let content = std::fs::read_to_string(conflict_path)?;
        let oid = repo.create_blob(content.as_bytes())?;
        metadata.add_file(conflict_path.clone(), oid.to_string());
    }
    metadata.save()?;

    // 2. Update .gitignore for all applied files
    for path in state.applied_files.iter().chain(state.conflict_files.iter()) {
        if let Err(e) = ensure_in_managed_block(path) {
            eprintln!("Warning: Could not update .gitignore: {}", e);
        }
    }

    // 3. Delete paused state file
    let state_path = PathBuf::from(".jin/.paused_apply.yaml");
    std::fs::remove_file(&state_path)
        .map_err(|e| JinError::Other(format!("Failed to remove paused state: {}", e)))?;

    // 4. Report completion
    println!("Apply operation completed successfully.");
    println!("Applied {} total files.", state.applied_files.len() + state.conflict_files.len());

    Ok(())
}

// ============================================
// ADD TO apply.rs: PausedApplyState::load()
// ============================================

// In src/commands/apply.rs, add to impl PausedApplyState block (after line 63)

impl PausedApplyState {
    // ... existing exists() and save() methods ...

    /// Load state from `.jin/.paused_apply.yaml`
    fn load() -> Result<Self> {
        let path = PathBuf::from(".jin/.paused_apply.yaml");

        if !path.exists() {
            return Err(JinError::Other(
                "No paused apply operation found".to_string()
            ));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(JinError::Io)?;

        serde_yaml::from_str(&content)
            .map_err(|e| JinError::Other(format!("Invalid paused state: {}", e)))
    }
}
```

### Integration Points

```yaml
COMMAND_REGISTRATION:
  - modify: src/cli/mod.rs
    add_to: Commands enum (after line 57)
    pattern: |
      /// Apply merged layers to workspace
      Apply(ApplyArgs),

      /// Resolve merge conflicts
      Resolve(ResolveArgs),

  - modify: src/cli/args.rs
    add_to: After PushArgs struct (around line 220)
    pattern: |
      /// Arguments for the `resolve` command
      #[derive(Args, Debug)]
      pub struct ResolveArgs {
          /// File(s) to resolve
          pub files: Vec<String>,

          /// Resolve all remaining conflicts
          #[arg(long, short = 'a')]
          pub all: bool,

          /// Skip confirmation prompts
          #[arg(long, short = 'f')]
          pub force: bool,

          /// Show what would be resolved
          #[arg(long)]
          pub dry_run: bool,
      }

  - modify: src/commands/mod.rs
    add_to: Module declarations (after line 9)
    pattern: pub mod resolve;

  - modify: src/commands/mod.rs
    add_to: execute() function match (after line 44)
    pattern: Commands::Resolve(args) => resolve::execute(args),

STATE_FILE_LOCATION:
  - file: .jin/.paused_apply.yaml
    format: YAML
    access: PausedApplyState::load() and save()
    gotcha: Hardcoded path, not configurable

JINMERGE_FILES:
  - extension: .jinmerge
    pattern: file_path.jinmerge for file_path workspace file
    access: JinMergeConflict::parse_from_file(), merge_path_for_file()

METADATA_UPDATES:
  - modify: WorkspaceMetadata after resolving files
    pattern: metadata.add_file(path, hash)
    location: src/staging/metadata.rs
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --bin jin                    # Fast compilation check
cargo clippy --bin jin -- -D warnings    # Linting with warnings as errors

# Format check
cargo fmt --check                        # Verify formatting

# Run these specifically for new files
cargo check --bin jin 2>&1 | grep -E "(resolve|error)"

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - Missing imports (use crate::merge::jinmerge::JinMergeConflict)
# - Unused variables (remove or prefix with _)
# - Missing async annotations (not needed for this command)
# - Type mismatches in Result returns
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run unit tests in resolve.rs
cargo test --bin jin resolve -- --test-threads=1

# Run specific test
cargo test --bin jin test_resolve_simple_conflict -- --test-threads=1

# Run with output
cargo test --bin jin resolve -- --nocapture --test-threads=1

# Expected: All tests pass. Watch for:
# - Test isolation failures (use TestFixture properly)
# - File path issues (use PathBuf correctly)
# - Mock setup issues

# Coverage check (optional)
cargo tarpaulin --out Html --exclude-files="*/tests/*" --timeout 120

# Expected: High coverage on execute() and helper functions
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the binary first
cargo build --release

# Run all integration tests
cargo test --test cli_resolve -- --test-threads=1

# Run specific integration test
cargo test --test cli_resolve test_resolve_simple_conflict -- --test-threads=1

# Run with verbose output
cargo test --test cli_resolve -- --verbose --test-threads=1

# Manual testing workflow:

# 1. Setup: Create a test repo with conflicts
cd /tmp/test_resolve
jin init
echo '{"port": 8080}' > config.json
jin add config.json --global
jin commit -m "Add global config"

echo '{"port": 9090}' > config.json
jin add config.json --mode
jin commit -m "Add mode config"

# 2. Trigger conflict
jin apply  # Should create .jinmerge file

# 3. Verify conflict file exists
ls config.json.jinmerge  # Should exist
cat .jin/.paused_apply.yaml  # Should exist

# 4. Manually resolve conflict
echo '{"port": 9090}' > config.json.jinmerge

# 5. Run resolve
jin resolve config.json  # Should succeed
ls config.json.jinmerge  # Should be deleted
cat .jin/.paused_apply.yaml  # Should be deleted if no more conflicts

# Expected: All integration tests pass, manual workflow succeeds
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test stale state handling
# Create paused state, wait 25 hours, try to resolve
# Should warn about stale state without --force

# Test partial resolution
# Create 3 conflicts, resolve 1, check state has 2 remaining
cat .jin/.paused_apply.yaml | grep conflict_count

# Test auto-completion
# Resolve all conflicts, verify apply completed
# Check workspace metadata updated

# Test error recovery
# Resolve with invalid markers, expect helpful error
# Resolve non-conflicted file, expect helpful error

# Test idempotency
# Resolve same file twice, should error on second attempt
# (file already resolved, .jinmerge deleted)

# Performance testing (if many conflicts)
# Create 100 conflicts, time resolution performance
hyperfine 'jin resolve --all'

# Edge cases
# - Empty .jinmerge file (only header)
# - Binary file with .jinmerge (should handle or reject)
# - Conflict in nested directory path
# - Concurrent resolve attempts (two terminals)

# Expected: All edge cases handled gracefully with clear error messages
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test -- --test-threads=1`
- [ ] No linting errors: `cargo clippy --bin jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] No check errors: `cargo check --bin jin`

### Feature Validation

- [ ] `jin resolve <file>` successfully resolves single conflict
- [ ] `jin resolve --all` resolves all remaining conflicts
- [ ] `.jinmerge` files are deleted after successful resolution
- [ ] `.jin/.paused_apply.yaml` is updated or deleted appropriately
- [ ] Apply operation auto-completes when all conflicts resolved
- [ ] Error cases handled: no paused state, invalid markers, missing files
- [ ] `--dry-run` shows what would be resolved without making changes
- [ ] `--force` skips stale state warnings

### Code Quality Validation

- [ ] Follows existing CLI patterns (apply, status commands)
- [ ] File placement matches desired codebase tree
- [ ] Atomic write pattern used for file operations
- [ ] Proper error handling with JinError types
- [ ] Helper functions are well-named and single-purpose
- [ ] Comments explain non-obvious logic (state transitions, validation)

### Integration Validation

- [ ] Command registered in CLI (src/cli/mod.rs)
- [ ] Arguments defined (src/cli/args.rs)
- [ ] Execution wired in commands/mod.rs
- [ ] Uses existing JinMergeConflict module
- [ ] Uses existing PausedApplyState structure
- [ ] Updates WorkspaceMetadata correctly

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Error messages are user-friendly and actionable
- [ ] Help text is clear (`jin resolve --help`)
- [ ] Integration with status command (shows paused state)

---

## Anti-Patterns to Avoid

- **Don't** create new merge logic - use existing `JinMergeConflict` module
- **Don't** skip validation of conflict markers - must verify user actually resolved
- **Don't** use non-atomic file writes - follow temp file then rename pattern
- **Don't** ignore stale paused state - should warn or error on old state
- **Don't** forget to delete `.jinmerge` files after successful resolution
- **Don't** leave `.paused_apply.yaml` when all conflicts resolved
- **Don't** require manual `jin apply` after resolving all conflicts - auto-complete
- **Don't** use sync I/O in async context (not applicable here, but good practice)
- **Don't** catch all exceptions with generic error handling - be specific
- **Don't** hardcode file paths that should use PathBuf - support cross-platform
- **Don't** assume `.jinmerge` file exists without checking first
- **Don't** modify files in `state.applied_files` - only `state.conflict_files`
- **Don't** forget to update WorkspaceMetadata after resolving files
- **Don't** run tests without `--test-threads=1` - can cause Git lock issues

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success likelihood

**Rationale**:
- Comprehensive context from completed tasks (P1.M1.T1 .jinmerge format, P1.M1.T2 state tracking)
- Clear patterns to follow (apply command, status command)
- All file paths and structures specified
- External research on resolve/retry patterns incorporated
- Test patterns documented with examples
- Gotchas and anti-patterns explicitly called out

**Remaining risks** (minor):
- Status command integration for displaying paused state (P1.M1.T4 - may need coordination)
- Edge cases in conflict marker validation (malformed .jinmerge files)
- Concurrent resolution attempts (two users/terminals resolving same conflicts)

**Validation**: The completed PRP enables an AI agent unfamiliar with the codebase to implement the resolve command successfully using only PRP content and codebase access. All necessary context is provided with specific file paths, line numbers, code patterns, and validation commands.
