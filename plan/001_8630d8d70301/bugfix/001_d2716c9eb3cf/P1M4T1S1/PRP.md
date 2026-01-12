# PRP: Create Test Mode Creation Helper Function

---

## Goal

**Feature Goal**: Create a reusable test helper function `create_test_mode_in_context()` that creates a Git mode reference in an isolated test environment using absolute paths from `UnitTestContext` instead of environment variables.

**Deliverable**: A new test helper function in `src/commands/scope.rs` test module that:
1. Creates a mode ref at `refs/jin/modes/{name}/_mode`
2. Uses absolute paths from `UnitTestContext` instead of `JIN_DIR` environment variable
3. Follows established test helper patterns in the codebase
4. Can be used by multiple tests for consistent mode creation

**Success Definition**:
- The helper function exists in the test module of `src/commands/scope.rs`
- The function creates a valid mode ref using the correct ref path pattern
- Tests can use the function to create modes in isolation
- The function uses `UnitTestContext` for proper test isolation
- All existing tests that create modes can be refactored to use this helper

## User Persona

**Target User**: Developer working on Jin test suite

**Use Case**: Tests need to create mode refs in isolation to test mode-bound scope creation and other mode-related functionality.

**User Journey**:
1. Developer writes a test that needs a mode to exist
2. Developer calls `create_test_mode_in_context("testmode", &ctx)` instead of manually creating mode refs
3. Mode ref is created in isolated test directory without affecting other tests
4. Test proceeds with mode ref available

**Pain Points Addressed**:
- **Inconsistent mode creation**: Tests currently manually create mode refs using JinRepo calls
- **Test isolation issues**: Current `create_test_mode()` uses `JinRepo::open_or_create()` which relies on environment variables
- **Code duplication**: Multiple tests need mode refs but duplicate the creation logic

## Why

- **Test infrastructure consistency**: Establishes a pattern for test helpers that use `UnitTestContext` for isolation
- **Enables parallel test execution**: Using absolute paths instead of environment variables allows tests to run in parallel
- **Supports future work**: Subtask P1.M4.T1.S3 (refactor test_create_mode_bound_scope) depends on this helper
- **Reduces test maintenance**: Centralizes mode creation logic in one place

## What

Create a new test helper function `create_test_mode_in_context(name: &str, ctx: &UnitTestContext)` in the test module of `src/commands/scope.rs` that:

1. Opens a `JinRepo` at the `ctx.jin_dir` path
2. Creates an empty Git tree
3. Creates a commit with a descriptive message
4. Sets the mode ref at `refs/jin/modes/{name}/_mode`
5. Uses absolute paths from the context instead of environment variables

### Success Criteria

- [ ] Helper function exists with correct signature: `create_test_mode_in_context(name: &str, ctx: &UnitTestContext)`
- [ ] Function creates mode ref at correct path: `refs/jin/modes/{name}/_mode`
- [ ] Function uses `JinRepo::open_or_create_at(&ctx.jin_dir)` for isolation
- [ ] Existing `create_test_mode()` function is kept for backward compatibility
- [ ] Function is callable from any test in the module

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes - this PRP provides exact method signatures, file paths, code patterns, and validation commands.

### Documentation & References

```yaml
# MUST READ - Architecture context
- file: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/test_infrastructure_analysis.md
  why: Explains the test infrastructure patterns and the exact helper function needed
  section: Helper Functions Needed (lines 200-232)
  critical: The document specifies the exact function signature and implementation pattern needed

# MUST READ - JinRepo API
- file: src/git/repo.rs
  why: Core API for creating Git trees, commits, and refs
  pattern: Look for ObjectOps and RefOps trait implementations
  gotcha: Use open_or_create_at() with explicit path for test isolation

# MUST READ - UnitTestContext definition
- file: src/test_utils.rs
  why: Defines the test context structure with jin_dir path
  pattern: UnitTestContext has pub jin_dir: PathBuf field
  gotcha: Keep UnitTestContext in scope throughout test to prevent cleanup

# MUST READ - Production mode creation pattern
- file: src/commands/mode.rs
  why: Shows the exact ref path pattern for modes: refs/jin/modes/{name}/_mode
  section: create() function (lines 53-84)
  pattern: Uses _mode suffix to make mode name a directory for nested scopes

# MUST READ - Current test module
- file: src/commands/scope.rs
  why: Location where new helper will be added, has existing create_test_mode()
  section: tests module (lines 431-698)
  pattern: Existing create_test_mode() at lines 462-475 shows current approach

# MUST READ - Rust testing best practices
- file: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/P1M4T1S1/research/rust_testing_best_practices.md
  why: Comprehensive guide on Rust testing patterns, test helpers, and isolation
  critical: Naming conventions, module structure, and tempfile usage patterns
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── scope.rs           # TARGET FILE - Add helper function here
│   └── mode.rs            # Production mode creation reference
├── git/
│   └── repo.rs            # JinRepo API for Git operations
├── test_utils.rs          # UnitTestContext definition
└── core/
    └── layer.rs           # Ref path patterns
```

### Desired Codebase Tree with Files to be Added

```bash
# No new files - helper function added to existing test module
src/commands/scope.rs (modified)
  └── tests module gets new function: create_test_mode_in_context()
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Mode ref path must use _mode suffix (not just mode name)
// This allows the mode to have child refs (scopes, projects)
// Example: refs/jin/modes/dev_mode/_mode (NOT refs/jin/modes/dev_mode)

// CRITICAL: Use JinRepo::open_or_create_at() with explicit path for test isolation
// Using open_or_create() relies on JIN_DIR env var which causes test interference
let repo = JinRepo::open_or_create_at(&ctx.jin_dir)?;

// CRITICAL: UnitTestContext must be kept in scope (use let _ctx = ...)
// If dropped prematurely, the temporary directory and all refs are deleted
let ctx = setup_unit_test();
// Keep ctx in scope throughout test

// CRITICAL: The underscore prefix on _mode is for Git ref validity
// Directories in Git refs must have underscore prefix
// See: https://git-scm.com/docs/git-check-ref-format

// PATTERN: Create empty tree with empty array
let empty_tree = repo.create_tree(&[])?;

// PATTERN: Initial commit has no parents (empty array)
let commit_oid = repo.create_commit(None, "message", tree_oid, &[])?;

// GOTCHA: Test helpers in #[cfg(test)] modules are private by default
// Use pub fn if helper needs to be visible to other test modules
```

## Implementation Blueprint

### Data models and structure

No new data models needed. This implementation uses existing types:

- `JinRepo` from `src/git/repo.rs` - Git repository operations
- `UnitTestContext` from `src/test_utils.rs` - Test isolation context
- `Oid` from `git2` crate - Git object identifier
- `Result<T, JinError>` - Error handling

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ANALYZE existing create_test_mode() function
  - LOCATION: src/commands/scope.rs tests module, lines 462-475
  - UNDERSTAND: Current implementation uses JinRepo::open_or_create()
  - IDENTIFY: Difference needed - use ctx.jin_dir instead of environment variable
  - KEEP: Existing function for backward compatibility with existing tests

Task 2: CREATE new helper function create_test_mode_in_context()
  - LOCATION: src/commands/scope.rs tests module (after existing create_test_mode)
  - SIGNATURE: fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext)
  - IMPLEMENTATION:
    1) Open repo at ctx.jin_dir: JinRepo::open_or_create_at(&ctx.jin_dir)?
    2) Create empty tree: repo.create_tree(&[])?
    3) Create commit: repo.create_commit(None, &format!("Initialize mode: {}", name), tree_oid, &[])?
    4) Set mode ref: repo.set_ref(&format!("refs/jin/modes/{}/_mode", name), commit_oid, &format!("create mode {}", name))?
  - PATTERN: Follow production mode creation pattern from src/commands/mode.rs:62
  - REF_PATH: Use format!("refs/jin/modes/{}/_mode", name) - underscore prefix is critical

Task 3: ADD documentation comment to helper function
  - DOCUMENT: Purpose - creates a test mode ref in isolated context
  - DOCUMENT: Parameters - name and UnitTestContext
  - DOCUMENT: Critical behavior - uses absolute paths, not environment variables
  - DOCUMENT: Ref path pattern - explains _mode suffix

Task 4: VERIFY function compiles
  - IMPORTS: Ensure use crate::git::{JinRepo, ObjectOps, RefOps} is present
  - IMPORTS: Ensure use crate::test_utils::setup_unit_test is present
  - BUILD: Run cargo build to verify compilation

Task 5: UPDATE existing test to use new helper (optional validation)
  - TARGET: test_create_mode_bound_scope (lines 516-528)
  - CHANGE: Replace create_test_mode("testmode") with create_test_mode_in_context("testmode", &ctx)
  - REQUIRES: First refactor test to use setup_unit_test() instead of setup_test_env()
  - NOTE: This is validation only - full refactor is P1.M4.T1.S3

Task 6: RUN test suite validation
  - COMMAND: cargo test scope::tests
  - EXPECTED: All existing tests still pass
  - VERIFY: New helper function is callable
```

### Implementation Patterns & Key Details

```rust
// Current implementation (uses environment variable - problematic for parallel tests)
fn create_test_mode(name: &str) {
    let repo = JinRepo::open_or_create().unwrap();  // Uses JIN_DIR env var
    let empty_tree = repo.create_tree(&[]).unwrap();
    let commit_oid = repo
        .create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])
        .unwrap();
    // Use _mode suffix to make the mode name a directory (allows nested scopes)
    repo.set_ref(
        &format!("refs/jin/modes/{}/_mode", name),  // _mode suffix is critical
        commit_oid,
        &format!("create mode {}", name),
    )
    .unwrap();
}

// NEW implementation (uses absolute paths - proper isolation)
/// Create a test mode in the given UnitTestContext
///
/// Creates a mode ref at `refs/jin/modes/{name}/_mode` using absolute paths
/// from the context, ensuring proper test isolation without relying on
/// environment variables.
///
/// # Arguments
/// * `name` - Mode name (e.g., "testmode")
/// * `ctx` - UnitTestContext containing the isolated jin_dir path
///
/// # Critical Behavior
/// - Uses `JinRepo::open_or_create_at(&ctx.jin_dir)` for isolation
/// - Creates ref at `refs/jin/modes/{name}/_mode` (underscore prefix required)
/// - Does NOT rely on JIN_DIR environment variable
fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext) {
    // CRITICAL: Use open_or_create_at with explicit path for isolation
    let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();

    // Create empty tree for initial commit
    let empty_tree = repo.create_tree(&[]).unwrap();

    // Create initial commit with no parents
    let commit_oid = repo
        .create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])
        .unwrap();

    // Set mode ref with _mode suffix (makes mode name a directory for nested scopes)
    repo.set_ref(
        &format!("refs/jin/modes/{}/_mode", name),
        commit_oid,
        &format!("create mode {}", name),
    )
    .unwrap();
}

// PATTERN: How the new helper will be used in tests
#[test]
fn test_create_mode_bound_scope_with_new_helper() {
    let ctx = setup_unit_test();

    // Create mode using isolated context
    create_test_mode_in_context("testmode", &ctx);

    let result = create("testscope", Some("testmode"));
    assert!(result.is_ok());

    // Verify ref was created
    let repo = JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
    assert!(repo.ref_exists("refs/jin/modes/testmode/scopes/testscope"));
}
```

### Integration Points

```yaml
NO EXTERNAL INTEGRATIONS - This is a test-only helper function

INTERNAL INTEGRATIONS:
  - uses: src/git/repo.rs (JinRepo, ObjectOps, RefOps traits)
  - uses: src/test_utils.rs (UnitTestContext struct)
  - used by: src/commands/scope.rs tests module
  - pattern: matches production mode creation in src/commands/mode.rs

TEST INFRASTRUCTURE:
  - works_with: setup_unit_test() from src/test_utils.rs
  - enables: test isolation without #[serial] attribute
  - supports: parallel test execution
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding the helper function - fix before proceeding
cargo check --lib                    # Check for compilation errors
cargo clippy --lib                   # Linting checks
cargo fmt --                         # Ensure consistent formatting

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the specific scope module with new helper
cargo test scope::tests --lib

# Test all scope-related functionality
cargo test scope --lib

# Run tests with output for debugging
cargo test scope::tests --lib -- --nocapture

# Expected: All existing tests pass. New helper function is available for use.
```

### Level 3: Integration Testing (System Validation)

```bash
# Run full test suite to ensure no regression
cargo test --lib

# Run tests in parallel to verify isolation
cargo test --lib --test-threads=4

# Verify the helper can be called from tests
cargo test create_mode_bound_scope --lib

# Expected: All tests pass, including those that use the new helper.
```

### Level 4: Manual Verification

```bash
# Create a simple test to manually verify the helper works
cat > tests/test_new_helper.rs << 'EOF'
#[cfg(test)]
mod tests {
    use jin::commands::scope::tests::create_test_mode_in_context;
    use jin::test_utils::setup_unit_test;

    #[test]
    fn verify_new_helper_works() {
        let ctx = setup_unit_test();
        create_test_mode_in_context("manual_test_mode", &ctx);

        // Verify the mode ref was created
        let repo = jin::git::JinRepo::open_or_create_at(&ctx.jin_dir).unwrap();
        assert!(repo.ref_exists("refs/jin/modes/manual_test_mode/_mode"));
    }
}
EOF

cargo test verify_new_helper_works

# Expected: Test passes, confirming helper creates valid mode refs.
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test scope --lib`
- [ ] No linting errors: `cargo clippy --lib`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Helper function compiles without errors
- [ ] Helper function is callable from test module

### Feature Validation

- [ ] Helper function signature matches: `fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext)`
- [ ] Function creates mode ref at correct path: `refs/jin/modes/{name}/_mode`
- [ ] Function uses `JinRepo::open_or_create_at(&ctx.jin_dir)` for isolation
- [ ] Existing `create_test_mode()` function preserved for backward compatibility
- [ ] Documentation comment explains purpose and critical behavior

### Code Quality Validation

- [ ] Follows existing test helper patterns in the codebase
- [ ] Matches production mode creation pattern from src/commands/mode.rs
- [ ] Uses absolute paths instead of environment variables
- [ ] Error handling uses unwrap() consistent with other test helpers
- [ ] Function placement is after existing create_test_mode() function

### Documentation & Deployment

- [ ] Function has documentation comment explaining purpose
- [ ] Comment documents the _mode suffix requirement
- [ ] Comment notes critical isolation behavior (absolute paths vs env vars)
- [ ] No changes to production code (test-only changes)

---

## Anti-Patterns to Avoid

- **Don't remove existing `create_test_mode()` function** - Keep it for backward compatibility with existing tests
- **Don't use `JinRepo::open_or_create()`** - This relies on JIN_DIR environment variable which breaks test isolation
- **Don't omit the `_mode` suffix** - Mode refs must use `refs/jin/modes/{name}/_mode` format (underscore prefix is critical)
- **Don't add `pub` to the function** - Test module functions are private by default, only make pub if needed by other test modules
- **Don't use environment variables** - The whole point is to use absolute paths from UnitTestContext
- **Don't forget to handle the `ctx` lifetime** - Keep UnitTestContext in scope throughout the test

---

## Related Work Items

- **P1.M4.T1.S2**: Create test mode cleanup helper function (complementary to this work)
- **P1.M4.T1.S3**: Refactor test_create_mode_bound_scope to use isolation helpers (depends on this work)
- **P1.M4.T1.S4**: Run test in parallel to verify isolation (depends on S1, S2, S3)

---

## Context Summary for Implementation

This PRP provides everything needed to implement the `create_test_mode_in_context()` helper function:

1. **Exact location**: Add to `src/commands/scope.rs` test module after existing `create_test_mode()` function
2. **Exact signature**: `fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext)`
3. **Exact ref path**: `refs/jin/modes/{name}/_mode` (underscore prefix is critical)
4. **Exact implementation**: Use `JinRepo::open_or_create_at(&ctx.jin_dir)` for isolation
5. **Validation commands**: All cargo test commands to verify the implementation

The helper function enables proper test isolation by using absolute paths from `UnitTestContext` instead of relying on the global `JIN_DIR` environment variable, allowing tests to run in parallel without interference.

---

**Confidence Score**: 10/10

This PRP provides the exact function signature, implementation pattern, file location, ref path format, and validation commands. The implementation is straightforward (add one helper function) with all dependencies clearly identified.
