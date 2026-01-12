# Research: File Deletion Patterns in Codebase

## Library Used

**`std::fs`** - Standard library filesystem module (synchronous operations)

No async file operations (`tokio::fs`, `async_std::fs`) are used in this codebase.

## Existing Deletion Patterns

### Pattern 1: Direct Deletion with `?` Operator
**File**: `src/commands/rm.rs:175`
```rust
if args.force && path.exists() {
    std::fs::remove_file(path)?;
}
```

### Pattern 2: Existence Check Before Deletion
**File**: `src/git/transaction.rs:335-345`
```rust
pub fn delete_at(path: PathBuf) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    // Also clean up any stale temp file
    let temp_path = path.with_extension("tmp");
    if temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }
    Ok(())
}
```

### Pattern 3: Custom Error Message
**File**: `src/commands/resolve.rs:149-150`
```rust
std::fs::remove_file(&merge_path)
    .map_err(|e| JinError::Other(format!("Failed to delete .jinmerge file: {}", e)))?;
```

### Pattern 4: Conditional Deletion with Error
**File**: `src/commands/resolve.rs:280-281`
```rust
if state_path.exists() {
    std::fs::remove_file(&state_path)
        .map_err(|e| JinError::Other(format!("Failed to remove paused state: {}", e)))?;
}
```

## Error Handling Patterns

### Pattern A: Direct Error Propagation
```rust
std::fs::remove_file(path)?;
// JinError has #[from] std::io::Error, so ? auto-converts to JinError::Io
```

### Pattern B: Custom Error Wrapper
```rust
std::fs::remove_file(path)
    .map_err(|e| JinError::Other(format!("Failed to delete {}: {}", path.display(), e)))?;
```

## Imports Required

```rust
use crate::core::{JinError, Result};
use std::path::{Path, PathBuf};
```

## Recommended Pattern for This Work Item

Based on codebase conventions, use the pattern from `transaction.rs`:

```rust
let metadata_path = WorkspaceMetadata::default_path();
if metadata_path.exists() {
    std::fs::remove_file(&metadata_path)?;
    println!("Cleared workspace metadata (scope changed from '{}' to '{}').", old_scope, new_scope);
    println!("Run 'jin apply' to apply new scope configuration.");
}
```

## Key Observations

1. **Synchronous only**: All file operations are synchronous
2. **Existence check**: Codebase consistently checks `path.exists()` before deletion
3. **Error conversion**: `std::io::Error` auto-converts to `JinError::Io` via `#[from]`
4. **Path handling**: Uses `std::path::Path` and `PathBuf` for cross-platform compatibility
5. **Atomic writes**: Uses temp file + rename pattern for write operations
