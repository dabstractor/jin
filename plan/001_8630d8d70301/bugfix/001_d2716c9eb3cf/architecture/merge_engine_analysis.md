# Merge Engine Architecture Analysis

## Overview

The Jin merge engine implements a sophisticated two-phase approach to file merging: conflict detection followed by merge execution. The engine supports both text files (3-way line-based merges) and structured files (JSON/YAML/TOML/INI with semantic deep merging).

## Current Architecture

### 1. Conflict Detection Flow

**File**: `src/merge/layer.rs`

The merge process follows this sequence:

```rust
pub fn merge_layers(...) -> Result<MergeResult> {
    // 1. Find all layers containing the file
    let layers_with_file = find_layers_containing_file(path, &config.layers, config, repo)?;

    // 2. If file exists in multiple layers, check for conflicts FIRST
    if layers_with_file.len() > 1 {
        let has_conflict = has_different_content_across_layers(path, &layers_with_file, config, repo)?;

        if has_conflict {
            // Conflict detected - skip merge and add to conflicts list
            result.conflict_files.push(path.clone());
            continue;
        }

        // No conflict - optimize for same content
        // Use first layer directly and add all layers to source_layers
    }

    // 3. Perform the actual merge
    merge_file_across_layers(path, &layers_with_file, config, repo)?;
}
```

### 2. Content Comparison Logic

**Function**: `has_different_content_across_layers()` (lines 604-624)

The current implementation has a **critical bug**: it checks if content is "different" rather than checking if a merge is "possible."

```rust
pub fn has_different_content_across_layers(
    file_path: &std::path::Path,
    layers_with_file: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<bool> {
    // For structured files, parse and compare MergeValue
    has_different_structured_content(file_path, layers_with_file, config, repo, format)
}
```

**Problem**: For structured files like JSON, `has_different_structured_content()` returns `true` when layers have different content, even though those differences should be resolved via deep merge (not conflicts).

### 3. Deep Merge Implementation

**File**: `src/merge/deep.rs`

The deep merge implementation is **correct** and follows RFC 7396 semantics:

```rust
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    match (base, overlay) {
        // Null in overlay = delete the key (RFC 7396)
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Both objects: recursive merge
        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            for (key, overlay_val) in overlay_obj {
                if let Some(base_val) = base_obj.remove(&key) {
                    base_obj.insert(key, deep_merge(base_val, overlay_val)?);
                } else {
                    base_obj.insert(key, overlay_val);
                }
            }
            Ok(MergeValue::Object(base_obj))
        }

        // Different types or scalars: overlay wins (layer precedence)
        (_, overlay) => Ok(overlay),
    }
}
```

**Key features**:
- Recursive object merging
- Null deletion semantics
- Layer precedence (higher layers override lower layers)
- Array merging with key-based matching (default keys: `["id", "name"]`)

### 4. Supported File Formats

**File**: `src/merge/layer.rs` (lines 17-28)

```rust
pub enum FileFormat {
    Json,     // .json
    Yaml,     // .yaml, .yml
    Toml,     // .toml
    Ini,      // .ini, .cfg, .conf
    Text,     // Any other extension
}
```

## The Bug

### Root Cause

The merge engine checks for conflicts **before** attempting deep merge:

1. `has_different_structured_content()` parses JSON from multiple layers
2. If the parsed `MergeValue` objects are different, it returns `true` (conflict)
3. The merge is skipped, and a `.jinmerge` file is created

**This is incorrect** because:
- Different JSON objects should be deep-merged, not flagged as conflicts
- Only deep merge failures should create conflicts
- Layer precedence should resolve differences automatically

### Example of Bug Behavior

```bash
# Layer 2 (ModeBase): {"common": {"a": 1}, "mode": true}
# Layer 7 (ProjectBase): {"common": {"a": 1, "b": 2}, "project": false}

# Expected: {"common": {"a": 1, "b": 2}, "mode": true, "project": false}
# Actual: Creates .jinmerge conflict file
```

### Correct Behavior

The merge should:

1. **Attempt deep merge first** for structured files
2. **Only create conflict files when**:
   - Text files have unresolvable 3-way merge conflicts (conflict markers after merge)
   - Deep merge fails for structured files (syntax errors, incompatible types)
3. **Use layer precedence** to resolve differences automatically

## Solution Architecture

### Fix Strategy

1. **Remove the pre-merge conflict check** for structured files
2. **Always attempt deep merge** for JSON/YAML/TOML/INI files
3. **Post-merge validation**: Check if deep merge succeeded before adding to result
4. **Text file behavior unchanged**: Keep 3-way merge with conflict marker detection

### Implementation Location

**File to modify**: `src/merge/layer.rs`

**Function to update**: `merge_layers()` (lines 127-168)

**Changes needed**:

```rust
// OLD (BUGGY):
if layers_with_file.len() > 1 {
    let has_conflict = has_different_content_across_layers(...)?;
    if has_conflict {
        result.conflict_files.push(path.clone());
        continue;
    }
}

// NEW (CORRECT):
if layers_with_file.len() > 1 {
    let format = detect_format(path);

    if format == FileFormat::Text {
        // Keep conflict detection for text files
        let has_conflict = has_different_text_content(...)?;
        if has_conflict {
            result.conflict_files.push(path.clone());
            continue;
        }
    }
    // For structured files: no pre-check, attempt deep merge directly
}

// Always attempt merge for non-conflicting files
merge_file_across_layers(path, &layers_with_file, config, repo)?;
```

## Test Coverage

### Existing Tests

The following test files validate merge behavior:

1. **`src/merge/deep.rs`**: Unit tests for deep merge logic
   - Null deletion semantics
   - Recursive object merging
   - Array merging with keys
   - Type conflicts

2. **`tests/conflict_workflow.rs`**: End-to-end conflict tests
   - Conflict file creation
   - Paused state persistence
   - Manual resolution workflow

3. **`tests/pull_merge.rs`**: 3-way merge tests
   - Fast-forward vs merge scenarios
   - Clean merges vs conflicts

### New Tests Needed

After fixing the bug, add tests for:

1. **Structured file auto-merge**: JSON files with different content should merge without conflicts
2. **Layer precedence**: Higher layers should override lower layers in deep merge
3. **Nested object merging**: Deep merging of nested JSON structures
4. **Array key merging**: Merging arrays by key (id, name)

## Dependencies

The merge fix depends on:

1. **No external dependencies** - All merge logic is internal
2. **Deep merge module** (`src/merge/deep.rs`) - Already correct, no changes needed
3. **File format detection** (`src/merge/layer.rs`) - Already correct, no changes needed

## Impact Assessment

### High-Level Impact

- **User experience**: Significantly improved - fewer manual conflict resolutions
- **PRD compliance**: Fixes §11.1 "Structured Merge Rules" and §11.2 "Merge Priority"
- **Breaking changes**: None - this is a bug fix, not a behavior change
- **Migration needed**: No - existing conflict files can be resolved manually

### Risk Assessment

- **Low risk**: The deep merge implementation is already correct and well-tested
- **Isolated change**: Only affects structured file merge behavior
- **Rollback safe**: Can easily revert if unexpected issues arise

## References

- **PRD §11.1**: "Structured Merge Rules" - Specifies deep merge semantics
- **PRD §11.2**: "Merge Priority" - Defines layer precedence order
- **RFC 7396**: JSON Merge Patch standard
- **Implementation files**:
  - `src/merge/layer.rs` - Layer merge orchestration
  - `src/merge/deep.rs` - Deep merge logic (correct)
  - `src/merge/text.rs` - Text file 3-way merge
