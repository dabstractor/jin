# Testing Patterns for Git Reference Operations

This document documents the testing patterns used in `/home/dustin/projects/jin-glm-doover/src/git/repo.rs` to maintain consistency when writing tests for new reference operations (delete, list, iterate).

## 1. TestFixture Pattern

### Complete Implementation

```rust
struct TestFixture {
    _temp_dir: TempDir,
    repo: JinRepo,
}

impl TestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        Self {
            _temp_dir: temp_dir,
            repo,
        }
    }

    fn create_initial_commit(&self) -> git2::Oid {
        let tree_builder = self.repo.treebuilder().unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = self.repo.find_tree(tree_oid).unwrap();

        let author = self
            .repo
            .signature("Test Author", "test@example.com")
            .unwrap();
        let committer = &author;

        self.repo
            .create_commit(
                Some("HEAD"),
                &author,
                committer,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap()
    }
}
```

### Key Features:
- **TempDir**: `_temp_dir: TempDir` ensures automatic cleanup
- **Owned Repository**: `repo: JinRepo` owns the repository for the test duration
- **Convenience Method**: `create_initial_commit()` creates a minimal commit for reference operations
- **Simple Construction**: `new()` creates a fresh bare repository

## 2. Test Naming Conventions

Tests follow this pattern: `test_[module]_[operation]_[condition]`

### Examples from existing code:
- `test_jinrepo_init_creates_bare_repo`
- `test_jinrepo_get_layer_ref_not_found`
- `test_jinrepo_set_layer_ref_updates`
- `test_jinrepo_create_layer_ref_fails_if_exists`
- `test_jinrepo_unversioned_layers_error`

### Naming rules:
- Start with `test_`
- Include module name (`jinrepo`)
- Use operation name (`get_layer_ref`, `set_layer_ref`)
- Describe the test condition (`not_found`, `updates`, `fails_if_exists`)

## 3. Assertion Patterns

### Successful Operations
```rust
// Direct value assertion
assert_eq!(reference.target(), Some(commit_oid));
assert_eq!(reference.name().unwrap(), "refs/jin/layers/global");

// Property checking
assert!(repo.is_bare());
assert!(result.is_none());
```

### Error Testing
```rust
// Match specific error variants
assert!(matches!(result, Err(JinError::RepoNotFound { .. })));
assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
assert!(matches!(result, Err(JinError::RefExists { .. })));
```

### Option Handling
```rust
// Check None for missing references
let result = fixture.repo.get_layer_ref(&Layer::GlobalBase).unwrap();
assert!(result.is_none());

// Check Some for existing references
let result = fixture.repo.get_layer_ref(&Layer::GlobalBase).unwrap();
assert!(result.is_some());
```

## 4. Tempfile Usage

### Pattern for isolated test repositories:
```rust
// Direct usage (for simple tests)
let temp_dir = TempDir::new().unwrap();
let repo = JinRepo::init(temp_dir.path()).unwrap();

// Through TestFixture (recommended)
let fixture = TestFixture::new();
```

### Key principles:
- **Isolation**: Each test gets its own temporary directory
- **Automatic Cleanup**: TempDir automatically deletes when dropped
- **Bare Repositories**: All tests use `JinRepo::init()` which creates bare repos
- **No Side Effects**: Tests don't leave artifacts on the filesystem

## 5. Error Testing Patterns

### Testing Repository Errors
```rust
#[test]
fn test_jinrepo_open_nonexistent_errors() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    let result = JinRepo::open(&nonexistent);
    assert!(matches!(result, Err(JinError::RepoNotFound { .. })));
}
```

### Testing Layer Validation Errors
```rust
#[test]
fn test_jinrepo_unversioned_layers_error() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // Test UserLocal should error
    let result = fixture.repo.set_layer_ref(&Layer::UserLocal, commit_oid);
    assert!(matches!(result, Err(JinError::InvalidLayer { .. })));

    // Test WorkspaceActive should error
    let result = fixture
        .repo
        .set_layer_ref(&Layer::WorkspaceActive, commit_oid);
    assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
}
```

### Testing Reference Existence Errors
```rust
#[test]
fn test_jinrepo_create_layer_ref_fails_if_exists() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // First call should succeed
    fixture
        .repo
        .create_layer_ref(&Layer::GlobalBase, commit_oid)
        .unwrap();

    // Second call should fail
    let result = fixture
        .repo
        .create_layer_ref(&Layer::GlobalBase, commit_oid);
    assert!(matches!(result, Err(JinError::RefExists { .. })));
}
```

## 6. Example Tests for Reference Operations

### get_layer_ref Tests
```rust
#[test]
fn test_jinrepo_get_layer_ref_not_found() {
    let fixture = TestFixture::new();

    // Non-existent layer should return None
    let result = fixture.repo.get_layer_ref(&Layer::GlobalBase).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_jinrepo_get_layer_ref_found() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // Set layer ref first
    fixture.repo.set_layer_ref(&Layer::GlobalBase, commit_oid).unwrap();

    // Then get it back
    let result = fixture.repo.get_layer_ref(&Layer::GlobalBase).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().target(), Some(commit_oid));
}
```

### set_layer_ref Tests
```rust
#[test]
fn test_jinrepo_set_layer_ref() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // Set layer ref
    let reference = fixture
        .repo
        .set_layer_ref(&Layer::GlobalBase, commit_oid)
        .unwrap();

    assert_eq!(reference.target(), Some(commit_oid));
}

#[test]
fn test_jinrepo_set_layer_ref_updates() {
    let fixture = TestFixture::new();
    let commit1 = fixture.create_initial_commit();
    
    // Create second commit
    let tree_builder = fixture.repo.treebuilder().unwrap();
    let tree_oid = tree_builder.write().unwrap();
    let tree = fixture.repo.find_tree(tree_oid).unwrap();
    let parent_commit = fixture.repo.find_commit(commit1).unwrap();
    
    let author = fixture
        .repo
        .signature("Test Author", "test@example.com")
        .unwrap();
    let committer = &author;

    let commit2 = fixture
        .repo
        .create_commit(
            Some("HEAD"),
            &author,
            committer,
            "Second commit",
            &tree,
            &[&parent_commit],
        )
        .unwrap();

    // Set to first commit, then update to second
    fixture
        .repo
        .set_layer_ref(&Layer::GlobalBase, commit1)
        .unwrap();
    
    let reference = fixture
        .repo
        .set_layer_ref(&Layer::GlobalBase, commit2)
        .unwrap();

    assert_eq!(reference.target(), Some(commit2));
}
```

### create_layer_ref Tests
```rust
#[test]
fn test_jinrepo_create_layer_ref() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // Create layer ref
    let reference = fixture
        .repo
        .create_layer_ref(&Layer::GlobalBase, commit_oid)
        .unwrap();

    assert_eq!(reference.target(), Some(commit_oid));
}

#[test]
fn test_jinrepo_create_layer_ref_fails_if_exists() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // First call should succeed
    fixture
        .repo
        .create_layer_ref(&Layer::GlobalBase, commit_oid)
        .unwrap();

    // Second call should fail
    let result = fixture
        .repo
        .create_layer_ref(&Layer::GlobalBase, commit_oid);
    assert!(matches!(result, Err(JinError::RefExists { .. })));
}
```

## 7. Commit Creation Testing Pattern

### Standard Test Commit
```rust
fn create_initial_commit(&self) -> git2::Oid {
    let tree_builder = self.repo.treebuilder().unwrap();
    let tree_oid = tree_builder.write().unwrap();
    let tree = self.repo.find_tree(tree_oid).unwrap();

    let author = self
        .repo
        .signature("Test Author", "test@example.com")
        .unwrap();
    let committer = &author;

    self.repo
        .create_commit(
            Some("HEAD"),
            &author,
            committer,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap()
}
```

### Testing Multiple Commits
```rust
#[test]
fn test_multiple_commits() {
    let fixture = TestFixture::new();
    let commit1 = fixture.create_initial_commit();
    
    // Create second commit with the first as parent
    let tree_builder = fixture.repo.treebuilder().unwrap();
    let tree_oid = tree_builder.write().unwrap();
    let tree = fixture.repo.find_tree(tree_oid).unwrap();
    let parent_commit = fixture.repo.find_commit(commit1).unwrap();
    
    let author = fixture
        .repo
        .signature("Test Author", "test@example.com")
        .unwrap();
    let committer = &author;

    let commit2 = fixture
        .repo
        .create_commit(
            Some("HEAD"),
            &author,
            committer,
            "Second commit",
            &tree,
            &[&parent_commit],
        )
        .unwrap();

    // Test operations with both commits
    fixture.repo.set_layer_ref(&Layer::GlobalBase, commit1).unwrap();
    fixture.repo.set_layer_ref(&Layer::ModeBase { mode: "test".to_string() }, commit2).unwrap();
}
```

## 8. Integration Testing Patterns

### Layer Reference Format Testing
```rust
#[test]
fn test_jinrepo_layer_git_ref_integration() {
    let fixture = TestFixture::new();
    let commit_oid = fixture.create_initial_commit();

    // Test GlobalBase ref format
    let reference = fixture
        .repo
        .set_layer_ref(&Layer::GlobalBase, commit_oid)
        .unwrap();
    assert_eq!(reference.name().unwrap(), "refs/jin/layers/global");

    // Test ModeBase ref format
    let reference = fixture
        .repo
        .set_layer_ref(
            &Layer::ModeBase {
                mode: "claude".to_string(),
            },
            commit_oid,
        )
        .unwrap();
    assert_eq!(reference.name().unwrap(), "refs/jin/layers/mode/claude");

    // Test ScopeBase ref format
    let reference = fixture
        .repo
        .set_layer_ref(
            &Layer::ScopeBase {
                scope: "python".to_string(),
            },
            commit_oid,
        )
        .unwrap();
    assert_eq!(reference.name().unwrap(), "refs/jin/layers/scope/python");

    // Test ProjectBase ref format
    let reference = fixture
        .repo
        .set_layer_ref(
            &Layer::ProjectBase {
                project: "myapp".to_string(),
            },
            commit_oid,
        )
        .unwrap();
    assert_eq!(reference.name().unwrap(), "refs/jin/layers/project/myapp");
}
```

## 9. Best Practices

1. **Use TestFixture**: Prefer `TestFixture::new()` for most tests
2. **Clean Separation**: Separate test modules with clear boundaries
3. **Test Both Success and Failure**: Ensure all error cases are covered
4. **Use TempDir**: Always use temporary directories for isolation
5. **Assert Specific Error Types**: Use `matches!` with specific error variants
6. **Test Reference Formats**: Verify correct reference naming for different layer types
7. **Test Edge Cases**: Include tests for empty repositories, duplicate operations, etc.
8. **Keep Tests Simple**: Each test should test one specific behavior
9. **Use Descriptive Names**: Test names should clearly describe what is being tested
10. **Cleanup Automatically**: Let TempDir handle cleanup automatically
