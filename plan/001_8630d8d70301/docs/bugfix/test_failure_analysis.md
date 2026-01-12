# Test Failure Analysis: Conflict Detection in `jin apply`

## Quick Summary

**Problem:** All 5 conflict tests fail because conflicts are NEVER detected.

**Root Cause:** Missing pre-merge collision detection. The code tries to catch `JinError::MergeConflict` from `deep_merge()`, but deep merge NEVER returns this error - it always succeeds by taking the higher layer's value.

**Fix:** Add collision detection BEFORE merge:
1. Check if file exists in >1 layer
2. Compare content across layers
3. If different → add to `conflict_files` (not `merged_files`)
4. All existing conflict handling will work

**Impact:** This is a complete feature failure - PRD section 11.3 requirements are not met at all.

**Files to modify:** `src/merge/layer.rs` (add detection in `merge_layers()`)

## Executive Summary

All 5 conflict tests in `tests/cli_apply_conflict.rs` are failing because **conflict detection is completely broken**. The merge system never reports conflicts, so files are always applied successfully instead of creating `.jinmerge` files and pausing the operation.

## Critical Finding

**The root cause**: `merge_file_across_layers()` in `src/merge/layer.rs` catches `JinError::MergeConflict` (line 115), but the `deep_merge()` function **NEVER returns this error**. Deep merge always succeeds by taking the overlay value when there are differences.

## Test Expectations vs Reality

### Test 1: `test_apply_with_conflicts_creates_jinmerge_files`

**Expected behavior:**
- When `config.json` exists in both global and mode layers with different content
- `jin apply` should detect a conflict
- Create `config.json.jinmerge` file with conflict markers
- Output: "Operation paused" and "jin resolve"

**Actual behavior:**
- Output: "Applied 1 files to workspace"
- No `.jinmerge` file created
- No pause state
- The mode layer's config.json completely overwrites the global layer's version

**Gap analysis:** The test assumes that when the same file exists in multiple layers with different content, it's a conflict. But the current implementation just does a deep merge where the higher layer wins.

### Test 2: `test_apply_dry_run_with_conflicts_shows_preview`

**Expected behavior:**
- `jin apply --dry-run` should show "Merge conflicts detected"
- Should mention `--force` flag
- No files should be written

**Actual behavior:**
- Shows "Would apply 1 files:" (or similar)
- No conflict warning
- No `--force` mentioned

**Gap analysis:** Dry-run mode checks `merged.conflict_files` but it's always empty because conflicts are never detected.

### Test 3: `test_apply_with_conflicts_applies_non_conflicting_files`

**Expected behavior:**
- With 2 files: `safe.json` (only in global) and `conflict.json` (in both global and mode)
- `safe.json` should be applied to workspace
- `conflict.json` should NOT be applied; only `conflict.json.jinmerge` created

**Actual behavior:**
- Both files are applied successfully
- No `.jinmerge` files created

**Gap analysis:** The test expects partial application on conflicts, but since conflicts are never detected, all files are applied.

### Test 4: `test_apply_with_multiple_conflicts`

**Expected behavior:**
- 3 files (`a.json`, `b.json`, `c.json`) each exist in both global and mode layers
- Should create 3 `.jinmerge` files
- Should show "3 files" in output
- Paused state should list all 3 conflicts

**Actual behavior:**
- All 3 files are applied successfully
- No `.jinmerge` files created
- Output shows "Applied 3 files to workspace"

**Gap analysis:** Multiple conflicts should be detected, but none are because the merge always succeeds.

### Test 5: `test_apply_with_conflicts_creates_paused_state`

**Expected behavior:**
- Should create `.jin/.paused_apply.yaml` file
- File should contain `timestamp`, `conflict_files`, and `applied_files`

**Actual behavior:**
- No paused state file created
- All files applied successfully

**Gap analysis:** Paused state is only created when `has_conflicts` is true, but it's always false.

## Root Cause Analysis

### The Merge Pipeline

1. **Layer collection** (`collect_all_file_paths`): ✅ Works correctly
   - Collects all unique file paths across all layers
   - Both global and mode layer files are found

2. **File merging** (`merge_file_across_layers`): ❌ **NEVER FAILS**
   ```rust
   match merge_file_across_layers(&path, &config.layers, config, repo) {
       Ok(merged) => {
           result.merged_files.insert(path, merged);  // Always takes this branch
       }
       Err(JinError::MergeConflict { .. }) => {
           result.conflict_files.push(path);  // NEVER REACHED
       }
       Err(e) => return Err(e),
   }
   ```

3. **Deep merge** (`deep_merge` in `src/merge/deep.rs`): ❌ **ALWAYS SUCCEEDS**
   - Objects: Merges recursively, overlay wins on conflicts
   - Arrays: Either merges by key OR replaces entirely
   - **Never returns an error for conflicting values**
   - Just takes the higher-precedence value

4. **Apply execution** (`execute` in `src/commands/apply.rs`):
   - Checks `if !merged.conflict_files.is_empty()` (line 139)
   - This is **ALWAYS false** because step 2 never adds conflicts
   - Skips all conflict handling code

### Why Deep Merge Never Returns Conflicts

Looking at `src/merge/deep.rs`:

```rust
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    // ...
    match (base, overlay) {
        // Different types or scalars: overlay wins
        (_, overlay) => Ok(overlay),  // ALWAYS succeeds
    }
}
```

The deep merge is designed for **deterministic layer merging**, not conflict detection:
- When values conflict, the higher layer wins (RFC 7396 semantics)
- This is correct for the merge operation itself
- But it means conflicts are never reported

## The PRD Expectation

Based on the tests, the PRD expects:

1. **Semantic conflict detection**: When the same file exists in multiple layers with different content, it should be treated as a conflict requiring manual resolution
2. **Pause workflow**: Instead of auto-merging, create `.jinmerge` files and pause
3. **Partial application**: Non-conflicting files should still be applied
4. **User intervention**: User must run `jin resolve` to handle each conflict

This is fundamentally different from the current implementation which:
- Auto-merges everything using deep merge
- Never pauses
- Never creates `.jinmerge` files during `apply`

## Implementation Gap

The current code has conflict handling infrastructure:
- `PausedApplyState` struct ✅
- `handle_conflicts()` function ✅
- `.jinmerge` file creation ✅
- Paused state persistence ✅

**But none of it is ever triggered** because:
```rust
let has_conflicts = !merged.conflict_files.is_empty();  // ALWAYS FALSE
```

## PRD Confirmation

**CRITICAL**: The PRD (section 11.3) explicitly states:

> When conflicts occur during merge:
> 1. Jin pauses the merge operation
> 2. Creates `.jinmerge` files showing conflicts
> 3. Displays Git-style conflict markers with layer information
> 4. User resolves conflicts manually
> 5. User runs `jin add <resolved-files>` and `jin commit` to complete merge

The PRD example shows:
```
Conflict in file: .claude/config.json
Layer 1: mode/claude/scope/language:javascript/
Layer 2: mode/claude/project/ui-dashboard/

<<<<<<< mode/claude/scope/language:javascript/
{ "mcpServers": ["server-a"] }
=======
{ "mcpServers": ["server-b"] }
>>>>>>> mode/claude/project/ui-dashboard/
```

**This is NOT about unmergeable content** - it's about the same file existing in multiple layers with different content. The PRD expects Jin to pause and require manual resolution.

## What the Tests Are Actually Testing

The tests are **NOT** testing:
- Git-style 3-way merge conflicts (which `text_merge` handles)
- Unmergeable changes in structured files
- Parse errors or invalid content

The tests ARE checking:
- **Multi-layer collision detection**: Same file path in multiple layers with different content = conflict
- **Manual resolution workflow**: Force user to choose between layers
- **Apply pause**: Don't proceed until conflicts resolved
- **PRD compliance**: Implementing section 11.3 requirements

This is the **PRD-mandated behavior**, not a design choice. The tests are verifying that Jin implements the conflict resolution workflow as specified.

## Comparison with Passing Tests

Looking at `test_apply_no_conflicts_works_normally`:
- Only adds `config.json` to global layer
- No mode layer activated
- Single layer = no possible conflicts
- **This test passes because there's only one layer**

All failing tests have:
- Mode layer activated
- Same file in multiple layers
- **These fail because the system doesn't detect multi-layer collisions as conflicts**

## Detailed Fix Requirements

### 1. Conflict Detection Logic

**Current behavior:**
```rust
// In merge_layers() - src/merge/layer.rs:110-120
for path in all_paths {
    match merge_file_across_layers(&path, &config.layers, config, repo) {
        Ok(merged) => {
            result.merged_files.insert(path, merged);  // Always succeeds
        }
        Err(JinError::MergeConflict { .. }) => {
            result.conflict_files.push(path);  // NEVER reached
        }
        Err(e) => return Err(e),
    }
}
```

**Required behavior:**
```rust
// BEFORE attempting merge, detect multi-layer collisions
for path in all_paths {
    let layers_with_file = find_layers_containing_file(&path, &config.layers, config, repo)?;

    if layers_with_file.len() > 1 {
        // File exists in multiple layers - check content
        if has_different_content_across_layers(&path, &layers_with_file, repo)? {
            result.conflict_files.push(path);
            continue;  // Skip merge
        }
    }

    // Single layer or same content - safe to merge
    match merge_file_across_layers(&path, &config.layers, config, repo) {
        Ok(merged) => result.merged_files.insert(path, merged),
        Err(e) => return Err(e),
    }
}
```

### 2. Required Helper Functions

#### `find_layers_containing_file()`
- Input: file path, layers list, config, repo
- Output: Vec of layer references that contain this file
- Logic: Iterate layers, check if file exists in each layer's tree

#### `has_different_content_across_layers()`
- Input: file path, layer references, repo
- Output: bool (true if content differs)
- Logic: Compare file content across all layers
- For structured files: compare parsed values (not raw strings)
- For text files: compare raw content

### 3. Conflict Handling Flow

**When conflicts are detected:**

1. **Collect conflicting layer content** (already implemented):
   - `get_conflicting_layer_contents()` in apply.rs:279
   - Gets content from the two conflicting layers
   - Creates layer labels for conflict markers

2. **Generate .jinmerge files** (already implemented):
   - `JinMergeConflict::from_text_merge()` in jinmerge.rs
   - Creates Git-style conflict markers
   - Writes to `<file>.jinmerge`

3. **Create paused state** (already implemented):
   - `PausedApplyState::save()` in apply.rs:46
   - Writes `.jin/.paused_apply.yaml`
   - Stores conflict and applied file lists

4. **Display user message** (already implemented):
   - Lines 169-184 in apply.rs
   - Shows "Operation paused"
   - Instructs to run `jin resolve`

**All of this works perfectly - it's just never triggered!**

### 4. File Format Considerations

**Structured files (JSON/YAML/TOML/INI):**
- Parse content from each layer
- Compare MergeValue objects
- Different values = conflict (even if mergeable)
- Example: `{"port": 8080}` vs `{"port": 9090}`

**Text files:**
- Compare raw content strings
- Any difference = conflict
- Use `text_merge()` to generate conflict markers

**Why this matters:**
- PRD shows example with JSON arrays: `[\"server-a\"]` vs `[\"server-b\"]`
- These ARE mergeable by deep merge (higher layer wins)
- But PRD says "pause and require manual resolution"
- Conclusion: PRD wants conservative behavior, not auto-merge

### 5. Edge Cases to Handle

#### Same content in multiple layers
- Should NOT be a conflict
- Merge normally (result is same as either layer)
- Optimization: skip merge, use any layer's content

#### File exists in 3+ layers
- PRD example shows 2 layers
- Tests use 2 layers (global + mode)
- Should handle N layers: compare all pairs
- If any differ, mark as conflict

#### File deleted in higher layer
- Deep merge uses `null` to delete keys
- But file deletion vs modification is different
- Need to clarify: is deletion a conflict?
- For now: treat as conflict if content differs

### 6. Implementation Strategy

**Phase 1: Core detection**
1. Add `find_layers_containing_file()` function
2. Add `has_different_content_across_layers()` function
3. Modify `merge_layers()` to detect collisions before merge
4. Test with existing test suite

**Phase 2: Testing**
1. Run all 5 failing tests - should now pass
2. Add test for 3+ layers with same file
3. Add test for same content in multiple layers
4. Add test for text file conflicts

**Phase 3: Refinement**
1. Performance optimization (caching, early exit)
2. Better error messages
3. Edge case handling
4. Documentation updates

## Code Locations

### Failing Code Path
- `src/merge/layer.rs:110-120`: `merge_layers()` never adds to `conflict_files`
- `src/merge/layer.rs:164-215`: `merge_file_across_layers()` always returns `Ok`
- `src/merge/deep.rs:74-117`: `deep_merge_with_config()` never returns conflict errors

### Conflict Handling (Never Reached)
- `src/commands/apply.rs:139-186`: Conflict detection and handling
- `src/commands/apply.rs:218-268`: `handle_conflicts()` function
- `src/commands/apply.rs:269-333`: `get_conflicting_layer_contents()` function

### Tests
- `tests/cli_apply_conflict.rs`: All 5 failing tests
- Test expectations are clear and consistent
- Tests match the PRD requirements

## Visual Flow Comparison

### Current Flow (BROKEN)
```
jin apply
  ↓
merge_layers()
  ↓
For each file:
  ├─ merge_file_across_layers() → ALWAYS succeeds
  ├─ deep_merge() → Higher layer wins, no error
  └─ Add to merged_files
  ↓
conflict_files is ALWAYS empty
  ↓
Skip all conflict handling
  ↓
Apply all files to workspace
  ↓
Output: "Applied N files to workspace"
```

### Required Flow (PRD COMPLIANT)
```
jin apply
  ↓
merge_layers()
  ↓
For each file:
  ├─ Check: Does file exist in >1 layer?
  │   ├─ NO → merge_file_across_layers() → Add to merged_files
  │   └─ YES → Check: Content differs across layers?
  │       ├─ NO → Same content, merge normally
  │       └─ YES → Add to conflict_files
  ↓
conflict_files has entries
  ↓
handle_conflicts()
  ├─ Create .jinmerge files
  ├─ Save .jin/.paused_apply.yaml
  └─ Display "Operation paused" message
  ↓
Apply ONLY non-conflicting files
  ↓
Output: "Operation paused. Resolve conflicts with: jin resolve"
```

## Summary

**The bug is NOT in the merge logic** - deep merge works correctly for its purpose.

**The bug is the missing conflict detection layer** that should run BEFORE merge:
- Detect when same file exists in multiple layers
- Check if content differs across layers
- Mark as conflict instead of auto-merging
- Trigger the pause workflow

**The fix is straightforward:**
1. Add pre-merge collision detection in `merge_layers()`
2. Compare file content across layers
3. Add to `conflict_files` instead of `merged_files` when different
4. All existing conflict handling code will work correctly

**All the infrastructure is there and working** - it just needs to be triggered by proper conflict detection.

## Test-by-Test Breakdown

### Test 1: `test_apply_with_conflicts_creates_jinmerge_files`

**Setup:**
```bash
# Add config.json to global layer
echo '{"port": 8080, "debug": true, "version": "1.0"}' > config.json
jin add config.json --global
jin commit -m "Add config to global"

# Modify and add to mode layer
echo '{"port": 9090, "debug": false, "production": true}' > config.json
jin add config.json --mode
jin commit -m "Add config to mode"

# Delete from workspace
rm config.json

# Run apply
jin apply
```

**What SHOULD happen (PRD):**
1. Detect config.json exists in both global and mode layers
2. Compare content: `{"port": 8080, ...}` ≠ `{"port": 9090, ...}`
3. Mark as conflict
4. Create `config.json.jinmerge`:
   ```
   <<<<<<< global
   {"port": 8080, "debug": true, "version": "1.0"}
   =======
   {"port": 9090, "debug": false, "production": true}
   >>>>>>> mode
   ```
5. Create `.jin/.paused_apply.yaml`
6. Output: "Operation paused. Resolve conflicts with: jin resolve"

**What ACTUALLY happens:**
1. merge_layers() processes config.json
2. Calls merge_file_across_layers()
3. Deep merges both layers (higher wins)
4. Result: `{"port": 9090, "debug": false, "production": true, "version": "1.0"}`
5. Adds to merged_files
6. Applies to workspace
7. Output: "Applied 1 files to workspace"

**Why:** No collision detection before merge → conflict_files is empty → skips pause logic

### Test 2: `test_apply_dry_run_with_conflicts_shows_preview`

**Setup:** Same as Test 1, but runs `jin apply --dry-run`

**Expected:**
- "Merge conflicts detected in 1 files:"
- "Use --force to apply non-conflicting files, or resolve conflicts first."

**Actual:**
- "Would apply 1 files:"
- No conflict warning

**Why:** Dry-run checks `merged.conflict_files` but it's empty (no detection)

### Test 3: `test_apply_with_conflicts_applies_non_conflicting_files`

**Setup:**
```bash
# safe.json - only in global layer
echo '{"safe": true}' > safe.json
jin add safe.json --global

# conflict.json - in both layers
echo '{"value": 1}' > conflict.json
jin add conflict.json --global

echo '{"value": 2}' > conflict.json
jin add conflict.json --mode

# Delete both from workspace
rm safe.json conflict.json

# Run apply
jin apply
```

**Expected:**
- `safe.json` applied (exists in workspace)
- `conflict.json` NOT applied (only `conflict.json.jinmerge` exists)
- Paused state created

**Actual:**
- Both files applied
- No .jinmerge files
- No paused state

**Why:** safe.json has 1 layer → merged; conflict.json has 2 layers → merged (no detection)

### Test 4: `test_apply_with_multiple_conflicts`

**Setup:**
- 3 files: a.json, b.json, c.json
- Each exists in both global and mode layers
- Different content in each layer

**Expected:**
- 3 .jinmerge files created
- Output: "3 files" (conflicts)
- Paused state lists all 3

**Actual:**
- All 3 files applied
- Output: "Applied 3 files to workspace"

**Why:** Multi-layer collision detection missing for all files

### Test 5: `test_apply_with_conflicts_creates_paused_state`

**Setup:** Similar to Test 1, checks for paused state file

**Expected:**
- `.jin/.paused_apply.yaml` created with:
  ```yaml
  timestamp: 2026-01-10T...
  layer_config:
    layers: [GlobalBase, ModeBase]
    mode: "test_mode_..."
  conflict_files: [settings.yaml]
  applied_files: []
  conflict_count: 1
  ```

**Actual:**
- No paused state file created

**Why:** Paused state only created when `has_conflicts` is true, but it's always false

## Key Insight

**All 5 tests fail for the SAME reason:**
```rust
let has_conflicts = !merged.conflict_files.is_empty();  // Line 139 in apply.rs
```

This is ALWAYS `false` because:
```rust
// Line 115 in layer.rs - NEVER adds to conflict_files
Err(JinError::MergeConflict { .. }) => {
    result.conflict_files.push(path);
}
```

And `JinError::MergeConflict` is NEVER returned because:
```rust
// deep_merge always returns Ok(overlay) for conflicts
(_, overlay) => Ok(overlay),  // Line 115 in deep.rs
```

**The fix is simple:** Add collision detection BEFORE calling merge_file_across_layers()
