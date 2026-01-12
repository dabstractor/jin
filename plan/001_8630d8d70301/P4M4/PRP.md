# PRP: P4.M4 - Workspace Commands

---

## Goal

**Feature Goal**: Implement `jin apply` and `jin reset` commands to enable bidirectional workspace management, allowing users to materialize merged layer configurations to their working directory and reset staged/committed changes.

**Deliverable**: Two fully functional CLI commands:
- `jin apply` - Applies merged layers to workspace with dry-run and force modes
- `jin reset` - Resets staged or committed changes with --soft, --mixed, and --hard modes

**Success Definition**:
- Users can preview merged configuration with `jin apply --dry-run`
- Users can materialize layer configurations to working directory with `jin apply`
- Users can unstage files with `jin reset` (default --mixed mode)
- Users can reset specific layers with layer flags (--mode, --scope, --project)
- All operations are atomic and can be safely interrupted
- Comprehensive error messages guide users on recovery
- All validation levels pass (syntax, unit tests, integration tests)

## Why

**Business Value:**
- Completes the core workflow loop: add → commit → apply → reset
- Enables users to safely experiment with layer configurations
- Provides granular control over workspace state
- Implements Git-familiar reset semantics for easier adoption
- Unblocks P5 (Synchronization) which depends on workspace management

**Integration with Existing Features:**
- Builds on staging system (P3.M1) for file tracking
- Uses layer merge system (P2.M3) for configuration composition
- Leverages transaction system (P1.M3) for atomicity
- Integrates with context management (P4.M3) for layer routing

**Problems This Solves:**
- Users can't currently materialize merged configurations (apply stub is incomplete)
- Users can't undo staged or committed changes (reset stub is incomplete)
- Workflow is one-directional (add/commit only, no rollback)
- Users have no preview mechanism before applying configurations
- No way to selectively reset individual layers

## What

### User-Visible Behavior

#### jin apply

**Basic Usage:**
```bash
jin apply                  # Apply merged layers to workspace
jin apply --dry-run        # Preview what would be applied (no changes)
jin apply --force          # Apply even if workspace has uncommitted changes
```

**Behavior:**
1. Loads active context (mode/scope/project)
2. Determines applicable layers based on context
3. Merges layers using layer merge system (P2.M3)
4. Compares merged result with current workspace files
5. Shows diff of changes that will be applied
6. Writes merged configuration files to working directory
7. Updates `.jin/workspace/` metadata (last-applied tracking)
8. Reports which files were added/modified/removed

**Dry-Run Mode (`--dry-run`):**
- Shows exactly what would be applied
- Displays file-by-file diff
- No modifications to workspace
- Exit code 0 if successful preview, 1 if errors

**Force Mode (`--force`):**
- Bypasses dirty workspace check
- Overwrites uncommitted workspace changes
- Requires explicit user intent for safety

**Error Conditions:**
- Workspace dirty without `--force` → Error with guidance
- Merge conflicts in layers → Error with conflict file paths
- No active context → Error suggesting `jin mode use` or `jin scope use`
- Git repository not initialized → Error suggesting `jin init`

#### jin reset

**Basic Usage:**
```bash
jin reset                                        # Reset staging (default --mixed)
jin reset --soft                                 # Keep changes in staging
jin reset --hard                                 # Discard all changes (DESTRUCTIVE)

# Layer-specific resets
jin reset --mode                                 # Reset active mode base (Layer 2)
jin reset --mode --project                       # Reset mode-project (Layer 5)
jin reset --mode --scope=X                       # Reset mode-scope (Layer 3)
jin reset --scope=X                              # Reset scope base (Layer 6)
```

**Reset Modes:**

**--soft**: Keep changes in staging area
- Clears staging index for specified layer(s)
- Files remain staged (no workspace changes)
- Equivalent to "undo commit, keep staged"

**--mixed** (default): Unstage changes but keep in workspace
- Clears staging index for specified layer(s)
- Removes files from staging
- Workspace files unchanged (modifications preserved)
- Equivalent to "undo commit and unstage"

**--hard**: Discard all changes
- Clears staging index for specified layer(s)
- Removes staged files
- Deletes workspace files (DESTRUCTIVE)
- Requires confirmation prompt or `--force`
- Equivalent to "undo everything, revert to committed state"

**Layer Targeting:**
- No flags → Resets Project Base (Layer 7) - default project layer
- `--mode` → Resets active Mode Base (Layer 2)
- `--mode --project` → Resets Mode → Project (Layer 5)
- `--mode --scope=X` → Resets Mode → Scope (Layer 3)
- `--mode --scope=X --project` → Resets Mode → Scope → Project (Layer 4)
- `--scope=X` → Resets Scope Base (Layer 6)
- `--global` → Resets Global Base (Layer 1)

**Safety Features:**
- `--hard` mode requires confirmation: "Are you sure? This will discard all changes. Type 'yes' to confirm:"
- Shows count of files that will be affected
- Displays warning for destructive operations
- Supports `--force` to skip confirmation for scripting

**Error Conditions:**
- No staged files → Warning "Nothing to reset"
- Invalid layer combination → Error with valid examples
- No active mode with `--mode` flag → Error suggesting `jin mode use`
- No active scope with `--scope` flag → Error suggesting `jin scope use`

### Success Criteria

- [ ] `jin apply` applies merged layers to workspace correctly
- [ ] `jin apply --dry-run` shows accurate preview without modifying files
- [ ] `jin apply --force` overwrites dirty workspace changes
- [ ] `jin apply` detects merge conflicts and reports them clearly
- [ ] `jin reset` (default) unstages files while preserving workspace
- [ ] `jin reset --soft` keeps changes staged
- [ ] `jin reset --hard` prompts for confirmation and discards changes
- [ ] Layer-specific resets target correct layers based on flags
- [ ] All operations are atomic (all succeed or none apply)
- [ ] Clear error messages guide users on recovery
- [ ] Workspace metadata tracks last-applied configuration
- [ ] `.gitignore` managed block updated for applied files

## All Needed Context

### Context Completeness Check

_Before implementing, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: YES - This PRP provides:
- Complete command specifications
- Exact file patterns to follow
- Specific implementation order
- Error handling strategies
- Testing approach
- All necessary context references

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# External Research (Best Practices)
- url: https://git-scm.com/docs/git-apply
  why: Apply command patterns - atomic vs partial application, dry-run modes
  critical: Git's atomic default behavior prevents partial application failures
  section: "--check and --stat for preview patterns"

- url: https://git-scm.com/docs/git-reset
  why: Reset modes (--soft/--mixed/--hard) and safety patterns
  critical: Only --hard can destroy data; implement confirmation prompts
  section: "Reset command safety table and three-step process"

- url: https://kubernetes.io/docs/tasks/manage-kubernetes-objects/declarative-config/
  why: Three-way merge strategy for apply operations
  critical: Track last-applied-configuration for intelligent diffs
  section: "How apply calculates differences and updates objects"

- url: https://clig.dev/
  why: CLI design patterns for destructive operations
  critical: Confirmation prompts, force flags, graceful Ctrl-C handling
  section: "Arguments and flags, Errors, Interactivity"

# Local Research Documentation
- file: plan/P4M4/research/APPLY_COMMAND_RESEARCH.md
  why: Comprehensive apply patterns from Git, Terraform, Kubectl
  pattern: Preview → Validate → Apply workflow with metadata tracking

- file: plan/P4M4/research/RESET_COMMAND_RESEARCH.md
  why: Reset modes, preservation strategies, safety practices
  pattern: Three-tier reset (soft/mixed/hard) with recovery mechanisms

- file: plan/P4M4/research/CODEBASE_PATTERNS.md
  why: Jin-specific implementation patterns and critical gotchas
  pattern: Command structure, transaction usage, error handling

# Codebase Pattern Files
- file: src/commands/add.rs
  why: Full command implementation example with error collection
  pattern: Validation → Context → Routing → Operation → Save → Report
  gotcha: Always check for empty files list before processing

- file: src/commands/mode.rs
  why: Subcommand pattern with validation helpers and comprehensive tests
  pattern: execute() dispatches to private subcommand functions
  gotcha: validate before operations, use consistent error messages

- file: src/commands/commit_cmd.rs
  why: Simple command with pipeline delegation
  pattern: Load context → Call pipeline → Report results
  gotcha: Pipeline handles all transaction logic

- file: src/commit/pipeline.rs
  why: LayerTransaction usage for atomic multi-layer commits
  pattern: Begin transaction → Queue updates → Commit atomically (lines 102-112)
  critical: Transaction automatically rolls back on failure

# Staging System Files
- file: src/staging/index.rs
  why: Staging index management - load, modify, save pattern
  pattern: load().unwrap_or_else(|_| new()) → modify → save()
  gotcha: Entries stored in HashMap (no guaranteed order)

- file: src/staging/workspace.rs
  why: Workspace file operations - read from working directory
  pattern: read_file() from workspace path → create_blob() in Jin repo
  critical: Jin repo is BARE - no working directory, always read from workspace

- file: src/staging/gitignore.rs
  why: Automatic .gitignore management with managed blocks
  pattern: ensure_in_managed_block() → auto-dedupe → sort → write
  gotcha: Never modifies content outside markers

# Layer Merge System
- file: src/merge/layer.rs
  why: Layer merge orchestration - combines multiple layers
  pattern: Determine active layers → Collect files → Deep merge → Return results
  critical: Returns merged_files, conflict_files, added_files, removed_files

# Transaction System
- file: src/git/transaction.rs
  why: Two-phase commit with crash recovery
  pattern: begin() → add_layer_update() → commit() (all or nothing)
  critical: Auto-recovery via RecoveryManager::auto_recover() at startup
  gotcha: Transaction log at .jin/.transaction_in_progress

# Context & Layer Management
- file: src/core/config.rs
  why: ProjectContext load/save, require_mode/require_scope patterns
  pattern: load() with NotInitialized handling → modify → save()
  gotcha: Default context if load fails (non-critical errors)

- file: src/core/layer.rs
  why: Layer enum, precedence, ref_path generation
  pattern: layer.ref_path(mode, scope, project) generates Git ref paths
  critical: WorkspaceActive (Layer 9) is DERIVED - never committed directly

- file: src/core/error.rs
  why: JinError enum and Result type alias
  pattern: Custom error types with thiserror, Result<T> = std::result::Result<T, JinError>
  gotcha: Use #[from] for automatic conversion from io::Error and git2::Error

# CLI Argument Files
- file: src/cli/args.rs
  why: ApplyArgs and ResetArgs structures (already defined)
  pattern: #[derive(Args, Debug)] with clap argument attributes
  gotcha: ApplyArgs has force and dry_run; ResetArgs has soft/mixed/hard and layer flags
```

### Current Codebase Tree

```bash
.
├── src
│   ├── cli
│   │   ├── args.rs              # ApplyArgs, ResetArgs already defined
│   │   └── mod.rs               # CLI parser with Commands enum
│   ├── commands
│   │   ├── add.rs               # ✅ Full implementation example
│   │   ├── apply.rs             # ⚠️  STUB - needs implementation
│   │   ├── commit_cmd.rs        # ✅ Transaction usage example
│   │   ├── mode.rs              # ✅ Subcommand pattern example
│   │   ├── reset.rs             # ⚠️  STUB - needs implementation
│   │   └── mod.rs               # Command dispatcher
│   ├── commit
│   │   └── pipeline.rs          # ✅ LayerTransaction usage (lines 102-112)
│   ├── staging
│   │   ├── index.rs             # ✅ StagingIndex management
│   │   ├── workspace.rs         # ✅ File read/write operations
│   │   ├── gitignore.rs         # ✅ .gitignore managed blocks
│   │   ├── entry.rs             # ✅ StagedEntry types
│   │   └── router.rs            # ✅ Layer routing logic
│   ├── merge
│   │   └── layer.rs             # ✅ Layer merge orchestration
│   ├── git
│   │   └── transaction.rs       # ✅ LayerTransaction implementation
│   └── core
│       ├── config.rs            # ✅ ProjectContext management
│       ├── layer.rs             # ✅ Layer enum and paths
│       └── error.rs             # ✅ JinError types
└── tests
    └── cli_basic.rs             # ✅ Integration test patterns
```

### Desired Codebase Tree (After Implementation)

```bash
src/commands/
├── apply.rs                      # IMPLEMENT: Full apply command
│   # - fn execute(args: ApplyArgs) -> Result<()>
│   # - fn apply_to_workspace(merged: &LayerMergeResult) -> Result<()>
│   # - fn preview_changes(merged: &LayerMergeResult) -> Result<()>
│   # - fn check_workspace_dirty() -> Result<bool>
│   # - Unit tests for all modes
│
├── reset.rs                      # IMPLEMENT: Full reset command
│   # - fn execute(args: ResetArgs) -> Result<()>
│   # - fn reset_staging(layer: Layer, mode: ResetMode) -> Result<()>
│   # - fn reset_workspace(layer: Layer) -> Result<()>
│   # - fn prompt_confirmation(message: &str) -> Result<bool>
│   # - fn determine_target_layer(args: &ResetArgs, context: &ProjectContext) -> Result<Layer>
│   # - Unit tests for all modes and layer combinations
│
└── mod.rs                        # UPDATE: Wire apply and reset (already wired, just stubs)

.jin/
└── workspace/                    # CREATE: Workspace metadata directory
    └── last_applied.json         # TRACK: Last applied configuration metadata
        # {
        #   "timestamp": "2025-12-27T10:30:00Z",
        #   "layers": ["global", "mode/claude", "project/my-proj"],
        #   "files": {
        #     ".claude/config.json": "abc123...",  # Content hash
        #     ".vscode/settings.json": "def456..."
        #   }
        # }

tests/
└── cli_workspace.rs              # CREATE: Integration tests for apply and reset
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Jin repository is BARE (no working directory)
// NEVER try to read files from jin_repo working tree
let jin_repo = JinRepo::open()?;  // This is a bare repo at ~/.jin/
// WRONG: jin_repo.workdir() returns None
// RIGHT: Read from workspace path, create blobs in jin_repo
let content = workspace::read_file(&workspace_path)?;
let oid = jin_repo.create_blob(&content)?;

// CRITICAL: Always check ref existence before resolving
if repo.ref_exists(&ref_path) {
    let oid = repo.resolve_ref(&ref_path)?;  // Safe
} else {
    // Handle missing ref (initial commit case)
}
// WRONG: repo.resolve_ref(&ref_path) without check → may panic

// CRITICAL: WorkspaceActive (Layer 9) is DERIVED - never commit to it
// This layer is the merge result, not a source layer
if layer == Layer::WorkspaceActive {
    return Err(JinError::Other("Cannot commit to workspace layer".into()));
}

// CRITICAL: Use LayerTransaction for multi-layer operations
// Do NOT use JinTransaction directly - it's not truly atomic
let mut tx = LayerTransaction::begin(&repo, "message")?;  // ✅ Correct
// NOT: let mut tx = JinTransaction::new(&repo)?;  // ❌ Wrong for multi-layer

// CRITICAL: Platform-specific file modes
// Unix: Detect executable bit (0o100755 vs 0o100644)
// Windows: Always 0o100644 (no executable bit)
let mode = workspace::get_file_mode(&path);

// CRITICAL: Staging index uses HashMap (no insertion order)
// If order matters, collect and sort keys
let mut paths: Vec<_> = staging.entries.keys().collect();
paths.sort();

// CRITICAL: RecoveryManager should run at startup (not yet implemented)
// Add this to src/lib.rs::run() before executing commands
if let Ok(repo) = JinRepo::open() {
    RecoveryManager::auto_recover(&repo)?;
}

// CRITICAL: Error collection pattern for batch operations
let mut errors = Vec::new();
for item in items {
    match process(item) {
        Ok(_) => success_count += 1,
        Err(e) => errors.push(format!("{}: {}", item, e)),
    }
}
// Report all errors at end, not per-item

// CRITICAL: .gitignore managed block safety
// ALWAYS use ensure_in_managed_block() - it handles:
// - Auto-deduplication
// - Sorting
// - Never touching content outside markers
ensure_in_managed_block(&path)?;
// NOT: Manually editing .gitignore
```

## Implementation Blueprint

### Data Models and Structure

```rust
// No new core data models needed - reuse existing:

// From src/staging/index.rs
struct StagingIndex {
    entries: HashMap<PathBuf, StagedEntry>,
    version: u32,
}

// From src/staging/entry.rs
struct StagedEntry {
    path: PathBuf,
    target_layer: Layer,
    content_hash: String,
    mode: u32,
    operation: StagedOperation,
}

// From src/core/config.rs
struct ProjectContext {
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
}

// NEW: Workspace metadata structure
// Location: .jin/workspace/last_applied.json
#[derive(Serialize, Deserialize)]
struct WorkspaceMetadata {
    timestamp: String,                       // RFC3339 timestamp
    applied_layers: Vec<String>,             // Layer names that were merged
    files: HashMap<PathBuf, String>,         // Path → content hash
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: IMPLEMENT workspace metadata module
  - CREATE: src/staging/metadata.rs
  - IMPLEMENT: WorkspaceMetadata struct with Serialize/Deserialize
  - IMPLEMENT: load(), save(), update() methods
  - NAMING: WorkspaceMetadata, .jin/workspace/last_applied.json
  - PLACEMENT: New module in src/staging/
  - DEPENDENCIES: None (foundational)
  - PURPOSE: Track what was last applied for three-way merge diffs

Task 2: IMPLEMENT apply command - core logic
  - MODIFY: src/commands/apply.rs (replace stub)
  - IMPLEMENT: execute(args: ApplyArgs) -> Result<()>
  - IMPLEMENT: apply_to_workspace(merged: &LayerMergeResult) -> Result<()>
  - IMPLEMENT: preview_changes(merged: &LayerMergeResult) -> Result<()>
  - IMPLEMENT: check_workspace_dirty() -> Result<bool>
  - FOLLOW pattern: src/commands/add.rs (validation → context → operation → report)
  - NAMING: execute(), apply_to_workspace(), preview_changes()
  - DEPENDENCIES: Task 1 (WorkspaceMetadata)
  - CRITICAL: Use src/merge/layer.rs::merge_layers() for layer composition
  - CRITICAL: Write files using std::fs::write() to workspace paths
  - CRITICAL: Update .gitignore managed block for new files

Task 3: IMPLEMENT reset command - layer targeting
  - MODIFY: src/commands/reset.rs (replace stub)
  - IMPLEMENT: determine_target_layer(args: &ResetArgs, context: &ProjectContext) -> Result<Layer>
  - FOLLOW pattern: src/staging/router.rs (layer routing logic)
  - NAMING: determine_target_layer()
  - DEPENDENCIES: None (helper function)
  - CRITICAL: Validate layer flags match available context (mode/scope/project)
  - CRITICAL: Default to ProjectBase (Layer 7) if no flags

Task 4: IMPLEMENT reset command - core logic
  - MODIFY: src/commands/reset.rs
  - IMPLEMENT: execute(args: ResetArgs) -> Result<()>
  - IMPLEMENT: reset_staging(layer: Layer, mode: ResetMode) -> Result<()>
  - IMPLEMENT: reset_workspace(layer: Layer) -> Result<()>
  - IMPLEMENT: prompt_confirmation(message: &str) -> Result<bool>
  - FOLLOW pattern: src/commands/mode.rs (validation → operation → report)
  - NAMING: execute(), reset_staging(), reset_workspace(), prompt_confirmation()
  - DEPENDENCIES: Task 3 (determine_target_layer)
  - CRITICAL: --hard mode MUST prompt for confirmation unless --force
  - CRITICAL: Clear staging index entries for specified layer
  - CRITICAL: Delete workspace files only for --hard mode

Task 5: IMPLEMENT apply command - transaction integration
  - MODIFY: src/commands/apply.rs
  - INTEGRATE: No transaction needed for apply (only writes workspace files)
  - IMPLEMENT: Atomic file writes with temp file + rename pattern
  - FOLLOW pattern: src/git/transaction.rs lines 313-326 (atomic file write)
  - CRITICAL: Write to temp file, then atomic rename for crash safety
  - DEPENDENCIES: Task 2 (apply core logic)

Task 6: IMPLEMENT reset command - staging cleanup
  - MODIFY: src/commands/reset.rs
  - IMPLEMENT: Integration with StagingIndex for --soft and --mixed modes
  - FOLLOW pattern: src/staging/index.rs (load → modify → save)
  - CRITICAL: --soft clears layer entries but leaves staging intact
  - CRITICAL: --mixed removes entries and clears staging
  - CRITICAL: --hard removes entries, clears staging, AND deletes workspace files
  - DEPENDENCIES: Task 4 (reset core logic)

Task 7: IMPLEMENT apply command - conflict detection
  - MODIFY: src/commands/apply.rs
  - IMPLEMENT: detect_conflicts(merged: &LayerMergeResult) -> Vec<PathBuf>
  - FOLLOW pattern: src/merge/layer.rs (conflict_files field in result)
  - CRITICAL: Check LayerMergeResult.conflict_files before applying
  - CRITICAL: Error with clear message listing conflict file paths
  - DEPENDENCIES: Task 2 (apply core logic)

Task 8: CREATE unit tests for apply command
  - CREATE: #[cfg(test)] mod tests in src/commands/apply.rs
  - IMPLEMENT: test_apply_dry_run(), test_apply_force(), test_apply_clean()
  - IMPLEMENT: test_apply_dirty_workspace(), test_apply_with_conflicts()
  - FOLLOW pattern: src/commands/mode.rs (setup_test_env helper)
  - NAMING: test_apply_*, use TempDir for isolation
  - COVERAGE: All modes (dry-run, force, normal), error cases
  - DEPENDENCIES: Task 2, 5, 7 (apply implementation complete)

Task 9: CREATE unit tests for reset command
  - CREATE: #[cfg(test)] mod tests in src/commands/reset.rs
  - IMPLEMENT: test_reset_soft(), test_reset_mixed(), test_reset_hard()
  - IMPLEMENT: test_reset_layer_targeting(), test_reset_confirmation()
  - FOLLOW pattern: src/commands/mode.rs (comprehensive test coverage)
  - NAMING: test_reset_*, use TempDir for isolation
  - COVERAGE: All modes (soft/mixed/hard), all layer combinations
  - DEPENDENCIES: Task 4, 6 (reset implementation complete)

Task 10: CREATE integration tests for workspace commands
  - CREATE: tests/cli_workspace.rs
  - IMPLEMENT: test_apply_workflow(), test_reset_workflow()
  - IMPLEMENT: test_apply_reset_roundtrip()
  - FOLLOW pattern: tests/cli_basic.rs (assert_cmd::Command usage)
  - NAMING: test_*_workflow, full end-to-end scenarios
  - COVERAGE: Complete workflows (add → commit → apply → reset)
  - DEPENDENCIES: Task 8, 9 (unit tests passing)
```

### Implementation Patterns & Key Details

```rust
// =============================================================================
// Pattern 1: Apply Command - Main Flow
// =============================================================================
pub fn execute(args: ApplyArgs) -> Result<()> {
    // 1. Load context
    let context = ProjectContext::load().unwrap_or_default();

    // 2. Check workspace dirty (unless --force)
    if !args.force && check_workspace_dirty()? {
        return Err(JinError::WorkspaceDirty {
            message: "Workspace has uncommitted changes. Use --force to override.".into()
        });
    }

    // 3. Open repository
    let repo = JinRepo::open_or_create()?;

    // 4. Merge layers based on active context
    let config = LayerMergeConfig {
        mode: context.mode.as_deref(),
        scope: context.scope.as_deref(),
        project: context.project.as_deref(),
    };
    let merged = merge_layers(&config, &repo)?;

    // 5. Check for conflicts
    if !merged.conflict_files.is_empty() {
        eprintln!("Merge conflicts detected in {} files:", merged.conflict_files.len());
        for path in &merged.conflict_files {
            eprintln!("  - {}", path.display());
        }
        return Err(JinError::MergeConflict {
            files: merged.conflict_files,
        });
    }

    // 6. Preview mode - show diff and exit
    if args.dry_run {
        preview_changes(&merged)?;
        return Ok(());
    }

    // 7. Apply to workspace
    apply_to_workspace(&merged)?;

    // 8. Update workspace metadata
    let metadata = WorkspaceMetadata {
        timestamp: chrono::Utc::now().to_rfc3339(),
        applied_layers: merged.source_layers,
        files: merged.merged_files.iter()
            .map(|(path, file)| (path.clone(), file.content_hash.clone()))
            .collect(),
    };
    metadata.save()?;

    // 9. Update .gitignore managed block
    for path in merged.merged_files.keys() {
        ensure_in_managed_block(path)?;
    }

    // 10. Report results
    println!("Applied {} files to workspace", merged.merged_files.len());
    if !merged.added_files.is_empty() {
        println!("  Added: {}", merged.added_files.len());
    }
    if !merged.removed_files.is_empty() {
        println!("  Removed: {}", merged.removed_files.len());
    }

    Ok(())
}

// =============================================================================
// Pattern 2: Apply to Workspace - Atomic File Writes
// =============================================================================
fn apply_to_workspace(merged: &LayerMergeResult) -> Result<()> {
    let mut applied_count = 0;
    let mut errors = Vec::new();

    // Process each merged file
    for (path, merged_file) in &merged.merged_files {
        match apply_file(path, merged_file) {
            Ok(_) => applied_count += 1,
            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
        }
    }

    // Report errors
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if applied_count == 0 {
            return Err(JinError::ApplyFailed {
                message: "Failed to apply any files".into()
            });
        }
    }

    Ok(())
}

fn apply_file(path: &Path, merged_file: &MergedFile) -> Result<()> {
    // CRITICAL: Atomic write pattern (temp file + rename)
    // From src/git/transaction.rs lines 313-326

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to temp file
    let temp_path = path.with_extension("jin-tmp");
    std::fs::write(&temp_path, &merged_file.content)?;

    // Atomic rename
    std::fs::rename(&temp_path, &path)?;

    // Set file mode (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(merged_file.mode);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

// =============================================================================
// Pattern 3: Preview Changes
// =============================================================================
fn preview_changes(merged: &LayerMergeResult) -> Result<()> {
    println!("Would apply {} files:", merged.merged_files.len());

    // Show added files
    if !merged.added_files.is_empty() {
        println!("\nAdded files:");
        for path in &merged.added_files {
            println!("  + {}", path.display());
        }
    }

    // Show modified files (compare with workspace)
    let mut modified = Vec::new();
    for (path, merged_file) in &merged.merged_files {
        if path.exists() {
            let workspace_content = std::fs::read(path)?;
            if workspace_content != merged_file.content {
                modified.push(path);
            }
        }
    }
    if !modified.is_empty() {
        println!("\nModified files:");
        for path in modified {
            println!("  M {}", path.display());
        }
    }

    // Show removed files
    if !merged.removed_files.is_empty() {
        println!("\nRemoved files:");
        for path in &merged.removed_files {
            println!("  - {}", path.display());
        }
    }

    Ok(())
}

// =============================================================================
// Pattern 4: Reset Command - Main Flow
// =============================================================================
pub fn execute(args: ResetArgs) -> Result<()> {
    // 1. Determine reset mode
    let mode = if args.soft {
        ResetMode::Soft
    } else if args.hard {
        ResetMode::Hard
    } else {
        ResetMode::Mixed  // Default
    };

    // 2. Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. Determine target layer
    let layer = determine_target_layer(&args, &context)?;

    // 4. Load staging
    let mut staging = StagingIndex::load()?;

    // 5. Get affected entries
    let entries: Vec<_> = staging.entries_for_layer(layer).collect();
    if entries.is_empty() {
        println!("Nothing to reset for layer: {}", layer_name(&layer));
        return Ok(());
    }

    // 6. Confirmation for --hard mode
    if mode == ResetMode::Hard && !args.force {
        let count = entries.len();
        let message = format!(
            "This will discard {} file(s) from staging AND workspace. Type 'yes' to confirm:",
            count
        );
        if !prompt_confirmation(&message)? {
            println!("Reset cancelled");
            return Ok(());
        }
    }

    // 7. Perform reset based on mode
    match mode {
        ResetMode::Soft => {
            // Keep in staging, just mark as "reset" (no-op for now)
            println!("Reset {} file(s) (kept in staging)", entries.len());
        }
        ResetMode::Mixed => {
            // Remove from staging, keep in workspace
            reset_staging(&mut staging, layer)?;
            staging.save()?;
            println!("Unstaged {} file(s) (kept in workspace)", entries.len());
        }
        ResetMode::Hard => {
            // Remove from staging AND workspace
            reset_staging(&mut staging, layer)?;
            reset_workspace(&entries)?;
            staging.save()?;
            println!("Discarded {} file(s) from staging and workspace", entries.len());
        }
    }

    Ok(())
}

// =============================================================================
// Pattern 5: Reset Staging - Remove Entries for Layer
// =============================================================================
fn reset_staging(staging: &mut StagingIndex, layer: Layer) -> Result<()> {
    let paths_to_remove: Vec<_> = staging.entries_for_layer(layer)
        .map(|e| e.path.clone())
        .collect();

    for path in paths_to_remove {
        staging.remove(&path);
    }

    Ok(())
}

// =============================================================================
// Pattern 6: Reset Workspace - Delete Files
// =============================================================================
fn reset_workspace(entries: &[&StagedEntry]) -> Result<()> {
    let mut errors = Vec::new();

    for entry in entries {
        // Remove from workspace
        if entry.path.exists() {
            if let Err(e) = std::fs::remove_file(&entry.path) {
                errors.push(format!("{}: {}", entry.path.display(), e));
            }
        }

        // Remove from .gitignore managed block
        if let Err(e) = remove_from_managed_block(&entry.path) {
            errors.push(format!("{}: {}", entry.path.display(), e));
        }
    }

    if !errors.is_empty() {
        eprintln!("Errors during workspace reset:");
        for error in &errors {
            eprintln!("  {}", error);
        }
    }

    Ok(())
}

// =============================================================================
// Pattern 7: Determine Target Layer from Flags
// =============================================================================
fn determine_target_layer(args: &ResetArgs, context: &ProjectContext) -> Result<Layer> {
    // FOLLOW: src/staging/router.rs routing logic

    // --mode + --scope=X + --project → Layer 4 (ModeScopeProject)
    if args.mode && args.scope.is_some() && args.project {
        let mode = context.require_mode()?;
        let scope = args.scope.as_ref().unwrap();
        return Ok(Layer::ModeScopeProject);
    }

    // --mode + --scope=X → Layer 3 (ModeScope)
    if args.mode && args.scope.is_some() {
        let mode = context.require_mode()?;
        let scope = args.scope.as_ref().unwrap();
        return Ok(Layer::ModeScope);
    }

    // --mode + --project → Layer 5 (ModeProject)
    if args.mode && args.project {
        let mode = context.require_mode()?;
        return Ok(Layer::ModeProject);
    }

    // --mode → Layer 2 (ModeBase)
    if args.mode {
        let mode = context.require_mode()?;
        return Ok(Layer::ModeBase);
    }

    // --scope=X → Layer 6 (ScopeBase)
    if let Some(ref scope) = args.scope {
        return Ok(Layer::ScopeBase);
    }

    // --project → Error (requires --mode)
    if args.project {
        return Err(JinError::Other(
            "--project requires --mode (use --mode --project)".into()
        ));
    }

    // Default: Layer 7 (ProjectBase)
    Ok(Layer::ProjectBase)
}

// =============================================================================
// Pattern 8: Confirmation Prompt
// =============================================================================
fn prompt_confirmation(message: &str) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("yes"))
}

// =============================================================================
// Pattern 9: Check Workspace Dirty
// =============================================================================
fn check_workspace_dirty() -> Result<bool> {
    // Check if workspace has uncommitted changes
    // This is simplified - in reality, would check:
    // 1. Modified files in workspace
    // 2. Files in staging area
    // 3. Untracked files (optionally)

    // For now, just check if any tracked workspace files have changed
    // compared to last applied configuration

    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(_) => return Ok(false),  // No metadata = clean
    };

    for (path, hash) in &metadata.files {
        if !path.exists() {
            return Ok(true);  // File deleted
        }

        let content = std::fs::read(path)?;
        let repo = JinRepo::open()?;
        let current_hash = repo.create_blob(&content)?;
        if current_hash.to_string() != *hash {
            return Ok(true);  // File modified
        }
    }

    Ok(false)
}
```

### Integration Points

```yaml
STAGING SYSTEM:
  - use: src/staging/index.rs::StagingIndex for reset operations
  - pattern: "load() → entries_for_layer() → remove() → save()"

LAYER MERGE:
  - use: src/merge/layer.rs::merge_layers() for apply operations
  - pattern: "LayerMergeConfig → merge_layers() → LayerMergeResult"

WORKSPACE FILES:
  - use: src/staging/workspace.rs::read_file() for dirty detection
  - pattern: "read_file(path) → compare hash"

GITIGNORE:
  - use: src/staging/gitignore.rs::ensure_in_managed_block() for apply
  - use: src/staging/gitignore.rs::remove_from_managed_block() for reset --hard
  - pattern: "Auto-dedupe, sort, write atomically"

CONTEXT:
  - use: src/core/config.rs::ProjectContext for active mode/scope/project
  - pattern: "load() → require_mode()/require_scope() → use in routing"

REPOSITORY:
  - use: src/git/repo.rs::JinRepo for blob creation and ref operations
  - pattern: "open_or_create() → create_blob() → resolve_ref()"

NEW MODULE:
  - create: src/staging/metadata.rs for WorkspaceMetadata
  - exports: WorkspaceMetadata struct, load(), save() functions
  - pattern: "Serialize to .jin/workspace/last_applied.json"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding

# Check apply.rs
cargo fmt -- --check src/commands/apply.rs
cargo clippy -- -D warnings src/commands/apply.rs

# Check reset.rs
cargo fmt -- --check src/commands/reset.rs
cargo clippy -- -D warnings src/commands/reset.rs

# Check metadata.rs (new file)
cargo fmt -- --check src/staging/metadata.rs
cargo clippy -- -D warnings src/staging/metadata.rs

# Project-wide validation
cargo fmt -- --check
cargo clippy -- -D warnings

# Build check
cargo build

# Expected: Zero errors, zero warnings. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test apply command in isolation
cargo test --lib commands::apply::tests -- --nocapture

# Test reset command in isolation
cargo test --lib commands::reset::tests -- --nocapture

# Test metadata module
cargo test --lib staging::metadata::tests -- --nocapture

# Run all unit tests
cargo test --lib

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Test workspace commands end-to-end
cargo test --test cli_workspace -- --nocapture

# Test apply workflow
cargo test --test cli_workspace test_apply_workflow -- --nocapture

# Test reset workflow
cargo test --test cli_workspace test_reset_workflow -- --nocapture

# Test apply + reset roundtrip
cargo test --test cli_workspace test_apply_reset_roundtrip -- --nocapture

# Run all integration tests
cargo test --test '*'

# Expected: All integrations working, proper responses, no panics
```

### Level 4: Manual Validation & Real-World Testing

```bash
# Setup test project
mkdir /tmp/jin-test && cd /tmp/jin-test
git init
jin init
jin link git@github.com:test/jin-config

# Test apply workflow
jin mode create claude
jin mode use claude
jin add .claude/config.json --mode
jin commit -m "Add claude config"

# Preview apply
jin apply --dry-run
# Expected: Shows files that would be applied

# Apply to workspace
jin apply
# Expected: Creates .claude/config.json in working directory
# Expected: Updates .gitignore managed block

# Modify workspace file
echo '{"modified": true}' > .claude/config.json

# Try apply without force (should fail)
jin apply
# Expected: Error "Workspace has uncommitted changes. Use --force to override."

# Force apply
jin apply --force
# Expected: Overwrites workspace changes

# Test reset workflow
jin add .vscode/settings.json --mode
# Expected: File staged

# Reset (default --mixed)
jin reset --mode
# Expected: File unstaged but still in workspace

# Reset --soft
jin add .vscode/settings.json --mode
jin reset --soft --mode
# Expected: File still staged

# Reset --hard (should prompt)
jin add .vscode/settings.json --mode
jin reset --hard --mode
# Expected: Prompts "Type 'yes' to confirm:"
# After confirmation: File removed from staging AND workspace

# Test layer-specific resets
jin add file1.txt --mode
jin add file2.txt --scope=lang:rust
jin add file3.txt --project
jin reset --mode        # Only resets file1.txt
jin reset --scope=lang:rust  # Only resets file2.txt
jin reset               # Only resets file3.txt (default --project)

# Expected: Each reset targets only specified layer
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test '*'`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt -- --check`
- [ ] Builds successfully: `cargo build`

### Feature Validation (Apply Command)

- [ ] `jin apply` merges layers and writes to workspace
- [ ] `jin apply --dry-run` shows accurate preview without changes
- [ ] `jin apply --force` overwrites dirty workspace
- [ ] Apply detects and reports merge conflicts
- [ ] Apply creates .jin/workspace/last_applied.json metadata
- [ ] Apply updates .gitignore managed block for new files
- [ ] Apply reports added/modified/removed file counts
- [ ] Apply errors clearly when no active context
- [ ] Apply atomic file writes prevent partial state (temp + rename)

### Feature Validation (Reset Command)

- [ ] `jin reset` (default --mixed) unstages files, keeps workspace
- [ ] `jin reset --soft` keeps files staged
- [ ] `jin reset --hard` prompts for confirmation
- [ ] `jin reset --hard` with confirmation removes staging AND workspace files
- [ ] Layer targeting works: --mode, --scope, --project flags
- [ ] Default reset targets ProjectBase (Layer 7)
- [ ] Reset errors clearly on invalid layer combinations
- [ ] Reset shows count of affected files
- [ ] Reset removes entries from .gitignore managed block (--hard only)
- [ ] Reset handles empty staging gracefully ("Nothing to reset")

### Code Quality Validation

- [ ] Follows existing command patterns from src/commands/add.rs and mode.rs
- [ ] Error handling uses JinError enum with clear messages
- [ ] File operations use atomic write pattern (temp + rename)
- [ ] No panics - all unwrap() replaced with proper error handling
- [ ] Comprehensive unit tests with setup_test_env() helper
- [ ] Integration tests cover full workflows
- [ ] Code is self-documenting with clear variable/function names

### Documentation & Safety

- [ ] --hard mode warns users about data loss
- [ ] Confirmation prompts for destructive operations
- [ ] Dry-run mode accurately previews changes
- [ ] Error messages explain what went wrong and how to fix
- [ ] Force flag documented and used consistently
- [ ] Workspace dirty detection prevents accidental overwrites

---

## Anti-Patterns to Avoid

- ❌ Don't read files from Jin repo working directory (it's BARE)
- ❌ Don't use JinTransaction directly - use LayerTransaction for consistency
- ❌ Don't commit to WorkspaceActive (Layer 9) - it's derived, not a source
- ❌ Don't resolve refs without checking existence first (may panic)
- ❌ Don't write files without atomic rename pattern (crash-unsafe)
- ❌ Don't skip confirmation for --hard mode (destructive)
- ❌ Don't unwrap() without proper error context
- ❌ Don't modify .gitignore directly - use ensure_in_managed_block()
- ❌ Don't ignore errors during batch operations - collect and report all
- ❌ Don't assume HashMap iteration order in staging index

---

## Confidence Score: 9/10

**Rationale for High Confidence:**

**Strengths:**
- Complete context provided (PRD sections, existing patterns, research)
- Clear implementation order with dependencies
- Specific file patterns to follow with line numbers
- Comprehensive error handling strategies
- Well-researched apply and reset patterns from Git, Terraform, Kubectl
- Existing codebase provides strong patterns to follow
- Transaction system already handles atomicity
- Staging system already implemented and tested

**Minor Risks:**
- Layer merge system (src/merge/layer.rs) behavior in edge cases (but can test incrementally)
- Workspace dirty detection heuristics (can iterate based on testing)
- File permission handling on Windows (but existing code has pattern)

**Mitigation:**
- Start with unit tests (Level 2) before integration tests
- Test each mode (soft/mixed/hard) independently
- Validate dry-run mode thoroughly before live apply
- Use TempDir for all testing (no accidental file deletion)

**Why Not 10/10:**
- Haven't personally validated the layer merge result structure
- Some assumptions about MergedFile contents (but can inspect type)
- Workspace dirty detection may need refinement based on real usage

---

## Implementation Validation

The completed implementation should enable an AI agent unfamiliar with the codebase to:

1. ✅ Understand the exact behavior of `jin apply` and `jin reset`
2. ✅ Follow existing patterns from referenced files
3. ✅ Implement atomic operations with proper error handling
4. ✅ Write comprehensive unit and integration tests
5. ✅ Handle edge cases (conflicts, dirty workspace, missing context)
6. ✅ Provide clear user feedback and error messages
7. ✅ Pass all 4 validation levels without manual intervention

**Success Metric:** One-pass implementation with all tests passing and no critical issues.
