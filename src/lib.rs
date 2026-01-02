//! # Jin - Phantom Git Layer System
//!
//! Jin is a meta-versioning system layered on top of Git that manages
//! developer-specific and tool-specific configuration without contaminating
//! a project's primary Git repository.
//!
//! ## Key Features
//!
//! - **9-layer hierarchy** for configuration precedence
//! - **Mode/Scope/Project** organization for flexible targeting
//! - **Deterministic merging** of structured config files (JSON, YAML, TOML)
//! - **Atomic commits** across multiple layers
//! - **Automatic .gitignore management**

pub mod audit;
pub mod cli;
pub mod commands;
pub mod commit;
pub mod core;
pub mod git;
pub mod merge;
pub mod staging;

// Re-export commonly used types
pub use core::error::{JinError, Result};
pub use core::layer::Layer;

/// Execute the Jin CLI with the parsed arguments
pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    commands::execute(cli).map_err(|e| anyhow::anyhow!("{}", e))
}
