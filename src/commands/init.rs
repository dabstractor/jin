//! Init command implementation.
//!
//! This module implements the `jin init` command that initializes Jin in a project
//! directory by creating the necessary directory structure, configuration files,
//! and setting up the initial state.

use crate::cli::args::InitCommand;
use crate::core::config::ProjectContext;
use crate::core::error::Result;
use crate::staging::index::StagingIndex;
use std::path::{Path, PathBuf};

/// Execute the init command.
///
/// Initializes Jin in the current working directory by creating:
/// - `.jin/context` with default ProjectContext (version: 1, mode: null, scope: null)
/// - `.jin/staging/index.json` with empty StagingIndex
/// - `.jin/workspace/` directory for applied files
/// - `.gitignore` entry with Jin managed block (if in a Git repository)
///
/// The command is idempotent - running it multiple times is safe and will
/// display a friendly message if already initialized.
///
/// # Arguments
///
/// * `_cmd` - The init command (currently has no fields)
///
/// # Errors
///
/// Returns `JinError::Io` if directory creation fails.
/// Returns `JinError::ConfigError` if file serialization fails.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::InitCommand;
/// use jin_glm::commands::init;
///
/// let cmd = InitCommand;
/// init::execute(&cmd)?;
/// ```
pub fn execute(_cmd: &InitCommand) -> Result<()> {
    // Get current directory
    let project_dir = std::env::current_dir()?;

    // Check if already initialized
    let context_path = ProjectContext::context_path(&project_dir);
    if context_path.exists() {
        println!("Jin already initialized in this directory");
        println!("Run 'jin status' to see current state");
        return Ok(());
    }

    println!("Initializing Jin in this directory...");
    println!();

    // Create .jin/context with default values
    // Note: ProjectContext::save() automatically creates the .jin directory
    println!("Creating Jin configuration...");
    let context = ProjectContext::default();
    context.save(&project_dir)?;
    println!("  Created .jin/context");

    // Create .jin/staging/index.json
    println!("Creating staging area...");
    let staging_index = StagingIndex::new();
    staging_index.save_to_disk(&project_dir)?;
    println!("  Created .jin/staging/index.json");

    // Create .jin/workspace/ directory
    println!("Creating workspace directory...");
    let workspace_dir = project_dir.join(".jin/workspace");
    std::fs::create_dir_all(&workspace_dir)?;
    println!("  Created .jin/workspace/");

    // Update .gitignore if in a Git repository
    update_gitignore(&project_dir)?;

    println!();
    println!("Jin initialized successfully!");
    println!("Run 'jin help' to see available commands");

    Ok(())
}

/// Updates `.gitignore` with Jin managed block if in a Git repository.
///
/// Only updates if the managed block doesn't already exist.
/// Silently skips if not in a Git repository.
///
/// # Arguments
///
/// * `project_dir` - Path to the project root directory
///
/// # Returns
///
/// Returns `Ok(())` on success or if skipped, or `Err(JinError)` on failure.
fn update_gitignore(project_dir: &Path) -> Result<()> {
    // Try to discover Git repository
    let _repo = match git2::Repository::discover(project_dir) {
        Ok(r) => r,
        Err(_) => {
            // Not in a Git repository, skip .gitignore update
            return Ok(());
        }
    };

    let gitignore_path = project_dir.join(".gitignore");
    let existing_content = std::fs::read_to_string(&gitignore_path).unwrap_or_default();

    // Check if Jin managed block already exists
    if existing_content.contains("# BEGIN JIN MANAGED") {
        return Ok(());
    }

    // Append Jin managed block
    let jin_block = "\n# BEGIN JIN MANAGED\n.jin/\n# END JIN MANAGED\n";
    let mut content = existing_content;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(jin_block);

    std::fs::write(&gitignore_path, content)?;

    println!("  Updated .gitignore");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Save the current directory and restore it when dropped.
    struct DirGuard {
        original_dir: PathBuf,
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

    #[test]
    fn test_init_creates_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        // Change to temp directory
        std::env::set_current_dir(project_dir).unwrap();

        // Run init
        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify directories
        assert!(project_dir.join(".jin").exists());
        assert!(project_dir.join(".jin/staging").exists());
        assert!(project_dir.join(".jin/workspace").exists());
    }

    #[test]
    fn test_init_creates_context_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify context file exists and has correct content
        let context_path = project_dir.join(".jin/context");
        assert!(context_path.exists());

        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.version, 1);
        assert!(context.mode.is_none());
        assert!(context.scope.is_none());
    }

    #[test]
    fn test_init_creates_staging_index() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify staging index file exists
        let index_path = project_dir.join(".jin/staging/index.json");
        assert!(index_path.exists());

        // Verify it's an empty index
        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_init_creates_workspace_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify workspace directory exists and is empty
        let workspace_dir = project_dir.join(".jin/workspace");
        assert!(workspace_dir.exists());
        assert!(workspace_dir.is_dir());

        let entries = fs::read_dir(workspace_dir).unwrap();
        assert_eq!(entries.count(), 0);
    }

    #[test]
    fn test_init_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let cmd = InitCommand;

        // First init should succeed
        execute(&cmd).unwrap();

        // Second init should also succeed (not error)
        execute(&cmd).unwrap();

        // Context should still be valid
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.version, 1);
    }

    #[test]
    fn test_init_updates_gitignore_in_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a Git repository
        git2::Repository::init(project_dir).unwrap();

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify .gitignore was updated
        let gitignore_path = project_dir.join(".gitignore");
        assert!(gitignore_path.exists());

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("# BEGIN JIN MANAGED"));
        assert!(content.contains(".jin/"));
        assert!(content.contains("# END JIN MANAGED"));
    }

    #[test]
    fn test_init_does_not_duplicate_gitignore_block() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a Git repository
        git2::Repository::init(project_dir).unwrap();

        let cmd = InitCommand;

        // First init
        execute(&cmd).unwrap();

        // Second init
        execute(&cmd).unwrap();

        // Verify .gitignore doesn't have duplicate blocks
        let gitignore_path = project_dir.join(".gitignore");
        let content = fs::read_to_string(&gitignore_path).unwrap();

        let begin_count = content.matches("# BEGIN JIN MANAGED").count();
        let end_count = content.matches("# END JIN MANAGED").count();

        assert_eq!(begin_count, 1);
        assert_eq!(end_count, 1);
    }

    #[test]
    fn test_init_skips_gitignore_outside_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize a Git repository

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify .gitignore was NOT created
        let gitignore_path = project_dir.join(".gitignore");
        assert!(!gitignore_path.exists());
    }

    #[test]
    fn test_init_preserves_existing_gitignore_content() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a Git repository
        git2::Repository::init(project_dir).unwrap();

        // Create .gitignore with existing content
        let gitignore_path = project_dir.join(".gitignore");
        fs::write(&gitignore_path, "node_modules/\n*.log\n").unwrap();

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify existing content is preserved
        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains("*.log"));
        assert!(content.contains("# BEGIN JIN MANAGED"));
        assert!(content.contains(".jin/"));
    }
}
