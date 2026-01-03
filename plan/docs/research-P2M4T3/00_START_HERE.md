# Git Lock Contention Research Summary

**Task**: P2.M4.T3 - Fix Git Lock Contention Issues in Mode Command Tests
**Date**: 2026-01-03
**Status**: Complete

## Executive Summary

This research identified the root cause of 7 failing integration tests in `tests/mode_scope_workflow.rs`. The issue is caused by parallel test execution combined with global environment variable modification (`JIN_DIR`), causing tests to share Git repositories and experience lock contention.

**Solution**: Add `#[serial]` attribute to tests that call `fixture.set_jin_dir()`.

**Confidence**: 10/10 - Root cause is clear, solution is established pattern.

---

## Problem Analysis

### Failing Tests (7 total)

| Test Name | Error Type | Error Message |
|-----------|------------|---------------|
| `test_layer_routing_project_base` | Git Lock | `failed to lock file '/tmp/.tmpnvMd37/.jin_global/config.lock'` |
| `test_scope_requires_mode_error` | Git Lock | `failed to lock file '/tmp/.tmpnvMd37/.jin_global/HEAD.lock'` |
| `test_layer_routing_mode_project` | Invalid Ref | `refs/jin/layers/mode/.../project/.tmpiNgScl is not valid` |
| `test_layer_routing_mode_scope` | Invalid Ref | Invalid reference name (temp dir name in path) |
| `test_layer_routing_mode_scope_project` | Invalid Ref | Invalid reference name (temp dir name in path) |
| `test_layer_precedence_higher_wins` | Parent Directory | `could not remove directory... parent is not directory` |
| `test_mode_scope_deep_merge` | Parent Directory | `parent is not directory` |

### Root Cause

**Evidence**: Both Git lock failures reference the SAME JIN_DIR path:
```
/tmp/.tmpnvMd37/.jin_global/config.lock
/tmp/.tmpnvMd37/.jin_global/HEAD.lock
```

**Why this happens**:
1. Rust runs tests in parallel by default
2. `fixture.set_jin_dir()` calls `std::env::set_var("JIN_DIR", jin_dir)`
3. `std::env::set_var()` sets **process-global** environment variables
4. When multiple tests run in parallel, they overwrite each other's `JIN_DIR`
5. Tests end up accessing the same Git repository (the last test's JIN_DIR)
6. Multiple tests accessing the same repo causes Git lock contention

---

## Solution

### Established Pattern

The codebase already has this solved in two files:

1. **`tests/destructive_validation.rs`** - All tests use `#[serial]`
2. **`tests/workspace_validation.rs`** - All tests use `#[serial]`

Example from line 80 of `destructive_validation.rs`:
```rust
#[test]
#[serial]
fn test_rejects_reset_hard_when_detached() {
    // ...
}
```

### Implementation

Add `#[serial]` attribute to the 7 failing tests in `mode_scope_workflow.rs`:

```rust
// BEFORE:
#[test]
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // Sets GLOBAL JIN_DIR
    // ...
}

// AFTER:
#[test]
#[serial]  // ADD THIS
fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // Now safe: runs sequentially
    // ...
}
```

### Tests to Modify

| Line | Test Function |
|------|---------------|
| 22 | `test_layer_routing_mode_base` |
| 72 | `test_layer_routing_mode_project` |
| 127 | `test_layer_routing_mode_scope` |
| 189 | `test_layer_routing_mode_scope_project` |
| 258 | `test_layer_precedence_higher_wins` |
| 342 | `test_mode_scope_deep_merge` |
| 474 | `test_layer_routing_project_base` |
| 512 | `test_scope_requires_mode_error` |

---

## Validation

### Commands to Run

```bash
# Run the specific test file
cargo test --test mode_scope_workflow

# Expected: test result: ok. 11 passed; 0 failed

# Run with serial execution (should also pass)
cargo test --test mode_scope_workflow -- --test-threads=1

# Run full test suite
cargo test

# Run multiple times to check for flakiness
for i in {1..5}; do cargo test --test mode_scope_workflow 2>&1 | grep "test result:"; done
```

### Success Criteria

- All 11 tests in `mode_scope_workflow.rs` pass
- Tests pass consistently across multiple runs
- No regressions in other test files

---

## Key References

### Code Files

- `tests/mode_scope_workflow.rs` - Target file with failing tests
- `tests/destructive_validation.rs` - Reference pattern for `#[serial]`
- `tests/workspace_validation.rs` - Reference pattern for `#[serial]`
- `tests/common/fixtures.rs` - `TestFixture::set_jin_dir()` implementation
- `Cargo.toml` - `serial_test = "3.0"` dependency (line 56)

### Documentation

- [serial_test crate](https://docs.rs/serial_test/latest/serial_test/)
- [std::env::set_var documentation](https://doc.rust-lang.org/std/env/fn.set_var.html)

### Related PRPs

- `plan/P2M4T1/PRP.md` - Test infrastructure improvements
- `plan/docs/P2M4T2_PRP_Fix_File_System_Path_Issues.md` - File system path fixes

---

## Why This Fix Works

### The `#[serial]` Attribute

The `serial_test` crate provides the `#[serial]` attribute which:

1. **Collects all tests marked with `#[serial]`**
2. **Runs them sequentially** (one at a time)
3. **Runs other tests in parallel** (without `#[serial]`)

This prevents:
- Multiple tests modifying `JIN_DIR` simultaneously
- Tests sharing the same Git repository
- Git lock contention

### Why Other Tests Don't Need `#[serial]`

Tests that DON'T call `fixture.set_jin_dir()` are safe because:
- They use the default `JIN_DIR` (or don't care about it)
- Each has its own `TestFixture` with isolated temp directories
- No global state is modified

### Why `test_multiple_modes_isolated` Passes

This test (line 558) already passes without `#[serial]`. Investigation needed:
- It may not actually call `set_jin_dir()` in the failing path
- Or its test isolation happens to work

**Recommendation**: Add `#[serial]` for consistency if it uses `set_jin_dir()`.

---

## Implementation Notes

### No Changes To

- Test logic or assertions
- `TestFixture` implementation
- `set_jin_dir()` function
- Cargo.toml (dependency already present)

### Only Changes

- Add `#[serial]` attribute before `#[test]` on 7 functions

### Placement

```rust
// CORRECT placement:
#[test]
#[serial]  // Goes AFTER #[test], BEFORE function name
fn test_something() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}
```

---

## Risk Assessment

**Risk Level**: ZERO

**Reasons**:
- Minimal code change (adding attributes only)
- Established pattern in codebase
- No logic modifications
- Easily reversible if issues arise
- Validation is straightforward

---

## Next Steps After Implementation

1. **P2.M4.T4** - Fix Test Expectation Mismatches
   - Depends on completion of P2.M4.T3
   - Will build on stable test foundation

2. **Full Test Suite Validation**
   - Run `cargo test` to verify no regressions
   - Check CI/CD pipeline results

3. **Consider Adding `#[serial]` to More Tests**
   - Review other integration tests for `set_jin_dir()` usage
   - Ensure consistency across test suite
