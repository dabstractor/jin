# Integration Test Naming Conventions in Rust Projects

## Overview

Rust has specific conventions for integration tests that differ from unit tests. Understanding these conventions is crucial for maintaining a well-organized test suite.

## File Organization

### Standard Integration Test Location

```
project/
├── src/
│   ├── main.rs
│   └── lib.rs
├── tests/
│   ├── cli_basic.rs          # Integration tests
│   ├── cli_add_local.rs
│   ├── conflict_workflow.rs
│   └── common/               # Shared test utilities
│       ├── mod.rs
│       ├── fixtures.rs
│       ├── assertions.rs
│       └── git_helpers.rs
└── Cargo.toml
```

**Key Points**:
- Integration tests go in the `tests/` directory at the crate root
- Each file in `tests/` is compiled as a separate crate
- Tests in `tests/` can only use public APIs from your crate
- Common utilities go in `tests/common/` module

### Test File Naming Patterns

From the jin project, the following patterns are observed:

#### 1. Command-Specific Tests
```
cli_basic.rs           # Basic command functionality
cli_add_local.rs       # Specific command (add --local)
cli_apply_conflict.rs  # Command + specific scenario
cli_import.rs          # Import command tests
cli_list.rs            # List command tests
cli_mv.rs              # Move/rename command tests
cli_reset.rs           # Reset command tests
cli_resolve.rs         # Resolve command tests
cli_diff.rs            # Diff command tests
```

**Pattern**: `cli_<command>.rs` for command-specific tests

#### 2. Workflow Tests
```
conflict_workflow.rs      # End-to-end conflict resolution
resolve_workflow.rs       # Resolve command workflow
sync_workflow.rs          # Sync operations workflow
mode_scope_workflow.rs    # Mode and scope interaction
core_workflow.rs          # Core functionality workflow
pull_merge.rs            # Pull and merge operations
```

**Pattern**: `<feature>_workflow.rs` for multi-step workflows

#### 3. Feature/Domain Tests
```
workspace_validation.rs     # Workspace validation logic
destructive_validation.rs   # Destructive operation validation
atomic_operations.rs        # Atomic operation guarantees
export_committed.rs         # Export committed files
error_scenarios.rs          # Error handling scenarios
repair_check.rs             # Repair command tests
```

**Pattern**: `<domain>_<type>.rs` for domain-specific tests

## Test Function Naming Conventions

### 1. Descriptive Snake Case

```rust
#[test]
fn test_add_local_routes_to_layer_8() -> Result<(), Box<dyn std::error::Error>> {
    // Clear description of what is being tested
}
```

**Pattern**: `test_<feature>_<condition>_<expected_result>()`

### 2. State-Based Naming

```rust
#[test]
fn test_status_shows_conflicts_during_pause() {
    // Tests that status command shows conflicts when paused
}

#[test]
fn test_status_no_conflicts_normal_display() {
    // Tests normal display when no conflicts
}
```

**Pattern**: `test_<command>_<state>_<condition>()`

### 3. Feature + Flag Naming

```rust
#[test]
fn test_add_local_rejects_mode_flag() {
    // Tests that --local flag cannot be combined with --mode
}

#[test]
fn test_add_local_rejects_global_flag() {
    // Tests that --local flag cannot be combined with --global
}
```

**Pattern**: `test_<feature>_<behavior>_<input>()`

### 4. Complete Workflow Naming

```rust
#[test]
fn test_full_workflow_conflict_to_resolution() {
    // Tests entire workflow from conflict detection through resolution
}

#[test]
fn test_add_local_commit_apply_workflow() {
    // Tests add -> commit -> apply workflow
}
```

**Pattern**: `test_<scope>_workflow_<scenario>()`

### 5. Edge Case Naming

```rust
#[test]
fn test_resolve_validates_conflict_markers() {
    // Tests validation of conflict markers
}

#[test]
fn test_resolve_no_paused_state() {
    // Tests behavior when no paused state exists
}

#[test]
fn test_resolve_file_not_in_conflict() {
    // Tests error when file not in conflict
}

#[test]
fn test_resolve_empty_jinmerge_fails() {
    // Tests failure on empty .jinmerge file
}

#[test]
fn test_resolve_missing_jinmerge_file() {
    // Tests error when .jinmerge file doesn't exist
}
```

**Pattern**: `test_<feature>_<edge_case>()`

### 6. Positive vs Negative Test Naming

```rust
// Positive case
#[test]
fn test_apply_creates_multiple_jinmerge_files() {
    // Tests that apply creates multiple .jinmerge files
}

// Negative case
#[test]
fn test_link_invalid_url_format() {
    // Tests that invalid URL format is rejected
}
```

**Patterns**:
- Positive: `test_<feature>_<expected_behavior>()`
- Negative: `test_<feature>_invalid_<input>()`

## Module Documentation Comments

**Best Practice**: Document test file purpose at the top

```rust
//! End-to-end integration tests for conflict resolution workflow
//!
//! These tests verify the complete workflow from conflict detection
//! through resolution, including:
//! - .jinmerge file creation during apply conflicts
//! - Paused state persistence and recovery
//! - Resolve command validation and workflow
//! - Status command conflict state display
//! - Error scenarios and edge cases
```

## Test Section Comments

**Best Practice**: Use comment sections to organize complex tests

```rust
#[test]
fn test_full_workflow_conflict_to_resolution() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // ========== STEP 1: Create and activate mode ==========
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ========== STEP 2: Add file to global layer ==========
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();
    // ...

    // ========== ASSERT: Verify .jinmerge file created ==========
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists());
}
```

## Regression Test Naming

For bug fixes, use descriptive naming:

```rust
/// Test that structured files (JSON) automatically deep merge across layers
/// without creating .jinmerge conflict files.
///
/// This is a regression test for the bug where structured files were incorrectly
/// creating .jinmerge files even when content could be deep merged.
#[test]
fn test_structured_file_auto_merge() -> Result<(), Box<dyn std::error::Error>> {
    // Test implementation
}
```

**Pattern**: `test_<bug_description>()` or `test_<feature>_regression()`

## Test Organization within Files

### Grouping by Category

```rust
// ================== TEST 1: ROUTING TO USERLOCAL ==================
#[test]
fn test_add_local_routes_to_layer_8() { /* ... */ }

// ================== TEST 2: REJECTS --MODE FLAG ==================
#[test]
fn test_add_local_rejects_mode_flag() { /* ... */ }

// ================== TEST 3: REJECTS --GLOBAL FLAG ==================
#[test]
fn test_add_local_rejects_global_flag() { /* ... */ }

// ================== TEST 4: COMPLETE WORKFLOW ==================
#[test]
fn test_add_local_commit_apply_workflow() { /* ... */ }
```

### Category Sections

```rust
// ============================================================
// Link Command Integration Tests
// ============================================================

#[test]
fn test_link_invalid_url_empty() { /* ... */ }

// ============================================================
// Completion Command Integration Tests
// ============================================================

#[test]
fn test_completion_bash() { /* ... */ }

// ============================================================
// Status Command - Conflict State Integration Tests
// ============================================================

#[test]
fn test_status_no_conflicts_normal_display() { /* ... */ }
```

## Naming Anti-Patterns to Avoid

### 1. Vague Names
```rust
// BAD
#[test]
fn test_it_works() { }

// GOOD
#[test]
fn test_add_command_stages_file_in_correct_layer() { }
```

### 2. Test Numbers
```rust
// BAD
#[test]
fn test_add_001() { }

// GOOD
#[test]
fn test_add_with_mode_flag() { }
```

### 3. Non-Descriptive Names
```rust
// BAD
#[test]
fn test_check() { }

// GOOD
#[test]
fn test_status_shows_dirty_workspace() { }
```

## Best Practices Summary

1. **Use descriptive snake_case**: `test_add_command_stages_file`
2. **Include expected outcome**: `test_add_fails_without_init`
3. **Group related tests**: Use comment sections
4. **Document file purpose**: Module-level doc comments
5. **Name for workflow tests**: `test_<feature>_workflow`
6. **Name for command tests**: `test_<command>_<scenario>`
7. **Name for edge cases**: `test_<feature>_<edge_case>`
8. **Use regression markers**: Document bug fixes clearly
9. **Organize by feature**: One file per major feature
10. **Common utilities module**: Shared test helpers in `tests/common/`

## Test File Structure Template

```rust
//! Integration tests for <feature>
//!
//! <Detailed description of what this test file covers>

mod common;
use common::fixtures::*;
use common::assertions::*;

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

// ============================================================
// Basic Functionality Tests
// ============================================================

#[test]
fn test_<feature>_basic_behavior() {
    // Test implementation
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_<feature>_edge_case_1() {
    // Test implementation
}

// ============================================================
// Workflow Tests
// ============================================================

#[test]
fn test_<feature>_workflow_scenario() {
    // Test implementation
}
```

## Additional Resources

- Rust testing conventions: https://rust-lang.github.io/api-guidelines/testing.html
- Integration test documentation: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
- Naming conventions RFC: https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html
