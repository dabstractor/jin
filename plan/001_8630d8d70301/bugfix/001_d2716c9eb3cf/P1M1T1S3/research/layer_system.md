# Layer System Research

## Jin 9-Layer Hierarchy

| Layer Number | Layer Name          | Description                                 | Precedence |
|--------------|---------------------|---------------------------------------------|------------|
| 1            | GlobalBase          | `jin/global/` - Shared defaults             | Lowest     |
| 2            | ModeBase            | `jin/mode/<mode>/` - Mode defaults          | 2          |
| 3            | ModeScope           | `jin/mode/<mode>/scope/<scope>/`            | 3          |
| 4            | ModeScopeProject    | Mode + Scope + Project combination          | 4          |
| 5            | ModeProject         | `jin/mode/<mode>/project/<project>/`        | 5          |
| 6            | ScopeBase           | `jin/scope/<scope>/` - Untethered scope     | 6          |
| 7            | ProjectBase         | `jin/project/<project>/` - Project-only     | 7          |
| 8            | UserLocal           | `~/.jin/local/` - Machine-specific          | 8          |
| 9            | WorkspaceActive     | `.jin/workspace/` - Derived merge result    | Highest    |

## Key Source Files

### Layer Definition
**File**: `/home/dustin/projects/jin/src/core/layer.rs`

```rust
// Line 35-46: Precedence values
pub fn precedence(&self) -> u8 {
    match self {
        Layer::GlobalBase => 1,
        Layer::ModeBase => 2,
        Layer::ModeScope => 3,
        Layer::ModeScopeProject => 4,
        Layer::ModeProject => 5,
        Layer::ScopeBase => 6,
        Layer::ProjectBase => 7,
        Layer::UserLocal => 8,
        Layer::WorkspaceActive => 9,
    }
}
```

### Git Ref Paths
```rust
// ModeBase uses /_ suffix to avoid conflict with child refs
Layer::ModeBase => format!("refs/jin/layers/mode/{}/_", mode.unwrap_or("default"))

// ProjectBase has no /_ suffix (no children)
Layer::ProjectBase => format!("refs/jin/layers/project/{}", project.unwrap_or("default"))
```

## Bug Context

The bug occurs when ModeBase (Layer 2) and ProjectBase (Layer 7) both have JSON content that should deep merge:

- **ModeBase content**: `{"common": {"a": 1}, "mode": true}`
- **ProjectBase content**: `{"common": {"a": 1, "b": 2}, "project": false}`
- **Expected merged result**: `{"common": {"a": 1, "b": 2}, "mode": true, "project": false}`

ProjectBase has higher precedence (7 > 2), so its values win when keys conflict at the same level.
