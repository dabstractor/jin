# PRP: Query JinMap for Layer Contents (P2.M2.T1)

## Goal

**Feature Goal**: Enable export command to validate and export files that are committed to Jin layers without requiring them to be in the staging index.

**Deliverable**: Modified export validation that checks both staging index (for staged files) and JinMap (for committed files) via layer content lookup.

**Success Definition**:
- Files committed to Jin layers can be exported without re-staging
- Export validation checks JinMap for committed files
- Committed file content is correctly extracted from layer trees
- Unit tests validate the new export behavior

## User Persona

**Target User**: Developer using Jin for configuration management

**Use Case**: User has committed files to Jin layers (e.g., via `jin commit`) and later wants to export them to Git without needing to re-stage them first.

**User Journey**:
1. User stages files with `jin add config.json`
2. User commits to layer: `jin commit`
3. Files are removed from staging index after commit
4. Later, user wants to export to Git: `jin export config.json`
5. **Current behavior**: Fails with "not Jin-tracked" error
6. **New behavior**: Succeeds by validating file exists in committed layer contents

**Pain Points Addressed**:
- **Re-staging overhead**: Users currently must re-stage committed files before export
- **Confusion**: Files that are "in Jin" (committed) cannot be exported
- **Workflow interruption**: Natural commit-then-export workflow is broken

## Why

- **Workflow completeness**: The commit operation removes files from staging, so export should handle committed files
- **User expectation**: If a file is "in Jin" (committed to a layer), it should be exportable
- **Consistency**: Import works with committed layers, export should too
- **Integration with existing feature**: JinMap already tracks layer-to-file mappings; this leverages that existing infrastructure

## What

Modify `src/commands/export.rs` to extend validation logic:
1. Check staging index first (existing behavior)
2. If not in staging, load JinMap and check if file exists in any layer
3. If in JinMap, extract file content from the committed layer tree
4. Proceed with export using extracted content

### Success Criteria

- [ ] Export accepts files that are in JinMap but not staging
- [ ] Export correctly reads file content from committed layer trees
- [ ] Export still rejects files that are neither staged nor committed
- [ ] Unit tests cover the new validation logic
- [ ] Integration tests verify end-to-end export of committed files

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test**: A developer unfamiliar with this codebase would need:
- Exact file paths for all modified code
- Specific method signatures and how to call them
- Test framework patterns used in this project
- How JinMap, layers, and tree operations work together

This PRP provides all of the above.

### Documentation & References

```yaml
# MUST READ - Core Implementation Files

- file: src/commands/export.rs
  why: Current export implementation - modify validate_jin_tracked() function
  pattern: Lines 137-152 show current staging-only validation
  gotcha: Line 136 has TODO comment about this exact feature

- file: src/core/jinmap.rs
  why: JinMap struct with load/save/contains_file methods
  pattern: Lines 94-108 for load(), 220-225 for contains_file()
  gotcha: Returns Default if .jin/.jinmap doesn't exist (first-run pattern)

- file: src/git/tree.rs
  why: TreeOps trait for reading file contents from Git trees
  pattern: Lines 133-136 for read_file_from_tree(), 138-157 for list_tree_files()
  gotcha: Path is relative to tree root, use Path::new(file_path)

- file: src/git/refs.rs
  why: RefOps trait for resolving layer refs to commit OIDs
  pattern: Lines 123-133 for ref_exists(), resolve_ref()
  gotcha: resolve_ref() returns Result<Oid>, must handle error

- file: src/core/layer.rs
  why: Layer enum with ref_path() method for getting Git ref paths
  pattern: Lines 49-86 for ref_path() implementation
  gotcha: ref_path() requires mode/scope/project Option<&str> args

- file: src/core/config.rs
  why: ProjectContext::load() for getting active mode/scope/project
  pattern: Lines 109-127 for load() method
  gotcha: Returns Err(JinError::NotInitialized) if .jin/context missing

- file: src/staging/index.rs
  why: StagingIndex struct for checking staged files (existing behavior)
  pattern: Lines 79-82 for get() method
  gotcha: Returns Option<&StagedEntry>, None means not staged

# Test Patterns

- file: src/commands/export.rs (lines 212-371)
  why: Existing unit test patterns for export validation
  pattern: Uses TempDir for isolation, TEST_LOCK mutex for serialization
  gotcha: Tests change working directory - always restore original_dir

- file: tests/cli_basic.rs
  why: Integration test pattern using assert_cmd
  pattern: jin().args([...]).current_dir(...).assert() pattern
  gotcha: Set JIN_DIR env var for test isolation
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── export.rs          # MODIFY: validate_jin_tracked() function
│   └── mod.rs             # May need imports (JinMap, ProjectContext)
├── core/
│   ├── config.rs          # USE: ProjectContext::load()
│   ├── error.rs           # USE: JinError variants
│   ├── jinmap.rs          # USE: JinMap::load(), contains_file()
│   └── layer.rs           # USE: Layer enum, ref_path()
├── git/
│   ├── mod.rs             # USE: re-exports TreeOps, RefOps
│   ├── refs.rs            # USE: RefOps trait (resolve_ref, ref_exists)
│   └── tree.rs            # USE: TreeOps trait (read_file_from_tree)
└── staging/
    └── index.rs           # USE: StagingIndex (existing behavior)
```

### Desired Codebase Tree (changes only)

```bash
# No new files - modifications only

src/commands/export.rs
├── MODIFY: validate_jin_tracked() function signature
│   └── ADD: repo: &JinRepo parameter for layer lookups
├── MODIFY: validate_jin_tracked() function body
│   └── ADD: Check JinMap if staging check fails
├── MODIFY: export_file() function signature
│   └── ADD: repo: &JinRepo parameter
└── MODIFY: execute() function
    └── ADD: Load JinMap and pass to validation
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: JinMap.load() returns Default if file doesn't exist
let jinmap = JinMap::load()?;  // Returns Ok(JinMap::default()) if missing
// Do NOT treat missing file as error - this is first-run pattern

// CRITICAL: Layer.ref_path() requires Option<&str> arguments
let ref_path = Layer::ModeBase.ref_path(
    context.mode.as_deref(),  // Must convert Option<String> to Option<&str>
    context.scope.as_deref(),
    context.project.as_deref(),
);

// CRITICAL: resolve_ref() fails if ref doesn't exist
// Always check ref_exists() first
if repo.ref_exists(&ref_path) {
    let commit_oid = repo.resolve_ref(&ref_path)?;
    // ...
}

// CRITICAL: read_file_from_tree() path is relative to tree root
// File paths in JinMap are already relative, use as-is
repo.read_file_from_tree(tree_oid, Path::new(file_path))?;

// CRITICAL: ProjectContext::load() fails if .jin/context missing
// This means Jin is not initialized - return clear error
let context = ProjectContext::load()
    .map_err(|e| JinError::Other("Jin not initialized. Run 'jin init' first.".to_string()))?;

// GOTCHA: Export test isolation requires mutex lock
static TEST_LOCK: Mutex<()> = Mutex::new();
let _lock = TEST_LOCK.lock();  // Serialize tests that change directory
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - existing structures sufficient:

```rust
// Existing - src/core/jinmap.rs:37-52
pub struct JinMap {
    pub version: u32,
    pub mappings: HashMap<String, Vec<String>>,  // layer_ref -> file paths
    pub meta: JinMapMeta,
}

// Existing - src/core/layer.rs:12-31
pub enum Layer {
    GlobalBase, ModeBase, ModeScope, ModeScopeProject,
    ModeProject, ScopeBase, ProjectBase, UserLocal, WorkspaceActive,
}

// Existing - src/core/config.rs:90-107
pub struct ProjectContext {
    pub version: u32,
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD imports to src/commands/export.rs
  - ADD: use crate::core::{JinMap, ProjectContext};
  - ADD: use crate::git::RefOps;
  - LOCATION: Top of file, after existing imports
  - PATTERN: Follow existing import structure (lines 7-11)

Task 2: MODIFY validate_jin_tracked() function signature
  - LOCATION: src/commands/export.rs:137
  - ADD: repo: &JinRepo parameter (after staging: &StagingIndex)
  - NEW SIGNATURE: fn validate_jin_tracked(path: &Path, staging: &StagingIndex, repo: &JinRepo) -> Result<()>
  - RATIONALE: Need repo for resolving layer refs and reading tree contents

Task 3: MODIFY validate_jin_tracked() function body
  - LOCATION: src/commands/export.rs:137-152
  - PRESERVE: File existence check (line 139-141)
  - PRESERVE: Staging index check (lines 143-149)
  - ADD: After staging check fails, try JinMap validation:
    1. Load JinMap: let jinmap = JinMap::load()?;
    2. Check if file in any layer: if !jinmap.contains_file(path_str) { return Err(...) }
    3. Load ProjectContext: let context = ProjectContext::load()?;
    4. For each layer in JinMap.layer_refs():
       a. Check if file is in this layer: if jinmap.get_layer_files(ref).map(|v| v.contains(&path_str))
       b. Resolve ref to commit: let commit_oid = repo.resolve_ref(ref)?
       c. Get tree from commit: let commit = repo.find_commit(commit_oid)?; let tree = commit.tree()?;
       d. Read file from tree: let content = repo.read_file_from_tree(tree.id(), path)?
       e. Verify file exists on filesystem matches tree content (optional validation)
    5. Return Ok(()) if found in any layer
  - GOTCHA: Convert Path to String for JinMap lookup: path.to_str().ok_or_else(...)

Task 4: MODIFY export_file() function signature
  - LOCATION: src/commands/export.rs:110
  - ADD: repo: &JinRepo parameter
  - NEW SIGNATURE: fn export_file(path: &Path, staging: &mut StagingIndex, repo: &JinRepo) -> Result<()>

Task 5: MODIFY execute() function to pass repo
  - LOCATION: src/commands/export.rs:29-101
  - REUSE: Existing repo variable (line 36: let _repo = JinRepo::open_or_create()?;)
  - CHANGE: Remove underscore: let repo = JinRepo::open_or_create()?;
  - UPDATE: Pass repo to export_file() calls (line 49)
  - UPDATE: Pass repo to validate_jin_tracked() calls (inside export_file)

Task 6: CREATE unit test for JinMap validation
  - LOCATION: src/commands/export.rs tests section (after line 371)
  - IMPLEMENT: test_validate_jin_tracked_committed_file()
  - SETUP:
    1. Create TempDir with Jin repo
    2. Create a blob and tree with test file
    3. Create a commit
    4. Create JinMap with file mapping
    5. Call validate_jin_tracked() with file not in staging
  - ASSERT: Validation succeeds (file found in JinMap)
  - PATTERN: Follow existing test structure (test_validate_jin_tracked_success)

Task 7: CREATE unit test for export committed file
  - LOCATION: src/commands/export.rs tests section
  - IMPLEMENT: test_export_committed_file()
  - SETUP:
    1. Create TempDir with Jin and Git repos
    2. Create blob, tree, commit in Jin repo
    3. Create JinMap with file mapping
    4. Create physical file in workspace
    5. Call export_file() with repo
  - ASSERT: Export succeeds, file added to Git index
  - PATTERN: Follow test_execute_file_not_jin_tracked structure

Task 8: CREATE unit test for file not in JinMap
  - LOCATION: src/commands/export.rs tests section
  - IMPLEMENT: test_validate_jin_tracked_not_in_jinmap()
  - SETUP:
    1. Create TempDir with physical file
    2. Empty staging index
    3. Empty JinMap (or file not in mappings)
  - ASSERT: Returns error with "not Jin-tracked" message
  - PATTERN: Follow test_validate_jin_tracked_not_in_staging structure
```

### Implementation Patterns & Key Details

```rust
// Pattern: Validation with JinMap fallback (Task 3)
fn validate_jin_tracked(path: &Path, staging: &StagingIndex, repo: &JinRepo) -> Result<()> {
    // 1. File existence check (EXISTING - keep)
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // 2. Staging index check (EXISTING - keep)
    if staging.get(path).is_some() {
        return Ok(());  // File is staged, good to go
    }

    // 3. NEW: Check JinMap for committed files
    let path_str = path.to_str()
        .ok_or_else(|| JinError::Other("Invalid file path".to_string()))?;

    let jinmap = JinMap::load()?;  // Returns default if missing
    if !jinmap.contains_file(path_str) {
        return Err(JinError::Other(format!(
            "{} is not Jin-tracked. Use `jin status` to see Jin-tracked files.",
            path.display()
        )));
    }

    // 4. NEW: Verify file exists in committed layer tree
    let context = ProjectContext::load()
        .map_err(|_| JinError::Other("Jin not initialized. Run 'jin init' first.".to_string()))?;

    for layer_ref in jinmap.layer_refs() {
        if let Some(files) = jinmap.get_layer_files(layer_ref) {
            if files.contains(&path_str.to_string()) {
                // Found the file in this layer - verify it exists in tree
                if repo.ref_exists(layer_ref) {
                    let commit_oid = repo.resolve_ref(layer_ref)?;
                    let commit = repo.find_commit(commit_oid)?;
                    let tree_oid = commit.tree_id();

                    // Read file from tree to verify it exists
                    repo.read_file_from_tree(tree_oid, path)?;
                    return Ok(());  // File found in committed layer
                }
            }
        }
    }

    // Should not reach here if contains_file() returned true
    Err(JinError::Other(format!(
        "{} is in JinMap but not found in any layer tree. Run 'jin repair' to fix.",
        path.display()
    )))
}

// Pattern: Updated export_file with repo parameter (Task 4)
fn export_file(path: &Path, staging: &mut StagingIndex, repo: &JinRepo) -> Result<()> {
    // 1. Validate file is Jin-tracked (NOW checks both staging and JinMap)
    validate_jin_tracked(path, staging, repo)?;

    // 2. Remove from .gitignore managed block (EXISTING - keep)
    if let Err(e) = remove_from_managed_block(path) {
        eprintln!("Warning: Could not remove {} from .gitignore: {}", path.display(), e);
    }

    // 3. Remove from Jin staging index if present
    // NOTE: Only remove if actually in staging (committed files aren't)
    if staging.get(path).is_some() {
        staging.remove(path);
    }

    // 4. Add to Git index (EXISTING - keep)
    add_to_git(path)?;

    Ok(())
}

// Pattern: Test setup with JinMap (Task 6)
#[test]
fn test_validate_jin_tracked_committed_file() {
    let temp = TempDir::new().unwrap();

    // Create Jin repo
    let repo_path = temp.path().join(".jin");
    let repo = JinRepo::create_at(&repo_path).unwrap();

    // Create a test file in a layer
    use crate::git::ObjectOps;
    let blob = repo.create_blob(b"test content").unwrap();
    let tree_oid = repo.create_tree_from_paths(&[
        ("config.json".to_string(), blob),
    ]).unwrap();
    let commit_oid = repo.create_commit(
        Some("refs/jin/layers/global"),
        "Test commit",
        tree_oid,
        &[]
    ).unwrap();

    // Create JinMap with file mapping
    let mut jinmap = JinMap::default();
    jinmap.add_layer_mapping(
        "refs/jin/layers/global",
        vec!["config.json".to_string()],
    );

    // Save to temp directory
    std::env::set_current_dir(temp.path()).unwrap();
    std::fs::create_dir_all(".jin").unwrap();
    let jinmap_path = PathBuf::from(".jin/.jinmap");
    let content = serde_yaml::to_string(&jinmap).unwrap();
    std::fs::write(jinmap_path, content).unwrap();

    // Create physical file
    let file = temp.path().join("config.json");
    std::fs::write(&file, b"test content").unwrap();

    // Empty staging index (file not staged)
    let staging = StagingIndex::new();

    // Validation should succeed via JinMap
    let result = validate_jin_tracked(&file, &staging, &repo);
    assert!(result.is_ok());

    // Always restore directory
    let _ = std::env::set_current_dir(std::env::current_dir().unwrap().parent().unwrap().parent().unwrap());
}
```

### Integration Points

```yaml
JINMAP:
  - load: Uses existing JinMap::load() from src/core/jinmap.rs
  - query: Uses contains_file() and get_layer_files() methods
  - path: Stored at .jin/.jinmap (YAML format)

PROJECT_CONTEXT:
  - load: Uses existing ProjectContext::load() from src/core/config.rs
  - path: Stored at .jin/context (YAML format)
  - purpose: Get mode/scope/project for layer ref paths

LAYER_REFS:
  - pattern: refs/jin/layers/{type}/{name}
  - examples:
    - refs/jin/layers/global
    - refs/jin/layers/mode/claude
    - refs/jin/layers/project/myproject

GIT_OPS:
  - resolve: Uses RefOps::resolve_ref() to get commit OID
  - read: Uses TreeOps::read_file_from_tree() to extract content
  - verify: Checks file exists in tree (returns error if not)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --message-format=short 2>&1 | head -50

# Auto-format and fix linting issues
cargo fmt --check

# Run clippy for lint checks
cargo clippy -- -D warnings 2>&1 | head -50

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test export module specifically
cargo test export -- --nocapture

# Test specific validation function
cargo test test_validate_jin_tracked -- --nocapture

# Test new JinMap validation tests
cargo test test_validate_jin_tracked_committed_file -- --nocapture
cargo test test_validate_jin_tracked_not_in_jinmap -- --nocapture

# Full test suite for export command
cargo test --bin jin export -- --nocapture

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual integration test - setup
cd /tmp && mkdir test_jin_export && cd test_jin_export
git init
jin init

# Stage and commit a file
echo "test content" > config.json
jin add config.json
jin commit

# Verify file is committed (not staged)
jin status
# Expected: config.json should NOT be in staging

# Export the committed file
jin export config.json
# Expected: Success - file exported to Git

# Verify Git has the file
git status
# Expected: config.json should be in Git index

# Cleanup
cd -
rm -rf /tmp/test_jin_export
```

### Level 4: Edge Case Validation

```bash
# Test 1: Export file that is neither staged nor committed
echo "untracked" > untracked.txt
jin export untracked.txt 2>&1 | grep "not Jin-tracked"
# Expected: Error message about not Jin-tracked

# Test 2: Export file from specific mode layer
jin mode create testmode
echo "mode content" > mode_config.json
jin add mode_config.json --layer mode
jin commit
jin export mode_config.json
# Expected: Success

# Test 3: Export from project layer
# (If project is set in context)
jin export project_config.json
# Expected: Success

# Test 4: JinMap missing but staging has file
rm .jin/.jinmap
echo "staged" > staged.txt
jin add staged.txt
jin export staged.txt
# Expected: Success (staging still works)
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test export`
- [ ] No clippy warnings: `cargo clippy`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Code compiles without errors: `cargo build`

### Feature Validation

- [ ] Files in staging can be exported (existing behavior preserved)
- [ ] Files committed to layers can be exported (new feature works)
- [ ] Files neither staged nor committed are rejected with clear error
- [ ] Export correctly reads content from layer trees
- [ ] All layer types (global, mode, project, scope) work correctly
- [ ] Error messages are helpful and actionable

### Code Quality Validation

- [ ] Follows existing codebase patterns (test structure, error handling)
- [ ] No new files created (only modifications)
- [ ] Function signatures updated consistently
- [ ] TODO comment in export.rs:136 can be removed/updated
- [ ] Tests cover both happy path and error cases

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable names
- [ ] Error messages guide users to correct actions
- [ ] Integration test demonstrates feature usage

## Anti-Patterns to Avoid

- **Don't change JinMap**: JinMap is read-only for export, don't modify it
- **Don't bypass staging check**: Always check staging first (fast path)
- **Don't fail if JinMap missing**: JinMap::load() returns default, handle gracefully
- **Don't ignore ref resolution errors**: If ref exists but can't resolve, that's an error
- **Don't skip ProjectContext**: Required for getting layer ref paths with mode/scope/project
- **Don't create new patterns**: Follow existing test structure, error handling, imports
- **Don't modify Git integration**: Keep existing git add/git reset logic
- **Don't hardcode layer refs**: Use Layer::ref_path() with context parameters

---

## Confidence Score

**8/10** - One-pass implementation success likelihood

**Rationale**:
- Clear modification points identified (3 functions, 1 file)
- Existing infrastructure (JinMap, TreeOps, RefOps) is mature
- Test patterns well-established in codebase
- Possible edge cases: ProjectContext::load() failure mode, ref_exists() vs resolve_ref() timing
- Integration test will validate end-to-end behavior

**Risk Mitigation**:
- Step-by-step task breakdown with exact line numbers
- Code patterns provided for all new functionality
- Test cases cover happy path and error cases
- Gotchas documented with workarounds
