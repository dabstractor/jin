//! Implementation of `jin resolve`
//!
//! Resolves merge conflicts by validating user-edited .jinmerge files
//! and completing the paused apply operation.

use crate::cli::ResolveArgs;
use crate::commands::apply::PausedApplyState;
use crate::core::{JinError, Result};
use crate::git::{JinRepo, ObjectOps};
use crate::merge::jinmerge::{JinMergeConflict, JINMERGE_HEADER};
use crate::staging::{ensure_in_managed_block, WorkspaceMetadata};
use chrono::{Duration, Utc};
use std::path::{Path, PathBuf};

/// Execute the resolve command
///
/// Validates .jinmerge files have been manually resolved and applies them.
///
/// # Arguments
///
/// * `args` - Command line arguments including files, all, force, and dry_run flags
///
/// # Errors
///
/// Returns an error if:
/// - No paused apply operation exists
/// - Paused state is stale and --force not used
/// - Specified files are not in conflict state
/// - .jinmerge files still contain conflict markers
/// - Files cannot be written
pub fn execute(args: ResolveArgs) -> Result<()> {
    // 1. Check for paused state
    if !PausedApplyState::exists() {
        return Err(JinError::Other(
            "No paused apply operation found. Run 'jin apply' first.".to_string(),
        ));
    }

    // 2. Load and validate paused state
    let state = PausedApplyState::load()?;

    // 3. Validate state is not stale (optional timeout check)
    let max_age = Duration::hours(24);
    if Utc::now() - state.timestamp > max_age {
        eprintln!("Warning: Paused operation is over 24 hours old.");
        if !args.force {
            return Err(JinError::Other(
                "Stale paused state. Use --force to proceed.".to_string(),
            ));
        }
    }

    // 4. Determine files to resolve
    let files_to_resolve = if args.files.is_empty() || args.all {
        // Resolve all conflicts
        state.conflict_files.clone()
    } else {
        // Validate specified files are in conflict list
        for file in &args.files {
            let path = PathBuf::from(file);
            if !state.conflict_files.contains(&path) {
                return Err(JinError::Other(format!(
                    "File '{}' is not in conflict state. Use 'jin status' for details.",
                    file
                )));
            }
        }
        args.files.iter().map(PathBuf::from).collect()
    };

    // 5. Dry-run mode
    if args.dry_run {
        println!("Would resolve {} files:", files_to_resolve.len());
        for file in &files_to_resolve {
            println!("  - {}", file.display());
        }
        return Ok(());
    }

    // 6. Resolve each file
    let mut resolved_count = 0;
    let mut errors = Vec::new();

    for conflict_path in files_to_resolve {
        match resolve_single_file(&conflict_path, &state) {
            Ok(_) => resolved_count += 1,
            Err(e) => errors.push(format!("{}: {}", conflict_path.display(), e)),
        }
    }

    // 7. Report results
    if resolved_count > 0 {
        println!("Resolved {} file(s)", resolved_count);
    }

    if !errors.is_empty() {
        eprintln!("Errors resolving {} file(s):", errors.len());
        for error in &errors {
            eprintln!("  - {}", error);
        }
        if resolved_count == 0 {
            return Err(JinError::Other("Failed to resolve any files".to_string()));
        }
    }

    // 8. Check if all conflicts resolved
    let remaining_conflicts = state.conflict_files.len() - resolved_count;
    if remaining_conflicts == 0 {
        // Complete the apply operation automatically
        complete_apply_operation(&state)?;
        println!("All conflicts resolved. Apply operation completed.");
    } else {
        println!("Remaining conflicts: {}", remaining_conflicts);
        println!("Use 'jin resolve --all' to resolve remaining conflicts.");
    }

    Ok(())
}

/// Resolve a single conflicted file
///
/// # Arguments
///
/// * `conflict_path` - Original file path (without .jinmerge extension)
/// * `_state` - Paused apply state (currently unused but kept for future use)
fn resolve_single_file(conflict_path: &PathBuf, _state: &PausedApplyState) -> Result<()> {
    // 1. Locate .jinmerge file
    let merge_path = JinMergeConflict::merge_path_for_file(conflict_path);
    if !merge_path.exists() {
        return Err(JinError::Other(format!(
            "No .jinmerge file found for {}. Did you delete it?",
            conflict_path.display()
        )));
    }

    // 2. Parse .jinmerge file
    let _merge_conflict = JinMergeConflict::parse_from_file(&merge_path)?;

    // 3. Validate no conflict markers remain
    validate_no_conflict_markers(&merge_path)?;

    // 4. Read resolved content from .jinmerge file
    let resolved_content = std::fs::read_to_string(&merge_path).map_err(JinError::Io)?;

    // 5. Write resolved content to workspace file (atomic)
    apply_resolved_file(conflict_path, &resolved_content)?;

    // 6. Delete .jinmerge file
    std::fs::remove_file(&merge_path)
        .map_err(|e| JinError::Other(format!("Failed to delete .jinmerge file: {}", e)))?;

    // 7. Update state (remove from conflict_files)
    update_paused_state(conflict_path)?;

    Ok(())
}

/// Validate that .jinmerge file has no conflict markers
///
/// # Arguments
///
/// * `merge_path` - Path to the .jinmerge file
fn validate_no_conflict_markers(merge_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(merge_path).map_err(JinError::Io)?;

    // Check for conflict markers
    if content.contains("<<<<<<<") || content.contains("=======") || content.contains(">>>>>>>") {
        return Err(JinError::Other(
            "Conflict markers still present. Please resolve all conflicts before running 'jin resolve'.".to_string()
        ));
    }

    // Check for header (must be present in valid .jinmerge files)
    // After resolution, header may or may not be present, so we don't enforce this
    // But we do want to verify the file isn't empty
    let trimmed = content.trim();
    if trimmed.is_empty()
        || trimmed == JINMERGE_HEADER
        || (trimmed.starts_with("# Jin merge conflict") && trimmed.lines().count() == 1)
    {
        return Err(JinError::Other(
            "Empty resolution. Please keep the desired content in the file.".to_string(),
        ));
    }

    Ok(())
}

/// Apply resolved file to workspace atomically
///
/// # Arguments
///
/// * `file_path` - Original file path (without .jinmerge extension)
/// * `content` - Resolved content to write
fn apply_resolved_file(file_path: &PathBuf, content: &str) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).map_err(JinError::Io)?;
    }

    // Atomic write pattern (follow apply.rs:349-354)
    let temp_path = file_path.with_extension("jin-tmp");
    std::fs::write(&temp_path, content).map_err(JinError::Io)?;

    std::fs::rename(&temp_path, file_path).map_err(JinError::Io)?;

    Ok(())
}

/// Update paused state after resolving a file
///
/// # Arguments
///
/// * `resolved_path` - Path to the file that was resolved
fn update_paused_state(resolved_path: &PathBuf) -> Result<()> {
    let mut state = PausedApplyState::load()?;

    // Remove resolved file from conflict list
    state.conflict_files.retain(|p| p != resolved_path);
    state.conflict_count = state.conflict_files.len();

    // If no more conflicts, delete state file
    if state.conflict_files.is_empty() {
        let state_path = PathBuf::from(".jin/.paused_apply.yaml");
        std::fs::remove_file(&state_path)
            .map_err(|e| JinError::Other(format!("Failed to remove paused state: {}", e)))?;
    } else {
        // Save updated state
        state.save()?;
    }

    Ok(())
}

/// Complete the apply operation when all conflicts are resolved
///
/// # Arguments
///
/// * `state` - The original paused apply state
fn complete_apply_operation(state: &PausedApplyState) -> Result<()> {
    // 1. Update workspace metadata
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = state.layer_config.layers.clone();

    let repo = JinRepo::open()?;

    // Add all resolved conflict files to metadata
    for conflict_path in &state.conflict_files {
        let content = std::fs::read_to_string(conflict_path)?;
        let oid = repo.create_blob(content.as_bytes())?;
        metadata.add_file(conflict_path.clone(), oid.to_string());
    }

    // Also include previously applied files
    for applied_path in &state.applied_files {
        // These were already tracked, but we need to update metadata
        if applied_path.exists() {
            let content = std::fs::read_to_string(applied_path)?;
            let oid = repo.create_blob(content.as_bytes())?;
            metadata.add_file(applied_path.clone(), oid.to_string());
        }
    }

    metadata.save()?;

    // 2. Update .gitignore for all applied files
    let all_files = state
        .applied_files
        .iter()
        .chain(state.conflict_files.iter());
    for path in all_files {
        if let Err(e) = ensure_in_managed_block(path) {
            eprintln!("Warning: Could not update .gitignore: {}", e);
        }
    }

    // 3. Delete paused state file (should already be deleted by update_paused_state)
    let state_path = PathBuf::from(".jin/.paused_apply.yaml");
    if state_path.exists() {
        std::fs::remove_file(&state_path)
            .map_err(|e| JinError::Other(format!("Failed to remove paused state: {}", e)))?;
    }

    // 4. Report completion
    println!("Apply operation completed successfully.");
    println!(
        "Applied {} total files.",
        state.applied_files.len() + state.conflict_files.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_no_conflict_markers_with_markers() {
        let temp = tempfile::TempDir::new().unwrap();
        let merge_path = temp.path().join("test.jinmerge");

        std::fs::write(
            &merge_path,
            "# Jin merge conflict\n<<<<<<< layer1/\ncontent\n=======\ncontent2\n>>>>>>> layer2/\n",
        )
        .unwrap();

        let result = validate_no_conflict_markers(&merge_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_conflict_markers_resolved() {
        let temp = tempfile::TempDir::new().unwrap();
        let merge_path = temp.path().join("test.jinmerge");

        std::fs::write(&merge_path, "# Jin merge conflict\nresolved content\n").unwrap();

        let result = validate_no_conflict_markers(&merge_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_no_conflict_markers_empty() {
        let temp = tempfile::TempDir::new().unwrap();
        let merge_path = temp.path().join("test.jinmerge");

        std::fs::write(&merge_path, "# Jin merge conflict\n").unwrap();

        let result = validate_no_conflict_markers(&merge_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_resolved_file_creates_parent_dir() {
        let temp = tempfile::TempDir::new().unwrap();
        let file_path = temp.path().join("subdir").join("config.json");

        apply_resolved_file(&file_path, "{\"test\": true}").unwrap();

        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "{\"test\": true}");
    }
}
