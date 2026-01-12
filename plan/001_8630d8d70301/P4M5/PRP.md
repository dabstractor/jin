# PRP: P4.M5 - Utility Commands

---

## Goal

**Feature Goal**: Implement eight utility commands (`diff`, `log`, `context`, `import`, `export`, `repair`, `layers`, `list`) that provide inspection, debugging, and migration capabilities for the Jin layer system.

**Deliverable**: Eight fully functional CLI commands providing:
1. **diff** - Compare layers, workspace, and staged changes
2. **log** - Show commit history for layers
3. **context** - Display active mode/scope/project (ALREADY IMPLEMENTED - just needs CLI wiring)
4. **import** - Move Git-tracked files into Jin management
5. **export** - Move Jin-tracked files back to Git
6. **repair** - Verify and repair Jin repository integrity
7. **layers** - Show current layer composition and merge order
8. **list** - List available modes/scopes/projects from remote

**Success Definition**:
- All commands pass integration tests in `tests/cli_basic.rs`
- Commands follow existing patterns from `src/commands/add.rs` and `src/commands/status.rs`
- `cargo test` passes with zero errors
- `cargo clippy` reports zero warnings
- Commands provide clear, actionable output
- Error messages guide users toward resolution

---

## User Persona

**Target User**: Developer debugging layer conflicts, inspecting configuration history, or migrating files between Git and Jin

**Use Case**: A developer needs to:
- Understand why workspace differs from expected configuration (diff)
- Track when a configuration change was introduced (log)
- See current active layer composition (context, layers)
- Migrate existing Git-tracked configs to Jin (import)
- Promote Jin configs back to Git (export)
- Fix corrupt Jin state after interrupted operations (repair)
- Discover what modes/scopes exist in remote (list)

**User Journey**:
1. Developer notices unexpected config value in workspace
2. Runs `jin diff` to compare workspace vs merged layers
3. Runs `jin log --layer=mode-base` to find when change was committed
4. Uses `jin layers` to understand layer composition
5. Uses `jin import` to move .gitignore'd files into Jin
6. Uses `jin repair` if Jin state becomes corrupt

**Pain Points Addressed**:
- **No visibility into layer differences** - `diff` shows exactly what differs between layers
- **No commit history** - `log` provides Git-like history per layer
- **Config migration is manual** - `import/export` automate file movement
- **Corrupt state after crashes** - `repair` auto-fixes Jin repository
- **Don't know what's available remotely** - `list` shows modes/scopes/projects

---

## Why

**Business Value:**
- Completes the inspection/debugging toolset for Jin
- Enables safe migration workflows (Git ↔ Jin)
- Provides recovery mechanisms for production use
- Reduces user friction with familiar Git-like commands
- Unblocks production deployment by providing repair capabilities

**Integration with Existing Features:**
- Builds on Layer system (P1.M3) for comparison and composition
- Uses git2-rs infrastructure (P1.M2) for diff/log operations
- Leverages staging system (P3.M1) for diff --staged
- Integrates with context management (P4.M3) for active layer detection
- Extends apply/reset (P4.M4) with inspection capabilities

**Problems This Solves:**
- Users can't debug why workspace doesn't match expectations
- No way to track configuration change history
- Manual file migration between Git and Jin is error-prone
- Interrupted operations leave corrupt state with no repair tool
- Users don't know what modes/scopes exist in shared remote

**PRD Requirements:**
- Section 18.6 specifies these as inspection/status commands
- Section 26 Acceptance Criteria #10 explicitly requires implementation
- Section 21 File Lifecycle references import for Git-tracked migration

---

## What

### User-Visible Behavior

#### jin diff

Compare layers, workspace, or staged changes.

**Usage:**
```bash
jin diff                           # Compare workspace vs workspace-active (merged layers)
jin diff --staged                  # Compare staged changes vs HEAD
jin diff mode-base project-base    # Compare two specific layers
jin diff workspace-active          # Compare workspace vs specified layer
```

**Output:**
```
diff --jin a/mode-base b/project-base
--- a/mode-base/.vscode/settings.json
+++ b/project-base/.vscode/settings.json
@@ -1,5 +1,5 @@
 {
-  "editor.fontSize": 14,
+  "editor.fontSize": 16,
   "editor.tabSize": 2
 }
```

**Behavior:**
- Uses Myers diff algorithm (via git2-rs) for text files
- Performs structured diff for JSON/YAML/TOML (shows key-level changes)
- Compares Git tree objects for layer-to-layer diffs
- Compares workspace files against materialized trees
- Color-codes additions (green), deletions (red), context (white)
- Exit code 0 if no differences, 1 if differences found, 2 if error

**Error Conditions:**
- Unknown layer name → Error with list of valid layers
- Layer doesn't exist (no commits) → Error suggesting `jin status`
- Invalid layer combination → Error with examples
- Jin not initialized → Error suggesting `jin init`

#### jin log

Show commit history for layers.

**Usage:**
```bash
jin log                            # Show history for all layers (last 10)
jin log --layer=mode-base          # Show history for specific layer
jin log --layer=project-base --count=20  # Show last 20 commits
```

**Output:**
```
commit a1b2c3d4e5f6 (mode-base)
Author: Alice <alice@example.com>
Date:   2025-12-27 10:30:00 -0800

    Add Python IDE settings

    3 files changed

commit f6e5d4c3b2a1 (mode-base)
Author: Bob <bob@example.com>
Date:   2025-12-26 15:45:00 -0800

    Configure linter rules

    1 file changed
```

**Behavior:**
- Uses git2::Revwalk to traverse commit history
- Defaults to showing last 10 commits
- Groups commits by layer if no --layer specified
- Shows commit hash (short), author, date, message, file count
- Supports --count to limit output
- Color-codes layer names for easy scanning

**Error Conditions:**
- Unknown layer name → Error with list of valid layers
- Layer has no commits → Info message "No commits yet"
- Jin not initialized → Error suggesting `jin init`

#### jin context

Display active mode/scope/project context.

**Usage:**
```bash
jin context                        # Show current active context
```

**Output:**
```
Current Jin context:

  Active mode:   claude
  Active scope:  language:javascript
  Project:       ui-dashboard
  Last updated:  2025-12-27T10:30:00Z
```

**Behavior:**
- Loads `.jin/context` file
- Displays active mode, scope, project
- Shows when context was last updated
- Gracefully handles missing mode/scope (shows "(none)")

**NOTE**: This command is ALREADY IMPLEMENTED in `src/commands/context.rs`. Only needs CLI wiring which is already done. No implementation needed.

**Error Conditions:**
- Jin not initialized → Error suggesting `jin init`

#### jin import

Import Git-tracked files into Jin management.

**Usage:**
```bash
jin import .vscode/settings.json               # Import file to project-base
jin import .vscode/ --mode                     # Import directory to mode-base
jin import config.json --force                 # Force import even if modified
```

**Behavior:**
1. Verify file is currently Git-tracked (`git ls-files`)
2. Remove file from Git index (`git rm --cached`)
3. Add file to Jin staging (`jin add` internally)
4. Add file path to `.gitignore` managed block
5. Show summary of imported files

**Force Mode (`--force`):**
- Allows import of modified files
- Skips dirty workspace check
- Overwrites existing Jin-tracked version

**Error Conditions:**
- File not Git-tracked → Error suggesting `jin add` instead
- File is modified without --force → Error with guidance
- File already in Jin → Error suggesting `jin add` to update
- Multiple .gitignore conflicts → Error with file paths

#### jin export

Export Jin-tracked files back to Git.

**Usage:**
```bash
jin export .vscode/settings.json               # Export file to Git
jin export .claude/                            # Export directory to Git
```

**Behavior:**
1. Verify file is currently Jin-tracked
2. Remove file from Jin staging/layers
3. Add file to Git index (`git add`)
4. Remove file path from `.gitignore` managed block
5. Show summary of exported files

**Error Conditions:**
- File not Jin-tracked → Error suggesting `git add` instead
- File doesn't exist in workspace → Error suggesting `jin apply`
- File is modified → Warning with guidance
- Git repository has uncommitted changes → Warning (non-blocking)

#### jin repair

Verify and repair Jin repository integrity.

**Usage:**
```bash
jin repair                                     # Repair Jin state
jin repair --dry-run                           # Show what would be repaired
```

**Output:**
```
Checking Jin repository integrity...

✓ Repository structure valid
✓ All layer refs exist
✗ Staging index corrupted - REPAIRING
✓ .jinmap file valid
✗ Workspace metadata missing - REBUILDING

Repair complete. 2 issues fixed.
```

**Repair Operations:**
1. **Verify repository structure** - Check ~/.jin/ exists and is valid bare repo
2. **Validate layer refs** - Ensure all refs/jin/layers/* point to valid commits
3. **Check staging index** - Verify .jin/staging/index.json is valid JSON
4. **Verify .jinmap** - Check .jin/.jinmap is valid and matches refs
5. **Validate workspace metadata** - Check .jin/workspace/ tracking files
6. **Rebuild indexes** - Reconstruct staging index from Git objects if corrupted
7. **Recover refs** - Restore refs from reflog if missing

**Dry-Run Mode:**
- Shows issues without fixing
- Reports what would be repaired
- Exit code 0 if no issues, 1 if issues found

**Error Conditions:**
- Fatal corruption (unrecoverable) → Error with manual recovery steps
- Permission denied → Error with file paths

#### jin layers

Show current layer composition and merge order.

**Usage:**
```bash
jin layers                                     # Show active layer stack
```

**Output:**
```
Layer composition for current context:
  Mode:    claude
  Scope:   language:javascript
  Project: ui-dashboard

Merge order (lowest to highest precedence):
  1. global-base           [global/]
  2. mode-base            [mode/claude/]
  3. mode-scope           [mode/claude/scope/language:javascript/]
  4. mode-scope-project   [mode/claude/scope/language:javascript/project/ui-dashboard/]
  5. mode-project         [mode/claude/project/ui-dashboard/]
  7. project-base         [project/ui-dashboard/]
  9. workspace-active     [.jin/workspace/]

Active layers: 7 of 9 layers have files
Total files in workspace: 23
```

**Behavior:**
- Loads active context from `.jin/context`
- Shows layer hierarchy from Layer enum
- Displays precedence numbers (1-9)
- Shows which layers are active (have commits/files)
- Indicates storage paths for each layer
- Counts total files across all layers

**Error Conditions:**
- Jin not initialized → Error suggesting `jin init`

#### jin list

List available modes/scopes/projects from remote.

**Usage:**
```bash
jin list                                       # List all modes, scopes, projects
```

**Output:**
```
Available in Jin repository:

Modes:
  - claude
  - cursor
  - zed

Scopes:
  - language:javascript
  - language:python
  - language:rust
  - infra:docker
  - infra:kubernetes

Projects:
  - ui-dashboard
  - api-server
  - mobile-app

Use 'jin mode use <mode>' to activate a mode
Use 'jin scope use <scope>' to activate a scope
```

**Behavior:**
- Enumerates all refs under `refs/jin/layers/`
- Parses ref paths to extract mode/scope/project names
- Groups by category (modes, scopes, projects)
- Alphabetically sorts each category
- Shows friendly usage hints

**Error Conditions:**
- Jin not initialized → Error suggesting `jin init`
- No remote configured → Error suggesting `jin link`
- Remote not reachable → Error with connection details

### Success Criteria

- [ ] `jin diff` compares layers using git2 diff APIs
- [ ] `jin diff --staged` shows staged changes
- [ ] `jin diff` supports structured diff for JSON/YAML/TOML
- [ ] `jin log` shows commit history per layer
- [ ] `jin log --count` limits output correctly
- [ ] `jin context` displays active context (already works)
- [ ] `jin import` moves Git-tracked files to Jin
- [ ] `jin import` updates .gitignore managed block
- [ ] `jin export` moves Jin-tracked files to Git
- [ ] `jin export` removes from .gitignore managed block
- [ ] `jin repair` detects and fixes staging index corruption
- [ ] `jin repair --dry-run` shows issues without fixing
- [ ] `jin layers` displays layer composition
- [ ] `jin list` enumerates modes/scopes/projects
- [ ] All commands return proper exit codes
- [ ] All commands have comprehensive error messages
- [ ] Integration tests pass in `tests/cli_basic.rs`

---

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement these commands successfully?"_

**YES** - This PRP provides:
- Complete command specifications with examples
- Existing implementation patterns from add.rs, context.rs, status.rs
- Git2-rs API references with specific methods
- Error handling patterns from JinError enum
- Testing patterns from tests/cli_basic.rs
- Layer system context from src/core/layer.rs

### Documentation & References

```yaml
# MUST READ - Core Jin Patterns

- file: src/commands/add.rs:1-326
  why: Complete reference implementation for command structure, error handling, validation
  pattern: |
    - Load ProjectContext::load() for active mode/scope
    - Validate routing options
    - Open JinRepo with open_or_create()
    - Process files with proper error handling
    - Save state and print summary
  gotcha: |
    - MUST check for JinError::NotInitialized
    - MUST validate files before processing
    - MUST use ensure_in_managed_block() for .gitignore

- file: src/commands/context.rs:1-89
  why: ALREADY IMPLEMENTED - Shows context display pattern
  pattern: |
    - Load ProjectContext::load()
    - Handle NotInitialized error
    - Display active mode, scope, project with graceful (none) handling
  gotcha: |
    - Context command is complete - only needs CLI wiring (already done)
    - Follow this display format for other commands

- file: src/core/layer.rs:1-284
  why: Complete Layer enum definition with precedence, ref_path, storage_path methods
  pattern: |
    - Layer::all_in_precedence_order() for iteration
    - layer.precedence() for comparison (1-9)
    - layer.ref_path(mode, scope, project) for Git refs
    - layer.requires_mode(), requires_scope(), is_project_specific() for validation
  gotcha: |
    - Precedence flows bottom (1) to top (9) - higher overrides lower
    - WorkspaceActive (9) is derived, never a source of truth
    - Use Display trait for user-facing layer names

- file: src/git/repo.rs:1-200
  why: JinRepo wrapper around git2::Repository with Jin-specific operations
  pattern: |
    - JinRepo::open_or_create() for repository access
    - repo.inner() to access underlying git2::Repository
    - repo.create_blob() for content storage
  gotcha: |
    - Jin repository is bare, stored at ~/.jin/
    - Use repo.inner() to access git2::Repository for diff/log operations

- file: src/staging/index.rs:1-150
  why: StagingIndex for managing staged files
  pattern: |
    - StagingIndex::load().unwrap_or_else(|_| StagingIndex::new())
    - staging.entries_for_layer(layer) to filter by layer
    - staging.affected_layers() for layers with staged files
    - staging.save() to persist
  gotcha: |
    - Staging index stored at .jin/staging/index.json
    - affected_layers() returns sorted by precedence

# MUST READ - Git2-rs Documentation

- url: https://docs.rs/git2/latest/git2/struct.Diff.html
  why: Core diff operations for comparing trees and working directory
  critical: |
    - Repository::diff_tree_to_tree() for layer-to-layer comparison
    - Repository::diff_tree_to_workdir() for workspace comparison
    - Repository::diff_index_to_workdir() for staged changes
    - Diff::print() for formatting output
  section: "Methods section - diff creation and formatting"

- url: https://docs.rs/git2/latest/git2/struct.Revwalk.html
  why: Commit history traversal for log command
  critical: |
    - Repository::revwalk() creates walker
    - revwalk.push_ref() to start from ref
    - revwalk.next() for iteration
    - revwalk.sorting(Sort::TIME) for chronological order
  section: "Methods - push, sorting, iteration"

- url: https://docs.rs/git2/latest/git2/struct.Commit.html
  why: Commit metadata extraction
  critical: |
    - commit.author() for author info
    - commit.message() for commit message
    - commit.time() for timestamp
    - commit.tree() for file tree
  section: "Methods - metadata access"

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.index
  why: Repository index operations for import/export
  critical: |
    - repo.index() for Git index access
    - index.add_path() to stage files
    - index.remove_path() to unstage files
    - index.write() to persist changes
  section: "Index operations"

- url: https://git-scm.com/docs/git-ls-files
  why: Checking if file is Git-tracked (for import validation)
  critical: Use `git ls-files <path>` via Command, non-empty output = tracked
  section: "Description and examples"

# MUST READ - Existing Test Patterns

- file: tests/cli_basic.rs:1-340
  why: Integration test patterns using assert_cmd
  pattern: |
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_command() {
        Command::cargo_bin("jin")
            .unwrap()
            .arg("subcommand")
            .assert()
            .success()
            .stdout(predicate::str::contains("expected"));
    }
  gotcha: |
    - Tests check for "not yet implemented" currently
    - Update tests to check actual behavior after implementation
    - Use .failure() for error cases
    - Use .success() for happy path

# Git Diff Algorithm Resources

- url: https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/
  why: Understanding Myers diff algorithm implementation
  critical: git2 uses Myers algorithm internally, understanding helps debug diff issues
  section: "Part 1 - The algorithm explained"

- url: https://libgit2.org/docs/reference/main/diff/index.html
  why: Low-level libgit2 diff API that git2-rs wraps
  critical: |
    - diff_tree_to_tree for layer comparison
    - diff_index_to_workdir for staged changes
    - diff options for context lines, whitespace handling
  section: "API reference for diff functions"

# Repository Repair Resources

- url: https://git-scm.com/docs/git-fsck
  why: Repository integrity checking patterns
  critical: |
    - Verify object database consistency
    - Check dangling/unreachable objects
    - Validate ref integrity
  section: "fsck operations and recovery"

- url: https://git.seveas.net/repairing-and-recovering-broken-git-repositories.html
  why: Comprehensive git repair strategies
  critical: |
    - Index rebuild: rm .git/index && git reset
    - Ref recovery: git reflog expire --expire=now --all
    - Object verification: git fsck --full
  section: "Repair procedures for different corruption types"

# CLI Display Libraries

- url: https://docs.rs/comfy-table/latest/comfy_table/
  why: Table formatting for layers and list output
  critical: |
    use comfy_table::{Table, presets};
    let table = Table::new();
    table.load_preset(presets::UTF8_FULL);
  section: "Quick start and examples"

- url: https://docs.rs/ptree/latest/ptree/
  why: Tree formatting for hierarchical display
  critical: |
    use ptree::{TreeBuilder, print_tree};
    For layer hierarchy visualization
  section: "TreeBuilder API"

### Current Codebase Tree

```bash
.
├── src/
│   ├── commands/
│   │   ├── add.rs              # COMPLETE - Pattern reference
│   │   ├── context.rs          # COMPLETE - Already implemented
│   │   ├── diff.rs             # SKELETON - Implement here
│   │   ├── export.rs           # SKELETON - Implement here
│   │   ├── import_cmd.rs       # SKELETON - Implement here
│   │   ├── layers.rs           # SKELETON - Implement here
│   │   ├── list.rs             # SKELETON - Implement here
│   │   ├── log.rs              # SKELETON - Implement here
│   │   ├── repair.rs           # SKELETON - Implement here
│   │   └── mod.rs              # Export all commands
│   ├── core/
│   │   ├── layer.rs            # Layer enum with precedence
│   │   ├── config.rs           # ProjectContext
│   │   └── error.rs            # JinError enum
│   ├── git/
│   │   ├── repo.rs             # JinRepo wrapper
│   │   └── refs.rs             # Ref operations
│   ├── staging/
│   │   ├── index.rs            # StagingIndex
│   │   └── gitignore.rs        # .gitignore management
│   └── cli/
│       └── args.rs             # Clap argument structs
├── tests/
│   └── cli_basic.rs            # Integration tests
└── Cargo.toml                  # Dependencies
```

### Desired Codebase Tree with Implementations

```bash
src/commands/
├── diff.rs                      # IMPLEMENT: Tree diff, workspace diff, staged diff
├── log.rs                       # IMPLEMENT: Commit history traversal and formatting
├── context.rs                   # COMPLETE: No changes needed
├── import_cmd.rs                # IMPLEMENT: Git → Jin migration with git rm --cached
├── export.rs                    # IMPLEMENT: Jin → Git migration with git add
├── repair.rs                    # IMPLEMENT: Integrity checks and automated repair
├── layers.rs                    # IMPLEMENT: Layer composition display
└── list.rs                      # IMPLEMENT: Mode/scope/project enumeration
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2 Diff requires valid Tree objects
// You CANNOT diff against workspace-active ref directly if it doesn't exist yet
// Solution: Use diff_tree_to_workdir() for workspace comparison

// GOTCHA: git2::Revwalk requires push_ref() or push_head() before iteration
// Failure to push causes empty iterator
let mut walk = repo.revwalk()?;
walk.push_ref("refs/jin/layers/mode-base")?; // Required!
walk.set_sorting(git2::Sort::TIME)?;

// CRITICAL: Layer refs may not exist if no commits yet
// ALWAYS check Reference::lookup() returns Ok before using
match repo.inner().find_reference(&ref_path) {
    Ok(reference) => { /* ref exists */ },
    Err(_) => { /* layer has no commits yet */ }
}

// GOTCHA: git ls-files for import checking must run in project Git repo
// NOT in Jin repo at ~/.jin/
// Use std::process::Command with correct working directory

// CRITICAL: Import must be atomic: git rm --cached, jin add, .gitignore update
// If any step fails, rollback previous steps

// GOTCHA: Structured diff for JSON/YAML requires parsing
// Use serde_json::from_str() and compare Value trees
// Fall back to text diff if parsing fails

// CRITICAL: Repair operations must be idempotent
// Running repair multiple times should be safe and produce same result

// GOTCHA: Layer::all_in_precedence_order() includes ALL 9 layers
// Filter by requires_mode(), requires_scope() for active layers only
// Some layers may not exist (no commits) - handle gracefully
```

---

## Implementation Blueprint

### Implementation Strategy

Commands are grouped by complexity and dependency:

**Group 1: Display Commands (Simplest)**
- `layers` - Enumerate and display layer hierarchy
- `list` - Parse refs and group by type
- **context already complete** - No work needed

**Group 2: Git Operations (Moderate)**
- `log` - Revwalk and commit formatting
- `diff` - Tree comparison and diff formatting

**Group 3: Migration Commands (Complex)**
- `import` - Multi-step: git rm, jin add, gitignore update
- `export` - Multi-step: jin rm, git add, gitignore remove

**Group 4: Repair (Most Complex)**
- `repair` - Multiple validation and recovery operations

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: IMPLEMENT src/commands/layers.rs
  - IMPLEMENT: execute() function
  - READ: ProjectContext to get active mode/scope/project
  - ITERATE: Layer::all_in_precedence_order()
  - FILTER: Skip layers that don't apply to active context
  - FORMAT: Display with precedence numbers and storage paths
  - COUNT: Total files across all active layers (use tree walking)
  - ERROR HANDLING: NotInitialized error
  - FOLLOW pattern: src/commands/context.rs (similar display logic)
  - PLACEMENT: src/commands/layers.rs:1-150 (estimated)

Task 2: IMPLEMENT src/commands/list.rs
  - IMPLEMENT: execute() function
  - OPEN: JinRepo with open_or_create()
  - ENUMERATE: All refs under refs/jin/layers/ using Repository::references()
  - PARSE: Ref paths to extract mode/scope/project names
  - GROUP: By category (modes, scopes, projects) using HashSet for deduplication
  - SORT: Alphabetically within each category
  - DISPLAY: Formatted output with usage hints
  - ERROR HANDLING: NotInitialized, no refs found
  - FOLLOW pattern: src/git/refs.rs for ref operations
  - PLACEMENT: src/commands/list.rs:1-150 (estimated)

Task 3: IMPLEMENT src/commands/log.rs
  - IMPLEMENT: execute(args: LogArgs) function
  - PARSE: args.layer to determine target layer(s)
  - OPEN: JinRepo with open_or_create()
  - DETERMINE: Ref path using Layer::ref_path() for specified layer
  - CREATE: Revwalk with repo.inner().revwalk()
  - CONFIGURE: walk.push_ref() for target ref, walk.set_sorting(Sort::TIME)
  - ITERATE: walk.next() up to args.count commits
  - FETCH: Commit details with repo.inner().find_commit()
  - FORMAT: Display commit hash, author, date, message, file count
  - COLOR: Layer names for visual grouping
  - ERROR HANDLING: Unknown layer, layer has no commits, NotInitialized
  - FOLLOW pattern: git2-rs examples/log.rs
  - DEPENDENCIES: Requires git2::Revwalk, Commit APIs
  - PLACEMENT: src/commands/log.rs:1-200 (estimated)

Task 4: IMPLEMENT src/commands/diff.rs
  - IMPLEMENT: execute(args: DiffArgs) function
  - PARSE: args to determine diff mode (workspace vs layer, layer vs layer, staged)
  - OPEN: JinRepo with open_or_create()
  - BRANCH on args:
    - NO ARGS: diff workspace vs workspace-active (merged layers)
    - --staged: diff staged changes vs HEAD using diff_index_to_workdir()
    - layer1 layer2: diff two layers using diff_tree_to_tree()
    - layer1 only: diff workspace vs layer using diff_tree_to_workdir()
  - FETCH: Tree objects for layers using Repository::find_reference() → resolve() → peel_to_tree()
  - CREATE: Diff using appropriate Repository::diff_*() method
  - DETECT: File type (JSON/YAML/TOML) and use structured diff if applicable
  - FORMAT: Use Diff::print() with callbacks for colored output
  - EXIT CODE: 0 if no diff, 1 if differences, 2 if error
  - ERROR HANDLING: Unknown layer, layer doesn't exist, invalid combination
  - FOLLOW pattern: git2-rs examples/diff.rs
  - DEPENDENCIES: Requires git2::Diff, Tree, DiffOptions APIs
  - PLACEMENT: src/commands/diff.rs:1-300 (estimated)

Task 5: IMPLEMENT src/commands/import_cmd.rs
  - IMPLEMENT: execute(args: ImportArgs) function
  - VALIDATE: Files are Git-tracked using `git ls-files <path>` via Command
  - DETERMINE: Target layer from flags (use staging::route_to_layer)
  - FOR EACH FILE:
    1. CHECK: File is Git-tracked (error if not)
    2. REMOVE: From Git index using `git rm --cached <path>` via Command
    3. STAGE: To Jin using internal jin add logic (validate, create blob, add to staging)
    4. UPDATE: .gitignore managed block using ensure_in_managed_block()
  - TRANSACTION: If any step fails, rollback previous steps
  - SAVE: StagingIndex after all files processed
  - DISPLAY: Summary of imported files
  - FORCE MODE: Skip modification check if args.force
  - ERROR HANDLING: File not Git-tracked, file modified, import failed
  - FOLLOW pattern: src/commands/add.rs for staging logic
  - DEPENDENCIES: Requires std::process::Command, StagingIndex, ensure_in_managed_block
  - PLACEMENT: src/commands/import_cmd.rs:1-250 (estimated)

Task 6: IMPLEMENT src/commands/export.rs
  - IMPLEMENT: execute(args: ExportArgs) function
  - VALIDATE: Files are Jin-tracked (check StagingIndex and layer commits)
  - FOR EACH FILE:
    1. CHECK: File is Jin-tracked (error if not)
    2. REMOVE: From Jin staging using StagingIndex::remove()
    3. ADD: To Git index using `git add <path>` via Command
    4. REMOVE: From .gitignore managed block (parse, remove line, write back)
  - TRANSACTION: If any step fails, rollback previous steps
  - SAVE: StagingIndex after all files processed
  - DISPLAY: Summary of exported files
  - WARNING: If Git working directory has uncommitted changes
  - ERROR HANDLING: File not Jin-tracked, file doesn't exist, export failed
  - FOLLOW pattern: src/commands/add.rs for validation, inverse of import
  - DEPENDENCIES: Requires std::process::Command, StagingIndex, gitignore parsing
  - PLACEMENT: src/commands/export.rs:1-250 (estimated)

Task 7: IMPLEMENT src/commands/repair.rs
  - IMPLEMENT: execute(args: RepairArgs) function
  - CHECK 1: Repository structure - verify ~/.jin/ is valid bare repo
  - CHECK 2: Layer refs - ensure refs/jin/layers/* point to valid commits
  - CHECK 3: Staging index - validate .jin/staging/index.json is parseable JSON
  - CHECK 4: .jinmap - verify .jin/.jinmap exists and is valid YAML
  - CHECK 5: Workspace metadata - check .jin/workspace/ tracking files
  - FOR EACH ISSUE:
    - DRY RUN: Report issue without fixing
    - REPAIR: Auto-fix (rebuild index, restore refs from reflog, recreate metadata)
  - REBUILD OPERATIONS:
    - Staging index: Parse from Git objects, reconstruct StagingIndex
    - Refs: Recover from reflog using Repository::reflog()
    - Metadata: Regenerate from current layer state
  - DISPLAY: Checklist with ✓/✗ for each validation
  - SUMMARY: Count of issues found and fixed
  - ERROR HANDLING: Fatal corruption (report manual recovery steps)
  - FOLLOW pattern: git fsck approach - check, report, fix
  - DEPENDENCIES: Requires git2::Reflog, Repository verification APIs
  - PLACEMENT: src/commands/repair.rs:1-400 (estimated)

Task 8: UPDATE tests/cli_basic.rs
  - UPDATE: test_diff_subcommand() to check actual diff output
  - UPDATE: test_log_subcommand() to check log formatting
  - UPDATE: test_context_subcommand() to verify actual context display
  - UPDATE: test_import_subcommand() to set up Git repo and test import
  - UPDATE: test_export_subcommand() to set up Jin files and test export
  - UPDATE: test_repair_subcommand() to corrupt and repair test repo
  - UPDATE: test_layers_subcommand() to verify layer display
  - UPDATE: test_list_subcommand() to check enumeration output
  - ADD: Integration tests for edge cases (no commits, corrupt state, etc.)
  - REMOVE: "not yet implemented" assertions
  - FOLLOW pattern: existing tests in cli_basic.rs
  - PLACEMENT: tests/cli_basic.rs (modify existing tests)

Task 9: ADD unit tests to each command file
  - ADD: #[cfg(test)] mod tests to each new implementation
  - TEST: Happy path for each command
  - TEST: Error conditions (NotInitialized, invalid input, etc.)
  - TEST: Edge cases (empty layers, no commits, corrupt data)
  - FOLLOW pattern: src/commands/add.rs tests (lines 206-326)
  - USE: tempfile::TempDir for isolated test environments
  - MOCK: Git operations where necessary
  - PLACEMENT: At end of each command file
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Layer iteration and filtering (for layers command)
use crate::core::{Layer, ProjectContext};

let context = ProjectContext::load()?;
let all_layers = Layer::all_in_precedence_order();

for layer in &all_layers {
    // Skip layers that don't apply to current context
    if layer.requires_mode() && context.mode.is_none() {
        continue;
    }
    if layer.requires_scope() && context.scope.is_none() {
        continue;
    }

    println!("{:2}. {:<20} [{}]",
        layer.precedence(),
        layer.to_string(),
        layer.storage_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref()
        )
    );
}

// Pattern 2: Commit history traversal (for log command)
use git2::{Sort, Repository};

let repo = jin_repo.inner();
let ref_path = layer.ref_path(mode, scope, project);

// Check if ref exists
let reference = match repo.find_reference(&ref_path) {
    Ok(r) => r,
    Err(_) => {
        println!("No commits yet for layer: {}", layer);
        return Ok(());
    }
};

let mut revwalk = repo.revwalk()?;
revwalk.push_ref(&ref_path)?;
revwalk.set_sorting(Sort::TIME)?;

for (i, oid) in revwalk.enumerate() {
    if i >= count {
        break;
    }
    let oid = oid?;
    let commit = repo.find_commit(oid)?;

    println!("commit {} ({})",
        oid.to_string()[..7].to_string(),  // Short hash
        layer
    );
    println!("Author: {} <{}>",
        commit.author().name().unwrap_or("unknown"),
        commit.author().email().unwrap_or("unknown")
    );

    // Format timestamp
    let time = commit.time();
    let timestamp = chrono::NaiveDateTime::from_timestamp(time.seconds(), 0);
    println!("Date:   {}", timestamp.format("%Y-%m-%d %H:%M:%S"));

    if let Some(msg) = commit.message() {
        println!("\n    {}\n", msg);
    }
}

// Pattern 3: Tree diff comparison (for diff command)
use git2::{Diff, DiffOptions, Repository};

fn diff_layers(repo: &Repository, layer1: Layer, layer2: Layer,
               mode: Option<&str>, scope: Option<&str>, project: Option<&str>)
    -> Result<Diff>
{
    let ref1 = layer1.ref_path(mode, scope, project);
    let ref2 = layer2.ref_path(mode, scope, project);

    let tree1 = repo.find_reference(&ref1)?
        .resolve()?
        .peel_to_tree()?;
    let tree2 = repo.find_reference(&ref2)?
        .resolve()?
        .peel_to_tree()?;

    let mut opts = DiffOptions::new();
    opts.context_lines(3);

    repo.diff_tree_to_tree(Some(&tree1), Some(&tree2), Some(&mut opts))
}

// Then format with colors
diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
    match line.origin() {
        '+' => print!("\x1b[32m+{}\x1b[0m", std::str::from_utf8(line.content()).unwrap()),
        '-' => print!("\x1b[31m-{}\x1b[0m", std::str::from_utf8(line.content()).unwrap()),
        _ => print!(" {}", std::str::from_utf8(line.content()).unwrap()),
    }
    true
})?;

// Pattern 4: Git ls-files for import validation
use std::process::Command;

fn is_git_tracked(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--")
        .arg(path)
        .output()?;

    Ok(!output.stdout.is_empty())
}

// Pattern 5: Atomic import transaction
fn import_file(path: &Path, layer: Layer) -> Result<()> {
    // Step 1: Verify Git-tracked
    if !is_git_tracked(path)? {
        return Err(JinError::Other(format!(
            "{} is not Git-tracked. Use 'jin add' instead.",
            path.display()
        )));
    }

    // Step 2: Remove from Git (cached only, keep in workspace)
    Command::new("git")
        .arg("rm")
        .arg("--cached")
        .arg(path)
        .status()
        .map_err(|e| JinError::Other(format!("git rm failed: {}", e)))?;

    // Step 3: Add to Jin (if this fails, rollback git rm)
    if let Err(e) = stage_to_jin(path, layer) {
        // Rollback: re-add to Git
        let _ = Command::new("git")
            .arg("add")
            .arg(path)
            .status();
        return Err(e);
    }

    // Step 4: Update .gitignore
    ensure_in_managed_block(path)?;

    Ok(())
}

// Pattern 6: Repository repair with validation
fn repair_repository(dry_run: bool) -> Result<Vec<String>> {
    let mut issues_fixed = Vec::new();

    // Check 1: Repository structure
    match JinRepo::open() {
        Ok(_) => println!("✓ Repository structure valid"),
        Err(_) => {
            println!("✗ Repository corrupted");
            if !dry_run {
                JinRepo::create()?;
                issues_fixed.push("Repository recreated".to_string());
            }
        }
    }

    // Check 2: Staging index
    match StagingIndex::load() {
        Ok(_) => println!("✓ Staging index valid"),
        Err(_) => {
            println!("✗ Staging index corrupted - REPAIRING");
            if !dry_run {
                let new_index = StagingIndex::new();
                new_index.save()?;
                issues_fixed.push("Staging index rebuilt".to_string());
            }
        }
    }

    // Additional checks...

    Ok(issues_fixed)
}
```

### Integration Points

```yaml
DEPENDENCIES:
  - All commands require JinRepo (P1.M2)
  - All commands require Layer enum (P1.M3)
  - import/export require .gitignore management (P3.M1)
  - diff requires git2::Diff APIs
  - log requires git2::Revwalk APIs
  - repair requires git2::Reflog APIs

CONFIG:
  - No new config needed
  - Uses existing .jin/context for active context
  - Uses existing .jin/staging/index.json for staged files

EXTERNAL COMMANDS:
  - import uses: `git ls-files`, `git rm --cached`
  - export uses: `git add`
  - All must handle Command failures gracefully
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation/modification
cargo check                              # Quick syntax check
cargo clippy -- -D warnings              # Strict linting
cargo fmt --check                        # Format verification

# Expected: Zero errors, zero warnings
# If errors: Read output, fix issues, re-run
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test individual commands as implemented
cargo test commands::diff::tests         # Diff command tests
cargo test commands::log::tests          # Log command tests
cargo test commands::import_cmd::tests   # Import command tests
cargo test commands::export::tests       # Export command tests
cargo test commands::repair::tests       # Repair command tests
cargo test commands::layers::tests       # Layers command tests
cargo test commands::list::tests         # List command tests

# Run all unit tests
cargo test --lib

# Expected: All tests pass
# If failing: Debug root cause, fix implementation, re-run
```

### Level 3: Integration Testing (System Validation)

```bash
# Integration tests in tests/cli_basic.rs
cargo test --test cli_basic test_diff_subcommand
cargo test --test cli_basic test_log_subcommand
cargo test --test cli_basic test_context_subcommand
cargo test --test cli_basic test_import_subcommand
cargo test --test cli_basic test_export_subcommand
cargo test --test cli_basic test_repair_subcommand
cargo test --test cli_basic test_layers_subcommand
cargo test --test cli_basic test_list_subcommand

# Run all CLI tests
cargo test --test cli_basic

# Expected: All integration tests pass
# If failing: Check command output, verify behavior matches spec
```

### Level 4: Manual Validation & Edge Cases

```bash
# Build release binary
cargo build --release

# Test diff command
./target/release/jin init
./target/release/jin add test.json
./target/release/jin commit -m "Initial"
./target/release/jin add test.json  # Modify file first
./target/release/jin diff --staged  # Should show staged changes

# Test log command
./target/release/jin log --layer=project-base --count=5

# Test context command
./target/release/jin context

# Test import command
echo "test" > git-file.txt
git add git-file.txt
git commit -m "Add test file"
./target/release/jin import git-file.txt
git ls-files git-file.txt  # Should be empty (no longer Git-tracked)
cat .gitignore  # Should contain git-file.txt in managed block

# Test export command
./target/release/jin export git-file.txt
git ls-files git-file.txt  # Should show file (now Git-tracked)
cat .gitignore  # Should NOT contain git-file.txt

# Test repair command
rm .jin/staging/index.json  # Corrupt staging
./target/release/jin repair --dry-run  # Should report issue
./target/release/jin repair  # Should fix issue

# Test layers command
./target/release/jin mode use testmode
./target/release/jin scope use testscope
./target/release/jin layers  # Should show layer composition

# Test list command
./target/release/jin list  # Should enumerate modes/scopes/projects

# Expected: All commands produce correct, well-formatted output
# Expected: Error messages are clear and actionable
# Expected: Exit codes are correct (0 = success, non-zero = error)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo test` passes with zero failures
- [ ] `cargo clippy` reports zero warnings
- [ ] `cargo fmt --check` passes
- [ ] All commands have unit tests
- [ ] All commands have integration tests in `tests/cli_basic.rs`

### Feature Validation

- [ ] `jin diff` compares layers correctly
- [ ] `jin diff --staged` shows staged changes
- [ ] `jin log` displays commit history
- [ ] `jin log --layer=X` filters by layer
- [ ] `jin context` displays active context (already works)
- [ ] `jin import` moves Git files to Jin
- [ ] `jin import` updates .gitignore
- [ ] `jin export` moves Jin files to Git
- [ ] `jin export` removes from .gitignore
- [ ] `jin repair` detects corruption
- [ ] `jin repair --dry-run` reports without fixing
- [ ] `jin repair` fixes issues automatically
- [ ] `jin layers` shows layer composition
- [ ] `jin list` enumerates modes/scopes/projects

### Code Quality Validation

- [ ] Follows patterns from `src/commands/add.rs`
- [ ] Uses JinError enum consistently
- [ ] Proper error handling (no unwrap in non-test code)
- [ ] Clear variable and function names
- [ ] Comments explain non-obvious logic
- [ ] No code duplication across commands

### User Experience Validation

- [ ] All commands have clear output
- [ ] Error messages guide toward resolution
- [ ] Help text explains command purpose
- [ ] Examples in help text are accurate
- [ ] Exit codes follow Unix conventions (0 = success)
- [ ] Commands feel responsive (no unnecessary delays)

---

## Anti-Patterns to Avoid

- ❌ Don't use `unwrap()` outside of tests - use `?` operator or match for error handling
- ❌ Don't assume refs exist - always check with `find_reference()` and handle Err case
- ❌ Don't parse structured data without fallback - if JSON parse fails, use text diff
- ❌ Don't modify Git working directory without validation - check for uncommitted changes
- ❌ Don't perform partial imports/exports - ensure atomic operations with rollback
- ❌ Don't show raw Git errors to users - wrap with JinError and provide context
- ❌ Don't hardcode paths - use Layer methods for ref_path and storage_path
- ❌ Don't iterate all commits without limit - respect --count argument
- ❌ Don't skip context validation - always check active mode/scope before layer operations
- ❌ Don't ignore DRY principle - extract common patterns (repo opening, context loading) into helpers

---

## Confidence Score

**9/10** - High confidence for one-pass implementation success

**Reasoning:**
- ✅ Complete command specifications with examples
- ✅ Multiple reference implementations (add.rs, context.rs)
- ✅ Clear git2-rs API documentation
- ✅ Existing test patterns to follow
- ✅ Well-defined error handling approach
- ✅ Atomic transaction patterns established
- ⚠️ Structured diff for JSON/YAML adds complexity (hence 9/10 not 10/10)
- ✅ Repair command has clear validation checklist
- ✅ Import/export patterns well-documented

**Success Enablers:**
1. Context command already complete - one less command to implement
2. Git2-rs has excellent examples and documentation
3. Add command provides 326-line reference implementation
4. Layer system is well-designed with helper methods
5. Testing infrastructure already in place
6. Error handling patterns established

**Risk Mitigation:**
- Structured diff complexity → Implement text diff first, add structured diff as enhancement
- Import/export atomicity → Follow established pattern from add command with rollback
- Repair complexity → Implement checks incrementally, start with simplest validations

This PRP provides all necessary context for successful one-pass implementation of all 8 utility commands.
