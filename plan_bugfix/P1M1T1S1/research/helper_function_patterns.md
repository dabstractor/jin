# Helper Function Patterns in Jin Merge Layer Module

This document analyzes existing helper function patterns in `/home/dustin/projects/jin/src/merge/layer.rs` to establish conventions for the new `find_layers_containing_file()` function.

## Overview

The merge layer module contains helper functions that orchestrate layer operations. Key functions analyzed:
- `get_applicable_layers()` - Determines which layers apply given context
- `collect_all_file_paths()` - Aggregates files across all layers
- `merge_file_across_layers()` - Merges a single file across multiple layers

## Function Signature Patterns

### Pattern 1: Layer Enumeration Functions

**Example: `get_applicable_layers()`**
```rust
pub fn get_applicable_layers(
    mode: Option<&str>,
    scope: Option<&str>,
    _project: Option<&str>,
) -> Vec<Layer>
```

**Characteristics:**
- Returns `Vec<Layer>` - layers in precedence order
- Takes optional string references for context parameters
- Public API function (not internal helper)
- Pure function (no repo parameter needed)
- Simple return type (no Result)

### Pattern 2: Layer Iteration with Repository Access

**Example: `collect_all_file_paths()`**
```rust
fn collect_all_file_paths(
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<HashSet<PathBuf>>
```

**Characteristics:**
- Private helper function (not pub)
- Takes slice of layers: `&[Layer]`
- Takes config struct: `&LayerMergeConfig`
- Takes repository reference: `&JinRepo`
- Returns `Result<T>` for error handling
- Returns collection type (`HashSet<PathBuf>`)

### Pattern 3: Single File Operations Across Layers

**Example: `merge_file_across_layers()`**
```rust
fn merge_file_across_layers(
    path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<MergedFile>
```

**Characteristics:**
- Private helper function
- Takes specific file path: `&std::path::Path`
- Takes layer slice for iteration
- Takes config and repo references
- Returns Result with complex type (struct)
- Processes layers in precedence order

## Error Handling Conventions

### Error Propagation Pattern

All repository operations use the `?` operator for error propagation:

```rust
// From collect_all_file_paths()
if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
    let commit = repo.inner().find_commit(commit_oid)?;
    let tree_oid = commit.tree_id();

    for file_path in repo.list_tree_files(tree_oid)? {
        paths.insert(PathBuf::from(file_path));
    }
}
```

### Conditional Error Handling

**Pattern: Check ref_exists() before resolve_ref()**

This is a CRITICAL pattern used throughout:

```rust
// CRITICAL: Check ref_exists() before resolve_ref()
if repo.ref_exists(&ref_path) {
    if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
        // ... proceed with operations
    }
}
// Layer ref doesn't exist = no files in this layer (skip gracefully)
```

**Rationale:** The `ref_exists()` check prevents errors when layer refs don't exist yet. Layers gracefully skip if they don't have content.

### Error Types Used

Functions return these JinError variants:
- `JinError::NotFound` - When file doesn't exist in any layer
- `JinError::MergeConflict` - When merge conflicts occur
- `JinError::Git` - Git operation failures
- Propagated via `Result<T>` type alias

## Layer Iteration Patterns

### Pattern 1: Iterating with Context Resolution

```rust
for layer in layers {
    let ref_path = layer.ref_path(
        config.mode.as_deref(),
        config.scope.as_deref(),
        config.project.as_deref(),
    );

    // CRITICAL: Check ref_exists() before resolve_ref()
    if repo.ref_exists(&ref_path) {
        if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
            // ... operations on layer
        }
    }
}
```

**Key points:**
1. Call `layer.ref_path()` with context parameters
2. Check `ref_exists()` before proceeding
3. Use `if let Ok()` for graceful failure handling
4. Layers that don't exist are silently skipped

### Pattern 2: Building Collections While Iterating

```rust
let mut paths = HashSet::new();

for layer in layers {
    let ref_path = layer.ref_path(...);

    if repo.ref_exists(&ref_path) {
        if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
            let commit = repo.inner().find_commit(commit_oid)?;
            let tree_oid = commit.tree_id();

            for file_path in repo.list_tree_files(tree_oid)? {
                paths.insert(PathBuf::from(file_path));
            }
        }
    }
}

Ok(paths)
```

## TreeOps Trait Method Usage

### Available Methods

From `/home/dustin/projects/jin/src/git/tree.rs`:

```rust
pub trait TreeOps {
    // Walk tree in pre-order (parent before children)
    fn walk_tree_pre<F>(&self, tree_oid: Oid, callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult;

    // Get tree entry by path (e.g., "src/config.json")
    fn get_tree_entry(&self, tree_oid: Oid, path: &Path) -> Result<Oid>;

    // Read blob content by OID
    fn read_blob_content(&self, blob_oid: Oid) -> Result<Vec<u8>>;

    // Read file content from tree by path (convenience method)
    fn read_file_from_tree(&self, tree_oid: Oid, path: &Path) -> Result<Vec<u8>>;

    // List all files in tree recursively (returns Vec<String>)
    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>>;
}
```

### Usage Pattern 1: List All Files in Layer

```rust
let commit = repo.inner().find_commit(commit_oid)?;
let tree_oid = commit.tree_id();

for file_path in repo.list_tree_files(tree_oid)? {
    paths.insert(PathBuf::from(file_path));
}
```

### Usage Pattern 2: Read Specific File from Layer

```rust
if let Ok(content) = repo.read_file_from_tree(tree_oid, path) {
    let content_str = String::from_utf8_lossy(&content);
    // ... process content
}
```

### Usage Pattern 3: Check if File Exists in Tree

**Note:** `get_tree_entry()` returns `Err` if path doesn't exist:

```rust
fn get_tree_entry(&self, tree_oid: Oid, path: &Path) -> Result<Oid>;
// Returns JinError::Git if path doesn't exist in the tree
```

This can be used with `if let Ok()` pattern:

```rust
if let Ok(entry_oid) = repo.get_tree_entry(tree_oid, path) {
    // File exists, entry_oid is the blob OID
}
```

## RefOps Trait Method Usage

### Available Methods

From `/home/dustin/projects/jin/src/git/refs.rs`:

```rust
pub trait RefOps {
    fn find_ref(&self, name: &str) -> Result<Reference<'_>>;
    fn set_ref(&self, name: &str, oid: Oid, message: &str) -> Result<()>;
    fn delete_ref(&self, name: &str) -> Result<()>;
    fn list_refs(&self, pattern: &str) -> Result<Vec<String>>;
    fn ref_exists(&self, name: &str) -> bool;  // Returns bool, not Result!
    fn resolve_ref(&self, name: &str) -> Result<Oid>;
}
```

### Key Method: ref_exists()

```rust
fn ref_exists(&self, name: &str) -> bool {
    self.inner().find_reference(name).is_ok()
}
```

**Important:** Returns `bool` not `Result`, making it ideal for conditional checks.

## Naming Conventions

### Function Names

- **Verb-first**: `get_applicable_layers`, `collect_all_file_paths`, `merge_file_across_layers`
- **Snake_case**: All Rust functions use snake_case
- **Descriptive**: Names clearly indicate what they do
- **Scope indicators**: Public functions vs private helpers

### Parameter Names

- `layers` - Slice of layers to process
- `config` - Configuration struct with context
- `repo` - JinRepo reference for Git operations
- `path` - File path (borrowed reference)

### Return Types

- Collections: `Vec<T>`, `HashSet<T>`, `HashMap<K, V>`
- Wrapped in Result: `Result<T>` for fallible operations
- Complex types: Structs like `MergedFile`, `LayerMergeResult`

## Recommended Signature for find_layers_containing_file()

Based on the analyzed patterns, the recommended signature is:

```rust
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>>
```

### Rationale

1. **Public function** - This will be used by multiple modules
2. **Takes `&Path`** - Specific file to search for
3. **Takes `&[Layer]`** - Layers to search (flexible input)
4. **Takes `&LayerMergeConfig`** - Context for ref_path resolution
5. **Takes `&JinRepo`** - Git operations
6. **Returns `Result<Vec<Layer>>`** - Found layers, error if operation fails

### Implementation Outline

```rust
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>> {
    let mut found_layers = Vec::new();

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

            // Use get_tree_entry to check if file exists
            // Returns Err if path doesn't exist
            if repo.get_tree_entry(tree_oid, file_path).is_ok() {
                found_layers.push(*layer);
            }
        }
    }

    Ok(found_layers)
}
```

## Alternative Signatures Considered

### Alternative 1: Return with precedence info

```rust
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<(Layer, u8)>>  // (layer, precedence)
```

**Rejected:** Simpler to return `Vec<Layer>` and caller can use `layer.precedence()` if needed.

### Alternative 2: Use get_applicable_layers() internally

```rust
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
    mode: Option<&str>,
    scope: Option<&str>,
    project: Option<&str>,
    repo: &JinRepo,
) -> Result<Vec<Layer>>
```

**Rejected:** Less flexible - caller may want to search custom layer sets.

## Key Takeaways for Implementation

1. **Always check `ref_exists()` before `resolve_ref()`** - This prevents errors on non-existent layers
2. **Use `if let Ok()` for graceful failure** - Don't fail entire operation if one layer fails
3. **Use `get_tree_entry()` to check file existence** - Returns Err if file not found
4. **Maintain layer precedence order** - Don't reorder the input layers
5. **Return empty Vec if no layers found** - This is not an error condition
6. **Use `Result<Vec<Layer>>` for error propagation** - Only return Err for actual failures
