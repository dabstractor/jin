# Product Requirement Prompt: P1.M2.T2 - Implement Local vs Remote Comparison

---

## Goal

**Feature Goal**: Implement ahead/behind/diverged state detection between local and remote layer refs after fetch, preventing data loss by rejecting push operations when local commits are behind remote.

**Deliverable**:
1. A `compare_refs()` utility function in `src/git/refs.rs` that returns ahead/behind/diverged state
2. A new `BehindRemote` error variant in `src/core/error.rs`
3. Integration of comparison logic into `src/commands/push.rs` to reject push when behind
4. Integration tests validating the comparison and push rejection behavior

**Success Definition**:
- Push command detects when local layer refs are behind remote refs after automatic fetch
- Push is rejected with clear error message when behind (unless `--force` flag is used)
- Push succeeds when refs are up-to-date or when local is ahead only
- All tests pass including new integration tests for behind/diverged scenarios

## User Persona

**Target User**: Developer using Jin for collaborative configuration management

**Use Case**: Team member pushes layer changes to shared remote repository

**User Journey**:
1. Developer makes local changes and commits to a layer
2. Developer runs `jin push` to upload changes
3. Jin automatically fetches latest remote state (from P1.M2.T1)
4. **NEW**: Jin compares local vs remote refs for each modified layer
5. **NEW**: If local is behind remote, push is rejected with clear error message
6. Developer runs `jin pull` to merge remote changes
7. Developer runs `jin push` again (now succeeds)

**Pain Points Addressed**:
- **Data Loss**: Prevents accidental overwriting of remote commits
- **Silent Conflicts**: Catches divergent histories before they cause problems
- **Team Coordination**: Enforces pull-before-push workflow for safety

## Why

- **Safety**: Prevents accidental data loss when pushing to shared remotes
- **Collaboration**: Ensures team members are aware of remote changes before overwriting
- **Consistency**: Aligns with standard Git behavior (fast-forward only by default)
- **Foundation**: Enables future merge conflict detection and resolution features

## What

### User-Visible Behavior

**Before this change**:
```bash
$ jin push
Fetching from origin...
Already up to date  # May be misleading if fetch brought new refs
Pushing to origin...
Successfully pushed 1 layer(s)  # Could overwrite remote changes
```

**After this change**:
```bash
$ jin push
Fetching from origin...
Updates available:
  - mode/claude (1 file(s))

Run 'jin pull' to merge updates  # From P1.M2.T1

Pushing to origin...
Error: Push rejected: local layer 'mode/claude' is behind remote
The remote contains commits you don't have locally.
Run 'jin pull' to merge remote changes, or use '--force' to overwrite.
WARNING: --force may cause data loss!

$ jin pull
# ... merge happens ...

$ jin push
Fetching from origin...
Already up to date
Pushing to origin...
Successfully pushed 1 layer(s)
```

### Technical Requirements

1. **Comparison Utility**: `compare_refs(local_oid, remote_oid) -> RefComparison`
2. **State Enum**: `enum RefComparison { Ahead, Behind, Diverged, Equal }`
3. **Error Variant**: `JinError::BehindRemote { layer: String }`
4. **Push Integration**: Check comparison before each ref push
5. **Force Flag Override**: `--force` bypasses the behind check

### Success Criteria

- [ ] `compare_refs()` function correctly identifies ahead/behind/diverged states
- [ ] Push rejects with error when local is behind remote
- [ ] Push succeeds when local is ahead or equal to remote
- [ ] Push with `--force` bypasses behind check
- [ ] Error messages are clear and actionable
- [ ] Integration tests cover all scenarios (ahead, behind, diverged, equal, force)

---

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact file paths and line numbers to modify
- Complete code examples from existing similar implementations
- Specific git2-rs API usage with URLs
- Test patterns and fixture setup
- Error handling patterns
- All architectural context and gotchas

### Documentation & References

```yaml
# MUST READ - Critical for implementation

# Git2-rs Reference Comparison API
- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.graph_ahead_behind
  why: Core API for ahead/behind counting - used to detect ref state
  critical: Returns (ahead, behind) tuple - both zero means equal, only ahead means we can push, only behind means reject, both non-zero means diverged

# Git2-rs Merge Base API
- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.merge_base
  why: Alternative to graph_ahead_behind for finding common ancestor
  critical: Returns None if refs have no common ancestor (completely diverged)

# Git2-rs Reference Documentation
- url: https://docs.rs/git2/latest/git2/struct.Reference.html
  why: Understanding Git reference structure and OID resolution
  critical: reference.target() returns Option<Oid> - None means symbolic ref or peeled

# Existing Push Implementation (modify this file)
- file: /home/dustin/projects/jin/src/commands/push.rs
  why: This is the file to modify - adds comparison after fetch (line 24)
  pattern: Calls super::fetch::execute()? at line 24, then detect_modified_layers() at line 42
  gotcha: The detect_modified_layers function (line 105) has a TODO comment about remote ref comparison

# Existing Fetch Implementation (understand the flow)
- file: /home/dustin/projects/jin/src/commands/fetch.rs
  why: Fetch runs before push - need to understand what state it leaves
  pattern: report_updates() function (line 69) shows simple OID comparison at line 98
  gotcha: After fetch, both local and remote refs exist in Jin repo - need to compare them correctly

# Error Type Definitions (add new variant here)
- file: /home/dustin/projects/jin/src/core/error.rs
  why: Add BehindRemote error variant here
  pattern: Follow existing pattern like GitTracked or MergeConflict with descriptive message
  gotcha: Use #[error("...")] attribute with clear, actionable message

# Reference Operations Trait (add compare_refs method)
- file: /home/dustin/projects/jin/src/git/refs.rs
  why: Add compare_refs() utility function to this module
  pattern: Follow RefOps trait pattern - standalone function accepting JinRepo
  gotcha: Must handle error case where merge_base is None (completely diverged refs)

# Existing Test Patterns (follow these patterns)
- file: /home/dustin/projects/jin/tests/sync_workflow.rs
  why: Integration tests for push/fetch - add new tests here
  pattern: test_push_uploads_commits() shows RemoteFixture setup and verification
  gotcha: Tests use local filesystem remotes (file:// URLs) - no network needed

# Previous Task PRP (for context)
- docfile: /home/dustin/projects/jin/plan/P1M2T1/PRP.md
  why: Understand how fetch-before-push was implemented
  section: Lines 64, 230, 543, 816 mention P1.M2.T2 dependency
  gotcha: P1.M2.T1 added fetch call at line 24 of push.rs - P1.M2.T2 builds on this

# Git Research Documentation
- docfile: /home/dustin/projects/jin/plan/P1M2/research/phantom_git_patterns.md
  why: Understanding Jin's phantom Git architecture and ref namespace
  section: Sections on ref namespace and comparison patterns
  gotcha: All refs under refs/jin/layers/* - never touch user's .git
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── commands/
│   │   ├── push.rs          # MODIFY: Add comparison logic
│   │   ├── fetch.rs         # REFERENCE: Understand fetch flow
│   │   └── mod.rs           # Command routing
│   ├── git/
│   │   ├── refs.rs          # MODIFY: Add compare_refs() function
│   │   ├── repo.rs          # REFERENCE: JinRepo wrapper
│   │   ├── remote.rs        # REFERENCE: Remote operations
│   │   └── mod.rs           # Git module exports
│   ├── core/
│   │   ├── error.rs         # MODIFY: Add BehindRemote variant
│   │   └── mod.rs           # Core exports
│   └── cli/
│       ├── args.rs          # REFERENCE: PushArgs structure
│       └── mod.rs           # CLI definitions
└── tests/
    ├── sync_workflow.rs     # MODIFY: Add integration tests
    └── common/
        ├── fixtures.rs      # REFERENCE: RemoteFixture setup
        └── assertions.rs    # REFERENCE: Custom assertions
```

### Desired Codebase Tree with Files to be Added

```bash
jin/
├── src/
│   ├── commands/
│   │   └── push.rs          # MODIFY: Add compare_refs() call before push
│   ├── git/
│   │   └── refs.rs          # MODIFY: Add compare_refs() and RefComparison enum
│   └── core/
│       └── error.rs         # MODIFY: Add JinError::BehindRemote variant
└── tests/
    └── sync_workflow.rs     # MODIFY: Add test_push_rejected_when_behind* tests
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: git2-rs graph_ahead_behind() returns (usize, usize)
// First value = ahead count, Second value = behind count
// BOTH zero -> refs are equal (same commit)
// ahead > 0, behind = 0 -> local is ahead, safe to push
// ahead = 0, behind > 0 -> local is behind, REJECT PUSH
// ahead > 0, behind > 0 -> refs have diverged, REJECT PUSH

// CRITICAL: After fetch, remote refs are in LOCAL Jin repo under same names
// Don't look for "origin/" prefix - fetch downloads remote refs to local storage
// To compare: get local ref OID, get remote-tracking ref OID, then compare

// GOTCHA: The detect_modified_layers() function in push.rs:123 has this pattern:
//   if jin_repo.ref_exists(&ref_name) {
//       match jin_repo.resolve_ref(&ref_name) {
//           Ok(remote_oid) => local_oid != remote_oid,
//           ...
//       }
//   }
// This is BUGGY - it compares local ref to itself, not to remote!
// This is the bug we need to fix with proper comparison.

// GOTCHA: Jin uses bare repository at ~/.jin/ - refs are NOT in user's .git
// All operations on Jin repo, never on user's Git repository

// GOTCHA: User-local layer (refs/jin/layers/local) is NEVER pushed
// Always skip refs containing "/local" in comparison

// GOTCHA: git2::Repository::merge_base() returns Option<Oid>
// None means refs have completely diverged (no common ancestor)
// Some(Oid) is the merge base - use for comparison

// GOTCHA: Repository::graph_ahead_behind() can fail if commits aren't in graph
// Handle error gracefully - fall back to merge_base or OID comparison

// PATTERN: Error messages use thiserror derive - add #[error("...")] attribute
// Example: #[error("Push rejected: local layer '{layer}' is behind remote")]

// PATTERN: Integration tests use RemoteFixture from tests/common/fixtures.rs
// Creates local and bare remote repos in temp directory - auto cleanup
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// Add to src/git/refs.rs

/// Result of comparing two Git references
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefComparison {
    /// Local commit is ahead of remote (can push)
    Ahead,
    /// Local commit is behind remote (reject push)
    Behind,
    /// Local and remote have diverged (reject push)
    Diverged,
    /// Local and remote point to same commit
    Equal,
}

// Add to src/core/error.rs

/// Add to JinError enum:
/// Push rejected: local is behind remote
#[error("Push rejected: local layer '{layer}' is behind remote.\n\
The remote contains commits you don't have locally.\n\
Run 'jin pull' to merge remote changes, or use '--force' to overwrite.\n\
WARNING: --force may cause data loss!")]
BehindRemote { layer: String },
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD RefComparison enum to src/git/refs.rs
  - IMPLEMENT: RefComparison enum with Ahead, Behind, Diverged, Equal variants
  - FOLLOW pattern: Layer enum in src/core/layer.rs (derive Debug, Clone, Copy, PartialEq, Eq)
  - NAMING: CamelCase variants, descriptive names
  - PLACEMENT: Top of src/git/refs.rs after imports, before RefOps trait
  - DOCUMENTATION: Add doc comments explaining each state

Task 2: IMPLEMENT compare_refs() function in src/git/refs.rs
  - IMPLEMENT: pub fn compare_refs(repo: &JinRepo, local_oid: Oid, remote_oid: Oid) -> Result<RefComparison>
  - ALGORITHM:
    1. Call repo.graph_ahead_behind(local_commit, remote_commit)?
    2. Match on (ahead, behind):
       - (0, 0) => Equal
       - (n, 0) where n > 0 => Ahead
       - (0, n) where n > 0 => Behind
       - (m, n) where m > 0 && n > 0 => Diverged
  - ERROR HANDLING: Convert git2::Error to JinError::Git with ?
  - PLACEMENT: After RefOps impl block, before tests module
  - DEPENDENCIES: RefComparison enum from Task 1

Task 3: ADD JinError::BehindRemote variant to src/core/error.rs
  - IMPLEMENT: BehindRemote { layer: String } variant
  - FOLLOW pattern: Existing variants like MergeConflict { path: String }
  - MESSAGE: Clear, actionable error with pull suggestion and force warning
  - PLACEMENT: After MergeConflict variant, before Transaction variant
  - NAMING: BehindRemote (consistent with naming convention)

Task 4: MODIFY detect_modified_layers() in src/commands/push.rs
  - CURRENT BUG: Line 127 compares local_oid to itself (remote_oid is actually local)
  - FIX IMPLEMENTATION:
    1. After fetch, remote refs are in Jin repo
    2. For each local ref, compare local OID with remote OID
    3. Use compare_refs() to determine state
    4. If state is Behind or Diverged, return error with layer name
  - INTEGRATION: Call compare_refs(jin_repo, local_oid, remote_oid)?
  - ERROR: Return Err(JinError::BehindRemote { layer: ref_name })
  - EXCEPTION: Skip check if args.force is true
  - PLACEMENT: Modify function starting at line 105
  - DEPENDENCIES: compare_refs from Task 2, JinError::BehindRemote from Task 3

Task 5: ADD integration test: test_push_rejected_when_behind
  - IMPLEMENT: Test in tests/sync_workflow.rs
  - SETUP:
    1. Create RemoteFixture with local and bare remote
    2. Create commit in remote (via temp workspace)
    3. Push to remote
    4. Create divergent commit in local
  - VERIFY:
    1. Run jin push (should fail)
    2. Check stderr contains "behind remote"
    3. Check stderr mentions "jin pull"
  - FOLLOW pattern: test_push_uploads_commits() at line 227
  - NAMING: test_push_rejected_when_behind
  - PLACEMENT: After test_push_uploads_commits, before test_sync_complete_workflow

Task 6: ADD integration test: test_push_succeeds_with_force_when_behind
  - IMPLEMENT: Test with --force flag
  - SETUP: Same as Task 5
  - VERIFY:
    1. Run jin push --force (should succeed)
    2. Verify commit was pushed to remote
  - FOLLOW pattern: Similar to Task 5 but with --force flag
  - NAMING: test_push_succeeds_with_force_when_behind
  - PLACEMENT: After Task 5 test

Task 7: ADD integration test: test_push_succeeds_when_ahead
  - IMPLEMENT: Test for ahead scenario
  - SETUP:
    1. Create local commit
    2. Push to remote
    3. Create another local commit
    4. Push again (should succeed - ahead is OK)
  - VERIFY: Push succeeds without error
  - FOLLOW pattern: test_push_uploads_commits
  - NAMING: test_push_succeeds_when_ahead
  - PLACEMENT: After Task 6 test

Task 8: ADD unit tests for compare_refs() in src/git/refs.rs
  - IMPLEMENT: Unit tests in #[cfg(test)] mod tests
  - TEST CASES:
    1. Equal refs (same OID)
    2. Local ahead (local is descendant of remote)
    3. Local behind (remote is descendant of local)
    4. Diverged (both ahead and behind)
  - FOLLOW pattern: Existing tests like test_resolve_ref at line 214
  - NAMING: test_compare_refs_equal, test_compare_refs_ahead, etc.
  - PLACEMENT: In tests module at bottom of src/git/refs.rs
  - DEPENDENCIES: compare_refs function from Task 2
```

### Implementation Patterns & Key Details

```rust
// ============================================================
// PATTERN 1: RefComparison enum definition
// ============================================================
// File: src/git/refs.rs (add after imports, before RefOps trait)

/// Result of comparing two Git references
///
/// Used to determine if a push operation is safe to execute.
/// Derived from git's graph_ahead_behind comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefComparison {
    /// Local commit is ahead of remote (fast-forward possible)
    Ahead,

    /// Local commit is behind remote (must pull first)
    Behind,

    /// Local and remote have diverged (merge required)
    Diverged,

    /// Local and remote point to the same commit
    Equal,
}

// ============================================================
// PATTERN 2: compare_refs() utility function
// ============================================================
// File: src/git/refs.rs (add after RefOps impl block)

/// Compare local and remote Git references
///
/// Determines the relationship between two commits by analyzing
/// the commit graph. Returns whether local is ahead, behind,
/// diverged, or equal to remote.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `local_oid` - OID of local commit
/// * `remote_oid` - OID of remote commit
///
/// # Returns
///
/// `RefComparison` indicating the state relationship
///
/// # Errors
///
/// Returns `JinError::Git` if commit graph analysis fails
///
/// # Algorithm
///
/// Uses git2's `graph_ahead_behind` which counts commits
/// reachable from each ref but not the other:
/// - (0, 0) -> Same commit
/// - (n, 0) -> Local is ahead (n commits ahead, 0 behind)
/// - (0, n) -> Local is behind (0 ahead, n behind)
/// - (m, n) -> Diverged (both have unique commits)
pub fn compare_refs(
    repo: &JinRepo,
    local_oid: Oid,
    remote_oid: Oid,
) -> Result<RefComparison> {
    // Convert OIDs to Commit objects for graph analysis
    let local_commit = repo.inner().find_commit(local_oid)?;
    let remote_commit = repo.inner().find_commit(remote_oid)?;

    // CRITICAL: graph_ahead_behind returns (ahead_count, behind_count)
    // This is the core of git's ahead/behind detection
    let (ahead, behind) = repo.inner().graph_ahead_behind(&local_commit, &remote_commit)?;

    // Match on counts to determine state
    match (ahead, behind) {
        (0, 0) => Ok(RefComparison::Equal),
        (_, 0) => Ok(RefComparison::Ahead),
        (0, _) => Ok(RefComparison::Behind),
        (_, _) => Ok(RefComparison::Diverged),
    }
}

// ============================================================
// PATTERN 3: Error variant definition
// ============================================================
// File: src/core/error.rs (add after MergeConflict variant)

/// Push rejected: local layer is behind remote
#[error(
    "Push rejected: local layer '{layer}' is behind remote.\n\
The remote contains commits you don't have locally.\n\
Run 'jin pull' to merge remote changes, or use '--force' to overwrite.\n\
WARNING: --force may cause data loss!"
)]
BehindRemote { layer: String },

// ============================================================
// PATTERN 4: Integration into push command
// ============================================================
// File: src/commands/push.rs (modify detect_modified_layers function)

// CURRENT CODE (BUGGY - compares local to itself):
// Line 123-130:
// let should_push = if jin_repo.ref_exists(&ref_name) {
//     match jin_repo.resolve_ref(&ref_name) {
//         Ok(remote_oid) => local_oid != remote_oid,  // BUG: This is local OID!
//         Err(_) => true,
//     }
// } else {
//     true
// };

// REPLACEMENT CODE:
// After fetch, remote refs are downloaded to local Jin repo.
// We need to properly compare local vs remote.

fn detect_modified_layers(jin_repo: &JinRepo, args: &PushArgs) -> Result<Vec<String>> {
    let local_refs = jin_repo.list_refs("refs/jin/layers/*")?;
    let mut modified = Vec::new();

    for ref_name in local_refs {
        // Skip user-local layer
        if ref_name.contains("/local") {
            continue;
        }

        let local_oid = jin_repo.resolve_ref(&ref_name)?;

        // CRITICAL: After fetch, check if remote exists in our Jin repo
        // Remote refs are fetched to local storage under same names
        let should_push = match jin_repo.resolve_ref(&ref_name) {
            Ok(remote_oid) => {
                // Remote ref exists - compare states
                if args.force {
                    // Force flag bypasses safety checks
                    true
                } else {
                    // Compare refs to determine if push is safe
                    match crate::git::refs::compare_refs(jin_repo, local_oid, remote_oid)? {
                        RefComparison::Ahead | RefComparison::Equal => true,
                        RefComparison::Behind | RefComparison::Diverged => {
                            // REJECT: Local is behind or diverged from remote
                            return Err(JinError::BehindRemote {
                                layer: ref_name.clone(),
                            });
                        }
                    }
                }
            }
            Err(_) => {
                // No remote ref exists - this is a new local ref, push it
                true
            }
        };

        if should_push {
            modified.push(ref_name);
        }
    }

    Ok(modified)
}

// NOTE: This requires changing function signature to accept PushArgs
// Also update call site at line 42: detect_modified_layers(&jin_repo, &args)?

// ============================================================
// PATTERN 5: Integration test setup
// ============================================================
// File: tests/sync_workflow.rs (add new tests)

/// Test push rejected when local is behind remote
#[test]
fn test_push_rejected_when_behind() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("behind_test_{}", unique_test_id());
    let jin_dir = remote_fixture.local_path.join(".jin_global");

    // Step 1: Create remote commit via temp workspace
    let temp_workspace = TestFixture::new()?;
    temp_workspace.set_jin_dir();
    jin_init(temp_workspace.path())?;

    create_mode(&mode_name, Some(&jin_dir))?;
    jin().args(["mode", "use", &mode_name])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::write(temp_workspace.path().join("remote.txt"), "remote content")?;
    jin().args(["add", "remote.txt", "--mode"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin().args(["commit", "-m", "Remote commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Push to remote
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 2: In local repo, create divergent commit on same layer
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_mode(&mode_name, Some(&jin_dir))?;
    jin().args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create local commit (don't fetch first - simulating being behind)
    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;
    jin().args(["add", "local.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin().args(["commit", "-m", "Local commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Step 3: Try to push - should be rejected
    jin().arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("behind remote"))
        .stderr(predicate::str::contains("jin pull"))
        .stderr(predicate::str::contains("--force"));

    Ok(())
}
```

### Integration Points

```yaml
MODIFIED FILES:
  - src/commands/push.rs:
    - Function: detect_modified_layers() - add comparison logic
    - Change: Add args parameter, integrate compare_refs()
    - Line: ~105-141

  - src/git/refs.rs:
    - Addition: RefComparison enum (after imports)
    - Addition: compare_refs() function (after RefOps impl)
    - Dependency: Uses JinRepo from src/git/repo.rs

  - src/core/error.rs:
    - Addition: JinError::BehindRemote variant
    - Placement: After MergeConflict variant

  - tests/sync_workflow.rs:
    - Addition: 3 new integration tests
    - Placement: After existing push tests

DEPENDENCIES:
  - git2-rs: Repository::graph_ahead_behind()
  - git2-rs: Repository::find_commit()
  - Existing RefOps trait
  - Existing JinRepo wrapper

NO CHANGES NEEDED:
  - src/cli/args.rs (PushArgs already has force field)
  - src/git/repo.rs (JinRepo already implements needed access)
  - src/commands/fetch.rs (no changes - already working)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# After each file creation/modification - fix before proceeding
cargo check --bin jin                    # Check compilation
cargo clippy --bin jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                        # Verify formatting

# Project-wide validation after all changes
cargo check
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Expected: Zero errors, zero warnings, proper formatting
# If errors exist, READ output and fix before proceeding
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the compare_refs function specifically
cargo test compare_refs -- --exact        # Test only compare_refs tests
cargo test --lib src/git/refs             # Test all refs.rs tests

# Test error display
cargo test test_error_display            # Verify new error variant works

# Full unit test suite
cargo test --lib                          # Run all library unit tests

# Coverage validation (optional)
cargo tarpaulin --out Html --output-dir coverage/

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Run the new integration tests
cargo test test_push_rejected_when_behind -- --exact
cargo test test_push_succeeds_with_force_when_behind -- --exact
cargo test test_push_succeeds_when_ahead -- --exact

# Run all sync workflow tests
cargo test --test sync_workflow

# Manual testing with real jin binary
cargo build --release
./target/release/jin init
./target/release/jin link <remote-path>
# ... create commits ...
./target/release/jin push    # Test normal push
./target/release/jin push --force   # Test force push

# Expected: All integration tests pass, manual testing works as specified
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test various ref states with git commands

# Setup test scenario
cd /tmp && mkdir test_jin && cd test_jin
git init
jin init

# Create remote and local repos
mkdir remote && cd remote && git init --bare && cd ..
jin link file://$(pwd)/remote

# Test 1: Ahead scenario (should succeed)
jin mode use test
echo "v1" > file.txt && jin add file.txt --mode && jin commit -m "v1"
jin push
echo "v2" > file.txt && jin add file.txt --mode && jin commit -m "v2"
jin push    # Should succeed (ahead)

# Test 2: Behind scenario (should fail)
# (From another workspace, push v3 to remote)
# Then back here, create v4 locally and try push
jin push    # Should fail with "behind remote" error

# Test 3: Diverged scenario (should fail)
# Create commits that diverge from remote
jin push    # Should fail with "behind remote" error

# Test 4: Force push (should succeed)
jin push --force    # Should succeed despite being behind

# Expected: All scenarios behave as specified, error messages are clear
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] No compilation errors: `cargo check`

### Feature Validation

- [ ] Push rejects when local is behind remote (without --force)
- [ ] Push succeeds when local is ahead of remote
- [ ] Push succeeds when local and remote are equal
- [ ] Push with --force succeeds even when behind
- [ ] Error message mentions "jin pull" and "--force"
- [ ] Error message includes the layer name that's behind
- [ ] All success criteria from "What" section met

### Code Quality Validation

- [ ] Follows existing codebase patterns (RefOps trait, JinError enum)
- [ ] File placement matches desired codebase tree
- [ ] Anti-patterns avoided (no duplicate code, proper error handling)
- [ ] Dependencies properly imported (use crate::git::refs::compare_refs)
- [ ] Function signatures use appropriate types (Result<T>, JinRepo)

### Documentation & Deployment

- [ ] Public functions have doc comments (compare_refs)
- [ ] Enum variants have documentation (RefComparison)
- [ ] Error messages are user-friendly and actionable
- [ ] Code is self-documenting with clear variable names

---

## Anti-Patterns to Avoid

- **Don't compare OIDs directly for state detection** - Use `graph_ahead_behind()` for proper ahead/behind detection
- **Don't skip the comparison when --force is set** - Check the flag first, then compare
- **Don't forget to handle the Equal case** - Equal refs don't need pushing
- **Don't compare local ref to itself** - After fetch, need to properly identify remote refs
- **Don't ignore Diverged state** - Diverged refs are different from behind, both should reject
- **Don't forget to skip user-local layer** - Always filter out refs containing "/local"
- **Don't use sync functions for async operations** - All git2 operations are synchronous
- **Don't catch all exceptions** - Be specific with error types
- **Don't hardcode ref names** - Use the ref_name from iteration
- **Don't modify user's .git repository** - Only operate on Jin repo at ~/.jin/

---

## Implementation Summary

This PRP implements local vs remote ref comparison for the Jin push command, building on the automatic fetch-before-push from P1.M2.T1. The implementation:

1. **Adds comparison logic** using git2-rs's `graph_ahead_behind()` API
2. **Prevents data loss** by rejecting push when local is behind or diverged from remote
3. **Maintains workflow flexibility** with `--force` flag override
4. **Provides clear error messages** guiding users to pull or force push

The implementation follows existing codebase patterns (RefOps trait, JinError enum, RemoteFixture tests) and integrates seamlessly with the current push command flow.

**Key Files Modified**:
- `src/git/refs.rs` - Add RefComparison enum and compare_refs() function
- `src/core/error.rs` - Add BehindRemote error variant
- `src/commands/push.rs` - Integrate comparison logic into detect_modified_layers()
- `tests/sync_workflow.rs` - Add integration tests for all scenarios

---

## Confidence Score

**9/10** - High confidence for one-pass implementation success

**Rationale**:
- Comprehensive codebase research with exact file paths and line numbers
- Clear algorithm using well-documented git2-rs API
- Existing test patterns to follow (RemoteFixture, sync_workflow tests)
- Minimal changes required (4 files, ~100 lines of new code)
- Clear integration points with existing code
- Specific validation commands for each level

**Risk Mitigation**:
- The main risk is understanding the ref state after fetch - this is documented clearly
- Test fixtures already exist for remote operations
- The comparison algorithm is standard git practice
- Error handling patterns are well-established in the codebase

---

## Success Metrics

**Implementation Success**:
- [ ] PRP followed completely with all tasks completed in order
- [ ] All validation levels pass without modification
- [ ] Code review finds no deviations from patterns
- [ ] Zero bugs found in initial testing

**Feature Success**:
- [ ] Push behavior matches specification exactly
- [ ] Error messages are clear and actionable
- [ ] No regressions in existing push functionality
- [ ] Force flag behaves as specified

**Quality Success**:
- [ ] Code follows all existing patterns
- [ ] No clippy warnings or compiler errors
- [ ] All tests pass (unit + integration)
- [ ] Documentation is complete and accurate
