//! Implementation of `jin reset`

use crate::cli::ResetArgs;
use crate::core::Result;

/// Execute the reset command
///
/// Resets staged or committed changes.
pub fn execute(args: ResetArgs) -> Result<()> {
    // TODO: Implement in later milestone
    let mode = if args.soft {
        "soft"
    } else if args.hard {
        "hard"
    } else {
        "mixed"
    };
    println!("jin reset --{} - not yet implemented", mode);
    let _ = args;
    Ok(())
}
