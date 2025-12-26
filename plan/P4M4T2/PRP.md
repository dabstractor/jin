# PRP: Reset Command (P4.M4.T2)

---

## Goal

**Feature Goal**: Implement the `jin reset` command that allows users to undo staged and committed changes to Jin layers, following Git's reset semantics with `--soft`, `--mixed`, and `--hard` modes while respecting Jin's layer-based architecture.

**Deliverable**: A fully functional `jin reset` command that:
- Unstages files from the staging area (all reset modes)
- Optionally discards workspace file changes (hard mode)
- Targets specific layers based on `--mode`, `--scope`, `--project` flags
- Supports path-specific resets (reset only specified files)
- Provides clear user feedback on what will be and was reset

**Success Definition**:
- All three reset modes (`--soft`, `--mixed`, `--hard`) work correctly
- Staging index is properly updated (entries removed)
- Workspace files are optionally discarded (hard mode only)
- Layer targeting follows Jin's routing rules
- User sees clear confirmation of files affected
- Unit tests cover all reset modes and layer combinations

## User Persona

**Target User**: Developer working with Jin's layered configuration system who needs to undo staged changes or discard modifications to configuration files.

**Use Case**: After staging files with `jin add` but before committing, the user realizes they made a mistake and wants to either unstage the files or discard the changes entirely.

**User Journey**:
1. Developer stages files: `jin add config.toml --mode`
2. Developer realizes the changes are incorrect
3. Developer runs `jin reset --mode` to unstage
4. Files are removed from staging but kept in workspace
5. Developer can make new changes and re-stage

**Pain Points Addressed**:
- No manual editing of `.jin/staging/index.json` to unstage files
- Clear mental model matching `git reset` behavior
- Safe undo without losing work (unless `--hard` is explicitly used)
- Layer-aware reset (only affect files from specific layers)

## Why

- **Business value**: Provides a critical undo capability for Jin's workflow, enabling safe experimentation and error recovery
- **Integration**: Completes the add/commit/reset workflow trio that users expect from Git-like tools
- **Problems solved**:
  - Users make mistakes and need to undo staging
  - Testing different configurations requires quick reset/retry cycles
  - No manual manipulation of Jin's internal files

## What

The `jin reset` command unstages files from Jin's staging area, with optional workspace file discarding.

### Command Interface

```bash
# Unstage all files from project base layer (default)
jin reset

# Unstage all files from active mode layer
jin reset --mode

# Unstage specific files from mode layer
jin reset --mode config.toml settings.json

# Unstage and discard workspace changes (hard mode)
jin reset --hard --mode

# Keep staging area but update layer refs (soft mode - for future use)
jin reset --soft --mode

# Unstage but keep workspace files (mixed mode - default)
jin reset --mixed --mode
```

### Reset Modes

| Mode | Staging Index | Workspace Files | Use Case |
|------|--------------|-----------------|----------|
| `--soft` | Kept | Kept | Future: Move layer ref only (not implemented in this task) |
| `--mixed` (default) | Cleared | Kept | Unstage changes but keep work |
| `--hard` | Cleared | Discarded | Completely undo changes |

### Success Criteria

- [ ] Unstages files from staging index (all modes)
- [ ] Supports path-specific resets (reset only specified files)
- [ ] Supports layer targeting (`--mode`, `--scope`, `--project`)
- [ ] `--hard` discards workspace file changes
- [ ] `--mixed` (default) unstages but keeps workspace files
- [ ] `--soft` unstage only (for future layer ref movement)
- [ ] Shows summary of affected files before and after reset
- [ ] Unit tests cover all reset modes and layer combinations

## All Needed Context

### Context Completeness Check

**Passes "No Prior Knowledge" test**: This PRP provides complete file paths, exact code patterns, StagingIndex API usage, layer routing logic, testing patterns, and git2-rs integration. An implementer needs only this PRP and codebase access.

### Documentation & References

```yaml
# MUST READ - Command implementation pattern
- file: src/commands/add.rs
  why: Complete pattern for layer routing, project detection, staging operations
  pattern: execute() -> detect_project_name() -> determine_layer() -> staging operations -> save -> summary
  gotcha: Layer::from_flags() for explicit routing, context defaults for implicit

# MUST READ - Command implementation pattern
- file: src/commands/commit.rs
  why: Pattern for loading/storing StagingIndex, detecting project name
  pattern: StagingIndex::load_from_disk() -> normalize paths -> operations -> save_to_disk()
  gotcha: StagingIndex stores relative paths; convert from absolute using strip_prefix()

# MUST READ - StagingIndex API
- file: src/staging/index.rs
  why: Core API for staging operations needed by reset
  pattern:
    - StagingIndex::load_from_disk(&workspace_root) -> Result<StagingIndex>
    - staging.remove_entry(&path) -> Option<StagedEntry>
    - staging.clear() -> removes all entries
    - staging.save_to_disk(&workspace_root) -> Result<()>
    - staging.entries_by_layer(&layer) -> Vec<&StagedEntry>
    - staging.is_empty() -> bool
  gotcha: Paths in staging index are relative to workspace root

# MUST READ - Layer routing
- file: src/core/layer.rs
  why: Layer enum with from_flags() method for determining target layer
  pattern: Layer::from_flags(mode, scope, project, global) -> Option<Layer>
  gotcha: Returns None if no combination matches; handle gracefully
  critical:
    - Use same layer routing logic as add.rs
    - Respect active context when no explicit flags provided

# MUST READ - Project context
- file: src/core/config.rs
  why: ProjectContext loads active mode/scope from .jin/context
  pattern: ProjectContext::load(&workspace_root) -> Result<ProjectContext>
  gotcha: context.mode and context.scope are Option<String>

# MUST READ - CLI definition
- file: src/cli/args.rs:242-272
  why: ResetCommand struct already defined with all flags
  pattern: paths: Vec<PathBuf>, mode: bool, scope: Option<String>, project: bool
  gotcha: Already wired in main.rs as placeholder - just needs implementation

# MUST READ - Error handling
- file: src/core/error.rs
  why: JinError variants for proper error handling
  pattern: Use JinError::Message for custom errors, JinError::FileNotFound for missing files
  gotcha: Provide helpful error messages suggesting what user should do

# MUST READ - JinRepo for layer operations
- file: src/git/repo.rs
  why: JinRepo wraps git2 operations for layer refs
  pattern:
    - repo.get_layer_ref(&layer) -> Result<Option<git2::Reference>>
    - repo.set_layer_ref(&layer, oid) -> Result<git2::Reference>
  gotcha: For initial reset, we only need staging operations - layer ref operations are future work

# MUST READ - git2-rs reset API (from research)
- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.reset
  why: For future hard reset using git operations
  critical: Not needed for initial implementation - reset is staging-only
  note: Future work for --hard mode with layer refs would use git2::ResetType

# MUST READ - CLI reset UX patterns
- docfile: git2-rs-reset-research.md (in project root)
  why: Research on git2 reset API and best practices
  section: Reset Mode Details, Error Handling and Validation
  critical: Use these patterns for future hard reset with git operations

# MUST READ - Apply command for reference
- file: src/commands/apply.rs
  why: Shows workspace file operations for hard reset
  pattern: std::fs::remove_file() for discarding workspace changes
  gotcha: Check file exists before removing
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── src/
│   ├── cli/
│   │   ├── args.rs          # ResetCommand defined at line 242-272
│   │   └── mod.rs
│   ├── commands/
│   │   ├── add.rs           # Reference: layer routing, staging operations
│   │   ├── commit.rs        # Reference: staging load/save patterns
│   │   ├── status.rs        # Reference: staging display patterns
│   │   ├── apply.rs         # Reference: workspace file operations
│   │   ├── mod.rs           # ADD: pub mod reset; pub use reset::execute as reset_execute;
│   │   └── reset.rs         # CREATE THIS FILE
│   ├── core/
│   │   ├── config.rs        # ProjectContext::load()
│   │   ├── error.rs         # JinError variants
│   │   └── layer.rs         # Layer enum and from_flags()
│   ├── staging/
│   │   ├── index.rs         # StagingIndex API
│   │   └── entry.rs         # StagedEntry structure
│   └── main.rs              # Line 145-149: ResetCommand dispatcher (needs update)
└── plan/P4M4T2/
    ├── research/
    │   └── (research docs from agents)
    └── PRP.md               # This document
```

### Desired Codebase Tree (files to be added)

```bash
# NEW FILE TO CREATE
├── src/commands/reset.rs    # Main reset command implementation
│   ├── execute()            # Entry point
│   ├── determine_reset_layer() # Layer targeting logic
│   ├── filter_paths_by_layer() # Get paths to reset from target layer
│   ├── execute_soft_reset() # --soft mode (unstage only)
│   ├── execute_mixed_reset() # --mixed mode (unstage, keep workspace)
│   ├── execute_hard_reset() # --hard mode (unstage + discard workspace)
│   └── tests module         # Unit tests

# MODIFY
├── src/commands/mod.rs      # ADD: pub mod reset; pub use reset::execute as reset_execute;
├── src/main.rs              # UPDATE: Commands::Reset(cmd) -> call reset_execute(&cmd)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: StagingIndex stores paths as RELATIVE to workspace root
// When adding files, add.rs uses absolute path for metadata but converts to relative
// When resetting, cmd.paths may be relative or absolute - normalize to relative
let relative_path = path.strip_prefix(&workspace_root)
    .unwrap_or(&path)
    .to_path_buf();

// CRITICAL: Layer::from_flags() returns Option<Layer> - handle None case
let layer = Layer::from_flags(mode, scope, project, global)
    .ok_or_else(|| JinError::Message("Invalid layer combination".to_string()))?;

// CRITICAL: Active context from ProjectContext
let context = ProjectContext::load(&workspace_root)?;
// context.mode and context.scope are Option<String>
// Use context.mode.as_deref() for Option<&str>

// CRITICAL: detect_project_name() helper pattern (from add.rs)
// Try git remote origin, fallback to directory name
let project_name = detect_project_name(&workspace_root)?;

// CRITICAL: StagingIndex operations
let mut staging = StagingIndex::load_from_disk(&workspace_root)
    .unwrap_or_else(|_| StagingIndex::new());
// Use staging.remove_entry(&relative_path) for specific files
// Use staging.clear() for all files
// Always call staging.save_to_disk(&workspace_root)? after modifications

// CRITICAL: Filtering staged entries by layer
// Use staging.entries_by_layer(&target_layer) to get entries from specific layer
let layer_entries: Vec<&StagedEntry> = staging.entries_by_layer(&target_layer);

// CRITICAL: For hard reset - check file exists before removing
let workspace_file = workspace_root.join(&relative_path);
if workspace_file.exists() {
    std::fs::remove_file(&workspace_file)?;
}

// CRITICAL: Reset mode determination - clap ensures mutual exclusivity
// Only one of soft, mixed, hard can be true at a time
// Default (all false) is treated as mixed mode
let reset_mode = match (cmd.soft, cmd.mixed, cmd.hard) {
    (true, _, _) => ResetMode::Soft,
    (_, true, _) => ResetMode::Mixed,
    (_, _, true) => ResetMode::Hard,
    (false, false, false) => ResetMode::Mixed, // Default
};

// CRITICAL: Path normalization for empty paths vector
// If cmd.paths is empty, reset ALL files from target layer
let paths_to_reset = if cmd.paths.is_empty() {
    // Get all entries from target layer
    staging.entries_by_layer(&target_layer)
        .iter()
        .map(|e| e.path.clone())
        .collect()
} else {
    // Use specified paths
    cmd.paths.iter()
        .map(|p| normalize_path(p, &workspace_root))
        .collect::<Result<Vec<_>>>()?
};

// CRITICAL: Summary output pattern (from add.rs)
println!("\nSummary:");
for (layer_name, files) in affected_by_layer {
    println!("  {}:", layer_name);
    for file in files {
        println!("    - {}", file.display());
    }
}

// CRITICAL: Error messages should be helpful
return Err(JinError::Message(
    "No staged files found in target layer. Use 'jin status' to see staged files.".to_string()
));

// CRITICAL: Reset doesn't touch Git layer refs (future work)
// Initial implementation only operates on staging index
// Moving Git refs would require JinRepo operations and git2::ResetType
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - using existing types:
- `ResetCommand` (from `src/cli/args.rs`) - CLI arguments
- `ProjectContext` (from `src/core/config.rs`) - Active mode/scope
- `Layer` (from `src/core/layer.rs`) - Layer targeting
- `StagingIndex` (from `src/staging/index.rs`) - Staging operations
- `ResetMode` (local enum) - Reset mode tracking

```rust
// Local enum for reset mode tracking
enum ResetMode {
    Soft,   // Unstage only
    Mixed,  // Unstage, keep workspace (default)
    Hard,   // Unstage + discard workspace
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/reset.rs with module structure
  - IMPLEMENT: Module skeleton with use statements
  - INCLUDE: crate::cli::args::ResetCommand, crate::core::error::{JinError, Result}
  - INCLUDE: crate::core::{config::ProjectContext, layer::Layer}
  - INCLUDE: crate::staging::{index::StagingIndex, entry::StagedEntry}
  - INCLUDE: std::path::{Path, PathBuf}
  - PATTERN: Follow src/commands/add.rs structure
  - NAMING: execute() function, helper functions below

Task 2: IMPLEMENT determine_reset_layer() helper function
  - SIGNATURE: fn determine_reset_layer(cmd: &ResetCommand, context: &ProjectContext, project: &str) -> Result<Layer>
  - LOGIC: Use Layer::from_flags() with explicit flags first, fall back to context
  - REFERENCE: src/commands/add.rs:determine_layer() (similar pattern)
  - PATTERN:
    if cmd.mode || cmd.scope.is_some() || cmd.project {
        return Layer::from_flags(mode, scope, proj, false)
            .ok_or_else(|| JinError::Message("Invalid layer combination".to_string()));
    }
    // Fall back to active context or project base
  - GOTCHA: No --global flag for reset (unlike add)

Task 3: IMPLEMENT normalize_path() helper function
  - SIGNATURE: fn normalize_path(path: &Path, workspace_root: &Path) -> Result<PathBuf>
  - LOGIC: Convert absolute paths to relative, keep relative paths as-is
  - PATTERN:
    if path.is_absolute() {
        Ok(path.strip_prefix(workspace_root)
            .map_err(|_| JinError::Message("Path outside workspace".to_string()))?
            .to_path_buf())
    } else {
        Ok(path.to_path_buf())
    }

Task 4: IMPLEMENT get_paths_to_reset() helper function
  - SIGNATURE: fn get_paths_to_reset(cmd: &ResetCommand, target_layer: &Layer, staging: &StagingIndex, workspace_root: &Path) -> Result<Vec<PathBuf>>
  - LOGIC:
    - If cmd.paths is empty: return all paths from target_layer in staging
    - If cmd.paths has entries: normalize and return those paths
  - PATTERN:
    let layer_entries = staging.entries_by_layer(target_layer);
    if cmd.paths.is_empty() {
        Ok(layer_entries.iter().map(|e| e.path.clone()).collect())
    } else {
        cmd.paths.iter().map(|p| normalize_path(p, workspace_root)).collect()
    }

Task 5: IMPLEMENT execute_soft_reset() helper function
  - SIGNATURE: fn execute_soft_reset(paths: &[PathBuf], staging: &mut StagingIndex) -> Result<usize>
  - LOGIC: Remove entries from staging for specified paths
  - PATTERN:
    let mut count = 0;
    for path in paths {
        if staging.remove_entry(path).is_some() {
            count += 1;
        }
    }
    Ok(count)
  - RETURN: Count of entries actually removed

Task 6: IMPLEMENT execute_mixed_reset() helper function
  - SIGNATURE: fn execute_mixed_reset(paths: &[PathBuf], staging: &mut StagingIndex) -> Result<usize>
  - LOGIC: Same as soft reset - unstaging is the same for both modes
  - NOTE: The difference between soft/mixed is only relevant for Git ref operations (future)
  - PATTERN: Call execute_soft_reset() internally

Task 7: IMPLEMENT execute_hard_reset() helper function
  - SIGNATURE: fn execute_hard_reset(paths: &[PathBuf], staging: &mut StagingIndex, workspace_root: &Path) -> Result<usize>
  - LOGIC:
    1. Remove entries from staging (same as soft/mixed)
    2. For each path, remove the file from workspace if it exists
  - PATTERN:
    let mut count = 0;
    for path in paths {
        if staging.remove_entry(path).is_some() {
            count += 1;
        }
        let workspace_file = workspace_root.join(path);
        if workspace_file.exists() {
            std::fs::remove_file(&workspace_file)?;
            println!("  Removed: {}", path.display());
        }
    }
    Ok(count)
  - GOTCHA: Check file exists before removing to avoid errors

Task 8: IMPLEMENT execute() main function
  - SIGNATURE: pub fn execute(cmd: &ResetCommand) -> Result<()>
  - STEP 1: Get workspace_root (std::env::current_dir()?)
  - STEP 2: Load ProjectContext from workspace_root
  - STEP 3: Detect project name using detect_project_name()
  - STEP 4: Validate Git repository exists
  - STEP 5: Open JinRepo using JinRepo::open_or_create()
  - STEP 6: Load StagingIndex from disk
  - STEP 7: Determine target layer using determine_reset_layer()
  - STEP 8: Get paths to reset using get_paths_to_reset()
  - STEP 9: Determine reset mode (soft/mixed/hard)
  - STEP 10: Show preview of what will be reset
  - STEP 11: Execute appropriate reset function
  - STEP 12: Save staging index to disk
  - STEP 13: Print success summary
  - ERROR: Propagate all errors with context
  - PATTERN: Follow src/commands/add.rs::execute() structure

Task 9: ADD detect_project_name() helper function
  - SIGNATURE: fn detect_project_name(workspace_root: &Path) -> Result<String>
  - LOGIC: Copy from src/commands/add.rs (exact same pattern)
  - REFERENCE: src/commands/add.rs:236-261

Task 10: ADD src/commands/reset.rs to src/commands/mod.rs
  - ADD: pub mod reset;
  - EXPORT: pub use reset::execute as reset_execute;
  - PATTERN: Follow existing mod.rs structure (after commit import)

Task 11: UPDATE command dispatcher in src/main.rs
  - MODIFY: Commands::Reset(cmd) branch (line 145-149)
  - CHANGE: From placeholder to:
    match commands::reset_execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
  - PATTERN: Follow other command dispatchers (Add, Commit, Status)

Task 12: CREATE comprehensive unit tests in src/commands/reset.rs tests module
  - USE: tempfile::TempDir for isolated test directories
  - USE: DirGuard pattern for directory restoration
  - TEST: test_reset_soft_removes_from_staging() - Soft reset removes entries
  - TEST: test_reset_mixed_removes_from_staging() - Mixed reset removes entries
  - TEST: test_reset_hard_removes_files() - Hard reset discards workspace files
  - TEST: test_reset_with_paths() - Path-specific reset works
  - TEST: test_reset_all_paths_when_empty() - Empty paths resets all
  - TEST: test_reset_layer_targeting() - Layer flags work correctly
  - TEST: test_reset_no_staged_files_error() - Error when nothing staged
  - TEST: test_determine_reset_layer_with_flags() - Explicit flag routing
  - TEST: test_determine_reset_layer_with_context() - Context-based routing
  - PATTERN: Follow src/commands/add.rs test structure

Task 13: RUN validation and fix any issues
  - LINT: cargo clippy --all-targets --all-features
  - FMT: cargo fmt
  - TEST: cargo test --lib reset
  - BUILD: cargo build --release
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Command execute() structure
pub fn execute(cmd: &ResetCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Load project context
    let context = ProjectContext::load(&workspace_root)?;

    // 3. Detect project name
    let project_name = detect_project_name(&workspace_root)?;

    // 4. Validate Git repository
    let _git_repo = git2::Repository::discover(&workspace_root)
        .map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 6. Load staging index
    let mut staging = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());

    // 7. Determine target layer
    let target_layer = determine_reset_layer(cmd, &context, &project_name)?;

    // 8. Get paths to reset
    let paths_to_reset = get_paths_to_reset(cmd, &target_layer, &staging, &workspace_root)?;

    // 9. Validate there are files to reset
    if paths_to_reset.is_empty() {
        return Err(JinError::Message(
            "No staged files found in target layer. Use 'jin status' to see staged files.".to_string()
        ));
    }

    // 10. Determine reset mode
    let reset_mode = match (cmd.soft, cmd.mixed, cmd.hard) {
        (true, _, _) => ResetMode::Soft,
        (_, true, _) => ResetMode::Mixed,
        (_, _, true) => ResetMode::Hard,
        (false, false, false) => ResetMode::Mixed, // Default
    };

    // 11. Show preview
    println!("Resetting {} file(s) from layer: {}", paths_to_reset.len(), target_layer);
    match reset_mode {
        ResetMode::Soft => println!("Mode: soft (unstage only)"),
        ResetMode::Mixed => println!("Mode: mixed (unstage, keep workspace)"),
        ResetMode::Hard => println!("Mode: hard (unstage + discard workspace changes)"),
    }
    for path in &paths_to_reset {
        println!("  {}", path.display());
    }

    // 12. Execute reset
    let count = match reset_mode {
        ResetMode::Soft => execute_soft_reset(&paths_to_reset, &mut staging)?,
        ResetMode::Mixed => execute_mixed_reset(&paths_to_reset, &mut staging)?,
        ResetMode::Hard => execute_hard_reset(&paths_to_reset, &mut staging, &workspace_root)?,
    };

    // 13. Save staging index
    staging.save_to_disk(&workspace_root)?;

    // 14. Print success
    println!("\nReset complete. {} file(s) affected.", count);

    Ok(())
}

// Pattern 2: Layer determination (similar to add.rs)
fn determine_reset_layer(
    cmd: &ResetCommand,
    context: &ProjectContext,
    project: &str
) -> Result<Layer> {
    // If explicit flags, use them
    if cmd.mode || cmd.scope.is_some() || cmd.project {
        let mode = if cmd.mode {
            context.mode.as_deref()
        } else {
            None
        };
        let scope = cmd.scope.as_deref().or(context.scope.as_deref());
        let proj = if cmd.project { Some(project) } else { None };

        return Layer::from_flags(mode, scope, proj, false)
            .ok_or_else(|| JinError::Message(
                "No routing target specified. Use --mode, --scope, or --project".to_string()
            ));
    }

    // No explicit flags - use context or fall back to project base
    if let Some(mode) = &context.mode {
        if let Some(scope) = &context.scope {
            return Ok(Layer::ModeScope {
                mode: mode.clone(),
                scope: scope.clone(),
            });
        }
        return Ok(Layer::ModeBase { mode: mode.clone() });
    }

    if let Some(scope) = &context.scope {
        return Ok(Layer::ScopeBase {
            scope: scope.clone(),
        });
    }

    // Final fallback: project base layer
    Ok(Layer::ProjectBase {
        project: project.to_string(),
    })
}

// Pattern 3: Path normalization
fn normalize_path(path: &Path, workspace_root: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map(|p| p.to_path_buf())
            .map_err(|_| JinError::Message(format!(
                "File is outside workspace root: {}",
                path.display()
            )))
    } else {
        Ok(path.to_path_buf())
    }
}

// Pattern 4: Get paths to reset
fn get_paths_to_reset(
    cmd: &ResetCommand,
    target_layer: &Layer,
    staging: &StagingIndex,
    workspace_root: &Path
) -> Result<Vec<PathBuf>> {
    if cmd.paths.is_empty() {
        // Reset all files from target layer
        let entries = staging.entries_by_layer(target_layer);
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        Ok(entries.iter().map(|e| e.path.clone()).collect())
    } else {
        // Reset specific paths
        cmd.paths.iter()
            .map(|p| normalize_path(p, workspace_root))
            .collect::<Result<Vec<_>>>()
    }
}

// Pattern 5: Soft reset (unstage only)
fn execute_soft_reset(paths: &[PathBuf], staging: &mut StagingIndex) -> Result<usize> {
    let mut count = 0;
    for path in paths {
        if staging.remove_entry(path).is_some() {
            count += 1;
        }
    }
    Ok(count)
}

// Pattern 6: Mixed reset (same as soft for staging operations)
fn execute_mixed_reset(paths: &[PathBuf], staging: &mut StagingIndex) -> Result<usize> {
    // For now, mixed and soft are the same - both unstage
    // The difference would matter when we implement Git ref operations
    execute_soft_reset(paths, staging)
}

// Pattern 7: Hard reset (unstage + discard workspace)
fn execute_hard_reset(
    paths: &[PathBuf],
    staging: &mut StagingIndex,
    workspace_root: &Path
) -> Result<usize> {
    let mut count = 0;
    for path in paths {
        if staging.remove_entry(path).is_some() {
            count += 1;
        }

        // Remove from workspace if exists
        let workspace_file = workspace_root.join(path);
        if workspace_file.exists() {
            std::fs::remove_file(&workspace_file)
                .map_err(|e| JinError::Message(format!(
                    "Failed to remove {}: {}",
                    workspace_file.display(),
                    e
                )))?;
            println!("  Discarded: {}", path.display());
        }
    }
    Ok(count)
}

// Pattern 8: Project name detection (from add.rs)
fn detect_project_name(workspace_root: &Path) -> Result<String> {
    use git2::Repository;

    let repo = Repository::discover(workspace_root)
        .map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // Try git remote origin
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.rsplit('/').next() {
                let name = name.trim_end_matches(".git");
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }

    // Fallback to directory name
    workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| JinError::Message("Cannot determine project name".to_string()))
}

// Local enum for reset modes
enum ResetMode {
    Soft,
    Mixed,
    Hard,
}
```

### Integration Points

```yaml
COMMAND_MODULE:
  - modify: src/commands/mod.rs
  - add: pub mod reset;
  - add: pub use reset::execute as reset_execute;
  - pattern: Follow existing module exports (after commit)

MAIN_DISPATCHER:
  - modify: src/main.rs
  - update: Commands::Reset(cmd) branch (line 145-149)
  - pattern: match commands::reset_execute(&cmd) { Ok(()) => ..., Err(e) => ... }

STAGING_INDEX:
  - method: StagingIndex::load_from_disk(&Path)
  - method: StagingIndex::remove_entry(&Path) -> Option<StagedEntry>
  - method: StagingIndex::clear()
  - method: StagingIndex::save_to_disk(&Path)
  - method: StagingIndex::entries_by_layer(&Layer) -> Vec<&StagedEntry>

LAYER_ROUTING:
  - method: Layer::from_flags(mode, scope, project, global) -> Option<Layer>
  - use: Same routing as add.rs (without --global flag)

PROJECT_CONTEXT:
  - method: ProjectContext::load(&Path) -> Result<ProjectContext>
  - field: context.mode: Option<String>
  - field: context.scope: Option<String>

WORKSPACE_OPERATIONS:
  - use: std::fs::remove_file() for hard reset
  - use: std::fs::remove_dir() for empty directories (optional cleanup)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each significant code change
cargo fmt                          # Auto-format code
cargo clippy --all-targets         # Lint checking

# Expected: Zero warnings, zero errors
# Common clippy warnings to fix:
# - unused_variables
# - redundant_clone
# - unwrap_used (use proper error handling instead)
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test reset command specifically
cargo test --lib reset

# Test all command tests
cargo test --lib commands

# Test with output
cargo test --lib reset -- --nocapture

# Expected: All tests pass
# Key test cases:
# - test_reset_soft_removes_from_staging: Soft reset removes entries
# - test_reset_mixed_removes_from_staging: Mixed reset removes entries
# - test_reset_hard_removes_files: Hard reset discards workspace files
# - test_reset_with_paths: Path-specific reset works
# - test_reset_all_paths_when_empty: Empty paths resets all
# - test_reset_layer_targeting: Layer flags work correctly
# - test_reset_no_staged_files_error: Error when nothing staged
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the project
cargo build --release

# Manual integration test in a temporary directory
cd /tmp
rm -rf test_jin_reset
mkdir test_jin_reset && cd test_jin_reset
git init
jin init                     # Initialize Jin

# Create and stage a config file
echo "setting = true" > config.toml
jin add config.toml          # Stage to project base
jin status                   # Verify staged

# Test 1: Mixed reset (default)
jin reset                    # Should unstage but keep file
jin status                   # Verify no staged files
cat config.toml              # File should still exist

# Test 2: Stage again and test hard reset
jin add config.toml
jin reset --hard             # Should unstage AND delete file
jin status                   # Verify no staged files
ls config.toml               # File should NOT exist (error expected)

# Test 3: Create file and test layer targeting
echo "mode_setting = 1" > mode.conf
jin mode create claude
jin mode use claude
jin add mode.conf --mode
jin status                   # Verify staged in mode layer
jin reset --mode             # Reset from mode layer
jin status                   # Verify no staged files

# Test 4: Test path-specific reset
echo "a = 1" > a.txt
echo "b = 2" > b.txt
jin add a.txt b.txt
jin reset a.txt              # Reset only a.txt
jin status                   # Only b.txt should be staged

# Test 5: Test soft reset (same as mixed for now)
jin add b.txt
jin reset --soft
jin status                   # No staged files

# Expected: All manual tests pass, correct reset behavior
```

### Level 4: CLI & Domain-Specific Validation

```bash
# Test with mode and scope context
cd /tmp/reset_context_test
git init
jin init

# Set up context
jin mode create claude
jin mode use claude
jin scope create language:rust
jin scope use language:rust

# Create configs in different layers
echo "mode_setting = 1" > mode.conf
jin add mode.conf --mode
echo "scope_setting = 2" > scope.conf
jin add scope.conf --scope=language:rust
echo "project_setting = 3" > project.conf
jin add project.conf

# Verify all staged
jin status
# Expected: 3 files in different layers

# Test reset with active mode (should reset mode layer)
jin reset --mode
jin status
# Expected: mode.conf unstaged, others still staged

# Test reset with no flags (should use context: mode+scope)
jin reset
jin status
# Expected: scope.conf unstaged (context: mode+scope)

# Test reset to project base
jin reset --project
jin status
# Expected: project.conf unstaged

# Test hard reset with new file
echo "temporary = 1" > temp.conf
jin add temp.conf
jin reset --hard --project
ls temp.conf                 # Should error (file deleted)

# Test error case: reset with no staged files
jin reset                    # Should show error message
# Expected: "No staged files found in target layer"

# Expected: All scenarios work correctly
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo build --release`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check`
- [ ] All tests pass: `cargo test --lib`
- [ ] No unused imports or dead code

### Feature Validation

- [ ] `--soft` unstages files from staging index
- [ ] `--mixed` (default) unstages files but keeps workspace
- [ ] `--hard` unstages files AND discards workspace changes
- [ ] Layer targeting works with `--mode`, `--scope`, `--project` flags
- [ ] Active context used when no explicit flags provided
- [ ] Path-specific resets work (reset only specified files)
- [ ] Empty paths resets all files from target layer
- [ ] Error when no files staged in target layer
- [ ] Shows summary of affected files
- [ ] Staging index properly saved after reset

### Code Quality Validation

- [ ] Follows `src/commands/add.rs` patterns
- [ ] Proper error handling with `Result<>`
- [ ] Comprehensive doc comments on public functions
- [ ] Unit tests cover all major scenarios
- [ ] Tests use `tempfile` and `DirGuard` pattern
- [ ] No `unwrap()` calls (use proper error handling)
- [ ] No `expect()` calls (use proper error handling)

### Documentation & Deployment

- [ ] `execute()` has comprehensive doc comment
- [ ] Helper functions have doc comments
- [ ] User-facing error messages are clear
- [ ] Success output shows file count and layer

---

## Anti-Patterns to Avoid

- **Don't** skip the Git repository check - always validate we're in a Git repo
- **Don't** use `unwrap()` or `expect()` - propagate errors with `?`
- **Don't** forget to normalize paths - staging uses relative paths
- **Don't** forget to save staging index after modifications
- **Don't** assume paths exist - check before removing files (hard reset)
- **Don't** ignore empty paths - empty means "reset all files from layer"
- **Don't** bypass layer targeting - always determine target layer correctly
- **Don't** forget to update `src/commands/mod.rs` - add module and export
- **Don't** forget to update `src/main.rs` dispatcher - wire up execute function
- **Don't** show raw paths to user - always display as relative to workspace
- **Don't** implement Git ref operations yet - focus on staging only for this task
- **Don't** skip testing hard reset - file removal is critical behavior

## Confidence Score: 9/10

**Reasoning**:
- Complete codebase analysis with specific file references and line numbers
- All required infrastructure (StagingIndex, Layer, ProjectContext) exists and is well-understood
- Clear implementation pattern from existing commands (add, commit) to follow
- Specific helper functions and reset mode logic documented
- Unit test pattern established from other commands
- git2-rs research provides context for future hard reset with Git operations

**Remaining risks**:
1. **Path normalization**: Ensuring absolute vs relative path handling is correct
2. **Layer targeting complexity**: Context-based fallback logic needs thorough testing
3. **Hard reset safety**: File removal is destructive - needs careful testing

**Mitigation**: The PRP provides exact patterns for path normalization and layer determination, including the complete `determine_reset_layer()` function that handles all cases (explicit flags, context fallback, project base default). Testing strategy includes specific tests for all layer combinations.

## Appendix: External Research References

### git2-rs Reset API
- [Repository::reset method](https://docs.rs/git2/latest/git2/struct.Repository.html#method.reset)
- [ResetType enum](https://docs.rs/git2/latest/git2/enum.ResetType.html)
- Source: `/home/dustin/projects/jin-glm-doover/git2-rs-reset-research.md`

### CLI Reset UX Patterns
- Git reset semantics (--soft, --mixed, --hard)
- User confirmation for destructive operations
- Information display before/during/after reset
- Source: Research agent output on CLI reset patterns

### Jin-Specific Patterns
- Layer routing: `src/commands/add.rs:determine_layer()`
- Staging operations: `src/staging/index.rs`
- Project context: `src/core/config.rs:ProjectContext`

### Implementation Order
1. Module structure and imports
2. Helper functions (layer determination, path normalization)
3. Reset mode functions (soft, mixed, hard)
4. Main execute() function
5. Module exports and main.rs wiring
6. Comprehensive unit tests
7. Validation and testing
