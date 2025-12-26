//! Implementation of `jin import`

use crate::cli::ImportArgs;
use crate::core::Result;

/// Execute the import command
///
/// Imports Git-tracked files into Jin.
pub fn execute(args: ImportArgs) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin import {:?} - not yet implemented", args.files);
    Ok(())
}
