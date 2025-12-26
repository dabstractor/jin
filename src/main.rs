//! Jin CLI entry point

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = jin::cli::Cli::parse();
    jin::run(cli)
}
