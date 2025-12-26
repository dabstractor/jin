//! Pre-commit validation for staged entries.
//!
//! This module provides validation functions that ensure all staged files
//! meet Jin's requirements before being committed to layers. Validations
//! check for symlinks, binary files, git-tracked files, and file size limits.
//!
//! # Validation Rules
//!
//! - **Symlinks**: Not supported (Jin manages files directly)
//! - **Binary files**: Not supported (text-based config only)
//! - **Git-tracked files**: Not allowed (Jin is separate from project Git)
//! - **File size**: Maximum 10MB per file
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::commit::validate::validate_staging_index;
//! use jin_glm::staging::StagingIndex;
//!
//! let staging = StagingIndex::new();
//! let workspace_root = Path::new("/my/project");
//!
//! let result = validate_staging_index(&staging, workspace_root)?;
//! if !result.errors.is_empty() {
//!     eprintln!("Validation errors: {:?}", result.errors);
//! }
//! ```

use crate::core::error::{JinError, Result};
use crate::staging::{StagedEntry, StagingIndex};
use std::path::Path;

// ===== VALIDATION RESULT =====

/// Pre-commit validation result.
///
/// Contains validation errors and warnings discovered during
/// the validation phase of the commit pipeline.
///
/// # Fields
///
/// - `errors`: Fatal errors that prevent commit
/// - `warnings`: Warnings that don't prevent commit
///
/// # Examples
///
/// ```ignore
/// if !result.errors.is_empty() {
///     for error in &result.errors {
///         eprintln!("Error: {} - {:?}", error.path, error.error_type);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Fatal errors that prevent commit
    pub errors: Vec<ValidationError>,
    /// Warnings that don't prevent commit
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Returns true if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    /// Creates a new empty validation result.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

// ===== VALIDATION ERROR =====

/// Validation error for a specific file.
///
/// Represents a validation failure for a single file that prevents
/// the commit from proceeding.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// File path that failed validation
    pub path: std::path::PathBuf,
    /// Type of validation failure
    pub error_type: ValidationErrorType,
}

impl ValidationError {
    /// Creates a new validation error.
    pub fn new(path: std::path::PathBuf, error_type: ValidationErrorType) -> Self {
        Self { path, error_type }
    }
}

/// Type of validation error that occurred.
///
/// Each variant represents a specific validation rule that was violated.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    /// File is a symbolic link (not supported)
    SymlinkNotSupported,
    /// File contains binary content (not supported)
    BinaryFileNotSupported,
    /// File is tracked by Git in the workspace
    GitTrackedFile,
    /// File exceeds size limit
    FileSizeLimit,
}

// ===== VALIDATION WARNING =====

/// Validation warning for a specific file.
///
/// Represents a non-fatal issue that doesn't prevent commit
/// but may be worth noting to the user.
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// File path that generated a warning
    pub path: std::path::PathBuf,
    /// Warning message
    pub message: String,
}

impl ValidationWarning {
    /// Creates a new validation warning.
    pub fn new(path: std::path::PathBuf, message: String) -> Self {
        Self { path, message }
    }
}

// ===== VALIDATION FUNCTIONS =====

/// Validates all entries in the staging index.
///
/// Returns ValidationResult containing errors (fatal) and warnings.
/// Only returns Err if validation itself fails (not for validation errors).
///
/// # Arguments
///
/// * `staging` - The staging index to validate
/// * `workspace_root` - Path to the project workspace root
///
/// # Returns
///
/// - `Ok(ValidationResult)` - Validation completed (check errors field)
/// - `Err(JinError)` - Validation process failed
///
/// # Examples
///
/// ```ignore
/// let result = validate_staging_index(&staging, &workspace_root)?;
/// if result.errors.is_empty() {
///     println!("All files validated successfully");
/// } else {
///     eprintln!("Validation failed:");
///     for error in &result.errors {
///         eprintln!("  - {}: {:?}", error.path.display(), error.error_type);
///     }
/// }
/// ```
pub fn validate_staging_index(
    staging: &StagingIndex,
    workspace_root: &Path,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    for entry in staging.all_entries() {
        if let Err(e) = validate_entry(entry, workspace_root, &mut result) {
            // Validation process failed (not a validation error)
            return Err(e);
        }
    }

    Ok(result)
}

/// Validates a single staged entry.
///
/// Checks for symlinks, binary files, git-tracked files, and size limits.
///
/// # Arguments
///
/// * `entry` - The staged entry to validate
/// * `workspace_root` - Path to the project workspace root
/// * `result` - Validation result to accumulate errors into
///
/// # Returns
///
/// `Ok(())` if validation checks completed (errors accumulated in result)
fn validate_entry(
    entry: &StagedEntry,
    workspace_root: &Path,
    result: &mut ValidationResult,
) -> Result<()> {
    let full_path = workspace_root.join(&entry.path);

    // Check for symlink
    if let Err(_e) = check_symlink(&full_path, &entry.path) {
        result.errors.push(ValidationError::new(
            entry.path.clone(),
            ValidationErrorType::SymlinkNotSupported,
        ));
        return Ok(()); // Continue checking other files
    }

    // Check for binary file
    if let Err(_e) = check_binary_file(&full_path, &entry.path) {
        result.errors.push(ValidationError::new(
            entry.path.clone(),
            ValidationErrorType::BinaryFileNotSupported,
        ));
        return Ok(());
    }

    // Check if git-tracked
    if let Err(_e) = check_git_tracked(workspace_root, &entry.path) {
        result.errors.push(ValidationError::new(
            entry.path.clone(),
            ValidationErrorType::GitTrackedFile,
        ));
        return Ok(());
    }

    // Check file size
    if let Err(_e) = check_file_size(&full_path, &entry.path, 10_000_000) {
        result.errors.push(ValidationError::new(
            entry.path.clone(),
            ValidationErrorType::FileSizeLimit,
        ));
    }

    Ok(())
}

/// Checks if a path is a symlink.
///
/// # Arguments
///
/// * `path` - Full path to the file
/// * `relative_path` - Relative path for error messages
///
/// # Returns
///
/// - `Ok(())` - File is not a symlink
/// - `Err(JinError::SymlinkNotSupported)` - File is a symlink
fn check_symlink(path: &Path, relative_path: &std::path::Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_e| JinError::FileNotFound {
        path: relative_path.display().to_string(),
    })?;

    if metadata.file_type().is_symlink() {
        return Err(JinError::SymlinkNotSupported {
            path: relative_path.display().to_string(),
        });
    }

    Ok(())
}

/// Checks if a file contains binary content.
///
/// Uses a simple heuristic: checks for null bytes in the file content.
///
/// # Arguments
///
/// * `path` - Full path to the file
/// * `relative_path` - Relative path for error messages
///
/// # Returns
///
/// - `Ok(())` - File is not binary
/// - `Err(JinError::BinaryFileNotSupported)` - File is binary
fn check_binary_file(path: &Path, relative_path: &std::path::Path) -> Result<()> {
    let content = std::fs::read(path)?;

    // Check for null bytes (simple binary detection)
    if content.iter().any(|&b| b == 0x00) {
        return Err(JinError::BinaryFileNotSupported {
            path: relative_path.display().to_string(),
        });
    }

    Ok(())
}

/// Checks if a file is tracked by Git in the workspace.
///
/// Jin files must not be tracked by the project's Git repository.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
/// * `relative_path` - Relative path for error messages
///
/// # Returns
///
/// - `Ok(())` - File is not git-tracked
/// - `Err(JinError::ValidationError)` - File is git-tracked
pub fn check_git_tracked(workspace_root: &Path, relative_path: &std::path::Path) -> Result<()> {
    // Open the workspace's Git repo (not Jin's repo)
    let git_repo = git2::Repository::open(workspace_root)
        .map_err(|_| JinError::Message("Not a Git repository".to_string()))?;

    // Check file status - use Status::is_index_new() to check if it's tracked
    // A file is tracked if it's in the index (WT_NEW means it's new and untracked)
    let status = match git_repo.status_file(relative_path) {
        Ok(s) => s,
        Err(_) => {
            // If we can't get status, file might not exist or repo might be bare
            // In this case, assume it's not tracked
            return Ok(());
        }
    };

    // File is tracked if it's NOT in the "new" state (WT_NEW means untracked)
    // Check if the file has any status other than being a new worktree file
    if status.contains(git2::Status::WT_NEW) {
        // File is new/untracked
        return Ok(());
    }

    // If file has any status bits set (other than WT_NEW), it's tracked
    if status != git2::Status::empty() {
        return Err(JinError::ValidationError {
            message: format!("File is tracked by Git: {}", relative_path.display()),
        });
    }

    Ok(())
}

/// Checks if a file exceeds the size limit.
///
/// # Arguments
///
/// * `path` - Full path to the file
/// * `relative_path` - Relative path for error messages
/// * `max_size` - Maximum file size in bytes
///
/// # Returns
///
/// - `Ok(())` - File is within size limit
/// - `Err(JinError::ValidationError)` - File exceeds limit
fn check_file_size(path: &Path, relative_path: &std::path::Path, max_size: u64) -> Result<()> {
    let metadata = std::fs::metadata(path)?;

    if metadata.len() > max_size {
        return Err(JinError::ValidationError {
            message: format!(
                "File size {} bytes exceeds limit of {} bytes",
                metadata.len(),
                max_size
            ),
        });
    }

    Ok(())
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ===== ValidationResult Tests =====

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validation_result_is_valid_true() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validation_result_is_valid_false() {
        let mut result = ValidationResult::new();
        result.errors.push(ValidationError::new(
            std::path::PathBuf::from("test.txt"),
            ValidationErrorType::SymlinkNotSupported,
        ));
        assert!(!result.is_valid());
    }

    // ===== ValidationError Tests =====

    #[test]
    fn test_validation_error_new() {
        let error = ValidationError::new(
            std::path::PathBuf::from("config.json"),
            ValidationErrorType::BinaryFileNotSupported,
        );
        assert_eq!(error.path, std::path::PathBuf::from("config.json"));
        assert_eq!(
            error.error_type,
            ValidationErrorType::BinaryFileNotSupported
        );
    }

    // ===== ValidationWarning Tests =====

    #[test]
    fn test_validation_warning_new() {
        let warning = ValidationWarning::new(
            std::path::PathBuf::from("config.json"),
            "File is very large".to_string(),
        );
        assert_eq!(warning.path, std::path::PathBuf::from("config.json"));
        assert_eq!(warning.message, "File is very large");
    }

    // ===== check_symlink Tests =====

    #[test]
    fn test_check_symlink_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("regular.txt");
        fs::write(&file_path, b"content").unwrap();

        let result = check_symlink(&file_path, std::path::Path::new("regular.txt"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_symlink_symlink_file() {
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        fs::write(&target_path, b"content").unwrap();

        let link_path = temp_dir.path().join("link.txt");
        std::os::unix::fs::symlink(&target_path, &link_path).unwrap();

        let result = check_symlink(&link_path, std::path::Path::new("link.txt"));
        assert!(matches!(result, Err(JinError::SymlinkNotSupported { .. })));
    }

    #[test]
    fn test_check_symlink_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = check_symlink(&file_path, std::path::Path::new("nonexistent.txt"));
        assert!(matches!(result, Err(JinError::FileNotFound { .. })));
    }

    // ===== check_binary_file Tests =====

    #[test]
    fn test_check_binary_file_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("text.txt");
        fs::write(&file_path, b"This is plain text").unwrap();

        let result = check_binary_file(&file_path, std::path::Path::new("text.txt"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_binary_file_binary_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.bin");
        let content = b"Text\x00with\x00null\x00bytes";
        fs::write(&file_path, content).unwrap();

        let result = check_binary_file(&file_path, std::path::Path::new("binary.bin"));
        assert!(matches!(
            result,
            Err(JinError::BinaryFileNotSupported { .. })
        ));
    }

    // ===== check_file_size Tests =====

    #[test]
    fn test_check_file_size_within_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("small.txt");
        fs::write(&file_path, b"small content").unwrap();

        let result = check_file_size(&file_path, std::path::Path::new("small.txt"), 10_000_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_file_size_exceeds_limit() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        fs::write(&file_path, b"x".repeat(1000)).unwrap();

        let result = check_file_size(&file_path, std::path::Path::new("large.txt"), 100);
        assert!(result.is_err());
    }
}
