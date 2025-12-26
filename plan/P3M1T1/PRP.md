# Product Requirement Prompt (PRP): Staging Implementation (P3.M1.T1)

---

## Goal

**Feature Goal**: Implement the staging system for Jin that tracks files to be committed to specific layers, providing a Git-like staging experience with layer-aware routing.

**Deliverable**: A `src/staging/` module with:
- `StagedEntry` struct representing a single staged file with layer and metadata
- `StagingIndex` struct managing the collection of staged entries
- Layer router that determines target layer based on CLI flags and context
- Persistence for staging state (JSON format for debugging, future binary format)
- Comprehensive unit tests with temp directories

**Success Definition**:
- `StagedEntry` struct compiles with all required fields
- `StagingIndex` can add, remove, query, and persist entries
- Layer routing correctly maps flag combinations to target layers
- `cargo test --package jin --lib staging` passes all tests
- No clippy warnings or rustc errors
- Module exported from `src/staging/mod.rs`

## User Persona

**Target User**: AI coding agent implementing Jin's staging system foundation

**Use Case**: The agent needs to create the staging infrastructure that:
- Tracks files to be committed to specific layers
- Routes files to appropriate layers based on CLI flags and active context
- Persists staging state across sessions
- Integrates with existing `Layer`, `JinError`, and `JinRepo` types

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/staging/entry.rs` with `StagedEntry` struct
3. Creates `src/staging/index.rs` with `StagingIndex` struct
4. Creates `src/staging/router.rs` with layer routing logic
5. Implements persistence for staging state
6. Adds comprehensive unit tests
7. Validates compilation and test success

**Pain Points Addressed**:
- No manual layer routing logic scattered across CLI commands
- Consistent staging state management
- Layer-aware file tracking from the start
- Integration with existing error handling and layer types

## Why

- **Foundation for commit workflow**: Every `jin add` and `jin commit` operation depends on staging
- **Layer routing logic**: Centralizes complex flag-to-layer mapping logic
- **State persistence**: Enables staging across multiple `jin add` calls before commit
- **Integration point**: Bridges CLI commands with Git layer operations
- **Problems this solves**:
  - Provides consistent staging state for multi-add workflows
  - Centralizes layer routing logic for maintainability
  - Enables status command to show staged files
  - Supports future `jin reset` operations

## What

Implement the staging system that tracks files to be committed to specific layers, with layer-aware routing and persistence.

### Success Criteria

- [ ] `src/staging/entry.rs` created with `StagedEntry` struct
- [ ] `src/staging/index.rs` created with `StagingIndex` struct
- [ ] `src/staging/router.rs` created with layer routing logic
- [ ] `StagedEntry` contains path, layer, content_hash, status, metadata
- [ ] `StagingIndex` can add, remove, and query entries by path and layer
- [ ] Layer router implements PRD §9.1 routing table correctly
- [ ] Staging state can be saved to and loaded from `.jin/staging/index.json`
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover all public methods with temp directories
- [ ] Module exported from `src/staging/mod.rs`
- [ ] All tests pass: `cargo test --package jin --lib staging`

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact `StagedEntry` and `StagingIndex` struct specifications with all fields
- Research documents with Git index format and Rust patterns
- Specific patterns from existing codebase to follow (Layer, JinError patterns)
- Complete integration guide with existing types
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Complete staging and commit specification with routing table
  section: Lines 202-240 for Layer Routing Table (§9.1)
  critical: "jin add <file>" → Project Base, "jin add --mode" → Mode Base, etc.

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  pattern: JinError::Io, JinError::FileNotFound, JinError::InvalidLayer
  gotcha: Use ? operator for automatic std::io::Error conversion

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum with from_flags() for routing logic
  section: Lines 282-378 for from_flags() implementation
  critical: Layer::from_flags() implements exact routing table from PRD

- file: /home/dustin/projects/jin-glm-doover/src/core/config.rs
  why: Context and Config types for active mode/scope
  section: Context structure for active mode/scope storage

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: JinRepo wrapper for future integration with layer refs
  section: Lines 170-343 for layer reference operations
  critical: set_layer_ref(), get_layer_ref() for future commit integration

- file: /home/dustin/projects/jin-glm-doover/src/staging/mod.rs
  why: Module must export staging types after creation
  pattern: Currently placeholder - needs to export entry, index, router

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify dependencies are available
  section: Lines 19-34 for dependencies
  critical: indexmap (2.7), serde (1.0), serde_json (1.0), uuid (1.0), sha2 (0.10)

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P3M1T1/research/git_staging_concepts.md
  why: Git index format and cache entry structure
  section: Complete index file format with stat info and object IDs
  critical: Shows Git's binary format for future reference

- docfile: /home/dustin/projects/jin-glm-doover/plan/P3M1T1/research/rust_staging_patterns.md
  why: Rust patterns for staging/index data structures
  section: IndexMap usage, bitflags for status, path handling
  critical: IndexMap for insertion order, bitflags for file status

# EXTERNAL - Git Documentation

- url: https://github.com/git/git/blob/master/Documentation/technical/index-format.txt
  why: Git index file format specification
  critical: Cache entry structure with stat info, OID, flags, path

- url: https://git-scm.com/docs/git-add
  why: Git add behavior reference for staging UX
  critical: Multiple files can be staged before commit

# EXTERNAL - Rust Crate Documentation

- url: https://docs.rs/indexmap/latest/indexmap/
  why: IndexMap for order-preserving map operations
  critical: Use IndexMap instead of HashMap for deterministic output

- url: https://docs.rs/serde/latest/serde/
  why: Serde serialization for staging persistence
  critical: Serialize, Deserialize traits for JSON format

- url: https://docs.rs/sha2/latest/sha2/
  why: SHA-256 hashing for content integrity
  critical: Sha256, Digest traits for content_hash computation

- url: https://docs.rs/bitflags/latest/bitflags/
  why: Bitflags for efficient file status representation
  critical: bitflags macro for FileStatus enum
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                      # Has all required dependencies
├── PRD.md                          # Staging and routing specification
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs                 # Exports error, layer, config
│   │   ├── error.rs               # Has Io, FileNotFound errors
│   │   ├── layer.rs               # Has Layer enum with from_flags()
│   │   └── config.rs              # Has Context and Config types
│   ├── git/
│   │   ├── mod.rs                 # Exports repo, transaction
│   │   ├── repo.rs                # Has JinRepo with layer ref operations
│   │   └── transaction.rs         # Transaction system (P1.M3.T1)
│   ├── merge/
│   │   ├── mod.rs                 # Exports value, text, layer
│   │   ├── value.rs               # MergeValue type (P2.M1.T1)
│   │   ├── text.rs                # 3-way text merge (P2.M4.T1)
│   │   └── layer.rs               # Layer merge orchestration
│   ├── staging/
│   │   └── mod.rs                 # Currently placeholder - needs implementation
│   ├── commit/
│   │   └── mod.rs                 # Commit pipeline (future integration)
│   ├── workspace/
│   │   └── mod.rs                 # Workspace operations
│   ├── cli/
│   │   └── mod.rs                 # CLI framework
│   └── commands/
│       └── mod.rs                 # Core commands
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── staging/
│       ├── mod.rs                 # MODIFY: Add pub mod entry, index, router; pub use ...
│       ├── entry.rs               # CREATE: StagedEntry and FileStatus
│       ├── index.rs               # CREATE: StagingIndex with CRUD operations
│       └── router.rs              # CREATE: Layer routing logic
└── tests/
    └── staging/
        ├── entry_test.rs          # CREATE: Unit tests for StagedEntry
        ├── index_test.rs          # CREATE: Unit tests for StagingIndex
        └── router_test.rs         # CREATE: Unit tests for layer router
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Use IndexMap, not HashMap, for StagingIndex.entries
// IndexMap preserves insertion order, required for deterministic output
use indexmap::IndexMap;
// Good: entries: IndexMap<PathBuf, StagedEntry>
// Bad: entries: HashMap<PathBuf, StagedEntry>

// CRITICAL: Layer routing uses Layer::from_flags() for consistency
// ALWAYS use Layer::from_flags() instead of manual matching
// Good:
//   let layer = Layer::from_flags(mode, scope, project, global)
//       .ok_or_else(|| JinError::Message("No routing target".to_string()))?;
// Bad:
//   if global { Layer::GlobalBase } else if mode { ... }

// CRITICAL: Path normalization before insertion into index
// Remove "." and ".." components, ensure consistent separators
// Good:
//   let normalized = path.components()
//       .filter(|c| !matches!(c, Component::CurDir | Component::ParentDir))
//       .collect::<PathBuf>();

// CRITICAL: Content hash uses SHA-256, not Git's SHA-1
// Jin uses SHA-256 for content hashing (more secure, future-proof)
use sha2::{Sha256, Digest};
// Good:
//   let hash = Sha256::digest(content);
//   entry.content_hash = hash.to_vec();
// Bad: Using SHA-1 from Git

// CRITICAL: Staging state stored in .jin/staging/index.json
// JSON format for debugging (future: binary format for production)
// Path relative to workspace root, not absolute paths
// Good:
//   let staging_path = workspace_root.join(".jin/staging/index.json");
// Bad: Hardcoded absolute paths

// CRITICAL: FileStatus uses bitflags for efficient state
// Multiple states can be combined (STAGED | MODIFIED)
use bitflags::bitflags;
// Good:
//   entry.status |= FileStatus::STAGED;
//   if entry.status.is_staged() { ... }

// GOTCHA: PathBuf in HashMap/IndexMap requires special handling
// PathBuf doesn't implement Hash directly in some cases
// Good: Use std::path::PathBuf which implements Hash
// Or use String keys if PathBuf causes issues

// GOTCHA: Serialization of PathBuf can be tricky
// Use serde's default for PathBuf, or convert to String
// Good: #[serde(as = "Option<String>")] for Option<PathBuf>

// PATTERN: Follow existing error.rs enum structure:
// - Group variants with comment dividers (// ===== ===== =====)
// - Use #[non_exhaustive] for public enums
// - Implement helper methods in impl block after enum definition
// - Add comprehensive doc comments

// PATTERN: Follow layer.rs naming conventions:
// - Struct: PascalCase (StagedEntry, not staged_entry)
// - Variants: PascalCase (Clean, Modified, not CLEAN, MODIFIED)
// - Methods: snake_case (add_entry, remove_entry, not addEntry, removeEntry)
// - File name: snake_case (entry.rs, index.rs, router.rs)

// GOTCHA: Layer::from_flags() returns Option<Layer>
// None means no flags provided (use default/project inference)
// Handle this case appropriately in router

// GOTCHA: Active context (mode/scope) stored in .jin/context
// For this PRP, focus on flag-based routing from Layer::from_flags()
// Future: Integrate with context for default behavior

// FUTURE: Binary format similar to Git index (P3.M2 or later)
// For this PRP, use JSON for simplicity and debuggability
// Future implementation will use custom binary format
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
/// File status flags for staged entries.
///
/// Uses bitflags for efficient status representation.
/// Multiple flags can be combined (e.g., STAGED | MODIFIED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileStatus(u8);

impl FileStatus {
    /// File has not been modified
    pub const CLEAN: Self = Self(0b00000001);
    /// File has been modified but not staged
    pub const MODIFIED: Self = Self(0b00000010);
    /// File is staged for commit
    pub const STAGED: Self = Self(0b00000100);
    /// File has been removed (staged deletion)
    pub const REMOVED: Self = Self(0b00001000);
    /// File is new (not in previous commit)
    pub const NEW: Self = Self(0b00010000);
}

/// A single staged file entry with layer and metadata.
///
/// Represents a file that has been staged for commit to a specific layer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StagedEntry {
    /// Path relative to workspace root
    pub path: PathBuf,
    /// Target layer for this entry
    pub layer: Layer,
    /// SHA-256 hash of file content
    pub content_hash: Vec<u8>,
    /// File status flags
    pub status: FileStatus,
    /// When file was staged (None for unstaged)
    pub staged_at: Option<SystemTime>,
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub modified_at: SystemTime,
}

/// Staging index managing all staged entries.
///
/// Provides CRUD operations for staged entries with layer-aware querying.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct StagingIndex {
    /// Primary index: path -> entry (ordered by insertion)
    #[serde(skip)]
    entries: IndexMap<PathBuf, StagedEntry>,
    /// Secondary index: layer -> paths (for layer-based queries)
    by_layer: HashMap<Layer, Vec<PathBuf>>,
}

/// Layer router for determining target layer from flags.
///
/// Implements PRD §9.1 routing table.
pub struct LayerRouter {
    /// Project context (for default/project inference)
    project: String,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/staging/entry.rs
  - IMPLEMENT: FileStatus bitflags enum
  - IMPLEMENT: StagedEntry struct with all fields
  - DERIVES: Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize for StagedEntry
  - IMPORTS:
    * use crate::core::Layer
    * use crate::core::error::{JinError, Result}
    * use std::path::PathBuf
    * use std::time::SystemTime
    * use bitflags::bitflags
    * use serde::{Serialize, Deserialize}
  - NAMING: FileStatus, StagedEntry (PascalCase)
  - PLACEMENT: New file src/staging/entry.rs

Task 2: CREATE src/staging/router.rs
  - IMPLEMENT: LayerRouter struct with project field
  - IMPLEMENT: LayerRouter::route() method using Layer::from_flags()
  - PATTERN: Delegate to Layer::from_flags() for consistency
  - CODE TEMPLATE:
    pub struct LayerRouter {
        project: String,
    }

    impl LayerRouter {
        pub fn new(project: String) -> Self {
            Self { project }
        }

        pub fn route(
            &self,
            mode: Option<&str>,
            scope: Option<&str>,
            project_flag: bool,
            global: bool,
        ) -> Result<Layer> {
            Layer::from_flags(mode, scope, Some(&self.project), global)
                .ok_or_else(|| JinError::Message(
                    "No routing target (use --mode, --scope, or --project)".to_string()
                ))
        }
    }
  - INTEGRATION: Uses existing Layer::from_flags() from src/core/layer.rs
  - PLACEMENT: New file src/staging/router.rs
  - DEPENDENCIES: Layer enum (already exists)

Task 3: IMPLEMENT StagedEntry helper methods
  - IMPLEMENT: impl StagedEntry with helper methods
  - METHODS:
    * pub fn new(path: PathBuf, layer: Layer, content: &[u8]) -> Result<Self>
    * pub fn is_staged(&self) -> bool
    * pub fn is_modified(&self) -> bool
    * pub fn is_removed(&self) -> bool
    * pub fn stage(&mut self)
    * pub fn unstage(&mut self)
  - CODE TEMPLATE:
    impl StagedEntry {
        pub fn new(path: PathBuf, layer: Layer, content: &[u8]) -> Result<Self> {
            use sha2::{Sha256, Digest};
            let metadata = std::fs::metadata(&path)?;
            let modified = metadata.modified()?;
            let hash = Sha256::digest(content);

            Ok(Self {
                path,
                layer,
                content_hash: hash.to_vec(),
                status: FileStatus::MODIFIED,
                staged_at: None,
                size: metadata.len(),
                modified_at: modified,
            })
        }

        pub fn is_staged(&self) -> bool {
            self.status.intersects(FileStatus::STAGED)
        }

        pub fn stage(&mut self) {
            self.status |= FileStatus::STAGED;
            self.staged_at = Some(SystemTime::now());
        }

        pub fn unstage(&mut self) {
            self.status.remove(FileStatus::STAGED);
            self.staged_at = None;
        }
    }
  - PLACEMENT: impl block in src/staging/entry.rs
  - DEPENDENCIES: Task 1

Task 4: CREATE src/staging/index.rs
  - IMPLEMENT: StagingIndex struct with entries and by_layer fields
  - DERIVES: Debug, Default, Serialize, Deserialize
  - IMPORTS:
    * use crate::core::{Layer, error::{JinError, Result}}
    * use crate::staging::entry::{StagedEntry, FileStatus}
    * use indexmap::IndexMap
    * use std::collections::HashMap
    * use std::path::{Path, PathBuf}
  - PLACEMENT: New file src/staging/index.rs
  - DEPENDENCIES: Task 1 (StagedEntry must exist)

Task 5: IMPLEMENT StagingIndex CRUD operations
  - IMPLEMENT: Core CRUD methods for StagingIndex
  - METHODS:
    * pub fn new() -> Self
    * pub fn add_entry(&mut self, entry: StagedEntry) -> Result<()>
    * pub fn remove_entry(&mut self, path: &Path) -> Option<StagedEntry>
    * pub fn get_entry(&self, path: &Path) -> Option<&StagedEntry>
    * pub fn get_entry_mut(&mut self, path: &Path) -> Option<&mut StagedEntry>
    * pub fn entries_by_layer(&self, layer: &Layer) -> Vec<&StagedEntry>
    * pub fn len(&self) -> usize
    * pub fn is_empty(&self) -> bool
  - CODE TEMPLATE:
    impl StagingIndex {
        pub fn add_entry(&mut self, entry: StagedEntry) -> Result<()> {
            let path = entry.path.clone();
            let layer = entry.layer.clone();

            // Remove from old layer if exists
            if let Some(old_entry) = self.entries.remove(&path) {
                self.remove_from_layer_index(&old_entry);
            }

            // Add to entries and layer index
            self.entries.insert(path.clone(), entry);
            self.by_layer.entry(layer).or_insert_with(Vec::new).push(path);

            Ok(())
        }

        pub fn entries_by_layer(&self, layer: &Layer) -> Vec<&StagedEntry> {
            self.by_layer
                .get(layer)
                .map(|paths| {
                    paths.iter()
                        .filter_map(|p| self.entries.get(p))
                        .collect()
                })
                .unwrap_or_default()
        }

        fn remove_from_layer_index(&mut self, entry: &StagedEntry) {
            if let Some(paths) = self.by_layer.get_mut(&entry.layer) {
                if let Some(pos) = paths.iter().position(|p| p == &entry.path) {
                    paths.remove(pos);
                }
                if paths.is_empty() {
                    self.by_layer.remove(&entry.layer);
                }
            }
        }
    }
  - PLACEMENT: impl StagingIndex block in src/staging/index.rs
  - DEPENDENCIES: Task 4

Task 6: IMPLEMENT StagingIndex persistence
  - IMPLEMENT: Save and load methods for StagingIndex
  - METHODS:
    * pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()>
    * pub fn load_from_disk(workspace_root: &Path) -> Result<Self>
  - PATTERN: JSON format for debugging, stored at .jin/staging/index.json
  - CODE TEMPLATE:
    impl StagingIndex {
        pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()> {
            let staging_dir = workspace_root.join(".jin/staging");
            std::fs::create_dir_all(&staging_dir)?;

            let index_file = staging_dir.join("index.json");
            let file = std::fs::File::create(&index_file)?;
            let writer = std::io::BufWriter::new(file);

            // Serialize to JSON
            serde_json::to_writer_pretty(writer, self)
                .map_err(|e| JinError::Message(format!("Failed to serialize index: {}", e)))?;

            Ok(())
        }

        pub fn load_from_disk(workspace_root: &Path) -> Result<Self> {
            let index_file = workspace_root.join(".jin/staging/index.json");

            if !index_file.exists() {
                return Ok(Self::default());
            }

            let file = std::fs::File::open(&index_file)?;
            let reader = std::io::BufReader::new(file);

            serde_json::from_reader(reader)
                .map_err(|e| JinError::Message(format!("Failed to deserialize index: {}", e)))
        }
    }
  - GOTCHA: Create .jin/staging directory if it doesn't exist
  - GOTCHA: Return empty index if file doesn't exist (not an error)
  - PLACEMENT: impl StagingIndex block in src/staging/index.rs
  - DEPENDENCIES: Task 5

Task 7: MODIFY src/staging/mod.rs
  - ADD: pub mod entry; pub mod index; pub mod router;
  - ADD: pub use entry::{FileStatus, StagedEntry};
  - ADD: pub use index::StagingIndex;
  - ADD: pub use router::LayerRouter;
  - PRESERVE: Any existing comments or structure
  - FINAL FILE:
    pub mod entry;
    pub mod index;
    pub mod router;

    pub use entry::{FileStatus, StagedEntry};
    pub use index::StagingIndex;
    pub use router::LayerRouter;
  - PLACEMENT: src/staging/mod.rs
  - DEPENDENCIES: Tasks 1, 2, 4 (files must exist)

Task 8: CREATE tests/staging/entry_test.rs
  - IMPLEMENT: Unit tests for StagedEntry
  - TESTS:
    * test_file_status_combinations() - verify bitflags work
    * test_staged_entry_new() - verify entry creation
    * test_staged_entry_stage_unstage() - verify status transitions
    * test_staged_entry_is_staged() - verify status checks
  - FIXTURE: Create temp file with known content for testing
  - USE: tempfile crate (already in Cargo.toml)
  - PLACEMENT: tests/staging/entry_test.rs (create tests/staging/ first)
  - DEPENDENCIES: Task 1, Task 3

Task 9: CREATE tests/staging/router_test.rs
  - IMPLEMENT: Unit tests for LayerRouter
  - TESTS:
    * test_router_global_flag() - verify global routing
    * test_router_mode_only() - verify mode routing
    * test_router_scope_only() - verify scope routing
    * test_router_mode_and_scope() - verify mode+scope routing
    * test_router_full_hierarchy() - verify mode+scope+project routing
    * test_router_no_flags_errors() - verify error without flags
    * test_router_project_inference() - verify project field usage
  - FOLLOW: Pattern from Layer::from_flags() tests in layer.rs
  - USE: Direct comparison with Layer variants
  - PLACEMENT: tests/staging/router_test.rs
  - DEPENDENCIES: Task 2

Task 10: CREATE tests/staging/index_test.rs
  - IMPLEMENT: Unit tests for StagingIndex
  - FIXTURE: TestFixture with temp directory and staging area
  - TESTS:
    * test_staging_index_new() - verify empty index
    * test_staging_index_add_entry() - verify add operation
    * test_staging_index_add_replace() - verify replacement on add
    * test_staging_index_remove_entry() - verify remove operation
    * test_staging_index_get_entry() - verify get operation
    * test_staging_index_entries_by_layer() - verify layer filtering
    * test_staging_index_multiple_layers() - verify entries across layers
    * test_staging_index_persistence() - verify save/load
    * test_staging_index_load_nonexistent() - verify returns empty
  - FOLLOW: Pattern from repo.rs tests (TestFixture)
  - USE: tempfile for temp directories
  - PLACEMENT: tests/staging/index_test.rs
  - DEPENDENCIES: Task 5, Task 6

Task 11: UPDATE src/lib.rs if needed
  - VERIFY: staging module is re-exported
  - ADD: pub use staging::*; if not present
  - PLACEMENT: src/lib.rs
  - DEPENDENCIES: Task 7
```

### Implementation Patterns & Key Details

```rust
// ===== FILE STATUS BITFLAGS PATTERN =====
// Use bitflags for efficient status representation
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FileStatus: u8 {
        const CLEAN = 0b00000001;
        const MODIFIED = 0b00000010;
        const STAGED = 0b00000100;
        const REMOVED = 0b00001000;
        const NEW = 0b00010000;
    }
}

// ===== STAGED ENTRY CREATION PATTERN =====
impl StagedEntry {
    pub fn new(path: PathBuf, layer: Layer, content: &[u8]) -> Result<Self> {
        use sha2::{Sha256, Digest};

        let metadata = std::fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let hash = Sha256::digest(content);

        Ok(Self {
            path,
            layer,
            content_hash: hash.to_vec(),
            status: FileStatus::MODIFIED,
            staged_at: None,
            size: metadata.len(),
            modified_at: modified,
        })
    }
}

// ===== LAYER ROUTER PATTERN =====
// Delegates to Layer::from_flags() for consistency
impl LayerRouter {
    pub fn route(
        &self,
        mode: Option<&str>,
        scope: Option<&str>,
        project_flag: bool,
        global: bool,
    ) -> Result<Layer> {
        Layer::from_flags(mode, scope, Some(&self.project), global)
            .ok_or_else(|| JinError::Message(
                "No routing target (use --mode, --scope, or --project)".to_string()
            ))
    }
}

// ===== STAGING INDEX CRUD PATTERN =====
impl StagingIndex {
    pub fn add_entry(&mut self, entry: StagedEntry) -> Result<()> {
        let path = entry.path.clone();
        let layer = entry.layer.clone();

        // Remove from old layer if exists
        if let Some(old_entry) = self.entries.remove(&path) {
            self.remove_from_layer_index(&old_entry);
        }

        // Add to entries and layer index
        self.entries.insert(path.clone(), entry);
        self.by_layer.entry(layer).or_insert_with(Vec::new).push(path);

        Ok(())
    }

    fn remove_from_layer_index(&mut self, entry: &StagedEntry) {
        if let Some(paths) = self.by_layer.get_mut(&entry.layer) {
            if let Some(pos) = paths.iter().position(|p| p == &entry.path) {
                paths.remove(pos);
            }
            if paths.is_empty() {
                self.by_layer.remove(&entry.layer);
            }
        }
    }
}

// ===== PERSISTENCE PATTERN =====
// JSON format for debugging (future: binary format)
impl StagingIndex {
    pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()> {
        let staging_dir = workspace_root.join(".jin/staging");
        std::fs::create_dir_all(&staging_dir)?;

        let index_file = staging_dir.join("index.json");
        let file = std::fs::File::create(&index_file)?;
        let writer = std::io::BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| JinError::Message(format!("Failed to serialize index: {}", e)))?;

        Ok(())
    }

    pub fn load_from_disk(workspace_root: &Path) -> Result<Self> {
        let index_file = workspace_root.join(".jin/staging/index.json");

        if !index_file.exists() {
            return Ok(Self::default());
        }

        let file = std::fs::File::open(&index_file)?;
        let reader = std::io::BufReader::new(file);

        serde_json::from_reader(reader)
            .map_err(|e| JinError::Message(format!("Failed to deserialize index: {}", e)))
    }
}
```

### Integration Points

```yaml
LAYER_SYSTEM:
  - use: src/core/layer.rs
  - types:
    * Layer enum (already exists)
    * Layer::from_flags() for routing (already exists)
  - integration: StagedEntry.layer field, LayerRouter.route()

ERROR_HANDLING:
  - use: src/core/error.rs
  - variants:
    * JinError::Io - automatic from std::io::Error
    * JinError::FileNotFound - for missing files
    * JinError::Message - for custom errors
    * JinError::InvalidLayer - from Layer operations

CONFIG_CONTEXT:
  - use: src/core/config.rs (future integration)
  - types:
    * Context struct with active mode/scope
  - future: Integrate with active context for default routing

MODULE_EXPORTS:
  - modify: src/staging/mod.rs
  - add: pub mod entry; pub mod index; pub mod router;
  - add: pub use entry::{FileStatus, StagedEntry};
  - add: pub use index::StagingIndex;
  - add: pub use router::LayerRouter;

TESTING:
  - create: tests/staging/ directory
  - use: tempfile crate (already in Cargo.toml)
  - pattern: TestFixture from repo.rs tests

FUTURE_INTEGRATION:
  - P3.M2: Commit Pipeline will use StagingIndex for staged files
  - P4.M2: Add command will use LayerRouter + StagingIndex
  - P4.M5: Reset command will use StagingIndex.remove_entry()
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating each staging file - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "unused_imports" -> remove unused imports
# - "dead_code" -> public methods are used by tests, mark pub
# - "cannot find IndexMap" -> verify use indexmap::IndexMap
# - Pattern matching errors -> ensure all FileStatus bits handled
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test staging module specifically
cargo test --package jin --lib staging --verbose

# Run specific test files
cargo test --package jin --lib staging::entry --verbose
cargo test --package jin --lib staging::index --verbose
cargo test --package jin --lib staging::router --verbose

# Run with output
cargo test --package jin --lib staging::entry -- --nocapture

# Expected: All tests pass. Look for:
# - test_file_status_combinations: bitflags work
# - test_staged_entry_new: entry creation works
# - test_router_global_flag: global routing correct
# - test_router_mode_only: mode routing correct
# - test_staging_index_add_entry: add works
# - test_staging_index_persistence: save/load works
```

### Level 3: Integration Testing (System Validation)

```bash
# Test staging state persistence
cd /tmp
mkdir test_jin_staging
cd test_jin_staging

# Run manual test Python script or use cargo test with output
cargo test --package jin test_staging_integration -- --exact

# Verify:
# 1. Create StagingIndex
# 2. Add entries with different layers
# 3. Save to disk
# 4. Load from disk
# 5. Verify entries match

# Test layer routing:
# 1. Create LayerRouter with project name
# 2. Test various flag combinations
# 3. Verify correct layer selection

# Expected:
# - Persistence works correctly
# - Layer routing matches PRD table
# - Entries can be added/removed/queried
```

### Level 4: Domain-Specific Validation

```bash
# Verify Layer routing matches PRD exactly
cargo test --package jin test_prd_routing_table -- --exact
# Asserts: All flag combinations route to correct layers

# Verify IndexMap insertion order preserved
cargo test --package jin test_insertion_order -- --exact
# Asserts: entries maintain insertion order (not sorted)

# Verify bitflags combinations work
cargo test --package jin test_bitflags_combinations -- --exact
# Asserts: STAGED | MODIFIED works correctly

# Verify persistence format is valid JSON
cargo test --package jin test_json_serialization -- --exact
# Asserts: saved JSON can be parsed and loaded

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

- [ ] `StagedEntry` struct with all required fields compiles
- [ ] `FileStatus` bitflags enum works correctly
- [ ] `StagingIndex` can add, remove, get, and query entries
- [ ] `StagingIndex.entries_by_layer()` filters correctly
- [ ] `LayerRouter.route()` implements PRD §9.1 routing table
- [ ] Staging state can be saved to `.jin/staging/index.json`
- [ ] Staging state can be loaded from disk (returns empty if missing)
- [ ] Integration with `Layer`, `JinError` types

### Code Quality Validation

- [ ] Follows existing codebase patterns (layer.rs, error.rs structure)
- [ ] File placement matches desired tree structure
- [ ] Module exported from `src/staging/mod.rs`
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods

### Documentation & Deployment

- [ ] Module-level doc comment explains staging system purpose
- [ ] Each struct has doc comment explaining Jin-specific semantics
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (routing, path normalization, persistence)

---

## Anti-Patterns to Avoid

- ❌ Don't use `HashMap` instead of `IndexMap` - order must be preserved
- ❌ Don't skip `Layer::from_flags()` for routing - use existing function
- ❌ Don't use absolute paths - store paths relative to workspace root
- ❌ Don't use SHA-1 for content hash - use SHA-256
- ❌ Don't skip path normalization - remove `.` and `..` components
- ❌ Don't skip creating `.jin/staging` directory - it may not exist
- ❌ Don't error on missing index file - return empty index (first add)
- ❌ Don't skip bitflags for FileStatus - enables efficient combinations
- ❌ Don't serialize entries field with IndexMap - use #[serde(skip)]
- ❌ Don't forget to update secondary index in add/remove operations

---

## Appendix: Quick Reference

### Staging API Summary

```rust
// StagedEntry
impl StagedEntry {
    pub fn new(path: PathBuf, layer: Layer, content: &[u8]) -> Result<Self>
    pub fn is_staged(&self) -> bool
    pub fn stage(&mut self)
    pub fn unstage(&mut self)
}

// StagingIndex
impl StagingIndex {
    pub fn new() -> Self
    pub fn add_entry(&mut self, entry: StagedEntry) -> Result<()>
    pub fn remove_entry(&mut self, path: &Path) -> Option<StagedEntry>
    pub fn get_entry(&self, path: &Path) -> Option<&StagedEntry>
    pub fn entries_by_layer(&self, layer: &Layer) -> Vec<&StagedEntry>
    pub fn save_to_disk(&self, workspace_root: &Path) -> Result<()>
    pub fn load_from_disk(workspace_root: &Path) -> Result<Self>
}

// LayerRouter
impl LayerRouter {
    pub fn new(project: String) -> Self
    pub fn route(&self, mode: Option<&str>, scope: Option<&str>, project_flag: bool, global: bool) -> Result<Layer>
}
```

### Routing Table Reference (PRD §9.1)

| Flags | Target Layer |
|-------|--------------|
| `--global` | GlobalBase |
| `--mode --scope --project` | ModeScopeProject |
| `--mode --project` | ModeProject |
| `--mode --scope` | ModeScope |
| `--scope` | ScopeBase |
| `--mode` | ModeBase |
| `--project` | ProjectBase |
| (none) | Error (no routing target) |

### Staging File Format

```json
{
  "entries": {
    ".claude/config.json": {
      "path": ".claude/config.json",
      "layer": "ProjectBase { project: \"myapp\" }",
      "content_hash": [ ... ],
      "status": 4,
      "staged_at": "2025-12-26T10:00:00Z",
      "size": 1024,
      "modified_at": "2025-12-26T10:00:00Z"
    }
  },
  "by_layer": {
    "ProjectBase { project: \"myapp\" }": [".claude/config.json"]
  }
}
```

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
