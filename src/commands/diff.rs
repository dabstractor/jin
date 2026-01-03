//! Implementation of `jin diff`
//!
//! Shows differences between layers, workspace, or staged changes.

use crate::cli::DiffArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, TreeOps};
use crate::merge::{get_applicable_layers, merge_layers, LayerMergeConfig};
use crate::staging::StagingIndex;
use crate::staging::WorkspaceMetadata;
use git2::{DiffFormat, DiffOptions};
use std::path::Path;

/// Execute the diff command
///
/// Shows differences between layers.
pub fn execute(args: DiffArgs) -> Result<()> {
    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;
    let git_repo = repo.inner();

    // Determine diff mode
    if args.staged {
        // Show staged changes
        show_staged_diff(git_repo, &context)?;
    } else if let (Some(layer1_name), Some(layer2_name)) = (&args.layer1, &args.layer2) {
        // Compare two specific layers
        let layer1 = parse_layer_name(layer1_name)?;
        let layer2 = parse_layer_name(layer2_name)?;
        diff_layers(git_repo, layer1, layer2, &context)?;
    } else if let Some(layer_name) = &args.layer1 {
        // Compare workspace vs specified layer
        let layer = parse_layer_name(layer_name)?;
        diff_workspace_vs_layer(git_repo, layer, &context)?;
    } else {
        // Default: compare workspace vs workspace-active (merged layers)
        diff_workspace_vs_workspace_active(git_repo, &context)?;
    }

    Ok(())
}

/// Show staged changes
fn show_staged_diff(_repo: &git2::Repository, _context: &ProjectContext) -> Result<()> {
    let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    if staging.is_empty() {
        println!("No staged changes");
        return Ok(());
    }

    println!("Staged changes:");
    println!();

    // Show each staged file
    for entry in staging.entries() {
        let path = &entry.path;
        println!("  {} -> {}", path.display(), entry.target_layer);

        // Try to show diff if file exists in workspace
        if path.exists() {
            // Get blob from Jin repo
            if let Ok(oid) = git2::Oid::from_str(&entry.content_hash) {
                if let Ok(blob) = _repo.find_blob(oid) {
                    // Read workspace content
                    if let Ok(workspace_content) = std::fs::read(path) {
                        // Compare
                        if blob.content() != workspace_content.as_slice() {
                            println!("    (modified since staging)");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Diff two specific layers
fn diff_layers(
    repo: &git2::Repository,
    layer1: Layer,
    layer2: Layer,
    context: &ProjectContext,
) -> Result<()> {
    let ref1 = layer1.ref_path(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );
    let ref2 = layer2.ref_path(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );

    // Get trees for both layers
    let tree1 = match repo.find_reference(&ref1) {
        Ok(r) => r.peel_to_tree()?,
        Err(_) => {
            return Err(JinError::Other(format!("Layer {} has no commits", layer1)));
        }
    };

    let tree2 = match repo.find_reference(&ref2) {
        Ok(r) => r.peel_to_tree()?,
        Err(_) => {
            return Err(JinError::Other(format!("Layer {} has no commits", layer2)));
        }
    };

    // Create diff
    let mut opts = DiffOptions::new();
    opts.context_lines(3);

    let diff = repo.diff_tree_to_tree(Some(&tree1), Some(&tree2), Some(&mut opts))?;

    if diff.deltas().count() == 0 {
        println!("No differences between {} and {}", layer1, layer2);
        return Ok(());
    }

    // Print diff header
    println!("diff --jin a/{} b/{}", layer1, layer2);
    println!();

    // Print diff
    print_diff(&diff)?;

    Ok(())
}

/// Diff workspace vs specific layer
fn diff_workspace_vs_layer(
    repo: &git2::Repository,
    layer: Layer,
    context: &ProjectContext,
) -> Result<()> {
    let ref_path = layer.ref_path(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );

    // Get tree for layer
    let tree = match repo.find_reference(&ref_path) {
        Ok(r) => r.peel_to_tree()?,
        Err(_) => {
            return Err(JinError::Other(format!("Layer {} has no commits", layer)));
        }
    };

    let tree_id = tree.id();

    println!("Comparing workspace vs {}", layer);
    println!();

    // Collect all files in the layer tree
    let jin_repo = JinRepo::open()?;
    let layer_files = jin_repo.list_tree_files(tree_id)?;

    let mut has_changes = false;

    for file_path in layer_files {
        let path = Path::new(&file_path);

        // Read layer content
        let layer_content = match jin_repo.read_file_from_tree(tree_id, path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        // Check if file exists in workspace
        if path.exists() {
            // Read workspace content
            let workspace_content = match std::fs::read(path) {
                Ok(content) => content,
                Err(_) => continue,
            };

            // Compare contents
            if layer_content != workspace_content {
                has_changes = true;

                // Generate diff between layer and workspace
                let layer_str = String::from_utf8_lossy(&layer_content);
                let workspace_str = String::from_utf8_lossy(&workspace_content);

                println!("--- a/{} (layer)", file_path);
                println!("+++ b/{} (workspace)", file_path);

                // Print a simple line-by-line diff
                let layer_lines: Vec<&str> = layer_str.lines().collect();
                let workspace_lines: Vec<&str> = workspace_str.lines().collect();

                print_text_diff(&layer_lines, &workspace_lines);
                println!();
            }
        } else {
            // File exists in layer but not in workspace
            has_changes = true;
            println!("Only in {}: {}", layer, file_path);
            println!();
        }
    }

    if !has_changes {
        println!("No differences between workspace and {}", layer);
    }

    Ok(())
}

/// Print a simple line-by-line diff for text files
fn print_text_diff(old_lines: &[&str], new_lines: &[&str]) {
    // Simple line-by-line comparison with unified diff output
    let mut old_idx = 0;
    let mut new_idx = 0;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        let old_line = if old_idx < old_lines.len() {
            old_lines[old_idx]
        } else {
            ""
        };
        let new_line = if new_idx < new_lines.len() {
            new_lines[new_idx]
        } else {
            ""
        };

        if old_line == new_line {
            // Lines are equal
            println!(" {}", old_line);
            old_idx += 1;
            new_idx += 1;
        } else {
            // Lines differ - find the next match
            let old_next = find_next_match(old_idx, old_lines, new_idx, new_lines);
            let new_next = find_next_match(new_idx, new_lines, old_idx, old_lines);

            // Print deletions from old
            while old_idx < old_lines.len() && (old_idx < old_next.0 || old_next.0 == usize::MAX) {
                println!("\x1b[31m-{}\x1b[0m", old_lines[old_idx]);
                old_idx += 1;
            }

            // Print insertions from new
            while new_idx < new_lines.len() && (new_idx < new_next.0 || new_next.0 == usize::MAX) {
                println!("\x1b[32m+{}\x1b[0m", new_lines[new_idx]);
                new_idx += 1;
            }
        }
    }
}

/// Find the next matching line between two sequences
fn find_next_match(
    current_idx: usize,
    current_lines: &[&str],
    other_idx: usize,
    other_lines: &[&str],
) -> (usize, usize) {
    let search_radius = 5; // Look ahead up to 5 lines

    for i in 0..=search_radius {
        let curr_pos = current_idx + i;
        if curr_pos >= current_lines.len() {
            break;
        }
        let curr_line = current_lines[curr_pos];

        for j in 0..=search_radius {
            let other_pos = other_idx + j;
            if other_pos >= other_lines.len() {
                break;
            }
            if curr_line == other_lines[other_pos] {
                return (curr_pos, other_pos);
            }
        }
    }

    (usize::MAX, usize::MAX)
}

/// Diff workspace vs workspace-active (merged layers)
fn diff_workspace_vs_workspace_active(
    _repo: &git2::Repository,
    context: &ProjectContext,
) -> Result<()> {
    println!("Comparing workspace vs workspace-active");
    println!();

    // Check if workspace metadata exists
    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => {
            println!("No workspace metadata found.");
            println!("Run 'jin apply' to create an initial workspace state.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Get applicable layers for current context
    let layers = get_applicable_layers(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );

    // Merge layers to get workspace-active content
    let jin_repo = JinRepo::open()?;
    let config = LayerMergeConfig {
        layers,
        mode: context.mode.clone(),
        scope: context.scope.clone(),
        project: context.project.clone(),
    };

    let merged = match merge_layers(&config, &jin_repo) {
        Ok(m) => m,
        Err(JinError::NotFound(_)) => {
            println!("No layers found to merge.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut has_changes = false;

    // Compare each merged file to actual workspace file
    for (path, merged_file) in &merged.merged_files {
        // Serialize merged content to string
        let merged_str = match serialize_merged_content(merged_file) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Read workspace file
        let workspace_str = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                // File doesn't exist in workspace
                has_changes = true;
                println!("Only in workspace-active: {}", path.display());
                println!();
                continue;
            }
        };

        // Compare contents
        if merged_str != workspace_str {
            has_changes = true;

            println!("--- a/{} (workspace-active)", path.display());
            println!("+++ b/{} (workspace)", path.display());

            let merged_lines: Vec<&str> = merged_str.lines().collect();
            let workspace_lines: Vec<&str> = workspace_str.lines().collect();

            print_text_diff(&merged_lines, &workspace_lines);
            println!();
        }
    }

    // Check for files in workspace but not in merged result
    for path in metadata.files.keys() {
        if !merged.merged_files.contains_key(path) {
            has_changes = true;
            println!("Only in workspace: {}", path.display());
            println!();
        }
    }

    if !has_changes {
        println!("No differences between workspace and workspace-active");
    }

    Ok(())
}

/// Serialize merged content to string based on file format
fn serialize_merged_content(merged_file: &crate::merge::MergedFile) -> Result<String> {
    use crate::merge::FileFormat;

    match merged_file.format {
        FileFormat::Json => merged_file.content.to_json_string(),
        FileFormat::Yaml => merged_file.content.to_yaml_string(),
        FileFormat::Toml => merged_file.content.to_toml_string(),
        FileFormat::Ini => merged_file.content.to_ini_string(),
        FileFormat::Text => {
            if let Some(text) = merged_file.content.as_str() {
                Ok(text.to_string())
            } else {
                Err(JinError::Other(
                    "Text file has non-string content".to_string(),
                ))
            }
        }
    }
}

/// Print a git diff with colored output
fn print_diff(diff: &git2::Diff) -> Result<()> {
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let content = std::str::from_utf8(line.content()).unwrap_or("<binary>");

        match origin {
            '+' => print!("\x1b[32m+{}\x1b[0m", content),
            '-' => print!("\x1b[31m-{}\x1b[0m", content),
            ' ' => print!(" {}", content),
            'F' => print!("--- {}", content),
            'T' => print!("+++ {}", content),
            'H' => print!("@@ {}", content),
            _ => print!("{}", content),
        }
        true
    })?;

    Ok(())
}

/// Parse layer name from string
fn parse_layer_name(name: &str) -> Result<Layer> {
    match name {
        "global-base" => Ok(Layer::GlobalBase),
        "mode-base" => Ok(Layer::ModeBase),
        "mode-scope" => Ok(Layer::ModeScope),
        "mode-scope-project" => Ok(Layer::ModeScopeProject),
        "mode-project" => Ok(Layer::ModeProject),
        "scope-base" => Ok(Layer::ScopeBase),
        "project-base" => Ok(Layer::ProjectBase),
        "user-local" => Ok(Layer::UserLocal),
        "workspace-active" => Ok(Layer::WorkspaceActive),
        _ => Err(JinError::Other(format!(
            "Unknown layer: {}. Valid layers: global-base, mode-base, mode-scope, \
             mode-scope-project, mode-project, scope-base, project-base, user-local, workspace-active",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_not_initialized() {
        let temp = tempfile::TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let args = DiffArgs {
            layer1: None,
            layer2: None,
            staged: false,
        };

        let result = execute(args);
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_execute_staged_empty() {
        // setup_unit_test() already creates the staging index
        let _ctx = crate::test_utils::setup_unit_test();

        let args = DiffArgs {
            layer1: None,
            layer2: None,
            staged: true,
        };

        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_layer_name() {
        assert!(matches!(
            parse_layer_name("global-base"),
            Ok(Layer::GlobalBase)
        ));
        assert!(matches!(parse_layer_name("mode-base"), Ok(Layer::ModeBase)));
        assert!(parse_layer_name("invalid").is_err());
    }
}
