name: "PRP: P1.M4.T1.S2 - Create Test Mode Cleanup Helper Function"
description: |

---

## Goal

**Feature Goal**: Create a `cleanup_test_mode()` helper function that removes all Git refs associated with a test mode to ensure clean test isolation.

**Deliverable**: A new test helper function `cleanup_test_mode(name: &str, ctx: &UnitTestContext)` in `src/commands/scope.rs` that deletes the mode ref and all associated scope refs.

**Success Definition**: The function successfully deletes all refs associated with a test mode (mode ref at `refs/jin/modes/{name}/_mode` and all scope refs at `refs/jin/modes/{name}/scopes/*`) without errors when refs don't exist.

## User Persona (if applicable)

**Target User**: Test authors writing unit tests for mode-bound scope functionality

**Use Case**: Ensuring tests start with a clean state by removing any artifacts from previous test runs

**User Journey**: Test calls `cleanup_test_mode("testmode", &ctx)` before `create_test_mode_in_context("testmode", &ctx)` to guarantee isolation

**Pain Points Addressed**: Flaky tests caused by leftover Git refs from previous test runs interfering with current test assertions

## Why

- **Test Reliability**: Tests currently fail when run in parallel due to shared Git ref state from previous runs
- **Isolation**: Each test should start with a clean slate, independent of execution order
- **Debuggability**: Clean test state makes it easier to identify the actual cause of test failures
- **Best Practices**: Following established patterns from `src/commands/mode.rs` delete function

## What

Create a cleanup helper function that removes all Git refs associated with a test mode, handling cases where refs don't exist gracefully.

### Function Signature

```rust
fn cleanup_test_mode(name: &str, ctx: &UnitTestContext)
```

### Ref Paths to Clean

1. **Mode ref**: `refs/jin/modes/{name}/_mode` (the main mode reference created by S1)
2. **Scope refs**: `refs/jin/modes/{name}/scopes/*` (any scope refs created during tests)

### Behavior Requirements

- Open JinRepo using `JinRepo::open_or_create_at(&ctx.jin_dir)` for isolation
- Delete mode ref if it exists (no error if it doesn't)
- List and delete all scope refs matching the pattern
- Use `let _ =` to ignore errors during cleanup (best-effort deletion)
- Return type: `()` (panics only on repo open failure, not on ref deletion)

### Success Criteria

- [ ] Function compiles without errors
- [ ] Function deletes mode ref when it exists
- [ ] Function handles non-existent mode ref without error
- [ ] Function deletes all scope refs for the mode
- [ ] Function handles empty scope list without error
- [ ] Pattern matches `create_test_mode_in_context` style from S1

## All Needed Context

### Context Completeness Check

_The implementing agent needs to understand:_
1. How `JinRepo::open_or_create_at()` works for isolated test repos
2. The pattern for ref existence checking and safe deletion
3. The glob pattern for listing scope refs
4. How `create_test_mode_in_context` creates refs (to reverse the operation)
5. The existing cleanup pattern from `mode.rs` delete function

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: src/commands/scope.rs
  why: Contains create_test_mode_in_context (S1) to understand ref creation pattern
  lines: 492-511
  critical: Uses open_or_create_at with ctx.jin_dir for isolation, creates refs at refs/jin/modes/{name}/_mode

- file: src/commands/scope.rs
  why: Target location for cleanup_test_mode function
  lines: 511-520 (insert after create_test_mode_in_context)
  gotcha: Must be within #[cfg(test)] mod tests block

- file: src/commands/mode.rs
  why: Contains production cleanup pattern to follow
  lines: 239-259
  critical: Shows pattern for deleting mode ref and iterating through scope refs with error ignoring
  pattern: Use let _ = repo.delete_ref() for safe deletion, list_refs for wildcard matching

- file: src/git/refs.rs
  why: Core ref operations API documentation
  lines: 103-125
  critical: delete_ref() fails if ref doesn't exist, ref_exists() checks first, list_refs() uses glob patterns

- file: src/test_utils.rs
  why: UnitTestContext definition and setup pattern
  lines: 48-59 (struct), 124-186 (setup_unit_test function)
  critical: ctx.jin_dir provides absolute path to isolated .jin_global directory

- docfile: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/test_infrastructure_analysis.md
  why: Architecture document explaining test isolation requirements
  section: Lines 217-232 show exact cleanup function specification
  critical: Specifies ref paths and "delete if exists" behavior requirement

# EXTERNAL REFERENCES
- url: https://doc.rust-lang.org/std/result/enum.Result.html#method.ok
  why: Converting Result to Option for "ignore error" pattern

- url: https://docs.rs/tempfile/latest/tempfile/
  why: Understanding TempDir auto-cleanup (ctx._temp_dir field)

- url: https://matklad.github.io/2021/05/31/how-to-test.html
  why: Rust testing best practices for cleanup and isolation
```

### Current Codebase tree (src/ directory)

```bash
src/
├── commands/
│   ├── scope.rs          # TARGET FILE - Add cleanup_test_mode here (line ~511)
│   └── mode.rs           # REFERENCE - Production delete pattern (lines 239-259)
├── git/
│   ├── refs.rs           # REFERENCE - RefOps trait (delete_ref, list_refs, ref_exists)
│   └── repo.rs           # REFERENCE - JinRepo::open_or_create_at method
├── test_utils.rs         # REFERENCE - UnitTestContext struct
└── core/
    └── layer.rs          # REFERENCE - Ref path patterns
```

### Desired Codebase tree with files to be added

```bash
src/commands/scope.rs     # MODIFY - Add cleanup_test_mode function
  Line ~511:              # INSERT after create_test_mode_in_context
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: JinRepo::delete_ref() FAILS if ref doesn't exist
// ALWAYS check ref_exists() first OR use let _ = to ignore errors
// Pattern from mode.rs lines 250-257:
let _ = repo.delete_ref(pattern);  // Safe: ignore if doesn't exist

// CRITICAL: Use open_or_create_at(&ctx.jin_dir) for test isolation
// Do NOT use open_or_create() which relies on JIN_DIR env var
let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();

// CRITICAL: Mode ref path uses _mode suffix (underscore makes it a directory)
// Ref path: refs/jin/modes/{name}/_mode (NOT refs/jin/modes/{name})
format!("refs/jin/modes/{}/_mode", name)

// CRITICAL: Scope refs use scopes/* pattern for glob matching
// Pattern: refs/jin/modes/{name}/scopes/*
format!("refs/jin/modes/{}/scopes/*", name)

// CRITICAL: list_refs() returns full ref paths, not patterns
// Must iterate through results and delete each ref individually
if let Ok(refs) = repo.list_refs(pattern) {
    for ref_path in refs {
        let _ = repo.delete_ref(&ref_path);
    }
}

// CRITICAL: Test helper functions are in #[cfg(test)] module
// Must be within the `mod tests` block in scope.rs
```

## Implementation Blueprint

### Data models and structure

No new data models needed. Uses existing:
- `JinRepo` from `crate::git::repo`
- `UnitTestContext` from `crate::test_utils`
- Standard `String`, `&str` types

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD cleanup_test_mode function to src/commands/scope.rs
  - INSERT: After create_test_mode_in_context function (after line 511)
  - IMPLEMENT: cleanup_test_mode(name: &str, ctx: &UnitTestContext)
  - FOLLOW pattern: src/commands/mode.rs lines 239-259 (delete function)
  - REVERSE: create_test_mode_in_context logic (delete what it creates)
  - NAMING: snake_case function name, descriptive comments
  - PLACEMENT: Within #[cfg(test)] mod tests block

Task 2: IMPLEMENT mode ref deletion
  - DELETE: refs/jin/modes/{name}/_mode ref if it exists
  - USE: let _ = repo.delete_ref(&mode_ref) to ignore errors
  - PATTERN: Safe deletion without existence check preferred
  - FORMAT: format!("refs/jin/modes/{}/_mode", name)

Task 3: IMPLEMENT scope refs deletion
  - LIST: All refs matching refs/jin/modes/{name}/scopes/* pattern
  - ITERATE: Through returned refs and delete each one
  - USE: let _ = for all delete operations (ignore individual failures)
  - PATTERN: if let Ok(refs) = repo.list_refs(&scope_pattern) { for ref in refs { let _ = repo.delete_ref(&ref); } }

Task 4: VERIFY function signature matches requirements
  - PARAMS: name: &str, ctx: &UnitTestContext
  - RETURN: () (unit type, no return value)
  - PANICS: Only on repo.open_or_create_at failure (use .unwrap())
  - NO PANICS: On ref deletion failures (use let _ =)
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Test helper function structure (from S1)
#[cfg(test)]
mod tests {
    use crate::test_utils::{setup_unit_test, UnitTestContext};
    use crate::git::repo::JinRepo;

    // EXISTING S1 FUNCTION (for reference)
    fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext) {
        // CRITICAL: Use open_or_create_at with explicit path for isolation
        let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
        // ... creates refs ...
    }

    // NEW FUNCTION TO ADD
    /// Cleanup all refs associated with a test mode.
    ///
    /// This ensures test isolation by removing artifacts from previous runs.
    /// Handles missing refs gracefully (no errors if refs don't exist).
    ///
    /// # Arguments
    ///
    /// * `name` - The mode name to clean up
    /// * `ctx` - The test context providing isolated jin_dir path
    ///
    /// # Behavior
    ///
    /// - Opens JinRepo at ctx.jin_dir for isolation
    /// - Deletes mode ref: `refs/jin/modes/{name}/_mode`
    /// - Deletes all scope refs: `refs/jin/modes/{name}/scopes/*`
    /// - Ignores errors for non-existent refs
    fn cleanup_test_mode(name: &str, ctx: &UnitTestContext) {
        // PATTERN: Open repo at isolated path (same as S1)
        let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();

        // PATTERN: Delete mode ref (safe deletion, ignore errors)
        // GOTCHA: delete_ref() fails if ref doesn't exist, use let _ =
        let mode_ref = format!("refs/jin/modes/{}/_mode", name);
        let _ = repo.delete_ref(&mode_ref);

        // PATTERN: List and delete scope refs (from mode.rs lines 253-257)
        // CRITICAL: Use glob pattern for wildcard matching
        let scope_ref_pattern = format!("refs/jin/modes/{}/scopes/*", name);

        // PATTERN: Safe iteration through refs
        // GOTCHA: list_refs may fail if no refs match, use if let Ok()
        if let Ok(refs) = repo.list_refs(&scope_ref_pattern) {
            for ref_path in refs {
                // PATTERN: Ignore individual deletion errors
                let _ = repo.delete_ref(&ref_path);
            }
        }
    }
}

// CRITICAL: Ref path construction (MUST match S1 exactly)
// Mode ref: refs/jin/modes/{name}/_mode (underscore suffix required)
// Scope refs: refs/jin/modes/{name}/scopes/* (wildcard for glob matching)

// CRITICAL: Error handling philosophy for cleanup functions
// - Repo open failures: Should panic (test setup is broken)
// - Ref deletion failures: Should be ignored (refs may not exist)
// - Pattern: let _ = operation;  // Silent ignore
```

### Integration Points

```yaml
SOURCE_FILE:
  - modify: src/commands/scope.rs
  - location: Line 511 (after create_test_mode_in_context)
  - context: Within #[cfg(test)] mod tests block

IMPORTS:
  - existing: use crate::git::repo::JinRepo; (already imported)
  - existing: use crate::test_utils::{setup_unit_test, UnitTestContext}; (already imported)
  - no new imports needed

CALLERS:
  - test_create_mode_bound_scope will use this function (P1.M4.T1.S3)
  - Future test helpers can call this for cleanup
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding the function - fix before proceeding
cargo check --bin jin 2>&1 | head -50

# Check for compilation errors specifically in scope.rs
cargo check --bin jin 2>&1 | grep -A 5 "scope.rs"

# Run clippy for lint checks
cargo clippy --bin jin 2>&1 | grep -A 5 "scope.rs"

# Format check
cargo fmt --check

# Expected: Zero errors. Function should compile cleanly.
# If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run scope.rs tests specifically
cargo test --bin jin commands::scope::tests --nocapture

# Run all tests in scope module
cargo test --bin jin scope -- --test-threads=1

# Expected: All tests pass. The new function is a helper,
# so it will be tested indirectly by tests that use it.

# Note: cleanup_test_mode itself is a test helper,
# validation comes from P1.M4.T1.S3 which refactors tests to use it.
```

### Level 3: Integration Testing (System Validation)

```bash
# Full test suite for affected module
cargo test --bin jin --test-threads=1

# Run the specific test that will use this helper (after S3 is done)
cargo test --bin jin test_create_mode_bound_scope -- --exact

# Verify no global state pollution by running tests multiple times
for i in {1..5}; do
  cargo test --bin jin commands::scope::tests --test-threads=1 || exit 1
done

# Expected: All tests pass consistently across multiple runs.
# The cleanup helper ensures tests don't interfere with each other.
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test cleanup function manually with a custom test
cat > src/commands/scope_test_cleanup.rs << 'EOF'
#[cfg(test)]
mod test_cleanup_validation {
    use super::tests::{cleanup_test_mode, create_test_mode_in_context, setup_unit_test};
    use crate::git::repo::JinRepo;

    #[test]
    fn test_cleanup_removes_all_refs() {
        let ctx = setup_unit_test();

        // Create a mode
        create_test_mode_in_context("cleanup_test", &ctx);

        // Verify refs exist
        let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
        assert!(repo.ref_exists("refs/jin/modes/cleanup_test/_mode"));

        // Run cleanup
        cleanup_test_mode("cleanup_test", &ctx);

        // Verify refs are gone
        assert!(!repo.ref_exists("refs/jin/modes/cleanup_test/_mode"));

        // Running cleanup again should not error
        cleanup_test_mode("cleanup_test", &ctx);
    }

    #[test]
    fn test_cleanup_with_scopes() {
        let ctx = setup_unit_test();

        // Create mode and add a scope ref manually
        create_test_mode_in_context("cleanup_with_scope", &ctx);
        let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
        let empty_tree = repo.create_tree(&[]).unwrap();
        let commit = repo.create_commit(None, "test", empty_tree, &[]).unwrap();
        repo.set_ref("refs/jin/modes/cleanup_with_scope/scopes/testscope", commit, "test").unwrap();

        // Verify both refs exist
        assert!(repo.ref_exists("refs/jin/modes/cleanup_with_scope/_mode"));
        assert!(repo.ref_exists("refs/jin/modes/cleanup_with_scope/scopes/testscope"));

        // Cleanup
        cleanup_test_mode("cleanup_with_scope", &ctx);

        // Verify both are gone
        assert!(!repo.ref_exists("refs/jin/modes/cleanup_with_scope/_mode"));
        assert!(!repo.ref_exists("refs/jin/modes/cleanup_with_scope/scopes/testscope"));
    }
}
EOF

# Run the validation tests
cargo test --bin jin test_cleanup_validation -- --exact

# Expected: All validation tests pass, proving:
# 1. Cleanup removes mode refs
# 2. Cleanup removes scope refs
# 3. Cleanup handles non-existent refs gracefully
```

## Final Validation Checklist

### Technical Validation

- [ ] Function compiles without errors: `cargo check --bin jin`
- [ ] No clippy warnings: `cargo clippy --bin jin`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Function placement is correct (after create_test_mode_in_context, within #[cfg(test)])
- [ ] Uses `JinRepo::open_or_create_at(&ctx.jin_dir)` for isolation
- [ ] Ref path patterns match S1 exactly

### Feature Validation

- [ ] Deletes mode ref at `refs/jin/modes/{name}/_mode`
- [ ] Lists and deletes scope refs at `refs/jin/modes/{name}/scopes/*`
- [ ] Handles non-existent refs without panicking
- [ ] Uses `let _ =` for safe deletion (ignores errors)
- [ ] Follows pattern from `mode.rs` delete function

### Code Quality Validation

- [ ] Function has descriptive documentation comments
- [ ] Variable names are clear (`mode_ref`, `scope_ref_pattern`, `repo`)
- [ ] No hardcoded values (uses `format!` for ref paths)
- [ ] Follows existing code style in scope.rs
- [ ] Proper placement within #[cfg(test)] mod tests block

### Integration Readiness

- [ ] Function signature matches contract: `fn cleanup_test_mode(name: &str, ctx: &UnitTestContext)`
- [ ] Ready for use by P1.M4.T1.S3 (refactor test_create_mode_bound_scope)
- [ ] No new dependencies introduced
- [ ] Follows established test helper patterns

---

## Anti-Patterns to Avoid

- ❌ Don't use `JinRepo::open_or_create()` - it relies on JIN_DIR env var (not isolated)
- ❌ Don't panic on ref deletion failures - use `let _ =` to ignore errors
- ❌ Don't check `ref_exists()` before deletion - just delete and ignore errors
- ❌ Don't use `unwrap()` on delete operations - tests should pass even if refs don't exist
- ❌ Don't forget the `_mode` suffix in ref path (it's `refs/jin/modes/{name}/_mode`, not `refs/jin/modes/{name}`)
- ❌ Don't place function outside of `#[cfg(test)]` mod tests block
- ❌ Don't return `Result` type - test helpers should panic on setup failure only
- ❌ Don't delete refs individually without listing - use glob pattern for scope refs
- ❌ Don't add new imports - all needed types are already imported

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Validation**: The completed function should enable test isolation by cleaning up all refs associated with a test mode, matching the exact inverse of what `create_test_mode_in_context` creates, following the production deletion pattern from `mode.rs`.

**Next Step**: After this PRP is implemented, proceed to P1.M4.T1.S3 to refactor `test_create_mode_bound_scope` to use both `create_test_mode_in_context` (S1) and `cleanup_test_mode` (S2).
