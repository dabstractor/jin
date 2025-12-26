//! Implementation of `jin export`

use crate::cli::ExportArgs;
use crate::core::Result;

/// Execute the export command
///
/// Exports Jin files back to Git.
pub fn execute(args: ExportArgs) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin export {:?} - not yet implemented", args.files);
    Ok(())
}
