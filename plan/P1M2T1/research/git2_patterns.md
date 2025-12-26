# git2-rs Repository Wrapper Patterns and Best Practices

## Table of Contents
1. [Repository Initialization and Opening Patterns](#repository-initialization-and-opening-patterns)
2. [Reference Management](#reference-management)
3. [Blob/Tree/Commit Object Creation](#blobtree-commit-object-creation)
4. [Error Handling Patterns](#error-handling-patterns)
5. [Common Gotchas and Best Practices](#common-gotchas-and-best-practices)
6. [Performance Considerations](#performance-considerations)
7. [Repository Wrapper Design Patterns](#repository-wrapper-design-patterns)

## Repository Initialization and Opening Patterns

### Basic Repository Opening

```rust
use git2::Repository;
use crate::core::error::{JinError, Result};

/// Open an existing repository at the specified path
pub fn open_repository(path: &str) -> Result<Repository> {
    Repository::open(path)
        .map_err(|e| JinError::RepoNotFound {
            path: path.to_string(),
        })
}

/// Open repository from current working directory
pub fn open_repository_from_cwd() -> Result<Repository> {
    Repository::open_from_env()
        .map_err(|e| JinError::Message(format!("Failed to open repo from environment: {}", e)))
}

/// Open repository with explicit search for .git directory
pub fn open_repository_recursive(path: &str) -> Result<Repository> {
    Repository::open_ext(path, git2::RepositoryOpenFlags::SEARCH_FROMcwd, &[])
        .map_err(|e| JinError::RepoNotFound {
            path: path.to_string(),
        })
}
```

### Repository Initialization

```rust
/// Initialize a new bare repository
pub fn init_bare_repository(path: &str) -> Result<Repository> {
    Repository::init_bare(path)
        .map_err(|e| JinError::Message(format!("Failed to init bare repo: {}", e)))
}

/// Initialize a new non-bare repository
pub fn init_repository(path: &str) -> Result<Repository> {
    Repository::init(path)
        .map_err(|e| JinError::Message(format!("Failed to init repo: {}", e)))
}

/// Initialize a new repository with options
pub fn init_repository_with_options(
    path: &str,
    bare: bool,
    odb: Option<git2::Odb>,
) -> Result<Repository> {
    let mut opts = git2::RepositoryInitOptions::new();
    opts.bare(bare);

    Repository::init_ext(path, Some(&opts), odb.as_ref())
        .map_err(|e| JinError::Message(format!("Failed to init repo with options: {}", e)))
}
```

### Repository Discovery Pattern

```rust
/// Find repository by walking up directory tree
pub fn find_repository(path: &str) -> Result<Repository> {
    let current_path = std::path::Path::new(path);
    let mut path_to_check = current_path.to_path_buf();

    while path_to_check.parent().is_some() {
        if path_to_check.join(".git").exists() {
            return Repository::open(&path_to_check)
                .map_err(|e| JinError::Message(format!("Failed to open repo: {}", e)));
        }
        path_to_check = path_to_check.parent()
            .unwrap()
            .to_path_buf();
    }

    Err(JinError::RepoNotFound {
        path: path.to_string(),
    })
}
```

## Reference Management

### Finding References

```rust
/// Get a reference by name
pub fn find_reference(repo: &Repository, ref_name: &str) -> Result<git2::Reference> {
    repo.find_reference(ref_name)
        .map_err(|e| JinError::RefNotFound {
            name: ref_name.to_string(),
            layer: "unknown".to_string(),
        })
}

/// Get a reference by short name (e.g., "main" -> "refs/heads/main")
pub fn find_reference_short(repo: &Repository, short_name: &str) -> Result<git2::Reference> {
    let full_name = if short_name.starts_with("refs/") {
        short_name.to_string()
    } else {
        match short_name {
            "HEAD" => "HEAD".to_string(),
            _ => format!("refs/heads/{}", short_name),
        }
    };

    find_reference(repo, &full_name)
}

/// Get all references matching a pattern
pub fn find_references_matching(repo: &Repository, pattern: &str) -> Result<Vec<git2::Reference>> {
    let mut references = Vec::new();

    for entry in repo.references()? {
        let reference = entry
            .map_err(|e| JinError::Message(format!("Error reading reference: {}", e)))?;

        if reference.name().map_or(false, |name| name.contains(pattern)) {
            references.push(reference);
        }
    }

    Ok(references)
}
```

### Creating References

```rust
/// Create a new reference from a commit
pub fn create_reference_from_commit(
    repo: &Repository,
    ref_name: &str,
    commit: &git2::Commit,
    force: bool,
) -> Result<git2::Reference> {
    repo.reference(ref_name, commit.id(), force, &format!("Create {}", ref_name))
        .map_err(|e| if force {
            JinError::Message(format!("Failed to create reference even with force: {}", e))
        } else {
            JinError::RefExists {
                name: ref_name.to_string(),
                layer: "unknown".to_string(),
            }
        })
}

/// Create a new symbolic reference
pub fn create_symbolic_reference(
    repo: &Repository,
    ref_name: &str,
    target: &str,
    force: bool,
) -> Result<git2::Reference> {
    repo.reference_symbolic(ref_name, target, force, &format!("Create symbolic {}", ref_name))
        .map_err(|e| if force {
            JinError::Message(format!("Failed to create symbolic reference even with force: {}", e))
        } else {
            JinError::RefExists {
                name: ref_name.to_string(),
                layer: "unknown".to_string(),
            }
        })
}
```

### Updating References

```rust
/// Update a reference to point to a new commit
pub fn update_reference_to_commit(
    repo: &Repository,
    ref_name: &str,
    commit: &git2::Commit,
    message: &str,
) -> Result<()> {
    let mut reference = find_reference(repo, ref_name)?;

    reference.set_target(commit.id(), message)
        .map_err(|e| JinError::Message(format!("Failed to update reference: {}", e)))?;

    Ok(())
}

/// Atomic reference update with callback
pub fn update_reference_atomically<F>(
    repo: &Repository,
    ref_name: &str,
    update_fn: F,
) -> Result<()>
where
    F: FnOnce(&mut git2::Reference) -> Result<()>,
{
    let mut reference = find_reference(repo, ref_name)?;

    update_fn(&mut reference)?;

    Ok(())
}
```

## Blob/Tree/Commit Object Creation

### Blob Creation

```rust
/// Create a blob from file contents
pub fn create_blob_from_file(repo: &Repository, path: &str) -> Result<git2::Blob> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| JinError::FileNotFound {
            path: path.to_string(),
        })?;

    repo.blob_stream(&mut file)
        .map_err(|e| JinError::Message(format!("Failed to create blob from file: {}", e)))
}

/// Create a blob from in-memory data
pub fn create_blob_from_data(repo: &Repository, data: &[u8]) -> Result<git2::Blob> {
    repo.blob(data)
        .map_err(|e| JinError::Message(format!("Failed to create blob from data: {}", e)))
}

/// Create blob with content safety check
pub fn create_safe_blob(repo: &Repository, data: &[u8) -> Result<git2::Blob> {
    // Check for binary data
    if data.iter().any(|&b| b == 0 || b == 0x7F) {
        return Err(JinError::BinaryFileNotSupported {
            path: "in-memory blob".to_string(),
        });
    }

    create_blob_from_data(repo, data)
}
```

### Tree Creation

```rust
/// Create a tree from a directory
pub fn create_tree_from_directory(repo: &Repository, path: &str) -> Result<git2::Tree> {
    let walkdir = walkdir::WalkDir::new(path);
    let mut builder = repo.treebuilder(None)?;

    for entry in walkdir {
        let entry = entry.map_err(|e| JinError::Message(format!("Directory walk error: {}", e)))?;

        if entry.file_type().is_dir() {
            continue;
        }

        let relative_path = entry.path()
            .strip_prefix(path)
            .map_err(|e| JinError::Message(format!("Path error: {}", e)))?;

        let blob = create_blob_from_file(repo, entry.path().to_str()
            .ok_or_else(|| JinError::Message("Invalid UTF-8 path".to_string()))?)?;

        builder.insert(
            relative_path.to_str()
                .ok_or_else(|| JinError::Message("Invalid UTF-8 path".to_string()))?,
            blob.id(),
            git2::FileMode::from_bits(0o100644).unwrap(),
        )?;
    }

    let tree_id = builder.write()?;
    repo.find_tree(tree_id)
        .map_err(|e| JinError::Message(format!("Failed to find created tree: {}", e)))
}

/// Create tree from index entries
pub fn create_tree_from_index(repo: &Repository, index: &git2::Index) -> Result<git2::Tree> {
    let mut builder = repo.treebuilder(None)?;

    for entry in index.iter() {
        let id = entry.id;
        builder.insert(entry.path, id, entry.mode)?;
    }

    let tree_id = builder.write()?;
    repo.find_tree(tree_id)
        .map_err(|e| JinError::Message(format!("Failed to find created tree: {}", e)))
}
```

### Commit Creation

```rust
/// Create a commit with parent commits
pub fn create_commit(
    repo: &Repository,
    update_ref: Option<&str>,
    author: &git2::Signature,
    committer: &git2::Signature,
    message: &str,
    tree: &git2::Tree,
    parents: &[&git2::Commit],
) -> Result<git2::Oid> {
    repo.commit(
        update_ref,
        author,
        committer,
        message,
        tree,
        parents,
    ).map_err(|e| JinError::Message(format!("Failed to create commit: {}", e)))
}

/// Create a commit from working directory changes
pub fn create_commit_from_workdir(
    repo: &Repository,
    update_ref: &str,
    author: &git2::Signature,
    committer: &git2::Signature,
    message: &str,
) -> Result<git2::Oid> {
    // Get the index
    let mut index = repo.index()?;
    index.read(true)?;

    // Add all tracked files
    repo.add_all(&["*"], None, None)?;

    // Stage the changes
    index.write()?;

    // Create tree from index
    let tree = create_tree_from_index(repo, &index)?;

    // Get parent commit
    let parents = if let Ok(head) = repo.head() {
        let parent_commit = repo.find_commit(head.target()?)?;
        vec![&parent_commit]
    } else {
        vec![]
    };

    // Create commit
    create_commit(repo, Some(update_ref), author, committer, message, &tree, &parents)
}
```

## Error Handling Patterns

### git2::Error Conversion

```rust
/// Convert git2::Error with context
pub fn git_error_with_context(e: git2::Error, context: &str) -> JinError {
    match e.code() {
        git2::ErrorCode::NotFound => match e.message() {
            msg if msg.contains("reference") => JinError::RefNotFound {
                name: "unknown".to_string(),
                layer: "unknown".to_string(),
            },
            msg if msg.contains("repository") => JinError::RepoNotFound {
                path: "unknown".to_string(),
            },
            _ => JinError::Message(format!("{}: {}", context, e)),
        },
        git2::ErrorCode::Generic => JinError::Message(format!("{}: {}", context, e)),
        git2::ErrorCode::Conflict => JinError::TransactionConflict {
            conflict: e.message().to_string(),
        },
        git2::ErrorCode::Locked => JinError::Message(format!("{}: Resource locked: {}", context, e)),
        git2::ErrorCode::Modified => JinError::Message(format!("{}: Resource modified: {}", context, e)),
        git2::ErrorCode::NotFound => JinError::Message(format!("{}: Not found: {}", context, e)),
        _ => JinError::Message(format!("{}: {}", context, e)),
    }
}

/// Wrap git2 operation with error handling
pub fn git_operation<T, F>(context: &str, operation: F) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, git2::Error>,
{
    operation().map_err(|e| git_error_with_context(e, context))
}
```

### Retry Logic

```rust
/// Retry a git operation with exponential backoff
pub fn retry_git_operation<T, F>(
    context: &str,
    mut operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> std::result::Result<T, git2::Error>,
{
    let mut attempt = 0;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                if attempt > max_retries {
                    return Err(git_error_with_context(e, context));
                }

                if !is_retryable_git_error(&e) {
                    return Err(git_error_with_context(e, context));
                }

                // Exponential backoff
                let delay = std::time::Duration::from_millis(2u64.pow(attempt.min(6)) * 100);
                std::thread::sleep(delay);
            }
        }
    }
}

/// Check if a git error is retryable
pub fn is_retryable_git_error(e: &git2::Error) -> bool {
    matches!(
        e.code(),
        git2::ErrorCode::Locked
            | git2::ErrorCode::Modified
            | git2::ErrorCode::Conflict
    )
}
```

### Transaction Pattern

```rust
/// Git operation transaction
pub struct GitTransaction<'a> {
    repo: &'a Repository,
    operations: Vec<Box<dyn FnOnce(&Repository) -> Result<()>>>,
}

impl<'a> GitTransaction<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        GitTransaction {
            repo,
            operations: Vec::new(),
        }
    }

    pub fn add_operation<F>(&mut self, operation: F)
    where
        F: FnOnce(&Repository) -> Result<()> + 'static,
    {
        self.operations.push(Box::new(operation));
    }

    pub fn execute(&mut self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.read(true)?;

        for operation in &mut self.operations {
            operation(self.repo)?;
        }

        index.write()?;
        Ok(())
    }
}
```

## Common Gotchas and Best Practices

### 1. Repository Lifetime Management

**Gotcha:** git2-rs objects maintain references to the repository, causing dangling pointers.

**Solution:** Always keep the Repository object alive while using git2 objects.

```rust
// Good
fn process_repo(repo_path: &str) -> Result<String> {
    let repo = Repository::open(repo_path)?; // Keep repo alive
    let head = repo.head()?; // References repo
    let commit = repo.find_commit(head.target()?)?; // References repo
    Ok(commit.id().to_string())
}
```

### 2. String Encoding

**Gotcha:** git2-rs expects UTF-8 strings. Non-UTF-8 paths will cause errors.

**Solution:** Validate and convert paths.

```rust
pub fn safe_path_operation(repo: &Repository, path: &str) -> Result<()> {
    // Validate UTF-8
    if !path.is_ascii() {
        return Err(JinError::Message("Non-ASCII paths not supported".to_string()));
    }

    // Continue with operation...
    Ok(())
}
```

### 3. Memory Management

**Gotcha:** git2 objects like Commit, Tree, Blob are not cheap to create.

**Solution:** Cache when possible and minimize object creation.

```rust
struct RepositoryCache {
    commits: std::collections::HashMap<git2::Oid, git2::Commit>,
    trees: std::collections::HashMap<git2::Oid, git2::Tree>,
}

impl RepositoryCache {
    fn get_or_create_commit(&mut self, repo: &Repository, id: git2::Oid) -> Result<git2::Commit> {
        if let Some(commit) = self.commits.get(&id) {
            return Ok(commit.clone());
        }

        let commit = repo.find_commit(id)?;
        self.commits.insert(id, commit.clone());
        Ok(commit)
    }
}
```

### 4. Repository State Checking

**Gotcha:** Not all operations work on all repository types.

**Solution:** Check repository state before operations.

```rust
pub fn ensure_workdir_repo(repo: &Repository) -> Result<()> {
    if repo.is_bare() {
        return Err(JinError::BareRepo {
            path: repo.path().to_string_lossy().to_string(),
        });
    }

    if repo.is_empty()? {
        return Err(JinError::Message("Repository is empty".to_string()));
    }

    Ok(())
}
```

### 5. Atomic Operations

**Gotcha:** Git operations should be atomic to prevent corruption.

**Solution:** Use reference transactions for multiple updates.

```rust
pub fn atomic_update(repo: &Repository) -> Result<()> {
    let mut transaction = repo.reference_transaction()?;

    // Add operations to transaction
    transaction.lock(&["refs/heads/new-feature"], true)?;

    // Perform updates
    // ...

    // Commit transaction
    transaction.commit("atomic update")?;

    Ok(())
}
```

## Performance Considerations

### 1. Batch Operations

```rust
/// Batch file operations to reduce round trips
pub fn batch_file_operations(
    repo: &Repository,
    operations: Vec<BatchFileOperation>,
) -> Result<()> {
    for operation in operations {
        match operation {
            BatchFileOperation::Add(path) => {
                repo.add_all(&[path], None, None)?;
            },
            BatchFileOperation::Remove(path) => {
                repo.remove(&[path], None)?;
            },
        }
    }

    Ok(())
}

enum BatchFileOperation {
    Add(String),
    Remove(String),
}
```

### 2. Lazy Loading

```rust
/// Lazy commit loading with caching
pub struct LazyCommit {
    repo: Repository,
    oid: git2::Oid,
    commit: Option<git2::Commit>,
}

impl LazyCommit {
    pub fn new(repo: Repository, oid: git2::Oid) -> Self {
        LazyCommit {
            repo,
            oid,
            commit: None,
        }
    }

    pub fn get(&mut self) -> Result<&git2::Commit> {
        if self.commit.is_none() {
            self.commit = Some(self.repo.find_commit(self.oid)?);
        }

        Ok(self.commit.as_ref().unwrap())
    }
}
```

### 3. Index Optimization

```rust
/// Optimize index operations
pub fn optimized_index_operation(repo: &Repository) -> Result<()> {
    // Read index once
    let mut index = repo.index()?;
    index.read(true)?;

    // Batch modifications
    // ...

    // Write once
    index.write()?;

    Ok(())
}
```

## Repository Wrapper Design Patterns

### 1. Wrapper Struct Pattern

```rust
pub struct JinRepo {
    inner: Repository,
    path: String,
    cache: RepositoryCache,
}

impl JinRepo {
    pub fn open(path: &str) -> Result<Self> {
        let inner = Repository::open(path)
            .map_err(|e| JinError::RepoNotFound {
                path: path.to_string(),
            })?;

        Ok(JinRepo {
            inner,
            path: path.to_string(),
            cache: RepositoryCache::new(),
        })
    }

    pub fn find_layer_commit(&mut self, layer: &str) -> Result<git2::Commit> {
        // Use cache
        if let Some(commit) = self.cache.get_layer_commit(layer) {
            return Ok(commit.clone());
        }

        // Load from repository
        let ref_name = format!("refs/heads/{}", layer);
        let reference = self.inner.find_reference(&ref_name)?;
        let commit = self.inner.find_commit(reference.target()?)?;

        // Cache the result
        self.cache.cache_layer_commit(layer, commit.clone());

        Ok(commit)
    }

    // Other wrapper methods...
}
```

### 2. Repository Context Pattern

```rust
pub struct RepoContext<'a> {
    repo: &'a mut Repository,
    layer: String,
    author: git2::Signature,
    committer: git2::Signature,
}

impl<'a> RepoContext<'a> {
    pub fn new(
        repo: &'a mut Repository,
        layer: String,
        author: git2::Signature,
        committer: git2::Signature,
    ) -> Self {
        RepoContext {
            repo,
            layer,
            author,
            committer,
        }
    }

    pub fn commit_file(&mut self, path: &str, content: &[u8], message: &str) -> Result<git2::Oid> {
        // Create blob
        let blob = self.repo.blob(content)?;

        // Build tree
        let mut builder = self.repo.treebuilder(None)?;
        builder.insert(path, blob, git2::FileMode::Blob)?;
        let tree = self.repo.find_tree(builder.write()?)?;

        // Get parent
        let parent = if let Ok(head) = self.repo.head() {
            Some(self.repo.find_commit(head.target()?)?)
        } else {
            None
        };

        // Create commit
        let parents = if let Some(parent) = parent {
            vec![&parent]
        } else {
            vec![]
        };

        let ref_name = format!("refs/heads/{}", self.layer);
        self.repo.commit(
            Some(&ref_name),
            &self.author,
            &self.committer,
            message,
            &tree,
            &parents,
        ).map_err(|e| git_error_with_context(e, "commit_file"))
    }
}
```

### 3. Builder Pattern for Repository

```rust
pub struct RepositoryBuilder {
    path: String,
    bare: bool,
    init_options: Option<git2::RepositoryInitOptions>,
}

impl RepositoryBuilder {
    pub fn new(path: String) -> Self {
        RepositoryBuilder {
            path,
            bare: false,
            init_options: None,
        }
    }

    pub fn bare(mut self, bare: bool) -> Self {
        self.bare = bare;
        self
    }

    pub fn with_init_options(mut self, options: git2::RepositoryInitOptions) -> Self {
        self.init_options = Some(options);
        self
    }

    pub fn build(self) -> Result<Repository> {
        if let Some(options) = self.init_options {
            Repository::init_ext(&self.path, Some(&options), None)
        } else if self.bare {
            Repository::init_bare(&self.path)
        } else {
            Repository::init(&self.path)
        }.map_err(|e| JinError::Message(format!("Failed to create repository: {}", e)))
    }
}
```

## Additional Resources

### Official Documentation
- [git2-rs on docs.rs](https://docs.rs/git2/) - Official API documentation
- [libgit2 documentation](https://libgit2.org/docs/) - Underlying C library documentation
- [git2-rs GitHub repository](https://github.com/rust-lang/git2-rs) - Source code and examples

### Best Practices Guides
- [Rust Git Operations Guide](https://github.com/rust-lang/git2-rs/blob/master/EXAMPLES.md) - Collection of examples
- [libgit2 Best Practices](https://libgit2.org/docs/guides/best-practices/) - General Git best practices

### Error Handling
- [git2::Error documentation](https://docs.rs/git2/latest/git2/struct.Error.html) - Error types and codes
- [thiserror crate documentation](https://docs.rs/thiserror/latest/thiserror/) - Error deriving

---

*Note: This document provides patterns and examples for working with git2-rs. Always consult the official documentation for the most current API information.*