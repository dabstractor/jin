# PRP: P2.M3 - Deep Merge Algorithm

---

## Goal

**Feature Goal**: Implement the complete deep merge algorithm with null-deletion semantics and configurable array merge strategies, enabling Jin's deterministic 9-layer configuration merge system to combine structured files (JSON, YAML, TOML, INI) according to PRD Section 11.1 specifications.

**Deliverable**:
1. Enhanced `src/merge/deep.rs` with configurable array strategies and robust edge case handling
2. Complete `src/merge/layer.rs` implementation for multi-layer merge orchestration
3. Comprehensive unit tests for all merge scenarios

**Success Definition**:
- All merge tests pass: `cargo test merge::`
- PRD merge rules correctly implemented (null deletes keys, keyed arrays merge by id/name, unkeyed arrays replace)
- Layer merge orchestration correctly applies 9-layer precedence order
- Edge cases handled gracefully (type conflicts, empty values, deeply nested structures)
- `cargo check && cargo clippy && cargo test` all pass with zero errors/warnings

---

## User Persona

**Target User**: Jin internals (commit pipeline, apply command, workspace synchronization)

**Use Case**: The deep merge algorithm is used by:
- `merge_layers()` to combine configurations across Jin's 9-layer hierarchy
- `jin apply` to materialize merged configurations into the workspace
- `jin commit` pipeline to validate merge compatibility before committing
- `jin sync` to apply remote updates with local overlays

**User Journey**: Users don't interact directly with deep_merge - they experience seamless layer composition when activating modes/scopes. A file like `.claude/config.json` automatically combines base mode settings with scope-specific additions and project-level overrides.

**Pain Points Addressed**:
- Deterministic merging regardless of source format
- Null values explicitly delete keys (intentional removal)
- Arrays with keyed objects merge intelligently by id/name
- Clear precedence: higher layers override lower layers

---

## Why

- **PRD Requirement**: Section 11.1 specifies exact merge behaviors:
  - JSON/YAML/TOML: Deep key merge
  - Arrays (keyed): Merge by `id` or `name`
  - Arrays (unkeyed): Higher layer replaces
  - `null`: Deletes key
  - Ordering: Preserved from highest layer
- **Foundation for Apply Command**: P4.M4 (Apply Command) depends on complete layer merge
- **Commit Validation**: Merge must be deterministic and reversible per PRD Section 11.1
- **9-Layer System**: PRD Section 4.1 requires correct precedence handling

---

## What

### User-Visible Behavior

After this milestone:
```rust
// Deep merge two MergeValues
let base = MergeValue::from_json(r#"{"name": "base", "items": [{"id": "1", "val": "a"}]}"#)?;
let overlay = MergeValue::from_json(r#"{"name": "override", "items": [{"id": "1", "val": "b"}]}"#)?;
let merged = deep_merge(base, overlay)?;
// Result: {"name": "override", "items": [{"id": "1", "val": "b"}]}

// Null deletes keys
let overlay = MergeValue::from_json(r#"{"name": null}"#)?;
let merged = deep_merge(base, overlay)?;
// Result: {"items": [...]} - name key removed

// Layer merge orchestration
let config = LayerMergeConfig {
    layers: vec![Layer::GlobalBase, Layer::ModeBase, Layer::ProjectBase],
    mode: Some("claude".to_string()),
    scope: None,
    project: Some("my-project".to_string()),
};
let result = merge_layers(&config, &repo)?;
// Returns merged file contents with layer precedence applied
```

### Technical Requirements

1. **Null-Deletion**: Setting a key to `null` removes it from the merged result
2. **Object Deep Merge**: Nested objects merge recursively
3. **Keyed Array Merge**: Arrays of objects with `id` or `name` fields merge by key
4. **Unkeyed Array Replace**: Other arrays are replaced by higher-precedence layer
5. **Type Conflict Resolution**: Different types at same key = overlay wins
6. **Ordering Preservation**: Key order from highest-precedence layer preserved via IndexMap
7. **Layer Orchestration**: Multiple layers merged in precedence order (1→9)

### Success Criteria

- [ ] `deep_merge()` correctly handles null-deletion at all nesting levels
- [ ] `deep_merge()` correctly merges objects recursively
- [ ] `deep_merge()` correctly merges keyed arrays by `id` or `name`
- [ ] `deep_merge()` correctly replaces unkeyed arrays
- [ ] `deep_merge()` handles type conflicts (overlay wins)
- [ ] `deep_merge()` preserves key ordering from overlay
- [ ] `merge_layers()` applies correct precedence order
- [ ] `merge_layers()` tracks merged/conflict/added/removed files
- [ ] All edge cases handled (empty objects, deeply nested, mixed types)
- [ ] Comprehensive test coverage for all scenarios

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement the deep merge algorithm. An AI agent with access to this PRP and the codebase can implement the feature in one pass._

### Documentation & References

```yaml
# MUST READ - Core Implementation Context

- file: src/merge/deep.rs
  why: Current deep merge implementation - needs enhancement
  lines: 176
  critical: |
    - Already has: deep_merge(base, overlay) -> Result<MergeValue>
    - Already has: Null deletion (overlay null removes key)
    - Already has: Keyed array merge by "id" or "name"
    - Already has: Unkeyed array replacement
    - NEEDS: Better edge case handling, path tracking for errors
    - NEEDS: Configurable array key fields via MergeConfig
    - Pattern: Uses IndexMap::shift_remove() to preserve order
  pattern: |
    match (base, overlay) {
        (_, MergeValue::Null) => Ok(MergeValue::Null),
        (MergeValue::Object(base_obj), MergeValue::Object(overlay_obj)) => {...}
        (MergeValue::Array(base_arr), MergeValue::Array(overlay_arr)) => {...}
        (_, overlay) => Ok(overlay),  // Type mismatch: overlay wins
    }

- file: src/merge/layer.rs
  why: Layer merge orchestration - needs completion
  lines: 143
  critical: |
    - Has: LayerMergeConfig struct with layers, mode, scope, project
    - Has: LayerMergeResult struct with merged/conflict/added/removed
    - Has: get_applicable_layers() to determine active layers
    - NEEDS: Complete merge_layers() implementation
    - NEEDS: Integration with git::TreeOps to read layer files
    - NEEDS: Per-file merge with format detection

- file: src/merge/value.rs
  why: MergeValue type - the core data structure (complete from P2.M2)
  lines: 1161
  critical: |
    - MergeValue enum: Null, Bool, Integer, Float, String, Array, Object
    - Object uses IndexMap<String, MergeValue> for ordering
    - Has: from_json(), from_yaml(), from_toml(), from_ini() parsers
    - Has: to_json_string(), to_yaml_string(), etc. serializers
    - Has: is_null(), is_object(), as_object(), as_str(), as_i64() helpers
    - Already complete - no changes needed

- file: src/core/layer.rs
  why: Layer enum and precedence
  lines: 284
  critical: |
    - 9 layers: GlobalBase(1) through WorkspaceActive(9)
    - precedence() returns u8 (1-9, higher overrides lower)
    - ref_path(mode, scope, project) for git ref names
    - storage_path(mode, scope, project) for layer directories
    - requires_mode(), requires_scope() for validation

- file: src/core/error.rs
  why: Error types for merge operations
  lines: 102
  critical: |
    - JinError::MergeConflict { path: String } for conflicts
    - JinError::Parse { format, message } for format errors
    - JinError::NotFound(String) for missing files
    - Use MergeConflict when merge cannot proceed

- file: src/git/tree.rs
  why: Reading layer files from git
  lines: 349
  critical: |
    - TreeOps trait implemented on JinRepo
    - read_file_from_tree(tree_oid: Oid, path: &Path) -> Result<Vec<u8>>
    - list_tree_files(tree_oid: Oid) -> Result<Vec<String>>
    - get_tree_entry(tree_oid: Oid, path: &Path) -> Result<Oid>
    - walk_tree_pre(tree_oid, callback) for tree traversal
    - Used by layer.rs to read layer contents from git

- file: src/git/refs.rs
  why: Resolving layer refs to OIDs
  lines: 246
  critical: |
    - RefOps trait implemented on JinRepo
    - resolve_ref(name: &str) -> Result<Oid>  # Gets target OID
    - ref_exists(name: &str) -> bool          # Checks if ref exists
    - set_ref(name, oid, message) -> Result<()>
    - find_ref(name) -> Result<Reference<'_>>
    - Layer refs under refs/jin/layers/...
    - IMPORTANT: Always check ref_exists() before resolve_ref()

# RESEARCH DOCUMENTS - Comprehensive analysis already completed

- file: DEEP_MERGE_RESEARCH.md (in project root)
  why: Comprehensive deep merge algorithm research
  lines: ~1400
  critical: |
    - Lodash _.merge, _.mergeWith patterns
    - webpack-merge strategies and customizations
    - deepmerge library patterns
    - Rust-specific patterns with serde_json and IndexMap
    - Performance considerations and benchmarks
    - Decision matrix for choosing strategies

- file: NULL_DELETION_PATTERNS.md (in project root)
  why: Null-deletion semantics research
  lines: ~1200
  critical: |
    - RFC 7396 (JSON Merge Patch) complete specification
    - Kubernetes Strategic Merge Patch patterns
    - Terraform null-as-unset semantics
    - Implementation patterns for nested null deletion
    - Edge cases and gotchas

- file: ARRAY_MERGE_STRATEGIES.md (in project root)
  why: Array merge strategy research
  lines: ~1360
  critical: |
    - Replace, Append, Prepend, Union, Keyed merge strategies
    - Kubernetes Strategic Merge Patch array handling
    - webpack-merge customizeArray patterns
    - Edge cases: empty arrays, null elements, mixed types
    - Order preservation patterns

# EXTERNAL REFERENCES

- url: https://datatracker.ietf.org/doc/html/rfc7396
  why: JSON Merge Patch specification (the standard Jin follows)
  critical: |
    - Null semantics: "If the value is null, remove the member"
    - Recursive merge for objects
    - Arrays are replaced, not merged (Jin extends with keyed merge)

- url: https://kubernetes.io/docs/tasks/manage-kubernetes-objects/update-api-object-kubectl-patch/
  why: Strategic merge patch patterns
  critical: |
    - Keyed array merge via "patchMergeKey" (like Jin's id/name)
    - Directive markers like $patch: delete
    - Jin uses similar keyed merge but simpler

- url: https://crates.io/crates/merge
  why: Rust merge crate for reference
  critical: |
    - Shows idiomatic Rust patterns for merging
    - Uses traits and derive macros
```

### Current Codebase Tree (Relevant Files)

```bash
jin/
├── src/
│   ├── core/
│   │   ├── error.rs          # JinError::MergeConflict, Parse, NotFound
│   │   ├── layer.rs          # Layer enum (9 layers), precedence()
│   │   └── mod.rs            # Core exports
│   ├── git/
│   │   ├── refs.rs           # RefOps: resolve_ref(), ref_exists()
│   │   ├── tree.rs           # TreeOps: read_file_from_tree(), list_tree_files()
│   │   ├── objects.rs        # ObjectOps: create_blob(), create_tree()
│   │   ├── repo.rs           # JinRepo wrapper
│   │   └── mod.rs            # Git exports
│   └── merge/
│       ├── mod.rs            # Module exports
│       ├── deep.rs           # deep_merge() - TO BE ENHANCED (176 lines)
│       ├── layer.rs          # merge_layers() - TO BE COMPLETED (143 lines)
│       ├── value.rs          # MergeValue (complete, 1161 lines)
│       └── text.rs           # text_merge() for plain text
├── Cargo.toml                # Dependencies (complete)
├── DEEP_MERGE_RESEARCH.md    # Research: deep merge algorithms
├── NULL_DELETION_PATTERNS.md # Research: null-deletion patterns
├── ARRAY_MERGE_STRATEGIES.md # Research: array merge strategies
└── tests/
    └── integration/
        └── cli_basic.rs      # CLI integration tests
```

### Desired Codebase Tree After P2.M3

```bash
jin/
├── src/
│   └── merge/
│       ├── mod.rs            # Updated exports (add MergeConfig if public)
│       ├── deep.rs           # ENHANCED (~350 lines):
│       │   ├── MergeConfig struct
│       │   ├── deep_merge(base, overlay) - backward compatible
│       │   ├── deep_merge_with_config(base, overlay, config)
│       │   ├── merge_arrays_with_config(base, overlay, config)
│       │   ├── extract_array_keys(arr, key_fields)
│       │   └── comprehensive #[cfg(test)] tests (~50 cases)
│       ├── layer.rs          # COMPLETED (~400 lines):
│       │   ├── FileFormat enum
│       │   ├── MergedFile struct
│       │   ├── merge_layers(config, repo) - full implementation
│       │   ├── merge_file_across_layers(path, layers, config, repo)
│       │   ├── collect_all_file_paths(layers, config, repo)
│       │   ├── detect_format(path) -> FileFormat
│       │   ├── parse_content(content, format) -> MergeValue
│       │   └── comprehensive tests
│       ├── value.rs          # (unchanged from P2.M2)
│       └── text.rs           # (unchanged)
└── plan/
    └── P2M3/
        └── PRP.md            # This file
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: Git API for layer resolution
// ============================================================
// ALWAYS check ref_exists() BEFORE calling resolve_ref()!
// resolve_ref() will return an error if the ref doesn't exist.
//
// CORRECT PATTERN:
// let ref_path = layer.ref_path(mode, scope, project);
// if repo.ref_exists(&ref_path) {
//     if let Ok(oid) = repo.resolve_ref(&ref_path) {
//         let commit = repo.inner().find_commit(oid)?;
//         let tree_oid = commit.tree_id();
//         // Now read files from tree
//     }
// }
// // If ref doesn't exist, skip this layer gracefully

// ============================================================
// CRITICAL: Null-deletion semantics (RFC 7396 JSON Merge Patch)
// ============================================================
// When overlay has null, the KEY IS REMOVED from result:
//
// base:    {"keep": 1, "delete": 2}
// overlay: {"delete": null}
// result:  {"keep": 1}  // "delete" key is GONE
//
// This is different from SETTING a key to null!
// Jin uses null as a deletion marker, not a value.

// ============================================================
// CRITICAL: Keyed array merge order
// ============================================================
// When merging arrays by key, the ORDER follows:
// 1. Base array items (in original order)
// 2. Overlay items that match base keys (merged in place)
// 3. Overlay items with new keys (appended)
//
// Example:
// base:    [{"id": "1", "v": "a"}, {"id": "2", "v": "b"}]
// overlay: [{"id": "2", "v": "B"}, {"id": "3", "v": "c"}]
// result:  [{"id": "1", "v": "a"}, {"id": "2", "v": "B"}, {"id": "3", "v": "c"}]

// ============================================================
// GOTCHA: Mixed array detection
// ============================================================
// If an array has SOME objects with id/name and SOME without,
// or if SOME are objects and SOME are scalars:
// → Treat as UNKEYED (overlay replaces)
// → Don't partially merge
//
// This prevents inconsistent behavior.

// ============================================================
// GOTCHA: Empty arrays and objects
// ============================================================
// Empty array [] in overlay should REPLACE base array
// Empty object {} in overlay should MERGE (not replace)
// This matches RFC 7396 semantics

// ============================================================
// PATTERN: IndexMap ordering preservation
// ============================================================
// Use shift_remove() when removing keys to preserve order
// Use insert() to add keys (appends to end)
// The overlay's key order takes precedence for new keys

// ============================================================
// PATTERN: Type conflict resolution
// ============================================================
// When types differ at same key:
// base:    {"config": {"nested": true}}
// overlay: {"config": "override"}
// result:  {"config": "override"}  // overlay wins completely
//
// This is intentional - overlay can "flatten" nested structures

// ============================================================
// PATTERN: Layer file collection
// ============================================================
// When collecting files across layers:
// 1. Start from lowest precedence (GlobalBase = 1)
// 2. Track all unique file paths
// 3. For each path, merge from all layers that contain it
// 4. Higher layers override lower for same path
//
// Files only in higher layers are "added"
// Files in lower layers but nulled in higher are "removed"
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/merge/deep.rs ADDITIONS ==================

use crate::core::Result;
use super::MergeValue;
use indexmap::IndexMap;

/// Configuration for merge operations
#[derive(Debug, Clone, Default)]
pub struct MergeConfig {
    /// Key fields to use for keyed array merge (default: ["id", "name"])
    pub array_key_fields: Vec<String>,
}

impl MergeConfig {
    /// Create config with default settings
    pub fn new() -> Self {
        Self {
            array_key_fields: vec!["id".to_string(), "name".to_string()],
        }
    }

    /// Create config with custom key fields
    pub fn with_key_fields(fields: Vec<String>) -> Self {
        Self {
            array_key_fields: fields,
        }
    }
}

// ================== src/merge/layer.rs ADDITIONS ==================

use crate::core::{Layer, Result, JinError};
use crate::git::{JinRepo, RefOps, TreeOps};
use super::{deep_merge, MergeValue};
use std::collections::HashSet;
use std::path::PathBuf;

/// File format for parsing and serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Json,
    Yaml,
    Toml,
    Ini,
    Text,
}

/// Represents a merged file across layers
#[derive(Debug)]
pub struct MergedFile {
    /// Final merged content
    pub content: MergeValue,
    /// Layers that contributed to this file
    pub source_layers: Vec<Layer>,
    /// Original format (for serialization)
    pub format: FileFormat,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD MergeConfig struct to src/merge/deep.rs
  - ADD: MergeConfig struct with array_key_fields field
  - ADD: MergeConfig::new() with default ["id", "name"]
  - ADD: MergeConfig::with_key_fields(fields) constructor
  - PLACEMENT: Near top of file, after imports
  - NAMING: Follow Rust conventions

Task 2: ADD deep_merge_with_config() to src/merge/deep.rs
  - ADD: fn deep_merge_with_config(base, overlay, config) -> Result<MergeValue>
  - MODIFY: Existing deep_merge() to call deep_merge_with_config with defaults
  - ENSURE: Backward compatibility (existing callers work unchanged)
  - PLACEMENT: After MergeConfig

Task 3: ENHANCE merge_arrays() in src/merge/deep.rs
  - RENAME: merge_arrays -> merge_arrays_with_config
  - MODIFY: Accept &MergeConfig parameter
  - MODIFY: extract_array_keys() to use config.array_key_fields
  - ENSURE: Order preservation (base items first, then new overlay items)
  - PLACEMENT: After deep_merge functions

Task 4: ADD helper types to src/merge/layer.rs
  - ADD: FileFormat enum (Json, Yaml, Toml, Ini, Text)
  - ADD: MergedFile struct (content, source_layers, format)
  - ADD: detect_format(path: &PathBuf) -> FileFormat helper
  - ADD: parse_content(content: &str, format: FileFormat) -> Result<MergeValue>
  - PLACEMENT: After existing structs, before functions

Task 5: IMPLEMENT collect_all_file_paths() in src/merge/layer.rs
  - SIGNATURE: fn collect_all_file_paths(layers: &[Layer], config: &LayerMergeConfig, repo: &JinRepo) -> Result<HashSet<PathBuf>>
  - IMPORT: use crate::git::RefOps;  // at top of file
  - USE: repo.ref_exists(&ref_path) to check before resolving
  - USE: repo.resolve_ref(&ref_path) to get commit OID
  - USE: repo.list_tree_files(tree_oid) to get file paths
  - HANDLE: Layers that don't exist yet - skip gracefully
  - PLACEMENT: Helper function before merge_layers()

Task 6: IMPLEMENT merge_file_across_layers() in src/merge/layer.rs
  - SIGNATURE: fn merge_file_across_layers(path: &PathBuf, layers: &[Layer], config: &LayerMergeConfig, repo: &JinRepo) -> Result<MergedFile>
  - IMPORT: use crate::git::RefOps;  // at top of file
  - CHECK: repo.ref_exists() before repo.resolve_ref()
  - READ: File content from each layer using repo.read_file_from_tree()
  - PARSE: Using parse_content() with detect_format()
  - MERGE: Using deep_merge() in precedence order (lowest first)
  - TRACK: source_layers for each contributing layer
  - PLACEMENT: After collect_all_file_paths()

Task 7: COMPLETE merge_layers() in src/merge/layer.rs
  - REPLACE: Current TODO stub implementation
  - USE: collect_all_file_paths() to gather all file paths
  - ITERATE: Each path, call merge_file_across_layers()
  - TRACK: merged_files, conflict_files in LayerMergeResult
  - HANDLE: JinError::MergeConflict -> add to conflict_files
  - RETURN: Complete LayerMergeResult

Task 8: ADD comprehensive tests for deep.rs
  - FILE: In #[cfg(test)] mod tests section
  - TEST: Null deletion (shallow, nested, deeply nested, entire object)
  - TEST: Object merge (add keys, overlay wins, type conflicts both ways)
  - TEST: Keyed array merge (by id, by name, order preservation, append new)
  - TEST: Unkeyed array replacement (primitives, strings, mixed)
  - TEST: Empty values (empty object merges, empty array replaces)
  - TEST: MergeConfig (default, custom key fields)
  - PATTERN: Use serde_json::json! macro
  - VERIFY: All existing tests still pass

Task 9: ADD comprehensive tests for layer.rs
  - FILE: In #[cfg(test)] mod tests section
  - TEST: detect_format() for all extensions
  - TEST: parse_content() for all formats
  - TEST: get_applicable_layers() various combinations
  - TEST: Layer precedence validation
  - MOCK: Use tempfile and JinRepo::create_at() for test repos if needed
  - PATTERN: Follow existing test patterns in file

Task 10: UPDATE src/merge/mod.rs exports
  - ADD: pub use deep::MergeConfig; if needed externally
  - ADD: pub use layer::{FileFormat, MergedFile}; if needed
  - PRESERVE: All existing exports
  - VERIFY: cargo build succeeds
```

### Implementation Patterns & Key Details

```rust
// ================== Enhanced deep_merge ==================

/// Perform a deep merge of two MergeValues (backward compatible)
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    deep_merge_with_config(base, overlay, &MergeConfig::new())
}

/// Deep merge with custom configuration
pub fn deep_merge_with_config(
    base: MergeValue,
    overlay: MergeValue,
    config: &MergeConfig,
) -> Result<MergeValue> {
    match (base, overlay) {
        // Null in overlay = delete the key (RFC 7396)
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Both objects: recursive merge
        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            for (key, overlay_val) in overlay_obj {
                if overlay_val.is_null() {
                    // Null removes the key entirely
                    base_obj.shift_remove(&key);
                } else if let Some(base_val) = base_obj.shift_remove(&key) {
                    // Recursively merge existing keys
                    let merged = deep_merge_with_config(base_val, overlay_val, config)?;
                    if !merged.is_null() {
                        base_obj.insert(key, merged);
                    }
                } else {
                    // Add new keys from overlay
                    base_obj.insert(key, overlay_val);
                }
            }
            Ok(MergeValue::Object(base_obj))
        }

        // Both arrays: attempt keyed merge, otherwise replace
        (MergeValue::Array(base_arr), MergeValue::Array(overlay_arr)) => {
            // Empty overlay array replaces entirely
            if overlay_arr.is_empty() {
                return Ok(MergeValue::Array(overlay_arr));
            }

            let result = merge_arrays_with_config(base_arr, overlay_arr, config)?;
            Ok(MergeValue::Array(result))
        }

        // Different types or scalars: overlay wins
        (_, overlay) => Ok(overlay),
    }
}

// ================== Layer merge orchestration ==================

/// Merge all applicable layers for the given configuration
pub fn merge_layers(config: &LayerMergeConfig, repo: &JinRepo) -> Result<LayerMergeResult> {
    use crate::git::RefOps;

    let mut result = LayerMergeResult::new();

    // Collect all unique file paths across all layers
    let all_paths = collect_all_file_paths(&config.layers, config, repo)?;

    // Merge each file path
    for path in all_paths {
        match merge_file_across_layers(&path, &config.layers, config, repo) {
            Ok(_merged) => {
                result.merged_files.push(path);
            }
            Err(JinError::MergeConflict { .. }) => {
                result.conflict_files.push(path);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(result)
}

/// Collect all unique file paths across all applicable layers
fn collect_all_file_paths(
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<HashSet<PathBuf>> {
    use crate::git::RefOps;

    let mut paths = HashSet::new();

    for layer in layers {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref()
        if repo.ref_exists(&ref_path) {
            if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
                let commit = repo.inner().find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                for file_path in repo.list_tree_files(tree_oid)? {
                    paths.insert(PathBuf::from(file_path));
                }
            }
        }
        // Layer ref doesn't exist = no files in this layer (skip)
    }

    Ok(paths)
}

/// Merge a single file across multiple layers
fn merge_file_across_layers(
    path: &PathBuf,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<MergedFile> {
    use crate::git::RefOps;

    let mut accumulated: Option<MergeValue> = None;
    let mut source_layers = Vec::new();
    let mut format = FileFormat::Text;

    // Process layers in precedence order (lowest first)
    for layer in layers {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref()
        if !repo.ref_exists(&ref_path) {
            continue;
        }

        if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
            let commit = repo.inner().find_commit(commit_oid)?;
            let tree_oid = commit.tree_id();

            if let Ok(content) = repo.read_file_from_tree(tree_oid, path.as_path()) {
                let content_str = String::from_utf8_lossy(&content);

                format = detect_format(path);
                let layer_value = parse_content(&content_str, format)?;

                source_layers.push(*layer);

                accumulated = Some(match accumulated {
                    Some(base) => deep_merge(base, layer_value)?,
                    None => layer_value,
                });
            }
        }
    }

    match accumulated {
        Some(content) => Ok(MergedFile {
            content,
            source_layers,
            format,
        }),
        None => Err(JinError::NotFound(path.display().to_string())),
    }
}

fn detect_format(path: &PathBuf) -> FileFormat {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "json" => FileFormat::Json,
        "yaml" | "yml" => FileFormat::Yaml,
        "toml" => FileFormat::Toml,
        "ini" | "cfg" | "conf" => FileFormat::Ini,
        _ => FileFormat::Text,
    }
}

fn parse_content(content: &str, format: FileFormat) -> Result<MergeValue> {
    match format {
        FileFormat::Json => MergeValue::from_json(content),
        FileFormat::Yaml => MergeValue::from_yaml(content),
        FileFormat::Toml => MergeValue::from_toml(content),
        FileFormat::Ini => MergeValue::from_ini(content),
        FileFormat::Text => Ok(MergeValue::String(content.to_string())),
    }
}
```

### Integration Points

```yaml
DEPENDENCIES (already in Cargo.toml):
  - indexmap = { version = "2.0", features = ["serde"] }
  - serde_json, serde_yaml, toml, rust-ini (for MergeValue)
  - git2 (for TreeOps, RefOps)
  - thiserror (for JinError)

IMPORTS NEEDED in layer.rs:
  - use crate::git::RefOps;  # For resolve_ref(), ref_exists()

MERGE MODULE:
  - value.rs: MergeValue type (complete from P2.M2)
  - text.rs: text_merge() for plain text (separate from deep_merge)
  - deep.rs: Core algorithm (this milestone)
  - layer.rs: Multi-layer orchestration (this milestone)

GIT MODULE:
  - repo.rs: JinRepo wrapper
  - refs.rs: RefOps (resolve_ref, ref_exists)
  - tree.rs: TreeOps (read_file_from_tree, list_tree_files)
  - No changes needed to git module

CORE MODULE:
  - layer.rs: Layer enum and precedence
  - error.rs: JinError types
  - No changes needed to core module
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
# Run deep merge tests
cargo test merge::deep::              # All deep.rs tests

# Run layer merge tests
cargo test merge::layer::             # All layer.rs tests

# Run specific test categories
cargo test test_null_delete           # Null deletion tests
cargo test test_keyed_array           # Keyed array merge tests
cargo test test_layer                 # Layer tests
cargo test test_empty_                # Empty value tests
cargo test test_type_conflict         # Type conflict tests
cargo test test_detect_format         # Format detection tests

# Run with output for debugging
cargo test merge:: -- --nocapture

# Expected: All tests pass
```

### Level 4: Integration Testing

```bash
# Full test suite including existing tests
cargo test

# Verify P2.M2 tests still pass (regression test)
cargo test merge::value::             # Format parser tests

# Verify text merge tests still pass
cargo test merge::text::              # Text merge tests

# Verify core module tests still pass
cargo test core::                     # Layer, error tests

# Verify git module tests still pass
cargo test git::                      # Tree, refs tests

# Expected: All tests pass, no regressions
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
- [ ] `cargo test merge::deep::` all tests pass
- [ ] `cargo test merge::layer::` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] Null deletion works at top level
- [ ] Null deletion works in nested objects
- [ ] Null deletion works in deeply nested objects (3+ levels)
- [ ] Object merge is recursive
- [ ] Object merge handles type conflicts (overlay wins)
- [ ] Keyed array merge by "id" field works
- [ ] Keyed array merge by "name" field works
- [ ] Keyed array merge with custom key fields works
- [ ] New keyed items are appended to result
- [ ] Keyed array merge preserves base item order
- [ ] Unkeyed arrays are replaced entirely
- [ ] Mixed arrays (some keyed, some not) are replaced
- [ ] Arrays of primitives are replaced
- [ ] Empty array in overlay replaces base array
- [ ] Empty object in overlay merges (doesn't replace)
- [ ] Key ordering preserved from overlay via IndexMap
- [ ] detect_format() works for all extensions
- [ ] parse_content() works for all formats
- [ ] ref_exists() checked before resolve_ref() in all places

### Code Quality Validation

- [ ] All new methods have doc comments
- [ ] Error handling uses JinError types consistently
- [ ] No unwrap() in library code (only in tests)
- [ ] Uses IndexMap throughout for key ordering
- [ ] Uses shift_remove() for order-preserving removal
- [ ] Tests cover all edge cases
- [ ] Follows existing code patterns in merge module
- [ ] MergeConfig provides backward compatibility

---

## Anti-Patterns to Avoid

- ❌ Don't use `HashMap` - use `IndexMap` for key order preservation
- ❌ Don't use `remove()` on IndexMap - use `shift_remove()` to preserve order
- ❌ Don't partially merge mixed arrays - replace entirely if not all keyed
- ❌ Don't treat null as a value - null means DELETE the key
- ❌ Don't call `resolve_ref()` without checking `ref_exists()` first
- ❌ Don't assume all layers exist - check and skip gracefully
- ❌ Don't ignore format detection - wrong format = parse error
- ❌ Don't use unwrap() in library code - propagate errors with ?
- ❌ Don't break existing tests - all P2.M1 and P2.M2 tests must pass
- ❌ Don't forget to import RefOps trait when using repo.ref_exists()

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- deep.rs already has working basic implementation (176 lines) - enhancement only
- layer.rs has clear stubs with defined types (143 lines) - completion path is clear
- All dependencies in place (IndexMap, git module, MergeValue)
- PRD requirements are specific and unambiguous (Section 11.1)
- RFC 7396 provides clear semantics for null-deletion
- Test patterns well established from P2.M1 and P2.M2
- No architectural changes needed - builds on existing foundation
- Extensive research already completed (3000+ lines)
- Git APIs documented and tested (TreeOps, RefOps)
- Clear API contract: ref_exists() before resolve_ref()

**Remaining Risks:**
- Layer ref resolution edge cases - mitigated by ref_exists() check
- Large file performance (unlikely for config files)
- Text file handling via text_merge() (deferred to P2.M4)

---

## Research Artifacts

Comprehensive research has been completed and is available in the project root:

| File | Lines | Description |
|------|-------|-------------|
| `DEEP_MERGE_RESEARCH.md` | ~1400 | Lodash, webpack-merge, deepmerge patterns; Rust implementations |
| `NULL_DELETION_PATTERNS.md` | ~1200 | RFC 7396, K8s Strategic Merge Patch, nested deletion |
| `ARRAY_MERGE_STRATEGIES.md` | ~1360 | Replace, Append, Union, Keyed strategies; edge cases |

Key external references:
- JSON Merge Patch (RFC 7396): https://datatracker.ietf.org/doc/html/rfc7396
- Kubernetes Strategic Merge Patch: https://kubernetes.io/docs/tasks/manage-kubernetes-objects/update-api-object-kubectl-patch/
- webpack-merge strategies: https://github.com/survivejs/webpack-merge
- deepmerge library: https://github.com/TehShrike/deepmerge
- Rust merge crate: https://crates.io/crates/merge

---

## Appendix: Merge Behavior Matrix

| Base | Overlay | Result | Notes |
|------|---------|--------|-------|
| `{"a": 1}` | `{"b": 2}` | `{"a": 1, "b": 2}` | Keys merged |
| `{"a": 1}` | `{"a": 2}` | `{"a": 2}` | Overlay wins |
| `{"a": 1}` | `{"a": null}` | `{}` | Key deleted |
| `{"a": {"b": 1}}` | `{"a": {"c": 2}}` | `{"a": {"b": 1, "c": 2}}` | Nested merge |
| `{"a": {"b": 1}}` | `{"a": null}` | `{}` | Nested deletion |
| `{"a": 1}` | `{"a": {"b": 2}}` | `{"a": {"b": 2}}` | Type change: overlay wins |
| `{"a": {"b": 1}}` | `{"a": "string"}` | `{"a": "string"}` | Type change: overlay wins |
| `[1, 2]` | `[3, 4]` | `[3, 4]` | Unkeyed: replace |
| `["a", "b"]` | `["c"]` | `["c"]` | Unkeyed strings: replace |
| `[{"id": "1", "v": "a"}]` | `[{"id": "1", "v": "b"}]` | `[{"id": "1", "v": "b"}]` | Keyed: merge |
| `[{"id": "1"}]` | `[{"id": "2"}]` | `[{"id": "1"}, {"id": "2"}]` | Keyed: append new |
| `[{"name": "x", "v": 1}]` | `[{"name": "x", "v": 2}]` | `[{"name": "x", "v": 2}]` | Keyed by name |
| `[1, {"id": "1"}]` | `[{"id": "2"}]` | `[{"id": "2"}]` | Mixed: replace |
| `[1, 2]` | `[]` | `[]` | Empty overlay replaces |
| `{"a": 1}` | `{}` | `{"a": 1}` | Empty overlay merges |
| `{}` | `{"a": 1}` | `{"a": 1}` | Empty base: takes overlay |

---

## Appendix: Test Case Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn json_to_merge(json: serde_json::Value) -> MergeValue {
        MergeValue::from(json)
    }

    // ========== Null Deletion Tests ==========

    #[test]
    fn test_null_deletes_top_level_key() {
        let base = json_to_merge(serde_json::json!({"keep": 1, "delete": 2}));
        let overlay = json_to_merge(serde_json::json!({"delete": null}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert!(obj.contains_key("keep"));
        assert!(!obj.contains_key("delete"));
    }

    #[test]
    fn test_null_deletes_nested_key() {
        let base = json_to_merge(serde_json::json!({"outer": {"keep": 1, "delete": 2}}));
        let overlay = json_to_merge(serde_json::json!({"outer": {"delete": null}}));

        let result = deep_merge(base, overlay).unwrap();
        let outer = result.as_object().unwrap().get("outer").unwrap();
        let inner = outer.as_object().unwrap();

        assert!(inner.contains_key("keep"));
        assert!(!inner.contains_key("delete"));
    }

    #[test]
    fn test_null_deletes_deeply_nested_key() {
        let base = json_to_merge(serde_json::json!({
            "a": { "b": { "c": { "keep": 1, "delete": 2 } } }
        }));
        let overlay = json_to_merge(serde_json::json!({
            "a": { "b": { "c": { "delete": null } } }
        }));

        let result = deep_merge(base, overlay).unwrap();
        let c = result.as_object().unwrap()
            .get("a").unwrap().as_object().unwrap()
            .get("b").unwrap().as_object().unwrap()
            .get("c").unwrap().as_object().unwrap();

        assert!(c.contains_key("keep"));
        assert!(!c.contains_key("delete"));
    }

    // ========== Object Merge Tests ==========

    #[test]
    fn test_object_merge_adds_new_keys() {
        let base = json_to_merge(serde_json::json!({"a": 1}));
        let overlay = json_to_merge(serde_json::json!({"b": 2}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_i64(), Some(2));
    }

    #[test]
    fn test_object_merge_overlay_wins_on_conflict() {
        let base = json_to_merge(serde_json::json!({"a": 1}));
        let overlay = json_to_merge(serde_json::json!({"a": 2}));

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(result.as_object().unwrap().get("a").unwrap().as_i64(), Some(2));
    }

    #[test]
    fn test_type_conflict_object_to_scalar() {
        let base = json_to_merge(serde_json::json!({"a": {"nested": true}}));
        let overlay = json_to_merge(serde_json::json!({"a": "string"}));

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(result.as_object().unwrap().get("a").unwrap().as_str(), Some("string"));
    }

    #[test]
    fn test_type_conflict_scalar_to_object() {
        let base = json_to_merge(serde_json::json!({"a": "string"}));
        let overlay = json_to_merge(serde_json::json!({"a": {"nested": true}}));

        let result = deep_merge(base, overlay).unwrap();
        assert!(result.as_object().unwrap().get("a").unwrap().as_object().is_some());
    }

    // ========== Keyed Array Merge Tests ==========

    #[test]
    fn test_keyed_array_merge_by_id() {
        let base = json_to_merge(serde_json::json!([
            {"id": "1", "value": "a"},
            {"id": "2", "value": "b"}
        ]));
        let overlay = json_to_merge(serde_json::json!([{"id": "2", "value": "B"}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 2);
        let item2 = arr.iter().find(|v| {
            v.as_object().unwrap().get("id").unwrap().as_str() == Some("2")
        }).unwrap();
        assert_eq!(item2.as_object().unwrap().get("value").unwrap().as_str(), Some("B"));
    }

    #[test]
    fn test_keyed_array_preserves_order() {
        let base = json_to_merge(serde_json::json!([
            {"id": "1"}, {"id": "2"}, {"id": "3"}
        ]));
        let overlay = json_to_merge(serde_json::json!([{"id": "2", "new": true}, {"id": "4"}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        let ids: Vec<_> = arr.iter()
            .map(|v| v.as_object().unwrap().get("id").unwrap().as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["1", "2", "3", "4"]);
    }

    // ========== Empty Value Tests ==========

    #[test]
    fn test_empty_overlay_array_replaces() {
        let base = json_to_merge(serde_json::json!([1, 2, 3]));
        let overlay = json_to_merge(serde_json::json!([]));

        let result = deep_merge(base, overlay).unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_empty_overlay_object_merges() {
        let base = json_to_merge(serde_json::json!({"a": 1, "b": 2}));
        let overlay = json_to_merge(serde_json::json!({}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_i64(), Some(2));
    }

    // ========== MergeConfig Tests ==========

    #[test]
    fn test_merge_config_default() {
        let config = MergeConfig::new();
        assert_eq!(config.array_key_fields, vec!["id", "name"]);
    }

    #[test]
    fn test_merge_config_custom_keys() {
        let config = MergeConfig::with_key_fields(vec!["key".into(), "uuid".into()]);

        let base = json_to_merge(serde_json::json!([{"key": "1", "v": "a"}]));
        let overlay = json_to_merge(serde_json::json!([{"key": "1", "v": "b"}]));

        let result = deep_merge_with_config(base, overlay, &config).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_object().unwrap().get("v").unwrap().as_str(), Some("b"));
    }

    // ========== Format Detection Tests ==========

    #[test]
    fn test_detect_format_all_types() {
        assert_eq!(detect_format(&PathBuf::from("x.json")), FileFormat::Json);
        assert_eq!(detect_format(&PathBuf::from("x.yaml")), FileFormat::Yaml);
        assert_eq!(detect_format(&PathBuf::from("x.yml")), FileFormat::Yaml);
        assert_eq!(detect_format(&PathBuf::from("x.toml")), FileFormat::Toml);
        assert_eq!(detect_format(&PathBuf::from("x.ini")), FileFormat::Ini);
        assert_eq!(detect_format(&PathBuf::from("x.cfg")), FileFormat::Ini);
        assert_eq!(detect_format(&PathBuf::from("x.md")), FileFormat::Text);
        assert_eq!(detect_format(&PathBuf::from("x.txt")), FileFormat::Text);
    }

    // ========== Layer Precedence Tests ==========

    #[test]
    fn test_layer_precedence_order() {
        let layers = Layer::all_in_precedence_order();
        assert_eq!(layers[0], Layer::GlobalBase);
        assert_eq!(layers[8], Layer::WorkspaceActive);

        for i in 0..layers.len() - 1 {
            assert!(layers[i].precedence() < layers[i + 1].precedence());
        }
    }

    #[test]
    fn test_applicable_layers_mode_only() {
        let layers = get_applicable_layers(Some("claude"), None, None);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ModeBase));
        assert!(!layers.contains(&Layer::ModeScope));
    }

    #[test]
    fn test_applicable_layers_mode_and_scope() {
        let layers = get_applicable_layers(Some("claude"), Some("lang:js"), None);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ModeBase));
        assert!(layers.contains(&Layer::ModeScope));
        assert!(layers.contains(&Layer::ScopeBase));
    }
}
```
