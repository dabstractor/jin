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
}
