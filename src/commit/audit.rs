//! Audit trail logging for Jin commits.
//!
//! This module provides audit logging for all Jin commit operations.
//! Each commit is logged with complete metadata including user, project,
//! layer, files, and commit information.
//!
//! # Audit Log Storage
//!
//! Audit logs are stored in `~/.jin/repo/.audit/` directory (separate from
//! the project workspace). One JSON entry per line, append-only.
//!
//! # Audit Log Format
//!
//! Each audit entry is a JSON object:
//!
//! ```json
//! {
//!   "timestamp": "2025-10-19T15:04:02Z",
//!   "user": "dustin",
//!   "project": "ui-dashboard",
//!   "mode": null,
//!   "scope": null,
//!   "layer": 7,
//!   "files": [".claude/config.json"],
//!   "base_commit": "abc123...",
//!   "merge_commit": "def456...",
//!   "context": {
//!     "active_mode": null,
//!     "active_scope": null
//!   }
//! }
//! ```
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::commit::audit::AuditEntry;
//!
//! let entry = AuditEntry::new(
//!     "user".to_string(),
//!     "myproject".to_string(),
//!     None, // mode
//!     None, // scope
//!     Layer::ProjectBase { project: "myproject".to_string() },
//!     vec!["config.json".to_string()],
//!     base_oid,
//!     merge_oid,
//!     None, // active_mode
//!     None, // active_scope
//! );
//!
//! entry.save(&jin_repo_path)?;
//! ```

use crate::core::error::{JinError, Result};
use crate::core::Layer;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ===== AUDIT ENTRY =====

/// Audit log entry for a commit.
///
/// Records all information about a commit for debugging
/// and recovery purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the commit was made
    pub timestamp: DateTime<Utc>,
    /// User who made the commit (from git config)
    pub user: String,
    /// Project name
    pub project: String,
    /// Active mode at commit time (if any)
    pub mode: Option<String>,
    /// Active scope at commit time (if any)
    pub scope: Option<String>,
    /// Layer number that was committed to (1-9)
    pub layer: u8,
    /// Files that were committed
    pub files: Vec<String>,
    /// Base commit OID (parent)
    pub base_commit: String,
    /// New commit OID
    pub merge_commit: String,
    /// Active context at commit time
    pub context: AuditContext,
}

impl AuditEntry {
    /// Creates a new audit entry from commit information.
    ///
    /// # Arguments
    ///
    /// * `user` - User who made the commit
    /// * `project` - Project name
    /// * `mode` - Active mode at commit time (optional)
    /// * `scope` - Active scope at commit time (optional)
    /// * `layer` - Layer that was committed to
    /// * `files` - Files that were committed
    /// * `base_commit` - Parent commit OID
    /// * `merge_commit` - New commit OID
    /// * `active_mode` - Active mode from context (optional)
    /// * `active_scope` - Active scope from context (optional)
    ///
    /// # Returns
    ///
    /// A new audit entry with the current timestamp.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let entry = AuditEntry::new(
    ///     "user".to_string(),
    ///     "myproject".to_string(),
    ///     Some("claude".to_string()),
    ///     Some("python".to_string()),
    ///     Layer::ModeScope { mode: "claude".to_string(), scope: "python".to_string() },
    ///     vec!["config.json".to_string()],
    ///     parent_oid,
    ///     new_oid,
    ///     Some("claude".to_string()),
    ///     Some("python".to_string()),
    /// );
    /// ```
    pub fn new(
        user: String,
        project: String,
        mode: Option<String>,
        scope: Option<String>,
        layer: Layer,
        files: Vec<String>,
        base_commit: git2::Oid,
        merge_commit: git2::Oid,
        active_mode: Option<String>,
        active_scope: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            user,
            project,
            mode,
            scope,
            layer: layer.layer_number(),
            files,
            base_commit: base_commit.to_string(),
            merge_commit: merge_commit.to_string(),
            context: AuditContext {
                active_mode,
                active_scope,
            },
        }
    }

    /// Saves this audit entry to the audit log.
    ///
    /// Appends the entry as a JSON line to the appropriate audit log file
    /// (one file per day: `YYYY-MM-DD.log`).
    ///
    /// # Arguments
    ///
    /// * `jin_repo_path` - Path to the Jin repository (contains `.audit/` subdirectory)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Audit entry saved successfully
    /// - `Err(JinError)` - Failed to save
    ///
    /// # Examples
    ///
    /// ```ignore
    /// entry.save(&jin_repo.path())?;
    /// ```
    pub fn save(&self, jin_repo_path: &Path) -> Result<()> {
        let audit_dir = jin_repo_path.join(".audit");
        std::fs::create_dir_all(&audit_dir)?;

        // Append to audit log (one JSON per line)
        let filename = format!("{}", self.timestamp.format("%Y-%m-%d.log"));
        let audit_file = audit_dir.join(&filename);
        let line = serde_json::to_string(self)
            .map_err(|e| JinError::Message(format!("Failed to serialize audit entry: {}", e)))?;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&audit_file)?;

        use std::io::Write;
        writeln!(file, "{}", line)?;

        Ok(())
    }
}

// ===== AUDIT CONTEXT =====

/// Context information for audit log.
///
/// Captures the active mode and scope at the time of commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    /// Active mode at commit time
    pub active_mode: Option<String>,
    /// Active scope at commit time
    pub active_scope: Option<String>,
}

// ===== HELPER FUNCTIONS =====

/// Returns the audit directory path for a Jin repository.
///
/// # Arguments
///
/// * `jin_repo_path` - Path to the Jin repository
///
/// # Returns
///
/// The path to the `.audit/` subdirectory.
///
/// # Examples
///
/// ```ignore
/// let audit_dir = get_audit_dir(&jin_repo.path());
/// assert!(audit_dir.ends_with(".audit"));
/// ```
pub fn get_audit_dir(jin_repo_path: &Path) -> PathBuf {
    jin_repo_path.join(".audit")
}

/// Formats a timestamp as ISO 8601 string.
///
/// # Arguments
///
/// * `timestamp` - The timestamp to format
///
/// # Returns
///
/// ISO 8601 formatted string (e.g., "2025-12-26T10:00:00Z")
///
/// # Examples
///
/// ```ignore
/// let timestamp = Utc::now();
/// let formatted = format_timestamp(&timestamp);
/// assert!(formatted.contains("T"));
/// assert!(formatted.ends_with("Z"));
/// ```
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ===== LAYER HELPER =====

/// Helper trait to get layer number for audit logging.
pub trait LayerNumber {
    /// Returns the layer number (1-9) for audit logging.
    fn layer_number(&self) -> u8;
}

impl LayerNumber for Layer {
    /// Returns the layer number (1-9) for audit logging.
    fn layer_number(&self) -> u8 {
        match self {
            Layer::GlobalBase => 1,
            Layer::ModeBase { .. } => 2,
            Layer::ModeScope { .. } => 3,
            Layer::ModeScopeProject { .. } => 4,
            Layer::ModeProject { .. } => 5,
            Layer::ScopeBase { .. } => 6,
            Layer::ProjectBase { .. } => 7,
            Layer::UserLocal => 8,       // Not versioned
            Layer::WorkspaceActive => 9, // Not versioned
        }
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ===== AuditEntry::new Tests =====

    #[test]
    fn test_audit_entry_new() {
        let layer = Layer::GlobalBase;
        let files = vec!["config.json".to_string()];
        let base_oid = git2::Oid::zero();
        let merge_oid = git2::Oid::from_str("abc123").unwrap();

        let entry = AuditEntry::new(
            "user".to_string(),
            "myproject".to_string(),
            Some("claude".to_string()),
            Some("python".to_string()),
            layer,
            files,
            base_oid,
            merge_oid,
            Some("claude".to_string()),
            Some("python".to_string()),
        );

        assert_eq!(entry.user, "user");
        assert_eq!(entry.project, "myproject");
        assert_eq!(entry.mode, Some("claude".to_string()));
        assert_eq!(entry.scope, Some("python".to_string()));
        assert_eq!(entry.layer, 1);
        assert_eq!(entry.files.len(), 1);
        assert_eq!(entry.context.active_mode, Some("claude".to_string()));
        assert_eq!(entry.context.active_scope, Some("python".to_string()));
    }

    #[test]
    fn test_audit_entry_new_no_mode_scope() {
        let layer = Layer::ProjectBase {
            project: "myproject".to_string(),
        };
        let files = vec!["config.json".to_string()];
        let base_oid = git2::Oid::zero();
        let merge_oid = git2::Oid::from_str("abc123").unwrap();

        let entry = AuditEntry::new(
            "user".to_string(),
            "myproject".to_string(),
            None,
            None,
            layer,
            files,
            base_oid,
            merge_oid,
            None,
            None,
        );

        assert!(entry.mode.is_none());
        assert!(entry.scope.is_none());
        assert!(entry.context.active_mode.is_none());
        assert!(entry.context.active_scope.is_none());
    }

    // ===== AuditEntry::save Tests =====

    #[test]
    fn test_audit_entry_save() {
        let temp_dir = TempDir::new().unwrap();
        let audit_dir = temp_dir.path().join(".audit");

        let layer = Layer::GlobalBase;
        let files = vec!["config.json".to_string()];
        let base_oid = git2::Oid::zero();
        let merge_oid = git2::Oid::from_str("abc123").unwrap();

        let entry = AuditEntry::new(
            "user".to_string(),
            "myproject".to_string(),
            None,
            None,
            layer,
            files,
            base_oid,
            merge_oid,
            None,
            None,
        );

        entry.save(temp_dir.path()).unwrap();

        // Verify audit directory was created
        assert!(audit_dir.exists());
        assert!(audit_dir.is_dir());

        // Verify audit file was created
        let log_files: Vec<_> = fs::read_dir(&audit_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(log_files.len(), 1);

        // Verify file contains valid JSON
        let log_content = fs::read_to_string(log_files[0].path()).unwrap();
        let _parsed: AuditEntry = serde_json::from_str(&log_content.trim()).unwrap();
    }

    #[test]
    fn test_audit_entry_save_append() {
        let temp_dir = TempDir::new().unwrap();

        let layer = Layer::GlobalBase;
        let files = vec!["config.json".to_string()];
        let base_oid = git2::Oid::zero();
        let merge_oid = git2::Oid::from_str("abc123").unwrap();

        // Save first entry
        let entry1 = AuditEntry::new(
            "user".to_string(),
            "myproject".to_string(),
            None,
            None,
            layer.clone(),
            files.clone(),
            base_oid,
            merge_oid,
            None,
            None,
        );
        entry1.save(temp_dir.path()).unwrap();

        // Save second entry
        let entry2 = AuditEntry::new(
            "user".to_string(),
            "myproject".to_string(),
            None,
            None,
            layer,
            files,
            base_oid,
            merge_oid,
            None,
            None,
        );
        entry2.save(temp_dir.path()).unwrap();

        // Verify both entries are in the file
        let audit_dir = temp_dir.path().join(".audit");
        let log_files: Vec<_> = fs::read_dir(&audit_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(log_files.len(), 1);

        let log_content = fs::read_to_string(log_files[0].path()).unwrap();
        let lines: Vec<_> = log_content.lines().collect();
        assert_eq!(lines.len(), 2);

        // Both lines should be valid JSON
        for line in lines {
            let _parsed: AuditEntry = serde_json::from_str(line).unwrap();
        }
    }

    // ===== get_audit_dir Tests =====

    #[test]
    fn test_get_audit_dir() {
        let temp_dir = TempDir::new().unwrap();
        let audit_dir = get_audit_dir(temp_dir.path());

        assert!(audit_dir.ends_with(".audit"));
        assert!(audit_dir.starts_with(temp_dir.path()));
    }

    // ===== format_timestamp Tests =====

    #[test]
    fn test_format_timestamp() {
        let timestamp = DateTime::parse_from_rfc3339("2025-12-26T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let formatted = format_timestamp(&timestamp);

        assert_eq!(formatted, "2025-12-26T10:00:00Z");
    }

    #[test]
    fn test_format_timestamp_now() {
        let now = Utc::now();
        let formatted = format_timestamp(&now);

        // Verify format: YYYY-MM-DDTHH:MM:SSZ
        assert!(formatted.len() == 20);
        assert!(formatted.contains("T"));
        assert!(formatted.ends_with("Z"));
    }

    // ===== LayerNumber Tests =====

    #[test]
    fn test_layer_number_all_variants() {
        assert_eq!(Layer::GlobalBase.layer_number(), 1);
        assert_eq!(
            Layer::ModeBase {
                mode: "claude".to_string()
            }
            .layer_number(),
            2
        );
        assert_eq!(
            Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            }
            .layer_number(),
            3
        );
        assert_eq!(
            Layer::ModeScopeProject {
                mode: "claude".to_string(),
                scope: "python".to_string(),
                project: "myapp".to_string()
            }
            .layer_number(),
            4
        );
        assert_eq!(
            Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            }
            .layer_number(),
            5
        );
        assert_eq!(
            Layer::ScopeBase {
                scope: "python".to_string()
            }
            .layer_number(),
            6
        );
        assert_eq!(
            Layer::ProjectBase {
                project: "myapp".to_string()
            }
            .layer_number(),
            7
        );
        assert_eq!(Layer::UserLocal.layer_number(), 8);
        assert_eq!(Layer::WorkspaceActive.layer_number(), 9);
    }

    // ===== AuditContext Tests =====

    #[test]
    fn test_audit_context_new() {
        let context = AuditContext {
            active_mode: Some("claude".to_string()),
            active_scope: Some("python".to_string()),
        };

        assert_eq!(context.active_mode, Some("claude".to_string()));
        assert_eq!(context.active_scope, Some("python".to_string()));
    }

    #[test]
    fn test_audit_context_none() {
        let context = AuditContext {
            active_mode: None,
            active_scope: None,
        };

        assert!(context.active_mode.is_none());
        assert!(context.active_scope.is_none());
    }

    // ===== Serialization Tests =====

    #[test]
    fn test_audit_entry_serialization() {
        let layer = Layer::GlobalBase;
        let files = vec!["config.json".to_string()];
        let base_oid = git2::Oid::zero();
        let merge_oid = git2::Oid::from_str("abc123").unwrap();

        let entry = AuditEntry::new(
            "user".to_string(),
            "myproject".to_string(),
            Some("claude".to_string()),
            Some("python".to_string()),
            layer,
            files,
            base_oid,
            merge_oid,
            Some("claude".to_string()),
            Some("python".to_string()),
        );

        let json = serde_json::to_string(&entry).unwrap();

        // Verify key fields are present
        assert!(json.contains("\"user\":\"user\""));
        assert!(json.contains("\"project\":\"myproject\""));
        assert!(json.contains("\"mode\":\"claude\""));
        assert!(json.contains("\"scope\":\"python\""));
        assert!(json.contains("\"layer\":1"));
        assert!(json.contains("\"files\":"));
        assert!(json.contains("\"config.json\""));
    }

    #[test]
    fn test_audit_entry_deserialization() {
        let json = r#"{
            "timestamp": "2025-12-26T10:00:00Z",
            "user": "user",
            "project": "myproject",
            "mode": "claude",
            "scope": "python",
            "layer": 3,
            "files": ["config.json"],
            "base_commit": "0000000000000000000000000000000000000000",
            "merge_commit": "abc123",
            "context": {
                "active_mode": "claude",
                "active_scope": "python"
            }
        }"#;

        let entry: AuditEntry = serde_json::from_str(json).unwrap();

        assert_eq!(entry.user, "user");
        assert_eq!(entry.project, "myproject");
        assert_eq!(entry.mode, Some("claude".to_string()));
        assert_eq!(entry.scope, Some("python".to_string()));
        assert_eq!(entry.layer, 3);
        assert_eq!(entry.files.len(), 1);
        assert_eq!(entry.files[0], "config.json");
    }
}
