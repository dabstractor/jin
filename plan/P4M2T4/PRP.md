# Product Requirement Prompt: Status Command (P4.M2.T4)

---

## Goal

**Feature Goal**: Implement `jin status` command that displays workspace state, active contexts (mode/scope/project), staged changes, workspace cleanliness, and layer composition summary.

**Deliverable**: Enhanced `src/commands/status.rs` with comprehensive status display matching PRD specification.

**Success Definition**:
- Command runs successfully in any initialized Jin project
- Displays active mode, scope, and project name
- Shows workspace state (clean/dirty) with file counts
- Lists staged changes with target layer mapping
- Provides layer summary showing files per layer
- All tests pass: `cargo test --test cli_basic`, `cargo test --test core_workflow`

## User Persona

**Target User**: Developer using Jin to manage layered configuration across different modes (dev/staging/prod), scopes (language:javascript, env:prod), and projects.

**Use Case**: Developer wants to quickly understand the current state of their Jin workspace - what's active, what's staged, what's modified, and how files are distributed across layers.

**User Journey**:
1. Developer runs `jin status` in project directory
2. Command shows active mode/scope/project at top
3. Shows if workspace is clean or has uncommitted changes
4. Lists any staged changes ready for commit
5. Provides summary of files per layer
6. Offers helpful next-step commands if relevant

**Pain Points Addressed**:
- **Visibility**: Without status, developer must run multiple commands (context, layers, diff) to understand state
- **Confusion**: Hard to remember which mode/scope is active without checking
- **Safety**: Risk of committing to wrong layer without clear visibility

## Why

- **Workflow visibility**: Status is the primary "checkpoint" command in VCS workflows (git status is most frequently used command)
- **Error prevention**: Clear display of active context prevents accidental commits to wrong layer
- **Decision support**: Helps developers decide next action (commit, apply, add, etc.)
- **Integration with existing commands**: Completes the core command trio (init → add → commit → status)

## What

`jin status` displays a comprehensive overview of the Jin workspace state.

### Expected Output Format

**Clean workspace with active mode and staged files:**
```bash
$ jin status
Jin status:

  Mode:  dev (active)
  Scope: env:prod (active)
  Project: my-api-service

Workspace state: Clean
Staged changes (2 files):
  config.json → mode+project
  settings.yaml → mode

Use 'jin commit -m "message"' to commit staged changes.

Layer summary:
  global: 0 files
  mode/dev: 3 files
  mode/dev/scope/env:prod: 1 file
  mode/dev/project/my-api-service: 2 files (1 staged)
  scope/env:prod: 0 files
  project/my-api-service: 0 files
```

**Dirty workspace (uncommitted changes):**
```bash
$ jin status
Jin status:

  Mode:  dev (active)
  Scope: (none)
  Project: my-api-service

Workspace state: Dirty (2 files modified)
  config.json (modified)
  .vscode/settings.json (modified)

Use 'jin diff' to see changes or 'jin add <file>' to stage them.

No staged changes.
Use 'jin add <file>' to stage files for commit.

Layer summary:
  global: 1 file
  mode/dev: 2 files
  mode/dev/project/my-api-service: 5 files
```

**Fresh initialization (no active mode, empty):**
```bash
$ jin status
Jin status:

  Mode:  (none)
  Scope: (none)
  Project: my-api-service

Workspace state: Clean
No staged changes.

Use 'jin add <file> --mode' to stage files to a mode layer.

Layer summary:
  global: 0 files
  project/my-api-service: 0 files
```

### Success Criteria

- [ ] Command fails with `JinError::NotInitialized` if Jin not initialized
- [ ] Shows active mode, scope, project (or "none" if not set)
- [ ] Detects workspace dirty state using `WorkspaceMetadata`
- [ ] Lists modified files when workspace is dirty
- [ ] Shows staged changes count and file-to-layer mapping
- [ ] Displays layer summary with file counts per applicable layer
- [ ] Provides contextually relevant help text
- [ ] Existing test passes: `test_status_subcommand`
- [ ] New tests added for all status scenarios

---

## All Needed Context

### Context Completeness Check

_Before implementing, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: YES - This PRP provides all necessary context including:
- Exact file paths and line numbers for patterns to follow
- Complete API references for staging, workspace metadata, and layer systems
- Specific code patterns extracted from existing commands
- Test patterns and validation commands
- External research on VCS status display patterns

### Documentation & References

```yaml
# MUST READ - PRP Template Concepts
- docfile: plan/P4M2T4/PRP-README
  why: Understand PRP methodology and context requirements
  section: Full document

# MUST READ - PRD Specification for Status Command
- url: plan/PRD.md#section-18.6
  why: Exact specification for what status should show
  critical: Status command shows workspace state, active contexts, dirty files

- url: plan/PRD.md#section-20
  why: Example workflow showing expected status output format
  critical: Example shows "Active mode: claude", "Active scope: language:javascript", "Clean workspace", "Files tracked: 15"

- url: plan/PRD.md#section-12
  why: Layer system context and workspace dirty detection
  critical: Status separates "workspace dirty" from "layer dirty"

# CLI Command Patterns - Follow These Exactly

- file: src/commands/init.rs
  why: Simple command pattern (no args)
  pattern: pub fn execute() -> Result<()>
  gotcha: Always check initialization first

- file: src/commands/add.rs
  why: Complex command pattern with args, error handling, context loading
  pattern: Load context, validate, process, display summary
  lines: 34-127 (execute function), 160-189 (validate_file function)

- file: src/commands/commit_cmd.rs
  why: Command that uses staging index and displays results
  pattern: Load staging, process, display formatted results
  lines: 25-59 (execute with error handling), 62-80 (display_commit_result)

- file: src/commands/layers.rs
  why: Layer iteration and file counting pattern to reuse
  pattern: Iterate layers, check refs, count files, display summary
  lines: 11-124 (main execute), 110-124 (count_files_in_layer function)
  critical: Use count_files_in_layer pattern for layer summary

- file: src/commands/apply.rs
  why: Workspace dirty detection pattern to reuse
  pattern: check_workspace_dirty function using WorkspaceMetadata
  lines: 246-272 (check_workspace_dirty function)
  critical: Exact pattern for detecting modified/deleted files

- file: src/commands/status.rs
  why: Current basic implementation - enhance this file
  pattern: Load context, display mode/scope, show staging info
  lines: 1-58 (current implementation)
  gotcha: Already shows basic mode/scope and staging - need to add project, workspace state, layer summary

# Core System APIs

- file: src/core/config.rs
  why: ProjectContext structure and API
  pattern: ProjectContext::load() -> Result<ProjectContext>
  pattern: context.mode, context.scope, context.project fields
  lines: 78-154 (ProjectContext impl)

- file: src/core/layer.rs
  why: Layer enum and API for layer operations
  pattern: Layer::all_in_precedence_order() for iteration
  pattern: layer.ref_path(), layer.storage_path() for display
  pattern: layer.requires_mode(), layer.requires_scope() for filtering
  lines: 10-158 (Layer enum and methods)

- file: src/staging/index.rs
  why: StagingIndex API for staged files
  pattern: StagingIndex::load(), staging.entries(), staging.len()
  pattern: staging.entries_for_layer(Layer) for layer-specific counts
  lines: 24-129 (StagingIndex impl)

- file: src/staging/metadata.rs
  why: WorkspaceMetadata API for workspace dirty detection
  pattern: WorkspaceMetadata::load() -> Result<WorkspaceMetadata>
  pattern: metadata.files HashMap<PathBuf, String> for hash comparison
  lines: 27-97 (WorkspaceMetadata impl)

- file: src/git/repo.rs
  why: JinRepo API for Git operations
  pattern: JinRepo::open_or_create() for repo access
  pattern: repo.create_blob() for content hashing

# CLI Registration

- file: src/cli/mod.rs
  why: Commands enum - Status already registered
  pattern: Commands::Status (line 40) - no args needed
  gotcha: Status is already registered as simple command (no args struct)

- file: src/cli/args.rs
  why: Args struct definitions - Status doesn't need args
  gotcha: Status command uses unit variant, no AddArgs-style struct needed

- file: src/commands/mod.rs
  why: Central command dispatcher
  pattern: Commands::Status => status::execute(),
  lines: 31-57 (match statement)
  gotcha: Status already wired to execute() function

# Test Patterns

- file: tests/cli_basic.rs
  why: Basic CLI test pattern
  pattern: jin() Command builder, assert() for validation
  lines: 47-59 (test_status_subcommand)
  pattern: Use tempfile::TempDir for isolation, predicate::str::contains for output validation

- file: tests/common/fixtures.rs
  why: Test fixtures for Jin repository setup
  pattern: TestFixture::new(), setup_test_repo()
  pattern: Use std::process::id() for unique test names

- file: tests/common/assertions.rs
  why: Custom assertions for Jin state
  pattern: assert_jin_initialized(), assert_staging_contains()

# External Research - Status Command UI Patterns

- url: https://git-scm.com/docs/git-status
  why: Git status output format reference
  critical: Section headers ("Changes to be committed:", "Changes not staged for commit:"), status symbols (M, A, D, ??)

- url: https://clig.dev/
  why: CLI design guidelines for output formatting
  critical: "Be concise", "Use color sparingly", "Group related information"
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── src/
│   ├── cli/
│   │   ├── mod.rs          # Commands enum (Status at line 40)
│   │   └── args.rs         # Args structs (Status doesn't need args)
│   ├── commands/
│   │   ├── mod.rs          # Command dispatcher (Status at line 40)
│   │   ├── status.rs       # EXISTING - Enhance this file
│   │   ├── init.rs         # Simple command pattern
│   │   ├── add.rs          # Complex command with args
│   │   ├── commit_cmd.rs   # Staging usage pattern
│   │   ├── layers.rs       # Layer iteration and file counting
│   │   └── apply.rs        # Workspace dirty detection
│   ├── core/
│   │   ├── config.rs       # ProjectContext API
│   │   ├── layer.rs        # Layer enum and methods
│   │   └── error.rs        # JinError types
│   ├── staging/
│   │   ├── index.rs        # StagingIndex API
│   │   ├── entry.rs        # StagedEntry structure
│   │   └── metadata.rs     # WorkspaceMetadata API
│   └── git/
│       └── repo.rs         # JinRepo API
└── tests/
    ├── cli_basic.rs        # Basic CLI tests (status test exists)
    ├── core_workflow.rs    # Workflow integration tests
    └── common/
        ├── fixtures.rs     # Test fixtures
        └── assertions.rs   # Custom assertions
```

### Desired Codebase Tree (Changes Only)

```bash
# File to modify:
src/commands/status.rs      # Enhance with workspace state and layer summary

# Tests to add:
tests/cli_basic.rs          # Add: test_status_with_mode, test_status_dirty_workspace
tests/core_workflow.rs      # Add: test_status_command_in_workflow
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: WorkspaceMetadata::load() returns NotFound if doesn't exist
// Pattern from apply.rs lines 250-252:
let metadata = match WorkspaceMetadata::load() {
    Ok(m) => m,
    Err(_) => return Ok(false), // No metadata = clean workspace
};

// CRITICAL: JinRepo operations need JIN_DIR environment variable for tests
// Pattern from cli_basic.rs lines 34-36:
let jin_dir = temp.path().join(".jin_global");
std::env::set_var("JIN_DIR", &jin_dir);

// GOTCHA: Git tree walking for file counting uses git2::TreeWalkMode::PreOrder
// Pattern from layers.rs lines 115-121:
tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
    if entry.kind() == Some(git2::ObjectType::Blob) {
        count += 1;
    }
    git2::TreeWalkResult::Ok
})?;

// GOTCHA: Layer filtering based on active context
// Pattern from layers.rs lines 50-56:
if layer.requires_mode() && context.mode.is_none() {
    continue;
}
if layer.requires_scope() && context.scope.is_none() {
    continue;
}

// GOTCHA: Error message for "not initialized" should be consistent
// Pattern from status.rs line 12:
return Err(JinError::NotInitialized);

// CRITICAL: Status command is already registered and wired
// DO NOT modify src/cli/mod.rs or src/commands/mod.rs
// Only modify src/commands/status.rs

// GOTCHA: Workspace state check should distinguish "clean" from "dirty"
// Clean: All tracked files have matching hashes
// Dirty: Any tracked file deleted, modified, or new file added

// PATTERN: Display formatted output with sections
// Pattern from init.rs lines 32-37:
println!("Initialized Jin in {}", jin_dir.display());
println!();
println!("Next steps:");

// PATTERN: Conditional help text based on state
// Pattern from status.rs lines 39-42:
if staged_count == 0 {
    println!("No staged changes.");
    println!();
    println!("Use 'jin add <file>' to stage files for commit.");
}
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models needed. Use existing structures:

```rust
// Existing - src/core/config.rs:78-97
pub struct ProjectContext {
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    // ...
}

// Existing - src/staging/index.rs:10-17
pub struct StagingIndex {
    entries: HashMap<PathBuf, StagedEntry>,
    // ...
}

// Existing - src/staging/metadata.rs:18-25
pub struct WorkspaceMetadata {
    pub timestamp: String,
    pub applied_layers: Vec<String>,
    pub files: HashMap<PathBuf, String>, // path -> hash
}

// Existing - src/staging/entry.rs:9-20
pub struct StagedEntry {
    pub path: PathBuf,
    pub target_layer: Layer,
    pub content_hash: String,
    pub operation: StagedOperation,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/commands/status.rs - Add Project Display
  - IMPLEMENT: Display context.project field
  - FOLLOW pattern: src/commands/status.rs lines 22-31 (mode/scope display)
  - NAMING: Match existing "Mode:" and "Scope:" format
  - PLACEMENT: After scope display, before blank line

Task 2: MODIFY src/commands/status.rs - Add Workspace State Detection
  - IMPLEMENT: check_workspace_state() helper function
  - FOLLOW pattern: src/commands/apply.rs lines 246-272 (check_workspace_dirty)
  - NAMING: fn check_workspace_state() -> Result<WorkspaceState>
  - DEPENDENCIES: Uses WorkspaceMetadata::load()
  - RETURNS: WorkspaceState enum (Clean, Dirty { modified: Vec<PathBuf>, deleted: Vec<PathBuf> })
  - PLACEMENT: Private helper function before execute()

Task 3: MODIFY src/commands/status.rs - Display Workspace State
  - IMPLEMENT: Show "Clean" or "Dirty (N files modified)" after context section
  - FOLLOW pattern: src/commands/init.rs lines 32-37 (formatted output)
  - NAMING: match workspace_state { Clean => ..., Dirty { .. } => ... }
  - PLACEMENT: After project display, before staged changes section
  - FORMAT: "Workspace state: Clean" or "Workspace state: Dirty (N files modified)"

Task 4: MODIFY src/commands/status.rs - List Modified Files When Dirty
  - IMPLEMENT: Iterate dirty files and display with status symbol
  - FOLLOW pattern: git status format (M for modified, D for deleted)
  - NAMING: "  {} (modified)" or "  {} (deleted)"
  - PLACEMENT: Under "Workspace state: Dirty" line, indented
  - HELP: Add "Use 'jin diff' to see changes..." hint

Task 5: MODIFY src/commands/status.rs - Add Layer Summary Section
  - IMPLEMENT: show_layer_summary() helper function
  - FOLLOW pattern: src/commands/layers.rs lines 45-96 (layer iteration and counting)
  - NAMING: fn show_layer_summary(context: &ProjectContext, repo: &JinRepo, staging: &StagingIndex)
  - DEPENDENCIES: JinRepo for file counting, StagingIndex for staged counts
  - PLACEMENT: After staged changes section, before return
  - FORMAT: "  jin/global: N files" or "  jin/mode/{name}: N files (M staged)"

Task 6: MODIFY src/commands/status.rs - Update execute() Function
  - INTEGRATE: Call new helper functions in correct order
  - PRESERVE: Existing initialization check and context loading
  - ADD: JinRepo::open_or_create() for layer summary
  - ADD: WorkspaceMetadata::load() for workspace state
  - ADD: Layer summary display at end
  - PLACEMENT: Extend existing execute() function

Task 7: ADD WorkspaceState Enum to status.rs
  - IMPLEMENT: Enum representing workspace state
  - NAMING: enum WorkspaceState { Clean, Dirty { modified: Vec<PathBuf>, deleted: Vec<PathBuf> } }
  - PLACEMENT: At top of file, after imports

Task 8: CREATE tests/cli_basic.rs - Add Status Tests
  - IMPLEMENT: test_status_with_active_mode(), test_status_dirty_workspace()
  - FOLLOW pattern: tests/cli_basic.rs lines 47-59 (test_status_subcommand)
  - NAMING: test_status_[scenario] function naming
  - COVERAGE: Clean workspace, dirty workspace, with mode/scope, with staged files
  - FIXTURES: Use setup_test_repo() from common/fixtures

Task 9: CREATE tests/core_workflow.rs - Add Status to Workflow Test
  - IMPLEMENT: Extend existing workflow test to check status output
  - FOLLOW pattern: tests/core_workflow.rs workflow test structure
  - INTEGRATION: Test status after init, add, and commit operations
  - ASSERTION: Verify status shows correct state at each step
```

### Implementation Patterns & Key Details

```rust
// ============================================================
// Task 2: Workspace State Detection Helper
// ============================================================
// PATTERN: Load metadata, compare file hashes, return state
// FOLLOW: src/commands/apply.rs lines 246-272

enum WorkspaceState {
    Clean,
    Dirty { modified: Vec<PathBuf>, deleted: Vec<PathBuf> },
}

fn check_workspace_state() -> Result<WorkspaceState> {
    // PATTERN: Handle missing metadata as clean state
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => return Ok(WorkspaceState::Clean),
        Err(e) => return Err(e),
    };

    // CRITICAL: Open repo for hash comparison
    let repo = JinRepo::open()?;

    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    // PATTERN: Compare current file hashes to stored hashes
    for (path, expected_hash) in &metadata.files {
        if !path.exists() {
            deleted.push(path.clone());
        } else {
            let content = std::fs::read(path)?;
            let current_hash = repo.create_blob(&content)?.to_string();
            if current_hash != *expected_hash {
                modified.push(path.clone());
            }
        }
    }

    if modified.is_empty() && deleted.is_empty() {
        Ok(WorkspaceState::Clean)
    } else {
        Ok(WorkspaceState::Dirty { modified, deleted })
    }
}

// ============================================================
// Task 5: Layer Summary Helper
// ============================================================
// PATTERN: Iterate applicable layers, count files, display
// FOLLOW: src/commands/layers.rs lines 45-96

fn show_layer_summary(
    context: &ProjectContext,
    repo: &JinRepo,
    staging: &StagingIndex,
) -> Result<()> {
    let git_repo = repo.inner();
    println!();
    println!("Layer summary:");

    // PATTERN: Get applicable layers for current context
    for layer in Layer::all_in_precedence_order() {
        // GOTCHA: Skip layers that don't apply to current context
        if layer.requires_mode() && context.mode.is_none() {
            continue;
        }
        if layer.requires_scope() && context.scope.is_none() {
            continue;
        }

        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // PATTERN: Count files in layer using tree walk
        let committed_files = if git_repo.find_reference(&ref_path).is_ok() {
            count_files_in_layer(git_repo, &ref_path).unwrap_or(0)
        } else {
            0
        };

        // PATTERN: Count staged files for this layer
        let staged_files = staging.entries_for_layer(layer).len();

        let storage_path = layer.storage_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // PATTERN: Display with staged count if any
        if committed_files > 0 || staged_files > 0 {
            let staged_note = if staged_files > 0 {
                format!(" ({} staged)", staged_files)
            } else {
                String::new()
            };
            println!("  {}: {} file{}{}",
                storage_path,
                committed_files + staged_files,
                if (committed_files + staged_files) == 1 { "" } else { "s" },
                staged_note
            );
        }
    }

    Ok(())
}

// PATTERN: File counting from layers.rs lines 110-124
fn count_files_in_layer(repo: &git2::Repository, ref_path: &str) -> Result<usize> {
    let reference = repo.find_reference(ref_path)?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    let mut count = 0;
    tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            count += 1;
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(count)
}

// ============================================================
// Task 6: Enhanced execute() Function Structure
// ============================================================
// PATTERN: Extend existing execute() with new sections

pub fn execute() -> Result<()> {
    // PATTERN: Check initialization first (line 11-13)
    if !ProjectContext::is_initialized() {
        return Err(JinError::NotInitialized);
    }

    // Load context (line 16)
    let context = ProjectContext::load()?;

    // Open repo for layer operations
    let repo = JinRepo::open_or_create()?;

    // Load staging
    let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // HEADER SECTION (existing, with project added)
    println!("Jin status:");
    println!();

    // Context display (add project to existing)
    match &context.mode {
        Some(mode) => println!("  Mode:  {} (active)", mode),
        None => println!("  Mode:  (none)"),
    }
    match &context.scope {
        Some(scope) => println!("  Scope: {} (active)", scope),
        None => println!("  Scope: (none)"),
    }
    // NEW: Project display
    match &context.project {
        Some(project) => println!("  Project: {}", project),
        None => println!("  Project: (none)"),
    }
    println!();

    // NEW: Workspace state section
    let workspace_state = check_workspace_state()?;
    match workspace_state {
        WorkspaceState::Clean => {
            println!("Workspace state: Clean");
            println!();
        }
        WorkspaceState::Dirty { modified, deleted } => {
            let total = modified.len() + deleted.len();
            println!("Workspace state: Dirty ({} file{} modified)",
                total,
                if total == 1 { "" } else { "s" }
            );
            // List modified files
            for path in &modified {
                println!("  {} (modified)", path.display());
            }
            for path in &deleted {
                println!("  {} (deleted)", path.display());
            }
            println!();
            println!("Use 'jin diff' to see changes or 'jin add <file>' to stage them.");
            println!();
        }
    }

    // STAGED CHANGES SECTION (existing, enhanced)
    let staged_count = staging.len();

    if staged_count == 0 {
        println!("No staged changes.");
        // Context-sensitive help
        if context.mode.is_none() && context.scope.is_none() && context.project.is_none() {
            println!();
            println!("Use 'jin add <file> --mode' to stage files to a mode layer.");
        } else {
            println!();
            println!("Use 'jin add <file>' to stage files for commit.");
        }
    } else {
        println!(
            "Staged changes ({} file{}):",
            staged_count,
            if staged_count == 1 { "" } else { "s" }
        );
        for entry in staging.entries() {
            println!("  {} -> {}", entry.path.display(), entry.target_layer);
        }
        println!();
        println!("Use 'jin commit -m <message>' to commit staged changes.");
    }

    // NEW: Layer summary section
    show_layer_summary(&context, &repo, &staging)?;

    Ok(())
}
```

### Integration Points

```yaml
NO NEW INTEGRATIONS NEEDED:

All required modules are already imported and used:
- src/core/config (ProjectContext) - already imported
- src/staging (StagingIndex) - already imported
- src/git (JinRepo) - need to add to imports
- src/core/layer (Layer) - need to add to imports

MODIFY:
  file: src/commands/status.rs
  add_imports: |
    use crate::core::Layer;
    use crate::git::JinRepo;
    use crate::staging::WorkspaceMetadata;
    use std::path::PathBuf;
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modifying status.rs - fix before proceeding
cargo check --bin jin 2>&1 | head -50

# Format check
cargo fmt -- --check src/commands/status.rs

# Clippy for linting
cargo clippy --bin jin 2>&1 | grep -A5 "status.rs"

# Expected: No errors, no warnings. If errors exist, READ output and fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run unit tests in status.rs
cargo test --lib status 2>&1 | tail -20

# Run all command tests
cargo test --lib commands 2>&1 | tail -30

# Expected: All tests pass. Check specifically for:
# - test_check_workspace_state_clean
# - test_check_workspace_state_dirty
# - test_show_layer_summary
# - test_execute_with_all_sections
```

### Level 3: Integration Testing (System Validation)

```bash
# CLI basic tests (existing + new)
cargo test --test cli_basic test_status 2>&1 | tail -30

# Expected: All status tests pass including:
# - test_status_subcommand (existing)
# - test_status_with_active_mode (new)
# - test_status_dirty_workspace (new)
# - test_status_with_staged_files (new)

# Core workflow integration
cargo test --test core_workflow 2>&1 | tail -30

# Test manually in real project
cd /tmp/jin-test-$$ && mkdir -p .jin
echo "version: 1" > .jin/context
echo "mode: dev" >> .jin/context
jin status
# Expected: Shows "Mode: dev (active)", no errors

# Test with staged file
echo '{}' > config.json
jin add config.json --mode
jin status
# Expected: Shows "Staged changes (1 file):", "config.json -> mode-base"
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test status output formatting
jin status 2>&1 | tee /tmp/status-output.txt
# Verify:
# - Header "Jin status:" present
# - Context section shows mode/scope/project
# - Workspace state section present
# - Layer summary shows applicable layers
# - Output is human-readable and properly formatted

# Test with different contexts
jin mode create testmode-$$ && jin mode use testmode-$$
jin status | grep "Mode:.*testmode"
# Expected: Shows active mode

# Test layer summary with actual commits
jin add config.json --mode
jin commit -m "test commit"
jin status | grep "Layer summary" -A5
# Expected: Shows files in mode layer

# Edge case testing
# 1. Status in non-initialized directory
cd /tmp && jin status
# Expected: Fails with "Jin not initialized"

# 2. Status with no active mode/scope
jin mode unset
jin status
# Expected: Shows "Mode: (none)", "Scope: (none)"

# 3. Status with corrupted metadata
rm .jin/workspace/last_applied.json
jin status
# Expected: Shows workspace as Clean (no metadata = clean)

# Output formatting check (no trailing whitespace, proper spacing)
jin status | sed 's/$/$/' | grep '\s$' && echo "FAIL: trailing whitespace" || echo "PASS: no trailing whitespace"
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --test cli_basic test_status`
- [ ] No linting errors: `cargo clippy --bin jin`
- [ ] No formatting issues: `cargo fmt -- --check`
- [ ] Module compiles: `cargo check --bin jin`
- [ ] `WorkspaceState` enum properly defined
- [ ] `check_workspace_state()` function works correctly
- [ ] `show_layer_summary()` function works correctly
- [ ] `count_files_in_layer()` helper function works correctly

### Feature Validation

- [ ] Shows active mode (or "none" if not set)
- [ ] Shows active scope (or "none" if not set)
- [ ] Shows project name (or "none" if not set)
- [ ] Detects and displays workspace state (Clean/Dirty)
- [ ] Lists modified/deleted files when workspace is dirty
- [ ] Shows staged changes with file-to-layer mapping
- [ ] Displays layer summary with file counts per layer
- [ ] Provides contextually relevant help text
- [ ] Fails with `JinError::NotInitialized` when not initialized

### Code Quality Validation

- [ ] Follows existing codebase patterns (init.rs for simple, add.rs for complex)
- [ ] File placement matches desired structure (only status.rs modified)
- [ ] Uses existing imports (added Layer, JinRepo, WorkspaceMetadata)
- [ ] Error handling matches existing patterns (Result<> return, JinError variants)
- [ ] Display format matches existing commands (indentation, spacing, help text)
- [ ] No anti-patterns (see below)

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Complex logic has explanatory comments
- [ ] Output is clear and user-friendly
- [ ] Help text is contextually appropriate

---

## Anti-Patterns to Avoid

- **Don't** modify src/cli/mod.rs or src/commands/mod.rs (Status already registered)
- **Don't** create new args struct in src/cli/args.rs (Status takes no arguments)
- **Don't** duplicate file counting logic - reuse `count_files_in_layer` helper
- **Don't** hardcode layer names - use `Layer::all_in_precedence_order()`
- **Don't** ignore workspace metadata when checking dirty state
- **Don't** show all 9 layers - filter by applicable context (mode/scope requirements)
- **Don't** use color output without terminal detection (keep it simple for now)
- **Don't** create parallel functions for each layer type - use iteration pattern
- **Don't** panic on missing metadata - handle gracefully as Clean state
- **Don't** show empty layers in summary - skip layers with 0 files

---

## Example Expected Outputs

### Scenario 1: Fresh Initialization
```bash
$ cd /tmp/myproject && jin init
Initialized Jin in /tmp/myproject/.jin

$ jin status
Jin status:

  Mode:  (none)
  Scope: (none)
  Project: (none)

Workspace state: Clean
No staged changes.

Use 'jin add <file> --mode' to stage files to a mode layer.

Layer summary:
  global: 0 files
```

### Scenario 2: After Creating and Using Mode
```bash
$ jin mode create dev
Created mode: dev

$ jin mode use dev
Using mode: dev

$ jin status
Jin status:

  Mode:  dev (active)
  Scope: (none)
  Project: (none)

Workspace state: Clean
No staged changes.

Use 'jin add <file>' to stage files for commit.

Layer summary:
  global: 0 files
  jin/mode/dev: 0 files
```

### Scenario 3: After Adding Files
```bash
$ echo '{"key": "value"}' > config.json
$ jin add config.json --mode
Staged 1 file(s) to mode-base layer

$ jin status
Jin status:

  Mode:  dev (active)
  Scope: (none)
  Project: (none)

Workspace state: Clean
Staged changes (1 file):
  config.json -> mode-base

Use 'jin commit -m <message>' to commit staged changes.

Layer summary:
  global: 0 files
  jin/mode/dev: 0 files (1 staged)
```

### Scenario 4: After Commit
```bash
$ jin commit -m "Add config"
Committed 1 file(s) to 1 layer(s):
  mode/dev: abc123def

$ jin status
Jin status:

  Mode:  dev (active)
  Scope: (none)
  Project: (none)

Workspace state: Clean
No staged changes.

Use 'jin add <file>' to stage files for commit.

Layer summary:
  global: 0 files
  jin/mode/dev: 1 file
```

### Scenario 5: Dirty Workspace After Apply
```bash
$ echo '{"modified": true}' > config.json

$ jin status
Jin status:

  Mode:  dev (active)
  Scope: (none)
  Project: (none)

Workspace state: Dirty (1 file modified)
  config.json (modified)

Use 'jin diff' to see changes or 'jin add <file>' to stage them.

No staged changes.
Use 'jin add <file>' to stage files for commit.

Layer summary:
  global: 0 files
  jin/mode/dev: 1 file
```

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success likelihood

**Justification**:
- **High confidence** because:
  - Existing implementation provides solid foundation (~60% complete)
  - All required APIs are well-documented and tested
  - Clear patterns to follow from similar commands (layers.rs, apply.rs)
  - No new dependencies or infrastructure needed
  - Test patterns are established and working

- **Minor risk** (1 point deduction):
  - Workspace state detection requires careful hash comparison logic
  - Layer filtering based on context needs careful testing
  - File counting performance for large repositories (mitigated by using existing pattern)

**Mitigation for remaining risk**:
- Use exact `check_workspace_dirty` pattern from apply.rs
- Use exact layer iteration pattern from layers.rs
- Comprehensive test coverage for all scenarios
