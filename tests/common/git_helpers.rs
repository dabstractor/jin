//! Git helper utilities for Jin integration tests
//!
//! Provides utilities for handling Git lock files and test environment cleanup.
//! These utilities help prevent test failures due to stale Git lock files.

use std::fs;
use std::path::Path;

/// Cleans up stale Git lock files in a repository
///
/// This function removes common Git lock files that may be left behind
/// when tests fail or are interrupted. Lock files cleaned include:
/// - `.git/index.lock` - Index lock file
/// - `.git/HEAD.lock` - HEAD reference lock file
/// - `.git/config.lock` - Config lock file (for JIN_DIR)
/// - `.git/refs/heads/main.lock` - Main branch lock file
/// - `.git/packed-refs.lock` - Packed refs lock file
/// - All `.lock` files under `.git/refs/`
///
/// # Arguments
/// * `repo_path` - Path to the Git repository
///
/// # Returns
/// * `Ok(())` if cleanup succeeds or locks don't exist
/// * `Err` if filesystem operations fail
///
/// # Gotchas
/// - Silently ignores errors for individual lock files (they may not exist)
/// - Should be called in Drop implementations for automatic cleanup
pub fn cleanup_git_locks(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo_path.join(".git");

    // Only proceed if .git directory exists
    if !git_dir.exists() {
        return Ok(());
    }

    // Clean index.lock (most common stale lock)
    let index_lock = git_dir.join("index.lock");
    if index_lock.exists() {
        fs::remove_file(&index_lock)?;
    }

    // Clean config.lock (critical for JIN_DIR isolation)
    let config_lock = git_dir.join("config.lock");
    if config_lock.exists() {
        fs::remove_file(&config_lock)?;
    }

    // Clean HEAD.lock
    let head_lock = git_dir.join("HEAD.lock");
    if head_lock.exists() {
        fs::remove_file(&head_lock)?;
    }

    // Clean packed-refs.lock
    let packed_refs_lock = git_dir.join("packed-refs.lock");
    if packed_refs_lock.exists() {
        fs::remove_file(&packed_refs_lock)?;
    }

    // Clean other common lock files
    // Ignore errors for files that may not exist
    let lock_files = &[
        "refs/heads/main.lock",
        "refs/heads/master.lock",
    ];

    for lock_file in lock_files {
        let lock_path = git_dir.join(lock_file);
        if lock_path.exists() {
            let _ = fs::remove_file(&lock_path); // Ignore errors
        }
    }

    // Recursively clean all .lock files under refs directory
    let refs_dir = git_dir.join("refs");
    if refs_dir.exists() {
        clean_lock_files_recursive(&refs_dir)?;
    }

    Ok(())
}

/// Recursively remove all .lock files in a directory
fn clean_lock_files_recursive(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            clean_lock_files_recursive(&path)?;
        } else if path.extension().map_or(false, |ext| ext == "lock") {
            let _ = fs::remove_file(&path); // Ignore errors
        }
    }
    Ok(())
}

/// Wrapper for test environments with automatic Git lock cleanup
///
/// This struct manages a temporary directory with automatic cleanup of
/// Git lock files when the struct is dropped. This prevents lock file
/// conflicts between test runs, especially when tests run in parallel.
///
/// # Gotchas
/// - TempDir cleanup happens on Drop - must keep GitTestEnv in scope
/// - Lock cleanup happens BEFORE temp directory deletion
/// - Use `path()` method to get the directory path for operations
///
/// # Example
/// ```rust
/// let env = GitTestEnv::new()?;
/// // ... perform Git operations ...
/// // Locks automatically cleaned up when env goes out of scope
/// ```
pub struct GitTestEnv {
    temp_dir: tempfile::TempDir,
    repo_path: std::path::PathBuf,
}

impl GitTestEnv {
    /// Create a new test environment with automatic lock cleanup
    ///
    /// # Returns
    /// * `Ok(GitTestEnv)` with a new temporary directory
    /// * `Err` if temporary directory creation fails
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::TempDir::new()?;
        let repo_path = temp_dir.path().to_path_buf();
        Ok(Self {
            temp_dir,
            repo_path,
        })
    }

    /// Get the path to the test directory
    ///
    /// # Returns
    /// * Reference to the repository path
    pub fn path(&self) -> &Path {
        &self.repo_path
    }
}

impl Drop for GitTestEnv {
    fn drop(&mut self) {
        // CRITICAL: Clean up Git locks BEFORE temp dir is deleted
        // This prevents lock file errors in subsequent test runs
        let _ = cleanup_git_locks(&self.repo_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_git_locks_nonexistent_repo() {
        // Should not error on non-existent repository
        let temp = tempfile::TempDir::new().unwrap();
        let result = cleanup_git_locks(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_cleanup_git_locks_no_locks() {
        // Should succeed when no locks exist
        let temp = tempfile::TempDir::new().unwrap();
        let _ = git2::Repository::init(temp.path());
        let result = cleanup_git_locks(temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_test_env_creates_directory() {
        let env = GitTestEnv::new().unwrap();
        assert!(env.path().exists());
        assert!(env.path().is_dir());
    }

    #[test]
    fn test_git_test_env_cleanup_on_drop() {
        let temp_path = {
            let env = GitTestEnv::new().unwrap();
            env.path().to_path_buf()
        };
        // Directory is cleaned up when env is dropped
        assert!(!temp_path.exists());
    }
}
