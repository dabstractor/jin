# Git2-RS Mocking and Testing Research

**Research Date:** 2026-01-10
**Focus:** Mocking `git2::Repository`, `Tree`, and `Tree::get_path()` operations for unit tests

---

## Executive Summary

Based on extensive research of git2-rs documentation, community discussions, and best practices, the recommended approach for testing git2 operations in Rust is:

1. **Use real git repositories with tempfile** for integration tests (idiomatic approach)
2. **Create trait abstractions** for unit testing with mock implementations
3. **Avoid direct mocking** of git2 types (they're foreign types without trait implementations)
4. **Leverage TreeBuilder** for creating test fixtures with specific tree structures

---

## Table of Contents

1. [Key API Documentation](#key-api-documentation)
2. [Testing Approaches](#testing-approaches)
3. [Test Fixture Patterns](#test-fixture-patterns)
4. [Mocking Strategies](#mocking-strategies)
5. [Code Examples](#code-examples)
6. [Best Practices](#best-practices)
7. [Additional Resources](#additional-resources)

---

## Key API Documentation

### 1. Tree API Documentation

**Source:** [Tree in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Tree.html)

The `Tree` struct represents a git tree object with the following key methods:

#### `get_path()` - The Core Method You Need to Test

```rust
pub fn get_path(&self, path: &Path) -> Result<TreeEntry<'static>, Error>
```

- **Purpose:** Retrieve a tree entry contained in a tree or any of its subtrees, given its relative path
- **Returns:** `Result<TreeEntry<'static>, Error>`
- **Key Insight:** Returns `'static` lifetime because the entry is owned
- **Error Handling:** Returns `Error` when path doesn't exist

#### Related Tree Methods

```rust
pub fn get_name(&self, filename: &str) -> Option<TreeEntry<'_>>  // Direct children only
pub fn get_name_bytes(&self, filename: &[u8]) -> Option<TreeEntry<'_>>  // Non-UTF8 names
pub fn get_id(&self, id: Oid) -> Option<TreeEntry<'_>>  // Lookup by SHA
pub fn get(&self, n: usize) -> Option<TreeEntry<'_>>  // By position
pub fn iter(&self) -> TreeIter<'_>  // Iterate all entries
pub fn walk(&self, mode: TreeWalkMode, callback: C) -> Result<(), Error>  // Traverse subtrees
```

### 2. TreeBuilder API Documentation

**Source:** [TreeBuilder in git2 - Rust](https://docs.rs/git2/latest/git2/struct.TreeBuilder.html)

**Source:** [treebuilder.rs source code](https://docs.rs/git2/latest/src/git2/treebuilder.rs.html)

The `TreeBuilder` struct is essential for creating test fixtures:

```rust
pub struct TreeBuilder<'repo>
```

#### Key Characteristics:

- **Handles one level of nested tree structure at a time**
- Each path passed to `insert()` must be a single component (no "/" in paths)
- For multi-level directories, you must create child trees first, then insert them into parent trees

#### Key Methods:

```rust
pub fn new(repo: &Repository) -> Result<TreeBuilder<'_>, Error>  // Create empty builder
pub fn insert(&mut self, filename: &str, id: Oid, filemode: i32) -> Result<(), Error>
pub fn remove(&mut self, filename: &str) -> Result<Option<ObjectType>, Error>
pub fn get(&self, filename: &str) -> Option<TreeEntry<'_>>
pub fn write(&self) -> Result<Oid, Error>  // Finalize and write to repository
pub fn is_empty(&self) -> bool
pub fn len(&self) -> usize
```

### 3. Repository API Documentation

**Source:** [Repository in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Repository.html)

```rust
pub struct Repository { /* private fields */ }
```

#### Key Methods for Testing:

```rust
pub fn init(path: &Path) -> Result<Repository, Error>
pub fn open(path: &Path) -> Result<Repository, Error>
pub fn find_tree(&self, oid: Oid) -> Result<Tree<'_>, Error>
pub fn treebuilder(&self) -> Result<TreeBuilder<'_>, Error>
pub fn blob(&self, data: &[u8]) -> Result<Oid, Error>  // Create blob from data
```

**Source:** [RepositoryInitOptions in git2 - Rust](https://docs.rs/git2/latest/git2/struct.RepositoryInitOptions.html)

For more control over repository initialization:

```rust
pub struct RepositoryInitOptions
```

---

## Testing Approaches

### Approach 1: Integration Tests with Real Repositories (RECOMMENDED)

**Rationale:** Git2 types are foreign types without trait implementations, making traditional mocking difficult. The idiomatic Rust approach is to use real repositories in temporary directories.

**Advantages:**
- Tests actual git2 behavior
- No need to maintain mock implementations
- Catches integration issues early
- Community-approved pattern

**Disadvantages:**
- Slower than pure mocks
- Requires file system operations

**Community Reference:**
- [Rust Users Forum - Run unit tests in isolated environment](https://users.rust-lang.org/t/run-unit-tests-in-isolated-docker-container-environment/97550)
- Mentions using `tempfile::TempDir` for `git2::Repository` tests

### Approach 2: Trait Abstraction with Mock Implementations

**Rationale:** Create a trait that abstracts git operations, then implement it for both real git2 operations and mock versions.

**Advantages:**
- Fast unit tests
- Predictable test scenarios
- Follows Rust idioms for testing

**Disadvantages:**
- Requires code refactoring
- Must maintain trait implementations
- Risk of drift between real and mock behavior

**Community References:**
- [Idiomatic Rust way of testing/mocking](https://users.rust-lang.org/t/idiomatic-rust-way-of-testing-mocking/128024) (April 2025)
- [How can I mock the repository layer? : r/rust](https://www.reddit.com/r/rust/comments/j3yzlc/how_can_i_mock_the_repository_layer/)
- Suggests using trait-based stub implementations rather than mocking
- Mentions `mockall` library for automated trait mocking

### Approach 3: git2-testing Crate

**Source:** [git2-testing - crates.io](https://crates.io/crates/git2-testing)

A dedicated helper crate that provides:
> "convenience functions on top of git2-rs for convenient unittest repository generation"

**Status:** Version 0.1.0 (early stage, limited documentation)

**Note:** This crate appears to be minimal and may not provide comprehensive mocking utilities.

---

## Test Fixture Patterns

### Pattern 1: Basic TempDir Repository

**Concept:** Create a temporary directory, initialize a git repository, use it for testing, and let it be automatically cleaned up.

**Key Crates:**
- [tempfile - crates.io](https://docs.rs/tempfile/) - Main tempfile crate
- [temp-dir - crates.io](https://crates.io/crates/temp-dir) - Alternative with simpler API
- [test_dir - crates.io](https://crates.io/crates/test_dir) - Test-specific directory utilities

**References:**
- [Stebalien/tempfile GitHub](https://github.com/Stebalien/tempfile)
- [StackOverflow - Testing Rust code with temp directories](https://stackoverflow.com/questions/72550689/testing-rust-code-by-using-temp-directory-files-while-avoiding-duplicate-code)
- [Testing in Rust: Temporary Files - ndrew's Blog](http://www.andrewra.dev/2019/03/01/testing-in-rust-temporary-files/)

**Example Pattern:**
```rust
use tempfile::TempDir;
use git2::Repository;

#[test]
fn test_git_operations() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Perform tests...

    // TempDir automatically cleaned up when dropped
}
```

### Pattern 2: Pre-populated Repository with Test Data

**Concept:** Create a repository with files, commits, and specific tree structures for testing.

**Workflow:**
1. Create temporary directory
2. Initialize repository
3. Create test files
4. Add files to index
5. Create commits
6. Test tree operations

**References:**
- [StackOverflow - Creating temp GIT repo for testing](https://stackoverflow.com/questions/64589807/creating-temp-git-repo-for-testing-is-not-working)
- [add and commit issues · Issue #561](https://github.com/rust-lang/git2-rs/issues/561) - Contains minimal example

### Pattern 3: TreeBuilder for Specific Tree Structures

**Concept:** Use TreeBuilder to create custom tree structures without using the file system.

**Use Case:** Testing `Tree::get_path()` with specific file/directory layouts.

**Key Reference:**
- [StackOverflow - LibGit2 treebuilder multi level insertion](https://stackoverflow.com/questions/47518544/libgit2-treebuilder-multi-level-insertion)

---

## Mocking Strategies

### Strategy 1: Trait-Based Abstraction (RECOMMENDED for Unit Tests)

**Pattern:**

```rust
// Define trait for operations you need
trait GitTreeAccess {
    fn file_exists(&self, path: &Path) -> Result<bool, Error>;
    fn get_file_id(&self, path: &Path) -> Result<Oid, Error>;
}

// Real implementation using git2
struct RealGitTree {
    tree: Tree<'static>,
}

impl GitTreeAccess for RealGitTree {
    fn file_exists(&self, path: &Path) -> Result<bool, Error> {
        match self.tree.get_path(path) {
            Ok(_) => Ok(true),
            Err(e) if e.class() == ErrorClass::Tree => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn get_file_id(&self, path: &Path) -> Result<Oid, Error> {
        let entry = self.tree.get_path(path)?;
        Ok(entry.id())
    }
}

// Mock implementation for testing
struct MockGitTree {
    files: HashMap<PathBuf, Oid>,
}

impl GitTreeAccess for MockGitTree {
    fn file_exists(&self, path: &Path) -> Result<bool, Error> {
        Ok(self.files.contains_key(path))
    }

    fn get_file_id(&self, path: &Path) -> Result<Oid, Error> {
        self.files.get(path)
            .copied()
            .ok_or_else(|| Error::from_str("file not found"))
    }
}
```

**References:**
- [Of Architecture, Traits and Unit Testing](https://users.rust-lang.org/t/of-architecture-traits-and-unit-testing/37287)
- [Apollo GraphQL Rust Best Practices](https://github.com/apollographql/rust-best-practices)

### Strategy 2: Mockall Crate for Automated Mocking

**Reference:** [How can I mock the repository layer?](https://www.reddit.com/r/rust/comments/j3yzlc/how_can_i_mock_the_repository_layer/)

```rust
use mockall::mock;

#[automock]
trait GitTreeAccess {
    fn file_exists(&self, path: &Path) -> Result<bool, Error>;
}

#[test]
fn test_with_mockall() {
    let mut mock = MockGitTreeAccess::new();
    mock.expect_file_exists()
        .with(predicate::eq(PathBuf::from("test.txt")))
        .returning(|_| Ok(true));

    // Test with mock...
}
```

### Strategy 3: In-Memory Test Repositories

**Concept:** Use git2's ability to work with repositories without cloning from remote.

**Pattern:**
```rust
// Create repository in memory or temp directory
// Use TreeBuilder to create structures
// Test operations
```

---

## Code Examples

### Example 1: Basic Test Setup with TempDir

```rust
use tempfile::TempDir;
use git2::{Repository, ObjectType};
use std::path::Path;

#[test]
fn test_tree_get_path() {
    // Setup: Create temporary repository
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Create test file and commit
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new("test.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();

    let tree = repo.find_tree(tree_id).unwrap();

    // Test: Check file exists
    let entry = tree.get_path(Path::new("test.txt")).unwrap();
    assert_eq!(entry.name(), Some("test.txt"));
    assert_eq!(entry.kind(), Some(ObjectType::Blob));

    // Test: Non-existent file
    let result = tree.get_path(Path::new("nonexistent.txt"));
    assert!(result.is_err());
}
```

### Example 2: Creating Tree Structure with TreeBuilder

```rust
use git2::{Repository, TreeBuilder, ObjectType};
use tempfile::TempDir;

#[test]
fn test_tree_with_nested_directories() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Create blob for file content
    let blob_id = repo.blob(b"file content").unwrap();

    // Create nested tree structure
    // Note: TreeBuilder only handles one level at a time

    // Create child tree (dir/subdir/)
    let mut child_builder = repo.treebuilder().unwrap();
    child_builder.insert("file.txt", blob_id, 0o100644).unwrap();
    let child_tree_id = child_builder.write().unwrap();

    // Create parent tree (dir/)
    let mut parent_builder = repo.treebuilder().unwrap();
    parent_builder.insert("subdir", child_tree_id, 0o040000).unwrap();
    let parent_tree_id = parent_builder.write().unwrap();

    // Create root tree
    let mut root_builder = repo.treebuilder().unwrap();
    root_builder.insert("dir", parent_tree_id, 0o040000).unwrap();
    let root_tree_id = root_builder.write().unwrap();

    // Test path lookup
    let root_tree = repo.find_tree(root_tree_id).unwrap();

    // Test nested path access
    let entry = root_tree.get_path(Path::new("dir/subdir/file.txt")).unwrap();
    assert_eq!(entry.name(), Some("file.txt"));
}
```

### Example 3: Testing File Existence Checks

```rust
use git2::{Repository, Error};
use tempfile::TempDir;
use std::path::Path;

fn file_exists_in_tree(tree: &git2::Tree, path: &Path) -> bool {
    tree.get_path(path).is_ok()
}

#[test]
fn test_file_existence_in_tree() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Setup: Create file
    let file_path = temp_dir.path().join("existing.txt");
    std::fs::write(&file_path, "content").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new("existing.txt")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    // Test: Existing file
    assert!(file_exists_in_tree(&tree, Path::new("existing.txt")));

    // Test: Non-existing file
    assert!(!file_exists_in_tree(&tree, Path::new("nonexistent.txt")));
}
```

### Example 4: Trait-Based Abstraction

```rust
use git2::{Repository, Oid, Error, Tree};
use std::path::Path;
use tempfile::TempDir;

// Trait abstraction
trait GitRepository {
    fn check_file_exists(&self, commit_oid: Oid, path: &Path) -> Result<bool, Error>;
}

// Real implementation
struct RealGitRepository {
    repo: Repository,
}

impl GitRepository for RealGitRepository {
    fn check_file_exists(&self, commit_oid: Oid, path: &Path) -> Result<bool, Error> {
        let commit = self.repo.find_commit(commit_oid)?;
        let tree = commit.tree()?;
        match tree.get_path(path) {
            Ok(_) => Ok(true),
            Err(e) if e.class() == git2::ErrorClass::Tree => Ok(false),
            Err(e) => Err(e),
        }
    }
}

// Mock implementation for testing
struct MockGitRepository {
    files: Vec<String>,
}

impl GitRepository for MockGitRepository {
    fn check_file_exists(&self, _commit_oid: Oid, path: &Path) -> Result<bool, Error> {
        Ok(self.files.iter().any(|f| Path::new(f) == path))
    }
}

#[test]
fn test_with_mock() {
    let mock = MockGitRepository {
        files: vec!["file1.txt".to_string(), "dir/file2.txt".to_string()],
    };

    assert!(mock.check_file_exists(Oid::zero(), Path::new("file1.txt")).unwrap());
    assert!(!mock.check_file_exists(Oid::zero(), Path::new("nonexistent.txt")).unwrap());
}
```

---

## Best Practices

### 1. Choose the Right Testing Approach

**Use Integration Tests (Real Repositories) when:**
- Testing actual git2 behavior
- File system performance is acceptable
- You need to verify git operations work correctly

**Use Unit Tests (Mock/Trait Abstraction) when:**
- Testing business logic that uses git operations
- Fast test execution is critical
- You need to test edge cases and error conditions

### 2. Follow Rust Idioms for Testing

**References:**
- [Idiomatic Rust way of testing/mocking](https://users.rust-lang.org/t/idiomatic-rust-way-of-testing-mocking/128024)
- [Rust Testing Mastery: From Basics to Best Practices](https://blog.devgenius.io/rust-testing-mastery-from-basics-to-best-practices-c6b37bb214f)

**Key Principles:**
- Prefer trait-based stub implementations over mocking frameworks
- Use dependency injection to provide real or mock implementations
- Keep test fixtures simple and focused
- Avoid mocking foreign types (like git2 types) directly

### 3. TreeBuilder Multi-Level Pattern

**Reference:** [StackOverflow - LibGit2 treebuilder multi level insertion](https://stackoverflow.com/questions/47518544/libgit2-treebuilder-multi-level-insertion)

**Pattern for Creating Nested Trees:**
1. Create child TreeBuilder (deepest level)
2. Insert blobs into child
3. Write child tree, get Oid
4. Create parent TreeBuilder
5. Insert child tree Oid into parent
6. Repeat for each level

### 4. Error Handling in Tests

**Pattern:**
```rust
// Good: Explicit error checking
let result = tree.get_path(Path::new("file.txt"));
assert!(result.is_ok());

// Good: Check specific error types
let result = tree.get_path(Path::new("nonexistent.txt"));
match result {
    Err(e) if e.class() == git2::ErrorClass::Tree => {
        // Expected: file not found
    }
    Err(e) => panic!("Unexpected error: {:?}", e),
    Ok(_) => panic!("Expected error for non-existent file"),
}
```

### 5. Test Organization

**Reference:** [Testing - Command Line Applications in Rust](https://rust-cli.github.io/book/tutorial/testing.html)

**Structure:**
```
tests/
  ├── integration/
  │   ├── git_operations_tests.rs  # Tests with real repos
  │   └── tree_tests.rs
  └── unit/
      ├── tree_access_tests.rs  # Tests with mocks
      └── file_checker_tests.rs
```

### 6. Performance Considerations

**Tips:**
- Use `cargo test`'s test parallelization
- Put slow integration tests in `tests/` directory (run separately)
- Use `#[serial]` attribute for tests that need exclusive repo access
- Cache repository setup in test fixtures

---

## Additional Resources

### Documentation

1. **[git2 - docs.rs](https://docs.rs/git2)** - Main crate documentation
2. **[Tree in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Tree.html)** - Complete Tree API
3. **[TreeBuilder in git2 - Rust](https://docs.rs/git2/latest/git2/struct.TreeBuilder.html)** - TreeBuilder API
4. **[Repository in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Repository.html)** - Repository API
5. **[treebuilder.rs source](https://docs.rs/git2/latest/src/git2/treebuilder.rs.html)** - Implementation details

### Testing Libraries

1. **[tempfile - docs.rs](https://docs.rs/tempfile/)** - Temporary file/directory creation
2. **[temp-dir - crates.io](https://crates.io/crates/temp-dir)** - Simplified temp directories
3. **[test_dir - crates.io](https://crates.io/crates/test_dir)** - Test-specific utilities
4. **[assert_fs - docs.rs](https://docs.rs/assert_fs/latest/assert_fs/fixture/index.html)** - Filesystem fixtures
5. **[git2-testing - crates.io](https://crates.io/crates/git2-testing)** - Git2 test helpers

### Community Discussions

1. **[Idiomatic Rust way of testing/mocking](https://users.rust-lang.org/t/idiomatic-rust-way-of-testing-mocking/128024)** - April 2025 discussion on testing patterns
2. **[How can I mock the repository layer?](https://www.reddit.com/r/rust/comments/j3yzlc/how_can_i_mock_the_repository_layer/)** - Reddit discussion on mocking
3. **[Of Architecture, Traits and Unit Testing](https://users.rust-lang.org/t/of-architecture-traits-and-unit-testing/37287)** - Trait-based testing architecture
4. **[Run unit tests in isolated environment](https://users.rust-lang.org/t/run-unit-tests-in-isolated-docker-container-environment/97550)** - Mentions git2 + TempDir pattern

### Code Examples

1. **[add and commit issues · Issue #561](https://github.com/rust-lang/git2-rs/issues/561)** - Minimal add/commit example
2. **[git2-rs/examples/log.rs](https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs)** - Commit tree operations
3. **[StackOverflow - Creating temp GIT repo for testing](https://stackoverflow.com/questions/64589807/creating-temp-git-repo-for-testing-is-not-working)** - TempDir + git2 pattern
4. **[StackOverflow - LibGit2 treebuilder multi level insertion](https://stackoverflow.com/questions/47518544/libgit2-treebuilder-multi-level-insertion)** - Multi-level tree creation
5. **[git2_credentials/README.md](https://github.com/davidB/git2_credentials/blob/master/README.md)** - Testing patterns

### Best Practices

1. **[Apollo GraphQL Rust Best Practices](https://github.com/apollographql/rust-best-practices)** - General Rust patterns
2. **[Mastering Rust Traits: 15 Practical Examples](https://medium.com/rust-rock/mastering-rust-traits-15-practical-examples-that-will-transform-your-code-0c34f8558a67)** - Trait patterns
3. **[Rust Testing Mastery: From Basics to Best Practices](https://blog.devgenius.io/rust-testing-mastery-from-basics-to-best-practices-c6b37bb214f)** - Testing guide
4. **[Testing - Command Line Applications in Rust](https://rust-cli.github.io/book/tutorial/testing.html)** - CLI testing patterns

### Tutorial Articles

1. **[Create your own GitOps controller with Rust](https://itnext.io/create-your-own-gitops-controller-with-rust-70b6d077e2d0)** - Practical git2 usage
2. **[git2-rs：Rust语言下的Git库教程](https://blog.csdn.net/gitblog_01081/article/details/141451600)** - Chinese tutorial
3. **[How to use git2::Remote::push correctly?](https://users.rust-lang.org/t/how-to-use-git2-remotepush-correctly/97202)** - git2 operations

---

## Key Takeaways

### For Mocking `git2::Repository`:

1. **Don't try to mock directly** - git2 types are foreign types without trait implementations
2. **Create trait abstractions** around the operations you need
3. **Use tempfile + Repository::init()** for integration tests
4. **Consider git2-testing crate** for test helpers (though it's early-stage)

### For Mocking `Tree` and `Tree::get_path()`:

1. **Use TreeBuilder** to create test trees with specific structures
2. **Create real trees in temp repos** for integration tests
3. **Wrap in trait abstraction** for unit tests with mock implementations
4. **Remember TreeBuilder limitation** - only handles one level at a time

### For File Existence Checks:

1. **Integration test pattern:** Create repo → Add files → Test `tree.get_path()` → Check Result
2. **Unit test pattern:** Create trait with `file_exists()` → Mock with HashMap → Test logic
3. **Error handling:** Check `ErrorClass::Tree` for "not found" errors

### Testing Pattern Recommendation:

```rust
// File: tests/git_tree_tests.rs

use tempfile::TempDir;
use git2::Repository;
use std::path::Path;

#[test]
fn test_file_exists_in_tree() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let repo = setup_test_repo(&temp_dir);

    // Create test data
    let tree = create_test_tree(&repo, vec!["file1.txt", "dir/file2.txt"]);

    // Test
    assert!(tree.get_path(Path::new("file1.txt")).is_ok());
    assert!(tree.get_path(Path::new("dir/file2.txt")).is_ok());
    assert!(tree.get_path(Path::new("nonexistent.txt")).is_err());
}

fn setup_test_repo(temp_dir: &TempDir) -> Repository {
    Repository::init(temp_dir.path()).unwrap()
}

fn create_test_tree(repo: &Repository, files: Vec<&str>) -> git2::Tree {
    // Implementation using TreeBuilder...
    // See Example 2 above
}
```

---

## Appendix: Common Pitfalls

### Pitfall 1: Trying to Mock Foreign Types Directly

```rust
// DON'T DO THIS - Won't work
use mockall::mock;
mock! {
    pub Tree {}  // Error: Tree is a foreign type
}

// DO THIS - Use trait abstraction
trait TreeAccess {
    fn get_entry(&self, path: &Path) -> Result<Option<TreeEntry>, Error>;
}
```

### Pitfall 2: Multi-Level TreeBuilder Usage

```rust
// DON'T DO THIS - TreeBuilder doesn't handle "/" in paths
let mut builder = repo.treebuilder().unwrap();
builder.insert("dir/file.txt", blob_id, 0o100644).unwrap();  // Error!

// DO THIS - Create child tree first
let mut child = repo.treebuilder().unwrap();
child.insert("file.txt", blob_id, 0o100644).unwrap();
let child_id = child.write().unwrap();

let mut parent = repo.treebuilder().unwrap();
parent.insert("dir", child_id, 0o040000).unwrap();
```

### Pitfall 3: Ignoring Error Types

```rust
// DON'T DO THIS - Loses error information
let exists = tree.get_path(path).is_ok();

// DO THIS - Check specific error types
let exists = match tree.get_path(path) {
    Ok(_) => true,
    Err(e) if e.class() == git2::ErrorClass::Tree => false,
    Err(e) => return Err(e),  // Propagate unexpected errors
};
```

---

## Conclusion

The research indicates that **mocking git2 operations in Rust follows a different pattern than traditional mocking** due to git2 being a foreign library with C bindings. The recommended approaches are:

1. **Integration tests with real repositories** using `tempfile` - most idiomatic
2. **Trait-based abstractions** with mock implementations - for fast unit tests
3. **TreeBuilder** for creating specific tree structures in tests

The key insight from the community discussions is that **Rust favors trait-based stub implementations over runtime mocking frameworks**, especially when working with foreign types like git2.

---

**Document Version:** 1.0
**Last Updated:** 2026-01-10
**Research Coverage:** git2-rs 0.20.x, Rust testing ecosystem 2025-2026
