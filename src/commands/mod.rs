//! Command implementations for Jin
//!
//! Each command module contains the implementation for a specific CLI command.

use crate::cli::{Cli, Commands};
use crate::core::Result;

pub mod add;
pub mod apply;
pub mod commit_cmd;
pub mod completion;
pub mod context;
pub mod diff;
pub mod export;
pub mod fetch;
pub mod import_cmd;
pub mod init;
pub mod layers;
pub mod link;
pub mod list;
pub mod log;
pub mod mode;
pub mod pull;
pub mod push;
pub mod repair;
pub mod reset;
pub mod scope;
pub mod status;
pub mod sync;

/// Execute the appropriate command based on CLI arguments
pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => init::execute(),
        Commands::Add(args) => add::execute(args),
        Commands::Commit(args) => commit_cmd::execute(args),
        Commands::Status => status::execute(),
        Commands::Mode(action) => mode::execute(action),
        Commands::Scope(action) => scope::execute(action),
        Commands::Apply(args) => apply::execute(args),
        Commands::Reset(args) => reset::execute(args),
        Commands::Diff(args) => diff::execute(args),
        Commands::Log(args) => log::execute(args),
        Commands::Context => context::execute(),
        Commands::Import(args) => import_cmd::execute(args),
        Commands::Export(args) => export::execute(args),
        Commands::Repair(args) => repair::execute(args),
        Commands::Layers => layers::execute(),
        Commands::List => list::execute(),
        Commands::Link(args) => link::execute(args),
        Commands::Fetch => fetch::execute(),
        Commands::Pull => pull::execute(),
        Commands::Push(args) => push::execute(args),
        Commands::Sync => sync::execute(),
        Commands::Completion { shell } => completion::execute(shell),
    }
}
