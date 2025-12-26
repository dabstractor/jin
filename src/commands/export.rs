//! Export command implementation.
//!
//! This module implements the `jin export` command that exports files from
//! Jin's workspace directory back to the Git working directory. Export is the
//! inverse of `jin apply` - it reads files from `.jin/workspace/` and writes
//! them to their original locations in the Git working tree.
//!
//! # Export Workflow
//!
//! 1. Validate Jin is initialized
//! 2. Verify files exist in `.jin/workspace/` directory
//! 3. Read file content from workspace
//! 4. Write files to Git working directory
//! 5. Optionally stage exported files in Git
//! 6. Provide summary of exported files
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::cli::args::ExportCommand;
//! use jin_glm::commands::export;
//! use std::path::PathBuf;
//!
//! let cmd = ExportCommand {
//!     files: vec![PathBuf::from("config.toml")],
//! };
//!
//! export::execute(&cmd)?;
//! ```

use crate::cli::args::ExportCommand;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use std::path::{Path, PathBuf};

/// Execute the export command.
///
/// Exports files from Jin's workspace directory back to the Git working directory,
/// then stages them in Git for commit. This is the inverse of the `jin apply`
/// command, which writes merged files to the workspace.
///
/// # Arguments
///
/// * `cmd` - The export command containing files to export
///
/// # Errors
///
/// Returns `JinError::Message` if Jin is not initialized.
/// Returns `JinError::Message` if a file is not found in the workspace.
/// Returns `JinError::RepoNotFound` if not in a Git repository.
/// Propagates I/O errors for file operations.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::ExportCommand;
/// use jin_glm::commands::export;
/// use std::path::PathBuf;
///
/// let cmd = ExportCommand {
///     files: vec![PathBuf::from("config.toml"), PathBuf::from(".env")],
/// };
///
/// export::execute(&cmd)?;
/// ```
pub fn execute(cmd: &ExportCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Verify Jin is initialized
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize."
                .to_string(),
        ));
    }

    // 3. Open Git repository for staging
    let git_repo = git2::Repository::discover(&workspace_root)
        .map_err(|_| JinError::Message("Not a Git repository".to_string()))?;

    // 4. Get workspace directory
    let workspace_dir = workspace_root.join(".jin/workspace");

    // 5. Track exported files
    let mut exported_files = Vec::new();

    // 6. Process each file
    for file_path in &cmd.files {
        let exported = export_file(&workspace_root, &workspace_dir, file_path)?;
        exported_files.push(exported);
    }

    // 7. Stage all exported files in Git
    if !exported_files.is_empty() {
        stage_files_in_git(&git_repo, &exported_files)?;
    }

    // 8. Print summary
    println!(
        "\nExported {} file(s) to Git working directory",
        exported_files.len()
    );
    println!("Staged in Git:");
    for file in &exported_files {
        println!("  {}", file.display());
    }

    Ok(())
}

/// Exports a single file from workspace to Git working directory.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
/// * `workspace_dir` - Path to `.jin/workspace/` directory
/// * `file_path` - Path to the file to export (can be relative or absolute)
///
/// # Returns
///
/// The relative path of the exported file (for Git staging).
///
/// # Errors
///
/// Returns `JinError::Message` if file is outside workspace root.
/// Returns `JinError::Message` if file is not found in Jin workspace.
/// Propagates I/O errors for file operations.
fn export_file(workspace_root: &Path, workspace_dir: &Path, file_path: &Path) -> Result<PathBuf> {
    // Resolve to absolute path if relative
    let resolved_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        workspace_root.join(file_path)
    };

    // Get relative path for Git operations
    let relative_path = resolved_path.strip_prefix(workspace_root).map_err(|_| {
        JinError::Message(format!(
            "File is outside workspace root: {}",
            resolved_path.display()
        ))
    })?;

    // CRITICAL: Check file exists in workspace
    let workspace_file_path = workspace_dir.join(relative_path);
    if !workspace_file_path.exists() {
        return Err(JinError::Message(format!(
            "File not found in Jin workspace: {}\n\
             Use 'jin status' to see managed files.",
            file_path.display()
        )));
    }

    // Read content from workspace
    let content = std::fs::read(&workspace_file_path)?;

    // Build target path in Git working directory
    let target_path = workspace_root.join(relative_path);

    // Check for conflict (file exists and differs)
    if target_path.exists() {
        let existing_content = std::fs::read(&target_path)?;
        if existing_content != content {
            eprintln!(
                "Warning: {} differs between workspace and Git working directory",
                target_path.display()
            );
        }
    }

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to Git working directory
    std::fs::write(&target_path, content)?;

    println!("Exported {} to Git working directory", file_path.display());

    Ok(relative_path.to_path_buf())
}

/// Stages exported files in Git using the git index.
///
/// # Arguments
///
/// * `git_repo` - The Git repository
/// * `files` - List of relative file paths to stage
///
/// # Errors
///
/// Returns `JinError::Message` if Git index operations fail.
///
/// # Gotchas
///
/// - Git index expects paths relative to repo root
/// - Must call index.write() to persist changes
fn stage_files_in_git(git_repo: &git2::Repository, files: &[PathBuf]) -> Result<()> {
    // Get the Git index
    let mut index = git_repo
        .index()
        .map_err(|e| JinError::Message(format!("Failed to get Git index: {}", e)))?;

    // Add each file to the index
    for file_path in files {
        // PATTERN: Git expects relative paths
        if let Err(e) = index.add_path(file_path) {
            eprintln!("Warning: Failed to stage {}: {}", file_path.display(), e);
        }
    }

    // Write the index to persist changes
    index
        .write()
        .map_err(|e| JinError::Message(format!("Failed to write Git index: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
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

    /// Helper to initialize a Git repo
    fn init_git_repo(dir: &Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    /// Helper to initialize Jin in a directory
    fn init_jin(dir: &Path) {
        let staging_index = crate::staging::index::StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();

        // Create context file (required by export command)
        let context = crate::core::config::ProjectContext::default();
        context.save(dir).unwrap();
    }

    /// Helper to create a test file in the workspace
    fn create_workspace_file(dir: &Path, relative_path: &str, content: &str) {
        let file_path = dir.join(".jin/workspace").join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
    }

    #[test]
    fn test_export_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file in workspace
        create_workspace_file(project_dir, "config.toml", "[settings]\nenabled = true");

        let cmd = ExportCommand {
            files: vec![PathBuf::from("config.toml")],
        };

        execute(&cmd).unwrap();

        // Verify file exists in Git working directory
        let exported_file = project_dir.join("config.toml");
        assert!(exported_file.exists());
        let content = fs::read_to_string(&exported_file).unwrap();
        assert_eq!(content, "[settings]\nenabled = true");
    }

    #[test]
    fn test_export_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create multiple files in workspace
        create_workspace_file(project_dir, "config.toml", "[settings]\nenabled = true");
        create_workspace_file(project_dir, ".env", "DATABASE_URL=postgres://localhost");
        create_workspace_file(project_dir, "settings.json", "{\"api_key\": \"test\"}");

        let cmd = ExportCommand {
            files: vec![
                PathBuf::from("config.toml"),
                PathBuf::from(".env"),
                PathBuf::from("settings.json"),
            ],
        };

        execute(&cmd).unwrap();

        // Verify all files exist in Git working directory
        assert!(project_dir.join("config.toml").exists());
        assert!(project_dir.join(".env").exists());
        assert!(project_dir.join("settings.json").exists());
    }

    #[test]
    fn test_export_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file in a nested directory
        create_workspace_file(
            project_dir,
            "config/development/database.json",
            "{\"host\": \"localhost\"}",
        );

        let cmd = ExportCommand {
            files: vec![PathBuf::from("config/development/database.json")],
        };

        execute(&cmd).unwrap();

        // Verify file and directories exist
        let exported_file = project_dir.join("config/development/database.json");
        assert!(exported_file.exists());
        assert!(project_dir.join("config").exists());
        assert!(project_dir.join("config/development").exists());
    }

    #[test]
    fn test_export_file_not_in_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Don't create the file in workspace - should fail
        let cmd = ExportCommand {
            files: vec![PathBuf::from("config.toml")],
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::Message(_))));
    }

    #[test]
    fn test_export_stages_in_git() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file in workspace
        create_workspace_file(project_dir, "config.toml", "test = true");

        let cmd = ExportCommand {
            files: vec![PathBuf::from("config.toml")],
        };

        execute(&cmd).unwrap();

        // Verify file is staged in Git
        let index = repo.index().unwrap();
        let staged_files: Vec<_> = index.iter().map(|e| e.path).collect();
        let expected = b"config.toml".to_vec();
        assert!(staged_files.contains(&&expected));
    }

    #[test]
    fn test_export_with_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file in Git working directory with different content
        let working_dir_file = project_dir.join("config.toml");
        fs::write(&working_dir_file, "old = value").unwrap();

        // Create a file in workspace with different content
        create_workspace_file(project_dir, "config.toml", "new = value");

        let cmd = ExportCommand {
            files: vec![PathBuf::from("config.toml")],
        };

        // Export should succeed (warns but doesn't abort)
        execute(&cmd).unwrap();

        // File should have workspace content (export overwrites)
        let content = fs::read_to_string(&working_dir_file).unwrap();
        assert_eq!(content, "new = value");
    }

    #[test]
    fn test_export_nested_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a deeply nested file
        create_workspace_file(project_dir, "a/b/c/d/e/config.toml", "nested = true");

        let cmd = ExportCommand {
            files: vec![PathBuf::from("a/b/c/d/e/config.toml")],
        };

        execute(&cmd).unwrap();

        // Verify file exists
        let exported_file = project_dir.join("a/b/c/d/e/config.toml");
        assert!(exported_file.exists());
        let content = fs::read_to_string(&exported_file).unwrap();
        assert_eq!(content, "nested = true");
    }

    #[test]
    fn test_export_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file in workspace
        create_workspace_file(project_dir, "config.toml", "test = true");

        let cmd = ExportCommand {
            files: vec![PathBuf::from("config.toml")],
        };

        // First export
        execute(&cmd).unwrap();

        // Get file content
        let content_after_first = fs::read_to_string(project_dir.join("config.toml")).unwrap();

        // Second export (should be idempotent)
        execute(&cmd).unwrap();

        let content_after_second = fs::read_to_string(project_dir.join("config.toml")).unwrap();

        assert_eq!(content_after_first, content_after_second);
    }

    #[test]
    fn test_export_with_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file in workspace
        create_workspace_file(project_dir, "config.toml", "absolute = true");

        // Use absolute path
        let absolute_path = project_dir.join("config.toml");

        let cmd = ExportCommand {
            files: vec![absolute_path],
        };

        execute(&cmd).unwrap();

        // Verify file exists in Git working directory
        let exported_file = project_dir.join("config.toml");
        assert!(exported_file.exists());
        let content = fs::read_to_string(&exported_file).unwrap();
        assert_eq!(content, "absolute = true");
    }

    #[test]
    fn test_export_jin_not_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        // Don't initialize Jin

        let cmd = ExportCommand {
            files: vec![PathBuf::from("config.toml")],
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::Message(_))));
    }

    #[test]
    fn test_export_file_outside_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Use a path outside the workspace
        let outside_path = PathBuf::from("/etc/passwd");

        let cmd = ExportCommand {
            files: vec![outside_path],
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        // The error depends on the system - /etc/passwd might not be in workspace
        assert!(matches!(result, Err(JinError::Message(_))));
    }

    #[test]
    fn test_export_empty_files_list() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = ExportCommand { files: vec![] };

        // Should succeed with no files exported
        execute(&cmd).unwrap();
    }
}
