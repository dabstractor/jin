# Detached Workspace Conditions

## Overview

A **detached workspace** in Jin is a state where the workspace files or metadata no longer correspond to a valid combination of layer commits from the active context. This violates the PRD's non-negotiable invariant that "the workspace is never source of truth."

This document defines the three conditions that cause a workspace to become detached, how to detect each condition, and provides examples for future implementation of validation logic (P1.M3.T2).

---

## Background: Jin's Architecture

### Workspace Layer (Layer 9)

```
9. Workspace Active Layer    | Derived merge result | `.jin/workspace/`
```

The workspace is:
- The 9th and topmost layer in Jin's nine-layer hierarchy
- Where all layer configurations are merged and applied to actual files
- **Never the source of truth** - it only reflects merged layer content
- Tracked by `WorkspaceMetadata` stored in `.jin/workspace/last_applied.json`

### Active Context

```rust
// From src/core/config.rs
pub struct ProjectContext {
    pub mode: Option<String>,      // Currently active mode (e.g., "claude")
    pub scope: Option<String>,     // Currently active scope (e.g., "language:javascript")
    pub project: Option<String>,   // Project name (auto-inferred)
    pub last_updated: Option<String>, // Last update timestamp
}
```

Stored in `.jin/context` (YAML format), the active context determines which layers from the hierarchy are merged into the workspace.

---

## Condition 1: Workspace Files Don't Match Layer Merge Result

### Description

The workspace files have been modified outside of Jin operations, so their current content no longer matches the expected merge result from the active layers.

### Detection Logic

```rust
// Pseudo-code for P1.M3.T2 implementation
fn detect_file_mismatch(workspace: &Workspace, metadata: &WorkspaceMetadata) -> bool {
    // 1. Load WorkspaceMetadata from .jin/workspace/last_applied.json
    let stored_metadata = WorkspaceMetadata::load()?;

    // 2. Compute current hash of each workspace file
    for file_path in workspace.files() {
        let current_hash = hash_file(&file_path)?;

        // 3. Compare to stored hash
        if let Some(stored_hash) = stored_metadata.file_hashes.get(&file_path) {
            if current_hash != *stored_hash {
                // File has been modified externally
                return true;
            }
        }
    }

    false
}
```

### Example Scenario

```bash
# User has active context with mode=claude, scope=default
$ jin context
mode: claude
scope: default

# User manually edits a workspace file (outside of Jin)
$ nano .jin/workspace/config.json
# ... makes changes ...

# Workspace is now detached - files don't match layer merge result
$ jin status
Error: Workspace is in a detached state.
Workspace files have been modified outside of Jin operations.

Recovery: Run 'jin reset --hard refs/jin/layers/modes/claude/scopes/default' to restore
```

### Error Message Template

```
Workspace is in a detached state.
Workspace files have been modified outside of Jin operations. The following files have unexpected content:
  - .jin/workspace/config.json
  - .jin/workspace/settings.yaml

Recovery: Run 'jin reset --hard refs/jin/layers/modes/claude/scopes/default' to restore your workspace to the correct state.
```

---

## Condition 2: WorkspaceMetadata References Non-Existent Commits

### Description

The `WorkspaceMetadata` references layer commits that no longer exist in the Jin repository. This can occur due to garbage collection or manual deletion of Git objects in the `.jin` directory.

### Detection Logic

```rust
// Pseudo-code for P1.M3.T2 implementation
fn detect_missing_commits(metadata: &WorkspaceMetadata, repo: &JinRepo) -> Result<bool> {
    // 1. Get layer commit refs from WorkspaceMetadata
    for layer_ref in metadata.layer_commits.values() {
        // 2. Attempt to find each commit in Jin repository
        match repo.find_commit(layer_ref) {
            Ok(_) => continue,  // Commit exists
            Err(e) if e.is_not_found() => {
                // Commit referenced in metadata doesn't exist
                return Ok(true);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(false)
}
```

### Example Scenario

```bash
# User has applied layers with specific commits
$ jin apply
Applied: mode=claude (commit: abc123def...)
Applied: scope=default (commit: def456ghi...)

# User runs aggressive garbage collection in .jin directory
$ cd .jin
$ git gc --prune=now
# ... commits are now deleted ...

# Workspace is now detached - metadata references deleted commits
$ jin reset --hard
Error: Workspace is in a detached state.
Workspace metadata references commits that no longer exist in the Jin repository.
Missing commit: abc123def (layer: modes/claude)

Recovery: Run 'jin repair --check' to diagnose, or 'jin apply' to rebuild from current context
```

### Error Message Template

```
Workspace is in a detached state.
Workspace metadata references commits that no longer exist in the Jin repository.
Missing commit: abc123def (layer: modes/claude)
This can occur after aggressive git garbage collection.

Recovery: Run 'jin apply' to rebuild your workspace from the current active context.
```

---

## Condition 3: Active Context References Deleted Modes/Scopes

### Description

The active context (stored in `.jin/context`) points to a mode or scope that has been deleted from the Jin repository. The workspace may still be valid, but it's "detached" because its source layer definition no longer exists.

### Detection Logic

```rust
// Pseudo-code for P1.M3.T2 implementation
fn detect_invalid_context(context: &ProjectContext, repo: &JinRepo) -> Result<bool> {
    // 1. Check if active mode exists
    if let Some(mode) = &context.mode {
        let mode_ref = format!("refs/jin/layers/modes/{}", mode);
        if !repo.reference_exists(&mode_ref) {
            return Ok(true);
        }
    }

    // 2. Check if active scope exists
    if let Some(scope) = &context.scope {
        // Scope path depends on mode - handle both cases
        let scope_ref = if let Some(mode) = &context.mode {
            format!("refs/jin/layers/modes/{}/scopes/{}", mode, scope)
        } else {
            format!("refs/jin/layers/scopes/{}", scope)
        };

        if !repo.reference_exists(&scope_ref) {
            return Ok(true);
        }
    }

    Ok(false)
}
```

### Example Scenario

```bash
# User has active context with mode=production
$ jin context
mode: production
scope: default

# User deletes the active mode
$ jin mode delete production
Deleted mode: production

# Workspace is now detached - active context points to deleted mode
$ jin apply
Error: Workspace is in a detached state.
Active context references a mode or scope that no longer exists.
Invalid mode: 'production'

Recovery: Run 'jin mode activate <valid-mode>' to set a new active mode, or 'jin mode activate --unset' to clear mode.
```

### Error Message Template

```
Workspace is in a detached state.
Active context references a mode or scope that no longer exists in the Jin repository.
Invalid mode: 'production' (deleted or never created)

Recovery: Run 'jin mode activate <valid-mode>' to set a new active mode.
Available modes: development, staging, testing
```

---

## Combined Detection Logic (for P1.M3.T2)

The `validate_workspace_attached()` function to be implemented in P1.M3.T2 should check all three conditions:

```rust
// Implementation target for P1.M3.T2
// Location: src/staging/workspace.rs

use crate::core::error::{JinError, Result};
use crate::core::config::ProjectContext;

pub fn validate_workspace_attached(
    workspace: &Workspace,
    context: &ProjectContext,
    repo: &JinRepo
) -> Result<()> {
    // Check Condition 1: File mismatch
    if detect_file_mismatch(workspace)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: workspace.current_commit_hash()?,
            expected_layer_ref: context.active_layer_ref()?,
            details: "Workspace files have been modified outside of Jin operations".to_string(),
            recovery_hint: format!("Run 'jin reset --hard {}' to restore", context.active_layer_ref()?),
        });
    }

    // Check Condition 2: Missing commits
    if detect_missing_commits(&workspace.metadata, repo)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: None,  // Unknown - commits don't exist
            expected_layer_ref: "<unknown>".to_string(),
            details: "Workspace metadata references commits that no longer exist".to_string(),
            recovery_hint: "Run 'jin apply' to rebuild from current active context".to_string(),
        });
    }

    // Check Condition 3: Invalid context
    if detect_invalid_context(context, repo)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: workspace.current_commit_hash()?,
            expected_layer_ref: format!("mode:{}", context.mode.as_deref().unwrap_or("<none>")),
            details: format!("Active context references deleted mode: {}", context.mode.as_deref().unwrap_or("<none>")),
            recovery_hint: "Run 'jin mode activate <valid-mode>' to set a new active mode".to_string(),
        });
    }

    Ok(())
}
```

---

## Prevention: How Jin Maintains Attached Workspace State

To prevent detachment, Jin should:

1. **Prevent External File Modification**: Design workspace files as read-only where possible (or add warnings)

2. **Safe Garbage Collection**: Document that `.jin` directory should not be manually garbage collected

3. **Deletion Safety**: Prevent deletion of active modes/scopes without explicit confirmation:
   ```bash
   $ jin mode delete production
   Error: Cannot delete active mode 'production'.
   Run 'jin mode activate --unset' first to deactivate, then delete.
   ```

4. **Status Indication**: The `jin status` command should display workspace state (attached/detached)

---

## References

- **PRD Non-negotiable Invariant #4**: "Jin will abort any operation that would create a detached state"
- **Implementation Gap Analysis**: Critical Gap #3 - "Missing Detached Workspace State Detection"
- **Git Detached HEAD**: [Git Documentation](https://git-scm.com/book/en/v2/Git-Internals-Git-References) - Note: Jin's concept is different from Git's detached HEAD
- **Error Type Definition**: `src/core/error.rs` - `DetachedWorkspace` variant (P1.M3.T1)
- **Validation Implementation**: `src/staging/workspace.rs` - `validate_workspace_attached()` (P1.M3.T2)

---

## Future Integration

This research document supports:

- **P1.M3.T1** (current task): Define error variant
- **P1.M3.T2**: Implement validation logic using detection functions above
- **P1.M3.T3**: Integrate validation into destructive operations (reset --hard, apply --force)
- **P1.M3.T4**: Add `jin repair --check` to diagnose and report detachment
- **P1.M3.T5**: Integration tests for all detachment scenarios
