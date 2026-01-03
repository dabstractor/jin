# PRP: Define Detached Workspace State

---

## Goal

**Feature Goal**: Add the `DetachedWorkspace` error type to `JinError` enum and comprehensively document what constitutes a detached workspace state in Jin's architecture.

**Deliverable**:
1. `DetachedWorkspace` error variant added to `src/core/error.rs`
2. Comprehensive documentation of detached workspace conditions stored in `plan/P1M3T1/research/detached_workspace_conditions.md`
3. Unit tests for the new error variant in `src/core/error.rs` test module

**Success Definition**:
- Error variant compiles without errors
- Error message is user-friendly and actionable
- Documentation clearly defines all detachment conditions with examples
- Unit tests pass and cover error creation and display

## Why

**Business Value and User Impact**:
- **PRD Compliance**: This implements Non-negotiable Invariant #4: "Jin will abort any operation that would create a detached state" (Critical Gap #3 identified in implementation analysis)
- **Data Integrity**: Prevents users from working in undefined workspace states where files don't match any valid layer configuration
- **User Safety**: Provides clear error messages and recovery guidance when workspace becomes inconsistent

**Integration with Existing Features**:
- Foundation for P1.M3.T2 (workspace validation logic implementation)
- Required for P1.M3.T3 (validation integration into destructive operations: reset --hard, apply --force, checkout)
- Enables P1.M3.T4 (repair --check command for detached state detection)

**Problems This Solves**:
- Currently, users can manually modify workspace files outside of Jin operations, creating inconsistent states
- No detection exists for when `WorkspaceMetadata` references deleted commits
- Missing validation for when active context points to deleted modes/scopes

## What

**User-Visible Behavior**: Users attempting destructive operations (reset --hard, apply --force) on a detached workspace will receive a clear error message explaining the detachment and providing recovery guidance.

**Technical Requirements**:
1. Add `DetachedWorkspace` variant to `JinError` enum with structured fields for diagnosis
2. Document the three conditions that cause detachment with concrete examples
3. Ensure error message includes actionable recovery hints

### Success Criteria

- [ ] `DetachedWorkspace` error variant added to `JinError` enum in `src/core/error.rs`
- [ ] Error includes fields: `workspace_commit` (Option<String>), `expected_layer_ref` (String), `recovery_hint` (String)
- [ ] Error message is descriptive and actionable
- [ ] Documentation file created in `plan/P1M3T1/research/detached_workspace_conditions.md`
- [ ] Unit tests added for error display and field access
- [ ] All tests pass: `cargo test --package jin --lib core::error`

---

## All Needed Context

### Context Completeness Check

_Validation: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Yes** - This PRP provides:
- Exact file locations and patterns to follow
- Complete error variant specification with field types
- Specific documentation requirements
- Test patterns used in this codebase
- Links to all relevant architecture documents

### Documentation & References

```yaml
MUST READ - Critical Implementation Context:

- file: src/core/error.rs
  why: Main error type definition - shows exact pattern for adding new error variants
  pattern: thiserror derive macro, #[error] attribute with structured fields, descriptive messages
  gotcha: All error variants must be snake_case or CamelCase consistently (codebase uses CamelCase)
  critical: Lines 1-100 show the JinError enum structure with 15+ existing variants

- file: src/core/error.rs (test module)
  why: Shows test pattern for error variants - use same structure for DetachedWorkspace tests
  pattern: #[cfg(test)] mod tests with test_* functions using assert_eq! for display messages
  section: Lines 140-200 contain error tests

- file: src/staging/workspace.rs
  why: Contains WorkspaceMetadata and workspace operations - understand what "detached" means
  pattern: File I/O with Result<T> return types using JinError
  gotcha: Workspace is .jin/workspace/ directory - this is where detachment is detected

- file: src/core/config.rs
  why: ProjectContext structure defines active context (mode, scope, project)
  pattern: Struct with Option<String> fields, load() method for YAML parsing
  critical: Lines 20-50 show ProjectContext - detachment can occur when active context is invalid

- file: plan/docs/implementation-gaps-analysis.md
  why: Defines Critical Gap #3 - detached workspace detection as PRD compliance issue
  section: "Critical Gap #3: Missing Detached Workspace State Detection"
  critical: States "Zero code exists" - this is greenfield implementation

- file: plan/docs/WORKFLOWS.md
  why: Documents Jin workflows and workspace state management
  section: "Workspace State" and "Destructive Operations"
  critical: "Workspace is never source of truth" - core invariant to maintain

- url: https://git-scm.com/book/en/v2/Git-Internals-Git-References
  why: Git's detached HEAD concept - Jin's "detached workspace" is analogous but different
  insight: Jin's detachment is about workspace-to-layer consistency, not branch references
  gotcha: Don't confuse Git's detached HEAD with Jin's detached workspace - different concepts

- url: https://stackoverflow.com/questions/52221558/programmatically-check-if-head-is-detached
  why: Shows detection patterns - Jin will use similar approach for workspace validation
  pattern: Check state, return structured error with recovery hint
```

### Current Codebase Tree

```bash
src/
├── core/
│   ├── mod.rs              # Module exports
│   ├── error.rs            # MAIN TARGET: Add DetachedWorkspace variant here
│   └── config.rs           # ProjectContext definition (active context)
├── staging/
│   └── workspace.rs        # Workspace operations (future validation location)
└── commands/
    └── reset.rs            # Will use DetachedWorkspace error in P1.M3.T3

tests/
├── error_scenarios.rs      # Integration test patterns for errors
└── common/
    └── fixtures.rs         # Test utilities

plan/
└── P1M3T1/
    ├── PRP.md              # This document
    └── research/           # Store research findings here
        └── detached_workspace_conditions.md  # CREATE: Detachment conditions doc
```

### Desired Codebase Tree (New Files Only)

```bash
# No new files - only modifications to existing files
# Modified files (with additions):
src/core/error.rs          # Add DetachedWorkspace variant + tests
plan/P1M3T1/research/      # Add detached_workspace_conditions.md
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: thiserror requires specific macro usage pattern
// MUST use #[error("...")] attribute with proper formatting
// Fields in braces must match variant field names exactly

// CRITICAL: Error messages should be user-facing and actionable
// Include recovery hints in error, not just "detached workspace"
// Example from BehindRemote error: "Run 'jin pull' to merge remote changes"

// GOTCHA: JinError is #[non_exhaustive] - can add variants without breaking consumers
// But follow existing pattern: descriptive messages, structured fields

// GOTCHA: Use String for commit refs, not git2::Oid - avoid coupling to git2 types
// Workspace commit is Option<String> because workspace might have no valid commit

// PATTERN: Recovery hints should be actionable commands users can run
// Reference existing commands: jin reset, jin repair, jin status
```

---

## Implementation Blueprint

### Data Models and Structure

The error variant follows existing `JinError` patterns:

```rust
// In src/core/error.rs - JinError enum
// Add after line ~36 (after BehindRemote variant)

/// Detached workspace state - workspace doesn't match any valid layer configuration
#[error(
    "Workspace is in a detached state.\n\
    {details}\n\
    \n\
    Recovery: {recovery_hint}"
)]
DetachedWorkspace {
    /// The commit hash the workspace is currently on (if detectable)
    workspace_commit: Option<String>,

    /// The layer ref that was expected based on active context
    expected_layer_ref: String,

    /// Human-readable explanation of why detachment occurred
    details: String,

    /// Actionable recovery suggestion
    recovery_hint: String,
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: CREATE research documentation file
  - CREATE: plan/P1M3T1/research/detached_workspace_conditions.md
  - DOCUMENT: Three detachment conditions with examples
  - INCLUDE: Detection logic descriptions for each condition
  - REFERENCE: ProjectContext, WorkspaceMetadata, layer commit validation
  - PLACEMENT: plan/P1M3T1/research/

Task 2: MODIFY src/core/error.rs - Add DetachedWorkspace variant
  - ADD: DetachedWorkspace variant to JinError enum (after BehindRemote, line ~36)
  - FOLLOW pattern: Existing structured error variants (BehindRemote, MergeConflict)
  - FIELDS: workspace_commit: Option<String>, expected_layer_ref: String, details: String, recovery_hint: String
  - MESSAGE: Multi-line user-friendly error with recovery hint
  - NAMING: CamelCase variant name, snake_case field names
  - PLACEMENT: src/core/error.rs in JinError enum

Task 3: MODIFY src/core/error.rs - Add unit tests
  - ADD: Test function test_detached_workspace_display()
  - VERIFY: Error message formatting is correct
  - TEST: Field accessibility and error creation
  - FOLLOW pattern: Existing error tests in #[cfg(test)] module
  - COVERAGE: Error display, field values, recovery hint inclusion
  - PLACEMENT: src/core/error.rs test module

Task 4: VERIFY compilation and run tests
  - RUN: cargo build --package jin
  - RUN: cargo test --package jin --lib core::error
  - CHECK: No warnings, all tests pass
  - VALIDATE: Error is exported through lib.rs re-exports
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Structured error with multi-line message (see BehindRemote)
// Location: src/core/error.rs, line ~29-36

// Existing pattern to follow:
#[error(
    "Push rejected: local layer '{layer}' is behind remote.\n\
    The remote contains commits you don't have locally.\n\
    Run 'jin pull' to merge remote changes, or use '--force' to overwrite.\n\
    WARNING: --force may cause data loss!"
)]
BehindRemote { layer: String }

// New DetachedWorkspace pattern:
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
}

// USAGE EXAMPLE (for future tasks, not this PRP):
// When workspace files don't match layer merge result:
return Err(JinError::DetachedWorkspace {
    workspace_commit: Some("abc123".to_string()),
    expected_layer_ref: "refs/jin/layers/modes/claude/scopes/default".to_string(),
    details: "Workspace files have been modified outside of Jin operations".to_string(),
    recovery_hint: "Run 'jin reset --hard refs/jin/layers/modes/claude/scopes/default' to restore".to_string(),
});

// GOTCHA: Use \n\ for line continuations in error messages
// This is how thiserror handles multi-line error messages
```

### Integration Points

```yaml
ERROR_MODULE:
  - file: src/core/error.rs
  - modify: JinError enum (add variant after BehindRemote)
  - pattern: Keep variants roughly grouped by category (workspace errors together)

TEST_MODULE:
  - file: src/core/error.rs
  - add_to: #[cfg(test)] mod tests
  - pattern: test_error_display() functions using assert_eq!

DOCUMENTATION:
  - create: plan/P1M3T1/research/detached_workspace_conditions.md
  - content: Three detachment conditions with detection logic
  - audience: Future implementers of P1.M3.T2 (validation logic)

RE-EXPORTS:
  - file: src/lib.rs
  - auto: JinError is already re-exported
  - verify: cargo doc shows new variant
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding error variant - fix before proceeding
cargo build --package jin                   # Compile check
cargo clippy --package jin -- -D warnings   # Lint checking

# Expected: Zero errors, zero warnings. If errors exist:
# 1. Check thiserror macro syntax (braces, quotes, escapes)
# 2. Verify field names match error message placeholders
# 3. Ensure no trailing commas in wrong places
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test error variant specifically
cargo test --package jin --lib core::error::tests::test_detached_workspace_display

# Test all error module tests
cargo test --package jin --lib core::error

# Run with output
cargo test --package jin --lib core::error -- --nocapture

# Expected: All tests pass. If failing:
# 1. Check assert_eq! expected vs actual message format
# 2. Verify error creation syntax
# 3. Check for missing or extra newlines in error message
```

### Level 3: Integration Testing (System Validation)

```bash
# Verify error is properly exported
cargo doc --package jin --no-deps --open

# Check documentation includes new error variant
# Manual: Open docs and verify DetachedWorkspace appears

# Verify compilation of dependent code (even though integration is future tasks)
cargo build --bin jin

# Expected: Clean build, DetachedWorkspace accessible via jin::JinError
```

### Level 4: Documentation Validation

```bash
# Verify research documentation exists and is complete
cat plan/P1M3T1/research/detached_workspace_conditions.md

# Check for:
# - Clear definition of detached workspace
# - Three detachment conditions listed
# - Detection logic for each condition
# - Examples of when each occurs

# Expected: Comprehensive documentation that enables P1.M3.T2 implementation
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `DetachedWorkspace` variant compiles without errors
- [ ] All unit tests pass: `cargo test --package jin --lib core::error`
- [ ] No clippy warnings: `cargo clippy --package jin`
- [ ] Error message format is correct (check with `cargo doc`)
- [ ] All fields are accessible and properly typed

### Feature Validation

- [ ] Error variant includes all four required fields
- [ ] Error message is multi-line and user-friendly
- [ ] Error message includes dynamic recovery hint
- [ ] Documentation file created at `plan/P1M3T1/research/detached_workspace_conditions.md`
- [ ] Documentation covers all three detachment conditions

### Code Quality Validation

- [ ] Follows existing error variant pattern (compare to `BehindRemote`)
- [ ] Field naming consistent with codebase (camelCase variant, snake_case fields)
- [ ] Error message uses `\n\` continuation pattern
- [ ] Unit test follows existing test pattern

### Documentation & Future Integration

- [ ] Research documentation includes concrete examples
- [ ] Documentation provides detection logic for future validation implementation
- [ ] Error variant designed for easy integration in P1.M3.T2 (validation logic)
- [ ] Recovery hint field enables actionable user guidance

---

## Anti-Patterns to Avoid

- **Don't** add git2::Oid type to error fields - use String for decoupling
- **Don't** create a separate error module - use existing `src/core/error.rs`
- **Don't** use single-line error message - follow BehindRemote pattern with `\n\`
- **Don't** skip the research documentation - P1.M3.T2 needs this context
- **Don't** add validation logic in this task - that's P1.M3.T2
- **Don't** make fields pub without need - error struct fields are private by default
- **Don't** use generic error messages - include specific details about detachment cause
- **Don't** forget recovery hint - users need actionable guidance

---

## Appendix: Detachment Conditions Reference

For the research documentation (Task 1), document these three conditions:

### Condition 1: Workspace Files Don't Match Layer Merge Result
- **Cause**: Files manually edited outside of Jin operations
- **Detection**: Compare workspace file hashes to WorkspaceMetadata stored hashes
- **Example**: User edits `.jin/workspace/config.json` directly

### Condition 2: WorkspaceMetadata References Non-Existent Commits
- **Cause**: Jin repository garbage collected, layer commits deleted
- **Detection**: Check if commits in WorkspaceMetadata exist in Jin repository
- **Example**: User runs `git gc --prune=now` in .jin directory

### Condition 3: Active Context References Deleted Modes/Scopes
- **Cause**: Active mode or scope was deleted from Jin repository
- **Detection**: Validate ProjectContext mode/scope references exist in layer tree
- **Example**: User deletes currently active mode with `jin mode delete <active-mode>`

---

## Confidence Score

**8/10** for one-pass implementation success

**Reasoning**:
- ✅ Well-defined scope (single error variant + documentation)
- ✅ Clear pattern to follow (BehindRemote error)
- ✅ No external dependencies or complex integration
- ✅ Comprehensive test patterns exist
- ⚠️ Minor uncertainty: Exact placement in enum (but pattern is clear)

**Validation**: This PRP provides sufficient context for an implementer unfamiliar with the codebase to add the error variant successfully.
