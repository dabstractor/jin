# Product Requirement Prompt (PRP): Tree Reading and Walking (P1.M2.T4)

---

## Goal

**Feature Goal**: Provide tree traversal functionality for reading layer content stored as Git tree objects, enabling merge operations, diff generation, and workspace application.

**Deliverable**: Extended `src/git/repo.rs` module with tree reading and walking methods:
- `walk_tree()` - Recursively walk tree entries with a callback
- `list_tree_files()` - Collect all file paths and content from a tree
- `find_in_tree()` - Find a specific file path in a tree
- Comprehensive unit tests for all tree walking operations

**Success Definition**:
- `cargo build` compiles with zero errors
- All unit tests pass with isolated test repositories
- Trees can be walked recursively to extract all file entries
- File content can be read from blob entries during traversal
- Integration with existing `JinError` types for error handling
- Operations work with nested directory structures
- Empty trees and single-file trees handled correctly

## User Persona

**Target User**: AI coding agent implementing Jin's Git tree reading and walking layer

**Use Case**: The agent needs to establish tree traversal operations that:
- Enable reading layer content stored as Git trees
- Support recursive directory traversal for merge operations
- Extract file paths and content for workspace application
- Provide proper error handling and JinError integration
- Find specific files within tree structures

**User Journey**:
1. Agent receives this PRP as context
2. Implements `walk_tree()` for recursive traversal with callback
3. Implements `list_tree_files()` for collecting all files
4. Implements `find_in_tree()` for finding specific files
5. Adds comprehensive unit tests for various tree structures
6. Validates compilation and test success

**Pain Points Addressed**:
- No manual tree iteration and recursion needed in calling code
- Consistent error handling with `JinError` integration
- Clear abstraction over git2's tree walking APIs
- Support for both callback-based and collection-based patterns

## Why

- **Merge Operations**: P2 (Merge Engine) needs to read file content from multiple layers
- **Diff Generation**: P4 (CLI Commands) needs to compare trees between layers
- **Workspace Application**: P4 (Apply Command) needs to extract merged content to filesystem
- **Content Listing**: Various commands need to show what files exist in a layer
- **Problems this solves**:
  - No consistent interface for traversing tree structures
  - Direct git2 calls scattered throughout codebase for tree operations
  - No abstraction for recursive directory traversal
  - No helper for finding specific files in trees

## What

Implement tree reading and walking helper methods in `JinRepo` that wrap git2's tree APIs with consistent error handling and Jin-specific patterns.

### Success Criteria

- [ ] `walk_tree()` method implemented with callback pattern
- [ ] `list_tree_files()` method implemented for collecting files
- [ ] `find_in_tree()` method implemented for finding specific files
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover empty trees, single files, multiple files, and nested directories
- [ ] `cargo test` passes all tests
- [ ] Recursive traversal handles deeply nested structures correctly
- [ ] File content can be read from blob entries during traversal

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact method specifications with all parameters and return types
- Research documents with code examples for all tree walking patterns
- Specific patterns from existing codebase to follow
- Complete integration guide with `JinError` types
- Validation commands specific to this project
- Test patterns matching existing test structure

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Git Architecture specification - how trees store layer content
  section: Lines 84-115 for Logical Branch Model, Lines 558-585 for Git and Environment
  critical: Layer content is stored as Git trees referenced by layer refs

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Git ref namespace and layer storage structure
  section: Lines 103-116 for Git Ref Namespace format
  critical: Layer commits point to trees containing directory structures

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: Existing JinRepo implementation - add tree walking methods here
  section: Lines 727-757 for existing find_tree() method (Delegation Methods section)
  critical: Follow existing delegation pattern with ? operator, tests use TestFixture pattern

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  section: Lines 30-33 for JinError::Git (transparent error)
  critical: Use #[from] for automatic git2::Error conversion

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum's git_ref() method for accessing tree commits
  section: Lines 215-279 for git_ref() implementation
  critical: Tree OIDs are accessed via layer commit->tree

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T4/research/git2_tree_walking.md
  why: Complete git2-rs tree walking patterns with code examples
  section: Method 2: Iterator Pattern (lines 45-68), Recursive Tree Traversal (lines 95-123)
  critical: Shows exact Tree::iter(), TreeEntry API, and recursive patterns

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T4/research/testing_patterns.md
  why: Testing patterns from repo.rs for consistent test writing
  section: TestFixture Pattern (lines 5-60), Unit Test Patterns (lines 70-220)
  critical: Integration with existing tempfile usage, tree creation helpers

# EXTERNAL - git2-rs Documentation

- url: https://docs.rs/git2/0.20/git2/struct.Tree.html
  why: Tree API documentation - core of tree walking implementation
  section: Methods: iter(), get(), len(), get_id(), get_name(), get_kind()
  critical: pub fn iter(&self) -> Iter<'_> returns iterator over TreeEntry

- url: https://docs.rs/git2/0.20/git2/struct.TreeEntry.html
  why: TreeEntry API - properties of each entry in a tree
  section: Methods: name(), id(), kind(), filemode(), to_object()
  critical: name() returns Option<&str>, kind() returns Option<ObjectType>

- url: https://docs.rs/git2/0.20/git2/enum.ObjectType.html
  why: Object type enum - distinguishes Blob from Tree
  section: Variants: Blob, Tree
  critical: Match on kind() to determine if entry is file or directory

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.find_tree
  why: Finding tree by OID before walking
  section: pub fn find_tree(&self, oid: Oid) -> Result<Tree>
  critical: Call this first to get Tree reference

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.find_blob
  why: Reading blob content during traversal
  section: pub fn find_blob(&self, oid: Oid) -> Result<Blob>
  critical: Call Blob::content() to get &[u8] of file data
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── Cargo.toml                      # Has git2 = "0.20" dependency
├── PRD.md
├── src/
│   ├── core/
│   │   ├── error.rs               # Has JinError::Git (transparent)
│   │   ├── layer.rs               # Has Layer enum with git_ref() method
│   │   └── config.rs
│   └── git/
│       ├── mod.rs                 # Exports JinRepo
│       └── repo.rs                # Has Delegation Methods section (lines 690-771)
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added/Modified

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── git/
│       └── repo.rs                # MODIFY: Add Tree Walking Methods section
                                    # Add walk_tree(), list_tree_files(), find_in_tree()
                                    # Add comprehensive tests in #[cfg(test)] module
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Tree walking uses iteration, not the walk() API
// The repo.walk() API requires the Repository to remain borrowed
// Pattern: Use tree.iter() which doesn't borrow repo
// Example:
//   for entry in tree.iter() { ... }  // GOOD - no repo borrow
//   repo.walk(tree_id, mode) { ... }  // BAD - tricky lifetime issues

// CRITICAL: TreeEntry::name() returns Option<&str>
// Always handle None case (though unlikely in valid trees)
// Pattern:
//   let name = entry.name().unwrap_or("<unnamed>");

// CRITICAL: TreeEntry::kind() returns Option<ObjectType>
// Distinguish Blob (file) from Tree (directory) for recursion
// Pattern:
//   match entry.kind() {
//       Some(ObjectType::Blob) => { /* file */ },
//       Some(ObjectType::Tree) => { /* recurse */ },
//       _ => { /* ignore other types */ },
//   }

// CRITICAL: TreeEntry has filemode() for Unix permissions
// Use git2::FileMode to interpret
// Pattern:
//   match FileMode::from_bits(entry.filemode()) {
//       Some(FileMode::Blob) => "regular file",
//       Some(FileMode::Link) => "symlink",
//       _ => "other",
//   }

// CRITICAL: Blob content is accessed via Repository::find_blob()
// TreeEntry only has OID, need to look up Blob to read content
// Pattern:
//   let blob = repo.find_blob(entry.id())?;
//   let content = blob.content();

// CRITICAL: Recursive traversal uses stack or callback
// Avoid stack overflow with very deep trees (use iterative stack approach)
// Pattern:
//   fn walk_tree_recursive(repo, tree_id, path, callback) {
//       let tree = repo.find_tree(tree_id)?;
//       for entry in tree.iter() {
//           match entry.kind() {
//               Some(ObjectType::Tree) => {
//                   walk_tree_recursive(repo, entry.id(), new_path, callback)?;
//               },
//               Some(ObjectType::Blob) => {
//                   callback(path, entry)?;
//               },
//               _ => {},
//           }
//       }
//   }

// CRITICAL: Empty trees are valid (zero entries)
// tree.len() returns 0, tree.iter() yields no items
// Pattern:
//   if tree.len() == 0 {
//       return Ok(());  // Nothing to walk
//   }

// CRITICAL: TreeEntry name is basename only, not full path
// Must build full path during recursive traversal
// Pattern:
//   let full_path = format!("{}/{}", base_path, entry_name);

// PATTERN: Follow existing delegation pattern in repo.rs
// Methods like find_commit(), find_tree() use simple delegation
// Tree walking should follow same pattern but add traversal logic
// Example:
//   pub fn walk_tree<F>(&self, tree_id: Oid, mut callback: F) -> Result<()>
//   where F: FnMut(&str, &TreeEntry) -> Result<()>

// GOTCHA: TestFixture helper methods for creating test trees
// Use create_tree_with_files() and create_nested_tree() helpers
// Pattern from testing_patterns.md lines 25-60
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// No new data models - extending existing JinRepo

// Methods to add to JinRepo (new Tree Walking Methods section):
impl JinRepo {
    /// Walk a tree recursively, calling a callback for each file entry
    pub fn walk_tree<F>(
        &self,
        tree_id: git2::Oid,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str, &git2::TreeEntry) -> Result<()>,
    {
        // Implementation uses iterative stack to avoid recursion depth issues
        // Each file entry gets: full_path, &TreeEntry
    }

    /// Collect all files in a tree as a map of path -> content
    pub fn list_tree_files(
        &self,
        tree_id: git2::Oid,
    ) -> Result<std::collections::HashMap<String, Vec<u8>>>
    {
        // Uses walk_tree() internally
        // Reads blob content for each file
    }

    /// Find a specific file path in a tree
    pub fn find_in_tree(
        &self,
        tree_id: git2::Oid,
        path: &str,
    ) -> Result<Option<git2::Oid>>
    {
        // Uses walk_tree() internally with early exit
        // Returns Some(blob_oid) if found, None if not found
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD Tree Walking Methods section to src/git/repo.rs
  - LOCATION: After Delegation Methods section (around line 771)
  - ADD: Comment divider "===== Tree Walking Methods ====="
  - PATTERN: Follow existing section structure with comment dividers
  - CODE:
    // ===== Tree Walking Methods =====
  - PLACEMENT: Before Helper Methods section (line 773)
  - DEPENDENCIES: None

Task 2: IMPLEMENT walk_tree() method
  - SIGNATURE: pub fn walk_tree<F>(&self, tree_id: git2::Oid, mut callback: F) -> Result<()>
  - WHERE: F: FnMut(&str, &git2::TreeEntry) -> Result<()>
  - LOGIC:
    * Use Vec<(Oid, String)> as iterative stack (avoid recursion depth)
    * Push (root_tree_id, String::new()) to start
    * While stack not empty:
      - Pop (tree_id, base_path)
      - Find tree: let tree = self.find_tree(tree_id)?
      - For each entry in tree.iter():
        - Build full_path = format!("{}/{}", base_path, entry.name().unwrap())
        - Match entry.kind():
          * Blob => callback(full_path, entry)?
          * Tree => push (entry.id(), full_path) to stack
          * _ => ignore
    * Return Ok(())
  - ERROR HANDLING: Use ? for automatic JinError conversion
  - PATTERN: Follow git2_tree_walking.md lines 125-155 (Stack-Based Iterative)
  - PLACEMENT: In Tree Walking Methods section
  - DEPENDENCIES: Task 1

Task 3: IMPLEMENT list_tree_files() method
  - SIGNATURE: pub fn list_tree_files(&self, tree_id: git2::Oid) -> Result<HashMap<String, Vec<u8>>>
  - LOGIC:
    * Create empty HashMap
    * Call walk_tree(tree_id, |path, entry| {
        - match entry.kind() { Some(Blob) => {
            * Find blob: let blob = self.find_blob(entry.id())?
            * Insert content: files.insert(path.to_string(), blob.content().to_vec())
          * _ => Ok(())
        * })
    * Return files HashMap
  - GOTCHA: Need &self in callback closure for find_blob, use move or explicit capture
  - PATTERN: Follow git2_tree_walking.md lines 157-185 (Collecting All Files)
  - PLACEMENT: After walk_tree() in Tree Walking Methods section
  - DEPENDENCIES: Task 2

Task 4: IMPLEMENT find_in_tree() method
  - SIGNATURE: pub fn find_in_tree(&self, tree_id: git2::Oid, path: &str) -> Result<Option<git2::Oid>>
  - LOGIC:
    * Use early-exit pattern with Result<Option<>> control flow
    * Call walk_tree(tree_id, |entry_path, entry| {
        - If entry_path == path:
          * Return Err(ControlFlow::Break(entry.id())) to exit early
          * Or use cell/atomic for early exit
        - Ok(())
    * ) with custom error type for early exit
    * Return Ok(None) if not found, Ok(Some(oid)) if found
  - ALTERNATIVE: Use RefCell<Option<Oid>> for early exit in callback
  - PATTERN: Follow git2_tree_walking.md lines 195-209 (Finding a Specific File)
  - PLACEMENT: After list_tree_files() in Tree Walking Methods section
  - DEPENDENCIES: Task 2

Task 5: ADD unit tests for walk_tree() - Empty tree
  - FUNCTION: test_jinrepo_walk_empty_tree()
  - VERIFY: walk_tree() on empty tree completes without error
  - VERIFY: Callback is never invoked (no entries)
  - PATTERN: Follow testing_patterns.md lines 70-83
  - CODE:
    #[test]
    fn test_jinrepo_walk_empty_tree() {
        let fixture = TestFixture::new();
        let tree_oid = fixture.repo.treebuilder().unwrap().write().unwrap();

        let mut count = 0;
        fixture.repo.walk_tree(tree_oid, |_path, _entry| {
            count += 1;
            Ok(())
        }).unwrap();

        assert_eq!(count, 0);
    }
  - PLACEMENT: tests module in repo.rs (new Tree Walking Tests section)
  - DEPENDENCIES: Tasks 1-2

Task 6: ADD unit tests for walk_tree() - Single file
  - FUNCTION: test_jinrepo_walk_single_file_tree()
  - VERIFY: walk_tree() calls callback once for single file
  - VERIFY: Path and entry are correct
  - PATTERN: Follow testing_patterns.md lines 85-103
  - CODE:
    #[test]
    fn test_jinrepo_walk_single_file_tree() {
        let fixture = TestFixture::new();
        let mut builder = fixture.repo.treebuilder().unwrap();
        let blob_oid = fixture.repo.create_blob(b"content").unwrap();
        builder.insert("file.txt", blob_oid, git2::FileMode::Blob.into()).unwrap();
        let tree_oid = builder.write().unwrap();

        let mut entries = Vec::new();
        fixture.repo.walk_tree(tree_oid, |path, entry| {
            entries.push((path.to_string(), entry.id()));
            Ok(())
        }).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "file.txt");
        assert_eq!(entries[0].1, blob_oid);
    }
  - PLACEMENT: tests module after test_jinrepo_walk_empty_tree
  - DEPENDENCIES: Task 5

Task 7: ADD unit tests for walk_tree() - Nested directories
  - FUNCTION: test_jinrepo_walk_nested_tree()
  - VERIFY: walk_tree() traverses subdirectories recursively
  - VERIFY: Full paths are correct (e.g., "subdir/file.txt")
  - PATTERN: Follow testing_patterns.md lines 125-180
  - CODE:
    #[test]
    fn test_jinrepo_walk_nested_tree() {
        let fixture = TestFixture::new();
        // Create nested tree structure
        let mut subdir_builder = fixture.repo.treebuilder().unwrap();
        let sub_blob = fixture.repo.create_blob(b"sub content").unwrap();
        subdir_builder.insert("sub.txt", sub_blob, git2::FileMode::Blob.into()).unwrap();
        let subdir_oid = subdir_builder.write().unwrap();

        let mut root_builder = fixture.repo.treebuilder().unwrap();
        let root_blob = fixture.repo.create_blob(b"root content").unwrap();
        root_builder.insert("root.txt", root_blob, git2::FileMode::Blob.into()).unwrap();
        root_builder.insert("subdir", subdir_oid, git2::FileMode::Tree.into()).unwrap();
        let tree_oid = root_builder.write().unwrap();

        let mut paths = Vec::new();
        fixture.repo.walk_tree(tree_oid, |path, _entry| {
            paths.push(path.to_string());
            Ok(())
        }).unwrap();

        paths.sort();
        assert_eq!(paths, &["root.txt", "subdir/sub.txt"]);
    }
  - PLACEMENT: tests module after test_jinrepo_walk_single_file_tree
  - DEPENDENCIES: Task 6

Task 8: ADD unit tests for list_tree_files()
  - FUNCTION: test_jinrepo_list_tree_files()
  - VERIFY: list_tree_files() returns all files with content
  - VERIFY: HashMap keys are full paths
  - VERIFY: Blob content matches what was written
  - PATTERN: Follow testing_patterns.md lines 105-123 (multiple files)
  - CODE:
    #[test]
    fn test_jinrepo_list_tree_files() {
        let fixture = TestFixture::new();
        let mut builder = fixture.repo.treebuilder().unwrap();
        let blob1 = fixture.repo.create_blob(b"content 1").unwrap();
        let blob2 = fixture.repo.create_blob(b"content 2").unwrap();
        builder.insert("a.txt", blob1, git2::FileMode::Blob.into()).unwrap();
        builder.insert("b.txt", blob2, git2::FileMode::Blob.into()).unwrap();
        let tree_oid = builder.write().unwrap();

        let files = fixture.repo.list_tree_files(tree_oid).unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(files.get("a.txt"), Some(&b"content 1"[..]));
        assert_eq!(files.get("b.txt"), Some(&b"content 2"[..]));
    }
  - PLACEMENT: tests module after test_jinrepo_walk_nested_tree
  - DEPENDENCIES: Task 3, Task 7

Task 9: ADD unit tests for find_in_tree()
  - FUNCTION: test_jinrepo_find_in_tree()
  - VERIFY: find_in_tree() returns Some(oid) for existing files
  - VERIFY: find_in_tree() returns None for non-existent files
  - VERIFY: Works with nested paths like "subdir/file.txt"
  - PATTERN: Follow testing_patterns.md lines 193-218
  - CODE:
    #[test]
    fn test_jinrepo_find_in_tree() {
        let fixture = TestFixture::new();
        // Create nested tree
        let mut subdir_builder = fixture.repo.treebuilder().unwrap();
        let sub_blob = fixture.repo.create_blob(b"sub content").unwrap();
        subdir_builder.insert("sub.txt", sub_blob, git2::FileMode::Blob.into()).unwrap();
        let subdir_oid = subdir_builder.write().unwrap();

        let mut root_builder = fixture.repo.treebuilder().unwrap();
        let root_blob = fixture.repo.create_blob(b"root content").unwrap();
        root_builder.insert("root.txt", root_blob, git2::FileMode::Blob.into()).unwrap();
        root_builder.insert("subdir", subdir_oid, git2::FileMode::Tree.into()).unwrap();
        let tree_oid = root_builder.write().unwrap();

        // Find root file
        let found = fixture.repo.find_in_tree(tree_oid, "root.txt").unwrap();
        assert_eq!(found, Some(root_blob));

        // Find nested file
        let found = fixture.repo.find_in_tree(tree_oid, "subdir/sub.txt").unwrap();
        assert_eq!(found, Some(sub_blob));

        // Not found
        let found = fixture.repo.find_in_tree(tree_oid, "missing.txt").unwrap();
        assert!(found.is_none());
    }
  - PLACEMENT: tests module after test_jinrepo_list_tree_files
  - DEPENDENCIES: Task 4, Task 7

Task 10: ADD TestFixture helper methods
  - MODIFY: TestFixture struct in tests module
  - ADD: create_tree_with_files(files: &[(&str, &[u8])]) -> git2::Oid
  - ADD: create_nested_tree() -> git2::Oid
  - PATTERN: Follow testing_patterns.md lines 25-60
  - CODE:
    impl TestFixture {
        fn create_tree_with_files(&self, files: &[(&str, &[u8])]) -> git2::Oid {
            let mut builder = self.repo.treebuilder().unwrap();
            for (path, content) in files {
                let blob_oid = self.repo.create_blob(content).unwrap();
                builder.insert(path, blob_oid, git2::FileMode::Blob.into()).unwrap();
            }
            builder.write().unwrap()
        }

        fn create_nested_tree(&self) -> git2::Oid {
            // ... (see testing_patterns.md lines 40-60)
        }
    }
  - PLACEMENT: In TestFixture impl block
  - DEPENDENCIES: None
```

### Implementation Patterns & Key Details

```rust
// ===== TREE WALKING PATTERN (Iterative Stack) =====
// Uses Vec as stack to avoid recursion depth issues
impl JinRepo {
    pub fn walk_tree<F>(&self, tree_id: git2::Oid, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &git2::TreeEntry) -> Result<()>,
    {
        let mut stack = vec![(tree_id, String::new())];

        while let Some((current_id, base_path)) = stack.pop() {
            let tree = self.find_tree(current_id)?;

            // Walk in reverse order to maintain correct processing order
            for entry in tree.iter().rev() {
                let name = entry.name().unwrap_or("<unnamed>");
                let full_path = if base_path.is_empty() {
                    name.to_string()
                } else {
                    format!("{}/{}", base_path, name)
                };

                match entry.kind() {
                    Some(git2::ObjectType::Blob) => {
                        callback(&full_path, entry)?;
                    },
                    Some(git2::ObjectType::Tree) => {
                        stack.push((entry.id(), full_path));
                    },
                    _ => {},
                }
            }
        }

        Ok(())
    }
}

// ===== LIST FILES PATTERN =====
// Collects all files with their content into HashMap
impl JinRepo {
    pub fn list_tree_files(&self, tree_id: git2::Oid) -> Result<HashMap<String, Vec<u8>>> {
        let mut files = HashMap::new();

        self.walk_tree(tree_id, |path, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let blob = self.find_blob(entry.id())?;
                files.insert(path.to_string(), blob.content().to_vec());
            }
            Ok(())
        })?;

        Ok(files)
    }
}

// ===== FIND IN TREE PATTERN (Early Exit with ControlFlow) =====
// Uses std::ops::ControlFlow for early exit
impl JinRepo {
    pub fn find_in_tree(&self, tree_id: git2::Oid, target_path: &str) -> Result<Option<git2::Oid>> {
        use std::ops::ControlFlow;

        let result = std::cell::RefCell::new(None);

        let walk_result = self.walk_tree(tree_id, |path, entry| {
            if path == target_path {
                *result.borrow_mut() = Some(entry.id());
                return Err(JinError::Message("found".to_string())); // Use sentinel error
            }
            Ok(())
        });

        match walk_result {
            Err(_) if result.borrow().is_some() => Ok(result.into_inner()),
            Ok(()) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

// Alternative: Use RefCell<Option<Oid>> without sentinel error
impl JinRepo {
    pub fn find_in_tree(&self, tree_id: git2::Oid, target_path: &str) -> Result<Option<git2::Oid>> {
        let result = std::cell::RefCell::new(None);

        self.walk_tree(tree_id, |path, entry| {
            if path == target_path && entry.kind() == Some(git2::ObjectType::Blob) {
                *result.borrow_mut() = Some(entry.id());
            }
            Ok(())
        })?;

        Ok(result.into_inner())
    }
}

// ===== USAGE EXAMPLES =====

// Example 1: List all files in a layer
let layer = Layer::GlobalBase;
if let Some(reference) = repo.get_layer_ref(&layer)? {
    let commit = repo.find_commit(reference.target().unwrap())?;
    let tree = commit.tree()?;
    let files = repo.list_tree_files(tree.id())?;
    for (path, content) in files {
        println!("{}: {} bytes", path, content.len());
    }
}

// Example 2: Find specific file
let layer = Layer::ProjectBase { project: "myapp".to_string() };
if let Some(reference) = repo.get_layer_ref(&layer)? {
    let commit = repo.find_commit(reference.target().unwrap())?;
    let tree = commit.tree()?;
    if let Some(blob_oid) = repo.find_in_tree(tree.id(), "config.json")? {
        let blob = repo.find_blob(blob_oid)?;
        let config: serde_json::Value = serde_json::from_slice(blob.content())?;
    }
}

// Example 3: Walk with custom callback
repo.walk_tree(tree_id, |path, entry| {
    match entry.kind() {
        Some(git2::ObjectType::Blob) => {
            let blob = repo.find_blob(entry.id())?;
            println!("File: {} ({} bytes)", path, blob.content().len());
        },
        Some(git2::ObjectType::Tree) => {
            println!("Dir: {}", path);
        },
        _ => {},
    }
    Ok(())
})?;

// Example 4: Filter by file extension
let json_files: HashMap<String, Vec<u8>> = repo.list_tree_files(tree_id)?
    .into_iter()
    .filter(|(path, _)| path.ends_with(".json"))
    .collect();
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs
  - pattern: JinError::Git (transparent) - automatic via #[from]
  - All git2::Error auto-converts through ? operator

LAYER_INTEGRATION:
  - use: src/core/layer.rs
  - method: layer.git_ref() -> get layer ref -> commit -> tree
  - pattern:
    * let layer = Layer::GlobalBase;
    * let reference = repo.get_layer_ref(&layer)?;
    * let commit = repo.find_commit(reference.target().unwrap())?;
    * let tree_id = commit.tree_id();
    * repo.walk_tree(tree_id, callback)?

BLOB_READING:
  - use: existing find_blob() delegation method
  - pattern:
    * let blob = repo.find_blob(entry.id())?;
    * let content = blob.content();

MERGE_ENGINE (FUTURE):
  - P2.M3: Will use walk_tree() to read layer content for merging
  - Will call list_tree_files() to get all files in a layer
  - Will find_in_tree() to locate specific config files

CLI_COMMANDS (FUTURE):
  - P4.M5: Diff command will use walk_tree() to compare layers
  - P4.M4: Apply command will use list_tree_files() to extract merged content
  - P4.M5: Log command may use walk_tree() to show changed files
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
# Test JinRepo module specifically - tree walking tests
cargo test --package jin --lib git::repo::tests::test_jinrepo_walk --verbose
cargo test --package jin --lib git::repo::tests::test_jinrepo_list --verbose
cargo test --package jin --lib git::repo::tests::test_jinrepo_find --verbose

# Run specific tree walking tests
cargo test --package jin test_jinrepo_walk_empty_tree -- --exact
cargo test --package jin test_jinrepo_walk_single_file_tree -- --exact
cargo test --package jin test_jinrepo_walk_nested_tree -- --exact
cargo test --package jin test_jinrepo_list_tree_files -- --exact
cargo test --package jin test_jinrepo_find_in_tree -- --exact

# Expected: All tests pass. Look for:
# - test_jinrepo_walk_empty_tree: Verifies empty tree handling
# - test_jinrepo_walk_single_file_tree: Verifies single file callback
# - test_jinrepo_walk_nested_tree: Verifies recursive traversal
# - test_jinrepo_list_tree_files: Verifies file collection with content
# - test_jinrepo_find_in_tree: Verifies file finding with nested paths
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual tree operations with real git2
cargo test --package jin --lib git::repo --verbose

# Manual verification of tree walking:
# After test creates nested tree, verify:
# 1. All files are visited by walk_tree()
# 2. Full paths include directory separators
# 3. Blob content matches what was written
# 4. Nested files are correctly traversed

# Expected:
# - Empty tree: callback never invoked
# - Single file: callback invoked once with correct path
# - Nested tree: all files visited, paths like "subdir/file.txt"
# - list_tree_files: returns HashMap with all file content
# - find_in_tree: finds files at any depth, returns None for missing
```

### Level 4: Domain-Specific Validation

```bash
# Verify recursive traversal depth handling
cargo test --package jin test_jinrepo_walk_nested_tree -- --exact
# Asserts: paths include "subdir/sub.txt" with correct depth

# Verify file content round-trip
cargo test --package jin test_jinrepo_list_tree_files -- --exact
# Asserts: HashMap values match original content

# Verify path matching for find_in_tree
cargo test --package jin test_jinrepo_find_in_tree -- --exact
# Asserts: exact string match for paths like "subdir/sub.txt"

# Expected: All Jin-specific requirements met
# - Full paths use "/" separator regardless of OS
# - Empty base path produces just filename
# - Non-existent files return None, not error
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all new public methods
- [ ] Stack-based traversal avoids recursion depth issues

### Feature Validation

- [ ] `walk_tree()` traverses empty trees without error
- [ ] `walk_tree()` traverses single-file trees correctly
- [ ] `walk_tree()` handles nested directories with proper paths
- [ ] `list_tree_files()` returns all files with correct content
- [ ] `find_in_tree()` finds files at any depth
- [ ] `find_in_tree()` returns None for non-existent files
- [ ] Integration with `JinError::Git` (transparent conversion)

### Code Quality Validation

- [ ] Follows existing repo.rs patterns
- [ ] Uses delegation pattern where appropriate
- [ ] Error handling matches existing patterns
- [ ] Test coverage for all public methods
- [ ] Tests follow testing_patterns.md conventions
- [ ] TestFixture helpers simplify test setup

### Documentation & Deployment

- [ ] All public methods have doc comments with examples
- [ ] Complex patterns (stack traversal) documented
- [ ] Gotchas documented (name Option, early exit pattern)

---

## Anti-Patterns to Avoid

- Don't use recursion for tree walking - use iterative stack to avoid stack overflow
- Don't use repo.walk() API - has tricky lifetime issues, use tree.iter() instead
- Don't forget to handle Option<&str> from entry.name() - use unwrap_or() for safety
- Don't forget to match on entry.kind() - need to distinguish Blob from Tree
- Don't skip testing nested directory structures - that's the primary use case
- Don't use early exit with panic/unreachable - use RefCell or sentinel error pattern
- Don't forget to build full paths during traversal - entry names are basenames only
- Don't read blob content in walk_tree callback if not needed - use find_in_tree() for OIDs only
- Don't forget to reverse iteration order when using stack - maintains natural processing order
- Don't ignore empty trees - they're valid and should complete without error

---

## Appendix: Quick Reference

### Tree Walking API Summary

```rust
// Recursive tree walking with callback
pub fn walk_tree<F>(&self, tree_id: git2::Oid, callback: F) -> Result<()>
where F: FnMut(&str, &git2::TreeEntry) -> Result<()>

// Collect all files with content
pub fn list_tree_files(&self, tree_id: git2::Oid) -> Result<HashMap<String, Vec<u8>>>

// Find specific file path
pub fn find_in_tree(&self, tree_id: git2::Oid, path: &str) -> Result<Option<git2::Oid>>
```

### TreeEntry API Summary

| Method | Return Type | Description |
|--------|-------------|-------------|
| `name()` | `Option<&str>` | Entry basename (may be None) |
| `id()` | `Oid` | Object ID (blob or tree) |
| `kind()` | `Option<ObjectType>` | Blob (file) or Tree (directory) |
| `filemode()` | `u32` | Unix file mode bits |

### Tree Walking Flow

```
Layer.git_ref()
  -> Reference::target()
  -> Commit::tree_id()
  -> JinRepo::walk_tree(tree_id, callback)
    -> callback receives (full_path, TreeEntry)
      -> TreeEntry::id()
      -> JinRepo::find_blob(oid)
        -> Blob::content()
```

### Typical Workflows

```rust
// Workflow 1: List all files in a layer
let commit = repo.find_commit(layer_ref_target)?;
let files = repo.list_tree_files(commit.tree_id())?;

// Workflow 2: Find specific file in layer
let commit = repo.find_commit(layer_ref_target)?;
if let Some(blob_oid) = repo.find_in_tree(commit.tree_id(), "config.json")? {
    let blob = repo.find_blob(blob_oid)?;
    let content = blob.content();
}

// Workflow 3: Walk with custom logic
repo.walk_tree(tree_id, |path, entry| {
    if let Some(git2::ObjectType::Blob) = entry.kind() {
        if path.ends_with(".json") {
            let blob = repo.find_blob(entry.id())?;
            // Process JSON file
        }
    }
    Ok(())
})?;
```

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 10/10 - All context provided, patterns documented, implementation ready to begin
