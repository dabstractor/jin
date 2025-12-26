//! Implementation of `jin push`

use crate::cli::PushArgs;
use crate::core::Result;

/// Execute the push command
///
/// Pushes local changes.
pub fn execute(args: PushArgs) -> Result<()> {
    // TODO: Implement in later milestone
    if args.force {
        println!("jin push --force - not yet implemented");
    } else {
        println!("jin push - not yet implemented");
    }
    Ok(())
}
