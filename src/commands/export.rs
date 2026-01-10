//! Implementation of `jin export`
//!
//! This command exports Jin-tracked files back to Git.
//! Files are validated, removed from Jin's staging index, added to Git,
//! and removed from the .gitignore managed block.

use crate::cli::ExportArgs;
use crate::core::{JinError, JinMap, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps, RefOps, TreeOps};
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
    let repo = JinRepo::open_or_create()?;

    // 3. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 4. Process each file with atomic rollback capability
    let mut exported_count = 0;
    let mut errors = Vec::new();
    let mut successfully_exported = Vec::new();

    for path_str in &args.files {
        let path = PathBuf::from(path_str);

        match export_file(&path, &mut staging, &repo) {
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
/// 2. Remove from .gitignore managed block (before git add)
/// 3. Remove from Jin staging
/// 4. Extract from layer if committed and not in workspace
/// 5. Add to Git index
fn export_file(path: &Path, staging: &mut StagingIndex, repo: &JinRepo) -> Result<()> {
    // 1. Validate file is Jin-tracked and check if it's committed
    let is_committed = validate_jin_tracked(path, staging, repo)?;

    // 2. Remove from .gitignore managed block FIRST (before git add)
    // If this fails, we should still continue - the user can manually fix .gitignore
    if let Err(e) = remove_from_managed_block(path) {
        eprintln!(
            "Warning: Could not remove {} from .gitignore: {}",
            path.display(),
            e
        );
    }

    // 3. Remove from Jin staging index if present
    // NOTE: Only remove if actually in staging (committed files aren't)
    if staging.get(path).is_some() {
        staging.remove(path);
    }

    // 4. For committed files not in workspace, extract from layer
    if is_committed && !path.exists() {
        extract_file_from_layer(path, repo)?;
    }

    // 5. Add to Git index (now that it's not in .gitignore)
    add_to_git(path)?;

    Ok(())
}

/// Extract a file from its committed layer to the workspace
fn extract_file_from_layer(path: &Path, repo: &JinRepo) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| JinError::Other("Invalid file path".to_string()))?;

    let jinmap = JinMap::load()?;

    // Find the file in any layer that contains it
    for layer_ref in jinmap.layer_refs() {
        if let Some(files) = jinmap.get_layer_files(layer_ref) {
            if files.contains(&file_name.to_string()) && repo.ref_exists(layer_ref) {
                let commit_oid = repo.resolve_ref(layer_ref)?;
                let commit = repo.find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                // Read file content from tree
                let content = repo.read_file_from_tree(tree_oid, Path::new(file_name))?;

                // Create parent directory if needed
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(JinError::Io)?;
                }

                // Write file to workspace
                std::fs::write(path, content).map_err(JinError::Io)?;

                return Ok(());
            }
        }
    }

    Err(JinError::Other(format!(
        "Could not extract {} from any layer",
        path.display()
    )))
}

/// Validate that a file is Jin-tracked
///
/// A file is considered Jin-tracked if it exists in:
/// 1. The staging index (files staged for commit), or
/// 2. Any committed Jin layer (files in JinMap)
///
/// Returns `Ok(true)` if the file is committed (not in staging),
/// `Ok(false)` if the file is in staging, or an error if not Jin-tracked.
fn validate_jin_tracked(path: &Path, staging: &StagingIndex, repo: &JinRepo) -> Result<bool> {
    // Check if file is in staging index (fast path)
    if staging.get(path).is_some() {
        // For staged files, verify file exists in workspace
        if !path.exists() {
            return Err(JinError::NotFound(path.display().to_string()));
        }
        return Ok(false); // File is in staging (not committed)
    }

    // File not in staging - check JinMap for committed files
    // JinMap stores relative paths, so we need to get just the filename
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| JinError::Other("Invalid file path".to_string()))?;

    let jinmap = JinMap::load()?;
    if !jinmap.contains_file(file_name) {
        return Err(JinError::Other(format!(
            "{} is not Jin-tracked. Use `jin status` to see Jin-tracked files.",
            path.display()
        )));
    }

    // For committed files, verify file exists in committed layer tree
    // The file doesn't need to exist in the workspace for committed files
    let _context = ProjectContext::load()
        .map_err(|_| JinError::Other("Jin not initialized. Run 'jin init' first.".to_string()))?;

    // Find the file in any layer that contains it
    for layer_ref in jinmap.layer_refs() {
        if let Some(files) = jinmap.get_layer_files(layer_ref) {
            if files.contains(&file_name.to_string()) && repo.ref_exists(layer_ref) {
                let commit_oid = repo.resolve_ref(layer_ref)?;
                let commit = repo.find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                // Read file from tree to verify it exists
                // Use just the filename for tree lookup (relative to tree root)
                repo.read_file_from_tree(tree_oid, Path::new(file_name))?;
                return Ok(true); // File found in committed layer
            }
        }
    }

    // Should not reach here if contains_file() returned true
    Err(JinError::Other(format!(
        "{} is in JinMap but not found in any layer tree. Run 'jin repair' to fix.",
        path.display()
    )))
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
    use serial_test::serial;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to serialize tests that change working directory
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_validate_jin_tracked_file_not_found() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let staging = StagingIndex::new();
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = validate_jin_tracked(&path, &staging, &repo);
        // File that doesn't exist and isn't in JinMap returns "not Jin-tracked"
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is not Jin-tracked"));
    }

    #[test]
    fn test_validate_jin_tracked_not_in_staging() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let staging = StagingIndex::new();
        let result = validate_jin_tracked(&file, &staging, &repo);
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

        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let mut staging = StagingIndex::new();
        let entry = StagedEntry::new(file.clone(), Layer::ProjectBase, "hash123".to_string());
        staging.add(entry);

        let result = validate_jin_tracked(&file, &staging, &repo);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_validate_jin_tracked_committed_file() {
        let _lock = TEST_LOCK.lock(); // Serialize with other directory-changing tests

        let temp = TempDir::new().unwrap();

        // Set up JIN_DIR for test isolation FIRST
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);
        std::fs::create_dir_all(&jin_dir).unwrap();

        // Create Jin repo using JIN_DIR
        let repo = JinRepo::open_or_create().unwrap();

        // Create a test file in a layer
        use crate::git::ObjectOps;
        let blob = repo.create_blob(b"test content").unwrap();
        let tree_oid = repo
            .create_tree_from_paths(&[("config.json".to_string(), blob)])
            .unwrap();
        let _commit_oid = repo
            .create_commit(Some("refs/jin/layers/global"), "Test commit", tree_oid, &[])
            .unwrap();

        // Create JinMap with file mapping
        let mut jinmap = JinMap::default();
        jinmap.add_layer_mapping("refs/jin/layers/global", vec!["config.json".to_string()]);

        // Save JinMap to JIN_DIR
        let jinmap_path = jin_dir.join(".jinmap");
        let content = serde_yaml::to_string(&jinmap).unwrap();
        std::fs::write(jinmap_path, content).unwrap();

        // Save original directory (may fail if previous test left us in a deleted dir)
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory so ProjectContext can find .jin/context
        std::env::set_current_dir(temp.path()).unwrap();

        // Create .jin/context file in temp directory (where ProjectContext looks)
        let local_jin = temp.path().join(".jin");
        std::fs::create_dir_all(&local_jin).unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        // Create physical file
        let file = temp.path().join("config.json");
        std::fs::write(&file, b"test content").unwrap();

        // Empty staging index (file not staged)
        let staging = StagingIndex::new();

        // Validation should succeed via JinMap
        let result = validate_jin_tracked(&file, &staging, &repo);
        if let Err(e) = &result {
            eprintln!("Validation error: {}", e);
        }
        assert!(result.is_ok());

        // Always restore directory if we have a valid one
        if let Some(ref dir) = original_dir {
            if dir.exists() {
                let _ = std::env::set_current_dir(dir);
            }
        }

        // Clean up JIN_DIR
        std::env::remove_var("JIN_DIR");
    }

    #[test]
    #[serial]
    fn test_validate_jin_tracked_not_in_jinmap() {
        let _lock = TEST_LOCK.lock(); // Serialize with other directory-changing tests

        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // Set up JIN_DIR for test isolation
        let jin_dir = temp.path().join(".jin");
        std::env::set_var("JIN_DIR", &jin_dir);
        std::fs::create_dir_all(&jin_dir).unwrap();

        // Create empty JinMap
        let jinmap = JinMap::default();
        let jinmap_path = jin_dir.join(".jinmap");
        let content = serde_yaml::to_string(&jinmap).unwrap();
        std::fs::write(jinmap_path, content).unwrap();

        // Create .jin/context file
        let context = ProjectContext::default();
        let context_path = jin_dir.join("context");
        let context_content = serde_yaml::to_string(&context).unwrap();
        std::fs::write(context_path, context_content).unwrap();

        // Empty staging index
        let staging = StagingIndex::new();

        // Validation should fail - file not in JinMap
        let result = validate_jin_tracked(&file, &staging, &repo);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is not Jin-tracked"));

        // Clean up JIN_DIR
        std::env::remove_var("JIN_DIR");
    }

    #[test]
    fn test_execute_no_files() {
        let args = ExportArgs { files: vec![] };
        let result = execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files"));
    }

    #[test]
    #[serial]
    fn test_execute_file_not_jin_tracked() {
        let _lock = TEST_LOCK.lock(); // Serialize with other directory-changing tests

        let temp = TempDir::new().unwrap();

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        // Save original directory (may fail if previous test left us in a deleted dir)
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
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

        // Always restore directory if we have a valid one
        if let Some(ref dir) = original_dir {
            if dir.exists() {
                let _ = std::env::set_current_dir(dir);
            }
        }

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is not Jin-tracked"));
    }

    #[test]
    #[serial]
    fn test_add_to_git_no_git_repo() {
        let _lock = TEST_LOCK.lock(); // Serialize with other directory-changing tests

        let temp = TempDir::new().unwrap();

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        let file = temp.path().join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        // Save original directory (may fail if previous test left us in a deleted dir)
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory (no Git repo)
        std::env::set_current_dir(temp.path()).unwrap();

        let result = add_to_git(&file);

        // Always restore directory if we have a valid one
        if let Some(ref dir) = original_dir {
            if dir.exists() {
                let _ = std::env::set_current_dir(dir);
            }
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("git add failed"));
    }

    #[test]
    #[serial]
    fn test_add_to_git_success() {
        let _lock = TEST_LOCK.lock(); // Serialize with other directory-changing tests

        // Create a manual temp directory with unique name (using timestamp)
        let base_temp = std::env::temp_dir();
        let unique_name = format!(
            "jin_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_path = base_temp.join(&unique_name);
        std::fs::create_dir_all(&temp_path).expect("Failed to create temp directory");

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp_path.join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        // Save original directory
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory and initialize Git repo
        std::env::set_current_dir(&temp_path).expect("Failed to change to temp directory");

        // Initialize Git repo
        Command::new("git")
            .arg("init")
            .output()
            .expect("Failed to init git repo");
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test")
            .output()
            .expect("Failed to set git user.name");
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .output()
            .expect("Failed to set git user.email");

        // Create file using absolute path
        let test_file = temp_path.join("test.json");
        std::fs::write(&test_file, b"{}").expect("Failed to write test file");

        // CRITICAL: Verify and fix current directory before calling add_to_git
        let cwd = std::env::current_dir();
        if cwd.is_err() || !temp_path.exists() {
            // Try to restore current directory to temp_path
            let _ = std::env::set_current_dir(&temp_path);
            let cwd_after = std::env::current_dir();
            eprintln!(
                "Current directory was invalid, restored. cwd={:?}, temp_path.exists={}, cwd_after={:?}",
                cwd,
                temp_path.exists(),
                cwd_after
            );
        }

        // Call add_to_git
        let result = add_to_git(&test_file);

        // Restore directory
        if let Some(ref dir) = original_dir {
            if dir.exists() {
                let _ = std::env::set_current_dir(dir);
            }
        }

        // Clean up temp directory manually
        let _ = std::fs::remove_dir_all(&temp_path);

        if let Err(ref e) = result {
            eprintln!("add_to_git failed: {}", e);
        }
        assert!(result.is_ok());
    }
}
