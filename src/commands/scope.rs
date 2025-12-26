//! Implementation of `jin scope` subcommands

use crate::cli::ScopeAction;
use crate::core::Result;

/// Execute a scope subcommand
pub fn execute(action: ScopeAction) -> Result<()> {
    match action {
        ScopeAction::Create { name, mode } => create(&name, mode.as_deref()),
        ScopeAction::Use { name } => use_scope(&name),
        ScopeAction::List => list(),
        ScopeAction::Delete { name } => delete(&name),
        ScopeAction::Show => show(),
        ScopeAction::Unset => unset(),
    }
}

fn create(name: &str, mode: Option<&str>) -> Result<()> {
    // TODO: Implement in later milestone
    match mode {
        Some(m) => println!(
            "jin scope create {} --mode={} - not yet implemented",
            name, m
        ),
        None => println!("jin scope create {} - not yet implemented", name),
    }
    Ok(())
}

fn use_scope(name: &str) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin scope use {} - not yet implemented", name);
    Ok(())
}

fn list() -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin scope list - not yet implemented");
    Ok(())
}

fn delete(name: &str) -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin scope delete {} - not yet implemented", name);
    Ok(())
}

fn show() -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin scope show - not yet implemented");
    Ok(())
}

fn unset() -> Result<()> {
    // TODO: Implement in later milestone
    println!("jin scope unset - not yet implemented");
    Ok(())
}
