# Product Requirement Prompt (PRP): P1.M1.T1.S1 - Implement find_layers_containing_file() Helper

**Work Item:** P1.M1.T1.S1 - Implement find_layers_containing_file() helper
**Parent Task:** P1.M1.T1 - Add Collision Detection Helper Functions
**Parent Module:** P1.M1 - Fix Conflict Detection System
**Priority:** Critical (Bug Fix - Conflict Detection & Safety)
**Points:** 2

---

## Goal

**Feature Goal:** Implement a helper function that identifies which layers in Jin's 9-layer hierarchy contain a specific file, enabling pre-merge collision detection.

**Deliverable:** A public function `find_layers_containing_file()` in `src/merge/layer.rs` that:
- Takes a file path, layer list, merge config, and repository reference
- Returns a `Vec<Layer>` of all layers containing that file (in precedence order)
- Returns empty vec if file not found in any layer
- Uses existing `TreeOps` trait methods for file existence checks

**Success Definition:**
1. Function compiles without errors or warnings
2. All unit tests pass (unit tests in `layer.rs`, integration tests using `TestFixture`)
3. Function is exported via `src/merge/mod.rs`
4. Function correctly handles edge cases: non-existent files, empty layer list, missing layer refs
5. Returned layers maintain input precedence order

---

## User Persona

**Target User:** Jin core system developers working on conflict detection and merge pipeline logic.

**Use Case:** During the merge process, before attempting to merge files, the system needs to detect which layers contain a given file. This enables:
- Early collision detection (files modified in multiple layers)
- Conflict routing decisions (text merge vs deep merge vs conflict markers)
- Layer provenance display in `jin list` and `jin status` commands

**User Journey:**
1. System calls `merge_layers()` with active layer configuration
2. For each file to merge, system calls `find_layers_containing_file(path, layers, config, repo)`
3. If function returns 2+ layers, system checks for content conflicts
4. Based on conflict detection, system routes to appropriate merge strategy

**Pain Points Addressed:**
- Currently no way to detect which layers contain a file before merge
- Collision detection happens too late (during merge, not before)
- Cannot provide helpful layer provenance information to users

---

## Why

- **Critical for conflict detection:** This helper is the foundation for P1.M1.T2 (Integrate Collision Detection into Merge Pipeline)
- **Enables user-friendly diagnostics:** `jin list` and `jin status` can show which layers contain each file
- **Prevents data loss:** Early detection of files modified in multiple layers prevents silent overwrites
- **Follows existing patterns:** Builds on proven patterns from `collect_all_file_paths()` and `merge_file_across_layers()`

---

## What

### Function Specification

```rust
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>>
```

### Behavior

1. **Input Validation:**
   - Accepts any valid `Path` reference
   - Accepts empty layer slice (returns empty vec)
   - Accepts layers that don't exist yet (gracefully skipped)

2. **Iteration Logic:**
   - Iterates through `layers` in the order provided (maintains precedence)
   - For each layer:
     - Resolves layer's Git ref path using `layer.ref_path()`
     - Checks if ref exists using `repo.ref_exists()`
     - If ref exists, resolves commit and gets tree OID
     - Checks if file exists in tree using `repo.get_tree_entry()`

3. **Return Value:**
   - `Ok(Vec<Layer>)`: All layers containing the file (in input order)
   - Empty vec if file not found in any layer (not an error)
   - `Err(JinError)`: Only for actual failures (Git errors, repo corruption)

### Success Criteria

- [ ] Function signature matches specification exactly
- [ ] Handles non-existent layer refs gracefully (skips, doesn't error)
- [ ] Returns empty Vec when file not in any layer
- [ ] Returns layers in same order as input (maintains precedence)
- [ ] Propagates Git errors appropriately
- [ ] Has comprehensive unit tests covering all edge cases
- [ ] Is exported via `src/merge/mod.rs`

---

## All Needed Context

### Context Completeness Check

This PRP provides complete context for implementing `find_layers_containing_file()`:
- File location and structure patterns to follow
- Exact function signature and implementation logic
- Error handling patterns from existing code
- Test patterns from existing test fixtures
- Git operation patterns from TreeOps trait
- Module export patterns
- External documentation references

### Documentation & References

```yaml
# MUST READ - Core implementation files

- file: /home/dustin/projects/jin/src/merge/layer.rs
  why: Contains the pattern to follow for helper functions
  pattern: Follow `collect_all_file_paths()` pattern for layer iteration and ref resolution
  gotcha: Always check `ref_exists()` before `resolve_ref()` to avoid errors on non-existent layers

- file: /home/dustin/projects/jin/src/merge/mod.rs
  why: Shows module export pattern - need to add new function here
  pattern: Add `find_layers_containing_file` to the `pub use layer::{...}` list

- file: /home/dustin/projects/jin/src/core/layer.rs
  why: Layer enum definition with ref_path() method
  pattern: Layer::ref_path(mode, scope, project) returns Git ref path for each layer
  gotcha: Some layers use `/_` suffix to avoid Git ref naming conflicts (ModeBase, ModeScope)

- file: /home/dustin/projects/jin/src/git/tree.rs
  why: TreeOps trait with get_tree_entry() method for file existence checks
  pattern: `repo.get_tree_entry(tree_oid, path)` returns Err if file doesn't exist
  gotcha: Use `is_ok()` check rather than catching specific error

- file: /home/dustin/projects/jin/src/git/refs.rs
  why: RefOps trait with ref_exists() method
  pattern: `repo.ref_exists(&ref_path)` returns bool (not Result!)
  gotcha: Check ref_exists before resolve_ref to prevent errors

- file: /home/dustin/projects/jin/src/core/error.rs
  why: JinError enum for error handling
  pattern: Use `JinError::NotFound` for missing files, `JinError::Git` for Git failures

- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Test fixture patterns for integration tests
  pattern: Use `TestFixture::new()` and `fixture.set_jin_dir()` for test isolation
  gotcha: Always store TempDir in variable or it gets dropped prematurely

# Research documentation

- docfile: plan_bugfix/P1M1T1S1/research/helper_function_patterns.md
  why: Extracted patterns from existing helper functions
  section: "Recommended Signature" and "Implementation Outline"

- docfile: plan_bugfix/P1M1T1S1/research/codebase_tree.md
  why: Shows where the new helper fits in the codebase structure
  section: "Desired Codebase Tree"

- docfile: plan_bugfix/P1M1T1S1/research/git_mocking_research.md
  why: Patterns for testing git operations without full repository
  section: "Code Examples" for test setup patterns

# External documentation

- url: https://docs.rs/git2/latest/git2/struct.Tree.html#method.get_path
  why: Tree::get_path() API documentation - returns Err if path doesn't exist
  critical: This is how we check file existence in a tree

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.find_tree
  why: Repository::find_tree() for getting Tree from OID
  critical: Needed to access tree after resolving commit

- url: https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html
  why: TempDir for creating isolated test repositories
  critical: Auto-cleanup when dropped - must keep in scope
```

### Current Codebase Tree (Relevant Sections)

```bash
src/
├── merge/
│   ├── mod.rs                      # Module exports
│   └── layer.rs                    # TARGET FILE - Add helper here
│       ├── Public: merge_layers(), get_applicable_layers(), detect_format(), parse_content()
│       ├── Private: collect_all_file_paths(), merge_file_across_layers()
│       └── Tests: #[cfg(test)] mod tests
│
├── git/
│   ├── tree.rs                     # TreeOps trait (get_tree_entry, read_file_from_tree)
│   ├── refs.rs                     # RefOps trait (ref_exists, resolve_ref)
│   └── repo.rs                     # JinRepo wrapper
│
├── core/
│   ├── layer.rs                    # Layer enum with ref_path() method
│   └── error.rs                    # JinError enum and Result type
│
└── commands/
    ├── list.rs                     # Will use this helper (future work)
    └── status.rs                   # Will use this helper (future work)

tests/
└── common/
    └── fixtures.rs                 # TestFixture setup helpers
```

### Desired Codebase Tree (After Implementation)

```bash
src/merge/layer.rs                  # MODIFIED FILE
│
├── Current Public API:
│   ├── merge_layers()
│   ├── get_applicable_layers()
│   ├── detect_format()
│   └── parse_content()
│
├── NEW HELPER TO ADD:
│   └── find_layers_containing_file()   # <-- NEW FUNCTION
│       ├── Location: After get_applicable_layers()
│       ├── Visibility: pub (exported via mod.rs)
│       ├── Signature: (file_path: &Path, layers: &[Layer], config: &LayerMergeConfig, repo: &JinRepo) -> Result<Vec<Layer>>
│       └── Tests: Add unit tests in #[cfg(test)] mod tests section
│
└── Private Helpers (unchanged):
    ├── collect_all_file_paths()
    └── merge_file_across_layers()

src/merge/mod.rs                    # MODIFIED FILE
│
└── pub use layer::{
    detect_format,
    get_applicable_layers,
    merge_layers,
    parse_content,
    find_layers_containing_file,    # <-- NEW EXPORT
    FileFormat,
    LayerMergeConfig,
    LayerMergeResult,
    MergedFile,
};
```

### Known Gotchas of our Codebase & Library Quirks

```rust
// CRITICAL: Always check ref_exists() before resolve_ref()
// Layers that don't exist yet should be silently skipped, not cause errors
if !repo.ref_exists(&ref_path) {
    continue;  // Skip this layer
}

// CRITICAL: Some layer refs use /_ suffix to avoid Git ref conflicts
// ModeBase and ModeScope use /_ because they have child refs
Layer::ModeBase.ref_path(Some("dev"), None, None)
// Returns: "refs/jin/layers/mode/dev/_"  (note the /_ suffix)

// CRITICAL: TreeOps::get_tree_entry() returns Err for non-existent files
// Use .is_ok() check rather than matching specific error
if repo.get_tree_entry(tree_oid, file_path).is_ok() {
    // File exists
}

// CRITICAL: TempDir cleanup in tests
// TempDir MUST be stored in a variable or it gets dropped immediately
let fixture = TestFixture::new().unwrap();  // Keep fixture in scope!
// If you write `TestFixture::new().unwrap().set_jin_dir();` the dir is deleted before set_jin_dir runs

// CRITICAL: Layer ref paths
// Layer::ref_path() requires mode/scope/project for some layers
// Pass None for layers that don't need those parameters
layer.ref_path(
    config.mode.as_deref(),   // Option<&str>
    config.scope.as_deref(),  // Option<&str>
    config.project.as_deref(), // Option<&str>
)

// CRITICAL: Error handling - use JinError::Git for git2 errors
// Don't try to catch specific git2 errors, let them propagate as JinError::Git
let commit = repo.inner().find_commit(commit_oid)?;  // ? operator converts to JinError

// CRITICAL: Layer is Copy type
// When pushing to Vec, use *layer to copy the Layer value
found_layers.push(*layer);  // NOT: found_layers.push(layer.clone());
```

---

## Implementation Blueprint

### Data Models and Structure

No new data models needed - uses existing types:
- `Layer`: Copy enum from `src/core/layer.rs`
- `LayerMergeConfig`: Existing struct with mode/scope/project context
- `JinRepo`: Wrapper around git2::Repository
- `Result<T>`: Type alias for `std::result::Result<T, JinError>`

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD find_layers_containing_file() to src/merge/layer.rs
  - LOCATION: After get_applicable_layers() function (around line 246)
  - IMPLEMENT: Function following exact signature specification
  - FOLLOW pattern: collect_all_file_paths() for layer iteration logic
  - NAMING: snake_case function name, descriptive parameter names
  - LOGIC:
    1. Create empty Vec<Layer> for results
    2. Iterate through input layers slice
    3. For each layer, resolve ref_path with context
    4. Check ref_exists() - skip if not found
    5. Resolve commit and get tree OID
    6. Check get_tree_entry() for file existence
    7. If exists, push *layer to results
    8. Return Ok(results)
  - ERROR HANDLING: Use ? operator to propagate Git errors
  - PLACEMENT: src/merge/layer.rs, public function section

Task 2: EXPORT function via src/merge/mod.rs
  - FIND: pub use layer::{...} block (around line 33)
  - ADD: find_layers_containing_file to the export list
  - PRESERVE: Alphabetical ordering or logical grouping
  - PLACEMENT: src/merge/mod.rs

Task 3: ADD unit tests to src/merge/layer.rs
  - LOCATION: #[cfg(test)] mod tests section (around line 276)
  - IMPLEMENT: 5+ test functions covering:
    1. Single layer containing file
    2. Multiple layers containing file
    3. File not in any layer (empty vec)
    4. Non-existent file path
    5. Empty layer list
    6. Layers with non-existent refs (gracefully skipped)
    7. Precedence order maintained
  - FOLLOW pattern: Existing tests in layer.rs use crate-level imports
  - USE: Test fixtures from tests/common/fixtures.rs pattern
  - MOCK: Create real test repos with tempfile (not trait mocks)
  - PLACEMENT: In #[cfg(test)] mod tests, add new section "find_layers_containing_file Tests"

Task 4: VERIFY compilation and type checking
  - RUN: cargo check --all-targets
  - EXPECTED: No errors, no warnings
  - FIX: Any type mismatches or missing imports
```

### Implementation Patterns & Key Details

```rust
// Location: src/merge/layer.rs - add after get_applicable_layers() around line 246

use crate::core::{JinError, Layer, Result};
use crate::git::{JinRepo, RefOps, TreeOps};
use std::path::Path;

/// Find which layers contain a specific file.
///
/// Iterates through the provided layers in precedence order and checks
/// if each layer's Git tree contains the specified file. Layers that
/// don't exist yet are gracefully skipped.
///
/// # Arguments
///
/// * `file_path` - Path to the file to search for (relative to repo root)
/// * `layers` - Layers to search, in precedence order
/// * `config` - Merge configuration with mode/scope/project context
/// * `repo` - Jin repository for Git operations
///
/// # Returns
///
/// * `Ok(Vec<Layer>)` - Layers containing the file, in input order
/// * `Err(JinError)` - Git operation failure
///
/// # Examples
///
/// ```ignore
/// use jin::merge::{find_layers_containing_file, LayerMergeConfig};
/// use jin::core::Layer;
/// use std::path::Path;
///
/// let config = LayerMergeConfig { /* ... */ };
/// let layers = vec![Layer::GlobalBase, Layer::ModeBase];
/// let containing = find_layers_containing_file(
///     Path::new("config.json"),
///     &layers,
///     &config,
///     &repo
/// )?;
/// ```
pub fn find_layers_containing_file(
    file_path: &Path,
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
        // Layer refs may not exist yet - skip gracefully
        if !repo.ref_exists(&ref_path) {
            continue;
        }

        // Resolve the commit for this layer
        let commit_oid = repo.resolve_ref(&ref_path);
        if let Ok(commit_oid) = commit_oid {
            let commit = repo.inner().find_commit(commit_oid)?;
            let tree_oid = commit.tree_id();

            // Check if file exists in this layer's tree
            // get_tree_entry() returns Err if file not found
            if repo.get_tree_entry(tree_oid, file_path).is_ok() {
                containing_layers.push(*layer);
            }
        }
        // If resolve_ref fails, skip this layer (may not be initialized)
    }

    Ok(containing_layers)
}
```

### Integration Points

```yaml
NO INTEGRATION POINTS NEEDED FOR THIS SUBTASK
- This is a pure helper function addition
- No changes to existing code required
- No database schema changes
- No configuration changes
- Future integrations (P1.M1.T2) will use this function

FUTURE USAGE (not part of this task):
- P1.M1.T2.S1: Add collision detection loop in merge_layers()
- P1.M1.T1.S2: has_different_content_across_layers() will call this
- commands/list.rs: Show layer provenance for files
- commands/status.rs: Display file layer information
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after implementing the function - fix before proceeding
cargo check --all-targets
# Expected: Compiles successfully with no errors or warnings

# Format check
cargo fmt --check
# Expected: No formatting differences

# Lint check
cargo clippy --all-targets -- -D warnings
# Expected: No clippy warnings

# Type check
cargo check --all-features
# Expected: No type errors
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run the new tests for find_layers_containing_file
cargo test find_layers_containing_file --verbose
# Expected: All new tests pass

# Run all tests in merge/layer module
cargo test --lib merge::layer --verbose
# Expected: All existing tests still pass + new tests pass

# Run all library tests
cargo test --lib
# Expected: No test failures

# Run with output
cargo test --lib -- --nocapture
# Expected: Clean test run with helpful output

# Coverage check (if coverage tools available)
cargo tarpaulin --out Html --output-dir coverage
# Expected: New function shows 100% coverage
```

### Level 3: Integration Testing (System Validation)

```bash
# Ensure no regressions in merge functionality
cargo test --lib merge --verbose
# Expected: All merge tests pass

# Test with real repository
cargo test --test cli_list --verbose  # Future: will use this helper
# Expected: Existing list tests still pass

# Test merge pipeline integration
cargo test --test conflict_workflow --verbose
# Expected: Conflict tests still pass (no regression)

# Full test suite
cargo test --all
# Expected: All tests pass across all modules
```

### Level 4: Manual Verification (Developer Validation)

```bash
# Manual smoke test using jin CLI (after full integration)
cd /tmp/test_jin_helper
mkdir -p test_repo && cd test_repo
jin init
echo '{"test": true}' > config.json
jin add config.json --global
jin commit -m "Add to global"
jin mode create testmode
jin mode use testmode
echo '{"test": false}' > config.json
jin add config.json --mode
jin commit -m "Add to mode"
rm config.json
jin apply --dry-run

# Expected: Should show that config.json is in both GlobalBase and ModeBase layers
# (This verification requires P1.M1.T2 integration to be complete)

# Verify function is callable from other modules
# Create a simple integration test:
cd /home/dustin/projects/jin
cat > test_find_layers_manual.rs << 'EOF'
use jin::merge::{find_layers_containing_file, LayerMergeConfig, get_applicable_layers};
use jin::git::JinRepo;
use std::path::Path;

fn main() {
    let repo = JinRepo::open_or_create().unwrap();
    let layers = get_applicable_layers(Some("test"), None, None);
    let config = LayerMergeConfig {
        layers,
        mode: Some("test".to_string()),
        scope: None,
        project: None,
    };

    let result = find_layers_containing_file(
        Path::new("test.txt"),
        &config.layers,
        &config,
        &repo
    );

    println!("Found layers: {:?}", result);
}
EOF
cargo run --example test_find_layers_manual
# Expected: Compiles and runs without error
```

---

## Final Validation Checklist

### Technical Validation

- [ ] Function compiles without errors or warnings
- [ ] Function follows exact signature from specification
- [ ] Function is exported via `src/merge/mod.rs`
- [ ] All unit tests pass: `cargo test find_layers_containing_file`
- [ ] No regression in existing tests: `cargo test --lib`
- [ ] No linting errors: `cargo clippy --all-targets`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Type checking passes: `cargo check --all-features`

### Feature Validation

- [ ] Returns empty Vec when file not in any layer
- [ ] Returns layers in same order as input (precedence maintained)
- [ ] Handles non-existent layer refs gracefully (skips, doesn't error)
- [ ] Handles empty layer list (returns empty vec)
- [ ] Propagates Git errors appropriately
- [ ] Works with files in nested directories
- [ ] Works with all 9 layer types
- [ ] Test coverage includes all edge cases

### Code Quality Validation

- [ ] Follows existing codebase patterns (matches `collect_all_file_paths()`)
- [ ] Function placement matches desired codebase tree
- [ ] Uses existing types (no new structs or enums)
- [ ] Doc comment follows rustdoc conventions
- [ ] Doc comment includes example usage
- [ ] No code duplication with existing functions
- [ ] Dependencies are properly imported

### Documentation & Deployment

- [ ] Function has rustdoc doc comment
- [ ] Doc comment explains parameters and return value
- [ ] Doc comment includes usage example
- [ ] Code is self-documenting with clear variable names
- [ ] No TODO or FIXME comments left in implementation
- [ ] Research documentation preserved in plan_bugfix/P1M1T1S1/research/

---

## Anti-Patterns to Avoid

- ❌ Don't create a new trait for this - use existing TreeOps pattern
- ❌ Don't return `Vec<&Layer>` - return `Vec<Layer>` (Layer is Copy)
- ❌ Don't use `layer.clone()` - use `*layer` to copy Layer value
- ❌ Don't skip the `ref_exists()` check - prevents errors on uninitialized layers
- ❌ Don't panic on missing files - return empty Vec (not an error condition)
- ❌ Don't create test mocks - use real repos with tempfile (follows existing patterns)
- ❌ Don't put tests in separate file - use inline `#[cfg(test)]` module
- ❌ Don't forget to export the function in mod.rs - follow existing export pattern
- ❌ Don't use `unwrap()` in tests - use proper error handling with `?` operator
- ❌ Don't hardcode layer refs - always use `layer.ref_path()` with context

---

## Appendix: Related Research Documents

The following research documents were created during PRP development and are preserved for reference:

1. **plan_bugfix/P1M1T1S1/research/git_mocking_research.md**
   - Comprehensive guide to mocking git2 operations in Rust
   - Test fixture patterns using tempfile
   - TreeBuilder usage for creating test trees

2. **plan_bugfix/P1M1T1S1/research/codebase_tree.md**
   - Current codebase structure for merge and git modules
   - Desired state showing where new helper fits
   - Integration points with commands

3. **plan_bugfix/P1M1T1S1/research/helper_function_patterns.md**
   - Analysis of existing helper function patterns
   - Signature recommendations based on codebase conventions
   - Implementation outline following established patterns

---

**PRP Version:** 1.0
**Created:** 2026-01-10
**Confidence Score:** 9/10 (High confidence in one-pass implementation success)

This PRP provides complete context for implementing `find_layers_containing_file()` following established codebase patterns. All necessary file locations, code patterns, error handling approaches, and test strategies are documented with specific examples.
