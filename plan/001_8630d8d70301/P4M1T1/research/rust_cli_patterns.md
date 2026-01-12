# Rust CLI Patterns Research

This document researches Rust CLI projects that use clap for inspiration and best practices, focusing on command organization, dispatching, error handling, subcommands, testing, and naming conventions.

## Key Resources

### Foundational Resources
1. **[CLI Structure in Rust - Kevin K.](https://kbknapp.dev/cli-structure-01/)**
   - Comprehensive guide on subcommand-based CLI structure
   - Written by a core clap contributor
   - Covers most interesting and complicated patterns

2. **[Rust CLI Recommendations - Handling arguments and subcommands](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html)**
   - Community-curated best practices
   - Comprehensive guide on argument and subcommand handling

3. **[Building CLI Tools with clap and structopt](https://dev.to/sgchris/building-cli-tools-with-clap-and-structopt-62j)**
   - Practical guide with code examples
   - Modern argument parsing patterns

4. **[This Pattern Made My Rust CLI Tool 10x Easier to Maintain](https://medium.com/@syntaxSavage/this-pattern-made-my-rust-cli-tool-10x-easier-to-maintain-da67817175cf)**
   - Modular CLI structure patterns
   - Integration of clap, anyhow, and thiserror

## 1. Command Organization Patterns

### Module Structure
Most Rust CLI tools follow a consistent modular pattern:

```
src/
├── main.rs          # Entry point with app definition
├── cli/             # CLI argument parsing modules
│   ├── mod.rs
│   ├── app.rs      # clap App configuration
│   └── command.rs  # Command enum and dispatching
├── commands/        # Business logic for each command
│   ├── mod.rs
│   ├── cmd_add.rs
│   ├── cmd_build.rs
│   └── ...
├── error.rs         # Custom error types
└── lib.rs           # Core library functionality
```

### Example from clap's git.rs example
```rust
// src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[arg(short, long)]
        message: String,
    },
    Commit {
        #[arg(short, long)]
        message: String,
    },
    Push,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { message } => {
            // handle add command
        }
        Commands::Commit { message } => {
            // handle commit command
        }
        Commands::Push => {
            // handle push command
        }
    }
}
```

## 2. Command Dispatcher Patterns

### Pattern 1: Enum-based Dispatch
```rust
// src/command.rs
#[derive(Subcommand)]
enum Commands {
    #[command(about = "Add files to staging")]
    Add {
        #[arg(short, long)]
        verbose: bool,
        files: Vec<String>,
    },
    #[command(about = "Show status")]
    Status {
        #[arg(long)]
        porcelain: bool,
    },
}

// src/main.rs
fn dispatch_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Add { verbose, files } => {
            add::execute(files, verbose)?;
        }
        Commands::Status { porcelain } => {
            status::execute(porcelain)?;
        }
    }
    Ok(())
}
```

### Pattern 2: Trait-based Dispatch
```rust
// src/commands/mod.rs
pub trait Command {
    fn execute(&self) -> Result<()>;
}

pub struct AddCommand {
    files: Vec<String>,
    verbose: bool,
}

impl Command for AddCommand {
    fn execute(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}

// src/main.rs
fn dispatch_command(cmd: Box<dyn Command>) -> Result<()> {
    cmd.execute()
}
```

### Pattern 3: Module-based Organization
```rust
// Each command in its own module
mod commands {
    pub mod add;
    pub mod status;
    pub mod commit;

    // Command dispatcher
    pub fn dispatch(cmd: &str, args: &[String]) -> Result<()> {
        match cmd {
            "add" => add::execute(args),
            "status" => status::execute(args),
            "commit" => commit::execute(args),
            _ => Err(anyhow!("Unknown command: {}", cmd)),
        }
    }
}
```

## 3. Error Handling Patterns

### Best Practice: Combine anyhow and thiserror
```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Command failed: {0}")]
    Command(String),
}

// src/main.rs
use anyhow::{Context, Result};

fn main() -> Result<()> {
    // Use anyhow for application-level error handling
    let config = load_config().context("Failed to load configuration")?;

    // Use specific error types where needed
    process_files(&config).map_err(|e| AppError::Command(e.to_string()))?;

    Ok(())
}
```

### Error Handling in Commands
```rust
// src/commands/add.rs
use anyhow::{Context, Result};
use crate::error::AppError;

pub fn execute(files: Vec<String>, verbose: bool) -> Result<()> {
    for file in files {
        std::fs::metadata(&file)
            .with_context(|| format!("Failed to access file: {}", file))?;

        if verbose {
            println!("Adding: {}", file);
        }
        // ... add file logic
    }
    Ok(())
}
```

## 4. Nested Subcommands (Cargo-style)

### Pattern 1: Nested Enums
```rust
#[derive(Subcommand)]
enum Commands {
    #[command(about = "Project management")]
    Project {
        #[command(subcommand)]
        cmd: ProjectCommands,
    },
    #[command(about = "Dependency management")]
    Deps {
        #[command(subcommand)]
        cmd: DepsCommands,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    Init {
        #[arg(long)]
        name: String,
    },
    Build {
        #[arg(long)]
        release: bool,
    },
}

#[derive(Subcommand)]
enum DepsCommands {
    Add {
        #[arg(short, long)]
        name: String,
    },
    Remove {
        #[arg(short, long)]
        name: String,
    },
}
```

### Pattern 2: Using clap-nested-commands
```rust
// For complex nested structures
use clap_nested::{Command, derive_clap};

#[derive_clap]
pub struct ProjectAddArgs {
    #[clap(short, long)]
    pub name: String,
}

pub fn project_add_command() -> Command<ProjectAddArgs> {
    Command::new("add")
        .description("Add a new project")
        .args_override_self(true)
        .clap_command_derive()
        .run(|_args, matches| {
            // Handle project add
        })
}
```

### Real Example: Cargo-like Structure
```rust
// Simulating cargo build --list
#[derive(Parser)]
#[command(about = "Rust's package manager")]
struct Cli {
    #[command(subcommand)]
    command: CargoCommand,
}

#[derive(Subcommand)]
enum CargoCommand {
    #[command(about = "Compile the current package")]
    Build {
        #[arg(long)]
        list: bool,  // cargo build --list
        #[arg(long)]
        release: bool,
    },
    #[command(about = "Run tests")]
    Test {
        #[arg(long)]
        list: bool,  // cargo test --list
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        CargoCommand::Build { list, release } => {
            if list {
                println!("Available targets:");
                // List available build targets
            } else {
                // Build logic
            }
        }
        CargoCommand::Test { list } => {
            if list {
                println!("Available tests:");
                // List available tests
            } else {
                // Test logic
            }
        }
    }
}
```

## 5. Naming Conventions

### Command Names
- Use kebab-case for top-level commands: `git status`, `cargo build`
- Use snake_case for internal variables: `command_status`, `build_release`
- Use PascalCase for structs and enums: `Cli`, `Commands`, `AddCommand`

### Argument Names
- Short flags: single character, lowercase: `-v`, `-h`, `-f`
- Long flags: lowercase with hyphens: `--verbose`, `--help`, `--file`
- Required arguments: no special prefix
- Optional arguments: start with `--` or `-`

### File Organization
- `main.rs` - Entry point with app definition
- `cli.rs` or `cli/mod.rs` - CLI argument parsing
- `command.rs` - Command enum and dispatching
- `commands/` - Business logic modules
- `error.rs` - Error types
- `lib.rs` - Core library functionality

## 6. Testing Patterns

### Unit Testing Commands
```rust
// tests/commands_test.rs
use anyhow::Result;
use jin::commands;

#[test]
fn test_add_command() -> Result<()> {
    // Test the add command with mock data
    let result = commands::add::execute(
        vec!["test.txt".to_string()],
        false
    )?;

    assert!(result.is_ok());
    Ok(())
}
```

### Integration Testing
```rust
// tests/integration_test.rs
use assert_cmd::Command;
use std::process::Command as StdCommand;

#[test]
fn test_cli_help() {
    let mut cmd = StdCommand::cargo_bin("jin")
        .expect("Unable to find jin binary");

    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_add_subcommand() {
    let mut cmd = StdCommand::cargo_bin("jin")
        .expect("Unable to find jin binary");

    cmd.arg("add").arg("test.txt");
    cmd.assert().success();
}
```

### Testing with Mock Data
```rust
// tests/common/mod.rs
pub fn setup_test_files() {
    // Create test files for testing
    std::fs::write("test.txt", "test content").unwrap();
}

pub fn cleanup_test_files() {
    // Clean up test files
    let _ = std::fs::remove_file("test.txt");
}
```

### Advanced Testing with rexpect (for interactive CLIs)
```rust
// tests/interactive_test.rs
use rexpect::session::Session;

#[test]
fn test_interactive_mode() {
    let mut p = Session::spawn("cargo run", Some(10000))
        .expect("Failed to spawn process");

    p.send_line("interactive")?;
    p.exp_string("Interactive mode enabled")?;

    p.send_line("exit")?;
    p.exp_eof()?;
}
```

## Real Project Examples

### ripgrep (rg)
- **Repository**: https://github.com/BurntSushi/ripgrep
- **Pattern**: Single command with many flags
- **Structure**: Uses clap for argument parsing, single Args struct
- **Key Feature**: Performance-optimized regex search

### bat
- **Repository**: https://github.com/sharkdp/bat
- **Pattern**: Single command with subcommands (`bat cache --build`)
- **Structure**: Modular design with separate modules for syntax highlighting, paging
- **Key Feature**: cat replacement with syntax highlighting

### sd (sed replacement)
- **Repository**: https://github.com/chazmcgarvey/sd
- **Pattern**: Single command focused interface
- **Structure**: Simple, focused design
- **Key Feature**: Intuitive find & replace

### fd (find replacement)
- **Repository**: https://github.com/sharkdp/fd
- **Pattern**: Single command with extensive options
- **Structure**: Clean, minimal interface
- **Key Feature**: User-friendly alternative to find

## Best Practices Summary

### 1. Command Organization
- Use separate modules for each command
- Keep CLI parsing separate from business logic
- Use enums for subcommand dispatching
- Follow consistent naming conventions

### 2. Error Handling
- Use `anyhow` for application code
- Use `thiserror` for library code
- Provide context-rich error messages
- Handle errors gracefully at boundaries

### 3. Subcommands
- Use clap's derive macros for simple cases
- Consider nested enums for cargo-style subcommands
- Use separate modules for complex command hierarchies
- Provide help and examples for each subcommand

### 4. Testing
- Unit test business logic
- Integration test the CLI interface
- Use assert_cmd for CLI testing
- Mock external dependencies
- Test both success and error cases

### 5. Performance
- Parse arguments first, then execute
- Minimize allocations in hot paths
- Use appropriate data structures
- Profile and optimize critical paths

## Recommended Tool Stack

1. **clap** - Command line argument parsing
2. **anyhow** - Error handling for application code
3. **thiserror** - Error types for library code
4. **assert_cmd** - CLI integration testing
5. **rexpect** - Interactive CLI testing
6. **log + env_logger** - Logging with verbosity levels
7. **serde + serde_json** - Configuration file handling

This research provides a comprehensive foundation for building well-structured, maintainable Rust CLI applications using clap and following industry best practices.