# Product Requirement Prompt: Implement 3-Way Merge for Divergent Histories (P2.M1.T2)

---

## Goal

**Feature Goal**: Implement 3-way merge in the `jin pull` command to handle divergent layer histories, using the existing text merge infrastructure and `.jinmerge` workflow from Phase 1.

**Deliverable**: Enhanced pull command that performs 3-way merge when `MergeType::Divergent` is detected, creating `.jinmerge` files for conflicts and writing merged content to layers.

**Success Definition**:
- Pull command successfully performs 3-way merge on divergent layers
- Clean merges automatically write merged content to the layer
- Conflicts create `.jinmerge` files following the Phase 1 workflow
- Existing fast-forward merge behavior is preserved
- All integration tests pass

## Why

- **Completes the PRD requirement for P2.M1**: Milestone 2.1 requires 3-way merge implementation in the pull command
- **Enables distributed collaboration**: Users can work on the same layer concurrently and merge their changes
- **Leverages existing infrastructure**: The text merge system (`diffy`) and `.jinmerge` workflow are already implemented
- **Maintains consistency**: Uses the same conflict resolution flow as `jin apply`

## What

When `jin pull` detects divergent histories (via `detect_merge_type()` returning `MergeType::Divergent`), it will:

1. Find the merge base commit between local and remote layer refs
2. Extract file contents from three commits: base, local, and remote
3. Perform 3-way text merge using the existing `text_merge()` function
4. For clean merges: Write merged content directly to the layer
5. For conflicts: Create `.jinmerge` files following the Phase 1 workflow
6. Update the layer ref to point to the new merge commit

### Success Criteria

- [ ] Divergent layers trigger 3-way merge instead of being skipped
- [ ] Clean merges automatically update the layer with merged content
- [ ] Conflicts create `.jinmerge` files that can be resolved with `jin resolve`
- [ ] Fast-forward merges still work as before
- [ ] Integration tests cover: fast-forward, clean 3-way merge, and conflicted 3-way merge

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

Yes. This PRP provides:
- Exact file paths for all code to be modified
- Complete code patterns to follow (with line numbers)
- Specific function signatures and data structures
- Test patterns and validation commands
- External references for understanding 3-way merge algorithms

### Documentation & References

```yaml
# MUST READ - Core Merge Infrastructure
- file: src/merge/text.rs
  why: Contains text_merge() function for 3-way merging using diffy crate
  pattern: pub fn text_merge(base: &str, ours: &str, theirs: &str) -> Result<TextMergeResult>
  gotcha: diffy::merge() returns Ok(String) for clean merge, Err(String) for conflicts (NOT an error!)

- file: src/merge/jinmerge.rs
  why: Contains .jinmerge file format and generation for conflict workflow
  pattern: JinMergeConflict::from_text_merge() creates conflict structure
  critical: Use exact marker constants MARKER_START, MARKER_SEP, MARKER_END for Git compatibility

- file: src/git/merge.rs
  why: Contains MergeType enum and detect_merge_type() function from P2.M1.T1
  pattern: Use graph_ahead_behind() to determine merge type
  gotcha: Always check OID equality first before calling graph_ahead_behind (expensive)

- file: src/git/tree.rs
  why: Contains TreeOps trait for reading file contents from commits
  pattern: repo.read_file_from_tree(tree_oid, path) for two-step content extraction
  gotcha: Must check ref_exists() before resolve_ref() to avoid errors

- file: src/git/transaction.rs
  why: Contains LayerTransaction for atomic multi-layer updates
  pattern: tx.add_layer_update() to queue updates, tx.commit() to apply atomically
  critical: Always save transaction log after each update for crash safety

- file: src/commands/pull.rs
  why: Main entry point for pull command - where divergent handling is added
  pattern: Lines 74-81 show where Divergent case currently just prints warning
  integration: Add merge logic here, follow same pattern as FastForward case

- file: src/core/layer.rs
  why: Contains Layer enum with ref_path() method for computing layer ref paths
  pattern: layer.ref_path(mode, scope, project) returns full Git ref path

- url: https://docs.rs/diffy/0.4/diffy/fn.merge.html
  why: Documentation for diffy::merge() function used for 3-way text merging
  critical: Understand return value semantics (Ok = clean, Err = conflicts)

- url: https://github.com/GitoxideLabs/gitoxide/tree/main/gix-merge
  why: Reference implementation of 3-way merge in pure Rust
  insight: Study how they handle merge base computation and tree merging

- url: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
  why: Detailed explanation of 3-way merge algorithm and diff3 format
  insight: Explains why base version is important for conflict detection
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── pull.rs          # MODIFY: Add 3-way merge for Divergent case (lines 74-81)
│   ├── apply.rs         # REFERENCE: Shows .jinmerge workflow (lines 218-268)
│   └── resolve.rs       # REFERENCE: Shows conflict resolution workflow
├── git/
│   ├── merge.rs         # REFERENCE: Has detect_merge_type() and MergeType enum
│   ├── tree.rs          # USE: TreeOps trait for file content extraction
│   ├── transaction.rs   # USE: LayerTransaction for atomic layer updates
│   ├── objects.rs       # USE: ObjectOps trait for blob creation
│   └── refs.rs          # USE: RefOps trait for ref resolution
├── merge/
│   ├── text.rs          # USE: text_merge() function for 3-way text merging
│   ├── jinmerge.rs      # USE: .jinmerge file format and conflict generation
│   └── layer.rs         # REFERENCE: Layer merge orchestration patterns
├── core/
│   ├── layer.rs         # USE: Layer enum with ref_path() method
│   └── error.rs         # USE: Error types for merge operations
└── staging/
    └── index.rs         # REFERENCE: Staging index patterns

tests/
├── sync_workflow.rs     # REFERENCE: Shows pull/fetch test patterns
└── conflict_workflow.rs # REFERENCE: Shows .jinmerge conflict test patterns
```

### Desired Codebase Tree with Files to be Added

```bash
# No new files - modifications only to existing files
src/
├── commands/
│   └── pull.rs          # MODIFY: Add perform_three_way_merge() function and integration
└── git/
    └── merge.rs         # MODIFY: Add find_merge_base() helper function

tests/
└── pull_merge.rs        # NEW: Integration tests for 3-way merge in pull
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: diffy::merge() return semantics are inverted from typical Rust patterns
// - Ok(String) = clean merge result (success, no conflicts)
// - Err(String) = content WITH conflict markers (NOT an error, contains conflicts!)
match diffy::merge(base, ours, theirs) {
    Ok(merged) => TextMergeResult::Clean(merged),      // No conflicts
    Err(conflict_content) => {                          // Has conflict markers
        // This is EXPECTED for conflicts, not an error!
        TextMergeResult::Conflict { content: conflict_content, conflict_count }
    }
}

// CRITICAL: git2 merge_base() returns merge base OID, not commit object
// Must find_commit() separately to access tree_id()
let base_commit = repo.inner().find_commit(base_oid)?;
let base_tree = base_commit.tree_id();

// CRITICAL: Always check ref_exists() before resolve_ref()
// Unresolved refs cause panics in resolve_ref()
if repo.ref_exists(&ref_path) {
    let oid = repo.resolve_ref(&ref_path)?;
} else {
    // Handle missing ref gracefully
}

// CRITICAL: LayerTransaction uses two-phase commit with persistent log
// Log saved after each update for crash safety
tx.add_layer_update(...)?;  // Saves log automatically
tx.commit()?;                // Deletes log on success

// CRITICAL: .jinmerge markers must be exactly 7 characters for Git compatibility
const MARKER_START: &str = "<<<<<<< ";  // 8 chars (7 + space)
const MARKER_SEP: &str = "=======";     // 7 chars
const MARKER_END: &str = ">>>>>>> ";    // 8 chars (7 + space)

// GOTCHA: graph_ahead_behind() is expensive - check OID equality first
if local_oid == remote_oid {
    return Ok(MergeType::UpToDate);
}
let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;

// GOTCHA: Commit requires parent commits as &[&Commit] with proper lifetime
// Use local array and transmute for single parent (see src/git/merge.rs:141-147)
let parent_array: [&git2::Commit<'_>; 1] = unsafe { std::mem::transmute([&parent_commit]) };
repo.commit(None, &sig, &sig, message, &tree, &parent_array)?;

// GOTCHA: When creating merge commit with two parents, order matters:
// - First parent = "our" commit (local)
// - Second parent = "their" commit (remote)
let parents: [&git2::Commit<'_>; 2] = unsafe { std::mem::transmute([&local_commit, &remote_commit]) };
repo.commit(None, &sig, &sig, message, &tree, &parents)?;
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models required. Using existing structures:

```rust
// From src/git/merge.rs (lines 15-27)
pub enum MergeType {
    UpToDate,
    FastForward,
    LocalAhead,
    Divergent,  // This triggers our 3-way merge
}

// From src/merge/text.rs (lines 26-37)
pub enum TextMergeResult {
    Clean(String),
    Conflict { content: String, conflict_count: usize },
}

// From src/merge/jinmerge.rs (lines 54-68, 74-80)
pub struct JinMergeRegion {
    pub layer1_ref: String,
    pub layer1_content: String,
    pub layer2_ref: String,
    pub layer2_content: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct JinMergeConflict {
    pub file_path: PathBuf,
    pub conflicts: Vec<JinMergeRegion>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD find_merge_base() helper to src/git/merge.rs
  - IMPLEMENT: find_merge_base(repo, local_oid, remote_oid) -> Result<Oid>
  - FOLLOW: git2::merge_base() pattern (line 115 in existing code)
  - ERROR HANDLING: Treat unrelated histories (no merge base) as divergent
  - PATTERN: Use repo.inner().merge_base(local_oid, remote_oid)?
  - PLACEMENT: Add after detect_merge_type_with_base() function (around line 120)
  - EXPORT: Add to pub use in src/git/mod.rs if needed by pull.rs

Task 2: CREATE perform_three_way_merge() function in src/commands/pull.rs
  - IMPLEMENT: fn perform_three_way_merge(jin_repo, layer, context, local_oid, remote_oid) -> Result<MergeOutcome>
  - MERGE BASE: Call find_merge_base() to get common ancestor
  - FILE DISCOVERY: Use repo.list_tree_files() to find all files in any of the three trees
  - PER-FILE MERGE: For each file, extract base/local/remote content and call text_merge()
  - CONFLICT HANDLING: Use JinMergeConflict::from_text_merge() for conflict files
  - TREE BUILDING: Create new tree with merged files using repo.create_tree_from_paths()
  - COMMIT CREATION: Create merge commit with two parents (local, remote)
  - PLACEMENT: Add as private function after parse_ref_path() (around line 225)
  - RETURN TYPE: MergeOutcome enum (define locally) indicating success/conflicts

Task 3: DEFINE MergeOutcome enum in src/commands/pull.rs
  - IMPLEMENT: enum MergeOutcome { Clean(Oid), Conflicts { merged_oid: Oid, conflict_files: Vec<PathBuf> } }
  - CLEAN: All files merged successfully, new commit OID
  - CONFLICTS: Merge commit created, but some files have .jinmerge conflicts
  - PLACEMENT: Define before perform_three_way_merge() function (around line 220)

Task 4: INTEGRATE 3-way merge into pull command execution loop
  - MODIFY: execute() function in src/commands/pull.rs (lines 74-81)
  - REPLACE: Divergent placeholder with actual 3-way merge call
  - BRANCH: Match on MergeOutcome to handle clean vs conflicted merges
  - CLEAN OUTPUT: Print success message similar to FastForward case
  - CONFLICT OUTPUT: Print message about .jinmerge files and next steps
  - TRANSACTION: Use existing tx.add_layer_update() pattern for atomic update
  - PRESERVE: All existing behavior for UpToDate, FastForward, LocalAhead cases

Task 5: CREATE integration tests in tests/pull_merge.rs
  - IMPLEMENT: test_pull_fast_forward_still_works()
    - SETUP: Create local + remote repos, commit to remote, pull
    - ASSERT: Layer updated, no conflicts
    - FOLLOW: Pattern from tests/sync_workflow.rs
  - IMPLEMENT: test_pull_divergent_clean_merge()
    - SETUP: Create divergent commits with non-overlapping changes
    - ASSERT: Layer merged with combined content
  - IMPLEMENT: test_pull_divergent_with_conflicts()
    - SETUP: Create divergent commits with overlapping changes
    - ASSERT: .jinmerge file created, layer ref updated to merge commit
    - ASSERT: User can resolve with jin resolve
  - FIXTURES: Use setup_jin_with_remote() from tests/common/fixtures.rs
  - PLACEMENT: Create new file tests/pull_merge.rs

Task 6: ADD helper module export to src/git/mod.rs (if needed)
  - CHECK: If find_merge_base() needs to be public for pull.rs to access
  - ADD: pub use merge::find_merge_base; to exports (around line 19)
  - ALTERNATIVE: Keep private and access through merge:: module path
```

### Implementation Patterns & Key Details

```rust
// ============================================================================
// PATTERN: Finding merge base between two commits
// ============================================================================
// Location: src/git/merge.rs (new function after line 120)

pub fn find_merge_base(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<Oid> {
    // Use git2's merge_base to find common ancestor
    match repo.inner().merge_base(local_oid, remote_oid) {
        Ok(base_oid) => Ok(base_oid),
        // No merge base means unrelated histories
        // In this case, treat empty content as base for merge
        Err(_) => {
            // Create empty tree as base for unrelated histories
            let empty_tree = repo.inner().treebuilder(None)?.write()?;
            // Could also create an empty commit, but empty tree OID is simpler
            // The text merge will handle empty base correctly
            Ok(empty_tree)
        }
    }
}

// ============================================================================
// PATTERN: Per-file 3-way merge in pull context
// ============================================================================
// Location: src/commands/pull.rs (inside perform_three_way_merge function)

use crate::merge::text::{text_merge, TextMergeResult};
use crate::merge::jinmerge::JinMergeConflict;
use crate::git::TreeOps;

fn perform_three_way_merge(
    jin_repo: &JinRepo,
    layer: Layer,
    mode: Option<&str>,
    scope: Option<&str>,
    project: Option<&str>,
    local_oid: Oid,
    remote_oid: Oid,
) -> Result<MergeOutcome> {
    // Step 1: Find merge base
    let base_oid = crate::git::merge::find_merge_base(jin_repo, local_oid, remote_oid)?;

    // Step 2: Get commit objects for all three
    let base_commit = jin_repo.inner().find_commit(base_oid)?;
    let local_commit = jin_repo.inner().find_commit(local_oid)?;
    let remote_commit = jin_repo.inner().find_commit(remote_oid)?;

    // Step 3: Collect all unique files from all three trees
    let mut all_files = std::collections::HashSet::new();
    for tree_oid in [base_commit.tree_id(), local_commit.tree_id(), remote_commit.tree_id()] {
        for file in jin_repo.list_tree_files(tree_oid)? {
            all_files.insert(std::path::PathBuf::from(file));
        }
    }

    // Step 4: Merge each file
    let mut merged_files = Vec::new();  // (path, blob_oid) for tree building
    let mut conflict_files = Vec::new();  // Paths with conflicts

    for file_path in all_files {
        // Extract contents from base, local, remote
        let base_content = extract_file_content(jin_repo, base_commit.tree_id(), &file_path)?;
        let local_content = extract_file_content(jin_repo, local_commit.tree_id(), &file_path)?;
        let remote_content = extract_file_content(jin_repo, remote_commit.tree_id(), &file_path)?;

        // Perform 3-way merge using existing text_merge()
        match text_merge(&base_content, &local_content, &remote_content)? {
            TextMergeResult::Clean(merged) => {
                // Create blob with merged content
                let blob_oid = jin_repo.create_blob(merged.as_bytes())?;
                merged_files.push((file_path.display().to_string(), blob_oid));
            }
            TextMergeResult::Conflict { content, .. } => {
                // Create .jinmerge file for this conflict
                let local_ref = layer.ref_path(mode, scope, project);
                let remote_ref = format!("origin/{}", local_ref);  // Or compute actual remote ref

                let merge_conflict = JinMergeConflict::from_text_merge(
                    file_path.clone(),
                    local_ref,
                    local_content,
                    remote_ref,
                    remote_content,
                );

                // Write .jinmerge file to workspace
                let merge_path = JinMergeConflict::merge_path_for_file(&file_path);
                merge_conflict.write_to_file(&merge_path)?;

                // For now, use local version in the merge
                // TODO: Could ask user to choose, or mark as conflicted
                let blob_oid = jin_repo.create_blob(local_content.as_bytes())?;
                merged_files.push((file_path.display().to_string(), blob_oid));
                conflict_files.push(file_path);
            }
        }
    }

    // Step 5: Create merge tree
    let merge_tree_oid = jin_repo.create_tree_from_paths(&merged_files)?;

    // Step 6: Create merge commit with two parents
    let sig = jin_repo.inner().signature()?;
    let message = format!("Merge remote changes into {}", layer.ref_path(mode, scope, project));

    // CRITICAL: Parent order matters! Local first, then remote
    let parents: [&git2::Commit<'_>; 2] = unsafe {
        std::mem::transmute([&local_commit, &remote_commit])
    };

    let merge_commit_oid = jin_repo.inner().commit(
        None,
        &sig,
        &sig,
        &message,
        &jin_repo.inner().find_tree(merge_tree_oid)?,
        &parents,
    )?;

    // Step 7: Return outcome
    if conflict_files.is_empty() {
        Ok(MergeOutcome::Clean(merge_commit_oid))
    } else {
        Ok(MergeOutcome::Conflicts {
            merged_oid: merge_commit_oid,
            conflict_files,
        })
    }
}

// ============================================================================
// PATTERN: Extract file content from tree, return empty string if not found
// ============================================================================

fn extract_file_content(repo: &JinRepo, tree_oid: git2::Oid, path: &std::path::Path) -> Result<String> {
    match repo.read_file_from_tree(tree_oid, path) {
        Ok(content) => Ok(String::from_utf8_lossy(&content).to_string()),
        Err(_) => Ok(String::new()),  // File doesn't exist in this tree
    }
}

// ============================================================================
// PATTERN: Integration into pull command execution loop
// ============================================================================
// Location: src/commands/pull.rs (modify lines 74-81)

// Inside execute() function, within the update loop:
for (ref_path, update_info) in &updates {
    match update_info.merge_type {
        MergeType::UpToDate => {
            continue;
        }
        MergeType::FastForward => {
            // EXISTING CODE: Keep as-is
            tx.add_layer_update(...)?;
            println!("  ✓ {}: Updated (fast-forward)", format_ref_path(ref_path));
            merge_count += 1;
        }
        MergeType::LocalAhead => {
            // EXISTING CODE: Keep as-is
            println!("  − {}: Local is ahead of remote", format_ref_path(ref_path));
        }
        MergeType::Divergent => {
            // NEW CODE: Implement 3-way merge
            match perform_three_way_merge(
                &jin_repo,
                update_info.layer,
                update_info.mode.as_deref(),
                update_info.scope.as_deref(),
                update_info.project.as_deref(),
                update_info.local_oid.unwrap(),  // Safe because divergent means local exists
                update_info.remote_oid,
            )? {
                MergeOutcome::Clean(merge_oid) => {
                    tx.add_layer_update(
                        update_info.layer,
                        update_info.mode.as_deref(),
                        update_info.scope.as_deref(),
                        update_info.project.as_deref(),
                        merge_oid,
                    )?;
                    println!("  ✓ {}: Merged (3-way)", format_ref_path(ref_path));
                    merge_count += 1;
                }
                MergeOutcome::Conflicts { merged_oid, conflict_files } => {
                    tx.add_layer_update(
                        update_info.layer,
                        update_info.mode.as_deref(),
                        update_info.scope.as_deref(),
                        update_info.project.as_deref(),
                        merged_oid,
                    )?;
                    println!("  ! {}: Merged with {} conflicts", format_ref_path(ref_path), conflict_files.len());
                    for file in conflict_files {
                        println!("      - {} has conflicts (.jinmerge created)", file.display());
                    }
                    merge_count += 1;
                }
            }
        }
    }
}

// ============================================================================
// PATTERN: MergeOutcome enum definition
// ============================================================================
// Location: src/commands/pull.rs (define before perform_three_way_merge, around line 220)

#[derive(Debug)]
enum MergeOutcome {
    /// Clean merge with no conflicts
    Clean(git2::Oid),
    /// Merge completed but has conflicts requiring resolution
    Conflicts {
        /// The merge commit OID (already created)
        merged_oid: git2::Oid,
        /// Files that have conflicts (with .jinmerge files)
        conflict_files: Vec<std::path::PathBuf>,
    },
}
```

### Integration Points

```yaml
PULL_COMMAND:
  - file: src/commands/pull.rs
  - modify: execute() function, lines 74-81 (Divergent case)
  - pattern: Replace placeholder println!() with perform_three_way_merge() call
  - preserve: All other merge type handling (UpToDate, FastForward, LocalAhead)

GIT_MERGE_MODULE:
  - file: src/git/merge.rs
  - add: find_merge_base() helper function
  - export: Add to pub use in src/git/mod.rs if needed

TEXT_MERGE_INFRASTRUCTURE:
  - use: src/merge/text.rs::text_merge()
  - note: Already handles diffy crate integration

JINMERGE_WORKFLOW:
  - use: src/merge/jinmerge.rs::JinMergeConflict
  - pattern: Follow Phase 1 conflict resolution flow
  - integration: .jinmerge files compatible with jin resolve command

LAYER_TRANSACTION:
  - use: src/git/transaction.rs::LayerTransaction
  - pattern: tx.add_layer_update() for atomic layer updates
  - preserve: Existing transaction and rollback behavior
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --message-format=short 2>&1 | head -50
cargo clippy --all-targets --message-format=short 2>&1 | head -50
cargo fmt --check

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
# Common issues to watch for:
# - Missing imports (add use statements at top of file)
# - Type mismatches (check function signatures)
# - Lifetime issues (ensure proper lifetime annotations)
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test merge type detection (existing)
cargo test detect_merge_type -- --nocapture

# Test text merge functionality (existing)
cargo test text_merge -- --nocapture

# Test .jinmerge conflict parsing (existing)
cargo test parse_jinmerge -- --nocapture

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Test the new pull merge functionality
cargo test --test pull_merge -- --nocapture

# Run all integration tests to ensure no regressions
cargo test --test sync_workflow -- --nocapture
cargo test --test conflict_workflow -- --nocapture

# Expected: All tests pass, including new pull_merge tests.
# Test specific scenarios:
# 1. Fast-forward still works (regression test)
# 2. Clean 3-way merge produces merged content
# 3. Conflicted 3-way merge creates .jinmerge files
# 4. Resolution of .jinmerge files works with jin resolve
```

### Level 4: Manual End-to-End Testing

```bash
# Setup: Create two repos for testing
cd /tmp
mkdir test_jin_merge
cd test_jin_merge
jin init
mkdir .jin-remote
git init --bare .jin-remote
git remote add origin .jin-remote

# Scenario 1: Test fast-forward (baseline)
echo "version = 1" > config.toml
jin add config.toml --global
jin commit -m "Initial commit"
git push origin master
# Should work as before
jin pull

# Scenario 2: Test clean 3-way merge
# On remote:
cd /tmp/test_jin_merge_clone
git clone /tmp/test_jin_merge/test_jin_merge/.jin-remote .
jin init
echo "version = 2" > config.toml
jin add config.toml --global
jin commit -m "Remote change"
git push origin master

# On local (make divergent change):
cd /tmp/test_jin_merge
echo "debug = true" >> config.toml
jin add config.toml --global
jin commit -m "Local change"
# Now pull with divergent history
jin pull
# Should merge successfully with both changes

# Scenario 3: Test conflicting 3-way merge
# On local, make conflicting change:
echo "version = 3" > config.toml
jin add config.toml --global
jin commit -m "Conflicting local change"
# Pull remote with different version value
jin pull
# Should create config.toml.jinmerge file
# Should print message about conflicts
cat config.toml.jinmerge  # Should show conflict markers

# Resolution:
# Edit config.toml.jinmerge to resolve conflict
echo "version = 4" > config.toml
jin resolve config.toml
# Should succeed and remove .jinmerge file

# Expected outputs:
# - Fast-forward: "✓ global: Updated (fast-forward)"
# - Clean merge: "✓ global: Merged (3-way)"
# - Conflicts: "! global: Merged with 1 conflicts"
#   "- config.toml has conflicts (.jinmerge created)"
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check` returns clean
- [ ] No compilation errors: `cargo check` passes

### Feature Validation

- [ ] Fast-forward merge still works (regression test passes)
- [ ] Clean 3-way merge produces correct merged content
- [ ] Conflicted 3-way merge creates .jinmerge files
- [ ] .jinmerge files follow exact format from Phase 1
- [ ] Layer refs updated to merge commit OID
- [ ] Manual testing confirms all three scenarios work

### Code Quality Validation

- [ ] Follows existing codebase patterns (tree ops, transaction, merge detection)
- [ ] Error handling matches existing patterns (Result types, JinError variants)
- [ ] Function naming consistent with codebase (snake_case, descriptive)
- [ ] No code duplication (reuse existing text_merge, JinMergeConflict)
- [ ] Comments explain non-obvious logic (especially merge base handling)

### Integration & Compatibility

- [ ] Compatible with existing `jin resolve` command
- [ ] Compatible with LayerTransaction atomic updates
- [ ] Does not break existing pull/fetch/push workflow
- [ ] .jinmerge files use correct marker format (7 chars)
- [ ] Merge commits have correct parent order (local, then remote)

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable/function names
- [ ] Complex logic has inline comments explaining why
- [ ] User-facing messages are clear and actionable
- [ ] Error messages guide user to resolution (e.g., "run jin resolve")

---

## Anti-Patterns to Avoid

- ❌ **Don't** create new merge algorithm - use existing `text_merge()` from `src/merge/text.rs`
- ❌ **Don't** bypass LayerTransaction - use `tx.add_layer_update()` for atomic updates
- ❌ **Don't** hardcode ref paths - use `layer.ref_path(mode, scope, project)`
- ❌ **Don't** ignore the merge base - must find common ancestor for proper 3-way merge
- ❌ **Don't** treat unrelated histories as error - use empty tree/base as fallback
- ❌ **Don't** skip creating .jinmerge files - must follow Phase 1 workflow
- ❌ **Don't** forget to check `ref_exists()` before `resolve_ref()` - prevents panics
- ❌ **Don't** get diffy return semantics wrong - `Ok()` is clean, `Err()` is conflicts (NOT an error!)
- ❌ **Don't** mess up parent order in merge commit - local first, then remote
- ❌ **Don't** write tests without cleanup - use fixtures and temp directories
