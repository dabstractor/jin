# PRP: P3.M2 - Commit Pipeline

---

## Goal

**Feature Goal**: Implement the orchestrated commit flow that transforms staged entries into atomic multi-layer Git commits, using the transaction system for crash recovery and atomicity guarantees.

**Deliverable**:
1. Complete implementation of `CommitPipeline::execute()` in `src/commit/pipeline.rs`
2. Integration with `LayerTransaction` for atomic multi-layer commits
3. Integration with `StagingIndex` for reading staged entries
4. Proper tree building per layer using `ObjectOps`
5. Parent commit resolution via `RefOps`
6. Staging cleanup on successful commit
7. Comprehensive unit tests for all commit scenarios

**Success Definition**:
- `jin commit -m "message"` creates atomic commits across all affected layers
- Each layer gets its own commit with proper tree structure
- Parent commits are correctly resolved from existing layer refs
- LayerTransaction ensures atomicity (all succeed or all fail)
- Staging index is cleared after successful commit
- Recovery from interrupted commits works via `RecoveryManager`
- All tests pass: `cargo test commit::`
- `cargo check && cargo clippy && cargo test` all pass with zero errors/warnings

---

## User Persona

**Target User**: Developer using Jin to version tool-specific configuration files

**Use Case**: A developer has staged multiple files across different layers and wants to commit them atomically to Jin's phantom repository.

**User Journey**:
1. Developer stages files with `jin add .claude/config.json --mode`
2. Developer stages more files with `jin add .editorconfig --scope=language:javascript`
3. Developer runs `jin status` to review staged files
4. Developer runs `jin commit -m "Add Claude config and editorconfig"`
5. Jin groups staged entries by target layer
6. Jin creates a commit for each affected layer atomically
7. Jin clears the staging index
8. Developer receives confirmation with commit hashes

**Pain Points Addressed**:
- Atomic commits prevent partial state corruption
- Multi-layer commits happen in single command
- Crash recovery ensures no lost work
- Clear commit hashes for each layer enable debugging

---

## Why

- **PRD Requirement**: Section 6.2 states "`jin commit` is atomic across all affected layers. Partial commits are impossible."
- **PRD Invariant**: Section 25 lists "Atomic multi-layer commits" as non-negotiable
- **Foundation from P3.M1**: The staging system (complete) provides grouped entries ready for commit
- **Foundation from P1.M3**: The transaction system (complete) provides atomicity guarantees
- **User-Facing Command**: `jin commit` is the primary way users persist changes
- **Data Integrity**: Without atomic commits, crashes leave refs in inconsistent state

---

## What

### User-Visible Behavior

After this milestone:

```bash
# Stage files to different layers
jin mode use claude
jin add .claude/config.json --mode
jin add .claude/settings.json --mode --project
jin add .editorconfig --scope=language:javascript

# Commit all staged files atomically
jin commit -m "Add Claude configuration"
# Output:
# Committed 3 files to 3 layers:
#   mode-base (2): 1 file → abc1234
#   mode-project (5): 1 file → def5678
#   scope-base (6): 1 file → ghi9012

# Staging is now empty
jin status
# Output:
# Nothing staged

# Dry-run to preview (already implemented)
jin commit -m "Test" --dry-run
# Output:
# Would commit 3 files to 3 layers:
#   mode-base (2): 1 file
#     .claude/config.json
#   ...

# Error: nothing staged
jin commit -m "Empty"
# Output:
# Error: Nothing to commit
```

### Technical Requirements

1. **Layer Grouping**: Group staged entries by target layer (already provided by `StagingIndex.entries_for_layer()`)
2. **Tree Building**: For each layer, build a Git tree from staged entries using `ObjectOps.create_tree_from_paths()`
3. **Parent Resolution**: Get current commit OID from layer ref if exists (via `RefOps.resolve_ref()`)
4. **Commit Creation**: Create commit objects with tree, parent, and message using `ObjectOps.create_commit()`
5. **Atomic Transaction**: Use `LayerTransaction` to update all layer refs atomically
6. **Staging Cleanup**: Clear staging index after successful commit
7. **Context Integration**: Read mode/scope/project from `ProjectContext` for ref path generation
8. **Error Handling**: Propagate errors with transaction rollback on failure

### Success Criteria

- [ ] Commits are created for each affected layer
- [ ] Trees correctly represent staged file paths and content
- [ ] Parent commits are resolved from existing layer refs
- [ ] New layer refs are created if layer has no prior commits
- [ ] All layer ref updates are atomic via `LayerTransaction`
- [ ] Staging index is cleared only after successful commit
- [ ] Dry-run mode works correctly (already implemented)
- [ ] Empty staging returns appropriate error
- [ ] `CommitResult` contains correct commit hashes per layer
- [ ] Recovery from crash restores previous state

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement the commit pipeline. An AI agent with access to this PRP and the codebase can implement the feature in one pass._

### Documentation & References

```yaml
# MUST READ - Current Implementation (TO BE COMPLETED)

- file: src/commit/pipeline.rs
  why: Main implementation target - currently stubbed with TODO
  lines: 176
  pattern: |
    CommitPipeline { staging: StagingIndex }
    CommitConfig { message, author_name, author_email, dry_run }
    CommitResult { committed_layers, file_count, commit_hashes }
    execute(&mut self, config) -> Result<CommitResult>
    abort(&mut self) -> Result<()>
  critical: |
    - Dry-run mode already implemented (lines 79-104)
    - Real commit logic is stubbed at lines 106-120
    - Tests exist for config and empty commit cases
  gotcha: |
    - staging.affected_layers() returns layers sorted by precedence
    - staging.entries_for_layer(layer) returns &StagedEntry items

- file: src/commit/mod.rs
  why: Module exports
  lines: 4
  pattern: |
    pub use pipeline::{CommitConfig, CommitPipeline, CommitResult};

# MUST READ - Staging System (COMPLETE - from P3.M1)

- file: src/staging/index.rs
  why: StagingIndex API for reading staged entries
  lines: 204
  pattern: |
    StagingIndex { entries: HashMap<PathBuf, StagedEntry>, version }
    load() / save() - persistence to .jin/staging/index.json
    entries_for_layer(layer) -> Vec<&StagedEntry>
    affected_layers() -> Vec<Layer> - sorted by precedence!
    is_empty() / len() / clear()
  critical: |
    - Use affected_layers() to get layers needing commits
    - Use entries_for_layer() to get files for each layer
    - Call clear() then save() after successful commit

- file: src/staging/entry.rs
  why: StagedEntry structure with content hash
  lines: 89
  pattern: |
    StagedEntry { path, target_layer, content_hash, mode, operation }
    StagedOperation: AddOrModify, Delete, Rename
  critical: |
    - content_hash is hex string (40 chars) of Git blob OID
    - mode is u32: 0o100644 (regular) or 0o100755 (executable)
    - path is PathBuf for file location
    - Convert content_hash to Oid using Oid::from_str()

# MUST READ - Transaction System (COMPLETE - from P1.M3)

- file: src/git/transaction.rs
  why: LayerTransaction for atomic multi-layer commits
  lines: 737
  pattern: |
    LayerTransaction::begin(repo, message) -> Result<Self>
    tx.add_layer_update(layer, mode, scope, project, new_commit) -> Result<()>
    tx.commit() -> Result<()> - atomic apply
    tx.abort() -> Result<()> - rollback
    RecoveryManager::auto_recover(repo) - startup recovery
  critical: |
    - Always use LayerTransaction, NOT JinTransaction directly
    - Transaction log at .jin/.transaction_in_progress
    - add_layer_update() saves old_oid for rollback
    - commit() applies all updates atomically
  gotcha: |
    - Check for incomplete transaction before starting new one
    - Transaction automatically blocked if one exists

# MUST READ - Object Operations (COMPLETE - from P1.M2)

- file: src/git/objects.rs
  why: Tree and commit creation
  lines: 451
  pattern: |
    ObjectOps trait on JinRepo:
    create_tree_from_paths(files: &[(String, Oid)]) -> Result<Oid>
    create_commit(update_ref, message, tree_oid, parents) -> Result<Oid>
    find_tree(oid) / find_commit(oid) / find_blob(oid)
  critical: |
    - Use create_tree_from_paths() for nested directory handling
    - Files format: Vec<(path_string, blob_oid)>
    - Pass None for update_ref - we update refs via transaction
    - Parents is &[Oid] - empty for initial commit, [parent] otherwise

- file: src/git/refs.rs
  why: Reference operations for parent resolution
  lines: 100+
  pattern: |
    RefOps trait on JinRepo:
    ref_exists(name: &str) -> bool
    resolve_ref(name: &str) -> Result<Oid>
    set_ref(name, oid, message) -> Result<()>
  critical: |
    - ALWAYS check ref_exists() before resolve_ref() to avoid panic
    - Layer refs: refs/jin/layers/{context}

# MUST READ - Core Types

- file: src/core/layer.rs
  why: Layer enum with ref_path() method
  lines: 284
  pattern: |
    Layer enum: GlobalBase(1), ModeBase(2), ModeScope(3), etc.
    layer.ref_path(mode, scope, project) -> String
    layer.requires_mode() / requires_scope()
  critical: |
    - Use ref_path() to compute Git ref for each layer
    - Pass context values from ProjectContext
    - Example: Layer::ModeBase.ref_path(Some("claude"), None, None)
      → "refs/jin/layers/mode/claude"

- file: src/core/config.rs
  why: ProjectContext for active mode/scope/project
  lines: 231
  pattern: |
    ProjectContext { mode, scope, project, ... }
    ProjectContext::load() -> Result<Self>
  gotcha: load() returns NotInitialized if .jin/context doesn't exist

- file: src/git/repo.rs
  why: JinRepo for repository access
  lines: 279
  pattern: |
    JinRepo::open_or_create() -> Result<Self>
    inner() -> &Repository for git2 access
  critical: Jin repo is BARE at ~/.jin/

# EXTERNAL REFERENCES

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.commit
  why: git2 commit API reference
  critical: |
    - repo.commit(update_ref, author, committer, message, tree, parents)
    - Pass None for update_ref when not updating refs directly
    - Our ObjectOps.create_commit() wraps this

- url: https://github.com/rust-lang/git2-rs/tree/master/examples
  why: git2-rs usage examples
  critical: Check pull.rs and log.rs for commit patterns
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── cli/
│   │   ├── args.rs           # CLI arguments (CommitArgs here)
│   │   └── mod.rs
│   ├── commands/
│   │   ├── commit_cmd.rs     # jin commit CLI handler (calls pipeline)
│   │   └── mod.rs
│   ├── commit/
│   │   ├── pipeline.rs       # CommitPipeline - TO BE COMPLETED
│   │   └── mod.rs
│   ├── core/
│   │   ├── config.rs         # ProjectContext for mode/scope/project
│   │   ├── error.rs          # JinError types
│   │   ├── layer.rs          # Layer enum with ref_path()
│   │   └── mod.rs
│   ├── git/
│   │   ├── objects.rs        # ObjectOps: create_tree_from_paths, create_commit
│   │   ├── refs.rs           # RefOps: ref_exists, resolve_ref
│   │   ├── repo.rs           # JinRepo wrapper
│   │   ├── transaction.rs    # LayerTransaction for atomicity
│   │   ├── tree.rs           # TreeOps for reading
│   │   └── mod.rs
│   ├── staging/
│   │   ├── entry.rs          # StagedEntry { path, target_layer, content_hash, mode }
│   │   ├── index.rs          # StagingIndex { entries_for_layer, affected_layers, clear }
│   │   ├── router.rs         # Layer routing (used by jin add)
│   │   ├── workspace.rs      # File reading utilities
│   │   ├── gitignore.rs      # .gitignore management
│   │   └── mod.rs
│   └── lib.rs
├── Cargo.toml
└── tests/
    └── integration/
        └── cli_basic.rs
```

### Desired Codebase Tree After P3.M2

```bash
jin/
├── src/
│   ├── commit/
│   │   ├── pipeline.rs       # COMPLETED (~250 lines):
│   │   │   ├── CommitPipeline::execute() - full implementation
│   │   │   ├── build_layer_tree() - create tree from entries
│   │   │   ├── get_parent_commit() - resolve layer ref
│   │   │   └── comprehensive tests
│   │   └── mod.rs            # (unchanged)
│   └── ... (all other files unchanged)
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: Jin repository is BARE - no working directory
// ============================================================
// JinRepo at ~/.jin/ is a bare repository
// Blobs are already created during `jin add` (content_hash in StagedEntry)
// We just reference existing blob OIDs when building trees

// ============================================================
// CRITICAL: Check ref_exists() before resolve_ref()
// ============================================================
// resolve_ref() panics if ref doesn't exist
// CORRECT:
if repo.ref_exists(&ref_path) {
    let parent_oid = repo.resolve_ref(&ref_path)?;
    parents.push(parent_oid);
}
// WRONG:
let parent = repo.resolve_ref(&ref_path)?; // May panic!

// ============================================================
// PATTERN: Convert content_hash to Oid
// ============================================================
// StagedEntry.content_hash is 40-char hex string
// Convert to Oid for tree building:
use git2::Oid;
let oid = Oid::from_str(&entry.content_hash).map_err(|e| {
    JinError::Transaction(format!("Invalid OID: {}", e))
})?;

// ============================================================
// PATTERN: Tree building from staged entries
// ============================================================
// ObjectOps.create_tree_from_paths() expects Vec<(String, Oid)>
let files: Vec<(String, Oid)> = entries
    .iter()
    .filter(|e| !e.is_delete())  // Skip deletions
    .map(|e| {
        let oid = Oid::from_str(&e.content_hash)?;
        Ok((e.path.display().to_string(), oid))
    })
    .collect::<Result<Vec<_>>>()?;

let tree_oid = repo.create_tree_from_paths(&files)?;

// ============================================================
// PATTERN: Layer ref path computation
// ============================================================
// Use Layer::ref_path() with context values
let ref_path = layer.ref_path(
    context.mode.as_deref(),
    context.scope.as_deref(),
    context.project.as_deref(),
);
// Example results:
// GlobalBase → "refs/jin/layers/global"
// ModeBase → "refs/jin/layers/mode/claude"
// ModeProject → "refs/jin/layers/mode/claude/project/ui-dashboard"

// ============================================================
// PATTERN: Transaction usage for atomic commits
// ============================================================
// Begin transaction with commit message
let mut tx = LayerTransaction::begin(&repo, &config.message)?;

// Add each layer update (transaction log saved after each)
for (layer, commit_oid) in layer_commits {
    tx.add_layer_update(
        layer,
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
        commit_oid,
    )?;
}

// Atomic commit - all succeed or all rollback
tx.commit()?;

// ============================================================
// GOTCHA: Handle file mode from StagedEntry
// ============================================================
// Current create_tree_from_paths uses EntryMode::Blob (0o100644)
// For executable support, we'd need to extend the API
// For now, all files treated as regular (acceptable limitation)

// ============================================================
// GOTCHA: Delete operations
// ============================================================
// For StagedOperation::Delete entries:
// - Skip them when building the new tree
// - The file won't exist in the new tree (effectively deleted)
// This works because we build complete tree, not incremental

// ============================================================
// PATTERN: Staging cleanup on success
// ============================================================
// Only clear staging AFTER transaction commits successfully
tx.commit()?;
self.staging.clear();
self.staging.save()?;
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/commit/pipeline.rs ADDITIONS ==================

use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, LayerTransaction, ObjectOps, RefOps};
use crate::staging::StagingIndex;
use git2::Oid;

// Existing types remain unchanged:
// - CommitConfig
// - CommitResult
// - CommitPipeline

impl CommitPipeline {
    /// Execute the commit - FULL IMPLEMENTATION
    pub fn execute(&mut self, config: &CommitConfig) -> Result<CommitResult> {
        // 1. Validate staging not empty
        if self.staging.is_empty() {
            return Err(JinError::Other("Nothing to commit".to_string()));
        }

        let affected_layers = self.staging.affected_layers();
        let file_count = self.staging.len();

        // 2. Handle dry-run (already implemented)
        if config.dry_run {
            return self.execute_dry_run(&affected_layers, file_count);
        }

        // 3. Load context for ref path generation
        let context = ProjectContext::load().unwrap_or_default();

        // 4. Open Jin repository
        let repo = JinRepo::open_or_create()?;

        // 5. Create commits for each layer
        let mut layer_commits: Vec<(Layer, Oid)> = Vec::new();

        for layer in &affected_layers {
            let entries = self.staging.entries_for_layer(*layer);
            let commit_oid = self.create_layer_commit(
                &repo,
                *layer,
                &entries,
                &context,
                &config.message,
            )?;
            layer_commits.push((*layer, commit_oid));
        }

        // 6. Apply all updates atomically via transaction
        let mut tx = LayerTransaction::begin(&repo, &config.message)?;
        for (layer, commit_oid) in &layer_commits {
            tx.add_layer_update(
                *layer,
                context.mode.as_deref(),
                context.scope.as_deref(),
                context.project.as_deref(),
                *commit_oid,
            )?;
        }
        tx.commit()?;

        // 7. Clear staging on success
        self.staging.clear();
        self.staging.save()?;

        // 8. Build result
        let commit_hashes: Vec<(Layer, String)> = layer_commits
            .iter()
            .map(|(l, oid)| (*l, oid.to_string()))
            .collect();

        Ok(CommitResult {
            committed_layers: affected_layers,
            file_count,
            commit_hashes,
        })
    }

    /// Create a commit for a single layer
    fn create_layer_commit(
        &self,
        repo: &JinRepo,
        layer: Layer,
        entries: &[&crate::staging::StagedEntry],
        context: &ProjectContext,
        message: &str,
    ) -> Result<Oid> {
        // Build tree from entries
        let tree_oid = self.build_layer_tree(repo, entries)?;

        // Get parent commit if layer ref exists
        let parent_oids = self.get_parent_commits(repo, layer, context)?;

        // Create commit (don't update ref directly - transaction handles that)
        let commit_oid = repo.create_commit(
            None, // No ref update - transaction handles this
            message,
            tree_oid,
            &parent_oids,
        )?;

        Ok(commit_oid)
    }

    /// Build a tree from staged entries
    fn build_layer_tree(
        &self,
        repo: &JinRepo,
        entries: &[&crate::staging::StagedEntry],
    ) -> Result<Oid> {
        // Convert entries to (path, oid) tuples
        let files: Vec<(String, Oid)> = entries
            .iter()
            .filter(|e| !e.is_delete())
            .map(|e| {
                let oid = Oid::from_str(&e.content_hash).map_err(|err| {
                    JinError::Transaction(format!(
                        "Invalid content hash for {}: {}",
                        e.path.display(),
                        err
                    ))
                })?;
                Ok((e.path.display().to_string(), oid))
            })
            .collect::<Result<Vec<_>>>()?;

        // Handle empty tree (all deletions)
        if files.is_empty() {
            // Create empty tree
            return repo.create_tree(&[]);
        }

        repo.create_tree_from_paths(&files)
    }

    /// Get parent commit OIDs for a layer
    fn get_parent_commits(
        &self,
        repo: &JinRepo,
        layer: Layer,
        context: &ProjectContext,
    ) -> Result<Vec<Oid>> {
        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        if repo.ref_exists(&ref_path) {
            let parent_oid = repo.resolve_ref(&ref_path)?;
            Ok(vec![parent_oid])
        } else {
            // No parent - this is the initial commit for this layer
            Ok(vec![])
        }
    }

    /// Execute dry-run mode (existing implementation, extracted)
    fn execute_dry_run(
        &self,
        affected_layers: &[Layer],
        file_count: usize,
    ) -> Result<CommitResult> {
        println!(
            "Would commit {} files to {} layers:",
            file_count,
            affected_layers.len()
        );
        for layer in affected_layers {
            let layer_entries = self.staging.entries_for_layer(*layer);
            println!(
                "  {} ({}): {} files",
                layer,
                layer.precedence(),
                layer_entries.len()
            );
            for entry in layer_entries {
                println!("    {}", entry.path.display());
            }
        }

        Ok(CommitResult {
            committed_layers: affected_layers.to_vec(),
            file_count,
            commit_hashes: Vec::new(),
        })
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD helper method build_layer_tree() to CommitPipeline
  - IMPLEMENT: build_layer_tree(&self, repo, entries) -> Result<Oid>
  - CONVERT: StagedEntry.content_hash (hex string) to Oid
  - FILTER: Skip entries where is_delete() is true
  - BUILD: Vec<(String, Oid)> from path and content_hash
  - CALL: repo.create_tree_from_paths(&files)
  - HANDLE: Empty files vec (create empty tree)
  - PLACEMENT: Private method in CommitPipeline impl
  - TESTS: Test tree creation with single file, multiple files, nested paths

Task 2: ADD helper method get_parent_commits() to CommitPipeline
  - IMPLEMENT: get_parent_commits(&self, repo, layer, context) -> Result<Vec<Oid>>
  - COMPUTE: ref_path using layer.ref_path(mode, scope, project)
  - CHECK: repo.ref_exists(&ref_path) before resolving
  - RESOLVE: repo.resolve_ref(&ref_path) if exists
  - RETURN: Vec with parent oid, or empty vec for initial commit
  - PLACEMENT: Private method in CommitPipeline impl
  - TESTS: Test with existing ref, non-existing ref

Task 3: ADD helper method create_layer_commit() to CommitPipeline
  - IMPLEMENT: create_layer_commit(&self, repo, layer, entries, context, message) -> Result<Oid>
  - CALL: build_layer_tree() to create tree
  - CALL: get_parent_commits() to get parents
  - CALL: repo.create_commit(None, message, tree_oid, &parent_oids)
  - RETURN: commit Oid
  - PLACEMENT: Private method in CommitPipeline impl
  - TESTS: Test commit creation with and without parents

Task 4: COMPLETE execute() method in CommitPipeline
  - PRESERVE: Existing dry-run logic (extract to execute_dry_run())
  - ADD: Load ProjectContext for mode/scope/project
  - ADD: Open JinRepo via JinRepo::open_or_create()
  - ADD: Loop through affected_layers, call create_layer_commit()
  - ADD: Begin LayerTransaction with commit message
  - ADD: Call tx.add_layer_update() for each layer commit
  - ADD: Call tx.commit() for atomic apply
  - ADD: Clear staging with staging.clear() and staging.save()
  - ADD: Build CommitResult with commit hashes
  - HANDLE: All errors with proper propagation
  - TESTS: Integration tests for full commit flow

Task 5: UPDATE imports in src/commit/pipeline.rs
  - ADD: use crate::core::ProjectContext;
  - ADD: use crate::git::{JinRepo, LayerTransaction, ObjectOps, RefOps};
  - ADD: use git2::Oid;
  - VERIFY: All existing imports still needed
  - PLACEMENT: Top of file with existing imports

Task 6: ADD comprehensive tests
  - FILE: Tests in #[cfg(test)] section of pipeline.rs
  - TESTS:
    - test_build_layer_tree_single_file: Tree with one entry
    - test_build_layer_tree_multiple_files: Tree with multiple entries
    - test_build_layer_tree_nested_paths: Tree with subdirectories
    - test_build_layer_tree_with_deletions: Deletions filtered out
    - test_get_parent_commits_existing_ref: Returns parent oid
    - test_get_parent_commits_no_ref: Returns empty vec
    - test_create_layer_commit_initial: First commit (no parent)
    - test_create_layer_commit_with_parent: Subsequent commit
    - test_execute_single_layer: Full commit to one layer
    - test_execute_multiple_layers: Full commit to multiple layers
    - test_execute_clears_staging: Staging empty after commit
    - test_execute_transaction_atomicity: All or nothing semantics
  - USE: tempfile for isolated test directories
  - MOCK: Create test Jin repository and staging index
  - PATTERN: Follow existing tests in pipeline.rs
```

### Implementation Patterns & Key Details

```rust
// ================== COMPLETE IMPLEMENTATION ==================

use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, LayerTransaction, ObjectOps, RefOps};
use crate::staging::{StagedEntry, StagingIndex};
use git2::Oid;

/// Configuration for a commit operation
#[derive(Debug)]
pub struct CommitConfig {
    pub message: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub dry_run: bool,
}

impl CommitConfig {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            author_name: None,
            author_email: None,
            dry_run: false,
        }
    }

    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

/// Result of a commit operation
#[derive(Debug)]
pub struct CommitResult {
    pub committed_layers: Vec<Layer>,
    pub file_count: usize,
    pub commit_hashes: Vec<(Layer, String)>,
}

/// Pipeline for executing atomic commits across layers
#[derive(Debug)]
pub struct CommitPipeline {
    staging: StagingIndex,
}

impl CommitPipeline {
    pub fn new(staging: StagingIndex) -> Self {
        Self { staging }
    }

    /// Execute the commit
    pub fn execute(&mut self, config: &CommitConfig) -> Result<CommitResult> {
        // Validate staging not empty
        if self.staging.is_empty() {
            return Err(JinError::Other("Nothing to commit".to_string()));
        }

        let affected_layers = self.staging.affected_layers();
        let file_count = self.staging.len();

        // Handle dry-run
        if config.dry_run {
            return self.execute_dry_run(&affected_layers, file_count);
        }

        // Load context for ref path generation
        let context = ProjectContext::load().unwrap_or_default();

        // Open Jin repository
        let repo = JinRepo::open_or_create()?;

        // Create commits for each layer
        let mut layer_commits: Vec<(Layer, Oid)> = Vec::new();

        for layer in &affected_layers {
            let entries = self.staging.entries_for_layer(*layer);
            let commit_oid = self.create_layer_commit(
                &repo,
                *layer,
                &entries,
                &context,
                &config.message,
            )?;
            layer_commits.push((*layer, commit_oid));
        }

        // Apply all updates atomically via transaction
        let mut tx = LayerTransaction::begin(&repo, &config.message)?;
        for (layer, commit_oid) in &layer_commits {
            tx.add_layer_update(
                *layer,
                context.mode.as_deref(),
                context.scope.as_deref(),
                context.project.as_deref(),
                *commit_oid,
            )?;
        }
        tx.commit()?;

        // Clear staging on success
        self.staging.clear();
        self.staging.save()?;

        // Build result
        let commit_hashes: Vec<(Layer, String)> = layer_commits
            .iter()
            .map(|(l, oid)| (*l, oid.to_string()))
            .collect();

        Ok(CommitResult {
            committed_layers: affected_layers,
            file_count,
            commit_hashes,
        })
    }

    /// Create a commit for a single layer
    fn create_layer_commit(
        &self,
        repo: &JinRepo,
        layer: Layer,
        entries: &[&StagedEntry],
        context: &ProjectContext,
        message: &str,
    ) -> Result<Oid> {
        let tree_oid = self.build_layer_tree(repo, entries)?;
        let parent_oids = self.get_parent_commits(repo, layer, context)?;

        repo.create_commit(None, message, tree_oid, &parent_oids)
    }

    /// Build a tree from staged entries
    fn build_layer_tree(
        &self,
        repo: &JinRepo,
        entries: &[&StagedEntry],
    ) -> Result<Oid> {
        let files: Vec<(String, Oid)> = entries
            .iter()
            .filter(|e| !e.is_delete())
            .map(|e| {
                let oid = Oid::from_str(&e.content_hash).map_err(|err| {
                    JinError::Transaction(format!(
                        "Invalid content hash for {}: {}",
                        e.path.display(),
                        err
                    ))
                })?;
                Ok((e.path.display().to_string(), oid))
            })
            .collect::<Result<Vec<_>>>()?;

        if files.is_empty() {
            return repo.create_tree(&[]);
        }

        repo.create_tree_from_paths(&files)
    }

    /// Get parent commit OIDs for a layer
    fn get_parent_commits(
        &self,
        repo: &JinRepo,
        layer: Layer,
        context: &ProjectContext,
    ) -> Result<Vec<Oid>> {
        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        if repo.ref_exists(&ref_path) {
            let parent_oid = repo.resolve_ref(&ref_path)?;
            Ok(vec![parent_oid])
        } else {
            Ok(vec![])
        }
    }

    /// Execute dry-run mode
    fn execute_dry_run(
        &self,
        affected_layers: &[Layer],
        file_count: usize,
    ) -> Result<CommitResult> {
        println!(
            "Would commit {} files to {} layers:",
            file_count,
            affected_layers.len()
        );
        for layer in affected_layers {
            let layer_entries = self.staging.entries_for_layer(*layer);
            println!(
                "  {} ({}): {} files",
                layer,
                layer.precedence(),
                layer_entries.len()
            );
            for entry in layer_entries {
                println!("    {}", entry.path.display());
            }
        }

        Ok(CommitResult {
            committed_layers: affected_layers.to_vec(),
            file_count,
            commit_hashes: Vec::new(),
        })
    }

    /// Abort the commit and roll back any changes
    pub fn abort(&mut self) -> Result<()> {
        // If there's an incomplete transaction, recovery manager handles it
        // This method exists for explicit abort during pipeline execution
        Ok(())
    }
}
```

### Integration Points

```yaml
DEPENDENCIES (already in Cargo.toml - NO NEW DEPS NEEDED):
  - git2 = { version = "0.19", features = ["vendored-libgit2"] }
  - serde = { version = "1.0", features = ["derive"] }
  - serde_json = "1.0"
  - thiserror = "2.0"
  - chrono = { version = "0.4", features = ["serde"] }

IMPORTS NEEDED in pipeline.rs:
  - use crate::core::ProjectContext;
  - use crate::git::{JinRepo, LayerTransaction, ObjectOps, RefOps};
  - use git2::Oid;

STAGING MODULE:
  - StagingIndex.entries_for_layer(layer) -> Vec<&StagedEntry>
  - StagingIndex.affected_layers() -> Vec<Layer>
  - StagingIndex.clear() -> clears all entries
  - StagingIndex.save() -> persists to disk
  - StagedEntry.path, .content_hash, .is_delete()

GIT MODULE:
  - JinRepo::open_or_create() -> access Jin's bare repo
  - ObjectOps.create_tree_from_paths() -> build tree
  - ObjectOps.create_commit() -> create commit object
  - RefOps.ref_exists(), .resolve_ref() -> parent lookup
  - LayerTransaction -> atomic multi-ref updates

CORE MODULE:
  - ProjectContext.load() -> get mode/scope/project
  - Layer.ref_path() -> compute Git ref path
  - JinError -> error handling

CLI MODULE:
  - commands/commit_cmd.rs calls CommitPipeline
  - No changes needed to CLI layer
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build with no warnings
```

### Level 3: Unit Tests (Component Validation)

```bash
# Run commit module tests
cargo test commit::                   # All commit tests

# Run specific test files
cargo test commit::pipeline::         # Pipeline tests

# Run with output for debugging
cargo test commit:: -- --nocapture

# Run related module tests
cargo test staging::                  # Staging module (dependency)
cargo test git::transaction::         # Transaction module (dependency)

# Expected: All tests pass
```

### Level 4: Integration Testing

```bash
# Full test suite
cargo test

# Verify existing tests still pass (regression)
cargo test core::
cargo test git::
cargo test staging::

# Manual testing in temp directory
cd /tmp && mkdir jin-test && cd jin-test
git init  # Create project repo
jin init  # Initialize Jin

# Set up mode
jin mode create claude
jin mode use claude

# Create and stage test files
mkdir -p .claude
echo '{"config": true}' > .claude/config.json
echo '{"local": true}' > .claude/local.json

jin add .claude/config.json --mode
jin add .claude/local.json --mode --project

# Check staging
jin status

# Test dry-run
jin commit -m "Test commit" --dry-run

# Test real commit
jin commit -m "Add Claude configuration"

# Verify commit created
# (Check refs in ~/.jin/)

# Verify staging cleared
jin status
# Output: Nothing staged

# Cleanup
cd /tmp && rm -rf jin-test

# Expected: All manual tests work correctly
```

### Level 5: Full Validation

```bash
# Complete validation pipeline
cargo fmt -- --check && \
cargo clippy -- -D warnings && \
cargo build && \
cargo test

# Expected: All commands succeed with zero errors
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo test commit::` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] `jin commit -m "msg"` creates commits for all affected layers
- [ ] Each layer gets its own commit with correct tree structure
- [ ] Parent commits correctly resolved from existing layer refs
- [ ] New layers (no prior commits) get initial commits with no parents
- [ ] LayerTransaction ensures atomicity across all layer updates
- [ ] If any layer fails, all updates are rolled back
- [ ] Staging index is cleared after successful commit
- [ ] CommitResult contains correct commit hashes per layer
- [ ] Dry-run mode works correctly (shows what would be committed)
- [ ] Empty staging returns "Nothing to commit" error
- [ ] Nested file paths create correct tree structure

### Code Quality Validation

- [ ] All new methods have doc comments
- [ ] Error handling uses JinError types consistently
- [ ] No unwrap() in library code (only in tests)
- [ ] Tests use tempfile for isolation
- [ ] Follows existing code patterns
- [ ] Private helper methods are properly encapsulated

### Integration Validation

- [ ] Works with StagingIndex from P3.M1
- [ ] Works with LayerTransaction from P1.M3
- [ ] Works with ObjectOps and RefOps from P1.M2
- [ ] ProjectContext loaded correctly for context values
- [ ] Layer.ref_path() generates correct ref names

---

## Anti-Patterns to Avoid

- **Don't call resolve_ref() without checking ref_exists() first** - will panic
- **Don't update refs directly** - use LayerTransaction for atomicity
- **Don't clear staging before transaction commits** - data loss on failure
- **Don't use JinTransaction directly** - use LayerTransaction for atomicity
- **Don't forget to save() staging after clear()** - index won't persist
- **Don't store blobs during commit** - blobs already created during `jin add`
- **Don't use unwrap() in library code** - propagate errors with `?`
- **Don't modify the dry-run logic** - it's already working correctly
- **Don't assume context values exist** - use .unwrap_or_default() or handle None
- **Don't skip deletion entries in count** - they're in file_count but filtered from tree

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Staging system (P3.M1) provides clean API: `affected_layers()`, `entries_for_layer()`
- Transaction system (P1.M3) provides atomic multi-ref updates
- Object operations (P1.M2) provide tree and commit creation
- Layer.ref_path() already computes correct ref paths
- Dry-run mode already implemented and working
- All dependencies are complete and well-tested
- Clear data flow: staging → tree → commit → transaction → cleanup
- Existing tests provide patterns to follow
- No new dependencies required

**Remaining Risks:**
- File mode handling (executable bit) - current impl uses 0o100644 for all files
- Large number of files per layer - tree building may need optimization
- Complex nested paths - rely on create_tree_from_paths() which is tested
- Context loading when .jin/context doesn't exist - handled with unwrap_or_default()

---

## Research Artifacts

Research has been completed covering:

| Topic | Key Insights |
|-------|--------------|
| **Staging Integration** | StagingIndex provides entries_for_layer(), affected_layers(), clear() |
| **Transaction System** | LayerTransaction provides atomic multi-ref updates with crash recovery |
| **Object Operations** | ObjectOps.create_tree_from_paths() handles nested directories |
| **git2 Patterns** | Tree building from file list, commit creation with parents |
| **Layer Refs** | Layer.ref_path() computes refs/jin/layers/{context} |

Key internal references:
- `plan/P3M1/PRP.md` - Staging system implementation
- `plan/P1M3/PRP.md` - Transaction system implementation
- `src/git/objects.rs` - ObjectOps trait with tree/commit creation
- `src/git/transaction.rs` - LayerTransaction for atomicity

Key external references:
- git2 Documentation: https://docs.rs/git2
- git2-rs Examples: https://github.com/rust-lang/git2-rs/tree/master/examples
- libgit2 Samples: https://libgit2.org/docs/guides/101-samples/

---

## Appendix: Commit Flow Diagram

```
jin commit -m "message"
        │
        ▼
┌───────────────────┐
│ Load StagingIndex │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Validate not empty│
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Get affected_layers│
└───────────────────┘
        │
        ▼
┌───────────────────────────────┐
│ For each layer:               │
│  1. Get entries_for_layer()   │
│  2. Build tree from entries   │
│  3. Get parent commit (if any)│
│  4. Create commit object      │
│  5. Store (layer, commit_oid) │
└───────────────────────────────┘
        │
        ▼
┌───────────────────────────────┐
│ LayerTransaction.begin()      │
│  For each (layer, commit_oid):│
│    tx.add_layer_update()      │
│  tx.commit() ← ATOMIC         │
└───────────────────────────────┘
        │
        ▼
┌───────────────────┐
│ staging.clear()   │
│ staging.save()    │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Return CommitResult│
│  - layers         │
│  - file_count     │
│  - commit_hashes  │
└───────────────────┘
```

---

## Appendix: Test Case Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::staging::StagedEntry;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_setup() -> (TempDir, JinRepo) {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        std::fs::create_dir_all(".jin/staging").unwrap();

        let repo_path = temp.path().join(".jin-repo");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

    #[test]
    fn test_build_layer_tree_single_file() {
        let (temp, repo) = create_test_setup();

        // Create a blob first
        let blob_oid = repo.create_blob(b"content").unwrap();

        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from("config.json"),
            Layer::ProjectBase,
            blob_oid.to_string(),
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        assert_eq!(tree.len(), 1);
        assert!(tree.get_name("config.json").is_some());
    }

    #[test]
    fn test_build_layer_tree_nested_paths() {
        let (temp, repo) = create_test_setup();

        let blob_oid = repo.create_blob(b"nested content").unwrap();

        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from(".claude/config/settings.json"),
            Layer::ModeBase,
            blob_oid.to_string(),
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        // Should have .claude directory
        assert!(tree.get_name(".claude").is_some());
    }

    #[test]
    fn test_build_layer_tree_with_deletions() {
        let (temp, repo) = create_test_setup();

        let blob_oid = repo.create_blob(b"content").unwrap();

        let staging = StagingIndex::new();
        let keep_entry = StagedEntry::new(
            PathBuf::from("keep.json"),
            Layer::ProjectBase,
            blob_oid.to_string(),
        );
        let delete_entry = StagedEntry::delete(
            PathBuf::from("delete.json"),
            Layer::ProjectBase,
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&keep_entry, &delete_entry];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        // Only keep.json should be in tree
        assert_eq!(tree.len(), 1);
        assert!(tree.get_name("keep.json").is_some());
        assert!(tree.get_name("delete.json").is_none());
    }

    #[test]
    fn test_get_parent_commits_no_ref() {
        let (temp, repo) = create_test_setup();

        let staging = StagingIndex::new();
        let context = ProjectContext::default();
        let pipeline = CommitPipeline::new(staging);

        let parents = pipeline.get_parent_commits(
            &repo,
            Layer::GlobalBase,
            &context,
        ).unwrap();

        assert!(parents.is_empty());
    }

    #[test]
    fn test_get_parent_commits_with_existing_ref() {
        let (temp, repo) = create_test_setup();

        // Create initial commit
        let blob_oid = repo.create_blob(b"initial").unwrap();
        let tree_oid = repo.create_tree(&[
            crate::git::TreeEntry::blob("file.txt", blob_oid),
        ]).unwrap();
        let initial_commit = repo.create_commit(
            Some("refs/jin/layers/global"),
            "Initial",
            tree_oid,
            &[],
        ).unwrap();

        let staging = StagingIndex::new();
        let context = ProjectContext::default();
        let pipeline = CommitPipeline::new(staging);

        let parents = pipeline.get_parent_commits(
            &repo,
            Layer::GlobalBase,
            &context,
        ).unwrap();

        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], initial_commit);
    }

    #[test]
    fn test_execute_single_layer() {
        let (temp, repo) = create_test_setup();

        // Create blob
        let blob_oid = repo.create_blob(b"config content").unwrap();

        // Create staging with entry
        let mut staging = StagingIndex::new();
        staging.add(StagedEntry::new(
            PathBuf::from("config.json"),
            Layer::ProjectBase,
            blob_oid.to_string(),
        ));

        let mut pipeline = CommitPipeline::new(staging);
        let config = CommitConfig::new("Test commit");

        let result = pipeline.execute(&config).unwrap();

        assert_eq!(result.committed_layers.len(), 1);
        assert_eq!(result.file_count, 1);
        assert_eq!(result.commit_hashes.len(), 1);
        assert_eq!(result.commit_hashes[0].0, Layer::ProjectBase);
    }

    #[test]
    fn test_execute_clears_staging() {
        let (temp, repo) = create_test_setup();

        let blob_oid = repo.create_blob(b"content").unwrap();

        let mut staging = StagingIndex::new();
        staging.add(StagedEntry::new(
            PathBuf::from("file.json"),
            Layer::GlobalBase,
            blob_oid.to_string(),
        ));

        let mut pipeline = CommitPipeline::new(staging);
        let config = CommitConfig::new("Clear test");

        pipeline.execute(&config).unwrap();

        // Staging should be cleared
        assert!(pipeline.staging.is_empty());
    }

    // Existing tests preserved
    #[test]
    fn test_commit_config_new() {
        let config = CommitConfig::new("Test commit");
        assert_eq!(config.message, "Test commit");
        assert!(!config.dry_run);
    }

    #[test]
    fn test_commit_config_dry_run() {
        let config = CommitConfig::new("Test").dry_run(true);
        assert!(config.dry_run);
    }

    #[test]
    fn test_commit_pipeline_empty() {
        let staging = StagingIndex::new();
        let mut pipeline = CommitPipeline::new(staging);
        let config = CommitConfig::new("Empty commit");

        let result = pipeline.execute(&config);
        assert!(result.is_err());
    }
}
```
