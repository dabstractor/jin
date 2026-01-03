//! Implementation of `jin fetch`
//!
//! Downloads remote layer refs without modifying workspace or active layers.
//! This is a safe, read-only operation from the user's perspective.

use crate::core::{JinConfig, JinError, ProjectContext, Result};
use crate::git::remote::build_fetch_options;
use crate::git::{JinRepo, RefOps};
use git2::ErrorCode;
use std::collections::HashMap;

/// Execute the fetch command
///
/// Downloads all layer refs from remote repository and reports available updates.
/// Does NOT modify workspace or active layers - read-only operation.
pub fn execute() -> Result<()> {
    // 1. Load configuration and validate remote exists
    let config = JinConfig::load()?;
    let remote_config = config.remote.ok_or(JinError::Config(
        "No remote configured. Run 'jin link <url>'.".into(),
    ))?;

    // 1.5. Load project context with graceful fallback for uninitialized projects
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => ProjectContext::default(),
        Err(e) => return Err(e),
    };

    // 2. Open Jin repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 3. Find the remote
    let mut remote = repo.find_remote("origin").map_err(|e| {
        if e.code() == ErrorCode::NotFound {
            JinError::Config(
                "Remote 'origin' not found in repository. Run 'jin link <url>'.".into(),
            )
        } else {
            e.into()
        }
    })?;

    // 4. Setup fetch options with callbacks
    let mut fetch_opts = build_fetch_options()?;

    // 5. Perform fetch
    println!("Fetching from origin ({})...", remote_config.url);

    // Fetch using configured refspec from link (no custom refspec needed)
    let refspecs: &[&str] = &[];
    match remote.fetch(refspecs, Some(&mut fetch_opts), None) {
        Ok(()) => {
            println!(); // New line after progress
        }
        Err(e) => {
            println!(); // New line after progress even on error
            return match e.code() {
                ErrorCode::Auth => Err(JinError::Config(
                    "Authentication failed. Check your SSH keys or credentials.\n\
                    Try: ssh -T git@github.com (for GitHub)"
                        .into(),
                )),
                _ => Err(e.into()),
            };
        }
    }

    // 6. Report available updates
    report_updates(&jin_repo, &context)?;

    Ok(())
}

/// Report available updates by comparing local and remote refs
fn report_updates(jin_repo: &JinRepo, context: &ProjectContext) -> Result<()> {
    // Store context for P2.M3.T2 filtering (suppress unused warning until then)
    let _context = context;

    // Get all remote refs in Jin namespace
    let remote_refs = jin_repo.list_refs("refs/jin/layers/*")?;

    if remote_refs.is_empty() {
        println!("No remote configurations found");
        return Ok(());
    }

    // Compare with local refs to find updates
    let mut updates: HashMap<String, UpdateInfo> = HashMap::new();

    for remote_ref in &remote_refs {
        // Skip user-local layer (never synced)
        if remote_ref.contains("/local") {
            continue;
        }

        let remote_ref_obj = jin_repo.find_ref(remote_ref)?;
        let remote_oid = remote_ref_obj
            .target()
            .ok_or(JinError::Git(git2::Error::from_str(
                "Remote ref has no target",
            )))?;

        // Check if we have this ref locally
        let has_local = jin_repo.ref_exists(remote_ref);
        let is_update = if has_local {
            let local_oid = jin_repo.resolve_ref(remote_ref)?;
            local_oid != remote_oid
        } else {
            true // New ref
        };

        if is_update {
            // Parse layer type from ref path
            let layer_path = remote_ref
                .strip_prefix("refs/jin/layers/")
                .unwrap_or(remote_ref);

            // Determine layer category for grouping
            let category = categorize_layer(layer_path);

            updates
                .entry(category)
                .or_insert_with(|| UpdateInfo {
                    category: layer_path.to_string(),
                    refs: Vec::new(),
                })
                .refs
                .push(layer_path.to_string());
        }
    }

    // Print results
    if updates.is_empty() {
        println!("Already up to date");
    } else {
        println!("\nUpdates available:");
        let mut categories: Vec<_> = updates.keys().collect();
        categories.sort();

        for category in categories {
            let info = &updates[category];
            println!("  - {} ({} file(s))", info.refs[0], info.refs.len());
        }

        println!("\nRun 'jin pull' to merge updates");
    }

    Ok(())
}

/// Information about updates for a layer
#[derive(Debug)]
struct UpdateInfo {
    #[allow(dead_code)]
    category: String,
    refs: Vec<String>,
}

/// Categorize a layer path for grouping
fn categorize_layer(path: &str) -> String {
    // Extract the base layer type for grouping
    // E.g., "mode/claude/project/foo" -> "mode/claude"
    let parts: Vec<&str> = path.split('/').collect();
    match parts.first() {
        Some(&"mode") if parts.len() >= 2 => format!("{}/{}", parts[0], parts[1]),
        Some(&"scope") if parts.len() >= 2 => format!("{}/{}", parts[0], parts[1]),
        Some(&"project") if parts.len() >= 2 => format!("{}/{}", parts[0], parts[1]),
        Some(&"global") => "global".to_string(),
        _ => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_layer() {
        assert_eq!(categorize_layer("global"), "global");
        assert_eq!(categorize_layer("mode/claude"), "mode/claude");
        assert_eq!(
            categorize_layer("mode/claude/project/dashboard"),
            "mode/claude"
        );
        assert_eq!(
            categorize_layer("scope/language:rust"),
            "scope/language:rust"
        );
        assert_eq!(categorize_layer("project/my-app"), "project/my-app");
    }
}
