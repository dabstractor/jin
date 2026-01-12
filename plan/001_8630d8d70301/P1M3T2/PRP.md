# PRP: Implement Workspace Validation Logic

---

## Goal

**Feature Goal**: Create a validation function that detects detached workspace states by checking if the workspace is properly attached to valid layer commits, preventing users from performing destructive operations on inconsistent workspace states.

**Deliverable**:
1. `validate_workspace_attached()` function in `src/staging/workspace.rs` (or new module if needed)
2. Three detection functions for the detachment conditions
3. Unit tests for all validation scenarios
4. Integration with existing error types

**Success Definition**:
- Validation function detects all three detachment conditions (file mismatch, missing commits, invalid context)
- Returns `Ok(())` when workspace is properly attached
- Returns `Err(JinError::DetachedWorkspace)` with actionable recovery hints when detached
- Unit tests cover all detachment scenarios and edge cases
- No existing functionality is broken

## Why

**Business Value and User Impact**:
- **PRD Compliance**: Implements Non-negotiable Invariant #4: "Jin will abort any operation that would create a detached state" (Critical Gap #3 identified in implementation analysis)
- **Data Integrity**: Prevents destructive operations on workspaces that don't match any valid layer configuration
- **User Safety**: Catches workspace inconsistencies before they cause data loss or undefined behavior
- **Clear Recovery**: Provides specific, actionable error messages for each detachment condition

**Integration with Existing Features**:
- Builds on P1.M3.T1 (DetachedWorkspace error type definition)
- Enables P1.M3.T3 (validation integration into destructive operations: reset --hard, apply --force, checkout)
- Supports P1.M3.T4 (repair --check command for detached state detection)
- Required for P1.M3.T5 (integration tests)

**Problems This Solves**:
- Currently no detection exists for when workspace files are modified outside of Jin operations
- No validation that WorkspaceMetadata references existing commits
- Missing checks for when active context points to deleted modes/scopes
- Users can accidentally work in undefined workspace states

## What

**User-Visible Behavior**: Before destructive operations (reset --hard, apply --force, checkout), Jin validates workspace attachment. If detached, users see a clear error message explaining the issue and providing recovery guidance.

**Technical Requirements**:
1. Implement `validate_workspace_attached()` function that checks three detachment conditions
2. Each condition returns specific DetachedWorkspace error with appropriate recovery hint
3. Function must use existing JinRepo methods for Git operations
4. Follow existing validation patterns in the codebase

### Success Criteria

- [ ] `validate_workspace_attached()` function implemented in `src/staging/workspace.rs`
- [ ] Condition 1 detection: Workspace file hash mismatch with WorkspaceMetadata
- [ ] Condition 2 detection: WorkspaceMetadata references non-existent commits
- [ ] Condition 3 detection: Active context references deleted modes/scopes
- [ ] Unit tests for all three conditions with both attached and detached scenarios
- [ ] Error messages include specific recovery hints for each condition
- [ ] All tests pass: `cargo test --package jin --lib staging::workspace`
- [ ] No existing tests broken: `cargo test --package jin`

---

## All Needed Context

### Context Completeness Check

**Validation**: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"

**Yes** - This PRP provides:
- Exact file structure and patterns to follow
- Complete function signatures with types
- Specific detection logic for each detachment condition
- Integration points with existing code
- Test patterns and fixtures used in this codebase
- External research references for Git workspace validation

### Documentation & References

```yaml
MUST READ - Critical Implementation Context:

- file: src/core/error.rs (lines 38-54)
  why: DetachedWorkspace error variant definition - must use this exact structure
  pattern: Four-field error with workspace_commit, expected_layer_ref, details, recovery_hint
  critical: Error message format is multi-line with \n\ continuations

- file: src/staging/metadata.rs (lines 1-100)
  why: WorkspaceMetadata structure - stores file hashes and applied layers for validation
  pattern: JSON serialization with load()/save() methods, HashMap for file hashes
  gotcha: File paths are PathBuf, hashes are String (Git blob OIDs)
  critical: .jin/workspace/last_applied.json is the storage location

- file: src/core/config.rs (lines 78-156)
  why: ProjectContext structure - defines active mode/scope/project for validation
  pattern: YAML serialization with Option<String> fields, require_mode()/require_scope() methods
  gotcha: Use require_mode() not .mode.as_ref() - returns Result<&str> with proper error

- file: src/core/layer.rs (lines 49-86)
  why: Layer::ref_path() method - generates Git ref paths for active context
  pattern: match statement with mode/scope/project parameters
  critical: Returns format like "refs/jin/layers/mode/{mode}/scope/{scope}"

- file: src/git/refs.rs
  why: RefOps trait - Git reference operations for checking if refs exist
  pattern: find_ref(), reference_exists() methods
  gotcha: Use reference_exists() for Condition 3, not find_ref() (handles missing refs gracefully)

- file: src/commands/repair.rs (lines 130-264)
  why: Layer ref validation pattern - existing implementation for ref validation with reflog recovery
  pattern: Iterates refs, checks existence, provides recovery hints
  critical: Shows how to construct user-friendly error messages for missing refs

- file: plan/docs/P1M3T1_Detached_Workspace_Conditions.md (lines 41-287)
  why: Complete specification of three detachment conditions with detection logic
  pattern: Pseudo-code for each detection function
  critical: Provides exact error message templates and recovery hints

- file: plan/docs/P1M3T1_PRP_Detached_Workspace_State.md
  why: Completed PRP for DetachedWorkspace error type
  pattern: Error construction with all four fields populated
  critical: Shows how to construct DetachedWorkspace errors for each condition

- file: tests/common/fixtures.rs
  why: TestFixture and RemoteFixture patterns for isolated test environments
  pattern: Creates temp directories, maintains isolated Jin directories, automatic cleanup
  gotcha: Use JIN_DIR environment variable for isolation

- file: tests/error_scenarios.rs
  why: Integration test patterns for error conditions
  pattern: Uses assert_cmd with predicate matching for error messages
  critical: Shows how to test DetachedWorkspace errors in CLI context

- url: https://docs.rs/git2/latest/git2/struct.Repository.html
  why: git2-rs Repository documentation - shows find_commit(), reference_exists() methods
  section: Methods for commit and reference lookup
  critical: Use repo.find_commit() for Condition 2, repo.reference_exists() for Condition 3

- url: /home/dustin/projects/jin/plan/P1M3T2/research/git_workspace_validation.md
  why: External research on Git workspace validation patterns
  section: Exit Code Pattern and Library Pattern sections
  critical: Shows common pitfalls and best practices for workspace state validation
```

### Current Codebase Tree

```bash
src/
├── core/
│   ├── mod.rs              # Module exports
│   ├── error.rs            # DetachedWorkspace error type (lines 38-54)
│   ├── config.rs           # ProjectContext with active mode/scope/project
│   ├── layer.rs            # Layer enum with ref_path() method
│   └── jinmap.rs           # File-to-layer mapping metadata
├── staging/
│   ├── mod.rs              # Staging module exports
│   ├── metadata.rs         # WorkspaceMetadata (load/save, file hashes)
│   └── router.rs           # Layer routing validation patterns
├── git/
│   ├── mod.rs              # Git module exports
│   ├── refs.rs             # RefOps trait (find_ref, reference_exists)
│   └── commit.rs           # Commit operations
└── commands/
    ├── reset.rs            # Will use validation in P1.M3.T3
    ├── apply.rs            # Will use validation in P1.M3.T3
    └── repair.rs           # Reference for validation patterns

tests/
├── common/
│   ├── fixtures.rs         # TestFixture, RemoteFixture patterns
│   └── assertions.rs       # Custom assertion helpers
└── error_scenarios.rs      # Error condition test patterns

plan/
└── P1M3T2/
    ├── PRP.md              # This document
    └── research/
        └── git_workspace_validation.md  # External research findings
```

### Desired Codebase Tree (New Files and Modifications)

```bash
# Modified files:
src/staging/
├── mod.rs                  # ADD: pub use workspace::validate_workspace_attached;
└── workspace.rs            # ADD: validate_workspace_attached() + helper functions
                            # ADD: detect_file_mismatch(), detect_missing_commits(), detect_invalid_context()
                            # ADD: #[cfg(test)] mod tests with unit tests

tests/
└── workspace_validation.rs # CREATE: Integration tests for workspace validation
                             # Test all three detachment conditions
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2::Repository::find_commit() returns Err(git2::Error) for non-existent commits
// Use pattern: match repo.find_commit(oid) { Ok(_) => ..., Err(_) if error.class() == ErrorCode::NotFound => ... }

// CRITICAL: git2::Repository::reference_exists() is NOT a direct method
// Use pattern: repo.find_reference(name).is_ok() or repo.refname_to_id(name).is_ok()

// GOTCHA: WorkspaceMetadata.files HashMap keys are PathBuf, not String
// When iterating, use: for (path, hash) in metadata.files.iter() { ... }

// GOTCHA: File hashing must match Git blob OID format
// Use git2::Oid::from() or compute SHA-1 of file content

// PATTERN: When constructing DetachedWorkspace error, include all recovery context
// recovery_hint should be an actionable command: "Run 'jin reset --hard <ref>'"

// CRITICAL: Active context ref path depends on mode+scope combination
// Use Layer::ref_path() method to construct, don't hardcode paths

// GOTCHA: ProjectContext.mode is Option<String>, use require_mode() for validated access
// Pattern: context.require_mode()? instead of context.mode.as_ref().ok_or(...)

// CRITICAL: Validation order matters - check most specific conditions first
// Order: 1) File mismatch (most specific), 2) Missing commits, 3) Invalid context

// GOTCHA: WorkspaceMetadata.load() returns Err if file doesn't exist
// Handle gracefully: if WorkspaceMetadata::load().is_err() { return Ok(()); } // Fresh workspace

// PATTERN: Use JinRepo wrapper, not git2::Repository directly
// Access via JinRepo::inner() or use JinRepo methods for Git operations
```

---

## Implementation Blueprint

### Data Models and Structure

The validation function uses existing data structures:

```rust
// From src/staging/metadata.rs
pub struct WorkspaceMetadata {
    pub timestamp: String,
    pub applied_layers: Vec<String>,
    pub files: HashMap<PathBuf, String>,  // File path -> Git blob OID
}

// From src/core/config.rs
pub struct ProjectContext {
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}

// From src/core/error.rs (already defined)
JinError::DetachedWorkspace {
    workspace_commit: Option<String>,
    expected_layer_ref: String,
    details: String,
    recovery_hint: String,
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: ANALYZE existing workspace and metadata code
  - READ: src/staging/metadata.rs for WorkspaceMetadata structure
  - READ: src/staging/mod.rs for current module exports
  - IDENTIFY: Where workspace operations are currently defined
  - DECIDE: Add to workspace.rs or create new validation.rs file
  - OUTPUT: Understanding of current structure

Task 2: CREATE detection helper functions
  - CREATE: src/staging/workspace.rs (if not exists) or add to existing file
  - IMPLEMENT: fn detect_file_mismatch(metadata: &WorkspaceMetadata, repo: &JinRepo) -> Result<Option<Vec<PathBuf>>>
    - LOAD: current workspace files and compute hashes
    - COMPARE: current hashes vs metadata.files HashMap
    - RETURN: Ok(Some(modified_files)) or Ok(None) if no mismatch
  - IMPLEMENT: fn detect_missing_commits(metadata: &WorkspaceMetadata, repo: &JinRepo) -> Result<Option<Vec<String>>>
    - EXTRACT: commit OIDs from metadata (if stored in applied_layers)
    - CHECK: each commit exists using repo.find_commit()
    - RETURN: Ok(Some(missing_commits)) or Ok(None) if all exist
  - IMPLEMENT: fn detect_invalid_context(context: &ProjectContext, repo: &JinRepo) -> Result<Option<String>>
    - CONSTRUCT: ref paths for active mode/scope using Layer::ref_path()
    - CHECK: refs exist using repo.reference_exists()
    - RETURN: Ok(Some(invalid_ref_name)) or Ok(None) if all valid
  - FOLLOW pattern: Validation functions in src/staging/router.rs
  - NAMING: detect_* prefix for helper functions
  - PLACEMENT: src/staging/workspace.rs (private functions, pub(crate))

Task 3: IMPLEMENT main validation function
  - IMPLEMENT: pub fn validate_workspace_attached(context: &ProjectContext, repo: &JinRepo) -> Result<()>
    - CHECK: if WorkspaceMetadata exists, if not return Ok(()) (fresh workspace)
    - CALL: detect_file_mismatch() - if Some(files), return DetachedWorkspace error
    - CALL: detect_missing_commits() - if Some(commits), return DetachedWorkspace error
    - CALL: detect_invalid_context() - if Some(ref), return DetachedWorkspace error
    - RETURN: Ok(()) if all checks pass
  - ERROR: Construct DetachedWorkspace with all four fields for each condition
  - FOLLOW pattern: require_mode(), require_scope() in src/core/config.rs
  - NAMING: validate_workspace_attached (matches established naming convention)
  - PLACEMENT: src/staging/workspace.rs (public function)

Task 4: ADD module exports
  - MODIFY: src/staging/mod.rs
  - ADD: pub use workspace::validate_workspace_attached;
  - ENSURE: workspace module is properly included
  - FOLLOW pattern: Existing pub use statements in mod.rs
  - PLACEMENT: src/staging/mod.rs

Task 5: CREATE unit tests
  - CREATE: #[cfg(test)] mod tests in src/staging/workspace.rs
  - IMPLEMENT: test_validate_workspace_attached_success() - all checks pass
  - IMPLEMENT: test_detect_file_mismatch() - modified files detected
  - IMPLEMENT: test_detect_missing_commits() - deleted commit detected
  - IMPLEMENT: test_detect_invalid_context() - deleted mode/scope detected
  - IMPLEMENT: test_fresh_workspace_no_metadata() - Ok(()) when no metadata
  - FOLLOW pattern: Test structure in src/staging/metadata.rs
  - NAMING: test_* functions with descriptive scenario names
  - FIXTURES: Use mock JinRepo, WorkspaceMetadata, ProjectContext
  - PLACEMENT: src/staging/workspace.rs (test module at end of file)

Task 6: CREATE integration tests
  - CREATE: tests/workspace_validation.rs
  - IMPLEMENT: test_validation_passes_for_clean_workspace() - happy path
  - IMPLEMENT: test_validation_detects_modified_files() - Condition 1
  - IMPLEMENT: test_validation_detects_missing_commits() - Condition 2
  - IMPLEMENT: test_validation_detects_deleted_mode() - Condition 3
  - IMPLEMENT: test_validation_allows_fresh_workspace() - no metadata case
  - FOLLOW pattern: tests/error_scenarios.rs for CLI integration tests
  - NAMING: test_*_detached_* for error cases, test_*_success for happy path
  - FIXTURES: Use TestFixture from tests/common/fixtures.rs
  - PLACEMENT: tests/workspace_validation.rs (new file)
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Helper detection function - returns Option to indicate condition found
// Location: src/staging/workspace.rs

fn detect_file_mismatch(metadata: &WorkspaceMetadata, repo: &JinRepo) -> Result<Option<Vec<PathBuf>>> {
    let mut modified_files = Vec::new();

    // Iterate through tracked files in metadata
    for (path, stored_hash) in metadata.files.iter() {
        // Check if file exists in workspace
        if !path.exists() {
            modified_files.push(path.clone());
            continue;
        }

        // Compute current hash using Git blob hash
        let content = std::fs::read(path)?;
        let oid = repo.blob(content)?;
        let current_hash = oid.to_string();

        // Compare with stored hash
        if current_hash != *stored_hash {
            modified_files.push(path.clone());
        }
    }

    Ok(if modified_files.is_empty() { None } else { Some(modified_files) })
}

// PATTERN: Missing commit detection
fn detect_missing_commits(metadata: &WorkspaceMetadata, repo: &JinRepo) -> Result<Option<Vec<String>>> {
    // Note: WorkspaceMetadata doesn't directly store commit OIDs
    // This may need metadata extension or different approach
    // For now, check if applied_layers refs exist
    let mut missing_refs = Vec::new();

    for layer_name in &metadata.applied_layers {
        let ref_path = format!("refs/jin/layers/{}", layer_name);
        match repo.find_reference(&ref_path) {
            Ok(_) => {},
            Err(_) if error.class() == git2::ErrorClass::Reference => {
                missing_refs.push(ref_path);
            }
            Err(e) => return Err(JinError::Git(e)),
        }
    }

    Ok(if missing_refs.is_empty() { None } else { Some(missing_refs) })
}

// PATTERN: Invalid context detection
fn detect_invalid_context(context: &ProjectContext, repo: &JinRepo) -> Result<Option<String>> {
    // Check active mode exists
    if let Some(mode) = &context.mode {
        let mode_ref = format!("refs/jin/layers/mode/{}", mode);
        if repo.find_reference(&mode_ref).is_err() {
            return Ok(Some(format!("mode:{}", mode)));
        }
    }

    // Check active scope exists (scope ref path depends on mode)
    if let Some(scope) = &context.scope {
        let scope_ref = if let Some(mode) = &context.mode {
            format!("refs/jin/layers/mode/{}/scope/{}", mode, scope)
        } else {
            format!("refs/jin/layers/scope/{}", scope)
        };

        if repo.find_reference(&scope_ref).is_err() {
            return Ok(Some(format!("scope:{}", scope)));
        }
    }

    Ok(None)
}

// PATTERN: Main validation function - orchestrates all checks
pub fn validate_workspace_attached(context: &ProjectContext, repo: &JinRepo) -> Result<()> {
    // Fresh workspace - no metadata means no attachment to validate
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => return Ok(()),
        Err(e) => return Err(e),
    };

    // Condition 1: File mismatch
    if let Some(modified_files) = detect_file_mismatch(&metadata, repo)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: repo.head().and_then(|h| h.target().map(|t| t.to_string())).ok(),
            expected_layer_ref: format!("active context ({})", describe_context(context)),
            details: format!(
                "Workspace files have been modified outside of Jin operations. Modified files:\n  {}",
                modified_files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n  ")
            ),
            recovery_hint: format!("Run 'jin apply' to restore from active context"),
        });
    }

    // Condition 2: Missing commits/refs
    if let Some(missing_refs) = detect_missing_commits(&metadata, repo)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: None,
            expected_layer_ref: "<unknown>".to_string(),
            details: format!(
                "Workspace metadata references layers that no longer exist. Missing refs:\n  {}",
                missing_refs.join("\n  ")
            ),
            recovery_hint: "Run 'jin apply' to rebuild from current active context".to_string(),
        });
    }

    // Condition 3: Invalid context
    if let Some(invalid_ref) = detect_invalid_context(context, repo)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: repo.head().and_then(|h| h.target().map(|t| t.to_string())).ok(),
            expected_layer_ref: invalid_ref.clone(),
            details: format!("Active context references a mode or scope that no longer exists: {}", invalid_ref),
            recovery_hint: "Run 'jin mode activate <valid-mode>' or 'jin scope activate <valid-scope>'".to_string(),
        });
    }

    Ok(())
}

// HELPER: Describe active context for error messages
fn describe_context(context: &ProjectContext) -> String {
    let parts: Vec<&str> = vec![
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    ].into_iter().flatten().collect();

    if parts.is_empty() {
        "no active context".to_string()
    } else {
        parts.join("/")
    }
}

// GOTCHA: Use JinRepo methods, not git2::Repository directly
// JinRepo wraps git2::Repository and provides project-specific operations

// CRITICAL: Error recovery hints must be actionable commands
// Include specific refs/modes/scopes where relevant
```

### Integration Points

```yaml
STAGING_MODULE:
  - file: src/staging/mod.rs
  - modify: Add pub use workspace::validate_workspace_attached;
  - ensure: workspace module is properly declared with mod workspace;

WORKSPACE_MODULE:
  - file: src/staging/workspace.rs (CREATE or MODIFY)
  - add: validate_workspace_attached() (public)
  - add: detect_file_mismatch() (private)
  - add: detect_missing_commits() (private)
  - add: detect_invalid_context() (private)
  - add: describe_context() (private helper)

TEST_MODULE:
  - file: tests/workspace_validation.rs (CREATE)
  - add: Integration tests using TestFixture
  - follow: tests/error_scenarios.rs patterns

FUTURE_INTEGRATION (P1.M3.T3):
  - file: src/commands/reset.rs
  - call: validate_workspace_attached() before destructive operations
  - file: src/commands/apply.rs
  - call: validate_workspace_attached() before --force operations

ERROR_HANDLING:
  - use: JinError::DetachedWorkspace from src/core/error.rs
  - construct: All four fields populated for each condition
  - ensure: recovery_hint is actionable command
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each function is added - fix before proceeding
cargo build --package jin                        # Compilation check
cargo clippy --package jin -- -D warnings        # Lint check

# Focus on staging module
cargo build --package jin --lib staging          # Check staging module
cargo clippy --package jin --lib staging -- -D warnings

# Expected: Zero errors, zero warnings. If errors exist:
# 1. Check function signatures match types
# 2. Verify JinRepo method calls exist
# 3. Ensure all imports are present
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test workspace validation module specifically
cargo test --package jin --lib staging::workspace

# Test with output
cargo test --package jin --lib staging::workspace -- --nocapture

# Test individual detection functions
cargo test --package jin --lib staging::workspace::tests::test_detect_file_mismatch
cargo test --package jin --lib staging::workspace::tests::test_detect_missing_commits
cargo test --package jin --lib staging::workspace::tests::test_detect_invalid_context

# Expected: All tests pass. If failing:
# 1. Check mock setup (JinRepo, WorkspaceMetadata, ProjectContext)
# 2. Verify error construction matches JinError::DetachedWorkspace fields
# 3. Ensure file I/O operations in tests use temp directories
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all integration tests
cargo test --test workspace_validation

# Run specific integration test scenarios
cargo test --test workspace_validation test_validation_passes_for_clean_workspace
cargo test --test workspace_validation test_validation_detects_modified_files
cargo test --test workspace_validation test_validation_detects_missing_commits
cargo test --test workspace_validation test_validation_detects_deleted_mode

# Run all tests to ensure no regressions
cargo test --package jin

# Expected: All tests pass, no existing tests broken
# If existing tests fail, check for:
# 1. Module export conflicts in src/staging/mod.rs
# 2. Function signature changes affecting existing code
# 3. Import changes in lib.rs
```

### Level 4: Manual Validation

```bash
# Setup test environment
cd /tmp
mkdir test_workspace_validation && cd test_workspace_validation
git init
jin init

# Create a mode and activate it
echo "test" > test.txt
jin add test.txt
jin commit -m "test"
jin mode create test_mode
jin mode activate test_mode

# Test 1: Clean workspace should pass validation
jin status
# Expected: No detached workspace error

# Test 2: Modify workspace file externally
echo "modified" > .jin/workspace/test.txt
# (In future P1.M3.T3, jin reset --hard would catch this)
# For now, we'll test via direct function call or repair --check

# Test 3: Delete active mode ref
cd .jin
git update-ref -d refs/jin/layers/mode/test_mode
cd ..
# Validation should detect this

# Cleanup
cd ..
rm -rf test_workspace_validation

# Expected: Each scenario is handled correctly
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All detection functions compile without errors
- [ ] `validate_workspace_attached()` function compiles and is exported
- [ ] All unit tests pass: `cargo test --package jin --lib staging::workspace`
- [ ] All integration tests pass: `cargo test --test workspace_validation`
- [ ] No existing tests broken: `cargo test --package jin`
- [ ] No clippy warnings: `cargo clippy --package jin`
- [ ] Code formatted: `cargo fmt --check`

### Feature Validation

- [ ] Condition 1 (file mismatch) detected with specific file list
- [ ] Condition 2 (missing commits) detected with specific missing refs
- [ ] Condition 3 (invalid context) detected with specific mode/scope
- [ ] Fresh workspace (no metadata) returns Ok(())
- [ ] Clean workspace returns Ok(())
- [ ] All DetachedWorkspace errors include all four required fields
- [ ] All recovery hints are actionable commands
- [ ] Error messages are user-friendly and specific

### Code Quality Validation

- [ ] Follows existing validation patterns (compare to router.rs validation)
- [ ] Function naming matches codebase conventions (validate_*, detect_*)
- [ ] Error handling uses Result<T, JinError> consistently
- [ ] Helper functions are private, main function is public
- [ ] Module exports updated in src/staging/mod.rs
- [ ] Test fixtures use TestFixture pattern from tests/common/

### Documentation & Future Integration

- [ ] Function-level documentation comments present
- [ ] Complex logic has inline comments
- [ ] Test function names describe scenarios clearly
- [ ] Ready for integration in P1.M3.T3 (destructive operations)
- [ ] Recovery hints reference existing Jin commands

---

## Anti-Patterns to Avoid

- **Don't** use git2::Repository directly - use JinRepo wrapper for consistency
- **Don't** hardcode layer ref paths - use Layer::ref_path() method
- **Don't** skip fresh workspace handling - return Ok(()) when no metadata exists
- **Don't** use generic error messages - include specific files/refs that are problematic
- **Don't** forget to validate all three conditions - each is a distinct detachment scenario
- **Don't** use String for PathBuf keys - WorkspaceMetadata.files uses PathBuf
- **Don't** assume WorkspaceMetadata exists - handle NotFound gracefully
- **Don't** skip recovery hints - every DetachedWorkspace error must include actionable guidance
- **Don't** check conditions in random order - file mismatch first, then commits, then context
- **Don't** create new modules unnecessarily - add to existing workspace.rs or appropriate module

---

## Confidence Score

**9/10** for one-pass implementation success

**Reasoning**:
- ✅ Well-defined scope with three specific conditions to detect
- ✅ Clear error type already defined (P1.M3.T1 complete)
- ✅ Existing patterns to follow (validation in router.rs, error handling in error.rs)
- ✅ Comprehensive test fixtures available
- ✅ External research completed for Git workspace patterns
- ✅ Module structure clear (staging/workspace.rs or new file)
- ⚠️ Minor uncertainty: Exact JinRepo methods available (but git2 patterns are known)

**Validation**: This PRP provides sufficient context for an implementer unfamiliar with the codebase to implement workspace validation logic successfully. All necessary file patterns, error structures, and test approaches are specified.
