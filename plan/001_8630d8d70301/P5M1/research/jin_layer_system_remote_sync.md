# Jin Layer System Remote Sync Analysis

## Overview

This document analyzes Jin's layer system architecture and its implications for implementing remote synchronization operations (fetch, pull, push, sync).

## Jin Architecture Key Points

### 1. Bare Repository Structure

- **Location**: `~/.jin/` (default, configurable)
- **Type**: Bare Git repository (no working directory)
- **Purpose**: Stores all layer configurations as Git objects and refs
- **Initialized by**: `JinRepo::open_or_create()` in `src/git/repo.rs`

### 2. Layer Reference Model

Jin uses Git refs under a custom namespace to represent layers:

```
refs/jin/layers/global                                # Layer 1: Global Base
refs/jin/layers/mode/<mode>                          # Layer 2: Mode Base
refs/jin/layers/mode/<mode>/scope/<scope>            # Layer 3: Mode → Scope
refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>  # Layer 4: Mode → Scope → Project
refs/jin/layers/mode/<mode>/project/<project>        # Layer 5: Mode → Project
refs/jin/layers/scope/<scope>                        # Layer 6: Scope Base
refs/jin/layers/project/<project>                    # Layer 7: Project Base
refs/jin/layers/local                                # Layer 8: User Local
refs/jin/layers/workspace                            # Layer 9: Workspace Active
```

**Key Implementation**: See `src/core/layer.rs::Layer::ref_path()`

### 3. Remote Configuration

From `src/core/config.rs`:

```rust
pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}
```

Stored in `~/.jin/config.toml` (global configuration).

### 4. Custom Refspec

From `src/commands/link.rs:50`:

```rust
repo.remote_with_fetch("origin", &args.url, "+refs/jin/layers/*:refs/jin/layers/*")?;
```

**Refspec Breakdown**:
- `+` prefix: Force update (non-fast-forward allowed)
- `refs/jin/layers/*`: Source pattern (remote refs)
- `:refs/jin/layers/*`: Destination pattern (local refs)
- Matches ALL layer refs, preserving hierarchy

## Remote Sync Implications

### Fetch Operation

**Purpose**: Download layer refs from remote to local bare repository

**What to sync**:
1. All `refs/jin/layers/*` refs from remote
2. Associated Git objects (blobs, trees, commits)
3. NO workspace files (bare repo only)

**Key considerations**:
- Fetch is read-only (safe operation)
- Does NOT modify working directory
- Does NOT change active context
- Should notify user if updates affect active mode/scope/project

**Implementation pattern** (from `link.rs`):
```rust
let mut remote = repo.find_remote("origin")?;
remote.connect(Direction::Fetch)?;
// Fetch with custom refspec
remote.fetch(&["refs/jin/layers/*"], Some(&mut fetch_options), None)?;
remote.disconnect()?;
```

### Pull Operation

**Purpose**: Fetch + merge updates into layers

**Sequence**:
1. `jin fetch` (download refs)
2. Detect which layers have updates
3. For each updated layer:
   - Check if local has uncommitted changes
   - Merge remote changes using Jin's merge engine
   - Update local ref to point to merge result

**Constraints** (from PRD §14):
- Requires clean state (no uncommitted changes)
- Conflicts must be resolved before completion
- Atomic operation (all layers updated or none)

**Key difference from Git pull**:
- Multiple layers may be updated simultaneously
- Uses Jin's deep merge engine (not Git's merge)
- Merges config files (JSON, YAML, TOML) with special semantics

### Push Operation

**Purpose**: Upload local layer refs to remote

**Requirements** (from PRD §14):
1. `jin fetch` must be run first
2. Clean merge state required
3. No unresolved conflicts

**What to push**:
- Mode layers (if mode is active or modified)
- Scope layers (if scope is active or modified)
- Project-specific layers for current project
- Global layers (if modified)
- User local layer is NEVER pushed (machine-specific)

**Refspec strategy**:
```
Push only modified refs:
refs/jin/layers/mode/<active-mode>:refs/jin/layers/mode/<active-mode>
refs/jin/layers/scope/<active-scope>:refs/jin/layers/scope/<active-scope>
refs/jin/layers/project/<current-project>:refs/jin/layers/project/<current-project>
```

**Force push considerations**:
- Normal push: Fast-forward only
- `--force` flag: Allow non-fast-forward (dangerous)
- Must verify remote state before force push

### Sync Operation

**Purpose**: Comprehensive synchronization

**Sequence**:
1. `jin fetch` - Download remote refs
2. `jin pull` - Merge updates
3. `jin apply` - Regenerate workspace files

**Use case**: Quick update workflow for developers

## Layer Sync Rules

### Layers That Sync

| Layer | Sync to Remote? | Notes |
|-------|----------------|--------|
| Global Base | ✅ Yes | Shared defaults for all users |
| Mode Base | ✅ Yes | Mode-specific configs |
| Mode → Scope | ✅ Yes | Scoped mode configs |
| Mode → Scope → Project | ✅ Yes | Project overrides for scoped mode |
| Mode → Project | ✅ Yes | Project overrides for mode |
| Scope Base | ✅ Yes | Untethered scope configs |
| Project Base | ✅ Yes | Project-only configs |
| User Local | ❌ NO | Machine-specific, never synced |
| Workspace Active | ❌ NO | Derived, regenerated by `apply` |

### Update Notifications

From PRD §14, after `jin fetch`:

```
Updates available for:
  - mode/claude (3 files changed)
  - scope/language:javascript (1 file changed)
  - project/ui-dashboard (2 files changed)

Run 'jin pull' to merge updates
```

## Testing Considerations

### Network Operations Testing

1. **Mock remote repository**
   - Create temp bare repo for tests
   - Use `file://` URLs for local testing

2. **Authentication testing**
   - SSH key handling
   - Credential callbacks
   - Permission errors

3. **Conflict scenarios**
   - Divergent refs
   - Non-fast-forward pushes
   - Merge conflicts in config files

4. **Error handling**
   - Network failures
   - Authentication failures
   - Repository not found
   - Permission denied

## Related Files

- `src/commands/link.rs` - Remote setup and connectivity testing
- `src/git/repo.rs` - JinRepo wrapper
- `src/git/refs.rs` - Reference operations
- `src/core/config.rs` - RemoteConfig structure
- `src/core/layer.rs` - Layer hierarchy and ref paths
- `src/merge/layer.rs` - Layer merge orchestration

## Key Dependencies

- **git2-rs**: Rust bindings to libgit2
- **RemoteCallbacks**: Authentication, progress reporting
- **FetchOptions**: Fetch behavior configuration
- **PushOptions**: Push behavior configuration

## Security Considerations

1. **Authentication**
   - Respect SSH agent
   - Support SSH keys
   - Handle credential prompts (if interactive)

2. **Force push protection**
   - Warn before force push
   - Require explicit `--force` flag
   - Never force push to main/master equivalents

3. **Network safety**
   - Validate remote URLs (already done in `link.rs`)
   - Handle timeouts gracefully
   - Retry transient failures

## Implementation Strategy

1. **Fetch**: Straightforward - use existing refspec, add progress reporting
2. **Pull**: Complex - requires merge integration with Jin's deep merge engine
3. **Push**: Medium - detect modified layers, push relevant refs
4. **Sync**: Orchestrator - calls fetch + pull + apply in sequence

## Questions to Resolve

1. What happens if remote has layers that don't exist locally?
   - Answer: Fetch them and notify user

2. What if local has layers that don't exist on remote?
   - Answer: Push creates them (if user has permission)

3. How to handle project auto-detection during sync?
   - Answer: Use existing `ProjectContext` from `.jin/context`

4. Should fetch be automatic before other operations?
   - Answer: No for fetch/push, yes for sync
