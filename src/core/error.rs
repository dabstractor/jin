//! Error types for Jin

use thiserror::Error;

/// Unified error type for Jin operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JinError {
    /// IO and filesystem errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Git operations
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Parse errors (JSON, YAML, TOML, INI)
    #[error("Parse error in {format}: {message}")]
    Parse { format: String, message: String },

    /// Merge conflicts
    #[error("Merge conflict in {path}")]
    MergeConflict { path: String },

    /// Push rejected: local layer is behind remote
    #[error(
        "Push rejected: local layer '{layer}' is behind remote.\n\
The remote contains commits you don't have locally.\n\
Run 'jin pull' to merge remote changes, or use '--force' to overwrite.\n\
WARNING: --force may cause data loss!"
    )]
    BehindRemote { layer: String },

    /// Transaction failures
    #[error("Transaction failed: {0}")]
    Transaction(String),

    /// Layer routing errors
    #[error("Invalid layer: {0}")]
    InvalidLayer(String),

    /// Context errors
    #[error("No active {context_type}")]
    NoActiveContext { context_type: String },

    /// File not found
    #[error("File not found: {0}")]
    NotFound(String),

    /// Already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// File is tracked by Git
    #[error("File is tracked by Git: {path}. Use `jin import` instead.")]
    GitTracked { path: String },

    /// Path is a symlink
    #[error("Symlinks are not supported: {path}")]
    Symlink { path: String },

    /// Staging operation failed
    #[error("Staging failed for {path}: {reason}")]
    StagingFailed { path: String, reason: String },

    /// Not initialized
    #[error("Jin not initialized in this project")]
    NotInitialized,

    /// General errors
    #[error("{0}")]
    Other(String),
}

/// Result type alias using JinError
pub type Result<T> = std::result::Result<T, JinError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = JinError::NotInitialized;
        assert_eq!(err.to_string(), "Jin not initialized in this project");
    }

    #[test]
    fn test_config_error() {
        let err = JinError::Config("invalid value".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid value");
    }

    #[test]
    fn test_parse_error() {
        let err = JinError::Parse {
            format: "JSON".to_string(),
            message: "unexpected token".to_string(),
        };
        assert_eq!(err.to_string(), "Parse error in JSON: unexpected token");
    }

    #[test]
    fn test_merge_conflict_error() {
        let err = JinError::MergeConflict {
            path: ".claude/config.json".to_string(),
        };
        assert_eq!(err.to_string(), "Merge conflict in .claude/config.json");
    }

    #[test]
    fn test_no_active_context_error() {
        let err = JinError::NoActiveContext {
            context_type: "mode".to_string(),
        };
        assert_eq!(err.to_string(), "No active mode");
    }

    #[test]
    fn test_git_tracked_error() {
        let err = JinError::GitTracked {
            path: ".claude/config.json".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "File is tracked by Git: .claude/config.json. Use `jin import` instead."
        );
    }

    #[test]
    fn test_symlink_error() {
        let err = JinError::Symlink {
            path: "link.txt".to_string(),
        };
        assert_eq!(err.to_string(), "Symlinks are not supported: link.txt");
    }

    #[test]
    fn test_staging_failed_error() {
        let err = JinError::StagingFailed {
            path: "file.json".to_string(),
            reason: "cannot read file".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Staging failed for file.json: cannot read file"
        );
    }
}
