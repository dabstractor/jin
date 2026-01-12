# PRP: Integrate Validation into Destructive Operations

**Task ID**: P1.M3.T3
**Work Item Title**: Integrate Validation into Destructive Operations
**Milestone**: P1.M3 - Detached Workspace State Detection

---

## Goal

**Feature Goal**: Integrate `validate_workspace_attached()` into destructive operations (`reset --hard` and `apply --force`) to prevent operations that could create or exacerbate detached workspace states, providing early detection and preventing data loss.

**Deliverable**: Modified command implementations in `src/commands/reset.rs` and `src/commands/apply.rs` that call workspace validation before executing destructive operations, returning `DetachedWorkspace` errors when the workspace is not properly attached.

**Success Definition**:
- `jin reset --hard` is rejected with clear error message when workspace is detached
- `jin apply --force` is rejected with clear error message when workspace is detached
- Error messages include actionable recovery hints
- All existing unit tests continue to pass
- New integration tests verify validation behavior

---

## User Persona

**Target User**: Developer using Jin for configuration management

**Use Case**: A developer attempts to run destructive operations (reset --hard or apply --force) on a workspace that has been modified outside of Jin operations or has invalid layer references

**User Journey**:
1. Developer's workspace enters a detached state (files modified externally, layer refs deleted, or mode/scope deleted)
2. Developer attempts to run `jin reset --hard` or `jin apply --force`
3. Jin detects the detached state BEFORE performing destructive operation
4. Jin displays clear error message explaining the problem AND recovery steps
5. Developer runs suggested recovery command (e.g., `jin apply` to restore)

**Pain Points Addressed**:
- **Data Loss Prevention**: Prevents irreversible destructive operations on already-damaged workspaces
- **Early Detection**: Catches detached states before they're compounded by destructive operations
- **Clear Recovery**: Error messages include actionable next steps instead of cryptic failures

---

## Why

- **Data Safety**: Destructive operations on detached workspaces can cause irreversible data loss. Validating first prevents this.
- **PRD Compliance**: Enforces the PRD non-negotiable invariant that "the workspace must always be attached to a valid layer commit"
- **User Experience**: Early error detection with clear recovery hints is significantly better than cryptic failures or silent corruption
- **Integration Point**: This task builds on P1.M3.T2 (validation logic implementation) and feeds into P1.M3.T4 (repair command enhancements) and P1.M3.T5 (integration tests)

---

## What

### Behavior Changes

**Before Integration**:
- `jin reset --hard` executes without checking workspace state, potentially deleting files in a detached workspace
- `jin apply --force` executes without checking workspace state, potentially overwriting files in a detached workspace

**After Integration**:
- Both commands call `validate_workspace_attached()` BEFORE performing any destructive operations
- If workspace is detached, commands return `JinError::DetachedWorkspace` with:
  - Workspace commit (if detectable)
  - Expected layer ref based on active context
  - Human-readable details about what's wrong
  - Actionable recovery hint

**Important Note - Checkout Command**: The original work item mentions integrating validation into a `checkout` command. However, **no checkout command exists in the Jin codebase**. This PRP covers only the existing destructive operations: `reset --hard` and `apply --force`.

### Success Criteria

- [ ] `reset --hard` calls `validate_workspace_attached()` before any destructive operations
- [ ] `apply --force` calls `validate_workspace_attached()` before any destructive operations
- [ ] Both commands reject operations with `DetachedWorkspace` error when validation fails
- [ ] Error messages include recovery hints
- [ ] All existing unit tests pass
- [ ] New integration tests verify validation rejection scenarios
- [ ] Validation is NOT called for non-destructive operations (e.g., `reset --soft`, `reset --mixed`, `apply` without `--force`)

---

## All Needed Context

### Context Completeness Check

**Validation**: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"

- [x] Exact file paths and line numbers for all modifications
- [x] Complete function signatures to call
- [x] Error type definitions
- [x] Test patterns and fixtures
- [x] Integration points with existing code
- [x] Scope boundaries and constraints

### Documentation & References

```yaml
# MUST READ - Validation Function Implementation
- file: src/staging/workspace.rs
  lines: 328-399
  why: Complete validate_workspace_attached() function implementation
  critical: |
    Function returns Result<()>, returns Ok(()) for fresh workspaces (no metadata),
    checks three conditions in order: file mismatches, missing refs, invalid context.
    Each error returns JinError::DetachedWorkspace with specific details and recovery hints.
  signature: |
    pub fn validate_workspace_attached(
        context: &ProjectContext,
        repo: &JinRepo,
    ) -> Result<()>

# MUST READ - Error Type Definition
- file: src/core/error.rs
  lines: 38-54
  why: DetachedWorkspace error variant structure
  critical: |
    Error has four fields: workspace_commit (Option<String>), expected_layer_ref (String),
    details (String), recovery_hint (String). Error message auto-formats with these fields.
  pattern: |
    #[error(
        "Workspace is in a detached state.\n\
        {details}\n\
        \n\
        Recovery: {recovery_hint}"
    )]
    DetachedWorkspace { workspace_commit, expected_layer_ref, details, recovery_hint }

# MUST READ - Reset Command Implementation
- file: src/commands/reset.rs
  lines: 35-109
  why: Current execute() function structure for reset command
  critical: |
    ResetMode enum (Soft/Mixed/Hard). Only Hard mode is destructive and needs validation.
    Confirmation prompt at lines 66-79, but validation should happen BEFORE confirmation.
    Key: validate AFTER loading context (line 46) and repo, BEFORE confirmation prompt.
  gotcha: |
    Validation should happen BEFORE user confirmation - don't prompt for confirmation
    if the operation will be rejected anyway. Insert validation at line 52-53 (after
    determining target layer, before loading staging).

# MUST READ - Apply Command Implementation
- file: src/commands/apply.rs
  lines: 96-210
  why: Current execute() function structure for apply command
  critical: |
    Only needs validation when --force flag is set (line 107: if !args.force check).
    Current dirty check at lines 106-111. Validation should happen AFTER dirty check
    but BEFORE opening repo and applying layers. Insert at line 112-113.
  gotcha: |
    Only validate when --force is used. Regular apply operations don't need workspace
    validation since they're restoring workspace state, not performing destructive operations.

# MUST READ - Test Patterns
- file: tests/workspace_validation.rs
  why: Complete integration test patterns for workspace validation
  pattern: |
    Uses TestFixture from tests/common/fixtures.rs for isolation.
    Creates Jin repository with JinRepo::create_at().
    Sets JIN_DIR environment variable for test isolation.
    Tests error conditions and verifies error variant types.

# MUST READ - Test Fixtures
- file: tests/common/fixtures.rs
  lines: 1-100
  why: TestFixture implementation for isolated test environments
  pattern: |
    let fixture = TestFixture::new().unwrap();
    fixture.set_jin_dir();  // CRITICAL: set before any Jin operations
    std::env::set_current_dir(fixture.path()).unwrap();

# REFERENCE - Module Exports
- file: src/staging/mod.rs
  line: 18
  why: validate_workspace_attached is already exported from staging module
  pattern: |
    use crate::staging::validate_workspace_attached;

# REFERENCE - ProjectContext Loading Pattern
- file: src/commands/reset.rs
  lines: 45-50
  why: Standard pattern for loading ProjectContext with error handling
  pattern: |
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

# REFERENCE - JinRepo Opening Pattern
- file: src/commands/apply.rs
  line: 114
  why: Standard pattern for opening Jin repository
  pattern: |
    let repo = JinRepo::open()?;
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── reset.rs           # MODIFY: Add validation to execute() function
│   ├── apply.rs           # MODIFY: Add validation to execute() function
│   └── mod.rs             # No changes needed
├── staging/
│   ├── workspace.rs       # READ: Contains validate_workspace_attached()
│   └── mod.rs             # READ: Already exports validate_workspace_attached
├── core/
│   ├── error.rs           # READ: Contains DetachedWorkspace error variant
│   └── config.rs          # READ: Contains ProjectContext type
├── git/
│   └── repo.rs            # READ: Contains JinRepo type
├── cli/
│   └── args.rs            # READ: Contains ResetArgs, ApplyArgs
└── lib.rs                 # No changes needed

tests/
├── workspace_validation.rs    # REFERENCE: Test patterns
├── destructive_validation.rs  # CREATE: New integration tests for this task
└── common/
    ├── fixtures.rs            # REFERENCE: TestFixture pattern
    └── mod.rs                 # No changes needed
```

### Desired Codebase Tree (Files to Modify)

```bash
# MODIFY: src/commands/reset.rs
# Changes:
# - Add import: use crate::staging::validate_workspace_attached;
# - In execute() function, after line 53 (after determining target layer):
#   let repo = JinRepo::open()?;
#   validate_workspace_attached(&context, &repo)?;
# - Validation happens BEFORE confirmation prompt

# MODIFY: src/commands/apply.rs
# Changes:
# - Add import: use crate::staging::validate_workspace_attached;
# - After line 111 (after dirty check), add validation:
#   // Validate workspace state before destructive apply
#   let repo = JinRepo::open()?;
#   validate_workspace_attached(&context, &repo)?;
# - Move existing JinRepo::open()? call from line 114 to before validation

# CREATE: tests/destructive_validation.rs
# New integration tests for:
# - test_reset_hard_rejected_when_files_modified
# - test_reset_hard_rejected_when_layer_refs_missing
# - test_reset_hard_rejected_when_context_invalid
# - test_apply_force_rejected_when_files_modified
# - test_apply_force_rejected_when_layer_refs_missing
# - test_apply_force_rejected_when_context_invalid
# - test_reset_soft_skips_validation
# - test_apply_without_force_skips_validation
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Validation function signature
// validate_workspace_attached requires BOTH &ProjectContext AND &JinRepo
// You must load BOTH before calling validation
validate_workspace_attached(&context, &repo)?;

// CRITICAL: Early exit for fresh workspaces
// validate_workspace_attached returns Ok(()) if no metadata exists
// This means fresh/uninitialized workspaces pass validation automatically
// DO NOT add additional checks for fresh workspaces

// CRITICAL: Only validate destructive operations
// reset --soft: NO validation (not destructive)
// reset --mixed: NO validation (not destructive)
// reset --hard: YES validation (destructive)
// apply (without --force): NO validation (restores workspace, not destructive)
// apply --force: YES validation (overwrites workspace, destructive)

// CRITICAL: Validation timing in reset command
// Insert validation AFTER loading context and determining target layer
// Insert BEFORE loading staging and confirmation prompt
// This prevents prompting user for an operation that will be rejected

// CRITICAL: Validation timing in apply command
// Insert validation AFTER workspace dirty check
// Insert BEFORE layer merging and file application
// Only validate when args.force is true

// CRITICAL: Error propagation
// validate_workspace_attached returns Result<()>
// Use ? operator to propagate DetachedWorkspace error
// DO NOT catch and re-wrap the error - let it propagate naturally

// CRITICAL: Test isolation
// Always use fixture.set_jin_dir() BEFORE any Jin operations
// Always set std::env::set_current_dir(fixture.path())
// Always clean up Git locks in Drop implementation
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models are required. This task uses existing types:

- `JinError::DetachedWorkspace` - Already defined in `src/core/error.rs`
- `ProjectContext` - Already defined in `src/core/config.rs`
- `JinRepo` - Already defined in `src/git/repo.rs`
- `validate_workspace_attached()` - Already implemented in `src/staging/workspace.rs`

### Implementation Tasks (Dependency-Ordered)

```yaml
Task 1: MODIFY src/commands/reset.rs - Add import for validation function
  - ADD import at top of file: use crate::staging::validate_workspace_attached;
  - FOLLOW pattern: Existing imports in file (lines 1-9)
  - PLACEMENT: After use crate::staging::{remove_from_managed_block, StagedEntry, StagingIndex};
  - DEPENDENCIES: None

Task 2: MODIFY src/commands/reset.rs - Add validation call in execute()
  - INSERT after line 53 (after determining target layer, before loading staging)
  - IMPLEMENT:
    ```rust
    // 3.5. Validate workspace is attached before destructive operation
    let repo = JinRepo::open()?;
    validate_workspace_attached(&context, &repo)?;
    ```
  - FOLLOW pattern: Context loading pattern (lines 45-50)
  - NAMING: Use exact variable names: context, repo
  - CRITICAL: Validation happens BEFORE confirmation prompt (line 66)
  - DEPENDENCIES: Task 1 (must have import)

Task 3: MODIFY src/commands/reset.rs - Adjust existing JinRepo::open() calls
  - FIND: Any later calls to JinRepo::open()? in the function
  - REMOVE: Remove redundant JinRepo::open()? calls since we already have repo
  - UPDATE: Change any repo operations to use the already-loaded repo variable
  - DEPENDENCIES: Task 2 (repo variable must exist)

Task 4: MODIFY src/commands/apply.rs - Add import for validation function
  - ADD import at top of file: use crate::staging::validate_workspace_attached;
  - FOLLOW pattern: Existing imports (lines 1-14)
  - PLACEMENT: After use crate::staging::{ensure_in_managed_block, WorkspaceMetadata};
  - DEPENDENCIES: None

Task 5: MODIFY src/commands/apply.rs - Add validation call in execute()
  - INSERT after line 111 (after dirty check, before layer operations)
  - IMPLEMENT:
    ```rust
    // 2.5. Validate workspace state before destructive apply (only with --force)
    if args.force {
        let repo = JinRepo::open()?;
        validate_workspace_attached(&context, &repo)?;
    }
    ```
  - FOLLOW pattern: Dirty check pattern (lines 106-111)
  - CRITICAL: Only validate when args.force is true
  - DEPENDENCIES: Task 4 (must have import)

Task 6: MODIFY src/commands/apply.rs - Remove redundant JinRepo::open() call
  - FIND: Existing JinRepo::open()? at line 114
  - REPLACE: Move JinRepo::open()? into the validation block from Task 5
  - UPDATE: Use the repo variable from validation for subsequent operations
  - DEPENDENCIES: Task 5 (repo variable must be created in validation block)

Task 7: CREATE tests/destructive_validation.rs
  - IMPLEMENT: Integration tests for validation in reset --hard
  - IMPLEMENT: Integration tests for validation in apply --force
  - IMPLEMENT: Tests confirming non-destructive operations skip validation
  - FOLLOW pattern: tests/workspace_validation.rs test structure
  - USE: TestFixture from tests/common/fixtures.rs
  - NAMING: test_reset_hard_rejected_when_files_modified, test_apply_force_rejected_when_*
  - COVERAGE: All three detachment conditions (file mismatch, missing refs, invalid context)
  - PLACEMENT: New test file in tests/ directory
  - DEPENDENCIES: Tasks 1-6 (implementation must be complete)

Task 8: VERIFY existing unit tests still pass
  - RUN: cargo test --lib (unit tests in src/)
  - VERIFY: All existing tests in src/commands/reset.rs pass
  - VERIFY: All existing tests in src/commands/apply.rs pass
  - FIX: Any test failures due to validation being added
  - DEPENDENCIES: Tasks 1-6 (implementation must be complete)
```

### Implementation Patterns & Key Details

```rust
// PATTERN 1: Validation integration in reset command
// File: src/commands/reset.rs
// Location: After determining target layer, before confirmation

pub fn execute(args: ResetArgs) -> Result<()> {
    // 1. Determine reset mode
    let mode = if args.soft { ResetMode::Soft }
        else if args.hard { ResetMode::Hard }
        else { ResetMode::Mixed };

    // 2. Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. Determine target layer
    let layer = determine_target_layer(&args, &context)?;

    // 3.5. NEW: Validate workspace is attached before destructive operation
    // CRITICAL: Only validate for Hard mode (destructive)
    if mode == ResetMode::Hard {
        let repo = JinRepo::open()?;
        validate_workspace_attached(&context, &repo)?;
        // Note: Keep repo variable for potential later use
    }

    // 4. Load staging
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // ... rest of function continues unchanged
}

// PATTERN 2: Validation integration in apply command
// File: src/commands/apply.rs
// Location: After dirty check, before layer merging

pub fn execute(args: ApplyArgs) -> Result<()> {
    // 1. Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 2. Check workspace dirty (unless --force)
    if !args.force && check_workspace_dirty()? {
        return Err(JinError::Other(
            "Workspace has uncommitted changes. Use --force to override.".to_string(),
        ));
    }

    // 2.5. NEW: Validate workspace state before destructive apply (only with --force)
    let repo = if args.force {
        let r = JinRepo::open()?;
        validate_workspace_attached(&context, &r)?;
        r
    } else {
        JinRepo::open()?
    };

    // 3. Open repository (now using repo from above)
    // 4. Determine applicable layers
    // ... rest of function continues using repo variable
}

// PATTERN 3: Integration test structure
// File: tests/destructive_validation.rs

#[test]
fn test_reset_hard_rejected_when_files_modified() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create a file and track it in metadata
    let file_path = "config.txt";
    let original_content = b"original content";
    fs::write(fixture.path().join(file_path), original_content).unwrap();

    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let oid = repo.inner().blob(original_content).unwrap();
    let hash = oid.to_string();

    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.add_file(PathBuf::from(file_path), hash);
    metadata.save().unwrap();

    // Stage the file
    jin::commands::execute jin_add...

    // Modify file externally to create detached state
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt reset --hard, should be rejected
    let result = jin_cmd()
        .args(["reset", "--hard"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    result
        .failure()
        .stderr(predicate::str::contains("detached"))
        .stderr(predicate::str::contains("modified"));
}

// GOTCHA: Validation order matters
// - In reset: Validate BEFORE prompting user (don't prompt for rejected operation)
// - In apply: Validate AFTER dirty check but BEFORE layer operations

// GOTCHA: Only validate destructive operations
// - reset --soft: Skip validation
// - reset --mixed: Skip validation
// - reset --hard: Validate
// - apply: Skip validation (restores workspace)
// - apply --force: Validate (overwrites workspace)

// GOTCHA: Error propagation
// - Use ? operator to propagate DetachedWorkspace error
// - Error auto-formats with details and recovery hint
// - DO NOT catch and re-wrap
```

### Integration Points

```yaml
RESET COMMAND:
  - file: src/commands/reset.rs
  - location: execute() function, line ~53-54
  - insert_after: determine_target_layer() call
  - insert_before: StagingIndex::load() call
  - condition: Only when mode == ResetMode::Hard
  - pattern: |
      if mode == ResetMode::Hard {
          let repo = JinRepo::open()?;
          validate_workspace_attached(&context, &repo)?;
      }

APPLY COMMAND:
  - file: src/commands/apply.rs
  - location: execute() function, line ~111-113
  - insert_after: check_workspace_dirty() call
  - insert_before: Layer operations
  - condition: Only when args.force is true
  - pattern: |
      let repo = if args.force {
          let r = JinRepo::open()?;
          validate_workspace_attached(&context, &r)?;
          r
      } else {
          JinRepo::open()?
      };

IMPORTS:
  - file: src/commands/reset.rs
  - add: use crate::staging::validate_workspace_attached;
  - placement: After existing staging imports

  - file: src/commands/apply.rs
  - add: use crate::staging::validate_workspace_attached;
  - placement: After existing staging imports

TESTS:
  - file: tests/destructive_validation.rs (NEW)
  - use: TestFixture from tests/common/fixtures.rs
  - use: assert_cmd for CLI testing
  - use: predicates::str::contains for output verification
  - pattern: Follow tests/workspace_validation.rs structure
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --bin jin                      # Fast syntax check
cargo clippy --bin jin -- -D warnings      # Lint with warnings as errors

# Format check
cargo fmt --all -- --check                 # Check formatting
cargo fmt --all                            # Auto-format if needed

# Expected: Zero errors, zero warnings. If errors exist, READ output and fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test unit tests in modified command files
cargo test --lib reset::tests              # Unit tests for reset command
cargo test --lib apply::tests              # Unit tests for apply command

# Full library unit test suite
cargo test --lib

# Expected: All tests pass. If failing, debug root cause and fix.
```

### Level 3: Integration Tests (System Validation)

```bash
# Run integration tests for workspace validation
cargo test --test workspace_validation     # Existing validation tests
cargo test --test destructive_validation   # New tests for this task

# Full integration test suite
cargo test --test

# Expected: All tests pass. Check specifically for:
# - reset --hard rejected when files modified
# - reset --hard rejected when layer refs missing
# - reset --hard rejected when context invalid
# - apply --force rejected when files modified
# - apply --force rejected when layer refs missing
# - apply --force rejected when context invalid
# - reset --soft still works (no validation)
# - apply without --force still works (no validation)
```

### Level 4: Manual CLI Testing (End-to-End Validation)

```bash
# Setup test environment
TEST_DIR=$(mktemp -d)
cd $TEST_DIR
jin init
export JIN_DIR="$TEST_DIR/.jin"

# Create a mode and add a file
echo "test" > config.txt
jin add config.txt
jin commit -m "Initial commit"

# Scenario 1: Modify file externally, try reset --hard
echo "modified externally" > config.txt
jin reset --hard
# Expected: Error message about detached workspace, recovery hint to run 'jin apply'

# Scenario 2: Verify apply --force also validates
echo "modified again" > config.txt
jin apply --force
# Expected: Error message about detached workspace, recovery hint to run 'jin apply'

# Scenario 3: Verify non-destructive operations work
jin reset --soft
# Expected: Success (no validation for --soft)

jin apply
# Expected: Success (no validation for apply without --force)

# Cleanup
cd -
rm -rf $TEST_DIR
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] Zero compilation errors or warnings

### Feature Validation

- [ ] `reset --hard` calls `validate_workspace_attached()` before destructive operations
- [ ] `apply --force` calls `validate_workspace_attached()` before destructive operations
- [ ] Non-destructive operations (`reset --soft`, `reset --mixed`, `apply` without `--force`) skip validation
- [ ] Detached workspace state triggers `JinError::DetachedWorkspace` with clear message
- [ ] Error messages include recovery hints (e.g., "Run 'jin apply' to restore")
- [ ] All three detachment conditions are detected (file mismatch, missing refs, invalid context)
- [ ] Manual testing confirms expected behavior

### Code Quality Validation

- [ ] Follows existing command structure patterns
- [ ] Uses existing error types (no new error variants)
- [ ] Validation happens at correct point in execution flow
- [ ] Imports follow existing file organization
- [ ] Test code follows existing test patterns (TestFixture, isolation, etc.)
- [ ] No redundant code or duplicate JinRepo::open() calls

### Documentation & Deployment

- [ ] Code comments explain why validation is added
- [ ] Error messages are user-friendly and actionable
- [ ] Test names clearly indicate what scenario they test
- [ ] No new environment variables or configuration required
- [ ] No breaking changes to existing functionality

---

## Anti-Patterns to Avoid

- **Don't validate non-destructive operations**: `reset --soft`, `reset --mixed`, and `apply` (without `--force`) should NOT call validation
- **Don't prompt for rejected operations**: Add validation BEFORE confirmation prompts in `reset --hard`
- **Don't catch and re-wrap errors**: Let `DetachedWorkspace` errors propagate naturally with `?` operator
- **Don't add fresh workspace checks**: `validate_workspace_attached()` already returns `Ok(())` for fresh workspaces
- **Don't duplicate JinRepo::open()**: Reuse the repo variable from validation
- **Don't skip tests**: This is a safety-critical feature - comprehensive tests are essential
- **Don't modify error types**: Use existing `DetachedWorkspace` variant as-is
- **Don't implement checkout command**: It doesn't exist in the codebase and is out of scope

---

## Confidence Score

**One-Pass Implementation Success Likelihood**: **9/10**

**Rationale**:
- **Complete context**: All file paths, line numbers, function signatures, and patterns provided
- **Simple scope**: Only 2 files to modify, no new types or complex logic
- **Clear patterns**: Existing test patterns and command structure to follow
- **Comprehensive validation**: 4-level validation loop with specific commands
- **Risk mitigation**: Clear anti-patterns and gotchas identified

**Remaining risk (1 point deduction)**:
- The checkout command mentioned in the original work item doesn't exist - clarification may be needed
- Integration with existing repo variable usage may require careful refactoring

---

## Appendix: Critical Code Snippets

### validate_workspace_attached Function Signature

```rust
// File: src/staging/workspace.rs, lines 328-331
pub fn validate_workspace_attached(
    context: &ProjectContext,
    repo: &JinRepo,
) -> Result<()>
```

### DetachedWorkspace Error Variant

```rust
// File: src/core/error.rs, lines 38-54
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

### Reset Command execute() Function (Lines 35-109, key sections)

```rust
pub fn execute(args: ResetArgs) -> Result<()> {
    // Lines 36-43: Determine reset mode
    let mode = if args.soft { ResetMode::Soft }
        else if args.hard { ResetMode::Hard }
        else { ResetMode::Mixed };

    // Lines 45-50: Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // Lines 52-53: Determine target layer
    let layer = determine_target_layer(&args, &context)?;

    // *** INSERT VALIDATION HERE (after line 53) ***
    // Only for Hard mode:
    // if mode == ResetMode::Hard {
    //     let repo = JinRepo::open()?;
    //     validate_workspace_attached(&context, &repo)?;
    // }

    // Lines 55-63: Load staging
    // ... rest of function
}
```

### Apply Command execute() Function (Lines 96-210, key sections)

```rust
pub fn execute(args: ApplyArgs) -> Result<()> {
    // Lines 98-104: Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // Lines 106-111: Check workspace dirty (unless --force)
    if !args.force && check_workspace_dirty()? {
        return Err(JinError::Other(
            "Workspace has uncommitted changes. Use --force to override.".to_string(),
        ));
    }

    // *** INSERT VALIDATION HERE (after line 111) ***
    // Only when --force is set:
    // let repo = if args.force {
    //     let r = JinRepo::open()?;
    //     validate_workspace_attached(&context, &r)?;
    //     r
    // } else {
    //     JinRepo::open()?
    // };

    // Line 114: Open repository (now uses repo from above)
    // ... rest of function
}
```
