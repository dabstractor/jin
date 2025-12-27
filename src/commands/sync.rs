//! Implementation of `jin sync`
//!
//! Orchestrates fetch + pull + apply for complete synchronization workflow.

use crate::cli::ApplyArgs;
use crate::core::Result;

/// Execute the sync command
///
/// Comprehensive workflow that combines:
/// 1. Fetch: Download remote updates
/// 2. Pull: Merge remote changes into local layers
/// 3. Apply: Regenerate workspace files
///
/// This is equivalent to running `jin fetch && jin pull && jin apply` in sequence.
pub fn execute() -> Result<()> {
    println!("=== Jin Sync: Fetch + Pull + Apply ===\n");

    // Step 1: Fetch remote updates
    println!("Step 1/3: Fetching remote updates...");
    match super::fetch::execute() {
        Ok(()) => println!("✓ Fetch completed\n"),
        Err(e) => {
            eprintln!("✗ Fetch failed: {}", e);
            return Err(e);
        }
    }

    // Step 2: Pull (merge) remote changes
    println!("Step 2/3: Merging remote changes...");
    match super::pull::execute() {
        Ok(()) => println!("✓ Pull completed\n"),
        Err(e) => {
            eprintln!("✗ Pull failed: {}", e);
            eprintln!("\nSync stopped at merge phase.");
            eprintln!("Resolve conflicts and run 'jin apply' to complete workspace update.");
            return Err(e);
        }
    }

    // Step 3: Apply to workspace
    println!("Step 3/3: Applying to workspace...");
    let apply_args = ApplyArgs {
        force: false,
        dry_run: false,
    };
    match super::apply::execute(apply_args) {
        Ok(()) => println!("✓ Apply completed\n"),
        Err(e) => {
            eprintln!("✗ Apply failed: {}", e);
            eprintln!("\nRemote changes merged successfully, but workspace update failed.");
            eprintln!("Run 'jin apply' manually to update workspace files.");
            return Err(e);
        }
    }

    println!("=== Sync completed successfully ===");
    println!("Your workspace is now synchronized with the remote repository.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_execute_signature() {
        // Verify the execute function signature is correct
        // Actual execution would require a full Jin environment
        fn _type_check() {
            let _: fn() -> Result<()> = execute;
        }
    }
}
