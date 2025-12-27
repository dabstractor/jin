use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::Cli;
use crate::core::Result;

/// Execute the completion command to generate shell completion scripts
///
/// Generates shell-specific completion scripts to stdout. The generated script
/// can be redirected to a file and sourced to enable tab completion in the shell.
///
/// # Arguments
///
/// * `shell` - The shell type to generate completions for (bash, zsh, fish, powershell)
///
/// # Examples
///
/// ```bash
/// jin completion bash > /usr/local/share/bash-completion/completions/jin
/// jin completion zsh > ~/.zsh/completions/_jin
/// jin completion fish > ~/.config/fish/completions/jin.fish
/// jin completion powershell > $PROFILE\..\Completions\jin_completion.ps1
/// ```
pub fn execute(shell: Shell) -> Result<()> {
    // Get the clap Command from Cli's derive macros
    // This allows clap_complete to introspect the full command structure
    let mut cmd = Cli::command();

    // Generate completion script to stdout
    // Binary name "jin" must match [[bin]] name in Cargo.toml
    generate(shell, &mut cmd, "jin", &mut io::stdout());

    Ok(())
}
