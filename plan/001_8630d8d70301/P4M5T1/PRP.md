name: "Diff Command PRP - Implementation and Completion Guide"
description: |

---

## Goal

**Feature Goal**: Implement a comprehensive diff command that shows differences between layers, workspace, and staged changes in Jin's 9-layer configuration system.

**Deliverable**: A fully functional `jin diff` command with four operational modes:
1. `--staged` - Show staged changes
2. `jin diff <layer1> <layer2>` - Compare two specific layers
3. `jin diff <layer>` - Compare workspace vs specified layer
4. `jin diff` - Compare workspace vs workspace-active (default)

**Success Definition**:
- All four diff modes are fully functional
- Colored diff output works correctly (green additions, red deletions)
- Integration tests cover all diff modes
- Error handling covers edge cases (uninitialized, no commits, invalid layers)
- Output follows git unified diff format conventions

## User Persona

**Target User**: Developers managing multi-layered configurations with Jin

**Use Case**: A developer needs to understand what changes exist between different configuration layers before committing or applying changes

**User Journey**:
1. User has made changes to workspace files
2. User runs `jin diff` to see what changed vs workspace-active
3. User stages specific files with `jin add config.yaml --mode`
4. User runs `jin diff --staged` to verify what will be committed
5. User optionally runs `jin diff mode-base mode-project` to compare layer contents
6. User commits changes with confidence

**Pain Points Addressed**:
- **"What changed?"** - Without diff, users don't know what differs between layers
- **"Will this break my config?"** - Staged diff prevents accidental commits
- **"Which layer introduced this setting?"** - Layer comparison aids debugging

## Why

- **Configuration debugging**: Developers need to trace which layer introduced specific configuration values
- **Safety before commits**: Staged diff prevents accidental commits of wrong files
- **Layer understanding**: Comparing layers helps understand the merge hierarchy
- **Git-like familiarity**: Users expect diff functionality from their VCS experience

## What

The diff command supports four comparison modes:

### Mode 1: Staged Changes (`--staged`)
Shows files in the staging index with their target layers and modification status.

```bash
$ jin diff --staged
Staged changes:
  config.json -> mode-base
  settings.yaml -> mode-project
  (modified since staging)
```

### Mode 2: Layer vs Layer (`jin diff <layer1> <layer2>`)
Shows unified diff between two specific layers using git diff.

```bash
$ jin diff mode-base mode-project
diff --jin a/mode-base b/mode-project
--- a/config.json
+++ b/config.json
@@ -1,5 +1,5 @@
 {
   "port": 3000,
-  "debug": false
+  "debug": true,
+  "logLevel": "info"
 }
```

### Mode 3: Workspace vs Layer (`jin diff <layer>`)
Shows actual diff between workspace files and a specific layer's content.

### Mode 4: Default - Workspace vs Workspace-Active (`jin diff`)
Shows differences between current workspace and the merged result of all applicable layers.

### Success Criteria

- [ ] All four diff modes execute without errors
- [ ] Colored output displays correctly in terminals
- [ ] Layer name parsing handles all 9 layer types
- [ ] Error messages are clear for missing layers/commits
- [ ] Binary files are handled gracefully
- [ ] Integration tests verify all modes

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes - this PRP provides:
- Exact file paths for all relevant code
- Implementation patterns from existing commands
- git2 library API references
- Testing patterns and fixtures
- Layer system architecture details
- Common gotchas and pitfalls

### Documentation & References

```yaml
# MUST READ - Core Implementation Files
- file: src/commands/diff.rs
  why: Existing diff command implementation (P4.M5.T1.S1 complete)
  pattern: Module-based execute function with mode dispatch
  gotcha: diff_workspace_vs_workspace_active is placeholder

- file: src/cli/args.rs:88-100
  why: DiffArgs struct definition with layer1, layer2, staged fields
  pattern: clap derive Args with Optional String and bool fields

- file: src/cli/mod.rs:57
  why: Commands enum with Diff variant
  pattern: Subcommand registration in main CLI

- file: src/commands/mod.rs:42
  why: Command dispatcher for diff
  pattern: Match arm that calls diff::execute(args)

# CRITICAL PATTERNS - Reference Implementations
- file: src/commands/status.rs:20-118
  why: Status command pattern for workspace state checking
  pattern: Load context, check workspace state, display structured output
  gotcha: Uses WorkspaceState enum (Clean/Dirty)

- file: src/commands/apply.rs:27-115
  why: Apply command shows layer merge and workspace operations
  pattern: Load context, check dirty state, perform operation, report results

- file: src/core/layer.rs:12-31
  why: Layer enum with all 9 layer types and ref_path/storage_path methods
  pattern: Enum with context-dependent path generation
  gotcha: Requires mode/scope for some layers

- file: src/staging/workspace.rs
  why: WorkspaceMetadata structure for tracking applied files
  pattern: HashMap<PathBuf, String> for file -> content hash tracking

# EXTERNAL RESEARCH - Best Practices
- url: https://git-scm.com/docs/git-diff
  why: Git diff command reference for output format standards
  critical: Unified diff format with ---/+++ headers and @@ hunk markers

- url: https://docs.rs/git2/latest/git2/
  why: git2 crate documentation for Diff and DiffOptions
  critical: diff_tree_to_tree, diff_tree_to_workdir_with_index APIs

- docfile: plan/P4M5T1/research/diff_best_practices.md
  why: Comprehensive CLI diff best practices research
  section: "Diff Output Formatting" for color standards and format options

- docfile: plan/P4M5/research/diff_research.md
  why: Git diff algorithms and Rust implementation patterns
  section: "Tree Comparison Patterns" for layer vs layer comparison

# TESTING REFERENCES
- file: tests/cli_basic.rs
  why: Basic CLI command integration test patterns
  pattern: assert_cmd with temp directory isolation

- file: tests/common/fixtures.rs
  why: TestFixture and setup_test_repo helper functions
  pattern: TempDir isolation with JIN_DIR environment variable

- file: tests/common/assertions.rs
  why: Custom assertions for Jin state verification
  pattern: assert_workspace_file, assert_staging_contains helpers
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── Cargo.toml                 # Dependencies: git2, clap, thiserror, etc.
├── src/
│   ├── main.rs               # Entry point
│   ├── lib.rs                # Library exports
│   ├── cli/
│   │   ├── mod.rs            # Commands enum with Diff variant (line 57)
│   │   └── args.rs           # DiffArgs struct (lines 88-100)
│   ├── commands/
│   │   ├── mod.rs            # Command dispatcher (line 42)
│   │   ├── diff.rs           # DIFF COMMAND IMPLEMENTATION (291 lines)
│   │   ├── status.rs         # Status command (workspace state pattern)
│   │   ├── apply.rs          # Apply command (layer merge pattern)
│   │   └── [other commands]
│   ├── core/
│   │   ├── layer.rs          # Layer enum with 9 types (lines 12-31)
│   │   ├── config.rs         # ProjectContext structure
│   │   └── error.rs          # JinError enum
│   ├── git/
│   │   └── lib.rs            # JinRepo wrapper
│   ├── staging/
│   │   ├── index.rs          # StagingIndex
│   │   └── workspace.rs      # WorkspaceMetadata
│   └── merge/
│       └── mod.rs            # Layer merge functions
└── tests/
    ├── cli_basic.rs          # Basic CLI integration tests
    ├── common/
    │   ├── fixtures.rs       # Test isolation helpers
    │   └── assertions.rs     # Custom assertions
    └── [other test files]
```

### Desired Codebase Tree (Additions Only)

```bash
# No new files needed - diff.rs already exists
# Enhancements needed in existing file:
src/commands/diff.rs
  ├── diff_workspace_vs_layer()      # ENHANCE: Show actual diff, not just file list
  └── diff_workspace_vs_workspace_active()  # COMPLETE: Currently placeholder
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2 diff behavior nuances

// 1. diff_tree_to_workdir vs diff_tree_to_workdir_with_index
// Use diff_tree_to_workdir_with_index to account for staged changes
let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(opts))?;
// NOT: diff_tree_to_workdir() which ignores staging

// 2. Layer reference paths require active mode/scope context
let ref_path = layer.ref_path(
    context.mode.as_deref(),  // Required for mode-* layers
    context.scope.as_deref(),  // Required for *scope* layers
    context.project.as_deref() // Required for *project layers
);

// 3. Tree comparison fails if layer has no commits
// Always handle git2::Error when finding references
match repo.find_reference(&ref_path) {
    Ok(r) => r.peel_to_tree()?,
    Err(_) => return Err(JinError::Other(format!("Layer {} has no commits", layer))),
}

// 4. ANSI color codes in diff output
// Green for additions, Red for deletions (universal convention)
match origin {
    '+' => print!("\x1b[32m+{}\x1b[0m", content),  // Green
    '-' => print!("\x1b[31m-{}\x1b[0m", content),  // Red
    // ...
}

// 5. Layer name parsing is case-sensitive and requires exact match
// Valid names: global-base, mode-base, mode-scope, mode-scope-project,
//              mode-project, scope-base, project-base, user-local, workspace-active
```

## Implementation Blueprint

### Data Models

No new data models needed. The diff command uses existing structures:

```rust
// From src/cli/args.rs (already exists)
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// First layer to compare
    pub layer1: Option<String>,

    /// Second layer to compare
    pub layer2: Option<String>,

    /// Show staged changes
    #[arg(long)]
    pub staged: bool,
}

// From src/core/layer.rs (already exists)
pub enum Layer {
    GlobalBase,
    ModeBase,
    ModeScope,
    ModeScopeProject,
    ModeProject,
    ScopeBase,
    ProjectBase,
    UserLocal,
    WorkspaceActive,
}

// From src/staging/workspace.rs (already exists)
pub struct WorkspaceMetadata {
    pub timestamp: String,
    pub applied_layers: Vec<String>,
    pub files: HashMap<PathBuf, String>,  // path -> content hash
}
```

### Implementation Tasks

```yaml
# NOTE: P4.M5.T1.S1 "Wire DiffCommand" is marked COMPLETE
# The basic CLI wiring and four-mode structure exist in src/commands/diff.rs
# These tasks complete the remaining gaps

Task 1: COMPLETE diff_workspace_vs_workspace_active() function
  - LOCATION: src/commands/diff.rs:181-192
  - CURRENT: Placeholder with "(Workspace diff not yet fully implemented)"
  - IMPLEMENT: Actual diff between workspace and workspace-active
  - APPROACH:
    1. Load WorkspaceMetadata to get last applied state
    2. Perform merge_layers() to get current workspace-active content
    3. Compare actual workspace files to merged content
    4. Display unified diff for changed files
  - PATTERN: Use apply.rs:245-272 check_workspace_dirty() as reference
  - DEPENDENCIES: merge::merge_layers, staging::WorkspaceMetadata

Task 2: ENHANCE diff_workspace_vs_layer() function
  - LOCATION: src/commands/diff.rs:141-178
  - CURRENT: Only lists files in layer, does not show actual diff
  - IMPLEMENT: Full diff comparing workspace files to layer content
  - APPROACH:
    1. Get tree for the specified layer
    2. Iterate through layer files
    3. Read workspace file content if exists
    4. Generate diff using git2::diff_buffers() or similar
    5. Print colored unified diff
  - PATTERN: Follow diff_layers() print_diff() approach (lines 195-213)
  - GOTCHA: Handle files that exist in workspace but not in layer

Task 3: VERIFY layer name parsing completeness
  - LOCATION: src/commands/diff.rs:216-233 (parse_layer_name function)
  - VERIFY: All 9 layer types are handled
  - VALIDATE: Error message lists all valid layer names
  - TEST: Add test for each layer type

Task 4: CREATE integration tests for all diff modes
  - LOCATION: tests/cli_diff.rs (new file)
  - IMPLEMENT: Tests for each of the 4 diff modes
  - FOLLOW: tests/cli_basic.rs pattern
  - TESTS:
    1. test_diff_staged_empty - No staged changes
    2. test_diff_staged_with_files - Show staged files
    3. test_diff_layers - Compare two layers
    4. test_diff_workspace_vs_layer - Workspace vs layer comparison
    5. test_diff_default - Workspace vs workspace-active
    6. test_diff_invalid_layer - Error on unknown layer
    7. test_diff_layer_no_commits - Error on layer with no commits
  - FIXTURES: Use setup_test_repo() from tests/common/fixtures.rs
  - ASSERTIONS: Use predicates::str::contains for output validation

Task 5: VERIFY color output handling
  - LOCATION: src/commands/diff.rs:195-213 (print_diff function)
  - VERIFY: ANSI color codes are correct
  - VALIDATE: Colors are green (+), red (-), cyan (@@ hunk headers)
  - TEST: Check output contains \x1b[31m (red) and \x1b[32m (green)

Task 6: VERIFY error handling completeness
  - SCENARIOS:
    1. Not initialized - JinError::NotInitialized
    2. Layer has no commits - JinError::Other with descriptive message
    3. Invalid layer name - JinError::Other with list of valid layers
  - VERIFY: All error paths return Result<> with specific errors
```

### Implementation Patterns & Key Details

```rust
// ============================================================================
// PATTERN 1: Diff Command Mode Dispatch (from src/commands/diff.rs:14-46)
// ============================================================================
pub fn execute(args: DiffArgs) -> Result<()> {
    let context = ProjectContext::load()?;
    let repo = JinRepo::open_or_create()?;

    // Determine diff mode based on arguments
    if args.staged {
        show_staged_diff(git_repo, &context)?;
    } else if let (Some(layer1), Some(layer2)) = (&args.layer1, &args.layer2) {
        let layer1 = parse_layer_name(layer1)?;
        let layer2 = parse_layer_name(layer2)?;
        diff_layers(git_repo, layer1, layer2, &context)?;
    } else if let Some(layer_name) = &args.layer1 {
        let layer = parse_layer_name(layer_name)?;
        diff_workspace_vs_layer(git_repo, layer, &context)?;
    } else {
        diff_workspace_vs_workspace_active(git_repo, &context)?;
    }
    Ok(())
}

// ============================================================================
// PATTERN 2: Layer vs Layer Diff (from src/commands/diff.rs:87-138)
// ============================================================================
fn diff_layers(repo: &git2::Repository, layer1: Layer, layer2: Layer, context: &ProjectContext) -> Result<()> {
    // Get ref paths with context
    let ref1 = layer1.ref_path(context.mode.as_deref(), context.scope.as_deref(), context.project.as_deref());
    let ref2 = layer2.ref_path(context.mode.as_deref(), context.scope.as_deref(), context.project.as_deref());

    // Peel references to trees (handle missing commits)
    let tree1 = repo.find_reference(&ref1)?.peel_to_tree()
        .map_err(|_| JinError::Other(format!("Layer {} has no commits", layer1)))?;
    let tree2 = repo.find_reference(&ref2)?.peel_to_tree()
        .map_err(|_| JinError::Other(format!("Layer {} has no commits", layer2)))?;

    // Configure diff options
    let mut opts = DiffOptions::new();
    opts.context_lines(3);  // Standard git context

    // Generate diff
    let diff = repo.diff_tree_to_tree(Some(&tree1), Some(&tree2), Some(&mut opts))?;

    // Print header and colored diff
    if diff.deltas().count() == 0 {
        println!("No differences between {} and {}", layer1, layer2);
    } else {
        println!("diff --jin a/{} b/{}", layer1, layer2);
        println!();
        print_diff(&diff)?;
    }

    Ok(())
}

// ============================================================================
// PATTERN 3: Colored Diff Output (from src/commands/diff.rs:195-213)
// ============================================================================
fn print_diff(diff: &git2::Diff) -> Result<()> {
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let content = std::str::from_utf8(line.content()).unwrap_or("<binary>");

        // Apply ANSI colors based on line type
        match origin {
            '+' => print!("\x1b[32m+{}\x1b[0m", content),  // Green for additions
            '-' => print!("\x1b[31m-{}\x1b[0m", content),  // Red for deletions
            ' ' => print!(" {}", content),                 // No color for context
            'F' => print!("--- {}", content),              // File header (old)
            'T' => print!("+++ {}", content),              // File header (new)
            'H' => print!("\x1b[36m@@ {}\x1b[0m", content), // Cyan for hunk headers
            _ => print!("{}", content),
        }
        true  // Continue iteration
    })?;
    Ok(())
}

// ============================================================================
// PATTERN 4: Workspace vs Layer Diff (ENHANCEMENT NEEDED)
// ============================================================================
fn diff_workspace_vs_layer(repo: &git2::Repository, layer: Layer, context: &ProjectContext) -> Result<()> {
    let ref_path = layer.ref_path(context.mode.as_deref(), context.scope.as_deref(), context.project.as_deref());
    let tree = repo.find_reference(&ref_path)?.peel_to_tree()?;

    println!("Comparing workspace vs {}", layer);
    println!();

    // ENHANCEMENT: Instead of just listing files, show actual diff
    // For each file in the layer:
    // 1. Read blob content from tree
    // 2. Read workspace file content if exists
    // 3. Generate diff between the two
    // 4. Print colored unified diff

    // CURRENT (incomplete):
    tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            println!("  {}", entry.name().unwrap_or("<unnamed>"));
        }
        git2::TreeWalkResult::Ok
    })?;

    // ENHANCED (proposed):
    for entry in layer_files {
        let layer_content = repo.read_file_from_tree(tree, entry.path)?;
        let workspace_content = std::fs::read(entry.path).ok();

        if let Some(ws_content) = workspace_content {
            let diff = diff_buffers(&layer_content, &ws_content)?;
            print_diff(&diff)?;
        } else {
            println!("Only in layer: {}", entry.path);
        }
    }

    Ok(())
}

// ============================================================================
// PATTERN 5: Workspace vs Workspace-Active Diff (COMPLETION NEEDED)
// ============================================================================
fn diff_workspace_vs_workspace_active(repo: &git2::Repository, context: &ProjectContext) -> Result<()> {
    // Load current workspace metadata
    let metadata = WorkspaceMetadata::load()?;

    // Merge all applicable layers to get workspace-active content
    let config = LayerMergeConfig {
        layers: get_applicable_layers(context.mode.as_deref(), context.scope.as_deref(), context.project.as_deref()),
        mode: context.mode.clone(),
        scope: context.scope.clone(),
        project: context.project.clone(),
    };
    let merged = merge_layers(&config, repo)?;

    // Compare each merged file to actual workspace file
    for (path, merged_file) in &merged.merged_files {
        let merged_content = serialize_merged_content(&merged_file.content, merged_file.format)?;
        let workspace_content = std::fs::read(path).ok();

        match (workspace_content, merged_content) {
            (Some(ws_bytes), merged_text) => {
                let ws_text = String::from_utf8(ws_bytes)?;
                if ws_text != merged_text {
                    // Generate and print diff
                    println!("--- a/{}", path.display());
                    println!("+++ b/{}", path.display());
                    // ... generate diff
                }
            }
            (None, merged_text) => {
                println!("Only in workspace-active: {}", path.display());
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Integration Points

```yaml
CLI:
  - command: jin diff
  - args: [layer1] [layer2] [--staged]
  - defined in: src/cli/mod.rs:57 (Commands::Diff)
  - args struct: src/cli/args.rs:88-100 (DiffArgs)

STAGING SYSTEM:
  - load: StagingIndex::load() from src/staging/index.rs
  - iterate: staging.entries() for staged files
  - check: file hash comparison for "modified since staging"

LAYER SYSTEM:
  - parse: parse_layer_name() for string -> Layer conversion
  - paths: layer.ref_path() for Git reference resolution
  - context: mode/scope/project required for some layers

MERGE ENGINE:
  - function: merge_layers() from src/merge/mod.rs
  - config: LayerMergeConfig with applicable layers
  - result: LayerMergeResult with merged_files HashMap

GIT OPERATIONS:
  - repo: JinRepo wrapper from src/git/lib.rs
  - diff: git2::Repository::diff_tree_to_tree()
  - tree: git2::Tree walking for file enumeration
  - blob: git2::Blob for file content
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Check diff.rs for syntax and style issues
cargo check --bin jin 2>&1 | grep -A 5 "diff.rs"

# Run clippy for linting
cargo clippy --bin jin 2>&1 | grep -A 5 "diff.rs"

# Format check
cargo fmt -- --check src/commands/diff.rs

# Expected: Zero errors, zero warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run embedded unit tests in diff.rs
cargo test --bin jin diff::tests --verbose

# Expected output:
# test test_parse_layer_name ... ok
# test test_execute_not_initialized ... ok
# test test_execute_staged_empty ... ok

# Run all command tests
cargo test --bin jin commands:: 2>&1 | tail -20

# Expected: All tests pass
```

### Level 3: Integration Tests (System Validation)

```bash
# Run diff-specific integration tests (to be created)
cargo test --test cli_diff --verbose

# Manual testing scenarios:

# Scenario 1: Initialize and verify empty diff
cd /tmp/test_jin_diff
mkdir test1 && cd test1
jin init
jin diff
# Expected: "Comparing workspace vs workspace-active" or similar

# Scenario 2: Create mode and test layer diff
jin mode create dev
echo '{"test": true}' > config.json
jin add config.json --mode
jin commit -m "Initial config"
jin diff mode-base mode-base
# Expected: "No differences between mode-base and mode-base"

# Scenario 3: Test staged diff
echo '{"test": false}' > config.json
jin add config.json --mode
jin diff --staged
# Expected: Shows config.json -> mode-base

# Scenario 4: Test invalid layer
jin diff invalid-layer
# Expected: Error with list of valid layer names
```

### Level 4: Diff-Specific Validation

```bash
# Color output validation (requires terminal with color support)
jin diff mode-base mode-project 2>&1 | cat -A
# Look for: \x1b[31m (red), \x1b[32m (green), \x1b[0m (reset)

# Binary file handling
echo -ne '\x00\x01\x02\x03' > binary.bin
jin add binary.bin --mode
jin diff --staged
# Expected: Shows <binary> marker, not raw bytes

# Empty layer handling
jin diff mode-base mode-base
# Expected: "No differences between mode-base and mode-base"

# Large file performance
dd if=/dev/zero of=large.json bs=1M count=10
jin add large.json --mode
jin diff --staged
# Expected: Completes in reasonable time (<5 seconds)
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --bin jin diff::tests`
- [ ] No clippy warnings: `cargo clippy --bin jin`
- [ ] No formatting issues: `cargo fmt -- --check src/commands/diff.rs`
- [ ] Integration tests pass: `cargo test --test cli_diff`

### Feature Validation

- [ ] `jin diff --staged` shows staged files with target layers
- [ ] `jin diff layer1 layer2` shows unified diff between layers
- [ ] `jin diff layer` shows workspace vs layer comparison
- [ ] `jin diff` shows workspace vs workspace-active comparison
- [ ] Invalid layer names produce helpful error messages
- [ ] Layers with no commits produce clear error messages
- [ ] Colored output displays correctly in terminal
- [ ] Binary files handled gracefully

### Code Quality Validation

- [ ] Follows existing command patterns (status.rs, apply.rs)
- [ ] Error handling uses JinError variants appropriately
- [ ] Function names follow snake_case convention
- [ ] Documentation comments present on public functions
- [ ] No unwrap() or expect() in production code paths

### Documentation & Deployment

- [ ] Help text available: `jin diff --help`
- [ ] Error messages are user-friendly and actionable
- [ ] Layer name list includes all 9 valid layer types
- [ ] Integration tests cover all four diff modes
- [ ] Unit tests cover edge cases (empty, missing, invalid)

---

## Anti-Patterns to Avoid

- **Don't** use `diff_tree_to_workdir()` - use `diff_tree_to_workdir_with_index()` to account for staged changes
- **Don't** panic on missing layers - return `JinError::Other` with descriptive message
- **Don't** hardcode layer names - use the `parse_layer_name()` function
- **Don't** skip color codes - users expect green additions, red deletions
- **Don't** ignore binary files - display `<binary>` marker
- **Don't** leave placeholder implementations - complete all four diff modes
- **Don't** use unwrap() for git operations - handle errors gracefully
- **Don't** forget to pass mode/scope/project context to `layer.ref_path()`

## Status Summary

**Current Implementation Status:**
- ✅ CLI wiring complete (P4.M5.T1.S1)
- ✅ Four-mode structure implemented
- ✅ Layer name parsing works
- ✅ Colored diff output functional
- ⚠️ `diff_workspace_vs_layer` only shows file list (needs enhancement)
- ⚠️ `diff_workspace_vs_workspace_active` is placeholder (needs completion)
- ❌ Integration tests not yet created

**Completion Priority:**
1. **High**: Complete `diff_workspace_vs_workspace_active()` - core functionality
2. **Medium**: Enhance `diff_workspace_vs_layer()` - better UX
3. **High**: Create integration tests - validation and regression prevention
4. **Low**: Add diff output format options (--stat, --name-only, etc.)

**Confidence Score**: 8/10 for one-pass implementation success

All necessary context, patterns, and research are provided in this PRP. The implementation follows established codebase patterns, and all dependencies are documented.
