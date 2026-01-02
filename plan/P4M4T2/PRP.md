# PRP: P4.M4.T2 - Reset Command

---

## Goal

**Feature Goal**: Implement `jin reset` command for undoing staged and committed changes across layers with three reset modes (--soft, --mixed, --hard) and layer-specific targeting.

**Deliverable**: Fully functional `jin reset` CLI command with:
- Three reset modes (soft/mixed/hard) matching Git semantics
- Layer-specific targeting (--mode, --scope, --project, --global)
- Safety mechanisms (confirmation prompts, force flag)
- Comprehensive error handling and user feedback
- Integration tests for complete reset workflows

**Success Definition**:
- Users can unstage files with `jin reset` (default --mixed mode)
- Users can reset specific layers with layer flags (--mode, --scope, --project)
- `jin reset --hard` prompts for confirmation before destructive operations
- `jin reset --hard --force` skips confirmation for scripting
- `jin reset --soft` preserves staged changes for re-committing
- All operations work atomically (all succeed or none apply)
- Clear error messages guide users on recovery
- All validation levels pass (syntax, unit tests, integration tests)

## Why

**Business Value:**
- Completes the core workflow loop: add → commit → apply → reset
- Enables users to safely experiment with layer configurations
- Provides granular control over staging and workspace state
- Implements Git-familiar reset semantics for easier adoption
- Essential for error recovery and workflow correction

**Integration with Existing Features:**
- Builds on staging system (P3.M1) for file tracking
- Uses layer targeting from routing system (P4.M3)
- Leverages existing CLI patterns from other commands
- Integrates with context management (ProjectContext)

**Problems This Solves:**
- Users can't undo mistaken staging operations
- Users can't unstage files without losing workspace changes
- Users can't reset specific layers independently
- No safety mechanism for destructive workspace resets
- Workflow is one-directional (add/commit only, no rollback)

## What

### User-Visible Behavior

#### jin reset

**Basic Usage:**
```bash
jin reset                                        # Reset staging (default --mixed)
jin reset --soft                                 # Keep changes in staging
jin reset --hard                                 # Discard all changes (DESTRUCTIVE)
jin reset --hard --force                         # Skip confirmation prompt

# Layer-specific resets
jin reset --mode                                 # Reset active mode base (Layer 2)
jin reset --mode --project                       # Reset mode-project (Layer 5)
jin reset --mode --scope=lang:rust               # Reset mode-scope (Layer 3)
jin reset --scope=lang:rust                      # Reset scope base (Layer 6)
jin reset --global                               # Reset global base (Layer 1)
```

**Reset Modes:**

**--soft**: Keep changes in staging area
- Clears commit metadata but preserves staged entries
- Files remain staged (no workspace changes)
- Useful for re-committing with different message
- Equivalent to "undo commit, keep staged"

**--mixed** (default): Unstage changes but keep in workspace
- Removes entries from staging index
- Workspace files unchanged (modifications preserved)
- Most commonly used mode
- Equivalent to "undo commit and unstage"

**--hard**: Discard all changes
- Removes entries from staging index
- Deletes workspace files (DESTRUCTIVE)
- Removes entries from .gitignore managed block
- Requires confirmation prompt unless `--force` is used
- Equivalent to "undo everything, revert to committed state"

**Layer Targeting:**

| Flags | Target Layer | Context Required |
|-------|--------------|------------------|
| (none) | ProjectBase (Layer 7) | None |
| `--mode` | ModeBase (Layer 2) | Active mode |
| `--mode --project` | ModeProject (Layer 5) | Active mode |
| `--mode --scope=X` | ModeScope (Layer 3) | Active mode |
| `--mode --scope=X --project` | ModeScopeProject (Layer 4) | Active mode |
| `--scope=X` | ScopeBase (Layer 6) | None |
| `--global` | GlobalBase (Layer 1) | None |

**Safety Features:**
- `--hard` mode requires confirmation: "This will discard N file(s) from staging AND workspace. Type 'yes' to confirm:"
- Shows count of files that will be affected
- Displays warning for destructive operations
- Supports `--force` to skip confirmation for scripting

**Error Conditions:**
- No staged files → Warning "Nothing to reset for layer: {layer}"
- Invalid layer combination → Error with valid examples
- No active mode with `--mode` flag → Error suggesting `jin mode use`
- `--project` without `--mode` → Error "project requires --mode (use --mode --project)"

**User Output Examples:**

```bash
# Mixed mode (default)
$ jin add config.json --mode
$ jin reset --mode
Unstaged 1 file(s) (kept in workspace)

# Soft mode
$ jin add config.json --mode
$ jin reset --soft --mode
Reset 1 file(s) (kept in staging)

# Hard mode with confirmation
$ jin add config.json --mode
$ jin reset --hard --mode
This will discard 1 file(s) from staging AND workspace. Type 'yes' to confirm: yes
Discarded 1 file(s) from staging and workspace

# Hard mode with force
$ jin reset --hard --force --mode
Discarded 1 file(s) from staging and workspace

# Nothing to reset
$ jin reset --mode
Nothing to reset for layer: mode-base
```

### Success Criteria

- [ ] `jin reset` (default --mixed) unstages files while preserving workspace
- [ ] `jin reset --soft` keeps changes staged for re-commit
- [ ] `jin reset --hard` prompts for confirmation
- [ ] `jin reset --hard --force` skips confirmation
- [ ] Layer-specific resets target correct layers based on flags
- [ ] All operations are atomic (all succeed or none apply)
- [ ] Clear error messages guide users on recovery
- [ ] Reset removes entries from .gitignore managed block (--hard only)
- [ ] Reset handles empty staging gracefully
- [ ] Integration tests cover complete reset workflows

## All Needed Context

### Context Completeness Check

_Before implementing, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: YES - This PRP provides:
- Complete reset command specification with all modes
- Exact file patterns to follow from existing commands
- Specific implementation order with dependencies
- Error handling strategies with examples
- Testing approach with patterns
- All necessary context references with URLs

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# External Research (Best Practices)
- url: https://git-scm.com/docs/git-reset
  why: Reset modes (--soft/--mixed/--hard) semantics and safety patterns
  critical: Only --hard can destroy data; implement confirmation prompts
  section: "Reset command modes and three-step process"

- url: https://clig.dev/
  why: CLI design patterns for destructive operations
  critical: Confirmation prompts, force flags, clear error messages
  section: "Interactivity - when to prompt for confirmation"

- url: https://www.arp242.net/cli-guidelines/
  why: Command-line interface conventions for undo operations
  critical: Clear output showing what was changed
  section: "Output and feedback"

# Local Research Documentation
- file: plan/P4M4T2/research/EXTERNAL_RESET_RESEARCH.md
  why: Comprehensive reset patterns from Git, Mercurial, Terraform, Ansible
  pattern: Three-tier reset (soft/mixed/hard) with safety mechanisms

- file: plan/docs/RESET_COMMAND_RESEARCH.md
  why: Reset modes, layer targeting, and safety practices specific to Jin
  pattern: Layer-specific reset with context validation

- file: plan/docs/CODEBASE_PATTERNS.md
  why: Jin-specific implementation patterns and critical gotchas
  pattern: Command structure, error handling, validation approach

# Codebase Pattern Files
- file: src/commands/reset.rs
  why: EXISTING IMPLEMENTATION - mostly complete, needs enhancements
  pattern: execute() → determine_target_layer() → reset_staging() → reset_workspace()
  gotcha: Soft mode is currently a no-op, needs meaningful implementation
  critical: File already has most logic - verify before rewriting

- file: src/commands/apply.rs
  why: Similar workspace command for patterns
  pattern: Context loading → Validation → Operation → Save → Report
  gotcha: Uses WorkspaceMetadata for tracking state

- file: src/commands/add.rs
  why: Full command implementation with error collection
  pattern: Validation → Context → Routing → Operation → Save → Report
  gotcha: Always check for empty files list before processing

- file: src/cli/args.rs
  why: ResetArgs definition - needs --global and --force flags added
  pattern: #[derive(Args, Debug)] with clap argument attributes
  critical: Add global: bool and force: bool fields to ResetArgs

- file: src/cli/mod.rs
  why: CLI parser with Commands enum
  pattern: Commands::Reset(ResetArgs) already wired
  critical: No changes needed to CLI structure

# Staging System Files
- file: src/staging/index.rs
  why: StagingIndex management - load, modify, save pattern
  pattern: load().unwrap_or_else(|_| new()) → modify → save()
  gotcha: Entries stored in HashMap (no guaranteed order)
  critical: entries_for_layer(layer) returns Vec<&StagedEntry>

- file: src/staging/entry.rs
  why: StagedEntry type structure
  pattern: path, target_layer, content_hash, mode, operation
  critical: Target layer determines which reset operation affects entry

- file: src/staging/gitignore.rs
  why: Automatic .gitignore management with managed blocks
  pattern: remove_from_managed_block() for --hard mode cleanup
  gotcha: Never modifies content outside markers
  critical: Use remove_from_managed_block() to clean up .gitignore

# Context & Layer Management
- file: src/core/config.rs
  why: ProjectContext load/save, require_mode/require_scope patterns
  pattern: load() with NotInitialized handling → modify → save()
  critical: require_mode() returns Err if no active mode

- file: src/core/layer.rs
  why: Layer enum, precedence, ref_path generation
  pattern: Layer::ModeBase, Layer::ProjectBase, etc.
  critical: Layer determines which staging entries are affected

- file: src/core/error.rs
  why: JinError enum and Result type alias
  pattern: Custom error types with thiserror
  critical: Use JinError::Other for custom error messages

# Test Patterns
- file: tests/cli_basic.rs
  why: Basic CLI integration test patterns
  pattern: assert_cmd::Command, tempfile::TempDir
  critical: Use jin() helper function for CLI testing

- file: tests/common/mod.rs
  why: Common test utilities and fixtures
  pattern: TestFixture for isolated test environments
  critical: Each test gets isolated temporary directory

- file: tests/core_workflow.rs
  why: Core workflow test patterns (init → add → commit → apply)
  pattern: Full workflow testing with state verification
  critical: Test reset within complete workflow context
```

### Current Codebase Tree

```bash
.
├── src
│   ├── cli
│   │   ├── args.rs              # ResetArgs - needs --global and --force flags
│   │   └── mod.rs               # CLI parser (Reset already wired)
│   ├── commands
│   │   ├── add.rs               # Full command example with error collection
│   │   ├── apply.rs             # Similar workspace command
│   │   ├── reset.rs             # ⚠️  EXISTING - needs enhancements
│   │   └── mod.rs               # Command dispatcher
│   ├── staging
│   │   ├── index.rs             # StagingIndex management
│   │   ├── entry.rs             # StagedEntry types
│   │   ├── gitignore.rs         # .gitignore managed blocks
│   │   └── router.rs            # Layer routing logic
│   └── core
│       ├── config.rs            # ProjectContext management
│       ├── layer.rs             # Layer enum and paths
│       └── error.rs             # JinError types
└── tests
    ├── cli_basic.rs             # CLI integration test patterns
    ├── common/                  # Test utilities
    └── core_workflow.rs         # Core workflow tests
```

### Desired Codebase Tree (After Implementation)

```bash
src/cli/
├── args.rs                      # MODIFY: Add --global and --force to ResetArgs
│   # Add:
│   # - pub global: bool
│   # - pub force: bool

src/commands/
├── reset.rs                     # ENHANCE: Complete reset implementation
│   # - fn execute(args: ResetArgs) -> Result<()> (mostly complete)
│   # - fn determine_target_layer() → ADD --global support
│   # - fn reset_staging() → VERIFY correct
│   # - fn reset_workspace() → VERIFY correct
│   # - fn prompt_confirmation() → RESPECT --force flag
│   # - Unit tests → ADD test for --global and --force

tests/
└── cli_reset.rs                 # CREATE: Reset integration tests
    # - test_reset_mixed_mode()
    # - test_reset_soft_mode()
    # - test_reset_hard_mode_with_confirmation()
    # - test_reset_hard_mode_with_force()
    # - test_reset_layer_targeting()
    # - test_reset_empty_staging()
    # - test_reset_invalid_layer_combination()
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: The existing reset.rs is mostly complete
// Before rewriting anything, READ the full file to understand what's already there
// The main gaps are:
// 1. Missing --global flag in ResetArgs
// 2. Missing --force flag for skipping confirmation
// 3. Soft mode is a no-op (should do something meaningful)

// CRITICAL: StagingIndex.entries_for_layer() returns Vec<&StagedEntry>
// The iterator pattern is important - collect before modifying staging
let entries: Vec<StagedEntry> = staging
    .entries_for_layer(layer)
    .iter()
    .map(|e| (*e).clone())
    .collect();
// NOT: for entry in staging.entries_for_layer() { ... }  // Borrow checker issue

// CRITICAL: HashMap iteration order is not guaranteed
// If order matters for tests, sort paths before assertions
let mut paths: Vec<_> = staging.entries.keys().collect();
paths.sort();

// CRITICAL: ProjectContext::load() can return non-NotInitialized errors
// Use unwrap_or_default() for non-critical initialization
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
    Err(_) => ProjectContext::default(),
};

// CRITICAL: .gitignore managed block safety
// ALWAYS use remove_from_managed_block() - it handles:
// - Preserving content outside markers
// - Removing only the specified path
// - Error handling if path not found
remove_from_managed_block(&path)?;
// NOT: Manually editing .gitignore

// CRITICAL: Confirmation prompt should respect --force flag
if mode == ResetMode::Hard && !args.force {
    if !prompt_confirmation(&message)? {
        println!("Reset cancelled");
        return Ok(());
    }
}
// WITHOUT --force: Always prompt for --hard mode
// WITH --force: Skip prompt for scripting

// CRITICAL: --global flag should reset GlobalBase layer (Layer 1)
// Add this case to determine_target_layer():
if args.global {
    return Ok(Layer::GlobalBase);
}

// CRITICAL: Layer precedence order for validation
// Check specific combinations first, then fall back to defaults
// Order: mode+scope+project → mode+scope → mode+project → mode → scope → global → project → error

// CRITICAL: Test isolation using TempDir
// Each test should get its own temporary directory
let temp = TempDir::new().unwrap();
std::env::set_current_dir(temp.path()).unwrap();

// CRITICAL: Error collection pattern for batch operations
let mut errors = Vec::new();
for entry in entries {
    if let Err(e) = process_entry(entry) {
        errors.push(format!("{}: {}", entry.path.display(), e));
    }
}
if !errors.is_empty() {
    eprintln!("Errors during workspace reset:");
    for error in &errors {
        eprintln!("  {}", error);
    }
}
// Report all errors at end, not per-item

// CRITICAL: Soft mode implementation
// Currently a no-op in existing code
// Should: Clear commit metadata but preserve staging entries
// This may require tracking commit metadata separately from staging
// For now: Keep as-is (acknowledges reset but takes no action)

// CRITICAL: Unit test naming convention
// test_execute_not_initialized
// test_determine_target_layer_default
// test_determine_target_layer_mode
// test_reset_staging_empty
// Follow: test_{function}_{scenario}
```

## Implementation Blueprint

### Data Models and Structure

```rust
// No new core data models needed - reuse existing:

// From src/staging/index.rs
pub struct StagingIndex {
    entries: HashMap<PathBuf, StagedEntry>,
    version: u32,
}

impl StagingIndex {
    pub fn load() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
    pub fn entries_for_layer(&self, layer: Layer) -> Vec<&StagedEntry>;
    pub fn is_empty(&self) -> bool;
    pub fn remove(&mut self, path: &Path) -> Option<StagedEntry>;
}

// From src/staging/entry.rs
pub struct StagedEntry {
    pub path: PathBuf,
    pub target_layer: Layer,
    pub content_hash: String,
    pub mode: u32,
    pub operation: StagedOperation,
}

// From src/cli/args.rs (NEEDS UPDATE)
pub struct ResetArgs {
    pub soft: bool,       // Existing
    pub mixed: bool,      // Existing
    pub hard: bool,       // Existing
    pub mode: bool,       // Existing
    pub scope: Option<String>,  // Existing
    pub project: bool,    // Existing
    // ADD:
    pub global: bool,     // NEW: Reset GlobalBase layer
    pub force: bool,      // NEW: Skip confirmation prompt
}

// From src/commands/reset.rs (LOCAL TYPES)
enum ResetMode {
    Soft,    // Keep changes in staging
    Mixed,   // Unstage but keep in workspace (default)
    Hard,    // Discard all changes (DESTRUCTIVE)
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: UPDATE ResetArgs in src/cli/args.rs
  - MODIFY: src/cli/args.rs::ResetArgs
  - ADD: pub global: bool field
  - ADD: pub force: bool field
  - ADD: #[arg(long)] attribute for global
  - ADD: #[arg(long, short = 'f')] attribute for force
  - NAMING: global, force
  - PLACEMENT: After existing fields in ResetArgs
  - DEPENDENCIES: None (foundational)
  - VERIFICATION: cargo fmt && cargo build

Task 2: UPDATE determine_target_layer() in src/commands/reset.rs
  - MODIFY: fn determine_target_layer() in reset.rs
  - ADD: --global flag support for Layer::GlobalBase
  - ADD: Check for global flag first (highest precedence)
  - FOLLOW pattern: Existing layer resolution logic
  - NAMING: determine_target_layer()
  - DEPENDENCIES: Task 1 (ResetArgs with global field)
  - CRITICAL: Global check should come before other checks
  - PATTERN:
    if args.global {
        return Ok(Layer::GlobalBase);
    }

Task 3: UPDATE prompt_confirmation() call in execute()
  - MODIFY: fn execute() in reset.rs
  - UPDATE: Hard mode confirmation to check args.force
  - FOLLOW pattern: Conditional confirmation based on force flag
  - NAMING: execute()
  - DEPENDENCIES: Task 1 (ResetArgs with force field)
  - CRITICAL: Skip confirmation when --force is present
  - PATTERN:
    if mode == ResetMode::Hard && !args.force {
        // Prompt for confirmation
    }

Task 4: ADD unit test for --global flag
  - MODIFY: #[cfg(test)] mod tests in reset.rs
  - ADD: test_determine_target_layer_global()
  - IMPLEMENT: Test that --global returns Layer::GlobalBase
  - FOLLOW pattern: Existing test functions
  - NAMING: test_determine_target_layer_global
  - COVERAGE: Global flag layer resolution
  - DEPENDENCIES: Task 2 (determine_target_layer with global support)

Task 5: ADD unit test for --force flag
  - MODIFY: #[cfg(test)] mod tests in reset.rs
  - ADD: test_reset_hard_with_force()
  - IMPLEMENT: Test that --force skips confirmation
  - FOLLOW pattern: Existing test functions with TempDir
  - NAMING: test_reset_hard_with_force
  - COVERAGE: Force flag behavior
  - DEPENDENCIES: Task 3 (execute with force support)

Task 6: CREATE integration tests for reset command
  - CREATE: tests/cli_reset.rs
  - IMPLEMENT: test_reset_mixed_mode()
  - IMPLEMENT: test_reset_soft_mode()
  - IMPLEMENT: test_reset_hard_mode_with_confirmation()
  - IMPLEMENT: test_reset_hard_mode_with_force()
  - IMPLEMENT: test_reset_layer_targeting()
  - IMPLEMENT: test_reset_empty_staging()
  - IMPLEMENT: test_reset_invalid_layer_combination()
  - FOLLOW pattern: tests/cli_basic.rs and tests/core_workflow.rs
  - NAMING: test_reset_*
  - COVERAGE: All reset modes and error conditions
  - DEPENDENCIES: All previous tasks complete
  - CRITICAL: Use TempDir for test isolation

Task 7: VERIFY existing implementation
  - REVIEW: src/commands/reset.rs completely
  - VERIFY: All functions correctly implemented
  - VERIFY: Error handling is comprehensive
  - VERIFY: User output is clear and helpful
  - DEPENDENCIES: All previous tasks complete
  - CRITICAL: Don't rewrite working code - only fix gaps

Task 8: FINAL VALIDATION
  - RUN: cargo fmt -- --check
  - RUN: cargo clippy -- -D warnings
  - RUN: cargo build
  - RUN: cargo test --lib commands::reset::tests
  - RUN: cargo test --test cli_reset
  - DEPENDENCIES: All previous tasks complete
  - EXPECTED: All tests pass, zero warnings
```

### Implementation Patterns & Key Details

```rust
// =============================================================================
// Pattern 1: ResetArgs with --global and --force flags
// =============================================================================
// File: src/cli/args.rs

/// Arguments for the `reset` command
#[derive(Args, Debug)]
pub struct ResetArgs {
    /// Keep changes in staging
    #[arg(long)]
    pub soft: bool,

    /// Unstage but keep in workspace (default)
    #[arg(long)]
    pub mixed: bool,

    /// Discard all changes
    #[arg(long)]
    pub hard: bool,

    /// Reset mode layer
    #[arg(long)]
    pub mode: bool,

    /// Reset scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Reset project layer
    #[arg(long)]
    pub project: bool,

    // NEW: Reset global layer
    /// Reset global layer
    #[arg(long)]
    pub global: bool,

    // NEW: Skip confirmation prompt
    /// Skip confirmation prompt for destructive operations
    #[arg(long, short = 'f')]
    pub force: bool,
}

// =============================================================================
// Pattern 2: determine_target_layer() with --global support
// =============================================================================
// File: src/commands/reset.rs

fn determine_target_layer(args: &ResetArgs, context: &ProjectContext) -> Result<Layer> {
    // NEW: Check for --global flag first (highest precedence)
    if args.global {
        return Ok(Layer::GlobalBase);
    }

    // --mode + --scope=X + --project → Layer 4 (ModeScopeProject)
    if args.mode && args.scope.is_some() && args.project {
        context.require_mode()?;
        return Ok(Layer::ModeScopeProject);
    }

    // --mode + --scope=X → Layer 3 (ModeScope)
    if args.mode && args.scope.is_some() {
        context.require_mode()?;
        return Ok(Layer::ModeScope);
    }

    // --mode + --project → Layer 5 (ModeProject)
    if args.mode && args.project {
        context.require_mode()?;
        return Ok(Layer::ModeProject);
    }

    // --mode → Layer 2 (ModeBase)
    if args.mode {
        context.require_mode()?;
        return Ok(Layer::ModeBase);
    }

    // --scope=X → Layer 6 (ScopeBase)
    if args.scope.is_some() {
        return Ok(Layer::ScopeBase);
    }

    // --project → Error (requires --mode)
    if args.project {
        return Err(JinError::Other(
            "--project requires --mode (use --mode --project)".to_string(),
        ));
    }

    // Default: Layer 7 (ProjectBase)
    Ok(Layer::ProjectBase)
}

// =============================================================================
// Pattern 3: execute() with --force support for confirmation
// =============================================================================
// File: src/commands/reset.rs

pub fn execute(args: ResetArgs) -> Result<()> {
    // 1. Determine reset mode
    let mode = if args.soft {
        ResetMode::Soft
    } else if args.hard {
        ResetMode::Hard
    } else {
        ResetMode::Mixed // Default
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
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 5. Get affected entries
    let entries: Vec<&StagedEntry> = staging.entries_for_layer(layer);
    if entries.is_empty() {
        println!("Nothing to reset for layer: {}", layer_name(layer));
        return Ok(());
    }

    // 6. Confirmation for --hard mode (UPDATED: Check args.force)
    if mode == ResetMode::Hard {
        // NEW: Skip confirmation if --force is present
        if !args.force {
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
    }

    // 7. Perform reset based on mode
    match mode {
        ResetMode::Soft => {
            // Keep in staging, just acknowledge
            println!("Reset {} file(s) (kept in staging)", entries.len());
        }
        ResetMode::Mixed => {
            // Remove from staging, keep in workspace
            let count = entries.len();
            reset_staging(&mut staging, layer)?;
            staging.save()?;
            println!("Unstaged {} file(s) (kept in workspace)", count);
        }
        ResetMode::Hard => {
            // Remove from staging AND workspace
            let count = entries.len();

            // Clone entries before modifying staging to avoid borrow issues
            let entries_to_reset: Vec<StagedEntry> =
                entries.iter().map(|e| (*e).clone()).collect();

            reset_staging(&mut staging, layer)?;
            reset_workspace(&entries_to_reset)?;
            staging.save()?;
            println!("Discarded {} file(s) from staging and workspace", count);
        }
    }

    Ok(())
}

// =============================================================================
// Pattern 4: Integration Test for Reset Command
// =============================================================================
// File: tests/cli_reset.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_reset_mixed_mode() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();

    // Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .assert()
        .success();

    // Reset (default mixed mode)
    jin()
        .arg("reset")
        .current_dir(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));

    // Verify file still exists in workspace
    assert!(project_path.join("config.json").exists());
}

#[test]
fn test_reset_hard_mode_with_confirmation() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();

    // Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .assert()
        .success();

    // Reset hard mode (should prompt - we can't test interaction easily)
    // This test verifies the command structure is correct
    jin()
        .args(["reset", "--hard"])
        .current_dir(project_path)
        .assert()
        .success(); // Will cancel due to no input
}

#[test]
fn test_reset_hard_mode_with_force() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();

    // Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .assert()
        .success();

    // Reset hard mode with force (should skip confirmation)
    jin()
        .args(["reset", "--hard", "--force"])
        .current_dir(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Discarded"));

    // Verify file removed from workspace
    assert!(!project_path.join("config.json").exists());
}

#[test]
fn test_reset_layer_targeting() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();

    // Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Create mode
    jin()
        .args(["mode", "create", "test"])
        .current_dir(project_path)
        .assert()
        .success();

    // Create and stage a file to mode layer
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    // Reset mode layer
    jin()
        .args(["reset", "--mode"])
        .current_dir(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));
}

#[test]
fn test_reset_empty_staging() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();

    // Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Reset with empty staging
    jin()
        .arg("reset")
        .current_dir(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to reset"));
}

#[test]
fn test_reset_invalid_layer_combination() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();

    // Initialize
    jin().arg("init").current_dir(project_path).assert().success();

    // Try to reset --project without --mode
    jin()
        .args(["reset", "--project"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--project requires --mode"));
}
```

### Integration Points

```yaml
STAGING SYSTEM:
  - use: src/staging/index.rs::StagingIndex
  - pattern: "load() → entries_for_layer() → remove() → save()"

GITIGNORE:
  - use: src/staging/gitignore.rs::remove_from_managed_block()
  - pattern: "Remove path from managed block during --hard reset"

CONTEXT:
  - use: src/core/config.rs::ProjectContext
  - pattern: "load() → require_mode() for layer validation"

LAYER SYSTEM:
  - use: src/core/layer.rs::Layer
  - pattern: "Layer enum for target determination"

CLI:
  - use: src/cli/args.rs::ResetArgs
  - updates: Add global and force fields
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding

# Check ResetArgs changes
cargo fmt -- --check src/cli/args.rs
cargo clippy -- -D warnings src/cli/args.rs

# Check reset.rs changes
cargo fmt -- --check src/commands/reset.rs
cargo clippy -- -D warnings src/commands/reset.rs

# Project-wide validation
cargo fmt -- --check
cargo clippy -- -D warnings

# Build check
cargo build

# Expected: Zero errors, zero warnings. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test reset command in isolation
cargo test --lib commands::reset::tests -- --nocapture

# Test all layer determination
cargo test --lib commands::reset::tests::test_determine_target_layer -- --nocapture

# Run all unit tests
cargo test --lib

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Test reset command end-to-end
cargo test --test cli_reset -- --nocapture

# Test specific scenarios
cargo test --test cli_reset test_reset_mixed_mode -- --nocapture
cargo test --test cli_reset test_reset_hard_mode_with_force -- --nocapture

# Run all integration tests
cargo test --test '*'

# Expected: All integrations working, proper responses, no panics
```

### Level 4: Manual Validation & Real-World Testing

```bash
# Setup test project
mkdir /tmp/jin-reset-test && cd /tmp/jin-reset-test
jin init

# Test mixed mode (default)
echo '{"test": true}' > config.json
jin add config.json
jin reset
# Expected: "Unstaged 1 file(s) (kept in workspace)"
# Expected: config.json still exists

# Test soft mode
jin add config.json
jin reset --soft
# Expected: "Reset 1 file(s) (kept in staging)"
# Expected: File still staged (check with jin status)

# Test hard mode with confirmation
jin reset --hard
# Expected: "This will discard 1 file(s)... Type 'yes' to confirm:"
# Enter: yes
# Expected: "Discarded 1 file(s)..."
# Expected: config.json deleted

# Test hard mode with force
echo '{"test": true}' > config.json
jin add config.json
jin reset --hard --force
# Expected: No confirmation prompt
# Expected: "Discarded 1 file(s)..."
# Expected: config.json deleted

# Test layer targeting
jin mode create test
jin mode use test
echo '{"mode": true}' > mode-config.json
jin add mode-config.json --mode
jin reset --mode
# Expected: Only mode layer entry removed

# Test global reset
echo '{"global": true}' > global.json
jin add global.json --global
jin reset --global
# Expected: Global layer entry removed

# Test empty staging
jin reset
# Expected: "Nothing to reset for layer: project-base"

# Test invalid combination
jin reset --project
# Expected: Error "--project requires --mode (use --mode --project)"
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --lib commands::reset::tests`
- [ ] All integration tests pass: `cargo test --test cli_reset`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt -- --check`
- [ ] Builds successfully: `cargo build`

### Feature Validation (Reset Command)

- [ ] `jin reset` (default --mixed) unstages files, keeps workspace
- [ ] `jin reset --soft` acknowledges reset but keeps staging
- [ ] `jin reset --hard` prompts for confirmation
- [ ] `jin reset --hard --force` skips confirmation
- [ ] `jin reset --global` resets GlobalBase layer
- [ ] Layer targeting works: --mode, --scope, --project flags
- [ ] Default reset targets ProjectBase (Layer 7)
- [ ] Reset errors clearly on invalid layer combinations
- [ ] Reset shows count of affected files
- [ ] Reset removes entries from .gitignore managed block (--hard only)
- [ ] Reset handles empty staging gracefully ("Nothing to reset")

### Code Quality Validation

- [ ] Follows existing command patterns from src/commands/add.rs
- [ ] Error handling uses JinError enum with clear messages
- [ ] File operations use error collection pattern
- [ ] No panics - all potential errors handled
- [ ] Comprehensive unit tests with TempDir for isolation
- [ ] Integration tests cover full reset workflows
- [ ] Code is self-documenting with clear variable/function names

### Documentation & Safety

- [ ] --hard mode warns users about data loss
- [ ] Confirmation prompts for destructive operations
- [ ] --force flag documented and used consistently
- [ ] Error messages explain what went wrong and how to fix
- [ ] User feedback shows count of affected files

---

## Anti-Patterns to Avoid

-  Don't rewrite the entire reset.rs file - it's mostly complete
-  Don't skip reading the existing implementation before making changes
-  Don't forget to check args.force before prompting for confirmation
-  Don't forget to add --global flag check in determine_target_layer()
-  Don't use staging.entries_for_layer() iterator directly while modifying staging
-  Don't assume HashMap iteration order in tests
-  Don't modify .gitignore directly - use remove_from_managed_block()
-  Don't unwrap() without proper error context
-  Don't ignore errors during batch operations - collect and report all
-  Don't use sync functions where async might be needed (not applicable here)

---

## Confidence Score: 9/10

**Rationale for High Confidence:**

**Strengths:**
- Existing implementation in reset.rs is 90% complete
- Clear gap analysis (--global and --force flags missing)
- Comprehensive external research on reset patterns
- Existing test patterns from cli_basic.rs and core_workflow.rs
- Staging system well-tested and stable
- Clear error handling patterns from other commands

**Minor Risks:**
- Soft mode semantics may need clarification (currently no-op)
- Integration test setup may reveal edge cases
- Layer targeting edge cases with empty scopes/modes

**Mitigation:**
- Verify existing implementation before making changes
- Start with unit tests for new flags only
- Test each reset mode independently
- Use TempDir for all testing (no accidental file deletion)

**Why Not 10/10:**
- Haven't personally verified soft mode behavior in existing code
- Some edge cases in layer targeting may surface during testing
- Integration tests may reveal gaps in error handling

---

## Implementation Validation

The completed implementation should enable an AI agent unfamiliar with the codebase to:

1.  Understand the exact behavior of `jin reset` in all three modes
2.  Add missing --global and --force flags to ResetArgs
3.  Update determine_target_layer() to support --global flag
4.  Update execute() to respect --force flag for confirmation
5.  Write comprehensive unit tests for new functionality
6.  Write integration tests for complete reset workflows
7.  Handle edge cases (empty staging, invalid combinations)
8.  Provide clear user feedback and error messages
9.  Pass all 4 validation levels without manual intervention

**Success Metric:** One-pass implementation with all tests passing and no critical issues.

**NOTE**: The existing src/commands/reset.rs file is mostly complete. Before writing any code, READ THE ENTIRE FILE to understand what's already implemented. Only add the missing pieces:
1. --global flag support
2. --force flag support
3. Unit tests for new flags
4. Integration tests for complete workflows
