//! Implementation of `jin apply`
//!
//! Applies merged layers to workspace with dry-run and force modes.

use crate::cli::ApplyArgs;
use crate::core::{JinError, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps};
use crate::merge::{get_applicable_layers, merge_layers, FileFormat, LayerMergeConfig};
use crate::staging::{ensure_in_managed_block, WorkspaceMetadata};
use std::path::Path;

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

    // 3. Open repository (must already exist)
    let repo = JinRepo::open()?;

    // 4. Determine applicable layers
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

    // 6. Check for conflicts
    if !merged.conflict_files.is_empty() {
        eprintln!(
            "Merge conflicts detected in {} files:",
            merged.conflict_files.len()
        );
        for path in &merged.conflict_files {
            eprintln!("  - {}", path.display());
        }
        return Err(JinError::Other(format!(
            "Cannot apply due to {} merge conflicts",
            merged.conflict_files.len()
        )));
    }

    // 7. Preview mode - show diff and exit
    if args.dry_run {
        preview_changes(&merged)?;
        return Ok(());
    }

    // 8. Apply to workspace
    apply_to_workspace(&merged, &repo)?;

    // 9. Update workspace metadata
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = config.layers.iter().map(|l| l.to_string()).collect();
    for (path, merged_file) in &merged.merged_files {
        // Get content hash by creating a blob
        let content = serialize_merged_content(&merged_file.content, merged_file.format)?;
        let oid = repo.create_blob(content.as_bytes())?;
        metadata.add_file(path.clone(), oid.to_string());
    }
    metadata.save()?;

    // 10. Update .gitignore managed block
    for path in merged.merged_files.keys() {
        if let Err(e) = ensure_in_managed_block(path) {
            eprintln!("Warning: Could not update .gitignore: {}", e);
        }
    }

    // 11. Report results
    println!("Applied {} files to workspace", merged.merged_files.len());
    if !merged.added_files.is_empty() {
        println!("  Added: {}", merged.added_files.len());
    }
    if !merged.removed_files.is_empty() {
        println!("  Removed: {}", merged.removed_files.len());
    }

    Ok(())
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
    println!("Would apply {} files:", merged.merged_files.len());

    // Show added files (files in merged result but not in workspace)
    let mut added = Vec::new();
    let mut modified = Vec::new();

    for (path, merged_file) in &merged.merged_files {
        if path.exists() {
            // File exists, check if it would be modified
            let workspace_content = std::fs::read_to_string(path)?;
            let merged_content =
                serialize_merged_content(&merged_file.content, merged_file.format)?;

            if workspace_content != merged_content {
                modified.push(path);
            }
        } else {
            // File doesn't exist, would be added
            added.push(path);
        }
    }

    if !added.is_empty() {
        println!("\nAdded files:");
        for path in added {
            println!("  + {}", path.display());
        }
    }

    if !modified.is_empty() {
        println!("\nModified files:");
        for path in modified {
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
