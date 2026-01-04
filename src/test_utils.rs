//! Test utilities for Jin unit tests
//!
//! This module provides shared test setup for unit tests across the codebase.

use std::path::{Path, PathBuf};

#[cfg(test)]
pub use tempfile::TempDir;

/// Clean up Git locks BEFORE running a test
///
/// This function should be called at the START of test setup to ensure
/// no stale locks from previous test runs cause failures.
#[cfg(test)]
fn cleanup_git_locks(repo_path: &Path) {
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        return;
    }

    // Clean common lock files
    let _ = std::fs::remove_file(git_dir.join("index.lock"));
    let _ = std::fs::remove_file(git_dir.join("HEAD.lock"));
    let _ = std::fs::remove_file(git_dir.join("config.lock"));
    let _ = std::fs::remove_file(git_dir.join("packed-refs.lock"));
}

/// Clean up Git locks before running a test
#[cfg(test)]
fn cleanup_before_test(jin_dir: &Path) {
    // Clean up JIN_DIR locks
    let _ = cleanup_git_locks(jin_dir);

    // Clean up current directory's .git locks
    if let Ok(current_dir) = std::env::current_dir() {
        let _ = cleanup_git_locks(&current_dir);
    }
}

/// Test context for unit tests
///
/// Provides all paths and context needed for unit tests.
/// CRITICAL: Keep _temp_dir in scope to prevent premature cleanup.
///
/// This context automatically restores the original directory and environment
/// when dropped, ensuring tests don't interfere with each other.
#[cfg(test)]
pub struct UnitTestContext {
    /// Temporary directory (must be kept in scope)
    _temp_dir: TempDir,
    /// Original directory (for restoration on drop, if valid)
    _original_dir: Option<PathBuf>,
    /// Original JIN_DIR value (for restoration on drop)
    _original_jin_dir: Option<String>,
    /// Absolute path to test project directory
    pub project_path: PathBuf,
    /// Absolute path to isolated JIN_DIR
    pub jin_dir: PathBuf,
}

#[cfg(test)]
impl Drop for UnitTestContext {
    fn drop(&mut self) {
        // Restore original directory only if it was valid and exists
        if let Some(ref dir) = self._original_dir {
            if dir.exists() {
                let _ = std::env::set_current_dir(dir);
            }
        }

        // Restore original JIN_DIR
        match &self._original_jin_dir {
            Some(val) => std::env::set_var("JIN_DIR", val),
            None => std::env::remove_var("JIN_DIR"),
        }
    }
}

#[cfg(test)]
impl UnitTestContext {
    /// Get the absolute path to .jin directory
    pub fn jin_path(&self) -> PathBuf {
        self.project_path.join(".jin")
    }

    /// Get the absolute path to .jin/context file
    pub fn context_path(&self) -> PathBuf {
        self.jin_path().join("context")
    }

    /// Get the absolute path to .jin/staging/index.json
    pub fn staging_index_path(&self) -> PathBuf {
        self.jin_path().join("staging/index.json")
    }
}

/// Unified test setup for unit tests
///
/// This function replaces the duplicated setup_test_env() functions
/// across all command test files.
///
/// # Returns
/// * UnitTestContext with all paths and temporary directory
///
/// # Gotchas
/// - Call cleanup_before_test() FIRST to remove stale locks
/// - Keep the returned UnitTestContext in scope (use let _ctx = ...)
/// - All paths are absolute, avoiding current_dir() issues
/// - Creates .jin directory structure for tests that need it
/// - Automatically restores original directory/JIN_DIR on drop
///
/// # Example
/// ```rust
/// #[test]
/// #[serial]  // Required because we set JIN_DIR
/// fn test_something() {
///     let ctx = setup_unit_test();
///     // Use ctx.project_path, ctx.jin_dir for absolute paths
///     let context_path = ctx.context_path();
///     std::fs::write(&context_path, "mode: test").unwrap();
/// }
/// ```
#[cfg(test)]
pub fn setup_unit_test() -> UnitTestContext {
    use crate::core::config::ProjectContext;
    use crate::git::repo::JinRepo;

    // Save original environment state
    // NOTE: current_dir() may fail if previous test cleaned up a temp directory
    // In that case, we won't try to restore (None means no valid restore point)
    let original_dir = std::env::current_dir().ok();
    let original_jin_dir = std::env::var("JIN_DIR").ok();

    // Create temporary directory for isolated test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    let jin_dir = temp_dir.path().join(".jin_global");

    // CRITICAL: Set JIN_DIR before any Jin operations
    std::env::set_var("JIN_DIR", &jin_dir);

    // CRITICAL: Create jin_dir directory explicitly using absolute path
    // This ensures the directory exists before JinRepo tries to use it
    std::fs::create_dir_all(&jin_dir).expect("Failed to create JIN_DIR");

    // CRITICAL: Clean up locks from previous test runs
    cleanup_before_test(&jin_dir);

    // Initialize Jin repository (before setting current directory)
    // CRITICAL: Don't ignore the result - the repo MUST be initialized
    JinRepo::open_or_create().expect("Failed to initialize Jin repository");

    // CRITICAL: Create .jin directory structure using absolute path
    // Do this BEFORE setting current directory to avoid relative path issues
    let jin_path = project_path.join(".jin");
    std::fs::create_dir_all(&jin_path).expect("Failed to create .jin directory");

    // Create and save default context manually using absolute path
    // This avoids issues with ProjectContext::save() using relative paths
    let context = ProjectContext::default();
    let context_path = jin_path.join("context");
    let content = serde_yaml::to_string(&context).expect("Failed to serialize context");
    std::fs::write(&context_path, content).expect("Failed to save context");

    // CRITICAL: Set current directory for tests that expect it
    // (Tests must use #[serial] attribute to prevent conflicts)
    // We do this AFTER creating all directories using absolute paths
    // Use unwrap_or to handle case where current dir was deleted by previous test
    if std::env::set_current_dir(&project_path).is_err() {
        panic!("Failed to set current directory to {:?}", project_path);
    }

    // Create empty staging index at the JIN_DIR location (where StagingIndex looks)
    // Note: JIN_DIR is jin_dir, NOT project_path/.jin
    let staging_dir = jin_dir.join("staging");
    std::fs::create_dir_all(&staging_dir).expect("Failed to create staging directory");
    std::fs::write(staging_dir.join("index.json"), "{}").expect("Failed to create staging index");

    UnitTestContext {
        _temp_dir: temp_dir,
        _original_dir: original_dir,
        _original_jin_dir: original_jin_dir,
        project_path,
        jin_dir,
    }
}
