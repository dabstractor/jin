//! Log command implementation.
//!
//! This module implements the `jin log` command that displays commit history
//! for Jin layers using git2-rs RevWalk API.
//!
//! The log command shows:
//! - Commit history for all layers or a specific layer
//! - Short SHA, author, relative time, and commit message
//! - Colored output similar to `git log --oneline`
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::cli::args::LogCommand;
//! use jin_glm::commands::log;
//!
//! // Show all layer commits
//! let cmd = LogCommand { layer: None, count: None };
//! log::execute(&cmd)?;
//!
//! // Show commits for a specific layer
//! let cmd = LogCommand {
//!     layer: Some("mode/claude".to_string()),
//!     count: None,
//! };
//! log::execute(&cmd)?;
//!
//! // Show last 5 commits
//! let cmd = LogCommand {
//!     layer: None,
//!     count: Some(5),
//! };
//! log::execute(&cmd)?;
//! ```

use crate::cli::args::LogCommand;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use std::path::Path;

// ===== ANSI COLOR CODES =====

const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_RESET: &str = "\x1b[0m";

// ===== LAYER PARSING =====

/// Parses a layer specification string into a Layer enum.
///
/// Supports formats like:
/// - "global" -> Layer::GlobalBase
/// - "mode/<name>" -> Layer::ModeBase
/// - "scope/<name>" -> Layer::ScopeBase
/// - "project/<name>" -> Layer::ProjectBase
/// - "mode/<m>/scope/<s>" -> Layer::ModeScope
/// - "mode/<m>/project/<p>" -> Layer::ModeProject
/// - "mode/<m>/scope/<s>/project/<p>" -> Layer::ModeScopeProject
///
/// # Arguments
///
/// * `spec` - The layer specification string to parse
/// * `_project` - The project name (currently unused)
///
/// # Returns
///
/// The parsed Layer variant.
///
/// # Errors
///
/// Returns `JinError::Message` if the specification format is invalid.
fn parse_layer_spec(spec: &str, _project: &str) -> Result<Layer> {
    let parts: Vec<&str> = spec.split('/').collect();

    match parts.as_slice() {
        ["global"] => Ok(Layer::GlobalBase),
        ["mode", name] => Ok(Layer::ModeBase {
            mode: name.to_string(),
        }),
        ["scope", name] => Ok(Layer::ScopeBase {
            scope: name.to_string(),
        }),
        ["project", name] => Ok(Layer::ProjectBase {
            project: name.to_string(),
        }),
        ["mode", mode_name, "scope", scope_name] => Ok(Layer::ModeScope {
            mode: mode_name.to_string(),
            scope: scope_name.to_string(),
        }),
        ["mode", mode_name, "project", proj_name] => Ok(Layer::ModeProject {
            mode: mode_name.to_string(),
            project: proj_name.to_string(),
        }),
        ["mode", mode_name, "scope", scope_name, "project", proj_name] => {
            Ok(Layer::ModeScopeProject {
                mode: mode_name.to_string(),
                scope: scope_name.to_string(),
                project: proj_name.to_string(),
            })
        }
        _ => Err(JinError::Message(format!(
            "Invalid layer specification: '{}'. Expected format: global, mode/<name>, scope/<name>, project/<name>, mode/<m>/scope/<s>, mode/<m>/project/<p>, or mode/<m>/scope/<s>/project/<p>",
            spec
        ))),
    }
}

// ===== PROJECT NAME DETECTION =====

/// Detects the project name from Git remote or directory name.
///
/// First tries to get the project name from the git remote origin URL.
/// Falls back to the directory name if no remote is configured.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// The detected project name.
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if not in a Git repository.
fn detect_project_name(workspace_root: &Path) -> Result<String> {
    let repo = git2::Repository::discover(workspace_root).map_err(|_| JinError::RepoNotFound {
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

// ===== COMMIT HISTORY WALKING =====

/// Walks the commit history for a specific layer.
///
/// Uses git2-rs RevWalk to iterate through commits in reverse
/// chronological order (newest first).
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `layer` - The layer to get history for
/// * `count` - Optional limit on number of commits to return
///
/// # Returns
///
/// A vector of git2 commits in reverse chronological order.
///
/// # Errors
///
/// Returns `JinError::Message` if the layer has no history or
/// if the git reference cannot be resolved.
fn walk_layer_history<'a>(
    repo: &'a JinRepo,
    layer: &Layer,
    count: Option<usize>,
) -> Result<Vec<git2::Commit<'a>>> {
    // Check if layer is versioned
    if !layer.is_versioned() {
        return Err(JinError::Message(format!(
            "Layer '{}' is not versioned and has no history",
            layer
        )));
    }

    // Get the layer's git reference
    let ref_name = layer.git_ref().ok_or_else(|| {
        JinError::Message(format!(
            "Layer '{}' has no git reference",
            layer
        ))
    })?;

    // Find the reference in the git repository
    let reference = repo
        .inner
        .find_reference(&ref_name)
        .map_err(|_| JinError::Message(format!("Layer '{}' has no history", layer)))?;

    // Get the commit OID that the reference points to
    let commit_oid = reference.target().ok_or_else(|| {
        JinError::Message("Layer reference points to nothing".to_string())
    })?;

    // Create a RevWalk for iterating commits
    // CRITICAL: Set sorting for newest-first order (TIME | REVERSE)
    let mut walk = repo.inner.revwalk()?;
    walk.push(commit_oid)?;
    walk.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)?;

    // Collect commits with optional limit
    // Note: RevWalk iterator returns Result<Oid, git2::Error>, need to handle that
    let mut commits = Vec::new();
    for oid_result in walk.take(count.unwrap_or(usize::MAX)) {
        let oid = oid_result.map_err(|e| JinError::Message(format!("Git error: {}", e)))?;
        if let Ok(commit) = repo.find_commit(oid) {
            commits.push(commit);
        }
    }

    Ok(commits)
}

// ===== TIME FORMATTING =====

/// Formats a commit timestamp as a relative time string.
///
/// # Arguments
///
/// * `seconds` - Unix timestamp in seconds
///
/// # Returns
///
/// A human-readable relative time string (e.g., "2 hours ago", "1 day ago")
fn format_relative_time(seconds: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let diff = now - seconds;

    if diff < 60 {
        format!("{} seconds ago", diff)
    } else if diff < 3600 {
        let minutes = diff / 60;
        if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        }
    } else if diff < 86400 {
        let hours = diff / 3600;
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if diff < 604800 {
        let days = diff / 86400;
        if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        }
    } else if diff < 2592000 {
        let weeks = diff / 604800;
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{} weeks ago", weeks)
        }
    } else if diff < 31536000 {
        let months = diff / 2592000;
        if months == 1 {
            "1 month ago".to_string()
        } else {
            format!("{} months ago", months)
        }
    } else {
        let years = diff / 31536000;
        if years == 1 {
            "1 year ago".to_string()
        } else {
            format!("{} years ago", years)
        }
    }
}

// ===== DISPLAY FUNCTIONS =====

/// Displays the commit history for a single layer.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `layer` - The layer to display history for
/// * `count` - Optional limit on number of commits to display
///
/// # Errors
///
/// Propagates errors from walk_layer_history.
fn display_layer_history(repo: &JinRepo, layer: &Layer, count: Option<usize>) -> Result<()> {
    let commits = walk_layer_history(repo, layer, count)?;

    println!();
    println!("{}:", layer);

    if commits.is_empty() {
        println!("  (no commits)");
        return Ok(());
    }

    for commit in commits {
        let short_sha = &commit.id().to_string()[..8];
        let author = commit.author();
        let author_name = author.name().unwrap_or("(unknown)");
        let time_str = format_relative_time(commit.time().seconds());
        let msg = commit.message().unwrap_or("(no message)");

        // Get first line of message
        let first_line = msg.lines().next().unwrap_or("");

        println!(
            "  {}{}{} {}{}{} <{}>",
            ANSI_YELLOW, short_sha, ANSI_RESET,
            ANSI_GREEN, author_name, ANSI_RESET,
            author.email().unwrap_or("")
        );
        println!(
            "    {}{}{} {}",
            ANSI_BLUE, time_str, ANSI_RESET,
            first_line
        );
    }

    Ok(())
}

// ===== MAIN EXECUTE =====

/// Execute the log command.
///
/// This is the main entry point for the `jin log` command.
///
/// # Behavior
///
/// - With no arguments: shows commits from all layers
/// - With a layer spec: shows commits for that specific layer
/// - With --count N: limits output to N commits per layer
///
/// # Arguments
///
/// * `cmd` - The log command containing CLI arguments
///
/// # Errors
///
/// Returns `JinError::Message` if:
/// - Not in a Jin-initialized directory
/// - Invalid layer specification is provided
/// - Git repository is not found
pub fn execute(cmd: &LogCommand) -> Result<()> {
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

    // 3. Detect project name
    let project_name = detect_project_name(&workspace_root)?;

    // 4. Validate Git repository exists
    let _git_repo = git2::Repository::discover(&workspace_root).map_err(|_| {
        JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        }
    })?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 6. Resolve layer(s) to query
    let layers = if let Some(layer_spec) = &cmd.layer {
        // Parse single layer
        vec![parse_layer_spec(layer_spec, &project_name)?]
    } else {
        // Get all layers with commits
        let refs = repo.list_layer_refs()?;
        if refs.is_empty() {
            println!("No layers with commits found.");
            return Ok(());
        }
        refs.into_iter().map(|(layer, _oid)| layer).collect()
    };

    // 7. Display commits for each layer
    for layer in layers {
        display_layer_history(&repo, &layer, cmd.count)?;
    }

    Ok(())
}

// ===== TESTS =====

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
        use crate::staging::index::StagingIndex;
        let staging_index = StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        // Create workspace directory
        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_parse_layer_spec_global() {
        let layer = parse_layer_spec("global", "testproject").unwrap();
        assert_eq!(layer, Layer::GlobalBase);
    }

    #[test]
    fn test_parse_layer_spec_mode() {
        let layer = parse_layer_spec("mode/claude", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_scope() {
        let layer = parse_layer_spec("scope/python", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_project() {
        let layer = parse_layer_spec("project/myproject", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ProjectBase {
                project: "myproject".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_mode_scope() {
        let layer = parse_layer_spec("mode/claude/scope/python", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_mode_project() {
        let layer = parse_layer_spec("mode/claude/project/myapp", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_mode_scope_project() {
        let layer =
            parse_layer_spec("mode/claude/scope/python/project/myapp", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeScopeProject {
                mode: "claude".to_string(),
                scope: "python".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_invalid() {
        let result = parse_layer_spec("invalid/format", "testproject");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_relative_time() {
        // Test various time differences
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert!(format_relative_time(now).contains("seconds"));
        assert!(format_relative_time(now - 120).contains("minutes"));
        assert!(format_relative_time(now - 7200).contains("hours"));
        assert!(format_relative_time(now - 172800).contains("days"));
        assert!(format_relative_time(now - 1814400).contains("weeks"));
    }

    #[test]
    fn test_detect_project_name_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Initialize a Git repo without remote
        git2::Repository::init(project_dir).unwrap();

        let name = detect_project_name(project_dir).unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_detect_project_name_from_git() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let repo = git2::Repository::init(project_dir).unwrap();
        repo.remote("origin", "https://github.com/user/myproject.git")
            .unwrap();

        let name = detect_project_name(project_dir).unwrap();
        assert_eq!(name, "myproject");
    }

    #[test]
    fn test_detect_project_name_no_git() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let result = detect_project_name(project_dir);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::RepoNotFound { .. })));
    }

    #[test]
    fn test_log_no_jin_initialized_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize Jin

        let cmd = LogCommand {
            layer: None,
            count: None,
        };
        let result = execute(&cmd);

        assert!(result.is_err());
        if let Err(JinError::Message(msg)) = result {
            assert!(msg.contains("Jin is not initialized"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_log_shows_empty_with_no_commits() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a regular git repo first (needed for discover())
        git2::Repository::init(project_dir).unwrap();
        init_jin(project_dir);

        let cmd = LogCommand {
            layer: None,
            count: None,
        };
        // Should not error, just show no layers
        execute(&cmd).unwrap();
    }

    #[test]
    fn test_log_shows_all_layers() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a regular git repo (this is what execute() will use)
        let git_repo = git2::Repository::init(project_dir).unwrap();
        init_jin(project_dir);

        // Create commits in different layers
        let time = git2::Time::new(0, 0);
        let signature = git2::Signature::new("Test User", "test@example.com", &time).unwrap();

        // Global layer commit
        let tree_oid = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree = git_repo.find_tree(tree_oid).unwrap();
        git_repo.commit(
            Some("refs/jin/layers/global"),
            &signature,
            &signature,
            "First global commit",
            &tree,
            &[],
        ).unwrap();

        // Mode layer commit
        let tree_oid2 = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree2 = git_repo.find_tree(tree_oid2).unwrap();
        git_repo.commit(
            Some("refs/jin/layers/mode/claude"),
            &signature,
            &signature,
            "Claude mode commit",
            &tree2,
            &[],
        ).unwrap();

        let cmd = LogCommand {
            layer: None,
            count: None,
        };
        execute(&cmd).unwrap();
    }

    #[test]
    fn test_log_specific_layer() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a regular git repo (this is what execute() will use)
        let git_repo = git2::Repository::init(project_dir).unwrap();
        init_jin(project_dir);

        // Create a commit directly in the git repo for the global layer
        let ref_name = "refs/jin/layers/global";
        let tree_oid = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree = git_repo.find_tree(tree_oid).unwrap();
        let time = git2::Time::new(0, 0);
        let signature = git2::Signature::new("Test User", "test@example.com", &time).unwrap();
        git_repo.commit(
            Some(ref_name),
            &signature,
            &signature,
            "Test commit",
            &tree,
            &[],
        ).unwrap();

        let cmd = LogCommand {
            layer: Some("global".to_string()),
            count: None,
        };
        execute(&cmd).unwrap();
    }

    #[test]
    fn test_log_with_count_limit() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Initialize a regular git repo (this is what execute() will use)
        let git_repo = git2::Repository::init(project_dir).unwrap();
        init_jin(project_dir);

        // Create multiple commits in the global layer
        let time = git2::Time::new(0, 0);
        let signature = git2::Signature::new("Test User", "test@example.com", &time).unwrap();
        let ref_name = "refs/jin/layers/global";

        // First commit
        let tree_oid = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree = git_repo.find_tree(tree_oid).unwrap();
        git_repo.commit(
            Some(ref_name),
            &signature,
            &signature,
            "Commit 1",
            &tree,
            &[],
        ).unwrap();

        // Second commit
        let tree_oid2 = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree2 = git_repo.find_tree(tree_oid2).unwrap();
        let commit1_oid = git_repo.commit(
            Some(ref_name),
            &signature,
            &signature,
            "Commit 2",
            &tree2,
            &[&git_repo.find_commit(git_repo.refname_to_id(ref_name).unwrap()).unwrap()],
        ).unwrap();

        // Third commit
        let tree_oid3 = git_repo.treebuilder(None).unwrap().write().unwrap();
        let tree3 = git_repo.find_tree(tree_oid3).unwrap();
        git_repo.commit(
            Some(ref_name),
            &signature,
            &signature,
            "Commit 3",
            &tree3,
            &[&git_repo.find_commit(commit1_oid).unwrap()],
        ).unwrap();

        let cmd = LogCommand {
            layer: Some("global".to_string()),
            count: Some(2),
        };
        execute(&cmd).unwrap();
    }

    #[test]
    fn test_log_invalid_layer_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = LogCommand {
            layer: Some("invalid_layer".to_string()),
            count: None,
        };
        let result = execute(&cmd);

        assert!(result.is_err());
    }

    #[test]
    fn test_walk_layer_history_with_empty_layer() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let repo = JinRepo::open_or_create(project_dir).unwrap();

        // Try to walk a layer with no commits
        let result = walk_layer_history(&repo, &Layer::GlobalBase, None);
        assert!(result.is_err());
    }
}
