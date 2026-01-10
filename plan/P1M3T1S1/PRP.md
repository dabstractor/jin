# PRP: P1.M3.T1.S1 - Load Workspace Metadata in Mode Use Handler

---

## Goal

**Feature Goal**: Enable the `jin mode use` command to load workspace metadata for comparison with the new mode, preparing for automatic metadata clearing on mode switches.

**Deliverable**: Modified `use_mode()` function in `src/commands/mode.rs` that loads `WorkspaceMetadata` after updating `ProjectContext`, handling the `NotFound` case gracefully.

**Success Definition**:
- `use_mode()` loads `WorkspaceMetadata` as `Option<WorkspaceMetadata>` after activating the new mode
- `NotFound` error is handled gracefully (returns `None`, not an error)
- Other errors (e.g., parse errors) are properly propagated
- Loaded metadata is available for comparison with the new mode (for P1.M3.T1.S2)

---

## User Persona

**Target User**: Jin internals (specifically P1.M3.T1.S2 which will compare modes and clear metadata).

**Use Case**: When a user switches modes with `jin mode use <name>`, the system needs to detect if the current workspace metadata was created with a different mode, enabling automatic cleanup to prevent detached workspace state.

**User Journey**:
1. User runs `jin mode use <new-mode>`
2. Mode is activated in ProjectContext
3. System attempts to load WorkspaceMetadata
4. If metadata exists and references a different mode, P1.M3.T1.S2 will clear it
5. User can run `jin apply` without encountering detached state errors

**Pain Points Addressed**:
- Prevents "detached workspace" errors when switching modes
- Eliminates manual `jin reset --hard` steps when changing modes
- Provides seamless mode switching experience

---

## Why

- **Problem**: Currently, switching modes leaves old workspace metadata that references the previous mode, causing "detached workspace" errors when running `jin apply`
- **Solution**: Load metadata during mode switch to enable detection and cleanup in P1.M3.T1.S2
- **Foundation**: This subtask is the first step in the auto-cleanup workflow (P1.M3.T1) - load the data, then P1.M3.T1.S2 will compare and clear if needed
- **User Experience**: Mode switching should "just work" without manual intervention

---

## What

### User-Visible Behavior

After this subtask (user-visible changes will be visible after P1.M3.T1.S2):

```bash
# Before: Switching modes causes detached state
$ jin mode use claude
Activated mode 'claude'
$ jin apply
Error: Workspace is in a detached state
# User must manually reset or use --force

# After P1.M3.T1 complete: Mode switch clears metadata
$ jin mode use production
Activated mode 'production'
Cleared workspace metadata (mode changed).
Run 'jin apply' to apply new mode.
$ jin apply
# Works seamlessly!
```

### Technical Requirements

1. **Modify `src/commands/mode.rs`**: Update the `use_mode()` function
2. **Add import**: `use crate::staging::metadata::WorkspaceMetadata;`
3. **Load metadata**: After `context.save()`, add metadata loading logic
4. **Handle NotFound gracefully**: Return `None` when metadata doesn't exist
5. **Preserve error propagation**: Other errors (parse, IO) should propagate normally
6. **Store for comparison**: The loaded metadata will be used in P1.M3.T1.S2

### Success Criteria

- [ ] `WorkspaceMetadata::load()` is called in `use_mode()` after mode activation
- [ ] `NotFound` error results in `None` (no error returned to user)
- [ ] Other errors (parse, IO) are properly propagated
- [ ] Metadata is available as `Option<WorkspaceMetadata>` for P1.M3.T1.S2
- [ ] No user-visible changes in this subtask (loading is internal)
- [ ] All existing mode tests still pass

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the exact function to modify, the pattern for graceful metadata loading, existing test patterns, and comprehensive research references._

### Documentation & References

```yaml
# MUST READ - Core Implementation Context

# Contract Definition (exact specifications)
- docfile: tasks.json (P1.M3.T1.S1 context_scope)
  why: Defines the exact contract for this work item
  section: |
    CONTRACT DEFINITION:
    1. RESEARCH NOTE: ModeAction::Use handler is in src/commands/mode.rs. WorkspaceMetadata is in
       src/staging/metadata.rs with load()/save() methods. See plan/architecture/fix_specifications.md.
    2. INPUT: None - modifying existing mode use logic.
    3. LOGIC: In the ModeAction::Use match arm, after successfully updating ProjectContext,
       attempt to load WorkspaceMetadata. Handle missing metadata gracefully (it may not exist yet).
       Get the current mode from metadata if it exists.
    4. OUTPUT: Optional WorkspaceMetadata loaded, ready for comparison with new mode.
  critical: "Load metadata AFTER context.save(), handle NotFound as None"

# File to modify (exact location)
- file: /home/dustin/projects/jin/src/commands/mode.rs
  why: Contains the use_mode() function that needs modification
  section: "use_mode() function (lines 86-121)"
  pattern: |
    // Current implementation ends at line 120:
    // Save context
    context.save()?;

    println!("Activated mode '{}'", name);
    println!("Stage files with: jin add --mode");

    Ok(())
    // ADD: Load WorkspaceMetadata here

# WorkspaceMetadata to load
- file: /home/dustin/projects/jin/src/staging/metadata.rs
  why: Defines WorkspaceMetadata::load() method we need to call
  section: "WorkspaceMetadata::load() method (lines 37-53)"
  pattern: |
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content).map_err(|e| JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            })
        } else {
            Err(JinError::NotFound(path.display().to_string()))
        }
    }
  critical: "Returns NotFound when .jin/workspace/last_applied.json doesn't exist"

# Error type to handle
- file: /home/dustin/projects/jin/src/core/error.rs
  why: Defines JinError::NotFound variant we need to match against
  section: "JinError::NotFound variant (line 69-70)"
  pattern: |
    /// File not found
    #[error("File not found: {0}")]
    NotFound(String),
  critical: "Match against JinError::NotFound(_) to handle missing metadata gracefully"

# Codebase pattern for graceful metadata loading
- file: /home/dustin/projects/jin/src/staging/workspace.rs
  why: Shows the established pattern for loading WorkspaceMetadata gracefully
  section: "validate_workspace_attached() function (line 327)"
  pattern: |
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => return Ok(()),  // Fresh workspace - no metadata
        Err(e) => return Err(e),
    };
  critical: "This is the pattern to follow: NotFound = graceful handling, other errors = propagate"

# Additional pattern examples
- file: /home/dustin/projects/jin/src/commands/status.rs
  why: Another example of the same graceful loading pattern
  section: "check_workspace_state() function (line 169)"
  pattern: |
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => return Ok(WorkspaceState::Clean),
        Err(e) => return Err(e),
    };

# Fix specifications context
- docfile: /home/dustin/projects/jin/plan/docs/fix_specifications.md
  why: Explains Fix 3: Mode Switching UX which this subtask is part of
  section: "Fix 3: Mode Switching UX"
  critical: |
    "Option A: Auto-Clear Metadata (Recommended)
     In ModeAction::Use handler:
     After activating mode, check if workspace metadata references different mode"

# Parent PRP context (P1.M3)
- docfile: /home/dustin/projects/jin/plan/P1M3/PRP.md
  why: Parent milestone PRP for understanding overall context
  section: "Goal", "What"
  note: "P1.M3 is about transaction system, this subtask is independent but related to mode switching UX"

# External Research: Rust Option/Result patterns
- url: https://effective-rust.com/transform.html
  why: Confirms explicit match pattern is idiomatic for distinguishing error types
  section: "Prefer Option and Result transforms"
  critical: |
    "When you need to distinguish between different error types, the explicit match
     pattern is recommended over Result::ok() which discards error information"

- url: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
  why: Official Rust documentation on Result handling
  section: "Recoverable Errors with Result"

- url: https://medium.com/@indrajit7448/mastering-result-and-option-the-simple-secrets-to-idiomatic-rust-error-handling-ae6b5dcfe0b4
  why: Comprehensive guide on error handling patterns
  section: "Converting Result to Option"
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── commands/
│   │   └── mode.rs                      # MODIFY: use_mode() function (lines 86-121)
│   ├── staging/
│   │   └── metadata.rs                  # REFERENCE: WorkspaceMetadata::load() (lines 37-53)
│   └── core/
│       ├── config.rs                    # REFERENCE: ProjectContext (lines 88-132)
│       └── error.rs                     # REFERENCE: JinError::NotFound (line 69-70)
└── plan/
    ├── docs/
    │   └── fix_specifications.md        # CONTEXT: Fix 3: Mode Switching UX
    └── P1M3T1S1/
        └── PRP.md                       # THIS FILE
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── src/
    └── commands/
        └── mode.rs                      # MODIFIED: use_mode() now loads WorkspaceMetadata
            ├── Import added: use crate::staging::metadata::WorkspaceMetadata;
            └── After context.save()?:
                let metadata = match WorkspaceMetadata::load() {
                    Ok(meta) => Some(meta),
                    Err(JinError::NotFound(_)) => None,
                    Err(e) => return Err(e),
                };
                // Metadata available for P1.M3.T1.S2 comparison
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Load metadata AFTER context.save(), not before
// The mode must be successfully updated first
// If mode activation fails, we shouldn't load metadata

// PATTERN: Convert Result<T, E> to Option<T> gracefully
let metadata = match WorkspaceMetadata::load() {
    Ok(meta) => Some(meta),              // Success: Some(metadata)
    Err(JinError::NotFound(_)) => None,   // Expected: Fresh workspace
    Err(e) => return Err(e),              // Unexpected: Propagate error
};

// GOTCHA: Don't use Result::ok() - it discards error information
// ANTI-PATTERN: let metadata = WorkspaceMetadata::load().ok();
// PROBLEM: Can't distinguish between "file not found" (ok) and "parse error" (bad)

// GOTCHA: WorkspaceMetadata path is .jin/workspace/last_applied.json
// Uses JIN_DIR environment variable for test isolation
// WorkspaceMetadata::default_path() handles this automatically

// PATTERN: Other commands use this same pattern
// - src/staging/workspace.rs:327 (validate_workspace_attached)
// - src/commands/status.rs:169 (check_workspace_state)
// - src/commands/diff.rs:306 (diff_workspace_vs_workspace_active)

// GOTCHA: This subtask doesn't clear metadata yet
// P1.M3.T1.S2 will add the comparison and clearing logic
// This subtask only loads the metadata as Option<WorkspaceMetadata>

// GOTCHA: Metadata contains applied_layers Vec<String>, not "current mode"
// For P1.M3.T1.S2: Need to check if any applied_layers start with "mode/{old_mode}/"
// or compare against context.mode from ProjectContext

// NOTE: WorkspaceMetadata structure:
// - timestamp: String (RFC3339)
// - applied_layers: Vec<String> (layer refs that were applied)
// - files: HashMap<PathBuf, String> (file -> content hash)
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - Using existing `WorkspaceMetadata` and `JinError::NotFound`.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD IMPORT for WorkspaceMetadata in mode.rs
  - ADD: use crate::staging::metadata::WorkspaceMetadata;
  - LOCATION: Top of src/commands/mode.rs after existing imports
  - DEPENDENCIES: None (first task)

Task 2: ADD METADATA LOADING in use_mode() function
  - MODIFY: use_mode() function (lines 86-121)
  - ADD: After context.save()? at line 115, add metadata loading
  - PATTERN: Follow established graceful loading pattern from workspace.rs:327
  - CODE:
    let metadata = match WorkspaceMetadata::load() {
        Ok(meta) => Some(meta),
        Err(JinError::NotFound(_)) => None,
        Err(e) => return Err(e),
    };
  - STORE: Keep metadata variable for P1.M3.T1.S2 (will be used next)
  - PLACEMENT: After line 115 (context.save()?), before println! statements
  - DEPENDENCIES: Task 1 (import must exist)

Task 3: VERIFY EXISTING TESTS STILL PASS
  - RUN: cargo test cli_basic
  - RUN: cargo test mode_scope_workflow
  - RUN: cargo test commands::mode::tests::test_use_mode
  - VERIFY: All existing mode functionality still works
  - EXPECTED: All tests pass (metadata loading is internal, no user-visible changes)
  - DEPENDENCIES: Task 2
```

### Implementation Patterns & Key Details

```rust
// ================== EXISTING CODE (src/commands/mode.rs) ==================
// Lines 86-121 (current use_mode function):

/// Activate a mode
fn use_mode(name: &str) -> Result<()> {
    // Validate mode name
    validate_mode_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Check if mode exists (using _mode suffix)
    let ref_path = format!("refs/jin/modes/{}/_mode", name);
    if !repo.ref_exists(&ref_path) {
        return Err(JinError::NotFound(format!(
            "Mode '{}' not found. Create it with: jin mode create {}",
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

    // Update mode
    context.mode = Some(name.to_string());

    // Save context
    context.save()?;

    // ========== INSERT HERE: Load WorkspaceMetadata ==========
    // NOTE: This is the only code change required for this subtask

    println!("Activated mode '{}'", name);
    println!("Stage files with: jin add --mode");

    Ok(())
}

// ================== CODE TO INSERT ==================

// Load workspace metadata (may not exist yet)
let metadata = match WorkspaceMetadata::load() {
    Ok(meta) => Some(meta),
    Err(JinError::NotFound(_)) => None,  // Fresh workspace - no metadata yet
    Err(e) => return Err(e),              // Other errors should propagate
};
// Metadata is now available for P1.M3.T1.S2 to compare and clear if needed

// ================== PATTERN EXPLANATION ==================
//
// 1. We use match instead of Result::ok() because:
//    - We need to distinguish NotFound (expected) from parse errors (unexpected)
//    - Result::ok() would discard ALL error information
//    - This pattern is used consistently across the codebase
//
// 2. NotFound means:
//    - Fresh workspace (no jin apply has been run yet)
//    - This is a valid state, not an error
//    - Return None to indicate "no metadata to compare"
//
// 3. Other errors (parse, IO) mean:
//    - Corrupted metadata file
//    - Permission issues
//    - These should be reported to the user
//
// 4. Timing (AFTER context.save()):
//    - Mode must be successfully activated first
//    - If mode activation fails, we don't care about metadata
//    - This ensures we only load metadata for the NEW mode
//
// 5. For P1.M3.T1.S2:
//    - Will compare metadata with new mode
//    - If different, will clear the metadata file
//    - This prevents "detached workspace" errors
```

### Integration Points

```yaml
MODE_SWITCHING_WORKFLOW:
  - current_subtask: P1.M3.T1.S1 (load metadata)
  - next_subtask: P1.M3.T1.S2 (compare and clear if different)
  - integration: Metadata loaded here is used in P1.M3.T1.S2

WORKSPACE_METADATA:
  - file: .jin/workspace/last_applied.json
  - format: JSON with timestamp, applied_layers, files
  - loaded via: WorkspaceMetadata::load()
  - path resolution: WorkspaceMetadata::default_path()

ERROR_HANDLING:
  - NotFound: Expected (fresh workspace)
  - Parse errors: Propagate to user
  - IO errors: Propagate to user

FUTURE_WORK:
  - P1.M3.T1.S2: Compare metadata.applied_layers with new mode
  - P1.M3.T1.S2: Clear metadata if modes differ
  - P1.M3.T2.S1: Add similar logic to scope use handler
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after code modification - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
# If clippy warns about unused variable, that's expected (will be used in P1.M3.T1.S2)
# Use #[allow(dead_code)] or let _metadata = ... if needed
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build, no errors
```

### Level 3: Unit Tests (Component Validation)

```bash
# Run mode-specific tests
cargo test commands::mode::tests::    # All mode module tests
cargo test test_use_mode              # Specific use_mode test
cargo test test_use_mode_nonexistent  # Error handling test

# Run related workflow tests
cargo test mode_scope_workflow        # Mode/scope integration tests

# Run with output for debugging
cargo test commands::mode:: -- --nocapture

# Expected: All tests pass
# Behavior should be identical (metadata loading is internal-only)
```

### Level 4: Integration Testing (System Validation)

```bash
# Full test suite
cargo test

# Manual verification (in temporary directory)
cd $(mktemp -d)
git init
jin init
jin mode create testmode
jin mode use testmode
# Should activate mode with no errors
# Metadata loading happens internally (no user-visible change)

# Verify metadata handling works
ls .jin/workspace/last_applied.json 2>/dev/null && echo "Metadata exists" || echo "No metadata (expected)"
# Both states are valid - NotFound is handled gracefully

# Expected: Mode switching works, no errors, metadata loaded (or not) gracefully
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings (or only expected unused var warning)
- [ ] `cargo build` succeeds
- [ ] `cargo test commands::mode::` all tests pass
- [ ] `cargo test` all tests pass (including existing)

### Feature Validation

- [ ] `use crate::staging::metadata::WorkspaceMetadata;` import added
- [ ] Metadata loading added after `context.save()?`
- [ ] `JinError::NotFound(_)` case returns `None`
- [ ] Other errors propagate with `return Err(e)`
- [ ] Metadata stored as variable for P1.M3.T1.S2
- [ ] No user-visible behavior changes (this is internal prep work)

### Code Quality Validation

- [ ] Follows existing codebase pattern (workspace.rs:327, status.rs:169)
- [ ] Error handling distinguishes NotFound from other errors
- [ ] Metadata loading happens AFTER mode activation
- [ ] Variable naming follows Rust conventions (metadata)
- [ ] Code placement is logical (after save, before output)

### Documentation & Deployment

- [ ] Code is self-documenting with clear intent
- [ ] Pattern is consistent with other metadata loading sites
- [ ] Ready for P1.M3.T1.S2 to use the loaded metadata
- [ ] No breaking changes to existing functionality

---

## Anti-Patterns to Avoid

- **Don't** load metadata BEFORE `context.save()` - mode must be activated first
- **Don't** use `Result::ok()` - it discards error information needed for proper handling
- **Don't** treat `NotFound` as an error - it's a valid state (fresh workspace)
- **Don't** propagate all errors - only non-NotFound errors should fail the operation
- **Don't** clear metadata in this subtask - that's P1.M3.T1.S2's job
- **Don't** add user-visible messages - this is internal preparation only
- **Don't** skip the import - `use crate::staging::metadata::WorkspaceMetadata;` is required
- **Don't** change the function signature - return type stays `Result<()>`
- **Don't** add println! statements - metadata loading is silent and internal

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single-file change**: Only `src/commands/mode.rs` needs modification
- **Well-established pattern**: 5+ examples of identical pattern in codebase
- **No new types**: Uses existing `WorkspaceMetadata` and `JinError::NotFound`
- **Isolated change**: Loading is internal, no user-visible impact
- **Clear placement**: Insert point is unambiguous (after `context.save()`)
- **Comprehensive examples**: workspace.rs, status.rs, diff.rs all use this pattern
- **No dependencies**: Independent of other subtasks (output used by P1.M3.T1.S2)
- **Test-friendly**: Can verify with existing test suite

**Zero Risk Factors**:
- Adding metadata loading cannot break existing mode switching
- NotFound is explicitly handled (graceful degradation)
- Other errors propagate correctly (user sees real issues)
- Code change is minimal (7 lines of code)
- All existing tests should pass without modification

**Current Status**: Ready for implementation - all context gathered, pattern identified, implementation is straightforward

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from this PRP creation):
- `plan/P1M3T1S1/research/` - Directory for any additional research findings

**Background Documentation**:
- `plan/docs/fix_specifications.md` - Fix 3: Mode Switching UX context
- `src/commands/mode.rs` - use_mode() function to modify (lines 86-121)
- `src/staging/metadata.rs` - WorkspaceMetadata::load() implementation
- `src/core/error.rs` - JinError::NotFound definition

**Pattern References** (graceful metadata loading):
- `src/staging/workspace.rs:327` - validate_workspace_attached()
- `src/commands/status.rs:169` - check_workspace_state()
- `src/commands/diff.rs:306` - diff_workspace_vs_workspace_active()
- `src/commands/apply.rs:468` - check_workspace_dirty()

**External Research**:
- [Effective Rust - Transforms](https://effective-rust.com/transform.html) - Result/Option patterns
- [Rust Book - Result](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) - Official documentation
- [Medium - Result and Option](https://medium.com/@indrajit7448/mastering-result-and-option-the-simple-secrets-to-idiomatic-rust-error-handling-ae6b5dcfe0b4) - Comprehensive guide

---

## Implementation Status Note

**Ready to implement**: This PRP provides complete context for loading `WorkspaceMetadata` in the `use_mode()` function.

**What this creates**:
- Modified: `src/commands/mode.rs` (use_mode function only)
- No new files or types

**Implementation order**:
1. Add `use crate::staging::metadata::WorkspaceMetadata;` import
2. Insert metadata loading after `context.save()?` at line 115
3. Follow the established graceful loading pattern
4. Run `cargo test` to verify existing tests pass

**Post-implementation verification**:
- All existing mode tests pass
- Manual test: `jin mode use <name>` works without errors
- Metadata variable is available for P1.M3.T1.S2 to use

**Next subtask** (P1.M3.T1.S2):
- Will use the loaded metadata to compare modes
- Will clear metadata if modes differ
- Will add user-visible messaging about metadata clearing
