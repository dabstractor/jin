# Tree Walking Testing Patterns

## Overview

This document covers testing patterns for tree walking functionality based on the existing test structure in `src/git/repo.rs`.

## TestFixture Pattern (from repo.rs)

The existing tests use a `TestFixture` struct for consistent test setup:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestFixture {
        _temp_dir: TempDir,      // Kept to clean up on drop
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

            let author = self.repo.signature("Test Author", "test@example.com").unwrap();
            let committer = &author;

            self.repo.create_commit(
                Some("HEAD"),
                &author,
                committer,
                "Initial commit",
                &tree,
                &[],
            ).unwrap()
        }

        fn create_tree_with_files(&self, files: &[(&str, &[u8])]) -> git2::Oid {
            let mut builder = self.repo.treebuilder().unwrap();

            for (path, content) in files {
                let blob_oid = self.repo.create_blob(content).unwrap();
                builder.insert(path, blob_oid, git2::FileMode::Blob.into()).unwrap();
            }

            builder.write().unwrap()
        }

        fn create_nested_tree(&self) -> git2::Oid {
            // Create a subdirectory tree
            let mut subdir_builder = self.repo.treebuilder().unwrap();
            let subfile_oid = self.repo.create_blob(b"subfile content").unwrap();
            subdir_builder.insert("subfile.txt", subfile_oid, git2::FileMode::Blob.into()).unwrap();
            let subdir_oid = subdir_builder.write().unwrap();

            // Create root tree with subdir
            let mut root_builder = self.repo.treebuilder().unwrap();
            let rootfile_oid = self.repo.create_blob(b"root file content").unwrap();
            root_builder.insert("root.txt", rootfile_oid, git2::FileMode::Blob.into()).unwrap();
            root_builder.insert("subdir", subdir_oid, git2::FileMode::Tree.into()).unwrap();
            root_builder.write().unwrap()
        }
    }
}
```

## Test Function Naming Conventions

From existing tests in repo.rs:

```rust
// Pattern: test_jinrepo_<method>_<scenario>
test_jinrepo_find_tree()           // Basic find
test_jinrepo_create_blob()          // Creation
test_jinrepo_layer_ref_exists_true  // Positive test
test_jinrepo_layer_ref_exists_false // Negative test
```

## Unit Test Patterns for Tree Walking

### Test 1: Empty Tree Walking

```rust
#[test]
fn test_jinrepo_walk_empty_tree() {
    let fixture = TestFixture::new();
    let tree_oid = fixture.repo.treebuilder().unwrap().write().unwrap();
    let tree = fixture.repo.find_tree(tree_oid).unwrap();

    // Should have zero entries
    assert_eq!(tree.len(), 0);

    // Walk should not error
    let mut entries = Vec::new();
    for entry in tree.iter() {
        entries.push(entry.name().unwrap_or("").to_string());
    }
    assert_eq!(entries.len(), 0);
}
```

### Test 2: Single File Tree

```rust
#[test]
fn test_jinrepo_walk_single_file_tree() {
    let fixture = TestFixture::new();

    let tree_oid = fixture.create_tree_with_files(&[
        ("README.md", b"# Hello"),
    ]);

    let tree = fixture.repo.find_tree(tree_oid).unwrap();
    assert_eq!(tree.len(), 1);

    let entry = tree.get(0).unwrap();
    assert_eq!(entry.name(), Some("README.md"));
    assert_eq!(entry.kind(), Some(git2::ObjectType::Blob));

    // Verify blob content
    let blob = fixture.repo.find_blob(entry.id()).unwrap();
    assert_eq!(std::str::from_utf8(blob.content()).unwrap(), "# Hello");
}
```

### Test 3: Multiple Files Tree

```rust
#[test]
fn test_jinrepo_walk_multiple_files_tree() {
    let fixture = TestFixture::new();

    let tree_oid = fixture.create_tree_with_files(&[
        ("a.txt", b"content a"),
        ("b.txt", b"content b"),
        ("c.txt", b"content c"),
    ]);

    let tree = fixture.repo.find_tree(tree_oid).unwrap();
    assert_eq!(tree.len(), 3);

    // Collect all names
    let mut names: Vec<_> = tree.iter()
        .filter_map(|e| e.name())
        .collect();
    names.sort();

    assert_eq!(names, &["a.txt", "b.txt", "c.txt"]);
}
```

### Test 4: Nested Directory Tree

```rust
#[test]
fn test_jinrepo_walk_nested_tree() {
    let fixture = TestFixture::new();
    let tree_oid = fixture.create_nested_tree();

    let tree = fixture.repo.find_tree(tree_oid).unwrap();

    // Root tree should have 2 entries (root.txt + subdir)
    assert_eq!(tree.len(), 2);

    let mut found_root = false;
    let mut found_subdir = false;

    for entry in tree.iter() {
        match entry.name() {
            Some("root.txt") => {
                found_root = true;
                assert_eq!(entry.kind(), Some(git2::ObjectType::Blob));
            },
            Some("subdir") => {
                found_subdir = true;
                assert_eq!(entry.kind(), Some(git2::ObjectType::Tree));

                // Walk into subtree
                let subtree = fixture.repo.find_tree(entry.id()).unwrap();
                assert_eq!(subtree.len(), 1);

                let subentry = subtree.get(0).unwrap();
                assert_eq!(subentry.name(), Some("subfile.txt"));
                assert_eq!(subentry.kind(), Some(git2::ObjectType::Blob));

                let blob = fixture.repo.find_blob(subentry.id()).unwrap();
                assert_eq!(std::str::from_utf8(blob.content()).unwrap(), "subfile content");
            },
            _ => panic!("Unexpected entry"),
        }
    }

    assert!(found_root);
    assert!(found_subdir);
}
```

### Test 5: Recursive Tree Walking

```rust
#[test]
fn test_jinrepo_walk_tree_recursive() {
    let fixture = TestFixture::new();
    let tree_oid = fixture.create_nested_tree();

    let mut files = Vec::new();

    // Simulate recursive walk
    fn collect_files(repo: &JinRepo, tree_id: git2::Oid, base: &str, files: &mut Vec<String>) {
        let tree = repo.find_tree(tree_id).unwrap();
        for entry in tree.iter() {
            let name = entry.name().unwrap();
            let full_path = if base.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", base, name)
            };

            match entry.kind() {
                Some(git2::ObjectType::Blob) => {
                    files.push(full_path);
                },
                Some(git2::ObjectType::Tree) => {
                    collect_files(repo, entry.id(), &full_path, files);
                },
                _ => {},
            }
        }
    }

    collect_files(&fixture.repo, tree_oid, "", &mut files);
    files.sort();

    assert_eq!(files, &["root.txt", "subdir/subfile.txt"]);
}
```

### Test 6: Find File in Tree

```rust
#[test]
fn test_jinrepo_find_file_in_tree() {
    let fixture = TestFixture::new();

    let tree_oid = fixture.create_tree_with_files(&[
        ("a.txt", b"content a"),
        ("b.txt", b"content b"),
    ]);

    let tree = fixture.repo.find_tree(tree_oid).unwrap();

    // Find existing file
    let found = tree.iter()
        .find(|e| e.name() == Some("b.txt"));
    assert!(found.is_some());

    let entry = found.unwrap();
    assert_eq!(entry.kind(), Some(git2::ObjectType::Blob));

    let blob = fixture.repo.find_blob(entry.id()).unwrap();
    assert_eq!(std::str::from_utf8(blob.content()).unwrap(), "content b");

    // Find non-existent file
    let not_found = tree.iter()
        .find(|e| e.name() == Some("c.txt"));
    assert!(not_found.is_none());
}
```

### Test 7: Get Entry by Index

```rust
#[test]
fn test_jinrepo_get_tree_entry_by_index() {
    let fixture = TestFixture::new();

    let tree_oid = fixture.create_tree_with_files(&[
        ("first.txt", b"first"),
        ("second.txt", b"second"),
    ]);

    let tree = fixture.repo.find_tree(tree_oid).unwrap();

    // Get first entry
    let entry = tree.get(0).unwrap();
    assert_eq!(entry.name(), Some("first.txt"));

    // Get second entry
    let entry = tree.get(1).unwrap();
    assert_eq!(entry.name(), Some("second.txt"));

    // Invalid index
    assert!(tree.get(2).is_none());
    assert!(tree.get(100).is_none());
}
```

### Test 8: Entry Properties

```rust
#[test]
fn test_jinrepo_tree_entry_properties() {
    let fixture = TestFixture::new();

    let blob_oid = fixture.repo.create_blob(b"content").unwrap();
    let mut builder = fixture.repo.treebuilder().unwrap();
    builder.insert("file.txt", blob_oid, git2::FileMode::Blob.into()).unwrap();
    let tree_oid = builder.write().unwrap();

    let tree = fixture.repo.find_tree(tree_oid).unwrap();
    let entry = tree.get(0).unwrap();

    // Test all properties
    assert_eq!(entry.name(), Some("file.txt"));
    assert_eq!(entry.id(), blob_oid);
    assert_eq!(entry.kind(), Some(git2::ObjectType::Blob));

    // Test filemode
    let filemode = entry.filemode();
    assert_eq!(filemode, git2::FileMode::Blob.into());
}
```

## Test Organization

Tests should be grouped by functionality:

```rust
#[cfg(test)]
mod tests {
    // ... TestFixture ...

    // ===== Tree Reading Tests =====
    #[test]
    fn test_jinrepo_find_tree() { ... }

    // ===== Tree Walking Tests =====
    #[test]
    fn test_jinrepo_walk_empty_tree() { ... }
    #[test]
    fn test_jinrepo_walk_single_file_tree() { ... }
    #[test]
    fn test_jinrepo_walk_multiple_files_tree() { ... }
    #[test]
    fn test_jinrepo_walk_nested_tree() { ... }

    // ===== Tree Entry Tests =====
    #[test]
    fn test_jinrepo_get_tree_entry_by_index() { ... }
    #[test]
    fn test_jinrepo_tree_entry_properties() { ... }

    // ===== Recursive Walking Tests =====
    #[test]
    fn test_jinrepo_walk_tree_recursive() { ... }
    #[test]
    fn test_jinrepo_find_file_in_tree() { ... }
}
```

## Running Tests

```bash
# Run all tree walking tests
cargo test --package jin --lib git::repo::tests::test_jinrepo_walk --verbose

# Run specific test
cargo test --package jin test_jinrepo_walk_nested_tree -- --exact

# Run with output
cargo test --package jin test_jinrepo_walk -- --nocapture

# Show test names
cargo test --package jin --lib -- --list
```

## Common Test Patterns

1. **TestFixture::new()** - Creates fresh JinRepo for each test
2. **tempfile::TempDir** - Auto-cleanup on test completion
3. **assert_eq!/assert!** - Standard assertions
4. **unwrap()** - OK in tests, will panic with useful message
5. **Filter_map** - For finding specific entries
6. **Collect** - For gathering results

## Test Data Patterns

```rust
// Helper to create file content
fn test_content(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

// Helper to create blob from string
fn create_test_blob(repo: &JinRepo, content: &str) -> git2::Oid {
    repo.create_blob(content.as_bytes()).unwrap()
}
```
