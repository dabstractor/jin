//! Implementation of `jin apply`
//!
//! Applies merged layers to workspace with dry-run and force modes.

use crate::cli::ApplyArgs;
use crate::core::{JinError, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps, RefOps, TreeOps};
use crate::merge::jinmerge::JinMergeConflict;
use crate::merge::{get_applicable_layers, merge_layers, FileFormat, LayerMergeConfig};
use crate::staging::{ensure_in_managed_block, validate_workspace_attached, WorkspaceMetadata};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// State for a paused apply operation due to conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedApplyState {
    /// When the operation was paused
    pub timestamp: DateTime<Utc>,
    /// Layer configuration used for the merge attempt
    pub layer_config: PausedLayerConfig,
    /// Files with conflicts (original paths, not .jinmerge paths)
    pub conflict_files: Vec<PathBuf>,
    /// Files that were successfully applied
    pub applied_files: Vec<PathBuf>,
    /// Number of conflicts total
    pub conflict_count: usize,
}

/// Simplified layer config for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedLayerConfig {
    /// Layer names as strings
    pub layers: Vec<String>,
    /// Active mode, if any
    pub mode: Option<String>,
    /// Active scope, if any
    pub scope: Option<String>,
    /// Project name
    pub project: Option<String>,
}

impl PausedApplyState {
    /// Save state to `.jin/.paused_apply.yaml`
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(".jin/.paused_apply.yaml");
        let content = serde_yaml::to_string(self)
            .map_err(|e| JinError::Other(format!("Failed to serialize paused state: {}", e)))?;

        // Atomic write pattern
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content).map_err(JinError::Io)?;
        std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;

        Ok(())
    }

    /// Check if a paused operation exists
    pub fn exists() -> bool {
        PathBuf::from(".jin/.paused_apply.yaml").exists()
    }

    /// Load state from `.jin/.paused_apply.yaml`
    pub fn load() -> Result<Self> {
        let path = PathBuf::from(".jin/.paused_apply.yaml");

        if !path.exists() {
            return Err(JinError::Other(
                "No paused apply operation found".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&path).map_err(JinError::Io)?;

        serde_yaml::from_str(&content)
            .map_err(|e| JinError::Other(format!("Invalid paused state: {}", e)))
    }
}

/// Execute the apply command
///
/// Applies merged layers to workspace.
///
/// # Arguments
///
/// * `args` - Command line arguments including force and dry_run flags
///
/// # Errors
///
/// Returns an error if:
/// - Jin is not initialized
/// - Workspace is dirty without --force
/// - Merge conflicts are detected
/// - Files cannot be written
pub fn execute(args: ApplyArgs) -> Result<()> {
    // 1. Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // 2. Check workspace dirty (unless --force)
    if !args.force && check_workspace_dirty()? {
        return Err(JinError::Other(
            "Workspace has uncommitted changes. Use --force to override.".to_string(),
        ));
    }

    // 2.5. Validate workspace state before destructive apply (only with --force)
    let repo = if args.force {
        let r = JinRepo::open()?;
        validate_workspace_attached(&context, &r)?;
        r
    } else {
        JinRepo::open()?
    };

    // 3. Determine applicable layers
    let layers = get_applicable_layers(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );

    // 5. Merge layers based on active context
    let config = LayerMergeConfig {
        layers,
        mode: context.mode.clone(),
        scope: context.scope.clone(),
        project: context.project.clone(),
    };
    let merged = merge_layers(&config, &repo)?;

    // 6. Check for conflicts and prepare paused state if needed
    let has_conflicts = !merged.conflict_files.is_empty();

    if has_conflicts {
        println!(
            "Merge conflicts detected in {} files:",
            merged.conflict_files.len()
        );
        for path in &merged.conflict_files {
            println!("  - {}", path.display());
        }
    }

    // 7. Preview mode - show diff and exit
    if args.dry_run {
        if has_conflicts {
            println!();
            println!("Use --force to apply non-conflicting files, or resolve conflicts first.");
        }
        preview_changes(&merged)?;
        return Ok(());
    }

    // 8. Apply to workspace (non-conflicting files only)
    apply_to_workspace(&merged, &repo)?;

    // 9. Handle conflicts if any
    if has_conflicts {
        // Handle conflicts: generate .jinmerge files and save state
        let paused_state = handle_conflicts(&merged.conflict_files, &config, &merged.merged_files)?;

        println!();
        println!("Created .jinmerge files for manual resolution:");
        for conflict_path in &merged.conflict_files {
            let merge_path = JinMergeConflict::merge_path_for_file(conflict_path);
            println!("  - {}", merge_path.display());
        }

        // Save paused state
        paused_state.save()?;

        println!();
        println!("Operation paused. Resolve conflicts with:");
        println!("  jin resolve <file>");
        println!();
        println!("For more information, run: jin status");

        return Ok(());
    }

    // 10. Update workspace metadata (only if no conflicts)
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = config.layers.iter().map(|l| l.to_string()).collect();
    for (path, merged_file) in &merged.merged_files {
        // Get content hash by creating a blob
        let content = serialize_merged_content(&merged_file.content, merged_file.format)?;
        let oid = repo.create_blob(content.as_bytes())?;
        metadata.add_file(path.clone(), oid.to_string());
    }
    metadata.save()?;

    // 11. Update .gitignore managed block
    for path in merged.merged_files.keys() {
        if let Err(e) = ensure_in_managed_block(path) {
            eprintln!("Warning: Could not update .gitignore: {}", e);
        }
    }

    // 12. Report results
    println!("Applied {} files to workspace", merged.merged_files.len());
    if !merged.added_files.is_empty() {
        println!("  Added: {}", merged.added_files.len());
    }
    if !merged.removed_files.is_empty() {
        println!("  Removed: {}", merged.removed_files.len());
    }

    Ok(())
}

/// Handle merge conflicts by generating .jinmerge files and creating paused state
///
/// # Arguments
///
/// * `conflict_files` - List of files that have conflicts
/// * `config` - Layer merge configuration
/// * `merged_files` - Successfully merged files (for tracking in state)
///
/// # Returns
///
/// PausedApplyState with conflict information
fn handle_conflicts(
    conflict_files: &[PathBuf],
    config: &LayerMergeConfig,
    merged_files: &HashMap<PathBuf, crate::merge::MergedFile>,
) -> Result<PausedApplyState> {
    // Collect successfully applied files
    let applied_files: Vec<PathBuf> = merged_files.keys().cloned().collect();

    for conflict_path in conflict_files {
        // Get the two conflicting layer contents
        let (layer1_ref, layer1_content, layer2_ref, layer2_content) =
            get_conflicting_layer_contents(conflict_path, config)?;

        // Create .jinmerge file
        let merge_conflict = JinMergeConflict::from_text_merge(
            conflict_path.clone(),
            layer1_ref,
            layer1_content,
            layer2_ref,
            layer2_content,
        );

        let merge_path = JinMergeConflict::merge_path_for_file(conflict_path);
        merge_conflict.write_to_file(&merge_path)?;
    }

    // Create paused state
    Ok(PausedApplyState {
        timestamp: Utc::now(),
        layer_config: PausedLayerConfig {
            layers: config.layers.iter().map(|l| l.to_string()).collect(),
            mode: config.mode.clone(),
            scope: config.scope.clone(),
            project: config.project.clone(),
        },
        conflict_files: conflict_files.to_vec(),
        applied_files,
        conflict_count: conflict_files.len(),
    })
}

/// Get content from the two conflicting layers for a file
///
/// Iterates layers in REVERSE (highest precedence first) to find the first
/// TWO layers that contain the conflicting file.
///
/// # Returns
///
/// (layer1_ref, layer1_content, layer2_ref, layer2_content)
/// where layer1 is lower precedence (ours) and layer2 is higher (theirs)
fn get_conflicting_layer_contents(
    file_path: &Path,
    config: &LayerMergeConfig,
) -> Result<(String, String, String, String)> {
    let repo = JinRepo::open()?;
    let mut layer_refs = Vec::new();

    // Iterate layers in REVERSE (highest precedence first)
    for layer in config.layers.iter().rev() {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        if repo.ref_exists(&ref_path) {
            if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
                let commit = repo.inner().find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                if let Ok(content) = repo.read_file_from_tree(tree_oid, file_path) {
                    let content_str = String::from_utf8_lossy(&content).to_string();

                    // Create short layer label (strip "refs/jin/layers/" prefix)
                    let short_label = ref_path
                        .strip_prefix("refs/jin/layers/")
                        .unwrap_or(&ref_path)
                        .to_string();

                    layer_refs.push((short_label, content_str));

                    if layer_refs.len() >= 2 {
                        break; // Got the two conflicting layers
                    }
                }
            }
        }
    }

    if layer_refs.len() < 2 {
        return Err(JinError::Other(format!(
            "Could not find two layers for conflict: {}",
            file_path.display()
        )));
    }

    // layer_refs[0] is higher precedence (theirs)
    // layer_refs[1] is lower precedence (ours)
    Ok((
        layer_refs[1].0.clone(), // layer1_ref (ours)
        layer_refs[1].1.clone(), // layer1_content
        layer_refs[0].0.clone(), // layer2_ref (theirs)
        layer_refs[0].1.clone(), // layer2_content
    ))
}

/// Apply merged files to workspace
fn apply_to_workspace(merged: &crate::merge::LayerMergeResult, _repo: &JinRepo) -> Result<()> {
    let mut applied_count = 0;
    let mut errors = Vec::new();

    // Process each merged file
    for (path, merged_file) in &merged.merged_files {
        match apply_file(path, merged_file) {
            Ok(_) => applied_count += 1,
            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
        }
    }

    // Report errors
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if applied_count == 0 {
            return Err(JinError::Other("Failed to apply any files".to_string()));
        }
    }

    Ok(())
}

/// Apply a single file to workspace with atomic write
fn apply_file(path: &Path, merged_file: &crate::merge::MergedFile) -> Result<()> {
    // Serialize content based on format
    let content = serialize_merged_content(&merged_file.content, merged_file.format)?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Atomic write pattern: write to temp file, then rename
    let temp_path = path.with_extension("jin-tmp");
    std::fs::write(&temp_path, &content)?;

    // Atomic rename
    std::fs::rename(&temp_path, path)?;

    // Set file mode (Unix only)
    #[cfg(unix)]
    {
        // File mode is determined by content, not stored in merge
        // Default to regular file mode
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o100644);
        std::fs::set_permissions(path, perms)?;
    }

    Ok(())
}

/// Serialize merged content based on file format
fn serialize_merged_content(
    content: &crate::merge::MergeValue,
    format: FileFormat,
) -> Result<String> {
    match format {
        FileFormat::Json => content.to_json_string(),
        FileFormat::Yaml => content.to_yaml_string(),
        FileFormat::Toml => content.to_toml_string(),
        FileFormat::Ini => content.to_ini_string(),
        FileFormat::Text => {
            // For text files, MergeValue should be a String variant
            if let Some(text) = content.as_str() {
                Ok(text.to_string())
            } else {
                Err(JinError::Other(
                    "Text file has non-string content".to_string(),
                ))
            }
        }
    }
}

/// Preview changes that would be applied
fn preview_changes(merged: &crate::merge::LayerMergeResult) -> Result<()> {
    eprintln!(
        "[DEBUG] preview_changes: merged_files.len() = {}",
        merged.merged_files.len()
    );
    println!("Would apply {} files:", merged.merged_files.len());

    // Show added files (files in merged result but not in workspace)
    let mut added = Vec::new();
    let mut modified = Vec::new();

    for (path, merged_file) in &merged.merged_files {
        eprintln!("[DEBUG] preview_changes: Checking path: {}", path.display());
        eprintln!("[DEBUG] preview_changes: path.exists() = {}", path.exists());
        if path.exists() {
            // File exists, check if it would be modified
            let workspace_content = std::fs::read_to_string(path)?;
            let merged_content =
                serialize_merged_content(&merged_file.content, merged_file.format)?;

            let content_differs = workspace_content != merged_content;
            eprintln!(
                "[DEBUG] preview_changes: File content differs: {}",
                content_differs
            );

            if content_differs {
                modified.push(path);
            }
        } else {
            // File doesn't exist, would be added
            added.push(path);
        }
    }

    if !added.is_empty() {
        println!("\nAdded files:");
        for path in &added {
            println!("  + {}", path.display());
        }
    }

    if !modified.is_empty() {
        println!("\nModified files:");
        for path in &modified {
            println!("  M {}", path.display());
        }
    }

    // Show removed files
    if !merged.removed_files.is_empty() {
        println!("\nRemoved files:");
        for path in &merged.removed_files {
            println!("  - {}", path.display());
        }
    }

    eprintln!(
        "[DEBUG] preview_changes: Added: {}, Modified: {}",
        added.len(),
        modified.len()
    );
    Ok(())
}

/// Check if workspace has uncommitted changes
fn check_workspace_dirty() -> Result<bool> {
    // Check if workspace has uncommitted changes by comparing
    // current workspace files to last applied configuration

    let metadata = match WorkspaceMetadata::load() {
        Ok(m) => m,
        Err(_) => return Ok(false), // No metadata = clean
    };

    // Check if any tracked files have changed
    for (path, expected_hash) in &metadata.files {
        // File deleted
        if !path.exists() {
            return Ok(true);
        }

        // File modified - compare hash
        let content = std::fs::read(path)?;
        let repo = JinRepo::open()?;
        let current_hash = repo.create_blob(&content)?;
        if current_hash.to_string() != *expected_hash {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let args = ApplyArgs {
            force: false,
            dry_run: false,
        };
        let result = execute(args);
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_check_workspace_dirty_no_metadata() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let result = check_workspace_dirty().unwrap();
        assert!(!result); // No metadata = clean
    }

    #[test]
    fn test_serialize_merged_content_json() {
        use crate::merge::MergeValue;

        let value = MergeValue::from_json(r#"{"key": "value"}"#).unwrap();
        let result = serialize_merged_content(&value, FileFormat::Json);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[test]
    fn test_serialize_merged_content_text() {
        use crate::merge::MergeValue;

        let value = MergeValue::String("Hello, World!".to_string());
        let result = serialize_merged_content(&value, FileFormat::Text);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }
}
