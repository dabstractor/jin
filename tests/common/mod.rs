//! Common test utilities for Jin integration tests
//!
//! This module provides shared fixtures and assertions used across
//! integration test files.

pub mod assertions;
pub mod fixtures;
pub mod git_helpers;

use crate::common::git_helpers::cleanup_before_test;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test context for unit tests
///
/// Provides all paths and context needed for unit tests.
/// CRITICAL: Keep _temp_dir in scope to prevent premature cleanup.
pub struct UnitTestContext {
    /// Temporary directory (must be kept in scope)
    _temp_dir: TempDir,
    /// Absolute path to test project directory
    pub project_path: PathBuf,
    /// Absolute path to isolated JIN_DIR
    pub jin_dir: PathBuf,
}

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
pub fn setup_unit_test() -> UnitTestContext {
    use jin::core::config::ProjectContext;
    use jin::git::repo::JinRepo;

    // CRITICAL: Clean up locks BEFORE creating new test environment
    cleanup_before_test(None);

    // Create temporary directory for isolated test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    let jin_dir = temp_dir.path().join(".jin_global");

    // CRITICAL: Set JIN_DIR before any Jin operations
    std::env::set_var("JIN_DIR", &jin_dir);

    // CRITICAL: Set current directory for tests that expect it
    // (Tests must use #[serial] attribute to prevent conflicts)
    std::env::set_current_dir(&project_path).expect("Failed to set current directory");

    // Initialize Jin repository
    let _ = JinRepo::open_or_create();

    // Create .jin directory structure
    std::fs::create_dir_all(".jin").expect("Failed to create .jin directory");

    // Create and save default context
    let context = ProjectContext::default();
    context.save().expect("Failed to save context");

    // Create empty staging index (many tests expect this to exist)
    let staging_dir = project_path.join(".jin/staging");
    std::fs::create_dir_all(&staging_dir).expect("Failed to create staging directory");
    std::fs::write(staging_dir.join("index.json"), "{}").expect("Failed to create staging index");

    UnitTestContext {
        _temp_dir: temp_dir,
        project_path,
        jin_dir,
    }
}
