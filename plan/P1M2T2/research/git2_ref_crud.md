# git2-rs Git Reference CRUD API Research

## Overview

This document provides comprehensive information for implementing a complete CRUD (Create, Read, Update, Delete) API for Git references using git2-rs, the Rust bindings for libgit2.

## 1. DELETE Operations

### Method Calls Needed

#### Basic Reference Deletion
```rust
use git2::Repository;
use std::path::Path;

fn delete_reference(repo: &Repository, ref_name: &str) -> Result<(), git2::Error> {
    // Find the reference by name
    let mut reference = repo.find_reference(ref_name)?;

    // Delete the reference
    reference.delete()?;

    Ok(())
}
```

#### Batch Deletion with Pattern Matching
```rust
use git2::Repository;
use glob::Pattern; // Requires glob crate

fn delete_references_by_pattern(repo: &Repository, pattern: &str) -> Result<usize, git2::Error> {
    let mut deleted_count = 0;
    let pattern = Pattern::new(pattern)?;

    // Get all references
    for reference in repo.references()? {
        let reference = reference?;
        let ref_name = reference.name().unwrap_or("");

        if pattern.matches(ref_name) {
            reference.delete()?;
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}
```

#### Namespace-Specific Deletion
```rust
fn delete_branches(repo: &Repository, branch_pattern: &str) -> Result<usize, git2::Error> {
    let mut deleted_count = 0;

    // Use references_glob for efficient pattern matching
    for reference in repo.references_glob(format!("refs/heads/{}", branch_pattern))? {
        let reference = reference?;
        reference.delete()?;
        deleted_count += 1;
    }

    Ok(deleted_count)
}
```

### Error Handling Patterns

```rust
fn safe_delete_reference(repo: &Repository, ref_name: &str) -> Result<(), git2::Error> {
    // First check if reference exists
    if repo.find_reference(ref_name).is_err() {
        return Err(git2::Error::from_str("Reference does not exist"));
    }

    // Attempt to delete
    match repo.find_reference(ref_name)?.delete() {
        Ok(_) => Ok(()),
        Err(e) => {
            // Handle specific error cases
            match e.code() {
                git2::ErrorCode::NotFound => Err(git2::Error::from_str("Reference not found")),
                git2::ErrorCode::Generic => {
                    eprintln!("Failed to delete reference: {}", e.message());
                    Err(e)
                }
                _ => Err(e),
            }
        }
    }
}
```

### Safety Considerations

1. **Reference Existence Check**: Always verify the reference exists before deletion
2. **Atomic Operations**: git2-rs provides atomic reference deletion
3. **Branch Protection**: For branches, consider HEAD protection
4. **Garbage Collection**: Deleted references may require garbage collection

```rust
// Safe deletion with HEAD check
fn delete_branch_safely(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
    let full_ref_name = format!("refs/heads/{}", branch_name);

    // Don't delete the current HEAD branch
    if let Ok(head) = repo.head() {
        if head.name() == Some(&full_ref_name) {
            return Err(git2::Error::from_str("Cannot delete current HEAD branch"));
        }
    }

    // Check if branch exists
    if repo.find_reference(&full_ref_name).is_err() {
        return Err(git2::Error::from_str("Branch does not exist"));
    }

    repo.find_reference(&full_ref_name)?.delete()?;
    Ok(())
}
```

## 2. LIST/ITERATE Operations

### Methods for Listing All References

```rust
use git2::Repository;

fn list_all_references(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    let mut references = Vec::new();

    for reference in repo.references()? {
        let reference = reference?;
        if let Some(name) = reference.name() {
            references.push(name.to_string());
        }
    }

    Ok(references)
}

// Alternative with more detail
fn list_references_with_details(repo: &Repository) -> Result<Vec<ReferenceInfo>, git2::Error> {
    let mut references = Vec::new();

    for reference in repo.references()? {
        let reference = reference?;

        let info = ReferenceInfo {
            name: reference.name().unwrap_or("").to_string(),
            target: reference.target().map(|t| t.to_string()),
            shorthand: reference.shorthand().map(|s| s.to_string()),
            is_branch: reference.name().unwrap_or("").starts_with("refs/heads/"),
            is_tag: reference.name().unwrap_or("").starts_with("refs/tags/"),
            is_remote: reference.name().unwrap_or("").starts_with("refs/remotes/"),
        };

        references.push(info);
    }

    Ok(references)
}

struct ReferenceInfo {
    name: String,
    target: Option<String>,
    shorthand: Option<String>,
    is_branch: bool,
    is_tag: bool,
    is_remote: bool,
}
```

### Methods for Listing References Matching a Pattern (glob)

```rust
use git2::Repository;

fn list_references_by_glob(repo: &Repository, pattern: &str) -> Result<Vec<String>, git2::Error> {
    let mut references = Vec::new();

    // Use built-in glob functionality
    for reference in repo.references_glob(pattern)? {
        let reference = reference?;
        if let Some(name) = reference.name() {
            references.push(name.to_string());
        }
    }

    Ok(references)
}

// Common patterns
fn list_all_branches(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    list_references_by_glob(repo, "refs/heads/*")
}

fn list_all_tags(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    list_references_by_glob(repo, "refs/tags/*")
}

fn list_all_remotes(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    list_references_by_glob(repo, "refs/remotes/*")
}

fn list_local_branches(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    list_references_by_glob(repo, "refs/heads/*")
}

fn list_remote_branches(repo: &Repository, remote_name: &str) -> Result<Vec<String>, git2::Error> {
    list_references_by_glob(repo, &format!("refs/remotes/{}/*", remote_name))
}
```

### Iterating Over References in a Namespace

```rust
use git2::Repository;

fn iterate_over_references<F>(repo: &Repository, mut callback: F) -> Result<(), git2::Error>
where
    F: FnMut(&str, &str) -> bool,
{
    for reference in repo.references()? {
        let reference = reference?;

        if let (Some(name), Some(target)) = (reference.name(), reference.target()) {
            let target_str = target.to_string();
            // Continue iteration unless callback returns false
            if !callback(name, &target_str) {
                break;
            }
        }
    }

    Ok(())
}

// Usage example:
fn print_all_references(repo: &Repository) -> Result<(), git2::Error> {
    iterate_over_references(repo, |name, target| {
        println!("{} -> {}", name, target);
        true // continue
    })
}
```

## 3. Reference Validation/Verification

### Methods for Checking Reference Existence

```rust
use git2::Repository;

fn reference_exists(repo: &Repository, name: &str) -> bool {
    repo.find_reference(name).is_ok()
}

// More efficient check for specific reference types
fn branch_exists(repo: &Repository, branch_name: &str) -> bool {
    repo.find_reference(&format!("refs/heads/{}", branch_name)).is_ok()
}

fn tag_exists(repo: &Repository, tag_name: &str) -> bool {
    repo.find_reference(&format!("refs/tags/{}", tag_name)).is_ok()
}

fn remote_branch_exists(repo: &Repository, remote: &str, branch: &str) -> bool {
    repo.find_reference(&format!("refs/remotes/{}/{}", remote, branch)).is_ok()
}
```

### Getting Reference Metadata

```rust
use git2::Repository;
use git2::ObjectType;

fn get_reference_metadata(repo: &Repository, name: &str) -> Option<ReferenceMetadata> {
    let reference = repo.find_reference(name).ok()?;

    Some(ReferenceMetadata {
        name: reference.name()?.to_string(),
        target: reference.target().map(|t| t.to_string()),
        shorthand: reference.shorthand().map(|s| s.to_string()),
        is_branch: reference.name()?.starts_with("refs/heads/"),
        is_tag: reference.name()?.starts_with("refs/tags/"),
        is_remote: reference.name()?.starts_with("refs/remotes/"),
        is_symbolic: reference.is_symbolic(),
        target_type: reference.target().map(|t| t.kind()),
        oid: reference.target().map(|t| t.id()),
    })
}

#[derive(Debug)]
struct ReferenceMetadata {
    name: String,
    target: Option<String>,
    shorthand: Option<String>,
    is_branch: bool,
    is_tag: bool,
    is_remote: bool,
    is_symbolic: bool,
    target_type: Option<ObjectType>,
    oid: Option<git2::Oid>,
}

// Additional metadata functions
fn get_branch_info(repo: &Repository, branch_name: &str) -> Option<BranchInfo> {
    let full_name = format!("refs/heads/{}", branch_name);
    let reference = repo.find_reference(&full_name).ok()?;

    let mut branch_info = BranchInfo {
        name: branch_name.to_string(),
        target: reference.target().map(|t| t.to_string()),
        is_head: false,
        upstream: None,
    };

    // Check if it's the HEAD branch
    if repo.head().ok()?.name() == Some(&full_name) {
        branch_info.is_head = true;
    }

    // Get upstream tracking information
    if let Ok(branch) = repo.branch_from_name(branch_name) {
        if let Some(upstream) = branch.upstream() {
            branch_info.upstream = Some(upstream.name().unwrap_or("").to_string());
        }
    }

    Some(branch_info)
}

#[derive(Debug)]
struct BranchInfo {
    name: String,
    target: Option<String>,
    is_head: bool,
    upstream: Option<String>,
}
```

### Reference Name Validation

```rust
use git2::Repository;

fn is_valid_reference_name(name: &str) -> bool {
    git2::Reference::is_valid_name(name)
}

fn is_valid_branch_name(name: &str) -> bool {
    // Branch names must be valid references and not contain certain patterns
    is_valid_reference_name(name) &&
    !name.contains("..") &&
    !name.startswith("@{") &&
    !name.contains("^") &&
    !name.contains(":") &&
    !name.contains(" ")
}

fn sanitize_branch_name(name: &str) -> String {
    name.trim()
        .replace("..", "")
        .replace("@{", "")
        .replace("^", "")
        .replace(":", "")
        .replace(" ", "-")
        .to_lowercase()
}
```

## Complete CRUD Example

```rust
use git2::Repository;
use std::collections::HashMap;

pub struct ReferenceManager {
    repo: Repository,
}

impl ReferenceManager {
    pub fn new(repo_path: &str) -> Result<Self, git2::Error> {
        let repo = Repository::open(repo_path)?;
        Ok(Self { repo })
    }

    // CREATE (already exists in git2-rs)
    // Use repo.reference() or repo.branch() to create references

    // READ
    pub fn list_all(&self) -> Result<Vec<String>, git2::Error> {
        list_all_references(&self.repo)
    }

    pub fn list_branches(&self) -> Result<Vec<String>, git2::Error> {
        list_all_branches(&self.repo)
    }

    pub fn list_tags(&self) -> Result<Vec<String>, git2::Error> {
        list_all_tags(&self.repo)
    }

    pub fn get_metadata(&self, name: &str) -> Option<ReferenceMetadata> {
        get_reference_metadata(&self.repo, name)
    }

    // UPDATE (already exists in git2-rs)
    // Use reference.set_target() to update references

    // DELETE
    pub fn delete(&self, name: &str) -> Result<(), git2::Error> {
        safe_delete_reference(&self.repo, name)
    }

    pub fn delete_branch(&self, branch_name: &str) -> Result<(), git2::Error> {
        delete_branch_safely(&self.repo, branch_name)
    }

    pub fn delete_by_pattern(&self, pattern: &str) -> Result<usize, git2::Error> {
        delete_references_by_pattern(&self.repo, pattern)
    }

    // UTILITY METHODS
    pub fn exists(&self, name: &str) -> bool {
        reference_exists(&self.repo, name)
    }

    pub fn branch_exists(&self, name: &str) -> bool {
        branch_exists(&self.repo, name)
    }

    pub fn tag_exists(&self, name: &str) -> bool {
        tag_exists(&self.repo, name)
    }

    pub fn validate_name(&self, name: &str) -> bool {
        is_valid_reference_name(name)
    }
}
```

## Error Handling Best Practices

### Common Error Types

```rust
use git2::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReferenceError {
    #[error("Reference not found: {0}")]
    NotFound(String),

    #[error("Invalid reference name: {0}")]
    InvalidName(String),

    #[error("Cannot delete HEAD reference: {0}")]
    CannotDeleteHead(String),

    #[error("Reference already exists: {0}")]
    AlreadyExists(String),

    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),
}

impl From<git2::Error> for ReferenceError {
    fn from(err: git2::Error) -> Self {
        match err.code() {
            git2::ErrorCode::NotFound => ReferenceError::NotFound(err.message().to_string()),
            _ => ReferenceError::GitError(err),
        }
    }
}
```

### Safe Operations

```rust
pub fn safe_reference_operation<F, R>(repo: &Repository, ref_name: &str, operation: F) -> Result<R, ReferenceError>
where
    F: FnOnce(&Repository, &str) -> Result<R, git2::Error>,
{
    // Validate reference name first
    if !is_valid_reference_name(ref_name) {
        return Err(ReferenceError::InvalidName(ref_name.to_string()));
    }

    // Perform the operation
    operation(repo, ref_name).map_err(|e| match e.code() {
        git2::ErrorCode::NotFound => ReferenceError::NotFound(ref_name.to_string()),
        _ => ReferenceError::GitError(e),
    })
}
```

## Gotchas and Best Practices

### 1. Reference Name Gotchas

- Always use full reference names (e.g., "refs/heads/main", not just "main")
- Reference names are case-sensitive
- Symbolic references can cause confusion - use `reference.resolve()` to get the target
- HEAD reference is special and protected

### 2. Performance Considerations

- Use `references_glob()` for pattern-based operations
- Batch operations are more efficient than individual operations
- Cache reference metadata when needed
- Consider using lazy evaluation for large repositories

### 3. Safety Precautions

- Never delete references without proper validation
- Implement backup strategies before bulk deletions
- Consider using transactions for complex operations
- Validate repository state before operations

### 4. Namespace Management

- Keep track of standard namespaces: refs/heads, refs/tags, refs/remotes
- Use custom namespaces for project-specific references
- Document custom namespace conventions

### 5. Thread Safety

- git2-rs Repository objects are not Send or Sync
- Clone the Repository for use in other threads
- Use Arc<Mutex<Repository>> for shared access

### 6. Memory Management

- Large repositories may have many references
- Process references in batches when memory is constrained
- Use iterators instead of collecting all references in memory

## Additional Resources

### Official Documentation

- **git2-rs API Documentation**: https://docs.rs/git2/latest/git2/
- **git2-rs GitHub Repository**: https://github.com/rust-lang/git2-rs
- **libgit2 Documentation**: https://libgit2.org/docs/

### Example Projects

- **git2-rs examples**: https://github.com/rust-lang/git2-rs/tree/master/examples
- **gitoxide**: Advanced Git implementation in Rust using git2-rs: https://github.com/Byron/gitoxide

### Community Resources

- **Stack Overflow**: https://stackoverflow.com/questions/tagged/git2-rs
- **Rust Users Forum**: https://users.rust-lang.org/tags/git2-rs
- **GitHub Discussions**: https://github.com/rust-lang/git2-rs/discussions

## Testing Reference Operations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create initial commit
        let mut index = repo.index().unwrap();
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();

        let tree = index.write_tree().unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();

        (temp_dir, repo)
    }

    #[test]
    fn test_reference_deletion() {
        let (_temp_dir, repo) = create_test_repo();

        // Create a test branch
        repo.branch("test", &repo.head().unwrap().target().unwrap(), false).unwrap();

        assert!(repo.find_reference("refs/heads/test").is_ok());

        // Delete the branch
        let manager = ReferenceManager::new(repo.path().to_str().unwrap()).unwrap();
        manager.delete_branch("test").unwrap();

        assert!(repo.find_reference("refs/heads/test").is_err());
    }

    #[test]
    fn test_reference_listing() {
        let (_temp_dir, repo) = create_test_repo();

        // Create some references
        repo.branch("feature", &repo.head().unwrap().target().unwrap(), false).unwrap();
        repo.branch("bugfix", &repo.head().unwrap().target().unwrap(), false).unwrap();

        let manager = ReferenceManager::new(repo.path().to_str().unwrap()).unwrap();
        let branches = manager.list_branches().unwrap();

        assert_eq!(branches.len(), 3); // + main branch
        assert!(branches.contains(&"refs/heads/main".to_string()));
        assert!(branches.contains(&"refs/heads/feature".to_string()));
        assert!(branches.contains(&"refs/heads/bugfix".to_string()));
    }
}
```

This comprehensive guide provides all the information needed to implement a robust CRUD API for Git references using git2-rs.