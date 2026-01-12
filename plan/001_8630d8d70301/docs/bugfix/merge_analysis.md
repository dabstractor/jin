# Merge Engine Architecture Analysis

## Executive Summary

The merge engine has a critical conflict detection gap: **deep_merge() never detects value changes as conflicts**, even when the same key is modified in different layers. This is by design according to RFC 7396, but creates silent data overwrites that could be dangerous for configuration management.

## Deep Merge Analysis (src/merge/deep.rs)

### Key Findings:

1. **RFC 7396 Compliance**: The deep merge implements JSON Merge Patch semantics where overlay values always win base values
   - Line 115: `(_, overlay) => Ok(overlay)` - Overlay completely replaces base for type conflicts
   - Line 326-335: Test shows `{"a": 1}` + `{"a": 2}` = `{"a": 2}` (no conflict detection)

2. **No Conflict Detection**: The function never returns `JinError::MergeConflict`
   - All merge operations succeed unconditionally
   - Only structural constraints (like array key matching) are enforced
   - Scalar value changes are silently resolved by overlay precedence

3. **Array Handling**:
   - Keyed arrays (with "id"/"name" fields) are merged by key
   - Non-keyed arrays are completely replaced by overlay
   - No conflict detection for modified keyed array items

### Why Deep Merge Doesn't Detect Conflicts:

The RFC 7396 specification intentionally allows deep merging without conflict detection - it's designed for overlaying configuration where higher precedence should win. However, this creates a blind spot in Jin's conflict detection system.

## Text Merge Analysis (src/merge/text.rs)

### Key Findings:

1. **Proper 3-Way Conflict Detection**: Uses `diffy::merge()` which properly detects overlapping changes
   - Returns `TextMergeResult::Conflict` with conflict markers
   - Supports both diff2 and diff3 (base included) formats
   - Line 142: `diffy::merge()` returns `Err(String)` for conflicts (NOT an error condition)

2. **Conflict Resolution Support**:
   - Can parse conflict markers into `ConflictRegion` structs
   - Provides conflict counting and region extraction
   - Supports custom conflict marker labels

3. **Is Text Merge Ever Called?**:
   - Only for files with unstructured extensions (no .json, .yaml, .toml, .ini)
   - Text files are wrapped as `MergeValue::String` and deep-merged
   - **Conflict**: Text merge is never actually invoked by the layer merge system

## Layer Merge Orchestration (src/merge/layer.rs)

### Key Findings:

1. **Format Detection Routing**:
   - Line 194: `format = detect_format(path)` determines merge strategy
   - Line 242: Text files become `MergeValue::String` and go through deep_merge
   - **Critical Issue**: Text files never use text_merge!

2. **Merge Flow**:
   - Line 200: `deep_merge(base, layer_value)?` - Always uses deep merge
   - No conflict checking during accumulation
   - Line 115: Only catches `JinError::MergeConflict` from individual operations

3. **Conflict Collection**:
   - Line 115-119: Only handles conflicts explicitly raised by merge operations
   - Since deep_merge never raises conflicts, none are collected
   - The `conflict_files` vector remains empty

## Apply Command Integration (src/commands/apply.rs)

### Key Findings:

1. **Merge Process**:
   - Line 136: `merge_layers(&config, &repo)` performs the merge
   - Line 139-149: Checks for conflicts after merge
   - Line 166: Only creates .jinmerge files if conflicts exist

2. **The Conflict Detection Gap**:
   - If no conflicts are detected by merge_layers(), apply succeeds
   - This means conflicting value changes in structured files are silently applied
   - No .jinmerge files are created for resolution

## Root Cause Analysis

### Why No Conflicts Are Detected:

1. **Deep Merge Design**: Intentionally follows RFC 7396 where overlay wins
2. **Wrong Merge Strategy**: Text files should use text_merge but use deep_merge instead
3. **No Conflict Hooks**: No mechanism to compare accumulated vs. new layer values
4. **Format Routing Bug**: detect_format() + parse_content() routes all files through MergeValue

### The Conflict Detection Gap:

```
Layer 1: {"key": "base"}
Layer 2: {"key": "modified"}
       ↓
deep_merge() → {"key": "modified"} (no conflict)
       ↓
apply_to_workspace() (silently overwrites)
```

## Critical Questions Answered

### Why doesn't deep_merge() detect value changes as conflicts?

By RFC 7396 design. Deep merge is for configuration overlay where higher precedence should always win. Conflict detection would violate this semantic.

### Is text_merge() ever actually called for non-structured files?

**No!** This is the routing bug. Even text files get wrapped as `MergeValue::String` and go through deep_merge. The text_merge function exists but is never invoked.

### Where should the conflict detection logic be injected?

1. **Short-term**: In `merge_file_across_layers()` before accumulating new layers
2. **Medium-term**: Add a conflict detection mode to `deep_merge_with_config()`
3. **Long-term**: Implement proper 3-way merging for structured files

### What would it take to add conflict detection mode?

1. **Track Original Values**: Store the base value before each layer merge
2. **Compare on Change**: When a new layer modifies a value, compare to original
3. **Conflict Strategy**: Either fail fast or accumulate conflicts for resolution
4. **API Changes**: New merge mode parameter or separate conflict-aware merge function

## Proposed Solutions

### Option 1: Conflict Detection Mode (Recommended)

```rust
pub enum MergeMode {
    Deep, // Current behavior - overlay wins
    Conflicted, // Detect conflicts and collect them
}

pub fn deep_merge_with_mode(
    base: MergeValue,
    overlay: MergeValue,
    mode: MergeMode,
) -> Result<MergeValue> {
    // Implementation would track original values and detect conflicts
}
```

### Option 2: Fix Text Merge Routing

```rust
// In merge_file_across_layers()
if format == FileFormat::Text {
    // Use actual text merging for conflict detection
    let text_content = accumulated.unwrap_or(MergeValue::String("".to_string())).as_str().unwrap_or("");
    match text_merge(text_content, content_str, content_str) {
        TextMergeResult::Clean(_) => /* proceed */,
        TextMergeResult::Conflict { .. } => return Err(JinError::MergeConflict),
    }
}
```

### Option 3: Hybrid Approach

- Keep deep merge for non-conflicting scenarios
- Add optional conflict detection mode
- Support conflict resolution for structured files (.jinmerge format)
- Preserve RFC 7396 behavior as default

## Impact Assessment

**Severity**: HIGH - Silent data overwrites without user awareness
**Scope**: All structured configuration files in multi-layer environments
**Risk**: Configuration conflicts could be silently resolved incorrectly

## Next Steps

1. Implement conflict detection mode for deep_merge
2. Add comprehensive tests for conflict scenarios
3. Update apply command to handle conflict detection mode
4. Consider backward compatibility implications
