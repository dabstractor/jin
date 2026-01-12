# Product Requirement Prompt (PRP): Modify Apply Command to Write .jinmerge Files

---

## Goal

**Feature Goal**: Transform the `jin apply` command from an abort-on-conflict workflow to a pause-and-resolve workflow. When conflicts are detected during layer merging, the command will write `.jinmerge` files and persist operation state instead of failing.

**Deliverable**: Modified `src/commands/apply.rs` with conflict state tracking, `.jinmerge` file generation, and pause operation persistence.

**Success Definition**:
1. `jin apply` completes successfully even when conflicts are detected
2. `.jinmerge` files are created for each conflicting file with layer-aware markers
3. Operation state is persisted to `.jin/.paused_apply.yaml`
4. User is instructed to use `jin resolve` to continue (P1.M1.T3)
5. All existing `apply` functionality (dry-run, force, metadata updates) works for non-conflict cases

---

## Why

- **User Impact**: Current abort-on-conflict behavior blocks all work when ANY conflict exists. Users must manually investigate and resolve without tool assistance.
- **Integration**: Enables the `jin resolve` command (P1.M1.T3) to resume operations after manual conflict resolution.
- **Problem Solved**: Allows partial apply operations - non-conflicting files are applied, conflicting files generate `.jinmerge` files for manual resolution.

---

## What

**User-Visible Behavior**:

When `jin apply` encounters merge conflicts:

**Before (Current)**:
```bash
$ jin apply
Merge conflicts detected in 3 files:
  - config.json
  - settings.yaml
  - .env
Error: Cannot apply due to 3 merge conflicts
# Operation ABORTED - no files written
```

**After (This Task)**:
```bash
$ jin apply
Merge conflicts detected in 3 files:
  - config.json
  - settings.yaml
  - .env

Created .jinmerge files for manual resolution:
  - config.json.jinmerge
  - settings.yaml.jinmerge
  - .env.jinmerge

Operation paused. Resolve conflicts with:
  jin resolve <file>

For more information, run: jin status
# Operation PAUSED - non-conflicting files applied successfully
```

**State Persistence**:
A `.jin/.paused_apply.yaml` file is created with:
- Operation timestamp
- List of conflicting files
- List of successfully applied files
- Layer configuration used for merge

**Technical Requirements**:

1. **Conflict Collection**: Continue collecting all conflicts (current behavior in `merge_layers`)
2. **.jinmerge File Generation**: For each conflict file, generate a `.jinmerge` file using `JinMergeConflict::from_text_merge()`
3. **State Persistence**: Write `.jin/.paused_apply.yaml` with operation details
4. **Conditional Apply**: Only apply non-conflicting files to workspace
5. **User Notification**: Display helpful next steps

### Success Criteria

- [ ] `jin apply` no longer aborts when conflicts are detected
- [ ] `.jinmerge` files are created for each conflicting file
- [ ] `.jin/.paused_apply.yaml` is written with operation state
- [ ] Non-conflicting files are still applied to workspace
- [ ] User is instructed to run `jin resolve` or `jin status`
- [ ] Dry-run mode shows what would be paused
- [ ] All existing tests continue to pass
- [ ] New tests cover conflict pause workflow

---

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes. This PRP provides:
- Exact file locations and line numbers to modify
- Complete data structure definitions from completed P1.M1.T1
- Existing code patterns to follow
- Test patterns and validation commands
- Error handling conventions

### Documentation & References

```yaml
# MUST READ - Critical dependencies from completed work

- file: src/merge/jinmerge.rs
  why: Contains JinMergeConflict and JinMergeRegion structures, write_to_file() method
  critical: |
    - JinMergeConflict::from_text_merge() creates conflict from merge results
    - JinMergeConflict::write_to_file() handles atomic file writes
    - merge_path_for_file() generates .jinmerge file path from original path
  gotcha: Must import JinMergeConflict via `use crate::merge::jinmerge::JinMergeConflict;`

- file: src/commands/apply.rs (lines 63-76)
  why: Current abort-on-conflict behavior to be replaced
  pattern: |
    Current code returns error immediately. New code should:
    1. Generate .jinmerge files for each conflict
    2. Write paused state
    3. Return Ok(()) after applying non-conflicting files
  gotcha: Lines 64-76 are the key section to modify

- file: src/merge/layer.rs (lines 103-123)
  why: merge_layers() function that returns LayerMergeResult with conflict_files
  pattern: |
    Collects all conflicts before returning. No changes needed here,
    but important to understand the data flow.
  critical: conflict_files Vec<PathBuf> is the source of truth for conflicts

- file: src/merge/layer.rs (lines 54-65)
  why: LayerMergeResult structure definition
  pattern: |
    pub struct LayerMergeResult {
        pub merged_files: HashMap<PathBuf, MergedFile>,
        pub conflict_files: Vec<PathBuf>,
        pub added_files: Vec<PathBuf>,
        pub removed_files: Vec<PathBuf>,
    }
  gotcha: merged_files contains ONLY successfully merged files

- file: src/merge/text.rs
  why: Text merge functionality that detects conflicts
  pattern: |
    TextMergeResult::Conflict { content, conflict_count }
    This is what triggers MergeConflict errors during layer merge

- file: src/core/error.rs (lines 25-27)
  why: MergeConflict error type definition
  pattern: |
    #[error("Merge conflict in {path}")]
    MergeConflict { path: String }

- file: tests/common/fixtures.rs
  why: TestFixture pattern for isolated test environments
  pattern: |
    pub struct TestFixture {
        _tempdir: TempDir,
        pub path: PathBuf,
        pub jin_dir: Option<PathBuf>,
    }

- file: tests/cli_basic.rs
  why: Basic command test patterns
  pattern: |
    Uses assert_cmd crate with Jin environment isolation via JIN_DIR

# STATE MANAGEMENT PATTERNS

- file: src/core/config.rs
  why: ProjectContext save/load pattern for YAML serialization
  pattern: |
    Use serde_yaml for YAML serialization
    Use std::fs::write with atomic temp-file-then-rename pattern

- file: src/staging/metadata.rs
  why: WorkspaceMetadata save/load pattern
  pattern: |
    Atomic write: temp_path.with_extension("tmp"), then fs::rename()

# EXTERNAL RESEARCH

- url: https://docs.rs/serde_yaml/latest/serde_yaml/
  why: YAML serialization for paused state file
  critical: serde_yaml::to_string() and serde_yaml::from_str() functions

- url: https://docs.rs/tempfile/latest/tempfile/
  why: Test isolation for integration tests
  critical: TempDir::new() creates temporary directories that auto-clean
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── Cargo.toml                     # Dependencies: serde_yaml, tempfile, etc.
├── src/
│   ├── commands/
│   │   ├── apply.rs               # MAIN FILE TO MODIFY (lines 63-76)
│   │   └── mod.rs                 # Command exports
│   ├── cli/
│   │   ├── args.rs                # ApplyArgs struct
│   │   └── mod.rs                 # CLI registration
│   ├── core/
│   │   ├── config.rs              # ProjectContext (YAML pattern)
│   │   ├── error.rs               # JinError::MergeConflict
│   │   ├── jinmap.rs              # JinMap (YAML pattern)
│   │   ├── layer.rs               # Layer enum
│   │   └── mod.rs                 # Core exports
│   ├── git/
│   │   ├── repo.rs                # JinRepo
│   │   └── mod.rs                 # Git exports
│   ├── merge/
│   │   ├── jinmerge.rs            # COMPLETED: JinMergeConflict, write_to_file()
│   │   ├── layer.rs               # merge_layers(), LayerMergeResult
│   │   ├── text.rs                # TextMergeResult::Conflict
│   │   ├── deep.rs                # Deep merge for structured files
│   │   ├── value.rs               # MergeValue enum
│   │   └── mod.rs                 # Merge exports
│   ├── staging/
│   │   ├── metadata.rs            # WorkspaceMetadata (YAML pattern)
│   │   └── mod.rs                 # Staging exports
│   ├── lib.rs                     # Library entry
│   └── main.rs                    # Binary entry
├── tests/
│   ├── common/
│   │   ├── fixtures.rs            # TestFixture pattern
│   │   ├── assertions.rs          # Custom assertions
│   │   └── mod.rs                 # Test utilities
│   ├── cli_basic.rs               # Apply command tests
│   └── ...
└── .jin/                          # Runtime directory (not in source)
    ├── context                    # ProjectContext YAML
    ├── .jinmap                    # JinMap YAML
    └── workspace/
        └── last_applied.json      # WorkspaceMetadata
```

### Desired Codebase Tree (Files to Add)

```bash
# NEW STATE FILE (runtime, not source)
.jin/.paused_apply.yaml             # PausedApplyState (created at runtime)

# MODIFIED FILE
src/commands/apply.rs               # Add conflict handling (lines 63-76+)

# NEW TEST FILE
tests/cli_apply_conflict.rs         # Tests for conflict pause workflow
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: .jinmerge module exports are in src/merge/mod.rs
// Must use: use crate::merge::jinmerge::JinMergeConflict;
// NOT: use crate::merge::JinMergeConflict;

// CRITICAL: LayerMergeResult.conflict_files contains file paths
// but merged_files does NOT include conflict files
// They are mutually exclusive in the result

// CRITICAL: Atomic write pattern must be used for ALL state files
// 1. Write to temp_path = path.with_extension("tmp")
// 2. fs::write(&temp_path, content)
// 3. fs::rename(&temp_path, path)  // Atomic on POSIX

// CRITICAL: JIN_DIR environment variable for test isolation
// Always set in tests: .env("JIN_DIR", &jin_dir)

// CRITICAL: Workspace dirty check uses WorkspaceMetadata
// If metadata doesn't exist, workspace is considered clean

// GOTCHA: text_merge() returns TextMergeResult::Conflict as OK result
// NOT as Err(). Check pattern: match text_merge(...) {
//     Ok(TextMergeResult::Conflict { .. }) => { /* handle conflict */ }
//     Ok(TextMergeResult::Clean(s)) => { /* handle clean merge */ }
//     Err(e) => return Err(e),
// }

// GOTCHA: Layer ref paths use '/' not '-' as separator
// Example: "mode/claude/scope:javascript/" not "mode-claude-scope:javascript-"

// GOTCHA: File format detection uses extension only
// detect_format() in src/merge/layer.rs
```

---

## Implementation Blueprint

### Data Models and Structure

#### 1. PausedApplyState Structure

Create new struct in `src/commands/apply.rs`:

```rust
// Add to src/commands/apply.rs after imports
use crate::core::{JinError, ProjectContext, Result};
use crate::merge::jinmerge::JinMergeConflict;
use crate::merge::{FileFormat, LayerMergeConfig, MergedFile};
use crate::git::JinRepo;
use crate::staging::WorkspaceMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// State for a paused apply operation due to conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PausedApplyState {
    /// When the operation was paused
    timestamp: DateTime<Utc>,
    /// Layer configuration used for the merge attempt
    layer_config: PausedLayerConfig,
    /// Files with conflicts (original paths, not .jinmerge paths)
    conflict_files: Vec<PathBuf>,
    /// Files that were successfully applied
    applied_files: Vec<PathBuf>,
    /// Number of conflicts total
    conflict_count: usize,
}

/// Simplified layer config for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PausedLayerConfig {
    layers: Vec<String>,  // Layer names as strings
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
}

impl PausedApplyState {
    /// Save state to `.jin/.paused_apply.yaml`
    fn save(&self) -> Result<()> {
        let path = PathBuf::from(".jin/.paused_apply.yaml");
        let content = serde_yaml::to_string(self)
            .map_err(|e| JinError::Other(format!("Failed to serialize paused state: {}", e)))?;

        // Atomic write pattern
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content)
            .map_err(JinError::Io)?;
        std::fs::rename(&temp_path, &path)
            .map_err(JinError::Io)?;

        Ok(())
    }

    /// Check if a paused operation exists
    fn exists() -> bool {
        PathBuf::from(".jin/.paused_apply.yaml").exists()
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD new imports to src/commands/apply.rs
  - ADD: use crate::merge::jinmerge::JinMergeConflict;
  - ADD: use chrono::{DateTime, Utc};
  - ADD: use serde::{Deserialize, Serialize};
  - LOCATION: Top of file after existing imports (lines 1-10)
  - PATTERN: Follow existing import style (alphabetized by crate)

Task 2: DEFINE PausedApplyState structs in src/commands/apply.rs
  - IMPLEMENT: PausedApplyState struct with timestamp, layer_config, conflict_files, applied_files
  - IMPLEMENT: PausedLayerConfig helper struct
  - IMPLEMENT: PausedApplyState::save() method using YAML + atomic write
  - IMPLEMENT: PausedApplyState::exists() static method
  - LOCATION: After imports, before execute() function (around line 12)
  - PATTERN: Follow WorkspaceMetadata pattern from src/staging/metadata.rs
  - NAMING: snake_case for functions, PascalCase for structs

Task 3: MODIFY execute() function - Replace abort with pause workflow
  - LOCATION: src/commands/apply.rs, lines 63-76 (current abort block)
  - BEFORE: return Err(JinError::Other(...))
  - AFTER:
    1. Call handle_conflicts() to generate .jinmerge files
    2. Apply only non-conflicting files (filter merged.merged_files)
    3. Create PausedApplyState and save it
    4. Display user instructions
    5. Return Ok(())
  - PRESERVE: All error handling for non-conflict errors

Task 4: IMPLEMENT handle_conflicts() function
  - CREATE: fn handle_conflicts(conflict_files: &[PathBuf], config: &LayerMergeConfig, repo: &JinRepo) -> Result<PausedApplyState>
  - FOR each conflict file in conflict_files:
    1. Read content from both conflicting layers
    2. Call JinMergeConflict::from_text_merge() with layer refs and content
    3. Call JinMergeConflict::write_to_file() to write .jinmerge
  - COLLECT: Successfully created conflict states
  - RETURN: PausedApplyState with all conflict info
  - LOCATION: After execute() function (around line 115)
  - NAMING: snake_case function name

Task 5: MODIFY apply_to_workspace() - Filter out conflict files
  - LOCATION: src/commands/apply.rs, line 118-141
  - CHANGE: Filter merged.merged_files to exclude conflict_files
  - PATTERN: |
    for (path, merged_file) in &merged.merged_files {
        if merged.conflict_files.contains(path) {
            continue;  // Skip conflict files
        }
        // ... existing apply logic
    }
  - PRESERVE: All existing atomic write logic

Task 6: MODIFY metadata update - Only track applied files
  - LOCATION: src/commands/apply.rs, lines 88-96
  - CHANGE: Only add files to metadata that were actually applied
  - PATTERN: |
    for (path, merged_file) in &merged.merged_files {
        if merged.conflict_files.contains(path) {
            continue;  // Don't track conflict files in metadata
        }
        // ... existing metadata logic
    }

Task 7: CREATE tests/cli_apply_conflict.rs
  - IMPLEMENT: Integration tests for conflict pause workflow
  - TEST: test_apply_with_conflicts_creates_jinmerge_files()
  - TEST: test_apply_with_conflicts_creates_paused_state()
  - TEST: test_apply_with_conflicts_applies_non_conflicting_files()
  - TEST: test_apply_dry_run_with_conflicts_shows_preview()
  - PATTERN: Follow tests/cli_basic.rs pattern using assert_cmd
  - SETUP: Use TestFixture from tests/common/fixtures.rs
  - ISOLATION: Set JIN_DIR environment variable

Task 8: UPDATE existing tests in src/commands/apply.rs
  - VERIFY: test_execute_not_initialized still passes
  - VERIFY: test_check_workspace_dirty_no_metadata still passes
  - VERIFY: test_serialize_merged_content_json still passes
  - LOCATION: src/commands/apply.rs lines 274-322
  - NO CHANGES: Tests should pass without modification
```

### Implementation Patterns & Key Details

```rust
// PATTERN 1: Reading layer content for .jinmerge generation
// This is tricky - need to get content from TWO layers that conflicted

// In handle_conflicts(), for each conflict file:
fn get_conflicting_layer_contents(
    file_path: &Path,
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<(String, String, String, String)> {
    // Returns: (layer1_ref, layer1_content, layer2_ref, layer2_content)

    // GET the two highest-precedence layers that contain this file
    // This requires iterating layers in REVERSE (highest first)
    // and finding the first TWO that have the file

    let mut layer_refs = Vec::new();
    for layer in config.layers.iter().rev() {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        if repo.ref_exists(&ref_path) {
            if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
                let commit = repo.inner().find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                if let Ok(content) = repo.read_file_from_tree(tree_oid, file_path) {
                    let content_str = String::from_utf8_lossy(&content).to_string();
                    layer_refs.push((ref_path, content_str));

                    if layer_refs.len() >= 2 {
                        break;  // Got the two conflicting layers
                    }
                }
            }
        }
    }

    if layer_refs.len() < 2 {
        return Err(JinError::Other(
            format!("Could not find two layers for conflict: {}", file_path.display())
        ));
    }

    // layer_refs[0] is higher precedence (theirs)
    // layer_refs[1] is lower precedence (ours)
    Ok((
        layer_refs[1].0.clone(),  // layer1_ref (ours)
        layer_refs[1].1.clone(),  // layer1_content
        layer_refs[0].0.clone(),  // layer2_ref (theirs)
        layer_refs[0].1.clone(),  // layer2_content
    ))
}

// PATTERN 2: JinMergeConflict creation
// In handle_conflicts():
for conflict_path in &conflict_files {
    let (layer1_ref, layer1_content, layer2_ref, layer2_content) =
        get_conflicting_layer_contents(conflict_path, config, repo)?;

    let merge_conflict = JinMergeConflict::from_text_merge(
        conflict_path.clone(),
        layer1_ref,
        layer1_content,
        layer2_ref,
        layer2_content,
    );

    let merge_path = JinMergeConflict::merge_path_for_file(conflict_path);
    merge_conflict.write_to_file(&merge_path)?;
}

// PATTERN 3: Atomic write for paused state
// This is CRITICAL - follow exact pattern from WorkspaceMetadata
fn save_paused_state(state: &PausedApplyState) -> Result<()> {
    let path = PathBuf::from(".jin/.paused_apply.yaml");
    let content = serde_yaml::to_string(state)
        .map_err(|e| JinError::Other(format!("Serialization failed: {}", e)))?;

    // Atomic write: temp file then rename
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content).map_err(JinError::Io)?;
    std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;

    Ok(())
}

// GOTCHA: Layer ref paths use specific format
// Layer::ref_path() returns strings like:
// - "refs/jin/layers/mode/claude/"
// - "refs/jin/layers/mode/claude/scope:language:javascript/"
// - "refs/jin/layers/project/myproject/"

// For .jinmerge labels, we want to strip "refs/jin/layers/" prefix
// Use layer.name() which returns the short form like "mode/claude/"
```

### Integration Points

```yaml
MERGE_ENGINE:
  - uses: src/merge/layer.rs::merge_layers()
  - returns: LayerMergeResult with conflict_files Vec<PathBuf>
  - behavior: No changes needed - already collects all conflicts

STATE_FILE:
  - location: .jin/.paused_apply.yaml
  - format: YAML (serde_yaml)
  - pattern: Follow ProjectContext pattern from src/core/config.rs
  - cleanup: Removed by `jin resolve --abort` or after successful resolve

JINMERGE_FILES:
  - created: In workspace root, same name + .jinmerge extension
  - format: Git-compatible conflict markers via JinMergeConflict
  - location: Created via JinMergeConflict::write_to_file()
  - example: config.json -> config.json.jinmerge

METADATA:
  - file: .jin/workspace/last_applied.json
  - change: Only include successfully applied files (exclude conflicts)
  - format: JSON (existing WorkspaceMetadata)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation/modification
cargo check --message-format=short
# Expected: No compilation errors

# Format check
cargo fmt --all -- --check
# Fix formatting issues: cargo fmt

# Lint check (if using clippy)
cargo clippy --all-targets --all-features -- -D warnings
# Expected: No clippy warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run apply command unit tests
cargo test --lib apply
# Expected: All existing tests pass

# Run new conflict-specific tests
cargo test --test cli_apply_conflict
# Expected: All new tests pass

# Run all tests
cargo test
# Expected: All tests pass

# Run with output
cargo test -- --nocapture
```

### Level 3: Integration Testing (System Validation)

```bash
# Setup test environment
export JIN_DIR=$(mktemp -d)
cd /tmp/test_project
jin init

# Test 1: Create conflicting layers
mkdir -p .jin/global
echo '{"port": 8080}' > config.json
jin add config.json --global
git commit -m "Add base config"

mkdir -p .jin/mode/production
echo '{"port": 9090, "debug": false}' > config.json
jin add config.json --mode
git commit -m "Add production config"

# Test 2: Trigger conflict scenario
# (Modify same file in overlapping layers)
# Then run: jin apply

# Expected behavior:
# 1. Command completes with exit code 0 (not error)
# 2. .jin/.paused_apply.yaml exists
# 3. config.json.jinmerge exists with conflict markers
# 4. Message shows "Resolve conflicts with: jin resolve <file>"

# Verify state file
cat .jin/.paused_apply.yaml
# Expected: Valid YAML with timestamp, conflict_files list

# Verify .jinmerge format
cat config.json.jinmerge
# Expected: Header comment + Git-compatible markers

# Test 3: Dry-run with conflicts
jin apply --dry-run
# Expected: Shows conflicts without writing files
```

### Level 4: End-to-End Workflow Testing

```bash
# Complete conflict workflow test
# 1. Setup: Create layers with overlapping files
# 2. Trigger: Run jin apply (should pause)
# 3. Verify: Check .jinmerge files and paused state
# 4. Manual: Edit .jinmerge file to resolve conflict
# 5. Resolve: Run jin resolve config.json (P1.M1.T3)
# 6. Verify: Check jin status shows "No conflicts"

# Test isolation with multiple conflicts
# 1. Create 3 files with conflicts
# 2. Run jin apply
# 3. Verify: 3 .jinmerge files created
# 4. Verify: conflict_files contains all 3 paths
# 5. Verify: Non-conflicting files still applied

# Test force flag with conflicts
jin apply --force
# Expected: Still pauses (force is for workspace dirty, not conflicts)

# Test with no conflicts
# 1. Create clean layers
# 2. Run jin apply
# 3. Verify: Normal apply behavior (no paused state)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] cargo check passes with no errors
- [ ] cargo test passes all existing and new tests
- [ ] cargo fmt --check shows no formatting issues
- [ ] No clippy warnings
- [ ] .jinmerge files are created with valid format
- [ ] .jin/.paused_apply.yaml is valid YAML
- [ ] Atomic writes verified (no tmp files left behind)

### Feature Validation

- [ ] `jin apply` completes successfully (exit code 0) with conflicts
- [ ] .jinmerge files created for each conflict file
- [ ] Non-conflicting files are still applied to workspace
- [ ] .jin/.paused_apply.yaml contains correct state
- [ ] User sees helpful next steps message
- [ ] Dry-run mode doesn't write state files
- [ ] Force flag doesn't bypass conflict pause
- [ ] Error cases handled gracefully

### Code Quality Validation

- [ ] Follows existing codebase patterns (atomic writes, YAML serialization)
- [ ] File placement matches desired tree structure
- [ ] Naming conventions consistent (snake_case functions, PascalCase structs)
- [ ] Proper error handling with specific error types
- [ ] No dead code or unused imports

### Integration Validation

- [ ] Works with all file types (JSON, YAML, TOML, INI, text)
- [ ] Compatible with existing layer system
- [ ] Metadata tracking excludes conflict files
- [ ] State file location in .jin directory
- [ ] Ready for P1.M1.T3 (resolve command integration)

---

## Anti-Patterns to Avoid

- **Don't** return early on first conflict - collect ALL conflicts before handling
- **Don't** modify existing merge_layers() function - it works correctly
- **Don't** skip atomic write pattern for state file - corruption risk
- **Don't** forget to filter conflict files from metadata - will break resolve
- **Don't** use direct string manipulation for YAML - use serde_yaml
- **Don't** hardcode layer ref paths - use Layer::ref_path() method
- **Don't** apply conflict files to workspace - creates corruption
- **Don't** ignore the return value of JinMergeConflict::write_to_file()
- **Don't** create new error types - use existing JinError variants
- **Don't** skip tests for edge cases (empty conflicts, single conflict, etc.)

---

## Confidence Score

**8.5/10** - One-pass implementation success likelihood

**Rationale**:
- **High confidence**: P1.M1.T1 (.jinmerge module) is complete and tested
- **High confidence**: Existing codebase patterns are well-documented
- **High confidence**: Test infrastructure is comprehensive
- **Medium confidence**: Getting two layer contents for .jinmerge requires careful layer iteration
- **Known risk**: Text merge conflict detection may need adjustment for structured files

**Mitigation for known risks**:
- Start with text files only (simpler conflict detection)
- Add structured file conflict handling in follow-up if needed
- Extensive test coverage for conflict scenarios

---

## Appendix: Example .jinmerge File Format

```markdown
# Jin merge conflict. Resolve and run 'jin resolve config.json'
<<<<<<< refs/jin/layers/mode/production/
{
  "port": 9090,
  "debug": false
}
=======
{
  "port": 8080,
  "debug": true
}
>>>>>>> refs/jin/layers/global/
```

**Key aspects**:
1. Header comment with user instruction
2. Layer refs (not generic "ours/theirs")
3. Git-compatible 7-character markers
4. Full content preservation
5. Trailing newline for editor compatibility
