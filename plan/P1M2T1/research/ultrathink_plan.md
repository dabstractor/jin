# ULTRATHINK Plan for P1.M2.T1 PRP: Jin Repository Wrapper

## Overview
This plan outlines the structure and content strategy for the PRP document that will enable one-pass implementation of `JinRepo` - a wrapper around `git2::Repository` for Jin-specific operations.

## PRP Section Planning

### Goal Section
**Content Strategy:**
- Feature Goal: Create `JinRepo` wrapper that provides Jin-specific Git operations (layer refs, staging refs, Jin-specific object creation)
- Deliverable: `src/git/repo.rs` with complete `JinRepo` struct implementation
- Success Definition: All unit tests pass, wrapper can open/create Jin repos, manages layer refs correctly

**Key Differentiators from Generic Git Wrapper:**
- Jin-specific ref namespace (`refs/jin/layers/...`)
- Bare repository pattern (at `~/.jin/repo`)
- Layer-aware operations (get/set refs by Layer enum)
- Integration with existing `JinError` and `Layer` types

### Context Section

**YAML Structure Planning:**

```yaml
# Internal Documentation
- file: PRD.md
  why: Git Architecture & Invariants section (§5, §19)
  section: Lines 84-115 (Logical Branch Model), Lines 558-585 (Git and Environment)
  critical: Jin uses bare repos, logical refs under refs/jin/, no user-facing branches

- file: plan/docs/system_context.md
  why: Module structure and Git ref namespace specification
  section: Lines 103-116 (Git Ref Namespace)
  critical: refs/jin/layers/ prefix, ref format for each layer

- file: src/core/error.rs
  why: Error handling patterns to follow
  pattern: JinError::Git (transparent), JinError::RepoNotFound, JinError::RefNotFound
  gotcha: Use #[from] for git2::Error automatic conversion

- file: src/core/layer.rs
  why: Layer enum's git_ref() method provides exact ref format
  section: Lines 215-279 (Git Reference impl)
  critical: Call layer.git_ref() to get ref names, don't hardcode

- file: src/core/config.rs
  why: JinConfig.repository provides repo path
  section: Lines 82-111 (load method)
  gotcha: Repository path comes from config, may not exist yet (needs init)

- file: Cargo.toml
  why: Verify git2 dependency and features
  section: Line 18 (git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] })
  critical: vendored-libgit2 feature required for portability

# External Documentation
- url: https://docs.rs/git2/0.20/git2/
  why: Official git2-rs API documentation
  section: Repository struct, reference methods, blob/tree/commit creation
  critical: Repository::open(), Repository::init_bare(), find_reference(), reference()

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html
  why: Complete Repository API reference
  section: All methods we'll delegate or wrap
  critical: is_bare(), path(), workdir(), find_reference(), reference()

- url: https://github.com/rust-lang/git2-rs/blob/master/examples.rs
  why: Official examples showing common patterns
  section: Examples for repo init, ref creation, commit creation
  critical: Shows proper error handling patterns

# Research Documents (Created)
- docfile: plan/P1M2T1/research/git2_patterns.md
  why: 50+ code examples for all git2 operations we need
  section: Repository Initialization, Reference Management, Object Creation
  critical: Wrapper design patterns, error handling with JinError

- docfile: plan/P1M2T1/research/repo_wrapper_patterns.md
  why: Architectural patterns from real projects (GitButler, gitoxide)
  section: Simple Wrapper Pattern, Repository Context Pattern
  critical: Shows how to structure JinRepo with domain-specific helpers

- docfile: plan/P1M2T1/research/testing_patterns.md
  why: Complete testing patterns for git2 code
  section: TestRepoFixture, tempfile usage, error condition testing
  critical: Integration with existing tempfile dependency
```

### Current Codebase Tree
```bash
# Command to verify
tree -L 3 -I 'target|Cargo.lock' /home/dustin/projects/jin-glm-doover

# Key files to reference:
# - src/git/mod.rs (exists, needs to export JinRepo)
# - src/core/error.rs (has Git error variants)
# - src/core/layer.rs (has git_ref() method)
# - src/core/config.rs (JinConfig.repository path)
```

### Desired Codebase Tree
```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── git/
│       ├── mod.rs                  # MODIFY: Export JinRepo
│       └── repo.rs                 # CREATE: JinRepo wrapper implementation
└── tests/
    └── git/
        └── repo_test.rs            # CREATE: Unit tests for JinRepo
```

### Known Gotchas
```rust
// CRITICAL: Jin uses BARE repositories
// Jin repos at ~/.jin/repo have no working directory
// repo.workdir() returns None, repo.is_bare() returns true
// Do NOT call methods that require workdir (status(), checkout(), etc.)

// CRITICAL: Jin refs are LOGICAL, not branches
// refs/jin/layers/global is NOT a user-facing branch
// Never check out these refs, never show them to users
// Use repo.reference() to create/update, NOT repo.branch()

// CRITICAL: Layer enum provides ref names
// ALWAYS use layer.git_ref() to get ref names
// NEVER hardcode "refs/jin/layers/" strings
// Example: let ref_name = layer.git_ref().unwrap();

// CRITICAL: Error conversion pattern
// git2::Error converts transparently via JinError::Git
// Use ? operator: Repository::open(path)?
// Use explicit for context: .map_err(|e| JinError::RepoNotFound { path })?

// CRITICAL: Repository may not exist on first open
// JinConfig.repository path might point to non-existent repo
// Implement open_or_create() pattern to handle this case
// First call: Initialize bare repo if it doesn't exist

// GOTCHA: git2::Repository doesn't implement Clone
// Cannot directly clone JinRepo if it owns Repository
// Consider Arc<Mutex<>> if shared access needed (future requirement)

// GOTCHA: Reference names must be valid UTF-8
// Layer names (mode, scope, project) come from user input
// Validate before creating refs with those names

// PATTERN: Follow thiserror derive pattern from error.rs
// Use #[error(transparent)] + #[from] for automatic conversion
// Use structured variants for context: JinError::RepoNotFound { path }
```

## Implementation Blueprint Planning

### Data Models
```rust
// JinRepo struct - minimal wrapper with Jin-specific methods
pub struct JinRepo {
    inner: git2::Repository,
}

// Core operations to implement:
// 1. Constructor: open(), open_or_create(), init()
// 2. Layer ref operations: get_layer_ref(), set_layer_ref(), create_layer_ref()
// 3. Object creation helpers: create_blob(), create_tree(), create_commit()
// 4. Ref iteration: list_layer_refs()
// 5. Delegation: common git2 methods (head(), find_commit(), etc.)
```

### Implementation Tasks (Dependency Order)

```yaml
Task 1: CREATE src/git/repo.rs
  - IMPLEMENT: JinRepo struct with owned Repository
  - PATTERN: Follow "Simple Wrapper Pattern" from research
  - FIELDS:
    * pub(crate) inner: git2::Repository  (owned, not borrowed)
  - DERIVE: Debug, Clone (if using Arc, else omit Clone)
  - IMPORTS:
    * use crate::core::error::{JinError, Result}
    * use crate::core::Layer
    * use git2::Repository
  - PLACEMENT: New file src/git/repo.rs
  - NAMING: JinRepo (matches project naming: JinError, JinConfig)

Task 2: IMPLEMENT JinRepo::open()
  - IMPLEMENT: pub fn open(path: &Path) -> Result<Self>
  - PATTERN: Use Repository::open() with JinError::RepoNotFound conversion
  - CODE:
    ```rust
    pub fn open(path: &Path) -> Result<Self> {
        let inner = Repository::open(path)
            .map_err(|e| JinError::RepoNotFound {
                path: path.display().to_string(),
            })?;
        Ok(Self { inner })
    }
    ```
  - ERROR HANDLING: Convert git2::Error to JinError with path context
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 3: IMPLEMENT JinRepo::open_or_create()
  - IMPLEMENT: pub fn open_or_create(path: &Path) -> Result<Self>
  - PATTERN: Try open(), fall back to init_bare() if not found
  - CODE:
    ```rust
    pub fn open_or_create(path: &Path) -> Result<Self> {
        match Repository::open(path) {
            Ok(repo) => Ok(Self { inner: repo }),
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                let inner = Repository::init_bare(path)
                    .map_err(|e| JinError::Message(format!("Failed to init repo: {}", e)))?;
                Ok(Self { inner })
            }
            Err(e) => Err(JinError::from(e)),
        }
    }
    ```
  - GOTCHA: Must handle bare repo init differently than regular repo
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 2

Task 4: IMPLEMENT layer reference operations
  - IMPLEMENT: pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>>
  - IMPLEMENT: pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference>
  - PATTERN: Use layer.git_ref() to get ref name
  - CODE:
    ```rust
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

    pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        self.inner.reference(&ref_name, oid, false, &format!("Update layer: {:?}", layer))
            .map_err(JinError::from)
    }
    ```
  - CRITICAL: Use layer.git_ref(), never hardcode ref strings
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1, knowledge of Layer enum

Task 5: IMPLEMENT object creation helpers
  - IMPLEMENT: pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid>
  - IMPLEMENT: pub fn create_tree(&self, entries: &[TreeEntry]) -> Result<git2::Oid>
  - IMPLEMENT: pub fn create_commit(&self, ...) -> Result<git2::Oid>
  - PATTERN: Delegate to inner repository with JinError conversion
  - REUSE: Examples from git2_patterns.md (lines 220-356)
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 6: IMPLEMENT convenience delegation methods
  - IMPLEMENT: pub fn head(&self) -> Result<git2::Reference>
  - IMPLEMENT: pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit>
  - IMPLEMENT: pub fn find_tree(&self, oid: git2::Oid) -> Result<git2::Tree>
  - PATTERN: Simple delegation to inner with ? operator
  - CODE:
    ```rust
    pub fn head(&self) -> Result<git2::Reference> {
        Ok(self.inner.head()?)
    }

    pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit> {
        Ok(self.inner.find_commit(oid)?)
    }
    ```
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Task 1

Task 7: MODIFY src/git/mod.rs
  - ADD: pub mod repo;
  - ADD: pub use repo::JinRepo;
  - PRESERVE: Any existing exports
  - PLACEMENT: src/git/mod.rs
  - DEPENDENCIES: Task 1 (repo.rs must exist)

Task 8: CREATE tests/git/repo_test.rs
  - IMPLEMENT: Unit tests for all JinRepo methods
  - FIXTURE: TestRepoFixture pattern from testing_patterns.md
  - TESTS:
    * test_jinrepo_open_existing()
    * test_jinrepo_open_or_create_new()
    * test_jinrepo_get_layer_ref_not_found()
    * test_jinrepo_set_layer_ref()
    * test_jinrepo_create_blob()
    * test_jinrepo_error_handling()
  - FOLLOW: Pattern from testing_patterns.md (lines 67-158)
  - USE: tempfile for temp repo directories
  - PLACEMENT: tests/git/repo_test.rs (create tests/git/ directory)
  - DEPENDENCIES: Tasks 1-6

Task 9: VERIFY module exports
  - RUN: cargo check --package jin
  - VERIFY: JinRepo is accessible from crate::git::JinRepo
  - VERIFY: All methods compile without errors
  - DEPENDENCIES: All previous tasks
```

### Implementation Patterns Section

```rust
// ===== WRAPPER STRUCT PATTERN =====
// JinRepo owns the Repository (not borrowed)
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
}

// ===== LAYER REF PATTERN =====
// Always use layer.git_ref() to get ref names
impl JinRepo {
    pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>> {
        let ref_name = layer.git_ref()  // NOT: "refs/jin/layers/global"
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        match self.inner.find_reference(&ref_name) {
            Ok(r) => Ok(Some(r)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(JinError::from(e)),
        }
    }
}

// ===== BARE REPOSITORY PATTERN =====
// Jin repos are bare - no working directory operations
impl JinRepo {
    pub fn new_bare(path: &Path) -> Result<Self> {
        let inner = Repository::init_bare(path)?;
        // Do NOT call workdir() dependent methods
        Ok(Self { inner })
    }
}

// ===== ERROR HANDLING PATTERN =====
// Use ? for automatic conversion, explicit for context
impl JinRepo {
    pub fn find_commit(&self, oid: Oid) -> Result<git2::Commit> {
        // ? operator auto-converts via JinError::Git
        Ok(self.inner.find_commit(oid)?)
    }

    pub fn open(path: &Path) -> Result<Self> {
        // Explicit conversion for context
        let inner = Repository::open(path)
            .map_err(|e| JinError::RepoNotFound {
                path: path.display().to_string(),
            })?;
        Ok(Self { inner })
    }
}
```

## Validation Gates Planning

### Level 1: Syntax & Style
```bash
cargo check --package jin
cargo clippy --package jin -- -D warnings
cargo fmt --check
```

### Level 2: Unit Tests
```bash
cargo test --package jin --lib git::repo --verbose
cargo test --package jin --lib git::repo::tests::test_jinrepo_open_or_create -- --exact
```

### Level 3: Integration Testing
```bash
# Test actual repository operations
cargo test --package jin --test repo_test

# Manual verification of created refs
# (test will create temp repo, set layer ref, verify it exists)
```

### Level 4: Domain-Specific Validation
```bash
# Verify bare repository creation
# Test that refs are created under refs/jin/layers/
# Verify integration with Layer.git_ref()
# Test error paths (non-existent repo, invalid layer)
```

## Final Checklist Planning

### Technical Validation
- [ ] cargo build succeeds
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Module exports work (crate::git::JinRepo)

### Feature Validation
- [ ] Can open existing bare repo
- [ ] Can create new bare repo if doesn't exist
- [ ] Can get/set layer refs using Layer enum
- [ ] Integration with JinError works
- [ ] Integration with Layer.git_ref() works

### Code Quality
- [ ] Follows error.rs pattern
- [ ] Follows layer.rs type definition style
- [ ] Uses proper documentation comments
- [ ] Test coverage for all public methods

## Anti-Patterns to Avoid

- ❌ Don't hardcode "refs/jin/layers/" strings - use layer.git_ref()
- ❌ Don't use workdir() methods - Jin repos are bare
- ❌ Don't use branch() methods - Jin refs are not branches
- ❌ Don't skip error conversion - use JinError consistently
- ❌ Don't create checked-out branches - refs only
- ❌ Don't expose inner Repository directly - wrap methods needed
- ❌ Don't use regular Repository::init() - use init_bare()

## Confidence Score Estimation

**Estimated Confidence: 9/10**

**Reasoning:**
- Comprehensive research documents with 50+ code examples
- Existing error types perfectly match use case (JinError::Git, RepoNotFound, RefNotFound)
- Layer.git_ref() method provides exact ref format
- Clear architectural pattern from existing PRPs
- Testing patterns well-documented
- Only uncertainty: Future requirements may need Arc<Mutex<>> for shared access

**Risk Mitigation:**
- Document current owned Repository pattern
- Add note about future Arc<Mutex<>> if threading needed
- Keep implementation simple and focused on current requirements
