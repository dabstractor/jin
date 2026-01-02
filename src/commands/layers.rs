//! Implementation of `jin layers`
//!
//! Shows current layer composition and merge order.

use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::JinRepo;

/// Execute the layers command
///
/// Shows current layer composition and merge order.
pub fn execute() -> Result<()> {
    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Open Jin repository to check which layers have commits
    let repo = JinRepo::open_or_create()?;
    let git_repo = repo.inner();

    // Display header
    println!("Layer composition for current context:");
    if context.mode.is_some() || context.scope.is_some() || context.project.is_some() {
        if let Some(mode) = &context.mode {
            println!("  Mode:    {}", mode);
        }
        if let Some(scope) = &context.scope {
            println!("  Scope:   {}", scope);
        }
        if let Some(project) = &context.project {
            println!("  Project: {}", project);
        }
    } else {
        println!("  (no active mode/scope/project)");
    }
    println!();

    // Display layers in precedence order
    println!("Merge order (lowest to highest precedence):");

    let all_layers = Layer::all_in_precedence_order();
    let mut active_count = 0;
    let mut total_files = 0;

    for layer in &all_layers {
        // Skip layers that don't apply to current context
        if layer.requires_mode() && context.mode.is_none() {
            continue;
        }
        if layer.requires_scope() && context.scope.is_none() {
            continue;
        }

        // Get ref path for this layer
        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // Check if layer has commits
        let has_commits = git_repo.find_reference(&ref_path).is_ok();
        let file_count = if has_commits {
            active_count += 1;
            count_files_in_layer(git_repo, &ref_path).unwrap_or(0)
        } else {
            0
        };
        total_files += file_count;

        // Format storage path
        let storage_path = layer.storage_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // Display layer
        let status = if has_commits { "✓" } else { " " };
        println!(
            "  {} {:2}. {:<20} [{}]{}",
            status,
            layer.precedence(),
            layer.to_string(),
            storage_path,
            if file_count > 0 {
                format!(" ({} files)", file_count)
            } else {
                String::new()
            }
        );
    }

    println!();
    println!(
        "Active layers: {} of {} layers have files",
        active_count,
        all_layers.len()
    );
    println!("Total files in workspace: {}", total_files);

    Ok(())
}

/// Count files in a layer by walking its tree
fn count_files_in_layer(repo: &git2::Repository, ref_path: &str) -> Result<usize> {
    let reference = repo.find_reference(ref_path)?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;

    let mut count = 0;
    tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            count += 1;
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// These tests modify global process state (environment variables and current directory).
    /// Run with single-threaded mode when testing the full suite:
    ///   cargo test --lib layers -- --test-threads=1
    fn setup_test_env() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        std::env::set_current_dir(temp.path()).unwrap();

        // Initialize .jin directory and context
        std::fs::create_dir(".jin").unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        temp
    }

    #[test]
    fn test_execute_default_context() {
        let _temp = setup_test_env();
        let result = execute();
        assert!(result.is_ok());
    }

    #[test]
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
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Don't initialize .jin
        let result = execute();
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_count_files_empty_layer() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin-test");
        let repo = git2::Repository::init_bare(&repo_path).unwrap();

        // Create an empty commit
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = {
            let tb = repo.treebuilder(None).unwrap();
            tb.write().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        let commit_id = repo
            .commit(
                Some("refs/heads/test"),
                &sig,
                &sig,
                "Empty commit",
                &tree,
                &[],
            )
            .unwrap();

        repo.find_commit(commit_id).unwrap();

        let count = count_files_in_layer(&repo, "refs/heads/test").unwrap();
        assert_eq!(count, 0);
    }
}
