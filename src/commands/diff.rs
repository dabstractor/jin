//! Implementation of `jin diff`
//!
//! Shows differences between layers, workspace, or staged changes.

use crate::cli::DiffArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::JinRepo;
use crate::staging::StagingIndex;
use git2::{DiffFormat, DiffOptions};

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

    println!("Comparing workspace vs {}", layer);
    println!();

    // Note: For now, we'll show which files exist in the layer
    // Full workspace diff would require comparing actual workspace files
    let mut file_count = 0;
    tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            file_count += 1;
            println!("  {}", entry.name().unwrap_or("<unnamed>"));
        }
        git2::TreeWalkResult::Ok
    })?;

    println!();
    println!("Layer contains {} files", file_count);

    Ok(())
}

/// Diff workspace vs workspace-active (merged layers)
fn diff_workspace_vs_workspace_active(
    _repo: &git2::Repository,
    _context: &ProjectContext,
) -> Result<()> {
    // For now, show a simplified version
    // Full implementation would require materializing merged layers
    println!("Comparing workspace vs workspace-active");
    println!();
    println!("(Workspace diff not yet fully implemented)");

    Ok(())
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
    fn test_parse_layer_name() {
        assert!(matches!(
            parse_layer_name("global-base"),
            Ok(Layer::GlobalBase)
        ));
        assert!(matches!(parse_layer_name("mode-base"), Ok(Layer::ModeBase)));
        assert!(parse_layer_name("invalid").is_err());
    }

    #[test]
    fn test_execute_not_initialized() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
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
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();

        // Set JIN_DIR to an isolated directory for this test
        let jin_dir = temp.path().join(".jin_global");
        std::env::set_var("JIN_DIR", &jin_dir);

        std::env::set_current_dir(temp.path()).unwrap();

        // Initialize .jin directory
        std::fs::create_dir(".jin").unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();

        let args = DiffArgs {
            layer1: None,
            layer2: None,
            staged: true,
        };

        let result = execute(args);
        assert!(result.is_ok());
    }
}
