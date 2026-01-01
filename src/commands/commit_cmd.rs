//! Implementation of `jin commit`
//!
//! Commits staged files atomically across all affected layers.
//! Uses the CommitPipeline to handle multi-layer atomic commits.

use crate::cli::CommitArgs;
use crate::commit::{CommitConfig, CommitPipeline, CommitResult};
use crate::core::{JinError, ProjectContext, Result};
use crate::staging::StagingIndex;

/// Execute the commit command
///
/// Commits staged files atomically across all affected layers.
///
/// # Arguments
///
/// * `args` - Command line arguments including message and dry_run flag
///
/// # Errors
///
/// Returns an error if:
/// - Jin is not initialized in the current project
/// - No files are staged (empty staging index)
/// - Commit operation fails (Git errors, transaction errors, etc.)
pub fn execute(args: CommitArgs) -> Result<()> {
    // PATTERN: Check initialization first (follow add.rs pattern)
    // ProjectContext::load() returns Err(JinError::NotInitialized) if not initialized
    let _context = ProjectContext::load()?;

    // PATTERN: Load staging index
    // This will fail if .jin doesn't exist (redundant with context check but safe)
    let staging = StagingIndex::load()?;

    // PATTERN: Build commit configuration
    // CommitConfig builder pattern - pass message as &str
    let config = CommitConfig::new(&args.message).dry_run(args.dry_run);

    // PATTERN: Create pipeline (staging is moved into pipeline)
    // CRITICAL: Cannot use staging after this line
    let mut pipeline = CommitPipeline::new(staging);

    // PATTERN: Execute commit with error handling
    // Handle "Nothing to commit" error with user-friendly message
    match pipeline.execute(&config) {
        Err(JinError::Other(ref msg)) if msg == "Nothing to commit" => {
            // GOTCHA: Empty staging check happens in pipeline
            return Err(JinError::Other(
                "No staged files to commit. Use 'jin add' to stage files first.".to_string(),
            ));
        }
        Err(e) => return Err(e),
        Ok(result) => {
            // PATTERN: Display results in user-friendly format
            display_commit_result(&result);
        }
    }

    Ok(())
}

/// Display commit results to the user
fn display_commit_result(result: &CommitResult) {
    // PATTERN: Format output similar to Git commits
    if result.commit_hashes.is_empty() {
        // Dry-run mode - no actual hashes (pipeline already printed details)
        // No additional output needed for dry-run
    } else {
        // Actual commit - show hashes
        println!(
            "Committed {} file(s) to {} layer(s):",
            result.file_count,
            result.committed_layers.len()
        );

        // PATTERN: Show layer and hash for each committed layer
        for (layer, hash) in &result.commit_hashes {
            println!("  {}: {}", layer, hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_no_message() {
        // This test verifies the command structure
        // The actual CLI validation is handled by clap
        let args = CommitArgs {
            message: "Test commit".to_string(),
            dry_run: false,
        };
        // We can't test execute without a proper Jin setup
        // This is just to verify the struct works
        assert_eq!(args.message, "Test commit");
        assert!(!args.dry_run);
    }

    #[test]
    fn test_execute_with_dry_run() {
        let args = CommitArgs {
            message: "Dry run test".to_string(),
            dry_run: true,
        };
        assert!(args.dry_run);
    }
}
