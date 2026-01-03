//! Implementation of `jin rm`
//!
//! This command removes files from the staging index and optionally from the workspace.
//! Files are marked for deletion with StagedOperation::Delete entries.
//! Similar to git rm, the default behavior removes from staging only (like git rm --cached),
//! while --force removes from both staging and workspace.

use crate::cli::RmArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::JinRepo;
use crate::staging::{
    remove_from_managed_block, route_to_layer, validate_routing_options, RoutingOptions,
    StagedEntry, StagingIndex,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Execute the rm command
///
/// Removes files from staging and optionally from workspace.
///
/// # Arguments
///
/// * `args` - Command line arguments including files and layer flags
///
/// # Errors
///
/// Returns an error if:
/// - No files are specified
/// - A file is not in staging
/// - Routing options are invalid
/// - Jin is not initialized
pub fn execute(args: RmArgs) -> Result<()> {
    // 1. Validate we have files to remove
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Load project context for active mode/scope
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // 3. Build and validate routing options
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
    validate_routing_options(&options)?;

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;

    // 5. Open Jin repository (needed for consistency with other commands)
    let _repo = JinRepo::open_or_create()?;

    // 6. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. Dry-run mode: print what would be removed and return
    if args.dry_run {
        for path_str in &args.files {
            let path = PathBuf::from(path_str);
            if staging.get(&path).is_some() {
                let workspace_action = if args.force && path.exists() {
                    "and from workspace"
                } else {
                    "from staging only"
                };
                println!("Would remove: {} ({})", path.display(), workspace_action);
            } else {
                eprintln!("Warning: {} not in staging", path.display());
            }
        }
        return Ok(());
    }

    // 8. Count files that need workspace removal for confirmation
    let files_to_remove_from_workspace: Vec<PathBuf> = args
        .files
        .iter()
        .filter(|path_str| {
            let path = PathBuf::from(path_str);
            // Check if file is in staging and exists in workspace
            staging.get(&path).is_some() && path.exists() && args.force
        })
        .map(|s| PathBuf::from(s.as_str()))
        .collect();

    // 9. Confirmation prompt for workspace deletion (without --force)
    if !files_to_remove_from_workspace.is_empty() && !args.force {
        let message = format!(
            "This will remove {} file(s) from workspace. Type 'yes' to confirm:",
            files_to_remove_from_workspace.len()
        );
        if !prompt_confirmation(&message)? {
            println!("Removal cancelled");
            return Ok(());
        }
    }

    // 10. Process each file (with error collection)
    let mut removed_count = 0;
    let mut errors = Vec::new();

    for path_str in &args.files {
        let path = PathBuf::from(path_str);
        match unstage_file(&path, target_layer, &mut staging, &args) {
            Ok(_) => removed_count += 1,
            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
        }
    }

    // 11. Save staging index
    staging.save()?;

    // 12. Print summary
    if removed_count > 0 {
        println!(
            "Removed {} file(s) from {} layer",
            removed_count,
            format_layer_name(target_layer)
        );
    }

    // 13. Handle errors (partial success pattern)
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if removed_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to remove", errors.len()),
            });
        }
    }

    Ok(())
}

/// Unstage a single file from the staging index
fn unstage_file(
    path: &Path,
    layer: Layer,
    staging: &mut StagingIndex,
    args: &RmArgs,
) -> Result<()> {
    // Check if file is in staging
    let _existing_entry = staging
        .get(path)
        .ok_or_else(|| JinError::NotFound(format!("File not in staging: {}", path.display())))?;

    // Remove from staging index
    staging.remove(path);

    // Create delete entry to mark for deletion on commit
    let delete_entry = StagedEntry::delete(path.to_path_buf(), layer);
    staging.add(delete_entry);

    // Remove from .gitignore managed block
    if let Err(e) = remove_from_managed_block(path) {
        eprintln!("Warning: Could not update .gitignore: {}", e);
    }

    // Remove from workspace if --force or confirmed
    if args.force && path.exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

/// Prompt user for confirmation
fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("yes"))
}

/// Format layer name for display
fn format_layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::GlobalBase => "global-base",
        Layer::ModeBase => "mode-base",
        Layer::ModeScope => "mode-scope",
        Layer::ModeScopeProject => "mode-scope-project",
        Layer::ModeProject => "mode-project",
        Layer::ScopeBase => "scope-base",
        Layer::ProjectBase => "project-base",
        Layer::UserLocal => "user-local",
        Layer::WorkspaceActive => "workspace-active",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execute_no_files() {
        let args = RmArgs {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let args = RmArgs {
            files: vec!["file.txt".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_format_layer_name() {
        assert_eq!(format_layer_name(Layer::GlobalBase), "global-base");
        assert_eq!(format_layer_name(Layer::ModeBase), "mode-base");
        assert_eq!(format_layer_name(Layer::ProjectBase), "project-base");
    }

    #[test]
    fn test_unstage_file_not_in_staging() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        let mut staging = StagingIndex::new();
        let args = RmArgs {
            files: vec!["test.json".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };

        let file_path = project_path.join("test.json");
        let result = unstage_file(&file_path, Layer::ProjectBase, &mut staging, &args);
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_unstage_file_success() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        // Create a test file
        let test_file = project_path.join("test.json");
        std::fs::write(&test_file, r#"{"test": true}"#).unwrap();

        // Stage the file
        let mut staging = StagingIndex::new();
        let entry = StagedEntry {
            path: test_file.clone(),
            target_layer: Layer::ProjectBase,
            content_hash: "abc123".to_string(),
            mode: 0o644,
            operation: crate::staging::StagedOperation::AddOrModify,
        };
        staging.add(entry);

        let args = RmArgs {
            files: vec!["test.json".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };

        // Unstage without force (should not delete from workspace)
        let result = unstage_file(&test_file, Layer::ProjectBase, &mut staging, &args);
        assert!(result.is_ok());

        // File should still exist in workspace
        assert!(test_file.exists());

        // Should have a delete entry now
        let delete_entry = staging.get(&test_file);
        assert!(delete_entry.is_some());
        assert!(delete_entry.unwrap().is_delete());
    }

    #[test]
    fn test_unstage_file_with_force() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        // Create a test file
        let test_file = project_path.join("test.json");
        std::fs::write(&test_file, r#"{"test": true}"#).unwrap();

        // Stage the file
        let mut staging = StagingIndex::new();
        let entry = StagedEntry {
            path: test_file.clone(),
            target_layer: Layer::ProjectBase,
            content_hash: "abc123".to_string(),
            mode: 0o644,
            operation: crate::staging::StagedOperation::AddOrModify,
        };
        staging.add(entry);

        let args = RmArgs {
            files: vec!["test.json".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: true,
            dry_run: false,
        };

        // Unstage with force (should delete from workspace)
        let result = unstage_file(&test_file, Layer::ProjectBase, &mut staging, &args);
        assert!(result.is_ok());

        // File should be deleted from workspace
        assert!(!test_file.exists());

        // Should have a delete entry
        let delete_entry = staging.get(&test_file);
        assert!(delete_entry.is_some());
        assert!(delete_entry.unwrap().is_delete());
    }

    #[test]
    fn test_execute_dry_run() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        // Create and stage a test file (use absolute path like other tests)
        let test_file = project_path.join("test.json");
        std::fs::write(&test_file, r#"{"test": true}"#).unwrap();

        let mut staging = StagingIndex::new();
        let entry = StagedEntry {
            path: test_file.clone(),
            target_layer: Layer::ProjectBase,
            content_hash: "abc123".to_string(),
            mode: 0o644,
            operation: crate::staging::StagedOperation::AddOrModify,
        };
        staging.add(entry);
        staging.save().unwrap();

        // Pass the full path string to match the staged path
        let args = RmArgs {
            files: vec![test_file.display().to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: true,
        };

        let result = execute(args);
        assert!(result.is_ok());

        // File should still be in staging after dry run
        let staging_after = StagingIndex::load().unwrap();
        assert!(staging_after.get(&test_file).is_some());
    }

    #[test]
    fn test_execute_project_without_mode() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        let args = RmArgs {
            files: vec!["file.txt".to_string()],
            mode: false,
            scope: None,
            project: true,
            global: false,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_global_with_mode() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        let args = RmArgs {
            files: vec!["file.txt".to_string()],
            mode: true,
            scope: None,
            project: false,
            global: true,
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }
}
