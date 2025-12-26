//! CLI definitions using clap derive macros.
//!
//! This module contains the complete command-line interface structure for Jin.

pub mod args;

// Re-export all CLI types for convenient use
pub use args::{
    AddCommand, ApplyCommand, Cli, Commands, CommitCommand, DiffCommand, ExportCommand,
    ImportCommand, InitCommand, LinkCommand, LogCommand, ModeCommand, MvCommand, RepairCommand,
    ResetCommand, RmCommand, ScopeCommand, StatusCommand,
};
