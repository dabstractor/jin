//! Workspace file operations for Jin staging
//!
//! This module provides utilities for reading files from the workspace
//! (project working directory) and checking file properties.

use crate::core::{JinError, Result};
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
}
