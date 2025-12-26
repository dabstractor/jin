//! Reset command implementation.
//!
//! This module implements the `jin reset` command that unstages files from
//! Jin's staging area following Git's reset semantics with `--soft`, `--mixed`,
//! and `--hard` modes while respecting Jin's layer-based architecture.

use crate::cli::args::ResetCommand;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::staging::index::StagingIndex;
use std::path::{Path, PathBuf};

/// Reset mode tracking.
#[derive(Debug, Clone, Copy)]
enum ResetMode {
    /// Unstage only (for future layer ref movement)
    Soft,
    /// Unstage, keep workspace (default)
    Mixed,
    /// Unstage + discard workspace changes
    Hard,
}

/// Execute the reset command.
///
/// Unstages files from Jin's staging system, optionally discarding workspace
/// file changes based on the reset mode.
///
/// # Arguments
///
/// * `cmd` - The reset command containing paths and mode flags
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if not in a Git repository.
/// Returns `JinError::Message` if no staged files found in target layer.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::ResetCommand;
/// use jin_glm::commands::reset;
/// use std::path::PathBuf;
///
/// let cmd = ResetCommand {
///     paths: vec![],
///     mode: false,
///     scope: None,
///     project: false,
///     soft: false,
///     mixed: false,
///     hard: false,
/// };
///
/// reset::execute(&cmd)?;
/// ```
pub fn execute(cmd: &ResetCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Load project context
    let context = ProjectContext::load(&workspace_root)?;

    // 3. Detect project name
    let project_name = detect_project_name(&workspace_root)?;

    // 4. Validate Git repository
    let _git_repo =
        git2::Repository::discover(&workspace_root).map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // 5. Open Jin repository (for future layer ref operations)
    let _repo = crate::git::JinRepo::open_or_create(&workspace_root)?;

    // 6. Load staging index
    let mut staging =
        StagingIndex::load_from_disk(&workspace_root).unwrap_or_else(|_| StagingIndex::new());

    // 7. Determine target layer
    let target_layer = determine_reset_layer(cmd, &context, &project_name)?;

    // 8. Get paths to reset
    let paths_to_reset = get_paths_to_reset(cmd, &target_layer, &staging, &workspace_root)?;

    // 9. Validate there are files to reset
    if paths_to_reset.is_empty() {
        return Err(JinError::Message(
            "No staged files found in target layer. Use 'jin status' to see staged files."
                .to_string(),
        ));
    }

    // 10. Determine reset mode
    let reset_mode = match (cmd.soft, cmd.mixed, cmd.hard) {
        (true, _, _) => ResetMode::Soft,
        (_, true, _) => ResetMode::Mixed,
        (_, _, true) => ResetMode::Hard,
        (false, false, false) => ResetMode::Mixed, // Default
    };

    // 11. Show preview
    println!(
        "Resetting {} file(s) from layer: {}",
        paths_to_reset.len(),
        target_layer
    );
    match reset_mode {
        ResetMode::Soft => println!("Mode: soft (unstage only)"),
        ResetMode::Mixed => println!("Mode: mixed (unstage, keep workspace)"),
        ResetMode::Hard => println!("Mode: hard (unstage + discard workspace changes)"),
    }
    for path in &paths_to_reset {
        println!("  {}", path.display());
    }

    // 12. Execute reset
    let count = match reset_mode {
        ResetMode::Soft => execute_soft_reset(&paths_to_reset, &mut staging)?,
        ResetMode::Mixed => execute_mixed_reset(&paths_to_reset, &mut staging)?,
        ResetMode::Hard => execute_hard_reset(&paths_to_reset, &mut staging, &workspace_root)?,
    };

    // 13. Save staging index
    staging.save_to_disk(&workspace_root)?;

    // 14. Print success
    println!("\nReset complete. {} file(s) affected.", count);

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
/// * `cmd` - The reset command with routing flags
/// * `context` - Active project context with mode/scope
/// * `project` - Project name for routing
///
/// # Returns
///
/// The target `Layer` for reset operations.
fn determine_reset_layer(
    cmd: &ResetCommand,
    context: &ProjectContext,
    project: &str,
) -> Result<Layer> {
    // If any flags specified, use explicit routing
    if cmd.mode || cmd.scope.is_some() || cmd.project {
        let mode = if cmd.mode {
            context.mode.as_deref()
        } else {
            None
        };
        let scope = cmd.scope.as_deref().or(context.scope.as_deref());
        let proj = if cmd.project { Some(project) } else { None };

        return Layer::from_flags(mode, scope, proj, false).ok_or_else(|| {
            JinError::Message(
                "No routing target specified. Use --mode, --scope, or --project".to_string(),
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

/// Normalizes a path to be relative to workspace root.
///
/// Converts absolute paths to relative, keeps relative paths as-is.
///
/// # Arguments
///
/// * `path` - Path to normalize
/// * `workspace_root` - Path to workspace root
///
/// # Returns
///
/// Normalized relative path.
fn normalize_path(path: &Path, workspace_root: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map(|p| p.to_path_buf())
            .map_err(|_| {
                JinError::Message(format!(
                    "File is outside workspace root: {}",
                    path.display()
                ))
            })
    } else {
        Ok(path.to_path_buf())
    }
}

/// Gets the list of paths to reset based on command arguments.
///
/// If paths are specified, returns those paths (normalized).
/// If no paths specified, returns all paths from the target layer.
///
/// # Arguments
///
/// * `cmd` - The reset command
/// * `target_layer` - The layer to reset from
/// * `staging` - The staging index
/// * `workspace_root` - Path to workspace root
///
/// # Returns
///
/// Vector of paths to reset.
fn get_paths_to_reset(
    cmd: &ResetCommand,
    target_layer: &Layer,
    staging: &StagingIndex,
    workspace_root: &Path,
) -> Result<Vec<PathBuf>> {
    if cmd.paths.is_empty() {
        // Reset all files from target layer
        let entries = staging.entries_by_layer(target_layer);
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        Ok(entries.iter().map(|e| e.path.clone()).collect())
    } else {
        // Reset specific paths
        cmd.paths
            .iter()
            .map(|p| normalize_path(p, workspace_root))
            .collect::<Result<Vec<_>>>()
    }
}

/// Executes a soft reset - removes entries from staging only.
///
/// # Arguments
///
/// * `paths` - Paths to reset
/// * `staging` - Mutable staging index
///
/// # Returns
///
/// Count of entries actually removed.
fn execute_soft_reset(paths: &[PathBuf], staging: &mut StagingIndex) -> Result<usize> {
    let mut count = 0;
    for path in paths {
        if staging.remove_entry(path).is_some() {
            count += 1;
        }
    }
    Ok(count)
}

/// Executes a mixed reset - removes entries from staging, keeps workspace.
///
/// For now, mixed and soft are the same - both unstage.
/// The difference would matter when we implement Git ref operations.
///
/// # Arguments
///
/// * `paths` - Paths to reset
/// * `staging` - Mutable staging index
///
/// # Returns
///
/// Count of entries actually removed.
fn execute_mixed_reset(paths: &[PathBuf], staging: &mut StagingIndex) -> Result<usize> {
    // For now, mixed and soft are the same - both unstage
    // The difference would matter when we implement Git ref operations
    execute_soft_reset(paths, staging)
}

/// Executes a hard reset - removes entries from staging AND discards workspace files.
///
/// # Arguments
///
/// * `paths` - Paths to reset
/// * `staging` - Mutable staging index
/// * `workspace_root` - Path to workspace root
///
/// # Returns
///
/// Count of entries actually removed.
fn execute_hard_reset(
    paths: &[PathBuf],
    staging: &mut StagingIndex,
    workspace_root: &Path,
) -> Result<usize> {
    let mut count = 0;
    for path in paths {
        if staging.remove_entry(path).is_some() {
            count += 1;
        }

        // Remove from workspace if exists
        let workspace_file = workspace_root.join(path);
        if workspace_file.exists() {
            std::fs::remove_file(&workspace_file).map_err(|e| {
                JinError::Message(format!(
                    "Failed to remove {}: {}",
                    workspace_file.display(),
                    e
                ))
            })?;
            println!("  Discarded: {}", path.display());
        }
    }
    Ok(count)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to prevent tests from running in parallel and causing state leakage
    static TEST_LOCK: Mutex<()> = Mutex::new(());

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

    /// Helper to stage a file to a specific layer
    fn stage_file(dir: &Path, file_name: &str, layer: Layer) {
        let file_path = dir.join(file_name);
        let content = fs::read_to_string(&file_path).unwrap();
        // Normalize path to relative like add command does
        let relative_path = file_path.strip_prefix(dir).unwrap();
        let mut staging = StagingIndex::load_from_disk(dir).unwrap();
        let entry = crate::staging::entry::StagedEntry::new(
            relative_path.to_path_buf(),
            layer,
            content.as_bytes(),
        )
        .unwrap();
        staging.add_entry(entry).unwrap();
        staging.save_to_disk(dir).unwrap();
    }

    /// Helper to get detected project name for a directory
    fn get_detected_project_name(dir: &Path) -> String {
        detect_project_name(dir).unwrap()
    }

    /// Setup test environment with serial execution lock
    fn setup_test_env() -> (TempDir, DirGuard) {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let guard = DirGuard::new().unwrap();
        (temp_dir, guard)
    }

    #[test]
    fn test_reset_soft_removes_from_staging() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "setting = true");
        let project_name = get_detected_project_name(project_dir);
        let layer = Layer::ProjectBase {
            project: project_name,
        };
        stage_file(project_dir, "config.toml", layer.clone());

        // Verify staged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);

        // Execute soft reset
        let cmd = ResetCommand {
            paths: vec![],
            mode: false,
            scope: None,
            project: false,
            soft: true,
            mixed: false,
            hard: false,
        };

        execute(&cmd).unwrap();

        // Verify unstaged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 0);

        // Verify file still exists
        assert!(project_dir.join("config.toml").exists());
    }

    #[test]
    fn test_reset_mixed_removes_from_staging() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "setting = true");
        let project_name = get_detected_project_name(project_dir);
        let layer = Layer::ProjectBase {
            project: project_name,
        };
        stage_file(project_dir, "config.toml", layer.clone());

        // Verify staged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);

        // Execute mixed reset
        let cmd = ResetCommand {
            paths: vec![],
            mode: false,
            scope: None,
            project: false,
            soft: false,
            mixed: true,
            hard: false,
        };

        execute(&cmd).unwrap();

        // Verify unstaged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 0);

        // Verify file still exists
        assert!(project_dir.join("config.toml").exists());
    }

    #[test]
    fn test_reset_hard_removes_files() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let _config_file = create_test_file(project_dir, "config.toml", "setting = true");
        let project_name = get_detected_project_name(project_dir);
        let layer = Layer::ProjectBase {
            project: project_name,
        };
        stage_file(project_dir, "config.toml", layer.clone());

        // Verify staged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);

        // Execute hard reset
        let cmd = ResetCommand {
            paths: vec![],
            mode: false,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: true,
        };

        execute(&cmd).unwrap();

        // Verify unstaged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 0);

        // Verify file removed
        assert!(!project_dir.join("config.toml").exists());
    }

    #[test]
    fn test_reset_with_paths() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        create_test_file(project_dir, "a.txt", "content a");
        create_test_file(project_dir, "b.txt", "content b");
        let project_name = get_detected_project_name(project_dir);
        let layer = Layer::ProjectBase {
            project: project_name,
        };
        stage_file(project_dir, "a.txt", layer.clone());
        stage_file(project_dir, "b.txt", layer.clone());

        // Verify both staged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 2);

        // Reset only a.txt
        let cmd = ResetCommand {
            paths: vec![PathBuf::from("a.txt")],
            mode: false,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: false,
        };

        execute(&cmd).unwrap();

        // Verify only a.txt unstaged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 1);
        assert!(staging.get_entry(Path::new("a.txt")).is_none());
        assert!(staging.get_entry(Path::new("b.txt")).is_some());
    }

    #[test]
    fn test_reset_all_paths_when_empty() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        create_test_file(project_dir, "a.txt", "content a");
        create_test_file(project_dir, "b.txt", "content b");
        let project_name = get_detected_project_name(project_dir);
        let layer = Layer::ProjectBase {
            project: project_name,
        };
        stage_file(project_dir, "a.txt", layer.clone());
        stage_file(project_dir, "b.txt", layer.clone());

        // Verify both staged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 2);

        // Reset with empty paths (should reset all)
        let cmd = ResetCommand {
            paths: vec![],
            mode: false,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: false,
        };

        execute(&cmd).unwrap();

        // Verify both unstaged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 0);
    }

    #[test]
    fn test_reset_layer_targeting() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set up context with mode BEFORE creating staging index
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.save(project_dir).unwrap();

        create_test_file(project_dir, "config.toml", "mode = claude");
        let mode_layer = Layer::ModeBase {
            mode: "claude".to_string(),
        };
        stage_file(project_dir, "config.toml", mode_layer);

        // Reset with --mode flag
        let cmd = ResetCommand {
            paths: vec![],
            mode: true,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: false,
        };

        execute(&cmd).unwrap();

        // Verify unstaged
        let staging = StagingIndex::load_from_disk(project_dir).unwrap();
        assert_eq!(staging.len(), 0);
    }

    #[test]
    fn test_reset_no_staged_files_error() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Try to reset when nothing staged
        let cmd = ResetCommand {
            paths: vec![],
            mode: false,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: false,
        };

        let result = execute(&cmd);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No staged files found"));
    }

    #[test]
    fn test_determine_reset_layer_with_flags() {
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));

        let cmd = ResetCommand {
            paths: vec![],
            mode: true,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: false,
        };

        let layer = determine_reset_layer(&cmd, &context, "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_determine_reset_layer_with_context() {
        let mut context = ProjectContext::default();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("rust".to_string()));

        let cmd = ResetCommand {
            paths: vec![],
            mode: false,
            scope: None,
            project: false,
            soft: false,
            mixed: false,
            hard: false,
        };

        let layer = determine_reset_layer(&cmd, &context, "testproject").unwrap();
        assert!(matches!(layer, Layer::ModeScope { .. }));
    }

    #[test]
    fn test_normalize_path_absolute() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        let absolute_path = workspace_root.join("test.txt");
        let normalized = normalize_path(&absolute_path, workspace_root).unwrap();

        assert_eq!(normalized, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_normalize_path_relative() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        let relative_path = PathBuf::from("config.txt");
        let normalized = normalize_path(&relative_path, workspace_root).unwrap();

        assert_eq!(normalized, PathBuf::from("config.txt"));
    }

    #[test]
    fn test_normalize_path_outside_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();

        let outside_path = PathBuf::from("/tmp/outside.txt");
        let result = normalize_path(&outside_path, workspace_root);

        assert!(result.is_err());
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
}
