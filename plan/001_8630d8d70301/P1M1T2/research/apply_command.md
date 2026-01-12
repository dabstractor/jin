# Jin Apply Command Research Summary

## 1. Apply Command Implementation Files

**Core Files:**
- `/home/dustin/projects/jin/src/commands/apply.rs` - Main apply command implementation
- `/home/dustin/projects/jin/src/cli/args.rs` - ApplyArgs struct definition
- `/home/dustin/projects/jin/src/cli/mod.rs` - CLI command registration
- `/home/dustin/projects/jin/src/commands/mod.rs` - Command dispatch
- `/home/dustin/projects/jin/src/lib.rs` - Main entry point (run function)
- `/home/dustin/projects/jin/src/main.rs` - CLI entry point

**Supporting Modules:**
- `/home/dustin/projects/jin/src/merge/layer.rs` - Layer merge orchestration
- `/home/dustin/projects/jin/src/merge/mod.rs` - Merge engine exports
- `/home/dustin/projects/jin/src/merge/deep.rs` - Deep merge logic for structured configs
- `/home/dustin/projects/jin/src/merge/text.rs` - 3-way text merge
- `/home/dustin/projects/jin/src/staging/metadata.rs` - Workspace metadata tracking
- `/home/dustin/projects/jin/src/core/error.rs` - Error type definitions

## 2. Command Structure

**CLI Entry Flow:**
```
main.rs -> lib.rs::run() -> commands/mod.rs::execute() -> apply::execute()
```

**ApplyArgs Structure:**
```rust
pub struct ApplyArgs {
    /// Force apply even if workspace is dirty
    pub force: bool,

    /// Show what would be applied
    pub dry_run: bool,
}
```

## 3. Current Conflict Handling (Abort-on-Conflict)

**Abort-on-Conflict Behavior:**
The apply command **immediately aborts** when conflicts are detected:

```rust
// In apply.rs, lines 64-76
if !merged.conflict_files.is_empty() {
    eprintln!(
        "Merge conflicts detected in {} files:",
        merged.conflict_files.len()
    );
    for path in &merged.conflict_files {
        eprintln!("  - {}", path.display());
    }
    return Err(JinError::Other(format!(
        "Cannot apply due to {} merge conflicts",
        merged.conflict_files.len()
    )));
}
```

## 4. Current Error Handling Patterns

**Error Types Used:**
1. `JinError::NotInitialized` - Jin not set up in project
2. `JinError::Other(String)` - Generic error with descriptive message
3. `JinError::MergeConflict { path: String }` - Conflicts during merge
4. `std::io::Error` - File system operations

## 5. Merge Operations Performed

**Layer Merge Process:**
1. Collect applicable layers based on current mode/scope/project context
2. Gather all file paths across layers using `collect_all_file_paths()`
3. Merge each file individually across layers in precedence order
4. Handle conflicts by tracking files that can't be merged cleanly

## 6. State Management

**WorkspaceMetadata Tracking:**
- **Location**: `.jin/workspace/last_applied.json`
- **Content**:
  - RFC3339 timestamp
  - Applied layer names
  - File path -> content hash mappings

## 7. Current Tests

**Test Location:**
- `/home/dustin/projects/jin/tests/cli_basic.rs` - Basic CLI tests

## 8. Key Implementation Details

**Atomic File Writes:**
```rust
// From apply.rs, lines 153-158
let temp_path = path.with_extension("jin-tmp");
std::fs::write(&temp_path, &content)?;
std::fs::rename(&temp_path, path)?;  // Atomic rename
```
