//! CLI argument definitions for Jin
//!
//! Uses clap derive API for command-line argument parsing.

pub mod args;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

pub use args::*;

/// Jin - Phantom Git layer system for developer configuration
#[derive(Parser, Debug)]
#[command(name = "jin")]
#[command(
    author,
    version,
    about = "Phantom Git layer system for developer configuration"
)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available Jin commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize Jin in current project
    Init,

    /// Stage files to appropriate layer
    Add(AddArgs),

    /// Commit staged files atomically
    Commit(CommitArgs),

    /// Show workspace state and active contexts
    Status,

    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),

    /// Scope lifecycle management
    #[command(subcommand)]
    Scope(ScopeAction),

    /// Apply merged layers to workspace
    Apply(ApplyArgs),

    /// Reset staged or committed changes
    Reset(ResetArgs),

    /// Show differences between layers
    Diff(DiffArgs),

    /// Show commit history
    Log(LogArgs),

    /// Show/set active context
    Context,

    /// Import Git-tracked files into Jin
    Import(ImportArgs),

    /// Export Jin files back to Git
    Export(ExportArgs),

    /// Repair Jin state
    Repair(RepairArgs),

    /// Show current layer composition
    Layers,

    /// List available modes/scopes/projects
    List,

    /// Link to shared Jin config repo
    Link(LinkArgs),

    /// Fetch updates from remote
    Fetch,

    /// Fetch and merge updates
    Pull,

    /// Push local changes
    Push(PushArgs),

    /// Fetch + merge + apply
    Sync,

    /// Generate shell completion scripts
    ///
    /// Outputs completion script to stdout. Redirect to a file and source it
    /// to enable tab completion in your shell.
    ///
    /// Installation:
    ///   Bash:       jin completion bash > /usr/local/share/bash-completion/completions/jin
    ///   Zsh:        jin completion zsh > ~/.zsh/completions/_jin
    ///   Fish:       jin completion fish > ~/.config/fish/completions/jin.fish
    ///   PowerShell: jin completion powershell > $PROFILE\..\Completions\jin_completion.ps1
    Completion {
        /// Shell type to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Mode subcommands
#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create {
        /// Name of the mode to create
        name: String,
    },
    /// Activate a mode
    Use {
        /// Name of the mode to activate
        name: String,
    },
    /// List available modes
    List,
    /// Delete a mode
    Delete {
        /// Name of the mode to delete
        name: String,
    },
    /// Show current mode
    Show,
    /// Deactivate current mode
    Unset,
}

/// Scope subcommands
#[derive(Subcommand, Debug)]
pub enum ScopeAction {
    /// Create a new scope
    Create {
        /// Name of the scope to create
        name: String,
        /// Associate with a mode
        #[arg(long)]
        mode: Option<String>,
    },
    /// Activate a scope
    Use {
        /// Name of the scope to activate
        name: String,
    },
    /// List available scopes
    List,
    /// Delete a scope
    Delete {
        /// Name of the scope to delete
        name: String,
    },
    /// Show current scope
    Show,
    /// Deactivate current scope
    Unset,
}
