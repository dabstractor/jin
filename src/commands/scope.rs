//! Implementation of `jin scope` subcommands

use crate::cli::ScopeAction;
use crate::core::{JinError, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps, RefOps};
use crate::staging::metadata::WorkspaceMetadata;

/// Execute a scope subcommand
pub fn execute(action: ScopeAction) -> Result<()> {
    match action {
        ScopeAction::Create { name, mode } => create(&name, mode.as_deref()),
        ScopeAction::Use { name } => use_scope(&name),
        ScopeAction::List => list(),
        ScopeAction::Delete { name } => delete(&name),
        ScopeAction::Show => show(),
        ScopeAction::Unset => unset(),
    }
}

/// Validate scope name
///
/// Scope names must be:
/// - Non-empty
/// - Alphanumeric, underscores, and colons only
/// - Not reserved names
fn validate_scope_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(JinError::Other("Scope name cannot be empty".to_string()));
    }

    // Check for valid characters (alphanumeric, underscore, and colon only)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
    {
        return Err(JinError::Other(format!(
            "Invalid scope name '{}'. Use alphanumeric characters, underscores, and colons only.",
            name
        )));
    }

    // Check for reserved names
    let reserved = ["default", "global", "base"];
    if reserved.contains(&name) {
        return Err(JinError::Other(format!(
            "Scope name '{}' is reserved.",
            name
        )));
    }

    Ok(())
}

/// Validate mode name (simpler version for scope creation)
fn validate_mode_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(JinError::Other("Mode name cannot be empty".to_string()));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(JinError::Other(format!(
            "Invalid mode name '{}'. Use alphanumeric characters and underscores only.",
            name
        )));
    }

    Ok(())
}

/// Create a new scope
fn create(name: &str, mode: Option<&str>) -> Result<()> {
    // Validate scope name
    validate_scope_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Convert scope name to ref-safe format (replace colons with slashes)
    let ref_safe_name = name.replace(':', "/");

    // Determine ref path based on mode parameter
    let ref_path = if let Some(mode_name) = mode {
        // Mode-bound scope
        validate_mode_name(mode_name)?;

        // Check if mode exists (using _mode suffix)
        let mode_ref = format!("refs/jin/modes/{}/_mode", mode_name);
        if !repo.ref_exists(&mode_ref) {
            return Err(JinError::NotFound(format!(
                "Mode '{}' not found. Create it with: jin mode create {}",
                mode_name, mode_name
            )));
        }

        format!("refs/jin/modes/{}/scopes/{}", mode_name, ref_safe_name)
    } else {
        // Untethered scope
        format!("refs/jin/scopes/{}", ref_safe_name)
    };

    // Check if scope already exists
    if repo.ref_exists(&ref_path) {
        return Err(JinError::AlreadyExists(format!(
            "Scope '{}' already exists",
            name
        )));
    }

    // Create empty tree for initial commit
    let empty_tree = repo.create_tree(&[])?;

    // Create initial commit
    let commit_message = if let Some(mode_name) = mode {
        format!("Initialize scope: {} (mode: {})", name, mode_name)
    } else {
        format!("Initialize scope: {}", name)
    };

    let commit_oid = repo.create_commit(None, &commit_message, empty_tree, &[])?;

    // Set Git ref
    let reflog_message = if let Some(mode_name) = mode {
        format!("create scope {} for mode {}", name, mode_name)
    } else {
        format!("create scope {}", name)
    };

    repo.set_ref(&ref_path, commit_oid, &reflog_message)?;

    // Print success message
    if let Some(mode_name) = mode {
        println!("Created scope '{}' bound to mode '{}'", name, mode_name);
    } else {
        println!("Created scope '{}' (untethered)", name);
    }
    println!("Activate with: jin scope use {}", name);

    Ok(())
}

/// Activate a scope
fn use_scope(name: &str) -> Result<()> {
    // Validate scope name
    validate_scope_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Convert scope name to ref-safe format (replace colons with slashes)
    let ref_safe_name = name.replace(':', "/");

    // Check if scope exists (check both mode-bound and untethered)
    let untethered_ref = format!("refs/jin/scopes/{}", ref_safe_name);
    let mode_bound_pattern = format!("refs/jin/modes/*/scopes/{}", ref_safe_name);

    let exists = repo.ref_exists(&untethered_ref)
        || !repo
            .list_refs(&mode_bound_pattern)
            .unwrap_or_default()
            .is_empty();

    if !exists {
        return Err(JinError::NotFound(format!(
            "Scope '{}' not found. Create it with: jin scope create {}",
            name, name
        )));
    }

    // Load project context
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Update scope
    context.scope = Some(name.to_string());

    // Save context
    context.save()?;

    // Load workspace metadata (may not exist yet)
    let metadata = match WorkspaceMetadata::load() {
        Ok(meta) => Some(meta),
        Err(JinError::NotFound(_)) => None, // Fresh workspace - no metadata yet
        Err(e) => return Err(e),            // Other errors should propagate
    };

    // Extract scope from metadata if present
    if let Some(meta) = &metadata {
        // Find scope layer in applied_layers (format: "scope/{name}")
        // IMPORTANT: Exclude mode+scope layers like "mode/production/scope/backend"
        let metadata_scope = meta
            .applied_layers
            .iter()
            .find(|layer| layer.starts_with("scope/") && !layer.starts_with("mode/"))
            .and_then(|layer| layer.strip_prefix("scope/"))
            .and_then(|s| s.split('/').next());

        // Compare with new scope
        if let Some(old_scope) = metadata_scope {
            if old_scope != name {
                // Scopes differ - clear metadata to prevent detached state
                let metadata_path = WorkspaceMetadata::default_path();
                if metadata_path.exists() {
                    std::fs::remove_file(&metadata_path)?;
                    println!(
                        "Cleared workspace metadata (scope changed from '{}' to '{}').",
                        old_scope, name
                    );
                    println!("Run 'jin apply' to apply new scope configuration.");
                }
            }
        } else {
            // No scope layer in metadata (only global or mode layers)
            // Clear metadata since we're now activating a scope
            let metadata_path = WorkspaceMetadata::default_path();
            if metadata_path.exists() {
                std::fs::remove_file(&metadata_path)?;
                println!("Cleared workspace metadata (activating scope '{}').", name);
                println!("Run 'jin apply' to apply new scope configuration.");
            }
        }
    }

    println!("Activated scope '{}'", name);
    println!("Stage files with: jin add --scope={}", name);

    Ok(())
}

/// List all scopes
pub fn list() -> Result<()> {
    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Load project context to identify active scope
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Find untethered scopes
    let untethered_refs = repo.list_refs("refs/jin/scopes/*").unwrap_or_default();

    // Find mode-bound scopes
    let mode_bound_refs = repo
        .list_refs("refs/jin/modes/*/scopes/*")
        .unwrap_or_default();

    if untethered_refs.is_empty() && mode_bound_refs.is_empty() {
        println!("No scopes found.");
        println!("Create one with: jin scope create <name>");
        return Ok(());
    }

    println!("Available scopes:");

    // Display untethered scopes
    for ref_path in untethered_refs {
        let ref_safe_name = ref_path
            .strip_prefix("refs/jin/scopes/")
            .unwrap_or(&ref_path);
        // Convert back from ref-safe format (slashes to colons)
        let display_name = ref_safe_name.replace('/', ":");

        if Some(display_name.as_str()) == context.scope.as_deref() {
            println!("  * {} (untethered) [active]", display_name);
        } else {
            println!("    {} (untethered)", display_name);
        }
    }

    // Display mode-bound scopes
    for ref_path in mode_bound_refs {
        // Parse: refs/jin/modes/{mode}/scopes/{scope}
        if let Some(rest) = ref_path.strip_prefix("refs/jin/modes/") {
            if let Some(mode_end) = rest.find("/scopes/") {
                let mode_name = &rest[..mode_end];
                let ref_safe_scope = &rest[mode_end + 8..]; // Skip "/scopes/"
                                                            // Convert back from ref-safe format (slashes to colons)
                let display_name = ref_safe_scope.replace('/', ":");

                if Some(display_name.as_str()) == context.scope.as_deref() {
                    println!("  * {} (mode: {}) [active]", display_name, mode_name);
                } else {
                    println!("    {} (mode: {})", display_name, mode_name);
                }
            }
        }
    }

    Ok(())
}

/// Delete a scope
fn delete(name: &str) -> Result<()> {
    // Validate scope name
    validate_scope_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Convert scope name to ref-safe format (replace colons with slashes)
    let ref_safe_name = name.replace(':', "/");

    // Find all refs for this scope (both mode-bound and untethered)
    let untethered_ref = format!("refs/jin/scopes/{}", ref_safe_name);
    let mode_bound_pattern = format!("refs/jin/modes/*/scopes/{}", ref_safe_name);

    let mut refs_to_delete = Vec::new();

    // Check untethered
    if repo.ref_exists(&untethered_ref) {
        refs_to_delete.push(untethered_ref);
    }

    // Check mode-bound
    if let Ok(mode_bound_refs) = repo.list_refs(&mode_bound_pattern) {
        refs_to_delete.extend(mode_bound_refs);
    }

    if refs_to_delete.is_empty() {
        return Err(JinError::NotFound(format!("Scope '{}' not found", name)));
    }

    // Load project context to check if active
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // If scope is active, unset it first
    if Some(name) == context.scope.as_deref() {
        println!("Scope '{}' is currently active. Deactivating...", name);
        context.scope = None;
        context.save()?;
    }

    // Delete all refs
    for ref_path in &refs_to_delete {
        repo.delete_ref(ref_path)?;
    }

    // Delete associated layer refs (may not exist if no files committed)
    // Silently ignore errors
    //
    // Note: Layer refs use the original scope name (with colons), not ref_safe_name
    // We need to try both the exact match and patterns that might match nested refs
    let layer_patterns = [
        // Direct match for scope layer refs
        format!("refs/jin/layers/scope/{}", name),
        format!("refs/jin/layers/mode/*/scope/{}", name),
        // Also try ref-safe version for layer refs that might have been created differently
        format!("refs/jin/layers/scope/{}", ref_safe_name),
        format!("refs/jin/layers/mode/*/scope/{}", ref_safe_name),
    ];

    for pattern in &layer_patterns {
        let _ = repo.delete_ref(pattern);

        if let Ok(refs) = repo.list_refs(pattern) {
            for ref_to_delete in refs {
                let _ = repo.delete_ref(&ref_to_delete);
            }
        }
    }

    println!("Deleted scope '{}'", name);

    Ok(())
}

/// Show currently active scope
fn show() -> Result<()> {
    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    match context.scope {
        Some(scope) => println!("Active scope: {}", scope),
        None => println!("No active scope"),
    }

    Ok(())
}

/// Unset (deactivate) current scope
fn unset() -> Result<()> {
    // Load project context
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Check if scope is set
    if context.scope.is_none() {
        println!("No active scope to unset");
        return Ok(());
    }

    // Unset scope
    context.scope = None;

    // Save context
    context.save()?;

    println!("Deactivated scope");
    println!("Scope layers no longer available for staging");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        // CRITICAL: Create JIN_DIR and initialize the Jin repository
        std::fs::create_dir_all(&jin_dir).unwrap();
        let _ = JinRepo::open_or_create();

        // Change to temp directory for project context
        // Use .ok() because current_dir() can fail if previous test left us in deleted dir
        let _ = std::env::set_current_dir(temp.path());

        // Initialize .jin directory and context
        std::fs::create_dir_all(temp.path().join(".jin")).unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        temp
    }

    fn create_test_mode(name: &str) {
        let repo = JinRepo::open_or_create().unwrap();
        let empty_tree = repo.create_tree(&[]).unwrap();
        let commit_oid = repo
            .create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])
            .unwrap();
        // Use _mode suffix to make the mode name a directory (allows nested scopes)
        repo.set_ref(
            &format!("refs/jin/modes/{}/_mode", name),
            commit_oid,
            &format!("create mode {}", name),
        )
        .unwrap();
    }

    #[test]
    fn test_validate_scope_name_valid() {
        assert!(validate_scope_name("testing").is_ok());
        assert!(validate_scope_name("test_scope").is_ok());
        assert!(validate_scope_name("language:javascript").is_ok());
        assert!(validate_scope_name("env:dev:debug").is_ok());
    }

    #[test]
    fn test_validate_scope_name_invalid_chars() {
        assert!(validate_scope_name("scope-name").is_err());
        assert!(validate_scope_name("scope/name").is_err());
        assert!(validate_scope_name("scope name").is_err());
    }

    #[test]
    fn test_validate_scope_name_empty() {
        assert!(validate_scope_name("").is_err());
    }

    #[test]
    fn test_validate_scope_name_reserved() {
        assert!(validate_scope_name("default").is_err());
        assert!(validate_scope_name("global").is_err());
        assert!(validate_scope_name("base").is_err());
    }

    #[test]
    #[serial]
    fn test_create_untethered_scope() {
        let _temp = setup_test_env();
        let result = create("testscope", None);
        assert!(result.is_ok());

        // Verify ref was created
        let repo = JinRepo::open_or_create().unwrap();
        assert!(repo.ref_exists("refs/jin/scopes/testscope"));
    }

    #[test]
    #[serial]
    fn test_create_mode_bound_scope() {
        let _temp = setup_test_env();
        create_test_mode("testmode");

        let result = create("testscope", Some("testmode"));
        assert!(result.is_ok());

        // Verify ref was created
        let repo = JinRepo::open_or_create().unwrap();
        assert!(repo.ref_exists("refs/jin/modes/testmode/scopes/testscope"));
    }

    #[test]
    #[serial]
    fn test_create_scope_with_colon() {
        let _temp = setup_test_env();
        let result = create("language:javascript", None);
        assert!(result.is_ok());

        // Colons are replaced with slashes in ref names
        let repo = JinRepo::open_or_create().unwrap();
        assert!(repo.ref_exists("refs/jin/scopes/language/javascript"));
    }

    #[test]
    #[serial]
    fn test_create_scope_nonexistent_mode() {
        let _temp = setup_test_env();
        let result = create("testscope", Some("nonexistent"));
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    #[serial]
    fn test_create_scope_duplicate() {
        let _temp = setup_test_env();
        create("testscope", None).unwrap();

        // Try to create again
        let result = create("testscope", None);
        assert!(matches!(result, Err(JinError::AlreadyExists(_))));
    }

    #[test]
    #[serial]
    fn test_use_scope() {
        let _temp = setup_test_env();
        create("testscope", None).unwrap();

        let result = use_scope("testscope");
        assert!(result.is_ok());

        // Verify context was updated
        let context = ProjectContext::load().unwrap();
        assert_eq!(context.scope, Some("testscope".to_string()));
    }

    #[test]
    #[serial]
    fn test_use_scope_nonexistent() {
        let _temp = setup_test_env();
        let result = use_scope("nonexistent");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    #[serial]
    fn test_list_empty() {
        let _temp = setup_test_env();
        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_list_with_scopes() {
        let _temp = setup_test_env();
        create("scope1", None).unwrap();
        create("scope2", None).unwrap();
        create_test_mode("testmode");
        create("scope3", Some("testmode")).unwrap();
        use_scope("scope1").unwrap();

        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_show_no_scope() {
        let _temp = setup_test_env();
        let result = show();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_show_with_scope() {
        let _temp = setup_test_env();
        create("testscope", None).unwrap();
        use_scope("testscope").unwrap();

        let result = show();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_unset() {
        let _temp = setup_test_env();
        create("testscope", None).unwrap();
        use_scope("testscope").unwrap();

        let result = unset();
        assert!(result.is_ok());

        // Verify scope was unset
        let context = ProjectContext::load().unwrap();
        assert_eq!(context.scope, None);
    }

    #[test]
    #[serial]
    fn test_unset_no_scope() {
        let _temp = setup_test_env();
        let result = unset();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_delete_untethered_scope() {
        let _temp = setup_test_env();
        create("testscope", None).unwrap();

        let result = delete("testscope");
        assert!(result.is_ok());

        // Verify ref was deleted
        let repo = JinRepo::open_or_create().unwrap();
        assert!(!repo.ref_exists("refs/jin/scopes/testscope"));
    }

    #[test]
    #[serial]
    fn test_delete_mode_bound_scope() {
        let _temp = setup_test_env();
        create_test_mode("testmode");
        create("testscope", Some("testmode")).unwrap();

        let result = delete("testscope");
        assert!(result.is_ok());

        // Verify ref was deleted
        let repo = JinRepo::open_or_create().unwrap();
        assert!(!repo.ref_exists("refs/jin/modes/testmode/scopes/testscope"));
    }

    #[test]
    #[serial]
    fn test_delete_active_scope() {
        let _temp = setup_test_env();
        create("testscope", None).unwrap();
        use_scope("testscope").unwrap();

        let result = delete("testscope");
        assert!(result.is_ok());

        // Verify scope was unset
        let context = ProjectContext::load().unwrap();
        assert_eq!(context.scope, None);
    }

    #[test]
    #[serial]
    fn test_delete_nonexistent() {
        let _temp = setup_test_env();
        let result = delete("nonexistent");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }
}
