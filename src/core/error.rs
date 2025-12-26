//! Core error types for Jin operations.
//!
//! This module defines the comprehensive error hierarchy used throughout Jin.
//! All Jin operations return `Result<T>` where the error type is `JinError`.

use thiserror::Error;

/// The primary error type for all Jin operations.
///
/// `JinError` provides comprehensive error coverage for:
/// - Git operations (repository, refs, objects)
/// - Transaction system (conflicts, preparation, commits)
/// - Merge operations (conflicts, parse failures)
/// - File I/O (permissions, not found, unsupported types)
/// - Configuration (invalid config, validation)
/// - Layer management (invalid layers, routing errors)
/// - Workspace operations (dirty state, apply failures)
/// - Serialization (JSON, YAML, TOML, INI parsing)
///
/// # Error Categories
///
/// Errors are grouped by category for easy pattern matching:
/// - Library errors use `#[error(transparent)]` and `#[from]` for automatic conversion
/// - Structured variants capture relevant context (paths, names, etc.)
/// - All variants implement `Display` for user-friendly messages
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JinError {
    // ===== Library Errors (transparent forwarding) =====
    /// Git operation error from libgit2.
    #[error(transparent)]
    Git(#[from] git2::Error),

    /// Standard I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON parse/serialize error.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// YAML parse/serialize error.
    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml_ng::Error),

    /// TOML parse/serialize error.
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// INI parse error.
    #[error("INI parse error: {0}")]
    IniParse(String),

    /// CLI argument parsing error.
    #[error("Command-line argument error: {0}")]
    Clap(#[from] clap::Error),

    // ===== Git Operation Errors =====
    /// Repository not found at the specified path.
    #[error("Repository not found at: {path}")]
    RepoNotFound { path: String },

    /// Git reference not found in a layer.
    #[error("Ref not found: '{name}' in layer '{layer}'")]
    RefNotFound { name: String, layer: String },

    /// Git reference already exists in a layer.
    #[error("Ref already exists: '{name}' in layer '{layer}'")]
    RefExists { name: String, layer: String },

    /// Invalid Git repository state.
    #[error("Invalid Git repository state: {message}")]
    InvalidGitState { message: String },

    /// Operation not supported on bare repositories.
    #[error("Bare repository not supported: {path}")]
    BareRepo { path: String },

    // ===== Transaction Errors =====
    /// Transaction conflict detected.
    #[error("Transaction conflict: {conflict}")]
    TransactionConflict { conflict: String },

    /// Transaction preparation failed.
    #[error("Transaction prepare failed: {source}")]
    PrepareFailed {
        /// The underlying error that caused the failure.
        #[source]
        source: Box<JinError>,
        /// Files that were staged in the transaction.
        files: Vec<String>,
    },

    /// Transaction commit failed.
    #[error("Transaction commit failed: {source}")]
    CommitFailed {
        /// The underlying error that caused the failure.
        #[source]
        source: Box<JinError>,
        /// Files that were staged in the transaction.
        files: Vec<String>,
    },

    // ===== Merge Errors =====
    /// Merge conflict detected in a file.
    #[error("Merge conflict in file: {file_path}")]
    MergeConflict { file_path: String },

    /// Merge operation failed for a file.
    #[error("Merge failed for file: {file_path}: {reason}")]
    MergeFailed { file_path: String, reason: String },

    /// File format not supported for merge operations.
    #[error("File format not supported for merge: {format}")]
    UnsupportedFormat { format: String },

    // ===== File Operation Errors =====
    /// File not found at the specified path.
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    /// File is not tracked by Git.
    #[error("File is not tracked by Git: {path}")]
    FileNotTracked { path: String },

    /// Permission denied for the specified path.
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    /// Symlinks are not supported by Jin.
    #[error("Symlinks are not supported: {path}")]
    SymlinkNotSupported { path: String },

    /// Binary files are not supported by Jin.
    #[error("Binary files are not supported: {path}")]
    BinaryFileNotSupported { path: String },

    /// Git submodules are not tracked by Jin.
    #[error("Submodules are not tracked by Jin: {path}")]
    SubmoduleNotSupported { path: String },

    // ===== Configuration Errors =====
    /// Generic configuration error.
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Invalid configuration detected.
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// Validation failed.
    #[error("Validation failed: {message}")]
    ValidationError { message: String },

    // ===== Layer Management Errors =====
    /// Invalid layer specified.
    #[error("Invalid layer: {name}")]
    InvalidLayer { name: String },

    /// Layer routing error.
    #[error("Layer routing error: {message}")]
    LayerRoutingError { message: String },

    /// Mode not found.
    #[error("Mode not found: {mode}")]
    ModeNotFound { mode: String },

    /// Scope not found.
    #[error("Scope not found: {scope}")]
    ScopeNotFound { scope: String },

    // ===== Workspace Errors =====
    /// Workspace has uncommitted changes.
    #[error("Workspace dirty: {files:?}")]
    WorkspaceDirty { files: Vec<String> },

    /// Workspace apply operation failed.
    #[error("Workspace apply failed: {reason}")]
    WorkspaceApplyFailed { reason: String },

    /// Git ignore update failed.
    #[error("Git ignore update failed: {message}")]
    GitignoreError { message: String },

    // ===== Generic Error =====
    /// Simple string message for generic errors.
    #[error("{0}")]
    Message(String),
}

/// Convenience type alias for Result with JinError.
///
/// This allows writing `Result<T>` instead of `std::result::Result<T, JinError>`.
pub type Result<T> = std::result::Result<T, JinError>;

// ===== Exit Code Mapping =====

impl From<JinError> for i32 {
    fn from(err: JinError) -> Self {
        match &err {
            // Not found errors (3)
            JinError::RepoNotFound { .. }
            | JinError::RefNotFound { .. }
            | JinError::FileNotFound { .. }
            | JinError::FileNotTracked { .. }
            | JinError::ModeNotFound { .. }
            | JinError::ScopeNotFound { .. } => 3,

            // Conflict errors (4)
            JinError::TransactionConflict { .. } | JinError::MergeConflict { .. } => 4,

            // Permission errors (5)
            JinError::PermissionDenied { .. } => 5,

            // Invalid argument errors (2)
            JinError::InvalidConfig { .. }
            | JinError::InvalidLayer { .. }
            | JinError::ValidationError { .. }
            | JinError::Clap(_) => 2,

            // General error (1)
            _ => 1,
        }
    }
}

// ===== Helper Methods =====

impl JinError {
    /// Returns true if this error is transient and potentially retryable.
    ///
    /// Transient errors include file locks and concurrent modifications
    /// that might succeed if retried.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if error.is_retryable() {
    ///     // Retry the operation after a delay
    /// }
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            JinError::Git(err) if matches!(
                err.code(),
                git2::ErrorCode::Locked | git2::ErrorCode::Modified
            )
        )
    }

    /// Returns true if this is a user error (not a system error).
    ///
    /// User errors are caused by invalid input, configuration, or usage
    /// rather than system failures. These typically don't indicate bugs.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if error.is_user_error() {
    ///     eprintln!("Usage error: {}", error);
    /// } else {
    ///     eprintln!("System error: {}", error);
    /// }
    /// ```
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            JinError::InvalidConfig { .. }
                | JinError::InvalidLayer { .. }
                | JinError::ValidationError { .. }
                | JinError::Clap(_)
        )
    }

    /// Get the exit code for this error.
    ///
    /// This provides explicit access to the exit code that would be
    /// returned by `Into<JinError>::into()`.
    ///
    /// # Exit Codes
    ///
    /// - `0`: Success (never returned from error)
    /// - `1`: General error
    /// - `2`: Invalid argument/usage error
    /// - `3`: Not found (repository, ref, file, mode, scope)
    /// - `4`: Conflict (transaction, merge)
    /// - `5`: Permission denied
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match operation() {
    ///     Ok(_) => std::process::exit(0),
    ///     Err(e) => std::process::exit(e.exit_code()),
    /// }
    /// ```
    pub fn exit_code(&self) -> i32 {
        match self {
            // Not found errors (3)
            JinError::RepoNotFound { .. }
            | JinError::RefNotFound { .. }
            | JinError::FileNotFound { .. }
            | JinError::FileNotTracked { .. }
            | JinError::ModeNotFound { .. }
            | JinError::ScopeNotFound { .. } => 3,

            // Conflict errors (4)
            JinError::TransactionConflict { .. } | JinError::MergeConflict { .. } => 4,

            // Permission errors (5)
            JinError::PermissionDenied { .. } => 5,

            // Invalid argument errors (2)
            JinError::InvalidConfig { .. }
            | JinError::InvalidLayer { .. }
            | JinError::ValidationError { .. }
            | JinError::Clap(_) => 2,

            // General error (1)
            _ => 1,
        }
    }
}

// ===== String Conversions =====

impl From<&str> for JinError {
    fn from(s: &str) -> Self {
        JinError::Message(s.to_owned())
    }
}

impl From<String> for JinError {
    fn from(s: String) -> Self {
        JinError::Message(s)
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        // Test structured error variants display correctly
        let err = JinError::RepoNotFound {
            path: "/tmp/notexist".to_string(),
        };
        assert!(err.to_string().contains("Repository not found"));
        assert!(err.to_string().contains("/tmp/notexist"));

        let err = JinError::RefNotFound {
            name: "main".to_string(),
            layer: "project/ui".to_string(),
        };
        assert!(err.to_string().contains("Ref not found"));
        assert!(err.to_string().contains("main"));
        assert!(err.to_string().contains("project/ui"));

        let err = JinError::MergeConflict {
            file_path: ".claude/config.json".to_string(),
        };
        assert!(err.to_string().contains("Merge conflict"));
        assert!(err.to_string().contains(".claude/config.json"));

        let err = JinError::ConfigError {
            message: "invalid mode".to_string(),
        };
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("invalid mode"));
    }

    #[test]
    fn test_exit_code_mapping() {
        // Not found errors -> 3
        assert_eq!(JinError::RepoNotFound { path: "x".into() }.exit_code(), 3);
        assert_eq!(
            JinError::RefNotFound {
                name: "x".into(),
                layer: "y".into()
            }
            .exit_code(),
            3
        );
        assert_eq!(JinError::FileNotFound { path: "x".into() }.exit_code(), 3);
        assert_eq!(JinError::ModeNotFound { mode: "x".into() }.exit_code(), 3);
        assert_eq!(JinError::ScopeNotFound { scope: "x".into() }.exit_code(), 3);

        // Conflict errors -> 4
        assert_eq!(
            JinError::TransactionConflict {
                conflict: "x".into()
            }
            .exit_code(),
            4
        );
        assert_eq!(
            JinError::MergeConflict {
                file_path: "x".into()
            }
            .exit_code(),
            4
        );

        // Permission errors -> 5
        assert_eq!(
            JinError::PermissionDenied { path: "x".into() }.exit_code(),
            5
        );

        // Invalid argument errors -> 2
        assert_eq!(
            JinError::InvalidConfig {
                message: "x".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(JinError::InvalidLayer { name: "x".into() }.exit_code(), 2);
        assert_eq!(
            JinError::ValidationError {
                message: "x".into()
            }
            .exit_code(),
            2
        );

        // General errors -> 1
        assert_eq!(JinError::Message("test".into()).exit_code(), 1);
        assert_eq!(JinError::WorkspaceDirty { files: vec![] }.exit_code(), 1);
    }

    #[test]
    fn test_is_user_error() {
        // User errors
        assert!(JinError::InvalidConfig {
            message: "x".into()
        }
        .is_user_error());
        assert!(JinError::InvalidLayer { name: "x".into() }.is_user_error());
        assert!(JinError::ValidationError {
            message: "x".into()
        }
        .is_user_error());

        // Not user errors
        assert!(!JinError::RepoNotFound { path: "x".into() }.is_user_error());
        assert!(!JinError::MergeConflict {
            file_path: "x".into()
        }
        .is_user_error());
        assert!(!JinError::PermissionDenied { path: "x".into() }.is_user_error());
        assert!(!JinError::Message("test".into()).is_user_error());
    }

    #[test]
    fn test_string_conversions() {
        // From &str
        let err: JinError = "test message".into();
        assert!(matches!(err, JinError::Message(_)));
        assert_eq!(err.to_string(), "test message");

        // From String
        let err: JinError = String::from("owned message").into();
        assert!(matches!(err, JinError::Message(_)));
        assert_eq!(err.to_string(), "owned message");
    }

    #[test]
    fn test_from_i32_conversion() {
        // Test that From<JinError> for i32 works
        let err = JinError::RepoNotFound {
            path: "/path".into(),
        };
        let code: i32 = err.into();
        assert_eq!(code, 3);
    }
}
