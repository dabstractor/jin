//! Add command implementation.
//!
//! This module implements the `jin add` command that stages files to appropriate
//! layers in Jin's 9-layer configuration hierarchy based on CLI routing flags
//! and active project context.

use crate::cli::args::AddCommand;
use crate::commit::validate::check_git_tracked;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::staging::entry::StagedEntry;
use crate::staging::index::StagingIndex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Execute the add command.
///
/// Stages files to Jin's staging system, routing them to the appropriate layer
/// based on CLI flags or active context. Each file is validated for existence,
/// git-tracked status, and type before being staged.
///
/// # Arguments
///
/// * `cmd` - The add command containing files to stage and routing flags
///
/// # Errors
///
/// Returns `JinError::FileNotFound` if a file doesn't exist.
/// Returns `JinError::ValidationError` if a file is git-tracked.
/// Returns `JinError::SymlinkNotSupported` if a file is a symlink.
/// Returns `JinError::BinaryFileNotSupported` if a file is binary.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::AddCommand;
/// use jin_glm::commands::add;
/// use std::path::PathBuf;
///
/// let cmd = AddCommand {
///     files: vec![PathBuf::from("config.toml")],
///     mode: false,
///     scope: None,
///     project: false,
///     global: false,
/// };
///
/// add::execute(&cmd)?;
/// ```
pub fn execute(cmd: &AddCommand) -> Result<()> {
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

    // Track staged files by layer for summary
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

        validate_file(&resolved_path, &workspace_root)?;

        let content = std::fs::read_to_string(&resolved_path)?;
        // Pass absolute path to StagedEntry::new() for metadata reading
        let entry = StagedEntry::new(resolved_path.clone(), layer.clone(), content.as_bytes())?;

        staging_index.add_entry(entry)?;

        let layer_key = format!("{}", layer);
        staged_by_layer
            .entry(layer_key)
            .or_default()
            .push(relative_path.to_path_buf());

        println!("Staged {} to {}", file_path.display(), layer);
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
/// * `cmd` - The add command with routing flags
/// * `context` - Active project context with mode/scope
/// * `project` - Project name for routing
///
/// # Returns
///
/// The target `Layer` for staging files.
fn determine_layer(cmd: &AddCommand, context: &ProjectContext, project: &str) -> Result<Layer> {
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

/// Validates that a file can be staged.
///
/// Checks:
/// - File exists
/// - File is not a symlink
/// - File is not git-tracked
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
fn validate_file(path: &Path, workspace_root: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    // Check not symlink
    if path.is_symlink() {
        return Err(JinError::SymlinkNotSupported {
            path: path.display().to_string(),
        });
    }

    // Check git-tracked status
    let relative_path = path.strip_prefix(workspace_root).map_err(|_| {
        JinError::Message(format!(
            "File is outside workspace root: {}",
            path.display()
        ))
    })?;

    check_git_tracked(workspace_root, relative_path)?;

    // Check binary vs text file
    let content = std::fs::read(path)?;
    if content.contains(&0x00) {
        return Err(JinError::BinaryFileNotSupported {
            path: path.display().to_string(),
        });
    }

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
        let staging_index = StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_add_to_project_base() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize Git and Jin
        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create test file
        let _config_file =
            create_test_file(project_dir, "config.toml", "[settings]\nenabled = true");

        // Execute add command (no flags = project base)
        let cmd = AddCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        // Verify file was staged
        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_add_to_mode_base() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);

        // Set up context with mode
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.save(project_dir).unwrap();

        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "mode = claude");

        let cmd = AddCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: true,
            scope: None,
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_add_to_scope_base() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "scope = rust");

        let cmd = AddCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: Some("language:rust".to_string()),
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_add_to_global() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "global setting");

        let cmd = AddCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: true,
        };

        execute(&cmd).unwrap();

        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_add_to_mode_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);

        // Set up context with mode
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.save(project_dir).unwrap();

        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "mode + scope");

        let cmd = AddCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: true,
            scope: Some("language:rust".to_string()),
            project: false,
            global: false,
        };

        execute(&cmd).unwrap();

        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_add_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = AddCommand {
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
    fn test_add_git_tracked_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and commit a file to Git
        let _tracked_file = create_test_file(project_dir, "tracked.txt", "git tracked");

        // Add to Git index
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("tracked.txt")).unwrap();
        index.write().unwrap();

        // Now try to add to Jin - should fail
        let cmd = AddCommand {
            files: vec![PathBuf::from("tracked.txt")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ValidationError { .. })));
    }

    #[test]
    fn test_add_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        create_test_file(project_dir, "config1.toml", "config 1");
        create_test_file(project_dir, "config2.toml", "config 2");
        create_test_file(project_dir, "config3.toml", "config 3");

        let cmd = AddCommand {
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

        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_add_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);

        // Set up context with both mode and scope
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "context mode+scope");

        // Add without flags - should use context
        let cmd = AddCommand {
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
    fn test_add_symlink_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a target file and symlink
        let target_file = create_test_file(project_dir, "target.txt", "content");
        let link_path = project_dir.join("link.txt");
        std::os::unix::fs::symlink(&target_file, &link_path).unwrap();

        let cmd = AddCommand {
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
    fn test_add_binary_file_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a binary file (with null bytes)
        let binary_path = project_dir.join("binary.bin");
        fs::write(&binary_path, b"text\x00with\x00null\x00bytes").unwrap();

        let cmd = AddCommand {
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
    fn test_add_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "config");

        let cmd = AddCommand {
            files: vec![PathBuf::from("config.toml")],
            mode: false,
            scope: None,
            project: false,
            global: false,
        };

        // First add
        execute(&cmd).unwrap();
        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);

        // Second add (same file) - should update, not duplicate
        execute(&cmd).unwrap();
        let index = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(index.len(), 1);
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
        let cmd = AddCommand {
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

        let cmd = AddCommand {
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

        let cmd = AddCommand {
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

        let cmd = AddCommand {
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

        let cmd = AddCommand {
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
