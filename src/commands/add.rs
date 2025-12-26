//! Implementation of `jin add`

use crate::cli::AddArgs;
use crate::core::Result;

/// Execute the add command
///
/// Stages files to the appropriate layer based on flags.
pub fn execute(args: AddArgs) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin add {:?} - not yet implemented", args.files);
    Ok(())
}
