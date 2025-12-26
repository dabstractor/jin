# PRP: P1.M2 - Git Layer Integration

---

## Goal

**Feature Goal**: Implement the phantom Git layer using libgit2 (via git2-rs), providing Jin with the ability to manage its own Git repository for storing layer configurations without interfering with the user's primary Git workflow.

**Deliverable**: A complete `src/git/` module with:
1. `JinRepo` wrapper struct for managing Jin's dedicated Git repository at `~/.jin/`
2. Reference operations for creating, reading, updating, and deleting refs under `refs/jin/layers/*`
3. Git object creation (blobs, trees, commits) for storing layer file contents
4. Tree walking utilities for reading layer contents from commits

**Success Definition**:
- All git module tests pass: `cargo test git::`
- Can create/open Jin repository programmatically
- Can create refs under `refs/jin/layers/*` namespace
- Can create blobs, trees, and commits
- Can walk tree contents and extract file data
- All operations handle errors using `JinError::Git`

---

## User Persona

**Target User**: Jin CLI and internal subsystems requiring Git operations

**Use Case**: The git layer is used by:
- `jin init` to create/open the Jin repository
- `jin add/commit` to stage and commit files to layer refs
- `jin apply` to read layer contents and merge into workspace
- Transaction system for atomic multi-layer commits

**User Journey**: This is internal infrastructure - users interact via CLI commands that delegate to this layer.

**Pain Points Addressed**:
- Provides safe, isolated Git operations that never touch the user's `.git`
- Enables atomic multi-ref updates for transaction safety
- Abstracts libgit2 complexity behind idiomatic Rust APIs

---

## Why

- **Foundation for All Layer Operations**: Every Jin command that reads/writes layer data depends on this module
- **Isolation Guarantee**: Jin uses refs under `refs/jin/` namespace, invisible to normal Git commands
- **Atomic Commits**: Transaction support enables the PRD's atomic multi-layer commit requirement
- **Git-Native Storage**: Leverages Git's proven object model and garbage collection

---

## What

### User-Visible Behavior

After this milestone, the following internal operations work:

```rust
// Open or create Jin repository
let jin_repo = JinRepo::open_or_create()?;

// Create a blob from file content
let blob_oid = jin_repo.create_blob(b"file contents")?;

// Build a tree with entries
let tree_oid = jin_repo.create_tree(&[
    ("config.json", blob_oid, FileMode::Blob),
])?;

// Create a commit on a layer ref
let commit_oid = jin_repo.create_commit(
    "refs/jin/layers/mode/claude",
    "Add Claude mode configuration",
    tree_oid,
    &[],  // parents
)?;

// Read tree contents
jin_repo.walk_tree(tree_oid, |path, entry| {
    println!("{}: {}", path, entry.name());
    TreeWalkResult::Ok
})?;

// Update ref atomically
jin_repo.set_ref("refs/jin/layers/mode/claude", new_oid)?;
```

### Technical Requirements

1. **JinRepo**: Wrapper around `git2::Repository` with Jin-specific methods
2. **Ref Operations**: CRUD for refs under `refs/jin/` namespace
3. **Object Creation**: Blobs, trees, commits with proper signatures
4. **Tree Walking**: Pre-order/post-order traversal with callbacks

### Success Criteria

- [ ] `JinRepo::open()` opens existing Jin repo at `~/.jin/`
- [ ] `JinRepo::create()` initializes bare repo at `~/.jin/`
- [ ] `JinRepo::open_or_create()` handles both cases
- [ ] `create_blob()` writes content to object database
- [ ] `create_tree()` builds tree from entries
- [ ] `create_commit()` creates commit with message, tree, parents
- [ ] `find_ref()` locates refs by name
- [ ] `set_ref()` creates/updates refs atomically
- [ ] `delete_ref()` removes refs
- [ ] `list_refs()` enumerates refs matching pattern
- [ ] `walk_tree()` traverses tree entries
- [ ] `get_tree_entry()` retrieves specific entry by path
- [ ] All errors wrap into `JinError::Git`
- [ ] Unit tests cover all public methods

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement Jin's Git layer, including exact API signatures, code examples, and gotchas from libgit2/git2-rs documentation._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# git2-rs Official Documentation
- url: https://docs.rs/git2/latest/git2/struct.Repository.html
  why: Repository struct - init, open, commit, blob, treebuilder methods
  critical: |
    - Repository::init_bare() for bare repos
    - Repository::open() for existing repos
    - Repository::blob() for creating blobs
    - Repository::treebuilder() for building trees
    - Repository::commit() for creating commits
    - Repository::find_reference() for ref lookup
    - Repository::reference() for creating refs
    - Repository::references_glob() for pattern matching

- url: https://docs.rs/git2/latest/git2/struct.Reference.html
  why: Reference operations - create, read, update, delete
  critical: |
    - Reference::target() returns Oid
    - Reference::set_target() for atomic updates
    - Reference::delete() for removal
    - Reference::is_valid_name() for validation

- url: https://docs.rs/git2/latest/git2/struct.TreeBuilder.html
  why: Building trees programmatically
  critical: |
    - TreeBuilder::insert() adds entries
    - TreeBuilder::write() finalizes to Oid
    - Single-level only: must recursively build nested trees
    - File modes: 0o100644 (file), 0o040000 (dir)

- url: https://docs.rs/git2/latest/git2/struct.Tree.html
  why: Tree walking and entry access
  critical: |
    - Tree::walk() for traversal
    - Tree::get_path() for entry by path
    - Tree::iter() for iteration

- url: https://docs.rs/git2/latest/git2/struct.Transaction.html
  why: Atomic multi-ref updates (with limitations)
  critical: |
    - Transaction::lock_ref() before updates
    - Transaction::set_target() queues update
    - Transaction::commit() applies all
    - NOT truly atomic: partial failures don't rollback

# P1.M1 Dependencies (must follow these patterns)
- file: plan/P1M1/PRP.md
  why: Contains JinError, Layer, JinConfig types we must use
  pattern: |
    - JinError::Git wraps git2::Error
    - Layer::ref_path() generates refs/jin/layers/* paths
    - JinConfig::default_path() returns ~/.jin/

# Phantom Git Patterns Research
- file: plan/P1M2/research/phantom_git_patterns.md
  why: Best practices for custom ref namespaces
  critical: |
    - Use refs/jin/layers/* namespace (invisible to git branch)
    - Objects referenced by refs are GC-protected
    - Atomic updates via git update-ref --stdin --atomic
```

### Current Codebase Tree (after P1.M1)

```bash
jin/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── error.rs          # JinError with Git variant
│   │   ├── layer.rs          # Layer enum with ref_path()
│   │   └── config.rs         # JinConfig, ProjectContext
│   ├── git/                   # STUBS - to be implemented
│   │   ├── mod.rs
│   │   ├── repo.rs           # JinRepo stub
│   │   ├── refs.rs           # RefOps stub
│   │   ├── objects.rs        # Object creation stub
│   │   └── transaction.rs    # Transaction stub
│   ├── merge/
│   ├── staging/
│   ├── commit/
│   ├── cli/
│   └── commands/
└── tests/
```

### Desired Codebase Tree After P1.M2

```bash
jin/
├── src/
│   ├── git/
│   │   ├── mod.rs            # pub exports for JinRepo, RefOps, etc.
│   │   ├── repo.rs           # JinRepo wrapper implementation
│   │   │   └── JinRepo struct with:
│   │   │       - open() -> Result<JinRepo>
│   │   │       - create() -> Result<JinRepo>
│   │   │       - open_or_create() -> Result<JinRepo>
│   │   │       - inner() -> &Repository
│   │   │       - path() -> PathBuf
│   │   ├── refs.rs           # Reference operations
│   │   │   └── RefOps trait/impl with:
│   │   │       - find_ref(&str) -> Result<Reference>
│   │   │       - set_ref(&str, Oid, &str) -> Result<()>
│   │   │       - delete_ref(&str) -> Result<()>
│   │   │       - list_refs(&str) -> Result<Vec<Reference>>
│   │   │       - ref_exists(&str) -> bool
│   │   ├── objects.rs        # Object creation
│   │   │   └── ObjectOps trait/impl with:
│   │   │       - create_blob(&[u8]) -> Result<Oid>
│   │   │       - create_blob_from_path(Path) -> Result<Oid>
│   │   │       - create_tree(entries) -> Result<Oid>
│   │   │       - create_commit(...) -> Result<Oid>
│   │   │       - find_blob(Oid) -> Result<Blob>
│   │   │       - find_tree(Oid) -> Result<Tree>
│   │   │       - find_commit(Oid) -> Result<Commit>
│   │   ├── tree.rs           # Tree walking utilities (NEW)
│   │   │   └── TreeOps trait/impl with:
│   │   │       - walk_tree(Oid, callback) -> Result<()>
│   │   │       - get_tree_entry(Oid, path) -> Result<TreeEntry>
│   │   │       - read_blob_content(Oid) -> Result<Vec<u8>>
│   │   └── transaction.rs    # Transaction wrapper (stub for now)
│   │       └── JinTransaction with:
│   │           - new(repo) -> JinTransaction
│   │           - lock_ref(&str) -> Result<()>
│   │           - set_target(&str, Oid) -> Result<()>
│   │           - commit() -> Result<()>
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2 requires libgit2 system library or vendored build
// Use vendored-libgit2 feature in Cargo.toml to avoid system dependency
// git2 = { version = "0.19", features = ["vendored-libgit2"] }

// CRITICAL: TreeBuilder is single-level only
// To create nested directories, build inner trees first, then reference them
// in outer tree with mode 0o040000 (directory)

// GOTCHA: Repository::init_bare() requires path to exist
// Must create parent directories first with std::fs::create_dir_all()

// GOTCHA: Signature requires both name and email
// Use git config user.name / user.email or fallback to "jin" / "jin@local"

// GOTCHA: Reference names must be valid
// Use Reference::is_valid_name() before creating refs
// refs/jin/layers/* is valid custom namespace

// GOTCHA: Transaction::commit() is NOT truly atomic
// If one ref update fails, previous updates are NOT rolled back
// For true atomicity in critical paths, consider external locking

// GOTCHA: Tree walking callback receives (path, entry) not (entry, path)
// path is the parent directory, entry.name() is the filename

// PATTERN: Jin repository is BARE (no working directory)
// Located at ~/.jin/ by default
// Uses refs/jin/layers/* namespace exclusively

// PATTERN: Layer refs follow this structure:
// refs/jin/layers/global
// refs/jin/layers/mode/{mode}
// refs/jin/layers/mode/{mode}/scope/{scope}
// refs/jin/layers/mode/{mode}/scope/{scope}/project/{project}
// refs/jin/layers/mode/{mode}/project/{project}
// refs/jin/layers/scope/{scope}
// refs/jin/layers/project/{project}
// refs/jin/layers/local
// refs/jin/layers/workspace

// PATTERN: Error handling - wrap git2::Error into JinError::Git
// Use ? operator with From<git2::Error> implementation
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/git/mod.rs ==================
//! Git layer integration for Jin phantom repository.
//!
//! This module provides:
//! - JinRepo: Wrapper for Jin's dedicated bare Git repository
//! - Reference operations under refs/jin/layers/* namespace
//! - Object creation (blobs, trees, commits)
//! - Tree walking utilities

pub mod repo;
pub mod refs;
pub mod objects;
pub mod tree;
pub mod transaction;

pub use repo::JinRepo;
pub use refs::RefOps;
pub use objects::ObjectOps;
pub use tree::TreeOps;
pub use transaction::JinTransaction;

// Re-export git2 types commonly used
pub use git2::{Oid, ObjectType, FileMode, TreeWalkMode, TreeWalkResult};


// ================== src/git/repo.rs ==================
use std::path::PathBuf;
use git2::{Repository, RepositoryInitOptions};
use crate::core::{Result, JinError, JinConfig};

/// Wrapper around git2::Repository for Jin's phantom Git layer.
///
/// Jin maintains a bare repository at ~/.jin/ that stores all layer
/// configurations. This wrapper provides Jin-specific operations
/// while exposing the underlying Repository for advanced use cases.
pub struct JinRepo {
    repo: Repository,
    path: PathBuf,
}

impl JinRepo {
    /// Opens an existing Jin repository.
    ///
    /// # Errors
    /// Returns `JinError::Git` if the repository doesn't exist or is corrupted.
    pub fn open() -> Result<Self> {
        let path = Self::default_path()?;
        let repo = Repository::open_bare(&path)?;
        Ok(Self { repo, path })
    }

    /// Creates a new Jin repository.
    ///
    /// # Errors
    /// Returns `JinError::Git` if creation fails or repo already exists.
    pub fn create() -> Result<Self> {
        let path = Self::default_path()?;

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize bare repository with options
        let mut opts = RepositoryInitOptions::new();
        opts.bare(true);
        opts.mkdir(true);
        opts.description("Jin phantom layer repository");

        let repo = Repository::init_opts(&path, &opts)?;
        Ok(Self { repo, path })
    }

    /// Opens existing or creates new Jin repository.
    pub fn open_or_create() -> Result<Self> {
        match Self::open() {
            Ok(repo) => Ok(repo),
            Err(_) => Self::create(),
        }
    }

    /// Returns the default Jin repository path (~/.jin/).
    pub fn default_path() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join(".jin"))
            .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
    }

    /// Returns path to the Jin repository.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns reference to the underlying git2::Repository.
    ///
    /// Use this for advanced operations not covered by JinRepo methods.
    pub fn inner(&self) -> &Repository {
        &self.repo
    }

    /// Returns mutable reference to the underlying git2::Repository.
    pub fn inner_mut(&mut self) -> &mut Repository {
        &mut self.repo
    }
}


// ================== src/git/refs.rs ==================
use git2::{Reference, Oid, ReferenceType};
use crate::core::Result;
use super::JinRepo;

/// Reference operations for Jin's layer refs.
pub trait RefOps {
    /// Finds a reference by name.
    fn find_ref(&self, name: &str) -> Result<Reference>;

    /// Creates or updates a reference to point to an OID.
    fn set_ref(&self, name: &str, oid: Oid, message: &str) -> Result<()>;

    /// Deletes a reference.
    fn delete_ref(&self, name: &str) -> Result<()>;

    /// Lists references matching a glob pattern.
    fn list_refs(&self, pattern: &str) -> Result<Vec<String>>;

    /// Checks if a reference exists.
    fn ref_exists(&self, name: &str) -> bool;

    /// Gets the OID a reference points to (resolving symbolic refs).
    fn resolve_ref(&self, name: &str) -> Result<Oid>;
}

impl RefOps for JinRepo {
    fn find_ref(&self, name: &str) -> Result<Reference> {
        Ok(self.repo.find_reference(name)?)
    }

    fn set_ref(&self, name: &str, oid: Oid, message: &str) -> Result<()> {
        // Validate reference name
        if !Reference::is_valid_name(name) {
            return Err(crate::core::JinError::InvalidLayer(
                format!("Invalid reference name: {}", name)
            ));
        }

        // Create or update the reference
        self.repo.reference(name, oid, true, message)?;
        Ok(())
    }

    fn delete_ref(&self, name: &str) -> Result<()> {
        let mut reference = self.find_ref(name)?;
        reference.delete()?;
        Ok(())
    }

    fn list_refs(&self, pattern: &str) -> Result<Vec<String>> {
        let refs = self.repo.references_glob(pattern)?;
        let mut names = Vec::new();

        for ref_result in refs {
            let reference = ref_result?;
            if let Some(name) = reference.name() {
                names.push(name.to_string());
            }
        }

        Ok(names)
    }

    fn ref_exists(&self, name: &str) -> bool {
        self.repo.find_reference(name).is_ok()
    }

    fn resolve_ref(&self, name: &str) -> Result<Oid> {
        let reference = self.find_ref(name)?;
        let resolved = reference.resolve()?;
        resolved.target().ok_or_else(|| {
            crate::core::JinError::Git(git2::Error::from_str("Reference has no target"))
        })
    }
}


// ================== src/git/objects.rs ==================
use std::path::Path;
use git2::{Oid, Signature, Tree, Blob, Commit};
use crate::core::Result;
use super::JinRepo;

/// File modes for tree entries.
#[derive(Debug, Clone, Copy)]
pub enum EntryMode {
    /// Regular file (0o100644)
    Blob,
    /// Executable file (0o100755)
    BlobExecutable,
    /// Symbolic link (0o120000)
    Link,
    /// Directory/subtree (0o040000)
    Tree,
}

impl EntryMode {
    pub fn as_i32(self) -> i32 {
        match self {
            EntryMode::Blob => 0o100644,
            EntryMode::BlobExecutable => 0o100755,
            EntryMode::Link => 0o120000,
            EntryMode::Tree => 0o040000,
        }
    }
}

/// Tree entry for building trees.
pub struct TreeEntry<'a> {
    pub name: &'a str,
    pub oid: Oid,
    pub mode: EntryMode,
}

/// Object creation operations.
pub trait ObjectOps {
    /// Creates a blob from byte content.
    fn create_blob(&self, content: &[u8]) -> Result<Oid>;

    /// Creates a blob from a file path.
    fn create_blob_from_path(&self, path: &Path) -> Result<Oid>;

    /// Creates a tree from entries.
    fn create_tree(&self, entries: &[TreeEntry]) -> Result<Oid>;

    /// Creates a tree from entries, handling nested paths.
    fn create_tree_from_paths(&self, files: &[(String, Oid)]) -> Result<Oid>;

    /// Creates a commit.
    fn create_commit(
        &self,
        update_ref: Option<&str>,
        message: &str,
        tree_oid: Oid,
        parents: &[Oid],
    ) -> Result<Oid>;

    /// Finds a blob by OID.
    fn find_blob(&self, oid: Oid) -> Result<Blob>;

    /// Finds a tree by OID.
    fn find_tree(&self, oid: Oid) -> Result<Tree>;

    /// Finds a commit by OID.
    fn find_commit(&self, oid: Oid) -> Result<Commit>;
}

impl ObjectOps for JinRepo {
    fn create_blob(&self, content: &[u8]) -> Result<Oid> {
        Ok(self.repo.blob(content)?)
    }

    fn create_blob_from_path(&self, path: &Path) -> Result<Oid> {
        Ok(self.repo.blob_path(path)?)
    }

    fn create_tree(&self, entries: &[TreeEntry]) -> Result<Oid> {
        let mut builder = self.repo.treebuilder(None)?;

        for entry in entries {
            builder.insert(entry.name, entry.oid, entry.mode.as_i32())?;
        }

        Ok(builder.write()?)
    }

    fn create_tree_from_paths(&self, files: &[(String, Oid)]) -> Result<Oid> {
        // Group files by top-level directory
        use std::collections::HashMap;

        let mut root_entries: Vec<TreeEntry> = Vec::new();
        let mut subdirs: HashMap<String, Vec<(String, Oid)>> = HashMap::new();

        for (path, oid) in files {
            if let Some(sep_pos) = path.find('/') {
                let dir = &path[..sep_pos];
                let rest = &path[sep_pos + 1..];
                subdirs
                    .entry(dir.to_string())
                    .or_default()
                    .push((rest.to_string(), *oid));
            } else {
                root_entries.push(TreeEntry {
                    name: path,
                    oid: *oid,
                    mode: EntryMode::Blob,
                });
            }
        }

        // Recursively create subdirectory trees
        for (dir_name, dir_files) in subdirs {
            let subtree_oid = self.create_tree_from_paths(&dir_files)?;
            root_entries.push(TreeEntry {
                name: &dir_name,
                oid: subtree_oid,
                mode: EntryMode::Tree,
            });
        }

        // This won't compile as-is due to lifetime issues with dir_name
        // The actual implementation needs to collect owned strings
        // See Implementation Notes below
        self.create_tree(&root_entries)
    }

    fn create_commit(
        &self,
        update_ref: Option<&str>,
        message: &str,
        tree_oid: Oid,
        parents: &[Oid],
    ) -> Result<Oid> {
        let tree = self.find_tree(tree_oid)?;

        // Get signature from git config or use defaults
        let signature = self.repo.signature().unwrap_or_else(|_| {
            Signature::now("jin", "jin@local").expect("Failed to create signature")
        });

        // Resolve parent OIDs to Commit objects
        let parent_commits: Vec<Commit> = parents
            .iter()
            .map(|oid| self.find_commit(*oid))
            .collect::<Result<Vec<_>>>()?;

        let parent_refs: Vec<&Commit> = parent_commits.iter().collect();

        let oid = self.repo.commit(
            update_ref,
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;

        Ok(oid)
    }

    fn find_blob(&self, oid: Oid) -> Result<Blob> {
        Ok(self.repo.find_blob(oid)?)
    }

    fn find_tree(&self, oid: Oid) -> Result<Tree> {
        Ok(self.repo.find_tree(oid)?)
    }

    fn find_commit(&self, oid: Oid) -> Result<Commit> {
        Ok(self.repo.find_commit(oid)?)
    }
}


// ================== src/git/tree.rs ==================
use std::path::Path;
use git2::{Oid, Tree, TreeEntry as Git2TreeEntry, TreeWalkMode, TreeWalkResult};
use crate::core::Result;
use super::JinRepo;

/// Tree walking and reading operations.
pub trait TreeOps {
    /// Walks a tree in pre-order (parent before children).
    fn walk_tree_pre<F>(&self, tree_oid: Oid, callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult;

    /// Walks a tree in post-order (children before parent).
    fn walk_tree_post<F>(&self, tree_oid: Oid, callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult;

    /// Gets a tree entry by path (e.g., "src/config.json").
    fn get_tree_entry(&self, tree_oid: Oid, path: &Path) -> Result<Oid>;

    /// Reads blob content by OID.
    fn read_blob_content(&self, blob_oid: Oid) -> Result<Vec<u8>>;

    /// Reads file content from a tree by path.
    fn read_file_from_tree(&self, tree_oid: Oid, path: &Path) -> Result<Vec<u8>>;

    /// Lists all files in a tree recursively.
    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>>;
}

impl TreeOps for JinRepo {
    fn walk_tree_pre<F>(&self, tree_oid: Oid, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult,
    {
        let tree = self.repo.find_tree(tree_oid)?;
        tree.walk(TreeWalkMode::PreOrder, |path, entry| callback(path, entry))?;
        Ok(())
    }

    fn walk_tree_post<F>(&self, tree_oid: Oid, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &Git2TreeEntry) -> TreeWalkResult,
    {
        let tree = self.repo.find_tree(tree_oid)?;
        tree.walk(TreeWalkMode::PostOrder, |path, entry| callback(path, entry))?;
        Ok(())
    }

    fn get_tree_entry(&self, tree_oid: Oid, path: &Path) -> Result<Oid> {
        let tree = self.repo.find_tree(tree_oid)?;
        let entry = tree.get_path(path)?;
        Ok(entry.id())
    }

    fn read_blob_content(&self, blob_oid: Oid) -> Result<Vec<u8>> {
        let blob = self.repo.find_blob(blob_oid)?;
        Ok(blob.content().to_vec())
    }

    fn read_file_from_tree(&self, tree_oid: Oid, path: &Path) -> Result<Vec<u8>> {
        let entry_oid = self.get_tree_entry(tree_oid, path)?;
        self.read_blob_content(entry_oid)
    }

    fn list_tree_files(&self, tree_oid: Oid) -> Result<Vec<String>> {
        let mut files = Vec::new();

        self.walk_tree_pre(tree_oid, |parent_path, entry| {
            if let Some(name) = entry.name() {
                // Only include files (blobs), not directories
                if entry.kind() == Some(git2::ObjectType::Blob) {
                    let full_path = if parent_path.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}{}", parent_path, name)
                    };
                    files.push(full_path);
                }
            }
            TreeWalkResult::Ok
        })?;

        Ok(files)
    }
}


// ================== src/git/transaction.rs ==================
use git2::{Transaction, Oid};
use crate::core::Result;
use super::JinRepo;

/// Transaction wrapper for atomic reference updates.
///
/// Note: git2::Transaction is NOT truly atomic - if a ref update fails,
/// previous successful updates are NOT rolled back. This wrapper provides
/// the same interface but documents this limitation.
pub struct JinTransaction<'repo> {
    inner: Transaction<'repo>,
}

impl<'repo> JinTransaction<'repo> {
    /// Creates a new transaction.
    pub fn new(repo: &'repo JinRepo) -> Result<Self> {
        let inner = repo.inner().transaction()?;
        Ok(Self { inner })
    }

    /// Locks a reference for update.
    pub fn lock_ref(&mut self, refname: &str) -> Result<()> {
        self.inner.lock_ref(refname)?;
        Ok(())
    }

    /// Sets the target of a locked reference.
    pub fn set_target(&mut self, refname: &str, target: Oid, message: &str) -> Result<()> {
        // Note: signature is optional in git2, if None it reads from config
        self.inner.set_target(refname, target, None, message)?;
        Ok(())
    }

    /// Removes a locked reference.
    pub fn remove(&mut self, refname: &str) -> Result<()> {
        self.inner.remove(refname)?;
        Ok(())
    }

    /// Commits the transaction.
    ///
    /// # Warning
    /// This is NOT truly atomic. If a ref update fails, previous
    /// successful updates are NOT rolled back.
    pub fn commit(self) -> Result<()> {
        self.inner.commit()?;
        Ok(())
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: UPDATE src/git/mod.rs
  - IMPLEMENT: Module exports and re-exports
  - EXPORTS: pub mod repo, refs, objects, tree, transaction
  - RE-EXPORTS: JinRepo, RefOps, ObjectOps, TreeOps, JinTransaction
  - RE-EXPORTS: git2::{Oid, ObjectType, TreeWalkMode, TreeWalkResult}
  - PLACEMENT: src/git/mod.rs
  - DEPENDS ON: Nothing (can be done first)

Task 2: CREATE src/git/repo.rs
  - IMPLEMENT: JinRepo struct as specified in Data Models
  - METHODS: open(), create(), open_or_create(), default_path(), path(), inner(), inner_mut()
  - FOLLOW pattern: P1.M1 JinConfig for path handling
  - NAMING: JinRepo (not GitRepo or Repository)
  - ERROR HANDLING: Wrap git2::Error into JinError::Git using ? operator
  - PLACEMENT: src/git/repo.rs
  - DEPENDS ON: Task 1

Task 3: CREATE src/git/refs.rs
  - IMPLEMENT: RefOps trait and implementation for JinRepo
  - METHODS: find_ref(), set_ref(), delete_ref(), list_refs(), ref_exists(), resolve_ref()
  - VALIDATION: Use Reference::is_valid_name() before creating refs
  - PATTERN: All ref names should start with "refs/jin/"
  - PLACEMENT: src/git/refs.rs
  - DEPENDS ON: Task 2

Task 4: CREATE src/git/objects.rs
  - IMPLEMENT: EntryMode enum, TreeEntry struct, ObjectOps trait
  - METHODS: create_blob(), create_blob_from_path(), create_tree(), create_tree_from_paths(), create_commit(), find_blob(), find_tree(), find_commit()
  - GOTCHA: create_tree_from_paths() needs owned strings, not references
  - SIGNATURE: Use repo.signature() with fallback to "jin"/"jin@local"
  - PLACEMENT: src/git/objects.rs
  - DEPENDS ON: Task 2

Task 5: CREATE src/git/tree.rs
  - IMPLEMENT: TreeOps trait and implementation for JinRepo
  - METHODS: walk_tree_pre(), walk_tree_post(), get_tree_entry(), read_blob_content(), read_file_from_tree(), list_tree_files()
  - CALLBACK: TreeWalkResult enum (Ok, Skip, Abort)
  - GOTCHA: walk callback receives (path, entry) where path is parent dir
  - PLACEMENT: src/git/tree.rs
  - DEPENDS ON: Task 2, Task 4

Task 6: CREATE src/git/transaction.rs
  - IMPLEMENT: JinTransaction wrapper struct
  - METHODS: new(), lock_ref(), set_target(), remove(), commit()
  - DOCUMENT: NOT truly atomic - partial failures don't rollback
  - LIFETIME: JinTransaction<'repo> borrows from JinRepo
  - PLACEMENT: src/git/transaction.rs
  - DEPENDS ON: Task 2

Task 7: CREATE src/git/tests/mod.rs
  - IMPLEMENT: Test module structure
  - FILES: mod.rs, repo_tests.rs, refs_tests.rs, objects_tests.rs, tree_tests.rs
  - PATTERN: Use tempfile::tempdir() for test repositories
  - PLACEMENT: src/git/tests/
  - DEPENDS ON: Tasks 1-6

Task 8: CREATE src/git/tests/repo_tests.rs
  - IMPLEMENT: Unit tests for JinRepo
  - TESTS:
    - test_create_jin_repo: Creates repo in temp dir
    - test_open_existing_repo: Opens after create
    - test_open_or_create_new: Creates when missing
    - test_open_or_create_existing: Opens when exists
    - test_repo_is_bare: Verifies bare repository
  - PATTERN: Use tempdir and override JIN_HOME or pass custom path
  - PLACEMENT: src/git/tests/repo_tests.rs
  - DEPENDS ON: Task 7

Task 9: CREATE src/git/tests/refs_tests.rs
  - IMPLEMENT: Unit tests for RefOps
  - TESTS:
    - test_set_and_find_ref: Create and retrieve ref
    - test_delete_ref: Create then delete
    - test_list_refs_glob: Multiple refs with pattern
    - test_ref_exists: Check existence
    - test_resolve_ref: Resolve to OID
    - test_invalid_ref_name: Error on invalid names
  - PLACEMENT: src/git/tests/refs_tests.rs
  - DEPENDS ON: Task 7

Task 10: CREATE src/git/tests/objects_tests.rs
  - IMPLEMENT: Unit tests for ObjectOps
  - TESTS:
    - test_create_blob: Create and read blob
    - test_create_tree_simple: Single-level tree
    - test_create_tree_nested: Multi-level tree
    - test_create_commit_initial: Commit without parents
    - test_create_commit_with_parent: Commit with parent
    - test_find_objects: Find blob, tree, commit by OID
  - PLACEMENT: src/git/tests/objects_tests.rs
  - DEPENDS ON: Task 7

Task 11: CREATE src/git/tests/tree_tests.rs
  - IMPLEMENT: Unit tests for TreeOps
  - TESTS:
    - test_walk_tree_pre_order: Pre-order traversal
    - test_walk_tree_post_order: Post-order traversal
    - test_get_tree_entry: Entry by path
    - test_read_blob_content: Read file content
    - test_read_file_from_tree: Full path read
    - test_list_tree_files: Recursive file listing
  - PLACEMENT: src/git/tests/tree_tests.rs
  - DEPENDS ON: Task 7
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Error conversion - add this to src/core/error.rs if not present
impl From<git2::Error> for JinError {
    fn from(err: git2::Error) -> Self {
        JinError::Git(err)
    }
}

// PATTERN: Test setup with tempdir
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_repo() -> (tempfile::TempDir, JinRepo) {
        let dir = tempdir().unwrap();
        let repo_path = dir.path().join(".jin");
        std::fs::create_dir_all(&repo_path).unwrap();

        let mut opts = git2::RepositoryInitOptions::new();
        opts.bare(true);
        let repo = git2::Repository::init_opts(&repo_path, &opts).unwrap();

        let jin_repo = JinRepo {
            repo,
            path: repo_path,
        };

        (dir, jin_repo)
    }

    #[test]
    fn test_create_blob() {
        let (_dir, repo) = create_test_repo();
        let content = b"Hello, Jin!";
        let oid = repo.create_blob(content).unwrap();

        let blob = repo.find_blob(oid).unwrap();
        assert_eq!(blob.content(), content);
    }
}

// PATTERN: Fixed create_tree_from_paths with owned data
impl ObjectOps for JinRepo {
    fn create_tree_from_paths(&self, files: &[(String, Oid)]) -> Result<Oid> {
        use std::collections::HashMap;

        // Collect data with owned strings
        struct OwnedEntry {
            name: String,
            oid: Oid,
            mode: EntryMode,
        }

        let mut root_entries: Vec<OwnedEntry> = Vec::new();
        let mut subdirs: HashMap<String, Vec<(String, Oid)>> = HashMap::new();

        for (path, oid) in files {
            if let Some(sep_pos) = path.find('/') {
                let dir = path[..sep_pos].to_string();
                let rest = path[sep_pos + 1..].to_string();
                subdirs.entry(dir).or_default().push((rest, *oid));
            } else {
                root_entries.push(OwnedEntry {
                    name: path.clone(),
                    oid: *oid,
                    mode: EntryMode::Blob,
                });
            }
        }

        // Recursively create subdirectory trees
        for (dir_name, dir_files) in subdirs {
            let subtree_oid = self.create_tree_from_paths(&dir_files)?;
            root_entries.push(OwnedEntry {
                name: dir_name,
                oid: subtree_oid,
                mode: EntryMode::Tree,
            });
        }

        // Build the tree
        let mut builder = self.repo.treebuilder(None)?;
        for entry in &root_entries {
            builder.insert(&entry.name, entry.oid, entry.mode.as_i32())?;
        }

        Ok(builder.write()?)
    }
}

// PATTERN: Layer ref path generation (uses Layer enum from P1.M1)
use crate::core::Layer;

impl JinRepo {
    /// Creates a reference for a layer with the given commit.
    pub fn set_layer_ref(
        &self,
        layer: Layer,
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
        commit_oid: Oid,
    ) -> Result<()> {
        let ref_path = layer.ref_path(mode, scope, project);
        self.set_ref(&ref_path, commit_oid, "Layer update")
    }
}
```

### Integration Points

```yaml
FILESYSTEM:
  - Jin repository at: ~/.jin/ (bare repo)
  - Layer refs at: refs/jin/layers/*
  - No working directory (bare repo)

DEPENDENCIES (from P1.M1):
  - JinError::Git variant for git2::Error
  - Layer::ref_path() for generating ref names
  - JinConfig::default_path() pattern for ~/.jin/

ERROR HANDLING:
  - All git2::Error wrapped into JinError::Git
  - Use ? operator for propagation
  - Custom errors for invalid ref names (JinError::InvalidLayer)

FUTURE INTEGRATION (P1.M3):
  - JinTransaction will be used by Transaction system
  - ObjectOps will be used by commit pipeline
  - RefOps will be used for atomic layer updates
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check - treat warnings as errors

# Expected: Zero errors, zero warnings
# If errors: READ output carefully, fix the specific issue, re-run
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build
cargo build --release                 # Release build

# Binary still works
./target/debug/jin --help             # Should show help text

# Expected: Clean build
```

### Level 3: Unit Tests

```bash
# Run git module tests specifically
cargo test git::                      # All git module tests
cargo test git::repo::                # JinRepo tests
cargo test git::refs::                # RefOps tests
cargo test git::objects::             # ObjectOps tests
cargo test git::tree::                # TreeOps tests

# Run with output
cargo test git:: -- --nocapture       # See println! output

# Run all tests
cargo test

# Expected: All tests pass
```

### Level 4: Integration Testing

```bash
# Create a test Jin repository and verify operations
cargo test --test git_integration

# Manual verification in temp directory
cd $(mktemp -d)
export JIN_HOME=$(pwd)/.jin

# Use the binary to init (once jin init is implemented)
# For now, verify via Rust tests only

# Expected: All operations work correctly
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo test git::` all tests pass
- [ ] `cargo test` all tests pass (including existing)

### Feature Validation

- [ ] JinRepo::open() opens existing Jin repo
- [ ] JinRepo::create() initializes bare repo
- [ ] JinRepo::open_or_create() handles both cases
- [ ] create_blob() writes content and returns OID
- [ ] create_tree() builds tree from entries
- [ ] create_tree_from_paths() handles nested directories
- [ ] create_commit() creates commit with tree and parents
- [ ] find_ref() locates refs by name
- [ ] set_ref() creates/updates refs
- [ ] delete_ref() removes refs
- [ ] list_refs() returns matching refs
- [ ] walk_tree_pre() traverses in pre-order
- [ ] walk_tree_post() traverses in post-order
- [ ] get_tree_entry() retrieves by path
- [ ] read_blob_content() returns bytes
- [ ] list_tree_files() returns all file paths

### Code Quality Validation

- [ ] All public types and methods have doc comments
- [ ] Error handling uses JinError::Git consistently
- [ ] No unwrap() in library code (except tests)
- [ ] Tests use tempdir for isolation
- [ ] Trait-based design for extensibility (RefOps, ObjectOps, TreeOps)

---

## Anti-Patterns to Avoid

- ❌ Don't use `unwrap()` or `expect()` in library code - use `?` operator
- ❌ Don't create refs outside `refs/jin/` namespace - could conflict with user's Git
- ❌ Don't forget to validate ref names with `Reference::is_valid_name()`
- ❌ Don't assume Repository::signature() succeeds - provide fallback
- ❌ Don't build nested trees in one TreeBuilder call - build inner trees first
- ❌ Don't forget the path parameter in tree walk is the PARENT directory
- ❌ Don't rely on Transaction atomicity for critical operations - it's not truly atomic
- ❌ Don't use the user's `.git` - Jin has its own bare repo at `~/.jin/`

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Comprehensive API documentation from git2-rs extracted and included
- Complete code examples for all core operations
- Clear dependency ordering between tasks
- Gotchas and limitations documented (TreeBuilder single-level, Transaction non-atomicity)
- Test patterns provided with tempdir isolation
- Integration with P1.M1 types clearly specified

**Remaining Risks:**
- git2 vendored build may have platform-specific issues (mitigated by feature flag)
- create_tree_from_paths() recursive implementation may need tuning for edge cases
- Transaction atomicity limitation may require design reconsideration in P1.M3

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M2/research/`

Key files:
- `phantom_git_patterns.md` - Comprehensive research on overlay Git patterns, ref namespaces, GIT_DIR redirection, garbage collection safety

External references:
- [git2-rs Repository Documentation](https://docs.rs/git2/latest/git2/struct.Repository.html)
- [git2-rs Reference Documentation](https://docs.rs/git2/latest/git2/struct.Reference.html)
- [git2-rs TreeBuilder Documentation](https://docs.rs/git2/latest/git2/struct.TreeBuilder.html)
- [git2-rs Transaction Documentation](https://docs.rs/git2/latest/git2/struct.Transaction.html)
- [libgit2 101 Samples](https://libgit2.org/docs/guides/101-samples/)
- [Git Custom Refs and GC](https://git-scm.com/docs/git-gc)
