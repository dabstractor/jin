# Architecture Analysis: Conflict Detection Bug

This directory contains the complete analysis of the conflict detection failure in Jin's apply workflow.

## Documents

### 1. [README.md](./README.md) - Executive Summary
**Start here** for a quick overview of the problem, root cause, and fix.

**Contents:**
- Problem statement
- Root cause analysis
- What should happen vs what actually happens
- The fix (with code example)
- Impact assessment
- Files requiring changes

### 2. [test_failure_analysis.md](./test_failure_analysis.md) - Complete Technical Analysis
Comprehensive analysis of all 5 failing tests with code references and visual diagrams.

**Contents:**
- Executive summary
- Critical finding: missing collision detection
- Test expectations vs reality (all 5 tests)
- Root cause analysis with code flow
- PRD confirmation (section 11.3)
- Detailed fix requirements
- Test-by-test breakdown with examples
- Visual flow comparison (current vs required)
- Implementation strategy

### 3. [merge_analysis.md](./merge_analysis.md) - Merge System Deep Dive
Technical analysis of how the merge system works and why it doesn't detect conflicts.

**Contents:**
- Overview of merge architecture
- Layer merge orchestration
- Deep merge implementation
- Text merge implementation
- Why conflicts are never detected
- Conflict handling infrastructure (exists but never triggered)

## Key Findings

### The Problem
All 5 conflict tests fail because **conflicts are never detected**. The merge system always succeeds by taking the higher layer's value, so the pause workflow is never triggered.

### The Root Cause
Missing **pre-merge collision detection**. The code expects `deep_merge()` to return `JinError::MergeConflict`, but deep merge is designed for deterministic merging and never returns conflicts.

### The Solution
Add collision detection BEFORE the merge:
1. Check if file exists in >1 layer
2. Compare content across layers
3. If different → mark as conflict (don't merge)
4. All existing conflict handling will work

### Why This Happened
- Deep merge works correctly for its purpose (RFC 7396 semantics)
- What's missing is a separate "conflict detector" layer
- The PRD requires conservative behavior (pause on collisions)
- Current implementation is liberal (auto-merge everything)

## Impact

- **Feature complete failure**: PRD section 11.3 requirements not met
- **All infrastructure exists**: `.jinmerge` files, paused state, resolution workflow
- **Fix is surgical**: Only need to add detection, no refactoring
- **Tests are correct**: All 5 tests properly validate PRD requirements

## Next Steps

1. Read [README.md](./README.md) for overview
2. Read [test_failure_analysis.md](./test_failure_analysis.md) for details
3. Review [merge_analysis.md](./merge_analysis.md) for technical context
4. Implement collision detection in `src/merge/layer.rs`
5. Run tests to verify fix

## Quick Reference

**Failing tests:** `tests/cli_apply_conflict.rs` (all 5 tests)

**File to modify:** `src/merge/layer.rs` (function: `merge_layers()`)

**Functions to add:**
- `find_layers_containing_file()` - Find which layers have a file
- `has_different_content_across_layers()` - Compare content across layers

**Code location:** Lines 110-120 in `src/merge/layer.rs`

**Expected result:** All 5 tests pass, conflicts are detected and paused

## Related PRD Sections

- **Section 11.3**: Conflict Resolution
- **Section 11.2**: Merge Priority
- **Section 11.1**: Structured Merge Rules
