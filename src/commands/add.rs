//! Implementation of `jin add`
//!
//! This command stages files to the appropriate layer based on flags.
//! Files are validated, their content is hashed into Git blobs, and
//! they are added to the staging index for later commit.

use crate::cli::AddArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, ObjectOps};
use crate::staging::{
    ensure_in_managed_block, get_file_mode, is_git_tracked, is_symlink, read_file, route_to_layer,
    validate_routing_options, walk_directory, RoutingOptions, StagedEntry, StagedOperation,
    StagingIndex,
};
use std::path::{Path, PathBuf};

/// Execute the add command
///
/// Stages files to the appropriate layer based on flags.
///
/// # Arguments
///
/// * `args` - Command line arguments including files and layer flags
///
/// # Errors
///
/// Returns an error if:
/// - No files are specified
/// - A file doesn't exist
/// - A file is a symlink
/// - A file is tracked by Git
/// - Routing options are invalid
/// - No active mode when --mode flag is used
pub fn execute(args: AddArgs) -> Result<()> {
    // 1. Validate we have files to stage
    if args.files.is_empty() {
        return Err(JinError::Other("No files specified".to_string()));
    }

    // 2. Load project context for active mode/scope
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // 3. Build and validate routing options
    let options = RoutingOptions {
        mode: args.mode,
        scope: args.scope.clone(),
        project: args.project,
        global: args.global,
    };
    validate_routing_options(&options)?;

    // 4. Determine target layer
    let target_layer = route_to_layer(&options, &context)?;

    // 5. Open Jin repository
    let repo = JinRepo::open_or_create()?;

    // 6. Load staging index
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 7. Process each file
    let mut staged_count = 0;
    let mut errors = Vec::new();

    for path_str in &args.files {
        let path = PathBuf::from(path_str);

        // Expand directories
        let files_to_stage = if path.is_dir() {
            match walk_directory(&path) {
                Ok(files) => files,
                Err(e) => {
                    errors.push(format!("{}: {}", path.display(), e));
                    continue;
                }
            }
        } else {
            vec![path.clone()]
        };

        for file_path in files_to_stage {
            match stage_file(&file_path, target_layer, &repo, &mut staging) {
                Ok(_) => {
                    // Add to .gitignore managed block
                    if let Err(e) = ensure_in_managed_block(&file_path) {
                        eprintln!("Warning: Could not update .gitignore: {}", e);
                    }
                    staged_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", file_path.display(), e));
                }
            }
        }
    }

    // 8. Save staging index
    staging.save()?;

    // 9. Print summary
    if staged_count > 0 {
        println!(
            "Staged {} file(s) to {} layer",
            staged_count,
            format_layer_name_with_context(target_layer, &context)
        );
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        if staged_count == 0 {
            return Err(JinError::StagingFailed {
                path: "multiple files".to_string(),
                reason: format!("{} files failed to stage", errors.len()),
            });
        }
    }

    Ok(())
}

/// Stage a single file to the staging index
fn stage_file(path: &Path, layer: Layer, repo: &JinRepo, staging: &mut StagingIndex) -> Result<()> {
    // Validate file
    validate_file(path)?;

    // Read content from workspace
    let content = read_file(path)?;

    // Create blob in Jin's bare repository
    let oid = repo.create_blob(&content)?;

    // Get file mode (executable or regular)
    let mode = get_file_mode(path);

    // Create staged entry
    let entry = StagedEntry {
        path: path.to_path_buf(),
        target_layer: layer,
        content_hash: oid.to_string(),
        mode,
        operation: StagedOperation::AddOrModify,
    };

    // Add to staging index
    staging.add(entry);

    Ok(())
}

/// Validate a file for staging
fn validate_file(path: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::NotFound(path.display().to_string()));
    }

    // Check not a directory (should have been expanded)
    if path.is_dir() {
        return Err(JinError::Other(format!(
            "{} is a directory, not a file",
            path.display()
        )));
    }

    // Check not a symlink
    if is_symlink(path)? {
        return Err(JinError::Symlink {
            path: path.display().to_string(),
        });
    }

    // Check not tracked by project's Git
    if is_git_tracked(path)? {
        return Err(JinError::GitTracked {
            path: path.display().to_string(),
        });
    }

    Ok(())
}

/// Format layer name for display, including context (mode/scope names)
fn format_layer_name_with_context(layer: Layer, context: &ProjectContext) -> String {
    match layer {
        Layer::GlobalBase => "global".to_string(),
        Layer::ModeBase => {
            if let Some(ref mode) = context.mode {
                format!("'{}' (mode)", mode)
            } else {
                "mode-base".to_string()
            }
        }
        Layer::ModeScope => {
            match (&context.mode, &context.scope) {
                (Some(mode), Some(scope)) => format!("'{}/{}' (mode/scope)", mode, scope),
                _ => "mode-scope".to_string(),
            }
        }
        Layer::ModeScopeProject => "mode-scope-project".to_string(),
        Layer::ModeProject => {
            if let Some(ref mode) = context.mode {
                format!("'{}/project' (mode/project)", mode)
            } else {
                "mode-project".to_string()
            }
        }
        Layer::ScopeBase => {
            if let Some(ref scope) = context.scope {
                format!("'{}' (scope)", scope)
            } else {
                "scope-base".to_string()
            }
        }
        Layer::ProjectBase => "project".to_string(),
        Layer::UserLocal => "user-local".to_string(),
        Layer::WorkspaceActive => "workspace-active".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn test_validate_file_not_found() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = validate_file(&path);
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    fn test_validate_file_is_directory() {
        let temp = TempDir::new().unwrap();
        let result = validate_file(temp.path());
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_validate_file_success() {
        let ctx = crate::test_utils::setup_unit_test();
        let file = ctx.project_path.join("test.json");
        std::fs::write(&file, b"{}").unwrap();

        let result = validate_file(&file);
        assert!(result.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_file_symlink() {
        use std::os::unix::fs::symlink;
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("file.txt");
        std::fs::write(&file, b"content").unwrap();
        let link = temp.path().join("link.txt");
        symlink(&file, &link).unwrap();

        let result = validate_file(&link);
        assert!(matches!(result, Err(JinError::Symlink { .. })));
    }

    #[test]
    #[serial]
    fn test_stage_file_creates_blob() {
        let ctx = crate::test_utils::setup_unit_test();
        let repo = JinRepo::open_or_create().unwrap();

        let file = ctx.project_path.join("test.json");
        std::fs::write(&file, b"{\"key\": \"value\"}").unwrap();

        let mut staging = StagingIndex::new();
        let result = stage_file(&file, Layer::ProjectBase, &repo, &mut staging);

        assert!(result.is_ok());
        assert_eq!(staging.len(), 1);
        let entry = staging.get(&file).unwrap();
        assert_eq!(entry.target_layer, Layer::ProjectBase);
        assert!(!entry.content_hash.is_empty());
    }

    #[test]
    fn test_format_layer_name_with_context() {
        let empty_context = ProjectContext::default();
        let mode_context = ProjectContext {
            mode: Some("claude".to_string()),
            ..Default::default()
        };

        assert_eq!(
            format_layer_name_with_context(Layer::GlobalBase, &empty_context),
            "global"
        );
        assert_eq!(
            format_layer_name_with_context(Layer::ModeBase, &mode_context),
            "'claude' (mode)"
        );
        assert_eq!(
            format_layer_name_with_context(Layer::ProjectBase, &empty_context),
            "project"
        );
    }

    #[test]
    fn test_execute_no_files() {
        let args = AddArgs {
            files: vec![],
            mode: false,
            scope: None,
            project: false,
            global: false,
            local: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_project_without_mode() {
        let args = AddArgs {
            files: vec!["file.txt".to_string()],
            mode: false,
            scope: None,
            project: true,
            global: false,
            local: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_global_with_mode() {
        let args = AddArgs {
            files: vec!["file.txt".to_string()],
            mode: true,
            scope: None,
            project: false,
            global: true,
            local: false,
        };
        let result = execute(args);
        assert!(result.is_err());
    }
}
