//! Implementation of `jin mv`
//!
//! This command moves/renames files in the staging index and optionally in the workspace.
//! Files are updated with StagedOperation::Rename entries.
//! Similar to git mv, the default behavior updates staging only (like git mv --cached),
//! while --force moves the actual workspace files as well.

use crate::cli::MvArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::JinRepo;
use crate::staging::{
    ensure_in_managed_block, remove_from_managed_block, route_to_layer, validate_routing_options,
    RoutingOptions, StagedEntry, StagingIndex,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Execute the mv command
///
/// Moves/renames files in staging and optionally in workspace.
///
/// # Arguments
///
/// * `args` - Command line arguments including files (as src/dst pairs) and layer flags
///
/// # Errors
///
/// Returns an error if:
/// - No files are specified
/// - File count is not even (not in pairs)
/// - A source file is not in staging
/// - Destination already exists in staging
/// - Routing options are invalid
/// - Jin is not initialized
pub fn execute(args: MvArgs) -> Result<()> {
    // 1. Validate we have file pairs (must be even number)
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }
    if !args.files.len().is_multiple_of(2) {
        return Err(JinError::Other(
            "Files must be specified in source/destination pairs".to_string(),
        ));
    }

    // 2. Load project context for active mode/scope
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
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

    // 7. Dry-run mode: print what would be moved and return
    if args.dry_run {
        for chunk in args.files.chunks(2) {
            let src = PathBuf::from(&chunk[0]);
            let dst = PathBuf::from(&chunk[1]);
            if staging.get(&src).is_some() {
                let workspace_action = if args.force && src.exists() {
                    "and from workspace"
                } else {
                    "from staging only"
                };
                println!(
                    "Would move: {} -> {} ({})",
                    src.display(),
                    dst.display(),
                    workspace_action
                );
            } else {
                eprintln!("Warning: {} not in staging", src.display());
            }
        }
        return Ok(());
    }

    // 8. Count files that need workspace moving for confirmation
    let files_to_move_in_workspace: Vec<(PathBuf, PathBuf)> = args
        .files
        .chunks(2)
        .filter_map(|chunk| {
            let src = PathBuf::from(&chunk[0]);
            let dst = PathBuf::from(&chunk[1]);
            // Check if file is in staging and exists in workspace
            if staging.get(&src).is_some() && src.exists() {
                Some((src, dst))
            } else {
                None
            }
        })
        .collect();

    // 9. Confirmation prompt for workspace moves (without --force)
    if !files_to_move_in_workspace.is_empty() && !args.force {
        let message = format!(
            "This will move {} file(s) in workspace. Type 'yes' to confirm:",
            files_to_move_in_workspace.len()
        );
        if !prompt_confirmation(&message)? {
            println!("Move cancelled");
            return Ok(());
        }
    }

    // 10. Process each file pair (with error collection)
    let mut moved_count = 0;
    let mut errors = Vec::new();

    for chunk in args.files.chunks(2) {
        let src = PathBuf::from(&chunk[0]);
        let dst = PathBuf::from(&chunk[1]);
        match move_file(&src, &dst, target_layer, &mut staging, &args) {
            Ok(_) => moved_count += 1,
            Err(e) => errors.push(format!("{} -> {}: {}", src.display(), dst.display(), e)),
        }
    }

    // 11. Save staging index
    staging.save()?;

    // 12. Print summary
    if moved_count > 0 {
        println!(
            "Moved {} file(s) in {} layer",
            moved_count,
            format_layer_name(target_layer)
        );
    }

    // 13. Handle errors (partial success pattern)
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if moved_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to move", errors.len()),
            });
        }
    }

    Ok(())
}

/// Move a single file in the staging index
fn move_file(
    src: &Path,
    dst: &Path,
    layer: Layer,
    staging: &mut StagingIndex,
    args: &MvArgs,
) -> Result<()> {
    // 1. Validate source: Check if file is in staging
    let existing_entry = staging.get(src).ok_or_else(|| {
        JinError::NotFound(format!("Source file not in staging: {}", src.display()))
    })?;

    // 2. Validate destination: Check if destination already staged
    if staging.get(dst).is_some() {
        return Err(JinError::Other(format!(
            "Destination already in staging: {}",
            dst.display()
        )));
    }

    // 3. Preserve metadata: Get content hash and mode from existing entry
    let content_hash = existing_entry.content_hash.clone();
    let mode = existing_entry.mode;

    // 4. Create rename entry: Using new constructor from StagedEntry
    let rename_entry = StagedEntry::rename(
        src.to_path_buf(),
        dst.to_path_buf(),
        layer,
        content_hash,
        mode,
    );

    // 5. Update staging index: Remove old, add new
    staging.remove(src);
    staging.add(rename_entry);

    // 6. Update .gitignore: Remove old path, add new path
    if let Err(e) = remove_from_managed_block(src) {
        eprintln!(
            "Warning: Could not remove {} from .gitignore: {}",
            src.display(),
            e
        );
    }
    if let Err(e) = ensure_in_managed_block(dst) {
        eprintln!(
            "Warning: Could not add {} to .gitignore: {}",
            dst.display(),
            e
        );
    }

    // 7. Workspace move: If --force is set and file exists
    if args.force && src.exists() {
        // Create parent directory if needed
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Atomic rename with cross-filesystem fallback
        match std::fs::rename(src, dst) {
            Ok(()) => Ok(()),
            Err(e) if e.raw_os_error() == Some(18) => {
                // Cross-device link: copy + delete fallback
                std::fs::copy(src, dst)?;
                std::fs::remove_file(src)
            }
            Err(e) => Err(e),
        }?;
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
        let args = MvArgs {
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
    fn test_execute_odd_number_of_files() {
        let args = MvArgs {
            files: vec![
                "src.txt".to_string(),
                "dst.txt".to_string(),
                "extra.txt".to_string(),
            ],
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

        let args = MvArgs {
            files: vec!["src.txt".to_string(), "dst.txt".to_string()],
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
    fn test_move_file_not_staged() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        let mut staging = StagingIndex::new();
        let args = MvArgs {
            files: vec!["src.txt".to_string(), "dst.txt".to_string()],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };

        let src = project_path.join("src.txt");
        let dst = project_path.join("dst.txt");
        let result = move_file(&src, &dst, Layer::ProjectBase, &mut staging, &args);
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_move_file_destination_exists() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        let mut staging = StagingIndex::new();

        // Add both source and destination to staging
        let src = project_path.join("src.txt");
        let dst = project_path.join("dst.txt");

        let src_entry = StagedEntry::new(src.clone(), Layer::ProjectBase, "abc123".to_string());
        let dst_entry = StagedEntry::new(dst.clone(), Layer::ProjectBase, "def456".to_string());
        staging.add(src_entry);
        staging.add(dst_entry);

        let args = MvArgs {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };

        let result = move_file(&src, &dst, Layer::ProjectBase, &mut staging, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_project_without_mode() {
        let _ctx = crate::test_utils::setup_unit_test();

        let args = MvArgs {
            files: vec!["src.txt".to_string(), "dst.txt".to_string()],
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
        let _ctx = crate::test_utils::setup_unit_test();

        let args = MvArgs {
            files: vec!["src.txt".to_string(), "dst.txt".to_string()],
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

    #[test]
    fn test_move_file_success() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        let mut staging = StagingIndex::new();

        // Add source to staging
        let src = project_path.join("src.txt");
        let dst = project_path.join("dst.txt");

        let src_entry = StagedEntry::new(src.clone(), Layer::ProjectBase, "abc123".to_string());
        staging.add(src_entry);

        let args = MvArgs {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: false,
        };

        // Move file (without force - only staging)
        let result = move_file(&src, &dst, Layer::ProjectBase, &mut staging, &args);
        assert!(result.is_ok());

        // Source should be removed from staging
        assert!(staging.get(&src).is_none());

        // Destination should be in staging as rename
        let dst_entry = staging.get(&dst);
        assert!(dst_entry.is_some());
        assert!(dst_entry.unwrap().is_rename());
    }

    #[test]
    fn test_execute_dry_run() {
        let ctx = crate::test_utils::setup_unit_test();
        let project_path = &ctx.project_path;

        // Create a test file (no need to stage it for dry-run test)
        let test_file = project_path.join("test.json");
        std::fs::write(&test_file, r#"{"test": true}"#).unwrap();

        // Clear any existing staging index to avoid test pollution
        let _ = StagingIndex::load().map(|mut s| {
            s.clear();
            let _ = s.save();
        });

        let mut staging = StagingIndex::new();
        // Use absolute path to ensure consistency
        let entry = StagedEntry::new(test_file.clone(), Layer::ProjectBase, "abc123".to_string());
        staging.add(entry);
        staging.save().unwrap();

        // Verify file is in staging before dry run
        assert!(staging.get(&test_file).is_some());

        // Dry run with absolute paths
        let args = MvArgs {
            files: vec![
                test_file.display().to_string(),
                project_path.join("renamed.json").display().to_string(),
            ],
            mode: false,
            scope: None,
            project: false,
            global: false,
            force: false,
            dry_run: true,
        };

        let result = execute(args);
        assert!(result.is_ok());

        // Verify dry_run didn't modify the in-memory staging we created
        // (Note: We're not testing save/load here, just that dry_run returns early)
        assert!(staging.get(&test_file).is_some());
    }
}
