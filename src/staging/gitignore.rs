//! Git ignore management for Jin
//!
//! This module manages a dedicated block in `.gitignore` for Jin-staged files.
//! The managed block is delimited by special markers to prevent conflicts
//! with user-managed entries.

use crate::core::Result;
use std::path::Path;

/// Start marker for Jin managed block
pub const MANAGED_START: &str = "# --- JIN MANAGED START ---";
/// End marker for Jin managed block
pub const MANAGED_END: &str = "# --- JIN MANAGED END ---";
/// Default path to .gitignore
const GITIGNORE_PATH: &str = ".gitignore";

/// Ensure a path is in the .gitignore managed block
///
/// This function:
/// 1. Reads the existing .gitignore (or creates empty content)
/// 2. Parses the managed block
/// 3. Adds the path if not already present
/// 4. Writes the updated content back
///
/// # Arguments
///
/// * `path` - Path to add to the managed block
///
/// # Errors
///
/// Returns `JinError::Io` if the file cannot be read or written
pub fn ensure_in_managed_block(path: &Path) -> Result<()> {
    ensure_in_managed_block_at(path, Path::new(GITIGNORE_PATH))
}

/// Ensure a path is in the .gitignore managed block at a specific gitignore path
///
/// Internal function for testing with custom gitignore locations.
fn ensure_in_managed_block_at(path: &Path, gitignore_path: &Path) -> Result<()> {
    let content = read_gitignore_at(gitignore_path);
    let path_str = normalize_path(path);

    // Parse existing content
    let (before, managed, after) = parse_managed_block(&content);

    // Check if already present
    if managed.iter().any(|p| p == &path_str) {
        return Ok(());
    }

    // Add to managed block
    let mut new_managed = managed;
    new_managed.push(path_str);
    new_managed.sort();
    new_managed.dedup();

    // Rebuild content
    let new_content = build_gitignore(&before, &new_managed, &after);
    write_gitignore_at(&new_content, gitignore_path)?;

    Ok(())
}

/// Remove a path from the .gitignore managed block
///
/// # Arguments
///
/// * `path` - Path to remove from the managed block
///
/// # Errors
///
/// Returns `JinError::Io` if the file cannot be read or written
pub fn remove_from_managed_block(path: &Path) -> Result<()> {
    remove_from_managed_block_at(path, Path::new(GITIGNORE_PATH))
}

/// Remove a path from the .gitignore managed block at a specific gitignore path
///
/// Internal function for testing with custom gitignore locations.
fn remove_from_managed_block_at(path: &Path, gitignore_path: &Path) -> Result<()> {
    let content = read_gitignore_at(gitignore_path);
    let path_str = normalize_path(path);

    // Parse existing content
    let (before, managed, after) = parse_managed_block(&content);

    // Remove the path
    let new_managed: Vec<String> = managed.into_iter().filter(|p| p != &path_str).collect();

    // Rebuild content
    let new_content = build_gitignore(&before, &new_managed, &after);
    write_gitignore_at(&new_content, gitignore_path)?;

    Ok(())
}

/// Normalize a path for gitignore entry
///
/// Converts path to a string suitable for .gitignore,
/// using forward slashes and adding trailing slash for directories.
fn normalize_path(path: &Path) -> String {
    let path_str = path.display().to_string();
    // Convert backslashes to forward slashes for cross-platform
    path_str.replace('\\', "/")
}

/// Read the .gitignore file at a specific path, returning empty string if it doesn't exist
fn read_gitignore_at(gitignore_path: &Path) -> String {
    std::fs::read_to_string(gitignore_path).unwrap_or_default()
}

/// Write content to .gitignore at a specific path
fn write_gitignore_at(content: &str, gitignore_path: &Path) -> Result<()> {
    std::fs::write(gitignore_path, content)?;
    Ok(())
}

/// Parse the .gitignore content into three parts:
/// - before: content before the managed block
/// - managed: entries inside the managed block
/// - after: content after the managed block
fn parse_managed_block(content: &str) -> (String, Vec<String>, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut before = Vec::new();
    let mut managed = Vec::new();
    let mut after = Vec::new();
    let mut in_block = false;
    let mut after_block = false;

    for line in lines {
        if line == MANAGED_START {
            in_block = true;
            continue;
        }
        if line == MANAGED_END {
            in_block = false;
            after_block = true;
            continue;
        }

        if in_block {
            // Only add non-empty, non-comment lines as managed entries
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                managed.push(trimmed.to_string());
            }
        } else if after_block {
            after.push(line.to_string());
        } else {
            before.push(line.to_string());
        }
    }

    (before.join("\n"), managed, after.join("\n"))
}

/// Build the .gitignore content from the three parts
fn build_gitignore(before: &str, managed: &[String], after: &str) -> String {
    let mut result = String::new();

    // Add content before managed block
    if !before.is_empty() {
        result.push_str(before);
        if !before.ends_with('\n') {
            result.push('\n');
        }
    }

    // Add managed block
    result.push_str(MANAGED_START);
    result.push('\n');
    for entry in managed {
        result.push_str(entry);
        result.push('\n');
    }
    result.push_str(MANAGED_END);
    result.push('\n');

    // Add content after managed block
    if !after.is_empty() {
        result.push_str(after);
        if !after.ends_with('\n') {
            result.push('\n');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_in_managed_block_creates_gitignore() {
        let temp = TempDir::new().unwrap();
        let gitignore = temp.path().join(".gitignore");

        ensure_in_managed_block_at(Path::new(".claude/"), &gitignore).unwrap();

        let content = std::fs::read_to_string(&gitignore).unwrap();
        assert!(content.contains(MANAGED_START));
        assert!(content.contains(".claude/"));
        assert!(content.contains(MANAGED_END));
    }

    #[test]
    fn test_ensure_in_managed_block_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let gitignore = temp.path().join(".gitignore");
        std::fs::write(&gitignore, "node_modules/\n").unwrap();

        ensure_in_managed_block_at(Path::new(".claude/"), &gitignore).unwrap();

        let content = std::fs::read_to_string(&gitignore).unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains(".claude/"));
    }

    #[test]
    fn test_ensure_in_managed_block_deduplicates() {
        let temp = TempDir::new().unwrap();
        let gitignore = temp.path().join(".gitignore");

        ensure_in_managed_block_at(Path::new(".claude/"), &gitignore).unwrap();
        ensure_in_managed_block_at(Path::new(".claude/"), &gitignore).unwrap();

        let content = std::fs::read_to_string(&gitignore).unwrap();
        let count = content.matches(".claude/").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ensure_in_managed_block_sorts_entries() {
        let temp = TempDir::new().unwrap();
        let gitignore = temp.path().join(".gitignore");

        ensure_in_managed_block_at(Path::new("z-file"), &gitignore).unwrap();
        ensure_in_managed_block_at(Path::new("a-file"), &gitignore).unwrap();

        let content = std::fs::read_to_string(&gitignore).unwrap();
        let a_pos = content.find("a-file").unwrap();
        let z_pos = content.find("z-file").unwrap();
        assert!(a_pos < z_pos);
    }

    #[test]
    fn test_remove_from_managed_block() {
        let temp = TempDir::new().unwrap();
        let gitignore = temp.path().join(".gitignore");

        ensure_in_managed_block_at(Path::new(".claude/"), &gitignore).unwrap();
        ensure_in_managed_block_at(Path::new(".vscode/"), &gitignore).unwrap();

        remove_from_managed_block_at(Path::new(".claude/"), &gitignore).unwrap();

        let content = std::fs::read_to_string(&gitignore).unwrap();
        assert!(!content.contains(".claude/"));
        assert!(content.contains(".vscode/"));
    }

    #[test]
    fn test_parse_managed_block_empty() {
        let (before, managed, after) = parse_managed_block("");
        assert!(before.is_empty());
        assert!(managed.is_empty());
        assert!(after.is_empty());
    }

    #[test]
    fn test_parse_managed_block_with_content() {
        let content = format!(
            "before\n{}\n.claude/\n.vscode/\n{}\nafter",
            MANAGED_START, MANAGED_END
        );
        let (before, managed, after) = parse_managed_block(&content);
        assert_eq!(before, "before");
        assert_eq!(managed, vec![".claude/", ".vscode/"]);
        assert_eq!(after, "after");
    }

    #[test]
    fn test_build_gitignore_empty_managed() {
        let result = build_gitignore("", &[], "");
        assert!(result.contains(MANAGED_START));
        assert!(result.contains(MANAGED_END));
    }

    #[test]
    fn test_build_gitignore_with_all_parts() {
        let result = build_gitignore(
            "node_modules/",
            &[".claude/".to_string(), ".vscode/".to_string()],
            "# end comment",
        );
        assert!(result.contains("node_modules/"));
        assert!(result.contains(MANAGED_START));
        assert!(result.contains(".claude/"));
        assert!(result.contains(".vscode/"));
        assert!(result.contains(MANAGED_END));
        assert!(result.contains("# end comment"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new(".claude/")), ".claude/");
        assert_eq!(
            normalize_path(Path::new(".claude/config.json")),
            ".claude/config.json"
        );
    }
}
