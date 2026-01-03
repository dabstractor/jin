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

    /// Detached workspace state - workspace doesn't match any valid layer configuration
    #[error(
        "Workspace is in a detached state.\n\
{details}\n\
\n\
Recovery: {recovery_hint}"
    )]
    DetachedWorkspace {
        /// The commit hash the workspace is currently on (if detectable)
        workspace_commit: Option<String>,
        /// The layer ref that was expected based on active context
        expected_layer_ref: String,
        /// Human-readable explanation of why detachment occurred
        details: String,
        /// Actionable recovery suggestion
        recovery_hint: String,
    },

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

    #[test]
    fn test_detached_workspace_error() {
        let err = JinError::DetachedWorkspace {
            workspace_commit: Some("abc123def".to_string()),
            expected_layer_ref: "refs/jin/layers/modes/claude/scopes/default".to_string(),
            details: "Workspace files have been modified outside of Jin operations".to_string(),
            recovery_hint: "Run 'jin reset --hard refs/jin/layers/modes/claude/scopes/default' to restore".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Workspace is in a detached state."));
        assert!(msg.contains("Workspace files have been modified outside of Jin operations"));
        assert!(msg.contains("Recovery:"));
        assert!(msg.contains("Run 'jin reset --hard refs/jin/layers/modes/claude/scopes/default' to restore"));
    }

    #[test]
    fn test_detached_workspace_error_no_commit() {
        let err = JinError::DetachedWorkspace {
            workspace_commit: None,
            expected_layer_ref: "<unknown>".to_string(),
            details: "Workspace metadata references commits that no longer exist".to_string(),
            recovery_hint: "Run 'jin apply' to rebuild from current active context".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Workspace is in a detached state."));
        assert!(msg.contains("Workspace metadata references commits that no longer exist"));
        assert!(msg.contains("Recovery:"));
        assert!(msg.contains("Run 'jin apply' to rebuild from current active context"));
    }

    #[test]
    fn test_detached_workspace_error_deleted_mode() {
        let err = JinError::DetachedWorkspace {
            workspace_commit: Some("xyz789".to_string()),
            expected_layer_ref: "mode:production".to_string(),
            details: "Active context references deleted mode: production".to_string(),
            recovery_hint: "Run 'jin mode activate <valid-mode>' to set a new active mode".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Workspace is in a detached state."));
        assert!(msg.contains("Active context references deleted mode: production"));
        assert!(msg.contains("Recovery:"));
        assert!(msg.contains("Run 'jin mode activate <valid-mode>' to set a new active mode"));
    }
}
