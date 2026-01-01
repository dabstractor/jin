//! Implementation of `jin status`

use crate::core::{JinError, ProjectContext, Result};
use crate::staging::StagingIndex;

/// Execute the status command
///
/// Shows workspace state and active contexts.
pub fn execute() -> Result<()> {
    // Check if Jin is initialized
    if !ProjectContext::is_initialized() {
        return Err(JinError::NotInitialized);
    }

    // Load context
    let context = ProjectContext::load()?;

    println!("Jin status:");
    println!();

    // Show active mode
    match &context.mode {
        Some(mode) => println!("  Mode:  {} (active)", mode),
        None => println!("  Mode:  (none)"),
    }

    // Show active scope
    match &context.scope {
        Some(scope) => println!("  Scope: {} (active)", scope),
        None => println!("  Scope: (none)"),
    }

    println!();

    // Show staged files
    let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());
    let staged_count = staging.len();

    if staged_count == 0 {
        println!("No staged changes.");
        println!();
        println!("Use 'jin add <file>' to stage files for commit.");
    } else {
        println!("Staged changes ({} file{}):", staged_count, if staged_count == 1 { "" } else { "s" });
        for entry in staging.entries() {
            println!("  {} -> {}", entry.path.display(), entry.target_layer);
        }
        println!();
        println!("Use 'jin commit -m <message>' to commit staged changes.");
    }

    Ok(())
}
