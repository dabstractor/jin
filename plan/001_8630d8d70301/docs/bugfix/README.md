# Conflict Detection Bug: Executive Summary

## The Problem

Jin's conflict detection feature is completely non-functional. All 5 integration tests in `tests/cli_apply_conflict.rs` fail because the system never detects conflicts when the same file exists in multiple layers with different content.

## Root Cause

The code expects `deep_merge()` to return `JinError::MergeConflict` when layers collide, but this never happens:

```rust
// src/merge/layer.rs:115 - NEVER EXECUTED
Err(JinError::MergeConflict { .. }) => {
    result.conflict_files.push(path);
}
```

```rust
// src/merge/deep.rs:115 - ALWAYS RETURNS SUCCESS
(_, overlay) => Ok(overlay),  // Higher layer wins, no error
```

## What Should Happen (PRD Section 11.3)

When a file exists in multiple layers with different content:
1. Jin detects the collision
2. Creates `.jinmerge` files with conflict markers
3. Pauses the apply operation
4. Instructs user to run `jin resolve`

## What Actually Happens

1. Files are automatically deep-merged
2. Higher layer's content wins
3. No `.jinmerge` files created
4. No pause state
5. Output: "Applied N files to workspace"

## The Fix

Add collision detection in `src/merge/layer.rs` BEFORE calling `merge_file_across_layers()`:

```rust
for path in all_paths {
    // NEW: Check for multi-layer collisions
    let layers_with_file = find_layers_containing_file(&path, ...)?;

    if layers_with_file.len() > 1 {
        if has_different_content_across_layers(&path, &layers_with_file, ...)? {
            result.conflict_files.push(path);
            continue;  // Skip merge
        }
    }

    // Existing merge logic
    match merge_file_across_layers(&path, ...) {
        Ok(merged) => result.merged_files.insert(path, merged),
        Err(e) => return Err(e),
    }
}
```

## Impact

- **Feature complete failure**: PRD requirements not met
- **All conflict handling code exists and works** but is never triggered
- **Infrastructure is sound**: `.jinmerge` files, paused state, resolution workflow all implemented
- **Fix is surgical**: Only need to add detection layer, no refactoring

## Files Requiring Changes

1. **`src/merge/layer.rs`** - Add collision detection in `merge_layers()`
2. **New helper functions** - `find_layers_containing_file()`, `has_different_content_across_layers()`

## Testing

All 5 existing tests will pass once collision detection is added:
- `test_apply_with_conflicts_creates_jinmerge_files`
- `test_apply_dry_run_with_conflicts_shows_preview`
- `test_apply_with_conflicts_applies_non_conflicting_files`
- `test_apply_with_multiple_conflicts`
- `test_apply_with_conflicts_creates_paused_state`

## Documentation

See `test_failure_analysis.md` for complete technical analysis including:
- Detailed test-by-test breakdown
- Visual flow diagrams
- Code location references
- Implementation strategy
- Edge case handling

## Conclusion

This is a **missing feature** not a bug in existing code. The deep merge logic works correctly for its purpose (deterministic layer merging). What's missing is the conservative conflict detection layer that should run BEFORE merge to implement PRD requirements.

The fix is straightforward and all supporting infrastructure is already in place and working correctly.
