# Conflict Detection Patterns Research

## 1. How Merge Conflicts Are Currently Detected

### Text Merge Conflicts (src/merge/text.rs)
- Uses the `diffy` crate for line-level 3-way merging
- Conflict detection happens in `text_merge()` and `text_merge_with_config()`
- When `diffy::merge()` returns `Err(String)`, it means conflicts exist with conflict markers already inserted
- The `diffy` library returns:
  - `Ok(String)` = clean merge result
  - `Err(String)` = content WITH conflict markers (NOT an error condition)

### Structured Data Conflicts (src/merge/deep.rs)
- Deep merge for JSON/YAML/TOML/INI files using RFC 7396 semantics
- No explicit conflict detection - merge is always attempted
- Overlapping modifications result in the overlay winning completely

### Layer Merge Conflicts (src/merge/layer.rs)
- During `merge_layers()`, if a file merge returns `JinError::MergeConflict`, the file path is added to `conflict_files`
- The merge operation continues to process all files before returning the full result

## 2. Error Types for Conflicts

### Primary Error Type
```rust
// src/core/error.rs:25-27
#[error("Merge conflict in {path}")]
MergeConflict { path: String },
```

### Result Types for Merge Operations

#### Text Merge Result
```rust
// src/merge/text.rs:26-37
pub enum TextMergeResult {
    Clean(String),  // No conflicts
    Conflict {      // Has conflicts
        content: String,        // Content with conflict markers
        conflict_count: usize,  // Number of conflict regions
    },
}
```

#### Layer Merge Result
```rust
// src/merge/layer.rs:54-65
pub struct LayerMergeResult {
    pub merged_files: HashMap<PathBuf, MergedFile>,
    pub conflict_files: Vec<PathBuf>,
    pub added_files: Vec<PathBuf>,
    pub removed_files: Vec<PathBuf>,
}
```

## 3. Functions That Detect or Return Conflicts

### Text Merge Functions
- `text_merge(base, ours, theirs) -> Result<TextMergeResult>` (src/merge/text.rs:116)
- `text_merge_with_config(base, ours, theirs, config) -> Result<TextMergeResult>` (src/merge/text.rs:133)
- `has_conflict_markers(content: &str) -> bool` (src/merge/text.rs:175)
- `parse_conflicts(content: &str) -> Result<Vec<ConflictRegion>>` (src/merge/text.rs:203)
- `count_conflict_regions(content: &str) -> usize` (src/merge/text.rs:297)

### Layer Merge Functions
- `merge_layers(config: &LayerMergeConfig, repo: &JinRepo) -> Result<LayerMergeResult>` (src/merge/layer.rs:103)

### JinMerge Functions
- `JinMergeConflict::from_text_merge(file_path, layer1_ref, layer1_content, layer2_ref, layer2_content)` (src/merge/jinmerge.rs:110)
- `JinMergeConflict::write_to_file(&self, merge_path: &Path) -> Result<()>` (src/merge/jinmerge.rs:165)
- `JinMergeConflict::parse_from_file(merge_path: &Path) -> Result<Self>` (src/merge/jinmerge.rs:202)

## 4. Patterns for Collecting Errors vs Aborting

### Pattern 1: Early Termination (Apply Command)
```rust
// src/commands/apply.rs:63-76
if !merged.conflict_files.is_empty() {
    eprintln!("Merge conflicts detected in {} files:", merged.conflict_files.len());
    for path in &merged.conflict_files {
        eprintln!("  - {}", path.display());
    }
    return Err(JinError::Other(format!(
        "Cannot apply due to {} merge conflicts",
        merged.conflict_files.len()
    )));
}
```

### Pattern 2: Collect All Conflicts (Layer Merge)
```rust
// src/merge/layer.rs:110-120
for path in all_paths {
    match merge_file_across_layers(&path, &config.layers, config, repo) {
        Ok(merged) => {
            result.merged_files.insert(path, merged);
        }
        Err(JinError::MergeConflict { .. }) => {
            result.conflict_files.push(path);
        }
        Err(e) => return Err(e),
    }
}
```

## 5. Key File Paths and Line Numbers

### Error Definitions
- `src/core/error.rs:25-27` - `MergeConflict` error type
- `src/core/error.rs:99-104` - Test for merge conflict error

### Text Merge Implementation
- `src/merge/text.rs:26-37` - `TextMergeResult` enum
- `src/merge/text.rs:116-118` - `text_merge()` function
- `src/merge/text.rs:133-160` - `text_merge_with_config()` function
- `src/merge/text.rs:175-177` - `has_conflict_markers()` function
- `src/merge/text.rs:203-275` - `parse_conflicts()` function

### Layer Merge Implementation
- `src/merge/layer.rs:54-65` - `LayerMergeResult` struct
- `src/merge/layer.rs:103-123` - `merge_layers()` function
- `src/merge/layer.rs:110-119` - Conflict collection pattern

### JinMerge Format
- `src/merge/jinmerge.rs:54-80` - `JinMergeConflict` struct
- `src/merge/jinmerge.rs:110-135` - `from_text_merge()` constructor
- `src/merge/jinmerge.rs:165-179` - `write_to_file()` method

### Command Handling
- `src/commands/apply.rs:63-76` - Conflict handling in apply command
- `src/commands/sync.rs:36` - Conflict reference in sync command
