//! Implementation of `jin status`
//!
//! Shows workspace state, active contexts, staged changes, and layer composition.

use crate::commands::apply::PausedApplyState;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps};
use crate::merge::jinmerge::JinMergeConflict;
use crate::staging::StagingIndex;
use crate::staging::WorkspaceMetadata;
use std::path::PathBuf;

/// Workspace state representation
enum WorkspaceState {
    Clean,
    Dirty {
        modified: Vec<PathBuf>,
        deleted: Vec<PathBuf>,
    },
}

/// Check for in-progress apply operation with conflicts
fn check_for_conflicts() -> Option<PausedApplyState> {
    // Follow pattern from resolve.rs:33-37
    if !PausedApplyState::exists() {
        return None;
    }

    // Graceful degradation: if load fails, return None
    PausedApplyState::load().ok()
}

/// Display conflict state from paused apply operation
fn show_conflict_state(state: &PausedApplyState) -> Result<()> {
    // Follow pluralization pattern from line 71-73 in status.rs
    let count = state.conflict_count;
    println!(
        "Merge conflicts ({} file{}):",
        count,
        if count == 1 { "" } else { "s" }
    );

    // List each .jinmerge file
    // CRITICAL: conflict_files contains original paths, convert to .jinmerge paths
    for original_path in &state.conflict_files {
        let merge_path = JinMergeConflict::merge_path_for_file(original_path);
        println!("  {}", merge_path.display());
    }

    // Show resolve instruction
    println!("  Resolve with: jin resolve <files>");

    // Show timestamp - use RFC3339 format for ISO 8601
    println!("  Detected: {}", state.timestamp.to_rfc3339());

    // Blank line for spacing (follow status section pattern)
    println!();

    Ok(())
}

/// Execute the status command
///
/// Shows workspace state and active contexts.
pub fn execute() -> Result<()> {
    // Check if Jin is initialized
    if !ProjectContext::is_initialized() {
        return Err(JinError::NotInitialized);
    }

    // Load context
    let context = ProjectContext::load()?;

    // Open repo for layer operations
    let repo = JinRepo::open_or_create()?;

    // Load staging
    let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    println!("Jin status:");
    println!();

    // Show active mode
    match &context.mode {
        Some(mode) => println!("  Mode:  {} (active)", mode),
        None => println!("  Mode:  (none)"),
    }

    // Show active scope
    match &context.scope {
        Some(scope) => println!("  Scope: {} (active)", scope),
        None => println!("  Scope: (none)"),
    }

    // Show project
    match &context.project {
        Some(project) => println!("  Project: {}", project),
        None => println!("  Project: (none)"),
    }

    println!();

    // Check and display workspace state
    let workspace_state = check_workspace_state()?;
    match workspace_state {
        WorkspaceState::Clean => {
            println!("Workspace state: Clean");
            println!();
        }
        WorkspaceState::Dirty { modified, deleted } => {
            let total = modified.len() + deleted.len();
            println!(
                "Workspace state: Dirty ({} file{} modified)",
                total,
                if total == 1 { "" } else { "s" }
            );
            // List modified files
            for path in &modified {
                println!("  {} (modified)", path.display());
            }
            for path in &deleted {
                println!("  {} (deleted)", path.display());
            }
            println!();
            println!("Use 'jin diff' to see changes or 'jin add <file>' to stage them.");
            println!();
        }
    }

    // Check and display conflict state
    if let Some(conflict_state) = check_for_conflicts() {
        show_conflict_state(&conflict_state)?;
    }

    // Show staged files
    let staged_count = staging.len();

    if staged_count == 0 {
        println!("No staged changes.");
        // Context-sensitive help
        if context.mode.is_none() && context.scope.is_none() && context.project.is_none() {
            println!();
            println!("Use 'jin add <file> --mode' to stage files to a mode layer.");
        } else {
            println!();
            println!("Use 'jin add <file>' to stage files for commit.");
        }
    } else {
        println!(
            "Staged changes ({} file{}):",
            staged_count,
            if staged_count == 1 { "" } else { "s" }
        );
        for entry in staging.entries() {
            println!("  {} -> {}", entry.path.display(), entry.target_layer);
        }
        println!();
        println!("Use 'jin commit -m <message>' to commit staged changes.");
    }

    // Show layer summary
    show_layer_summary(&context, &repo, &staging)?;

    Ok(())
}

/// Check workspace state by comparing current files to metadata
fn check_workspace_state() -> Result<WorkspaceState> {
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => return Ok(WorkspaceState::Clean),
        Err(e) => return Err(e),
    };

    let repo = JinRepo::open()?;

    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    // Compare current file hashes to stored hashes
    for (path, expected_hash) in &metadata.files {
        if !path.exists() {
            deleted.push(path.clone());
        } else {
            let content = std::fs::read(path)?;
            let current_hash = repo.create_blob(&content)?.to_string();
            if current_hash != *expected_hash {
                modified.push(path.clone());
            }
        }
    }

    if modified.is_empty() && deleted.is_empty() {
        Ok(WorkspaceState::Clean)
    } else {
        Ok(WorkspaceState::Dirty { modified, deleted })
    }
}

/// Show layer summary with file counts
fn show_layer_summary(
    context: &ProjectContext,
    repo: &JinRepo,
    staging: &StagingIndex,
) -> Result<()> {
    let git_repo = repo.inner();
    println!();
    println!("Layer summary:");

    let mut has_layers = false;

    // Iterate through applicable layers
    for layer in Layer::all_in_precedence_order() {
        // Skip layers that don't apply to current context
        if layer.requires_mode() && context.mode.is_none() {
            continue;
        }
        if layer.requires_scope() && context.scope.is_none() {
            continue;
        }

        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // Count files in layer using tree walk
        let committed_files = if git_repo.find_reference(&ref_path).is_ok() {
            count_files_in_layer(git_repo, &ref_path).unwrap_or(0)
        } else {
            0
        };

        // Count staged files for this layer
        let staged_files = staging.entries_for_layer(layer).len();

        let storage_path = layer.storage_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // Display with staged count if any
        if committed_files > 0 || staged_files > 0 {
            has_layers = true;
            let staged_note = if staged_files > 0 {
                format!(" ({} staged)", staged_files)
            } else {
                String::new()
            };
            println!(
                "  {}: {} file{}{}",
                storage_path,
                committed_files + staged_files,
                if (committed_files + staged_files) == 1 {
                    ""
                } else {
                    "s"
                },
                staged_note
            );
        }
    }

    if !has_layers {
        println!("  (no layers with files)");
    }

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
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn test_check_workspace_state_clean_no_metadata() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = check_workspace_state();
        assert!(matches!(result, Ok(WorkspaceState::Clean)));
    }

    #[test]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = execute();
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    #[serial]
    fn test_check_for_conflicts_no_state() {
        let temp = TempDir::new().unwrap();
        // Set JIN_DIR to isolate this test
        std::env::set_var("JIN_DIR", temp.path().join(".jin_global"));
        // Use .ok() because current_dir() can fail if previous test left us in deleted dir
        let _ = std::env::set_current_dir(temp.path());

        // No paused state file exists
        let result = check_for_conflicts();
        assert!(result.is_none());
    }

    #[test]
    #[serial]
    fn test_check_for_conflicts_with_state() {
        use crate::commands::apply::PausedLayerConfig;
        use chrono::Utc;
        use std::fs;
        use std::path::PathBuf;

        let temp = TempDir::new().unwrap();
        let temp_path = temp.path();
        // Set JIN_DIR to isolate this test
        std::env::set_var("JIN_DIR", temp_path.join(".jin_global"));
        // Use .ok() because current_dir() can fail if previous test left us in deleted dir
        let _ = std::env::set_current_dir(temp_path);

        // Create .jin directory using absolute path
        let jin_dir = temp_path.join(".jin");
        fs::create_dir_all(&jin_dir).unwrap();

        // Create a paused state
        let state = PausedApplyState {
            timestamp: Utc::now(),
            layer_config: PausedLayerConfig {
                layers: vec!["mode-base".to_string()],
                mode: Some("test_mode".to_string()),
                scope: None,
                project: None,
            },
            conflict_files: vec![PathBuf::from("config.json")],
            applied_files: vec![],
            conflict_count: 1,
        };

        // Manually write the state file to .jin/.paused_apply.yaml
        let state_path = temp_path.join(".jin").join(".paused_apply.yaml");
        let content = serde_yaml::to_string(&state).unwrap();
        fs::write(&state_path, content).unwrap();

        // Verify check_for_conflicts returns Some
        let result = check_for_conflicts();
        assert!(result.is_some());

        // Verify the state has expected content
        let loaded_state = result.unwrap();
        assert_eq!(loaded_state.conflict_count, 1);
        assert_eq!(loaded_state.conflict_files.len(), 1);
    }
}
