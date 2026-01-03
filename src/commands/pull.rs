//! Implementation of `jin pull`
//!
//! Fetches remote updates and merges them into local layers.
//! Requires clean workspace (no uncommitted changes).

use crate::core::{JinError, Layer, Result};
use crate::git::merge::{detect_merge_type, find_merge_base, MergeType};
use crate::git::{JinRepo, LayerTransaction, ObjectOps, RefOps, TreeOps};
use crate::merge::jinmerge::JinMergeConflict;
use crate::merge::text::{text_merge, TextMergeResult};
use crate::staging::StagingIndex;
use git2::Oid;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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
                // Perform 3-way merge for divergent histories
                match perform_three_way_merge(
                    &jin_repo,
                    update_info.layer,
                    update_info.mode.as_deref(),
                    update_info.scope.as_deref(),
                    update_info.project.as_deref(),
                    update_info.local_oid.unwrap(), // Safe because divergent means local exists
                    update_info.remote_oid,
                )? {
                    MergeOutcome::Clean(merge_oid) => {
                        tx.add_layer_update(
                            update_info.layer,
                            update_info.mode.as_deref(),
                            update_info.scope.as_deref(),
                            update_info.project.as_deref(),
                            merge_oid,
                        )?;
                        println!("  ✓ {}: Merged (3-way)", format_ref_path(ref_path));
                        merge_count += 1;
                    }
                    MergeOutcome::Conflicts {
                        merged_oid,
                        conflict_files,
                    } => {
                        tx.add_layer_update(
                            update_info.layer,
                            update_info.mode.as_deref(),
                            update_info.scope.as_deref(),
                            update_info.project.as_deref(),
                            merged_oid,
                        )?;
                        println!(
                            "  ! {}: Merged with {} conflicts",
                            format_ref_path(ref_path),
                            conflict_files.len()
                        );
                        for file in conflict_files {
                            println!(
                                "      - {} has conflicts (.jinmerge created)",
                                file.display()
                            );
                        }
                        merge_count += 1;
                    }
                }
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

/// Outcome of a 3-way merge operation
///
/// Indicates whether the merge completed cleanly or has conflicts
/// that require resolution.
#[derive(Debug)]
enum MergeOutcome {
    /// Clean merge with no conflicts
    Clean(Oid),
    /// Merge completed but has conflicts requiring resolution
    Conflicts {
        /// The merge commit OID (already created)
        merged_oid: Oid,
        /// Files that have conflicts (with .jinmerge files)
        conflict_files: Vec<PathBuf>,
    },
}

/// Perform a 3-way merge for divergent layer histories
///
/// This function implements the 3-way merge algorithm:
/// 1. Find the merge base between local and remote
/// 2. Extract file contents from base, local, and remote
/// 3. Perform 3-way text merge on each file
/// 4. Create .jinmerge files for conflicts
/// 5. Create a merge commit with two parents
///
/// # Arguments
///
/// * `jin_repo` - The Jin repository
/// * `layer` - The layer being merged
/// * `mode` - Mode name (if applicable)
/// * `scope` - Scope name (if applicable)
/// * `project` - Project name (if applicable)
/// * `local_oid` - OID of local commit
/// * `remote_oid` - OID of remote commit
///
/// # Returns
///
/// `MergeOutcome` indicating clean merge or conflicts
///
/// # Errors
///
/// Returns `JinError::Git` if Git operations fail
/// Returns `JinError::Merge` if merge operations fail
fn perform_three_way_merge(
    jin_repo: &JinRepo,
    layer: Layer,
    mode: Option<&str>,
    scope: Option<&str>,
    project: Option<&str>,
    local_oid: Oid,
    remote_oid: Oid,
) -> Result<MergeOutcome> {
    // Step 1: Find merge base
    let base_oid = find_merge_base(jin_repo, local_oid, remote_oid)?;

    // Step 2: Get commit objects for all three
    let base_commit = jin_repo.inner().find_commit(base_oid)?;
    let local_commit = jin_repo.inner().find_commit(local_oid)?;
    let remote_commit = jin_repo.inner().find_commit(remote_oid)?;

    // Step 3: Collect all unique files from all three trees
    let mut all_files = HashSet::new();
    for tree_oid in [
        base_commit.tree_id(),
        local_commit.tree_id(),
        remote_commit.tree_id(),
    ] {
        for file in jin_repo.list_tree_files(tree_oid)? {
            all_files.insert(PathBuf::from(file));
        }
    }

    // Step 4: Merge each file
    let mut merged_files = Vec::new(); // (path, blob_oid) for tree building
    let mut conflict_files = Vec::new(); // Paths with conflicts

    for file_path in all_files {
        // Extract contents from base, local, remote
        let base_content = extract_file_content(jin_repo, base_commit.tree_id(), &file_path)?;
        let local_content = extract_file_content(jin_repo, local_commit.tree_id(), &file_path)?;
        let remote_content = extract_file_content(jin_repo, remote_commit.tree_id(), &file_path)?;

        // Perform 3-way merge using existing text_merge()
        match text_merge(&base_content, &local_content, &remote_content)? {
            TextMergeResult::Clean(merged) => {
                // Create blob with merged content
                let blob_oid = jin_repo.create_blob(merged.as_bytes())?;
                merged_files.push((file_path.display().to_string(), blob_oid));
            }
            TextMergeResult::Conflict { .. } => {
                // Create .jinmerge file for this conflict
                let local_ref = layer.ref_path(mode, scope, project);
                let remote_ref = format!(
                    "origin/{}",
                    local_ref.trim_start_matches("refs/jin/layers/")
                );

                let merge_conflict = JinMergeConflict::from_text_merge(
                    file_path.clone(),
                    local_ref,
                    local_content.clone(),
                    remote_ref,
                    remote_content,
                );

                // Write .jinmerge file to workspace
                let merge_path = JinMergeConflict::merge_path_for_file(&file_path);
                merge_conflict.write_to_file(&merge_path)?;

                // For now, use local version in the merge
                // TODO: Could ask user to choose, or mark as conflicted
                let blob_oid = jin_repo.create_blob(local_content.as_bytes())?;
                merged_files.push((file_path.display().to_string(), blob_oid));
                conflict_files.push(file_path);
            }
        }
    }

    // Step 5: Create merge tree
    let merge_tree_oid = jin_repo.create_tree_from_paths(&merged_files)?;

    // Step 6: Create merge commit with two parents
    let _sig = jin_repo.inner().signature()?;
    let message = format!(
        "Merge remote changes into {}",
        layer.ref_path(mode, scope, project)
    );

    // CRITICAL: Parent order matters! Local first, then remote
    let parents: Vec<Oid> = vec![local_oid, remote_oid];
    let merge_commit_oid = jin_repo.create_commit(None, &message, merge_tree_oid, &parents)?;

    // Step 7: Return outcome
    if conflict_files.is_empty() {
        Ok(MergeOutcome::Clean(merge_commit_oid))
    } else {
        Ok(MergeOutcome::Conflicts {
            merged_oid: merge_commit_oid,
            conflict_files,
        })
    }
}

/// Extract file content from a tree, returning empty string if file not found
///
/// This helper function safely extracts file content from a tree. If the file
/// doesn't exist in the tree (e.g., it was added in one branch but not the other),
/// it returns an empty string, which is the correct behavior for 3-way merge.
///
/// # Arguments
///
/// * `repo` - The Jin repository
/// * `tree_oid` - OID of the tree to read from
/// * `path` - Path to the file within the tree
///
/// # Returns
///
/// File content as a string, or empty string if file doesn't exist
fn extract_file_content(repo: &JinRepo, tree_oid: Oid, path: &PathBuf) -> Result<String> {
    match repo.read_file_from_tree(tree_oid, path.as_path()) {
        Ok(content) => Ok(String::from_utf8_lossy(&content).to_string()),
        Err(_) => Ok(String::new()), // File doesn't exist in this tree
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
