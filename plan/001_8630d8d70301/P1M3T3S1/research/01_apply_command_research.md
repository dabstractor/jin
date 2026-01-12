# Apply Command Research - Agent Output

## Research Findings

Based on the agent research of the `jin apply` command and detached workspace detection:

### 1. How `jin apply` detects "detached workspace" state

The detached workspace detection is implemented in `validate_workspace_attached()` function in `/home/dustin/projects/jin/src/staging/workspace.rs` (lines 325-399). It checks three conditions:

**Condition 1: File mismatch (lines 334-358)**
- Compares current file content hashes with stored hashes in `WorkspaceMetadata`
- Detects if files have been modified or deleted externally
- This is checked first (highest priority)

**Condition 2: Missing layer refs (lines 360-371)**
- Verifies that layer refs stored in `metadata.applied_layers` still exist in the repository
- Converts layer names like "mode/production" to ref paths like "refs/jin/layers/mode/production"

**Condition 3: Invalid context (lines 373-396)**
- Checks if active mode/scope in `ProjectContext` still exist as refs
- Validates references to modes and scopes

### 2. Exact error message for detached state

The error is defined in `/home/dustin/projects/jin/src/core/error.rs` (lines 38-54):

```rust
#[error(
    "Workspace is in a detached state.\n\
{details}\n\
\n\
Recovery: {recovery_hint}"
)]
DetachedWorkspace {
    workspace_commit: Option<String>,
    expected_layer_ref: String,
    details: String,
    recovery_hint: String,
},
```

**Example error messages:**

For file modifications:
```
Workspace is in a detached state.
Workspace files have been modified outside of Jin operations. Modified files:
  - config.txt
  - settings.json

Recovery: Run 'jin apply' to restore from active context
```

For missing layer refs:
```
Workspace is in a detached state.
Workspace metadata references layers that no longer exist. Missing refs:
  - refs/jin/layers/mode/production
  - refs/jin/layers/scope/backend

Recovery: Run 'jin apply' to rebuild from current active context
```

For invalid context:
```
Workspace is in a detached state.
Active context references a mode or scope that no longer exists: mode:production

Recovery: Run 'jin mode activate <valid-mode>' or 'jin scope activate <valid-scope>'
```

### 3. How WorkspaceMetadata is used in the apply process

**In `/home/dustin/projects/jin/src/commands/apply.rs`:**

1. **Loading and checking workspace dirty state (lines 464-490)**
   - Uses `WorkspaceMetadata::load()` to get last applied configuration
   - Compares current file hashes with stored hashes to detect modifications
   - Returns error if workspace is dirty without `--force`

2. **Validation with --force (lines 114-120)**
   - When `--force` is used, calls `validate_workspace_attached()` before destructive operations
   - Only runs validation if force flag is present

3. **Updating metadata on successful apply (lines 188-197)**
   - Creates new metadata with applied layers and file hashes
   - Saves metadata to `.jin/workspace/last_applied.json`
   - Tracks content hashes using Git blob OIDs

### 4. Apply flow comparison

**When metadata EXISTS:**
1. Load context
2. Check workspace dirty (if not --force)
3. If --force: validate workspace attached
4. Get applicable layers
5. Merge layers
6. Handle conflicts/dry run
7. Apply files to workspace
8. **Update workspace metadata with new hashes and layers**
9. Update .gitignore

**When metadata DOESN'T exist (fresh workspace):**
1. Load context
2. Check workspace dirty (if not --force) - returns false
3. If --force: validate workspace attached - returns Ok() (fresh workspace)
4. Get applicable layers
5. Merge layers
6. Handle conflicts/dry run
7. Apply files to workspace
8. **Create new workspace metadata**
9. Update .gitignore

### 5. Tests that exercise detached state error

The test suite includes comprehensive tests in:

**`/home/dustin/projects/jin/tests/workspace_validation.rs`:**
- Tests for all three detachment conditions
- Test validation order (file mismatch checked first)
- Tests for fresh workspaces
- Tests for recovery hints

**`/home/dustin/projects/jin/tests/destructive_validation.rs`:**
- Tests that `apply --force` is properly rejected in detached state
- Tests that regular `apply` fails with dirty check instead of detached error
- Tests recovery hints in error messages

Key test cases include:
- `test_apply_force_rejected_when_files_modified()` - verifies --force is blocked when files are modified
- `test_apply_force_rejected_when_layer_refs_missing()` - verifies --force is blocked when layers are deleted
- `test_apply_without_force_skips_validation()` - verifies normal apply uses dirty check instead

The implementation follows a safety-first approach where destructive operations (--force) validate workspace attachment to prevent data loss, while normal operations use lighter-weight checks.
