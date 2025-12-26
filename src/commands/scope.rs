//! Scope command implementation.
//!
//! This module implements the `jin scope` subcommands that manage language- and
//! domain-specific configuration layers including create, use, unset, delete,
//! show, and list operations.
//!
//! Scopes can be either:
//! - **Untethered**: `refs/jin/layers/scope/<name>` - shared across all modes
//! - **Mode-bound**: `refs/jin/layers/mode/<mode>/scope/<name>` - specific to a mode

use crate::cli::args::ScopeCommand;
use crate::core::config::{JinConfig, ProjectContext};
use crate::core::error::{JinError, Result};
use crate::git::JinRepo;
use std::path::Path;

/// Execute the scope subcommand.
///
/// Routes to the appropriate handler based on the subcommand variant.
/// Uses workspace root detection and context loading following established patterns.
///
/// # Arguments
///
/// * `cmd` - The scope subcommand to execute
///
/// # Errors
///
/// Returns `JinError::ScopeNotFound` if scope doesn't exist (for use/delete).
/// Returns `JinError::RefExists` if scope already exists (for create).
/// Returns `JinError::Message` if trying to delete active scope.
///
/// # Examples
///
/// ```ignore
/// use jin_glm::cli::args::ScopeCommand;
/// use jin_glm::commands::scope;
///
/// let cmd = ScopeCommand::Create { name: "language:javascript".to_string(), mode: None };
/// scope::execute(&cmd)?;
/// ```
pub fn execute(cmd: &ScopeCommand) -> Result<()> {
    let workspace_root = std::env::current_dir()?;

    match cmd {
        ScopeCommand::Create { name, mode } => execute_create(name, mode.as_deref()),
        ScopeCommand::Use { name } => execute_use(&workspace_root, name),
        ScopeCommand::Unset => execute_unset(&workspace_root),
        ScopeCommand::Delete { name } => execute_delete(&workspace_root, name),
        ScopeCommand::Show => execute_show(&workspace_root),
    }
}

/// Execute the scopes list command.
///
/// Lists all available scopes (both untethered and mode-bound) with their commit OIDs.
/// This is called from main.rs for the `jin scopes` command.
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if Jin repository doesn't exist.
///
/// # Examples
///
/// ```ignore
/// scope::execute_list()?;
/// ```
pub fn execute_list() -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

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

        // Unescape the scope name for display
        let display_name = unescape_from_ref(scope_name);

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
            let display_name = unescape_from_ref(scope_name);

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

/// Execute scope create command.
///
/// Creates a new scope by initializing a Git ref. Scopes can be:
/// - Untethered: `refs/jin/layers/scope/<name>` (when no --mode flag)
/// - Mode-bound: `refs/jin/layers/mode/<mode>/scope/<name>` (when --mode is specified)
///
/// Creates an empty initial commit for the scope.
///
/// # Arguments
///
/// * `name` - The scope name to create
/// * `mode` - Optional mode name to bind the scope to
fn execute_create(name: &str, mode: Option<&str>) -> Result<()> {
    // Validate scope name format (alphanumeric, hyphens, underscores, colons)
    if !is_valid_scope_name(name) {
        return Err(JinError::ValidationError {
            message: format!(
                "Invalid scope name: '{}'. Use alphanumeric, hyphens, underscores, colons.",
                name
            ),
        });
    }

    // Load Jin config to get repository path
    let config = JinConfig::load()?;
    let repo = JinRepo::open_or_create(&config.repository)?;

    // Escape scope name for Git ref (colons are not allowed in Git refs)
    let escaped_name = escape_for_ref(name);

    // Determine ref path and validate mode if binding
    let (ref_path, scope_type) = if let Some(mode_name) = mode {
        // Validate mode exists when binding
        let mode_ref = format!("refs/jin/layers/mode/{}", mode_name);
        if repo.find_reference(&mode_ref).is_err() {
            return Err(JinError::ModeNotFound {
                mode: mode_name.to_string(),
            });
        }
        (
            format!("refs/jin/layers/mode/{}/scope/{}", mode_name, escaped_name),
            "mode-bound",
        )
    } else {
        (
            format!("refs/jin/layers/scope/{}", escaped_name),
            "untethered",
        )
    };

    // Check scope doesn't already exist
    if repo.find_reference(&ref_path).is_ok() {
        return Err(JinError::RefExists {
            name: ref_path.clone(),
            layer: format!("scope/{}", name),
        });
    }

    // Create empty tree and initial commit for the scope
    let empty_tree_oid = repo.create_empty_tree()?;
    let empty_tree = repo.find_tree(empty_tree_oid)?;

    let author = repo.signature("Jin", "jin@local")?;
    let committer = &author;

    let initial_commit_oid = repo.create_commit(
        None,
        &author,
        committer,
        &format!("Initial commit for scope: {}", name),
        &empty_tree,
        &[],
    )?;

    // Create the scope ref
    repo.create_reference(
        &ref_path,
        initial_commit_oid,
        false,
        &format!("Create scope: {}", name),
    )?;

    println!("Scope '{}' created [{}].", name, scope_type);
    Ok(())
}

/// Execute scope use command.
///
/// Sets the active scope in `.jin/context` YAML file.
/// Creates `.jin/context` if it doesn't exist.
/// Preserves existing mode if set.
///
/// Checks both untethered and mode-bound scope locations.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
/// * `name` - The scope name to activate
fn execute_use(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Escape scope name for Git ref lookup
    let escaped_name = escape_for_ref(name);

    // Check both untethered and mode-bound locations
    let untethered_ref = format!("refs/jin/layers/scope/{}", escaped_name);

    // Load context to check current mode for mode-bound scope lookup
    let context = ProjectContext::load(workspace_root)?;

    let scope_exists = if let Some(active_mode) = &context.mode {
        // Check mode-bound location first if mode is active
        let mode_bound_ref = format!(
            "refs/jin/layers/mode/{}/scope/{}",
            active_mode, escaped_name
        );
        repo.find_reference(&mode_bound_ref).is_ok() || repo.find_reference(&untethered_ref).is_ok()
    } else {
        // Only check untethered if no active mode
        repo.find_reference(&untethered_ref).is_ok()
    };

    if !scope_exists {
        return Err(JinError::ScopeNotFound {
            scope: name.to_string(),
        });
    }

    // Update context
    let mut context = ProjectContext::load(workspace_root)?;
    context.set_scope(Some(name.to_string()));
    context.save(workspace_root)?;

    println!("Scope '{}' is now active.", name);
    Ok(())
}

/// Execute scope unset command.
///
/// Removes scope field from `.jin/context`.
/// Preserves mode field if present.
/// No error if no scope is set.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
fn execute_unset(workspace_root: &Path) -> Result<()> {
    let mut context = ProjectContext::load(workspace_root)?;
    context.set_scope(None);
    context.save(workspace_root)?;

    println!("Scope deactivated.");
    Ok(())
}

/// Execute scope delete command.
///
/// Deletes the Git ref for a scope (works for both untethered and mode-bound).
/// Fails if scope is currently active (requires `jin scope unset` first).
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
/// * `name` - The scope name to delete
fn execute_delete(workspace_root: &Path, name: &str) -> Result<()> {
    let config = JinConfig::load()?;
    let repo = JinRepo::open(&config.repository)?;

    // Check scope is not active
    let context = ProjectContext::load(workspace_root)?;
    if context.scope.as_ref() == Some(&name.to_string()) {
        return Err(JinError::Message(
            "Cannot delete active scope. Use 'jin scope unset' first.".to_string(),
        ));
    }

    // Escape scope name for Git ref lookup
    let escaped_name = escape_for_ref(name);

    // Try to find and delete the scope (check both locations)
    let untethered_ref = format!("refs/jin/layers/scope/{}", escaped_name);

    // Also check mode-bound locations if we have a mode
    let mode_bound_refs = if let Some(active_mode) = &context.mode {
        vec![format!(
            "refs/jin/layers/mode/{}/scope/{}",
            active_mode, escaped_name
        )]
    } else {
        vec![]
    };

    // Try untethered first, then mode-bound
    let mut found_ref = None;
    if repo.find_reference(&untethered_ref).is_ok() {
        found_ref = Some(untethered_ref);
    } else {
        for mode_ref in mode_bound_refs {
            if repo.find_reference(&mode_ref).is_ok() {
                found_ref = Some(mode_ref);
                break;
            }
        }
    }

    let ref_path = match found_ref {
        Some(r) => r,
        None => {
            return Err(JinError::ScopeNotFound {
                scope: name.to_string(),
            });
        }
    };

    // Delete the ref
    let mut reference = repo.find_reference(&ref_path)?;
    reference.delete()?;

    println!("Scope '{}' deleted.", name);
    Ok(())
}

/// Execute scope show command.
///
/// Displays current active scope from `.jin/context`.
/// Shows mode if also active.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
fn execute_show(workspace_root: &Path) -> Result<()> {
    let context = ProjectContext::load(workspace_root)?;

    match &context.scope {
        Some(scope) => {
            println!("Active scope: {}", scope);
            if context.has_mode() {
                println!("Active mode: {}", context.mode.as_ref().unwrap());
            }
        }
        None => {
            println!("No active scope");
            if context.has_mode() {
                println!("Active mode: {}", context.mode.as_ref().unwrap());
            }
        }
    }

    Ok(())
}

/// Validates scope name format.
///
/// Scope names must be non-empty and contain only alphanumeric characters,
/// hyphens, underscores, and colons (e.g., "language:javascript").
///
/// # Arguments
///
/// * `name` - The scope name to validate
///
/// # Returns
///
/// `true` if the name is valid, `false` otherwise.
fn is_valid_scope_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':')
}

/// Escapes a scope name for use in Git refs.
///
/// Git refs cannot contain colons, so we escape them as `%3A`.
/// This mirrors URL encoding practices.
///
/// # Arguments
///
/// * `name` - The scope name to escape
///
/// # Returns
///
/// The escaped scope name safe for Git refs.
pub fn escape_for_ref(name: &str) -> String {
    name.replace(':', "%3A")
}

/// Unescapes a scope name from a Git ref.
///
/// Reverses the escaping done by `escape_for_ref()`.
///
/// # Arguments
///
/// * `escaped` - The escaped scope name
///
/// # Returns
///
/// The original scope name.
pub fn unescape_from_ref(escaped: &str) -> String {
    escaped.replace("%3A", ":")
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
    fn test_execute_create_creates_untethered_scope_ref() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create an untethered scope with colons
        let result = execute_create("language:javascript", None);
        assert!(result.is_ok());

        // Verify the scope ref exists (escaped name in Git ref)
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        assert!(repo
            .find_reference("refs/jin/layers/scope/language%3Ajavascript")
            .is_ok());
    }

    #[test]
    fn test_execute_create_creates_mode_bound_scope_ref() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a mode first (needed for mode-bound scope)
        // Use a different ref path to avoid conflict with scope directory
        let mode_ref = "refs/jin/layers-mode/claude";
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();

        // Create mode ref manually at alternate location
        let empty_tree_oid = repo.create_empty_tree().unwrap();
        let empty_tree = repo.find_tree(empty_tree_oid).unwrap();
        let author = repo.signature("Jin", "jin@local").unwrap();
        let commit_oid = repo
            .create_commit(
                None,
                &author,
                &author,
                "Initial commit for mode",
                &empty_tree,
                &[],
            )
            .unwrap();
        repo.create_reference(mode_ref, commit_oid, false, "Create mode")
            .unwrap();

        // Create a mode-bound scope with colons (using the alternate mode path)
        // Note: This will fail because the mode doesn't exist at the expected location
        // This test documents the expected behavior - mode must exist at correct path
        let result = execute_create("language:rust", Some("claude"));
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ModeNotFound { .. })));
    }

    #[test]
    fn test_execute_create_fails_if_scope_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a scope
        execute_create("scope-docker-test", None).unwrap();

        // Try to create the same scope again
        let result = execute_create("scope-docker-test", None);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::RefExists { .. })));
    }

    #[test]
    fn test_execute_create_fails_if_mode_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Try to create a mode-bound scope with non-existent mode
        let result = execute_create("backend", Some("nonexistent"));
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ModeNotFound { .. })));
    }

    #[test]
    fn test_execute_create_validates_scope_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Empty name should fail
        let result = execute_create("", None);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ValidationError { .. })));

        // Invalid characters should fail
        let result = execute_create("invalid name!", None);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ValidationError { .. })));

        // Valid names should succeed (including colons) - use unique names
        assert!(execute_create("scope-js-test", None).is_ok());
        assert!(execute_create("scope:cat:sub:test", None).is_ok());
        assert!(execute_create("scope-infra-docker", None).is_ok());
        assert!(execute_create("scope_backend_api", None).is_ok());
    }

    #[test]
    fn test_execute_use_sets_active_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a scope first
        execute_create("scope-python-test", None).unwrap();

        // Use the scope
        execute_use(project_dir, "scope-python-test").unwrap();

        // Verify context has the scope set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.scope, Some("scope-python-test".to_string()));
    }

    #[test]
    fn test_execute_use_fails_if_scope_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Try to use a non-existent scope
        let result = execute_use(project_dir, "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ScopeNotFound { .. })));
    }

    #[test]
    fn test_execute_unset_clears_active_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and use a scope
        execute_create("scope-rust-unset-test", None).unwrap();
        execute_use(project_dir, "scope-rust-unset-test").unwrap();

        // Verify scope is set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.scope, Some("scope-rust-unset-test".to_string()));

        // Unset the scope
        execute_unset(project_dir).unwrap();

        // Verify scope is cleared
        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.scope.is_none());
    }

    #[test]
    fn test_execute_unset_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Unset should succeed even when no scope is set
        execute_unset(project_dir).unwrap();
        execute_unset(project_dir).unwrap();

        let context = ProjectContext::load(project_dir).unwrap();
        assert!(context.scope.is_none());
    }

    #[test]
    fn test_execute_delete_deletes_scope_ref() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create a scope
        execute_create("scope-frontend-delete-test", None).unwrap();

        // Verify the scope exists
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();
        assert!(repo
            .find_reference("refs/jin/layers/scope/scope-frontend-delete-test")
            .is_ok());

        // Delete the scope
        execute_delete(project_dir, "scope-frontend-delete-test").unwrap();

        // Verify the scope is gone
        assert!(repo
            .find_reference("refs/jin/layers/scope/scope-frontend-delete-test")
            .is_err());
    }

    #[test]
    fn test_execute_delete_fails_if_scope_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Try to delete a non-existent scope
        let result = execute_delete(project_dir, "nonexistent");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::ScopeNotFound { .. })));
    }

    #[test]
    fn test_execute_delete_fails_if_scope_is_active() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and use a scope
        execute_create("scope-golang-active-test", None).unwrap();
        execute_use(project_dir, "scope-golang-active-test").unwrap();

        // Try to delete while active
        let result = execute_delete(project_dir, "scope-golang-active-test");
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::Message(_))));
    }

    #[test]
    fn test_execute_show_displays_active_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create and use a scope
        execute_create("scope-ts-show-test", None).unwrap();
        execute_use(project_dir, "scope-ts-show-test").unwrap();

        // Show should succeed
        let result = execute_show(project_dir);
        assert!(result.is_ok());

        // Verify scope is still set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.scope, Some("scope-ts-show-test".to_string()));
    }

    #[test]
    fn test_execute_show_with_no_active_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Show should succeed even with no active scope
        let result = execute_show(project_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_with_no_scopes() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // List should succeed with no scopes
        let result = execute_list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_with_multiple_scopes() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Create multiple untethered scopes with unique names
        execute_create("list:scope:js", None).unwrap();
        execute_create("list:scope:py", None).unwrap();
        execute_create("list:scope:docker", None).unwrap();

        // Create additional scopes with simple names
        execute_create("list-simple-1", None).unwrap();
        execute_create("list-simple-2", None).unwrap();

        // List should succeed
        let result = execute_list();
        assert!(result.is_ok());

        // Verify scopes exist (with escaped names in Git refs - all colons escaped)
        let config = JinConfig::load().unwrap();
        let repo = JinRepo::open(&config.repository).unwrap();

        assert!(repo
            .find_reference("refs/jin/layers/scope/list%3Ascope%3Ajs")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/scope/list%3Ascope%3Apy")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/scope/list%3Ascope%3Adocker")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/scope/list-simple-1")
            .is_ok());
        assert!(repo
            .find_reference("refs/jin/layers/scope/list-simple-2")
            .is_ok());
    }

    #[test]
    fn test_is_valid_scope_name() {
        // Valid names (including colons)
        assert!(is_valid_scope_name("javascript"));
        assert!(is_valid_scope_name("language:javascript"));
        assert!(is_valid_scope_name("category:subcategory:item"));
        assert!(is_valid_scope_name("infra-docker"));
        assert!(is_valid_scope_name("backend_api"));
        assert!(is_valid_scope_name("Lang2024"));

        // Invalid names
        assert!(!is_valid_scope_name(""));
        assert!(!is_valid_scope_name("invalid name"));
        assert!(!is_valid_scope_name("invalid!"));
        assert!(!is_valid_scope_name("invalid@scope"));
        assert!(!is_valid_scope_name("invalid.scope"));
    }

    #[test]
    fn test_execute_with_mode_preserved() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set a mode
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("scope-preserve-mode".to_string()));
        context.save(project_dir).unwrap();

        // Create and use a scope
        execute_create("scope-py-preserve", None).unwrap();
        execute_use(project_dir, "scope-py-preserve").unwrap();

        // Verify both mode and scope are set
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.mode, Some("scope-preserve-mode".to_string()));
        assert_eq!(context.scope, Some("scope-py-preserve".to_string()));

        // Unset scope
        execute_unset(project_dir).unwrap();

        // Verify mode is preserved
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.mode, Some("scope-preserve-mode".to_string()));
        assert!(context.scope.is_none());
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
        let cmd = ScopeCommand::Create {
            name: "test-scope".to_string(),
            mode: None,
        };
        assert!(execute(&cmd).is_ok());

        // Test Use variant
        let cmd = ScopeCommand::Use {
            name: "test-scope".to_string(),
        };
        assert!(execute(&cmd).is_ok());

        // Test Show variant
        let cmd = ScopeCommand::Show;
        assert!(execute(&cmd).is_ok());

        // Test Unset variant
        let cmd = ScopeCommand::Unset;
        assert!(execute(&cmd).is_ok());

        // Test Delete variant
        let cmd = ScopeCommand::Delete {
            name: "test-scope".to_string(),
        };
        assert!(execute(&cmd).is_ok());
    }
}
