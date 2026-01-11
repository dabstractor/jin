//! Implementation of `jin import`
//!
//! This command imports Git-tracked files into Jin management.
//! For each file:
//! 1. Validates it's tracked by Git
//! 2. Removes it from Git index (keeping in workspace)
//! 3. Stages it to Jin using the same logic as `jin add`
//! 4. Updates .gitignore to prevent Git from tracking it again

use crate::cli::ImportArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps};
use crate::staging::{
    ensure_in_managed_block, get_file_mode, is_git_tracked, is_symlink, read_file, route_to_layer,
    validate_routing_options, walk_directory, RoutingOptions, StagedEntry, StagedOperation,
    StagingIndex,
};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute the import command
///
/// Imports Git-tracked files into Jin management by:
/// - Validating files are Git-tracked
/// - Removing from Git index (keeping in workspace)
/// - Staging to Jin
/// - Adding to .gitignore managed block
///
/// # Arguments
///
/// * `args` - Command line arguments including files and layer flags
///
/// # Errors
///
/// Returns an error if:
/// - No files are specified
/// - A file doesn't exist
/// - A file is a symlink
/// - A file is NOT tracked by Git
/// - Git rm command fails
/// - Routing options are invalid
pub fn execute(args: ImportArgs) -> Result<()> {
    // 1. Validate we have files to import
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
    // ImportArgs doesn't have layer flags, so we use default routing (ProjectBase)
    let options = RoutingOptions::default();
    validate_routing_options(&options)?;

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // 6. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. Process each file with atomic rollback
    let mut imported_count = 0;
    let mut errors = Vec::new();
    let mut git_removed_files = Vec::new(); // Track for rollback

    for path_str in &args.files {
        let path = PathBuf::from(path_str);

        // Expand directories
        let files_to_import = if path.is_dir() {
            match walk_directory(&path) {
                Ok(files) => files,
                Err(e) => {
                    errors.push(format!("{}: {}", path.display(), e));
                    continue;
                }
            }
        } else {
            vec![path.clone()]
        };

        for file_path in files_to_import {
            match import_file(
                &file_path,
                target_layer,
                &repo,
                &mut staging,
                &mut git_removed_files,
                args.force,
            ) {
                Ok(_) => {
                    imported_count += 1;
                }
                Err(e) => {
                    // Rollback: re-add all previously removed files back to Git
                    if !git_removed_files.is_empty() {
                        eprintln!("Error occurred, rolling back changes...");
                        rollback_git_removals(&git_removed_files);
                    }
                    errors.push(format!("{}: {}", file_path.display(), e));
                    break; // Stop processing on first error
                }
            }
        }

        // If we had errors, don't continue with more files
        if !errors.is_empty() {
            break;
        }
    }

    // 8. Save staging index only if we had successful imports
    if imported_count > 0 {
        staging.save()?;
    }

    // 9. Print summary
    if imported_count > 0 {
        println!(
            "Imported {} file(s) from Git to {} layer",
            imported_count,
            format_layer_name(target_layer)
        );
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if imported_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to import", errors.len()),
            });
        }
    }

    Ok(())
}

/// Import a single file from Git to Jin
///
/// This performs the complete import process:
/// 1. Validate the file is Git-tracked
/// 2. Remove from Git index
/// 3. Stage to Jin
/// 4. Update .gitignore
///
/// # Arguments
///
/// * `path` - Path to the file to import
/// * `layer` - Target layer for the file
/// * `repo` - Jin repository
/// * `staging` - Staging index to add the file to
/// * `git_removed_files` - List of files removed from Git (for rollback)
/// * `force` - Skip modification check if true
fn import_file(
    path: &Path,
    layer: Layer,
    repo: &JinRepo,
    staging: &mut StagingIndex,
    git_removed_files: &mut Vec<PathBuf>,
    force: bool,
) -> Result<()> {
    // Validate file for import
    validate_import_file(path, force)?;

    // Remove from Git index (keeping in workspace)
    remove_from_git(path)?;
    git_removed_files.push(path.to_path_buf());

    // Read content from workspace
    let content = read_file(path)?;

    // Create blob in Jin's bare repository
    let oid = repo.create_blob(&content)?;

    // Get file mode (executable or regular)
    let mode = get_file_mode(path);

    // Create staged entry
    let entry = StagedEntry {
        path: path.to_path_buf(),
        target_layer: layer,
        content_hash: oid.to_string(),
        mode,
        operation: StagedOperation::AddOrModify,
    };

    // Add to staging index
    staging.add(entry);

    // Add to .gitignore managed block
    if let Err(e) = ensure_in_managed_block(path) {
        eprintln!("Warning: Could not update .gitignore: {}", e);
    }

    Ok(())
}

/// Validate a file for import
///
/// Checks that:
/// - File exists
/// - File is not a directory
/// - File is not a symlink
/// - File IS tracked by Git (opposite of `jin add`)
fn validate_import_file(path: &Path, force: bool) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // Check not a directory (should have been expanded)
    if path.is_dir() {
        return Err(JinError::Other(format!(
            "{} is a directory, not a file",
            path.display()
        )));
    }

    // Check not a symlink
    if is_symlink(path)? {
        return Err(JinError::Symlink {
            path: path.display().to_string(),
        });
    }

    // Check IS tracked by project's Git (opposite of add.rs)
    if !is_git_tracked(path)? {
        return Err(JinError::Other(format!(
            "{} is not tracked by Git. Use `jin add` instead.",
            path.display()
        )));
    }

    // If not force mode, check if file is modified
    if !force && is_git_modified(path)? {
        return Err(JinError::Other(format!(
            "{} has uncommitted changes in Git. Use --force to import anyway.",
            path.display()
        )));
    }

    Ok(())
}

/// Remove a file from Git index using `git rm --cached`
///
/// This removes the file from Git's tracking while keeping it in the workspace.
///
/// # Arguments
///
/// * `path` - Path to remove from Git
///
/// # Errors
///
/// Returns an error if the git rm command fails
fn remove_from_git(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("rm")
        .arg("--cached")
        .arg(path)
        .output()
        .map_err(|e| JinError::Other(format!("Failed to execute git rm: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JinError::Other(format!(
            "git rm --cached failed for {}: {}",
            path.display(),
            stderr
        )));
    }

    Ok(())
}

/// Check if a file has uncommitted changes in Git
///
/// Uses `git diff-index` to check if the file differs from HEAD.
fn is_git_modified(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("diff-index")
        .arg("--quiet")
        .arg("HEAD")
        .arg("--")
        .arg(path)
        .output()
        .map_err(|e| JinError::Other(format!("Failed to execute git diff-index: {}", e)))?;

    // Exit code 0 = no changes, 1 = has changes
    Ok(!output.status.success())
}

/// Rollback Git removals by re-adding files to Git index
///
/// This is called when an error occurs during import to restore the Git state.
fn rollback_git_removals(files: &[PathBuf]) {
    for file in files {
        let output = Command::new("git").arg("add").arg(file).output();

        match output {
            Ok(output) if output.status.success() => {
                eprintln!("Rolled back: {}", file.display());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Failed to rollback {}: {}", file.display(), stderr);
            }
            Err(e) => {
                eprintln!("Failed to rollback {}: {}", file.display(), e);
            }
        }
    }
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
    fn test_validate_import_file_not_found() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = validate_import_file(&path, false);
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_validate_import_file_is_directory() {
        let temp = TempDir::new().unwrap();
        let result = validate_import_file(temp.path(), false);
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_import_file_symlink() {
        use std::os::unix::fs::symlink;
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();
        let link = temp.path().join("link.txt");
        symlink(&file, &link).unwrap();

        let result = validate_import_file(&link, false);
        assert!(matches!(result, Err(JinError::Symlink { .. })));
    }

    #[test]
    fn test_format_layer_name() {
        assert_eq!(format_layer_name(Layer::GlobalBase), "global-base");
        assert_eq!(format_layer_name(Layer::ModeBase), "mode-base");
        assert_eq!(format_layer_name(Layer::ProjectBase), "project-base");
    }

    #[test]
    fn test_execute_no_files() {
        let args = ImportArgs {
            files: vec![],
            force: false,
            mode: false,
            scope: None,
            project: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    // Integration tests with actual Git repo would go here
    // but require more complex setup with a real Git repository
}
