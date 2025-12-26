//! 3-way text merge for plain text files

use crate::core::{JinError, Result};

/// Result of a text merge operation
#[derive(Debug)]
pub enum TextMergeResult {
    /// Merge succeeded without conflicts
    Clean(String),
    /// Merge has conflicts that need resolution
    Conflict {
        /// The merged content with conflict markers
        content: String,
        /// Number of conflict regions
        conflict_count: usize,
    },
}

/// Perform a 3-way text merge
///
/// TODO: Implement proper 3-way merge algorithm in later milestone
///
/// # Arguments
/// * `base` - The common ancestor content
/// * `ours` - Our version of the content
/// * `theirs` - Their version of the content
///
/// # Returns
/// * `TextMergeResult::Clean` if merge succeeds
/// * `TextMergeResult::Conflict` if there are conflicts
pub fn text_merge(base: &str, ours: &str, theirs: &str) -> Result<TextMergeResult> {
    // TODO: Implement proper 3-way merge
    // For now, just use a simple strategy

    if ours == theirs {
        // Both sides made the same changes
        return Ok(TextMergeResult::Clean(ours.to_string()));
    }

    if ours == base {
        // We didn't change, take theirs
        return Ok(TextMergeResult::Clean(theirs.to_string()));
    }

    if theirs == base {
        // They didn't change, take ours
        return Ok(TextMergeResult::Clean(ours.to_string()));
    }

    // Both sides changed differently - this is a conflict
    let conflict_content = format!(
        "<<<<<<< ours\n{}\n=======\n{}\n>>>>>>> theirs\n",
        ours.trim_end(),
        theirs.trim_end()
    );

    Ok(TextMergeResult::Conflict {
        content: conflict_content,
        conflict_count: 1,
    })
}

/// Check if content contains conflict markers
pub fn has_conflict_markers(content: &str) -> bool {
    content.contains("<<<<<<<") && content.contains("=======") && content.contains(">>>>>>>")
}

/// Parse conflict markers from content
///
/// TODO: Implement proper conflict parsing in later milestone
pub fn parse_conflicts(_content: &str) -> Result<Vec<ConflictRegion>> {
    // TODO: Implement
    Err(JinError::Other("Not implemented".to_string()))
}

/// Represents a conflict region in a file
#[derive(Debug)]
pub struct ConflictRegion {
    /// Starting line number
    pub start_line: usize,
    /// Ending line number
    pub end_line: usize,
    /// Our version of the content
    pub ours: String,
    /// Their version of the content
    pub theirs: String,
    /// Optional base version
    pub base: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_merge_identical() {
        let base = "line1\nline2\n";
        let ours = "line1\nline2\nline3\n";
        let theirs = "line1\nline2\nline3\n";

        match text_merge(base, ours, theirs).unwrap() {
            TextMergeResult::Clean(content) => {
                assert_eq!(content, "line1\nline2\nline3\n");
            }
            _ => panic!("Expected clean merge"),
        }
    }

    #[test]
    fn test_text_merge_ours_only() {
        let base = "original\n";
        let ours = "modified\n";
        let theirs = "original\n";

        match text_merge(base, ours, theirs).unwrap() {
            TextMergeResult::Clean(content) => {
                assert_eq!(content, "modified\n");
            }
            _ => panic!("Expected clean merge"),
        }
    }

    #[test]
    fn test_text_merge_theirs_only() {
        let base = "original\n";
        let ours = "original\n";
        let theirs = "modified\n";

        match text_merge(base, ours, theirs).unwrap() {
            TextMergeResult::Clean(content) => {
                assert_eq!(content, "modified\n");
            }
            _ => panic!("Expected clean merge"),
        }
    }

    #[test]
    fn test_text_merge_conflict() {
        let base = "original\n";
        let ours = "our change\n";
        let theirs = "their change\n";

        match text_merge(base, ours, theirs).unwrap() {
            TextMergeResult::Conflict {
                content,
                conflict_count,
            } => {
                assert!(has_conflict_markers(&content));
                assert_eq!(conflict_count, 1);
            }
            _ => panic!("Expected conflict"),
        }
    }

    #[test]
    fn test_has_conflict_markers() {
        assert!(!has_conflict_markers("normal content"));
        assert!(has_conflict_markers(
            "<<<<<<< ours\ncontent\n=======\nother\n>>>>>>> theirs"
        ));
    }
}
