# Product Requirement Prompt (PRP): Repair Command

**Task ID**: P4.M5.T6
**Task Title**: Repair Command
**Task Description**: Repair Jin state.

---

## Goal

**Feature Goal**: Implement `jin repair` command that detects and repairs corrupted Jin state including orphaned transactions, invalid staging indices, missing layer references, and workspace inconsistencies.

**Deliverable**: A fully functional CLI command (`jin repair`) that:
- Detects common Jin state corruption scenarios
- Provides dry-run mode for safe preview
- Repairs orphaned transactions
- Regenerates missing/corrupted metadata files
- Validates and fixes layer references
- Re-applies layers to workspace when needed

**Success Definition**:
- Command executes without errors in clean and corrupted states
- Dry-run mode accurately reports what would be repaired
- Actual repairs restore Jin to consistent state
- All validation levels pass (syntax, tests, integration)
- Comprehensive test coverage for repair scenarios

## User Persona

**Target User**: Developers using Jin who encounter system corruption or inconsistent state.

**Use Case**: When Jin's internal state becomes corrupted due to:
- Interrupted operations (crashes, forced termination)
- Git repository manipulation outside Jin
- Manual edits to Jin metadata files
- Disk/filesystem issues

**User Journey**:
1. User notices Jin behaving unexpectedly (e.g., `jin status` shows errors)
2. User runs `jin repair --dry-run` to see what would be fixed
3. User reviews the proposed repairs
4. User runs `jin repair` to apply fixes
5. User verifies Jin is working correctly again

**Pain Points Addressed**:
- No built-in recovery mechanism for corrupted state
- Manual repair requires deep knowledge of Jin internals
- Risk of data loss when attempting manual repairs
- No visibility into what's wrong before attempting fixes

## Why

**Business value and user impact**:
- Reduces support burden by providing self-service recovery
- Increases user confidence in Jin's reliability
- Prevents data loss from corruption scenarios
- Maintains system integrity over long-term use

**Integration with existing features**:
- Builds on TransactionManager's recovery functions (P1.M3.T3)
- Uses ProjectContext validation (P4.M3)
- Integrates with StagingIndex management (P3.M1)
- Supports layer system from P2.M3

**Problems this solves**:
- Orphaned transaction refs blocking operations
- Corrupted .jin/context causing command failures
- Missing .jinmap preventing layer operations
- Workspace state mismatched with layer content
- .gitignore managed block corruption

## What

**User-visible behavior**:
```bash
# Dry-run mode - shows what would be repaired
$ jin repair --dry-run
=== DRY RUN - Jin State Repair Assessment ===
Checking Jin state integrity...

[?] Found 2 orphaned transaction refs:
    - refs/jin/staging/uuid-1
    - refs/jin/staging/uuid-2
    [Would clean up]

[?] Staging index has 3 entries for missing files:
    - config.toml (not found on disk)
    - settings.yaml (not found on disk)
    - .env.local (not found on disk)
    [Would remove from staging]

[OK] .jinmap is valid

[?] .gitignore managed block corrupted
    [Would repair]

=== DRY RUN COMPLETE ===
Run 'jin repair' to apply repairs.

# Actual repair
$ jin repair
=== REPAIRING JIN STATE ===
Checking Jin state integrity...

[*] Cleaned up 2 orphaned transaction refs
[*] Removed 3 stale staging entries
[*] .jinmap is valid (no action needed)
[*] Repaired .gitignore managed block

=== REPAIR COMPLETE ===
Jin state is now healthy.
```

**Technical requirements**:
1. **Transaction Recovery**: Clean orphaned refs under `refs/jin/staging/*`
2. **Staging Cleanup**: Remove entries for files that no longer exist
3. **Jinmap Validation**: Check/regenerate `.jinmap` from Git history
4. **Context Validation**: Validate/reset `.jin/context` file
5. **Layer Ref Validation**: Ensure all layer refs point to valid commits
6. **Gitignore Repair**: Fix .gitignore managed block
7. **Workspace Re-apply**: Optionally re-apply layers to workspace

### Success Criteria

- [ ] `jin repair --dry-run` completes without modifying state
- [ ] Orphaned transaction refs are detected and cleaned
- [ ] Staging entries for missing files are removed
- [ ] Jinmap corruption is detected and repaired
- [ ] Context file corruption is detected and reset
- [ ] All repair operations report their actions clearly
- [ ] Command succeeds even when Jin is partially corrupted
- [ ] All unit tests pass with 80%+ coverage
- [ ] Integration tests verify repair in corrupted state

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed?

**Answer**: YES - This PRP provides:
- Complete file structure and patterns to follow
- Specific code examples from similar commands
- All dependency locations and signatures
- Test patterns and utilities
- Validation commands specific to this codebase
- External references for repair patterns

### Documentation & References

```yaml
# CRITICAL INTERNAL CODEBASE REFERENCES

# Command implementation pattern - MUST FOLLOW EXACTLY
- file: src/commands/reset.rs
  why: Closest command to repair - shows full command structure, layer targeting, staging manipulation
  pattern: 14-step execution flow: load context, validate Git repo, open Jin repo, load staging, execute operation, save staging
  gotcha: Uses unwrap_or_else for non-critical staging load - always save staging if modified

# CLI arguments - ALREADY DEFINED
- file: src/cli/args.rs
  line: 350-356
  why: RepairCommand struct with dry_run flag is already defined
  pattern: Single --dry-run flag, no additional arguments needed
  gotcha: Commands enum variant at line 105-106 already exists

# Command dispatch - NEEDS UPDATE
- file: src/main.rs
  line: 215-218
  why: Current placeholder needs actual command wiring
  pattern: Commands::Repair(cmd) => commands::repair_execute(&cmd)
  gotcha: Must import repair_execute from commands module

# Transaction recovery - CORE DEPENDENCY
- file: src/git/transaction.rs
  line: 570-604
  why: TransactionManager::detect_orphaned() and recover_all() methods
  pattern: Returns Vec<String> of transaction IDs, recover returns Result<()>
  gotcha: Uses RAII pattern - cleanup is automatic on Drop

# Staging index manipulation - CORE DEPENDENCY
- file: src/staging/index.rs
  why: StagingIndex::load_from_disk(), save_to_disk(), remove_entry(), entries() iterator
  pattern: IndexMap<PathBuf, StagedEntry> for entries, HashMap<Layer, Vec<PathBuf>> for by_layer
  gotcha: by_layer is reconstructed from entries during deserialization (#[serde(skip)])

# Project context validation
- file: src/core/config.rs
  why: ProjectContext::load() and save() for context validation
  pattern: Returns Result<ProjectContext>, validates YAML format
  gotcha: Context may be corrupted - handle parse errors gracefully

# Layer system understanding
- file: src/core/layer.rs
  why: Understanding 9-layer hierarchy for repair scope
  pattern: Layer enum with storage_path() and git_ref() methods
  gotcha: Not all layers have git_ref() - WorkspaceActive and UserLocal return None

# Error types to handle
- file: src/core/error.rs
  why: JinError variants for specific repair scenarios
  pattern: RepoNotFound, InvalidGitState, ConfigError, ValidationError, Io errors
  gotcha: Use JinError::Message for custom user-facing error messages

# Jinmap management (if exists)
- glob: src/**/*jinmap*.rs
  why: Understanding .jinmap structure for regeneration
  pattern: Load/save pattern, validation functions

# Git repository wrapper
- file: src/git/repo.rs
  why: JinRepo wrapper for Git operations
  pattern: open_or_create(), reference management, tree walking

# Test patterns
- file: src/commands/reset.rs
  section: tests module at end of file
  why: Complete test pattern for command testing
  pattern: tempfile::TempDir, DirGuard, init_git_repo, init_jin helpers, TEST_LOCK

# EXTERNAL REFERENCES FOR REPAIR PATTERNS

- url: https://git-scm.com/docs/git-fsck
  why: Understanding Git's fsck (file system check) for repair patterns
  critical: Check for unreachable objects, dangling blobs, corrupted references
  pattern: Diagnostic first, then repair options

- url: https://git-scm.com/docs/git-gc
  why: Garbage collection patterns for cleaning orphaned refs
  critical: Prune unreferenced objects, clean up reflogs
  pattern: Safe cleanup with --auto flag

- url: https://svnbook.subversion.org/1.7/svn.ref.svnadmin.c.verify.html
  why: Subversion's verify and recover patterns
  critical: Transaction journal processing, recovery from interruption
  pattern: Separate verify and recover commands

- url: https://www.mercurial-scm.org/doc/hg.1.html#verify
  why: Mercurial's verify command for integrity checking
  critical: Cross-link validation, repository integrity verification
  pattern: Detailed output of what's being checked
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── mod.rs              # CLI module exports
│   └── args.rs             # ALL command args (line 350-356: RepairCommand)
├── commands/
│   ├── mod.rs              # Command exports (NEEDS: repair_execute)
│   ├── init.rs             # Init command pattern (simple)
│   ├── add.rs              # Add command pattern (file operations)
│   ├── commit.rs           # Commit command pattern
│   ├── status.rs           # Status command pattern (state display)
│   ├── reset.rs            # Reset command (PATTERN TO FOLLOW - complex)
│   ├── diff.rs             # Diff command
│   ├── log.rs              # Log command
│   ├── context.rs          # Context command
│   ├── import.rs           # Import command
│   ├── export.rs           # Export command
│   └── repair.rs           # <<< TO BE CREATED
├── core/
│   ├── mod.rs              # Core exports (Layer, etc.)
│   ├── error.rs            # JinError enum
│   ├── layer.rs            # Layer enum (9-layer hierarchy)
│   └── config.rs           # ProjectContext, JinConfig
├── git/
│   ├── mod.rs
│   ├── repo.rs             # JinRepo wrapper
│   └── transaction.rs      # TransactionManager (line 594: recover_all)
├── staging/
│   ├── mod.rs
│   ├── entry.rs            # StagedEntry
│   └── index.rs            # StagingIndex (load/save/remove)
├── commit/
│   └── validate.rs         # Validation patterns
├── merge/
│   └── layer.rs            # Layer merge patterns
├── lib.rs                  # Library exports
└── main.rs                 # Command dispatch (line 215-218: needs update)
```

### Desired Codebase Tree (After Implementation)

```bash
src/
├── commands/
│   └── repair.rs           # NEW: Repair command implementation
│                           #   - execute() function
│                           #   - check_*() diagnostic functions
│                           #   - repair_*() actual repair functions
│                           #   - tests module
```

### Known Gotchas of This Codebase

```rust
// CRITICAL: StagingIndex load may fail if corrupted
// Pattern: Use unwrap_or_else to create empty index if load fails
let mut staging = StagingIndex::load_from_disk(&workspace_root)
    .unwrap_or_else(|_| StagingIndex::new());

// CRITICAL: Always save staging if modified
// Pattern: Save staging index at end of execute()
staging.save_to_disk(&workspace_root)?;

// CRITICAL: Commands must return Result<()>
// Pattern: Use ? operator for error propagation
pub fn execute(cmd: &RepairCommand) -> Result<()>

// CRITICAL: Use JinError::Message for user-facing errors
// Pattern: Return descriptive error messages
return Err(JinError::Message(
    "No staged files found in target layer. Use 'jin status' to see staged files.".to_string()
));

// CRITICAL: Git repo validation requires path in error
// Pattern: Include workspace_root in RepoNotFound error
let _git_repo = git2::Repository::discover(&workspace_root)
    .map_err(|_| JinError::RepoNotFound {
        path: workspace_root.display().to_string()
    })?;

// CRITICAL: Tests use TEST_LOCK for parallel safety
static TEST_LOCK: Mutex<()> = Mutex::new(());

// CRITICAL: Tests use DirGuard for directory restoration
struct DirGuard {
    original_dir: PathBuf,
}

// CRITICAL: by_layer HashMap is #[serde(skip)] - reconstructed from entries
// Pattern: Don't rely on by_layer being valid immediately after deserialization

// CRITICAL: Not all layers have git_ref() - WorkspaceActive and UserLocal return None
// Pattern: Check git_ref().is_some() before accessing

// CRITICAL: TransactionManager uses refs/jin/staging/* pattern
// Pattern: Orphan detection looks for refs under this path
```

## Implementation Blueprint

### Data Models and Structures

No new data models needed - Repair command uses existing types:

```rust
// Existing types used:
use crate::cli::args::RepairCommand;  // Already defined
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use crate::staging::index::StagingIndex;
use std::path::{Path, PathBuf};
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: CREATE src/commands/repair.rs
  - IMPLEMENT: Module header with documentation explaining repair purpose
  - FOLLOW pattern: src/commands/reset.rs (module documentation style)
  - CONTENT: Full module with execute(), diagnostics, repairs, and tests
  - PLACEMENT: src/commands/repair.rs

Task 2: IMPLEMENT execute() function signature
  - FUNCTION: pub fn execute(cmd: &RepairCommand) -> Result<()>
  - FOLLOW pattern: src/commands/reset.rs execute() (lines 1-50)
  - NAMING: execute as entry point matching other commands
  - RETURNS: Result<()> for error handling
  - PLACEMENT: Top of repair.rs module

Task 3: IMPLEMENT diagnostic functions (check_*)

  3.1: IMPLEMENT check_orphan_transactions()
    - FUNCTION: fn check_orphan_transactions(workspace_root: &Path) -> Result<Vec<String>>
    - DEPENDENCY: git2::Repository for ref iteration
    - PATTERN: Iterate refs/jin/staging/*, collect orphan IDs
    - RETURNS: List of orphan transaction IDs
    - GOTCHA: Use references_glob("refs/jin/staging/*") to find all staging refs

  3.2: IMPLEMENT check_staging_integrity()
    - FUNCTION: fn check_staging_integrity(workspace_root: &Path, staging: &StagingIndex) -> Result<Vec<PathBuf>>
    - DEPENDENCY: StagingIndex from Task 2, std::fs for file existence checks
    - PATTERN: Iterate staging.entries(), check file.exists()
    - RETURNS: List of paths that are staged but missing on disk
    - GOTCHA: PathBuf from staging needs workspace_root prefix for existence check

  3.3: IMPLEMENT check_jinmap()
    - FUNCTION: fn check_jinmap(workspace_root: &Path) -> Result<JinmapStatus>
    - DEPENDENCY: Check .jinmap file existence and validity
    - PATTERN: enum JinmapStatus { Valid, Missing, Corrupted }
    - RETURNS: Status indicating if jinmap needs repair
    - GOTCHA: May need to create JinmapStatus enum if not exists

  3.4: IMPLEMENT check_gitignore()
    - FUNCTION: fn check_gitignore(workspace_root: &Path) -> Result<GitignoreStatus>
    - DEPENDENCY: Read .gitignore, check for JIN MANAGED markers
    - PATTERN: enum GitignoreStatus { Valid, Missing, Corrupted }
    - RETURNS: Status indicating if gitignore needs repair
    - GOTCHA: Managed block markers are "### JIN MANAGED START" and "### JIN MANAGED END"

Task 4: IMPLEMENT repair functions (repair_*)

  4.1: IMPLEMENT repair_orphan_transactions()
    - FUNCTION: fn repair_orphan_transactions(workspace_root: &Path, orphans: &[String]) -> Result<usize>
    - DEPENDENCY: git2::Repository for ref deletion
    - PATTERN: Call TransactionManager::recover_all() or directly delete refs
    - RETURNS: Count of refs cleaned
    - GOTCHA: Must handle ref.delete() errors gracefully

  4.2: IMPLEMENT repair_staging_integrity()
    - FUNCTION: fn repair_staging_integrity(workspace_root: &Path, staging: &mut StagingIndex, missing: &[PathBuf]) -> Result<usize>
    - DEPENDENCY: StagingIndex remove_entry()
    - PATTERN: Remove entries for missing files, save staging
    - RETURNS: Count of entries removed
    - GOTCHA: Call staging.save_to_disk() after modifications

  4.3: IMPLEMENT repair_jinmap()
    - FUNCTION: fn repair_jinmap(workspace_root: &Path) -> Result<()>
    - DEPENDENCY: Jinmap regeneration from Git history
    - PATTERN: If missing/corrupted, regenerate from layer refs
    - RETURNS: Ok(()) on success
    - GOTCHA: May need to traverse Git history to rebuild

  4.4: IMPLEMENT repair_gitignore()
    - FUNCTION: fn repair_gitignore(workspace_root: &Path) -> Result<()>
    - DEPENDENCY: std::fs for file read/write
    - PATTERN: Remove old managed block, add new managed block
    - RETURNS: Ok(()) on success
    - GOTCHA: Preserve user entries outside managed block

Task 5: IMPLEMENT execute() main logic
  - INTEGRATE: All diagnostic and repair functions
  - PATTERN: If dry_run, run all check_* and report; else run repair_*
  - FLOW: Load context -> Load staging -> Run checks -> (dry run: report / actual: repair) -> Save staging
  - GOTCHA: Only save staging if actual repairs were done
  - PLACEMENT: Main execute() body

Task 6: MODIFY src/commands/mod.rs
  - ADD: pub mod repair;
  - PATTERN: Follow existing mod declarations (line ~30-40)
  - PLACEMENT: In command module list, alphabetically
  - GOTCHA: Ensure repair_execute is exported

Task 7: MODIFY src/main.rs
  - UPDATE: Commands::Repair dispatch (line 215-218)
  - REPLACE: Placeholder with actual command execution
  - PATTERN: commands::repair_execute(&cmd) with error handling
  - PLACEMENT: In match arms for Commands enum
  - GOTCHA: Import repair_execute at top of file

Task 8: CREATE comprehensive tests
  - IMPLEMENT: #[cfg(test)] mod tests at end of repair.rs
  - FOLLOW pattern: src/commands/reset.rs tests module
  - TESTS:
    - test_repair_dry_run_no_issues
    - test_repair_dry_run_with_orphans
    - test_repair_dry_run_with_missing_staged_files
    - test_repair_actual_cleanup
    - test_repair_with_corrupted_context
    - test_repair_with_missing_jinmap
  - NAMING: test_<function>_<scenario>
  - COVERAGE: All public functions with positive and negative cases
  - PLACEMENT: End of repair.rs file
```

### Implementation Patterns & Key Details

```rust
// ===== MAIN EXECUTION PATTERN (Follow reset.rs structure) =====

/// Executes the repair command.
///
/// This command diagnoses and repairs common issues with Jin state:
/// - Orphaned transaction refs
/// - Staging entries for missing files
/// - Corrupted or missing .jinmap
/// - Corrupted .gitignore managed block
///
/// # Arguments
///
/// * `cmd` - The repair command arguments
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err` if repair fails
///
/// # Examples
///
/// ```ignore
/// let cmd = RepairCommand { dry_run: false };
/// repair::execute(&cmd)?;
/// ```
pub fn execute(cmd: &RepairCommand) -> Result<()> {
    // STEP 1: Get workspace root (follow reset.rs pattern)
    let workspace_root = std::env::current_dir()?;

    // STEP 2: Load project context (may fail if corrupted)
    let context_result = ProjectContext::load(&workspace_root);

    // STEP 3: Validate Git repository (required for Jin)
    let git_repo = git2::Repository::discover(&workspace_root)
        .map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // STEP 4: Open Jin repository
    let _repo = JinRepo::open_or_create(&workspace_root)?;

    // STEP 5: Load staging index (may be corrupted)
    let mut staging = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());

    // STEP 6: Run diagnostics
    println!("=== Jin State Repair Assessment ===\n");

    let orphans = check_orphan_transactions(&workspace_root)?;
    let missing_files = check_staging_integrity(&workspace_root, &staging)?;
    let jinmap_status = check_jinmap(&workspace_root)?;
    let gitignore_status = check_gitignore(&workspace_root)?;

    // STEP 7: Dry run or actual repair
    if cmd.dry_run {
        execute_dry_run(&orphans, &missing_files, &jinmap_status, &gitignore_status)?;
    } else {
        execute_actual_repair(
            &workspace_root,
            &mut staging,
            &orphans,
            &missing_files,
            &jinmap_status,
            &gitignore_status,
        )?;
    }

    println!("=== {} ===", if cmd.dry_run { "DRY RUN COMPLETE" } else { "REPAIR COMPLETE" });

    Ok(())
}

// ===== DIAGNOSTIC FUNCTIONS =====

/// Status enum for jinmap check
#[derive(Debug, PartialEq)]
enum JinmapStatus {
    Valid,
    Missing,
    Corrupted,
}

/// Status enum for gitignore check
#[derive(Debug, PartialEq)]
enum GitignoreStatus {
    Valid,
    Missing,
    Corrupted,
}

/// Checks for orphaned transaction refs.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// Vector of orphan transaction IDs
fn check_orphan_transactions(workspace_root: &Path) -> Result<Vec<String>> {
    let repo = git2::Repository::discover(workspace_root)?;
    let mut orphans = Vec::new();

    // Pattern: Use references_glob to find staging refs
    for reference in repo.references_glob("refs/jin/staging/*")? {
        let reference = reference?;
        if let Some(name) = reference.name() {
            // Extract transaction ID from ref name
            if let Some(tx_id) = name.strip_prefix("refs/jin/staging/") {
                orphans.push(tx_id.to_string());
            }
        }
    }

    Ok(orphans)
}

/// Checks staging index for files that no longer exist.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
/// * `staging` - The staging index to check
///
/// # Returns
///
/// Vector of paths that are staged but missing on disk
fn check_staging_integrity(
    workspace_root: &Path,
    staging: &StagingIndex,
) -> Result<Vec<PathBuf>> {
    let mut missing = Vec::new();

    // PATTERN: Iterate entries and check file existence
    for (path, _entry) in staging.entries() {
        let full_path = workspace_root.join(path);
        if !full_path.exists() {
            missing.push(path.clone());
        }
    }

    Ok(missing)
}

/// Checks .jinmap file status.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// JinmapStatus indicating file state
fn check_jinmap(workspace_root: &Path) -> Result<JinmapStatus> {
    let jinmap_path = workspace_root.join(".jinmap");

    if !jinmap_path.exists() {
        return Ok(JinmapStatus::Missing);
    }

    // GOTCHA: Simple validity check - file exists and is readable
    // More sophisticated checks could verify JSON/YAML structure
    match std::fs::read_to_string(&jinmap_path) {
        Ok(_) => Ok(JinmapStatus::Valid),
        Err(_) => Ok(JinmapStatus::Corrupted),
    }
}

/// Checks .gitignore managed block status.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// GitignoreStatus indicating managed block state
fn check_gitignore(workspace_root: &Path) -> Result<GitignoreStatus> {
    let gitignore_path = workspace_root.join(".gitignore");

    if !gitignore_path.exists() {
        return Ok(GitignoreStatus::Missing);
    }

    let content = std::fs::read_to_string(&gitignore_path)?;

    // PATTERN: Check for managed block markers
    let has_start = content.contains("### JIN MANAGED START");
    let has_end = content.contains("### JIN MANAGED END");

    match (has_start, has_end) {
        (true, true) => Ok(GitignoreStatus::Valid),
        (false, false) => Ok(GitignoreStatus::Missing), // No managed block
        _ => Ok(GitignoreStatus::Corrupted), // Partial markers
    }
}

// ===== REPAIR FUNCTIONS =====

/// Executes dry run - shows what would be repaired without making changes.
fn execute_dry_run(
    orphans: &[String],
    missing_files: &[PathBuf],
    jinmap_status: &JinmapStatus,
    gitignore_status: &GitignoreStatus,
) -> Result<()> {
    println!("Running in DRY RUN mode - no changes will be made\n");

    // Report orphans
    if orphans.is_empty() {
        println!("[OK] No orphaned transaction refs found");
    } else {
        println!("[?] Found {} orphaned transaction ref(s):", orphans.len());
        for orphan in orphans {
            println!("    - refs/jin/staging/{}", orphan);
        }
        println!("    [Would clean up]\n");
    }

    // Report missing staged files
    if missing_files.is_empty() {
        println!("[OK] All staged files exist on disk");
    } else {
        println!("[?] Staging has {} entries for missing files:", missing_files.len());
        for file in missing_files {
            println!("    - {}", file.display());
        }
        println!("    [Would remove from staging]\n");
    }

    // Report jinmap status
    match jinmap_status {
        JinmapStatus::Valid => println!("[OK] .jinmap is valid\n"),
        JinmapStatus::Missing => println!("[?] .jinmap is missing\n    [Would regenerate]\n"),
        JinmapStatus::Corrupted => println!("[?] .jinmap is corrupted\n    [Would regenerate]\n"),
    }

    // Report gitignore status
    match gitignore_status {
        GitignoreStatus::Valid => println!("[OK] .gitignore managed block is valid"),
        GitignoreStatus::Missing => println!("[?] .gitignore has no managed block\n    [Would add]"),
        GitignoreStatus::Corrupted => println!("[?] .gitignore managed block is corrupted\n    [Would repair]"),
    }

    println!("\nRun 'jin repair' without --dry-run to apply repairs.");
    Ok(())
}

/// Executes actual repairs.
fn execute_actual_repair(
    workspace_root: &Path,
    staging: &mut StagingIndex,
    orphans: &[String],
    missing_files: &[PathBuf],
    jinmap_status: &JinmapStatus,
    gitignore_status: &GitignoreStatus,
) -> Result<()> {
    let mut total_repaired = 0;

    // Repair orphaned transactions
    if !orphans.is_empty() {
        let count = repair_orphan_transactions(workspace_root, orphans)?;
        println!("[*] Cleaned up {} orphaned transaction ref(s)", count);
        total_repaired += count;
    }

    // Repair staging integrity
    if !missing_files.is_empty() {
        let count = repair_staging_integrity(workspace_root, staging, missing_files)?;
        println!("[*] Removed {} stale staging entr(ies)", count);
        total_repaired += count;
    }

    // Repair jinmap
    match jinmap_status {
        JinmapStatus::Valid => {
            println!("[*] .jinmap is valid (no action needed)");
        }
        JinmapStatus::Missing | JinmapStatus::Corrupted => {
            repair_jinmap(workspace_root)?;
            println!("[*] Regenerated .jinmap");
            total_repaired += 1;
        }
    }

    // Repair gitignore
    match gitignore_status {
        GitignoreStatus::Valid => {
            println!("[*] .gitignore managed block is valid (no action needed)");
        }
        GitignoreStatus::Missing | GitignoreStatus::Corrupted => {
            repair_gitignore(workspace_root)?;
            println!("[*] Repaired .gitignore managed block");
            total_repaired += 1;
        }
    }

    // GOTCHA: Save staging after modifications
    if !missing_files.is_empty() {
        staging.save_to_disk(workspace_root)?;
    }

    if total_repaired == 0 {
        println!("\nJin state is already healthy - no repairs needed.");
    } else {
        println!("\nRepaired {} issue(s). Jin state is now healthy.", total_repaired);
    }

    Ok(())
}

/// Repairs orphaned transaction refs by deleting them.
fn repair_orphan_transactions(
    workspace_root: &Path,
    orphans: &[String],
) -> Result<usize> {
    let repo = git2::Repository::discover(workspace_root)?;
    let mut cleaned = 0;

    for tx_id in orphans {
        let ref_name = format!("refs/jin/staging/{}", tx_id);
        if let Ok(mut reference) = repo.find_reference(&ref_name) {
            if reference.delete().is_ok() {
                cleaned += 1;
            }
        }
    }

    Ok(cleaned)
}

/// Repairs staging integrity by removing entries for missing files.
fn repair_staging_integrity(
    _workspace_root: &Path,
    staging: &mut StagingIndex,
    missing_files: &[PathBuf],
) -> Result<usize> {
    let mut removed = 0;

    for path in missing_files {
        staging.remove_entry(path);
        removed += 1;
    }

    Ok(removed)
}

/// Repairs .jinmap by regenerating it.
fn repair_jinmap(workspace_root: &Path) -> Result<()> {
    // GOTCHA: For now, create a minimal .jinmap
    // Full implementation would regenerate from Git history
    let jinmap_path = workspace_root.join(".jinmap");
    let content = r#"# Jin Map - Auto-generated by jin repair
# This file maps Jin layers to their Git references
# Run 'jin repair' to regenerate if needed
"#;

    std::fs::write(&jinmap_path, content)
        .map_err(|e| JinError::Io(e))?;

    Ok(())
}

/// Repairs .gitignore managed block.
fn repair_gitignore(workspace_root: &Path) -> Result<()> {
    let gitignore_path = workspace_root.join(".gitignore");

    // Read existing content or start fresh
    let existing_content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    // Remove old managed block if exists
    let content = if let Some(start_idx) = existing_content.find("### JIN MANAGED START") {
        let before = &existing_content[..start_idx];
        if let Some(end_idx) = existing_content.find("### JIN MANAGED END") {
            let after = &existing_content[end_idx + "### JIN MANAGED END".len()..];
            format!("{}\n{}\n{}", before.trim(), "### JIN MANAGED START\n.jin\n.jinmap\n### JIN MANAGED END", after.trim())
        } else {
            existing_content
        }
    } else {
        existing_content
    };

    // Add managed block if missing
    let final_content = if !content.contains("### JIN MANAGED START") {
        format!(
            "{}\n### JIN MANAGED START\n.jin\n.jinmap\n### JIN MANAGED END\n",
            if content.is_empty() { "" } else { &content }
        )
    } else {
        content
    };

    std::fs::write(&gitignore_path, final_content)
        .map_err(|e| JinError::Io(e))?;

    Ok(())
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // GOTCHA: Use TEST_LOCK for parallel test safety
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    struct DirGuard {
        original_dir: PathBuf,
    }

    impl DirGuard {
        fn new() -> std::io::Result<Self> {
            Ok(Self {
                original_dir: std::env::current_dir()?,
            })
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    fn init_git_repo(dir: &Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    fn init_jin(dir: &Path) {
        let workspace_dir = dir.join(".jin/workspace");
        fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_repair_dry_run_no_issues() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = RepairCommand { dry_run: true };
        let result = execute(&cmd);

        assert!(result.is_ok());
    }

    #[test]
    fn test_repair_dry_run_with_orphans() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create orphaned transaction ref
        let oid = repo.head().unwrap().target().unwrap();
        repo.reference("refs/jin/staging/test-orphan", oid, false, "test").unwrap();

        let cmd = RepairCommand { dry_run: true };
        let result = execute(&cmd);

        assert!(result.is_ok());
    }

    #[test]
    fn test_repair_actual_cleanup() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create orphaned transaction ref
        let oid = repo.head().unwrap().target().unwrap();
        repo.reference("refs/jin/staging/test-orphan", oid, false, "test").unwrap();

        let cmd = RepairCommand { dry_run: false };
        let result = execute(&cmd);

        assert!(result.is_ok());

        // Verify orphan was cleaned
        assert!(repo.find_reference("refs/jin/staging/test-orphan").is_err());
    }
}
```

### Integration Points

```yaml
TRANSACTION_SYSTEM:
  - dependency: "src/git/transaction.rs"
  - method: "TransactionManager::recover_all()"
  - pattern: "Detect and clean refs/jin/staging/* refs"

STAGING_SYSTEM:
  - dependency: "src/staging/index.rs"
  - method: "StagingIndex::load_from_disk(), save_to_disk(), remove_entry()"
  - pattern: "Remove entries for missing files, save after modification"

CONFIG_SYSTEM:
  - dependency: "src/core/config.rs"
  - method: "ProjectContext::load()"
  - pattern: "Handle corrupted context gracefully"

CLI_DISPATCH:
  - file: "src/main.rs"
  - line: "215-218"
  - pattern: "Replace placeholder with commands::repair_execute(&cmd)"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating src/commands/repair.rs - fix before proceeding
cargo check --bin jin                    # Check compilation
cargo clippy --bin jin                   # Lint checking
cargo fmt --check src/commands/repair.rs # Format check

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test repair command specifically
cargo test repair --lib                  # Run repair unit tests
cargo test repair --lib -- --nocapture   # With output

# Full test suite for commands
cargo test --lib                         # All library tests

# Coverage validation (if using cargo-tarpaulin)
cargo tarpaulin --out Html --output-dir coverage

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the CLI
cargo build --release

# Test repair in clean project
cd /tmp/test_clean && git init && jin init
jin repair --dry-run                     # Should report healthy
jin repair                               # Should report no repairs needed

# Test repair with orphaned transactions
cd /tmp/test_orphan && git init && jin init
# Manually create orphan ref:
git update-ref refs/jin/staging/test-orphan HEAD
jin repair --dry-run                     # Should detect orphan
jin repair                               # Should clean orphan

# Test repair with corrupted staging
cd /tmp/test_staging && git init && jin init
echo "test" > test.txt && jin add test.txt
rm test.txt                               # Remove staged file
jin repair --dry-run                     # Should detect missing file
jin repair                               # Should clean staging

# Test repair with missing .gitignore
cd /tmp/test_gitignore && git init && jin init
rm .gitignore
jin repair --dry-run                     # Should detect missing managed block
jin repair                               # Should add managed block

# Expected: All scenarios handled correctly, proper exit codes.
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test repair in various corruption scenarios

# Scenario 1: Multiple orphans
for i in {1..5}; do git update-ref "refs/jin/staging/orphan-$i" HEAD; done
jin repair --dry-run
jin repair
# Verify: All 5 orphans cleaned

# Scenario 2: Corrupted .jin/context
echo "invalid: yaml: {" > .jin/context
jin repair --dry-run
# Expected: Graceful error or context reset

# Scenario 3: Missing .jinmap
rm .jinmap
jin repair --dry-run
jin repair
# Expected: .jinmap regenerated

# Scenario 4: Partial .gitignore corruption
echo "### JIN MANAGED START" > .gitignore
# Missing END marker
jin repair --dry-run
jin repair
# Expected: Managed block completed

# Performance test: Repair with many orphans
for i in {1..100}; do git update-ref "refs/jin/staging/orphan-$i" HEAD; done
time jin repair
# Expected: Completes in reasonable time (<5s for 100 refs)

# Expected: All creative validations pass, recovery successful.
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --lib`
- [ ] No linting errors: `cargo clippy --bin jin`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Code compiles: `cargo build --release`

### Feature Validation

- [ ] `jin repair --dry-run` reports issues without modifying state
- [ ] Orphaned transaction refs are detected and cleaned
- [ ] Staging entries for missing files are removed
- [ ] .jinmap corruption is detected and regenerated
- [ ] .gitignore managed block corruption is detected and repaired
- [ ] Command succeeds when Jin is partially corrupted
- [ ] Clear output shows what was checked and what was repaired
- [ ] Exit code is 0 on success, non-zero on error

### Code Quality Validation

- [ ] Follows existing command patterns from reset.rs
- [ ] File placement matches desired codebase tree (src/commands/repair.rs)
- [ ] Uses existing types: JinError, Result, StagingIndex, ProjectContext
- [ ] Dependencies properly imported
- [ ] Error handling is specific (JinError variants) not generic
- [ ] Test coverage includes positive and negative cases
- [ ] Tests use proper isolation (TEST_LOCK, DirGuard, TempDir)

### Documentation & Deployment

- [ ] Module documentation explains repair purpose and scenarios
- [ ] Function documentation includes Examples section
- [ ] CLI help text is clear: `jin repair --help`
- [ ] Error messages are actionable
- [ ] Dry-run output is clear and comprehensive
- [ ] No new dependencies added (uses existing)

---

## Anti-Patterns to Avoid

- **Don't** modify Git history or user data - only repair Jin metadata
- **Don't** silently delete data - always report what's being repaired
- **Don't** skip dry-run validation - user should see what will happen
- **Don't** ignore errors during repair - report and continue or fail fast
- **Don't** create new patterns - follow reset.rs structure exactly
- **Don't** hardcode paths - use workspace_root from environment
- **Don't** assume files exist - check before accessing
- **Don't** forget to save staging after modifications
- **Don't** use sync operations where async is appropriate (not applicable here - CLI is sync)
- **Don't** catch all exceptions - be specific with JinError variants
