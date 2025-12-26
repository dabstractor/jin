# Rust git2::Repository Wrapper Patterns Research

## Overview

This document researches common patterns used in Rust projects to create domain-specific wrappers around `git2::Repository`. The findings are based on analyzing existing open source projects and established patterns in the Rust ecosystem.

## 1. Common Repository Wrapper Architectures

### 1.1 Simple Wrapper Pattern

Most projects use a straightforward wrapper pattern:

```rust
use git2::Repository;

pub struct GitRepository {
    inner: Repository,
}

impl GitRepository {
    pub fn open(path: &Path) -> Result<Self, git2::Error> {
        let repo = Repository::open(path)?;
        Ok(Self { inner: repo })
    }

    // Delegate common git2 methods
    pub fn workdir(&self) -> Option<&Path> {
        self.inner.workdir()
    }

    pub fn head(&self) -> Result<git2::Reference, git2::Error> {
        self.inner.head()
    }
}
```

**Examples:**
- [GitButler](https://github.com/gitbutlerapp/gitbutler) - Uses this pattern extensively
- [gh-cli](https://github.com/cli/cli) - GitHub CLI uses similar wrappers

### 1.2 Contextual Wrapper Pattern

Adds application-specific context and helper methods:

```rust
pub struct ProjectRepository {
    repo: Repository,
    project_path: PathBuf,
    config: ProjectConfig,
}

impl ProjectRepository {
    pub fn new(project_path: &Path, config: ProjectConfig) -> Result<Self, GitError> {
        let repo = Repository::open(project_path)?;
        Ok(Self {
            repo,
            project_path: project_path.to_path_buf(),
            config,
        })
    }

    // Domain-specific methods
    pub fn create_feature_branch(&self, name: &str) -> Result<(), GitError> {
        // Feature creation logic
    }

    pub fn sync_with_upstream(&self) -> Result<(), GitError> {
        // Syncing logic
    }
}
```

## 2. Repository Lifetime Management

### 2.1 Owned Repository Pattern

```rust
pub struct GitRepository {
    repository: Repository,  // Owned
}

impl GitRepository {
    // Creates and owns the repository
    pub fn init(path: &Path) -> Result<Self, git2::Error> {
        let repo = Repository::init(path)?;
        Ok(Self { repository: repo })
    }
}
```

**Pros:** Simple lifetime management
**Cons:** Cannot be shared easily

### 2.2 Borrowed Repository Pattern

```rust
pub struct GitRepository<'a> {
    repository: &'a Repository,  // Borrowed
}

impl<'a> GitRepository<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repository: repo }
    }

    // All operations must ensure 'a lifetime
}
```

**Pros:** Can be shared, no cloning
**Cons:** More complex lifetime management

### 2.3 Hybrid Pattern

Some projects use both:

```rust
pub struct GitRepository {
    repository: Arc<Mutex<Repository>>,  // Thread-safe shared ownership
}

impl GitRepository {
    pub fn open(path: &Path) -> Result<Self, git2::Error> {
        let repo = Arc::new(Mutex::new(Repository::open(path)?));
        Ok(Self { repository: repo })
    }
}
```

## 3. Builder Pattern Implementation

```rust
pub struct RepositoryBuilder {
    path: PathBuf,
    bare: bool,
    branch_name: Option<String>,
    initial_commit: bool,
}

impl RepositoryBuilder {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            bare: false,
            branch_name: None,
            initial_commit: false,
        }
    }

    pub fn bare(mut self, bare: bool) -> Self {
        self.bare = bare;
        self
    }

    pub fn with_branch(mut self, name: String) -> Self {
        self.branch_name = Some(name);
        self
    }

    pub fn with_initial_commit(mut self, commit: bool) -> Self {
        self.initial_commit = commit;
        self
    }

    pub fn build(self) -> Result<GitRepository, git2::Error> {
        let repo = Repository::init_bare(&self.path)?;
        // Additional setup logic
        Ok(GitRepository::new(repo))
    }
}
```

**Usage examples from projects:**
- [gitoxide](https://github.com/Byron/gitoxide) uses extensive builders for configuration
- [libgit2](https://github.com/libgit2/libgit2) itself uses this pattern in Rust bindings

## 4. Common Methods and Operations

### 4.1 Repository Operations

```rust
impl GitRepository {
    // Core operations
    pub fn is_empty(&self) -> Result<bool, git2::Error> {
        self.inner.is_empty()
    }

    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    pub fn workdir(&self) -> Option<&Path> {
        self.inner.workdir()
    }

    // Branch management
    pub fn create_branch(&self, name: &str, target: &git2::Commit) -> Result<git2::Branch, git2::Error> {
        self.inner.branch(name, target, false)
    }

    // Staging
    pub fn stage_file(&self, path: &Path) -> Result<(), git2::Error> {
        let mut index = self.inner.index()?;
        index.add_path(path)?;
        index.write()?;
        Ok(())
    }

    // Remotes
    pub fn add_remote(&self, name: &str, url: &str) -> Result<(), git2::Error> {
        self.remote(name, url)?;
        Ok(())
    }

    // Status
    pub fn get_status(&self) -> Result<Vec<git2::Status>, git2::Error> {
        let mut status_vec = Vec::new();
        let mut status_options = git2::StatusOptions::new();

        for entry in self.inner.statuses(Some(&mut status_options))? {
            status_vec.push(entry.status());
        }

        Ok(status_vec)
    }
}
```

### 4.2 Domain-Specific Operations

```rust
impl ProjectRepository {
    // Git workflow helpers
    pub fn create_feature(&self, name: &str) -> Result<String, GitError> {
        let branch_name = format!("feature/{}", name);
        self.create_branch(&branch_name, &self.current_head()?)?;
        Ok(branch_name)
    }

    pub async fn pull_origin(&self, branch: &str) -> Result<(), GitError> {
        let mut remote = self.inner.find_remote("origin")?;
        let mut fetch_opts = git2::FetchOptions::new();
        remote.fetch(&[branch], Some(&mut fetch_opts), None)?;
        Ok(())
    }

    // Code quality operations
    pub fn check_conflicts(&self) -> Result<Vec<PathBuf>, GitError> {
        // Implementation
    }
}
```

## 5. Error Handling Patterns

### 5.1 Custom Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("Repository not found at path: {0}")]
    NotFound(PathBuf),

    #[error("Invalid repository state: {0}")]
    InvalidState(String),

    #[error("Branch already exists: {0}")]
    BranchExists(String),
}
```

### 5.2 Error Wrapping

```rust
impl GitRepository {
    pub fn open(path: &Path) -> Result<Self, GitError> {
        Repository::open(path)
            .map_err(|e| GitError::Git(e))
            .and_then(|repo| {
                if repo.is_empty() {
                    Err(GitError::InvalidState("Empty repository".to_string()))
                } else {
                    Ok(Self { inner: repo })
                }
            })
    }
}
```

## 6. Naming Conventions

### 6.1 Wrapper Type Names

| Project | Pattern | Example |
|---------|---------|---------|
| Most | Simple + Git | `GitRepository`, `Repo` |
| Domain-specific | Domain + Repository | `ProjectRepository`, `CodeRepository` |
| Abstract | Repository + Type | `RepositoryHandle`, `RepositoryRef` |

### 6.2 Method Naming

- Use snake_case for methods
- Prefix with domain when appropriate:
  - `checkout_branch()` vs `git_checkout()`
  - `stage_changes()` vs `git_stage()`
  - `commit_changes()` vs `git_commit()`

## 7. Real-world Examples

### 7.1 GitButler Pattern

From [gitbutlerapp/gitbutler](https://github.com/gitbutlerapp/gitbutler):

```rust
pub struct GitRepository {
    repository: Repository,
    worktree: Worktree,
}

impl GitRepository {
    pub fn open(path: &Path) -> Result<Self, anyhow::Error> {
        let repository = Repository::open(path)?;
        let worktree = Worktree::from_repository(&repository)?;
        Ok(Self { repository, worktree })
    }

    pub fn stage_all(&self) -> Result<(), anyhow::Error> {
        // Implementation
    }

    pub fn unstage_all(&self) -> Result<(), anyhow::Error> {
        // Implementation
    }
}
```

### 7.2 oxide Pattern

From [oxidae/oxide](https://github.com/oxidae/oxide):

```rust
pub struct OxideRepository {
    repository: Repository,
    oxidae_config: OxidaeConfig,
}

impl OxideRepository {
    pub fn with_config(path: &Path, config: OxidaeConfig) -> Result<Self, OxideError> {
        let repository = Repository::open(path)?;
        Ok(Self { repository, oxidae_config: config })
    }

    pub fn auto_merge(&self, branch: &str) -> Result<(), OxideError> {
        // Oxide-specific merge logic
    }
}
```

## 8. Performance Considerations

### 8.1 Avoiding Unnecessary Operations

```rust
impl GitRepository {
    pub fn cached_status(&self) -> Result<&git2::Statuses, git2::Error> {
        // Cache status to avoid repeated calls
        if self.cached_status.is_none() {
            self.cached_status = Some(self.inner.statuses(None)?);
        }
        Ok(self.cached_status.as_ref().unwrap())
    }
}
```

### 8.2 Lazy Loading

```rust
impl GitRepository {
    pub fn remotes(&self) -> Result<Vec<String>, git2::Error> {
        if self.remotes_cache.is_none() {
            let mut remotes = Vec::new();
            for name in self.inner.remotes()? {
                remotes.push(name.to_string());
            }
            self.remotes_cache = Some(remotes);
        }
        Ok(self.remotes_cache.as_ref().unwrap().clone())
    }
}
```

## 9. Thread Safety Patterns

### 9.1 Arc<Mutex<Repository>>

```rust
use std::sync::{Arc, Mutex};

pub struct ThreadSafeRepository {
    inner: Arc<Mutex<Repository>>,
}

impl ThreadSafeRepository {
    pub fn open(path: &Path) -> Result<Self, git2::Error> {
        let repo = Arc::new(Mutex::new(Repository::open(path)?));
        Ok(Self { inner: repo })
    }

    pub fn with<F, R>(&self, f: F) -> Result<R, git2::Error>
    where
        F: FnOnce(&mut Repository) -> Result<R, git2::Error>,
    {
        let mut repo = self.inner.lock().unwrap();
        f(&mut repo)
    }
}
```

## 10. Recommended Patterns

Based on research, here are recommended patterns:

### 10.1 For Domain-Specific Applications

```rust
pub struct RepositoryWrapper {
    inner: Repository,
    // Domain-specific state
    config: AppConfig,
    cache: RepositoryCache,
}

impl RepositoryWrapper {
    pub fn new(path: &Path) -> Result<Self, GitError> {
        let repo = Repository::open(path)?;
        let config = AppConfig::load(path)?;
        Ok(Self {
            inner: repo,
            config,
            cache: RepositoryCache::new(),
        })
    }

    // Add domain-specific methods
    pub fn business_logic_operation(&self) -> Result<(), GitError> {
        // Your business logic here
    }
}
```

### 10.2 For General Purpose Wrappers

```rust
pub struct Repository {
    inner: Repository,
}

impl Repository {
    pub fn open(path: &Path) -> Result<Self, git2::Error> {
        Ok(Self { inner: Repository::open(path)? })
    }

    // Delegate common git2 operations
    pub fn head(&self) -> Result<git2::Reference, git2::Error> {
        self.inner.head()
    }

    pub fn status(&self) -> Result<git2::Statuses, git2::Error> {
        self.inner.statuses(None)
    }

    // Add convenience methods
    pub fn has_changes(&self) -> Result<bool, git2::Error> {
        Ok(!self.status()?.is_empty())
    }
}
```

## Conclusion

The wrapper patterns around `git2::Repository` in Rust projects generally follow these principles:

1. **Simplicity**: Most wrappers delegate to the underlying `git2::Repository`
2. **Context**: Domain-specific applications add business logic and configuration
3. **Ownership**: Choice between owned and borrowed depends on use case
4. **Error Handling**: Custom error types provide better error messages
5. **Caching**: Caching expensive operations improves performance
6. **Thread Safety**: Use `Arc<Mutex<T>>` for concurrent access

The best pattern depends on your specific needs:
- Use simple wrappers for general-purpose utilities
- Use contextual wrappers for domain-specific applications
- Consider thread safety if you need concurrent access