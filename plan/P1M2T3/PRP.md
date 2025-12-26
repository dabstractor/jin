# Product Requirement Prompt (PRP): Git Object Creation (P1.M2.T3)

---

## Goal

**Feature Goal**: Provide object creation helpers for blob, tree, and commit Git objects that layer content creation operations can use when staging and committing changes to layer references.

**Deliverable**: Extended `src/git/repo.rs` module with object creation helpers:
- `create_blob()` - Create blob objects from byte content
- `create_tree()` - Write tree builder to create tree objects
- `create_commit()` - Create commit objects with metadata
- Comprehensive unit tests for all object creation operations

**Success Definition**:
- `cargo build` compiles with zero errors
- All unit tests pass with isolated test repositories
- Blob objects can be created from content
- Tree objects can be created from tree builders
- Commit objects can be created with proper signatures and parent commits
- Integration with existing `JinError` types
- Object IDs can be used in layer reference updates

## User Persona

**Target User**: AI coding agent implementing Jin's Git object creation layer

**Use Case**: The agent needs to establish object creation operations that:
- Enable creating blob objects from file content for layer storage
- Enable creating tree objects from file/directory hierarchies
- Enable creating commit objects for layer versioning
- Provide proper error handling and JinError integration

**User Journey**:
1. Agent receives this PRP as context
2. Implements blob creation helper
3. Implements tree creation helper
4. Implements commit creation helper with full parameter support
5. Adds comprehensive unit tests
6. Validates compilation and test success

**Pain Points Addressed**:
- No manual git2 object API calls needed
- Consistent error handling with `JinError` integration
- Clear delegation pattern to underlying git2 Repository
- Support for multi-parent commits (future merge support)

## Why

- **Layer Versioning Foundation**: P1.M3 (Transaction System) needs object creation for commits
- **Staging System**: P3.M1 (Staging System) needs blob/tree creation for staging files
- **Content Storage**: Layer content must be stored as Git objects for versioning
- **Problems this solves**:
  - No consistent interface for creating Git objects
  - Direct git2 calls scattered throughout codebase
  - Error handling inconsistencies with different object types
  - No abstraction for commit creation with Jin-specific patterns

## What

Implement object creation helper methods in `JinRepo` that wrap git2's object creation API with consistent error handling and Jin-specific patterns.

### Success Criteria

- [ ] `create_blob()` method implemented and tested
- [ ] `create_tree()` method implemented and tested
- [ ] `create_commit()` method implemented with full parameter support
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover all public methods
- [ ] `cargo test` passes all tests
- [ ] Object IDs can be used with layer reference operations

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact method specifications with all parameters
- Research documents with code examples for all operations
- Specific patterns from existing codebase to follow
- Complete integration guide with `Layer` and `JinError` types
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Git Architecture specification - objects, commits, layer refs
  section: Lines 84-115 for Logical Branch Model, Lines 558-585 for Git and Environment
  critical: Jin uses Git objects for layer content storage

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Git ref namespace and 9-layer hierarchy
  section: Lines 103-116 for Git Ref Namespace format
  critical: Objects are referenced by layer refs under refs/jin/layers/

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: Existing JinRepo implementation - add object creation methods here
  section: Lines 600-688 for existing Object Creation Helpers section
  critical: Follow existing delegation pattern with ? operator

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  section: Lines 30-33 for JinError::Git (transparent error)
  critical: Use #[from] for automatic git2::Error conversion

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum's git_ref() method for storing object IDs
  section: Lines 215-279 for git_ref() implementation
  critical: Created object IDs will be stored in layer references

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T3/research/git2_object_creation.md
  why: Complete git2-rs object creation patterns with code examples
  section: Blob Creation (lines 7-89), Tree Creation (lines 91-213), Commit Creation (lines 215-367)
  critical: Shows exact blob(), treebuilder(), commit() method signatures

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T3/research/testing_patterns.md
  why: Testing patterns from repo.rs for consistent test writing
  section: TestFixture Pattern (lines 5-48), Object Testing (lines 89-157)
  critical: Integration with existing tempfile usage, commit testing patterns

# EXTERNAL - git2-rs Documentation

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.blob
  why: Blob creation method - core of create_blob implementation
  critical: pub fn blob(&self, data: &[u8]) -> Result<Oid>

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.treebuilder
  why: Tree builder creation - used before create_tree
  critical: pub fn treebuilder(&self, iter: Option<&[TreeEntry]>) -> Result<TreeBuilder>

- url: https://docs.rs/git2/0.20/git2/struct.TreeBuilder.html#method.write
  why: Tree builder write - core of create_tree implementation
  critical: pub fn write(&mut self) -> Result<Oid>

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.commit
  why: Commit creation method - core of create_commit implementation
  critical: Full parameter list for author, committer, message, tree, parents

- url: https://docs.rs/git2/0.20/git2/struct.Signature.html#method.now
  why: Signature creation for commit author/committer
  critical: pub fn now(name: &str, email: &str) -> Result<Signature>

- url: https://github.com/rust-lang/git2-rs/blob/master/examples.rs
  why: Official examples showing object creation
  section: Examples for blob, tree, and commit creation
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── Cargo.toml                      # Has git2 dependency with features
├── PRD.md
├── src/
│   ├── core/
│   │   ├── error.rs               # Has JinError::Git (transparent)
│   │   ├── layer.rs               # Has Layer enum with git_ref() method
│   │   └── config.rs
│   └── git/
│       ├── mod.rs                 # Exports JinRepo
│       └── repo.rs                # Has Object Creation Helpers section (lines 600-688)
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Modified

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── git/
│       └── repo.rs                # MODIFY: Object Creation Helpers section already exists
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Object creation delegates to git2 with simple wrapper pattern
// The helpers don't add business logic - they provide consistent error handling
// Pattern: Wrap underlying git2 call with ? operator for auto-conversion

// CRITICAL: Blob creation takes byte slice (&[u8])
// Use .as_bytes() for strings, or read file bytes for file content
// Example:
//   repo.create_blob(content.as_bytes())?
//   repo.create_blob(&std::fs::read(path)?)?

// CRITICAL: Tree creation uses TreeBuilder pattern
// TreeBuilder is created separately, then passed to create_tree()
// Pattern:
//   let mut builder = repo.treebuilder()?;
//   builder.insert("file.txt", blob_oid, file_mode)?;
//   let tree_oid = repo.create_tree(&mut builder)?;
// Note: &mut builder required for write()

// CRITICAL: Commit creation requires git2::Signature for author/committer
// Use Signature::now() to create with current timestamp
// Example:
//   let sig = repo.signature("User", "user@example.com")?;
// Or use git2::Signature::now() directly

// CRITICAL: Commit creation takes slice of parent commit references
// Empty slice &[] for root commits
// Multiple parents for merge commits
// Pattern:
//   let parents: &[&git2::Commit] = &[];
//   let parents: &[&git2::Commit] = &[&parent_commit];
//   let parents: &[&git2::Commit] = &[&parent1, &parent2]; // merge

// GOTCHA: update_ref parameter can be None
// None = don't update any ref (create dangling commit)
// Some("HEAD") = update HEAD (not used in Jin for layer refs)
// Jin pattern: Use None, then call set_layer_ref() separately

// GOTCHA: Tree object reference required, not OID
// Must call repo.find_tree(tree_oid)? to get Tree reference
// Cannot pass OID directly to commit()

// GOTCHA: Parent commits require Commit references, not OIDs
// Must call repo.find_commit(parent_oid)? for each parent
// Build Vec<&git2::Commit> or use array slice

// PATTERN: Follow existing delegation pattern in repo.rs
// Other methods like find_commit(), find_tree() use simple delegation
// Object creation should follow same pattern:
//   pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> {
//       Ok(self.inner.blob(data)?)
//   }

// PATTERN: TreeBuilder is created by treebuilder() method, not create_tree()
// create_tree() only writes the builder to get OID
// Separation of concerns:
//   - treebuilder() creates the builder (delegation method)
//   - create_tree() writes it (object creation method)
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// No new data models - extending existing JinRepo

// Methods to add to JinRepo (Object Creation Helpers section):
impl JinRepo {
    // Blob creation
    pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid>;

    // Tree creation (writes TreeBuilder)
    pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid>;

    // Commit creation (full parameter support)
    pub fn create_commit(
        &self,
        update_ref: Option<&str>,
        author: &git2::Signature,
        committer: &git2::Signature,
        message: &str,
        tree: &git2::Tree,
        parents: &[&git2::Commit],
    ) -> Result<git2::Oid>;
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY existing create_blob() implementation
  - CHECK: pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> exists
  - VERIFY: Implementation wraps self.inner.blob(data)?
  - ENSURE: Method is in Object Creation Helpers section (around line 619)
  - PATTERN: Simple delegation with ? operator
  - CODE:
    pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> {
        Ok(self.inner.blob(data)?)
    }
  - DEPENDENCIES: None

Task 2: VERIFY existing create_tree() implementation
  - CHECK: pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid> exists
  - VERIFY: Implementation wraps builder.write()?
  - ENSURE: Method is in Object Creation Helpers section (around line 641)
  - PATTERN: Delegation to TreeBuilder::write()
  - CODE:
    pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid> {
        Ok(builder.write()?)
    }
  - GOTCHA: Takes &mut TreeBuilder, not &Repository
  - DEPENDENCIES: None

Task 3: VERIFY existing create_commit() implementation
  - CHECK: pub fn create_commit() with full parameter list exists
  - VERIFY: All parameters match git2::Repository::commit() signature
  - ENSURE: Method is in Object Creation Helpers section (around line 675)
  - PATTERN: Delegation to self.inner.commit()
  - CODE:
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
  - PARAMETERS:
    * update_ref: Option<&str> - ref to update (None for layer commits)
    * author: &git2::Signature - author signature
    * committer: &git2::Signature - committer signature
    * message: &str - commit message
    * tree: &git2::Tree - tree object (not OID)
    * parents: &[&git2::Commit] - parent commits slice
  - DEPENDENCIES: None

Task 4: VERIFY unit tests for create_blob()
  - CHECK: test_jinrepo_create_blob() exists in tests module
  - VERIFY: Test creates blob from byte content
  - VERIFY: Test reads back blob and verifies content
  - PATTERN: Follow testing_patterns.md lines 89-121
  - CODE:
    #[test]
    fn test_jinrepo_create_blob() {
        let fixture = TestFixture::new();
        let data = b"Hello, World!";

        let blob_oid = fixture.repo.create_blob(data).unwrap();
        let blob = fixture.repo.find_blob(blob_oid).unwrap();

        assert_eq!(blob.content(), data);
    }
  - PLACEMENT: tests module in repo.rs (around line 1037)
  - DEPENDENCIES: Tasks 1-3

Task 5: VERIFY unit tests for create_tree()
  - CHECK: test_jinrepo_create_tree() exists in tests module
  - VERIFY: Test creates blob, then tree with that blob
  - VERIFY: Test verifies tree has correct number of entries
  - PATTERN: Follow testing_patterns.md lines 123-157
  - CODE:
    #[test]
    fn test_jinrepo_create_tree() {
        let fixture = TestFixture::new();
        let blob_oid = fixture.repo.create_blob(b"content").unwrap();

        let mut builder = fixture.repo.treebuilder().unwrap();
        builder.insert("file.txt", blob_oid, git2::FileMode::Blob.into()).unwrap();

        let tree_oid = fixture.repo.create_tree(&mut builder).unwrap();
        let tree = fixture.repo.find_tree(tree_oid).unwrap();

        assert_eq!(tree.len(), 1);
    }
  - PLACEMENT: tests module in repo.rs (around line 1048)
  - DEPENDENCIES: Tasks 1-4

Task 6: VERIFY unit tests for create_commit()
  - CHECK: test_jinrepo_create_commit() exists in tests module
  - VERIFY: Test creates commit with tree and signature
  - VERIFY: Test verifies commit message
  - PATTERN: Follow testing_patterns.md lines 159-213
  - CODE:
    #[test]
    fn test_jinrepo_create_commit() {
        let fixture = TestFixture::new();
        let tree_builder = fixture.repo.treebuilder().unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = fixture.repo.find_tree(tree_oid).unwrap();

        let author = fixture.repo.signature("Test Author", "test@example.com").unwrap();
        let committer = &author;

        let commit_oid = fixture.repo.create_commit(
            Some("HEAD"),
            &author,
            committer,
            "Test commit",
            &tree,
            &[],
        ).unwrap();

        let commit = fixture.repo.find_commit(commit_oid).unwrap();
        assert_eq!(commit.message().unwrap(), "Test commit");
    }
  - PLACEMENT: tests module in repo.rs (around line 1064)
  - DEPENDENCIES: Tasks 1-5
```

### Implementation Patterns & Key Details

```rust
// ===== BLOB CREATION PATTERN =====
// Simple delegation - no business logic
impl JinRepo {
    pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> {
        Ok(self.inner.blob(data)?)
    }
}

// ===== TREE CREATION PATTERN =====
// Delegates to TreeBuilder::write()
impl JinRepo {
    pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid> {
        Ok(builder.write()?)
    }
}

// Usage example:
// let mut builder = repo.treebuilder()?;
// builder.insert("file.txt", blob_oid, git2::FileMode::Blob.into())?;
// let tree_oid = repo.create_tree(&mut builder)?;

// ===== COMMIT CREATION PATTERN =====
// Full delegation to Repository::commit()
impl JinRepo {
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

// Usage example (root commit):
// let tree = repo.find_tree(tree_oid)?;
// let sig = repo.signature("User", "user@example.com")?;
// let commit_oid = repo.create_commit(
//     None,  // Don't update refs
//     &sig,
//     &sig,
//     "Initial commit",
//     &tree,
//     &[],  // No parents
// )?;

// Usage example (with parent):
// let parent = repo.find_commit(parent_oid)?;
// let commit_oid = repo.create_commit(
//     None,
//     &sig,
//     &sig,
//     "Second commit",
//     &tree,
//     &[&parent],  // One parent
// )?;

// Usage example (merge commit):
// let parent1 = repo.find_commit(parent1_oid)?;
// let parent2 = repo.find_commit(parent2_oid)?;
// let commit_oid = repo.create_commit(
//     None,
//     &sig,
//     &sig,
//     "Merge branch",
//     &tree,
//     &[&parent1, &parent2],  // Two parents
// )?;

// ===== SIGNATURE CREATION PATTERN =====
// Use helper method or git2::Signature::now()
impl JinRepo {
    pub fn signature(&self, name: &str, email: &str) -> Result<git2::Signature> {
        Ok(git2::Signature::now(name, email)?)
    }
}

// ===== LAYER REF INTEGRATION PATTERN =====
// After creating commit, store OID in layer ref
// let layer = Layer::GlobalBase;
// let commit_oid = repo.create_commit(...)?;
// repo.set_layer_ref(&layer, commit_oid)?;
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs
  - pattern: JinError::Git (transparent) - automatic via #[from]
  - All git2::Error auto-converts through ? operator

LAYER_INTEGRATION:
  - use: src/core/layer.rs
  - method: layer.git_ref() for storing commit OIDs
  - pattern:
    * Create commit with create_commit()
    * Get commit OID
    * Store in layer ref with set_layer_ref()

SIGNATURE_CREATION:
  - use: repo.signature() helper method
  - or: git2::Signature::now() directly
  - pattern:
    * let sig = repo.signature("User", "user@example.com")?;
    * let sig = git2::Signature::now("User", "user@example.com")?;

TREE_BUILDER:
  - use: repo.treebuilder() delegation method
  - pattern:
    * let mut builder = repo.treebuilder()?;
    * builder.insert(path, blob_oid, filemode)?;
    * let tree_oid = repo.create_tree(&mut builder)?;

TRANSACTION_SYSTEM (FUTURE):
  - P1.M3: Will use create_blob(), create_tree(), create_commit()
  - Will create commits for staged changes
  - Will update staging refs, then layer refs on commit

STAGING_SYSTEM (FUTURE):
  - P3.M1: Will use create_blob() for file content
  - Will use create_tree() for directory structures
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modifying repo.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test JinRepo module specifically
cargo test --package jin --lib git::repo::tests::test_jinrepo_create --verbose

# Run specific object creation tests
cargo test --package jin test_jinrepo_create_blob -- --exact
cargo test --package jin test_jinrepo_create_tree -- --exact
cargo test --package jin test_jinrepo_create_commit -- --exact

# Expected: All tests pass. Look for:
# - test_jinrepo_create_blob: Verifies blob creation and content
# - test_jinrepo_create_tree: Verifies tree creation with entries
# - test_jinrepo_create_commit: Verifies commit with signature and message
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual object operations with real git2
cargo test --package jin --lib git::repo --verbose

# Manual verification of object creation:
# After test creates blob/tree/commit, verify they exist
# Objects should be readable via find_blob/find_tree/find_commit

# Expected:
# - Blob content matches what was written
# - Tree has correct number of entries
# - Commit has correct message and metadata
```

### Level 4: Domain-Specific Validation

```bash
# Verify blob content round-trip
cargo test --package jin test_jinrepo_create_blob -- --exact
# Asserts: blob.content() == original data

# Verify tree entry count
cargo test --package jin test_jinrepo_create_tree -- --exact
# Asserts: tree.len() == number of inserted entries

# Verify commit message and author
cargo test --package jin test_jinrepo_create_commit -- --exact
# Asserts: commit.message() == expected message

# Expected: All Jin-specific requirements met
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all new public methods

### Feature Validation

- [ ] `create_blob()` creates blob objects from byte content
- [ ] `create_tree()` writes tree builder to create tree objects
- [ ] `create_commit()` creates commits with all parameters
- [ ] Object IDs are valid and can be read back
- [ ] Integration with `JinError::Git` (transparent conversion)

### Code Quality Validation

- [ ] Follows existing repo.rs patterns
- [ ] Uses delegation pattern with ? operator
- [ ] Error handling matches existing patterns
- [ ] Test coverage for all public methods
- [ ] Tests follow testing_patterns.md conventions

### Documentation & Deployment

- [ ] All public methods have doc comments
- [ ] Examples in doc comments where helpful
- [ ] Gotchas documented (mut builder, signature creation)

---

## Anti-Patterns to Avoid

- Don't add business logic to object creation helpers - keep them simple wrappers
- Don't forget &mut for TreeBuilder in create_tree() - compile error
- Don't pass OIDs directly to create_commit() - need object references
- Don't forget to convert OIDs to objects before commit creation
- Don't use update_ref for layer commits - use None, call set_layer_ref() separately
- Don't create signatures manually - use Signature::now() or repo.signature()
- Don't skip testing content round-trip (blob content, tree entries, commit message)
- Don't ignore empty parent slice for root commits - &[] is valid

---

## Appendix: Quick Reference

### Object Creation API Summary

```rust
// Blob creation
pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid>

// Tree creation
pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid>

// Commit creation
pub fn create_commit(
    &self,
    update_ref: Option<&str>,
    author: &git2::Signature,
    committer: &git2::Signature,
    message: &str,
    tree: &git2::Tree,
    parents: &[&git2::Commit],
) -> Result<git2::Oid>
```

### git2 Object Creation Summary

| Object Type | Creation Method | Input | Returns |
|-------------|----------------|-------|---------|
| Blob | `repo.blob(data)` | `&[u8]` | `Oid` |
| Tree | `builder.write()` | `&mut TreeBuilder` | `Oid` |
| Commit | `repo.commit(...)` | See signature | `Oid` |

### Typical Workflow

```rust
// 1. Create blob from content
let blob_oid = repo.create_blob(b"file content")?;

// 2. Create tree with blob entry
let mut builder = repo.treebuilder()?;
builder.insert("file.txt", blob_oid, git2::FileMode::Blob.into())?;
let tree_oid = repo.create_tree(&mut builder)?;

// 3. Create commit with tree
let tree = repo.find_tree(tree_oid)?;
let sig = repo.signature("User", "user@example.com")?;
let commit_oid = repo.create_commit(None, &sig, &sig, "Message", &tree, &[])?;

// 4. Store in layer ref
repo.set_layer_ref(&Layer::GlobalBase, commit_oid)?;
```

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 10/10 - Implementation already exists, just documentation and verification needed
