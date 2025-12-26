//! Status command implementation.
//!
//! This module implements the `jin status` command that displays
//! workspace and staging state including active context, staged files,
//! and layer commit information.

use crate::cli::args::StatusCommand;
use crate::core::config::ProjectContext;
use crate::core::error::Result;
use crate::git::JinRepo;
use crate::staging::index::StagingIndex;
use std::collections::HashMap;
use std::path::Path;

/// Execute the status command.
///
/// Displays comprehensive workspace status including:
/// - Active mode/scope from `.jin/context`
/// - Staged files grouped by layer
/// - Layer refs from Git repository
/// - Workspace cleanliness state
///
/// # Arguments
///
/// * `_cmd` - The status command (currently has no fields)
///
/// # Errors
///
/// Returns `JinError::Message` if not in a Jin-initialized directory.
/// Returns `JinError::Io` if context/staging files cannot be read.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::StatusCommand;
/// use jin_glm::commands::status;
///
/// let cmd = StatusCommand;
/// status::execute(&cmd)?;
/// ```
pub fn execute(_cmd: &StatusCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(crate::core::error::JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize.".to_string(),
        ));
    }

    // 3. Load and display active context
    let context = ProjectContext::load(&workspace_root)?;
    display_active_context(&context);

    // 4. Load and display staged files
    let staging = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());
    display_staged_files(&staging);

    // 5. Load and display layer refs (if Git exists)
    if let Ok(_git_repo) = git2::Repository::discover(&workspace_root) {
        if let Ok(repo) = JinRepo::open_or_create(&workspace_root) {
            display_layer_refs(&repo)?;
        }
    }

    // 6. Display workspace status
    display_workspace_status(&staging);

    Ok(())
}

/// Displays active mode/scope context.
fn display_active_context(context: &ProjectContext) {
    println!();
    if let Some(mode) = &context.mode {
        println!("Active mode: {}", mode);
    }
    if let Some(scope) = &context.scope {
        println!("Active scope: {}", scope);
    }
    if context.mode.is_none() && context.scope.is_none() {
        println!("No active mode or scope");
    }
}

/// Displays staged files grouped by layer.
fn display_staged_files(staging: &StagingIndex) {
    println!();

    if staging.is_empty() {
        println!("No files staged");
        return;
    }

    // Group by layer
    let mut layers: HashMap<String, Vec<String>> = HashMap::new();
    for entry in staging.all_entries() {
        let layer_name = format!("{}", entry.layer);
        let path = entry.path.display().to_string();
        layers.entry(layer_name).or_default().push(path);
    }

    println!("Staged files ({}):", staging.len());
    for (layer, mut files) in layers {
        println!("  {}:", layer);
        files.sort();
        for file in files {
            println!("    {}", file);
        }
    }
}

/// Displays layer refs from Git repository.
fn display_layer_refs(repo: &JinRepo) -> Result<()> {
    println!();

    let refs = repo.list_layer_refs()?;

    if refs.is_empty() {
        println!("No layers with commits");
        return Ok(());
    }

    println!("Layers with commits:");
    for (layer, oid) in refs {
        println!("  {}: {}", layer, oid);
    }

    Ok(())
}

/// Displays workspace cleanliness status.
fn display_workspace_status(staging: &StagingIndex) {
    println!();
    if staging.is_empty() {
        println!("Workspace: clean");
    } else {
        println!("Workspace: dirty");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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

    /// Helper to create a test file with content
    fn create_test_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
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
        assert!(context_path.exists(), "Context file should exist after init_jin");

        // Create staging index
        let staging_index = StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        // Create workspace directory
        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_status_shows_active_context() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize Jin
        init_jin(project_dir);

        // Set mode and scope in context
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:javascript".to_string()));
        context.save(project_dir).unwrap();

        // Run status
        let cmd = StatusCommand;
        execute(&cmd).unwrap();

        // Context should show mode and scope
        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("claude".to_string()));
        assert_eq!(loaded.scope, Some("language:javascript".to_string()));
    }

    #[test]
    fn test_status_shows_staged_files_by_layer() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and stage test files
        let _config1 = create_test_file(project_dir, "config1.toml", "config 1");
        let _config2 = create_test_file(project_dir, "config2.json", "config 2");

        let mut index = StagingIndex::load_from_disk(project_dir).unwrap();

        // Stage files to different layers
        let entry1 = crate::staging::entry::StagedEntry::new(
            project_dir.join("config1.toml"),
            crate::core::Layer::ProjectBase {
                project: "test".to_string(),
            },
            b"config 1",
        )
        .unwrap();
        let entry1 = crate::staging::entry::StagedEntry {
            path: std::path::PathBuf::from("config1.toml"),
            ..entry1
        };

        let entry2 = crate::staging::entry::StagedEntry::new(
            project_dir.join("config2.json"),
            crate::core::Layer::GlobalBase,
            b"config 2",
        )
        .unwrap();
        let entry2 = crate::staging::entry::StagedEntry {
            path: std::path::PathBuf::from("config2.json"),
            ..entry2
        };

        index.add_entry(entry1).unwrap();
        index.add_entry(entry2).unwrap();
        index.save_to_disk(project_dir).unwrap();

        // Verify staging
        let loaded = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_status_shows_layer_refs() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Run status - should not error with no layer refs
        let cmd = StatusCommand;
        execute(&cmd).unwrap();
    }

    #[test]
    fn test_status_no_jin_initialized_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize Jin

        let cmd = StatusCommand;
        let result = execute(&cmd);

        assert!(result.is_err());
        if let Err(crate::core::error::JinError::Message(msg)) = result {
            assert!(msg.contains("Jin is not initialized"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_status_empty_staging() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = StatusCommand;
        execute(&cmd).unwrap();

        // Verify staging is empty
        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert!(index.is_empty());
    }
}
