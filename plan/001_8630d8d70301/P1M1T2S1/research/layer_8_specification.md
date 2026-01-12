# Layer 8 (UserLocal) Specification

## From src/core/layer.rs

```rust
/// Layer 8: Machine-only overlays (~/.jin/local/)
UserLocal,
```

## Layer Properties

| Property        | Value                              |
|-----------------|------------------------------------|
| Precedence      | 8 (second highest, only WorkspaceActive is higher) |
| Storage Path    | `~/.jin/local/`                    |
| Git Ref         | `refs/jin/layers/local`            |
| Requires Mode   | No                                 |
| Requires Scope  | No                                 |
| Project-Specific| No                                 |

## Semantics

- **Independence**: UserLocal is independent of mode, scope, and project context
- **Machine-Specific**: Contains configuration that should only apply to the current machine
- **Override Behavior**: Overrides all layers (1-7) except WorkspaceActive (9)
- **Storage**: Stored outside the project workspace (at `~/.jin/local/`)

## Use Cases

1. Local development environment overrides
2. Machine-specific paths (e.g., `/Users/` vs `/home/`)
3. Local database connections
4. Developer-specific tool configurations
5. Temporary development overrides

## Display Name

From `src/commands/add.rs` line 224:
```rust
Layer::UserLocal => "user-local".to_string(),
```

## Integration Points

1. **Layer enum**: Already defined in `src/core/layer.rs` (line 28)
2. **precedence()**: Returns 8 (line 44)
3. **ref_path()**: Returns "refs/jin/layers/local" (line 93)
4. **storage_path()**: Returns "~/.jin/local/" (line 126)
5. **requires_mode()**: Returns false (not in matches)
6. **requires_scope()**: Returns false (not in matches)
7. **is_project_specific()**: Returns false (not in matches)

## No Additional Implementation Needed

The Layer enum already supports UserLocal completely. This subtask only adds the routing option flag.
