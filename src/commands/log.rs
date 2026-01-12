//! Implementation of `jin log`
//!
//! Shows commit history for layers.

use crate::cli::LogArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{refs::RefOps, JinRepo};
use chrono::{DateTime, Utc};
use git2::Sort;
use std::collections::HashMap;

/// Execute the log command
///
/// Shows commit history.
pub fn execute(args: LogArgs) -> Result<()> {
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

    // Determine which layers to show history for
    if let Some(layer_name) = &args.layer {
        // Show history for specific layer
        let layer = parse_layer_name(layer_name)?;
        show_layer_history(git_repo, layer, &context, args.count)?;
    } else {
        // Show history for all layers with commits
        // Discover all layer refs dynamically
        let all_refs = repo.list_refs("refs/jin/layers/**")?;

        // Group refs by layer type
        let mut layer_refs: HashMap<Layer, Vec<String>> = HashMap::new();
        for path in all_refs {
            if let Some(layer) = Layer::parse_layer_from_ref_path(&path) {
                layer_refs.entry(layer).or_default().push(path);
            }
        }

        // Display in precedence order
        let all_layers = Layer::all_in_precedence_order();
        let mut shown_any = false;

        for layer in &all_layers {
            // Skip layers that don't apply to current context
            if layer.requires_mode() && context.mode.is_none() {
                continue;
            }
            if layer.requires_scope() && context.scope.is_none() {
                continue;
            }

            // Get all refs for this layer type
            if let Some(refs) = layer_refs.get(layer) {
                for path in refs {
                    if shown_any {
                        println!();
                    }
                    println!("=== {} ===", layer);
                    println!();
                    show_history_for_ref_path(git_repo, path, *layer, args.count)?;
                    shown_any = true;
                }
            }
        }

        if !shown_any {
            println!("No commits found in any layer");
        }
    }

    Ok(())
}

/// Show commit history for a specific layer
fn show_layer_history(
    repo: &git2::Repository,
    layer: Layer,
    context: &ProjectContext,
    count: usize,
) -> Result<()> {
    let ref_path = layer.ref_path(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );

    show_history_for_ref_path(repo, &ref_path, layer, count)
}

/// Show commit history for a specific ref path
///
/// This is a helper function that displays commit history for an arbitrary
/// ref path, used internally for dynamic layer ref discovery.
fn show_history_for_ref_path(
    repo: &git2::Repository,
    ref_path: &str,
    layer: Layer,
    count: usize,
) -> Result<()> {
    // Check if ref exists
    let _reference = match repo.find_reference(ref_path) {
        Ok(r) => r,
        Err(_) => {
            println!("No commits yet for layer: {}", layer);
            return Ok(());
        }
    };

    // Create revwalk
    let mut revwalk = repo.revwalk()?;
    revwalk.push_ref(ref_path)?;
    revwalk.set_sorting(Sort::TIME)?;

    // Iterate through commits
    for (i, oid_result) in revwalk.enumerate() {
        if i >= count {
            break;
        }

        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Format commit hash (short)
        let hash_short = &oid.to_string()[..7];

        // Get commit metadata
        let author = commit.author();
        let author_name = author.name().unwrap_or("unknown");
        let author_email = author.email().unwrap_or("unknown");
        let message = commit.message().unwrap_or("(no message)");

        // Format timestamp
        let time = commit.time();
        let timestamp = DateTime::from_timestamp(time.seconds(), 0)
            .unwrap_or_else(|| DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH));

        // Count files changed in this commit
        let file_count = count_files_in_commit(repo, &commit)?;

        // Display commit
        println!("commit {} ({})", hash_short, layer);
        println!("Author: {} <{}>", author_name, author_email);
        println!("Date:   {}", timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!();
        println!("    {}", message.trim());
        println!();
        println!("    {} file(s) changed", file_count);
        println!();
    }

    Ok(())
}

/// Count files in a commit by comparing with parent
fn count_files_in_commit(repo: &git2::Repository, commit: &git2::Commit) -> Result<usize> {
    let tree = commit.tree()?;

    // If no parent, count all files in tree
    if commit.parent_count() == 0 {
        let mut count = 0;
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                count += 1;
            }
            git2::TreeWalkResult::Ok
        })?;
        return Ok(count);
    }

    // Otherwise, diff with parent
    let parent = commit.parent(0)?;
    let parent_tree = parent.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

    Ok(diff.deltas().count())
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
    fn test_parse_layer_name() {
        assert!(matches!(
            parse_layer_name("global-base"),
            Ok(Layer::GlobalBase)
        ));
        assert!(matches!(parse_layer_name("mode-base"), Ok(Layer::ModeBase)));
        assert!(matches!(
            parse_layer_name("project-base"),
            Ok(Layer::ProjectBase)
        ));
        assert!(parse_layer_name("invalid").is_err());
    }

    #[test]
    fn test_execute_not_initialized() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let args = LogArgs {
            layer: None,
            count: 10,
        };

        let result = execute(args);
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_count_files_empty_commit() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin-test");
        let repo = git2::Repository::init_bare(&repo_path).unwrap();

        // Create an empty commit
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let tree_id = {
            let tb = repo.treebuilder(None).unwrap();
            tb.write().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        let commit_id = repo
            .commit(
                Some("refs/heads/test"),
                &sig,
                &sig,
                "Empty commit",
                &tree,
                &[],
            )
            .unwrap();

        let commit = repo.find_commit(commit_id).unwrap();
        let count = count_files_in_commit(&repo, &commit).unwrap();
        assert_eq!(count, 0);
    }
}
