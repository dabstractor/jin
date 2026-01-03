//! .jinmerge file format for layer-aware conflict resolution
//!
//! This module provides the file format and parsing for `.jinmerge` files,
//! which are generated when `jin apply` detects conflicts between layers.
//!
//! The format uses Git-compatible conflict markers with layer ref paths as labels:
//! ```text
//! # Jin merge conflict. Resolve and run 'jin resolve <file>'
//! <<<<<<< mode/claude/scope:javascript/
//! {"target": "es6", "modules": true}
//! =======
//! {"target": "es2020", "modules": false, "strict": true}
//! >>>>>>> mode/claude/project/ui-dashboard/
//! ```
//!
//! # Example
//!
//! ```ignore
//! use jin::merge::jinmerge::{JinMergeConflict, JINMERGE_HEADER};
//! use std::path::PathBuf;
//!
//! // Create a conflict from two layer versions
//! let conflict = JinMergeConflict::from_text_merge(
//!     PathBuf::from("config.json"),
//!     "mode/claude/scope:javascript/".to_string(),
//!     "{\"target\": \"es6\"}".to_string(),
//!     "mode/claude/project/ui-dashboard/".to_string(),
//!     "{\"target\": \"es2020\"}".to_string(),
//! );
//!
//! // Write to .jinmerge file
//! conflict.write_to_file(&PathBuf::from("config.json.jinmerge"))?;
//!
//! // Parse existing .jinmerge file
//! let loaded = JinMergeConflict::parse_from_file(&PathBuf::from("config.json.jinmerge"))?;
//! ```

use crate::core::{JinError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Header comment added to all .jinmerge files
pub const JINMERGE_HEADER: &str = "# Jin merge conflict. Resolve and run 'jin resolve <file>'";

/// Marker constants (Git-compatible - exactly 7 characters)
pub const MARKER_START: &str = "<<<<<<< ";
pub const MARKER_SEP: &str = "=======";
pub const MARKER_END: &str = ">>>>>>> ";

/// Represents a single conflict region with layer-aware labels
///
/// This structure captures the two conflicting versions along with their
/// layer ref paths for clear conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeRegion {
    /// Full ref path for first layer (e.g., "mode/claude/scope:javascript/")
    pub layer1_ref: String,
    /// Content from first layer
    pub layer1_content: String,
    /// Full ref path for second layer (e.g., "mode/claude/project/ui-dashboard/")
    pub layer2_ref: String,
    /// Content from second layer
    pub layer2_content: String,
    /// Starting line number (1-indexed, for user display)
    pub start_line: usize,
    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,
}

/// Represents a complete .jinmerge file
///
/// Contains the original file path and all conflict regions.
/// Can be written to disk (with .jinmerge extension) or parsed from existing files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeConflict {
    /// Original file path (without .jinmerge extension)
    pub file_path: PathBuf,
    /// All conflict regions in the file
    pub conflicts: Vec<JinMergeRegion>,
}

impl JinMergeConflict {
    /// Create from text merge result and layer ref paths
    ///
    /// Creates a single conflict region from the provided content.
    /// Line numbers are calculated based on the content.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Original file path (without .jinmerge extension)
    /// * `layer1_ref` - Full ref path for first layer
    /// * `layer1_content` - Content from first layer
    /// * `layer2_ref` - Full ref path for second layer
    /// * `layer2_content` - Content from second layer
    ///
    /// # Example
    ///
    /// ```
    /// use jin::merge::jinmerge::JinMergeConflict;
    /// use std::path::PathBuf;
    ///
    /// let conflict = JinMergeConflict::from_text_merge(
    ///     PathBuf::from("config.json"),
    ///     "global/".to_string(),
    ///     "version: 1".to_string(),
    ///     "mode/claude/".to_string(),
    ///     "version: 2".to_string(),
    /// );
    /// ```
    pub fn from_text_merge(
        file_path: PathBuf,
        layer1_ref: String,
        layer1_content: String,
        layer2_ref: String,
        layer2_content: String,
    ) -> Self {
        // Calculate line count for end_line
        let line_count = layer1_content
            .lines()
            .count()
            .max(layer2_content.lines().count());
        let end_line = line_count + 2; // Account for marker lines

        Self {
            file_path,
            conflicts: vec![JinMergeRegion {
                layer1_ref,
                layer1_content,
                layer2_ref,
                layer2_content,
                start_line: 1,
                end_line,
            }],
        }
    }

    /// Write to .jinmerge file with layer-aware markers
    ///
    /// Uses atomic write pattern: write to temp file first, then rename.
    ///
    /// # Arguments
    ///
    /// * `merge_path` - Path where the .jinmerge file should be written
    ///
    /// # Errors
    ///
    /// Returns `JinError::Io` if file operations fail.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use jin::merge::jinmerge::JinMergeConflict;
    /// use std::path::PathBuf;
    ///
    /// let conflict = JinMergeConflict::from_text_merge(
    ///     PathBuf::from("config.json"),
    ///     "global/".to_string(),
    ///     "content1".to_string(),
    ///     "mode/claude/".to_string(),
    ///     "content2".to_string(),
    /// );
    ///
    /// conflict.write_to_file(&PathBuf::from("config.json.jinmerge"))?;
    /// ```
    pub fn write_to_file(&self, merge_path: &Path) -> Result<()> {
        let content = self.to_jinmerge_format()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = merge_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Atomic write pattern - use temp file
        let temp_path = merge_path.with_extension("jinmerge.tmp");
        std::fs::write(&temp_path, content).map_err(JinError::Io)?;
        std::fs::rename(&temp_path, merge_path).map_err(JinError::Io)?;

        Ok(())
    }

    /// Parse existing .jinmerge file
    ///
    /// Extracts conflict regions and layer refs from a .jinmerge file.
    ///
    /// # Arguments
    ///
    /// * `merge_path` - Path to the .jinmerge file
    ///
    /// # Errors
    ///
    /// Returns `JinError::Parse` with format "jinmerge" if markers are malformed.
    /// Returns `JinError::Io` if file cannot be read.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use jin::merge::jinmerge::JinMergeConflict;
    /// use std::path::PathBuf;
    ///
    /// let conflict = JinMergeConflict::parse_from_file(&PathBuf::from("config.json.jinmerge"))?;
    /// ```
    pub fn parse_from_file(merge_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(merge_path)?;
        parse_jinmerge_content(&content, merge_path)
    }

    /// Count total conflict regions
    ///
    /// # Example
    ///
    /// ```
    /// use jin::merge::jinmerge::JinMergeConflict;
    /// use std::path::PathBuf;
    ///
    /// let conflict = JinMergeConflict::from_text_merge(
    ///     PathBuf::from("test.txt"),
    ///     "layer1".to_string(),
    ///     "a".to_string(),
    ///     "layer2".to_string(),
    ///     "b".to_string(),
    /// );
    /// assert_eq!(conflict.conflict_count(), 1);
    /// ```
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Check if file appears to be a .jinmerge file
    ///
    /// Checks both the file extension and the header comment.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check
    ///
    /// # Example
    ///
    /// ```ignore
    /// use jin::merge::jinmerge::JinMergeConflict;
    /// use std::path::PathBuf;
    ///
    /// assert!(JinMergeConflict::is_jinmerge_file(&PathBuf::from("config.json.jinmerge")));
    /// assert!(!JinMergeConflict::is_jinmerge_file(&PathBuf::from("config.json")));
    /// ```
    pub fn is_jinmerge_file(path: &Path) -> bool {
        // Check extension
        if path.extension().and_then(|s| s.to_str()) != Some("jinmerge") {
            return false;
        }

        // Check first line for header
        match std::fs::read_to_string(path) {
            Ok(content) => content
                .lines()
                .next()
                .map(|line| line.starts_with("# Jin merge conflict"))
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Get the .jinmerge file path for an original file
    ///
    /// # Arguments
    ///
    /// * `original` - Original file path
    ///
    /// # Example
    ///
    /// ```
    /// use jin::merge::jinmerge::JinMergeConflict;
    /// use std::path::PathBuf;
    ///
    /// let merge_path = JinMergeConflict::merge_path_for_file(&PathBuf::from("config.json"));
    /// assert_eq!(merge_path, PathBuf::from("config.json.jinmerge"));
    /// ```
    pub fn merge_path_for_file(original: &Path) -> PathBuf {
        let mut merge_path = original.as_os_str().to_owned();
        merge_path.push(".jinmerge");
        PathBuf::from(merge_path)
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Convert to .jinmerge file format string
    fn to_jinmerge_format(&self) -> Result<String> {
        let mut output = String::new();

        // Add header
        output.push_str(JINMERGE_HEADER);
        output.push('\n');

        // Add each conflict region
        for conflict in &self.conflicts {
            output.push_str(MARKER_START);
            output.push_str(&conflict.layer1_ref);
            output.push('\n');
            output.push_str(&conflict.layer1_content);
            // Ensure newline after content if not present
            if !conflict.layer1_content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(MARKER_SEP);
            output.push('\n');
            output.push_str(&conflict.layer2_content);
            // Ensure newline after content if not present
            if !conflict.layer2_content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str(MARKER_END);
            output.push_str(&conflict.layer2_ref);
            output.push('\n');
        }

        Ok(output)
    }
}

/// Parse .jinmerge content into a JinMergeConflict
///
/// This is a private helper function that parses the content of a .jinmerge file.
fn parse_jinmerge_content(content: &str, merge_path: &Path) -> Result<JinMergeConflict> {
    let lines: Vec<&str> = content.lines().collect();
    let mut conflicts = Vec::new();
    let mut i = 0;

    // Skip header if present
    if i < lines.len() && lines[i].starts_with("# Jin merge conflict") {
        i += 1;
    }

    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            let start_line = i + 1; // 1-indexed

            // Extract layer1_ref from start marker
            let layer1_ref = if lines[i].len() > MARKER_START.len() {
                lines[i][MARKER_START.len()..].trim().to_string()
            } else {
                return Err(JinError::Parse {
                    format: "jinmerge".to_string(),
                    message: "Missing layer ref in start marker".to_string(),
                });
            };

            // Find separator (=======)
            let sep_idx = lines[i..]
                .iter()
                .position(|l| l.starts_with(MARKER_SEP))
                .ok_or_else(|| JinError::Parse {
                    format: "jinmerge".to_string(),
                    message: "Missing separator marker".to_string(),
                })?;
            let sep_idx = i + sep_idx;

            // Extract layer1 content
            let layer1_content = lines[i + 1..sep_idx].join("\n");

            // Find end marker (>>>>>>>)
            let end_idx = lines[sep_idx..]
                .iter()
                .position(|l| l.starts_with(">>>>>>>"))
                .ok_or_else(|| JinError::Parse {
                    format: "jinmerge".to_string(),
                    message: "Missing end marker".to_string(),
                })?;
            let end_idx = sep_idx + end_idx;

            // Extract layer2_ref from end marker
            let layer2_ref = if lines[end_idx].len() > MARKER_END.len() {
                lines[end_idx][MARKER_END.len()..].trim().to_string()
            } else {
                return Err(JinError::Parse {
                    format: "jinmerge".to_string(),
                    message: "Missing layer ref in end marker".to_string(),
                });
            };

            // Extract layer2 content
            let layer2_content = lines[sep_idx + 1..end_idx].join("\n");

            conflicts.push(JinMergeRegion {
                layer1_ref,
                layer1_content,
                layer2_ref,
                layer2_content,
                start_line,
                end_line: end_idx + 1, // 1-indexed
            });

            i = end_idx + 1;
        } else {
            i += 1;
        }
    }

    // Extract original file path from merge path
    // Use just the file name (without directory prefix) and remove .jinmerge extension
    let file_name = merge_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    // Remove .jinmerge extension if present
    let file_name = file_name.strip_suffix(".jinmerge").unwrap_or(file_name);
    let file_path = PathBuf::from(file_name);

    Ok(JinMergeConflict {
        file_path,
        conflicts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ========== Constructor Tests ==========

    #[test]
    fn test_from_text_merge_basic() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.json"),
            "global/".to_string(),
            "content1".to_string(),
            "mode/claude/".to_string(),
            "content2".to_string(),
        );

        assert_eq!(conflict.file_path, PathBuf::from("test.json"));
        assert_eq!(conflict.conflicts.len(), 1);
        assert_eq!(conflict.conflicts[0].layer1_ref, "global/");
        assert_eq!(conflict.conflicts[0].layer1_content, "content1");
        assert_eq!(conflict.conflicts[0].layer2_ref, "mode/claude/");
        assert_eq!(conflict.conflicts[0].layer2_content, "content2");
        assert_eq!(conflict.conflicts[0].start_line, 1);
    }

    #[test]
    fn test_from_text_merge_multiline() {
        let content1 = "line1\nline2\nline3";
        let content2 = "lineA\nlineB";

        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.txt"),
            "layer1/".to_string(),
            content1.to_string(),
            "layer2/".to_string(),
            content2.to_string(),
        );

        // end_line should account for content lines + marker lines
        assert!(conflict.conflicts[0].end_line > 1);
    }

    // ========== Format Generation Tests ==========

    #[test]
    fn test_to_jinmerge_format_single_conflict() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("config.json"),
            "global/".to_string(),
            "{\"target\": \"es6\"}".to_string(),
            "mode/claude/".to_string(),
            "{\"target\": \"es2020\"}".to_string(),
        );

        let format = conflict.to_jinmerge_format().unwrap();

        assert!(format.contains(JINMERGE_HEADER));
        assert!(format.contains("<<<<<<< global/"));
        assert!(format.contains("{\"target\": \"es6\"}"));
        assert!(format.contains("======="));
        assert!(format.contains("{\"target\": \"es2020\"}"));
        assert!(format.contains(">>>>>>> mode/claude/"));
    }

    #[test]
    fn test_to_jinmerge_format_preserves_trailing_newline() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.txt"),
            "layer1/".to_string(),
            "content\n".to_string(),
            "layer2/".to_string(),
            "content\n".to_string(),
        );

        let format = conflict.to_jinmerge_format().unwrap();

        // Should have exactly one newline after content (the preserved one)
        let layer1_section: Vec<&str> = format.split("=======").collect();
        assert!(layer1_section[0].ends_with("content\n"));
    }

    #[test]
    fn test_to_jinmerge_format_adds_newline_if_missing() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.txt"),
            "layer1/".to_string(),
            "content".to_string(),
            "layer2/".to_string(),
            "content".to_string(),
        );

        let format = conflict.to_jinmerge_format().unwrap();

        // Should add newline if content doesn't have one
        assert!(format.contains("content\n======="));
    }

    // ========== File I/O Tests ==========

    #[test]
    fn test_write_to_file_creates_valid_jinmerge() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("config.json.jinmerge");

        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("config.json"),
            "global/".to_string(),
            "{\"target\": \"es6\"}".to_string(),
            "mode/claude/".to_string(),
            "{\"target\": \"es2020\"}".to_string(),
        );

        conflict.write_to_file(&merge_path).unwrap();

        // Verify file exists
        assert!(merge_path.exists());

        // Verify content
        let content = fs::read_to_string(&merge_path).unwrap();
        assert!(content.contains(JINMERGE_HEADER));
        assert!(content.contains("<<<<<<< global/"));
        assert!(content.contains(">>>>>>> mode/claude/"));
    }

    #[test]
    fn test_write_to_file_atomic() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("config.json.jinmerge");
        let temp_path = merge_path.with_extension("jinmerge.tmp");

        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("config.json"),
            "global/".to_string(),
            "content1".to_string(),
            "mode/claude/".to_string(),
            "content2".to_string(),
        );

        conflict.write_to_file(&merge_path).unwrap();

        // Temp file should be cleaned up
        assert!(!temp_path.exists());
        // Final file should exist
        assert!(merge_path.exists());
    }

    #[test]
    fn test_write_to_file_creates_parent_dir() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("subdir").join("config.json.jinmerge");

        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("config.json"),
            "global/".to_string(),
            "content1".to_string(),
            "mode/claude/".to_string(),
            "content2".to_string(),
        );

        conflict.write_to_file(&merge_path).unwrap();

        assert!(merge_path.exists());
        assert!(merge_path.parent().unwrap().exists());
    }

    // ========== Parsing Tests ==========

    #[test]
    fn test_parse_from_file_basic() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("config.json.jinmerge");

        let content = format!(
            "{}\n<<<<<<< global/\n{{\"target\": \"es6\"}}\n=======\n{{\"target\": \"es2020\"}}\n>>>>>>> mode/claude/\n",
            JINMERGE_HEADER
        );

        fs::write(&merge_path, content).unwrap();

        let parsed = JinMergeConflict::parse_from_file(&merge_path).unwrap();

        assert_eq!(parsed.file_path, PathBuf::from("config.json"));
        assert_eq!(parsed.conflicts.len(), 1);
        assert_eq!(parsed.conflicts[0].layer1_ref, "global/");
        assert_eq!(parsed.conflicts[0].layer1_content, "{\"target\": \"es6\"}");
        assert_eq!(parsed.conflicts[0].layer2_ref, "mode/claude/");
        assert_eq!(
            parsed.conflicts[0].layer2_content,
            "{\"target\": \"es2020\"}"
        );
    }

    #[test]
    fn test_parse_from_file_multiline_content() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("test.txt.jinmerge");

        let content = format!(
            "{}\n<<<<<<< layer1/\nline1\nline2\nline3\n=======\nlineA\nlineB\n>>>>>>> layer2/\n",
            JINMERGE_HEADER
        );

        fs::write(&merge_path, content).unwrap();

        let parsed = JinMergeConflict::parse_from_file(&merge_path).unwrap();

        assert_eq!(parsed.conflicts[0].layer1_content, "line1\nline2\nline3");
        assert_eq!(parsed.conflicts[0].layer2_content, "lineA\nlineB");
    }

    #[test]
    fn test_parse_from_file_missing_separator() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("test.jinmerge");

        let content = "<<<<<<< layer1/\ncontent\n>>>>>>> layer2/\n";
        fs::write(&merge_path, content).unwrap();

        let result = JinMergeConflict::parse_from_file(&merge_path);
        assert!(result.is_err());
        if let Err(JinError::Parse { format, .. }) = result {
            assert_eq!(format, "jinmerge");
        } else {
            panic!("Expected Parse error");
        }
    }

    #[test]
    fn test_parse_from_file_missing_end_marker() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("test.jinmerge");

        let content = "<<<<<<< layer1/\ncontent1\n=======\ncontent2\n";
        fs::write(&merge_path, content).unwrap();

        let result = JinMergeConflict::parse_from_file(&merge_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_from_file_multiple_conflicts() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("test.txt.jinmerge");

        let content = format!(
            "{}\n<<<<<<< layer1/\ncontent1\n=======\ncontent2\n>>>>>>> layer2/\n<<<<<<< layer1/\ncontent3\n=======\ncontent4\n>>>>>>> layer2/\n",
            JINMERGE_HEADER
        );

        fs::write(&merge_path, content).unwrap();

        let parsed = JinMergeConflict::parse_from_file(&merge_path).unwrap();

        assert_eq!(parsed.conflicts.len(), 2);
        assert_eq!(parsed.conflicts[0].layer1_content, "content1");
        assert_eq!(parsed.conflicts[1].layer1_content, "content3");
    }

    // ========== Round-trip Tests ==========

    #[test]
    fn test_roundtrip_write_then_parse() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("config.json.jinmerge");

        let original = JinMergeConflict::from_text_merge(
            PathBuf::from("config.json"),
            "global/".to_string(),
            "{\"target\": \"es6\"}".to_string(),
            "mode/claude/".to_string(),
            "{\"target\": \"es2020\"}".to_string(),
        );

        original.write_to_file(&merge_path).unwrap();
        let parsed = JinMergeConflict::parse_from_file(&merge_path).unwrap();

        assert_eq!(parsed.file_path, original.file_path);
        assert_eq!(parsed.conflicts.len(), original.conflicts.len());
        assert_eq!(
            parsed.conflicts[0].layer1_ref,
            original.conflicts[0].layer1_ref
        );
        assert_eq!(
            parsed.conflicts[0].layer1_content,
            original.conflicts[0].layer1_content
        );
        assert_eq!(
            parsed.conflicts[0].layer2_ref,
            original.conflicts[0].layer2_ref
        );
        assert_eq!(
            parsed.conflicts[0].layer2_content,
            original.conflicts[0].layer2_content
        );
    }

    // ========== Helper Method Tests ==========

    #[test]
    fn test_conflict_count() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.txt"),
            "layer1/".to_string(),
            "a".to_string(),
            "layer2/".to_string(),
            "b".to_string(),
        );

        assert_eq!(conflict.conflict_count(), 1);
    }

    #[test]
    fn test_conflict_count_empty() {
        let conflict = JinMergeConflict {
            file_path: PathBuf::from("test.txt"),
            conflicts: vec![],
        };

        assert_eq!(conflict.conflict_count(), 0);
    }

    #[test]
    fn test_is_jinmerge_file_by_extension() {
        let temp = TempDir::new().unwrap();
        let jinmerge_path = temp.path().join("config.json.jinmerge");
        let regular_path = temp.path().join("config.json");

        fs::write(&jinmerge_path, JINMERGE_HEADER).unwrap();
        fs::write(&regular_path, "content").unwrap();

        assert!(JinMergeConflict::is_jinmerge_file(&jinmerge_path));
        assert!(!JinMergeConflict::is_jinmerge_file(&regular_path));
    }

    #[test]
    fn test_is_jinmerge_file_by_header() {
        let temp = TempDir::new().unwrap();
        let valid_path = temp.path().join("test.jinmerge");
        let invalid_path = temp.path().join("test2.jinmerge");

        let valid_content = format!("{}\ncontent", JINMERGE_HEADER);
        let invalid_content = "# Wrong header\ncontent";

        fs::write(&valid_path, valid_content).unwrap();
        fs::write(&invalid_path, invalid_content).unwrap();

        assert!(JinMergeConflict::is_jinmerge_file(&valid_path));
        assert!(!JinMergeConflict::is_jinmerge_file(&invalid_path));
    }

    #[test]
    fn test_is_jinmerge_file_nonexistent() {
        assert!(!JinMergeConflict::is_jinmerge_file(&PathBuf::from(
            "nonexistent.jinmerge",
        )));
    }

    #[test]
    fn test_merge_path_for_file_basic() {
        let original = PathBuf::from("config.json");
        let merge_path = JinMergeConflict::merge_path_for_file(&original);

        assert_eq!(merge_path, PathBuf::from("config.json.jinmerge"));
    }

    #[test]
    fn test_merge_path_for_file_with_path() {
        let original = PathBuf::from("subdir/config.json");
        let merge_path = JinMergeConflict::merge_path_for_file(&original);

        assert_eq!(merge_path, PathBuf::from("subdir/config.json.jinmerge"));
    }

    #[test]
    fn test_merge_path_for_file_replaces_existing_extension() {
        let original = PathBuf::from("config.json");
        let merge_path = JinMergeConflict::merge_path_for_file(&original);

        assert_eq!(merge_path.extension().unwrap().to_str(), Some("jinmerge"));
    }

    // ========== Marker Format Tests ==========

    #[test]
    fn test_marker_lengths() {
        assert_eq!(MARKER_START.len(), 8); // 7 chars + space
        assert_eq!(MARKER_SEP.len(), 7);
        assert_eq!(MARKER_END.len(), 8); // 7 chars + space
    }

    #[test]
    fn test_marker_format_matches_git() {
        // Git uses exactly 7 characters for each marker
        assert!(MARKER_START.starts_with("<<<<<<<"));
        assert_eq!(MARKER_SEP, "=======");
        assert!(MARKER_END.starts_with(">>>>>>>"));
    }

    // ========== Line Number Tests ==========

    #[test]
    fn test_line_numbers_are_1_indexed() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.txt"),
            "layer1/".to_string(),
            "content".to_string(),
            "layer2/".to_string(),
            "content".to_string(),
        );

        assert_eq!(conflict.conflicts[0].start_line, 1);
        assert!(conflict.conflicts[0].end_line >= 1);
    }

    #[test]
    fn test_parse_preserves_line_numbers() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("test.txt.jinmerge");

        let content = format!(
            "{}\n<<<<<<< layer1/\ncontent\n=======\ncontent\n>>>>>>> layer2/\n",
            JINMERGE_HEADER
        );

        fs::write(&merge_path, content).unwrap();
        let parsed = JinMergeConflict::parse_from_file(&merge_path).unwrap();

        // Line 2 is where <<<<<<< is (header is line 1)
        assert_eq!(parsed.conflicts[0].start_line, 2);
        assert!(parsed.conflicts[0].end_line > parsed.conflicts[0].start_line);
    }

    // ========== Empty Content Tests ==========

    #[test]
    fn test_empty_layer_content() {
        let conflict = JinMergeConflict::from_text_merge(
            PathBuf::from("test.txt"),
            "layer1/".to_string(),
            "".to_string(),
            "layer2/".to_string(),
            "content".to_string(),
        );

        assert_eq!(conflict.conflicts[0].layer1_content, "");
        assert_eq!(conflict.conflicts[0].layer2_content, "content");
    }

    #[test]
    fn test_parse_empty_layer_content() {
        let temp = TempDir::new().unwrap();
        let merge_path = temp.path().join("test.txt.jinmerge");

        let content = format!(
            "{}\n<<<<<<< layer1/\n=======\ncontent\n>>>>>>> layer2/\n",
            JINMERGE_HEADER
        );

        fs::write(&merge_path, content).unwrap();
        let parsed = JinMergeConflict::parse_from_file(&merge_path).unwrap();

        assert_eq!(parsed.conflicts[0].layer1_content, "");
        assert_eq!(parsed.conflicts[0].layer2_content, "content");
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_full_workflow() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.json");
        let merge_path = JinMergeConflict::merge_path_for_file(&config_path);

        // Create a conflict
        let conflict = JinMergeConflict::from_text_merge(
            config_path.clone(),
            "global/".to_string(),
            "{\"version\": 1}".to_string(),
            "mode/claude/".to_string(),
            "{\"version\": 2}".to_string(),
        );

        // Write it
        conflict.write_to_file(&merge_path).unwrap();
        assert!(JinMergeConflict::is_jinmerge_file(&merge_path));

        // Parse it back
        let loaded = JinMergeConflict::parse_from_file(&merge_path).unwrap();
        assert_eq!(loaded.conflict_count(), 1);
        // Note: parse_from_file extracts just the filename, not the full path
        assert_eq!(loaded.file_path, PathBuf::from("config.json"));
    }
}
