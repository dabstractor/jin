# PRP: Add Detached State Detection to Repair Command

**Task ID**: P1.M3.T4
**Work Item Title**: Add Detached State Detection to Repair Command
**Milestone**: P1.M3 - Detached Workspace State Detection

---

## Goal

**Feature Goal**: Extend the `jin repair` command with a `--check` flag that detects and reports detached workspace states, providing users with a diagnostic tool when they encounter detached workspace errors.

**Deliverable**:
1. Modified `RepairArgs` struct in `src/cli/args.rs` to add `--check` flag
2. Modified `execute()` function in `src/commands/repair.rs` to check workspace state when `--check` is passed
3. Integration tests for the `jin repair --check` command

**Success Definition**:
- `jin repair --check` runs workspace validation and reports results
- The command succeeds (exit code 0) when workspace is attached
- The command reports a clear message when workspace is detached
- All three detachment conditions are properly detected
- The regular `jin repair` command (without `--check`) continues to work unchanged
- All existing tests continue to pass

---

## User Persona

**Target User**: Developer using Jin for configuration management

**Use Case**: A developer encounters a detached workspace error and wants to diagnose the issue without running any repair operations

**User Journey**:
1. Developer encounters a `DetachedWorkspace` error (or suspects workspace is detached)
2. Developer runs `jin repair --check` to diagnose the issue
3. Jin checks the three detachment conditions and reports:
   - "Workspace is properly attached" (success) OR
   - "Workspace is detached: [specific condition details]" (with recovery hint)
4. Developer can then take appropriate recovery action based on the diagnostic output

**Pain Points Addressed**:
- **Diagnostic Visibility**: Users can check workspace state without running repair operations
- **Error Understanding**: Clear explanation of which detachment condition is detected
- **Safe Exploration**: No changes are made - it's a read-only diagnostic operation
- **Recovery Guidance**: Output includes actionable recovery hints

---

## Why

- **User-Friendly Diagnostics**: The `repair --check` command provides a safe way to diagnose workspace issues without modifying anything
- **PRD Compliance**: Completes the detached workspace state detection milestone by providing user-facing diagnostics
- **Integration with P1.M3.T2**: Uses the `validate_workspace_attached()` function implemented in that task
- **Enables Recovery**: Users can understand their workspace state before taking recovery actions
- **No Side Effects**: The `--check` flag is read-only, making it safe to run in any situation

---

## What

### Behavior Changes

**Before This Task**:
- `jin repair` performs 7 integrity checks and repairs issues
- `jin repair --dry-run` shows what would be repaired
- No dedicated workspace state check command exists

**After This Task**:
- `jin repair --check` runs workspace validation and reports the result
- The command exits successfully (0) if workspace is attached
- The command reports detachment details if workspace is detached (still exits 0 for check mode)
- Output format matches other repair checks (with checkmark and message)

### Success Criteria

- [ ] `--check` flag added to `RepairArgs` struct in `src/cli/args.rs`
- [ ] `execute()` function in `src/commands/repair.rs` handles `--check` flag
- [ ] Workspace validation called when `--check` is true
- [ ] Output shows "Workspace attachment: ✓" when attached
- [ ] Output shows "Workspace attachment: ✗ [details]" when detached
- [ ] Regular `jin repair` continues to work unchanged
- [ ] All existing repair tests still pass
- [ ] New integration tests for `--check` flag added

---

## All Needed Context

### Context Completeness Check

**Validation**: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"

- [x] Exact file paths and line numbers for modifications
- [x] Complete struct definition for adding the flag
- [x] Validation function signature to call
- [x] Output format patterns to follow
- [x] Test patterns and fixtures
- [x] Integration points with existing code

### Documentation & References

```yaml
# MUST READ - Validation Function to Call
- file: src/staging/workspace.rs
  lines: 328-399
  why: Complete validate_workspace_attached() function implementation
  signature: |
    pub fn validate_workspace_attached(
        context: &ProjectContext,
        repo: &JinRepo,
    ) -> Result<()>
  critical: |
    Returns Ok(()) for attached workspaces (including fresh ones with no metadata).
    Returns Err(JinError::DetachedWorkspace) with details and recovery hint when detached.
    Checks three conditions: file mismatch, missing refs, invalid context.

# MUST READ - Repair Command Implementation
- file: src/commands/repair.rs
  lines: 1-200
  why: Complete repair command structure showing check pattern and output format
  pattern: |
    Each check function: print!("Checking... ") -> do check -> println!("✓" or "✗")
    Issues tracked in issues_found Vec<String>, displayed in summary at end.
  critical: |
    Key locations:
    - Line 10: Import crate::cli::RepairArgs
    - Line 30-127: execute() function with check/fix pattern
    - Line 80-79: Project context check as an example of a check function

# MUST READ - RepairArgs Struct
- file: src/cli/args.rs
  lines: 194-200
  why: Current RepairArgs definition - add check field here
  pattern: |
    #[derive(Args, Debug)]
    pub struct RepairArgs {
        #[arg(long)]
        pub dry_run: bool,
    }
  gotcha: |
    Use #[arg(long)] attribute for boolean flags. No short option needed.

# MUST READ - Error Type
- file: src/core/error.rs
  lines: 38-54
  why: DetachedWorkspace error variant for understanding error format
  structure: |
    JinError::DetachedWorkspace {
        workspace_commit: Option<String>,
        expected_layer_ref: String,
        details: String,
        recovery_hint: String,
    }

# REFERENCE - ProjectContext Loading
- file: src/commands/repair.rs
  lines: 571-636
  why: check_project_context() function shows how to load ProjectContext
  pattern: |
    let context_path = ProjectContext::default_path();
    if !context_path.parent().map(|p| p.exists()).unwrap_or(false) {
        println!("✓ (not initialized)");
        return;
    }
    match ProjectContext::load() {
        Ok(ctx) => { /* use ctx */ }
        Err(_) => { /* handle error */ }
    }

# REFERENCE - Test Patterns
- file: tests/common/fixtures.rs
  lines: 1-100
  why: TestFixture pattern for isolated test environments
  pattern: |
    let fixture = TestFixture::new().unwrap();
    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

# REFERENCE - Unit Tests in repair.rs
- file: src/commands/repair.rs
  lines: 661-903
  why: Test patterns specific to repair command
  pattern: |
    Uses DirGuard for directory management, setup_isolated_test() for JIN_DIR setup
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── repair.rs          # MODIFY: Add --check handling in execute()
│   └── mod.rs             # No changes needed
├── cli/
│   ├── args.rs            # MODIFY: Add check field to RepairArgs
│   └── mod.rs             # No changes needed
├── staging/
│   ├── workspace.rs       # READ: Contains validate_workspace_attached()
│   └── mod.rs             # No changes needed
├── core/
│   ├── error.rs           # READ: Contains DetachedWorkspace error
│   └── config.rs          # READ: Contains ProjectContext type
├── git/
│   └── repo.rs            # READ: Contains JinRepo type
└── lib.rs                 # No changes needed

tests/
├── repair_check.rs        # CREATE: Integration tests for --check flag
└── common/
    ├── fixtures.rs        # REFERENCE: TestFixture pattern
    └── mod.rs             # No changes needed
```

### Desired Codebase Tree (Files to Modify)

```bash
# MODIFY: src/cli/args.rs (lines 194-200)
# Add to RepairArgs struct:
#   /// Check workspace state without making repairs
#   #[arg(long)]
#   pub check: bool,

# MODIFY: src/commands/repair.rs
# In execute() function (around line 30-127):
# - Add early check block after line 31 (after initial println!)
# - When args.check is true:
#   1. Load ProjectContext
#   2. Open JinRepo
#   3. Call validate_workspace_attached()
#   4. Print result with ✓/✗ pattern
#   5. Return Ok(()) (don't run other checks)

# CREATE: tests/repair_check.rs
# Integration tests for:
# - test_repair_check_success_when_attached
# - test_repair_check_reports_detached_file_mismatch
# - test_repair_check_reports_detached_missing_refs
# - test_repair_check_reports_detached_invalid_context
# - test_repair_without_check_runs_all_checks
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: --check should be mutually exclusive with --dry-run
// --check is a diagnostic-only mode, --dry-run shows what would be repaired
// They can be used together (check shows state, dry-run shows repairs)

// CRITICAL: --check should run BEFORE any other checks
// Insert at the beginning of execute(), right after initial println!
// This ensures users get immediate workspace state feedback

// GOTCHA: validate_workspace_attached requires BOTH ProjectContext AND JinRepo
// Order of loading matters:
// 1. Load ProjectContext (may not exist - that's OK)
// 2. Open JinRepo (must exist for Jin to work)
// 3. Call validate_workspace_attached(&context, &repo)?

// CRITICAL: Fresh workspace handling
// validate_workspace_attached returns Ok(()) if no metadata exists
// This means fresh/uninitialized workspaces are "attached" (nothing to detach from)

// GOTCHA: ProjectContext::load() can return NotInitialized error
// Handle this gracefully - if not initialized, workspace can't be "detached"
// Pattern: match ProjectContext::load() { Ok(ctx) => ctx, Err(JinError::NotInitialized) => return Ok(()), ... }

// PATTERN: Print output matching other repair checks
// Use: print!("Checking workspace attachment... ");
// Then: println!("✓") for success, println!("✗") with details for failure

// CRITICAL: Don't modify issues_found for --check mode
// The --check flag is informational only, doesn't contribute to repair summary
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models are required. This task adds a boolean flag to an existing struct:

```rust
// In src/cli/args.rs
#[derive(Args, Debug)]
pub struct RepairArgs {
    /// Show what would be repaired
    #[arg(long)]
    pub dry_run: bool,

    /// NEW: Check workspace state without making repairs
    #[arg(long)]
    pub check: bool,
}
```

### Implementation Tasks (Dependency-Ordered)

```yaml
Task 1: MODIFY src/cli/args.rs - Add check flag to RepairArgs
  - ADD field to RepairArgs struct (after dry_run field)
  - IMPLEMENT:
    ```rust
    /// Check workspace state without making repairs
    #[arg(long)]
    pub check: bool,
    ```
  - FOLLOW pattern: dry_run field definition (line 197-199)
  - NAMING: check (simple, descriptive)
  - PLACEMENT: After dry_run field in RepairArgs
  - DEPENDENCIES: None

Task 2: MODIFY src/commands/repair.rs - Add import for validation
  - ADD import at top of file (after line 11)
  - IMPLEMENT: use crate::staging::validate_workspace_attached;
  - FOLLOW pattern: Existing imports (lines 10-14)
  - DEPENDENCIES: None

Task 3: MODIFY src/commands/repair.rs - Add check_workspace_attachment function
  - CREATE: New check function following existing check pattern
  - IMPLEMENT:
    ```rust
    fn check_workspace_attachment(
        args: &RepairArgs,
        issues_found: &mut Vec<String>,
    ) {
        print!("Checking workspace attachment... ");

        // Load project context
        let context = match ProjectContext::load() {
            Ok(ctx) => ctx,
            Err(JinError::NotInitialized) => {
                println!("✓ (not initialized)");
                return;
            }
            Err(_) => {
                println!("✓ (no context)");
                return;
            }
        };

        // Open Jin repository
        let repo = match JinRepo::open() {
            Ok(r) => r,
            Err(_) => {
                println!("✓ (no repository)");
                return;
            }
        };

        // Validate workspace
        match validate_workspace_attached(&context, &repo) {
            Ok(()) => {
                println!("✓");
            }
            Err(JinError::DetachedWorkspace { details, recovery_hint, .. }) => {
                println!("✗");
                let issue = format!("Workspace is detached. {}. Recovery: {}", details, recovery_hint);
                issues_found.push(issue);

                if !args.dry_run {
                    println!("  Issue: {}", issue);
                }
            }
            Err(e) => {
                println!("✗");
                let issue = format!("Workspace check failed: {}", e);
                issues_found.push(issue);

                if !args.dry_run {
                    println!("  Issue: {}", issue);
                }
            }
        }
    }
    ```
  - FOLLOW pattern: check_project_context() function (lines 571-636)
  - NAMING: check_workspace_attachment
  - PLACEMENT: After check_project_context() function
  - DEPENDENCIES: Task 2 (must have import)

Task 4: MODIFY src/commands/repair.rs - Call check function in execute()
  - INSERT at line 32 (right after "println!();" at line 31)
  - IMPLEMENT:
    ```rust
    // Check workspace attachment if --check flag is set
    if args.check {
        check_workspace_attachment(&args, &mut issues_found);
        return Ok(());
    }
    ```
  - CRITICAL: This is an early return - --check runs ONLY this check
  - DEPENDENCIES: Task 3 (function must exist)

Task 5: MODIFY src/commands/repair.rs - Add check to normal repair flow
  - INSERT at line 80 (after check_project_context, before summary)
  - IMPLEMENT:
    ```rust
    // Check 8: Workspace attachment (skip if --check mode, already ran)
    if !args.check {
        check_workspace_attachment(&args, &mut issues_found);
    }
    ```
  - GOTCHA: Only run in normal repair mode, not --check mode (already ran)
  - DEPENDENCIES: Task 3 (function must exist)

Task 6: CREATE tests/repair_check.rs
  - IMPLEMENT: Integration tests for --check flag behavior
  - FOLLOW pattern: tests/workspace_validation.rs structure
  - USE: TestFixture from tests/common/fixtures.rs
  - NAMING: test_repair_check_*, test_repair_normal_without_check
  - COVERAGE: All three detachment conditions, normal mode unchanged
  - PLACEMENT: New test file in tests/ directory
  - DEPENDENCIES: Tasks 1-5 (implementation must be complete)
```

### Implementation Patterns & Key Details

```rust
// PATTERN 1: Check function structure (matches existing repair checks)
// File: src/commands/repair.rs
// Location: After check_project_context(), before execute()

fn check_workspace_attachment(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
) {
    print!("Checking workspace attachment... ");

    // Load project context (handle not initialized)
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            println!("✓ (not initialized)");
            return;
        }
        Err(_) => {
            println!("✓ (no context)");
            return;
        }
    };

    // Open Jin repository (handle missing repo)
    let repo = match JinRepo::open() {
        Ok(r) => r,
        Err(_) => {
            println!("✓ (no repository)");
            return;
        }
    };

    // Validate workspace
    match validate_workspace_attached(&context, &repo) {
        Ok(()) => {
            println!("✓");
        }
        Err(JinError::DetachedWorkspace { details, recovery_hint, .. }) => {
            println!("✗");
            let issue = format!(
                "Workspace is detached. {}. Recovery: {}",
                details, recovery_hint
            );
            issues_found.push(issue);

            if !args.dry_run {
                println!("  Issue: {}", issue);
            }
        }
        Err(e) => {
            println!("✗");
            let issue = format!("Workspace check failed: {}", e);
            issues_found.push(issue);

            if !args.dry_run {
                println!("  Issue: {}", issue);
            }
        }
    }
}

// PATTERN 2: Early check mode in execute()
// File: src/commands/repair.rs
// Location: After initial println!(), before any other checks

pub fn execute(args: RepairArgs) -> Result<()> {
    println!("Checking Jin repository integrity...");
    println!();

    let mut issues_found = Vec::new();
    let mut issues_fixed = Vec::new();

    // NEW: Check workspace attachment if --check flag is set
    if args.check {
        check_workspace_attachment(&args, &mut issues_found);

        // Display summary and return early
        println!();
        if issues_found.is_empty() {
            println!("Workspace is properly attached.");
        } else {
            println!("Workspace state:");
            for issue in &issues_found {
                println!("  - {}", issue);
            }
        }
        return Ok(());
    }

    // ... rest of existing checks continue unchanged
}

// PATTERN 3: Integration test structure
// File: tests/repair_check.rs

#[test]
fn test_repair_check_success_when_attached() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create Jin repository
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Run repair --check (should pass)
    let result = jin_cmd()
        .args(["repair", "--check"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert();

    result
        .success()
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("attached"));
}

// GOTCHA: --check runs ONLY workspace check, not other repair checks
// Use early return after check, don't fall through to other checks

// GOTCHA: --check should still work with --dry-run
// Both flags can be used together (though --check is already read-only)

// CRITICAL: Handle not-initialized case gracefully
// If Jin is not initialized, workspace can't be "detached"
// Return success with "(not initialized)" message
```

### Integration Points

```yaml
ARGS_STRUCT:
  - file: src/cli/args.rs
  - location: RepairArgs struct, after dry_run field (line ~200)
  - add: |
      /// Check workspace state without making repairs
      #[arg(long)]
      pub check: bool,

REPAIR_COMMAND:
  - file: src/commands/repair.rs
  - import: Add use crate::staging::validate_workspace_attached; at top
  - function: Add check_workspace_attachment() function
  - execute: Add early check block at start of execute()
  - normal_flow: Add check to normal repair flow (after other checks)

IMPORTS:
  - file: src/commands/repair.rs
  - add: use crate::staging::validate_workspace_attached;
  - placement: After line 11 (after other staging imports)

TESTS:
  - file: tests/repair_check.rs (NEW)
  - use: TestFixture from tests/common/fixtures.rs
  - use: assert_cmd for CLI testing
  - use: predicates for output verification
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
# Test unit tests in repair command
cargo test --lib repair::tests              # Unit tests for repair command

# Full library unit test suite
cargo test --lib

# Expected: All tests pass. If failing, debug root cause and fix.
```

### Level 3: Integration Tests (System Validation)

```bash
# Run integration tests for repair --check
cargo test --test repair_check

# Run existing repair tests to ensure no regression
cargo test --lib repair::tests

# Full integration test suite
cargo test --test

# Expected: All tests pass. Check specifically for:
# - repair --check reports attached when workspace is clean
# - repair --check reports detached when files modified
# - repair --check reports detached when refs missing
# - repair --check reports detached when context invalid
# - repair (without --check) still runs all 7 checks
```

### Level 4: Manual CLI Testing (End-to-End Validation)

```bash
# Setup test environment
TEST_DIR=$(mktemp -d)
cd $TEST_DIR
jin init
export JIN_DIR="$TEST_DIR/.jin"

# Scenario 1: Check clean workspace
jin repair --check
# Expected: "Checking workspace attachment... ✓"
# Expected: "Workspace is properly attached."

# Scenario 2: Create detached state (modify file externally)
echo "test" > config.txt
jin add config.txt
jin commit -m "Add config"
echo "modified externally" > config.txt  # Modify outside Jin

jin repair --check
# Expected: "Checking workspace attachment... ✗"
# Expected: "Workspace is detached: [details]"
# Expected: "Recovery: [recovery hint]"

# Scenario 3: Verify normal repair still works
jin repair
# Expected: All 7 checks run (repository, refs, staging, etc.)
# Expected: Workspace attachment check also runs

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

- [ ] `jin repair --check` runs workspace validation and reports result
- [ ] `--check` flag is recognized by CLI
- [ ] Output shows "✓" when workspace is attached
- [ ] Output shows "✗" with details when workspace is detached
- [ ] `--check` mode exits early (doesn't run other repair checks)
- [ ] Normal `jin repair` (without `--check`) continues to run all 7 checks
- [ ] Not-initialized case handled gracefully
- [ ] All three detachment conditions are detected

### Code Quality Validation

- [ ] Follows existing repair command structure and patterns
- [ ] Uses existing validate_workspace_attached() function
- [ ] Output format matches other repair checks (✓/✗ pattern)
- [ ] Error handling is comprehensive (NotInitialized, missing repo, etc.)
- [ ] Test code follows existing patterns (TestFixture, isolation)
- [ ] No redundant code or duplicated checks

### Documentation & Deployment

- [ ] CLI flag has documentation comment
- [ ] Function has doc comment explaining purpose
- [ ] Output messages are clear and user-friendly
- [ ] Recovery hints from DetachedWorkspace error are displayed
- [ ] No new environment variables or configuration required

---

## Anti-Patterns to Avoid

- **Don't modify other repair checks**: The `--check` flag only adds workspace attachment check
- **Don't run all checks in --check mode**: `--check` runs ONLY workspace attachment check, then exits
- **Don't fail on not-initialized**: Return success with "(not initialized)" message instead
- **Don't catch all errors**: Handle specific error types (NotInitialized, DetachedWorkspace, others)
- **Don't modify validate_workspace_attached()**: Use it as-is, don't change its behavior
- **Don't duplicate validation logic**: Call the existing function, don't reimplement
- **Don't skip tests for not-initialized case**: Test that `--check` handles uninitialized Jin gracefully
- **Don't forget dry_run compatibility**: `--check` should work with or without `--dry-run`

---

## Confidence Score

**One-Pass Implementation Success Likelihood**: **9/10**

**Rationale**:
- **Complete context**: All file paths, line numbers, function signatures, and patterns provided
- **Simple scope**: Only 2 files to modify, no new types or complex logic
- **Clear patterns**: Existing repair check structure to follow, validation function already implemented
- **Comprehensive validation**: 4-level validation loop with specific commands
- **Risk mitigation**: Clear anti-patterns and gotchas identified

**Remaining risk (1 point deduction)**:
- Minor uncertainty: Integration with existing issues_found tracking needs careful testing

---

## Appendix: Critical Code Snippets

### RepairArgs Struct (Current)

```rust
// File: src/cli/args.rs, lines 194-200
#[derive(Args, Debug)]
pub struct RepairArgs {
    /// Show what would be repaired
    #[arg(long)]
    pub dry_run: bool,
}
```

### Repair Command execute() Function (Beginning)

```rust
// File: src/commands/repair.rs, lines 29-35
pub fn execute(args: RepairArgs) -> Result<()> {
    println!("Checking Jin repository integrity...");
    println!();

    let mut issues_found = Vec::new();
    let mut issues_fixed = Vec::new();

    // *** INSERT --check HANDLER HERE ***
```

### Existing Check Function Pattern

```rust
// File: src/commands/repair.rs, lines 571-636 (check_project_context)
fn check_project_context(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking project context... ");

    let context_path = ProjectContext::default_path();

    // Only check if .jin directory exists
    if !context_path.parent().map(|p| p.exists()).unwrap_or(false) {
        println!("✓ (not initialized)");
        return;
    }

    // ... rest of check logic
}
```

### validate_workspace_attached Function Signature

```rust
// File: src/staging/workspace.rs, lines 328-331
pub fn validate_workspace_attached(
    context: &ProjectContext,
    repo: &JinRepo,
) -> Result<()>
```
