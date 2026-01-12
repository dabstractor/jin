# Research: ScopeAction::Use Handler Analysis

## File Location
**Path**: `/home/dustin/projects/jin/src/commands/scope.rs`
**Function**: `use_scope()` (lines 142-188)

## Current Implementation

```rust
/// Activate a scope
fn use_scope(name: &str) -> Result<()> {
    // Validate scope name
    validate_scope_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Convert scope name to ref-safe format (replace colons with slashes)
    let ref_safe_name = name.replace(':', "/");

    // Check if scope exists (check both mode-bound and untethered)
    let untethered_ref = format!("refs/jin/scopes/{}", ref_safe_name);
    let mode_bound_pattern = format!("refs/jin/modes/*/scopes/{}", ref_safe_name);

    let exists = repo.ref_exists(&untethered_ref)
        || !repo
            .list_refs(&mode_bound_pattern)
            .unwrap_or_default()
            .is_empty();

    if !exists {
        return Err(JinError::NotFound(format!(
            "Scope '{}' not found. Create it with: jin scope create {}",
            name, name
        )));
    }

    // Load project context
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Update scope
    context.scope = Some(name.to_string());

    // Save context
    context.save()?;

    println!("Activated scope '{}'", name);
    println!("Stage files with: jin add --scope={}", name);

    Ok(())
}
```

## Key Integration Point for Metadata Clearing

**Line 179** (`context.scope = Some(name.to_string());`) is where scope is updated.

**After line 182** (`context.save()?;`) is where metadata clearing logic should be added.

## Imports Used

```rust
use crate::cli::ScopeAction;
use crate::core::{JinError, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps, RefOps};
```

## Error Handling Patterns

- Scope validation: Returns `JinError::Other` for invalid names
- Scope existence: Returns `JinError::NotFound` if scope doesn't exist
- Context loading: Handles `JinError::NotInitialized` specifically
- File operations: Uses `?` operator for error propagation

## Scope Existence Check Logic

The handler checks two locations:
1. **Untethered scopes**: `refs/jin/scopes/{name}`
2. **Mode-bound scopes**: `refs/jin/modes/*/scopes/{name}`

## Context Update Pattern

```rust
// Load context with fallback to default
let mut context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
    Err(_) => ProjectContext::default(),
};

// Update scope field
context.scope = Some(name.to_string());

// Persist to disk
context.save()?;
```

## Integration Location for Metadata Clearing

```rust
// After context.save() at line 182:
// ADD: Load metadata, compare scopes, clear if different
```
