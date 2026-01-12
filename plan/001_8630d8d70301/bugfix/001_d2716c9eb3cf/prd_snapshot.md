# Bug Fix Requirements

## Overview

Comprehensive end-to-end testing was performed on the Jin CLI implementation against the original PRD requirements. Testing included:

- Manual testing of all major workflows (init, add, commit, mode, scope, apply)
- Execution of the full test suite (unit + integration tests)
- Layer routing verification
- Merge engine functionality testing
- Gitignore management verification
- Edge case testing

**Overall Assessment**: The implementation is **functionally complete** with correct architecture, but contains **2 Major bugs** and **multiple test failures** (mostly test bugs, not implementation bugs).

---

## Critical Issues (Must Fix)

**None found.**

The core functionality works correctly:
- Jin initializes properly
- Files can be added and committed to layers
- Mode and scope switching works
- The `.gitignore` managed block is correct
- SIGPIPE handling works (piping to `head` doesn't panic)
- Layer routing help text is comprehensive and accurate

---

## Major Issues (Should Fix)

### Issue 1: Incorrect Conflict Detection for Structured Files

**Severity**: Major
**PRD Reference**: §11.1 "Structured Merge Rules", §11.2 "Merge Priority"
**Expected Behavior**: According to the PRD, JSON/YAML/TOML files should use **deep key merge** with layer precedence determining the winner. For example:
- Layer 2 (ModeBase): `{"common": {"a": 1}, "mode": true}`
- Layer 7 (ProjectBase): `{"common": {"a": 1, "b": 2}, "project": false}`

Should merge to: `{"common": {"a": 1, "b": 2}, "mode": true, "project": false}`

Because Layer 7 has higher precedence than Layer 2, the merge should:
1. Deep-merge the `common` object (combining keys)
2. Include both `mode` and `project` keys
3. No conflict should occur

**Actual Behavior**: The implementation treats **any content difference** as a conflict, even for structured files that should be deep-merged. When two layers have different JSON content, a `.jinmerge` conflict file is created.

**Steps to Reproduce**:
```bash
cd /tmp/test && rm -rf . && jin init
jin mode create dev && jin mode use dev
echo '{"a": 1}' > config.json && jin add config.json --mode && jin commit -m "Mode"
echo '{"a": 2}' > config.json && jin add config.json && jin commit -m "Project"
jin apply
# Result: Creates .jinmerge conflict file
# Expected: Merges to {"a": 2} (ProjectBase wins) with no conflict
```

**Root Cause**: The `has_different_content_across_layers()` function in `src/merge/layer.rs` checks if content is "different" rather than checking if a merge is "possible". For structured files, differences should be resolved via deep merge, not flagged as conflicts.

**Suggested Fix**:
1. Modify `merge_layers()` in `src/merge/layer.rs` to attempt deep merge for structured files (JSON/YAML/TOML/INI) before checking for conflicts
2. Only create conflict files when:
   - Text files have unresolvable 3-way merge conflicts, OR
   - Deep merge fails for structured files
3. The deep merge implementation in `src/merge/deep.rs` is already correct - it just needs to be used instead of pre-checking for conflicts

**Impact**: Users must manually resolve conflicts that should be auto-merged. This significantly degrades the user experience and contradicts the PRD's promise of "deterministic structured merges."

---

### Issue 2: `jin log` Does Not Show All Layer Commits

**Severity**: Major
**PRD Reference**: §18.6 "`jin log [layer]` - Show commit history for layer"
**Expected Behavior**: `jin log` should show commits from all layers that have commits.

**Actual Behavior**: `jin log` only shows commits from certain layers (mode-base, project-base), but not from mode-scope, mode-project, etc.

**Steps to Reproduce**:
```bash
cd /tmp/test && rm -rf . && rm -rf /tmp/jin-test && mkdir -p /tmp/jin-test
JIN_DIR=/tmp/jin-test/.jin jin init
jin mode create testmode && jin mode use testmode
jin scope create lang:rust --mode=testmode && jin scope use lang:rust
echo '{"mode": "base"}' > mode.json && jin add mode.json --mode && jin commit -m "Mode base"
echo '{"scope": "test"}' > scope.json && jin add scope.json --mode --scope=lang:rust && jin commit -m "Mode scope"
jin log
# Result: Only shows "Mode base" commit, missing "Mode scope" commit
```

**Verification**: The commit exists in the Git repo (`cat ~/.jin/refs/jin/layers/mode/<mode>/scope/<scope>/_`) but is not displayed by `jin log`.

**Suggested Fix**: Check `src/commands/log.rs` to ensure it iterates through all layer refs, not just a hardcoded subset. The function should:
1. Discover all layer refs under `refs/jin/layers/`
2. Display commits for each ref that exists
3. Include mode-scope, mode-project, and mode-scope-project layers

**Impact**: Users cannot see commit history for important layers, making debugging and auditing difficult.

---

## Minor Issues (Nice to Fix)

### Issue 3: Test Suite Has Incorrect Ref Paths

**Severity**: Minor (test bug, not implementation bug)
**PRD Reference**: N/A
**Expected Behavior**: All tests should pass.

**Actual Behavior**: Several integration tests fail because they check for incorrect ref paths. The tests expect paths like `refs/jin/layers/mode/<name>` but the implementation correctly uses `refs/jin/layers/mode/<name>/_` (the `/_` suffix is required to allow nested refs).

**Failing Tests**:
- `test_layer_routing_mode_base` in `tests/mode_scope_workflow.rs`
- `test_layer_routing_mode_project` in `tests/mode_scope_workflow.rs`
- `test_layer_routing_mode_scope` in `tests/mode_scope_workflow.rs`
- `test_layer_routing_mode_scope_project` in `tests/mode_scope_workflow.rs`
- `test_layer_routing_project_base` in `tests/mode_scope_workflow.rs`
- `test_multiple_modes_isolated` in `tests/mode_scope_workflow.rs`
- Plus 4 tests in `tests/validation_tests.rs`

**Suggested Fix**: Update the `assert_layer_ref_exists()` calls to include the `/_` suffix:
```rust
// Wrong:
assert_layer_ref_exists(&format!("refs/jin/layers/mode/{}", mode_name), ...);

// Correct:
assert_layer_ref_exists(&format!("refs/jin/layers/mode/{}/_", mode_name), ...);
```

**Impact**: Test suite doesn't pass, giving false impression of implementation bugs. The actual implementation is correct.

---

### Issue 4: One Flaky Unit Test

**Severity**: Minor
**Test**: `commands::scope::tests::test_create_mode_bound_scope`
**Expected Behavior**: Test should pass consistently.

**Actual Behavior**: Test fails when run with other tests but passes when run individually. This indicates a test isolation issue (likely sharing global state like `JIN_DIR`).

**Suggested Fix**: Ensure the test properly sets up and tears down its own `JIN_DIR` environment, or add `#[serial]` attribute to prevent concurrent execution.

**Impact**: CI/CD may have intermittent failures.

---

## Testing Summary

- **Total tests performed**: ~650 (unit + integration)
- **Passing**: ~640+
- **Failing**: ~8-12 (mostly test bugs, not implementation bugs)
- **Manual workflows tested**: 15+
- **Manual workflows passing**: All core workflows work correctly

**Areas with good coverage**:
- Layer routing (all 9 layers)
- Mode and scope lifecycle
- Gitignore managed block
- Basic add/commit workflow
- SIGPIPE handling

**Areas needing more attention**:
- Structured merge behavior (has bug)
- Commit history display (has bug)
- Test suite reliability (needs ref path fixes)

---

## Verified Working Features

The following PRD requirements are **correctly implemented**:

1. ✅ 9-layer hierarchy enforced (§4)
2. ✅ Layer routing with all flag combinations (§9.1)
3. ✅ Active context persistence (§7)
4. ✅ `.gitignore` managed block (§8)
5. ✅ Mode and scope lifecycle commands (§13)
6. ✅ Jin init/add/commit workflow (§6)
7. ✅ SIGPIPE handling (recently fixed)
8. ✅ `--local` flag for Layer 8 (recently added)
9. ✅ Help text with layer routing table (recently added)
10. ✅ Atomic commits across layers (§6.2)

---

## Conclusion

The Jin CLI implementation is **architecturally sound** and **mostly complete**. The two Major bugs are fixable without major refactoring:

1. **Structured merge conflict detection** - Need to use deep merge instead of content comparison
2. **`jin log` display** - Need to show all layer commits

After fixing these issues and updating the test ref paths, the implementation will fully comply with the PRD requirements.
