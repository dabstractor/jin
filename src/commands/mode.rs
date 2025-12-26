//! Mode command implementation.
//!
//! This module implements the `jin mode` subcommands that manage AI/editor-specific
//! configuration layers including create, use, unset, delete, and show operations.

use crate::cli::args::ModeCommand;
use crate::core::config::{JinConfig, ProjectContext};
use crate::core::error::{JinError, Result};
use crate::git::JinRepo;
use std::path::Path;

/// Execute the mode subcommand.
///
/// Routes to the appropriate handler based on the subcommand variant.
/// Uses workspace root detection and context loading following established patterns.
///
/// # Arguments
///
/// * `cmd` - The mode subcommand to execute
///
/// # Errors
///
/// Returns `JinError::ModeNotFound` if mode doesn't exist (for use/delete).
/// Returns `JinError::RefExists` if mode already exists (for create).
/// Returns `JinError::Message` if trying to delete active mode.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::ModeCommand;
/// use jin_glm::commands::mode;
///
/// let cmd = ModeCommand::Create { name: "claude".to_string() };
/// mode::execute(&cmd)?;
/// ```
pub fn execute(cmd: &ModeCommand) -> Result<()> {
    let workspace_root = std::env::current_dir()?;

    match cmd {
        ModeCommand::Create { name } => execute_create(name),
        ModeCommand::Use { name } => execute_use(&workspace_root, name),
        ModeCommand::Unset => execute_unset(&workspace_root),
        ModeCommand::Delete { name } => execute_delete(&workspace_root, name),
        ModeCommand::Show => execute_show(&workspace_root),
    }
}

/// Execute the modes list command.
///
/// Lists all available modes with their commit OIDs.
/// This is called from main.rs for the `jin modes` command.
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if Jin repository doesn't exist.
///
/// # Examples
///
/// ```ignore
/// mode::execute_list()?;
/// ```
pub fn execute_list() -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Find all mode refs using the layer ref pattern
    let mode_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        return Ok(());
    }

    println!("Available modes:");
    for ref_name in mode_refs {
        // Extract mode name from ref path
        let mode_name = ref_name
            .strip_prefix("refs/jin/layers/mode/")
            .unwrap_or(&ref_name);

        // Get the commit OID for this ref
        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:20} {}", mode_name, short_oid);
            }
        }
    }

    Ok(())
}

/// Execute mode create command.
///
/// Creates a new mode by initializing a Git ref at `refs/jin/layers/mode/<name>`.
/// Creates an empty initial commit for the mode.
///
/// # Arguments
///
/// * `name` - The mode name to create
fn execute_create(name: &str) -> Result<()> {
    // Validate mode name format (alphanumeric, hyphens, underscores)
    if !is_valid_mode_name(name) {
        return Err(JinError::ValidationError {
            message: format!(
                "Invalid mode name: '{}'. Use alphanumeric, hyphens, underscores.",
                name
            ),
        });
    }

    // Load Jin config to get repository path
    let config = JinConfig::load()?;
    let repo = JinRepo::open_or_create(&config.repository)?;

    // Check mode doesn't already exist
    let ref_path = format!("refs/jin/layers/mode/{}", name);
    if repo.find_reference(&ref_path).is_ok() {
        return Err(JinError::RefExists {
            name: ref_path.clone(),
            layer: format!("mode/{}", name),
        });
    }

    // Create empty tree and initial commit for the mode
    let empty_tree_oid = repo.create_empty_tree()?;
    let empty_tree = repo.find_tree(empty_tree_oid)?;

    let author = repo.signature("Jin", "jin@local")?;
    let committer = &author;

    let initial_commit_oid = repo.create_commit(
        None,
        &author,
        committer,
        &format!("Initial commit for mode: {}", name),
        &empty_tree,
        &[],
    )?;

    // Create the mode ref pointing to the initial commit
    repo.create_reference(
        &ref_path,
        initial_commit_oid,
        false,
        &format!("Create mode: {}", name),
    )?;

    println!("Mode '{}' created.", name);
    Ok(())
}

/// Execute mode use command.
///
/// Sets the active mode in `.jin/context` YAML file.
/// Creates `.jin/context` if it doesn't exist.
/// Preserves existing scope if set.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
/// * `name` - The mode name to activate
fn execute_use(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check mode exists
    let ref_path = format!("refs/jin/layers/mode/{}", name);
    if repo.find_reference(&ref_path).is_err() {
        return Err(JinError::ModeNotFound {
            mode: name.to_string(),
        });
    }

    // Load and update context
    let mut context = ProjectContext::load(workspace_root)?;
    context.set_mode(Some(name.to_string()));
    context.save(workspace_root)?;

    println!("Mode '{}' is now active.", name);
    Ok(())
}

/// Execute mode unset command.
///
/// Removes mode field from `.jin/context`.
/// Preserves scope field if present.
/// No error if no mode is set.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
fn execute_unset(workspace_root: &Path) -> Result<()> {
    let mut context = ProjectContext::load(workspace_root)?;
    context.set_mode(None);
    context.save(workspace_root)?;

    println!("Mode deactivated.");
    Ok(())
}

/// Execute mode delete command.
///
/// Deletes the Git ref at `refs/jin/layers/mode/<name>`.
/// Fails if mode is currently active (requires `jin mode unset` first).
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
/// * `name` - The mode name to delete
fn execute_delete(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check mode exists
    let ref_path = format!("refs/jin/layers/mode/{}", name);
    if repo.find_reference(&ref_path).is_err() {
        return Err(JinError::ModeNotFound {
            mode: name.to_string(),
        });
    }

    // Check mode is not active
    let context = ProjectContext::load(workspace_root)?;
    if context.mode.as_ref() == Some(&name.to_string()) {
        return Err(JinError::Message(
            "Cannot delete active mode. Use 'jin mode unset' first.".to_string(),
        ));
    }

    // Delete the ref
    let mut reference = repo.find_reference(&ref_path)?;
    reference.delete()?;

    println!("Mode '{}' deleted.", name);
    Ok(())
}

/// Execute mode show command.
///
/// Displays current active mode from `.jin/context`.
/// Shows scope if also active.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
fn execute_show(workspace_root: &Path) -> Result<()> {
    let context = ProjectContext::load(workspace_root)?;

    match &context.mode {
        Some(mode) => {
            println!("Active mode: {}", mode);
            if context.has_scope() {
                println!("Active scope: {}", context.scope.as_ref().unwrap());
            }
        }
        None => {
            println!("No active mode");
            if context.has_scope() {
                println!("Active scope: {}", context.scope.as_ref().unwrap());
            }
        }
    }

    Ok(())
}

/// Validates mode name format.
///
/// Mode names must be non-empty and contain only alphanumeric characters,
/// hyphens, and underscores.
///
/// # Arguments
///
/// * `name` - The mode name to validate
///
/// # Returns
///
/// `true` if the name is valid, `false` otherwise.
fn is_valid_mode_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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
        let context = ProjectContext::default();
        context.save(dir).unwrap();

        let staging_index = crate::staging::index::StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();

        // Create local .jin/config.yaml to use the current directory as the repository
        // This isolates tests from the global Jin repository
        let local_config = crate::core::config::JinConfig {
            version: 1,
            repository: PathBuf::from("."),
            default_mode: None,
            default_scope: None,
        };
        let config_path = dir.join(".jin").join("config.yaml");
        let yaml_content = serde_yaml_ng::to_string(&local_config).unwrap();
        std::fs::write(config_path, yaml_content).unwrap();
    }

    #[test]
    fn test_execute_create_creates_mode_ref() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a mode
        let result = execute_create("claude");
        assert!(result.is_ok());

        // Verify the mode ref exists
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        assert!(repo.find_reference("refs/jin/layers/mode/claude").is_ok());
    }

    #[test]
    fn test_execute_create_fails_if_mode_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a mode
        execute_create("claude").unwrap();

        // Try to create the same mode again
        let result = execute_create("claude");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::RefExists { .. })));
    }

    #[test]
    fn test_execute_create_validates_mode_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Empty name should fail
        let result = execute_create("");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ValidationError { .. })));

        // Invalid characters should fail
        let result = execute_create("invalid name!");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ValidationError { .. })));

        // Valid names should succeed
        assert!(execute_create("valid-mode").is_ok());
        assert!(execute_create("another_mode").is_ok());
        assert!(execute_create("Mode123").is_ok());
    }

    #[test]
    fn test_execute_use_sets_active_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a mode first
        execute_create("claude").unwrap();

        // Use the mode
        execute_use(project_dir, "claude").unwrap();

        // Verify context has the mode set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.mode, Some("claude".to_string()));
    }

    #[test]
    fn test_execute_use_fails_if_mode_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Try to use a non-existent mode
        let result = execute_use(project_dir, "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ModeNotFound { .. })));
    }

    #[test]
    fn test_execute_unset_clears_active_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and use a mode
        execute_create("claude").unwrap();
        execute_use(project_dir, "claude").unwrap();

        // Verify mode is set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.mode, Some("claude".to_string()));

        // Unset the mode
        execute_unset(project_dir).unwrap();

        // Verify mode is cleared
        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.mode.is_none());
    }

    #[test]
    fn test_execute_unset_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Unset should succeed even when no mode is set
        execute_unset(project_dir).unwrap();
        execute_unset(project_dir).unwrap();

        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.mode.is_none());
    }

    #[test]
    fn test_execute_delete_deletes_mode_ref() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a mode
        execute_create("claude").unwrap();

        // Verify the mode exists
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        assert!(repo.find_reference("refs/jin/layers/mode/claude").is_ok());

        // Delete the mode
        execute_delete(project_dir, "claude").unwrap();

        // Verify the mode is gone
        assert!(repo.find_reference("refs/jin/layers/mode/claude").is_err());
    }

    #[test]
    fn test_execute_delete_fails_if_mode_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Try to delete a non-existent mode
        let result = execute_delete(project_dir, "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ModeNotFound { .. })));
    }

    #[test]
    fn test_execute_delete_fails_if_mode_is_active() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and use a mode
        execute_create("claude").unwrap();
        execute_use(project_dir, "claude").unwrap();

        // Try to delete while active
        let result = execute_delete(project_dir, "claude");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::Message(_))));
    }

    #[test]
    fn test_execute_show_displays_active_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and use a mode
        execute_create("claude").unwrap();
        execute_use(project_dir, "claude").unwrap();

        // Show should succeed
        let result = execute_show(project_dir);
        assert!(result.is_ok());

        // Verify mode is still set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.mode, Some("claude".to_string()));
    }

    #[test]
    fn test_execute_show_with_no_active_mode() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Show should succeed even with no active mode
        let result = execute_show(project_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_with_no_modes() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // List should succeed with no modes
        let result = execute_list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_with_multiple_modes() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create multiple modes
        execute_create("claude").unwrap();
        execute_create("cursor").unwrap();
        execute_create("zed").unwrap();

        // List should succeed
        let result = execute_list();
        assert!(result.is_ok());

        // Verify modes exist
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        assert!(repo.find_reference("refs/jin/layers/mode/claude").is_ok());
        assert!(repo.find_reference("refs/jin/layers/mode/cursor").is_ok());
        assert!(repo.find_reference("refs/jin/layers/mode/zed").is_ok());
    }

    #[test]
    fn test_is_valid_mode_name() {
        // Valid names
        assert!(is_valid_mode_name("claude"));
        assert!(is_valid_mode_name("cursor"));
        assert!(is_valid_mode_name("zed-mode"));
        assert!(is_valid_mode_name("mode_123"));
        assert!(is_valid_mode_name("AI-Tool_2024"));

        // Invalid names
        assert!(!is_valid_mode_name(""));
        assert!(!is_valid_mode_name("invalid name"));
        assert!(!is_valid_mode_name("invalid!"));
        assert!(!is_valid_mode_name("invalid@mode"));
        assert!(!is_valid_mode_name("invalid.mode"));
    }

    #[test]
    fn test_execute_with_scope_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set a scope
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        // Create and use a mode
        execute_create("claude").unwrap();
        execute_use(project_dir, "claude").unwrap();

        // Verify both mode and scope are set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.mode, Some("claude".to_string()));
        assert_eq!(context.scope, Some("language:rust".to_string()));

        // Unset mode
        execute_unset(project_dir).unwrap();

        // Verify scope is preserved
        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.mode.is_none());
        assert_eq!(context.scope, Some("language:rust".to_string()));
    }

    #[test]
    fn test_execute_command_routing() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Test Create variant
        let cmd = ModeCommand::Create {
            name: "test-mode".to_string(),
        };
        assert!(execute(&cmd).is_ok());

        // Test Use variant
        let cmd = ModeCommand::Use {
            name: "test-mode".to_string(),
        };
        assert!(execute(&cmd).is_ok());

        // Test Show variant
        let cmd = ModeCommand::Show;
        assert!(execute(&cmd).is_ok());

        // Test Unset variant
        let cmd = ModeCommand::Unset;
        assert!(execute(&cmd).is_ok());

        // Test Delete variant
        let cmd = ModeCommand::Delete {
            name: "test-mode".to_string(),
        };
        assert!(execute(&cmd).is_ok());
    }
}
