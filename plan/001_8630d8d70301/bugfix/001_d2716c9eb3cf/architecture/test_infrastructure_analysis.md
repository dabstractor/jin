# Test Infrastructure Analysis

## Overview

The Jin test suite has two categories of issues:
1. **Incorrect ref path assertions** - Tests expect ref paths without the `/_` suffix (test bugs, not implementation bugs)
2. **Flaky test isolation** - One test fails when run with others but passes individually (test isolation bug)

## Test Architecture

### 1. Test Structure

The codebase has two types of tests:

1. **Unit tests**: Embedded in source files (e.g., `src/commands/scope.rs`)
2. **Integration tests**: Separate files in `tests/` directory

### 2. Test Utilities

**File**: `src/test_utils.rs`

Provides `UnitTestContext` for proper test isolation:

```rust
pub fn setup_unit_test() -> UnitTestContext {
    // Creates isolated temporary directory
    // Sets up isolated JIN_DIR
    // Provides absolute paths for test operations
}
```

**File**: `tests/common/fixtures.rs`

Provides `TestFixture` for integration tests:

```rust
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub project_path: PathBuf,
    pub jin_dir: PathBuf,
}
```

**File**: `tests/common/git_helpers.rs`

Provides Git lock cleanup:

```rust
pub fn cleanup_git_locks(repo_path: &Path) {
    // Removes stale .git/lock files
    // Ensures clean test state
}
```

## Issue 1: Incorrect Ref Path Assertions

### Root Cause

The implementation correctly uses the `/_` suffix for layer refs that can have child refs, but some test assertions were written expecting the incorrect ref paths.

### Ref Path Rules

**File**: `src/core/layer.rs` (lines 50-56)

The `/_` suffix is required for layers that can have child refs:

- **ModeBase**: `refs/jin/layers/mode/<name>/_` (has ModeScope and ModeProject children)
- **ModeScope**: `refs/jin/layers/mode/<name>/scope/<scope>/_` (has ModeScopeProject children)
- **ModeProject**: `refs/jin/layers/mode/<name>/project/<project>` (no children, no `/_`)
- **ModeScopeProject**: `refs/jin/layers/mode/<name>/scope/<scope>/project/<project>` (no children, no `/_`)

### Tests That Need Fixing

**File**: `tests/mode_scope_workflow.rs`

| Test Function | Line | Current (Incorrect) | Correct Path |
|---------------|------|---------------------|--------------|
| `test_layer_routing_mode_base` | 68 | `refs/jin/layers/mode/{mode}` | `refs/jin/layers/mode/{mode}/_` |
| `test_layer_routing_mode_scope` | 187 | `refs/jin/layers/mode/{mode}/scope/{scope}` | `refs/jin/layers/mode/{mode}/scope/{scope}/_` |
| `test_multiple_modes_isolated` | 634 | `refs/jin/layers/mode/{mode_a}` | `refs/jin/layers/mode/{mode_a}/_` |
| `test_multiple_modes_isolated` | 635 | `refs/jin/layers/mode/{mode_b}` | `refs/jin/layers/mode/{mode_b}/_` |

### Fix Strategy

1. Update `assert_layer_ref_exists()` calls to include the `/_` suffix
2. Verify all other ref path assertions in the test suite
3. Run tests to confirm all pass

### Example Fix

```rust
// BEFORE (incorrect):
assert_layer_ref_exists(&format!("refs/jin/layers/mode/{}", mode_name), ...);

// AFTER (correct):
assert_layer_ref_exists(&format!("refs/jin/layers/{}/_", mode_name), ...);
```

## Issue 2: Flaky Test Isolation

### Root Cause

**Test**: `commands::scope::tests::test_create_mode_bound_scope` in `src/commands/scope.rs`

The test fails when run with other tests due to **shared global state**:

1. **Global `JIN_DIR` environment variable** - Multiple tests may interfere
2. **Current working directory changes** - `std::env::set_current_dir()` affects all tests
3. **Git repository state** - Refs created in one test persist to the next
4. **No proper cleanup** - Test artifacts not removed between runs

### Current Test Setup

```rust
fn setup_test_env() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("JIN_DIR", temp_dir.path().join(".jin_global"));
    std::env::set_current_dir(temp_dir.path()).unwrap();
    temp_dir
}
```

**Problems**:
- Uses environment variables (shared across tests)
- Changes current directory (affects all tests)
- No cleanup of Git refs between tests
- Relies on `#[serial]` attribute as workaround

### Established Test Isolation Pattern

The codebase already has a robust pattern in `src/test_utils.rs`:

```rust
pub fn setup_unit_test() -> UnitTestContext {
    let temp_dir = tempfile::tempdir().unwrap();
    let jin_dir = temp_dir.path().join(".jin");

    // Use absolute paths instead of environment variables
    std::fs::create_dir_all(&jin_dir).unwrap();

    UnitTestContext {
        temp_dir,
        project_path: temp_dir.path().to_path_buf(),
        jin_dir,
    }
}

pub struct UnitTestContext {
    pub temp_dir: TempDir,
    pub project_path: PathBuf,
    pub jin_dir: PathBuf,
}
```

**Advantages**:
- Uses absolute paths (no environment variable conflicts)
- Each test has its own isolated `TempDir`
- Automatic cleanup via `Drop` trait
- No reliance on current directory

### Fix Strategy

1. **Replace manual setup** with `setup_unit_test()` from `src/test_utils.rs`
2. **Use absolute paths** instead of environment variables
3. **Add cleanup functions** for test artifacts (Git refs)
4. **Remove `#[serial]` attribute** once isolation is confirmed

### Example Fix

```rust
// BEFORE (problematic):
#[test]
#[serial]
fn test_create_mode_bound_scope() {
    let _temp = setup_test_env();
    // ... test code that relies on JIN_DIR env var and current dir
}

// AFTER (isolated):
#[test]
fn test_create_mode_bound_scope() {
    let ctx = setup_unit_test();

    // Clean up any existing test mode
    cleanup_test_mode("testmode", &ctx);

    // Create test mode using isolated context
    create_test_mode_in_context("testmode", &ctx);

    let result = create("testscope", Some("testmode"));
    assert!(result.is_ok());

    // Verify ref using paths from context
    let repo = JinRepo::open_or_create().unwrap();
    let expected_ref = format!("refs/jin/modes/testmode/scopes/testscope");
    assert!(repo.ref_exists(&expected_ref));
}
```

### Helper Functions Needed

```rust
fn create_test_mode_in_context(name: &str, ctx: &UnitTestContext) {
    let repo = JinRepo::open_or_create().unwrap();
    let empty_tree = repo.create_tree(&[]).unwrap();
    let commit_oid = repo
        .create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])
        .unwrap();
    repo.set_ref(
        &format!("refs/jin/modes/{}/_mode", name),
        commit_oid,
        &format!("create mode {}", name),
    )
    .unwrap();
}

fn cleanup_test_mode(name: &str, ctx: &UnitTestContext) {
    let repo = JinRepo::open_or_create().unwrap();
    let mode_ref = format!("refs/jin/modes/{}/_mode", name);
    let scope_ref_pattern = format!("refs/jin/modes/{}/scopes/*", name);

    if repo.ref_exists(&mode_ref) {
        let _ = repo.delete_ref(&mode_ref);
    }

    if let Ok(refs) = repo.list_refs(&scope_ref_pattern) {
        for ref_path in refs {
            let _ = repo.delete_ref(&ref_path);
        }
    }
}
```

## Test Dependencies

### External Dependencies

- **`serial_test` crate**: Provides `#[serial]` attribute for test ordering
- **`tempfile` crate**: Provides `TempDir` for temporary directories
- **`git2` crate**: Git operations for test setup

### Internal Dependencies

- **`src/test_utils.rs`**: `setup_unit_test()` function
- **`tests/common/fixtures.rs`**: `TestFixture` struct
- **`tests/common/git_helpers.rs`**: `cleanup_git_locks()` function

## Impact Assessment

### Test Ref Path Fixes

- **Risk**: Minimal - just updating string literals in assertions
- **Impact**: Test suite will pass, giving accurate feedback
- **Breaking changes**: None

### Flaky Test Fix

- **Risk**: Low - using established test patterns
- **Impact**: Tests will pass reliably in any order
- **Breaking changes**: None

## References

- **Implementation files**:
  - `src/test_utils.rs` - Unit test utilities
  - `tests/common/fixtures.rs` - Integration test fixtures
  - `tests/common/git_helpers.rs` - Git helper functions
  - `tests/mode_scope_workflow.rs` - Tests with incorrect ref paths
  - `src/commands/scope.rs` - Flaky test location
