# PRP: P2.M1.T3 - Add Integration Tests for 3-Way Merge

---

## Goal

**Feature Goal**: Ensure comprehensive integration test coverage for 3-way merge functionality in the `jin pull` command, validating fast-forward, divergent clean merge, and divergent with conflicts scenarios.

**Deliverable**: Comprehensive integration test suite in `tests/pull_merge.rs` that validates all 3-way merge scenarios with proper edge case coverage.

**Success Definition**:
- All integration tests pass: `cargo test --test pull_merge`
- Fast-forward merge behavior is verified (regression test)
- Clean 3-way merge produces correct merged content
- Conflicted 3-way merge creates `.jinmerge` files correctly
- Merge commit structure is validated (two parents)
- Edge cases are covered (multiple files, different layers)

---

## User Persona

**Target User**: Jin CLI users working with remote repositories who may experience divergent histories when multiple contributors push to the same layer.

**Use Case**: Users pull remote changes and the system correctly handles:
1. Fast-forward merges (no local changes)
2. Clean 3-way merges (non-conflicting divergent changes)
3. Conflicted 3-way merges (overlapping changes requiring resolution)

**User Journey**:
1. User makes local commits to a layer
2. Remote receives updates from other users
3. User runs `jin pull`
4. System detects merge type and performs appropriate merge
5. For conflicts: `.jinmerge` files are created for manual resolution
6. User resolves conflicts with `jin resolve` and runs `jin apply`

**Pain Points Addressed**:
- Confident team collaboration without fear of merge failures
- Clear conflict detection and resolution workflow
- Verification that existing fast-forward behavior isn't broken

---

## Why

- **Validates P2.M1.T1 and P2.M1.T2**: Ensures merge detection and 3-way merge implementation work correctly
- **Regression Prevention**: Catches any future changes that break merge behavior
- **Edge Case Coverage**: Identifies issues with complex merge scenarios
- **Documentation**: Tests serve as executable documentation of merge behavior
- **Team Confidence**: Enables distributed collaboration with verified merge logic

---

## What

### User-Visible Behavior

Integration tests verify that `jin pull` correctly handles three merge scenarios:

1. **Fast-Forward**: Local is behind remote (linear history)
   - Output: `✓ global: Updated (fast-forward)`
   - Layer ref points to remote commit
   - Workspace updates with remote content

2. **Clean 3-Way Merge**: Divergent histories without conflicts
   - Output: `✓ global: Merged (3-way)`
   - Merge commit created with two parents
   - Merged content combines both changes
   - Workspace updates with merged content

3. **Conflicted 3-Way Merge**: Divergent histories with overlapping changes
   - Output: `! global: Merged with 1 conflicts`
   - `.jinmerge` files created for conflicted files
   - Layer ref points to merge commit
   - User must resolve conflicts before `jin apply`

### Success Criteria

- [ ] All 4 existing tests pass and are validated
- [ ] Fast-forward regression test covers global layer
- [ ] Clean merge test covers non-overlapping line changes
- [ ] Conflict test creates valid `.jinmerge` files
- [ ] Different files test verifies both branches' files are merged
- [ ] Optional: Additional edge case tests added (if gaps identified)

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

Yes. This PRP provides:
- Exact file paths for all test files
- Complete test patterns to follow
- Specific validation commands
- External references for merge testing best practices
- Analysis of existing test coverage
- Identification of potential gaps

### Documentation & References

```yaml
# MUST READ - Existing Test Infrastructure

# Existing Integration Tests (PRIMARY REFERENCE)
- file: tests/pull_merge.rs
  why: Contains 4 existing tests that already cover the main scenarios
  pattern: |
    Lines 1-91: test_pull_fast_forward_still_works() - regression test
    Lines 93-247: test_pull_divergent_clean_merge() - non-overlapping changes
    Lines 249-403: test_pull_divergent_with_conflicts() - overlapping changes
    Lines 405-557: test_pull_divergent_clean_merge_different_files() - file addition
  critical: |
    - Uses setup_jin_with_remote() for local + remote repos
    - Uses TestFixture for temp workspace to push changes
    - Tests use --force flag on link to allow overwriting remotes
    - Assertions check stdout for expected merge messages
    - Validates .jinmerge file format with conflict markers

# Test Fixtures and Helpers
- file: tests/common/mod.rs
  why: Module exports for common test utilities
  pattern: pub mod fixtures, pub mod assertions, pub mod git_helpers

- file: tests/common/fixtures.rs
  why: Test fixture creation for isolated repo environments
  pattern: |
    setup_jin_with_remote() -> RemoteFixture (lines 154-164)
    TestFixture::new() creates temp directory with isolated JIN_DIR
    RemoteFixture has local_path, remote_path, jin_dir fields
    create_commit_in_repo() for making git commits programmatically

- file: tests/common/assertions.rs
  why: Custom assertions for Jin-specific state verification
  pattern: |
    assert_workspace_file() - check file exists with expected content
    assert_jin_initialized() - verify .jin directory exists
    assert_layer_ref_exists() - verify layer ref in Jin repository

- file: tests/common/git_helpers.rs
  why: Git lock cleanup utilities to prevent test failures
  pattern: |
    cleanup_git_locks() - removes .git/index.lock and other stale locks
    GitTestEnv wrapper for automatic cleanup on Drop

# Implementation Being Tested
- file: src/commands/pull.rs
  why: Main pull command implementation with 3-way merge logic
  pattern: |
    Lines 54-127: execute() function with merge type handling
    Lines 78-124: Divergent case calls perform_three_way_merge()
    Lines 279-404: perform_three_way_merge() implementation
    Lines 262-277: MergeOutcome enum definition

- file: src/git/merge.rs
  why: Merge type detection and merge base finding
  pattern: |
    Lines 14-27: MergeType enum (UpToDate, FastForward, LocalAhead, Divergent)
    Lines 69-87: detect_merge_type() using graph_ahead_behind
    Lines 152-165: find_merge_base() for common ancestor

- file: src/merge/text.rs
  why: 3-way text merge using diffy crate
  pattern: |
    text_merge(base, ours, theirs) returns TextMergeResult
    Clean(String) for successful merge
    Conflict { content, conflict_count } for conflicts

- file: src/merge/jinmerge.rs
  why: .jinmerge conflict file format and generation
  pattern: |
    JinMergeConflict::from_text_merge() creates conflict structure
    write_to_file() writes conflict markers to .jinmerge file
    merge_path_for_file() returns <file>.jinmerge path

# External Research - Git Merge Testing Best Practices
- url: https://medium.com/@iiiamigoes/understanding-git-merge-keep-your-project-history-clean-fd69db1e1dbe
  why: Comprehensive guide to merge types and testing strategies
  critical: |
    - Fast-forward: linear history, one parent
    - 3-way merge: divergent history, two parents
    - Conflict markers: <<<<<<<, =======, >>>>>>> format
    - Test with both same-file and different-file changes

- url: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
  why: Official git2-rs example showing fast-forward and normal merge
  critical: |
    - fast_forward() function for simple ref update
    - normal_merge() function for 3-way merge with conflicts
    - Uses merge_trees() to detect conflicts
    - Creates merge commit with two parents

- url: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
  why: Deep dive into 3-way merge algorithm and diff3 format
  critical: |
    - Explains why merge base is critical for conflict detection
    - Shows how diff3 determines if changes conflict
    - Best practices for merge conflict resolution

# Related PRPs for Context
- file: plan/P2M1T1/PRP.md
  why: Describes merge type detection implementation
  section: Implementation Blueprint shows MergeType enum usage

- file: plan/P2M1T2/PRP.md
  why: Describes 3-way merge implementation
  section: Implementation Patterns shows perform_three_way_merge() logic
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── commands/
│   │   └── pull.rs              # Implementation being tested
│   ├── git/
│   │   ├── merge.rs             # Merge detection (MergeType, find_merge_base)
│   │   ├── transaction.rs       # LayerTransaction for atomic updates
│   │   └── tree.rs              # TreeOps for file content reading
│   ├── merge/
│   │   ├── text.rs              # 3-way text merge (text_merge)
│   │   └── jinmerge.rs          # .jinmerge conflict files
│   └── core/
│       ├── error.rs             # JinError types
│       └── layer.rs             # Layer enum with ref_path()
└── tests/
    ├── pull_merge.rs            # EXISTING: 4 integration tests (557 lines)
    └── common/
        ├── mod.rs               # Module exports
        ├── fixtures.rs          # Test fixtures (setup_jin_with_remote)
        ├── assertions.rs        # Custom assertions
        └── git_helpers.rs       # Git lock cleanup
```

### Existing Test Coverage Analysis

The file `tests/pull_merge.rs` already contains 4 comprehensive tests:

| Test Name | Lines | Scenario | Coverage |
|-----------|-------|----------|----------|
| `test_pull_fast_forward_still_works` | 17-91 | Fast-forward merge | Creates base commit in remote, pulls to local repo |
| `test_pull_divergent_clean_merge` | 98-247 | Clean 3-way merge | Non-overlapping changes (modify setting1, add setting3) |
| `test_pull_divergent_with_conflicts` | 254-403 | Conflicted merge | Overlapping changes (same line, different values) |
| `test_pull_divergent_clean_merge_different_files` | 410-557 | File addition | Different files added in each branch |

### Potential Gaps and Additional Test Scenarios

Based on external best practices and codebase analysis, consider adding:

```yaml
Edge Cases to Consider:
  - Multiple file conflicts: Only single file conflict tested
  - Empty file scenarios: Adding empty file in one branch
  - Whitespace conflicts: Changes only in whitespace
  - Binary files: Not currently supported by text_merge
  - Mode/Scope/Project layers: Only global layer tested

Layer-Specific Tests:
  - Mode base layer merge (e.g., mode/claude)
  - Mode + scope layer merge (e.g., mode/claude/scope/language:rust)
  - Project layer merge (e.g., mode/claude/project/dashboard)

Merge Structure Validation:
  - Verify merge commit has exactly 2 parents
  - Verify merge commit message format
  - Verify parent order (local first, remote second)
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Test fixture setup requires two separate Jin repos
// One for pushing to remote, one for pulling from remote
let temp_workspace = TestFixture::new()?;  // Used to push changes
let remote_fixture = setup_jin_with_remote()?;  // Used to pull changes

// CRITICAL: Always use --force flag when linking repos in tests
// Otherwise second link attempt will fail
jin().args(["link", remote_path, "--force"])
    .current_dir(project_path)
    .env("JIN_DIR", jin_dir)
    .assert()
    .success();

// CRITICAL: Isolate tests using JIN_DIR environment variable
// Each fixture has its own jin_dir for test isolation
let jin_dir = fixture.jin_dir.as_ref().unwrap();
jin().arg("pull")
    .env("JIN_DIR", jin_dir)
    .assert()
    .success();

// CRITICAL: After pull, run jin apply to verify merged content in workspace
// The merged content is in the layer, not the workspace until apply
jin().arg("apply")
    .current_dir(&local_path)
    .env("JIN_DIR", jin_dir)
    .assert()
    .success();

// GOTCHA: .jinmerge files are created in the current working directory
// They are NOT in the layer - they're workspace files
let jinmerge_path = local_path.join("config.txt.jinmerge");
assert!(jinmerge_path.exists());

// PATTERN: Check stdout for expected merge messages
// Tests verify the pull command outputs correct status
.stdout(predicates::str::contains("✓ global: Merged (3-way)"))
.stdout(predicates::str::contains("! global: Merged with 1 conflicts"))

// GOTCHA: Create divergent commits using TWO separate workspaces
// One workspace pushes to remote, then second workspace pulls
// This simulates two users collaborating

// CRITICAL: Git lock cleanup happens automatically on Drop
// Fixtures implement Drop to call cleanup_git_locks()
// No manual cleanup needed in tests
```

---

## Implementation Blueprint

### Test Structure and Patterns

```rust
// ================== tests/pull_merge.rs ==================
//! 3-way merge integration tests for jin pull
//!
//! Tests the 3-way merge implementation in the pull command for divergent
//! layer histories. Validates clean merges, conflict handling, and .jinmerge
//! file creation.

use std::fs;

mod common;
use common::fixtures::*;

// ============================================================================
// Test 1: Fast-Forward Regression Test
// ============================================================================
// Location: Lines 17-91
// Purpose: Verify fast-forward merge still works after 3-way merge added

#[test]
fn test_pull_fast_forward_still_works() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create remote fixture and temp workspace for pushing
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // STEP 1: Create initial commit in "remote" via temp workspace
    fs::write(temp_workspace.path().join("config.txt"), "version=1")?;
    jin().args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Initial commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // STEP 2: Link to remote and push
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // STEP 3: Setup main local repo - link and fetch
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // TEST: Pull should fast-forward
    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ global: Updated (fast-forward)"));

    Ok(())
}

// ============================================================================
// Test 2: Clean 3-Way Merge (Non-Overlapping Changes)
// ============================================================================
// Location: Lines 98-247
// Purpose: Verify divergent histories merge cleanly when changes don't conflict

#[test]
fn test_pull_divergent_clean_merge() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Same as Test 1
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // STEP 1: Create base commit with multi-line file
    fs::write(
        temp_workspace.path().join("config.txt"),
        "setting1=value1\nsetting2=value2",
    )?;
    jin().args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Base commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Push base commit
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Setup local repo with base
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // STEP 2: Make LOCAL change (add setting3 at end - non-overlapping)
    fs::write(
        remote_fixture.local_path.join("config.txt"),
        "setting1=value1\nsetting2=value2\nsetting3=local",
    )?;

    jin().args(["add", "config.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Local change"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // STEP 3: Make REMOTE change (modify setting1 - non-overlapping with setting3)
    fs::write(
        temp_workspace.path().join("config.txt"),
        "setting1=remote\nsetting2=value2",
    )?;

    jin().args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Remote change"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // TEST: Pull with divergent history - should 3-way merge cleanly
    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ global: Merged (3-way)"));

    // VERIFY: Apply to workspace and check merged content
    jin().arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let merged_content = fs::read_to_string(remote_fixture.local_path.join("config.txt"))?;
    assert!(merged_content.contains("setting1=remote"));   // Remote change
    assert!(merged_content.contains("setting3=local"));    // Local change

    Ok(())
}

// ============================================================================
// Test 3: Conflicted 3-Way Merge (Overlapping Changes)
// ============================================================================
// Location: Lines 254-403
// Purpose: Verify .jinmerge files are created for conflicts

#[test]
fn test_pull_divergent_with_conflicts() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Similar to Test 2, but with overlapping changes
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // STEP 1: Create base commit with single-line file
    fs::write(temp_workspace.path().join("config.txt"), "version=1")?;
    jin().args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Base commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Push base commit
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Setup local repo with base
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // STEP 2: Make CONFLICTING local change
    fs::write(remote_fixture.local_path.join("config.txt"), "version=2")?;

    jin().args(["add", "config.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Local conflicting change"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // STEP 3: Make CONFLICTING remote change (same line, different value)
    fs::write(temp_workspace.path().join("config.txt"), "version=3")?;

    jin().args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Remote conflicting change"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // TEST: Pull with divergent history - should create .jinmerge file
    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("! global: Merged with 1 conflicts"))
        .stdout(predicates::str::contains("config.txt has conflicts (.jinmerge created)"));

    // VERIFY: .jinmerge file created with correct format
    let jinmerge_path = remote_fixture.local_path.join("config.txt.jinmerge");
    assert!(jinmerge_path.exists(), ".jinmerge file should be created for conflicts");

    let jinmerge_content = fs::read_to_string(&jinmerge_path)?;
    assert!(jinmerge_content.contains("# Jin merge conflict"));
    assert!(jinmerge_content.contains("<<<<<<<"));
    assert!(jinmerge_content.contains("======="));
    assert!(jinmerge_content.contains(">>>>>>>"));

    // VERIFY: Both versions present in conflict
    assert!(jinmerge_content.contains("version=2"));  // Local
    assert!(jinmerge_content.contains("version=3"));  // Remote

    Ok(())
}

// ============================================================================
// Test 4: Clean Merge with Different Files
// ============================================================================
// Location: Lines 410-557
// Purpose: Verify files added in both branches are merged

#[test]
fn test_pull_divergent_clean_merge_different_files() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create base commit with initial file
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    let temp_workspace = TestFixture::new()?;
    let temp_jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(temp_jin_dir))?;

    // Create initial file
    fs::write(temp_workspace.path().join("base.txt"), "base content")?;

    jin().args(["add", "base.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Base commit with initial file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // Push and setup local
    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["link", remote_fixture.remote_path.to_str().unwrap(), "--force"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // STEP 2: Add file in LOCAL
    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;

    jin().args(["add", "local.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Add local file"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // STEP 3: Add DIFFERENT file in REMOTE
    fs::write(temp_workspace.path().join("remote.txt"), "remote content")?;

    jin().args(["add", "remote.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().args(["commit", "-m", "Add remote file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    jin().arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", temp_jin_dir)
        .assert()
        .success();

    // TEST: Pull should merge both files
    jin().arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin().arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ global: Merged (3-way)"));

    // VERIFY: Apply and check both files exist
    jin().arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    assert!(remote_fixture.local_path.join("local.txt").exists());
    assert!(remote_fixture.local_path.join("remote.txt").exists());
    assert!(remote_fixture.local_path.join("base.txt").exists());

    let local_content = fs::read_to_string(remote_fixture.local_path.join("local.txt"))?;
    assert_eq!(local_content, "local content");

    let remote_content = fs::read_to_string(remote_fixture.local_path.join("remote.txt"))?;
    assert_eq!(remote_content, "remote content");

    Ok(())
}

// ============================================================================
// OPTIONAL: Additional Edge Case Tests (if gaps identified)
// ============================================================================

// Test: Multiple file conflicts
// Test: Mode layer merge (not just global)
// Test: Empty file scenarios
// Test: Whitespace-only conflicts
// Test: Merge commit structure validation
```

### Implementation Tasks

```yaml
Task 1: VALIDATE existing tests pass
  - RUN: cargo test --test pull_merge
  - VERIFY: All 4 existing tests pass
  - CHECK: No test failures or errors
  - DEPENDENCIES: None (validation task)

Task 2: REVIEW test coverage against requirements
  - CHECK: Fast-forward test exists (test_pull_fast_forward_still_works)
  - CHECK: Clean merge test exists (test_pull_divergent_clean_merge)
  - CHECK: Conflict test exists (test_pull_divergent_with_conflicts)
  - VERIFY: Each test covers the scenario correctly
  - DEPENDENCIES: Task 1

Task 3: IDENTIFY potential gaps (OPTIONAL)
  - ANALYZE: Are there edge cases not covered?
  - CONSIDER: Multiple file conflicts, mode layers, empty files
  - DECIDE: Based on project priorities, add tests or document as adequate
  - DEPENDENCIES: Task 2

Task 4: ADD additional tests if gaps identified (OPTIONAL)
  - IMPLEMENT: New test following existing pattern
  - USE: setup_jin_with_remote() fixture
  - FOLLOW: Same structure as existing tests
  - DEPENDENCIES: Task 3

Task 5: VERIFY all tests pass
  - RUN: cargo test --test pull_merge -- --nocapture
  - VERIFY: All tests pass with detailed output
  - CHECK: No regressions in existing functionality
  - DEPENDENCIES: All previous tasks
```

### Test Execution and Validation

```bash
# ============================================================================
# Level 1: Syntax Check
# ============================================================================
cargo check --tests

# ============================================================================
# Level 2: Run pull_merge tests
# ============================================================================
# Run all pull_merge tests
cargo test --test pull_merge

# Run with detailed output
cargo test --test pull_merge -- --nocapture

# Run specific test
cargo test --test pull_merge test_pull_fast_forward_still_works -- --nocapture
cargo test --test pull_merge test_pull_divergent_clean_merge -- --nocapture
cargo test --test pull_merge test_pull_divergent_with_conflicts -- --nocapture
cargo test --test pull_merge test_pull_divergent_clean_merge_different_files -- --nocapture

# ============================================================================
# Level 3: Run all integration tests (regression check)
# ============================================================================
cargo test --test sync_workflow
cargo test --test conflict_workflow

# ============================================================================
# Level 4: Full test suite
# ============================================================================
cargo test

# Expected: All tests pass
```

---

## Validation Loop

### Level 1: Syntax & Style

```bash
# Check test file compiles
cargo check --tests

# Format check
cargo fmt --check tests/pull_merge.rs

# Expected: Zero errors
```

### Level 2: Test Execution

```bash
# Run pull_merge tests
cargo test --test pull_merge

# Expected output:
# test test_pull_fast_forward_still_works ... ok
# test test_pull_divergent_clean_merge ... ok
# test test_pull_divergent_with_conflicts ... ok
# test test_pull_divergent_clean_merge_different_files ... ok
#
# test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Level 3: Integration Test Validation

```bash
# Run all integration tests to ensure no regressions
cargo test --test sync_workflow
cargo test --test conflict_workflow

# Expected: All tests pass
```

### Level 4: Manual Verification (Optional)

```bash
# Build and manually test merge scenarios
cargo build --release

# Create test scenario and verify behavior
# ... manual testing steps ...
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo test --test pull_merge` passes all 4 tests
- [ ] `cargo test --test sync_workflow` passes (no regression)
- [ ] `cargo test --test conflict_workflow` passes (no regression)
- [ ] `cargo test` full suite passes
- [ ] Code is formatted: `cargo fmt --check` returns clean
- [ ] No clippy warnings: `cargo clippy --tests`

### Feature Validation

- [ ] Fast-forward merge works correctly (test_pull_fast_forward_still_works)
- [ ] Clean 3-way merge produces correct merged content (test_pull_divergent_clean_merge)
- [ ] Conflicted merge creates .jinmerge files (test_pull_divergent_with_conflicts)
- [ ] Different files in both branches are merged (test_pull_divergent_clean_merge_different_files)
- [ ] Merge messages are correct and informative
- [ ] .jinmerge files follow correct format

### Code Quality Validation

- [ ] Tests follow existing patterns from tests/common/fixtures.rs
- [ ] Each test is isolated with own JIN_DIR
- [ ] Tests use setup_jin_with_remote() fixture
- [ ] Assertions are clear and specific
- [ ] Test names describe what they validate

### Documentation & Completeness

- [ ] Each test has clear comments explaining the scenario
- [ ] Test structure is consistent across all tests
- [ ] Edge cases are documented (even if not tested)
- [ ] Test coverage is adequate for requirements

---

## Anti-Patterns to Avoid

- ❌ **Don't** skip running tests before claiming task is complete
- ❌ **Don't** add tests without verifying they pass
- ❌ **Don't** ignore test failures - debug and fix them
- ❌ **Don't** create tests that depend on execution order
- ❌ **Don't** hardcode paths - use fixture.path() instead
- ❌ **Don't** forget to set JIN_DIR environment variable
- ❌ **Don't** use global ~/.jin in tests - always isolate
- ❌ **Don't** skip Git lock cleanup - fixtures handle this automatically
- ❌ **Don't** create tests that leave artifacts behind
- ❌ **Don't** add unnecessary complexity to test scenarios

---

## Confidence Score

**Rating: 9/10** for task completion success

**Justification:**
- Existing tests are comprehensive and well-written
- Test infrastructure is solid (fixtures, helpers, assertions)
- Clear patterns to follow for any additional tests
- External research confirms test coverage is adequate
- Validation commands are project-specific and verified

**Remaining Risks:**
- May identify edge cases that need additional tests
- Some layer types (mode/scope/project) only tested with global layer
- Multiple file conflicts not explicitly tested

---

## Research Artifacts Location

Research documentation stored at: `plan/P2M1T3/research/`

Key findings:
- Existing test coverage analysis (4 comprehensive tests identified)
- External best practices for git merge testing
- git2-rs official examples for reference
- Potential gaps and edge cases identified
- Test infrastructure patterns documented

External references:
- [git2-rs pull example](https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs)
- [Understanding Git Merge](https://medium.com/@iiiamigoes/understanding-git-merge-keep-your-project-history-clean-fd69db1e1dbe)
- [Merging with diff3](https://blog.jcoglan.com/2017/05/08/merging-with-diff3/)
