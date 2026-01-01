//! Implementation of `jin init`

use crate::core::{ProjectContext, Result};
use crate::git::JinRepo;
use std::fs;

/// Execute the init command
///
/// Initializes Jin in the current project directory.
pub fn execute() -> Result<()> {
    // Check if already initialized
    if ProjectContext::is_initialized() {
        println!("Jin is already initialized in this directory");
        return Ok(());
    }

    // Create .jin directory
    let jin_dir = ProjectContext::default_path()
        .parent()
        .expect("context path should have parent")
        .to_path_buf();

    fs::create_dir_all(&jin_dir)?;

    // Create default context
    let context = ProjectContext::default();
    context.save()?;

    // Ensure global Jin repository exists
    JinRepo::open_or_create()?;

    println!("Initialized Jin in {}", jin_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. Create a mode:     jin mode create <name>");
    println!("  2. Activate the mode: jin mode use <name>");
    println!("  3. Add files:         jin add <file> --mode");

    Ok(())
}
