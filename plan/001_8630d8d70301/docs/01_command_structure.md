# Research: Jin CLI Command Structure

## Overview
The Jin codebase uses **Clap 4.5** with the derive API for command-line parsing.

## Key Files

### CLI Entry Points
- `src/main.rs` - Main entry point
- `src/lib.rs` - Command router
- `src/cli/mod.rs` - CLI definition and Commands enum
- `src/cli/args.rs` - Shared argument types
- `src/commands/mod.rs` - Command dispatcher

### Command Module Location
All commands are in `src/commands/` directory, each as a separate module.

## Command Definition Pattern

```rust
// In src/cli/mod.rs
#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Command description
    CommandName(CommandArgs),
}

// In src/cli/args.rs
#[derive(Args, Debug)]
pub struct CommandArgs {
    #[arg(long)]
    pub flag: bool,
}
```

## Command Execution Flow

```
main.rs
  └─> jin::run(cli)
        └─> commands::execute(cli)
              └─> command::execute(args)
```

## Existing Commands Reference

### Simple Command (no args)
- `context` - Show/set active context (`src/commands/context.rs`)
- `status` - Show workspace state (`src/commands/status.rs`)
- `fetch` - Fetch updates (`src/commands/fetch.rs`)

### Complex Command (with args)
- `apply` - Apply merged layers (`src/commands/apply.rs`)
- `reset` - Reset staged changes (`src/commands/reset.rs`)

### Subcommand Pattern
- `mode` - Mode lifecycle (`src/commands/mode.rs`)
  - create, use, list, delete, show, unset
- `scope` - Scope lifecycle (`src/commands/scope.rs`)

## Common Implementation Patterns

### 1. Command Execute Pattern
```rust
pub fn execute(args: CommandArgs) -> Result<()> {
    // Check initialization
    if !ProjectContext::is_initialized() {
        return Err(JinError::NotInitialized);
    }

    // Load context
    let context = ProjectContext::load()?;

    // Do work...
    Ok(())
}
```

### 2. Error Handling
```rust
// Use JinError types
JinError::NotInitialized
JinError::Config(String)
JinError::NotFound(String)
```

### 3. Context Loading
```rust
let context = ProjectContext::load()?;
let repo = JinRepo::open_or_create()?;
```

## Dependencies
```toml
clap = { version = "4.5", features = ["derive", "cargo"] }
clap_complete = "4.5"
```

## Sources
- `/home/dustin/projects/jin/src/cli/mod.rs`
- `/home/dustin/projects/jin/src/cli/args.rs`
- `/home/dustin/projects/jin/src/commands/mod.rs`
- `/home/dustin/projects/jin/src/commands/mode.rs`
- `/home/dustin/projects/jin/src/commands/context.rs`
