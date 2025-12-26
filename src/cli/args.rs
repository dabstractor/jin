//! CLI definitions using clap derive macros.
//!
//! This module defines the complete command-line interface structure for Jin.
//! All commands use clap's derive API for type-safe, declarative argument parsing.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Main CLI structure for Jin.
///
/// This is the entry point for all command-line interactions.
#[derive(Parser)]
#[command(name = "jin")]
#[command(
    about = "Multi-layer Git overlay system",
    long_about = "
Jin is a meta-versioning system layered on top of Git that manages
developer-specific and tool-specific configuration without contaminating
a project's primary Git repository.

Use 'jin help <command>' for more information on a specific command.
"
)]
#[command(version)]
pub struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available Jin commands.
///
/// This enum defines all top-level commands from the PRD specification.
/// Commands are organized by functional area:
/// - Core operations (init, add, commit, reset, status)
/// - Context management (mode, modes, scope, scopes)
/// - Workspace operations (apply)
/// - Inspection (diff, log, context, layers, list)
/// - Import/export (import, export)
/// - Maintenance (repair)
/// - Remote operations (link, fetch, pull, push, sync)
/// - File operations (rm, mv)
#[derive(Subcommand)]
pub enum Commands {
    // ===== Core Commands =====
    /// Initialize Jin in the current project
    Init(InitCommand),

    /// Stage files to the appropriate layer
    Add(AddCommand),

    /// Commit staged files
    Commit(CommitCommand),

    /// Reset staged or committed changes
    Reset(ResetCommand),

    /// Show workspace state and status
    Status(StatusCommand),

    // ===== Mode Management =====
    /// Mode management commands
    #[command(subcommand)]
    Mode(ModeCommand),

    /// List all available modes
    Modes,

    // ===== Scope Management =====
    /// Scope management commands
    #[command(subcommand)]
    Scope(ScopeCommand),

    /// List all available scopes
    Scopes,

    // ===== Workspace Operations =====
    /// Merge layers into the workspace
    Apply(ApplyCommand),

    // ===== Inspection Commands =====
    /// Show differences between layers or workspace
    Diff(DiffCommand),

    /// Show commit history for layers
    Log(LogCommand),

    /// Show active context
    Context,

    /// Show current layer composition
    Layers,

    /// List available modes/scopes/projects
    List(ListCommand),

    // ===== Import/Export =====
    /// Import Git-tracked files into Jin
    Import(ImportCommand),

    /// Export Jin files back to Git
    Export(ExportCommand),

    // ===== Maintenance =====
    /// Repair Jin state
    Repair(RepairCommand),

    // ===== Remote Operations =====
    /// Link to shared Jin config repository
    Link(LinkCommand),

    /// Fetch updates from remote Jin repo
    Fetch,

    /// Fetch and merge updates
    Pull,

    /// Push local changes
    Push,

    /// Fetch, merge, and apply
    Sync,

    // ===== File Operations =====
    /// Remove file from layer
    Rm(RmCommand),

    /// Rename/move file within layer
    Mv(MvCommand),
}

/// Mode subcommands.
///
/// Manages mode lifecycle (create, activate, deactivate, delete, show).
#[derive(Subcommand)]
pub enum ModeCommand {
    /// Create a new mode
    Create {
        /// Mode name to create
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Activate a mode (set as active context)
    Use {
        /// Mode name to activate
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Deactivate the current mode
    Unset,

    /// Delete a mode
    Delete {
        /// Mode name to delete
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Show the current active mode
    Show,
}

/// Scope subcommands.
///
/// Manages scope lifecycle (create, activate, deactivate, delete, show).
#[derive(Subcommand)]
pub enum ScopeCommand {
    /// Create a new scope
    Create {
        /// Scope name to create
        #[arg(value_name = "SCOPE")]
        name: String,

        /// Bind to a specific mode
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
    },

    /// Activate a scope (set as active context)
    Use {
        /// Scope name to activate
        #[arg(value_name = "SCOPE")]
        name: String,
    },

    /// Deactivate the current scope
    Unset,

    /// Delete a scope
    Delete {
        /// Scope name to delete
        #[arg(value_name = "SCOPE")]
        name: String,
    },

    /// Show the current active scope
    Show,
}

/// Arguments for 'jin init'
#[derive(clap::Args)]
pub struct InitCommand;

/// Arguments for 'jin add'
#[derive(clap::Args)]
pub struct AddCommand {
    /// Files to stage
    #[arg(value_name = "FILE", num_args(1..), required = true)]
    pub files: Vec<PathBuf>,

    /// Route to mode base layer (uses active mode)
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Route to global layer
    #[arg(long)]
    pub global: bool,
}

/// Arguments for 'jin commit'
#[derive(clap::Args)]
pub struct CommitCommand {
    /// Commit message
    #[arg(long, short, required = true)]
    pub message: String,

    /// Allow empty commit
    #[arg(long)]
    pub allow_empty: bool,
}

/// Arguments for 'jin reset'
#[derive(clap::Args)]
pub struct ResetCommand {
    /// Paths to reset (optional, defaults to all)
    #[arg(value_name = "PATH", num_args(0..))]
    pub paths: Vec<PathBuf>,

    /// Route to mode layer
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Keep changes in staging area
    #[arg(long)]
    pub soft: bool,

    /// Unstage but keep in workspace (default)
    #[arg(long, conflicts_with = "soft", conflicts_with = "hard")]
    pub mixed: bool,

    /// Discard all changes
    #[arg(long, conflicts_with = "soft")]
    pub hard: bool,
}

/// Arguments for 'jin status'
#[derive(clap::Args)]
pub struct StatusCommand;

/// Arguments for 'jin list'
#[derive(clap::Args)]
pub struct ListCommand;

/// Arguments for 'jin apply'
#[derive(clap::Args)]
pub struct ApplyCommand {
    /// Skip dirty check and force apply
    #[arg(long)]
    pub force: bool,

    /// Show plan without applying
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for 'jin diff'
#[derive(clap::Args)]
pub struct DiffCommand {
    /// First layer (optional)
    #[arg(value_name = "LAYER1")]
    pub layer1: Option<String>,

    /// Second layer (optional)
    #[arg(value_name = "LAYER2")]
    pub layer2: Option<String>,

    /// Show staged vs layer current
    #[arg(long)]
    pub staged: bool,
}

/// Arguments for 'jin log'
#[derive(clap::Args)]
pub struct LogCommand {
    /// Layer to show history for (optional)
    #[arg(value_name = "LAYER")]
    pub layer: Option<String>,

    /// Limit number of entries
    #[arg(long, value_name = "N")]
    pub count: Option<usize>,
}

/// Arguments for 'jin import'
#[derive(clap::Args)]
pub struct ImportCommand {
    /// Files to import
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,

    /// Route to mode base layer (uses active mode)
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Route to global layer
    #[arg(long)]
    pub global: bool,
}

/// Arguments for 'jin export'
#[derive(clap::Args)]
pub struct ExportCommand {
    /// Files to export
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,
}

/// Arguments for 'jin repair'
#[derive(clap::Args)]
pub struct RepairCommand {
    /// Show plan without repairing
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for 'jin link'
#[derive(clap::Args)]
pub struct LinkCommand {
    /// Repository URL to link
    #[arg(value_name = "URL")]
    pub url: String,
}

/// Arguments for 'jin rm'
#[derive(clap::Args)]
pub struct RmCommand {
    /// Files to remove
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,

    /// Route to mode layer
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,
}

/// Arguments for 'jin mv'
#[derive(clap::Args)]
pub struct MvCommand {
    /// Source path
    #[arg(value_name = "OLD")]
    pub old_path: PathBuf,

    /// Destination path
    #[arg(value_name = "NEW")]
    pub new_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic parsing tests

    #[test]
    fn test_cli_basic_parsing() {
        // Should parse successfully with a subcommand
        let cli = Cli::try_parse_from(["jin", "init"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(matches!(cli.command, Commands::Init(_)));
    }

    #[test]
    fn test_init_command() {
        let cli = Cli::try_parse_from(["jin", "init"]).unwrap();
        assert!(matches!(cli.command, Commands::Init(_)));
    }

    #[test]
    fn test_add_command_with_files() {
        let cli = Cli::try_parse_from(["jin", "add", "file1.txt", "file2.txt"]).unwrap();
        match cli.command {
            Commands::Add(cmd) => {
                assert_eq!(cmd.files.len(), 2);
                assert_eq!(cmd.files[0], PathBuf::from("file1.txt"));
                assert_eq!(cmd.files[1], PathBuf::from("file2.txt"));
                assert!(!cmd.mode);
                assert!(!cmd.project);
                assert!(!cmd.global);
                assert!(cmd.scope.is_none());
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_add_command_with_mode_flag() {
        let cli = Cli::try_parse_from(["jin", "add", "file.txt", "--mode"]).unwrap();
        match cli.command {
            Commands::Add(cmd) => {
                assert!(cmd.mode);
                assert!(!cmd.project);
                assert!(!cmd.global);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_add_command_with_scope_flag() {
        let cli = Cli::try_parse_from(["jin", "add", "file.txt", "--scope", "python"]).unwrap();
        match cli.command {
            Commands::Add(cmd) => {
                assert_eq!(cmd.scope, Some("python".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_add_command_with_project_flag() {
        let cli = Cli::try_parse_from(["jin", "add", "file.txt", "--project"]).unwrap();
        match cli.command {
            Commands::Add(cmd) => {
                assert!(cmd.project);
                assert!(!cmd.mode);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_add_command_with_global_flag() {
        let cli = Cli::try_parse_from(["jin", "add", "file.txt", "--global"]).unwrap();
        match cli.command {
            Commands::Add(cmd) => {
                assert!(cmd.global);
            }
            _ => panic!("Expected Add command"),
        }
    }

    // Mode subcommand tests

    #[test]
    fn test_mode_create_command() {
        let cli = Cli::try_parse_from(["jin", "mode", "create", "claude"]).unwrap();
        match cli.command {
            Commands::Mode(ModeCommand::Create { name }) => {
                assert_eq!(name, "claude");
            }
            _ => panic!("Expected Mode::Create"),
        }
    }

    #[test]
    fn test_mode_use_command() {
        let cli = Cli::try_parse_from(["jin", "mode", "use", "claude"]).unwrap();
        match cli.command {
            Commands::Mode(ModeCommand::Use { name }) => {
                assert_eq!(name, "claude");
            }
            _ => panic!("Expected Mode::Use"),
        }
    }

    #[test]
    fn test_mode_unset_command() {
        let cli = Cli::try_parse_from(["jin", "mode", "unset"]).unwrap();
        assert!(matches!(cli.command, Commands::Mode(ModeCommand::Unset)));
    }

    #[test]
    fn test_mode_delete_command() {
        let cli = Cli::try_parse_from(["jin", "mode", "delete", "oldmode"]).unwrap();
        match cli.command {
            Commands::Mode(ModeCommand::Delete { name }) => {
                assert_eq!(name, "oldmode");
            }
            _ => panic!("Expected Mode::Delete"),
        }
    }

    #[test]
    fn test_mode_show_command() {
        let cli = Cli::try_parse_from(["jin", "mode", "show"]).unwrap();
        assert!(matches!(cli.command, Commands::Mode(ModeCommand::Show)));
    }

    #[test]
    fn test_modes_command() {
        let cli = Cli::try_parse_from(["jin", "modes"]).unwrap();
        assert!(matches!(cli.command, Commands::Modes));
    }

    // Scope subcommand tests

    #[test]
    fn test_scope_create_command() {
        let cli = Cli::try_parse_from(["jin", "scope", "create", "backend"]).unwrap();
        match cli.command {
            Commands::Scope(ScopeCommand::Create { name, mode }) => {
                assert_eq!(name, "backend");
                assert!(mode.is_none());
            }
            _ => panic!("Expected Scope::Create"),
        }
    }

    #[test]
    fn test_scope_create_with_mode() {
        let cli =
            Cli::try_parse_from(["jin", "scope", "create", "backend", "--mode", "claude"]).unwrap();
        match cli.command {
            Commands::Scope(ScopeCommand::Create { name, mode }) => {
                assert_eq!(name, "backend");
                assert_eq!(mode, Some("claude".to_string()));
            }
            _ => panic!("Expected Scope::Create with mode"),
        }
    }

    #[test]
    fn test_scope_use_command() {
        let cli = Cli::try_parse_from(["jin", "scope", "use", "backend"]).unwrap();
        match cli.command {
            Commands::Scope(ScopeCommand::Use { name }) => {
                assert_eq!(name, "backend");
            }
            _ => panic!("Expected Scope::Use"),
        }
    }

    #[test]
    fn test_scope_unset_command() {
        let cli = Cli::try_parse_from(["jin", "scope", "unset"]).unwrap();
        assert!(matches!(cli.command, Commands::Scope(ScopeCommand::Unset)));
    }

    #[test]
    fn test_scope_delete_command() {
        let cli = Cli::try_parse_from(["jin", "scope", "delete", "oldscope"]).unwrap();
        match cli.command {
            Commands::Scope(ScopeCommand::Delete { name }) => {
                assert_eq!(name, "oldscope");
            }
            _ => panic!("Expected Scope::Delete"),
        }
    }

    #[test]
    fn test_scope_show_command() {
        let cli = Cli::try_parse_from(["jin", "scope", "show"]).unwrap();
        assert!(matches!(cli.command, Commands::Scope(ScopeCommand::Show)));
    }

    #[test]
    fn test_scopes_command() {
        let cli = Cli::try_parse_from(["jin", "scopes"]).unwrap();
        assert!(matches!(cli.command, Commands::Scopes));
    }

    // Commit tests

    #[test]
    fn test_commit_with_message() {
        let cli = Cli::try_parse_from(["jin", "commit", "--message", "Initial commit"]).unwrap();
        match cli.command {
            Commands::Commit(cmd) => {
                assert_eq!(cmd.message, "Initial commit");
                assert!(!cmd.allow_empty);
            }
            _ => panic!("Expected Commit command"),
        }
    }

    #[test]
    fn test_commit_with_allow_empty() {
        let cli = Cli::try_parse_from(["jin", "commit", "-m", "Empty", "--allow-empty"]).unwrap();
        match cli.command {
            Commands::Commit(cmd) => {
                assert!(cmd.allow_empty);
            }
            _ => panic!("Expected Commit command"),
        }
    }

    // Reset tests

    #[test]
    fn test_reset_default() {
        let cli = Cli::try_parse_from(["jin", "reset"]).unwrap();
        match cli.command {
            Commands::Reset(cmd) => {
                assert!(cmd.paths.is_empty());
                assert!(!cmd.soft && !cmd.mixed && !cmd.hard);
            }
            _ => panic!("Expected Reset command"),
        }
    }

    #[test]
    fn test_reset_with_soft_flag() {
        let cli = Cli::try_parse_from(["jin", "reset", "--soft"]).unwrap();
        match cli.command {
            Commands::Reset(cmd) => {
                assert!(cmd.soft);
                assert!(!cmd.mixed);
                assert!(!cmd.hard);
            }
            _ => panic!("Expected Reset command"),
        }
    }

    #[test]
    fn test_reset_with_hard_flag() {
        let cli = Cli::try_parse_from(["jin", "reset", "--hard"]).unwrap();
        match cli.command {
            Commands::Reset(cmd) => {
                assert!(cmd.hard);
                assert!(!cmd.soft);
                assert!(!cmd.mixed);
            }
            _ => panic!("Expected Reset command"),
        }
    }

    #[test]
    fn test_reset_with_paths() {
        let cli = Cli::try_parse_from(["jin", "reset", "file1.txt", "file2.txt"]).unwrap();
        match cli.command {
            Commands::Reset(cmd) => {
                assert_eq!(cmd.paths.len(), 2);
            }
            _ => panic!("Expected Reset command"),
        }
    }

    // Apply tests

    #[test]
    fn test_apply_default() {
        let cli = Cli::try_parse_from(["jin", "apply"]).unwrap();
        match cli.command {
            Commands::Apply(cmd) => {
                assert!(!cmd.force);
                assert!(!cmd.dry_run);
            }
            _ => panic!("Expected Apply command"),
        }
    }

    #[test]
    fn test_apply_with_force_and_dry_run() {
        let cli = Cli::try_parse_from(["jin", "apply", "--force", "--dry-run"]).unwrap();
        match cli.command {
            Commands::Apply(cmd) => {
                assert!(cmd.force);
                assert!(cmd.dry_run);
            }
            _ => panic!("Expected Apply command"),
        }
    }

    // Diff tests

    #[test]
    fn test_diff_default() {
        let cli = Cli::try_parse_from(["jin", "diff"]).unwrap();
        match cli.command {
            Commands::Diff(cmd) => {
                assert!(cmd.layer1.is_none());
                assert!(cmd.layer2.is_none());
                assert!(!cmd.staged);
            }
            _ => panic!("Expected Diff command"),
        }
    }

    #[test]
    fn test_diff_with_layers() {
        let cli = Cli::try_parse_from(["jin", "diff", "mode/claude", "project"]).unwrap();
        match cli.command {
            Commands::Diff(cmd) => {
                assert_eq!(cmd.layer1, Some("mode/claude".to_string()));
                assert_eq!(cmd.layer2, Some("project".to_string()));
            }
            _ => panic!("Expected Diff command"),
        }
    }

    #[test]
    fn test_diff_with_staged() {
        let cli = Cli::try_parse_from(["jin", "diff", "--staged"]).unwrap();
        match cli.command {
            Commands::Diff(cmd) => {
                assert!(cmd.staged);
            }
            _ => panic!("Expected Diff command"),
        }
    }

    // Log tests

    #[test]
    fn test_log_default() {
        let cli = Cli::try_parse_from(["jin", "log"]).unwrap();
        match cli.command {
            Commands::Log(cmd) => {
                assert!(cmd.layer.is_none());
                assert!(cmd.count.is_none());
            }
            _ => panic!("Expected Log command"),
        }
    }

    #[test]
    fn test_log_with_layer() {
        let cli = Cli::try_parse_from(["jin", "log", "project"]).unwrap();
        match cli.command {
            Commands::Log(cmd) => {
                assert_eq!(cmd.layer, Some("project".to_string()));
            }
            _ => panic!("Expected Log command"),
        }
    }

    #[test]
    fn test_log_with_count() {
        let cli = Cli::try_parse_from(["jin", "log", "--count", "10"]).unwrap();
        match cli.command {
            Commands::Log(cmd) => {
                assert_eq!(cmd.count, Some(10));
            }
            _ => panic!("Expected Log command"),
        }
    }

    // Import/Export tests

    #[test]
    fn test_import_command() {
        let cli = Cli::try_parse_from(["jin", "import", "file1.txt", "file2.txt"]).unwrap();
        match cli.command {
            Commands::Import(cmd) => {
                assert_eq!(cmd.files.len(), 2);
            }
            _ => panic!("Expected Import command"),
        }
    }

    #[test]
    fn test_export_command() {
        let cli = Cli::try_parse_from(["jin", "export", "file1.txt", "file2.txt"]).unwrap();
        match cli.command {
            Commands::Export(cmd) => {
                assert_eq!(cmd.files.len(), 2);
            }
            _ => panic!("Expected Export command"),
        }
    }

    // Repair tests

    #[test]
    fn test_repair_default() {
        let cli = Cli::try_parse_from(["jin", "repair"]).unwrap();
        match cli.command {
            Commands::Repair(cmd) => {
                assert!(!cmd.dry_run);
            }
            _ => panic!("Expected Repair command"),
        }
    }

    #[test]
    fn test_repair_with_dry_run() {
        let cli = Cli::try_parse_from(["jin", "repair", "--dry-run"]).unwrap();
        match cli.command {
            Commands::Repair(cmd) => {
                assert!(cmd.dry_run);
            }
            _ => panic!("Expected Repair command"),
        }
    }

    // Link tests

    #[test]
    fn test_link_command() {
        let cli =
            Cli::try_parse_from(["jin", "link", "https://github.com/user/jin-config"]).unwrap();
        match cli.command {
            Commands::Link(cmd) => {
                assert_eq!(cmd.url, "https://github.com/user/jin-config");
            }
            _ => panic!("Expected Link command"),
        }
    }

    // Remote operation tests

    #[test]
    fn test_fetch_command() {
        let cli = Cli::try_parse_from(["jin", "fetch"]).unwrap();
        assert!(matches!(cli.command, Commands::Fetch));
    }

    #[test]
    fn test_pull_command() {
        let cli = Cli::try_parse_from(["jin", "pull"]).unwrap();
        assert!(matches!(cli.command, Commands::Pull));
    }

    #[test]
    fn test_push_command() {
        let cli = Cli::try_parse_from(["jin", "push"]).unwrap();
        assert!(matches!(cli.command, Commands::Push));
    }

    #[test]
    fn test_sync_command() {
        let cli = Cli::try_parse_from(["jin", "sync"]).unwrap();
        assert!(matches!(cli.command, Commands::Sync));
    }

    // Inspection command tests

    #[test]
    fn test_status_command() {
        let cli = Cli::try_parse_from(["jin", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Status(_)));
    }

    #[test]
    fn test_context_command() {
        let cli = Cli::try_parse_from(["jin", "context"]).unwrap();
        assert!(matches!(cli.command, Commands::Context));
    }

    #[test]
    fn test_layers_command() {
        let cli = Cli::try_parse_from(["jin", "layers"]).unwrap();
        assert!(matches!(cli.command, Commands::Layers));
    }

    #[test]
    fn test_list_command() {
        let cli = Cli::try_parse_from(["jin", "list"]).unwrap();
        assert!(matches!(cli.command, Commands::List(_)));
    }

    // File operation tests

    #[test]
    fn test_rm_command() {
        let cli = Cli::try_parse_from(["jin", "rm", "file1.txt", "file2.txt"]).unwrap();
        match cli.command {
            Commands::Rm(cmd) => {
                assert_eq!(cmd.files.len(), 2);
                assert!(!cmd.mode);
                assert!(!cmd.project);
            }
            _ => panic!("Expected Rm command"),
        }
    }

    #[test]
    fn test_rm_with_mode_flag() {
        let cli = Cli::try_parse_from(["jin", "rm", "file.txt", "--mode"]).unwrap();
        match cli.command {
            Commands::Rm(cmd) => {
                assert!(cmd.mode);
            }
            _ => panic!("Expected Rm command"),
        }
    }

    #[test]
    fn test_mv_command() {
        let cli = Cli::try_parse_from(["jin", "mv", "old.txt", "new.txt"]).unwrap();
        match cli.command {
            Commands::Mv(cmd) => {
                assert_eq!(cmd.old_path, PathBuf::from("old.txt"));
                assert_eq!(cmd.new_path, PathBuf::from("new.txt"));
            }
            _ => panic!("Expected Mv command"),
        }
    }

    // Error handling tests

    #[test]
    fn test_invalid_command() {
        let cli = Cli::try_parse_from(["jin", "invalid-command"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_missing_required_files() {
        // 'jin add' requires at least one file
        let cli = Cli::try_parse_from(["jin", "add"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_missing_required_commit_message() {
        // 'jin commit' requires -m/--message
        let cli = Cli::try_parse_from(["jin", "commit"]);
        assert!(cli.is_err());
    }

    #[test]
    fn test_conflicting_reset_flags() {
        // --soft and --hard are mutually exclusive
        let cli = Cli::try_parse_from(["jin", "reset", "--soft", "--hard"]);
        assert!(cli.is_err());
    }
}
