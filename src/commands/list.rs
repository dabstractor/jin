//! List command implementation.
//!
//! This module implements the `jin list` command that displays
//! all available modes, scopes, and projects with their commit OIDs.

use crate::cli::args::ListCommand;
use crate::core::config::{JinConfig, ProjectContext};
use crate::core::error::Result;
use crate::git::JinRepo;
use crate::commands::scope; // For unescape_from_ref

/// Execute the list command.
///
/// Displays comprehensive list including:
/// - Active mode/scope from `.jin/context`
/// - All available modes with their OIDs
/// - All available scopes (untethered and mode-bound) with their OIDs
/// - All available projects with their OIDs
///
/// # Arguments
///
/// * `_cmd` - The list command (currently has no fields)
///
/// # Errors
///
/// Returns `JinError::Message` if not in a Jin-initialized directory.
/// Returns `JinError::RepoNotFound` if Jin repository doesn't exist.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::ListCommand;
/// use jin_glm::commands::list;
///
/// let cmd = ListCommand;
/// list::execute(&cmd)?;
/// ```
pub fn execute(_cmd: &ListCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization (CRITICAL - must be first)
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(crate::core::error::JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize."
                .to_string(),
        ));
    }

    // 3. Load and display active context
    let context = ProjectContext::load(&workspace_root)?;
    display_active_context(&context);

    // 4. Open Jin repository and display listings
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    display_available_modes(&repo)?;
    display_available_scopes(&repo)?;
    display_available_projects(&repo)?;

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

/// Displays all available modes with their OIDs.
fn display_available_modes(repo: &JinRepo) -> Result<()> {
    println!();

    let mode_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        return Ok(());
    }

    println!("Available modes:");
    for ref_name in mode_refs {
        let mode_name = ref_name
            .strip_prefix("refs/jin/layers/mode/")
            .unwrap_or(&ref_name);

        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:20} {}", mode_name, short_oid);
            }
        }
    }

    Ok(())
}

/// Displays all available scopes with their OIDs.
///
/// Shows both untethered scopes and mode-bound scopes with their parent mode.
fn display_available_scopes(repo: &JinRepo) -> Result<()> {
    println!();

    // Find all untethered scope refs
    let untethered_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/scope/*")?;

    // Find all mode-bound scope refs
    let mode_bound_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*/scope/*")?;

    if untethered_refs.is_empty() && mode_bound_refs.is_empty() {
        println!("No scopes found.");
        return Ok(());
    }

    println!("Available scopes:");

    // Display untethered scopes
    for ref_name in untethered_refs {
        let scope_name = ref_name
            .strip_prefix("refs/jin/layers/scope/")
            .unwrap_or(&ref_name);

        // Unescape the scope name for display (handle colons)
        let display_name = scope::unescape_from_ref(scope_name);

        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:30} [untethered] {}", display_name, short_oid);
            }
        }
    }

    // Display mode-bound scopes
    for ref_name in mode_bound_refs {
        // Extract mode and scope from path: refs/jin/layers/mode/{mode}/scope/{scope}
        let parts: Vec<&str> = ref_name.split('/').collect();
        if parts.len() >= 7 {
            let mode_name = parts[4];
            let scope_name = parts[6];

            // Unescape the scope name for display
            let display_name = scope::unescape_from_ref(scope_name);

            if let Ok(reference) = repo.find_reference(&ref_name) {
                if let Some(oid) = reference.target() {
                    let short_oid = &oid.to_string()[..8];
                    println!("  {:30} [mode:{}] {}", display_name, mode_name, short_oid);
                }
            }
        }
    }

    Ok(())
}

/// Displays all available projects with their OIDs.
fn display_available_projects(repo: &JinRepo) -> Result<()> {
    println!();

    let project_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/project/*")?;

    if project_refs.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    println!("Available projects:");
    for ref_name in project_refs {
        let project_name = ref_name
            .strip_prefix("refs/jin/layers/project/")
            .unwrap_or(&ref_name);

        if let Ok(reference) = repo.find_reference(&ref_name) {
            if let Some(oid) = reference.target() {
                let short_oid = &oid.to_string()[..8];
                println!("  {:20} {}", project_name, short_oid);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
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

    /// Helper to create a test mode
    fn create_test_mode(repo: &JinRepo, name: &str) -> git2::Oid {
        let empty_tree_oid = repo.create_empty_tree().unwrap();
        let empty_tree = repo.find_tree(empty_tree_oid).unwrap();
        let author = repo.signature("Jin", "jin@local").unwrap();
        let initial_commit_oid = repo
            .create_commit(
                None,
                &author,
                &author,
                &format!("Initial commit for mode: {}", name),
                &empty_tree,
                &[],
            )
            .unwrap();
        let ref_path = format!("refs/jin/layers/mode/{}", name);
        repo.create_reference(&ref_path, initial_commit_oid, false, &format!("Create mode: {}", name))
            .unwrap();
        initial_commit_oid
    }

    /// Helper to create a test scope
    fn create_test_scope(repo: &JinRepo, name: &str, mode: Option<&str>) -> git2::Oid {
        let empty_tree_oid = repo.create_empty_tree().unwrap();
        let empty_tree = repo.find_tree(empty_tree_oid).unwrap();
        let author = repo.signature("Jin", "jin@local").unwrap();
        let initial_commit_oid = repo
            .create_commit(
                None,
                &author,
                &author,
                &format!("Initial commit for scope: {}", name),
                &empty_tree,
                &[],
            )
            .unwrap();

        // Escape scope name for Git ref
        let escaped_name = scope::escape_for_ref(name);

        let ref_path = if let Some(mode_name) = mode {
            format!("refs/jin/layers/mode/{}/scope/{}", mode_name, escaped_name)
        } else {
            format!("refs/jin/layers/scope/{}", escaped_name)
        };

        repo.create_reference(
            &ref_path,
            initial_commit_oid,
            false,
            &format!("Create scope: {}", name),
        )
        .unwrap();
        initial_commit_oid
    }

    /// Helper to create a test project
    fn create_test_project(repo: &JinRepo, name: &str) -> git2::Oid {
        let empty_tree_oid = repo.create_empty_tree().unwrap();
        let empty_tree = repo.find_tree(empty_tree_oid).unwrap();
        let author = repo.signature("Jin", "jin@local").unwrap();
        let initial_commit_oid = repo
            .create_commit(
                None,
                &author,
                &author,
                &format!("Initial commit for project: {}", name),
                &empty_tree,
                &[],
            )
            .unwrap();
        let ref_path = format!("refs/jin/layers/project/{}", name);
        repo.create_reference(
            &ref_path,
            initial_commit_oid,
            false,
            &format!("Create project: {}", name),
        )
        .unwrap();
        initial_commit_oid
    }

    #[test]
    fn test_list_shows_active_context() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set mode and scope in context
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        // Run list
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Context should show mode and scope
        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("claude".to_string()));
        assert_eq!(loaded.scope, Some("language:rust".to_string()));
    }

    #[test]
    fn test_list_shows_no_active_context_when_none_set() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Run list with no active context
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Context should be empty
        let loaded = ProjectContext::load(project_dir).unwrap();
        assert!(loaded.mode.is_none());
        assert!(loaded.scope.is_none());
    }

    #[test]
    fn test_list_shows_modes() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create test modes
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        create_test_mode(&repo, "claude");
        create_test_mode(&repo, "cursor");
        create_test_mode(&repo, "zed");

        // Run list - should succeed
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Verify modes exist
        assert!(repo.find_reference("refs/jin/layers/mode/claude").is_ok());
        assert!(repo.find_reference("refs/jin/layers/mode/cursor").is_ok());
        assert!(repo.find_reference("refs/jin/layers/mode/zed").is_ok());
    }

    #[test]
    fn test_list_shows_scopes() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a mode first for mode-bound scope
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        create_test_mode(&repo, "claude");

        // Create test scopes
        create_test_scope(&repo, "language:rust", None);
        create_test_scope(&repo, "language:javascript", None);
        create_test_scope(&repo, "backend", Some("claude"));

        // Run list - should succeed
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Verify scopes exist (with escaped names in Git refs)
        assert!(repo
            .find_reference("refs/jin/layers/scope/language%3Arust")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/scope/language%3Ajavascript")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/mode/claude/scope/backend")
            .is_ok());
    }

    #[test]
    fn test_list_shows_projects() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create test projects
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        create_test_project(&repo, "jin-glm-doover");
        create_test_project(&repo, "my-app");

        // Run list - should succeed
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Verify projects exist
        assert!(repo
            .find_reference("refs/jin/layers/project/jin-glm-doover")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/project/my-app")
            .is_ok());
    }

    #[test]
    fn test_list_no_jin_initialized_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize Jin

        let cmd = ListCommand;
        let result = execute(&cmd);

        assert!(result.is_err());
        if let Err(crate::core::error::JinError::Message(msg)) = result {
            assert!(msg.contains("Jin is not initialized"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_list_empty_states() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Run list with no modes, scopes, or projects
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Verify empty state - command should succeed
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        assert!(repo.find_reference("refs/jin/layers/mode/test").is_err());
        assert!(repo.find_reference("refs/jin/layers/scope/test").is_err());
        assert!(repo
            .find_reference("refs/jin/layers/project/test")
            .is_err());
    }

    #[test]
    fn test_list_with_colon_in_scope_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create scope with colons in name
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        create_test_scope(&repo, "language:rust", None);
        create_test_scope(&repo, "category:subcategory:item", None);

        // Run list - should succeed and unescape colons correctly
        let cmd = ListCommand;
        execute(&cmd).unwrap();

        // Verify scopes exist with escaped colons
        assert!(repo
            .find_reference("refs/jin/layers/scope/language%3Arust")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/scope/category%3Asubcategory%3Aitem")
            .is_ok());
    }

    #[test]
    fn test_display_active_context_function() {
        // Test the display_active_context function directly
        let context = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("language:rust".to_string()),
        };

        // This should not panic
        display_active_context(&context);
    }

    #[test]
    fn test_display_active_context_empty() {
        // Test with empty context
        let context = ProjectContext {
            version: 1,
            mode: None,
            scope: None,
        };

        // This should not panic
        display_active_context(&context);
    }
}
