# Codebase Tree Overview: Merge and Git Modules

**Generated:** 2026-01-10
**Focus:** Understanding codebase structure for implementing `find_layers_containing_file()` helper
**Related PRP:** plan_bugfix/P1M1T1S1/PRP.md

---

## Table of Contents

1. [Overall src/ Directory Structure](#overall-src-directory-structure)
2. [Current Codebase Tree](#current-codebase-tree)
3. [Helper Function Organization Patterns](#helper-function-organization-patterns)
4. [Desired Codebase Tree](#desired-codebase-tree)
5. [Key Insights for Implementation](#key-insights-for-implementation)

---

## Overall src/ Directory Structure

The Jin codebase follows a modular Rust architecture with clear separation of concerns:

```
src/
├── audit/          # Audit logging and entry tracking
├── cli/            # Command-line interface argument parsing
├── commands/       # CLI command implementations (add, apply, commit, etc.)
├── commit/         # Commit pipeline and workflow orchestration
├── core/           # Core types and infrastructure (Layer, JinError, Result, etc.)
├── git/            # Git layer integration (repository, refs, trees, objects)
├── merge/          # Merge engine (deep merge, layer merge orchestration, text merge)
├── staging/        # Staging area management (index, metadata, router)
├── lib.rs          # Library root with module declarations and public API
└── main.rs         # Binary entry point
```

---

## Current Codebase Tree

### Detailed Focus Areas

```
/home/dustin/projects/jin/src/
│
├── merge/                          # MERGE ENGINE - Primary location for new helper
│   ├── mod.rs                      # Module exports (deep_merge, merge_layers, text_merge)
│   ├── layer.rs                    # ✓ LAYER MERGE ORCHESTRATION
│   │   ├── Public API:
│   │   │   ├── merge_layers()              # Main orchestration function
│   │   │   ├── get_applicable_layers()     # Returns layers for context
│   │   │   ├── detect_format()             # File format detection
│   │   │   └── parse_content()             # Content parsing
│   │   ├── Private Helpers:
│   │   │   ├── collect_all_file_paths()    # Collects files across layers
│   │   │   └── merge_file_across_layers()  # Merges single file across layers
│   │   └── Tests: #![cfg(test)] module with unit tests
│   │
│   ├── deep.rs                      # RFC 7396 deep merge implementation
│   ├── jinmerge.rs                  # JinMerge conflict file format
│   ├── text.rs                      # 3-way text merge for plain text files
│   └── value.rs                     # MergeValue enum (universal data representation)
│
├── git/                            # GIT LAYER INTEGRATION
│   ├── mod.rs                      # Module exports (JinRepo, RefOps, TreeOps, etc.)
│   ├── repo.rs                     # JinRepo wrapper around git2::Repository
│   ├── refs.rs                     # Reference operations (RefOps trait)
│   ├── tree.rs                     # Tree walking utilities (TreeOps trait)
│   ├── objects.rs                  # Object creation (ObjectOps trait)
│   ├── transaction.rs              # Transaction wrapper for atomic updates
│   ├── merge.rs                    # Git merge operations
│   └── remote.rs                   # Remote operations (fetch, pull, push)
│
├── core/                          # CORE TYPES AND INFRASTRUCTURE
│   ├── mod.rs                     # Module exports (Layer, JinError, Result, etc.)
│   ├── layer.rs                   # ✓ Layer enum definition and methods
│   ├── error.rs                   # JinError enum and Result type
│   ├── config.rs                  # JinConfig, ProjectContext, UserConfig
│   └── jinmap.rs                  # JinMap type for configuration data
│
├── commands/                      # CLI COMMAND IMPLEMENTATIONS
│   ├── add.rs                     # jin add command
│   ├── apply.rs                   # jin apply command
│   ├── commit_cmd.rs              # jin commit command
│   ├── diff.rs                    # jin diff command
│   ├── export.rs                  # jin export command
│   ├── import_cmd.rs              # jin import command
│   ├── layers.rs                  # jin layers command
│   ├── list.rs                    # jin list command (will use new helper)
│   ├── mode.rs                    # jin mode command
│   ├── scope.rs                   # jin scope command
│   ├── status.rs                  # jin status command (will use new helper)
│   ├── reset.rs                   # jin reset command
│   ├── resolve.rs                 # jin resolve command
│   └── [other commands...]
│
└── tests/                         # INTEGRATION TESTS
    ├── cli_list.rs                # List command integration tests
    ├── cli_diff.rs                # Diff command integration tests
    ├── cli_resolve.rs             # Resolve command integration tests
    ├── core_workflow.rs           # Core workflow tests
    ├── mode_scope_workflow.rs     # Mode and scope workflow tests
    ├── conflict_workflow.rs       # Conflict resolution tests
    └── common/                    # Test utilities
        ├── fixtures.rs            # TestFixture setup
        ├── assertions.rs          # Custom assertions
        ├── git_helpers.rs         # Git operation helpers
        └── mod.rs                 # Common module exports
```

---

## Helper Function Organization Patterns

### Pattern 1: Module Organization in `merge/layer.rs`

The `merge/layer.rs` file demonstrates a clear pattern for organizing helper functions:

```rust
// Public API (exported via mod.rs)
pub fn merge_layers(...)           // Main orchestration
pub fn get_applicable_layers(...)  // Layer selection logic
pub fn detect_format(...)          // Format detection
pub fn parse_content(...)          // Content parsing

// Private helpers (module-internal)
fn collect_all_file_paths(...)     // Implementation detail
fn merge_file_across_layers(...)   // Implementation detail

// Tests
#[cfg(test)]
mod tests { ... }
```

**Key observations:**
- Public functions are exported via `mod.rs` for external use
- Private helpers are `fn` (not `pub`) and used only within the module
- Tests are in a nested `#[cfg(test)] mod tests` module
- Documentation comments explain purpose and usage

### Pattern 2: Trait Implementation in `git/tree.rs`

```rust
pub trait TreeOps {
    // Trait methods
    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>>;
    fn read_file_from_tree(&self, tree_oid: Oid, path: &Path) -> Result<Vec<u8>>;
    // ... more methods
}

impl TreeOps for JinRepo {
    // Implementation
    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>> {
        // ... implementation
    }
}
```

### Pattern 3: Existing Helper in `merge/layer.rs`

The existing `collect_all_file_paths()` helper shows the pattern we should follow:

```rust
/// Collect all unique file paths across all applicable layers.
///
/// Iterates through each layer, resolves its Git ref, and lists all files
/// in its tree. Returns a set of unique paths.
fn collect_all_file_paths(
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<HashSet<PathBuf>> {
    let mut paths = HashSet::new();

    for layer in layers {
        let ref_path = layer.ref_path(...);

        if repo.ref_exists(&ref_path) {
            if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
                let commit = repo.inner().find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                for file_path in repo.list_tree_files(tree_oid)? {
                    paths.insert(PathBuf::from(file_path));
                }
            }
        }
    }

    Ok(paths)
}
```

**Pattern characteristics:**
- Takes `&[Layer]` and `&JinRepo` as parameters
- Iterates through layers to collect information
- Returns `Result<Collection>`
- Handles missing refs gracefully with `ref_exists()` check

---

## Desired Codebase Tree

### Where the New Helper Will Go

```
src/merge/layer.rs                # THIS IS WHERE WE ADD THE NEW HELPER
│
├── Current Public API:
│   ├── merge_layers()
│   ├── get_applicable_layers()
│   ├── detect_format()
│   └── parse_content()
│
├── Current Private Helpers:
│   ├── collect_all_file_paths()
│   └── merge_file_across_layers()
│
├── ✓ NEW HELPER TO ADD:
│   └── find_layers_containing_file()
│       ├── Purpose: Find which layers contain a specific file
│       ├── Signature: pub fn find_layers_containing_file(path: &Path, layers: &[Layer], config: &LayerMergeConfig, repo: &JinRepo) -> Result<Vec<Layer>>
│       ├── Logic:
│       │   1. Iterate through provided layers
│       │   2. For each layer, resolve its Git ref
│       │   3. Check if file exists in layer's tree
│       │   4. Return list of layers that contain the file
│       │   5. Handle missing refs gracefully (return empty vec)
│       ├── Returns: Vec<Layer> - layers containing the file (in precedence order)
│       └── Export: Add to mod.rs exports
│
└── Tests: #[cfg(test)] mod tests
    └── Add new tests for find_layers_containing_file()
```

### Updated mod.rs Exports

```rust
// src/merge/mod.rs (AFTER adding the helper)

pub use layer::{
    detect_format,
    get_applicable_layers,
    merge_layers,
    parse_content,
    FileFormat,
    LayerMergeConfig,
    LayerMergeResult,
    MergedFile,
    find_layers_containing_file,  // ← NEW EXPORT
};
```

### Usage in Commands

```
src/commands/list.rs              # WILL USE NEW HELPER
│
└── List command implementation:
    ├── Call find_layers_containing_file() to get layers with specific file
    ├── Display which layers contain the file
    └── Show file status in each layer (merged/modified/deleted)

src/commands/status.rs            # WILL USE NEW HELPER
│
└── Status command implementation:
    ├── Call find_layers_containing_file() for tracked files
    ├── Show file's layer provenance
    └── Display merge status across layers
```

---

## Key Insights for Implementation

### 1. **Helper Should Live in `src/merge/layer.rs`**

**Rationale:**
- `layer.rs` already contains layer-related helpers (`collect_all_file_paths`, `merge_file_across_layers`)
- The new helper operates on layers and file paths, fitting the module's purpose
- Consistent with existing code organization patterns
- Can reuse existing imports and types

### 2. **Function Should Be Public**

**Rationale:**
- Will be used by commands in `src/commands/` (list.rs, status.rs)
- Needs to be exported via `src/merge/mod.rs`
- Follows pattern of `get_applicable_layers()` which is also public

### 3. **Function Signature**

```rust
pub fn find_layers_containing_file(
    path: &Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>>
```

**Design decisions:**
- Takes `&[Layer]` instead of computing applicable layers internally (flexibility)
- Takes `&LayerMergeConfig` for mode/scope/project context
- Takes `&JinRepo` for Git operations
- Returns `Result<Vec<Layer>>` for error handling
- Returns `Vec<Layer>` (not `Vec<&Layer>`) for ownership consistency

### 4. **Implementation Pattern**

Based on `collect_all_file_paths()`, the implementation should:

```rust
pub fn find_layers_containing_file(
    path: &Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>> {
    let mut containing_layers = Vec::new();

    for layer in layers {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref()
        if !repo.ref_exists(&ref_path) {
            continue; // Layer doesn't exist, skip it
        }

        if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
            let commit = repo.inner().find_commit(commit_oid)?;
            let tree_oid = commit.tree_id();

            // Check if file exists in this layer's tree
            match repo.read_file_from_tree(tree_oid, path) {
                Ok(_) => containing_layers.push(*layer),
                Err(JinError::NotFound(_)) => continue, // File not in this layer
                Err(e) => return Err(e), // Propagate other errors
            }
        }
    }

    Ok(containing_layers)
}
```

### 5. **Test Structure**

Add tests in the `#[cfg(test)] mod tests` section:

```rust
#[cfg(test)]
mod tests {
    // ... existing tests ...

    // ========== find_layers_containing_file Tests ==========

    #[test]
    fn test_find_layers_single_layer() {
        // Test file in single layer
    }

    #[test]
    fn test_find_layers_multiple_layers() {
        // Test file in multiple layers
    }

    #[test]
    fn test_find_layers_no_layers() {
        // Test file not in any layer
    }

    #[test]
    fn test_find_layers_nonexistent_file() {
        // Test with file that doesn't exist
    }

    #[test]
    fn test_find_layers_precedence_order() {
        // Verify returned layers maintain precedence order
    }
}
```

### 6. **Integration Test Location**

Add integration tests in:
- `tests/cli_list.rs` - Test `jin list --file <path>` functionality
- `tests/common/fixtures.rs` - May need helper functions for setup

---

## Summary: Implementation Plan

1. **Location:** Add `find_layers_containing_file()` to `/home/dustin/projects/jin/src/merge/layer.rs`

2. **Export:** Add to `/home/dustin/projects/jin/src/merge/mod.rs` public exports

3. **Implementation:** Follow pattern of `collect_all_file_paths()` with:
   - Iterate through layers
   - Check ref existence
   - Resolve Git refs
   - Check file in tree
   - Return containing layers

4. **Tests:** Add unit tests in `layer.rs` `#[cfg(test)]` module

5. **Integration:** Update `list.rs` and `status.rs` commands to use new helper

6. **Documentation:** Add doc comment explaining purpose, parameters, return value, and usage

---

## Related Files

- **Implementation target:** `/home/dustin/projects/jin/src/merge/layer.rs`
- **Module exports:** `/home/dustin/projects/jin/src/merge/mod.rs`
- **Core types:** `/home/dustin/projects/jin/src/core/layer.rs`
- **Git operations:** `/home/dustin/projects/jin/src/git/tree.rs`, `/home/dustin/projects/jin/src/git/refs.rs`
- **Command usage:** `/home/dustin/projects/jin/src/commands/list.rs`
- **Unit tests:** In `src/merge/layer.rs` (inline `#[cfg(test)]` module)
- **Integration tests:** `/home/dustin/projects/jin/tests/cli_list.rs`

---

## Appendix: File Counts and Statistics

**Module sizes:**
- `src/merge/`: 6 files (mod.rs, layer.rs, deep.rs, jinmerge.rs, text.rs, value.rs)
- `src/git/`: 8 files (mod.rs, repo.rs, refs.rs, tree.rs, objects.rs, transaction.rs, merge.rs, remote.rs)
- `src/core/`: 5 files (mod.rs, layer.rs, error.rs, config.rs, jinmap.rs)
- `tests/`: 23 test files + common/ module

**Helper function density:**
- `merge/layer.rs`: 4 public + 2 private helpers
- `git/tree.rs`: 6 trait methods (all public)
- `git/refs.rs`: 6 trait methods + 1 standalone function

This analysis confirms that `merge/layer.rs` is the appropriate location for the new `find_layers_containing_file()` helper function.
