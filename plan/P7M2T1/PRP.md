# PRP: Implement jin rm Command

## Goal

**Feature Goal**: Implement `jin rm` command to remove files from Jin layer staging system with proper workspace integration, supporting staged-only removal and workspace file deletion.

**Deliverable**: A fully functional `jin rm` command that:
- Removes files from the staging index
- Optionally removes files from the workspace (with `--force` or user confirmation)
- Supports layer targeting flags (`--mode`, `--scope`, `--project`, `--global`)
- Updates `.gitignore` managed block appropriately
- Follows existing command patterns in the codebase

**Success Definition**:
- `jin rm file.txt` removes file from staging (marks for deletion)
- `jin rm --force file.txt` removes from staging and workspace without confirmation
- `jin rm --mode file.txt` removes from mode layer staging
- Command follows error handling patterns of `jin add` and `jin reset`
- All validation tests pass
- Integration tests demonstrate full workflow

## User Persona

**Target User**: Developer using Jin to manage configuration files across different contexts (modes, scopes, projects)

**Use Case**: User has staged files they no longer want tracked in Jin, or wants to remove files from both Jin tracking and their workspace

**User Journey**:
1. User has previously staged files with `jin add config.json`
2. User wants to stop tracking `config.json` in Jin
3. User runs `jin rm config.json` to mark for deletion
4. Optionally, user runs `jin rm --force config.json` to also delete from workspace
5. User commits the deletion with `jin commit`

**Pain Points Addressed**:
- No way to remove files from Jin staging without manual JSON editing
- Risk of corrupting staging index if manually edited
- Need for atomic removal operations across multiple layers
- Consistency with existing `jin add` command patterns

## Why

- **Feature completeness**: The `jin rm` command is the counterpart to `jin add`, completing the file lifecycle management
- **User safety**: Proper staging integration prevents accidental data loss
- **Workflow consistency**: Follows the same patterns as existing commands (`add`, `reset`)
- **Git-like familiarity**: Users familiar with `git rm` will find the interface intuitive

## What

### User-Visible Behavior

```bash
# Remove file from staging (mark for deletion)
jin rm config.json

# Remove from staging AND workspace (with confirmation prompt)
jin rm config.json  # prompts: "Type 'yes' to confirm:"

# Remove without confirmation
jin rm --force config.json

# Remove from specific layer
jin rm --mode config.json
jin rm --scope myscope --project config.json

# Remove multiple files
jin rm file1.json file2.json file3.json

# Dry run to see what would be removed
jin rm --dry-run file.json
```

### Success Criteria

- [ ] `jin rm file.txt` creates a `StagedOperation::Delete` entry for the file
- [ ] `jin rm --force file.txt` removes from staging and deletes workspace file
- [ ] Layer targeting flags work correctly (same routing as `jin add`)
- [ ] `.gitignore` managed block is updated (path removed)
- [ ] Error handling for non-existent files, non-staged files
- [ ] Confirmation prompt for workspace deletion (without `--force`)
- [ ] Batch processing of multiple files with error collection
- [ ] All unit tests pass
- [ ] Integration test demonstrates full add->rm->commit workflow

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

Yes - this PRP provides:
- Exact file locations and patterns to follow
- Complete code structure references with line numbers
- External research on git rm patterns
- Specific validation commands
- Test patterns to follow
- Known gotchas and constraints

### Documentation & References

```yaml
# MUST READ - Internal Codebase Patterns
- file: src/commands/add.rs
  why: Complete command structure to mirror for rm (validation, layer routing, staging integration, error handling)
  pattern: Command execute() function structure with validation, context loading, routing, staging operations
  gotcha: Note how errors are collected during batch processing - follow this pattern

- file: src/staging/entry.rs
  why: StagedEntry type and StagedOperation::Delete enum already exist
  pattern: Use StagedEntry::delete() factory method (line 46) to create deletion entries
  gotcha: Delete entries have empty content_hash and mode=0

- file: src/staging/index.rs
  why: StagingIndex API for adding/removing entries and persistence
  pattern: Use staging.remove() to unstage, staging.add() to add delete entry, staging.save() to persist
  gotcha: Entries are keyed by PathBuf - use exact path matching

- file: src/staging/gitignore.rs
  why: remove_from_managed_block() function already exists (line 73)
  pattern: Call remove_from_managed_block(&path) after removing from staging
  gotcha: Function normalizes paths internally - pass PathBuf directly

- file: src/staging/mod.rs
  why: Re-exports staging utilities and routing functions
  pattern: Import route_to_layer, validate_routing_options, RoutingOptions
  gotcha: Routing logic is identical to add command

- file: src/core/error.rs
  why: All error variants available for proper error handling
  pattern: Use JinError::NotFound, JinError::StagingFailed, JinError::NotInitialized, JinError::Other
  gotcha: Follow existing error message patterns

- file: src/cli/args.rs
  why: Existing argument patterns to follow for RmArgs
  pattern: Mirror AddArgs structure (lines 6-26) with additional flags
  gotcha: Add --force and --dry-run flags specific to rm

- file: src/cli/mod.rs
  why: Commands enum registration location
  pattern: Add Rm(RmArgs) variant to Commands enum (line 28), wire in execute() (line 32)
  gotcha: Commands are in alphabetical order - insert between Reset and Diff

- file: src/commands/mod.rs
  why: Command module registration and execute() dispatcher
  pattern: Add pub mod rm; and Commands::Rm(args) => rm::execute(args) in execute() (line 32)
  gotcha: Keep imports in alphabetical order

# MUST READ - External Research
- url: https://git-scm.com/docs/git-rm
  why: Git's rm command behavior and flag conventions (--cached, --force, -n dry-run)
  critical: git rm --cached removes from index only, keeps workspace file - this is our default behavior
  critical: git rm (no flags) removes from both index and workspace - this is our --force behavior

# Implementation Reference
- file: src/commands/add.rs (lines 130-157)
  why: stage_file() function shows the pattern for staging operations
  pattern: Validate file, read/create blob, create entry, add to staging
  gotcha: For rm, we create delete entries instead of add/modify entries

- file: src/commands/add.rs (lines 192-204)
  why: format_layer_name() helper for user-friendly layer display
  pattern: Copy this function for consistent layer name formatting
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs          # Add RmArgs struct here
│   └── mod.rs           # Add Rm variant to Commands enum
├── commands/
│   ├── add.rs           # Primary pattern reference
│   ├── reset.rs         # Secondary reference for removal patterns
│   ├── mod.rs           # Add rm module and wire execute()
│   └── rm.rs            # NEW FILE - Create this
├── staging/
│   ├── entry.rs         # StagedEntry::delete() already exists
│   ├── gitignore.rs     # remove_from_managed_block() exists
│   ├── index.rs         # StagingIndex API exists
│   └── mod.rs           # Re-exports
├── core/
│   ├── error.rs         # All error types
│   ├── layer.rs         # Layer enum with routing
│   └── mod.rs
└── main.rs
```

### Desired Codebase Tree with Files to be Added

```bash
src/
├── cli/
│   ├── args.rs          # MODIFY: Add RmArgs struct (after ResetArgs)
│   └── mod.rs           # MODIFY: Add Rm(RmArgs) to Commands enum
├── commands/
│   ├── add.rs           # REFERENCE: Primary pattern
│   ├── mod.rs           # MODIFY: Add pub mod rm; and wire in execute()
│   └── rm.rs            # NEW FILE: Implement rm command
└── ...
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Staging uses PathBuf keys - exact path matching required
// The staging.entries HashMap uses PathBuf as key
// User input "file.txt" must match exactly what was staged
// TODO: Consider path normalization for cross-platform consistency

// CRITICAL: Jin requires project context for layer routing
// Always load ProjectContext before route_to_layer()
// Handle JinError::NotInitialized gracefully

// CRITICAL: Error collection pattern for batch operations
// Don't return early on first error - collect all errors
// Only fail if NO files were successfully processed
// See add.rs lines 115-125 for pattern

// CRITICAL: .gitignore managed block uses forward slashes
// remove_from_managed_block() handles normalization internally
// Pass PathBuf directly without manual normalization

// CRITICAL: Layer routing validation
// --project requires --mode (validate_routing_options enforces)
// --global cannot be combined with layer-specific flags
// These validations are already implemented - use them

// CRITICAL: StagedEntry::delete() exists and should be used
// Don't manually construct StagedEntry for deletions
// Use StagedEntry::delete(path, layer) factory method

// CRITICAL: File existence in workspace check
// A file may be staged but deleted from workspace
// Check entry.path.exists() before workspace deletion
// This is expected behavior - handle gracefully

// GOTCHA: StagingIndex.remove() returns Option<StagedEntry>
// Use this to check if file was actually staged
// None means file was not in staging index

// GOTCHA: Dry-run pattern
// No explicit dry-run support in add.rs
// Implement by printing what would be done then returning Ok(())
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - existing types are sufficient:

```rust
// src/staging/entry.rs - Already exists
pub enum StagedOperation {
    AddOrModify,
    Delete,      // Use this for rm command
    Rename,
}

// Factory method already exists (line 46)
pub fn delete(path: PathBuf, target_layer: Layer) -> Self
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/cli/args.rs
  - ADD: RmArgs struct after ResetArgs (line 87)
  - IMPLEMENT: Files vector, layer flags (--mode, --scope, --project, --global)
  - ADD: --force flag (skip confirmation for workspace deletion)
  - ADD: --dry-run flag (show what would be removed without doing it)
  - FOLLOW pattern: ResetArgs (lines 52-86) for layer flag structure
  - FOLLOW pattern: AddArgs (lines 5-26) for basic argument structure
  - NAMING: RmArgs struct, snake_case field names
  - PLACEMENT: After ResetArgs, before DiffArgs

Task 2: MODIFY src/cli/mod.rs
  - ADD: Rm(RmArgs) variant to Commands enum (line 28)
  - PLACE: Between Reset and Diff variants (alphabetical order)
  - ADD: help text "Remove files from staging and optionally workspace"
  - PRESERVE: All existing command variants

Task 3: CREATE src/commands/rm.rs
  - IMPLEMENT: execute(args: RmArgs) -> Result<()> function
  - FOLLOW pattern: src/commands/add.rs (lines 34-128) for overall structure
  - IMPLEMENT: Layer routing using route_to_layer() and validate_routing_options()
  - IMPLEMENT: File matching against staging entries
  - IMPLEMENT: Staging removal with StagedEntry::delete() creation
  - IMPLEMENT: Workspace file removal (with --force or confirmation)
  - IMPLEMENT: .gitignore managed block cleanup
  - IMPLEMENT: Error collection for batch operations
  - IMPLEMENT: Dry-run support (--dry-run flag)
  - ADD: format_layer_name() helper (copy from add.rs lines 192-204)
  - ADD: prompt_confirmation() helper for destructive operations
  - NAMING: execute function, private helpers as needed
  - PLACEMENT: New file in src/commands/

Task 4: MODIFY src/commands/mod.rs
  - ADD: pub mod rm; import (line 28, after reset)
  - MODIFY: execute() function to match Commands::Rm(args)
  - ADD: rm::execute(args) call in match arm
  - PRESERVE: All existing command wiring

Task 5: CREATE tests/ integration or unit tests in src/commands/rm.rs
  - IMPLEMENT: Unit tests for rm command logic
  - FOLLOW pattern: src/commands/add.rs (lines 206-325) for test structure
  - TEST: No files specified error
  - TEST: File not in staging error
  - TEST: Layer routing validation
  - TEST: Staging removal (creates delete entry)
  - TEST: Workspace removal with --force
  - TEST: Workspace removal with confirmation
  - TEST: .gitignore cleanup
  - TEST: Batch processing with error collection
  - TEST: Dry-run mode
  - COVERAGE: All code paths, error cases
  - PLACEMENT: #[cfg(test)] mod tests at end of rm.rs
```

### Implementation Patterns & Key Details

```rust
// Show critical patterns and gotchas

// PATTERN: Command execute() structure (mirror add.rs)
pub fn execute(args: RmArgs) -> Result<()> {
    // 1. Validate inputs
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Load project context (handle NotInitialized)
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. Build and validate routing options
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
    validate_routing_options(&options)?;

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;

    // 5. Open Jin repo
    let repo = JinRepo::open_or_create()?;

    // 6. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. Process each file (with error collection)
    let mut removed_count = 0;
    let mut errors = Vec::new();

    for path_str in &args.files {
        let path = PathBuf::from(path_str);
        match unstage_file(&path, target_layer, &repo, &mut staging, &args) {
            Ok(_) => removed_count += 1,
            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
        }
    }

    // 8. Save staging index
    staging.save()?;

    // 9. Print summary
    if removed_count > 0 {
        println!(
            "Removed {} file(s) from {} layer",
            removed_count,
            format_layer_name(target_layer)
        );
    }

    // 10. Handle errors (partial success pattern)
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if removed_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to remove", errors.len()),
            });
        }
    }

    Ok(())
}

// PATTERN: File unstaging logic
fn unstage_file(
    path: &Path,
    layer: Layer,
    repo: &JinRepo,
    staging: &mut StagingIndex,
    args: &RmArgs,
) -> Result<()> {
    // Check if file is in staging
    let existing_entry = staging.get(path)
        .ok_or_else(|| JinError::NotFound(format!(
            "File not in staging: {}", path.display()
        )))?;

    // Remove from staging index
    staging.remove(path);

    // Create delete entry to mark for deletion on commit
    let delete_entry = StagedEntry::delete(path.to_path_buf(), layer);
    staging.add(delete_entry);

    // Remove from .gitignore managed block
    if let Err(e) = remove_from_managed_block(path) {
        eprintln!("Warning: Could not update .gitignore: {}", e);
    }

    // Remove from workspace if --force or confirmed
    if should_remove_from_workspace(args) {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

// PATTERN: Confirmation helper (follow reset.rs pattern)
fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("yes"))
}

// PATTERN: Layer name formatter (copy from add.rs)
fn format_layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::GlobalBase => "global-base",
        Layer::ModeBase => "mode-base",
        Layer::ModeScope => "mode-scope",
        Layer::ModeScopeProject => "mode-scope-project",
        Layer::ModeProject => "mode-project",
        Layer::ScopeBase => "scope-base",
        Layer::ProjectBase => "project-base",
        Layer::UserLocal => "user-local",
        Layer::WorkspaceActive => "workspace-active",
    }
}

// CRITICAL: Dry-run handling
// Print what would be done, then return early
if args.dry_run {
    println!("Would remove: {}", path.display());
    return Ok(());
}

// CRITICAL: Confirmation before workspace deletion
// Only prompt if NOT using --force flag
if should_remove_from_workspace(args) && !args.force {
    let message = format!(
        "This will remove {} file(s) from workspace. Type 'yes' to confirm:",
        files_to_remove.len()
    );
    if !prompt_confirmation(&message)? {
        println!("Removal cancelled");
        return Ok(());
    }
}

// GOTCHA: Workspace removal should be conditional
// Default: Remove from staging only (like git rm --cached)
// --force: Remove from staging AND workspace
// User's existing workflow: rm stages deletion, commit applies it
fn should_remove_from_workspace(args: &RmArgs) -> bool {
    args.force
}
```

### Integration Points

```yaml
STAGING:
  - modify: "Load staging index, remove entries, add delete entries"
  - save: "Call staging.save() after modifications"
  - api: "staging.remove(path), staging.add(delete_entry)"

GITIGNORE:
  - modify: "Remove paths from managed block"
  - api: "remove_from_managed_block(path) from staging::gitignore"
  - location: ".gitignore file in project root"

CLI_ENUMS:
  - add to: "src/cli/mod.rs Commands enum"
  - pattern: "Rm(RmArgs) variant between Reset and Diff"
  - help: "Remove files from staging and optionally workspace"

COMMAND_DISPATCHER:
  - add to: "src/commands/mod.rs execute() function"
  - pattern: "Commands::Rm(args) => rm::execute(args)"
  - import: "pub mod rm;"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --color=always                       # Compile check with colored output
cargo clippy --color=always -- -D warnings       # Lint with warnings as errors

# Project-wide validation
cargo fmt --check                                 # Verify formatting
cargo check                                       # Full compilation check

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the rm command module
cargo test rm --lib -- --nocapture                # Run rm module tests

# Test all command tests
cargo test commands --lib -- --nocapture          # Run all command tests

# Run with output for debugging
cargo test execute --lib -- --nocapture --test-threads=1

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual integration testing sequence
cd /tmp && mkdir test-jin-rm && cd test-jin-rm    # Create test directory

# Initialize Jin project
jin init                                           # Should succeed

# Create test files
echo '{"test": true}' > config.json
echo '{"mode": true}' > mode-config.json

# Stage files to different layers
jin add config.json                               # Should stage to project-base
jin add --mode mode-config.json                   # Should stage to mode-base

# Verify staging with status
jin status                                        # Should show 2 staged files

# Test basic rm (staging only)
jin rm config.json                                # Should mark for deletion
jin status                                        # Should show 1 staged file, 1 delete

# Test rm with force (workspace removal)
echo '{"force": true}' > force-test.json
jin add force-test.json
jin rm --force force-test.json                    # Should remove from staging AND workspace
ls force-test.json                                # Should fail (file deleted)

# Test layer-specific rm
jin rm --mode mode-config.json                    # Should remove from mode layer
jin status                                        # Should reflect mode layer change

# Test commit with deletions
jin commit -m "Remove test files"                 # Should commit deletions

# Test error cases
jin rm nonexistent.txt                            # Should error: not in staging
jin rm                                            # Should error: no files specified

# Expected: All operations succeed, status reflects correct state
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Layer Precedence Validation
# Test that files are removed from correct layers
jin add --mode --project layered.json             # Stage to mode-project layer
jin rm --mode layered.json                        # Remove from mode layer (not mode-project)
jin status                                        # Verify correct layer behavior

# Batch Processing Validation
# Test error collection with multiple files
echo '{}' > a.json && echo '{}' > b.json && echo '{}' > c.json
jin add a.json b.json c.json
rm a.json                                         # Delete from workspace
jin rm a.json b.json c.json                       # Should succeed for b,c, warn about a
# Expected: Partial success with clear error messages

# Gitignore Cleanup Validation
# Verify .gitignore is properly cleaned
jin add test.json
cat .gitignore | grep test.json                   # Should be present
jin rm test.json
cat .gitignore | grep test.json                   # Should be absent

# Dry-run Validation
jin rm --dry-run *.json                           # Should print what would be removed
jin status                                        # Should show no changes

# Force Flag Validation
# Test that --force skips confirmation
echo '{}' > force-test.json
jin add force-test.json
echo "yes" | jin rm force-test.json               # Should prompt and succeed
jin add force-test.json                           # Re-stage
jin rm --force force-test.json                    # Should NOT prompt, succeed immediately

# Expected: All creative validations pass, edge cases handled gracefully
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --lib`
- [ ] No linting errors: `cargo clippy -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Compiles without warnings: `cargo check --color=always`

### Feature Validation

- [ ] All success criteria from "What" section met
- [ ] Manual testing successful (Level 3 commands)
- [ ] Error cases handled gracefully with proper error messages
- [ ] Integration points work as specified (staging, gitignore)
- [ ] Layer routing works correctly for all combinations

### Code Quality Validation

- [ ] Follows existing codebase patterns (mirrors add.rs structure)
- [ ] File placement matches desired codebase tree
- [ ] Anti-patterns avoided (see below)
- [ ] Dependencies properly managed and imported
- [ ] CLI help text is clear and accurate

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable/function names
- [ ] User-facing messages are informative but not verbose
- [ ] Error messages guide users to correct behavior

---

## Anti-Patterns to Avoid

- ❌ Don't manually construct StagedEntry for deletions - use `StagedEntry::delete()`
- ❌ Don't skip validation because "it should work" - always check file exists in staging
- ❌ Don't ignore failing tests - fix them before considering the task complete
- ❌ Don't use synchronous file operations without considering async context (though this is a CLI tool)
- ❌ Don't hardcode layer values - use routing functions
- ❌ Don't catch all exceptions - use specific JinError variants
- ❌ Don't return early on first error in batch operations - collect all errors
- ❌ Don't forget to call `staging.save()` after modifications
- ❌ Don't remove from .gitignore without using `remove_from_managed_block()`
- ❌ Don't implement dry-run by actually modifying state - print and return early
- ❌ Don't prompt for confirmation when `--force` flag is used
- ❌ Don't use custom confirmation prompt pattern - follow reset.rs pattern if it exists
- ❌ Don't forget to handle `JinError::NotInitialized` gracefully
- ❌ Don't assume file exists in workspace - check `path.exists()` before deletion
- ❌ Don't modify layer routing logic - reuse existing functions

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success likelihood

**Justification**:
- Complete codebase context with exact file locations and patterns
- Existing `StagedEntry::delete()` and `remove_from_managed_block()` functions
- Clear pattern reference in `add.rs` command
- Comprehensive validation commands for all testing levels
- All dependencies and imports identified
- Known gotchas documented

**Risk Factors**:
- Confirmation prompt implementation may need iteration
- Layer routing edge cases may require additional validation
- Integration with commit pipeline should be verified (P7.M1.T1)

**Mitigation**:
- Start with basic staging-only rm (like git rm --cached)
- Add workspace removal with --force flag in second pass
- Test with actual Jin repository to verify commit integration
