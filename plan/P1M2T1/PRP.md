# Product Requirement Prompt (PRP): Jin Repository Wrapper (P1.M2.T1)

---

## Goal

**Feature Goal**: Create `JinRepo`, a wrapper around `git2::Repository` that provides Jin-specific Git operations including layer reference management, bare repository initialization, and object creation helpers with proper `JinError` integration.

**Deliverable**: A `src/git/repo.rs` module with:
- `JinRepo` struct that wraps `git2::Repository` (owned, not borrowed)
- Constructor methods: `open()`, `open_or_create()`, `init()`
- Layer reference operations: `get_layer_ref()`, `set_layer_ref()`, `create_layer_ref()`
- Object creation helpers: `create_blob()`, `create_tree()`, `create_commit()`
- Delegation methods for common git2 operations
- Comprehensive unit tests using `tempfile`

**Success Definition**:
- `cargo build` compiles with zero errors
- All unit tests pass with isolated test repositories
- `JinRepo` can open/create bare Jin repositories at `~/.jin/repo`
- Layer references are correctly managed using `Layer.git_ref()`
- Integration with `JinError` and `Layer` types is seamless

## User Persona

**Target User**: AI coding agent implementing Jin's Git layer integration foundation

**Use Case**: The agent needs to establish the Git repository abstraction layer that:
- Opens/creates the bare Jin repository at `~/.jin/repo`
- Manages layer references under `refs/jin/layers/...`
- Provides object creation for staging and commits
- Integrates with existing `JinError` and `Layer` types

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/git/repo.rs` with `JinRepo` wrapper
3. Implements constructor methods for repo initialization
4. Implements layer reference operations using `Layer.git_ref()`
5. Implements object creation helpers
6. Adds comprehensive unit tests
7. Validates compilation and test success

**Pain Points Addressed**:
- No manual ref string construction - `Layer.git_ref()` provides exact format
- Consistent error handling with `JinError` integration
- Isolated test setup using `tempfile` patterns
- Clear bare repository semantics (no working directory confusion)

## Why

- **Foundation for all Git operations**: Every subsequent Git operation (refs, objects, transactions, staging, commit) depends on `JinRepo`
- **Jin-specific semantics**: Bare repositories, logical refs, layer-aware operations are unique to Jin's architecture
- **Integration point**: Bridges generic git2 operations with Jin's `Layer` enum and `JinError` types
- **Problems this solves**:
  - Prevents hardcoded ref strings that could diverge from `Layer.git_ref()`
  - Centralizes bare repository initialization logic
  - Provides consistent error conversion from git2 to JinError
  - Enables isolated testing with temporary repositories

## What

Create a `JinRepo` wrapper around `git2::Repository` that provides Jin-specific operations for managing the bare Git repository at `~/.jin/repo` and layer references under `refs/jin/layers/...`.

### Success Criteria

- [ ] `src/git/repo.rs` created with `JinRepo` struct
- [ ] `JinRepo::open()` opens existing bare repository
- [ ] `JinRepo::open_or_create()` opens or initializes bare repository
- [ ] `get_layer_ref()` returns layer reference or None using `Layer.git_ref()`
- [ ] `set_layer_ref()` creates/updates layer reference
- [ ] Object creation helpers (`create_blob`, `create_tree`, `create_commit`) work
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover all public methods with temp repos
- [ ] `cargo test` passes all tests
- [ ] Module exported from `src/git/mod.rs`

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact `JinRepo` struct specification with all methods
- Research documents with 50+ code examples for git2 operations
- Specific patterns from existing codebase to follow
- Complete integration guide with `Layer` and `JinError` types
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Git Architecture specification - bare repos, logical refs, ref namespace
  section: Lines 84-115 for Logical Branch Model, Lines 558-585 for Git and Environment
  critical: Jin uses bare repos, refs/jin/ namespace, no user-facing branches

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Git ref namespace and module structure
  section: Lines 103-116 for Git Ref Namespace format
  critical: refs/jin/layers/ prefix, exact ref format for each layer

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  pattern: JinError::Git (transparent), JinError::RepoNotFound, JinError::RefNotFound, JinError::RefExists
  gotcha: Use #[from] for automatic git2::Error conversion

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum's git_ref() method provides exact ref format - CRITICAL
  section: Lines 215-279 for git_ref() implementation
  critical: ALWAYS call layer.git_ref() to get ref names, NEVER hardcode strings

- file: /home/dustin/projects/jin-glm-doover/src/core/config.rs
  why: JinConfig.repository provides the repo path to open
  section: Lines 82-111 for JinConfig::load() method
  gotcha: Repository path may not exist yet, needs open_or_create pattern

- file: /home/dustin/projects/jin-glm-doover/src/git/mod.rs
  why: Module must export JinRepo after creation
  section: File shows current module structure
  gotcha: Need to add pub mod repo; and pub use repo::JinRepo;

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify git2 dependency with required features
  section: Line 18 for git2 dependency
  critical: git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] }

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T1/research/git2_patterns.md
  why: 50+ code examples for all git2 operations needed
  section: Repository Initialization (lines 12-94), Reference Management (lines 96-214), Object Creation (lines 216-356), Error Handling (lines 358-479)
  critical: Shows exact error conversion patterns, ref creation, blob/tree/commit creation

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T1/research/repo_wrapper_patterns.md
  why: Architectural patterns from real projects (GitButler, gitoxide)
  section: Simple Wrapper Pattern (lines 9-35), Contextual Wrapper Pattern (lines 42-70)
  critical: Shows how to structure wrapper with domain-specific helpers

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T1/research/testing_patterns.md
  why: Complete testing patterns with tempfile, fixtures, error testing
  section: TestRepoFixture (lines 67-158), Error Testing Patterns (lines 160-214), Complete Test Example (lines 442-534)
  critical: Integration with existing tempfile dependency in Cargo.toml

# EXTERNAL - git2-rs Documentation

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html
  why: Complete Repository API - all methods we'll delegate or wrap
  critical: open(), init_bare(), find_reference(), reference(), is_bare(), path()

- url: https://docs.rs/git2/0.20/git2/
  why: Top-level git2 crate documentation
  section: Module re-exports for Reference, Oid, Signature, TreeBuilder
  critical: Understanding types used in wrapper methods

- url: https://github.com/rust-lang/git2-rs/blob/master/examples.rs
  why: Official examples showing common operations
  section: Examples for bare repo init, ref creation, commit creation
  critical: Reference creation with force flag, commit with parents

- url: https://docs.rs/git2/0.20/git2/struct.Reference.html
  why: Reference API for get/set operations
  critical: name(), target(), set_target(), resolve()

- url: https://docs.rs/git2/0.20/git2/struct.TreeBuilder.html
  why: TreeBuilder API for creating trees from entries
  critical: insert(), write(), get()
```

### Current Codebase Tree

```bash
# Run this command to verify current state
tree -L 3 -I 'target|Cargo.lock' /home/dustin/projects/jin-glm-doover

# Expected output:
# /home/dustin/projects/jin-glm-doover
# ├── Cargo.toml                      # Has git2 dependency with features
# ├── PRD.md
# ├── src/
# │   ├── main.rs
# │   ├── lib.rs
# │   ├── cli/mod.rs
# │   ├── commands/mod.rs
# │   ├── commit/mod.rs
# │   ├── core/
# │   │   ├── mod.rs                 # Exports error, layer, config
# │   │   ├── error.rs               # Has JinError::Git, RepoNotFound, RefNotFound
# │   │   ├── layer.rs               # Has Layer enum with git_ref() method
# │   │   └── config.rs              # Has JinConfig.repository path
# │   ├── git/
# │   │   └── mod.rs                 # Currently empty, needs to export JinRepo
# │   ├── merge/mod.rs
# │   ├── staging/mod.rs
# │   └── workspace/mod.rs
# └── tests/
#     └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── git/
│       ├── mod.rs                    # MODIFY: Add pub mod repo; pub use repo::JinRepo;
│       └── repo.rs                   # CREATE: JinRepo wrapper implementation
└── tests/
    └── git/
        └── repo_test.rs              # CREATE: Unit tests for JinRepo
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Jin uses BARE repositories
// Jin repos at ~/.jin/repo have NO working directory
// repo.workdir() returns None, repo.is_bare() returns true
// Do NOT call methods that require workdir:
//   - repo.status() - will fail
//   - repo.checkout() - will fail
//   - repo.index() with workdir - will fail
// Solution: Always use Repository::init_bare() for Jin repos

// CRITICAL: Jin refs are LOGICAL, not branches
// refs/jin/layers/global is NOT a user-facing branch
// NEVER:
//   - Use repo.branch() - creates user-facing branches
//   - Check out these refs - not meant for working directory
// ALWAYS:
//   - Use repo.reference() to create/update refs
//   - Use repo.find_reference() to read refs
// These refs are internal implementation only

// CRITICAL: Layer enum provides ref names
// ALWAYS use layer.git_ref() to get ref names
// NEVER hardcode "refs/jin/layers/" strings
// Good:
//   let ref_name = layer.git_ref().unwrap();
// Bad:
//   let ref_name = format!("refs/jin/layers/{}", mode);
// Rationale: If Layer.git_ref() format changes, code adapts automatically

// CRITICAL: Error conversion pattern
// git2::Error converts transparently via JinError::Git
// Use ? operator for automatic conversion:
//   let commit = self.inner.find_commit(oid)?;
// Use explicit conversion for context:
//   let inner = Repository::open(path)
//       .map_err(|e| JinError::RepoNotFound { path: path.display().to_string() })?;

// CRITICAL: Repository may not exist on first open
// JinConfig.repository path might point to non-existent repo
// Implement open_or_create() pattern:
//   1. Try Repository::open()
//   2. If NotFound error, call Repository::init_bare()
//   3. Return JinRepo wrapping the result
// First call to jin init will trigger bare repo creation

// GOTCHA: git2::Repository doesn't implement Clone
// Cannot directly clone JinRepo if it owns Repository
// Workaround options if shared access needed (future):
//   - Use Arc<Mutex<Repository>> (adds complexity)
//   - Pass &Repository references
//   - Create multiple JinRepo instances (cheap after first open)
// For now: Owned Repository is fine, JinRepo is cheap to create

// GOTCHA: Reference names must be valid UTF-8
// Layer names (mode, scope, project) come from user input
// git2-rs expects UTF-8 strings for ref names
// If user provides non-UTF-8 input, ref creation will fail
// Solution: Validate user input before creating layers (future task)

// PATTERN: Follow thiserror derive pattern from error.rs
// JinError already has #[error(transparent)] for git2::Error
// Use #[from] for automatic conversion:
//   #[error(transparent)]
//   Git(#[from] git2::Error),
// This means ? operator on git2 operations auto-converts to JinError

// PATTERN: Reference creation force flag
// repo.reference(&name, oid, force, msg)
// force = false: Fails if ref exists (RefExists error)
// force = true: Overwrites existing ref
// For set_layer_ref: Use force=true to allow updates
// For create_layer_ref: Use force=false for safety

// PATTERN: TreeBuilder for tree creation
// TreeBuilder is used to create Git trees from entries
// Always call builder.write() to get the Tree ID
// Example:
//   let mut builder = repo.treebuilder(None)?;
//   builder.insert("path.txt", blob_id, file_mode)?;
//   let tree_id = builder.write()?;
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
/// JinRepo wraps a git2::Repository for Jin-specific operations.
///
/// The wrapper owns the Repository (not borrowed) and provides:
/// - Jin-aware constructors (bare repos only)
/// - Layer reference management using Layer.git_ref()
/// - Object creation helpers
/// - Delegated common operations with JinError conversion
///
/// # Jin-Specific Semantics
///
/// - **Bare Repository**: Jin repos have no working directory
/// - **Logical Refs**: Layer refs under `refs/jin/layers/...` are not branches
/// - **Layer Integration**: Ref names come from `Layer.git_ref()`, not hardcoded
///
/// # Examples
///
/// ```ignore
/// use jin_glm::git::JinRepo;
/// use jin_glm::core::Layer;
/// use std::path::Path;
///
/// // Open or create the Jin repository
/// let repo = JinRepo::open_or_create(Path::new("~/.jin/repo"))?;
///
/// // Get or create a layer reference
/// let layer = Layer::GlobalBase;
/// match repo.get_layer_ref(&layer)? {
///     Some(reference) => println!("Layer exists: {:?}", reference.target()),
///     None => println!("Layer does not exist yet"),
/// }
/// ```
pub struct JinRepo {
    /// The underlying git2 repository (owned, not borrowed)
    pub(crate) inner: git2::Repository,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/git/repo.rs
  - IMPLEMENT: JinRepo struct with owned Repository field
  - PATTERN: Follow "Simple Wrapper Pattern" from repo_wrapper_patterns.md
  - STRUCTURE:
    * pub struct JinRepo { pub(crate) inner: git2::Repository }
    * Use pub(crate) for inner to allow access from other git modules
  - DERIVE: Debug (Clone not possible since Repository doesn't implement Clone)
  - IMPORTS:
    * use crate::core::error::{JinError, Result}
    * use crate::core::Layer
    * use git2::Repository
    * use std::path::Path
  - PLACEMENT: New file src/git/repo.rs
  - NAMING: JinRepo (matches JinError, JinConfig, Layer naming pattern)

Task 2: IMPLEMENT JinRepo::open()
  - IMPLEMENT: pub fn open(path: &Path) -> Result<Self>
  - PATTERN: Use Repository::open() with JinError::RepoNotFound conversion
  - CODE TEMPLATE:
    pub fn open(path: &Path) -> Result<Self> {
        let inner = Repository::open(path)
            .map_err(|e| JinError::RepoNotFound {
                path: path.display().to_string(),
            })?;
        Ok(Self { inner })
    }
  - ERROR HANDLING: Explicit conversion provides path context
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 3: IMPLEMENT JinRepo::init()
  - IMPLEMENT: pub fn init(path: &Path) -> Result<Self>
  - PATTERN: Use Repository::init_bare() - Jin repos are always bare
  - CODE TEMPLATE:
    pub fn init(path: &Path) -> Result<Self> {
        let inner = Repository::init_bare(path)
            .map_err(|e| JinError::Message(format!("Failed to init bare repo: {}", e)))?;
        Ok(Self { inner })
    }
  - GOTCHA: MUST use init_bare(), not init() - Jin has no working directory
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 4: IMPLEMENT JinRepo::open_or_create()
  - IMPLEMENT: pub fn open_or_create(path: &Path) -> Result<Self>
  - PATTERN: Try open(), fall back to init() if not found
  - CODE TEMPLATE:
    pub fn open_or_create(path: &Path) -> Result<Self> {
        match Repository::open(path) {
            Ok(repo) => Ok(Self { inner: repo }),
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                Self::init(path)
            }
            Err(e) => Err(JinError::from(e)),
        }
    }
  - USE CASE: First call to jin init when ~/.jin/repo doesn't exist
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 2, Task 3

Task 5: IMPLEMENT get_layer_ref()
  - IMPLEMENT: pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>>
  - PATTERN: Use layer.git_ref() to get ref name, handle NotFound gracefully
  - CODE TEMPLATE:
    pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        match self.inner.find_reference(&ref_name) {
            Ok(reference) => Ok(Some(reference)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(JinError::from(e)),
        }
    }
  - CRITICAL: Use layer.git_ref(), never hardcode "refs/jin/layers/" strings
  - SEMANTICS: Returns None if ref doesn't exist (not an error)
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1, knowledge of Layer enum

Task 6: IMPLEMENT set_layer_ref()
  - IMPLEMENT: pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference>
  - PATTERN: Use layer.git_ref() with force=true to allow updates
  - CODE TEMPLATE:
    pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        self.inner.reference(
            &ref_name,
            oid,
            true,  // force=true to allow updates
            &format!("Update layer: {:?}", layer)
        ).map_err(JinError::from)
    }
  - CRITICAL: force=true enables updating existing refs
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1, Task 5

Task 7: IMPLEMENT create_layer_ref()
  - IMPLEMENT: pub fn create_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference>
  - PATTERN: Same as set_layer_ref but with force=false for safety
  - DIFFERENCE: Fails with RefExists if ref already exists
  - CODE TEMPLATE:
    pub fn create_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        self.inner.reference(
            &ref_name,
            oid,
            false,  // force=false - fail if exists
            &format!("Create layer: {:?}", layer)
        ).map_err(|e| match e.code() {
            git2::ErrorCode::Exists => JinError::RefExists {
                name: ref_name.clone(),
                layer: format!("{:?}", layer),
            },
            _ => JinError::from(e),
        })
    }
  - USE CASE: Initial layer creation where existence indicates error
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1, Task 6

Task 8: IMPLEMENT object creation helpers
  - IMPLEMENT: pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid>
  - IMPLEMENT: pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid>
  - IMPLEMENT: pub fn create_commit(&self, ...) -> Result<git2::Oid>
  - PATTERN: Delegate to inner repository with ? operator
  - REUSE: Examples from git2_patterns.md (lines 220-356)
  - CODE TEMPLATES:
    // Blob creation
    pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> {
        Ok(self.inner.blob(data)?)
    }

    // Tree creation helper
    pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid> {
        Ok(builder.write()?)
    }

    // Commit creation
    pub fn create_commit(
        &self,
        update_ref: Option<&str>,
        author: &git2::Signature,
        committer: &git2::Signature,
        message: &str,
        tree: &git2::Tree,
        parents: &[&git2::Commit],
    ) -> Result<git2::Oid> {
        Ok(self.inner.commit(update_ref, author, committer, message, tree, parents)?)
    }
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 9: IMPLEMENT convenience delegation methods
  - IMPLEMENT: pub fn head(&self) -> Result<git2::Reference>
  - IMPLEMENT: pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit>
  - IMPLEMENT: pub fn find_tree(&self, oid: git2::Oid) -> Result<git2::Tree>
  - IMPLEMENT: pub fn find_blob(&self, oid: git2::Oid) -> Result<git2::Blob>
  - IMPLEMENT: pub fn treebuilder(&self) -> Result<git2::TreeBuilder>
  - PATTERN: Simple delegation with ? operator for auto-conversion
  - CODE TEMPLATES:
    pub fn head(&self) -> Result<git2::Reference> {
        Ok(self.inner.head()?)
    }

    pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit> {
        Ok(self.inner.find_commit(oid)?)
    }

    pub fn find_tree(&self, oid: git2::Oid) -> Result<git2::Tree> {
        Ok(self.inner.find_tree(oid)?)
    }

    pub fn find_blob(&self, oid: git2::Oid) -> Result<git2::Blob> {
        Ok(self.inner.find_blob(oid)?)
    }

    pub fn treebuilder(&self) -> Result<git2::TreeBuilder> {
        Ok(self.inner.treebuilder(None)?)
    }
  - RATIONALE: Auto-converts git2::Error to JinError via ? operator
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 10: IMPLEMENT helper methods
  - IMPLEMENT: pub fn is_bare(&self) -> bool
  - IMPLEMENT: pub fn path(&self) -> &Path
  - IMPLEMENT: pub fn signature(&self, name: &str, email: &str) -> Result<git2::Signature>
  - PATTERN: Delegation or wrapper around git2 methods
  - CODE TEMPLATES:
    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    pub fn signature(&self, name: &str, email: &str) -> Result<git2::Signature> {
        Ok(git2::Signature::now(name, email)?)
    }
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 11: MODIFY src/git/mod.rs
  - ADD: pub mod repo;
  - ADD: pub use repo::JinRepo;
  - PRESERVE: Any existing content or comments
  - FINAL FILE:
    pub mod repo;
    pub use repo::JinRepo;
  - PLACEMENT: src/git/mod.rs
  - DEPENDENCIES: Task 1 (repo.rs must exist)

Task 12: CREATE tests/git/repo_test.rs
  - IMPLEMENT: Unit tests for all JinRepo methods
  - FIXTURE: TestRepoFixture pattern from testing_patterns.md (lines 67-158)
  - TESTS:
    * test_jinrepo_init_creates_bare_repo()
    * test_jinrepo_open_existing_repo()
    * test_jinrepo_open_or_create_new()
    * test_jinrepo_open_or_create_existing()
    * test_jinrepo_open_nonexistent_errors()
    * test_jinrepo_get_layer_ref_not_found()
    * test_jinrepo_set_layer_ref()
    * test_jinrepo_create_layer_ref()
    * test_jinrepo_create_layer_ref_fails_if_exists()
    * test_jinrepo_create_blob()
    * test_jinrepo_create_commit()
    * test_jinrepo_layer_git_ref_integration()
    * test_jinrepo_error_conversion()
  - FOLLOW: Pattern from testing_patterns.md
  - USE: tempfile for temp repo directories
  - ASSERTIONS: Use Result<()> return for tests
  - PLACEMENT: tests/git/repo_test.rs (create tests/git/ directory first)
  - DEPENDENCIES: Tasks 1-10
```

### Implementation Patterns & Key Details

```rust
// ===== WRAPPER STRUCT PATTERN =====
// JinRepo owns the Repository (not borrowed)
// This provides simple lifetime management
pub struct JinRepo {
    pub(crate) inner: git2::Repository,
}

// ===== CONSTRUCTOR PATTERN =====
// Convert git2 errors to JinError with context
impl JinRepo {
    pub fn open(path: &Path) -> Result<Self> {
        let inner = Repository::open(path)
            .map_err(|e| JinError::RepoNotFound {
                path: path.display().to_string(),
            })?;
        Ok(Self { inner })
    }

    pub fn init(path: &Path) -> Result<Self> {
        // CRITICAL: Use init_bare() for Jin repos
        let inner = Repository::init_bare(path)?;
        Ok(Self { inner })
    }
}

// ===== LAYER REF PATTERN =====
// ALWAYS use layer.git_ref() to get ref names
// NEVER hardcode "refs/jin/layers/" strings
impl JinRepo {
    pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>> {
        // GOOD: Use layer.git_ref()
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        // BAD: let ref_name = format!("refs/jin/layers/global");

        match self.inner.find_reference(&ref_name) {
            Ok(r) => Ok(Some(r)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(JinError::from(e)),
        }
    }

    pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        // force=true allows updating existing refs
        self.inner.reference(&ref_name, oid, true, &format!("Update layer: {:?}", layer))
            .map_err(JinError::from)
    }
}

// ===== ERROR HANDLING PATTERN =====
// Use ? for automatic conversion via JinError::Git
// Use explicit .map_err() for context-specific errors
impl JinRepo {
    // Automatic conversion via ? operator
    pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit> {
        // ? operator uses JinError::Git transparent conversion
        Ok(self.inner.find_commit(oid)?)
    }

    // Explicit conversion for context
    pub fn open(path: &Path) -> Result<Self> {
        // .map_err() provides path context in error
        let inner = Repository::open(path)
            .map_err(|e| JinError::RepoNotFound {
                path: path.display().to_string(),
            })?;
        Ok(Self { inner })
    }
}

// ===== BARE REPOSITORY PATTERN =====
// Jin repos are bare - no working directory operations
impl JinRepo {
    pub fn init(path: &Path) -> Result<Self> {
        // MUST use init_bare(), NOT init()
        let inner = Repository::init_bare(path)?;
        Ok(Self { inner })
    }

    pub fn is_bare(&self) -> bool {
        // Should always return true for Jin repos
        self.inner.is_bare()
    }
}

// ===== OBJECT CREATION PATTERN =====
impl JinRepo {
    pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> {
        Ok(self.inner.blob(data)?)
    }

    pub fn create_commit(
        &self,
        update_ref: Option<&str>,
        author: &git2::Signature,
        committer: &git2::Signature,
        message: &str,
        tree: &git2::Tree,
        parents: &[&git2::Commit],
    ) -> Result<git2::Oid> {
        Ok(self.inner.commit(update_ref, author, committer, message, tree, parents)?)
    }
}
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs
  - patterns:
    * JinError::Git (transparent) - automatic via #[from]
    * JinError::RepoNotFound { path } - explicit conversion
    * JinError::RefNotFound { name, layer } - for missing refs
    * JinError::RefExists { name, layer } - for duplicate refs
    * JinError::InvalidLayer { name } - for UserLocal/WorkspaceActive layers

LAYER_INTEGRATION:
  - use: src/core/layer.rs
  - method: layer.git_ref() returns Option<String>
  - layers: Returns None for UserLocal, WorkspaceActive (not versioned)
  - pattern:
    * let ref_name = layer.git_ref()
    *     .ok_or_else(|| JinError::InvalidLayer { ... })?;

CONFIG_INTEGRATION:
  - use: src/core/config.rs
  - method: JinConfig::load().repository provides repo path
  - pattern:
    * let config = JinConfig::load()?;
    * let repo = JinRepo::open_or_create(&config.repository)?;

MODULE_EXPORTS:
  - modify: src/git/mod.rs
  - add: pub mod repo;
  - add: pub use repo::JinRepo;
  - result: crate::git::JinRepo is accessible

TESTING:
  - create: tests/git/repo_test.rs
  - use: tempfile crate (already in Cargo.toml)
  - pattern: TestRepoFixture from testing_patterns.md

FUTURE_INTEGRATION:
  - P1.M2.T2: Reference Management will use get/set_layer_ref
  - P1.M2.T3: Object Creation will use create_blob/tree/commit
  - P1.M2.T4: Tree Walking will use find_tree()
  - P1.M3: Transaction System will use layer ref operations
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating repo.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "unused_imports" -> remove unused imports
# - "dead_code" -> public methods are used by tests, use #[allow(dead_code)] temporarily
# - Pattern matching errors -> ensure all JinError variants handled
# - Missing trait implementations -> Debug may be needed
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test JinRepo module specifically
cargo test --package jin --lib git::repo --verbose

# Run all git module tests
cargo test --package jin --lib git:: --verbose

# Run with output
cargo test --package jin --lib git::repo -- --nocapture

# Expected: All tests pass. Look for:
# - test_jinrepo_init_creates_bare_repo: Verifies bare repo creation
# - test_jinrepo_open_existing_repo: Verifies opening existing repo
# - test_jinrepo_open_or_create_new: Verifies init when not found
# - test_jinrepo_get_layer_ref_not_found: Returns None, not error
# - test_jinrepo_set_layer_ref: Creates/updates layer refs
# - test_jinrepo_layer_git_ref_integration: Uses Layer.git_ref()
# - test_jinrepo_error_conversion: JinError variants correct
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual repository operations with real git2
cargo test --package jin --test repo_test --verbose

# Manual verification of bare repository creation
# Create temp directory and test:
cd /tmp
mkdir test_jin_repo
cd test_jin_repo
python3 << 'EOF'
import sys
sys.path.insert(0, '/home/dustin/projects/jin-glm-doover/target/debug')
# Or use rust test that prints repo path
EOF

# Verify refs created in correct namespace
# After test creates a layer ref, verify:
# ls -la .git/refs/jin/layers/
# Should show refs created by tests

# Expected:
# - Bare repository created (no working directory)
# - Refs appear under refs/jin/layers/
# - Tests can read back refs they created
```

### Level 4: Domain-Specific Validation

```bash
# Verify bare repository semantics
cargo test --package jin test_jinrepo_init_creates_bare_repo -- --exact
# Asserts: repo.is_bare() == true

# Verify Layer.git_ref() integration
cargo test --package jin test_jinrepo_layer_git_ref_integration -- --exact
# Asserts: Ref names match Layer.git_ref() format exactly

# Verify error conversion
cargo test --package jin test_jinrepo_error_conversion -- --exact
# Asserts: RepoNotFound, RefNotFound, RefExists errors work

# Test UserLocal/WorkspaceActive layers return errors
# These layers have git_ref() = None
cargo test --package jin test_jinrepo_unversioned_layers_error -- --exact
# Asserts: InvalidLayer error for UserLocal, WorkspaceActive

# Expected: All Jin-specific requirements from PRD are met
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

- [ ] `JinRepo::init()` creates bare repository (is_bare() returns true)
- [ ] `JinRepo::open()` opens existing bare repository
- [ ] `JinRepo::open_or_create()` handles both cases
- [ ] `get_layer_ref()` returns None for missing refs (not error)
- [ ] `set_layer_ref()` creates/updates layer refs
- [ ] `create_layer_ref()` fails if ref exists (RefExists error)
- [ ] Layer refs use `Layer.git_ref()` format (not hardcoded)
- [ ] Object creation helpers (blob, tree, commit) work
- [ ] Integration with `JinError` (Git, RepoNotFound, RefNotFound, RefExists)

### Code Quality Validation

- [ ] Follows existing codebase patterns (error.rs, layer.rs structure)
- [ ] File placement matches desired tree structure
- [ ] Module exported from `src/git/mod.rs`
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods

### Documentation & Deployment

- [ ] Module-level doc comment explains JinRepo purpose
- [ ] Each struct has doc comment explaining Jin-specific semantics
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (bare repos, logical refs, Layer.git_ref())

---

## Anti-Patterns to Avoid

- ❌ Don't hardcode "refs/jin/layers/" strings - use `layer.git_ref()`
- ❌ Don't use `Repository::init()` - Jin repos are bare, use `init_bare()`
- ❌ Don't use workdir-dependent methods (status(), checkout()) - Jin repos are bare
- ❌ Don't use `repo.branch()` - Jin refs are not branches, use `repo.reference()`
- ❌ Don't skip error conversion - use `JinError` consistently
- ❌ Don't expose `inner` Repository as public - wrap methods needed
- ❌ Don't check out layer refs - they're internal implementation only
- ❌ Don't use force=false for `set_layer_ref` - prevents updates
- ❌ Don't forget to handle UserLocal/WorkspaceActive - their git_ref() returns None
- ❌ Don't create tests that assume working directory exists - use bare repos

---

## Appendix: Quick Reference

### JinRepo API Summary

```rust
// Constructors
pub fn open(path: &Path) -> Result<Self>
pub fn init(path: &Path) -> Result<Self>
pub fn open_or_create(path: &Path) -> Result<Self>

// Layer Reference Operations
pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>>
pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference>
pub fn create_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference>

// Object Creation
pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid>
pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid>
pub fn create_commit(&self, ...) -> Result<git2::Oid>

// Delegation Methods
pub fn head(&self) -> Result<git2::Reference>
pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit>
pub fn find_tree(&self, oid: git2::Oid) -> Result<git2::Tree>
pub fn find_blob(&self, oid: git2::Oid) -> Result<git2::Blob>
pub fn treebuilder(&self) -> Result<git2::TreeBuilder>

// Helper Methods
pub fn is_bare(&self) -> bool
pub fn path(&self) -> &Path
pub fn signature(&self, name: &str, email: &str) -> Result<git2::Signature>
```

### Layer Ref Namespace

| Layer Variant | git_ref() Returns | Versioned |
|---------------|-------------------|-----------|
| GlobalBase | `refs/jin/layers/global` | Yes |
| ModeBase { mode } | `refs/jin/layers/mode/{mode}` | Yes |
| ModeScope { mode, scope } | `refs/jin/layers/mode/{mode}/scope/{scope}` | Yes |
| ModeScopeProject { mode, scope, project } | `refs/jin/layers/mode/{mode}/scope/{scope}/project/{project}` | Yes |
| ModeProject { mode, project } | `refs/jin/layers/mode/{mode}/project/{project}` | Yes |
| ScopeBase { scope } | `refs/jin/layers/scope/{scope}` | Yes |
| ProjectBase { project } | `refs/jin/layers/project/{project}` | Yes |
| UserLocal | `None` | No |
| WorkspaceActive | `None` | No |

### Error Mapping

| Operation | Error Condition | JinError Variant |
|-----------|----------------|------------------|
| `open()` | Repo doesn't exist | `RepoNotFound { path }` |
| `get_layer_ref()` | Layer is UserLocal/WorkspaceActive | `InvalidLayer { name }` |
| `create_layer_ref()` | Ref already exists | `RefExists { name, layer }` |
| Any git2 operation | Generic git2 error | `Git(#[from] git2::Error)` |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
