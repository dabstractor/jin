//! Implementation of `jin mode` subcommands

use crate::cli::ModeAction;
use crate::core::Result;

/// Execute a mode subcommand
pub fn execute(action: ModeAction) -> Result<()> {
    match action {
        ModeAction::Create { name } => create(&name),
        ModeAction::Use { name } => use_mode(&name),
        ModeAction::List => list(),
        ModeAction::Delete { name } => delete(&name),
        ModeAction::Show => show(),
        ModeAction::Unset => unset(),
    }
}

fn create(name: &str) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin mode create {} - not yet implemented", name);
    Ok(())
}

fn use_mode(name: &str) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin mode use {} - not yet implemented", name);
    Ok(())
}

fn list() -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin mode list - not yet implemented");
    Ok(())
}

fn delete(name: &str) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin mode delete {} - not yet implemented", name);
    Ok(())
}

fn show() -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin mode show - not yet implemented");
    Ok(())
}

fn unset() -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin mode unset - not yet implemented");
    Ok(())
}
