//! Implementation of `jin reset`
//!
//! Resets staged or committed changes with --soft, --mixed, and --hard modes.

use crate::cli::ResetArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::staging::{remove_from_managed_block, StagedEntry, StagingIndex};
use std::io::{self, Write};

/// Reset mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResetMode {
    /// Keep changes in staging
    Soft,
    /// Unstage but keep in workspace (default)
    Mixed,
    /// Discard all changes (DESTRUCTIVE)
    Hard,
}

/// Execute the reset command
///
/// Resets staged or committed changes.
///
/// # Arguments
///
/// * `args` - Command line arguments including reset mode and layer flags
///
/// # Errors
///
/// Returns an error if:
/// - Jin is not initialized
/// - Invalid layer combination
/// - No active mode/scope when flags require them
pub fn execute(args: ResetArgs) -> Result<()> {
    // 1. Determine reset mode
    let mode = if args.soft {
        ResetMode::Soft
    } else if args.hard {
        ResetMode::Hard
    } else {
        ResetMode::Mixed // Default
    };

    // 2. Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. Determine target layer
    let layer = determine_target_layer(&args, &context)?;

    // 4. Load staging
    let mut staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());

    // 5. Get affected entries
    let entries: Vec<&StagedEntry> = staging.entries_for_layer(layer);
    if entries.is_empty() {
        println!("Nothing to reset for layer: {}", layer_name(layer));
        return Ok(());
    }

    // 6. Confirmation for --hard mode
    if mode == ResetMode::Hard {
        let count = entries.len();
        let message = format!(
            "This will discard {} file(s) from staging AND workspace. Type 'yes' to confirm:",
            count
        );
        if !prompt_confirmation(&message)? {
            println!("Reset cancelled");
            return Ok(());
        }
    }

    // 7. Perform reset based on mode
    match mode {
        ResetMode::Soft => {
            // Keep in staging, just acknowledge (no-op for now)
            println!("Reset {} file(s) (kept in staging)", entries.len());
        }
        ResetMode::Mixed => {
            // Remove from staging, keep in workspace
            let count = entries.len();
            reset_staging(&mut staging, layer)?;
            staging.save()?;
            println!("Unstaged {} file(s) (kept in workspace)", count);
        }
        ResetMode::Hard => {
            // Remove from staging AND workspace
            let count = entries.len();

            // Clone entries before modifying staging to avoid borrow issues
            let entries_to_reset: Vec<StagedEntry> = entries.iter().map(|e| (*e).clone()).collect();

            reset_staging(&mut staging, layer)?;
            reset_workspace(&entries_to_reset)?;
            staging.save()?;
            println!("Discarded {} file(s) from staging and workspace", count);
        }
    }

    Ok(())
}

/// Determine target layer from reset arguments and context
fn determine_target_layer(args: &ResetArgs, context: &ProjectContext) -> Result<Layer> {
    // --mode + --scope=X + --project → Layer 4 (ModeScopeProject)
    if args.mode && args.scope.is_some() && args.project {
        context.require_mode()?;
        return Ok(Layer::ModeScopeProject);
    }

    // --mode + --scope=X → Layer 3 (ModeScope)
    if args.mode && args.scope.is_some() {
        context.require_mode()?;
        return Ok(Layer::ModeScope);
    }

    // --mode + --project → Layer 5 (ModeProject)
    if args.mode && args.project {
        context.require_mode()?;
        return Ok(Layer::ModeProject);
    }

    // --mode → Layer 2 (ModeBase)
    if args.mode {
        context.require_mode()?;
        return Ok(Layer::ModeBase);
    }

    // --scope=X → Layer 6 (ScopeBase)
    if args.scope.is_some() {
        return Ok(Layer::ScopeBase);
    }

    // --project → Error (requires --mode)
    if args.project {
        return Err(JinError::Other(
            "--project requires --mode (use --mode --project)".to_string(),
        ));
    }

    // Default: Layer 7 (ProjectBase)
    Ok(Layer::ProjectBase)
}

/// Reset staging index for a specific layer
fn reset_staging(staging: &mut StagingIndex, layer: Layer) -> Result<()> {
    let paths_to_remove: Vec<_> = staging
        .entries_for_layer(layer)
        .iter()
        .map(|e| e.path.clone())
        .collect();

    for path in paths_to_remove {
        staging.remove(&path);
    }

    Ok(())
}

/// Reset workspace files (delete them)
fn reset_workspace(entries: &[StagedEntry]) -> Result<()> {
    let mut errors = Vec::new();

    for entry in entries {
        // Remove from workspace
        if entry.path.exists() {
            if let Err(e) = std::fs::remove_file(&entry.path) {
                errors.push(format!("{}: {}", entry.path.display(), e));
            }
        }

        // Remove from .gitignore managed block
        if let Err(e) = remove_from_managed_block(&entry.path) {
            errors.push(format!(
                "{}: Failed to update .gitignore: {}",
                entry.path.display(),
                e
            ));
        }
    }

    if !errors.is_empty() {
        eprintln!("Errors during workspace reset:");
        for error in &errors {
            eprintln!("  {}", error);
        }
    }

    Ok(())
}

/// Prompt user for confirmation
fn prompt_confirmation(message: &str) -> Result<bool> {
    print!("{} ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("yes"))
}

/// Get human-readable layer name
fn layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::GlobalBase => "global-base",
        Layer::ModeBase => "mode-base",
        Layer::ModeScope => "mode-scope",
        Layer::ModeScopeProject => "mode-scope-project",
        Layer::ModeProject => "mode-project",
        Layer::ScopeBase => "scope-base",
        Layer::ProjectBase => "project-base",
        Layer::UserLocal => "user-local",
        Layer::WorkspaceActive => "workspace-active",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_execute_not_initialized() {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        let args = ResetArgs {
            soft: false,
            mixed: false,
            hard: false,
            mode: false,
            scope: None,
            project: false,
        };
        let result = execute(args);
        assert!(matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_determine_target_layer_default() {
        let context = ProjectContext::default();
        let args = ResetArgs {
            soft: false,
            mixed: false,
            hard: false,
            mode: false,
            scope: None,
            project: false,
        };
        let result = determine_target_layer(&args, &context).unwrap();
        assert_eq!(result, Layer::ProjectBase);
    }

    #[test]
    fn test_determine_target_layer_mode() {
        let mut context = ProjectContext::default();
        context.mode = Some("claude".to_string());

        let args = ResetArgs {
            soft: false,
            mixed: false,
            hard: false,
            mode: true,
            scope: None,
            project: false,
        };
        let result = determine_target_layer(&args, &context).unwrap();
        assert_eq!(result, Layer::ModeBase);
    }

    #[test]
    fn test_determine_target_layer_mode_scope() {
        let mut context = ProjectContext::default();
        context.mode = Some("claude".to_string());

        let args = ResetArgs {
            soft: false,
            mixed: false,
            hard: false,
            mode: true,
            scope: Some("lang:rust".to_string()),
            project: false,
        };
        let result = determine_target_layer(&args, &context).unwrap();
        assert_eq!(result, Layer::ModeScope);
    }

    #[test]
    fn test_determine_target_layer_mode_project() {
        let mut context = ProjectContext::default();
        context.mode = Some("claude".to_string());

        let args = ResetArgs {
            soft: false,
            mixed: false,
            hard: false,
            mode: true,
            scope: None,
            project: true,
        };
        let result = determine_target_layer(&args, &context).unwrap();
        assert_eq!(result, Layer::ModeProject);
    }

    #[test]
    fn test_determine_target_layer_project_without_mode() {
        let context = ProjectContext::default();
        let args = ResetArgs {
            soft: false,
            mixed: false,
            hard: false,
            mode: false,
            scope: None,
            project: true,
        };
        let result = determine_target_layer(&args, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_staging_empty() {
        let mut staging = StagingIndex::new();
        let result = reset_staging(&mut staging, Layer::ProjectBase);
        assert!(result.is_ok());
        assert!(staging.is_empty());
    }

    #[test]
    fn test_layer_name() {
        assert_eq!(layer_name(Layer::GlobalBase), "global-base");
        assert_eq!(layer_name(Layer::ModeBase), "mode-base");
        assert_eq!(layer_name(Layer::ProjectBase), "project-base");
    }
}
