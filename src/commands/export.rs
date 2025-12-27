//! Implementation of `jin export`
//!
//! This command exports Jin-tracked files back to Git.
//! Files are validated, removed from Jin's staging index, added to Git,
//! and removed from the .gitignore managed block.

use crate::cli::ExportArgs;
use crate::core::{JinError, Result};
use crate::git::JinRepo;
use crate::staging::{remove_from_managed_block, StagingIndex};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute the export command
///
/// Exports Jin-tracked files back to Git.
///
/// # Arguments
///
/// * `args` - Command line arguments including files to export
///
/// # Errors
///
/// Returns an error if:
/// - No files are specified
/// - A file is not Jin-tracked
/// - Git add operation fails
/// - Rollback fails after partial completion
pub fn execute(args: ExportArgs) -> Result<()> {
    // 1. Validate we have files to export
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Open Jin repository (ensure it exists)
    let _repo = JinRepo::open_or_create()?;

    // 3. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 4. Process each file with atomic rollback capability
    let mut exported_count = 0;
    let mut errors = Vec::new();
    let mut successfully_exported = Vec::new();

    for path_str in &args.files {
        let path = PathBuf::from(path_str);

        match export_file(&path, &mut staging) {
            Ok(_) => {
                successfully_exported.push(path.clone());
                exported_count += 1;
            }
            Err(e) => {
                // If any export fails, attempt rollback
                errors.push(format!("{}: {}", path.display(), e));

                // Attempt to rollback previously exported files
                if !successfully_exported.is_empty() {
                    eprintln!("Error during export, attempting rollback...");
                    if let Err(rollback_err) = rollback_exports(&successfully_exported) {
                        eprintln!("Warning: Rollback failed: {}", rollback_err);
                        eprintln!("Manual intervention may be required for files:");
                        for file in &successfully_exported {
                            eprintln!("  - {}", file.display());
                        }
                    } else {
                        eprintln!("Rollback successful - no files were exported");
                    }
                }

                // Return the error
                return Err(JinError::Other(format!(
                    "Export failed: {}. {} file(s) were rolled back.",
                    errors.join(", "),
                    successfully_exported.len()
                )));
            }
        }
    }

    // 5. Save staging index after all files processed successfully
    staging.save()?;

    // 6. Print summary
    if exported_count > 0 {
        println!(
            "Exported {} file(s) to Git. Files are now tracked by Git and removed from Jin.",
            exported_count
        );
        println!("Don't forget to commit these changes to your Git repository.");
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
    }

    Ok(())
}

/// Export a single file from Jin to Git
///
/// # Steps
/// 1. Validate file is Jin-tracked
/// 2. Remove from Jin staging
/// 3. Add to Git index
/// 4. Remove from .gitignore managed block
fn export_file(path: &Path, staging: &mut StagingIndex) -> Result<()> {
    // 1. Validate file is Jin-tracked
    validate_jin_tracked(path, staging)?;

    // 2. Remove from Jin staging index
    staging.remove(path);

    // 3. Add to Git index
    add_to_git(path)?;

    // 4. Remove from .gitignore managed block
    if let Err(e) = remove_from_managed_block(path) {
        // This is not fatal, but we should warn the user
        eprintln!(
            "Warning: Could not remove {} from .gitignore: {}",
            path.display(),
            e
        );
    }

    Ok(())
}

/// Validate that a file is Jin-tracked
///
/// A file is considered Jin-tracked if it exists in the staging index.
/// TODO: In future milestones, also check layer commits for committed files.
fn validate_jin_tracked(path: &Path, staging: &StagingIndex) -> Result<()> {
    // Check if file exists
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // Check if file is in staging index
    if staging.get(path).is_none() {
        return Err(JinError::Other(format!(
            "{} is not Jin-tracked. Use `jin status` to see Jin-tracked files.",
            path.display()
        )));
    }

    Ok(())
}

/// Add a file to Git index using `git add`
fn add_to_git(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("add")
        .arg(path)
        .output()
        .map_err(|e| JinError::Other(format!("Failed to execute git add: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JinError::Other(format!(
            "git add failed for {}: {}",
            path.display(),
            stderr
        )));
    }

    Ok(())
}

/// Rollback exported files by removing them from Git and adding back to .gitignore
fn rollback_exports(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        // Remove from Git index (but keep in working directory)
        let output = Command::new("git")
            .arg("reset")
            .arg("HEAD")
            .arg(path)
            .output()
            .map_err(|e| {
                JinError::Other(format!(
                    "Failed to execute git reset during rollback: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(JinError::Other(format!(
                "git reset failed during rollback for {}: {}",
                path.display(),
                stderr
            )));
        }

        // Add back to .gitignore managed block
        if let Err(e) = crate::staging::ensure_in_managed_block(path) {
            eprintln!(
                "Warning during rollback: Could not add {} back to .gitignore: {}",
                path.display(),
                e
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Layer;
    use crate::staging::StagedEntry;
    use tempfile::TempDir;

    #[test]
    fn test_validate_jin_tracked_file_not_found() {
        let staging = StagingIndex::new();
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = validate_jin_tracked(&path, &staging);
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_validate_jin_tracked_not_in_staging() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        let staging = StagingIndex::new();
        let result = validate_jin_tracked(&file, &staging);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is not Jin-tracked"));
    }

    #[test]
    fn test_validate_jin_tracked_success() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        let mut staging = StagingIndex::new();
        let entry = StagedEntry::new(file.clone(), Layer::ProjectBase, "hash123".to_string());
        staging.add(entry);

        let result = validate_jin_tracked(&file, &staging);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_no_files() {
        let args = ExportArgs { files: vec![] };
        let result = execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files"));
    }

    #[test]
    fn test_execute_file_not_jin_tracked() {
        let temp = TempDir::new().unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Initialize Git repo
        Command::new("git").arg("init").output().unwrap();

        // Create file after changing to temp directory
        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        let args = ExportArgs {
            files: vec![file.display().to_string()],
        };
        let result = execute(args);

        std::env::set_current_dir(&original_dir).unwrap();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is not Jin-tracked"));
    }

    #[test]
    fn test_add_to_git_no_git_repo() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        // Change to temp directory (no Git repo)
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = add_to_git(&file);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("git add failed"));
    }

    #[test]
    fn test_add_to_git_success() {
        let temp = TempDir::new().unwrap();
        let temp_path = temp.path().to_path_buf();

        // Change to temp directory and initialize Git repo
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_path).unwrap();

        // Initialize Git repo
        Command::new("git").arg("init").output().unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test")
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .output()
            .unwrap();

        // Create file after changing to temp directory
        std::fs::write("test.json", b"{}").unwrap();

        // Use relative path since we're in the git repo directory
        let result = add_to_git(Path::new("test.json"));

        // Change back to original directory after git add completes
        // Use ok() instead of unwrap() since dir might not exist in test environment
        let _ = std::env::set_current_dir(&original_dir);

        assert!(result.is_ok());
    }
}
