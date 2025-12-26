# git2-rs Tree Walking Research

## Overview

This document covers tree walking and traversal patterns in git2-rs for implementing P1.M2.T4 "Tree Reading and Walking".

## Tree Object Basics

A Git tree represents a directory snapshot containing files (blobs) and subdirectories (subtrees).

```rust
use git2::Tree;

// Accessing tree properties
let tree = repo.find_tree(tree_oid)?;
let entry_count = tree.len();              // Number of entries
let tree_id = tree.id();                   // Object ID
```

## TreeEntry Structure

Each entry in a tree represents a file or subdirectory:

```rust
use git2::TreeEntry;

// TreeEntry properties from iteration or index access
let entry = tree.get(index)?;              // Get by index (0..len)
let name = entry.name();                   // Option<&str> - entry name
let oid = entry.id();                      // Oid - object ID
let kind = entry.kind();                   // Option<ObjectType> - Blob, Tree, etc.
let filemode = entry.kind();               // u32 - Unix file mode
```

## Method 1: Direct Index Access

```rust
for i in 0..tree.len() {
    let name = tree.get_name(i).unwrap_or("<unnamed>");
    let oid = tree.get_id(i);
    let kind = tree.get_kind(i);

    println!("{}: {} ({:?})", name, oid, kind);
}
```

## Method 2: Iterator Pattern (Recommended)

```rust
for entry in tree.iter() {
    let name = entry.name().unwrap_or("<unnamed>");
    let oid = entry.id();
    let kind = entry.kind();

    match kind {
        Some(ObjectType::Blob) => {
            // File entry - read blob content
            let blob = repo.find_blob(oid)?;
            let content = blob.content();
        },
        Some(ObjectType::Tree) => {
            // Directory entry - recurse
            // recursively walk subtree
        },
        _ => {},
    }
}
```

## Method 3: Tree Walker (Depth-First)

```rust
use git2::TreeWalkMode;

// Pre-order traversal (parent before children)
repo.walk(tree_id, TreeWalkMode::PreOrder)?
    .for_each(|path, entry| {
        let name = path.to_str().unwrap_or("<invalid>");
        let oid = entry.id();
        let kind = entry.kind();

        match kind {
            Some(ObjectType::Blob) => {
                // File with full path
                println!("File: {} -> {:?}", name, oid);
            },
            Some(ObjectType::Tree) => {
                println!("Dir: {}", name);
            },
            _ => {},
        }

        true // Continue traversal
    })?;
```

## Recursive Tree Traversal Pattern

```rust
fn walk_tree_recursive(repo: &Repository, tree_id: Oid, base_path: &str) -> Result<()> {
    let tree = repo.find_tree(tree_id)?;

    for entry in tree.iter() {
        let name = entry.name().unwrap_or("<unnamed>");
        let full_path = format!("{}/{}", base_path, name);
        let oid = entry.id();
        let kind = entry.kind();

        match kind {
            Some(ObjectType::Blob) => {
                let blob = repo.find_blob(oid)?;
                let content = blob.content();
                println!("File: {} ({} bytes)", full_path, content.len());
            },
            Some(ObjectType::Tree) => {
                println!("Entering directory: {}", full_path);
                walk_tree_recursive(repo, oid, &full_path)?;
            },
            _ => {},
        }
    }

    Ok(())
}
```

## Stack-Based Iterative Traversal (No Recursion)

```rust
fn walk_tree_iterative(repo: &Repository, root_tree_id: Oid) -> Result<()> {
    let mut stack = vec![(root_tree_id, String::new())];

    while let Some((tree_id, path)) = stack.pop() {
        let tree = repo.find_tree(tree_id)?;

        for entry in tree.iter() {
            let name = entry.name().unwrap_or("<unnamed>");
            let full_path = if path.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", path, name)
            };

            match entry.kind() {
                Some(ObjectType::Tree) => {
                    // Push subtree to stack
                    stack.push((entry.id(), full_path));
                },
                Some(ObjectType::Blob) => {
                    let blob = repo.find_blob(entry.id())?;
                    println!("File: {} ({} bytes)", full_path, blob.content().len());
                },
                _ => {},
            }
        }
    }

    Ok(())
}
```

## Finding a Specific File

```rust
fn find_file_in_tree(repo: &Repository, tree_id: Oid, filename: &str) -> Option<Oid> {
    let tree = repo.find_tree(tree_id).ok()?;

    for entry in tree.iter() {
        if let Some(name) = entry.name() {
            if name == filename {
                return Some(entry.id());
            }
        }
    }

    None
}
```

## Collecting All Files

```rust
use std::collections::HashMap;

fn collect_tree_files(repo: &Repository, tree_id: Oid) -> Result<HashMap<String, Vec<u8>>> {
    let mut files = HashMap::new();
    let mut stack = vec![(tree_id, String::new())];

    while let Some((tree_id, path)) = stack.pop() {
        let tree = repo.find_tree(tree_id)?;

        for entry in tree.iter() {
            let name = entry.name().unwrap_or("<unnamed>");
            let full_path = if path.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", path, name)
            };

            match entry.kind() {
                Some(ObjectType::Tree) => {
                    stack.push((entry.id(), full_path));
                },
                Some(ObjectType::Blob) => {
                    let blob = repo.find_blob(entry.id())?;
                    files.insert(full_path, blob.content().to_vec());
                },
                _ => {},
            }
        }
    }

    Ok(files)
}
```

## Gotchas and Common Pitfalls

### 1. Repository Lifetime Management

```rust
// BAD - Tree references repository, may cause use-after-free
fn bad_example() -> Option<Tree> {
    let repo = Repository::open(".")?;
    let tree = repo.find_tree(oid)?;
    drop(repo);
    Some(tree) // Tree is now invalid!
}

// GOOD - Keep repository alive or use OIDs
fn good_example() -> Option<Oid> {
    let repo = Repository::open(".")?;
    let tree = repo.find_tree(oid)?;
    Some(tree.id()) // Return OID instead
}
```

### 2. Entry Name Validation

```rust
// Always validate names before using
fn safe_entry_name(entry: &TreeEntry) -> Option<&str> {
    entry.name().and_then(|name| {
        if name.is_ascii() && !name.contains('\0') {
            Some(name)
        } else {
            None
        }
    })
}
```

### 3. File Mode Interpretation

```rust
use git2::FileMode;

fn get_entry_type(entry: &TreeEntry) -> &'static str {
    match FileMode::from_bits(entry.filemode()) {
        Some(FileMode::Blob) => "file",
        Some(FileMode::BlobExecutable) => "executable",
        Some(FileMode::Tree) => "directory",
        Some(FileMode::Link) => "symlink",
        _ => "unknown",
    }
}
```

### 4. Depth Limiting

```rust
fn walk_tree_depth_limited(repo: &Repository, tree_id: Oid, max_depth: usize) -> Result<()> {
    let mut stack = vec![(tree_id, String::new(), 0)];

    while let Some((tree_id, path, depth)) = stack.pop() {
        if depth > max_depth {
            continue;
        }

        let tree = repo.find_tree(tree_id)?;
        for entry in tree.iter() {
            let name = entry.name().unwrap_or("<unnamed>");
            let full_path = format!("{}/{}", path, name);

            match entry.kind() {
                Some(ObjectType::Tree) => {
                    stack.push((entry.id(), full_path, depth + 1));
                },
                Some(ObjectType::Blob) => {
                    println!("File: {} (depth: {})", full_path, depth);
                },
                _ => {},
            }
        }
    }

    Ok(())
}
```

## Key API Methods Summary

| Method | Signature | Description |
|--------|-----------|-------------|
| `tree.len()` | `fn len(&self) -> usize` | Entry count |
| `tree.get(i)` | `fn get(&self, i: usize) -> Option<TreeEntry>` | Get entry by index |
| `tree.get_id(i)` | `fn get_id(&self, i: usize) -> Oid` | Get entry OID |
| `tree.get_name(i)` | `fn get_name(&self, i: usize) -> Option<&str>` | Get entry name |
| `tree.get_kind(i)` | `fn get_kind(&self, i: usize) -> Option<ObjectType>` | Get entry type |
| `tree.iter()` | `impl Iterator<Item = TreeEntry>` | Iterate entries |
| `repo.walk()` | `fn walk(&self, id: Oid, mode: TreeWalkMode) -> TreeWalk` | Create walker |
| `TreeEntry::id()` | `fn id(&self) -> Oid` | Entry object ID |
| `TreeEntry::name()` | `fn name(&self) -> Option<&str>` | Entry name |
| `TreeEntry::kind()` | `fn kind(&self) -> Option<ObjectType>` | Entry type |
| `TreeEntry::filemode()` | `fn filemode(&self) -> u32` | File mode bits |

## External Resources

- https://docs.rs/git2/0.20/git2/struct.Tree.html - Tree API
- https://docs.rs/git2/0.20/git2/struct.TreeEntry.html - TreeEntry API
- https://docs.rs/git2/0.20/git2/enum.TreeWalkMode.html - Walk modes
- https://github.com/rust-lang/git2-rs/blob/master/examples/tree.rs - Examples
