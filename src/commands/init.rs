//! Implementation of `jin init`

use crate::core::{ProjectContext, Result};
use crate::git::JinRepo;
use std::fs;
use std::io::Write;

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

    // Add .jin/ to .gitignore if not already present
    add_to_gitignore(".jin/")?;

    println!("Initialized Jin in {}", jin_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. Create a mode:     jin mode create <name>");
    println!("  2. Activate the mode: jin mode use <name>");
    println!("  3. Add files:         jin add <file> --mode");

    Ok(())
}

/// Add an entry to .gitignore if not already present
fn add_to_gitignore(entry: &str) -> Result<()> {
    let gitignore_path = std::path::Path::new(".gitignore");

    // Check if entry already exists and determine if we need a leading newline
    let needs_newline = if gitignore_path.exists() {
        let contents = fs::read_to_string(gitignore_path)?;
        for line in contents.lines() {
            if line.trim() == entry || line.trim() == entry.trim_end_matches('/') {
                return Ok(()); // Already present
            }
        }
        !contents.is_empty() && !contents.ends_with('\n')
    } else {
        false
    };

    // Append entry to .gitignore
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(gitignore_path)?;

    if needs_newline {
        writeln!(file)?;
    }
    writeln!(file, "{}", entry)?;
    Ok(())
}
