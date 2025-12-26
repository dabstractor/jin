//! Implementation of `jin diff`

use crate::cli::DiffArgs;
use crate::core::Result;

/// Execute the diff command
///
/// Shows differences between layers.
pub fn execute(args: DiffArgs) -> Result<()> {
    // TODO: Implement in later milestone
    match (&args.layer1, &args.layer2) {
        (Some(l1), Some(l2)) => println!("jin diff {} {} - not yet implemented", l1, l2),
        (Some(l1), None) => println!("jin diff {} - not yet implemented", l1),
        _ if args.staged => println!("jin diff --staged - not yet implemented"),
        _ => println!("jin diff - not yet implemented"),
    }
    Ok(())
}
