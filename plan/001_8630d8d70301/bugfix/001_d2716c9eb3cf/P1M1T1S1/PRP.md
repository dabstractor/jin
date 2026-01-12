name: "P1.M1.T1.S1: Remove Pre-Merge Conflict Check for Structured Files"
description: |

---

## Goal

**Feature Goal**: Fix the merge engine to allow structured files (JSON/YAML/TOML/INI) to proceed directly to deep merge without pre-checking for content differences, while preserving conflict detection for text files.

**Deliverable**: Updated `merge_layers()` function in `src/merge/layer.rs` that conditionally applies conflict detection based on file format.

**Success Definition**:
- Structured files with different content across layers are deep-merged successfully (not flagged as conflicts)
- Text files with different content still trigger conflict detection as before
- All existing tests pass
- New integration test verifies structured file auto-merge behavior

## User Persona

**Target User**: Developers using Jin's layered configuration system who store configuration in structured formats (JSON, YAML, TOML, INI).

**Use Case**: A developer has configuration files spread across multiple layers (e.g., global defaults, mode-specific overrides, project settings). When these files have different values across layers, they expect automatic deep merge rather than manual conflict resolution.

**User Journey**:
1. Developer adds `config.json` to global layer with base settings
2. Developer adds same `config.json` to mode layer with overrides
3. Developer runs `jin apply`
4. **Expected**: Automatic merge combines settings from both layers
5. **Current (Bug)**: Conflict file created, manual resolution required

**Pain Points Addressed**:
- Eliminates unnecessary manual conflict resolution for structured files
- Aligns behavior with PRD specifications (§11.1, §11.2)
- Reduces cognitive overhead when managing layered configurations

## Why

- **PRD Compliance**: Fixes violation of PRD §11.1 "Structured Merge Rules" and §11.2 "Merge Priority" which specify that structured files should use deep merge with layer precedence
- **Existing Deep Merge is Correct**: The `deep_merge()` implementation in `src/merge/deep.rs` already correctly implements RFC 7396 semantics - the bug is only in the pre-merge check
- **User Experience**: Significantly reduces friction - users expect different JSON configs to merge automatically, not require manual intervention
- **Consistency**: Text files (line-based merges) still need conflict detection, but structured files (semantic merges) don't

## What

Modify the `merge_layers()` function to:
1. Detect file format before checking for conflicts
2. For text files: Keep existing `has_different_text_content()` check
3. For structured files: Skip pre-check entirely, proceed to `merge_file_across_layers()`
4. Deep merge will handle layer precedence automatically

### Success Criteria

- [ ] Structured files (JSON/YAML/TOML/INI) with different content merge successfully
- [ ] Text files with different content still trigger conflict detection
- [ ] Layer precedence works correctly in merged results
- [ ] All existing unit tests pass
- [ ] New integration test verifies the fix
- [ ] No regression in text file merge behavior

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully? ✓ Yes - this PRP provides exact line numbers, code patterns, test locations, and validation commands.

### Documentation & References

```yaml
# MUST READ - Core architecture understanding
- file: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/merge_engine_analysis.md
  why: Explains the bug root cause and fix strategy at high level
  critical: The "Solution Architecture" section shows exact code changes needed

# MUST READ - Target file for modification
- file: src/merge/layer.rs
  why: Contains the merge_layers() function that needs modification
  pattern: Lines 136-146 contain the buggy conflict check logic
  gotcha: The conflict check happens BEFORE merge_file_across_layers(), which causes structured files to be flagged incorrectly

# MUST READ - File format detection
- file: src/merge/layer.rs
  why: Contains detect_format() function (lines 433-442) and FileFormat enum (lines 17-28)
  pattern: Extension-based format detection: .json→Json, .yaml/.yml→Yaml, .toml→Toml, .ini/.cfg/.conf→Ini, else→Text
  gotcha: Format detection is case-insensitive and falls back to Text for unknown extensions

# MUST READ - Deep merge implementation (already correct, no changes needed)
- file: src/merge/deep.rs
  why: Contains deep_merge() function that correctly implements RFC 7396 semantics
  pattern: Recursive object merging, null deletion, keyed array merging with ["id", "name"] defaults
  critical: This is the CORRECT implementation - do NOT modify deep_merge()

# MUST READ - Existing test patterns
- file: src/merge/layer.rs
  why: Contains comprehensive unit tests for merge_layers() (lines 2253-2504)
  pattern: Test helpers create_layer_with_file() and create_layer_test_repo()
  gotcha: Tests use tempfile::TempDir for isolation and JinRepo::create_at() for test repos

# MUST READ - Integration test patterns
- file: tests/pull_merge.rs
  why: Shows integration test patterns for merge operations
  pattern: Uses TestFixture, jin_cmd(), and fs::write for end-to-end testing

# EXTERNAL - RFC 7396 JSON Merge Patch
- url: https://datatracker.ietf.org/doc/html/rfc7396
  why: Defines the null deletion semantics used by deep_merge()
  critical: Null in overlay = delete key from base (this is already implemented correctly in deep_merge())

# EXTERNAL - Rust Error Handling Pattern
- file: src/core/error.rs
  why: Defines JinError enum used throughout merge module
  pattern: Result<T> = std::result::Result<T, JinError>
  gotcha: MergeConflict is a valid JinError variant, not a panic/exception
```

### Current Codebase Structure

```bash
/home/dustin/projects/jin/
├── src/
│   ├── merge/
│   │   ├── layer.rs          # TARGET FILE - merge_layers() function (lines 109-200)
│   │   ├── deep.rs           # Deep merge implementation (already correct)
│   │   ├── text.rs           # 3-way text merge implementation
│   │   ├── jinmerge.rs       # Merge conflict file format
│   │   └── mod.rs            # Module exports
│   └── core/
│       ├── error.rs          # JinError enum definition
│       └── layer.rs          # Layer enum definition
└── tests/
    ├── pull_merge.rs         # Integration tests for merge operations
    └── common/
        ├── fixtures.rs       # TestFixture helpers
        └── assertions.rs     # Custom assertions

plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/
├── architecture/
│   └── merge_engine_analysis.md  # Architecture documentation
└── P1M1T1S1/
    └── PRP.md                 # This file
```

### Key Data Structures

```rust
// src/merge/layer.rs (lines 17-28)
pub enum FileFormat {
    Json,     // .json
    Yaml,     // .yaml, .yml
    Toml,     // .toml
    Ini,      // .ini, .cfg, .conf
    Text,     // Any other extension
}

// src/core/layer.rs
pub enum Layer {
    GlobalBase,           // Precedence 1
    ModeBase,             // Precedence 2
    ModeScope,            // Precedence 3
    ModeScopeProject,     // Precedence 4
    ModeProject,          // Precedence 5
    ScopeBase,            // Precedence 6
    ProjectBase,          // Precedence 7
    UserLocal,            // Precedence 8
    WorkspaceActive,      // Precedence 9
}

// src/merge/layer.rs (lines 30-39)
pub struct MergedFile {
    pub content: MergeValue,
    pub source_layers: Vec<Layer>,
    pub format: FileFormat,
}
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: has_different_content_across_layers() returns TRUE for structured files
// when content differs (line 623), even though those differences SHOULD be resolved
// via deep merge. This is the bug we're fixing.

// CRITICAL: The merge_layers() function does conflict checking in TWO places:
// 1. Lines 136-146: Pre-merge check (BUGGY for structured files)
// 2. Lines 178-182: Post-merge error handling (CORRECT - handles actual merge failures)

// CRITICAL: Text files MUST keep the pre-check because line-based 3-way merge
// produces conflict markers that need detection. Structured files don't have this issue.

// CRITICAL: File format detection is case-insensitive (line 435)
// ext.to_lowercase().as_str() ensures ".JSON" == ".json"

// CRITICAL: When a layer ref doesn't exist, repo.ref_exists() returns false
// This is checked at lines 233, 283, 539 before resolve_ref() to avoid panics

// CRITICAL: The optimization at lines 148-168 (same content across layers)
// should remain unchanged - it only applies when has_different_content_across_layers()
// returns false, which is correct behavior.

// CRITICAL: source_layers metadata completeness (lines 162-164)
// When using the same-content optimization, ALL layers must be added to source_layers
// for proper tracking, even though content is read from only the first layer.

// GOTCHA: parse_content() returns JinError::Parse with format and message fields
// Use pattern: parse_content(&content_str, format).map_err(|e| ...)?;
```

### Current Implementation (Buggy Code)

```rust
// src/merge/layer.rs (lines 136-146) - BUGGY CODE
if layers_with_file.len() > 1 {
    // File exists in multiple layers - check for content conflicts
    let has_conflict =
        has_different_content_across_layers(path, &layers_with_file, config, repo)?;

    if has_conflict {
        // Different content detected - add to conflicts and skip merge
        result.conflict_files.push(path.clone());
        continue; // Skip merge_file_across_layers() for this file
    }
    // ... optimization for same content ...
}
```

### Why This Code is Wrong

The bug is that `has_different_content_across_layers()` (line 604) calls `has_different_structured_content()` (line 623) which compares parsed `MergeValue` objects. When JSON files have different content across layers, it returns `true`, which causes the file to be added to `conflict_files` and `merge_file_across_layers()` is never called.

However, `merge_file_across_layers()` (line 264) would correctly deep-merge these different JSON values using `deep_merge()` (line 355), which implements RFC 7396 semantics. The pre-check incorrectly prevents this correct behavior.

## Implementation Blueprint

### Data Models

No new data models needed. Existing structures are sufficient:
- `FileFormat` enum (already exists)
- `Layer` enum (already exists)
- `MergedFile` struct (already exists)

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY merge_layers() function in src/merge/layer.rs (lines 136-146)
  - CURRENT: has_different_content_across_layers() is called for ALL file formats
  - NEW: Call detect_format() first, then conditional conflict check
  - For FileFormat::Text: Keep existing has_different_text_content() check
  - For FileFormat::{Json,Yaml,Toml,Ini}: Skip pre-check entirely
  - NAMING: Use existing function names, no new functions needed
  - PLACEMENT: src/merge/layer.rs, lines 136-146

Task 2: VERIFY existing optimization still works (lines 148-168)
  - The same-content optimization should work for both text and structured files
  - has_different_content_across_layers() already returns false for same content
  - No changes needed to optimization code

Task 3: ADD new integration test for structured file auto-merge
  - CREATE: tests/structured_merge_fix.rs OR add to tests/pull_merge.rs
  - IMPLEMENT: test_structured_files_auto_merge_without_conflict()
  - FOLLOW pattern: tests/pull_merge.rs (TestFixture usage, jin_cmd() invocation)
  - NAMING: test_structured_auto_merge_json, test_structured_auto_merge_yaml, etc.
  - COVERAGE: JSON, YAML, TOML, INI formats with different content across layers
  - ASSERTIONS: Verify merged content combines values from both layers correctly

Task 4: VERIFY text file conflict detection still works
  - ENSURE: Text files with different content still create conflicts
  - TEST: Add explicit test if not already covered
  - ASSERT: result.conflict_files contains the text file path

Task 5: RUN full test suite and fix any failures
  - EXECUTE: cargo test --all
  - VERIFY: All existing tests pass
  - DEBUG: Fix any regressions introduced by the change
```

### Implementation Patterns & Key Details

```rust
// ============================================================
// CURRENT (BUGGY) CODE - Lines 136-146
// ============================================================
if layers_with_file.len() > 1 {
    let has_conflict =
        has_different_content_across_layers(path, &layers_with_file, config, repo)?;

    if has_conflict {
        result.conflict_files.push(path.clone());
        continue;
    }
    // ... same-content optimization ...
}

// ============================================================
// NEW (CORRECT) CODE - Replace lines 136-146
// ============================================================
if layers_with_file.len() > 1 {
    // Detect file format to determine conflict check strategy
    let format = detect_format(path);

    // Only check for conflicts in text files (line-based 3-way merge)
    // Structured files will be deep-merged, so skip pre-check
    if format == FileFormat::Text {
        let has_conflict = has_different_text_content(path, &layers_with_file, config, repo)?;

        if has_conflict {
            result.conflict_files.push(path.clone());
            continue;
        }
    }
    // For structured files: no pre-check, attempt deep merge directly
}

// ============================================================
// CRITICAL IMPLEMENTATION NOTES
// ============================================================

// 1. Use detect_format(path) NOT has_different_content_across_layers()
//    detect_format() is at line 433 and returns FileFormat enum
//    has_different_content_across_layers() internally calls detect_format()
//    but we need the format BEFORE deciding whether to check for conflicts

// 2. Call has_different_text_content() directly for text files
//    This function is at line 627 and does raw string comparison
//    It's already called by has_different_content_across_layers() at line 619
//    Calling it directly skips the format detection branching

// 3. For structured files, DO NOT call any conflict check function
//    Let merge_file_across_layers() (line 264) handle deep merge
//    Deep merge will use deep_merge() (line 355) which correctly handles
//    layer precedence and RFC 7396 null deletion semantics

// 4. The same-content optimization (lines 148-168) remains unchanged
//    It activates when has_different_text_content() returns false
//    For structured files, deep merge will handle same-content efficiently

// ============================================================
// ERROR HANDLING PATTERN (from src/merge/layer.rs line 178)
// ============================================================
// When merge_file_across_layers() returns Err(JinError::MergeConflict),
// add to conflict_files. This catches ACTUAL merge failures (syntax errors),
// not pre-declared conflicts from different content.

match merge_file_across_layers(path, &config.layers, config, repo) {
    Ok(merged) => {
        result.merged_files.insert(path.clone(), merged);
    }
    Err(JinError::MergeConflict { .. }) => {
        result.conflict_files.push(path.clone());
    }
    Err(e) => {
        return Err(e);
    }
}
```

### Example of Correct Behavior After Fix

```rust
// Setup: Two layers with different JSON content
// Global layer: {"port": 8080, "debug": false}
// Mode layer:   {"port": 9090, "feature": true}

// Before fix (BUGGY):
// - has_different_structured_content() returns true
// - File added to conflict_files
// - merge_file_across_layers() never called
// - .jinmerge file created

// After fix (CORRECT):
// - File format detected as JSON
// - Pre-check skipped for structured files
// - merge_file_across_layers() called
// - Deep merge produces: {"port": 9090, "debug": false, "feature": true}
// - No conflict file created
```

### Integration Points

```yaml
NO_NEW_DEPENDENCIES: This change is isolated to merge_layers() function

NO_MIGRATION_NEEDED: Existing conflict files can be manually resolved

NO_CONFIG_CHANGES: Layer configuration and precedence unchanged

NO_API_CHANGES: Public function signatures unchanged

TEST_FRAMEWORK: Uses existing cargo test infrastructure
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after editing src/merge/layer.rs
cargo check --message-format=short
cargo clippy --all-targets --all-features -- -D warnings

# Expected: Zero errors, zero warnings
# If errors exist: READ output carefully and fix before proceeding
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run merge module unit tests
cargo test --lib merge::layer

# Run specific test functions related to merge_layers
cargo test --lib merge_layers

# Run all unit tests
cargo test --lib

# Expected: All tests pass
# If failing: Debug which test failed and why
```

### Level 3: Integration Testing (System Validation)

```bash
# Run the new integration test for structured file auto-merge
cargo test --test structured_merge_fix

# Run all integration tests
cargo test --test pull_merge
cargo test --test conflict_workflow

# Expected: All integration tests pass
# Critical: Verify structured files merge without conflicts
```

### Level 4: Manual Testing & Verification

```bash
# 1. Create a test repository
mkdir /tmp/jin_test && cd /tmp/jin_test
git init
jin init

# 2. Create a JSON config in global layer
echo '{"port": 8080, "debug": false}' > config.json
jin add config.json --global
jin commit -m "Add global config"

# 3. Modify and add to mode layer
jin mode create dev
jin mode use dev
echo '{"port": 9090, "feature": true}' > config.json
jin add config.json --mode
jin commit -m "Add mode config"

# 4. Apply and verify auto-merge (no conflict!)
jin apply

# 5. Check merged result
cat config.json
# Expected: {"port": 9090, "debug": false, "feature": true}
# Key: port from mode (9090), debug from global (false), feature from mode (true)

# 6. Verify no conflict file created
ls .jinmerge
# Expected: No such file or directory
```

### Level 5: Regression Testing

```bash
# Verify text files still create conflicts
echo 'original text' > README.txt
jin add README.txt --global
jin commit -m "Add global README"

jin mode use dev
echo 'different text' > README.txt
jin add README.txt --mode
jin commit -m "Add mode README"

jin apply
# Expected: Creates .jinmerge file for text file conflict

ls .jinmerge
# Expected: README.txt.jinmerge exists
```

## Final Validation Checklist

### Technical Validation

- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test`
- [ ] Zero clippy warnings: `cargo clippy`
- [ ] Zero check errors: `cargo check`
- [ ] Manual test passes: Structured files merge without conflicts
- [ ] Regression test passes: Text files still create conflicts

### Feature Validation

- [ ] Structured files (JSON/YAML/TOML/INI) with different content merge successfully
- [ ] Layer precedence works correctly (higher layers override lower layers)
- [ ] Deep merge correctly combines nested objects
- [ ] Text files with different content still trigger conflict detection
- [ ] No .jinmerge files created for structured file differences

### Code Quality Validation

- [ ] File placement matches codebase structure (src/merge/layer.rs)
- [ ] Uses existing function names (detect_format, has_different_text_content)
- [ ] Follows existing error handling patterns (Result<T>, JinError)
- [ ] No new functions added (minimal change principle)
- [ ] Comments updated to reflect new behavior

### Documentation & Deployment

- [ ] Function docstring updated (lines 95-108 in src/merge/layer.rs)
- [ ] Debug eprintln! statements preserved for troubleshooting
- [ ] No environment variables changed
- [ ] No configuration files changed

---

## Anti-Patterns to Avoid

- ❌ Don't modify `deep_merge()` or `deep.rs` - they're already correct
- ❌ Don't add new helper functions - use existing `detect_format()` and `has_different_text_content()`
- ❌ Don't change file format detection logic - it's working correctly
- ❌ Don't remove the optimization for same content (lines 148-168)
- ❌ Don't skip conflict detection for text files - they still need it
- ❌ Don't use hardcoded file extensions - use `FileFormat` enum
- ❌ Don't ignore the `Result<>` return type - handle errors properly
- ❌ Don't skip running tests - changes affect core merge behavior

## Example Test to Add

```rust
// Add this test to tests/pull_merge.rs or create tests/structured_merge_fix.rs

#[test]
fn test_structured_files_auto_merge_without_conflict() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode
    let mode_name = "test_mode";
    jin_cmd()
        .args(["mode", "create", mode_name])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success();

    // Add config.json to global layer
    let global_config = r#"{"port": 8080, "debug": false, "features": ["a", "b"]}"#;
    fs::write(fixture.path().join("config.json"), global_config).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--global"])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success();

    // Commit the global layer
    jin_cmd()
        .args(["commit", "-m", "Add global config"])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success();

    // Use the mode and add modified config.json
    jin_cmd()
        .args(["mode", "use", mode_name])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success();

    let mode_config = r#"{"port": 9090, "feature": true}"#;
    fs::write(fixture.path().join("config.json"), mode_config).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success();

    // Commit the mode layer
    jin_cmd()
        .args(["commit", "-m", "Add mode config"])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success();

    // Remove from workspace to trigger merge on apply
    fs::remove_file(fixture.path().join("config.json")).unwrap();

    // Apply should succeed without conflicts
    jin_cmd()
        .args(["apply"])
        .env("JIN_DIR", &jin_dir)
        .current_dir(fixture.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Merged"));

    // Verify merged content
    let merged_content = fs::read_to_string(fixture.path().join("config.json")).unwrap();
    let merged: serde_json::Value = serde_json::from_str(&merged_content).unwrap();

    // Port should be from mode layer (9090)
    assert_eq!(merged["port"], 9090);
    // Debug should be from global layer (false)
    assert_eq!(merged["debug"], false);
    // Feature should be from mode layer (true)
    assert_eq!(merged["feature"], true);
    // Features array should be from global layer (["a", "b"])
    assert_eq!(merged["features"], serde_json::json!(["a", "b"]));

    // Verify NO .jinmerge file was created
    assert!(!fixture.path().join(".jinmerge").exists());
}
```

## Confidence Score

**8/10** for one-pass implementation success

**Reasoning**:
- ✅ All context provided (exact line numbers, code patterns, test locations)
- ✅ Clear "before" and "after" code examples
- ✅ Comprehensive validation commands
- ✅ Example test implementation provided
- ⚠️ Risk: This is a core merge behavior change - thorough testing required
- ⚠️ Risk: Integration test setup complexity (TestFixture, JinRepo initialization)

**Mitigation**: The change is isolated to a single function (`merge_layers`) and follows existing patterns. The deep merge implementation is already correct - we're only removing an incorrect pre-check.

---

## Appendix: Quick Reference

### File: src/merge/layer.rs - Key Functions

| Function | Lines | Purpose | Changed? |
|----------|-------|---------|----------|
| `merge_layers()` | 109-200 | Main merge orchestration | ✅ YES |
| `detect_format()` | 433-442 | File format detection | ❌ No |
| `has_different_content_across_layers()` | 604-624 | Content comparison (buggy) | ❌ No |
| `has_different_text_content()` | 627-669 | Text file comparison | ❌ No |
| `has_different_structured_content()` | 672-717 | Structured file comparison | ❌ No |
| `merge_file_across_layers()` | 264-368 | Per-file merge routing | ❌ No |
| `deep_merge()` | (in deep.rs) | RFC 7396 deep merge | ❌ No |

### Change Summary

```
Lines to modify: 136-146 in src/merge/layer.rs
Lines added: ~10
Lines removed: ~5
Net change: ~5 lines
Risk level: Medium (core merge behavior)
Test coverage: High (comprehensive existing tests + new test needed)
```

### Success Verification Commands

```bash
# Quick check
cargo check && cargo test --lib merge::layer::merge_layers

# Full validation
cargo test --all

# Manual verification
cd /tmp && rm -rf jin_test && mkdir jin_test && cd jin_test
git init && jin init
echo '{"a":1}' > config.json && jin add config.json --global && jin commit -m "global"
jin mode create test && jin mode use test
echo '{"b":2}' > config.json && jin add config.json --mode && jin commit -m "mode"
rm config.json && jin apply
cat config.json  # Should show {"a":1,"b":2}
```

---

**PRP Version**: 1.0
**Created**: 2025-01-12
**For**: Subtask P1.M1.T1.S1 - Remove pre-merge conflict check for structured files
**Status**: Ready for Implementation
