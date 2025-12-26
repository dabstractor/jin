# Product Requirement Prompt (PRP): Pipeline Implementation (P3.M2.T1)

---

## Goal

**Feature Goal**: Implement the `CommitPipeline` that orchestrates atomic commits of staged files to Jin layers with validation, jinmap management, and audit logging.

**Deliverable**: A `src/commit/` module with:
- `pipeline.rs` containing `CommitPipeline` struct for orchestrating commits
- `validate.rs` containing pre-commit validation logic
- `jinmap.rs` containing `.jinmap` file generation and management
- `audit.rs` containing audit trail logging
- Comprehensive unit tests with temp directories

**Success Definition**:
- `CommitPipeline` can process `StagingIndex` entries and create atomic Git commits
- All validation rules are enforced (symlinks, binary files, git-tracked files)
- `.jinmap` is auto-generated and updated after each commit
- Audit trail records all commits with layer and file information
- Transaction atomicity is maintained (all-or-nothing updates)
- `cargo test --package jin --lib commit` passes all tests
- No clippy warnings or rustc errors
- Module exported from `src/commit/mod.rs`

## User Persona

**Target User**: AI coding agent implementing Jin's commit pipeline foundation

**Use Case**: The agent needs to create the commit pipeline infrastructure that:
- Orchestrates staged files into atomic Git layer commits
- Validates files before committing to prevent invalid state
- Generates and maintains `.jinmap` for layer-to-file tracking
- Logs audit trail for all commits
- Integrates with existing `Transaction`, `StagingIndex`, and `JinRepo` types

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/commit/validate.rs` with validation logic
3. Creates `src/commit/jinmap.rs` with jinmap management
4. Creates `src/commit/pipeline.rs` with commit orchestration
5. Creates `src/commit/audit.rs` with audit logging
6. Adds comprehensive unit tests
7. Validates compilation and test success

**Pain Points Addressed**:
- No manual orchestration of staging -> Git commit flow
- Consistent validation rules across all commits
- Automatic `.jinmap` maintenance without user intervention
- Complete audit trail for debugging and recovery
- Transaction atomicity guaranteed through existing `Transaction` type

## Why

- **Foundation for commit command**: Every `jin commit` operation depends on this pipeline
- **Validation enforcement**: Centralizes all validation rules for consistency
- **Automatic jinmap**: Removes manual `.jinmap` maintenance from users
- **Audit capability**: Provides complete history of all layer changes
- **Problems this solves**:
  - Provides consistent commit flow for all layer operations
  - Ensures only valid files enter Jin layers
  - Maintains accurate layer-to-file mapping automatically
  - Enables recovery and debugging through audit logs
  - Guarantees atomic multi-layer commits via existing `Transaction`

## What

Implement the commit pipeline that orchestrates atomic commits of staged files to Jin layers with validation, jinmap management, and audit logging.

### Success Criteria

- [ ] `src/commit/validate.rs` created with validation functions
- [ ] `src/commit/jinmap.rs` created with `Jinmap` struct and management
- [ ] `src/commit/pipeline.rs` created with `CommitPipeline` struct
- [ ] `src/commit/audit.rs` created with audit logging
- [ ] `CommitPipeline::execute()` validates all staged entries
- [ ] `CommitPipeline::execute()` builds Git trees for each affected layer
- [ ] `CommitPipeline::execute()` creates commits and updates refs atomically
- [ ] `CommitPipeline::execute()` updates `.jinmap` after successful commit
- [ ] `CommitPipeline::execute()` logs audit record for each commit
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover all public methods with temp directories
- [ ] Module exported from `src/commit/mod.rs`
- [ ] All tests pass: `cargo test --package jin --lib commit`

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact `CommitPipeline`, `Jinmap`, and validation function specifications
- Research documents with commit pipeline patterns and validation requirements
- Specific patterns from existing codebase to follow (`Transaction`, `StagingIndex`, `JinRepo`)
- Complete integration guide with existing types
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Complete commit specification with atomicity requirements and jinmap format
  section: Lines 106-134 for Commit Atomicity (§6.2), Lines 426-441 for .jinmap (§16), Lines 442-469 for Audit Logs (§17)
  critical: "jin commit is atomic across all affected layers", .jinmap format, audit log JSON structure

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Module structure and integration points
  section: Lines 54-85 for Module Structure showing commit/ module layout
  critical: commit/pipeline.rs, commit/jinmap.rs, commit/validate.rs, commit/audit.rs

- file: /home/dustin/projects/jin-glm-doover/src/staging/index.rs
  why: StagingIndex structure for processing staged entries
  pattern: StagingIndex.entries_by_layer() for grouping by layer, StagingIndex.all_entries()
  section: Lines 96-393 for StagingIndex implementation

- file: /home/dustin/projects/jin-glm-doover/src/staging/entry.rs
  why: StagedEntry structure with file metadata
  pattern: StagedEntry fields: path, layer, content_hash, status, size
  section: Lines 98-270 for StagedEntry implementation

- file: /home/dustin/projects/jin-glm-doover/src/git/transaction.rs
  why: Transaction system for atomic multi-layer commits
  pattern: Transaction::begin(), add_layer_update(), prepare(), commit()
  section: Lines 126-438 for Transaction implementation
  critical: Uses git2 transaction API for atomic ref updates

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: JinRepo wrapper for Git operations
  pattern: create_blob(), treebuilder(), create_tree(), create_commit(), set_layer_ref()
  section: Lines 600-821 for object creation helpers

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  pattern: JinError::SymlinkNotSupported, JinError::BinaryFileNotSupported, JinError::ValidationError
  gotcha: Use ? operator for automatic std::io::Error conversion

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum with git_ref() for layer reference paths
  section: Lines 282-456 for git_ref() implementation
  critical: Layer::git_ref() returns exact ref paths for each layer variant

- file: /home/dustin/projects/jin-glm-doover/src/commit/mod.rs
  why: Module must export commit types after creation
  pattern: Currently placeholder - needs to export validate, jinmap, pipeline, audit

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify dependencies are available
  section: Lines 19-34 for dependencies
  critical: git2 (0.20), serde (1.0), serde_yaml (0.9), chrono (0.4), tempfile (3.12)

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P3M2T1/research/commit_pipeline_patterns.md
  why: External patterns for commit pipeline implementation
  section: Git2 transaction patterns, atomic commit strategies, error handling
  critical: Two-phase commit pattern, staging reference management, RAII cleanup

# INTERNAL - Jinmap Specification

From PRD §16, .jinmap format is:
```yaml
version: 1
mappings:
  "mode/claude": [".claude/", "CLAUDE.md"]
meta:
  generated-by: jin
```

Key jinmap requirements:
- Auto-generated, never user-edited
- Recoverable from Git history
- Updated after every successful commit
- Layer paths use exact format from Layer::git_ref() (without refs/jin/layers/ prefix)

# INTERNAL - Audit Log Specification

From PRD §17, audit log entries are JSON:
```json
{
  "timestamp": "2025-10-19T15:04:02Z",
  "user": "dustin",
  "project": "ui-dashboard",
  "mode": "claude",
  "scope": "language:javascript",
  "layer": 4,
  "files": [".claude/config.json"],
  "base_commit": "abc123",
  "merge_commit": "def456",
  "context": {
    "active_mode": "claude",
    "active_scope": "language:javascript"
  }
}
```

Audit logs stored in `~/.jin/repo/.audit/` directory, append-only.

# EXTERNAL - Git Documentation

- url: https://docs.rs/git2/0.20/git2/struct.Transaction.html
  why: git2 Transaction API for atomic multi-ref updates
  critical: lock_ref(), set_target(), commit() methods

- url: https://git-scm.com/docs/git-read-tree#_atomic_tree_merge_updates
  why: Git's atomic update patterns
  critical: Understanding of atomic multi-tree operations

# EXTERNAL - Rust Crate Documentation

- url: https://docs.rs/serde_yaml/latest/serde_yaml/
  why: YAML serialization for .jinmap format
  critical: serialize, deserialize traits for YAML format

- url: https://docs.rs/chrono/latest/chrono/
  why: Timestamp generation for audit logs
  critical: DateTime<Utc>::now() for current timestamp

- url: https://docs.rs/tempfile/latest/tempfile/
  why: Temporary directory creation for testing
  critical: TempDir::new() for test isolation
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                      # Has all required dependencies
├── PRD.md                          # Commit, .jinmap, and audit specification
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs                 # Exports error, layer, config
│   │   ├── error.rs               # Has SymlinkNotSupported, BinaryFileNotSupported errors
│   │   ├── layer.rs               # Has Layer enum with git_ref() method
│   │   └── config.rs              # Has ProjectContext type
│   ├── git/
│   │   ├── mod.rs                 # Exports repo, transaction
│   │   ├── repo.rs                # Has JinRepo with blob/tree/commit creation
│   │   └── transaction.rs         # Has Transaction for atomic multi-layer commits
│   ├── staging/
│   │   ├── mod.rs                 # Exports entry, index, router
│   │   ├── entry.rs               # Has StagedEntry struct
│   │   ├── index.rs               # Has StagingIndex with entries_by_layer()
│   │   └── router.rs              # Has LayerRouter for layer determination
│   ├── commit/
│   │   └── mod.rs                 # Currently placeholder - needs implementation
│   ├── merge/
│   │   ├── mod.rs                 # Exports value, text, layer
│   │   └── layer.rs               # Has LayerMerge for merging layers
│   └── workspace/
│       └── mod.rs                 # Workspace operations
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── commit/
│       ├── mod.rs                 # MODIFY: Add pub mod validate, jinmap, pipeline, audit; pub use ...
│       ├── validate.rs            # CREATE: Validation functions for staged entries
│       ├── jinmap.rs              # CREATE: Jinmap struct and management
│       ├── pipeline.rs            # CREATE: CommitPipeline orchestration
│       └── audit.rs               # CREATE: Audit trail logging
├── tests/
│   └── commit/
│       ├── validate_test.rs       # CREATE: Unit tests for validation
│       ├── jinmap_test.rs         # CREATE: Unit tests for jinmap
│       ├── pipeline_test.rs       # CREATE: Unit tests for pipeline
│       └── audit_test.rs          # CREATE: Unit tests for audit logging
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Use existing Transaction for atomic commits
// The Transaction type in src/git/transaction.rs already handles:
// - Staging ref creation for transaction tracking
// - Locking multiple layer refs atomically
// - RAII cleanup via Drop trait
// Good:
//   let tm = TransactionManager::new(&repo);
//   let mut tx = tm.begin_transaction()?;
//   for (layer, oid) in layer_updates {
//       tx.add_layer_update(layer, oid)?;
//   }
//   tx.prepare()?;
//   tx.commit()?;
// Bad: Manual ref updates without transaction

// CRITICAL: Layer::git_ref() returns exact ref path for each layer
// Always use Layer::git_ref() instead of hardcoded strings
// Good:
//   let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer { ... })?;
// Bad:
//   let ref_name = match layer {
//       Layer::GlobalBase => "refs/jin/layers/global".to_string(),
//       ...
//   };

// CRITICAL: Group staged entries by layer before building trees
// Use StagingIndex.entries_by_layer() to get entries for each layer
// Good:
//   for layer in affected_layers {
//       let entries = staging.entries_by_layer(&layer);
//       let tree_oid = build_layer_tree(&repo, &entries)?;
//       layer_updates.push((layer, tree_oid));
//   }
// Bad: Processing all entries without layer grouping

// CRITICAL: Validate BEFORE building trees or creating commits
// Validation failures should be caught early before any Git operations
// Good:
//   validate_staging_index(&staging, &workspace_root)?;
//   let layer_updates = build_layer_trees(...)?;
//   let commits = create_commits(...)?;
// Bad: Building trees first, validating later

// CRITICAL: .jinmap uses layer path WITHOUT refs/jin/layers/ prefix
// When generating jinmap, strip the refs/jin/layers/ prefix from ref names
// Good:
//   let layer_path = ref_name.strip_prefix("refs/jin/layers/").unwrap();
//   jinmap.mappings.insert(layer_path.to_string(), files);
// Bad: Using full ref names in jinmap

// CRITICAL: Audit logs go to ~/.jin/repo/.audit/ directory
// This is separate from the project workspace
// Good:
//   let audit_dir = jin_repo.path().join(".audit");
//   std::fs::create_dir_all(&audit_dir)?;
// Bad: Writing audit logs to project workspace

// CRITICAL: Use SHA-256 for content hashing (already used in StagedEntry)
// Don't use Git's SHA-1
// Good: StagedEntry.content_hash already uses SHA-256
// Bad: Computing separate hash with SHA-1

// CRITICAL: Check for symlinks using std::fs::symlink_metadata()
// Cross-platform symlink detection
// Good:
//   let metadata = std::fs::symlink_metadata(path)?;
//   if metadata.file_type().is_symlink() {
//       return Err(JinError::SymlinkNotSupported { path: ... });
//   }
// Bad: Using metadata.is_symlink() which doesn't work on all platforms

// CRITICAL: Detect binary files by checking for null bytes
// Simple and reliable binary detection
// Good:
//   if content.iter().any(|&b| b == 0x00) {
//       return Err(JinError::BinaryFileNotSupported { path: ... });
//   }
// Bad: Using charset encoding or magic number detection

// CRITICAL: Check if file is tracked by Git in the project workspace
// Jin files must not be in Git (use git2::Repository::open() on workspace)
// Good:
//   let git_repo = git2::Repository::open(workspace_root)?;
//   let status = git_repo.status_file(path)?;
//   if status.is_tracked() {
//       return Err(JinError::GitTrackedFile { path: ... });
//   }
// Bad: Checking .gitignore manually (incomplete)

// CRITICAL: Use indexmap::IndexMap for jinmap mappings to maintain order
// Ensures consistent output
// Good:
//   pub mappings: IndexMap<String, Vec<String>>
// Bad: Using HashMap (non-deterministic order)

// GOTCHA: Git repo for Jin is at ~/.jin/repo (or $JIN_DIR/repo)
// Project workspace is separate from Jin's Git repo
// Good:
//   let jin_repo = JinRepo::open_or_create(jin_git_dir)?;
//   let workspace_git = git2::Repository::open(workspace_root)?;
// Bad: Using same repo path for both

// GOTCHA: TreeBuilder requires inserting with correct FileMode
// Use git2::FileMode::Blob for regular files
// Good:
//   builder.insert(path, blob_oid, git2::FileMode::Blob.into())?;
// Bad: Using git2::FileMode::Blob (without .into()) or wrong type

// PATTERN: Follow existing error.rs enum structure:
// - Group variants with comment dividers (// ===== ===== =====)
// - Use #[non_exhaustive] for public enums
// - Implement helper methods in impl block after enum definition
// - Add comprehensive doc comments

// PATTERN: Follow transaction.rs naming conventions:
// - Struct: PascalCase (CommitPipeline, not commit_pipeline)
// - Methods: snake_case (execute, not execute)
// - File name: snake_case (pipeline.rs, validate.rs, jinmap.rs, audit.rs)

// FUTURE: Audit log rotation and archival (not in this PRP)
// For this PRP, append to audit files without rotation
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
/// Pre-commit validation result.
///
/// Contains validation errors and warnings discovered during
/// the validation phase of the commit pipeline.
pub struct ValidationResult {
    /// Fatal errors that prevent commit
    pub errors: Vec<ValidationError>,
    /// Warnings that don't prevent commit
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error for a specific file.
#[derive(Debug)]
pub struct ValidationError {
    /// File path that failed validation
    pub path: PathBuf,
    /// Type of validation failure
    pub error_type: ValidationErrorType,
}

/// Jinmap file tracking layer-to-file mappings.
///
/// Auto-generated after each commit to track which files
/// belong to which layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jinmap {
    /// Format version
    pub version: u32,
    /// Layer path -> list of files mapping
    pub mappings: IndexMap<String, Vec<String>>,
    /// Metadata about generation
    pub meta: JinmapMeta,
}

/// Metadata section of .jinmap file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinmapMeta {
    /// Tool that generated this file
    pub generated_by: String,
    /// When this file was last updated
    pub last_updated: DateTime<Utc>,
}

/// Audit log entry for a commit.
///
/// Records all information about a commit for debugging
/// and recovery purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the commit was made
    pub timestamp: DateTime<Utc>,
    /// User who made the commit (from git config)
    pub user: String,
    /// Project name
    pub project: String,
    /// Active mode at commit time (if any)
    pub mode: Option<String>,
    /// Active scope at commit time (if any)
    pub scope: Option<String>,
    /// Layer number that was committed to (1-7)
    pub layer: u8,
    /// Files that were committed
    pub files: Vec<String>,
    /// Base commit OID (parent)
    pub base_commit: String,
    /// New commit OID
    pub merge_commit: String,
    /// Active context at commit time
    pub context: AuditContext,
}

/// Context information for audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    /// Active mode at commit time
    pub active_mode: Option<String>,
    /// Active scope at commit time
    pub active_scope: Option<String>,
}

/// Commit pipeline orchestrating the entire commit flow.
///
/// The pipeline processes staged entries through validation,
/// tree building, commit creation, and jinmap updates.
pub struct CommitPipeline<'a> {
    /// The Jin repository for layer operations
    repo: &'a JinRepo,
    /// Project workspace root
    workspace_root: &'a Path,
    /// Project name
    project: String,
}

/// Commit result containing all commit information.
pub struct CommitResult {
    /// Transaction ID for this commit
    pub transaction_id: String,
    /// Layer -> commit OID mapping
    pub commits: HashMap<Layer, git2::Oid>,
    /// Files that were committed
    pub files: Vec<PathBuf>,
    /// Updated jinmap
    pub jinmap: Jinmap,
    /// Audit entry
    pub audit_entry: AuditEntry,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commit/validate.rs
  - IMPLEMENT: ValidationError, ValidationErrorType, ValidationResult structs
  - IMPLEMENT: validate_staging_index() function
  - IMPLEMENT: validate_entry() helper function
  - IMPLEMENT: check_symlink(), check_binary_file(), check_git_tracked() helpers
  - IMPORTS:
    * use crate::core::error::{JinError, Result}
    * use crate::core::Layer
    * use crate::staging::{StagedEntry, StagingIndex}
    * use std::path::Path
  - NAMING: ValidationError, ValidationResult, validate_staging_index, validate_entry
  - VALIDATIONS:
    * Symlink detection -> JinError::SymlinkNotSupported
    * Binary file detection -> JinError::BinaryFileNotSupported
    * Git-tracked file -> JinError::GitTrackedFile
    * File size limit -> JinError::FileSizeLimit
  - PLACEMENT: New file src/commit/validate.rs
  - ERROR_TYPES: SymlinkNotSupported, BinaryFileNotSupported, GitTrackedFile, FileSizeLimit

Task 2: CREATE src/commit/jinmap.rs
  - IMPLEMENT: Jinmap, JinmapMeta structs with Serialize, Deserialize
  - IMPLEMENT: Jinmap::new(), Jinmap::add_mapping(), Jinmap::remove_mapping()
  - IMPLEMENT: Jinmap::load_from_disk(), Jinmap::save_to_disk()
  - IMPLEMENT: Jinmap::generate_from_layers() to scan all layer refs
  - IMPLEMENT: Jinmap::update_from_commit() to add/remove files after commit
  - IMPORTS:
    * use crate::core::{Layer, error::{JinError, Result}}
    * use crate::git::JinRepo
    * use indexmap::IndexMap
    * use serde::{Serialize, Deserialize}
    * use std::collections::HashMap
    * use std::path::{Path, PathBuf}
    * use chrono::{DateTime, Utc}
  - NAMING: Jinmap, JinmapMeta
  - FORMAT: version: 1, mappings: HashMap<String, Vec<String>>, meta: {generated_by, last_updated}
  - LAYER_PATH_FORMAT: Strip "refs/jin/layers/" prefix from ref names
  - PLACEMENT: New file src/commit/jinmap.rs
  - YAML_FORMAT:
    * version: 1
    * mappings: layer path -> list of files
    * meta.generated_by: "jin"
    * meta.last_updated: ISO 8601 timestamp

Task 3: CREATE src/commit/audit.rs
  - IMPLEMENT: AuditEntry, AuditContext structs with Serialize, Deserialize
  - IMPLEMENT: AuditEntry::new() constructor
  - IMPLEMENT: AuditEntry::save() to write to audit directory
  - IMPLEMENT: get_audit_dir() helper to get ~/.jin/repo/.audit/ path
  - IMPLEMENT: format_timestamp() helper for ISO 8601 output
  - IMPORTS:
    * use crate::core::{Layer, error::{JinError, Result}}
    * use crate::git::JinRepo
    * use serde::{Serialize, Deserialize}
    * use std::path::PathBuf
    * use chrono::{DateTime, Utc}
  - NAMING: AuditEntry, AuditContext
  - AUDIT_DIR: ~/.jin/repo/.audit/
  - FILE_FORMAT: One JSON entry per line (append-only)
  - PLACEMENT: New file src/commit/audit.rs
  - FIELDS: timestamp, user, project, mode (optional), scope (optional), layer (1-7), files, base_commit, merge_commit, context

Task 4: CREATE src/commit/pipeline.rs
  - IMPLEMENT: CommitPipeline struct with repo, workspace_root, project fields
  - IMPLEMENT: CommitPipeline::new() constructor
  - IMPLEMENT: CommitPipeline::execute() main orchestration method
  - IMPLEMENT: CommitPipeline::build_layer_trees() helper
  - IMPLEMENT: CommitPipeline::create_layer_commits() helper
  - IMPLEMENT: CommitPipeline::update_jinmap() helper
  - IMPLEMENT: CommitPipeline::log_audit_entry() helper
  - IMPLEMENT: CommitResult struct with commits, files, jinmap, audit_entry
  - IMPORTS:
    * use crate::commit::{validate::validate_staging_index, jinmap::Jinmap, audit::AuditEntry}
    * use crate::core::{Layer, error::{JinError, Result}}
    * use crate::git::{JinRepo, Transaction, TransactionManager}
    * use crate::staging::StagingIndex
    * use std::collections::HashMap
    * use std::path::{Path, PathBuf}
  - NAMING: CommitPipeline, CommitResult
  - ORCHESTRATION: validate -> build trees -> create commits -> update jinmap -> log audit
  - TRANSACTION: Use existing Transaction from src/git/transaction.rs
  - PLACEMENT: New file src/commit/pipeline.rs
  - DEPENDENCIES: Tasks 1, 2, 3 must be complete

Task 5: MODIFY src/commit/mod.rs
  - ADD: pub mod validate; pub mod jinmap; pub mod pipeline; pub mod audit;
  - ADD: pub use validate::{ValidationError, ValidationResult, validate_staging_index};
  - ADD: pub use jinmap::{Jinmap, JinmapMeta};
  - ADD: pub use pipeline::{CommitPipeline, CommitResult};
  - ADD: pub use audit::{AuditEntry, AuditContext};
  - PRESERVE: Any existing comments or structure
  - FINAL FILE:
    pub mod audit;
    pub mod jinmap;
    pub mod pipeline;
    pub mod validate;

    pub use audit::{AuditContext, AuditEntry};
    pub use jinmap::{Jinmap, JinmapMeta};
    pub use pipeline::{CommitPipeline, CommitResult};
    pub use validate::{ValidationError, ValidationResult, validate_staging_index};
  - PLACEMENT: src/commit/mod.rs
  - DEPENDENCIES: Tasks 1, 2, 3, 4 (files must exist)

Task 6: CREATE tests/commit/validate_test.rs
  - IMPLEMENT: Unit tests for validation functions
  - TESTS:
    * test_validate_entry_symlink() - verify symlink rejection
    * test_validate_entry_binary_file() - verify binary file rejection
    * test_validate_entry_git_tracked() - verify git-tracked file rejection
    * test_validate_staging_index_empty() - verify empty index passes
    * test_validate_staging_index_with_errors() - verify errors accumulate
  - FIXTURE: Create temp files with symlink, binary content, etc.
  - USE: tempfile crate (already in Cargo.toml)
  - PLACEMENT: tests/commit/validate_test.rs (create tests/commit/ first)
  - DEPENDENCIES: Task 1

Task 7: CREATE tests/commit/jinmap_test.rs
  - IMPLEMENT: Unit tests for Jinmap
  - TESTS:
    * test_jinmap_new() - verify new jinmap structure
    * test_jinmap_add_mapping() - verify adding files to layer
    * test_jinmap_remove_mapping() - verify removing files from layer
    * test_jinmap_persistence() - verify save/load roundtrip
    * test_jinmap_generate_from_layers() - verify generation from Git refs
  - FOLLOW: Pattern from repo.rs tests (TestFixture)
  - USE: tempfile for temp directories
  - PLACEMENT: tests/commit/jinmap_test.rs
  - DEPENDENCIES: Task 2

Task 8: CREATE tests/commit/pipeline_test.rs
  - IMPLEMENT: Unit tests for CommitPipeline
  - FIXTURE: TestFixture with temp Jin repo and workspace
  - TESTS:
    * test_pipeline_execute_empty() - verify empty staging handles correctly
    * test_pipeline_execute_single_file() - verify single file commit
    * test_pipeline_execute_multiple_files_same_layer() - verify same layer batching
    * test_pipeline_execute_multiple_layers() - verify multi-layer atomicity
    * test_pipeline_execute_validation_failure() - verify validation aborts commit
    * test_pipeline_execute_updates_jinmap() - verify .jinmap is updated
    * test_pipeline_execute_creates_audit() - verify audit log entry
  - FOLLOW: Pattern from transaction.rs tests (TestFixture)
  - USE: tempfile for temp directories
  - PLACEMENT: tests/commit/pipeline_test.rs
  - DEPENDENCIES: Task 4

Task 9: CREATE tests/commit/audit_test.rs
  - IMPLEMENT: Unit tests for audit logging
  - TESTS:
    * test_audit_entry_new() - verify audit entry creation
    * test_audit_entry_save() - verify audit file writing
    * test_get_audit_dir() - verify audit directory path
    * test_format_timestamp() - verify ISO 8601 formatting
  - FOLLOW: Pattern from repo.rs tests (TestFixture)
  - USE: tempfile for temp directories
  - PLACEMENT: tests/commit/audit_test.rs
  - DEPENDENCIES: Task 3

Task 10: UPDATE src/lib.rs if needed
  - VERIFY: commit module is re-exported
  - ADD: pub use commit::*; if not present
  - PLACEMENT: src/lib.rs
  - DEPENDENCIES: Task 5
```

### Implementation Patterns & Key Details

```rust
// ===== VALIDATION PATTERNS =====

/// Validates all entries in the staging index.
///
/// Returns ValidationResult containing errors (fatal) and warnings.
/// Only returns Err if validation itself fails (not for validation errors).
pub fn validate_staging_index(
    staging: &StagingIndex,
    workspace_root: &Path,
) -> Result<ValidationResult> {
    let mut result = ValidationResult {
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    for entry in staging.all_entries() {
        if let Err(e) = validate_entry(entry, workspace_root, &mut result) {
            // Validation process failed (not a validation error)
            return Err(e);
        }
    }

    Ok(result)
}

/// Validates a single staged entry.
///
/// Checks for symlinks, binary files, git-tracked files, and size limits.
fn validate_entry(
    entry: &StagedEntry,
    workspace_root: &Path,
    result: &mut ValidationResult,
) -> Result<()> {
    let full_path = workspace_root.join(&entry.path);

    // Check for symlink
    if let Err(e) = check_symlink(&full_path, &entry.path) {
        result.errors.push(ValidationError {
            path: entry.path.clone(),
            error_type: ValidationErrorType::SymlinkNotSupported,
        });
        return Ok(()); // Continue checking other files
    }

    // Check for binary file
    if let Err(e) = check_binary_file(&full_path, &entry.path) {
        result.errors.push(ValidationError {
            path: entry.path.clone(),
            error_type: ValidationErrorType::BinaryFileNotSupported,
        });
        return Ok(());
    }

    // Check if git-tracked
    if let Err(e) = check_git_tracked(workspace_root, &entry.path) {
        result.errors.push(ValidationError {
            path: entry.path.clone(),
            error_type: ValidationErrorType::GitTrackedFile,
        });
        return Ok(());
    }

    // Check file size
    if let Err(e) = check_file_size(&full_path, &entry.path, 10_000_000) {
        result.errors.push(ValidationError {
            path: entry.path.clone(),
            error_type: ValidationErrorType::FileSizeLimit,
        });
    }

    Ok(())
}

/// Checks if a path is a symlink.
fn check_symlink(path: &Path, relative_path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|e| JinError::FileNotFound {
            path: relative_path.display().to_string(),
        })?;

    if metadata.file_type().is_symlink() {
        return Err(JinError::SymlinkNotSupported {
            path: relative_path.display().to_string(),
        });
    }

    Ok(())
}

/// Checks if a file contains binary content.
fn check_binary_file(path: &Path, relative_path: &Path) -> Result<()> {
    let content = std::fs::read(path)
        .map_err(|e| JinError::Io {
            path: relative_path.display().to_string(),
            source: e.kind(),
        })?;

    // Check for null bytes (simple binary detection)
    if content.iter().any(|&b| b == 0x00) {
        return Err(JinError::BinaryFileNotSupported {
            path: relative_path.display().to_string(),
        });
    }

    Ok(())
}

/// Checks if a file is tracked by Git in the workspace.
fn check_git_tracked(workspace_root: &Path, relative_path: &Path) -> Result<()> {
    // Open the workspace's Git repo (not Jin's repo)
    let git_repo = git2::Repository::open(workspace_root)
        .map_err(|_| JinError::Message(
            "Not a Git repository".to_string()
        ))?;

    // Check file status
    let status = git_repo.status_file(relative_path)
        .map_err(|_| JinError::Message(
            format!("Failed to check Git status for {}", relative_path.display())
        ))?;

    if status.is_tracked() {
        return Err(JinError::GitTrackedFile {
            path: relative_path.display().to_string(),
        });
    }

    Ok(())
}

// ===== JINMAP PATTERNS =====

impl Jinmap {
    /// Creates a new empty jinmap.
    pub fn new() -> Self {
        Self {
            version: 1,
            mappings: IndexMap::new(),
            meta: JinmapMeta {
                generated_by: "jin".to_string(),
                last_updated: Utc::now(),
            },
        }
    }

    /// Adds or updates a file mapping for a layer.
    pub fn add_mapping(&mut self, layer_path: &str, file_path: &str) {
        self.mappings
            .entry(layer_path.to_string())
            .or_insert_with(Vec::new)
            .push(file_path.to_string());

        // Remove duplicates
        if let Some(files) = self.mappings.get_mut(layer_path) {
            files.sort();
            files.dedup();
        }

        self.meta.last_updated = Utc::now();
    }

    /// Removes a file from all layer mappings.
    pub fn remove_mapping(&mut self, file_path: &str) {
        for files in self.mappings.values_mut() {
            files.retain(|f| f != file_path);
        }

        // Remove empty layer entries
        self.mappings.retain(|_, files| !files.is_empty());

        self.meta.last_updated = Utc::now();
    }

    /// Loads jinmap from project root.
    pub fn load_from_disk(workspace_root: &Path) -> Result<Self> {
        let jinmap_path = workspace_root.join(".jinmap");

        if !jinmap_path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(&jinmap_path)
            .map_err(|e| JinError::Io {
                path: jinmap_path.display().to_string(),
                source: e.kind(),
            })?;

        serde_yaml::from_str(&content)
            .map_err(|e| JinError::Message(format!("Invalid .jinmap format: {}", e)))
    }

    /// Saves jinmap to project root.
    pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()> {
        let jinmap_path = workspace_root.join(".jinmap");

        let yaml = serde_yaml::to_string(self)
            .map_err(|e| JinError::Message(format!("Failed to serialize .jinmap: {}", e)))?;

        std::fs::write(&jinmap_path, yaml)
            .map_err(|e| JinError::Io {
                path: jinmap_path.display().to_string(),
                source: e.kind(),
            })
    }

    /// Generates jinmap by scanning all layer refs in Jin repo.
    pub fn generate_from_layers(repo: &JinRepo, project: &str) -> Result<Self> {
        let mut jinmap = Self::new();

        // Get all layer refs
        let layer_refs = repo.list_layer_refs()?;

        for (layer, commit_oid) in layer_refs {
            // Get the commit's tree
            let commit = repo.find_commit(commit_oid)?;
            let tree_id = commit.tree_id();
            let tree = repo.find_tree(tree_id)?;

            // Walk the tree and collect files
            let mut files = Vec::new();
            repo.walk_tree(tree_id, |path, _entry| {
                files.push(path.to_string());
                Ok(())
            })?;

            // Get layer path (strip refs/jin/layers/ prefix)
            let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;
            let layer_path = ref_name.strip_prefix("refs/jin/layers/")
                .ok_or_else(|| JinError::Message(
                    format!("Invalid layer ref format: {}", ref_name)
                ))?;

            // Add mapping
            if !files.is_empty() {
                jinmap.mappings.insert(layer_path.to_string(), files);
            }
        }

        jinmap.meta.last_updated = Utc::now();
        Ok(jinmap)
    }
}

// ===== AUDIT PATTERNS =====

impl AuditEntry {
    /// Creates a new audit entry from commit information.
    pub fn new(
        user: String,
        project: String,
        mode: Option<String>,
        scope: Option<String>,
        layer: Layer,
        files: Vec<String>,
        base_commit: git2::Oid,
        merge_commit: git2::Oid,
        active_mode: Option<String>,
        active_scope: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            user,
            project,
            mode,
            scope,
            layer: layer.layer_number(),
            files,
            base_commit: base_commit.to_string(),
            merge_commit: merge_commit.to_string(),
            context: AuditContext {
                active_mode,
                active_scope,
            },
        }
    }

    /// Saves this audit entry to the audit log.
    pub fn save(&self, jin_repo_path: &Path) -> Result<()> {
        let audit_dir = jin_repo_path.join(".audit");
        std::fs::create_dir_all(&audit_dir)
            .map_err(|e| JinError::Io {
                path: audit_dir.display().to_string(),
                source: e.kind(),
            })?;

        // Append to audit log (one JSON per line)
        let audit_file = audit_dir.format(&self.timestamp.format("%Y-%m-%d.log"))?;
        let line = serde_json::to_string(self)
            .map_err(|e| JinError::Message(format!("Failed to serialize audit entry: {}", e)))?;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(audit_file)
            .map_err(|e| JinError::Io {
                path: audit_file.display().to_string(),
                source: e.kind(),
            })?;

        use std::io::Write;
        writeln!(file, "{}", line)
            .map_err(|e| JinError::Io {
                path: audit_file.display().to_string(),
                source: e.kind(),
            })?;

        Ok(())
    }
}

// ===== COMMIT PIPELINE PATTERNS =====

impl<'a> CommitPipeline<'a> {
    /// Creates a new commit pipeline.
    pub fn new(repo: &'a JinRepo, workspace_root: &'a Path, project: String) -> Self {
        Self {
            repo,
            workspace_root,
            project,
        }
    }

    /// Executes the commit pipeline for a staging index.
    ///
    /// This is the main orchestration method that:
    /// 1. Validates all staged entries
    /// 2. Builds Git trees for each affected layer
    /// 3. Creates commits for each layer
    /// 4. Updates all layer refs atomically via Transaction
    /// 5. Updates .jinmap file
    /// 6. Logs audit entry
    pub fn execute(&self, staging: &mut StagingIndex) -> Result<CommitResult> {
        // Step 1: Validate
        let validation = validate_staging_index(staging, self.workspace_root)?;
        if !validation.errors.is_empty() {
            return Err(JinError::ValidationFailed {
                errors: validation.errors,
            });
        }

        // Step 2: Build trees for each layer
        let layer_updates = self.build_layer_trees(staging)?;

        // Step 3: Create commits for each layer
        let layer_commits = self.create_layer_commits(&layer_updates)?;

        // Step 4: Update refs atomically via Transaction
        let tm = TransactionManager::new(self.repo);
        let mut tx = tm.begin_transaction()?;

        for (layer, commit_oid) in &layer_commits {
            tx.add_layer_update(layer.clone(), *commit_oid)?;
        }

        tx.prepare()?;
        tx.commit()?;

        // Step 5: Update .jinmap
        let mut jinmap = Jinmap::load_from_disk(self.workspace_root)?;
        for entry in staging.all_entries() {
            let layer_path = entry.layer.git_ref()
                .ok_or_else(|| JinError::InvalidLayer {
                    name: format!("{:?}", entry.layer),
                })?
                .strip_prefix("refs/jin/layers/")
                .unwrap()
                .to_string();

            jinmap.add_mapping(&layer_path, &entry.path.to_string());
        }
        jinmap.save_to_disk(self.workspace_root)?;

        // Step 6: Log audit
        let audit_entry = self.create_audit_entry(&layer_commits, staging)?;
        audit_entry.save(self.repo.path())?;

        // Clear staging after successful commit
        staging.clear();

        Ok(CommitResult {
            transaction_id: tx.id().to_string(),
            commits: layer_commits,
            files: staging.all_entries().iter().map(|e| e.path.clone()).collect(),
            jinmap,
            audit_entry,
        })
    }

    /// Builds Git trees for all affected layers.
    fn build_layer_trees(&self, staging: &StagingIndex) -> Result<HashMap<Layer, git2::Oid>> {
        let mut layer_updates = HashMap::new();

        // Get unique layers from staging
        let mut layers = Vec::new();
        for entry in staging.all_entries() {
            if !layers.contains(&entry.layer) {
                layers.push(entry.layer.clone());
            }
        }

        // Build tree for each layer
        for layer in layers {
            let entries = staging.entries_by_layer(&layer);

            if entries.is_empty() {
                continue;
            }

            // Build tree from entries
            let mut builder = self.repo.treebuilder()?;

            for entry in entries {
                // Read file content
                let full_path = self.workspace_root.join(&entry.path);
                let content = std::fs::read(&full_path)
                    .map_err(|e| JinError::Io {
                        path: full_path.display().to_string(),
                        source: e.kind(),
                    })?;

                // Create blob
                let blob_oid = self.repo.create_blob(&content)?;

                // Insert into tree (use forward slashes)
                let tree_path = entry.path.to_str()
                    .ok_or_else(|| JinError::Message(
                        "Invalid path encoding".to_string()
                    ))?
                    .replace("\\", "/");

                builder.insert(&tree_path, blob_oid, git2::FileMode::Blob.into())?;
            }

            let tree_oid = self.repo.create_tree(&mut builder)?;
            layer_updates.insert(layer, tree_oid);
        }

        Ok(layer_updates)
    }

    /// Creates commits for each layer tree.
    fn create_layer_commits(
        &self,
        layer_updates: &HashMap<Layer, git2::Oid>,
    ) -> Result<HashMap<Layer, git2::Oid>> {
        let mut commits = HashMap::new();

        // Get signature for commits
        let signature = self.repo.signature("Jin", "jin@local")?;

        for (layer, tree_oid) in layer_updates {
            // Find the tree
            let tree = self.repo.find_tree(*tree_oid)?;

            // Get parent commit (if layer exists)
            let parent = self.repo.get_layer_ref(layer)?
                .and_then(|r| r.target())
                .and_then(|oid| self.repo.find_commit(oid).ok());

            let parents = if let Some(ref p) = parent {
                std::slice::from_ref(p)
            } else {
                &[][..]
            };

            // Create commit message
            let message = format!("Jin commit to layer: {:?}", layer);

            // Create commit
            let commit_oid = self.repo.create_commit(
                None, // Don't update HEAD
                &signature,
                &signature,
                &message,
                &tree,
                parents,
            )?;

            commits.insert(layer.clone(), commit_oid);
        }

        Ok(commits)
    }

    /// Creates an audit entry for this commit.
    fn create_audit_entry(
        &self,
        layer_commits: &HashMap<Layer, git2::Oid>,
        staging: &StagingIndex,
    ) -> Result<AuditEntry> {
        // Get first layer and commit for audit entry
        let (layer, commit_oid) = layer_commits.iter().next()
            .ok_or_else(|| JinError::Message("No commits created".to_string()))?;

        // Get parent commit
        let parent_oid = self.repo.get_layer_ref(layer)?
            .and_then(|r| r.target())
            .ok_or_else(|| JinError::Message("No parent commit".to_string()))?;

        // Collect files
        let files: Vec<String> = staging.all_entries()
            .iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect();

        // Get user from git config
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(AuditEntry::new(
            user,
            self.project.clone(),
            None, // mode - read from context in future
            None, // scope - read from context in future
            layer.clone(),
            files,
            parent_oid,
            *commit_oid,
            None, // active_mode - read from context in future
            None, // active_scope - read from context in future
        ))
    }
}

// ===== LAYER HELPER =====

impl Layer {
    /// Returns the layer number (1-7) for audit logging.
    fn layer_number(&self) -> u8 {
        match self {
            Layer::GlobalBase => 1,
            Layer::ModeBase { .. } => 2,
            Layer::ModeScope { .. } => 3,
            Layer::ModeScopeProject { .. } => 4,
            Layer::ModeProject { .. } => 5,
            Layer::ScopeBase { .. } => 6,
            Layer::ProjectBase { .. } => 7,
            Layer::UserLocal => 8,     // Not versioned
            Layer::WorkspaceActive => 9, // Not versioned
        }
    }
}
```

### Integration Points

```yaml
VALIDATION:
  - use: src/commit/validate.rs
  - integration: Called first in CommitPipeline::execute()
  - abort_on: ValidationError accumulates, aborts commit if any errors

STAGING:
  - use: src/staging/index.rs, src/staging/entry.rs
  - methods:
    * StagingIndex.all_entries() - get all staged entries
    * StagingIndex.entries_by_layer() - group by layer
    * StagingIndex.clear() - clear after successful commit
  - integration: Passed to CommitPipeline::execute()

TRANSACTION:
  - use: src/git/transaction.rs
  - methods:
    * TransactionManager::begin_transaction() - start transaction
    * Transaction::add_layer_update() - add layer update
    * Transaction::prepare() - lock refs
    * Transaction::commit() - atomic update
  - integration: Used for atomic multi-ref updates

JINREPO:
  - use: src/git/repo.rs
  - methods:
    * JinRepo::treebuilder() - create tree builder
    * JinRepo::create_blob() - create file blob
    * JinRepo::create_tree() - write tree
    * JinRepo::create_commit() - create commit
    * JinRepo::get_layer_ref() - get current layer ref
    * JinRepo::list_layer_refs() - list all layer refs
    * JinRepo::walk_tree() - walk tree for jinmap generation
  - integration: All Git operations go through JinRepo

JINMAP:
  - use: src/commit/jinmap.rs
  - file: .jinmap at workspace root
  - integration: Updated after successful commit

AUDIT:
  - use: src/commit/audit.rs
  - directory: ~/.jin/repo/.audit/
  - integration: Audit entry logged after successful commit

ERROR_HANDLING:
  - use: src/core/error.rs
  - variants:
    * JinError::SymlinkNotSupported
    * JinError::BinaryFileNotSupported
    * JinError::GitTrackedFile
    * JinError::FileSizeLimit
    * JinError::ValidationFailed
    * JinError::Io, JinError::Message (for general errors)

MODULE_EXPORTS:
  - modify: src/commit/mod.rs
  - add: pub mod validate, jinmap, pipeline, audit
  - add: pub use for all public types
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating each commit file - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "unused_imports" -> remove unused imports
# - "dead_code" -> public methods are used by tests, mark pub
# - "cannot find Transaction" -> verify use crate::git::Transaction
# - "cannot find StagingIndex" -> verify use crate::staging::StagingIndex
# - Pattern matching errors -> ensure all Layer variants handled
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test commit module specifically
cargo test --package jin --lib commit --verbose

# Run specific test files
cargo test --package jin --lib commit::validate --verbose
cargo test --package jin --lib commit::jinmap --verbose
cargo test --package jin --lib commit::pipeline --verbose
cargo test --package jin --lib commit::audit --verbose

# Run with output
cargo test --package jin --lib commit::validate -- --nocapture

# Expected: All tests pass. Look for:
# - test_validate_entry_symlink: symlink rejected
# - test_validate_entry_binary_file: binary file rejected
# - test_jinmap_add_mapping: mapping added correctly
# - test_pipeline_execute_single_file: single file committed
# - test_audit_entry_save: audit log entry written
```

### Level 3: Integration Testing (System Validation)

```bash
# Test commit pipeline end-to-end
cd /tmp
mkdir test_jin_commit
cd test_jin_commit

# Initialize Git repo
git init
echo "test" > test.txt

# Run integration test
cargo test --package jin test_commit_pipeline_integration -- --exact

# Verify:
# 1. StagingIndex can be created with entries
# 2. Validation rejects symlinks, binary files, git-tracked files
# 3. CommitPipeline::execute() creates Git commits
# 4. Layer refs are updated atomically
# 5. .jinmap is updated with new entries
# 6. Audit log entry is created

# Expected:
# - Validation catches invalid files
# - Commits are created for each affected layer
# - Transaction commits atomically
# - .jinmap contains correct mappings
# - Audit log has entry for commit
```

### Level 4: Domain-Specific Validation

```bash
# Verify validation rules match PRD requirements
cargo test --package jin test_prd_validation_rules -- --exact
# Asserts: Symlinks, binary files, git-tracked files all rejected

# Verify jinmap format matches PRD specification
cargo test --package jin test_jinmap_format -- --exact
# Asserts: YAML has version 1, mappings, meta sections

# Verify audit log format matches PRD specification
cargo test --package jin test_audit_log_format -- --exact
# Asserts: JSON has all required fields

# Verify transaction atomicity
cargo test --package jin test_transaction_atomicity -- --exact
# Asserts: All refs updated or none updated

# Expected: All Jin-specific requirements met
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all public methods
- [ ] All structs have doc comments

### Feature Validation

- [ ] `validate_staging_index()` validates all entries correctly
- [ ] Symlink detection works with `std::fs::symlink_metadata()`
- [ ] Binary file detection via null byte checking
- [ ] Git-tracked file detection via git2 status check
- [ ] `Jinmap` loads/saves in correct YAML format
- [ ] `Jinmap::generate_from_layers()` scans all layer refs
- [ ] `CommitPipeline::execute()` orchestrates full commit flow
- [ ] Transaction commits atomically via git2 Transaction
- [ ] `.jinmap` is updated after successful commit
- [ ] `AuditEntry` is logged to ~/.jin/repo/.audit/

### Code Quality Validation

- [ ] Follows existing codebase patterns (transaction.rs, repo.rs structure)
- [ ] File placement matches desired tree structure
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods
- [ ] Error messages are descriptive and actionable

### Documentation & Deployment

- [ ] Module-level doc comment explains commit system purpose
- [ ] Each struct has doc comment explaining Jin-specific semantics
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (layer path format, audit directory, etc.)

---

## Anti-Patterns to Avoid

- ❌ Don't skip validation - always validate before Git operations
- ❌ Don't update refs without Transaction - use existing Transaction type
- ❌ Don't hardcode layer ref paths - use Layer::git_ref()
- ❌ Don't use full ref paths in jinmap - strip "refs/jin/layers/" prefix
- ❌ Don't write audit to workspace - use ~/.jin/repo/.audit/
- ❌ Don't use SHA-1 for hashing - use SHA-256 (already in StagedEntry)
- ❌ Don't check .gitignore manually - use git2::Repository::status_file()
- ❌ Don't use HashMap for jinmap - use IndexMap for deterministic order
- ❌ Don't forget to clear staging after successful commit
- ❌ Don't skip audit logging - every commit must be logged

---

## Appendix: Quick Reference

### Commit Pipeline API Summary

```rust
// Validation
pub fn validate_staging_index(staging: &StagingIndex, workspace_root: &Path) -> Result<ValidationResult>

// Jinmap
impl Jinmap {
    pub fn new() -> Self
    pub fn add_mapping(&mut self, layer_path: &str, file_path: &str)
    pub fn remove_mapping(&mut self, file_path: &str)
    pub fn load_from_disk(workspace_root: &Path) -> Result<Self>
    pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()>
    pub fn generate_from_layers(repo: &JinRepo, project: &str) -> Result<Self>
}

// Audit
impl AuditEntry {
    pub fn new(...) -> Self
    pub fn save(&self, jin_repo_path: &Path) -> Result<()>
}

// Commit Pipeline
impl CommitPipeline {
    pub fn new(repo: &JinRepo, workspace_root: &Path, project: String) -> Self
    pub fn execute(&self, staging: &mut StagingIndex) -> Result<CommitResult>
}
```

### Jinmap Format

```yaml
version: 1
mappings:
  "global": ["config/global.json"]
  "mode/claude": [".claude/", "CLAUDE.md"]
  "project/myapp": ["project-config.json"]
meta:
  generated_by: "jin"
  last_updated: "2025-12-26T10:00:00Z"
```

### Audit Log Format

```json
{
  "timestamp": "2025-12-26T10:00:00Z",
  "user": "dustin",
  "project": "myapp",
  "mode": null,
  "scope": null,
  "layer": 7,
  "files": ["config.json"],
  "base_commit": "abc123...",
  "merge_commit": "def456...",
  "context": {
    "active_mode": null,
    "active_scope": null
  }
}
```

### Layer Numbers for Audit

| Layer | Number |
|-------|--------|
| GlobalBase | 1 |
| ModeBase | 2 |
| ModeScope | 3 |
| ModeScopeProject | 4 |
| ModeProject | 5 |
| ScopeBase | 6 |
| ProjectBase | 7 |
| UserLocal | 8 |
| WorkspaceActive | 9 |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
