//! Implementation of `jin log`

use crate::cli::LogArgs;
use crate::core::Result;

/// Execute the log command
///
/// Shows commit history.
pub fn execute(args: LogArgs) -> Result<()> {
    // TODO: Implement in later milestone
    match &args.layer {
        Some(layer) => println!(
            "jin log --layer={} --count={} - not yet implemented",
            layer, args.count
        ),
        None => println!("jin log --count={} - not yet implemented", args.count),
    }
    Ok(())
}
