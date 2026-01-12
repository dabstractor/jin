# Deep Merge Logic Research

## Implementation Location
**File**: `/home/dustin/projects/jin/src/merge/deep.rs`

## RFC 7396 JSON Merge Patch Semantics

### Core Merge Rules

1. **Null values delete keys** (RFC 7396 compliant)
2. **Objects merge recursively**
3. **Arrays with keyed items** (by "id" or "name") merge by key
4. **Other arrays are replaced** by the higher-precedence value
5. **Type conflicts**: overlay wins completely

### Key Functions

```rust
// Line 55-57: Main entry point
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    deep_merge_with_config(base, overlay, &MergeConfig::new())
}

// Line 74-121: Core merge logic with layer precedence
pub fn deep_merge_with_config(
    base: MergeValue,      // Lower precedence
    overlay: MergeValue,   // Higher precedence
    config: &MergeConfig,
) -> Result<MergeValue> {
    match (base, overlay) {
        // Null in overlay = delete the key (RFC 7396)
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Both objects: recursive merge
        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            // ... recursive merge logic
        }

        // Both arrays: attempt keyed merge, otherwise replace
        (MergeValue::Array(base_arr), MergeValue::Array(overlay_arr)) => {
            let result = merge_arrays_with_config(base_arr, overlay_arr, config)?;
            Ok(MergeValue::Array(result))
        }

        // Different types or scalars: overlay wins
        (_, overlay) => Ok(overlay),
    }
}
```

## Layer Precedence Verification

From commit 55c6ac3 - layer precedence is correctly implemented:

```rust
// Line 117-120: Verified comment
// VERIFIED: Layer precedence is correctly implemented via the accumulative merge pattern
// in merge_file_across_layers() (src/merge/layer.rs:369-376) combined with this catch-all
// match arm. Layers are passed in lowest-to-highest precedence order, and the overlay
// value (higher layer) wins when types differ or for scalar conflicts, per RFC 7396.
```

## Recent Changes (S1/S2)

### Commit 96f8874: Remove pre-merge conflict check for structured files

**Before**: All files with different content across layers created conflicts
**After**: Only text files get conflict detection; structured files always deep merge

```rust
// NEW - Only text files checked for conflicts
if layers_with_file.len() > 1 {
    let format = detect_format(path);

    if format == FileFormat::Text {
        // Text files: check for line-based conflicts
        let has_conflict = has_different_text_content(path, &layers_with_file, config, repo)?;
        if has_conflict {
            result.conflict_files.push(path.clone());
            continue;
        }
    }

    // Structured files: only optimization check, not conflict detection
    let same_content = !has_different_content_across_layers(path, &layers_with_file, config, repo)?;
    // ... proceed to deep merge
}
```

## Test Cases in src/merge/deep.rs

The file contains comprehensive unit tests (lines 206-739):
- Null deletion tests
- Object merge tests
- Keyed array merge tests
- Unkeyed array replacement tests
- Scalar override tests
- Complex nested merge scenarios
