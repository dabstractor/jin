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
        return Ok(());
    }

    // Split into active and other updates based on context
    let mut active_updates: HashMap<String, UpdateInfo> = HashMap::new();
    let mut other_updates: HashMap<String, UpdateInfo> = HashMap::new();

    for (category, info) in updates {
        // Check if any ref in this category is relevant to active context
        let is_relevant = info
            .refs
            .iter()
            .any(|ref_path| is_ref_relevant_to_context(ref_path, context));

        if is_relevant {
            active_updates.insert(category, info);
        } else {
            other_updates.insert(category, info);
        }
    }

    // Build section title with context info
    let active_title = if let (Some(mode), Some(scope)) = (&context.mode, &context.scope) {
        format!(
            "Updates for your active context (mode: {}, scope: {}):",
            mode, scope
        )
    } else if let Some(mode) = &context.mode {
        format!("Updates for your active context (mode: {}):", mode)
    } else if let Some(scope) = &context.scope {
        format!("Updates for your active context (scope: {}):", scope)
    } else {
        "Updates for your active context:".to_string()
    };

    // Display active updates section
    format_update_section(&active_title, &active_updates);

    // Display other updates section
    if !other_updates.is_empty() {
        format_update_section("Other updates:", &other_updates);
    }

    // Show next steps if any updates exist
    if !active_updates.is_empty() || !other_updates.is_empty() {
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

/// Check if a ref path is relevant to the active context
///
/// A ref is relevant if:
/// - It matches the active mode (e.g., "mode/claude" when mode is "claude")
/// - It matches the active scope with mode (e.g., "mode/claude/scope/js" when mode="claude", scope="js")
/// - It matches the active scope without mode (e.g., "scope/js" when mode=None, scope="js")
/// - Global refs are always relevant
fn is_ref_relevant_to_context(ref_path: &str, context: &ProjectContext) -> bool {
    // Strip prefix to get layer path
    let layer_path = match ref_path.strip_prefix("refs/jin/layers/") {
        Some(path) => path,
        None => return false,
    };

    // Global is always relevant
    if layer_path == "global" {
        return true;
    }

    // Parse the path components
    let parts: Vec<&str> = layer_path.split('/').collect();

    match parts.as_slice() {
        // Mode-scope refs: Check if matches both active mode and scope
        ["mode", mode, "scope", scope, ..] => {
            context.mode.as_deref() == Some(*mode) && context.scope.as_deref() == Some(*scope)
        }

        // Mode refs: Check if matches active mode
        ["mode", mode, ..] => context.mode.as_deref() == Some(*mode),

        // Untethered scope refs: Only relevant if no active mode
        ["scope", scope, ..] => context.mode.is_none() && context.scope.as_deref() == Some(*scope),

        // Project refs: Not relevant to mode/scope context
        ["project", ..] => false,

        // Other patterns: Not relevant to context
        _ => false,
    }
}

/// Format and display a section of updates with header
///
/// # Arguments
/// * `title` - Section header to display
/// * `updates` - HashMap of updates to display
fn format_update_section(title: &str, updates: &HashMap<String, UpdateInfo>) {
    if updates.is_empty() {
        // For active context section, show "no updates" message
        if title.contains("active context") {
            println!("{}", title);
        }
        return;
    }

    // Print section header
    println!();
    println!("{}", title);

    // Sort and display updates
    let mut categories: Vec<_> = updates.keys().collect();
    categories.sort();

    for category in categories {
        let info = &updates[category];
        println!("  - {} ({} file(s))", info.refs[0], info.refs.len());
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
