//! Implementation of `jin apply`

use crate::cli::ApplyArgs;
use crate::core::Result;

/// Execute the apply command
///
/// Applies merged layers to workspace.
pub fn execute(args: ApplyArgs) -> Result<()> {
    // TODO: Implement in later milestone
    if args.dry_run {
        println!("jin apply --dry-run - not yet implemented");
    } else {
        println!("jin apply - not yet implemented");
    }
    Ok(())
}
