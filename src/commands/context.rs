//! Context command implementation.
//!
//! This module implements the `jin context` command that displays
//! the current active context (mode, scope) from `.jin/context`.

use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use std::path::Path;

/// Execute the context command.
///
/// Displays the current active context including:
/// - Active mode (if set)
/// - Active scope (if set)
/// - Context file location
///
/// # Errors
///
/// Returns `JinError::Message` if Jin is not initialized.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::commands::context;
///
/// context::execute()?;
/// ```
pub fn execute() -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize."
                .to_string(),
        ));
    }

    // 3. Load and display context
    let context = ProjectContext::load(&workspace_root)?;
    display_context(&context, &context_path);

    Ok(())
}

/// Displays the active context information.
///
/// # Arguments
///
/// * `context` - The project context to display
/// * `context_path` - Path to the context file (for display)
fn display_context(context: &ProjectContext, context_path: &Path) {
    println!();

    // Display mode
    if let Some(mode) = &context.mode {
        println!("Active mode: {}", mode);
    } else {
        println!("No active mode");
    }

    // Display scope
    if let Some(scope) = &context.scope {
        println!("Active scope: {}", scope);
    } else {
        println!("No active scope");
    }

    // Display context file location
    println!();
    println!("Context file: {}", context_path.display());
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Save the current directory and restore it when dropped.
    struct DirGuard {
        original_dir: std::path::PathBuf,
    }

    impl DirGuard {
        fn new() -> std::io::Result<Self> {
            Ok(Self {
                original_dir: std::env::current_dir()?,
            })
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    /// Helper to initialize a Git repo
    fn init_git_repo(dir: &Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    /// Helper to initialize Jin in a directory
    fn init_jin(dir: &Path) {
        // Create .jin directory
        let jin_dir = dir.join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();

        // Create and save context
        let context = ProjectContext::default();
        context.save(dir).unwrap();

        // Verify context file was created
        let context_path = ProjectContext::context_path(dir);
        assert!(
            context_path.exists(),
            "Context file should exist after init_jin"
        );

        // Create staging index
        let staging_index = crate::staging::index::StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        // Create workspace directory
        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_execute_shows_mode_and_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set mode and scope
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        // Execute should succeed
        execute().unwrap();

        // Verify context is still set
        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("claude".to_string()));
        assert_eq!(loaded.scope, Some("language:rust".to_string()));
    }

    #[test]
    fn test_execute_no_context_set() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Execute with default context (no mode/scope)
        execute().unwrap();

        // Verify no context is set
        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.mode.is_none());
        assert!(context.scope.is_none());
    }

    #[test]
    fn test_execute_not_initialized_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize Jin

        let result = execute();
        assert!(result.is_err());
        if let Err(JinError::Message(msg)) = result {
            assert!(msg.contains("Jin is not initialized"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_execute_shows_only_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set only mode
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("cursor".to_string()));
        context.save(project_dir).unwrap();

        execute().unwrap();

        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("cursor".to_string()));
        assert!(loaded.scope.is_none());
    }

    #[test]
    fn test_execute_shows_only_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set only scope
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_scope(Some("language:javascript".to_string()));
        context.save(project_dir).unwrap();

        execute().unwrap();

        let loaded = ProjectContext::load(project_dir).unwrap();
        assert!(loaded.mode.is_none());
        assert_eq!(loaded.scope, Some("language:javascript".to_string()));
    }
}
