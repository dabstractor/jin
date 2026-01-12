# Test Isolation Issues Report

## 1. Global State Pollution in Mode/Scope Creation

### Issue Summary
Tests are creating modes and scopes in the global Jin repository (`~/.jin/`) without proper cleanup or unique naming strategies.

### Affected Files
- `/home/dustin/projects/jin/tests/common/fixtures.rs` (lines 174-213)
- `/home/dustin/projects/jin/tests/core_workflow.rs` (lines 76, 109, 151)
- `/home/dustin/projects/jin/tests/mode_scope_workflow.rs` (lines 27, 67)
- `/home/dustin/projects/jin/tests/cli_mv.rs` (line 105)
- `/home/dustin/projects/jin/tests/sync_workflow.rs`
- `/home/dustin/projects/jin/tests/atomic_operations.rs`
- `/home/dustin/projects/jin/tests/error_scenarios.rs`

### Specific Problems

**1. Global Repository Pollution:**
The `create_mode()` and `create_scope()` functions in `fixtures.rs` (lines 174-213) modify the global Jin repository at `~/.jin/` without ensuring unique mode names or cleanup.

**2. Non-Unique Mode Names:**
While some tests use `format!("test_mode_{}", std::process::id())` to create unique names, many tests still use hardcoded names like "testmode", "language:javascript", etc.

**3. Cross-Test Contamination:**
Tests in different files can create the same modes/scopes, causing failures when a test expects a mode/scope to not exist but another test already created it.

## 2. Hardcoded Global Paths in Assertions

### Issue Summary
The `assert_layer_ref_exists()` function references the global Jin repository path directly, causing tests to depend on the user's actual Jin configuration.

### Affected File
- `/home/dustin/projects/jin/tests/common/assertions.rs` (lines 134-158)

### Specific Problems

```rust
// Line 135-136 in assertions.rs
let home_dir = dirs::home_dir().expect("Failed to get home directory");
let jin_repo_path = home_dir.join(".jin");
```

This function always looks for layer refs in `~/.jin/`, which means:
- Tests will fail if the user doesn't have Jin initialized at `~/.jin/`
- Tests can interfere with the user's actual Jin repository
- Tests are not truly isolated as they depend on external state

## 3. Inconsistent JIN_DIR Usage

### Issue Summary
Tests inconsistently handle the `JIN_DIR` environment variable, leading to potential conflicts with the global Jin repository.

### Affected Files
- `/home/dustin/projects/jin/tests/cli_basic.rs` (multiple locations using `.jin_global`)
- `/home/dustin/projects/jin/tests/cli_import.rs` (multiple locations)
- `/home/dustin/projects/jin/tests/cli_reset.rs` (multiple locations)
- `/home/dustin/projects/jin/tests/cli_diff.rs` (multiple locations)

### Specific Problems

1. **Inconsistent Naming**: Some tests use `.jin_global` as the suffix, others use `.jin`.
2. **Partial Isolation**: While some tests set `env("JIN_DIR", temp.path().join(".jin_global"))`, not all operations respect this environment variable.
3. **Global Repository Access**: Even when `JIN_DIR` is set, some operations (like mode creation) still access the global `~/.jin/` repository.

## 4. Git Repository Lock Issues

### Issue Summary
Tests create and manipulate Git repositories simultaneously, which can cause lock file conflicts.

### Affected Files
- `/home/dustin/projects/jin/tests/common/fixtures.rs` (lines 97 - bare repository creation)
- Multiple test files that create commits simultaneously

### Specific Problems

1. **Bare Repository Creation**: The `setup_jin_with_remote()` function creates a bare repository at a fixed path within a temp directory.
2. **Concurrent Access**: Multiple tests may try to access the same global Git repositories simultaneously.
3. **Git Lock Files**: Git operations create `.git/index.lock` files that can cause conflicts between parallel test runs.

## 5. No Test Cleanup or Teardown

### Issue Summary
The test suite lacks proper cleanup mechanisms to remove created modes, scopes, and repositories after test execution.

### Affected Files
- All test files in `/home/dustin/projects/jin/tests/`

### Specific Problems

1. **Persistent Artifacts**: Created modes and scopes remain in the global Jin repository between test runs.
2. **State Accumulation**: Multiple test runs will accumulate more and more modes/scopes, potentially slowing down tests.
3. **Interference Between Test Suites**: Different test suites (e.g., `cargo test` vs IDE test runners) can interfere with each other.

## Recommendations

### Immediate Fixes

1. **Ensure Unique Mode/Scope Names**: All tests should use unique names based on process ID and test name:
   ```rust
   let mode_name = format!("test_{}_{}", test_name::current(), std::process::id());
   ```

2. **Isolate Global Repository Access**: Create a mock or test-specific global repository for tests that need to interact with layer refs.

3. **Consistent JIN_DIR Usage**: Standardize on a single naming convention (e.g., `.jin_test`) and ensure all operations respect it.

4. **Add Test Cleanup**: Implement a test cleanup mechanism that removes test-specific modes and scopes after each test.

### Priority Order

1. **High Priority**: Fix global state pollution and hardcoded paths
2. **Medium Priority**: Standardize JIN_DIR usage and add cleanup
3. **Low Priority**: Implement advanced isolation mechanisms
