# Product Requirement Prompt (PRP): JinMap Updates on Commits

**PRP ID**: P7.M5.T1
**Title**: Implement JinMap Updates
**Status**: Ready for Implementation
**Confidence Score**: 9/10

---

## Goal

**Feature Goal**: Automatically update `.jinmap` file after each successful commit to maintain accurate layer-to-file mappings for the entire repository.

**Deliverable**: A complete JinMap module (`src/core/jinmap.rs`) integrated with the CommitPipeline to track which files belong to which layers in the 9-layer hierarchy.

**Success Definition**:
- `.jinmap` file is automatically created/updated after every successful `jin commit`
- File contains accurate YAML mappings from layer ref paths to file paths
- Integration follows existing audit logging pattern (non-blocking, fails gracefully)
- All existing tests pass plus new JinMap tests pass
- `jin repair` can validate and regenerate `.jinmap` if corrupted

## User Persona (if applicable)

**Target User**: Jin CLI users (developers) who rely on layer-based configuration management

**Use Case**: After committing files to specific layers (e.g., `jin add .claude/config.json --mode claude` then `jin commit`), the `.jinmap` file should be updated to reflect that `.claude/config.json` belongs to the `mode/claude` layer.

**User Journey**:
1. User stages files with `jin add <file> --mode <mode>` or other layer targeting options
2. User commits with `jin commit -m "message"`
3. Commit completes successfully
4. `.jinmap` file at `.jin/.jinmap` is automatically updated with new mappings
5. User can query which layer a file belongs to using `jin layers` or inspect `.jinmap` directly

**Pain Points Addressed**:
- **Recovery**: Enables `jin repair` to validate repository integrity and rebuild layer state
- **Visibility**: Users can see layer-to-file mappings without parsing Git refs
- **Debugging**: Helps troubleshoot which layer a file belongs to when precedence issues occur

## Why

- **Repository State Recovery**: `.jinmap` provides a persistent record of layer mappings that can be used to recover Jin state if Git refs are corrupted or lost
- **Integration with PRD Section 826**: Guarantees ".jinmap auto-maintained and consistent after every commit"
- **Repair Command Foundation**: P4.M5.T6 (repair command) requires `.jinmap` validation - this provides the actual data to validate
- **Audit Trail**: Complements the audit logging system (P7.M4.T1) by providing a snapshot of current repository state

## What

**User-visible behavior**:
- `.jinmap` file is created at `.jin/.jinmap` on first commit (if it doesn't exist)
- After each successful `jin commit`, the file is updated with current layer-to-file mappings
- File format is YAML for human readability and editability (emergency repair)
- Mappings are stored as `layer_ref_path -> list of file paths`

**Technical requirements**:
- Create `JinMap` struct with YAML serialization/deserialization
- Implement layer-to-file mapping aggregation from Git tree objects
- Integrate into CommitPipeline after staging is cleared (line ~123)
- Follow non-blocking pattern: failure to update `.jinmap` logs warning but doesn't fail commit
- Add comprehensive unit tests for JinMap operations

### Success Criteria

- [ ] `JinMap` struct defined at `src/core/jinmap.rs` with `version`, `mappings`, and `meta` fields
- [ ] `.jinmap` file created/updated after every successful commit
- [ ] File content is valid YAML matching PRD format specification
- [ ] Mappings correctly aggregate files from all layer commits
- [ ] Non-blocking integration: commit succeeds even if `.jinmap` update fails
- [ ] All unit tests pass including new JinMap tests
- [ ] Existing `jin repair` validation works with new JinMap structure

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Complete codebase tree structure showing where files belong
- Specific file patterns to follow for serialization, error handling, and testing
- Exact integration point in CommitPipeline with line numbers
- YAML format specification with examples
- All necessary imports and dependencies
- Validation commands to verify implementation

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# PRD and Architecture Context
- url: plan/docs/PRD.md
  why: Defines .jinmap format specification in Section 16 and auto-maintenance requirement in Section 826
  critical: "Format: version: 1, mappings: {layer_path: [files]}, meta: {generated-by: jin}"
  section: "Section 16: JinMap Format, Section 826: Auto-maintenance guarantee"

# Key Implementation Pattern Files
- file: src/commit/pipeline.rs
  why: Shows exact integration point (line 123) and audit logging pattern to follow
  pattern: "Non-blocking operation after staging.clear() - log warning on failure"
  gotcha: "JinMap update must be AFTER tx.commit() succeeds and staging is cleared"

- file: src/staging/index.rs
  why: Shows JSON serialization pattern, default_path() pattern, load/save pattern
  pattern: "load() returns Ok(new()) if file doesn't exist, atomic write with temp file"
  gotcha: "Use serde_json for JSON, serde_yaml for YAML - JinMap uses YAML"

- file: src/audit/logger.rs
  why: Shows non-blocking file I/O pattern for post-commit operations
  pattern: "if let Err(e) = logger.log_entry() { eprintln!(\"Warning: {}\", e); }"
  gotcha: "Use BufWriter for efficient file writes, flush() to ensure persistence"

- file: src/core/layer.rs
  why: Layer enum with ref_path() method for generating layer ref strings
  pattern: "layer.ref_path(mode, scope, project) returns Git ref path like 'refs/jin/layers/mode/claude'"
  gotcha: "ref_path() returns String, storage_path() also available but ref_path is for .jinmap"

- file: src/core/config.rs
  why: Shows YAML serialization pattern for ProjectContext
  pattern: "serde_yaml::to_string() for serialization, serde_yaml::from_str() for deserialization"
  gotcha: "YAML requires serde_yaml crate, already in dependencies"

- file: src/commands/repair.rs
  why: Shows current .jinmap validation logic (lines 354-449) that needs to work with new format
  pattern: "serde_yaml::from_str::<serde_yaml::Value>() for validation"
  gotcha: "Current implementation creates basic comment file - needs to handle full YAML structure"

# Type and Module Structure
- file: src/staging/entry.rs
  why: StagedEntry shows path handling with PathBuf
  pattern: "pub path: PathBuf for file paths, convert to string with .display().to_string()"

- file: src/audit/entry.rs
  why: Shows complete serialization derive pattern
  pattern: "#[derive(Debug, Clone, Serialize, Deserialize)] with skip_serializing_if attributes"

# Testing Patterns
- file: src/staging/index.rs (lines 131-208)
  why: Unit test pattern for index operations with tempfile
  pattern: "create_test_setup() helper, assert_eq! for verification"

- file: src/audit/logger.rs (lines 93-274)
  why: Test pattern for file I/O operations with isolated temp directories
  pattern: "use tempfile::TempDir for isolated test environments"
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin/
├── src/
│   ├── core/
│   │   ├── mod.rs          # Module exports, add JinMap here
│   │   ├── error.rs        # JinError enum, Result type
│   │   ├── layer.rs        # Layer enum with ref_path(), storage_path()
│   │   └── config.rs       # YAML serialization pattern
│   ├── commit/
│   │   ├── mod.rs          # Commit module exports
│   │   └── pipeline.rs     # CommitPipeline - INTEGRATION POINT at line ~123
│   ├── staging/
│   │   ├── mod.rs
│   │   ├── index.rs        # StagingIndex with entries_for_layer()
│   │   └── entry.rs        # StagedEntry with path, target_layer, content_hash
│   ├── audit/
│   │   ├── mod.rs
│   │   ├── entry.rs        # AuditEntry type
│   │   └── logger.rs       # Non-blocking file I/O pattern
│   ├── git/
│   │   ├── mod.rs
│   │   ├── repo.rs         # JinRepo with tree walking methods
│   │   └── tree.rs         # Tree reading capabilities
│   └── commands/
│       └── repair.rs       # .jinmap validation (lines 354-449)
├── Cargo.toml              # Dependencies: serde_yaml = "0.9"
└── plan/
    └── P7M5T1/
        └── PRP.md          # This file
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin/
├── src/
│   ├── core/
│   │   ├── mod.rs          # MODIFY: Add `pub mod jinmap;` export
│   │   └── jinmap.rs       # NEW: JinMap struct, load/save/update methods
│   └── commit/
│       └── pipeline.rs     # MODIFY: Add JinMap update after staging.clear()
```

**File Responsibilities**:
- `src/core/jinmap.rs` (NEW): Core JinMap data structure and file I/O operations
- `src/core/mod.rs` (MODIFY): Export new jinmap module
- `src/commit/pipeline.rs` (MODIFY): Integrate JinMap update in execute() method

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Use serde_yaml, NOT serde_json for .jinmap
// The PRD specifies YAML format for human readability
// Dependencies already include serde_yaml = "0.9"

// CRITICAL: Integration point is AFTER staging.clear() and BEFORE audit logging
// Location: src/commit/pipeline.rs:123
// Must use commit result context (layer_commits, files) to build mappings

// CRITICAL: Non-blocking pattern required
// Follow audit logging pattern: if let Err(e) = update_jinmap() { eprintln!("Warning: {}", e); }
// Commit should NOT fail if .jinmap update fails

// CRITICAL: Layer ref paths use layer.ref_path(mode, scope, project)
// Returns Git ref paths like "refs/jin/layers/mode/claude"
// NOT storage_path() which returns directory paths like "jin/mode/claude/"

// GOTCHA: JinMap must aggregate from COMMITTED tree objects, not staged entries
// After commit, read tree from layer refs to get accurate file list
// Staging may have deletions not reflected in final tree

// GOTCHA: PathBuf to String conversion
// Use path.display().to_string() for consistent forward-slash paths
// Avoid path.to_str() which may fail on non-UTF8 paths

// PATTERN: Atomic write with temp file (from WorkspaceMetadata in staging/metadata.rs)
// let temp_path = path.with_extension("tmp");
// std::fs::write(&temp_path, content)?;
// std::fs::rename(&temp_path, &path)?;

// PATTERN: Default path pattern
// pub fn default_path() -> PathBuf { PathBuf::from(".jin").join(".jinmap") }
// Matches .jin/context, .jin/staging/index.json pattern

// PATTERN: Load returns Ok(new()) if file doesn't exist
// Allows first-run creation without error handling
// See src/staging/index.rs:35-45

// ERROR HANDLING: Use JinError::Parse for serialization failures
// Use JinError::Io for filesystem failures
// Map errors appropriately in save/load methods

// TESTING: Use tempfile crate for isolated test environments
// Already in dev-dependencies, see src/audit/logger.rs tests

// TESTING: Test YAML serialization round-trip
// Serialize -> Deserialize -> Assert equality
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// src/core/jinmap.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::core::{JinError, Layer, Result};

/// JinMap: Layer-to-file mapping metadata
///
/// Tracks which files belong to which layers in the 9-layer hierarchy.
/// Stored at `.jin/.jinmap` in YAML format.
///
/// Format per PRD Section 16:
/// ```yaml
/// version: 1
/// mappings:
///   "refs/jin/layers/mode/claude":
///     - ".claude/config.json"
///     - ".claude/prompt.md"
///   "refs/jin/layers/project/myproject":
///     - "config/settings.json"
/// meta:
///   generated-by: jin
///   last-updated: "2025-01-01T12:00:00Z"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinMap {
    /// Format version (for future migration support)
    #[serde(default = "default_version")]
    pub version: u32,

    /// Layer ref path -> list of file paths
    ///
    /// Key: Git ref path like "refs/jin/layers/mode/claude"
    /// Value: List of file paths relative to workspace root
    pub mappings: HashMap<String, Vec<String>>,

    /// Metadata about the JinMap file
    #[serde(default)]
    pub meta: JinMapMeta,
}

/// Metadata for JinMap file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinMapMeta {
    /// Tool that generated this file
    #[serde(default = "default_generated_by")]
    pub generated_by: String,

    /// Last update timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

fn default_version() -> u32 {
    1
}

fn default_generated_by() -> String {
    "jin".to_string()
}

impl Default for JinMap {
    fn default() -> Self {
        Self {
            version: default_version(),
            mappings: HashMap::new(),
            meta: JinMapMeta::default(),
        }
    }
}

impl Default for JinMapMeta {
    fn default() -> Self {
        Self {
            generated_by: default_generated_by(),
            last_updated: None,
        }
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/core/jinmap.rs - Core Data Structures
  - IMPLEMENT: JinMap struct with version, mappings, meta fields
  - IMPLEMENT: JinMapMeta struct with generated_by, last_updated fields
  - ADD: Default implementations for both structs
  - ADD: Helper functions: default_version(), default_generated_by()
  - FOLLOW pattern: src/core/config.rs (ProjectContext struct with derives)
  - NAMING: PascalCase for structs, snake_case for functions
  - DEPENDENCIES: None ( foundational types )
  - PLACEMENT: New file at src/core/jinmap.rs
  - SERIALIZATION: Use #[serde(default, skip_serializing_if = "Option::is_none")] for optional fields

Task 2: CREATE src/core/jinmap.rs - Load/Save Methods
  - IMPLEMENT: JinMap::load() -> Result<Self>
  - IMPLEMENT: JinMap::save(&self) -> Result<()>
  - IMPLEMENT: JinMap::default_path() -> PathBuf
  - FOLLOW pattern: src/staging/index.rs:32-62 (load/save pattern)
  - LOAD PATTERN: Return Ok(Self::default()) if file doesn't exist
  - SAVE PATTERN: Atomic write with temp file then rename
  - SERIALIZATION: Use serde_yaml::to_string() and serde_yaml::from_str()
  - ERROR HANDLING: Map serde errors to JinError::Parse with "YAML" format
  - DEPENDENCIES: Task 1 (data structures must exist first)
  - PLACEMENT: Methods in impl JinMap block in src/core/jinmap.rs

Task 3: CREATE src/core/jinmap.rs - Update Methods
  - IMPLEMENT: JinMap::update_from_commits(layer_commits: &[(Layer, Oid)], context: &ProjectContext, repo: &JinRepo) -> Result<()>
  - IMPLEMENT: JinMap::add_layer_mapping(layer_ref: &str, files: Vec<String>)
  - IMPLEMENT: Private helper: walk_layer_tree(repo, layer_oid) -> Result<Vec<String>>
  - ALGORITHM:
    1. For each (layer, oid) pair:
       a. Get layer ref path via layer.ref_path(mode, scope, project)
       b. Read tree object from oid
       c. Walk tree to collect all file paths
       d. Add to mappings HashMap
    2. Update meta.last_updated timestamp
  - TREE WALKING: Use repo.tree_walk(tree_oid) to get files
  - PATH HANDLING: Use path.display().to_string() for forward-slash consistency
  - DEPENDENCIES: Task 2 (load/save for persistence), src/git/repo.rs (tree reading)
  - PLACEMENT: Methods in impl JinMap block in src/core/jinmap.rs
  - GOTCHA: Must handle empty trees (all files deleted)

Task 4: MODIFY src/core/mod.rs - Module Export
  - ADD: `pub mod jinmap;` at top of file
  - PRESERVE: All existing module exports
  - PATTERN: Follow existing module declarations (config, layer, error)
  - PLACEMENT: After other pub mod declarations, alphabetically ordered
  - DEPENDENCIES: Task 1 (jinmap.rs must exist first)

Task 5: MODIFY src/commit/pipeline.rs - Import JinMap
  - ADD: `use crate::core::JinMap;` at top of file
  - PRESERVE: All existing imports
  - PATTERN: Follow existing import style (grouped by crate module)
  - PLACEMENT: In use statements section with other core imports
  - DEPENDENCIES: Task 4 (JinMap must be exported from core)

Task 6: MODIFY src/commit/pipeline.rs - Integrate JinMap Update
  - ADD: JinMap update call after staging.clear() (line 124)
  - LOCATION: After self.staging.save()? and before audit logging (line 133)
  - IMPLEMENT: self.update_jinmap(&layer_commits, &context, &repo)?
  - PATTERN: Non-blocking like audit logging (line 132-135)
  - ERROR HANDLING: if let Err(e) = update_jinmap() { eprintln!("Warning: Failed to update .jinmap: {}", e); }
  - CONTEXT AVAILABLE: layer_commits has (Layer, Oid) tuples, context has mode/scope/project
  - DEPENDENCIES: Task 3 (update_from_commits method), Task 5 (import)
  - PLACEMENT: New private method in impl CommitPipeline block

Task 7: MODIFY src/commit/pipeline.rs - Add update_jinmap Helper
  - IMPLEMENT: fn update_jinmap(&self, layer_commits: &[(Layer, Oid, Option<String>)], context: &ProjectContext, repo: &JinRepo) -> Result<()>
  - ALGORITHM:
    1. Load or create JinMap
    2. Call jinmap.update_from_commits(layer_commits, context, repo)
    3. Save updated JinMap
  - ERROR HANDLING: Return Result<()> to allow caller to decide blocking behavior
  - DEPENDENCIES: Task 6 (integration point)
  - PLACEMENT: Private method in impl CommitPipeline block after log_audit()
  - PATTERN: Follow log_audit() structure (lines 261-290)

Task 8: CREATE src/core/jinmap.rs - Unit Tests
  - IMPLEMENT: test_jinmap_default() - verify default structure
  - IMPLEMENT: test_jinmap_serialize_deserialize() - YAML round-trip
  - IMPLEMENT: test_jinmap_load_creates_default() - missing file handling
  - IMPLEMENT: test_jinmap_save_load_roundtrip() - persistence test
  - IMPLEMENT: test_jinmap_add_layer_mapping() - mapping updates
  - FOLLOW pattern: src/staging/index.rs:131-208 (test structure)
  - USE: tempfile::TempDir for isolated test environments
  - COVERAGE: All public methods with positive/negative cases
  - DEPENDENCIES: Task 3 (all methods implemented)
  - PLACEMENT: #[cfg(test)] mod tests in src/core/jinmap.rs

Task 9: MODIFY src/commit/pipeline.rs - Integration Tests
  - IMPLEMENT: test_commit_pipeline_updates_jinmap() - verify .jinmap created
  - IMPLEMENT: test_commit_pipeline_jinmap_content() - verify YAML format
  - IMPLEMENT: test_commit_pipeline_jinmap_multiple_layers() - verify aggregation
  - IMPLEMENT: test_commit_pipeline_jinmap_non_blocking() - verify warning on failure
  - FOLLOW pattern: src/commit/pipeline.rs:302-663 (existing test patterns)
  - USE: create_test_setup() helper from existing tests
  - ASSERT: .jinmap file exists, contains correct mappings
  - DEPENDENCIES: Task 7 (full integration complete)
  - PLACEMENT: In #[cfg(test)] mod tests in src/commit/pipeline.rs
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: YAML Serialization (from src/core/config.rs)
// Use serde_yaml for .jinmap files per PRD specification

use serde_yaml;

let yaml_content = serde_yaml::to_string(&jinmap)
    .map_err(|e| JinError::Parse {
        format: "YAML".to_string(),
        message: e.to_string(),
    })?;

let parsed: JinMap = serde_yaml::from_str(&yaml_content)
    .map_err(|e| JinError::Parse {
        format: "YAML".to_string(),
        message: e.to_string(),
    })?;

// Pattern 2: Load with Default Fallback (from src/staging/index.rs:35-45)
// Returns default instance if file doesn't exist (first-run pattern)

pub fn load() -> Result<Self> {
    let path = Self::default_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_yaml::from_str(&content).map_err(|e| JinError::Parse {
            format: "YAML".to_string(),
            message: e.to_string(),
        })
    } else {
        Ok(Self::default())
    }
}

// Pattern 3: Atomic Write (from src/staging/metadata.rs)
// Write to temp file first, then atomic rename to prevent corruption

pub fn save(&self) -> Result<()> {
    let path = Self::default_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_yaml::to_string(self)
        .map_err(|e| JinError::Parse {
            format: "YAML".to_string(),
            message: e.to_string(),
        })?;

    // Atomic write pattern
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, &path)?;

    Ok(())
}

// Pattern 4: Non-blocking Integration (from src/commit/pipeline.rs:132-135)
// Follow audit logging pattern - log warning but don't fail operation

// In CommitPipeline::execute() after staging.clear():
if let Err(e) = self.update_jinmap(&layer_commits, &context, &repo) {
    eprintln!("Warning: Failed to update .jinmap: {}", e);
}

// Pattern 5: Layer Ref Path Generation
// Use layer.ref_path(mode, scope, project) for mapping keys

let ref_path = layer.ref_path(
    context.mode.as_deref(),
    context.scope.as_deref(),
    context.project.as_deref(),
);
// Returns: "refs/jin/layers/mode/claude" or similar

// Pattern 6: Tree Walking for File Collection
// Read committed tree to get actual file list (not staged entries)

fn walk_layer_tree(&self, repo: &JinRepo, layer_oid: Oid) -> Result<Vec<String>> {
    let tree = repo.find_tree(layer_oid)?;

    let mut files = Vec::new();
    // Walk tree recursively collecting file paths
    // Use repo.tree_walk() or iterate tree entries
    // Convert PathBuf to String with .display().to_string()

    Ok(files)
}

// CRITICAL: Update timestamp on each modification
meta.last_updated = Some(chrono::Utc::now().to_rfc3339());

// CRITICAL: Handle empty tree (all files deleted from layer)
// Return empty Vec, don't add to mappings OR add with empty list
// PRD shows empty lists are valid: "mode/claude": []
```

### Integration Points

```yaml
COMMIT_PIPELINE:
  - file: src/commit/pipeline.rs
  - location: Line 124-135 (after staging.clear(), before audit logging)
  - code: |
    // Clear staging on success
    self.staging.clear();
    self.staging.save()?;

    // Update JinMap with new layer mappings (non-blocking)
    if let Err(e) = self.update_jinmap(&layer_commits, &context, &repo) {
        eprintln!("Warning: Failed to update .jinmap: {}", e);
    }

    // Write audit log (non-blocking - log warning on failure)
    if let Err(e) = self.log_audit(&layer_commits, &context, &files) {
        eprintln!("Warning: Failed to write audit log: {}", e);
    }

CORE_MODULE:
  - file: src/core/mod.rs
  - add: pub mod jinmap;
  - after: pub mod config; line

REPAIR_COMMAND:
  - file: src/commands/repair.rs
  - note: Existing validation (lines 354-449) should work with new YAML format
  - update: May need to adjust create_default_jinmap() to use full structure
  - code: |
    // Update create_default_jinmap() to use actual JinMap struct:
    let jinmap = JinMap::default();
    let content = serde_yaml::to_string(&jinmap)?;
    std::fs::write(path, content)?;
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --package jin 2>&1 | head -50

# Format check
cargo fmt --all -- --check

# Expected: Zero compilation errors. If errors exist, READ output and fix before proceeding.

# Common fixes:
# - Missing imports: Add use crate::core::{JinError, Result, Layer};
# - Missing derives: Add #[derive(Debug, Clone, Serialize, Deserialize)]
# - Wrong method visibility: Change pub fn to fn for private helpers
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test JinMap module
cargo test jinmap -- --nocapture

# Test individual functions
cargo test test_jinmap_default -- --nocapture
cargo test test_jinmap_serialize_deserialize -- --nocapture
cargo test test_jinmap_load_creates_default -- --nocapture

# Test commit pipeline integration
cargo test test_commit_pipeline_updates_jinmap -- --nocapture
cargo test test_commit_pipeline_jinmap_content -- --nocapture

# Run all tests
cargo test -- --nocapture

# Expected: All tests pass. If failing, debug root cause and fix implementation.

# Test coverage verification (optional, if cargo-llvm-cov installed)
cargo llvm-cov --lib --workspace
```

### Level 3: Integration Testing (System Validation)

```bash
# Initialize test repository
cd /tmp && mkdir -p jin-test-jinmap && cd jin-test-jinmap
git init
cargo run -- init

# Add files to different layers
echo '{"mode": "claude"}' > .claude/config.json
cargo run -- add .claude/config.json --mode claude

echo '{"project": "test"}' > config.json
cargo run -- add config.json --project test

# Commit changes
cargo run -- commit -m "Test JinMap update"

# Verify .jinmap was created
cat .jin/.jinmap

# Expected output (YAML format):
# ---
# version: 1
# mappings:
#   "refs/jin/layers/mode/claude":
#     - ".claude/config.json"
#   "refs/jin/layers/project/test":
#     - "config.json"
# meta:
#   generated-by: jin
#   last-updated: "2025-01-01T..."

# Verify YAML is valid
python3 -c "import yaml; yaml.safe_load(open('.jin/.jinmap'))"
echo $?  # Should be 0

# Verify repair command validates .jinmap
cargo run -- repair --fix

# Expected: "Checking .jinmap... ✓"

# Test non-blocking behavior (corrupt .jinmap and try commit)
echo "invalid yaml" > .jin/.jinmap
cargo run -- commit -m "Test non-blocking"
# Expected: Commit succeeds, warning about .jinmap failure

# Test file deletion from layer
cargo run -- rm .claude/config.json
cargo run -- commit -m "Remove file"
cat .jin/.jinmap
# Expected: .claude/config.json removed from mode/claude mapping
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Multi-layer commit validation
cargo run -- add file1.txt --mode claude
cargo run -- add file2.txt --scope language:rust
cargo run -- add file3.txt --project myproject
cargo run -- commit -m "Multi-layer test"

# Verify all three layers in .jinmap
grep -c "refs/jin/layers" .jin/.jinmap
# Expected: 3 (one per layer)

# Verify precedence order (mappings should be sorted)
cargo run -- layers
# Should show layers with files highlighted

# Edge case: Commit with only deletions
cargo run -- rm file1.txt
cargo run -- commit -m "Delete only"
cat .jin/.jinmap
# Expected: mode/claude entry has empty file list or is removed

# Edge case: Empty repository first commit
cd /tmp && mkdir -p jin-empty && cd jin-empty
git init
cargo run -- init
echo "test" > file.txt
cargo run -- add file.txt
cargo run -- commit -m "First commit"
cat .jin/.jinmap
# Expected: .jinmap created with initial mapping

# Performance test with many files (optional)
mkdir -p /tmp/jin-perf && cd /tmp/jin-perf
git init
cargo run -- init
seq 1 1000 | xargs -I{} sh -c 'echo "content" > file{}.txt && cargo run -- add file{}.txt --project test'
time cargo run -- commit -m "Bulk commit"
# Expected: Completes in reasonable time (< 10s for 1000 files)

# Recoverability test: Corrupt refs, rebuild from .jinmap
# (Advanced test for future enhancement)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test` returns with exit code 0
- [ ] No compilation errors: `cargo check` clean
- [ ] No formatting issues: `cargo fmt --check` clean
- [ ] No clippy warnings: `cargo clippy` clean

### Feature Validation

- [ ] `.jinmap` file created at `.jin/.jinmap` on first commit
- [ ] File contains valid YAML matching PRD Section 16 format
- [ ] Mappings accurately reflect committed layer-to-file relationships
- [ ] Update happens after EVERY successful commit
- [ ] Non-blocking: Commit succeeds even if .jinmap update fails
- [ ] Warning message shown on .jinmap update failure
- [ ] `jin repair` validates .jinmap correctly

### Code Quality Validation

- [ ] Follows existing serialization patterns (config.rs, staging/index.rs)
- [ ] File placement matches desired codebase tree structure
- [ ] Module exports added to src/core/mod.rs
- [ ] Integration point matches specification (pipeline.rs:124)
- [ ] Error handling uses JinError types consistently
- [ ] Public methods documented with doc comments
- [ ] Private helpers use snake_case naming

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable/function names
- [ ] Doc comments on public structs and methods (/// format)
- [ ] Module documentation (//! at top of jinmap.rs)
- [ ] Example YAML format in JinMap struct doc comment
- [ ] Integration point commented in pipeline.rs

### Anti-Patterns Avoided

- [x] No new serialization patterns created (uses existing serde_yaml)
- [x] No blocking operations (follows audit logging pattern)
- [x] No hardcoded layer paths (uses layer.ref_path())
- [x] No JSON format used (PRD specifies YAML)
- [x] No premature optimization (simple HashMap for mappings)
- [x] No panic on file I/O errors (returns Result)
- [x] No duplicate code (follows existing save/load patterns)

---

## Anti-Patterns to Avoid

- ❌ Don't use JSON format - PRD specifies YAML for .jinmap
- ❌ Don't block commit on .jinmap update - must be non-blocking like audit logging
- ❌ Don't use storage_path() for mapping keys - use ref_path() for Git refs
- ❌ Don't aggregate from staged entries - use committed tree objects
- ❌ Don't hardcode layer paths - use layer.ref_path() with context
- ❌ Don't fail commit on .jinmap errors - log warning and continue
- ❌ Don't create temp files outside .jin directory - use .jin/.jinmap.tmp
- ❌ Don't skip tests for edge cases (empty tree, all deletions, etc.)
- ❌ Don't forget to update metadata timestamp on each modification
- ❌ Don't use sync functions in async contexts (not applicable here, but good practice)

---

## Appendix: PRD References

### Relevant PRD Sections

- **Section 16**: JinMap format specification
  - Defines YAML structure with version, mappings, meta
  - Example: `"mode/claude": [".claude/", "CLAUDE.md"]`

- **Section 422**: Auto-repair guarantees
  - System can auto-repair corrupted .jinmap files

- **Section 564**: Update timing
  - .jinmap updated after atomic commits complete

- **Section 826**: Auto-maintenance guarantee
  - ".jinmap auto-maintained and consistent after every commit"

### Related Tasks

- **P7.M4.T1**: Audit Logging System - provides non-blocking pattern to follow
- **P4.M5.T6**: Repair Command - validates .jinmap integrity
- **P7.M2.T1/T2**: File Operations (rm/mv) - must update .jinmap when files moved/deleted

---

## Confidence Score: 9/10

**Rationale**:
- Complete codebase analysis with specific file paths and line numbers
- All dependencies (serde_yaml) already in project
- Clear integration point with existing patterns to follow
- Comprehensive validation commands for each implementation phase
- PRD specification is unambiguous and achievable
- Only risk: Git tree walking complexity may require iteration

**Mitigation for 10% risk**:
- Start with simplified tree reading (assume flat files)
- Add recursive directory walking in follow-up if needed
- Extensive test coverage catches edge cases early

---

**End of PRP**
