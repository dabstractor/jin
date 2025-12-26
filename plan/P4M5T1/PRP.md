# PRP: Diff Command (P4.M5.T1)

---

## Goal

**Feature Goal**: Implement the `jin diff` command that displays differences between Jin layers or between the workspace and merged layers, following Git's diff UX patterns while respecting Jin's multi-layer architecture.

**Deliverable**: A fully functional `jin diff` command that:
- Compares two specified layers and shows their differences
- Compares workspace files to their merged layer result (default)
- Compares staged files to their layer's committed version (--staged)
- Supports unified diff format with colored output
- Handles structured data (JSON, YAML, TOML) with semantic diffing
- Handles text files with line-based diffing

**Success Definition**:
- Running `jin diff` shows differences between workspace and merged result
- Running `jin diff layer1 layer2` shows differences between two layers
- Running `jin diff --staged` shows differences between staged files and committed versions
- Output follows Git's unified diff format conventions
- Proper exit codes (0 = no differences, 1 = differences found, 2 = error)
- Integration with existing similar crate for diff computation

## User Persona

**Target User**: Developer using Jin for multi-layer configuration management who needs to understand what changes exist between layers or between their workspace and the merged configuration.

**Use Case**: Developer wants to see what would change if they applied layers, or understand the difference between their workspace edits and what's committed to layers.

**User Journey**:
1. Developer has made workspace edits or wants to compare layers
2. Developer runs `jin diff` to see workspace vs merged differences
3. Developer runs `jin diff mode/claude project/myproject` to compare layers
4. Developer runs `jin diff --staged` to see what's staged vs committed
5. Output shows clearly formatted, colored diff with file paths and line changes

**Pain Points Addressed**:
- Without diff command, developers must manually compare files or use external tools
- No visibility into what changes exist between layers
- Hard to understand what would be applied by `jin apply`
- No way to see staged vs committed differences

## Why

- **Business value**: Provides critical visibility into configuration changes across layers
- **Integration**: Completes the inspection commands (status, diff, log, layers, list)
- **Problems solved**:
  - Developers need to see what changes exist before committing or applying
  - Understanding layer differences helps debug configuration issues
  - Staged vs committed comparison enables safe workflow
  - No manual file comparison needed

## What

The `jin diff` command displays differences between layers or between workspace and merged result.

### Command Interface

```bash
# Default: workspace vs merged result
jin diff

# Compare two specific layers
jin diff mode/claude project/myproject

# Compare staged files to committed versions
jin diff --staged

# Show summary statistics only
jin diff --stat

# Show specific file only
jin diff -- path/to/config.json

# Compare with context lines
jin diff -U5
```

### Comparison Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| No args | Workspace vs merged result | See what would change on apply |
| layer1 layer2 | Layer vs layer comparison | Understand layer differences |
| --staged | Staged vs committed | See what's staged for commit |
| --stat | Summary statistics | Quick overview of changes |

### Output Format

Follows Git's unified diff format:
```
--- a/config.json	(mode/claude)
+++ b/config.json	(project/myproject)
@@ -1,5 +1,5 @@
 {
   "database": {
-    "host": "localhost",
+    "host": "db.example.com",
     "port": 5432
   }
 }
```

### Success Criteria

- [ ] Shows differences between workspace and merged result (default)
- [ ] Shows differences between two specified layers
- [ ] Shows differences between staged and committed files (--staged)
- [ ] Supports unified diff format output
- [ ] Supports colored output (green for additions, red for deletions)
- [ ] Returns proper exit codes (0 = no diff, 1 = diff found, 2 = error)
- [ ] Handles structured data formats (JSON, YAML, TOML)
- [ ] Handles text files with line-based diffing
- [ ] Supports --stat for summary statistics
- [ ] Supports path filtering

## All Needed Context

### Context Completeness Check

**Passes "No Prior Knowledge" test**: This PRP provides complete file paths, exact code patterns, existing infrastructure APIs (similar crate, LayerMerge, JinRepo), CLI integration patterns, testing patterns, and external research references. An implementer needs only this PRP and codebase access.

### Documentation & References

```yaml
# MUST READ - Command implementation pattern
- file: src/commands/status.rs
  why: Reference for command structure, output formatting, JinRepo usage
  pattern: execute() -> load context -> display -> return Result
  gotcha: Check for Jin initialization first, handle missing layers gracefully

- file: src/commands/reset.rs
  why: Reference for layer targeting logic, project name detection
  pattern: detect_project_name(), determine layer from flags/context
  gotcha: Use Layer::from_flags() for explicit routing, context for implicit

- file: src/commands/apply.rs
  why: Reference for LayerMerge usage, workspace operations
  pattern: LayerMerge::new().with_mode().with_scope().merge_all()
  gotcha: Layer merge returns IndexMap<String, MergeValue>

# MUST READ - Merge and diff infrastructure
- file: src/merge/layer.rs
  why: LayerMerge orchestrator for comparing layers
  pattern: LayerMerge::new(&repo, project).with_mode().with_scope().merge_subset()
  gotcha: merge_subset() requires versioned layers only (no UserLocal/WorkspaceActive)

- file: src/merge/text.rs
  why: 3-way merge with similar crate, can be adapted for diff display
  pattern: TextMerge::three_way_merge(), similar::TextDiff usage
  gotcha: Already uses similar crate with Algorithm::Myers

- file: src/merge/value.rs
  why: MergeValue for structured data comparison
  pattern: MergeValue::from_json/yaml/toml(), merge operations
  gotcha: For diff, compare individual fields rather than merging

# MUST READ - CLI definition and routing
- file: src/cli/args.rs:290-304
  why: DiffCommand struct definition (already exists)
  pattern: layer1: Option<String>, layer2: Option<String>, staged: bool
  gotcha: Already wired in main.rs as placeholder - just needs implementation

- file: src/main.rs:168-171
  why: Current diff command routing (placeholder)
  pattern: Commands::Diff(_) needs to call diff_execute()
  action: Replace placeholder with actual command handler call

# MUST READ - Layer and project infrastructure
- file: src/core/layer.rs
  why: Layer enum with Display impl, from_flags() routing, git_ref()
  pattern: Layer::from_flags(mode, scope, project, global) -> Option<Layer>
  gotcha: Parse layer strings like "mode/claude" back to Layer enum

- file: src/git/repo.rs
  why: JinRepo for reading layer contents, getting layer refs
  pattern: repo.get_layer_ref(&layer) -> Result<Option<Reference>>
  gotcha: Layer may not exist (returns Ok(None))

- file: src/staging/index.rs
  why: StagingIndex for --staged mode comparison
  pattern: StagingIndex::load_from_disk(), all_entries(), entries_by_layer()
  gotcha: StagedEntry has content() method for staged file content

# MUST READ - Project context
- file: src/core/config.rs:164-188
  why: ProjectContext structure for mode/scope, load method
  pattern: ProjectContext::load(&workspace_root) -> Result<ProjectContext>
  gotcha: context.mode and context.scope are Option<String>

# EXTERNAL REFERENCES
- url: https://git-scm.com/docs/git-diff
  why: Git diff output format specification and UX patterns
  critical: Unified diff format, exit codes, --staged flag behavior

- url: https://docs.rs/similar/latest/similar/
  why: Similar crate documentation for diff operations
  critical: TextDiff::from_lines(), unified_diff() output formatting

- url: https://docs.rs/console/latest/console/
  why: Cross-platform colored terminal output
  critical: Style::new().red()/.green() for diff coloring

- docfile: git_diff_ux_patterns.md (in plan/P4M5T1/research/)
  why: Research on git diff UX patterns and best practices
  section: Unified Diff Format Display, Exit Codes, Error Handling

- docfile: rust_diff_libraries.md (in plan/P4M5T1/research/)
  why: Research on Rust diff libraries and implementation examples
  section: similar crate examples, console crate usage
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   ├── cli/
│   │   ├── args.rs              # DiffCommand at lines 290-304
│   │   └── mod.rs
│   ├── commands/
│   │   ├── mod.rs               # ADD: pub mod diff; pub use diff::execute as diff_execute;
│   │   ├── status.rs            # Reference: command structure, output format
│   │   ├── reset.rs             # Reference: layer targeting, project detection
│   │   ├── apply.rs             # Reference: LayerMerge usage
│   │   └── diff.rs              # CREATE THIS FILE
│   ├── core/
│   │   ├── config.rs            # ProjectContext for active mode/scope
│   │   ├── error.rs             # JinError variants, Result type
│   │   └── layer.rs             # Layer enum, Display impl, from_flags()
│   ├── git/
│   │   └── repo.rs              # JinRepo for layer operations
│   ├── merge/
│   │   ├── layer.rs             # LayerMerge orchestrator
│   │   ├── text.rs              # TextMerge with similar crate (reference)
│   │   └── value.rs             # MergeValue for structured data
│   ├── staging/
│   │   ├── index.rs             # StagingIndex for --staged mode
│   │   └── entry.rs             # StagedEntry with content access
│   └── main.rs                  # Lines 168-171: DiffCommand dispatcher
└── Cargo.toml                   # Already has similar crate dependency
```

### Desired Codebase Tree (Files to Add)

```bash
# NEW FILE TO CREATE
├── src/commands/diff.rs         # Main diff command implementation
│   ├── execute()                # Entry point
│   ├── parse_layer_spec()       # Parse "mode/claude" strings to Layer
│   ├── diff_layers()            # Compare two layers
│   ├── diff_workspace()         # Compare workspace to merged
│   ├── diff_staged()            # Compare staged to committed
│   ├── format_unified_diff()    # Generate unified diff format
│   ├── format_structured_diff() # Generate JSON/YAML/TOML semantic diff
│   ├── format_text_diff()       # Generate line-based text diff
│   └── tests module             # Unit tests

# MODIFY
├── src/commands/mod.rs          # ADD: pub mod diff; pub use diff::execute as diff_execute;
├── src/main.rs                   # UPDATE: Commands::Diff(_)-> call diff_execute(&cmd)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Layer enum parsing - need to parse strings like "mode/claude" back to Layer
// No built-in FromStr impl - must implement parse_layer_spec() function
// Format: "global", "mode/<name>", "scope/<name>", "project/<name>"
// Complex: "mode/<m>/scope/<s>/project/<p>", "mode/<m>/scope/<s>", "mode/<m>/project/<p>"

// CRITICAL: similar crate is already a dependency (used in src/merge/text.rs)
// Use: similar::{TextDiff, ChangeTag, Algorithm}
// Pattern: TextDiff::configure().algorithm(Algorithm::Myers).from_lines(old, new)

// CRITICAL: console crate NOT in dependencies - need to add for colored output
// Add to Cargo.toml: console = "0.15"
// Alternative: use colored crate (check if already available)

// CRITICAL: StagingIndex stores absolute paths in StagedEntry
// StagedEntry::path() returns absolute PathBuf
// For diff, need to show relative paths for clean output

// CRITICAL: LayerMerge::merge_subset() requires versioned layers only
// UserLocal and WorkspaceActive cause errors
// Use layer.is_versioned() check before including in subset

// CRITICAL: LayerMerge::read_layer_files() returns HashMap<String, Vec<u8>>
// Key: relative file path, Value: file content as bytes
// Must convert bytes to string for diff operations

// CRITICAL: Workspace files are in .jin/workspace/ directory
// For workspace vs merged diff, read from .jin/workspace/
// For staged diff, read from StagingIndex entries

// CRITICAL: detect_project_name() helper pattern (from add.rs, reset.rs, apply.rs)
// Try git remote origin, fallback to directory name
// Copy exact implementation from existing commands

// CRITICAL: JinRepo::get_layer_ref() returns Result<Option<Reference>>
// None means layer doesn't exist in Git yet (not an error)
// Handle gracefully by showing "Layer X is empty or doesn't exist"

// CRITICAL: File format detection for structured vs text diff
// Use FileFormat::from_path() from merge/layer.rs
// Structured formats (Json, Yaml, Toml, Ini) get semantic diff
// Text/Unknown get line-based diff

// CRITICAL: Exit code convention (like git diff)
// 0 = no differences found
// 1 = differences found (script-friendly)
// 2 = error occurred

// CRITICAL: Layer::Display impl produces paths like "mode/claude", "project/myproject"
// Use this for diff headers, not git_ref() which includes "refs/jin/layers/" prefix

// CRITICAL: When comparing workspace to merged, need to handle files only in workspace
// Files in workspace but not in merged result are "additions" (show with + prefix)
// Files in merged but not in workspace are "deletions" (show with - prefix)

// CRITICAL: For --staged mode, compare StagedEntry.content to committed content
// Need to read committed version from layer's Git tree
// Use JinRepo::read_layer_files() to get committed content
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - using existing types:
- `DiffCommand` (from `src/cli/args.rs`) - CLI arguments
- `ProjectContext` (from `src/core/config.rs`) - Active mode/scope
- `Layer` (from `src/core/layer.rs`) - Layer targeting and display
- `LayerMerge` (from `src/merge/layer.rs`) - Layer content comparison
- `MergeValue` (from `src/merge/value.rs`) - Structured data handling
- `TextDiff` (from `similar` crate) - Diff computation

Local helper types:
```rust
/// Represents which files are added, modified, or deleted in a diff
struct FileDiff {
    path: String,
    status: DiffStatus,
    old_content: Option<String>,
    new_content: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum DiffStatus {
    Added,      // File only in target (new)
    Deleted,    // File only in source (removed)
    Modified,   // File in both with differences
    Unchanged,  // File in both with same content
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: ADD console crate to Cargo.toml dependencies
  - ADD: console = "0.15" to dependencies
  - WHY: Cross-platform colored terminal output for diff display
  - LOCATION: Cargo.toml dependencies section
  - ALTERNATIVE: Use colored crate if preferred

Task 2: CREATE src/commands/diff.rs with module structure
  - IMPLEMENT: Module skeleton with use statements
  - INCLUDE: crate::cli::args::DiffCommand, crate::core::error::{JinError, Result}
  - INCLUDE: crate::core::{config::ProjectContext, layer::Layer}
  - INCLUDE: crate::git::JinRepo, crate::merge::layer::{LayerMerge, FileFormat}
  - INCLUDE: crate::merge::value::MergeValue, crate::staging::{index::StagingIndex, entry::StagedEntry}
  - INCLUDE: std::collections::HashMap, similar::{TextDiff, ChangeTag, Algorithm}
  - INCLUDE: console::Style
  - PATTERN: Follow src/commands/status.rs structure

Task 3: IMPLEMENT parse_layer_spec() helper function
  - SIGNATURE: fn parse_layer_spec(spec: &str, project: &str) -> Result<Layer>
  - LOGIC: Parse strings like "mode/claude", "project/myapp" to Layer enum
  - SUPPORTED FORMATS:
    - "global" -> Layer::GlobalBase
    - "mode/<name>" -> Layer::ModeBase
    - "scope/<name>" -> Layer::ScopeBase
    - "project/<name>" -> Layer::ProjectBase
    - "mode/<m>/scope/<s>" -> Layer::ModeScope
    - "mode/<m>/project/<p>" -> Layer::ModeProject
    - "mode/<m>/scope/<s>/project/<p>" -> Layer::ModeScopeProject
  - ERROR: Return JinError::Message for invalid format
  - PATTERN: Split on '/', match component count, construct Layer

Task 4: IMPLEMENT diff_files_in_layer() helper function
  - SIGNATURE: fn diff_files_in_layer(old_files: &HashMap<String, Vec<u8>>, new_files: &HashMap<String, Vec<u8>>) -> Vec<FileDiff>
  - LOGIC:
    - Collect all unique paths from both HashMaps
    - For each path, determine Added/Deleted/Modified/Unchanged
    - Store content as Option<String> for diff display
  - PATTERN: Use HashSet to collect all paths, then classify

Task 5: IMPLEMENT format_unified_diff() helper function
  - SIGNATURE: fn format_unified_diff(file_diff: &FileDiff, source_layer: &Layer, target_layer: &Layer) -> String
  - LOGIC:
    - Generate unified diff header with source/target layer paths
    - Use similar::TextDiff for line-based diff
    - Apply colors using console::Style
  - PATTERN:
    let header = format!("--- a/{}\\t+++ b/{}\\t", path, source_layer, target_layer);
    let diff = TextDiff::from_lines(old_content, new_content);
    // Format output with colors

Task 6: IMPLEMENT format_structured_diff() helper function
  - SIGNATURE: fn format_structured_diff(file_diff: &FileDiff, format: &FileFormat) -> Result<String>
  - LOGIC:
    - Parse both contents as MergeValue using appropriate parser
    - Compute field-by-field comparison
    - Display semantic changes (added/modified/deleted keys)
  - PATTERN:
    let old_val = MergeValue::from_json(old_content)?;
    let new_val = MergeValue::from_json(new_content)?;
    // Compare and display field-level changes

Task 7: IMPLEMENT diff_layers() function
  - SIGNATURE: fn diff_layers(repo: &JinRepo, layer1: &Layer, layer2: &Layer, project: &str) -> Result<i32>
  - LOGIC:
    - Read files from both layers using LayerMerge::read_layer_files()
    - Compute file differences using diff_files_in_layer()
    - Format and print diffs for each changed file
    - Return exit code (0 = no diff, 1 = diff found)
  - PATTERN: Follow status.rs display helper pattern

Task 8: IMPLEMENT diff_workspace() function
  - SIGNATURE: fn diff_workspace(repo: &JinRepo, project: &str, context: &ProjectContext) -> Result<i32>
  - LOGIC:
    - Read merged result using LayerMerge::new(repo, project).with_mode().with_scope().merge_all()
    - Read workspace files from .jin/workspace/ directory
    - Compare and display differences
    - Handle workspace-only files (additions) and merged-only files (deletions)
  - PATTERN: Use std::fs::read_dir() for workspace files

Task 9: IMPLEMENT diff_staged() function
  - SIGNATURE: fn diff_staged(repo: &JinRepo, project: &str) -> Result<i32>
  - LOGIC:
    - Load StagingIndex to get staged entries
    - For each staged entry, read committed content from layer
    - Compare staged content to committed content
    - Display diff for each changed staged file
  - PATTERN: Use staging.entries_by_layer() to group by layer

Task 10: IMPLEMENT detect_project_name() helper function
  - SIGNATURE: fn detect_project_name(workspace_root: &Path) -> Result<String>
  - LOGIC: Copy exact implementation from src/commands/add.rs or src/commands/apply.rs
  - REFERENCE: src/commands/apply.rs:100-133
  - PATTERN: Try git remote origin, fallback to directory name

Task 11: IMPLEMENT execute() main function
  - SIGNATURE: pub fn execute(cmd: &DiffCommand) -> Result<()>
  - STEP 1: Get workspace_root (std::env::current_dir()?)
  - STEP 2: Load ProjectContext from workspace_root
  - STEP 3: Detect project name using detect_project_name()
  - STEP 4: Open JinRepo using JinRepo::open_or_create()
  - STEP 5: Validate Git repository exists
  - STEP 6: Determine diff mode based on arguments
  - STEP 7: Execute appropriate diff function and get exit code
  - STEP 8: Set exit code (not via return value - use process exit)
  - ERROR: Propagate all errors with context
  - PATTERN: Follow src/commands/status.rs structure

Task 12: ADD exit code handling
  - NOTE: execute() returns Result<()> but diff needs to return exit code 1 for differences
  - SOLUTION: Use std::process::exit(code) at end of execute()
  - PATTERN:
    let diff_result = if cmd.staged { diff_staged(...) } else { ... };
    std::process::exit(diff_result?);

Task 13: ADD src/commands/diff.rs to src/commands/mod.rs
  - ADD: pub mod diff;
  - EXPORT: pub use diff::execute as diff_execute;
  - PATTERN: Follow existing mod.rs structure (after status import)

Task 14: UPDATE command dispatcher in src/main.rs
  - MODIFY: Commands::Diff(cmd) branch (lines 168-171)
  - CHANGE: From placeholder to:
    match commands::diff_execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
  - PATTERN: Follow other command dispatchers (Status, Apply, Reset)
  - NOTE: execute() handles its own exit() internally for diff=found case

Task 15: CREATE comprehensive unit tests in src/commands/diff.rs tests module
  - USE: tempfile::TempDir for isolated test directories
  - USE: DirGuard pattern for directory restoration
  - TEST: test_parse_layer_spec_valid_formats() - All supported layer specs parse correctly
  - TEST: test_parse_layer_spec_invalid_format() - Invalid format returns error
  - TEST: test_diff_files_in_layer_added() - New file detected as added
  - TEST: test_diff_files_in_layer_deleted() - Removed file detected as deleted
  - TEST: test_diff_files_in_layer_modified() - Changed file detected as modified
  - TEST: test_diff_layers_no_difference() - Returns exit code 0
  - TEST: test_diff_layers_has_difference() - Returns exit code 1
  - TEST: test_format_unified_diff() - Correct unified format output
  - TEST: test_diff_workspace_empty() - Handles empty workspace
  - TEST: test_diff_staged_no_files() - Handles no staged files
  - TEST: test_diff_layer_nonexistent() - Handles non-existent layer gracefully
  - PATTERN: Follow src/commands/status.rs test structure

Task 16: RUN validation and fix any issues
  - COMPILE: cargo build
  - TEST: cargo test --lib diff
  - MANUAL: cargo run -- diff (in test project)
  - MANUAL: cargo run -- diff mode/claude project/myproject
  - MANUAL: cargo run -- diff --staged
  - VERIFY: Output format matches specification
```

### Implementation Patterns & Key Details

```rust
// ===== MODULE STRUCTURE =====
//! Diff command implementation.
//!
//! This module implements the `jin diff` command that displays differences
//! between Jin layers or between workspace and merged result.

use crate::cli::args::DiffCommand;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use crate::merge::layer::{FileFormat, LayerMerge};
use crate::merge::value::MergeValue;
use crate::staging::{entry::StagedEntry, index::StagingIndex};
use console::Style;
use similar::{Algorithm, ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ===== LOCAL TYPES =====
#[derive(Debug, Clone)]
struct FileDiff {
    path: String,
    status: DiffStatus,
    old_content: Option<String>,
    new_content: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum DiffStatus {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

// ===== COLOR STYLES =====
static RED: Style = Style::new().red();
static GREEN: Style = Style::new().green();
static CYAN: Style = Style::new().cyan();
static DIM: Style = Style::new().dim();

// ===== LAYER PARSING =====
fn parse_layer_spec(spec: &str, project: &str) -> Result<Layer> {
    let parts: Vec<&str> = spec.split('/').collect();

    match parts.as_slice() {
        ["global"] => Ok(Layer::GlobalBase),
        ["mode", name] => Ok(Layer::ModeBase { mode: name.to_string() }),
        ["scope", name] => Ok(Layer::ScopeBase { scope: name.to_string() }),
        ["project", name] => Ok(Layer::ProjectBase { project: name.to_string() }),
        ["mode", mode_name, "scope", scope_name] => {
            Ok(Layer::ModeScope {
                mode: mode_name.to_string(),
                scope: scope_name.to_string(),
            })
        }
        ["mode", mode_name, "project", proj_name] => {
            Ok(Layer::ModeProject {
                mode: mode_name.to_string(),
                project: proj_name.to_string(),
            })
        }
        ["mode", mode_name, "scope", scope_name, "project", proj_name] => {
            Ok(Layer::ModeScopeProject {
                mode: mode_name.to_string(),
                scope: scope_name.to_string(),
                project: proj_name.to_string(),
            })
        }
        _ => Err(JinError::Message(format!(
            "Invalid layer specification: '{}'. Expected format: global, mode/<name>, scope/<name>, project/<name>, mode/<m>/scope/<s>, mode/<m>/project/<p>, or mode/<m>/scope/<s>/project/<p>",
            spec
        }))),
    }
}

// ===== FILE DIFF COMPUTATION =====
fn diff_files_in_layer(
    old_files: &HashMap<String, Vec<u8>>,
    new_files: &HashMap<String, Vec<u8>>,
) -> Vec<FileDiff> {
    let mut all_paths: HashSet<&str> = HashSet::new();
    for path in old_files.keys() { all_paths.insert(path); }
    for path in new_files.keys() { all_paths.insert(path); }

    let mut diffs = Vec::new();

    for path in all_paths {
        let old_bytes = old_files.get(path);
        let new_bytes = new_files.get(path);

        let old_str = old_bytes.and_then(|b| String::from_utf8(b.clone()).ok());
        let new_str = new_bytes.and_then(|b| String::from_utf8(b.clone()).ok());

        let status = match (old_bytes, new_bytes) {
            (None, Some(_)) => DiffStatus::Added,
            (Some(_), None) => DiffStatus::Deleted,
            (Some(o), Some(n)) => {
                if o == n { DiffStatus::Unchanged } else { DiffStatus::Modified }
            }
            (None, None) => continue, // Shouldn't happen
        };

        diffs.push(FileDiff {
            path: path.to_string(),
            status,
            old_content: old_str,
            new_content: new_str,
        });
    }

    diffs.sort_by(|a, b| a.path.cmp(&b.path));
    diffs
}

// ===== UNIFIED DIFF FORMAT =====
fn format_unified_diff(
    file_diff: &FileDiff,
    source_layer: &Layer,
    target_layer: &Layer,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("{}--- a/{}\\t{}{}\\n",
        RED, file_diff.path, source_layer, RESET));
    output.push_str(&format!("{}+++ b/{}\\t{}{}\\n",
        GREEN, file_diff.path, target_layer, RESET));

    match &file_diff.status {
        DiffStatus::Added => {
            if let Some(content) = &file_diff.new_content {
                output.push_str(&format_text_diff_as_added(content));
            }
        }
        DiffStatus::Deleted => {
            if let Some(content) = &file_diff.old_content {
                output.push_str(&format_text_diff_as_deleted(content));
            }
        }
        DiffStatus::Modified => {
            if let (Some(old), Some(new)) = (&file_diff.old_content, &file_diff.new_content) {
                output.push_str(&format_text_diff(old, new));
            }
        }
        DiffStatus::Unchanged => {
            // No output for unchanged files
        }
    }

    output
}

fn format_text_diff(old: &str, new: &str) -> String {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .from_lines(old, new);

    let mut output = String::new();
    let mut line_num_old = 1;
    let mut line_num_new = 1;

    for change in diff.iter_changes(None) {
        match change.tag() {
            ChangeTag::Delete => {
                for line in change.value().lines() {
                    output.push_str(&format!("{}-{}{}\\n", RED, line, RESET));
                }
                line_num_old += change.value().lines().count();
            }
            ChangeTag::Insert => {
                for line in change.value().lines() {
                    output.push_str(&format!("{}+{}{}\\n", GREEN, line, RESET));
                }
                line_num_new += change.value().lines().count();
            }
            ChangeTag::Equal => {
                let lines: Vec<&str> = change.value().lines().collect();
                if !lines.is_empty() {
                    // Show hunk header
                    output.push_str(&format!("@@ -{},{} +{},{} @@\\n",
                        line_num_old, lines.len(), line_num_new, lines.len()));
                }
                for line in lines {
                    output.push_str(&format!(" {}\\n", line));
                }
                line_num_old += lines.len();
                line_num_new += lines.len();
            }
        }
    }

    output
}

// ===== MAIN EXECUTE =====
pub fn execute(cmd: &DiffCommand) -> Result<()> {
    let workspace_root = std::env::current_dir()?;
    let context = ProjectContext::load(&workspace_root)?;
    let project_name = detect_project_name(&workspace_root)?;

    let _git_repo = git2::Repository::discover(&workspace_root)
        .map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    let repo = JinRepo::open_or_create(&workspace_root)?;

    let exit_code = if cmd.staged {
        diff_staged(&repo, &project_name)?
    } else {
        match (&cmd.layer1, &cmd.layer2) {
            (None, None) => diff_workspace(&repo, &project_name, &context)?,
            (Some(spec1), Some(spec2)) => {
                let layer1 = parse_layer_spec(spec1, &project_name)?;
                let layer2 = parse_layer_spec(spec2, &project_name)?;
                diff_layers(&repo, &layer1, &layer2, &project_name)?
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(JinError::Message(
                    "Specify both layers or neither layer".to_string()
                ));
            }
        }
    };

    // Use process::exit to set correct exit code
    std::process::exit(exit_code);
}

// ===== GOTCHAS TO HANDLE =====
// 1. parse_layer_spec must handle all layer variants
// 2. LayerMerge::read_layer_files returns HashMap<String, Vec<u8>>
// 3. Need to convert Vec<u8> to String (handle UTF-8 errors)
// 4. Exit code 1 for differences found (use std::process::exit)
// 5. Workspace files are in .jin/workspace/ directory
// 6. StagedEntry::path() returns absolute path - need to make relative
// 7. For --staged, need to compare staged content to committed content
```

### Integration Points

```yaml
COMMAND_MODULE:
  - file: src/commands/mod.rs
  - add: pub mod diff;
  - add: pub use diff::execute as diff_execute;
  - pattern: Follow existing module exports (after status)

MAIN_DISPATCHER:
  - file: src/main.rs
  - update: Commands::Diff(cmd) branch (lines 168-171)
  - pattern: match commands::diff_execute(&cmd) { Ok(()) => ..., Err(e) => ... }

CARGO_TOML:
  - file: Cargo.toml
  - add: console = "0.15"
  - optional: Could use colored crate instead

LAYER_MERGE:
  - method: LayerMerge::new(&repo, project).with_mode().with_scope()
  - method: LayerMerge::read_layer_files(&layer) -> Result<HashMap<String, Vec<u8>>>
  - method: LayerMerge::merge_all() -> Result<IndexMap<String, MergeValue>>

STAGING_INDEX:
  - method: StagingIndex::load_from_disk(&workspace_root)
  - method: staging.all_entries() -> Vec<&StagedEntry>
  - method: staging.entries_by_layer(&layer) -> Vec<&StagedEntry>

JINREPO:
  - method: JinRepo::open_or_create(&workspace_root)
  - method: repo.get_layer_ref(&layer) -> Result<Option<Reference>>
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
# Test diff command specifically
cargo test --lib diff

# Test all command tests
cargo test --lib commands

# Test with output
cargo test --lib diff -- --nocapture

# Expected: All tests pass
# Key test cases:
# - test_parse_layer_spec_valid_formats
# - test_parse_layer_spec_invalid_format
# - test_diff_files_in_layer_added
# - test_diff_files_in_layer_deleted
# - test_diff_files_in_layer_modified
# - test_diff_layers_no_difference
# - test_diff_layers_has_difference
# - test_format_unified_diff
# - test_diff_workspace_empty
# - test_diff_staged_no_files
# - test_diff_layer_nonexistent
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the project
cargo build --release

# Manual integration test in a temporary directory
cd /tmp
rm -rf test_jin_diff
mkdir test_jin_diff && cd test_jin_diff
git init
jin init

# Create mode with config file
mkdir -p .jin/mode/claude
echo '{"setting": "value1"}' > .jin/mode/claude/config.json

# Create project with different config
mkdir -p .jin/project/test_jin_diff
echo '{"setting": "value2"}' > .jin/project/test_jin_diff/config.json

# Stage and commit files
jin add .jin/mode/claude/config.json --mode
jin commit -m "Add mode config"
jin add .jin/project/test_jin_diff/config.json
jin commit -m "Add project config"

# Test 1: Compare two layers
jin diff mode/claude project/test_jin_diff
# Expected: Shows diff between mode and project configs

# Test 2: Apply and compare workspace
jin apply
jin diff
# Expected: Shows no differences (workspace matches merged)

# Test 3: Modify workspace and diff
echo '{"setting": "value3"}' > .jin/workspace/config.json
jin diff
# Expected: Shows workspace vs merged diff

# Test 4: Stage and use --staged
echo '{"setting": "value4"}' > config.json
jin add config.json --mode
jin diff --staged
# Expected: Shows staged vs committed diff

# Test 5: No differences
jin diff mode/claude mode/claude
# Expected: "No differences found"

# Expected: All manual tests pass with correct output
```

### Level 4: CLI & Domain-Specific Validation

```bash
# Test layer spec parsing
jin diff global project/myproject
jin diff mode/claude scope/python
jin diff mode/claude/scope/javascript/project/myapp

# Test with various file types
echo '{"key": "value"}' > config.json
echo "setting=value" > config.ini
echo "key: value" > config.yaml
echo "[section]\\nkey=value" > config.toml

# Test exit codes
jin diff identical_layer; echo "Exit code: $?"
# Expected: Exit code: 0 (no differences)

jin diff different_layers; echo "Exit code: $?"
# Expected: Exit code: 1 (differences found)

# Test error handling
jin diff nonexistent_layer
# Expected: Error message

# Test path filtering (if implemented)
echo "change" > specific_file.txt
jin diff -- specific_file.txt

# Expected: All scenarios work correctly
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo build --release`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check`
- [ ] All tests pass: `cargo test --lib`
- [ ] No unused imports or dead code
- [ ] console crate added to Cargo.toml

### Feature Validation

- [ ] Shows workspace vs merged result (default)
- [ ] Shows differences between two specified layers
- [ ] Shows differences between staged and committed (--staged)
- [ ] Output follows unified diff format
- [ ] Colored output (green additions, red deletions)
- [ ] Returns exit code 0 when no differences
- [ ] Returns exit code 1 when differences found
- [ ] Returns exit code 2 on error
- [ ] Handles JSON/YAML/TOML with semantic diff
- [ ] Handles text files with line-based diff
- [ ] Handles non-existent layers gracefully
- [ ] Handles empty workspace/staging gracefully

### Code Quality Validation

- [ ] Follows existing command patterns (status, reset, apply)
- [ ] Proper error handling with Result<>
- [ ] Comprehensive doc comments on public functions
- [ ] Unit tests cover all major scenarios
- [ ] Tests use tempfile and DirGuard pattern
- [ ] No unwrap() calls (use proper error handling)
- [ ] Module docstring present
- [ ] Helper functions have doc comments

### Documentation & Deployment

- [ ] execute() has comprehensive doc comment
- [ ] Helper functions have doc comments
- [ ] User-facing error messages are clear
- [ ] Success output shows file count and changes
- [ ] Layer parsing errors include valid format examples

---

## Anti-Patterns to Avoid

- **Don't** skip the layer parsing step - users expect "mode/claude" syntax
- **Don't** use unwrap() or expect() - propagate errors with ?
- **Don't** forget to convert Vec<u8> to String (handle UTF-8 errors)
- **Don't** forget to use std::process::exit() for correct exit codes
- **Don't** implement custom diff algorithm - use similar crate
- **Don't** hardcode color codes - use console::Style
- **Don't** forget to add console crate to Cargo.toml
- **Don't** forget to update src/commands/mod.rs
- **Don't** forget to update src/main.rs dispatcher
- **Don't** skip unit tests for parse_layer_spec (it's critical for UX)
- **Don't** show absolute paths in diff output - use relative paths
- **Don't** show unchanged files by default
- **Don't** implement complex structured diff initially - start with line-based for text

## Confidence Score: 9/10

**Reasoning**:
- Complete codebase analysis with specific file references and line numbers
- All required infrastructure exists (similar crate, LayerMerge, JinRepo, StagingIndex)
- Clear implementation pattern from existing commands (status, reset, apply)
- Comprehensive external research on git diff UX patterns
- Specific helper functions and diff logic documented
- Unit test pattern established from other commands
- Research on Rust diff libraries provides clear implementation path

**Remaining risks**:
1. **Layer spec parsing**: Need to handle all 9 layer variants correctly (mitigated: comprehensive parse_layer_spec function)
2. **Exit code handling**: Need std::process::exit() which terminates process (mitigated: documented pattern)
3. **Structured diff for JSON/YAML/TOML**: Can start with line-based, enhance later (mitigated: task separates structured diff as optional enhancement)

**Mitigation**: The PRP provides complete parse_layer_spec implementation covering all layer variants, documents the std::process::exit() pattern, and separates structured diff as optional enhancement. Initial implementation can use line-based diff for all file types with structured diff as a follow-up enhancement.
