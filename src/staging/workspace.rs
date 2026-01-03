//! Workspace file operations for Jin staging
//!
//! This module provides utilities for reading files from the workspace
//! (project working directory) and checking file properties.
//!
//! It also provides workspace validation logic to detect detached workspace
//! states where the workspace files or metadata no longer correspond to valid
//! layer configurations.

use crate::core::config::ProjectContext;
use crate::core::{JinError, Result};
use crate::git::JinRepo;
use crate::git::RefOps;
use crate::staging::metadata::WorkspaceMetadata;
use std::path::{Path, PathBuf};

/// Read a file from the workspace
///
/// # Arguments
///
/// * `path` - Path to the file in the workspace
///
/// # Returns
///
/// The file content as a byte vector
///
/// # Errors
///
/// Returns `JinError::NotFound` if the file doesn't exist
/// Returns `JinError::Io` for other IO errors
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            JinError::NotFound(path.display().to_string())
        } else {
            JinError::Io(e)
        }
    })
}

/// Check if a path is a symlink
///
/// # Arguments
///
/// * `path` - Path to check
///
/// # Returns
///
/// `true` if the path is a symlink, `false` otherwise
pub fn is_symlink(path: &Path) -> Result<bool> {
    let meta = std::fs::symlink_metadata(path)?;
    Ok(meta.file_type().is_symlink())
}

/// Check if a file is tracked by the project's Git repository
///
/// This checks the project's Git repository (not Jin's bare repo)
/// to determine if a file is already under Git version control.
///
/// # Arguments
///
/// * `path` - Path to check (can be relative or absolute)
///
/// # Returns
///
/// `true` if the file is tracked by Git, `false` otherwise
pub fn is_git_tracked(path: &Path) -> Result<bool> {
    // Determine the directory to search from
    let search_from = if path.is_absolute() {
        path.parent().unwrap_or(path)
    } else {
        Path::new(".")
    };

    // Try to discover project's Git repository
    let repo = match git2::Repository::discover(search_from) {
        Ok(r) => r,
        Err(_) => return Ok(false), // No Git repo = not tracked
    };

    // Get the index (staging area) of project's Git
    let index = repo.index().map_err(JinError::Git)?;

    // Normalize path relative to repo workdir
    let workdir = repo.workdir().unwrap_or_else(|| Path::new("."));
    let rel_path = if path.is_absolute() {
        path.strip_prefix(workdir).unwrap_or(path)
    } else {
        path
    };

    // Check if file is in the index
    Ok(index.get_path(rel_path, 0).is_some())
}

/// Get file mode (executable or regular)
///
/// Returns the Git file mode based on executable permissions.
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// `0o100755` for executable files, `0o100644` for regular files
#[cfg(unix)]
pub fn get_file_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) if meta.permissions().mode() & 0o111 != 0 => 0o100755,
        _ => 0o100644,
    }
}

#[cfg(not(unix))]
pub fn get_file_mode(_path: &Path) -> u32 {
    0o100644
}

/// Walk a directory recursively and return all file paths
///
/// # Arguments
///
/// * `path` - Path to the directory to walk
///
/// # Returns
///
/// A vector of file paths (not directories)
///
/// # Errors
///
/// Returns `JinError::Io` if the directory cannot be read
pub fn walk_directory(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory_recursive(path, &mut files)?;
    Ok(files)
}

fn walk_directory_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            walk_directory_recursive(&entry_path, files)?;
        } else {
            files.push(entry_path);
        }
    }
    Ok(())
}

/// Detect if workspace files have been modified outside of Jin operations
///
/// This function checks the current hash of each file in the workspace
/// metadata against the stored hash. If any file has been modified,
/// deleted, or added externally, this function returns the list of
/// mismatched files.
///
/// # Arguments
///
/// * `metadata` - The workspace metadata containing expected file hashes
/// * `repo` - The Jin repository (used to compute Git blob hashes)
///
/// # Returns
///
/// - `Ok(Some(files))` - List of files that don't match stored hashes
/// - `Ok(None)` - All files match (workspace is attached)
/// - `Err(JinError)` - Error reading files or computing hashes
fn detect_file_mismatch(
    metadata: &WorkspaceMetadata,
    repo: &JinRepo,
) -> Result<Option<Vec<PathBuf>>> {
    let mut modified_files = Vec::new();

    // Iterate through tracked files in metadata
    for (path, stored_hash) in metadata.files.iter() {
        // Check if file exists in workspace
        if !path.exists() {
            modified_files.push(path.clone());
            continue;
        }

        // Compute current hash using Git blob hash
        let content = std::fs::read(path)?;
        let oid = repo.inner().blob(&content)?;
        let current_hash = oid.to_string();

        // Compare with stored hash
        if current_hash != *stored_hash {
            modified_files.push(path.clone());
        }
    }

    Ok(if modified_files.is_empty() {
        None
    } else {
        Some(modified_files)
    })
}

/// Detect if workspace metadata references non-existent layer commits
///
/// This function checks if the layer refs stored in WorkspaceMetadata
/// (applied_layers) still exist in the Jin repository. If any referenced
/// layer has been deleted, this function returns the list of missing refs.
///
/// # Arguments
///
/// * `metadata` - The workspace metadata containing applied layer refs
/// * `repo` - The Jin repository
///
/// # Returns
///
/// - `Ok(Some(refs))` - List of layer refs that no longer exist
/// - `Ok(None)` - All referenced layers exist
/// - `Err(JinError)` - Error checking refs
fn detect_missing_commits(
    metadata: &WorkspaceMetadata,
    repo: &JinRepo,
) -> Result<Option<Vec<String>>> {
    let mut missing_refs = Vec::new();

    // Check if each applied layer ref exists
    for layer_name in &metadata.applied_layers {
        // Convert layer name to ref path
        // Layer names are stored like "mode/claude" or "scope/default"
        // We need to convert to "refs/jin/layers/mode/claude"
        let ref_path = format!("refs/jin/layers/{}", layer_name);

        if !repo.ref_exists(&ref_path) {
            missing_refs.push(ref_path);
        }
    }

    Ok(if missing_refs.is_empty() {
        None
    } else {
        Some(missing_refs)
    })
}

/// Detect if active context references deleted modes or scopes
///
/// This function checks if the active mode and scope stored in ProjectContext
/// still have valid refs in the Jin repository. If the active mode or scope
/// has been deleted, this function returns the invalid ref name.
///
/// # Arguments
///
/// * `context` - The project context containing active mode/scope
/// * `repo` - The Jin repository
///
/// # Returns
///
/// - `Ok(Some(ref_name))` - Name of invalid ref (e.g., "mode:production")
/// - `Ok(None)` - All active context refs are valid
/// - `Err(JinError)` - Error checking refs
fn detect_invalid_context(context: &ProjectContext, repo: &JinRepo) -> Result<Option<String>> {
    // Check active mode exists
    if let Some(mode) = &context.mode {
        let mode_ref = format!("refs/jin/layers/mode/{}", mode);
        if !repo.ref_exists(&mode_ref) {
            return Ok(Some(format!("mode:{}", mode)));
        }
    }

    // Check active scope exists (scope ref path depends on mode)
    if let Some(scope) = &context.scope {
        let scope_ref = if let Some(mode) = &context.mode {
            format!("refs/jin/layers/mode/{}/scope/{}", mode, scope)
        } else {
            format!("refs/jin/layers/scope/{}", scope)
        };

        if !repo.ref_exists(&scope_ref) {
            return Ok(Some(format!("scope:{}", scope)));
        }
    }

    Ok(None)
}

/// Describe the active context for error messages
///
/// This helper function formats the active context (mode/scope/project)
/// into a human-readable string for use in error messages.
fn describe_context(context: &ProjectContext) -> String {
    let parts: Vec<&str> = vec![
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    if parts.is_empty() {
        "no active context".to_string()
    } else {
        parts.join("/")
    }
}

/// Validate that the workspace is properly attached to valid layer commits
///
/// This function checks three detachment conditions:
/// 1. Workspace files have been modified outside of Jin operations
/// 2. Workspace metadata references non-existent layer commits
/// 3. Active context references deleted modes/scopes
///
/// If any condition is detected, returns a `DetachedWorkspace` error with
/// actionable recovery hints.
///
/// # Arguments
///
/// * `context` - The project context containing active mode/scope
/// * `repo` - The Jin repository
///
/// # Returns
///
/// - `Ok(())` - Workspace is properly attached
/// - `Err(JinError::DetachedWorkspace)` - Workspace is detached with details
/// - `Err(JinError)` - Other error (e.g., loading metadata)
pub fn validate_workspace_attached(context: &ProjectContext, repo: &JinRepo) -> Result<()> {
    // Fresh workspace - no metadata means no attachment to validate
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => return Ok(()),
        Err(e) => return Err(e),
    };

    // Condition 1: File mismatch (most specific - check first)
    if let Some(modified_files) = detect_file_mismatch(&metadata, repo)? {
        let workspace_commit = repo
            .inner()
            .head()
            .and_then(|h| {
                h.target()
                    .ok_or_else(|| git2::Error::from_str("HEAD has no target"))
            })
            .map(|t| t.to_string())
            .ok();

        return Err(JinError::DetachedWorkspace {
            workspace_commit,
            expected_layer_ref: format!("active context ({})", describe_context(context)),
            details: format!(
                "Workspace files have been modified outside of Jin operations. Modified files:\n  {}",
                modified_files
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            ),
            recovery_hint: "Run 'jin apply' to restore from active context".to_string(),
        });
    }

    // Condition 2: Missing commits/refs
    if let Some(missing_refs) = detect_missing_commits(&metadata, repo)? {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: None,
            expected_layer_ref: "<unknown>".to_string(),
            details: format!(
                "Workspace metadata references layers that no longer exist. Missing refs:\n  {}",
                missing_refs.join("\n  ")
            ),
            recovery_hint: "Run 'jin apply' to rebuild from current active context".to_string(),
        });
    }

    // Condition 3: Invalid context
    if let Some(invalid_ref) = detect_invalid_context(context, repo)? {
        let workspace_commit = repo
            .inner()
            .head()
            .and_then(|h| {
                h.target()
                    .ok_or_else(|| git2::Error::from_str("HEAD has no target"))
            })
            .map(|t| t.to_string())
            .ok();

        return Err(JinError::DetachedWorkspace {
            workspace_commit,
            expected_layer_ref: invalid_ref.clone(),
            details: format!(
                "Active context references a mode or scope that no longer exists: {}",
                invalid_ref
            ),
            recovery_hint:
                "Run 'jin mode activate <valid-mode>' or 'jin scope activate <valid-scope>'"
                    .to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_file_success() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, b"content").unwrap();

        let content = read_file(&file).unwrap();
        assert_eq!(content, b"content");
    }

    #[test]
    fn test_read_file_not_found() {
        let result = read_file(Path::new("/nonexistent/file.txt"));
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_is_symlink_false_for_regular_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();

        assert!(!is_symlink(&file).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn test_is_symlink_true_for_symlink() {
        use std::os::unix::fs::symlink;
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();
        let link = temp.path().join("link.txt");
        symlink(&file, &link).unwrap();

        assert!(is_symlink(&link).unwrap());
    }

    #[test]
    fn test_walk_directory() {
        let temp = TempDir::new().unwrap();

        // Create directory structure
        let subdir = temp.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        std::fs::write(temp.path().join("file1.txt"), b"1").unwrap();
        std::fs::write(temp.path().join("file2.txt"), b"2").unwrap();
        std::fs::write(subdir.join("nested.txt"), b"3").unwrap();

        let files = walk_directory(temp.path()).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_walk_empty_directory() {
        let temp = TempDir::new().unwrap();
        let files = walk_directory(temp.path()).unwrap();
        assert!(files.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_get_file_mode_regular() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();

        assert_eq!(get_file_mode(&file), 0o100644);
    }

    #[cfg(unix)]
    #[test]
    fn test_get_file_mode_executable() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("script.sh");
        std::fs::write(&file, b"#!/bin/bash").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(get_file_mode(&file), 0o100755);
    }

    #[test]
    fn test_is_git_tracked_no_repo() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();

        // Use absolute path - git2 will search for repo from file's parent
        // Since there's no Git repo in the temp directory, it should return false
        let result = is_git_tracked(&file);

        assert!(!result.unwrap());
    }

    // Tests for workspace validation functions

    #[test]
    fn test_describe_context_empty() {
        let context = ProjectContext::default();
        assert_eq!(describe_context(&context), "no active context");
    }

    #[test]
    fn test_describe_context_mode_only() {
        let context = ProjectContext {
            mode: Some("development".to_string()),
            ..Default::default()
        };
        assert_eq!(describe_context(&context), "development");
    }

    #[test]
    fn test_describe_context_mode_and_scope() {
        let context = ProjectContext {
            mode: Some("development".to_string()),
            scope: Some("backend".to_string()),
            ..Default::default()
        };
        assert_eq!(describe_context(&context), "development/backend");
    }

    #[test]
    fn test_describe_context_full() {
        let context = ProjectContext {
            mode: Some("development".to_string()),
            scope: Some("backend".to_string()),
            project: Some("myproject".to_string()),
            last_updated: Some("2024-01-01".to_string()),
            ..Default::default()
        };
        assert_eq!(describe_context(&context), "development/backend/myproject");
    }

    #[test]
    fn test_validate_workspace_attached_fresh_workspace() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        // Create a test repo
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // No metadata exists - should return Ok(())
        let context = ProjectContext::default();
        let result = validate_workspace_attached(&context, &repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_file_mismatch_no_files() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let metadata = WorkspaceMetadata::new();
        let result = detect_file_mismatch(&metadata, &repo).unwrap();
        assert!(result.is_none()); // No files = no mismatch
    }

    #[test]
    fn test_detect_file_mismatch_missing_file() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let mut metadata = WorkspaceMetadata::new();
        metadata.add_file(PathBuf::from("nonexistent.txt"), "abc123".to_string());

        let result = detect_file_mismatch(&metadata, &repo).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_detect_file_mismatch_modified_file() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // Create a file and compute its hash
        let file_path = PathBuf::from("test.txt"); // Use relative path for consistency
        std::fs::write(&file_path, b"original content").unwrap();
        let original_content = std::fs::read(&file_path).unwrap();
        let original_oid = repo.inner().blob(&original_content).unwrap();
        let original_hash = original_oid.to_string();

        // Create metadata with original hash
        let mut metadata = WorkspaceMetadata::new();
        metadata.add_file(file_path.clone(), original_hash);

        // Modify the file
        std::fs::write(&file_path, b"modified content").unwrap();

        // Detect mismatch
        let result = detect_file_mismatch(&metadata, &repo).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_detect_file_mismatch_no_mismatch() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // Create a file and compute its hash
        let file_path = PathBuf::from("test.txt"); // Use relative path for consistency
        std::fs::write(&file_path, b"content").unwrap();
        let content = std::fs::read(&file_path).unwrap();
        let oid = repo.inner().blob(&content).unwrap();
        let hash = oid.to_string();

        // Create metadata with matching hash
        let mut metadata = WorkspaceMetadata::new();
        metadata.add_file(file_path.clone(), hash);

        // No mismatch should be detected
        let result = detect_file_mismatch(&metadata, &repo).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_missing_commits_no_layers() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let metadata = WorkspaceMetadata::new();
        let result = detect_missing_commits(&metadata, &repo).unwrap();
        assert!(result.is_none()); // No layers = no missing commits
    }

    #[test]
    fn test_detect_missing_commits_all_exist() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // Create a test layer ref
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let oid = repo.inner().blob(b"test").unwrap();
        let tree_oid = {
            let mut builder = repo.inner().treebuilder(None).unwrap();
            builder.insert("test.txt", oid, 0o100644).unwrap();
            builder.write().unwrap()
        };
        let tree = repo.inner().find_tree(tree_oid).unwrap();
        let _commit_oid = repo
            .inner()
            .commit(
                Some("refs/jin/layers/mode/test"),
                &sig,
                &sig,
                "test",
                &tree,
                &[],
            )
            .unwrap();

        let mut metadata = WorkspaceMetadata::new();
        metadata.applied_layers = vec!["mode/test".to_string()];

        let result = detect_missing_commits(&metadata, &repo).unwrap();
        assert!(result.is_none()); // Layer exists = no missing commits
    }

    #[test]
    fn test_detect_missing_commits_layer_missing() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let mut metadata = WorkspaceMetadata::new();
        metadata.applied_layers = vec!["mode/nonexistent".to_string()];

        let result = detect_missing_commits(&metadata, &repo).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["refs/jin/layers/mode/nonexistent"]);
    }

    #[test]
    fn test_detect_invalid_context_no_mode_or_scope() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let context = ProjectContext::default();
        let result = detect_invalid_context(&context, &repo).unwrap();
        assert!(result.is_none()); // No mode/scope = no invalid context
    }

    #[test]
    fn test_detect_invalid_context_mode_exists() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // Create a test mode ref
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let oid = repo.inner().blob(b"test").unwrap();
        let tree_oid = {
            let mut builder = repo.inner().treebuilder(None).unwrap();
            builder.insert("test.txt", oid, 0o100644).unwrap();
            builder.write().unwrap()
        };
        let tree = repo.inner().find_tree(tree_oid).unwrap();
        repo.inner()
            .commit(
                Some("refs/jin/layers/mode/development"),
                &sig,
                &sig,
                "test",
                &tree,
                &[],
            )
            .unwrap();

        let context = ProjectContext {
            mode: Some("development".to_string()),
            ..Default::default()
        };

        let result = detect_invalid_context(&context, &repo).unwrap();
        assert!(result.is_none()); // Mode exists = no invalid context
    }

    #[test]
    fn test_detect_invalid_context_mode_missing() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        let context = ProjectContext {
            mode: Some("nonexistent".to_string()),
            ..Default::default()
        };

        let result = detect_invalid_context(&context, &repo).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "mode:nonexistent");
    }

    #[test]
    fn test_detect_invalid_context_scope_missing() {
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        // First create the mode ref (so mode exists)
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let oid = repo.inner().blob(b"test").unwrap();
        let tree_oid = {
            let mut builder = repo.inner().treebuilder(None).unwrap();
            builder.insert("test.txt", oid, 0o100644).unwrap();
            builder.write().unwrap()
        };
        let tree = repo.inner().find_tree(tree_oid).unwrap();
        repo.inner()
            .commit(
                Some("refs/jin/layers/mode/development"),
                &sig,
                &sig,
                "test",
                &tree,
                &[],
            )
            .unwrap();

        // Now check for missing scope
        let context = ProjectContext {
            mode: Some("development".to_string()),
            scope: Some("nonexistent".to_string()),
            ..Default::default()
        };

        let result = detect_invalid_context(&context, &repo).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "scope:nonexistent");
    }
}
