//! Implementation of `jin mode` subcommands

use crate::cli::ModeAction;
use crate::core::{JinError, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps, RefOps};

/// Execute a mode subcommand
pub fn execute(action: ModeAction) -> Result<()> {
    match action {
        ModeAction::Create { name } => create(&name),
        ModeAction::Use { name } => use_mode(&name),
        ModeAction::List => list(),
        ModeAction::Delete { name } => delete(&name),
        ModeAction::Show => show(),
        ModeAction::Unset => unset(),
    }
}

/// Validate mode name
///
/// Mode names must be:
/// - Non-empty
/// - Alphanumeric and underscores only
/// - Not reserved names
fn validate_mode_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(JinError::Other("Mode name cannot be empty".to_string()));
    }

    // Check for valid characters (alphanumeric and underscore only)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(JinError::Other(format!(
            "Invalid mode name '{}'. Use alphanumeric characters and underscores only.",
            name
        )));
    }

    // Check for reserved names
    let reserved = ["default", "global", "base"];
    if reserved.contains(&name) {
        return Err(JinError::Other(format!(
            "Mode name '{}' is reserved.",
            name
        )));
    }

    Ok(())
}

/// Create a new mode
fn create(name: &str) -> Result<()> {
    // Validate mode name
    validate_mode_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Check if mode already exists
    let ref_path = format!("refs/jin/modes/{}", name);
    if repo.ref_exists(&ref_path) {
        return Err(JinError::AlreadyExists(format!(
            "Mode '{}' already exists",
            name
        )));
    }

    // Create empty tree for initial commit
    let empty_tree = repo.create_tree(&[])?;

    // Create initial commit
    let commit_oid =
        repo.create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])?;

    // Set Git ref
    repo.set_ref(&ref_path, commit_oid, &format!("create mode {}", name))?;

    println!("Created mode '{}'", name);
    println!("Activate with: jin mode use {}", name);

    Ok(())
}

/// Activate a mode
fn use_mode(name: &str) -> Result<()> {
    // Validate mode name
    validate_mode_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Check if mode exists
    let ref_path = format!("refs/jin/modes/{}", name);
    if !repo.ref_exists(&ref_path) {
        return Err(JinError::NotFound(format!(
            "Mode '{}' not found. Create it with: jin mode create {}",
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

    // Update mode
    context.mode = Some(name.to_string());

    // Save context
    context.save()?;

    println!("Activated mode '{}'", name);
    println!("Stage files with: jin add --mode");

    Ok(())
}

/// List all modes
fn list() -> Result<()> {
    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Load project context to identify active mode
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Find all mode refs
    let mode_refs = repo.list_refs("refs/jin/modes/*")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        println!("Create one with: jin mode create <name>");
        return Ok(());
    }

    println!("Available modes:");

    // Extract names and display with active indicator
    for ref_path in mode_refs {
        let name = ref_path
            .strip_prefix("refs/jin/modes/")
            .unwrap_or(&ref_path);

        if Some(name) == context.mode.as_deref() {
            println!("  * {} [active]", name);
        } else {
            println!("    {}", name);
        }
    }

    Ok(())
}

/// Delete a mode
fn delete(name: &str) -> Result<()> {
    // Validate mode name
    validate_mode_name(name)?;

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // Check if mode exists
    let ref_path = format!("refs/jin/modes/{}", name);
    if !repo.ref_exists(&ref_path) {
        return Err(JinError::NotFound(format!("Mode '{}' not found", name)));
    }

    // Load project context to check if active
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // If mode is active, unset it first
    if Some(name) == context.mode.as_deref() {
        println!("Mode '{}' is currently active. Deactivating...", name);
        context.mode = None;
        context.save()?;
    }

    // Delete main mode ref
    repo.delete_ref(&ref_path)?;

    // Delete associated layer refs (may not exist if no files committed)
    // Silently ignore errors as these refs may not exist yet
    let layer_patterns = [
        format!("refs/jin/layers/mode/{}", name),
        format!("refs/jin/modes/{}/scopes/*", name),
    ];

    for pattern in &layer_patterns {
        // Try to delete, ignore errors
        let _ = repo.delete_ref(pattern);

        // Also try to list and delete individual refs matching pattern
        if let Ok(refs) = repo.list_refs(pattern) {
            for ref_to_delete in refs {
                let _ = repo.delete_ref(&ref_to_delete);
            }
        }
    }

    println!("Deleted mode '{}'", name);

    Ok(())
}

/// Show currently active mode
fn show() -> Result<()> {
    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    match context.mode {
        Some(mode) => println!("Active mode: {}", mode),
        None => println!("No active mode"),
    }

    Ok(())
}

/// Unset (deactivate) current mode
fn unset() -> Result<()> {
    // Load project context
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Check if mode is set
    if context.mode.is_none() {
        println!("No active mode to unset");
        return Ok(());
    }

    // Unset mode
    context.mode = None;

    // Save context
    context.save()?;

    println!("Deactivated mode");
    println!("Mode layer no longer available for staging");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        // Change to temp directory for project context
        std::env::set_current_dir(temp.path()).unwrap();

        // Initialize .jin directory and context
        std::fs::create_dir(".jin").unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        temp
    }

    #[test]
    fn test_validate_mode_name_valid() {
        assert!(validate_mode_name("claude").is_ok());
        assert!(validate_mode_name("test_mode").is_ok());
        assert!(validate_mode_name("mode123").is_ok());
    }

    #[test]
    fn test_validate_mode_name_invalid_chars() {
        assert!(validate_mode_name("mode-name").is_err());
        assert!(validate_mode_name("mode:name").is_err());
        assert!(validate_mode_name("mode/name").is_err());
        assert!(validate_mode_name("mode name").is_err());
    }

    #[test]
    fn test_validate_mode_name_empty() {
        assert!(validate_mode_name("").is_err());
    }

    #[test]
    fn test_validate_mode_name_reserved() {
        assert!(validate_mode_name("default").is_err());
        assert!(validate_mode_name("global").is_err());
        assert!(validate_mode_name("base").is_err());
    }

    #[test]
    fn test_create_mode() {
        let _temp = setup_test_env();
        let result = create("testmode");
        assert!(result.is_ok());

        // Verify ref was created
        let repo = JinRepo::open_or_create().unwrap();
        assert!(repo.ref_exists("refs/jin/modes/testmode"));
    }

    #[test]
    fn test_create_mode_duplicate() {
        let _temp = setup_test_env();
        create("testmode").unwrap();

        // Try to create again
        let result = create("testmode");
        assert!(matches!(result, Err(JinError::AlreadyExists(_))));
    }

    #[test]
    fn test_use_mode() {
        let _temp = setup_test_env();
        create("testmode").unwrap();

        let result = use_mode("testmode");
        assert!(result.is_ok());

        // Verify context was updated
        let context = ProjectContext::load().unwrap();
        assert_eq!(context.mode, Some("testmode".to_string()));
    }

    #[test]
    fn test_use_mode_nonexistent() {
        let _temp = setup_test_env();
        let result = use_mode("nonexistent");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_list_empty() {
        let _temp = setup_test_env();
        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_modes() {
        let _temp = setup_test_env();
        create("mode1").unwrap();
        create("mode2").unwrap();
        use_mode("mode1").unwrap();

        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_no_mode() {
        let _temp = setup_test_env();
        let result = show();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_with_mode() {
        let _temp = setup_test_env();
        create("testmode").unwrap();
        use_mode("testmode").unwrap();

        let result = show();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unset() {
        let _temp = setup_test_env();
        create("testmode").unwrap();
        use_mode("testmode").unwrap();

        let result = unset();
        assert!(result.is_ok());

        // Verify mode was unset
        let context = ProjectContext::load().unwrap();
        assert_eq!(context.mode, None);
    }

    #[test]
    fn test_unset_no_mode() {
        let _temp = setup_test_env();
        let result = unset();
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_mode() {
        let _temp = setup_test_env();
        create("testmode").unwrap();

        let result = delete("testmode");
        assert!(result.is_ok());

        // Verify ref was deleted
        let repo = JinRepo::open_or_create().unwrap();
        assert!(!repo.ref_exists("refs/jin/modes/testmode"));
    }

    #[test]
    fn test_delete_active_mode() {
        let _temp = setup_test_env();
        create("testmode").unwrap();
        use_mode("testmode").unwrap();

        let result = delete("testmode");
        assert!(result.is_ok());

        // Verify mode was unset
        let context = ProjectContext::load().unwrap();
        assert_eq!(context.mode, None);
    }

    #[test]
    fn test_delete_nonexistent() {
        let _temp = setup_test_env();
        let result = delete("nonexistent");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }
}
