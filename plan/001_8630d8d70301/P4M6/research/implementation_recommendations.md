# Implementation Recommendations for P4M6

## Overview

This document provides specific implementation recommendations based on research of similar tools and Jin's existing architecture.

---

## 1. Command Implementation Order

Recommended sequence for implementing remote configuration commands:

### Phase 1: Foundation (P4M6a)
1. **jin link** - Store remote URL, auto-fetch available modes/scopes
2. **jin fetch** - Pull mode/scope/project data from remote (no merge)
3. Update config storage if needed

### Phase 2: Synchronization (P4M6b)
4. **jin pull** - Merge fetched data; pause on conflicts
5. **jin push** - Push local changes to remote
6. **jin sync** - fetch + pull + apply (convenience command)

### Phase 3: Polish (P4M6c)
7. Update documentation and examples
8. Add tests for conflict scenarios
9. Add retry/recovery logic

---

## 2. jin link Implementation

### Command Signature
```bash
jin link <url> [--fetch]
# or
jin link <url> [--no-fetch]   # For explicit control
```

### Behavior

1. **Parse URL**: Validate Git repository URL
   - SSH: `git@github.com:org/repo.git` ✅
   - HTTPS: `https://github.com/org/repo.git` ✅
   - Local paths: (optional) `/path/to/repo` ⚠️

2. **Check existing remote**:
   ```
   If ~/.jin/config.toml already has remote:
     - Prompt: "Replace existing remote: git@old/url? (y/n)"
     - If yes: update
     - If no: abort
   ```

3. **Store URL globally**:
   ```toml
   # ~/.jin/config.toml
   [remote]
   url = "git@github.com:myorg/jin-config"
   fetch_on_init = true
   ```

4. **Auto-fetch** (recommended):
   ```
   jin link <url>
   # Fetches available modes/scopes from remote
   # Outputs:
   #   Available modes: claude, cursor, zed
   #   Available scopes: language:javascript, language:python, infra:docker
   ```

5. **If fetch fails**:
   - Still store the URL
   - Warn: "Could not fetch from remote; will try on next `jin fetch`"
   - Allow user to proceed (might be network issue)

### Error Handling

```
✗ Invalid URL format
✗ Remote not accessible (but store URL anyway; warn user)
✗ Remote is not a valid Jin config repo (no jin/ directories found; warn but continue)
✗ Cannot write to ~/.jin/config.toml (permission error; fail)
```

### Code Structure

```rust
// src/commands/link.rs
pub fn execute(args: LinkArgs) -> Result<()> {
    // 1. Validate URL
    validate_git_url(&args.url)?;

    // 2. Check existing remote
    let config = JinConfig::load()?;
    if let Some(existing) = &config.remote {
        if !confirm_replace_remote(&existing.url)? {
            return Ok(()); // User cancelled
        }
    }

    // 3. Update config
    let mut config = JinConfig::load()?;
    config.remote = Some(RemoteConfig {
        url: args.url.clone(),
        fetch_on_init: true,
    });
    config.save()?;

    // 4. Auto-fetch (or skip if --no-fetch)
    if !args.no_fetch {
        match fetch_remote(&args.url) {
            Ok(modes_scopes) => {
                println!("Available modes: {}", modes_scopes.modes.join(", "));
                println!("Available scopes: {}", modes_scopes.scopes.join(", "));
            }
            Err(e) => {
                eprintln!("Warning: Could not fetch from remote: {}", e);
            }
        }
    }

    Ok(())
}
```

---

## 3. jin fetch Implementation

### Command Signature
```bash
jin fetch                    # Fetch from remote
jin fetch --remote <url>     # Override default remote
```

### Behavior

1. **Read remote URL** from `~/.jin/config.toml`
2. **Verify project is initialized** (`.jin/context` exists)
3. **Fetch all refs** from remote:
   - `refs/jin/mode/*`
   - `refs/jin/scope/*`
   - `refs/jin/project/<project>/*`
4. **Show what's new**:
   ```
   Fetched from git@github.com:myorg/jin-config
   New: mode/cursor
   Updated: mode/claude (commit abc123)
   Updated: scope/language:python (commit def456)
   ```
5. **Cache the fetch** (for `jin pull` to reference)

### Integration with Existing Code

```rust
// src/commands/fetch.rs
pub fn execute() -> Result<()> {
    // 1. Load config
    let config = JinConfig::load()?;
    let remote_url = config.remote
        .ok_or(JinError::NoRemote)?
        .url;

    // 2. Verify initialized
    let _context = ProjectContext::load()?;

    // 3. Fetch from Git
    let fetched = git::fetch_refs(&remote_url)?;

    // 4. Show what was fetched
    for (ref_name, commit) in &fetched {
        println!("Updated: {} ({})", ref_name, commit);
    }

    // 5. Store fetch state for `jin pull` to reference
    store_fetch_state(&fetched)?;

    Ok(())
}
```

---

## 4. jin pull Implementation

### Command Signature
```bash
jin pull                     # Fetch + merge (requires clean state)
jin pull --no-fetch          # Merge only (assumes prior fetch)
```

### Behavior

1. **Require clean state**: No staged changes, no workspace conflicts
   ```
   ✗ Cannot pull with staged changes; commit or reset first
   ✗ Cannot pull with workspace conflicts; resolve with jin add + jin commit
   ```

2. **Auto-fetch if not done**: Run `jin fetch` if needed
   ```
   No prior fetch found; fetching from remote...
   ```

3. **Merge fetched refs into layers**:
   - For each fetched ref, merge into corresponding layer
   - Use existing merge logic from `jin apply` / layer system
   - If conflict: pause, create `.jinmerge` files, return error

4. **Show merge result**:
   ```
   Merged from remote (3 layers updated):
   - mode/claude (3 files changed)
   - scope/language:javascript (1 file changed)
   - project/ui-dashboard (2 files changed)

   Conflicts in: .claude/config.json
   Run: jin add <resolved-file> && jin commit
   ```

5. **Keep workspace in sync** (optional):
   - After successful merge, optionally re-apply layers to workspace
   - Or wait for explicit `jin apply`

### Conflict Handling

```
If conflicts during merge:
  1. Create .jinmerge files (Git-style conflict markers)
  2. Pause merge operation
  3. Print: "Conflicts found. Resolve and run: jin add <file> && jin commit"
  4. On next jin commit, verify all conflicts resolved
  5. Complete the merge as part of commit
```

### Code Structure

```rust
// src/commands/pull.rs
pub fn execute(args: PullArgs) -> Result<()> {
    // 1. Check clean state
    let workspace_status = check_workspace_clean()?;
    if !workspace_status.is_clean {
        return Err(JinError::DirtyStagingArea);
    }

    // 2. Fetch if needed
    if !args.no_fetch {
        fetch::execute()?;
    }

    // 3. Load fetched state
    let fetched_refs = load_fetch_state()?;

    // 4. Merge each ref into layer
    let context = ProjectContext::load()?;
    for (layer_ref, remote_commit) in fetched_refs {
        match merge_into_layer(&context, &layer_ref, &remote_commit) {
            Ok(_) => println!("Merged: {}", layer_ref),
            Err(JinError::MergeConflict { .. }) => {
                create_jinmerge_files(&context)?;
                return Err(JinError::MergeConflict {
                    message: "Conflicts found; resolve and commit".into()
                });
            }
            Err(e) => return Err(e),
        }
    }

    // 5. Re-apply to workspace (optional)
    if !args.no_apply {
        apply::execute(&ApplyArgs { .. })?;
    }

    Ok(())
}
```

---

## 5. jin push Implementation

### Command Signature
```bash
jin push                     # Push to remote
jin push --force             # Force push (only if necessary)
```

### Behavior

1. **Require prior fetch**: Prevent divergence
   ```
   ✗ Must run jin fetch before jin push (to avoid divergence)
   ```

2. **Require clean merge state**: No conflicts or unresolved merges
   ```
   ✗ Cannot push with unresolved merge conflicts
   ```

3. **Push only affected layers**:
   - Determine which layers changed since last push
   - Push only those refs to remote
   - Example: `git push origin refs/jin/mode/claude:refs/jin/mode/claude`

4. **Show what was pushed**:
   ```
   Pushed to git@github.com:myorg/jin-config:
   - mode/claude
   - project/ui-dashboard (project-specific override)
   ```

### Error Cases

```
✗ No remote configured; run jin link <url> first
✗ Remote unreachable
✗ Permission denied (no write access to remote)
✗ Divergence detected (local != remote base); run jin fetch first
✗ Unresolved conflicts; cannot push
```

### Code Structure

```rust
// src/commands/push.rs
pub fn execute(args: PushArgs) -> Result<()> {
    // 1. Load config and context
    let config = JinConfig::load()?;
    let remote_url = config.remote
        .ok_or(JinError::NoRemote)?
        .url;
    let context = ProjectContext::load()?;

    // 2. Check for prior fetch
    if !has_fetched_recently()? {
        return Err(JinError::MustFetchFirst);
    }

    // 3. Check clean merge state
    if has_merge_conflicts()? {
        return Err(JinError::UnresolvedConflicts);
    }

    // 4. Determine changed layers
    let changed_layers = detect_changed_layers(&context)?;

    // 5. Push each layer
    for layer in &changed_layers {
        git::push_ref(&remote_url, &layer.git_ref())?;
        println!("Pushed: {}", layer);
    }

    Ok(())
}
```

---

## 6. Configuration Changes Needed

### Current Config Structure (Already Exists)

```toml
# ~/.jin/config.toml
version = 1

[remote]
url = "git@github.com:myorg/jin-config"
fetch_on_init = true

[user]
name = "Jane Developer"
email = "jane@example.com"
```

### Recommended Additions (For Later Phases)

```toml
# Optional in v2
[remote]
url = "git@github.com:myorg/jin-config"
fetch_on_init = true
fetch_on_enter = false          # Auto-fetch on cd to project?
branch = "master"               # Which branch to track?

[sync]
auto_pull = false               # Auto-pull on jin apply?
push_before_exit = false        # Warn if unsynced on exit?
```

### Per-Project Context (Already Exists)

```yaml
# .jin/context
version: 1
project: ui-dashboard
mode: claude
scope: language:javascript
last_updated: "2025-01-15T10:30:00Z"
```

### New: Fetch State File (Internal)

```yaml
# .jin/.fetch_state (internal, auto-generated)
version: 1
last_fetch: "2025-01-15T10:30:00Z"
fetched_refs:
  "refs/jin/mode/claude": "abc123def456"
  "refs/jin/scope/language:javascript": "def456ghi789"
  "refs/jin/project/ui-dashboard": "ghi789jkl012"
```

---

## 7. Testing Strategy

### Unit Tests
- [ ] Valid/invalid URL parsing
- [ ] Config save/load
- [ ] Fetch state tracking
- [ ] Layer detection from fetched refs

### Integration Tests
- [ ] `jin link` → stores URL globally
- [ ] `jin fetch` → fetches all refs, shows what's available
- [ ] `jin pull` → merges without conflict
- [ ] `jin pull` → pauses on conflict, creates `.jinmerge`
- [ ] `jin add` after conflict → marks resolved
- [ ] `jin commit` → completes merge
- [ ] `jin push` → requires prior fetch (fail without it)
- [ ] Multiple projects → same remote URL, different scopes

### Scenario Tests
- [ ] Developer switches teams (re-link)
- [ ] Re-link to same URL (no-op)
- [ ] Network timeout during fetch (store what's possible, warn)
- [ ] Permission denied on push (clear error message)

---

## 8. Documentation Recommendations

### User-Facing Docs

**Command: jin link**
```
NAME
    jin link - Connect project to shared remote configuration

SYNOPSIS
    jin link <URL> [--no-fetch]

DESCRIPTION
    Stores the remote configuration repository URL in ~/.jin/config.toml
    and fetches available modes and scopes.

EXAMPLES
    $ jin link git@github.com:myorg/jin-config
    Available modes: claude, cursor
    Available scopes: language:javascript, infra:docker

    $ jin link https://github.com/myorg/jin-config

    $ jin link file:///path/to/local/repo --no-fetch
```

**Command: jin fetch**
```
NAME
    jin fetch - Download latest configuration from remote

SYNOPSIS
    jin fetch [--remote <URL>]

DESCRIPTION
    Fetches all modes, scopes, and project configuration from the
    remote repository. This does not merge changes; use jin pull for that.
```

**Workflow: Update Configuration**
```
WORKFLOW: Update configuration from team

1. jin fetch              # Check for updates
2. jin diff              # Review what changed
3. jin pull              # Merge updates (may pause on conflicts)
4. Resolve conflicts if needed
5. jin apply             # Apply to workspace
```

### Developer Docs

**Remote Configuration Architecture**

```
~/.jin/config.toml          # Global: remote URL, user info, defaults
.jin/context                # Per-project: active mode, scope, project
.jin/.fetch_state           # Internal: tracks what was fetched
.jin/staging/               # Per-project: staged changes
.jin/workspace/             # Per-project: merged + applied files

Remote Repository (git@github.com:myorg/jin-config)
├── jin/
│   ├── global/             # Shared defaults
│   ├── mode/
│   │   ├── claude/         # Mode config (Layer 2)
│   │   │   ├── scope/
│   │   │   │   └── language:javascript/  # Mode+Scope (Layer 3)
│   │   │   └── project/
│   │   │       └── ui-dashboard/        # Mode+Project (Layer 5)
│   │   └── cursor/
│   ├── scope/
│   │   └── language:javascript/         # Untethered scope (Layer 6)
│   └── project/
│       └── ui-dashboard/                # Project base (Layer 7)
```

---

## 9. Compatibility with Existing Code

### Existing Components Used

- **Config System** (`src/core/config.rs`):
  - `JinConfig` with `remote` field ✅ (already defined)
  - `ProjectContext` for per-project state ✅ (already exists)

- **Git Operations** (need to check/extend):
  - `git2-rs` for fetch operations
  - Refs under `refs/jin/...` namespace
  - Update Git operations to support remote fetching

- **Merge Logic** (reuse from `jin apply`):
  - 9-layer hierarchy
  - Merge engine
  - Conflict detection

- **Error Handling** (`src/core/error.rs`):
  - Add `NoRemote` error
  - Add `MustFetchFirst` error
  - Add `DivergenceDetected` error

### New Error Types Needed

```rust
pub enum JinError {
    // ... existing ...
    NoRemote,
    NoRemoteUrl,
    InvalidRemoteUrl,
    RemoteUnreachable { url: String, reason: String },
    MustFetchFirst,
    DivergenceDetected { local: String, remote: String },
    UnresolvedConflicts,
    FetchFailed { reason: String },
    MergeConflictDuringPull { files: Vec<String> },
}
```

---

## 10. Implementation Sequence & Estimated Effort

### Phase P4M6a: Link + Fetch (Week 1)
1. Implement `jin link` command
   - Parse URL
   - Update config
   - Auto-fetch (call fetch internally)
   - Est: 4 hours

2. Implement `jin fetch` command
   - Read config
   - Fetch refs from remote
   - Show available modes/scopes
   - Est: 6 hours

3. Tests + Documentation
   - Unit tests for both commands
   - User documentation
   - Est: 4 hours

**Total P4M6a: ~14 hours**

### Phase P4M6b: Pull + Push (Week 2)
1. Implement `jin pull` command
   - Merge logic (reuse from apply)
   - Conflict handling
   - `.jinmerge` file creation
   - Est: 8 hours

2. Implement `jin push` command
   - Fetch-first check
   - Changed layer detection
   - Push refs
   - Est: 6 hours

3. Tests + Documentation
   - Integration tests (multi-command workflows)
   - Error case coverage
   - Est: 6 hours

**Total P4M6b: ~20 hours**

### Phase P4M6c: Polish (Week 3)
1. Retry + Recovery logic
   - Handle network timeouts
   - Resume interrupted fetch/push
   - Est: 4 hours

2. Performance optimization
   - Avoid re-fetching same refs
   - Cache remote state locally
   - Est: 2 hours

3. Extended documentation + examples
   - Workflow guides
   - Troubleshooting
   - Security best practices
   - Est: 4 hours

**Total P4M6c: ~10 hours**

---

## 11. Success Criteria

- [ ] `jin link <url>` stores URL globally and auto-fetches
- [ ] `jin fetch` downloads modes/scopes, shows what's available
- [ ] `jin pull` merges without breaking existing `jin apply` logic
- [ ] `jin pull` pauses on conflicts and creates `.jinmerge` files
- [ ] `jin push` requires prior fetch (prevents divergence)
- [ ] Multiple projects can share same remote with different scopes
- [ ] Error messages guide users toward resolution
- [ ] Integration tests pass (no regressions)
- [ ] Documentation is clear with examples
- [ ] Security considerations documented (no secrets in remote)

---

## Appendix: Git Ref Naming Convention

Jin uses logical refs under `refs/jin/` namespace:

```
refs/jin/global/base                          # Layer 1
refs/jin/mode/<mode>/base                     # Layer 2
refs/jin/mode/<mode>/scope/<scope>/base       # Layer 3
refs/jin/mode/<mode>/scope/<scope>/project/<project>   # Layer 4
refs/jin/mode/<mode>/project/<project>        # Layer 5
refs/jin/scope/<scope>/base                   # Layer 6
refs/jin/project/<project>/base                # Layer 7
refs/jin/local/base                           # Layer 8
refs/jin/workspace/active                     # Layer 9

# Additional metadata refs
refs/jin/metadata/fetch-state
refs/jin/metadata/last-push
```

This namespace isolation:
- ✅ Prevents collision with user branches
- ✅ Makes Jin refs easily distinguishable
- ✅ Allows garbage collection of old refs
- ✅ Clear mental model for developers

---

Document Version: 1.0
Status: Ready for Implementation
Last Updated: 2025-01-15
