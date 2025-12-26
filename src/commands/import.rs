//! Import command implementation.
//!
//! This module implements the `jin import` command that imports Git-tracked files
//! from the workspace into Jin's layer management system. Import is the inverse
//! of `jin add` - it requires files to be Git-tracked, while add requires them
//! to be untracked.

use crate::cli::args::ImportCommand;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::staging::entry::StagedEntry;
use crate::staging::index::StagingIndex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Execute the import command.
///
/// Imports Git-tracked files into Jin's staging system, routing them to the
/// appropriate layer based on CLI flags or active context. Each file is
/// validated for git-tracked status, type, and existence before being imported.
///
/// # Arguments
///
/// * `cmd` - The import command containing files to import and routing flags
///
/// # Errors
///
/// Returns `JinError::FileNotFound` if a file doesn't exist.
/// Returns `JinError::FileNotTracked` if a file is not git-tracked.
/// Returns `JinError::SymlinkNotSupported` if a file is a symlink.
/// Returns `JinError::BinaryFileNotSupported` if a file is binary.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::ImportCommand;
/// use jin_glm::commands::import;
/// use std::path::PathBuf;
///
/// let cmd = ImportCommand {
///     files: vec![PathBuf::from("config.toml")],
///     mode: false,
///     scope: None,
///     project: false,
///     global: false,
/// };
///
/// import::execute(&cmd)?;
/// ```
pub fn execute(cmd: &ImportCommand) -> Result<()> {
    // Get workspace root
    let workspace_root = std::env::current_dir()?;

    // Load context for active mode/scope
    let context = ProjectContext::load(&workspace_root)?;

    // Detect project name for LayerRouter
    let project_name = detect_project_name(&workspace_root)?;

    // Determine target layer
    let layer = determine_layer(cmd, &context, &project_name)?;

    // Load staging index (create new if doesn't exist)
    let mut staging_index =
        StagingIndex::load_from_disk(&workspace_root).unwrap_or_else(|_| StagingIndex::new());

    // Track imported files by layer for summary
    let mut staged_by_layer: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Process each file
    for file_path in &cmd.files {
        let resolved_path = resolve_path(&workspace_root, file_path)?;

        // Get relative path for storage
        let relative_path = resolved_path.strip_prefix(&workspace_root).map_err(|_| {
            JinError::Message(format!(
                "File is outside workspace root: {}",
                resolved_path.display()
            ))
        })?;

        // CRITICAL: Import validates that file IS git-tracked (opposite of add)
        validate_importable_file(&resolved_path, &workspace_root)?;

        // Check not symlink (same as add)
        if resolved_path.is_symlink() {
            return Err(JinError::SymlinkNotSupported {
                path: resolved_path.display().to_string(),
            });
        }

        // Check binary (same as add)
        let content = std::fs::read(&resolved_path)?;
        if content.contains(&0x00) {
            return Err(JinError::BinaryFileNotSupported {
                path: resolved_path.display().to_string(),
            });
        }

        let text_content = std::fs::read_to_string(&resolved_path)?;
        // Pass absolute path to StagedEntry::new() for metadata reading
        let entry = StagedEntry::new(
            resolved_path.clone(),
            layer.clone(),
            text_content.as_bytes(),
        )?;

        staging_index.add_entry(entry)?;

        let layer_key = format!("{}", layer);
        staged_by_layer
            .entry(layer_key)
            .or_default()
            .push(relative_path.to_path_buf());

        println!("Imported {} to {}", file_path.display(), layer);
    }

    // Persist staging index
    staging_index.save_to_disk(&workspace_root)?;

    // Print summary
    if !staged_by_layer.is_empty() {
        println!();
        println!("Summary:");
        for (layer_name, files) in staged_by_layer {
            println!("  {}:", layer_name);
            for file in files {
                println!("    - {}", file.display());
            }
        }
    }

    Ok(())
}

/// Determines the target layer based on CLI flags and active context.
///
/// When flags are specified, routes explicitly using those flags.
/// When no flags are specified, falls back to context (mode/scope) and
/// ultimately to project base layer.
///
/// # Arguments
///
/// * `cmd` - The import command with routing flags
/// * `context` - Active project context with mode/scope
/// * `project` - Project name for routing
///
/// # Returns
///
/// The target `Layer` for importing files.
fn determine_layer(cmd: &ImportCommand, context: &ProjectContext, project: &str) -> Result<Layer> {
    // If any flags specified, use explicit routing
    if cmd.mode || cmd.scope.is_some() || cmd.project || cmd.global {
        let mode = if cmd.mode {
            context.mode.as_deref()
        } else {
            None
        };
        let scope = cmd.scope.as_deref().or(context.scope.as_deref());
        let proj = if cmd.project { Some(project) } else { None };

        return Layer::from_flags(mode, scope, proj, cmd.global).ok_or_else(|| {
            JinError::Message(
                "No routing target specified. Use --mode, --scope, --project, or --global"
                    .to_string(),
            )
        });
    }

    // No flags - use context defaults, or project as final fallback
    if let Some(mode) = &context.mode {
        if let Some(scope) = &context.scope {
            return Ok(Layer::ModeScope {
                mode: mode.clone(),
                scope: scope.clone(),
            });
        }
        return Ok(Layer::ModeBase { mode: mode.clone() });
    }

    if let Some(scope) = &context.scope {
        return Ok(Layer::ScopeBase {
            scope: scope.clone(),
        });
    }

    // Final fallback: project base layer
    Ok(Layer::ProjectBase {
        project: project.to_string(),
    })
}

/// Validates that a file is importable.
///
/// Checks:
/// - File exists
/// - File is git-tracked (CRITICAL: opposite of add command)
/// - File is not a symlink
/// - File is text (not binary)
///
/// # Arguments
///
/// * `path` - Absolute path to the file
/// * `workspace_root` - Path to workspace root
///
/// # Returns
///
/// `Ok(())` if file passes all validations, `Err` otherwise.
fn validate_importable_file(path: &Path, workspace_root: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    let relative_path = path.strip_prefix(workspace_root).map_err(|_| {
        JinError::Message(format!(
            "File is outside workspace root: {}",
            path.display()
        ))
    })?;

    // CRITICAL: Import requires file to be git-tracked (opposite of add)
    let git_repo = git2::Repository::open(workspace_root)
        .map_err(|_| JinError::Message("Not a Git repository".to_string()))?;

    let status = match git_repo.status_file(relative_path) {
        Ok(s) => s,
        Err(_) => {
            return Err(JinError::FileNotTracked {
                path: relative_path.display().to_string(),
            })
        }
    };

    // File must be tracked (WT_NEW means untracked worktree file)
    // If status contains WT_NEW, file is not tracked
    if status.contains(git2::Status::WT_NEW) {
        return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string(),
        });
    }

    // Empty status means file is tracked and unchanged - this is OK
    // Non-empty status (WT_MODIFIED, INDEX_MODIFIED, etc.) means tracked - also OK
    Ok(())
}

/// Detects the project name from Git remote or directory name.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// The detected project name.
fn detect_project_name(workspace_root: &Path) -> Result<String> {
    use git2::Repository;

    let repo = Repository::discover(workspace_root).map_err(|_| JinError::RepoNotFound {
        path: workspace_root.display().to_string(),
    })?;

    // Try to get from git remote origin
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.rsplit('/').next() {
                let name = name.trim_end_matches(".git");
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }

    // Fallback to directory name
    workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| JinError::Message("Cannot determine project name".to_string()))
}

/// Resolves a file path to absolute.
///
/// # Arguments
///
/// * `workspace_root` - Path to workspace root
/// * `file_path` - File path to resolve (can be relative or absolute)
///
/// # Returns
///
/// Absolute path to the file.
fn resolve_path(workspace_root: &Path, file_path: &Path) -> Result<PathBuf> {
    if file_path.is_absolute() {
        Ok(file_path.to_path_buf())
    } else {
        Ok(workspace_root.join(file_path))
    }
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

    /// Helper to create a test file with content
    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
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
        // Create .jin directory structure
        let jin_dir = dir.join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();

        // Create a .gitkeep in .jin to ensure it's tracked
        std::fs::write(jin_dir.join(".gitkeep"), "").unwrap();

        let staging_index = StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    /// Helper to commit a file to Git (making it tracked)
    fn commit_file_to_git(repo: &git2::Repository, path: &Path) {
        let mut index = repo.index().unwrap();
        index.add_path(path).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = repo.signature().unwrap();

        // Check if HEAD exists
        if repo.head().is_ok() {
            // Update existing commit
            let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "Add file", &tree, &[&head_commit])
                .unwrap();
        } else {
            // Initial commit
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }
    }

    #[test]
    fn test_import_to_project_base() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        // Initialize Git and Jin
        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and commit a test file to Git
        let _config_file =
            create_test_file(project_dir, "config.toml", "[settings]\nenabled = true");

        // Commit to Git so file is tracked
        commit_file_to_git(&repo, Path::new("config.toml"));

        // Execute import command (no flags = project base)
        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        // Verify file was staged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
    }

    #[test]
    fn test_import_to_mode_base() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);

        // Set up context with mode
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.save(project_dir).unwrap();

        init_jin(project_dir);

        // Create and commit a test file to Git
        let _config_file = create_test_file(project_dir, "config.toml", "mode = claude");
        commit_file_to_git(&repo, Path::new("config.toml"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: true,
            scope: None,
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
    }

    #[test]
    fn test_import_to_scope_base() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "scope = rust");
        commit_file_to_git(&repo, Path::new("config.toml"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: Some("language:rust".to_string()),
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
    }

    #[test]
    fn test_import_to_global() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "global setting");
        commit_file_to_git(&repo, Path::new("config.toml"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: true,
        };

        execute(&cmd).unwrap();

        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
    }

    #[test]
    fn test_import_to_mode_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);

        // Set up context with mode
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.save(project_dir).unwrap();

        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "mode + scope");
        commit_file_to_git(&repo, Path::new("config.toml"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: true,
            scope: Some("language:rust".to_string()),
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
    }

    #[test]
    fn test_import_file_not_tracked() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a file but DO NOT add to Git index
        let _untracked_file = create_test_file(project_dir, "untracked.txt", "not in git");

        let cmd = ImportCommand {
            files: vec![PathBuf::from("untracked.txt")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::FileNotTracked { .. })));
    }

    #[test]
    fn test_import_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = ImportCommand {
            files: vec![PathBuf::from("nonexistent.txt")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::FileNotFound { .. })));
    }

    #[test]
    fn test_import_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        create_test_file(project_dir, "config1.toml", "config 1");
        create_test_file(project_dir, "config2.toml", "config 2");
        create_test_file(project_dir, "config3.toml", "config 3");

        // Commit all files to Git
        commit_file_to_git(&repo, Path::new("config1.toml"));
        commit_file_to_git(&repo, Path::new("config2.toml"));
        commit_file_to_git(&repo, Path::new("config3.toml"));

        let cmd = ImportCommand {
            files: vec![
                PathBuf::from("config1.toml"),
                PathBuf::from("config2.toml"),
                PathBuf::from("config3.toml"),
            ],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 3);
    }

    #[test]
    fn test_import_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);

        // Set up context with both mode and scope
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "context mode+scope");
        commit_file_to_git(&repo, Path::new("config.toml"));

        // Import without flags - should use context
        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_import_symlink_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a target file and symlink
        let target_file = create_test_file(project_dir, "target.txt", "content");
        let link_path = project_dir.join("link.txt");
        std::os::unix::fs::symlink(&target_file, &link_path).unwrap();

        // Add symlink to Git
        commit_file_to_git(&repo, Path::new("link.txt"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("link.txt")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::SymlinkNotSupported { .. })));
    }

    #[test]
    fn test_import_binary_file_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a binary file (with null bytes)
        let binary_path = project_dir.join("binary.bin");
        fs::write(&binary_path, b"text\x00with\x00null\x00bytes").unwrap();

        // Add to Git
        commit_file_to_git(&repo, Path::new("binary.bin"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("binary.bin")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(JinError::BinaryFileNotSupported { .. })
        ));
    }

    #[test]
    fn test_import_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Clear any existing context
        let _ = std::fs::remove_file(project_dir.join(".jin/context"));

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "config");
        commit_file_to_git(&repo, Path::new("config.toml"));

        let cmd = ImportCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        // First import
        execute(&cmd).unwrap();
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);

        // Second import (same file) - should update, not duplicate
        execute(&cmd).unwrap();
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
    }

    #[test]
    fn test_resolve_path_absolute() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        let absolute_path = PathBuf::from("/tmp/test.txt");
        let resolved = resolve_path(workspace_root, &absolute_path).unwrap();

        assert_eq!(resolved, absolute_path);
    }

    #[test]
    fn test_resolve_path_relative() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        let relative_path = PathBuf::from("config.txt");
        let resolved = resolve_path(workspace_root, &relative_path).unwrap();

        assert_eq!(resolved, workspace_root.join("config.txt"));
    }

    #[test]
    fn test_detect_project_name_from_git_remote() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let repo = git2::Repository::init(project_dir).unwrap();

        // Create a fake remote
        repo.remote("origin", "https://github.com/user/myproject.git")
            .unwrap();

        let name = detect_project_name(project_dir).unwrap();
        assert_eq!(name, "myproject");
    }

    #[test]
    fn test_detect_project_name_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Initialize a Git repo without remote
        git2::Repository::init(project_dir).unwrap();

        // TempDir creates a directory with a unique name, check we get something
        let name = detect_project_name(project_dir).unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_determine_layer_with_global_flag() {
        let cmd = ImportCommand {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: true,
        };
        let context = ProjectContext::default();

        let layer = determine_layer(&cmd, &context, "testproject").unwrap();
        assert!(matches!(layer, Layer::GlobalBase));
    }

    #[test]
    fn test_determine_layer_with_mode_flag() {
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));

        let cmd = ImportCommand {
            files: vec![],
            mode: true,
            scope: None,
            project: false,
            global: false,
        };

        let layer = determine_layer(&cmd, &context, "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_determine_layer_with_scope_flag() {
        let context = ProjectContext::default();

        let cmd = ImportCommand {
            files: vec![],
            mode: false,
            scope: Some("rust".to_string()),
            project: false,
            global: false,
        };

        let layer = determine_layer(&cmd, &context, "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "rust".to_string()
            }
        );
    }

    #[test]
    fn test_determine_layer_no_flags_with_context() {
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("rust".to_string()));

        let cmd = ImportCommand {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let layer = determine_layer(&cmd, &context, "testproject").unwrap();
        assert!(matches!(layer, Layer::ModeScope { .. }));
    }

    #[test]
    fn test_determine_layer_no_flags_no_context() {
        let context = ProjectContext::default();

        let cmd = ImportCommand {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let layer = determine_layer(&cmd, &context, "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ProjectBase {
                project: "testproject".to_string()
            }
        );
    }
}
