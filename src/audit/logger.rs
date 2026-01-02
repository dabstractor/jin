//! Audit logger for writing audit entries to disk

use crate::audit::AuditEntry;
use crate::core::{JinError, Result};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Audit logger for writing audit entries to disk
///
/// Writes audit entries in JSON Lines format to daily log files
/// stored in `.jin/audit/audit-YYYY-MM-DD.jsonl`.
pub struct AuditLogger {
    /// Base directory for audit files
    audit_dir: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger with the specified audit directory
    ///
    /// # Errors
    ///
    /// Returns an error if the audit directory cannot be created.
    pub fn new(audit_dir: PathBuf) -> Result<Self> {
        // Ensure audit directory exists
        std::fs::create_dir_all(&audit_dir).map_err(JinError::Io)?;
        Ok(Self { audit_dir })
    }

    /// Get the audit file path for today
    fn today_path(&self) -> PathBuf {
        let date = chrono::Utc::now().format("%Y-%m-%d");
        self.audit_dir.join(format!("audit-{}.jsonl", date))
    }

    /// Write an audit entry to today's log file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened or created
    /// - The entry cannot be serialized to JSON
    /// - The write operation fails
    ///
    /// # Format
    ///
    /// Uses JSON Lines format: one JSON object per line, followed by a newline.
    pub fn log_entry(&self, entry: &AuditEntry) -> Result<()> {
        let path = self.today_path();

        // Open file in append mode, create if not exists
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(JinError::Io)?;

        let mut writer = BufWriter::new(file);

        // Serialize as single-line JSON (JSON Lines format)
        let json_line = serde_json::to_string(entry).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })?;

        // Write line with newline
        writeln!(writer, "{}", json_line).map_err(JinError::Io)?;

        // Flush to ensure write is complete
        writer.flush().map_err(JinError::Io)?;

        Ok(())
    }

    /// Create audit logger from project context
    ///
    /// Uses `.jin/audit/` as the audit directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the audit directory cannot be created.
    pub fn from_project() -> Result<Self> {
        let audit_dir = PathBuf::from(".jin").join("audit");
        Self::new(audit_dir)
    }

    /// Get the audit directory path
    pub fn audit_dir(&self) -> &Path {
        &self.audit_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditContext;
    use tempfile::TempDir;

    /// Create an isolated test audit logger
    fn create_test_logger() -> (TempDir, AuditLogger) {
        let temp = TempDir::new().unwrap();
        let audit_dir = temp.path().join("audit");
        let logger = AuditLogger::new(audit_dir).unwrap();
        (temp, logger)
    }

    #[test]
    fn test_audit_logger_new_creates_directory() {
        let temp = TempDir::new().unwrap();
        let audit_dir = temp.path().join("audit");

        // Directory shouldn't exist yet
        assert!(!audit_dir.exists());

        let _logger = AuditLogger::new(audit_dir.clone()).unwrap();

        // Directory should now exist
        assert!(audit_dir.exists());
    }

    #[test]
    fn test_audit_logger_today_path_format() {
        let (_temp, logger) = create_test_logger();
        let path = logger.today_path();

        // Path should end with audit-YYYY-MM-DD.jsonl
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("audit-20"));
        assert!(path_str.ends_with(".jsonl"));
    }

    #[test]
    fn test_audit_logger_log_entry_creates_file() {
        let (_temp, logger) = create_test_logger();

        let entry = AuditEntry {
            timestamp: "2025-10-19T15:04:02Z".to_string(),
            user: "test@example.com".to_string(),
            project: Some("test-project".to_string()),
            mode: None,
            scope: None,
            layer: Some(7),
            files: vec!["config.json".to_string()],
            base_commit: None,
            merge_commit: Some("abc123".to_string()),
            context: None,
        };

        logger.log_entry(&entry).unwrap();

        let audit_file = logger.today_path();
        assert!(audit_file.exists());
    }

    #[test]
    fn test_audit_logger_log_entry_json_format() {
        let (_temp, logger) = create_test_logger();

        let entry = AuditEntry {
            timestamp: "2025-10-19T15:04:02Z".to_string(),
            user: "test@example.com".to_string(),
            project: Some("test-project".to_string()),
            mode: Some("claude".to_string()),
            scope: Some("language:rust".to_string()),
            layer: Some(4),
            files: vec![".claude/config.json".to_string()],
            base_commit: Some("parent123".to_string()),
            merge_commit: Some("commit456".to_string()),
            context: Some(AuditContext {
                active_mode: Some("claude".to_string()),
                active_scope: Some("language:rust".to_string()),
            }),
        };

        logger.log_entry(&entry).unwrap();

        // Read the file and verify format
        let content = std::fs::read_to_string(logger.today_path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed["timestamp"], "2025-10-19T15:04:02Z");
        assert_eq!(parsed["user"], "test@example.com");
        assert_eq!(parsed["layer"], 4);
    }

    #[test]
    fn test_audit_logger_append_multiple_entries() {
        let (_temp, logger) = create_test_logger();

        let entry1 = AuditEntry {
            timestamp: "2025-10-19T15:04:02Z".to_string(),
            user: "user1@example.com".to_string(),
            project: None,
            mode: None,
            scope: None,
            layer: Some(1),
            files: vec!["file1.txt".to_string()],
            base_commit: None,
            merge_commit: Some("commit1".to_string()),
            context: None,
        };

        let entry2 = AuditEntry {
            timestamp: "2025-10-19T15:05:02Z".to_string(),
            user: "user2@example.com".to_string(),
            project: None,
            mode: None,
            scope: None,
            layer: Some(2),
            files: vec!["file2.txt".to_string()],
            base_commit: None,
            merge_commit: Some("commit2".to_string()),
            context: None,
        };

        logger.log_entry(&entry1).unwrap();
        logger.log_entry(&entry2).unwrap();

        // Read and verify both entries
        let content = std::fs::read_to_string(logger.today_path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        // Verify each line is valid JSON
        let parsed1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed1["user"], "user1@example.com");

        let parsed2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(parsed2["user"], "user2@example.com");
    }

    #[test]
    fn test_audit_logger_from_project() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let logger = AuditLogger::from_project().unwrap();
        assert_eq!(logger.audit_dir(), PathBuf::from(".jin").join("audit"));
    }

    #[test]
    fn test_audit_entry_from_commit_helper() {
        let entry = AuditEntry::from_commit(
            "test@example.com".to_string(),
            Some("my-project".to_string()),
            Some("claude".to_string()),
            Some("language:javascript".to_string()),
            Some(4),
            vec!["config.json".to_string()],
            Some("base123".to_string()),
            "merge456".to_string(),
        );

        assert_eq!(entry.user, "test@example.com");
        assert_eq!(entry.project, Some("my-project".to_string()));
        assert_eq!(entry.mode, Some("claude".to_string()));
        assert_eq!(entry.scope, Some("language:javascript".to_string()));
        assert_eq!(entry.layer, Some(4));
        assert_eq!(entry.base_commit, Some("base123".to_string()));
        assert_eq!(entry.merge_commit, Some("merge456".to_string()));
    }

    #[test]
    fn test_audit_logger_audit_dir_accessor() {
        let temp = TempDir::new().unwrap();
        let audit_dir = temp.path().join("custom-audit");
        let logger = AuditLogger::new(audit_dir.clone()).unwrap();

        assert_eq!(logger.audit_dir(), audit_dir);
    }
}
