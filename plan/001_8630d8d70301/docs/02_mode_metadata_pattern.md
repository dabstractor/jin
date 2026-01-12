# Research: Mode Metadata Clearing Pattern from P1.M3.T1.S2

## File Location
**Path**: `/home/dustin/projects/jin/src/commands/mode.rs`
**Function**: `use_mode()` (lines 86-130)

## Pattern from P1.M3.T1.S1 (Metadata Loading)

```rust
// Save context
context.save()?;

// Load workspace metadata (may not exist yet)
let _metadata = match WorkspaceMetadata::load() {
    Ok(meta) => Some(meta),
    Err(JinError::NotFound(_)) => None,
    Err(e) => return Err(e),
};

println!("Activated mode '{}'", name);
println!("Stage files with: jin add --mode");
```

## Pattern from P1.M3.T1.S2 (Mode Comparison and Clearing)

The pattern from the PRP specifies:

```rust
// Load workspace metadata (may not exist yet)
let metadata = match WorkspaceMetadata::load() {
    Ok(meta) => Some(meta),
    Err(JinError::NotFound(_)) => None,
    Err(e) => return Err(e),
};

// Extract mode from metadata if present
if let Some(meta) = &metadata {
    // Find mode layer in applied_layers (format: "mode/{name}")
    let metadata_mode = meta.applied_layers
        .iter()
        .find(|layer| layer.starts_with("mode/"))
        .and_then(|layer| layer.strip_prefix("mode/"))
        .and_then(|s| s.split('/').next());

    // Compare with new mode
    if let Some(old_mode) = metadata_mode {
        if old_mode != name {
            // Modes differ - clear metadata to prevent detached state
            let metadata_path = WorkspaceMetadata::default_path();
            if metadata_path.exists() {
                std::fs::remove_file(&metadata_path)?;
                println!("Cleared workspace metadata (mode changed from '{}' to '{}').", old_mode, name);
                println!("Run 'jin apply' to apply new mode configuration.");
            }
        }
    } else {
        // No mode layer in metadata (only global layers)
        // Clear metadata since we're now activating a mode
        let metadata_path = WorkspaceMetadata::default_path();
        if metadata_path.exists() {
            std::fs::remove_file(&metadata_path)?;
            println!("Cleared workspace metadata (activating mode '{}').", name);
            println!("Run 'jin apply' to apply new mode configuration.");
        }
    }
}
```

## User Message Format

```rust
// When modes differ:
println!("Cleared workspace metadata (mode changed from '{}' to '{}').", old_mode, name);
println!("Run 'jin apply' to apply new mode configuration.");

// When no mode layer existed:
println!("Cleared workspace metadata (activating mode '{}').", name);
println!("Run 'jin apply' to apply new mode configuration.");
```

## Key Pattern Elements

1. **Metadata Loading**: Handle `NotFound` as `None`, propagate other errors
2. **Mode Extraction**: Find layer starting with `"mode/"` in `applied_layers`
3. **Scope Handling**: Use `split('/').next()` to get first component after prefix
4. **Comparison**: Compare extracted mode with new mode name
5. **File Deletion**: Check `exists()` first, then `remove_file()` with `?` operator
6. **User Messaging**: Print BEFORE activation message, use single quotes

## Edge Cases Handled

- Fresh workspace (no metadata) → `None` variant, no action
- No mode layer in metadata → Clear and activate new mode
- Same mode switch → No clearing needed
- File deletion errors → Propagate via `?` operator

## Imports Required

```rust
use crate::staging::metadata::WorkspaceMetadata;
```

Note: This import should already be present from P1.M3.T1.S1.
