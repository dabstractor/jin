# Rust Staging Patterns Research

## Collection Types for StagingIndex

**Use IndexMap** (already in dependencies):
- O(1) operations with insertion order
- Preserves order like Git index
- Better than HashMap (no order) or BTreeMap (sorted, not insertion-ordered)

```rust
use indexmap::IndexMap;
use std::path::PathBuf;

pub struct StagingIndex {
    entries: IndexMap<PathBuf, StagedEntry>,
    // Secondary indexes for queries
    by_layer: HashMap<Layer, Vec<PathBuf>>,
    by_status: HashMap<FileStatus, Vec<PathBuf>>,
}
```

## File Status Tracking

Use bitflags for efficient status representation:

```rust
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FileStatus: u8 {
        const CLEAN    = 0b00000001;
        const MODIFIED = 0b00000010;
        const STAGED   = 0b00000100;
        const REMOVED  = 0b00001000;
        const NEW      = 0b00010000;
    }
}
```

## StagedEntry Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedEntry {
    pub path: PathBuf,
    pub layer: Layer,
    pub content_hash: Vec<u8>,  // SHA-256
    pub status: FileStatus,
    pub staged_at: Option<SystemTime>,
    pub size: u64,
}
```

## Path Handling

- Use `PathBuf` for owned paths, `&Path` for borrowed
- Normalize paths consistently: remove `.` and `..` components
- Use `normalize_path()` before inserting into index

## Persistence Patterns

**Binary format** (like Git):
- Compact and fast
- Custom byte order for serialization

**JSON format** (for debugging):
- Human-readable
- Easy to inspect during development

## References

- https://docs.rs/indexmap/latest/indexmap/
- https://docs.rs/serde/latest/serde/
