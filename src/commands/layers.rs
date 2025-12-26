//! Layers command implementation.
//!
//! This module implements the `jin layers` command that displays
//! the current layer composition and merge order.

use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use std::collections::HashSet;

/// Execute the layers command.
///
/// Displays all 9 layers in precedence order with:
/// - Layer number (1-9)
/// - Layer name (from Display impl)
/// - Commit count or "No commits"
/// - Active context markers for mode/scope
///
/// # Errors
///
/// Returns `JinError::Message` if Jin is not initialized.
pub fn execute() -> Result<()> {
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

    // 3. Load active context
    let context = ProjectContext::load(&workspace_root)?;

    // 4. Validate Git repository exists
    let _git_repo =
        git2::Repository::discover(&workspace_root).map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 6. Get layers with commits
    let committed_layers: HashSet<_> = repo
        .list_layer_refs()?
        .into_iter()
        .map(|(layer, _oid)| layer)
        .collect();

    // 7. Display all layers in precedence order
    display_layers(&context, &committed_layers);

    Ok(())
}

/// Displays all layers with their status.
///
/// # Arguments
///
/// * `context` - The active project context
/// * `committed_layers` - Set of layers that have commits
fn display_layers(context: &ProjectContext, committed_layers: &HashSet<Layer>) {
    println!();
    println!("Layer Composition (1=lowest, 9=highest precedence):");
    println!();

    // PATTERN: All Layer enum variants in declaration order = precedence order
    // Must manually list all 9 variants here

    // Layer 1: GlobalBase
    display_layer_entry(
        1,
        &Layer::GlobalBase,
        committed_layers.contains(&Layer::GlobalBase),
        false,  // never active context
        false,
    );

    // Layer 2: ModeBase - check if active mode
    let layer_2 = if let Some(ref mode) = context.mode {
        Layer::ModeBase { mode: mode.clone() }
    } else {
        Layer::ModeBase {
            mode: "(none)".to_string(),
        }
    };
    let is_active_2 = context.mode.is_some();
    display_layer_entry(2, &layer_2, committed_layers.contains(&layer_2), is_active_2, false);

    // Layer 3: ModeScope - check if active mode+scope
    let layer_3 = if let (Some(ref mode), Some(ref scope)) = (&context.mode, &context.scope) {
        Layer::ModeScope {
            mode: mode.clone(),
            scope: scope.clone(),
        }
    } else {
        // Display placeholder for clarity even if not active
        Layer::ModeScope {
            mode: "(none)".to_string(),
            scope: "(none)".to_string(),
        }
    };
    let is_active_3 = context.mode.is_some() && context.scope.is_some();
    display_layer_entry(3, &layer_3, committed_layers.contains(&layer_3), is_active_3, false);

    // Layer 4: ModeScopeProject
    let layer_4 = if let (Some(ref mode), Some(ref scope)) = (&context.mode, &context.scope) {
        // Get project name
        let project = detect_project_name().unwrap_or_else(|_| "(unknown)".to_string());
        Layer::ModeScopeProject {
            mode: mode.clone(),
            scope: scope.clone(),
            project,
        }
    } else {
        Layer::ModeScopeProject {
            mode: "(none)".to_string(),
            scope: "(none)".to_string(),
            project: "(none)".to_string(),
        }
    };
    let is_active_4 = context.mode.is_some() && context.scope.is_some();
    display_layer_entry(4, &layer_4, committed_layers.contains(&layer_4), is_active_4, false);

    // Layer 5: ModeProject
    let layer_5 = if let Some(ref mode) = context.mode {
        let project = detect_project_name().unwrap_or_else(|_| "(unknown)".to_string());
        Layer::ModeProject {
            mode: mode.clone(),
            project,
        }
    } else {
        Layer::ModeProject {
            mode: "(none)".to_string(),
            project: "(none)".to_string(),
        }
    };
    let is_active_5 = context.mode.is_some();
    display_layer_entry(5, &layer_5, committed_layers.contains(&layer_5), is_active_5, false);

    // Layer 6: ScopeBase
    let layer_6 = if let Some(ref scope) = context.scope {
        Layer::ScopeBase {
            scope: scope.clone(),
        }
    } else {
        Layer::ScopeBase {
            scope: "(none)".to_string(),
        }
    };
    let is_active_6 = context.scope.is_some();
    display_layer_entry(6, &layer_6, committed_layers.contains(&layer_6), is_active_6, false);

    // Layer 7: ProjectBase
    let project = detect_project_name().unwrap_or_else(|_| "(unknown)".to_string());
    let layer_7 = Layer::ProjectBase { project };
    display_layer_entry(7, &layer_7, committed_layers.contains(&layer_7), false, false);

    // Layer 8: UserLocal - not versioned
    display_layer_entry(8, &Layer::UserLocal, false, false, true);

    // Layer 9: WorkspaceActive - not versioned, derived
    display_layer_entry(9, &Layer::WorkspaceActive, false, false, true);

    println!();
}

/// Displays a single layer entry.
///
/// # GOTCHA: The display format shows:
/// - Layer number in brackets
/// - Layer name from Display trait
/// - Active context indicator if applicable
/// - Commit status or "Not versioned" note
fn display_layer_entry(
    index: usize,
    layer: &Layer,
    has_commits: bool,
    is_active_context: bool,
    is_not_versioned: bool,
) {
    let layer_name = format!("{}", layer);
    let status = if is_not_versioned {
        "Not versioned".to_string()
    } else if has_commits {
        // Could get commit count here, but "Has commits" is sufficient for MVP
        "Has commits".to_string()
    } else {
        "No commits".to_string()
    };

    let active_marker = if is_active_context {
        " - Active context"
    } else {
        ""
    };

    println!(
        "  [{}] {:<30} - {}{}",
        index, layer_name, status, active_marker
    );
}

/// Detects the project name from Git remote or directory name.
///
/// Copy of pattern from src/commands/log.rs lines 113-152
fn detect_project_name() -> Result<String> {
    let workspace_root = std::env::current_dir()?;
    let repo = git2::Repository::discover(&workspace_root).map_err(|_| JinError::RepoNotFound {
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

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Save the current directory and restore it when dropped.
    /// PATTERN: Copied from context.rs lines 85-101
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
    /// PATTERN: Copied from context.rs lines 104-106
    fn init_git_repo(dir: &std::path::Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    /// Helper to initialize Jin in a directory
    /// PATTERN: Copied from context.rs lines 109-132
    fn init_jin(dir: &std::path::Path) {
        use crate::staging::index::StagingIndex;

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
        let staging_index = StagingIndex::new();
        staging_index.save_to_disk(dir).unwrap();

        // Create workspace directory
        let workspace_dir = dir.join(".jin/workspace");
        std::fs::create_dir_all(workspace_dir).unwrap();
    }

    #[test]
    fn test_execute_shows_all_layers() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Execute should succeed
        execute().unwrap();
    }

    #[test]
    fn test_execute_with_active_context() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        // Set mode and scope
        let mut context = ProjectContext::load(project_dir).unwrap();
        context.set_mode(Some("claude".to_string()));
        context.set_scope(Some("language:rust".to_string()));
        context.save(project_dir).unwrap();

        // Execute should succeed
        execute().unwrap();

        // Verify context is still set
        let loaded = ProjectContext::load(project_dir).unwrap();
        assert_eq!(loaded.mode, Some("claude".to_string()));
        assert_eq!(loaded.scope, Some("language:rust".to_string()));
    }

    #[test]
    fn test_execute_not_initialized_error() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        // Don't initialize Jin

        let result = execute();
        assert!(result.is_err());
        if let Err(JinError::Message(msg)) = result {
            assert!(msg.contains("Jin is not initialized"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_execute_with_commits_in_layers() {
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
        git_repo
            .commit(
                Some("refs/jin/layers/global"),
                &signature,
                &signature,
                "First global commit",
                &tree,
                &[],
            )
            .unwrap();

        // Execute should show layers with commits
        execute().unwrap();
    }
}
