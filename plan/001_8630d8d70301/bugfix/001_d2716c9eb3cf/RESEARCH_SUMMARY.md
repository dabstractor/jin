# Bug Hunt Research Summary

## Overview

This document summarizes the research findings and task decomposition for the Jin CLI bug fix project. The research phase involved spawning multiple exploration agents to validate the PRD against the current codebase state before decomposing the work.

## Research Methodology

### 1. Spawned Research Agents

Four specialized exploration agents were dispatched to investigate:

1. **Merge Architecture Agent** - Analyzed `src/merge/layer.rs` and `src/merge/deep.rs`
2. **Log Command Agent** - Analyzed `src/commands/log.rs` and ref discovery patterns
3. **Test Ref Path Agent** - Found all incorrect ref path assertions in tests
4. **Flaky Test Agent** - Investigated test isolation issues in `src/commands/scope.rs`

### 2. Architecture Documentation

All research findings were documented in `plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/`:

- **merge_engine_analysis.md** - Detailed analysis of the two-phase merge flow and the conflict detection bug
- **log_command_analysis.md** - Analysis of hardcoded layer iteration vs dynamic ref discovery
- **test_infrastructure_analysis.md** - Test ref path patterns and isolation strategies

## Key Findings

### Bug 1: Structured Merge Conflict Detection (Major)

**Root Cause**: The merge engine checks for conflicts **before** attempting deep merge. For JSON/YAML/TOML files, `has_different_content_across_layers()` returns `true` when content differs, even though those differences should be resolved via deep merge.

**Correct Behavior**:
- Attempt deep merge first for structured files
- Only create conflicts when deep merge fails (syntax errors, incompatible types)
- Layer precedence should automatically resolve differences

**Fix Complexity**: 2 subtasks, 4 story points total
- Remove pre-merge conflict check for structured files (2 SP)
- Verify deep merge layer precedence (1 SP)
- Add integration test (2 SP)

### Bug 2: jin Log Missing Commits (Major)

**Root Cause**: The log command uses a hardcoded list of layers and canonical ref paths. It doesn't dynamically discover refs under `refs/jin/layers/`, so commits from mode-scope and other layers are missed.

**Correct Behavior**:
- Use `repo.list_refs("refs/jin/layers/**")` to discover all layer refs
- Parse each ref path to determine layer type
- Display commits for all discovered refs

**Fix Complexity**: 3 subtasks, 6 story points total
- Add `parse_layer_from_ref_path()` helper (2 SP)
- Replace hardcoded iteration with dynamic listing (2 SP)
- Add integration test (2 SP)

### Bug 3: Test Ref Path Assertions (Minor)

**Root Cause**: 4 test assertions in `tests/mode_scope_workflow.rs` expect ref paths without the `/_` suffix, but the implementation correctly requires the suffix for layers with child refs.

**Fix Complexity**: 4 subtasks, 4 story points total
- Fix `test_layer_routing_mode_base` (1 SP)
- Fix `test_layer_routing_mode_scope` (1 SP)
- Fix `test_multiple_modes_isolated` (1 SP)
- Run full test suite (1 SP)

### Bug 4: Flaky Test Isolation (Minor)

**Root Cause**: `test_create_mode_bound_scope` uses manual setup with environment variables and current directory changes, causing shared state issues when run with other tests.

**Correct Behavior**: Use the established `setup_unit_test()` pattern from `src/test_utils.rs` with absolute paths and proper cleanup.

**Fix Complexity**: 4 subtasks, 5 story points total
- Create test mode helper (1 SP)
- Create cleanup helper (1 SP)
- Refactor test to use isolation (2 SP)
- Verify parallel execution (1 SP)

## Task Decomposition

### Hierarchy

The project was decomposed into:

```
Phase (P1)
├── Milestone (P1.M1) - Structured Merge Fix
│   ├── Task (P1.M1.T1) - Update Merge Logic
│   │   ├── Subtask (P1.M1.T1.S1) - Remove pre-merge check [2 SP]
│   │   ├── Subtask (P1.M1.T1.S2) - Verify layer precedence [1 SP]
│   │   └── Subtask (P1.M1.T1.S3) - Add integration test [2 SP]
│   └── Task (P1.M1.T2) - Test Complex Merges
│       ├── Subtask (P1.M1.T2.S1) - Nested objects test [1 SP]
│       └── Subtask (P1.M1.T2.S2) - Array key merge test [1 SP]
├── Milestone (P1.M2) - jin Log Fix
│   └── Task (P1.M2.T1) - Dynamic Ref Discovery
│       ├── Subtask (P1.M2.T1.S1) - Add parser helper [2 SP]
│       ├── Subtask (P1.M2.T1.S2) - Replace iteration [2 SP]
│       └── Subtask (P1.M2.T1.S3) - Add integration test [2 SP]
├── Milestone (P1.M3) - Test Ref Path Fixes
│   └── Task (P1.M3.T1) - Update Assertions
│       ├── Subtask (P1.M3.T1.S1) - Fix mode_base test [1 SP]
│       ├── Subtask (P1.M3.T1.S2) - Fix mode_scope test [1 SP]
│       ├── Subtask (P1.M3.T1.S3) - Fix multiple_modes test [1 SP]
│       └── Subtask (P1.M3.T1.S4) - Run test suite [1 SP]
├── Milestone (P1.M4) - Flaky Test Fix
│   └── Task (P1.M4.T1) - Refactor Test Isolation
│       ├── Subtask (P1.M4.T1.S1) - Create helper [1 SP]
│       ├── Subtask (P1.M4.T1.S2) - Create cleanup [1 SP]
│       ├── Subtask (P1.M4.T1.S3) - Refactor test [2 SP]
│       └── Subtask (P1.M4.T1.S4) - Verify isolation [1 SP]
└── Milestone (P1.M5) - Verification
    └── Task (P1.M5.T1) - Final Verification
        ├── Subtask (P1.M5.T1.S1) - Run test suite [1 SP]
        ├── Subtask (P1.M5.T1.S2) - Manual verification [2 SP]
        └── Subtask (P1.M5.T1.S3) - Document fixes [1 SP]
```

### Story Point Summary

- **Total Subtasks**: 22
- **Total Story Points**: 23
- **Estimated Effort**: ~23 hours (assuming 1 hour per SP)
- **Major Bugs**: 2 (11 SP combined)
- **Minor Issues**: 2 (10 SP combined)
- **Verification**: 2 SP

## Subtask Context Scope Pattern

Each subtask includes a detailed `context_scope` field that defines:

1. **RESEARCH NOTE**: References to specific findings from `plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/`
2. **INPUT**: Specific data structures, functions, or dependencies from previous subtasks
3. **LOGIC**: Implementation instructions with file paths, line numbers, and code patterns
4. **OUTPUT**: Expected deliverables for consumption by subsequent subtasks

This ensures that each subtask can be executed by a developer who only sees that subtask's context, without needing to understand the entire project.

## PRD Compliance Verification

After completing all bug fixes, the implementation will fully comply with:

- **§11.1 "Structured Merge Rules"** - Deep merge for JSON/YAML/TOML with RFC 7396 semantics
- **§11.2 "Merge Priority"** - Layer precedence correctly determines merge winners
- **§18.6 "jin log [layer]"** - Commit history displayed for all layers

## Risk Assessment

- **Low Risk**: All fixes are isolated to specific modules
- **No Breaking Changes**: Bug fixes only, no API changes
- **Rollback Safe**: Each fix can be independently reverted if needed
- **Test Coverage**: Comprehensive test coverage ensures fixes work correctly

## Next Steps

1. **Review**: Product Owner reviews the bug_hunt_tasks.json
2. **Approval**: Stakeholder approval to proceed with implementation
3. **Execution**: PRP (Product Requirement Prompt) agents execute subtasks in dependency order
4. **Verification**: Final test suite execution and manual verification
5. **Release**: Bug fix release with summary documentation

## References

- **PRD**: `plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/prd_snapshot.md`
- **Architecture Docs**: `plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/`
- **Task Backlog**: `plan/001_8630d8d70301/bug_hunt_tasks.json`
