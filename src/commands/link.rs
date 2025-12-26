//! Implementation of `jin link`

use crate::cli::LinkArgs;
use crate::core::Result;

/// Execute the link command
///
/// Links to shared Jin config repo.
pub fn execute(args: LinkArgs) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin link {} - not yet implemented", args.url);
    Ok(())
}
