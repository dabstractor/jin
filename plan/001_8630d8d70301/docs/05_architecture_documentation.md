# Research: Architecture Documentation

## Fix Specifications

**File**: `plan/docs/fix_specifications.md`

### Fix 3: Mode Switching UX - Option A: Auto-Clear Metadata

The architecture specifies:

> "In ModeAction::Use handler:
> After activating mode, check if workspace metadata references different mode"

This same pattern applies to scope switching:

> "In ScopeAction::Use handler:
> After activating scope, check if workspace metadata references different scope"

## Architecture for Scope Metadata Clearing

### Core Pattern

- **File Location**: `src/commands/scope.rs`
- **Handler**: ScopeAction::Use handler (function: `use_scope()`)
- **Metadata Loading**: Load `WorkspaceMetadata` as `Option<WorkspaceMetadata>` after `context.save()`
- **Scope Comparison**: Extract scope from `applied_layers`, compare with new scope
- **File Deletion**: Use `std::fs::remove_file()` with existence check
- **User Messaging**: Clear informational message about metadata clearing

### Scope Extraction Pattern

```rust
fn get_metadata_scope(metadata: &WorkspaceMetadata) -> Option<String> {
    metadata.applied_layers
        .iter()
        .find(|layer| layer.starts_with("scope/") && !layer.starts_with("scope//"))
        .and_then(|layer| {
            layer
                .strip_prefix("scope/")
                .and_then(|s| s.split('/').next())
                .map(String::from)
        })
}
```

### User Messaging Format

```
Cleared workspace metadata (scope changed from '{old}' to '{new}').
Run 'jin apply' to apply new scope configuration.
```

## Integration Points

### With Mode System (P1.M3.T1.S2)

- P1.M3.T1.S2 implements mode metadata clearing
- P1.M3.T2.S1 implements scope metadata clearing
- Both use same `WorkspaceMetadata::load()` method
- Both operate on same metadata file (`last_applied.json`)

### With Layer System

Applied layers format:
- `"global"` - No mode/scope
- `"mode/claude"` - Mode layer
- `"mode/production/scope/backend"` - Mode + scope
- `"scope/backend"` - Scope-only layer
- `"scope/frontend/api"` - Scope with sub-path

## Edge Cases

1. **No scope layer in metadata**: Clear metadata when switching to a scope
2. **Fresh workspace (no metadata)**: Do nothing, no error
3. **Same scope switch**: Don't clear metadata
4. **Scope layer extraction**: Use first component after "scope/" prefix

## Architectural Decisions

1. **Scope inference**: No dedicated field - parse from `applied_layers`
2. **Safe approach**: Clear metadata when transitioning from no-scope to scope
3. **File deletion**: Follow codebase pattern (check exists() first, then remove)
4. **Error handling**: Propagate all errors except `NotFound` (graceful)
5. **User experience**: Clear messaging before activation message

## Future Work

- **P1.M3.T3.S1**: Integration tests for mode/scope switching workflow
- Tests will verify both mode and scope metadata clearing work correctly
