//! Implementation of `jin pull`
//!
//! Fetches remote updates and merges them into local layers.
//! Requires clean workspace (no uncommitted changes).

use crate::core::{JinError, Layer, Result};
use crate::git::merge::{detect_merge_type, MergeType};
use crate::git::{JinRepo, LayerTransaction, RefOps};
use crate::staging::StagingIndex;
use std::collections::HashMap;

/// Execute the pull command
///
/// Fetches remote updates and merges them into local layers using LayerTransaction.
/// Requires clean workspace to prevent data loss.
pub fn execute() -> Result<()> {
    // 1. Verify clean workspace
    let staging = StagingIndex::load()?;
    if !staging.is_empty() {
        return Err(JinError::Config(
            "Cannot pull with uncommitted changes. Commit or reset first.\n\
            Run 'jin status' to see uncommitted changes."
                .into(),
        ));
    }

    // 2. Implicit fetch
    println!("Fetching remote updates...");
    super::fetch::execute()?;

    // 3. Open repository
    let jin_repo = JinRepo::open_or_create()?;

    // 4. Detect which layers have updates
    let updates = detect_updates(&jin_repo)?;

    if updates.is_empty() {
        println!("Already up to date");
        return Ok(());
    }

    println!("\nMerging updates...");

    // 5. Start transaction for atomic updates
    let mut tx = LayerTransaction::begin(&jin_repo, "pull: merge remote updates")?;

    // 6. Add each update to the transaction
    let mut merge_count = 0;
    for (ref_path, update_info) in &updates {
        match update_info.merge_type {
            MergeType::UpToDate => {
                // Already up to date - skip
                continue;
            }
            MergeType::FastForward => {
                // Simple fast-forward update (existing behavior)
                tx.add_layer_update(
                    update_info.layer,
                    update_info.mode.as_deref(),
                    update_info.scope.as_deref(),
                    update_info.project.as_deref(),
                    update_info.remote_oid,
                )?;
                println!("  ✓ {}: Updated (fast-forward)", format_ref_path(ref_path));
                merge_count += 1;
            }
            MergeType::LocalAhead => {
                // Local is ahead of remote - no action needed for pull
                println!(
                    "  − {}: Local is ahead of remote",
                    format_ref_path(ref_path)
                );
            }
            MergeType::Divergent => {
                // TODO: P2.M1.T2 will implement 3-way merge for divergent histories
                // For now, skip with a warning message
                println!(
                    "  ! {}: Divergent history - 3-way merge not yet implemented (see P2.M1.T2)",
                    format_ref_path(ref_path)
                );
            }
        }
    }

    // 7. Commit transaction (atomic)
    tx.commit()?;

    // Only show success message if we actually merged something
    if merge_count > 0 {
        println!("\nSuccessfully merged {} layer(s)", merge_count);
        println!("Run 'jin apply' to update workspace files");
    } else if updates.is_empty() {
        // This shouldn't happen since we return early above, but keep for safety
        println!("Already up to date");
    } else {
        // Had updates but none were fast-forward (e.g., all local ahead or divergent)
        println!("\nNo layers merged");
    }

    Ok(())
}

/// Information about a layer update
#[derive(Debug)]
struct LayerUpdateInfo {
    layer: Layer,
    mode: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    #[allow(dead_code)]
    local_oid: Option<git2::Oid>,
    remote_oid: git2::Oid,
    merge_type: MergeType,
}

/// Detect which layers have remote updates
fn detect_updates(jin_repo: &JinRepo) -> Result<HashMap<String, LayerUpdateInfo>> {
    let mut updates = HashMap::new();

    // Get all remote refs
    let remote_refs = jin_repo.list_refs("refs/jin/layers/*")?;

    for ref_path in remote_refs {
        // Skip user-local layer (never synced)
        if ref_path.contains("/local") {
            continue;
        }

        let remote_oid = jin_repo.resolve_ref(&ref_path)?;

        // Check if we have this ref locally
        let local_oid = if jin_repo.ref_exists(&ref_path) {
            Some(jin_repo.resolve_ref(&ref_path)?)
        } else {
            None
        };

        // Determine merge type
        // For new layers (no local_oid), treat as FastForward
        let merge_type = match local_oid {
            Some(local) => detect_merge_type(jin_repo, local, remote_oid)?,
            None => MergeType::FastForward,
        };

        // Determine if update is needed based on merge type
        let needs_update = match merge_type {
            MergeType::UpToDate => false,
            MergeType::FastForward | MergeType::Divergent | MergeType::LocalAhead => true,
        };

        if needs_update {
            // Parse layer information from ref path
            let (layer, mode, scope, project) = parse_ref_path(&ref_path)?;

            updates.insert(
                ref_path.clone(),
                LayerUpdateInfo {
                    layer,
                    mode,
                    scope,
                    project,
                    local_oid,
                    remote_oid,
                    merge_type,
                },
            );
        }
    }

    Ok(updates)
}

/// Parse layer information from ref path
///
/// Converts "refs/jin/layers/mode/claude" to (Layer::ModeBase, Some("claude"), None, None)
#[allow(clippy::type_complexity)]
fn parse_ref_path(
    ref_path: &str,
) -> Result<(Layer, Option<String>, Option<String>, Option<String>)> {
    let path = ref_path
        .strip_prefix("refs/jin/layers/")
        .ok_or_else(|| JinError::InvalidLayer(format!("Invalid ref path: {}", ref_path)))?;

    let parts: Vec<&str> = path.split('/').collect();

    match parts.as_slice() {
        ["global"] => Ok((Layer::GlobalBase, None, None, None)),
        ["mode", mode] => Ok((Layer::ModeBase, Some(mode.to_string()), None, None)),
        ["mode", mode, "scope", scope] => Ok((
            Layer::ModeScope,
            Some(mode.to_string()),
            Some(scope.to_string()),
            None,
        )),
        ["mode", mode, "scope", scope, "project", project] => Ok((
            Layer::ModeScopeProject,
            Some(mode.to_string()),
            Some(scope.to_string()),
            Some(project.to_string()),
        )),
        ["mode", mode, "project", project] => Ok((
            Layer::ModeProject,
            Some(mode.to_string()),
            None,
            Some(project.to_string()),
        )),
        ["scope", scope] => Ok((Layer::ScopeBase, None, Some(scope.to_string()), None)),
        ["project", project] => Ok((Layer::ProjectBase, None, None, Some(project.to_string()))),
        ["local"] => Ok((Layer::UserLocal, None, None, None)),
        ["workspace"] => Ok((Layer::WorkspaceActive, None, None, None)),
        _ => Err(JinError::InvalidLayer(format!(
            "Unrecognized layer path: {}",
            path
        ))),
    }
}

/// Format ref path for display
fn format_ref_path(ref_path: &str) -> String {
    ref_path
        .strip_prefix("refs/jin/layers/")
        .unwrap_or(ref_path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ref_path_global() {
        let (layer, mode, scope, project) = parse_ref_path("refs/jin/layers/global").unwrap();
        assert_eq!(layer, Layer::GlobalBase);
        assert!(mode.is_none());
        assert!(scope.is_none());
        assert!(project.is_none());
    }

    #[test]
    fn test_parse_ref_path_mode() {
        let (layer, mode, scope, project) = parse_ref_path("refs/jin/layers/mode/claude").unwrap();
        assert_eq!(layer, Layer::ModeBase);
        assert_eq!(mode, Some("claude".to_string()));
        assert!(scope.is_none());
        assert!(project.is_none());
    }

    #[test]
    fn test_parse_ref_path_mode_scope() {
        let (layer, mode, scope, project) =
            parse_ref_path("refs/jin/layers/mode/claude/scope/language:rust").unwrap();
        assert_eq!(layer, Layer::ModeScope);
        assert_eq!(mode, Some("claude".to_string()));
        assert_eq!(scope, Some("language:rust".to_string()));
        assert!(project.is_none());
    }

    #[test]
    fn test_parse_ref_path_mode_project() {
        let (layer, mode, scope, project) =
            parse_ref_path("refs/jin/layers/mode/claude/project/dashboard").unwrap();
        assert_eq!(layer, Layer::ModeProject);
        assert_eq!(mode, Some("claude".to_string()));
        assert!(scope.is_none());
        assert_eq!(project, Some("dashboard".to_string()));
    }

    #[test]
    fn test_parse_ref_path_invalid() {
        let result = parse_ref_path("invalid/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_ref_path() {
        assert_eq!(
            format_ref_path("refs/jin/layers/mode/claude"),
            "mode/claude"
        );
        assert_eq!(format_ref_path("refs/jin/layers/global"), "global");
    }
}
