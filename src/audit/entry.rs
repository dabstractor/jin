//! Audit entry types for Jin operations

use serde::{Deserialize, Serialize};

/// Audit context information
///
/// Captures the active mode and scope at the time of commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    /// Currently active mode at time of commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_mode: Option<String>,
    /// Currently active scope at time of commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_scope: Option<String>,
}

/// Single audit log entry matching PRD Section 17 specification
///
/// # Example
///
/// ```json
/// {
///   "timestamp": "2025-10-19T15:04:02Z",
///   "user": "dustin",
///   "project": "ui-dashboard",
///   "mode": "claude",
///   "scope": "language:javascript",
///   "layer": 4,
///   "files": [".claude/config.json"],
///   "base_commit": "abc123",
///   "merge_commit": "def456",
///   "context": {
///     "active_mode": "claude",
///     "active_scope": "language:javascript"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// ISO 8601 timestamp (e.g., "2025-10-19T15:04:02Z")
    pub timestamp: String,
    /// User identity (from Git config or system)
    pub user: String,
    /// Project name (inferred from Git remote or context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Mode context (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Scope context (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Layer number (1-9) from Layer::precedence()
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<u8>,
    /// List of files affected by this operation
    pub files: Vec<String>,
    /// Base commit hash (parent commit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    /// Merge commit hash (newly created commit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_commit: Option<String>,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<AuditContext>,
}

impl AuditEntry {
    /// Create a new audit entry from commit information
    #[allow(clippy::too_many_arguments)]
    pub fn from_commit(
        user: String,
        project: Option<String>,
        mode: Option<String>,
        scope: Option<String>,
        layer: Option<u8>,
        files: Vec<String>,
        base_commit: Option<String>,
        merge_commit: String,
    ) -> Self {
        let context = if mode.is_some() || scope.is_some() {
            Some(AuditContext {
                active_mode: mode.clone(),
                active_scope: scope.clone(),
            })
        } else {
            None
        };

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            user,
            project,
            mode,
            scope,
            layer,
            files,
            base_commit,
            merge_commit: Some(merge_commit),
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: "2025-10-19T15:04:02Z".to_string(),
            user: "test@example.com".to_string(),
            project: Some("ui-dashboard".to_string()),
            mode: Some("claude".to_string()),
            scope: Some("language:javascript".to_string()),
            layer: Some(4),
            files: vec![".claude/config.json".to_string()],
            base_commit: Some("abc123".to_string()),
            merge_commit: Some("def456".to_string()),
            context: Some(AuditContext {
                active_mode: Some("claude".to_string()),
                active_scope: Some("language:javascript".to_string()),
            }),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"timestamp\":\"2025-10-19T15:04:02Z\""));
        assert!(json.contains("\"user\":\"test@example.com\""));
        assert!(json.contains("\"project\":\"ui-dashboard\""));
        assert!(json.contains("\"mode\":\"claude\""));
        assert!(json.contains("\"scope\":\"language:javascript\""));
        assert!(json.contains("\"layer\":4"));
        assert!(json.contains("\"base_commit\":\"abc123\""));
        assert!(json.contains("\"merge_commit\":\"def456\""));
    }

    #[test]
    fn test_audit_entry_serialization_skips_none() {
        let entry = AuditEntry {
            timestamp: "2025-10-19T15:04:02Z".to_string(),
            user: "test@example.com".to_string(),
            project: None,
            mode: None,
            scope: None,
            layer: None,
            files: vec![],
            base_commit: None,
            merge_commit: Some("abc123".to_string()),
            context: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        // None values should be skipped
        assert!(!json.contains("\"project\":null"));
        assert!(!json.contains("\"mode\":null"));
        assert!(!json.contains("\"scope\":null"));
        assert!(!json.contains("\"layer\":null"));
        assert!(!json.contains("\"base_commit\":null"));
        assert!(!json.contains("\"context\":null"));
    }

    #[test]
    fn test_audit_entry_from_commit() {
        let entry = AuditEntry::from_commit(
            "user@example.com".to_string(),
            Some("my-project".to_string()),
            Some("claude".to_string()),
            Some("language:rust".to_string()),
            Some(7),
            vec!["config.json".to_string()],
            Some("parent123".to_string()),
            "commit456".to_string(),
        );

        assert_eq!(entry.user, "user@example.com");
        assert_eq!(entry.project, Some("my-project".to_string()));
        assert_eq!(entry.mode, Some("claude".to_string()));
        assert_eq!(entry.scope, Some("language:rust".to_string()));
        assert_eq!(entry.layer, Some(7));
        assert_eq!(entry.files, vec!["config.json".to_string()]);
        assert_eq!(entry.base_commit, Some("parent123".to_string()));
        assert_eq!(entry.merge_commit, Some("commit456".to_string()));
        assert!(entry.context.is_some());
        assert_eq!(
            entry.context.as_ref().unwrap().active_mode,
            Some("claude".to_string())
        );
        assert_eq!(
            entry.context.as_ref().unwrap().active_scope,
            Some("language:rust".to_string())
        );
    }

    #[test]
    fn test_audit_entry_from_commit_no_context() {
        let entry = AuditEntry::from_commit(
            "user@example.com".to_string(),
            None,
            None,
            None,
            None,
            vec!["file.txt".to_string()],
            None,
            "commit123".to_string(),
        );

        assert_eq!(entry.user, "user@example.com");
        assert!(entry.project.is_none());
        assert!(entry.mode.is_none());
        assert!(entry.scope.is_none());
        assert!(entry.layer.is_none());
        assert!(entry.base_commit.is_none());
        assert!(entry.context.is_none());
    }

    #[test]
    fn test_audit_context_serialization() {
        let context = AuditContext {
            active_mode: Some("claude".to_string()),
            active_scope: Some("language:javascript".to_string()),
        };

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("\"active_mode\":\"claude\""));
        assert!(json.contains("\"active_scope\":\"language:javascript\""));
    }

    #[test]
    fn test_audit_context_skips_none() {
        let context = AuditContext {
            active_mode: None,
            active_scope: None,
        };

        let json = serde_json::to_string(&context).unwrap();
        // Empty context serializes to empty object
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_audit_entry_deserialization() {
        let json_str = r#"{
            "timestamp": "2025-10-19T15:04:02Z",
            "user": "test@example.com",
            "project": "ui-dashboard",
            "mode": "claude",
            "scope": "language:javascript",
            "layer": 4,
            "files": [".claude/config.json"],
            "base_commit": "abc123",
            "merge_commit": "def456",
            "context": {
                "active_mode": "claude",
                "active_scope": "language:javascript"
            }
        }"#;

        let entry: AuditEntry = serde_json::from_str(json_str).unwrap();
        assert_eq!(entry.timestamp, "2025-10-19T15:04:02Z");
        assert_eq!(entry.user, "test@example.com");
        assert_eq!(entry.project, Some("ui-dashboard".to_string()));
        assert_eq!(entry.mode, Some("claude".to_string()));
        assert_eq!(entry.scope, Some("language:javascript".to_string()));
        assert_eq!(entry.layer, Some(4));
        assert_eq!(entry.files.len(), 1);
        assert_eq!(entry.base_commit, Some("abc123".to_string()));
        assert_eq!(entry.merge_commit, Some("def456".to_string()));
    }
}
