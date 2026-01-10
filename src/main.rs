//! Jin CLI entry point

#[cfg(unix)]
extern crate libc;

use clap::Parser;

#[cfg(unix)]
fn reset_sigpipe() {
    // SAFETY: This is safe because:
    // - SIGPIPE is a valid signal number on all Unix systems
    // - SIG_DFL is a valid handler constant
    // - The call has no other side effects that depend on signal state
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {
    // SIGPIPE doesn't exist on non-Unix platforms
    // Windows handles broken pipes differently via error codes
}

fn main() -> anyhow::Result<()> {
    // Reset SIGPIPE BEFORE any other initialization
    // This must be called before CLI parsing to catch all stdout writes
    reset_sigpipe();

    let cli = jin::cli::Cli::parse();
    jin::run(cli)
}
