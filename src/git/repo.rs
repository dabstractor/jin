//! Jin repository wrapper
//!
//! This module provides [`JinRepo`], a wrapper around `git2::Repository`
//! for Jin's phantom Git layer. Jin maintains a bare repository at `~/.jin/`
//! that stores all layer configurations.

use crate::core::{JinError, Result};
use git2::{Repository, RepositoryInitOptions};
use std::path::PathBuf;

/// Wrapper around `git2::Repository` for Jin's phantom Git layer.
///
/// Jin maintains a bare repository at `~/.jin/` that stores all layer
/// configurations. This wrapper provides Jin-specific operations while
/// exposing the underlying Repository for advanced use cases.
///
/// # Example
///
/// ```no_run
/// use jin::git::JinRepo;
///
/// // Open or create the Jin repository
/// let repo = JinRepo::open_or_create()?;
///
/// // Access the underlying repository
/// let inner = repo.inner();
/// # Ok::<(), jin::JinError>(())
/// ```
pub struct JinRepo {
    repo: Repository,
    path: PathBuf,
}

impl std::fmt::Debug for JinRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JinRepo")
            .field("path", &self.path)
            .field("is_bare", &self.repo.is_bare())
            .finish()
    }
}

impl JinRepo {
    /// Opens an existing Jin repository.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the repository doesn't exist or is corrupted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::JinRepo;
    ///
    /// let repo = JinRepo::open()?;
    /// # Ok::<(), jin::JinError>(())
    /// ```
    pub fn open() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open_at(&path)
    }

    /// Opens an existing Jin repository at a specific path.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if the repository doesn't exist or is corrupted.
    pub fn open_at(path: &PathBuf) -> Result<Self> {
        let repo = Repository::open_bare(path)?;
        Ok(Self {
            repo,
            path: path.clone(),
        })
    }

    /// Creates a new Jin repository.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if creation fails or the repository already exists.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::JinRepo;
    ///
    /// let repo = JinRepo::create()?;
    /// # Ok::<(), jin::JinError>(())
    /// ```
    pub fn create() -> Result<Self> {
        let path = Self::default_path()?;
        Self::create_at(&path)
    }

    /// Creates a new Jin repository at a specific path.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Git` if creation fails or the repository already exists.
    pub fn create_at(path: &PathBuf) -> Result<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Initialize bare repository with options
        let mut opts = RepositoryInitOptions::new();
        opts.bare(true);
        opts.mkdir(true);
        opts.description("Jin phantom layer repository");

        let repo = Repository::init_opts(path, &opts)?;
        Ok(Self {
            repo,
            path: path.clone(),
        })
    }

    /// Opens an existing or creates a new Jin repository.
    ///
    /// This is the preferred method for most use cases, as it handles
    /// both first-time initialization and subsequent opens gracefully.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jin::git::JinRepo;
    ///
    /// let repo = JinRepo::open_or_create()?;
    /// # Ok::<(), jin::JinError>(())
    /// ```
    pub fn open_or_create() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open_or_create_at(&path)
    }

    /// Opens an existing or creates a new Jin repository at a specific path.
    pub fn open_or_create_at(path: &PathBuf) -> Result<Self> {
        match Self::open_at(path) {
            Ok(repo) => Ok(repo),
            Err(_) => Self::create_at(path),
        }
    }

    /// Returns the default Jin repository path (`~/.jin/`).
    ///
    /// Can be overridden with the `JIN_DIR` environment variable for testing.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Config` if the home directory cannot be determined.
    pub fn default_path() -> Result<PathBuf> {
        // Check for JIN_DIR environment variable first (for testing)
        if let Ok(jin_dir) = std::env::var("JIN_DIR") {
            return Ok(PathBuf::from(jin_dir));
        }

        dirs::home_dir()
            .map(|h| h.join(".jin"))
            .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
    }

    /// Returns the path to the Jin repository.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns a reference to the underlying `git2::Repository`.
    ///
    /// Use this for advanced operations not covered by JinRepo methods.
    pub fn inner(&self) -> &Repository {
        &self.repo
    }

    /// Returns a mutable reference to the underlying `git2::Repository`.
    pub fn inner_mut(&mut self) -> &mut Repository {
        &mut self.repo
    }

    /// Checks if this is a valid Jin repository.
    ///
    /// A valid Jin repository is a bare repository that may contain
    /// refs under the `refs/jin/` namespace.
    pub fn is_valid(&self) -> bool {
        self.repo.is_bare()
    }

    /// Checks if the repository has any refs in the Jin namespace.
    pub fn has_jin_refs(&self) -> bool {
        self.repo
            .references_glob("refs/jin/*")
            .map(|refs| refs.count() > 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, JinRepo) {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

    #[test]
    fn test_create_jin_repo() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");

        let repo = JinRepo::create_at(&repo_path).unwrap();
        assert!(repo.is_valid());
        assert!(repo.inner().is_bare());
        assert_eq!(repo.path(), &repo_path);
    }

    #[test]
    fn test_open_existing_repo() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");

        // Create first
        let _repo1 = JinRepo::create_at(&repo_path).unwrap();

        // Then open
        let repo2 = JinRepo::open_at(&repo_path).unwrap();
        assert!(repo2.is_valid());
    }

    #[test]
    fn test_open_nonexistent_fails() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");

        let result = JinRepo::open_at(&repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_or_create_new() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");

        // Should create because it doesn't exist
        let repo = JinRepo::open_or_create_at(&repo_path).unwrap();
        assert!(repo.is_valid());
    }

    #[test]
    fn test_open_or_create_existing() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");

        // Create first
        let _repo1 = JinRepo::create_at(&repo_path).unwrap();

        // Should open because it exists
        let repo2 = JinRepo::open_or_create_at(&repo_path).unwrap();
        assert!(repo2.is_valid());
    }

    #[test]
    fn test_repo_is_bare() {
        let (_temp, repo) = create_test_repo();
        assert!(repo.inner().is_bare());
    }

    #[test]
    fn test_has_jin_refs_empty() {
        let (_temp, repo) = create_test_repo();
        assert!(!repo.has_jin_refs());
    }

    #[test]
    fn test_debug_impl() {
        let (_temp, repo) = create_test_repo();
        let debug_str = format!("{:?}", repo);
        assert!(debug_str.contains("JinRepo"));
        assert!(debug_str.contains("path"));
        assert!(debug_str.contains("is_bare"));
    }
}
