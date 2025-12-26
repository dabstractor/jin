//! Diff command implementation.
//!
//! This module implements the `jin diff` command that displays differences
//! between Jin layers or between the workspace and merged layers.

use crate::cli::args::DiffCommand;
use crate::core::config::ProjectContext;
use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use crate::merge::layer::LayerMerge;
use crate::staging::index::StagingIndex;
use similar::TextDiff;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::Path;

// ===== LOCAL TYPES =====

/// Represents which files are added, modified, or deleted in a diff.
#[derive(Debug, Clone)]
struct FileDiff {
    path: String,
    status: DiffStatus,
    old_content: Option<String>,
    new_content: Option<String>,
}

/// Status of a file in a diff comparison.
#[derive(Debug, Clone, PartialEq)]
enum DiffStatus {
    Added,    // File only in target (new)
    Deleted,  // File only in source (removed)
    Modified, // File in both with differences
    Unchanged, // File in both with same content
}

// ===== ANSI COLOR CODES =====

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_RESET: &str = "\x1b[0m";

// ===== LAYER PARSING =====

/// Parses a layer specification string into a Layer enum.
///
/// Supports formats like:
/// - "global" -> Layer::GlobalBase
/// - "mode/<name>" -> Layer::ModeBase
/// - "scope/<name>" -> Layer::ScopeBase
/// - "project/<name>" -> Layer::ProjectBase
/// - "mode/<m>/scope/<s>" -> Layer::ModeScope
/// - "mode/<m>/project/<p>" -> Layer::ModeProject
/// - "mode/<m>/scope/<s>/project/<p>" -> Layer::ModeScopeProject
///
/// # Arguments
///
/// * `spec` - The layer specification string to parse
/// * `_project` - The project name (currently unused)
///
/// # Returns
///
/// The parsed Layer variant.
///
/// # Errors
///
/// Returns `JinError::Message` if the specification format is invalid.
fn parse_layer_spec(spec: &str, _project: &str) -> Result<Layer> {
    let parts: Vec<&str> = spec.split('/').collect();

    match parts.as_slice() {
        ["global"] => Ok(Layer::GlobalBase),
        ["mode", name] => Ok(Layer::ModeBase {
            mode: name.to_string(),
        }),
        ["scope", name] => Ok(Layer::ScopeBase {
            scope: name.to_string(),
        }),
        ["project", name] => Ok(Layer::ProjectBase {
            project: name.to_string(),
        }),
        ["mode", mode_name, "scope", scope_name] => Ok(Layer::ModeScope {
            mode: mode_name.to_string(),
            scope: scope_name.to_string(),
        }),
        ["mode", mode_name, "project", proj_name] => Ok(Layer::ModeProject {
            mode: mode_name.to_string(),
            project: proj_name.to_string(),
        }),
        ["mode", mode_name, "scope", scope_name, "project", proj_name] => {
            Ok(Layer::ModeScopeProject {
                mode: mode_name.to_string(),
                scope: scope_name.to_string(),
                project: proj_name.to_string(),
            })
        }
        _ => Err(JinError::Message(format!(
            "Invalid layer specification: '{}'. Expected format: global, mode/<name>, scope/<name>, project/<name>, mode/<m>/scope/<s>, mode/<m>/project/<p>, or mode/<m>/scope/<s>/project/<p>",
            spec
        ))),
    }
}

// ===== FILE DIFF COMPUTATION =====

/// Computes the differences between two sets of files.
///
/// Collects all unique paths from both HashMaps and determines the status
/// of each file (Added, Deleted, Modified, or Unchanged).
///
/// # Arguments
///
/// * `old_files` - Files from the source layer
/// * `new_files` - Files from the target layer
///
/// # Returns
///
/// A vector of FileDiff structures sorted by path.
fn diff_files_in_layer(
    old_files: &HashMap<String, Vec<u8>>,
    new_files: &HashMap<String, Vec<u8>>,
) -> Vec<FileDiff> {
    let all_paths: HashSet<&str> = old_files
        .keys()
        .chain(new_files.keys())
        .map(|s| s.as_str())
        .collect();

    let mut diffs = Vec::new();

    for path in all_paths {
        let old_bytes = old_files.get(path);
        let new_bytes = new_files.get(path);

        let old_str = old_bytes.and_then(|b| String::from_utf8(b.clone()).ok());
        let new_str = new_bytes.and_then(|b| String::from_utf8(b.clone()).ok());

        let status = match (old_bytes, new_bytes) {
            (None, Some(_)) => DiffStatus::Added,
            (Some(_), None) => DiffStatus::Deleted,
            (Some(o), Some(n)) => {
                if o == n {
                    DiffStatus::Unchanged
                } else {
                    DiffStatus::Modified
                }
            }
            (None, None) => continue,
        };

        diffs.push(FileDiff {
            path: path.to_string(),
            status,
            old_content: old_str,
            new_content: new_str,
        });
    }

    diffs.sort_by(|a, b| a.path.cmp(&b.path));
    diffs
}

// ===== UNIFIED DIFF FORMAT =====

/// Formats a file diff in unified diff format.
///
/// Generates a header with source and target layer information,
/// then formats the content changes using the TextDiff algorithm.
///
/// # Arguments
///
/// * `file_diff` - The file diff to format
/// * `source_layer` - The source layer (for header)
/// * `target_layer` - The target layer (for header)
///
/// # Returns
///
/// A formatted string in unified diff format.
fn format_unified_diff(
    file_diff: &FileDiff,
    source_layer: &Layer,
    target_layer: &Layer,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "{}--- a/{}\t{}{}\n",
        ANSI_RED, file_diff.path, source_layer, ANSI_RESET
    ));
    output.push_str(&format!(
        "{}+++ b/{}\t{}{}\n",
        ANSI_GREEN, file_diff.path, target_layer, ANSI_RESET
    ));

    match &file_diff.status {
        DiffStatus::Added => {
            if let Some(content) = &file_diff.new_content {
                output.push_str(&format_text_diff_as_added(content));
            }
        }
        DiffStatus::Deleted => {
            if let Some(content) = &file_diff.old_content {
                output.push_str(&format_text_diff_as_deleted(content));
            }
        }
        DiffStatus::Modified => {
            if let (Some(old), Some(new)) = (&file_diff.old_content, &file_diff.new_content) {
                output.push_str(&format_text_diff(old, new));
            }
        }
        DiffStatus::Unchanged => {}
    }

    output
}

/// Formats text content as added (all green with + prefix).
fn format_text_diff_as_added(content: &str) -> String {
    let mut output = String::new();
    for line in content.lines() {
        output.push_str(&format!("{}+{}{}\n", ANSI_GREEN, line, ANSI_RESET));
    }
    output
}

/// Formats text content as deleted (all red with - prefix).
fn format_text_diff_as_deleted(content: &str) -> String {
    let mut output = String::new();
    for line in content.lines() {
        output.push_str(&format!("{}-{}{}\n", ANSI_RED, line, ANSI_RESET));
    }
    output
}

/// Formats the difference between two text strings.
///
/// Uses the similar crate's Myers algorithm to compute and format
/// the line-by-line differences.
fn format_text_diff(old: &str, new: &str) -> String {
    let diff = TextDiff::from_lines(old, new);

    // Get unified diff format as a string
    let unified = diff.unified_diff().to_string();

    // Colorize the unified diff
    let mut output = String::new();
    for line in unified.lines() {
        if line.starts_with('-') && !line.starts_with("---") {
            output.push_str(&format!("{}{}{}\n", ANSI_RED, line, ANSI_RESET));
        } else if line.starts_with('+') && !line.starts_with("+++") {
            output.push_str(&format!("{}{}{}\n", ANSI_GREEN, line, ANSI_RESET));
        } else {
            output.push_str(&format!("{}\n", line));
        }
    }

    output
}

// ===== LAYER DIFF =====

/// Compares two layers and displays their differences.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `layer1` - The source layer
/// * `layer2` - The target layer
/// * `project` - The project name
///
/// # Returns
///
/// Exit code: 0 if no differences, 1 if differences found.
///
/// # Errors
///
/// Propagates errors from layer file reading or parsing.
fn diff_layers(
    repo: &JinRepo,
    layer1: &Layer,
    layer2: &Layer,
    project: &str,
) -> Result<i32> {
    let merger = LayerMerge::new(repo, project);

    // Only versioned layers can be read
    if !layer1.is_versioned() {
        return Err(JinError::Message(format!(
            "Cannot diff layer '{}': only versioned layers can be compared",
            layer1
        )));
    }
    if !layer2.is_versioned() {
        return Err(JinError::Message(format!(
            "Cannot diff layer '{}': only versioned layers can be compared",
            layer2
        )));
    }

    // Use merge_subset to get files from individual layers
    let old_result = merger.merge_subset(&[layer1.clone()])?;
    let new_result = merger.merge_subset(&[layer2.clone()])?;

    // Convert MergeValues to HashMap<String, Vec<u8>>
    let old_files: HashMap<String, Vec<u8>> = convert_merge_result_to_bytes(old_result)?;
    let new_files: HashMap<String, Vec<u8>> = convert_merge_result_to_bytes(new_result)?;

    // Compute differences
    let diffs = diff_files_in_layer(&old_files, &new_files);

    // Filter to only changed files
    let changed_diffs: Vec<_> = diffs
        .into_iter()
        .filter(|d| d.status != DiffStatus::Unchanged)
        .collect();

    if changed_diffs.is_empty() {
        println!("No differences found");
        return Ok(0);
    }

    // Display each changed file
    for file_diff in &changed_diffs {
        print!("{}", format_unified_diff(file_diff, layer1, layer2));
        let _ = io::stdout().flush();
    }

    println!("\n{} file(s) changed", changed_diffs.len());
    Ok(1)
}

/// Convert IndexMap<String, MergeValue> to HashMap<String, Vec<u8>>.
fn convert_merge_result_to_bytes(
    result: indexmap::IndexMap<String, crate::merge::value::MergeValue>,
) -> Result<HashMap<String, Vec<u8>>> {
    let mut files = HashMap::new();
    for (path, merge_value) in result {
        let content = serde_json::to_string_pretty(&merge_value).map_err(|e| {
            JinError::Message(format!("Failed to serialize merged result: {}", e))
        })?;
        files.insert(path, content.into_bytes());
    }
    Ok(files)
}

// ===== WORKSPACE DIFF =====

/// Compares workspace files to the merged layer result.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `project` - The project name
/// * `context` - The project context with active mode/scope
///
/// # Returns
///
/// Exit code: 0 if no differences, 1 if differences found.
///
/// # Errors
///
/// Propagates errors from layer merging or file reading.
fn diff_workspace(
    repo: &JinRepo,
    project: &str,
    context: &ProjectContext,
) -> Result<i32> {
    // Get merged result
    let mut merger = LayerMerge::new(repo, project);
    if let Some(ref mode) = context.mode {
        merger = merger.with_mode(mode);
    }
    if let Some(ref scope) = context.scope {
        merger = merger.with_scope(scope);
    }

    let merged_result = merger.merge_all()?;

    // Convert merged result to HashMap<String, Vec<u8>>
    let merged_files: HashMap<String, Vec<u8>> = convert_merge_result_to_bytes(merged_result)?;

    // Read workspace files
    let workspace_root = std::env::current_dir()?;
    let workspace_dir = workspace_root.join(".jin/workspace");

    let mut workspace_files: HashMap<String, Vec<u8>> = HashMap::new();

    if workspace_dir.exists() {
        for entry in walkdir::WalkDir::new(&workspace_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(rel_path) = entry.path().strip_prefix(&workspace_dir) {
                    let path_str = rel_path.to_string_lossy().to_string();
                    if let Ok(content) = std::fs::read(entry.path()) {
                        workspace_files.insert(path_str, content);
                    }
                }
            }
        }
    }

    // Compute differences
    let diffs = diff_files_in_layer(&merged_files, &workspace_files);

    // Filter to only changed files
    let changed_diffs: Vec<_> = diffs
        .into_iter()
        .filter(|d| d.status != DiffStatus::Unchanged)
        .collect();

    if changed_diffs.is_empty() {
        println!("No differences found between workspace and merged result");
        return Ok(0);
    }

    // Display each changed file
    for file_diff in &changed_diffs {
        print!(
            "{}",
            format_unified_diff(
                file_diff,
                &Layer::ProjectBase {
                    project: project.to_string()
                },
                &Layer::ProjectBase {
                    project: project.to_string()
                }
            )
        );
        let _ = io::stdout().flush();
    }

    println!("\n{} file(s) changed", changed_diffs.len());
    Ok(1)
}

// ===== STAGED DIFF =====

/// Compares staged files to their committed versions.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `project` - The project name
///
/// # Returns
///
/// Exit code: 0 if no differences, 1 if differences found.
///
/// # Errors
///
/// Propagates errors from staging index loading or file reading.
fn diff_staged(repo: &JinRepo, project: &str) -> Result<i32> {
    let workspace_root = std::env::current_dir()?;
    let staging =
        StagingIndex::load_from_disk(&workspace_root).unwrap_or_else(|_| StagingIndex::new());

    if staging.is_empty() {
        println!("No staged files");
        return Ok(0);
    }

    let merger = LayerMerge::new(repo, project);
    let mut all_diffs = Vec::new();

    // Group staged entries by layer
    for entry in staging.all_entries() {
        let layer = entry.layer.clone();

        if !layer.is_versioned() {
            continue;
        }

        // Read committed content from layer
        let committed_result = merger.merge_subset(&[layer.clone()])?;
        let committed_files: HashMap<String, Vec<u8>> = convert_merge_result_to_bytes(committed_result)?;

        // Get staged content from the staging directory
        let staging_dir = workspace_root.join(".jin/staging/files");
        // Use the hash bytes directly as the filename (first 16 chars as hex representation)
        let hash_str: String = entry.content_hash.iter()
            .take(16)
            .map(|b| format!("{:02x}", b))
            .collect();
        let staged_path = staging_dir.join(&hash_str);

        let staged_str = if staged_path.exists() {
            std::fs::read_to_string(&staged_path).map_err(|_| {
                JinError::Message(format!(
                    "Failed to read staged file '{}'",
                    entry.path.display()
                ))
            })?
        } else {
            // Fallback: try to read from the original path
            std::fs::read_to_string(&entry.path).map_err(|_| {
                JinError::Message(format!(
                    "Staged file '{}' is not available",
                    entry.path.display()
                ))
            })?
        };

        // Get committed content for this file (if exists)
        let file_path = entry.path.to_string_lossy().to_string();
        let committed_str = committed_files
            .get(&file_path)
            .and_then(|b| String::from_utf8(b.clone()).ok());

        // Create file diff
        // Since staged_str is always Some (we just read it), we check if committed_str is None
        let status = match &committed_str {
            None => DiffStatus::Added,
            Some(c) => {
                if c == &staged_str {
                    DiffStatus::Unchanged
                } else {
                    DiffStatus::Modified
                }
            }
        };

        if status != DiffStatus::Unchanged {
            all_diffs.push(FileDiff {
                path: entry.path.to_string_lossy().to_string(),
                status,
                old_content: committed_str,
                new_content: Some(staged_str),
            });
        }
    }

    if all_diffs.is_empty() {
        println!("No differences found between staged and committed files");
        return Ok(0);
    }

    // Display each changed file
    for file_diff in &all_diffs {
        let layer = &staging
            .get_entry(Path::new(&file_diff.path))
            .map(|e| e.layer.clone())
            .unwrap_or(Layer::ProjectBase {
                project: project.to_string(),
            });

        print!("{}", format_unified_diff(file_diff, layer, layer));
        let _ = io::stdout().flush();
    }

    println!("\n{} file(s) changed", all_diffs.len());
    Ok(1)
}

// ===== PROJECT NAME DETECTION =====

/// Detects the project name from Git remote or directory name.
///
/// First tries to get the project name from the git remote origin URL.
/// Falls back to the directory name if no remote is configured.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// The detected project name.
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if not in a Git repository.
fn detect_project_name(workspace_root: &Path) -> Result<String> {
    let repo = git2::Repository::discover(workspace_root).map_err(|_| JinError::RepoNotFound {
        path: workspace_root.display().to_string(),
    })?;

    // Try to get from git remote origin
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.rsplit('/').next() {
                let name = name.trim_end_matches(".git");
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }

    // Fallback to directory name
    workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| JinError::Message("Cannot determine project name".to_string()))
}

// ===== MAIN EXECUTE =====

/// Execute the diff command.
///
/// This is the main entry point for the `jin diff` command.
///
/// # Behavior
///
/// - With no arguments: compares workspace to merged result
/// - With two layer specs: compares the two layers
/// - With --staged: compares staged files to committed versions
///
/// # Exit Codes
///
/// - 0: No differences found
/// - 1: Differences found (script-friendly)
/// - 2: Error occurred
///
/// # Arguments
///
/// * `cmd` - The diff command containing CLI arguments
///
/// # Errors
///
/// Returns `JinError::RepoNotFound` if not in a Git repository.
/// Returns `JinError::Message` for other error conditions.
///
/// # Note
///
/// This function calls `std::process::exit()` internally, so it
/// will not return normally when differences are found.
pub fn execute(cmd: &DiffCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Load project context
    let context = ProjectContext::load(&workspace_root)?;

    // 3. Detect project name
    let project_name = detect_project_name(&workspace_root)?;

    // 4. Validate Git repository exists
    let _git_repo = git2::Repository::discover(&workspace_root).map_err(|_| {
        JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        }
    })?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 6. Determine diff mode and execute
    let exit_code = if cmd.staged {
        diff_staged(&repo, &project_name)?
    } else {
        match (&cmd.layer1, &cmd.layer2) {
            (None, None) => diff_workspace(&repo, &project_name, &context)?,
            (Some(spec1), Some(spec2)) => {
                let layer1 = parse_layer_spec(spec1, &project_name)?;
                let layer2 = parse_layer_spec(spec2, &project_name)?;
                diff_layers(&repo, &layer1, &layer2, &project_name)?
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(JinError::Message(
                    "Specify both layers or neither layer".to_string(),
                ));
            }
        }
    };

    // Use process::exit to set correct exit code
    std::process::exit(exit_code);
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== parse_layer_spec Tests =====

    #[test]
    fn test_parse_layer_spec_global() {
        let layer = parse_layer_spec("global", "testproject").unwrap();
        assert_eq!(layer, Layer::GlobalBase);
    }

    #[test]
    fn test_parse_layer_spec_mode() {
        let layer = parse_layer_spec("mode/claude", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_scope() {
        let layer = parse_layer_spec("scope/python", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_project() {
        let layer = parse_layer_spec("project/myproject", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ProjectBase {
                project: "myproject".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_mode_scope() {
        let layer = parse_layer_spec("mode/claude/scope/python", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_mode_project() {
        let layer = parse_layer_spec("mode/claude/project/myapp", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_mode_scope_project() {
        let layer =
            parse_layer_spec("mode/claude/scope/python/project/myapp", "testproject").unwrap();
        assert_eq!(
            layer,
            Layer::ModeScopeProject {
                mode: "claude".to_string(),
                scope: "python".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_parse_layer_spec_invalid() {
        let result = parse_layer_spec("invalid/format", "testproject");
        assert!(result.is_err());
    }

    // ===== diff_files_in_layer Tests =====

    #[test]
    fn test_diff_files_in_layer_added() {
        let old: HashMap<String, Vec<u8>> = HashMap::new();
        let mut new: HashMap<String, Vec<u8>> = HashMap::new();

        new.insert("new.txt".to_string(), b"new content".to_vec());

        let diffs = diff_files_in_layer(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "new.txt");
        assert_eq!(diffs[0].status, DiffStatus::Added);
        assert_eq!(diffs[0].old_content, None);
        assert_eq!(diffs[0].new_content, Some("new content".to_string()));
    }

    #[test]
    fn test_diff_files_in_layer_deleted() {
        let mut old: HashMap<String, Vec<u8>> = HashMap::new();
        let new: HashMap<String, Vec<u8>> = HashMap::new();

        old.insert("old.txt".to_string(), b"old content".to_vec());

        let diffs = diff_files_in_layer(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "old.txt");
        assert_eq!(diffs[0].status, DiffStatus::Deleted);
        assert_eq!(diffs[0].old_content, Some("old content".to_string()));
        assert_eq!(diffs[0].new_content, None);
    }

    #[test]
    fn test_diff_files_in_layer_modified() {
        let mut old: HashMap<String, Vec<u8>> = HashMap::new();
        let mut new: HashMap<String, Vec<u8>> = HashMap::new();

        old.insert("config.txt".to_string(), b"old value".to_vec());
        new.insert("config.txt".to_string(), b"new value".to_vec());

        let diffs = diff_files_in_layer(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "config.txt");
        assert_eq!(diffs[0].status, DiffStatus::Modified);
    }

    #[test]
    fn test_diff_files_in_layer_unchanged() {
        let mut old: HashMap<String, Vec<u8>> = HashMap::new();
        let mut new: HashMap<String, Vec<u8>> = HashMap::new();

        old.insert("same.txt".to_string(), b"same content".to_vec());
        new.insert("same.txt".to_string(), b"same content".to_vec());

        let diffs = diff_files_in_layer(&old, &new);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path, "same.txt");
        assert_eq!(diffs[0].status, DiffStatus::Unchanged);
    }

    #[test]
    fn test_diff_files_in_layer_multiple_files() {
        let mut old: HashMap<String, Vec<u8>> = HashMap::new();
        let mut new: HashMap<String, Vec<u8>> = HashMap::new();

        old.insert("unchanged.txt".to_string(), b"same".to_vec());
        old.insert("modified.txt".to_string(), b"old".to_vec());
        old.insert("deleted.txt".to_string(), b"gone".to_vec());

        new.insert("unchanged.txt".to_string(), b"same".to_vec());
        new.insert("modified.txt".to_string(), b"new".to_vec());
        new.insert("added.txt".to_string(), b"new file".to_vec());

        let diffs = diff_files_in_layer(&old, &new);
        assert_eq!(diffs.len(), 4);

        // Check sorting
        assert_eq!(diffs[0].path, "added.txt");
        assert_eq!(diffs[1].path, "deleted.txt");
        assert_eq!(diffs[2].path, "modified.txt");
        assert_eq!(diffs[3].path, "unchanged.txt");

        // Check statuses
        assert_eq!(diffs[0].status, DiffStatus::Added);
        assert_eq!(diffs[1].status, DiffStatus::Deleted);
        assert_eq!(diffs[2].status, DiffStatus::Modified);
        assert_eq!(diffs[3].status, DiffStatus::Unchanged);
    }

    // ===== format_text_diff Tests =====

    #[test]
    fn test_format_text_diff_addition() {
        let old = "";
        let new = "new line";

        let output = format_text_diff(old, new);
        assert!(output.contains("+new line"));
    }

    #[test]
    fn test_format_text_diff_deletion() {
        let old = "old line";
        let new = "";

        let output = format_text_diff(old, new);
        assert!(output.contains("-old line"));
    }

    #[test]
    fn test_format_text_diff_modification() {
        let old = "line 1\nline 2\nline 3";
        let new = "line 1\nline 2 modified\nline 3";

        let output = format_text_diff(old, new);
        assert!(output.contains("-line 2"));
        assert!(output.contains("+line 2 modified"));
    }

    // ===== detect_project_name Tests =====

    #[test]
    fn test_detect_project_name_from_git() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let repo = git2::Repository::init(project_dir).unwrap();

        // Create a fake remote
        repo.remote("origin", "https://github.com/user/myproject.git")
            .unwrap();

        let name = detect_project_name(project_dir).unwrap();
        assert_eq!(name, "myproject");
    }

    #[test]
    fn test_detect_project_name_from_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Initialize a Git repo without remote
        git2::Repository::init(project_dir).unwrap();

        // TempDir creates a directory with a unique name, check we get something
        let name = detect_project_name(project_dir).unwrap();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_detect_project_name_no_git() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let result = detect_project_name(project_dir);
        assert!(result.is_err());
        assert!(matches!(result, Err(JinError::RepoNotFound { .. })));
    }

    // ===== format_unified_diff Tests =====

    #[test]
    fn test_format_unified_diff_header() {
        let file_diff = FileDiff {
            path: "test.txt".to_string(),
            status: DiffStatus::Modified,
            old_content: Some("old".to_string()),
            new_content: Some("new".to_string()),
        };

        let source = Layer::GlobalBase;
        let target = Layer::ProjectBase {
            project: "test".to_string(),
        };

        let output = format_unified_diff(&file_diff, &source, &target);
        assert!(output.contains("--- a/test.txt"));
        assert!(output.contains("+++ b/test.txt"));
        assert!(output.contains("global"));
        assert!(output.contains("project/test"));
    }
}
