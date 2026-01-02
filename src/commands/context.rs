//! Implementation of `jin context`

use crate::core::{JinError, ProjectContext, Result};

/// Execute the context command
///
/// Shows the current active context including mode, scope, and project.
pub fn execute() -> Result<()> {
    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Display context information
    println!("Current Jin context:");
    println!();
    println!(
        "  Active mode:   {}",
        context.mode.as_deref().unwrap_or("(none)")
    );
    println!(
        "  Active scope:  {}",
        context.scope.as_deref().unwrap_or("(none)")
    );
    println!(
        "  Project:       {}",
        context.project.as_deref().unwrap_or("(auto-inferred)")
    );

    if let Some(last_updated) = context.last_updated.as_deref() {
        println!("  Last updated:  {}", last_updated);
    }

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

        // Change to temp directory for project context
        std::env::set_current_dir(temp.path()).unwrap();

        // Initialize .jin directory and context
        std::fs::create_dir(".jin").unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        temp
    }

    #[test]
    #[serial]
    fn test_execute_default_context() {
        let _temp = setup_test_env();
        let result = execute();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_execute_with_mode_and_scope() {
        let _temp = setup_test_env();

        // Set mode and scope
        let mut context = ProjectContext::load().unwrap();
        context.mode = Some("testmode".to_string());
        context.scope = Some("testscope".to_string());
        context.save().unwrap();

        let result = execute();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Don't initialize .jin
        let result = execute();
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }
}
