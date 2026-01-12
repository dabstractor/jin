# PRP: P2.M1.T1 - Detect Fast-Forward vs Divergent Histories

---

## Goal

**Feature Goal**: Add logic to detect when a pull operation is fast-forward vs divergent. Divergent histories require 3-way merge instead of simple ref updates.

**Deliverable**: Merge detection module in `src/git/` with:
1. `MergeType` enum with `FastForward` and `Divergent` variants
2. `detect_merge_type()` function using `git2::Repository::merge_base()`
3. Integration into `src/commands/pull.rs` to control merge flow

**Success Definition**:
- `cargo test git::merge_detection` passes
- `cargo test commands::pull::tests::test_*merge_type*` passes
- Pull can distinguish fast-forward from divergent scenarios
- TODO comment in pull.rs line 50 is addressed

---

## User Persona

**Target User**: Jin CLI users working with remote repositories who may experience divergent histories when multiple contributors push to the same layer.

**Use Case**: User A and User B both pull the same layer, make different commits, then push. User A pushes first. When User B pulls, they have a divergent history that requires 3-way merge.

**User Journey**:
1. User pulls remote changes
2. Jin detects if merge is fast-forward or divergent
3. Fast-forward: Simple ref update (existing behavior)
4. Divergent: 3-way merge using text_merge engine (P2.M1.T2)

**Pain Points Addressed**:
- Current pull only supports fast-forward, fails silently or corrupts data on divergent histories
- Enables team collaboration scenarios with proper merge conflict detection
- Foundation for proper 3-way merge in next task

---

## Why

- **Enables Team Collaboration**: Multiple users can work on same layers and merge their changes
- **Prevents Data Loss**: Detects divergent histories before applying potentially incorrect updates
- **Foundation for P2.M1.T2**: This task provides the merge type detection needed for 3-way merge implementation
- **Completes PRD Requirement**: "Pull must handle non-fast-forward cases using 3-way merge"

---

## What

### User-Visible Behavior

After this milestone, the pull command will:

```rust
// In pull.rs, detect_updates() will return merge type information
struct LayerUpdateInfo {
    layer: Layer,
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    local_oid: Option<git2::Oid>,
    remote_oid: git2::Oid,
    merge_type: MergeType,  // NEW: FastForward or Divergent
}

// Pull execution will branch based on merge type
for (ref_path, update_info) in &updates {
    match update_info.merge_type {
        MergeType::FastForward => {
            // Existing behavior: simple ref update
            tx.add_layer_update(...)?;
        }
        MergeType::Divergent => {
            // P2.M1.T2 will handle this case
            // For now, we can either skip or error with clear message
        }
    }
}
```

### Technical Requirements

1. **MergeType Enum**: `FastForward`, `Divergent` variants
2. **detect_merge_type()**: Uses git2 merge_base to determine merge scenario
3. **Integration**: Modify `detect_updates()` to return merge type
4. **Tests**: Unit tests for merge detection, integration tests for pull scenarios

### Success Criteria

- [ ] `MergeType` enum defined in `src/git/refs.rs` or new `src/git/merge.rs`
- [ ] `detect_merge_type()` function using `repo.merge_base()`
- [ ] `detect_updates()` in pull.rs returns `MergeType` for each layer
- [ ] Unit tests cover: fast-forward, divergent, up-to-date, new layer scenarios
- [ ] Integration tests verify pull behavior for each merge type
- [ ] No breaking changes to existing pull fast-forward behavior

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement merge detection, including exact API signatures, git2-rs patterns, test fixtures, and integration points._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# git2-rs Official Documentation
- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.merge_base
  why: Repository::merge_base() - core method for finding common ancestor
  critical: |
    - Finds most recent common ancestor between two commits
    - Returns Oid of merge base
    - Returns error if no merge base exists (should not happen with valid refs)

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.descendant_of
  why: Repository::descendant_of() - alternative for ancestry detection
  critical: |
    - Checks if one commit is descendant of another
    - Useful for fast-forward detection
    - Note: Unlike Git, git2-rs does NOT consider a commit a descendant of itself

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.graph_ahead_behind
  why: Already used in compare_refs() - similar pattern for merge detection
  critical: |
    - Returns (ahead_count, behind_count)
    - (0, 0) = equal, (n, 0) = ahead, (0, n) = behind, (m, n) = diverged
    - Used by RefComparison enum in src/git/refs.rs

# Git Merge Concepts
- url: https://git-scm.com/docs/git-merge
  why: Understanding fast-forward vs 3-way merge
  section: "#_fast_forward_merge
- critical: |
    - Fast-forward: local commit is ancestor of remote
    - 3-way merge: histories have diverged, need merge base

- url: https://git-scm.com/docs/git-merge-base
  why: Understanding merge base detection algorithm
  critical: |
    - Merge base = most recent common ancestor
    - Used for 3-way merge: merge(base, ours, theirs)

# Codebase References
- file: /home/dustin/projects/jin/src/commands/pull.rs
  why: Existing pull implementation, needs merge type integration
  pattern: |
    - Line 28: Implicit fetch via super::fetch::execute()
    - Line 34: detect_updates() finds layers with updates
    - Line 44: LayerTransaction for atomic updates
    - Line 50: TODO comment "Implement proper 3-way merge"
    - Lines 51-57: Current simple fast-forward update logic

- file: /home/dustin/projects/jin/src/git/refs.rs
  why: RefComparison enum pattern - similar to MergeType
  pattern: |
    - Lines 16-28: RefComparison enum (Ahead, Behind, Diverged, Equal)
    - Lines 164-177: compare_refs() using graph_ahead_behind
    - MergeType can follow same structure

- file: /home/dustin/projects/jin/src/merge/text.rs
  why: Text merge engine used by P2.M1.T2 for 3-way merge
  pattern: |
    - text_merge() function takes base, ours, theirs
    - Returns TextMergeResult::Clean or Conflict
    - Will be used in next task (P2.M1.T2)

- file: /home/dustin/projects/jin/src/core/error.rs
  why: Error types to use for merge failures
  pattern: |
    - JinError::Git for git2 errors
    - JinError::MergeConflict for merge conflicts
    - Use consistent error formatting with user-friendly messages

# Test Patterns
- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  why: Integration test patterns for pull/fetch operations
  pattern: |
    - setup_jin_with_remote() fixture for local+remote repos
    - RemoteFixture with local_path and remote_path
    - Test isolation via unique_test_id()
    - GitTestEnv wrapper for lock cleanup
    - Lines 135-221: test_pull_merges_changes pattern

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Common test fixtures for repository setup
  pattern: |
    - TestFixture for isolated test directories
    - RemoteFixture for local + bare remote setup
    - setup_jin_with_remote() creates linked repos

# External Examples
- url: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
  why: Official git2-rs pull example
  critical: |
    - Shows fetch + merge workflow
    - Demonstrates fast-forward detection
    - Uses descendant_of() for ancestry checks

- url: https://stackoverflow.com/questions/37648908/determine-if-a-merge-will-resolve-via-fast-forward
  why: Algorithm for fast-forward detection
  critical: |
    - Check if local is ancestor of remote (fast-forward)
    - Check if remote is ancestor of local (local ahead)
    - Otherwise: divergent (needs merge)
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── commands/
│   │   ├── pull.rs           # Current implementation with TODO at line 50
│   │   ├── fetch.rs          # Fetches remote refs
│   │   └── push.rs           # Has ref comparison patterns
│   ├── git/
│   │   ├── mod.rs            # Module exports
│   │   ├── refs.rs           # RefComparison enum, compare_refs()
│   │   ├── repo.rs           # JinRepo wrapper
│   │   └── remote.rs         # Remote operations
│   ├── merge/
│   │   ├── mod.rs            # Merge engine exports
│   │   └── text.rs           # 3-way text merge (used in P2.M1.T2)
│   └── core/
│       ├── error.rs          # JinError types
│       └── layer.rs          # Layer enum
└── tests/
    ├── sync_workflow.rs      # Pull/fetch integration tests
    └── common/
        └── fixtures.rs       # Test fixtures
```

### Desired Codebase Tree After P2.M1.T1

```bash
jin/
├── src/
│   ├── git/
│   │   ├── merge.rs          # NEW: Merge detection module
│   │   │   └── Contains:
│   │   │       - MergeType enum (FastForward, Divergent)
│   │   │       - detect_merge_type() function
│   │   │       - Unit tests for merge detection
│   │   └── mod.rs            # MODIFIED: Add pub use merge::MergeType, detect_merge_type
│   └── commands/
│       └── pull.rs           # MODIFIED: Use MergeType in LayerUpdateInfo
└── tests/
    └── sync_workflow.rs      # MODIFIED: Add merge type detection tests
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2-rs descendant_of() behavior differs from Git
// Git: a commit IS a descendant of itself
// git2-rs: a commit is NOT a descendant of itself
// This affects equality checks - use Oid comparison for equality

// Example of correct pattern:
fn detect_merge_type(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<MergeType> {
    // Check equality first (descendant_of returns false for same commit)
    if local_oid == remote_oid {
        return Ok(MergeType::UpToDate);
    }

    // Check fast-forward: local is ancestor of remote
    if repo.descendant_of(remote_oid, local_oid)? {
        return Ok(MergeType::FastForward);
    }

    // Check if remote is ancestor of local (local is ahead)
    if repo.descendant_of(local_oid, remote_oid)? {
        return Ok(MergeType::LocalAhead);
    }

    // Otherwise: divergent history
    Ok(MergeType::Divergent)
}

// GOTCHA: merge_base() can fail if commits are unrelated
// Handle gracefully - treat unrelated repos as divergent
let merge_base = match repo.merge_base(local_oid, remote_oid) {
    Ok(base) => base,
    Err(_) => return Ok(MergeType::Divergent),
};

// PATTERN: Use graph_ahead_behind for more detailed state
// This is what RefComparison uses in src/git/refs.rs
let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;
match (ahead, behind) {
    (0, 0) => MergeType::UpToDate,
    (_, 0) => MergeType::FastForward,
    (0, _) => MergeType::LocalAhead,
    (_, _) => MergeType::Divergent,
}

// GOTCHA: New layers (no local ref) should be treated as FastForward
// In detect_updates(), check if local_oid is None
let needs_update = match local_oid {
    Some(local) => {
        let merge_type = detect_merge_type(&jin_repo, local, remote_oid)?;
        merge_type != MergeType::UpToDate
    }
    None => true, // New layer - always fast-forward
};

// PATTERN: Error handling - wrap git2 errors into JinError::Git
// Use ? operator throughout

// PATTERN: Follow RefComparison enum structure from src/git/refs.rs
// MergeType should be similar: derive Debug, Clone, Copy, PartialEq, Eq
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/git/merge.rs ==================
//! Merge detection for determining merge strategy
//!
//! Provides [`MergeType`] enum and [`detect_merge_type()`] function for
//! determining whether a pull operation requires fast-forward or 3-way merge.

use crate::core::{JinError, Result};
use crate::git::JinRepo;
use git2::Oid;

/// Type of merge required to integrate remote changes
///
/// Determines the merge strategy based on the relationship between
/// local and remote commit histories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeType {
    /// Local and remote are at the same commit (no action needed)
    UpToDate,

    /// Local commit is an ancestor of remote (simple fast-forward possible)
    FastForward,

    /// Remote commit is an ancestor of local (local is ahead)
    LocalAhead,

    /// Local and remote have diverged (requires 3-way merge)
    Divergent,
}

/// Detect the merge type required to integrate remote changes
///
/// # Algorithm
///
/// 1. Check if OIDs are equal → `UpToDate`
/// 2. Check if local is ancestor of remote → `FastForward`
/// 3. Check if remote is ancestor of local → `LocalAhead`
/// 4. Otherwise → `Divergent`
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `local_oid` - OID of local commit
/// * `remote_oid` - OID of remote commit
///
/// # Returns
///
/// `MergeType` indicating the merge strategy required
///
/// # Errors
///
/// Returns `JinError::Git` if graph traversal fails
///
/// # Example
///
/// ```ignore
/// use jin::git::{JinRepo, merge::{detect_merge_type, MergeType}};
///
/// let repo = JinRepo::open_or_create()?;
/// let local_oid = repo.resolve_ref("refs/jin/layers/global")?;
/// let remote_oid = repo.resolve_ref("refs/remotes/origin/layers/global")?;
///
/// match detect_merge_type(&repo, local_oid, remote_oid)? {
///     MergeType::FastForward => println!("Fast-forward merge"),
///     MergeType::Divergent => println!("3-way merge needed"),
///     _ => println!("No merge needed"),
/// }
/// ```
pub fn detect_merge_type(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<MergeType> {
    // PATTERN: Check equality first - descendant_of returns false for same commit
    // GOTCHA: git2-rs does NOT consider a commit a descendant of itself
    if local_oid == remote_oid {
        return Ok(MergeType::UpToDate);
    }

    // PATTERN: Use graph_ahead_behind for accurate state detection
    // This matches the existing RefComparison pattern in src/git/refs.rs
    let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;

    match (ahead, behind) {
        (0, 0) => Ok(MergeType::UpToDate),    // Same commit (redundant check)
        (_, 0) => Ok(MergeType::FastForward), // Local is ancestor of remote
        (0, _) => Ok(MergeType::LocalAhead),  // Remote is ancestor of local
        (_, _) => Ok(MergeType::Divergent),   // Both have unique commits
    }
}

/// Alternative implementation using merge_base and descendant_of
///
/// This approach uses git2's merge_base and descendant_of methods
/// instead of graph_ahead_behind. Either approach is valid.
#[allow(dead_code)]
pub fn detect_merge_type_with_base(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<MergeType> {
    // Check equality first
    if local_oid == remote_oid {
        return Ok(MergeType::UpToDate);
    }

    // Check fast-forward: is local an ancestor of remote?
    if repo.inner().descendant_of(remote_oid, local_oid)? {
        return Ok(MergeType::FastForward);
    }

    // Check if remote is ancestor of local (local is ahead)
    if repo.inner().descendant_of(local_oid, remote_oid)? {
        return Ok(MergeType::LocalAhead);
    }

    // Find merge base to confirm divergence
    match repo.inner().merge_base(local_oid, remote_oid) {
        Ok(_) => Ok(MergeType::Divergent),
        // No merge base means unrelated histories - treat as divergent
        Err(_) => Ok(MergeType::Divergent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_commit_chain(repo: &git2::Repository, count: usize) -> Vec<Oid> {
        let sig = repo.signature().unwrap();
        let mut commits = Vec::new();

        let mut parent = None;
        for i in 0..count {
            let mut tree_builder = repo.treebuilder(None).unwrap();
            let blob_oid = repo.blob(format!("content{}", i).as_bytes()).unwrap();
            tree_builder.insert("file.txt", blob_oid, 0o100644).unwrap();
            let tree_oid = tree_builder.write().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();

            let oid = repo
                .commit(
                    None,
                    &sig,
                    &sig,
                    &format!("Commit {}", i),
                    &tree,
                    parent.as_ref().map(std::slice::from_ref).unwrap_or(&[]),
                )
                .unwrap();

            commits.push(oid);
            parent = Some(repo.find_commit(oid).unwrap());
        }

        commits
    }

    #[test]
    fn test_detect_merge_type_equal() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = git2::Repository::init_bare(&repo_path).unwrap();
        let jin_repo = JinRepo { repo, path: repo_path };

        let commits = create_test_commit_chain(&jin_repo.inner(), 1);
        let oid = commits[0];

        let result = detect_merge_type(&jin_repo, oid, oid).unwrap();
        assert_eq!(result, MergeType::UpToDate);
    }

    #[test]
    fn test_detect_merge_type_fast_forward() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = git2::Repository::init_bare(&repo_path).unwrap();
        let jin_repo = JinRepo { repo, path: repo_path };

        let commits = create_test_commit_chain(&jin_repo.inner(), 3);
        // local is ancestor of remote
        let local = commits[0];
        let remote = commits[2];

        let result = detect_merge_type(&jin_repo, local, remote).unwrap();
        assert_eq!(result, MergeType::FastForward);
    }

    #[test]
    fn test_detect_merge_type_local_ahead() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = git2::Repository::init_bare(&repo_path).unwrap();
        let jin_repo = JinRepo { repo, path: repo_path };

        let commits = create_test_commit_chain(&jin_repo.inner(), 3);
        // remote is ancestor of local
        let local = commits[2];
        let remote = commits[0];

        let result = detect_merge_type(&jin_repo, local, remote).unwrap();
        assert_eq!(result, MergeType::LocalAhead);
    }

    #[test]
    fn test_detect_merge_type_divergent() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = git2::Repository::init_bare(&repo_path).unwrap();
        let jin_repo = JinRepo { repo, path: repo_path };

        let sig = jin_repo.inner().signature().unwrap();

        // Create base commit
        let mut tree_builder = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid = jin_repo.inner().blob(b"base").unwrap();
        tree_builder.insert("file.txt", blob_oid, 0o100644).unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = jin_repo.inner().find_tree(tree_oid).unwrap();
        let base_oid = jin_repo
            .inner()
            .commit(None, &sig, &sig, "base", &tree, &[])
            .unwrap();
        let base_commit = jin_repo.inner().find_commit(base_oid).unwrap();

        // Create divergent commit 1
        let mut tree_builder1 = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid1 = jin_repo.inner().blob(b"divergent1").unwrap();
        tree_builder1.insert("file1.txt", blob_oid1, 0o100644).unwrap();
        let tree_oid1 = tree_builder1.write().unwrap();
        let tree1 = jin_repo.inner().find_tree(tree_oid1).unwrap();
        let divergent1 = jin_repo
            .inner()
            .commit(None, &sig, &sig, "divergent1", &tree1, &[&base_commit])
            .unwrap();

        // Create divergent commit 2 (same parent, different content)
        let mut tree_builder2 = jin_repo.inner().treebuilder(None).unwrap();
        let blob_oid2 = jin_repo.inner().blob(b"divergent2").unwrap();
        tree_builder2.insert("file2.txt", blob_oid2, 0o100644).unwrap();
        let tree_oid2 = tree_builder2.write().unwrap();
        let tree2 = jin_repo.inner().find_tree(tree_oid2).unwrap();
        let divergent2 = jin_repo
            .inner()
            .commit(None, &sig, &sig, "divergent2", &tree2, &[&base_commit])
            .unwrap();

        let result = detect_merge_type(&jin_repo, divergent1, divergent2).unwrap();
        assert_eq!(result, MergeType::Divergent);
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/git/merge.rs
  - IMPLEMENT: MergeType enum with UpToDate, FastForward, LocalAhead, Divergent variants
  - IMPLEMENT: detect_merge_type() function using graph_ahead_behind
  - IMPLEMENT: detect_merge_type_with_base() alternative implementation (optional)
  - IMPLEMENT: Unit tests for all merge type scenarios
  - FOLLOW pattern: RefComparison enum in src/git/refs.rs
  - NAMING: CamelCase for enum variants, snake_case for functions
  - DEPENDENCIES: None (can start immediately)
  - PLACEMENT: src/git/merge.rs

Task 2: MODIFY src/git/mod.rs
  - ADD: pub mod merge;
  - ADD: pub use merge::{MergeType, detect_merge_type};
  - PRESERVE: All existing exports
  - PLACEMENT: src/git/mod.rs
  - DEPENDS_ON: Task 1

Task 3: MODIFY src/commands/pull.rs
  - MODIFY: LayerUpdateInfo struct to include merge_type: MergeType field
  - MODIFY: detect_updates() to call detect_merge_type() for each layer
  - MODIFY: Pull execution loop to check merge_type before updating
  - PRESERVE: All existing fast-forward behavior
  - PRESERVE: TODO comment at line 50 (add comment that P2.M1.T2 will handle Divergent)
  - PLACEMENT: src/commands/pull.rs
  - DEPENDS_ON: Task 1, Task 2

Task 4: ADD unit tests to src/git/merge.rs
  - IMPLEMENT: test_detect_merge_type_equal() - same OID returns UpToDate
  - IMPLEMENT: test_detect_merge_type_fast_forward() - ancestor detection
  - IMPLEMENT: test_detect_merge_type_local_ahead() - local ahead scenario
  - IMPLEMENT: test_detect_merge_type_divergent() - divergent histories
  - PATTERN: Use tempfile::TempDir for test isolation
  - PLACEMENT: In merge.rs under #[cfg(test)] mod tests
  - DEPENDS_ON: Task 1

Task 5: ADD integration test to tests/sync_workflow.rs
  - IMPLEMENT: test_pull_detects_fast_forward() - verify fast-forward detection
  - IMPLEMENT: test_pull_detects_divergent() - verify divergent detection
  - FOLLOW pattern: test_pull_merges_changes() structure
  - USE: setup_jin_with_remote() fixture
  - PLACEMENT: tests/sync_workflow.rs
  - DEPENDS_ON: Task 3

Task 6: VERIFY existing tests still pass
  - RUN: cargo test pull
  - RUN: cargo test sync_workflow
  - RUN: cargo test git::refs
  - VERIFY: No regressions in existing functionality
  - DEPENDS_ON: All previous tasks
```

### Implementation Patterns & Key Details

```rust
// PATTERN: MergeType enum follows RefComparison structure
// src/git/refs.rs lines 16-28 shows the pattern to follow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeType {
    UpToDate,      // Analogous to RefComparison::Equal
    FastForward,   // Local is ancestor of remote
    LocalAhead,    // Remote is ancestor of local
    Divergent,     // Both have unique commits
}

// PATTERN: graph_ahead_behind returns (ahead, behind)
// Match on tuple to determine state
let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;
match (ahead, behind) {
    (0, 0) => MergeType::UpToDate,
    (_, 0) => MergeType::FastForward,
    (0, _) => MergeType::LocalAhead,
    (_, _) => MergeType::Divergent,
}

// INTEGRATION: Modify LayerUpdateInfo in pull.rs
// Current structure (lines 73-82):
#[derive(Debug)]
struct LayerUpdateInfo {
    layer: Layer,
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    local_oid: Option<git2::Oid>,
    remote_oid: git2::Oid,
}

// NEW structure with merge_type:
#[derive(Debug)]
struct LayerUpdateInfo {
    layer: Layer,
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    local_oid: Option<git2::Oid>,
    remote_oid: git2::Oid,
    merge_type: MergeType,  // NEW FIELD
}

// INTEGRATION: Modify detect_updates() to set merge_type
// In detect_updates(), after parsing layer info:
let merge_type = match local_oid {
    Some(local) => crate::git::merge::detect_merge_type(jin_repo, local, remote_oid)?,
    None => MergeType::FastForward,  // New layers are always fast-forward
};

updates.insert(
    ref_path.clone(),
    LayerUpdateInfo {
        layer,
        mode,
        scope,
        project,
        local_oid,
        remote_oid,
        merge_type,  // NEW FIELD
    },
);

// INTEGRATION: Pull execution loop checks merge_type
// In execute(), lines 46-61:
for (ref_path, update_info) in &updates {
    match update_info.merge_type {
        MergeType::UpToDate => continue,  // Skip, already up to date
        MergeType::FastForward => {
            // Existing fast-forward logic
            tx.add_layer_update(
                update_info.layer,
                update_info.mode.as_deref(),
                update_info.scope.as_deref(),
                update_info.project.as_deref(),
                update_info.remote_oid,
            )?;
            println!("  ✓ {}: Updated (fast-forward)", format_ref_path(ref_path));
            merge_count += 1;
        }
        MergeType::LocalAhead => {
            // Local is ahead - no action needed for pull
            println!("  − {}: Local is ahead of remote", format_ref_path(ref_path));
        }
        MergeType::Divergent => {
            // TODO: P2.M1.T2 will implement 3-way merge
            // For now, skip or warn
            println!("  ! {}: Divergent history - 3-way merge not yet implemented",
                    format_ref_path(ref_path));
        }
    }
}

// PATTERN: Test fixture for creating divergent histories
// Use in integration tests (tests/sync_workflow.rs):
fn create_divergent_histories(
    remote_fixture: &RemoteFixture,
    mode_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create base commit and push
    // ... (create commit, push)

    // Step 2: Create divergent commit in remote (direct git2 manipulation)
    let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
    let ref_path = format!("refs/jin/layers/mode/{}", mode_name);

    // Get current commit as parent
    let current_ref = remote_repo.find_reference(&ref_path)?;
    let current_oid = current_ref.target().unwrap();
    let parent_commit = remote_repo.find_commit(current_oid)?;

    // Create divergent commit on remote
    let sig = remote_repo.signature()?;
    let mut tree_builder = remote_repo.treebuilder(None)?;
    let blob_oid = remote_repo.blob(b"remote content")?;
    tree_builder.insert("remote.txt", blob_oid, 0o100644)?;
    let tree_oid = tree_builder.write()?;
    let tree = remote_repo.find_tree(tree_oid)?;

    let remote_commit_oid = remote_repo.commit(
        Some(&ref_path),
        &sig,
        &sig,
        "Remote divergent",
        &tree,
        &[&parent_commit],
    )?;

    // Update remote ref
    let mut remote_ref = remote_repo.find_reference(&ref_path)?;
    remote_ref.set_target(remote_commit_oid, "Remote update")?;

    // Step 3: Create divergent commit in local
    // ... (similar to above, using local repo)

    Ok(())
}
```

### Integration Points

```yaml
CODEBASE INTEGRATION:
  - modify: src/git/mod.rs
    add: "pub mod merge;"
    add: "pub use merge::{MergeType, detect_merge_type};"

  - modify: src/commands/pull.rs
    add: "use crate::git::merge::{detect_merge_type, MergeType};"
    modify: LayerUpdateInfo struct (line ~73)
    modify: detect_updates() function (line ~85)
    modify: execute() loop (line ~46)

  - modify: tests/sync_workflow.rs
    add: test_pull_detects_fast_forward()
    add: test_pull_detects_divergent()

DEPENDENCIES:
  - Uses: git2::Repository::graph_ahead_behind() (already used in refs.rs)
  - Uses: git2::Oid for commit identifiers
  - Uses: JinRepo wrapper for repository access
  - Uses: JinError::Git for error handling

FUTURE INTEGRATION (P2.M1.T2):
  - MergeType::Divergent will trigger 3-way merge
  - Will use text_merge::text_merge() for file merging
  - Will use jinmerge module for conflict files
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check                           # Type checking
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
# If errors: READ output and fix before proceeding
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test merge detection module
cargo test git::merge::               # All merge detection tests
cargo test git::merge::tests::test_detect_merge_type_equal
cargo test git::merge::tests::test_detect_merge_type_fast_forward
cargo test git::merge::tests::test_detect_merge_type_local_ahead
cargo test git::merge::tests::test_detect_merge_type_divergent

# Run with output
cargo test git::merge:: -- --nocapture

# Expected: All tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Test pull command behavior
cargo test pull::                     # Pull command unit tests
cargo test sync_workflow::test_pull_merges_changes
cargo test sync_workflow::test_pull_detects_fast_forward
cargo test sync_workflow::test_pull_detects_divergent

# Manual testing with test fixtures
cargo test --test sync_workflow

# Expected: All integration tests pass
```

### Level 4: Manual Verification (Optional)

```bash
# Build the binary
cargo build --release

# Create test scenario with divergent histories
cd /tmp/test_divergent
jin init
mkdir -p .jin
# ... setup remote, create divergent histories ...

# Test pull behavior
jin pull
# Should detect divergent history and display appropriate message

# Expected: Correct merge type detection in output
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo test git::merge::` all tests pass
- [ ] `cargo test pull::` all tests pass
- [ ] `cargo test sync_workflow::` all tests pass
- [ ] `cargo test` all tests pass (including existing)

### Feature Validation

- [ ] `MergeType` enum has 4 variants: UpToDate, FastForward, LocalAhead, Divergent
- [ ] `detect_merge_type()` correctly identifies fast-forward scenarios
- [ ] `detect_merge_type()` correctly identifies divergent scenarios
- [ ] `detect_merge_type()` correctly identifies up-to-date scenarios
- [ ] `detect_merge_type()` correctly identifies local ahead scenarios
- [ ] Pull command integrates merge type detection
- [ ] Pull handles each merge type appropriately
- [ ] Existing fast-forward behavior is preserved

### Code Quality Validation

- [ ] `MergeType` enum derives Debug, Clone, Copy, PartialEq, Eq
- [ ] `detect_merge_type()` has comprehensive doc comments
- [ ] Unit tests cover all merge type scenarios
- [ ] Integration tests verify end-to-end behavior
- [ ] Error handling uses `JinError::Git` consistently
- [ ] Code follows existing patterns from `RefComparison`

### Documentation & Deployment

- [ ] Doc comments explain merge detection algorithm
- [ ] Doc comments include example usage
- [ ] Module-level documentation in merge.rs
- [ ] Public types are properly documented
- [ ] TODO comment in pull.rs updated to reference P2.M1.T2

---

## Anti-Patterns to Avoid

- ❌ Don't forget to check OID equality first (descendant_of returns false for same commit)
- ❌ Don't use unwrap() in library code - use `?` operator
- ❌ Don't break existing fast-forward behavior
- ❌ Don't implement 3-way merge in this task (that's P2.M1.T2)
- ❌ Don't ignore the LocalAhead case (local is ahead of remote)
- ❌ Don't use sync git operations - everything is async via git2
- ❌ Don't forget to handle the "new layer" case (no local_oid)
- ❌ Don't modify error types without careful consideration
- ❌ Don't skip unit tests for edge cases
- ❌ Don't use graph_ahead_behind incorrectly (order of parameters matters)

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Complete git2-rs API documentation extracted and referenced
- Exact code patterns from existing codebase (`RefComparison`, `compare_refs`)
- Comprehensive unit test examples for all merge scenarios
- Integration with existing test fixtures (`RemoteFixture`, `setup_jin_with_remote`)
- Clear task dependencies and implementation order
- Known gotchas documented (descendant_of behavior, OID equality check)

**Remaining Risks:**
- Test isolation for divergent history scenarios may need refinement
- Edge cases with unrelated merge bases need handling
- Integration with existing pull flow needs careful testing

---

## Research Artifacts Location

Research documentation stored at: `plan/P2M1T1/research/`

Key files:
- Implementation examples from codebase analysis
- git2-rs merge detection API documentation
- Test patterns from sync_workflow.rs
- External references for merge algorithms

External references:
- [git2-rs Repository Documentation](https://docs.rs/git2/latest/git2/struct.Repository.html)
- [git2-rs pull example](https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs)
- [Git merge-base documentation](https://git-scm.com/docs/git-merge-base)
- [Fast-forward merge explanation](https://git-scm.com/docs/git-merge#_fast_forward_merge)
