# PRP: P1.M1.T1.S1 - Implement find_layers_containing_file() Helper

---

## Goal

**Feature Goal**: Implement `find_layers_containing_file()` helper function for collision detection in the merge system. This function iterates through applicable layers in precedence order and checks if a file exists in each layer's Git tree.

**Deliverable**: A public function `find_layers_containing_file()` in `src/merge/layer.rs` that:
1. Takes a file path, layers slice, LayerMergeConfig, and JinRepo as input
2. Iterates through layers in precedence order checking if file exists in each layer
3. Returns `Vec<Layer>` containing all layers that have this file
4. Returns empty vec if file not found in any layer
5. Handles missing layer refs gracefully (skip rather than error)

**Success Definition**:
- `cargo check` passes with zero errors
- `cargo test merge::` shows all new tests passing
- Function correctly identifies layers containing a given file
- Function returns empty vec for non-existent files
- All existing tests continue to pass

---

## User Persona

**Target User**: Internal - the merge system (`merge_layers()` function)

**Use Case**: Before merging files, the merge system needs to detect collision scenarios where a file exists in multiple layers with potentially different content. This helper function identifies which layers contain a specific file.

**User Journey**:
1. `merge_layers()` collects all unique file paths across all layers
2. For each file, `merge_layers()` calls `find_layers_containing_file()`
3. If 2+ layers contain the file, `has_different_content_across_layers()` is called to check for conflicts
4. If conflicts exist, file is added to `conflict_files` list and merge is skipped

**Pain Points Addressed**:
- Without this helper, merge system cannot detect collision scenarios before attempting merge
- Users could silently overwrite higher-precedence layer content with lower-precedence content
- No way to know which layers contributed to the final merged result

---

## Why

- **Foundation for Collision Detection**: This is the first step in implementing pre-merge collision detection. Without knowing which layers contain a file, we cannot detect conflicts.
- **Prevents Silent Data Loss**: By detecting files in multiple layers before merge, we can identify potential conflicts and alert users.
- **Enables Conflict Detection**: This helper is used by `has_different_content_across_layers()` (P1.M1.T1.S2) to check for content differences across layers.
- **Critical for Merge Safety**: The merge system cannot guarantee data integrity without collision detection.
- **Follows Existing Patterns**: Uses established Git operations (`ref_exists()`, `resolve_ref()`, `get_tree_entry()`) from the codebase.

---

## What

### User-Visible Behavior

This is an internal helper function with no direct user-facing behavior. However, it enables the merge system to:

1. **Detect collision scenarios** before merging
2. **Report which layers contain a file** when conflicts occur
3. **Skip merging** for conflicted files (rather than silently overwriting)

### Technical Requirements

1. **Function Signature**:
```rust
pub fn find_layers_containing_file(
    file_path: &Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>>
```

2. **Input Parameters**:
   - `file_path`: Path to file to search for (relative to repo root)
   - `layers`: Slice of layers to search (in precedence order)
   - `config`: Merge configuration with mode/scope/project context
   - `repo`: Jin repository for Git operations

3. **Output**:
   - `Ok(Vec<Layer>)`: Layers containing the file, in input order
   - `Err(JinError)`: Git operation failure

4. **Logic**:
   - Iterate through each layer in input order
   - For each layer:
     - Get ref_path using `layer.ref_path(mode, scope, project)`
     - Check if ref exists using `repo.ref_exists()` - skip if not
     - Resolve ref to commit OID
     - Get tree OID from commit
     - Check if file exists in tree using `repo.get_tree_entry()`
     - If exists, add layer to results
   - Return results (may be empty)

5. **Error Handling**:
   - Missing layer refs: Skip gracefully (not an error)
   - Git operation failures: Return `Err(JinError::Git(...))`
   - File not found in any layer: Return `Ok(vec![])` (empty vec, not error)

### Success Criteria

- [ ] Function added to `src/merge/layer.rs` with correct signature
- [ ] `cargo check` passes with 0 errors
- [ ] `cargo test merge::tests::find_layers` shows all tests passing
- [ ] Function returns empty vec for non-existent files
- [ ] Function correctly handles missing layer refs
- [ ] Function preserves input order in results
- [ ] All existing merge tests continue to pass

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context for implementing `find_layers_containing_file()` helper function. All required types, patterns, and test fixtures are documented with specific file references._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Target File for Implementation
- file: src/merge/layer.rs
  why: This is where the function must be added
  pattern: Add after get_applicable_layers() function, before merge_layers()
  placement: Lines ~270-370 area (after get_applicable_layers, before tests)
  gotcha: Must be public function (not in tests module)

# Layer Type Definition
- file: src/core/layer.rs
  why: Layer enum with ref_path() method needed for implementation
  section: Lines 33-96 (Layer enum definition and ref_path method)
  pattern: |
    layer.ref_path(
        config.mode.as_deref(),
        config.scope.as_deref(),
        config.project.as_deref(),
    )
  critical: ref_path generates Git ref paths like "refs/jin/layers/global"

# LayerMergeConfig Type
- file: src/merge/layer.rs
  why: Type definition for config parameter
  section: Lines 42-52 (LayerMergeConfig struct)
  pattern: |
    pub struct LayerMergeConfig {
        pub layers: Vec<Layer>,
        pub mode: Option<String>,
        pub scope: Option<String>,
        pub project: Option<String>,
    }

# JinRepo and RefOps Trait
- file: src/git/repo.rs
  why: JinRepo wrapper and RefOps trait methods
  section: Lines 84-134 (RefOps implementation for JinRepo)
  pattern: |
    repo.ref_exists(&ref_path)  // Check if layer ref exists
    repo.resolve_ref(&ref_path)  // Get commit OID from ref
  gotcha: Always check ref_exists() before resolve_ref() to avoid errors

# TreeOps Trait and get_tree_entry
- file: src/git/tree.rs
  why: TreeOps trait with get_tree_entry() method for file existence check
  section: Lines 47-54 (get_tree_entry documentation)
  pattern: |
    repo.get_tree_entry(tree_oid, path)  // Returns Ok(oid) if exists, Err if not
  critical: get_tree_entry returns Err when file not found - this is expected behavior

# Existing Test Pattern for Helper
- file: src/merge/layer.rs
  why: Test helper create_layer_with_file() shows how to create test layers
  section: Lines 785-803 (create_layer_with_file test helper)
  pattern: |
    let blob_oid = repo.create_blob(content)?;
    let tree_oid = repo.create_tree_from_paths(&[(file_path.to_string(), blob_oid)])?;
    let sig = git2::Signature::now("test", "test@test.com")?;
    let tree = repo.inner().find_tree(tree_oid)?;
    let commit_oid = repo.inner().commit(None, &sig, &sig, "test commit", &tree, &[])?;
    repo.set_ref(ref_name, commit_oid, "test layer")?;

# Test Fixtures Pattern
- file: tests/common/fixtures.rs
  why: Fixture patterns for test isolation and setup
  section: Lines 26-63 (TestFixture struct)
  pattern: Use tempfile::TempDir for isolated test repositories
  gotcha: TempDir must be kept in scope or directory is deleted

# Existing Merge Test Pattern
- file: src/merge/layer.rs
  why: Shows create_layer_test_repo() pattern for merge tests
  section: Lines 777-783 (create_layer_test_repo helper)
  pattern: |
    fn create_layer_test_repo() -> (tempfile::TempDir, JinRepo) {
        let temp = tempfile::TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

# Related Function (calls this helper)
- file: src/merge/layer.rs
  why: merge_layers() function that uses find_layers_containing_file()
  section: Lines 109-150 (merge_layers function with collision detection)
  pattern: |
    let layers_with_file = find_layers_containing_file(path, &config.layers, config, repo)?;
    if layers_with_file.len() > 1 {
        let has_conflict = has_different_content_across_layers(...)?;
  note: This shows how the helper is used in practice
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── merge/
│   │   ├── mod.rs                # Module exports (includes find_layers_containing_file)
│   │   ├── layer.rs              # TARGET FILE - add function here (~line 337)
│   │   ├── value.rs              # MergeValue type
│   │   ├── deep.rs               # Deep merge implementation
│   │   └── text.rs               # Text merge implementation
│   ├── core/
│   │   └── layer.rs              # Layer enum with ref_path() method
│   └── git/
│       ├── repo.rs               # JinRepo wrapper, RefOps trait
│       ├── refs.rs               # RefOps trait (ref_exists, resolve_ref)
│       ├── objects.rs            # ObjectOps trait
│       └── tree.rs               # TreeOps trait (get_tree_entry)
└── tests/
    └── common/
        └── fixtures.rs           # Test fixture patterns
```

### Desired Codebase Tree After Implementation

```bash
jin/
├── src/
│   └── merge/
│       └── layer.rs              # MODIFIED: Add find_layers_containing_file() function
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Always check ref_exists() before resolve_ref()
// Layer refs may not exist yet (layer not initialized)
// Pattern: if !repo.ref_exists(&ref_path) { continue; }

// CRITICAL: get_tree_entry() returns Err when file not found
// This is expected behavior, not an error condition
// Pattern: if repo.get_tree_entry(tree_oid, file_path).is_ok() { contains = true }

// GOTCHA: Layer::ref_path() requires mode/scope/project context
// Must pass config values to ref_path() method
// Pattern: layer.ref_path(config.mode.as_deref(), config.scope.as_deref(), config.project.as_deref())

// GOTCHA: Some layers use /_ suffix in ref_path to avoid Git ref conflicts
// ModeBase: "refs/jin/layers/mode/{mode}/_"
// ModeScope: "refs/jin/layers/mode/{mode}/scope/{scope}/_"
// This is documented in src/core/layer.rs lines 56-61

// PATTERN: Return empty vec (Ok(vec![])) when file not in any layer
// This is NOT an error condition - file simply doesn't exist in layers
// Error only occurs for actual Git operation failures

// PATTERN: Preserve input order in results
// Layers should be returned in the same order as input (precedence order)
// Use Vec::push() to maintain insertion order

// TESTING: Use tempfile for test isolation
// Tests must create isolated Jin repositories to avoid polluting ~/.jin/
// Pattern: JinRepo::create_at(&temp.path().join(".jin"))?

// TESTING: Keep TempDir in scope throughout test
// When TempDir is dropped, directory is deleted immediately
// Pattern: let (_temp, repo) = create_layer_test_repo();

// TESTING: Use create_layer_with_file() helper for test setup
// This helper creates blobs, trees, commits, and refs in one call
// See src/merge/layer.rs lines 785-803 for implementation
```

---

## Implementation Blueprint

### Data Models and Structure

**Function Signature** (to be added to `src/merge/layer.rs`):
```rust
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
    file_path: &std::path::Path,
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

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD function to src/merge/layer.rs
  - FILE: src/merge/layer.rs
  - LOCATION: After get_applicable_layers() function (~line 300), before merge_layers()
  - IMPLEMENT: Add find_layers_containing_file() function with full doc comment
  - SIGNATURE: pub fn find_layers_containing_file(file_path: &Path, layers: &[Layer], config: &LayerMergeConfig, repo: &JinRepo) -> Result<Vec<Layer>>
  - LOGIC:
    1. Create empty Vec<Layer> for results
    2. Iterate through layers slice
    3. For each layer: get ref_path, check ref_exists, resolve commit, get tree, check file exists
    4. If file exists, push layer to results
    5. Return results (may be empty)
  - PATTERN: Follow exact same pattern as collect_all_file_paths() (lines 156-185)
  - DEPENDENCIES: None (uses existing types and traits)

Task 2: EXPORT function in src/merge/mod.rs
  - FILE: src/merge/mod.rs
  - LOCATION: In pub use statements (around line 34)
  - ADD: pub use layer::find_layers_containing_file;
  - PATTERN: Follow existing export pattern for layer module functions

Task 3: ADD unit tests to src/merge/layer.rs
  - FILE: src/merge/layer.rs (in #[cfg(test)] module)
  - LOCATION: After has_different_content_across_layers tests
  - TESTS TO ADD:
    1. test_find_layers_single_layer_containing_file
    2. test_find_layers_multiple_layers_containing_file
    3. test_find_layers_file_not_in_any_layer
    4. test_find_layers_nonexistent_file_path
    5. test_find_layers_empty_layer_list
    6. test_find_layers_nonexistent_layer_refs_skipped
    7. test_find_layers_precedence_order_maintained
    8. test_find_layers_nested_directory_files
    9. test_find_layers_mode_scope_with_context
  - PATTERN: Follow existing test helper patterns (create_layer_test_repo, create_layer_with_file)
  - USE: tempfile for test isolation, create_layer_with_file for layer setup

Task 4: VERIFY merge_layers integration
  - FILE: src/merge/layer.rs
  - VERIFY: merge_layers() function calls find_layers_containing_file() correctly
  - LOCATION: Lines 120-133 (collision detection loop in merge_layers)
  - ENSURE: Function is called before merge_file_across_layers()
```

### Implementation Patterns & Key Details

```rust
// ================== EXACT FUNCTION TO ADD ==================
// Location: src/merge/layer.rs, after get_applicable_layers() (~line 301)

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
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
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

// ================== KEY IMPLEMENTATION DETAILS ==================

// 1. Use *layer to copy Layer value into Vec (Layer is Copy)
//    containing_layers.push(*layer);

// 2. Use .as_deref() to convert Option<String> to Option<&str>
//    config.mode.as_deref()

// 3. Use if let Ok() to handle Result from resolve_ref gracefully
//    if let Ok(commit_oid) = commit_oid { ... }

// 4. Use .is_ok() to check get_tree_entry result
//    if repo.get_tree_entry(tree_oid, file_path).is_ok() { ... }

// 5. Return Ok(vec![]) implicitly when no layers found
//    The Vec is empty if no layers are pushed to it

// ================== TEST HELPER USAGE ==================
// Tests should use the existing create_layer_test_repo() helper:

fn create_layer_test_repo() -> (tempfile::TempDir, JinRepo) {
    let temp = tempfile::TempDir::new().unwrap();
    let repo_path = temp.path().join(".jin");
    let repo = JinRepo::create_at(&repo_path).unwrap();
    (temp, repo)
}

// ================== TEST SETUP EXAMPLE ==================
// Creating test layers with files:

fn create_layer_with_file(
    repo: &JinRepo,
    ref_name: &str,
    file_path: &str,
    content: &[u8],
) -> Result<()> {
    let blob_oid = repo.create_blob(content)?;
    let tree_oid = repo.create_tree_from_paths(&[(file_path.to_string(), blob_oid)])?;
    let sig = git2::Signature::now("test", "test@test.com")?;
    let tree = repo.inner().find_tree(tree_oid)?;
    let commit_oid = repo.inner().commit(None, &sig, &sig, "test commit", &tree, &[])?;
    repo.set_ref(ref_name, commit_oid, "test layer")?;
    Ok(())
}

// ================== EXAMPLE TEST ==================
#[test]
fn test_find_layers_single_layer_containing_file() {
    let (_temp, repo) = create_layer_test_repo();

    // Create a layer with a file
    create_layer_with_file(
        &repo,
        "refs/jin/layers/global",
        "config.json",
        br#"{"key":"global"}"#,
    ).unwrap();

    let layers = vec![Layer::GlobalBase];
    let config = LayerMergeConfig {
        layers,
        mode: None,
        scope: None,
        project: None,
    };

    let result = find_layers_containing_file(
        Path::new("config.json"),
        &config.layers,
        &config,
        &repo
    ).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Layer::GlobalBase);
}
```

### Integration Points

```yaml
MODIFICATIONS:
  - file: src/merge/layer.rs
    change: Add find_layers_containing_file() function
    location: After get_applicable_layers() (~line 301)
    scope: Single function addition to public API

  - file: src/merge/mod.rs
    change: Export find_layers_containing_file in pub use statements
    location: Line ~34 (in layer module exports)
    add: pub use layer::{find_layers_containing_file, ...};

NO CHANGES TO:
  - src/core/layer.rs (Layer type already exists)
  - src/git/*.rs (All traits already implemented)
  - src/merge/other files (function goes in layer.rs)

DOWNSTREAM DEPENDENCIES:
  - P1.M1.T1.S2 (has_different_content_across_layers) uses this helper
  - merge_layers() function calls this helper for collision detection
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after function addition - must pass before proceeding
cargo check                           # Type checking - MUST pass with 0 errors

# Expected: Zero errors. If errors exist:
# - Check function signature matches types
# - Verify all imports are present
# - Check for missing semicolons or braces

# Format check
cargo fmt -- --check                  # Format check

# Expected: No formatting issues
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build with compilation successful

# Run merge module tests specifically
cargo test merge:: --no-run           # Compile tests only

# Expected: All tests compile successfully
```

### Level 3: Unit Tests (Component Validation)

```bash
# Test the new find_layers_containing_file function
cargo test merge::tests::find_layers   # Run all find_layers tests

# Expected: All 9 tests pass:
# - test_find_layers_single_layer_containing_file
# - test_find_layers_multiple_layers_containing_file
# - test_find_layers_file_not_in_any_layer
# - test_find_layers_nonexistent_file_path
# - test_find_layers_empty_layer_list
# - test_find_layers_nonexistent_layer_refs_skipped
# - test_find_layers_precedence_order_maintained
# - test_find_layers_nested_directory_files
# - test_find_layers_mode_scope_with_context

# Run with output
cargo test merge::tests::find_layers -- --nocapture

# Expected: All tests pass with clear output

# Test entire merge module
cargo test merge::                     # All merge module tests

# Expected: All existing tests still pass + 9 new tests pass
```

### Level 4: Integration Testing

```bash
# Test that merge_layers can use the new helper
cargo test merge::tests::test_merge_layers_single_layer_no_conflict

# Expected: Test passes (uses find_layers_containing_file internally)

# Test collision detection in merge_layers
cargo test merge::tests::test_merge_layers_two_layers_different_content_conflict

# Expected: Test passes, conflict is detected correctly

# Full test suite for merge module
cargo test merge:: -- --test-threads=1  # Run sequentially for debugging

# Expected: All merge tests pass
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo build` succeeds
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test merge::tests::find_layers` shows all 9 tests passing
- [ ] `cargo test merge::` shows all tests passing (including existing)
- [ ] Function exported in src/merge/mod.rs

### Feature Validation

- [ ] Function returns `Vec<Layer>` with correct layers
- [ ] Returns empty vec for non-existent files
- [ ] Handles missing layer refs gracefully (skips, doesn't error)
- [ ] Preserves input order in results
- [ ] Works with all layer types (GlobalBase, ModeBase, etc.)
- [ ] Correctly uses layer.ref_path() with mode/scope/project context
- [ ] Uses ref_exists() check before resolve_ref()

### Code Quality Validation

- [ ] Follows existing code patterns (similar to collect_all_file_paths)
- [ ] Comprehensive doc comment with all sections
- [ ] Uses proper error handling (Result type)
- [ ] No unwrap() or expect() in production code
- [ ] Test helpers follow existing patterns
- [ ] Test names follow test_*pattern convention

### Documentation & Deployment

- [ ] Function has complete doc comment with Arguments, Returns, Examples sections
- [ ] Doc comment explains behavior with missing refs
- [ ] Examples show correct usage pattern
- [ ] No breaking changes to existing API

---

## Anti-Patterns to Avoid

- **Don't** return `Err(JinError::NotFound)` when file not found - return `Ok(vec![])` instead
- **Don't** skip the `ref_exists()` check - it prevents errors for uninitialized layers
- **Don't** use `unwrap()` on `resolve_ref()` result - use `if let Ok()` pattern
- **Don't** modify the input `layers` slice - iterate read-only
- **Don't** use `HashMap` or `HashSet` for results - `Vec` preserves order
- **Don't** forget to copy Layer with `*layer` when pushing to Vec
- **Don't** create a test without using `tempfile::TempDir` for isolation
- **Don't** let TempDir drop before test completes - keep it in scope
- **Don't** hardcode ref paths - use `layer.ref_path()` method
- **Don't** test with global ~/.jin repository - always use isolated temp repos

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Exact Specification**: Function signature, logic, and behavior are precisely specified
- **Pattern Following**: Implementation follows existing `collect_all_file_paths()` pattern
- **Type Safety**: All types already exist and are well-documented
- **Test Pattern**: Test helpers and fixtures already exist
- **Isolated Scope**: Single function addition with minimal dependencies
- **Clear Success Criteria**: 9 specific test cases validate all scenarios
- **No External Deps**: Uses only existing traits and types from codebase
- **Comprehensive Context**: All file references, patterns, and gotchas documented

**Implementation is straightforward**:
1. Iterate layers
2. Check ref exists
3. Resolve commit
4. Get tree
5. Check file exists
6. Collect results

The function is essentially a filtered iteration over layers with Git existence checks - a well-understood pattern in the codebase.

---

## Research Artifacts Location

Research documentation stored at: `plan/bugfix/P1M1T1S1/research/`

**Key Internal References**:
- `src/merge/layer.rs` - Main implementation file with existing patterns
- `src/core/layer.rs` - Layer enum with ref_path() method
- `src/git/repo.rs` - JinRepo wrapper and RefOps trait
- `src/git/tree.rs` - TreeOps trait with get_tree_entry()
- `tests/common/fixtures.rs` - Test fixture patterns

**Note**: Web search was unavailable during research (monthly limit reached). However, all necessary context is available from the codebase itself. The git2 crate patterns used in this implementation are already established throughout the codebase.
