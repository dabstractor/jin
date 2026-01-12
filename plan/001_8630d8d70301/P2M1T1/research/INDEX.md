# Research Index: P2.M1.T1 - Detect Fast-Forward vs Divergent Histories

## Quick Reference

**Task**: Add logic to detect when a pull operation is fast-forward vs divergent.

**Key Files to Modify**:
- `src/git/merge.rs` (NEW) - MergeType enum and detect_merge_type()
- `src/git/mod.rs` - Add module exports
- `src/commands/pull.rs` - Integrate merge detection

**Key External Documentation**:
- [git2-rs Repository](https://docs.rs/git2/latest/git2/struct.Repository.html) - graph_ahead_behind, merge_base, descendant_of
- [git2-rs pull example](https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs)

## Codebase Analysis Findings

### Pull Command Implementation (`src/commands/pull.rs`)

**Current State**:
- Line 28: Implicit fetch via `super::fetch::execute()`
- Line 34: `detect_updates()` finds layers with updates
- Line 44: `LayerTransaction` for atomic updates
- Line 50: TODO comment "Implement proper 3-way merge"
- Lines 51-57: Simple fast-forward update logic

**Key Function**:
```rust
fn detect_updates(jin_repo: &JinRepo) -> Result<HashMap<String, LayerUpdateInfo>>
```
Returns layers that need updating based on OID comparison.

### Ref Comparison Pattern (`src/git/refs.rs`)

**Existing Pattern to Follow**:
- Lines 16-28: `RefComparison` enum (Ahead, Behind, Diverged, Equal)
- Lines 164-177: `compare_refs()` using `graph_ahead_behind()`

**Algorithm**:
```rust
let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;
match (ahead, behind) {
    (0, 0) => Ok(RefComparison::Equal),
    (_, 0) => Ok(RefComparison::Ahead),
    (0, _) => Ok(RefComparison::Behind),
    (_, _) => Ok(RefComparison::Diverged),
}
```

### Text Merge Engine (`src/merge/text.rs`)

**For P2.M1.T2**:
- `text_merge(base, ours, theirs)` - 3-way merge function
- Returns `TextMergeResult::Clean` or `Conflict`
- Will be used in next task for divergent merge

## Test Patterns

### Integration Tests (`tests/sync_workflow.rs`)

**Key Fixtures**:
- `setup_jin_with_remote()` - Creates local + bare remote repos
- `RemoteFixture` struct with `local_path` and `remote_path`
- `unique_test_id()` for test isolation

**Test Pattern**:
```rust
fn test_pull_merges_changes() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("pull_test_{}", unique_test_id());

    // 1. Setup remote commit via temp workspace
    // 2. Link and pull in main local repo
    // 3. Verify merged content
}
```

### Creating Divergent Histories

For testing divergent merge detection:

```rust
// 1. Create base commit and push
// 2. Create divergent commit in remote (direct git2 manipulation)
let remote_repo = git2::Repository::open(&remote_fixture.remote_path)?;
let sig = remote_repo.signature()?;
let mut tree_builder = remote_repo.treebuilder(None)?;
// ... create commit with different content ...
remote_repo.commit(Some(&ref_path), &sig, &sig, "Remote divergent", &tree, &[&parent_commit])?;

// 3. Create divergent commit in local
// 4. Pull should detect divergent history
```

## Git Merge Detection Algorithms

### Algorithm 1: graph_ahead_behind (Recommended)

Uses git2's `graph_ahead_behind()` method:

```rust
let (ahead, behind) = repo.inner().graph_ahead_behind(local_oid, remote_oid)?;
match (ahead, behind) {
    (0, 0) => MergeType::UpToDate,
    (_, 0) => MergeType::FastForward,
    (0, _) => MergeType::LocalAhead,
    (_, _) => MergeType::Divergent,
}
```

**Pros**: Single API call, well-tested in codebase
**Cons**: None

### Algorithm 2: merge_base + descendant_of

Alternative using merge base and descendant checks:

```rust
// Check equality first (important!)
if local_oid == remote_oid {
    return Ok(MergeType::UpToDate);
}

// Check fast-forward
if repo.inner().descendant_of(remote_oid, local_oid)? {
    return Ok(MergeType::FastForward);
}

// Check local ahead
if repo.inner().descendant_of(local_oid, remote_oid)? {
    return Ok(MergeType::LocalAhead);
}

// Find merge base to confirm divergence
match repo.inner().merge_base(local_oid, remote_oid) {
    Ok(_) => Ok(MergeType::Divergent),
    Err(_) => Ok(MergeType::Divergent),  // Unrelated = divergent
}
```

**Pros**: More explicit about ancestry
**Cons**: Multiple API calls, need to handle OID equality separately

**Critical Gotcha**: `descendant_of()` returns `false` when a commit is compared with itself. Always check OID equality first!

## External References

### Documentation URLs

1. [git2-rs Repository::merge_base](https://docs.rs/git2/latest/git2/struct.Repository.html#method.merge_base)
2. [git2-rs Repository::descendant_of](https://docs.rs/git2/latest/git2/struct.Repository.html#method.descendant_of)
3. [git2-rs Repository::graph_ahead_behind](https://docs.rs/git2/latest/git2/struct.Repository.html#method.graph_ahead_behind)
4. [git2-rs pull example](https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs)
5. [Git merge-base documentation](https://git-scm.com/docs/git-merge-base)
6. [Git fast-forward merge](https://git-scm.com/docs/git-merge#_fast_forward_merge)

### Stack Overflow References

1. [Determine if merge will be fast-forward](https://stackoverflow.com/questions/37648908/determine-if-a-merge-will-resolve-via-fast-forward)

## Implementation Checklist

- [ ] Create `src/git/merge.rs` with `MergeType` enum
- [ ] Implement `detect_merge_type()` function
- [ ] Add unit tests for all merge scenarios
- [ ] Modify `src/git/mod.rs` to export new module
- [ ] Modify `src/commands/pull.rs` `LayerUpdateInfo` struct
- [ ] Modify `detect_updates()` to call merge detection
- [ ] Modify pull execution loop to handle merge types
- [ ] Add integration tests for fast-forward detection
- [ ] Add integration tests for divergent detection
- [ ] Run full test suite to verify no regressions

## Dependencies on Other Tasks

**Depends On**:
- P1.M1.T1-T5: Conflict resolution workflow (for JinMerge patterns)
- P1.M2.T1-T3: Fetch and ref comparison (for `compare_refs` pattern)
- P1.M3.T1-T5: Detached workspace state (for error handling patterns)

**Enables**:
- P2.M1.T2: Implement 3-Way Merge (uses `MergeType::Divergent` detection)
- P2.M1.T3: Integration Tests for 3-Way Merge (tests both merge types)

## Scope Boundaries

**In Scope**:
- Merge type detection (fast-forward vs divergent)
- Integration into pull command
- Unit and integration tests
- Documentation

**Out of Scope**:
- 3-way merge implementation (P2.M1.T2)
- Conflict resolution (uses P1.M1 patterns)
- UI/UX improvements for merge display

## Known Gotchas

1. **descendant_of() behavior**: Unlike Git CLI, git2-rs does NOT consider a commit a descendant of itself. Always check OID equality first.

2. **New layers**: When `local_oid` is `None` (layer doesn't exist locally), treat as `FastForward`.

3. **Unrelated histories**: `merge_base()` can fail for unrelated repositories. Handle gracefully by treating as `Divergent`.

4. **graph_ahead_behind parameter order**: First parameter is the potential descendant, second is the potential ancestor. Order matters!

5. **Existing behavior**: Must preserve existing fast-forward behavior. No breaking changes to pull command.
