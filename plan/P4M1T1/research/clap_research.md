# Clap v4 Research for Jin CLI Implementation

## Overview

This document compiles research on clap v4 derive API patterns, specifically for implementing the complex multi-level command structure required by the Jin project.

## Clap v4 Derive API Reference

### Official Documentation

- **Main Documentation**: https://docs.rs/clap/4.5/clap/
- **Derive API**: https://docs.rs/clap/4.5/clap/_derive/index.html
- **Attribute Reference**: https://docs.rs/clap/4.5/clap/derive/attrs/index.html
- **Examples**: https://github.com/clap-rs/clap/tree/master/examples/derive_ref

## Key Derive Macros

### Parser Trait

Used for the top-level command parser:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "jin")]
#[command(about = "Multi-layer Git overlay system", long_about = "...")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

**Key Attributes**:
- `#[command(name = "...")]` - Set command name (defaults to struct name lowercase)
- `#[command(about = "...")]` - Short description for help
- `#[command(long_about = "...")]` - Long description for `--help`
- `#[command(version)]` - Enable version flag from Cargo.toml
- `#[command(subcommand)]` - Mark field as subcommand dispatcher

### Subcommand Trait

Used for defining command hierarchies:

```rust
use clap::Subcommand;

#[derive(Subcommand)]
enum Commands {
    Init(InitCommand),
    Add(AddCommand),
    #[command(subcommand)]
    Mode(ModeCommand),
}
```

## Multi-Level Subcommand Pattern

### Two-Level Commands (mode, scope)

```rust
#[derive(Subcommand)]
pub enum ModeCommand {
    Create {
        #[arg(value_name = "MODE")]
        name: String,
    },
    Use {
        #[arg(value_name = "MODE")]
        name: String,
    },
    Unset,
    Delete {
        #[arg(value_name = "MODE")]
        name: String,
    },
    Show,
}

// Usage:
// jin mode create claude
// jin mode use claude
// jin mode unset
```

### Value Arguments

```rust
// Single value
pub struct InitCommand;

// Multiple values (one or more)
pub struct AddCommand {
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,
}

// Optional value
pub struct DiffCommand {
    #[arg(value_name = "LAYER1")]
    pub layer1: Option<String>,
}
```

### Flag Arguments

```rust
pub struct AddCommand {
    // Boolean flag
    #[arg(long)]
    pub mode: bool,

    // Flag with value
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,
}
```

## Argument Attributes Reference

### Positional Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `value_name = "NAME"` | Placeholder in help text | `#[arg(value_name = "FILE")]` |
| `num_args(1..)` | One or more values | `#[arg(value_name = "FILE", num_args(1..))]` |
| `num_args(0..)` | Zero or more values | `#[arg(value_name = "PATH", num_args(0..))]` |

### Option Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `long` | Enable long flag | `#[arg(long)]` |
| `short = 'x'` | Enable short flag | `#[arg(short = 'f')]` |
| `value_name = "NAME"` | Value placeholder | `#[arg(long, value_name = "MODE")]` |
| `required = true` | Make flag required | `#[arg(long, required = true)]` |
| `default_value = "x"` | Default value | `#[arg(long, default_value = "false")]` |
| `help = "text"` | Short help | `#[arg(long, help = "Enable feature")]` |
| `long_help = "text"` | Long help | `#[arg(long, long_help = "Detailed...")]` |
| `conflicts_with = "x"` | Mutual exclusion | `#[arg(long, conflicts_with = "soft")]` |

## Common Patterns

### Layer Routing Flags

For Jin's layer routing system:

```rust
pub struct AddCommand {
    #[arg(long)]
    pub mode: bool,

    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    #[arg(long)]
    pub project: bool,

    #[arg(long)]
    pub global: bool,
}
```

### Conflicting Flags

```rust
pub struct ResetCommand {
    #[arg(long)]
    pub soft: bool,

    #[arg(long, conflicts_with = "soft", conflicts_with = "hard")]
    pub mixed: bool,

    #[arg(long, conflicts_with = "soft")]
    pub hard: bool,
}
```

### Optional Arguments with Defaults

```rust
pub struct LogCommand {
    #[arg(value_name = "LAYER")]
    pub layer: Option<String>,

    #[arg(long, value_name = "N")]
    pub count: Option<usize>,
}
```

## Command Dispatch Pattern

### Main Entry Point

```rust
use jin_glm::cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => match cli.command {
            Commands::Init(_) => {
                println!("jin init - not implemented");
                ExitCode::SUCCESS
            }
            Commands::Add(cmd) => {
                println!("Adding {:?} files", cmd.files);
                ExitCode::SUCCESS
            }
            Commands::Mode(ModeCommand::Create { name }) => {
                println!("Creating mode: {name}");
                ExitCode::SUCCESS
            }
            _ => {
                println!("Command not yet implemented");
                ExitCode::SUCCESS
            }
        },
        Err(e) => {
            eprint!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
```

## Testing Patterns

### Unit Tests for CLI Parsing

```rust
use clap::CommandFactory;

#[test]
fn test_cli_basic_parsing() {
    // Verify CLI can be parsed
    Cli::command().debug_assert();
}

#[test]
fn test_add_command_with_files() {
    let cli = Cli::try_parse_from(["jin", "add", "file1.txt", "file2.txt"]).unwrap();
    match cli.command {
        Commands::Add(cmd) => {
            assert_eq!(cmd.files.len(), 2);
            assert_eq!(cmd.files[0], PathBuf::from("file1.txt"));
        }
        _ => panic!("Expected Add command"),
    }
}

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
fn test_scope_create_with_mode() {
    let cli = Cli::try_parse_from(["jin", "scope", "create", "python", "--mode", "claude"]).unwrap();
    match cli.command {
        Commands::Scope(ScopeCommand::Create { name, mode }) => {
            assert_eq!(name, "python");
            assert_eq!(mode.as_deref(), Some("claude"));
        }
        _ => panic!("Expected Scope::Create"),
    }
}
```

## Examples from Open Source Projects

### ripgrep

- **Repo**: https://github.com/BurntSushi/ripgrep
- **File**: `src/cli.rs`
- **Patterns to note**: Clean separation of CLI args, extensive use of derive

### bat

- **Repo**: https://github.com/sharkdp/bat
- **File**: `src/args.rs`
- **Patterns to note**: Good help text organization, many subcommands

### git-delta

- **Repo**: https://github.com/dandavison/delta
- **File**: `src/cli/mod.rs`
- **Patterns to note**: Complex flag combinations, good use of value_name

## Gotchas and Best Practices

### DO: Use `try_parse()` for better error handling

```rust
match Cli::try_parse() {
    Ok(cli) => { /* handle */ }
    Err(e) => {
        eprint!("Error: {e}");
        return ExitCode::FAILURE;
    }
}
```

### DON'T: Use `parse()` if you want custom error handling

```rust
// This exits on error - can't customize
let cli = Cli::parse();
```

### DO: Provide `value_name` for all value arguments

```rust
#[arg(long, value_name = "SCOPE")]
scope: Option<String>,
```

### DON'T: Skip help text

```rust
// Bad: No help
#[arg(long)]
mode: bool,

// Good: Clear help
#[arg(long, help = "Route to mode base layer")]
mode: bool,
```

### DO: Use `conflicts_with` for mutually exclusive flags

```rust
#[arg(long, conflicts_with = "soft")]
hard: bool,
```

### DON'T: Forget `num_args` for multiple values

```rust
// Bad: Won't parse correctly
files: Vec<PathBuf>,

// Good: Parses one or more files
#[arg(value_name = "FILE", num_args(1..))]
files: Vec<PathBuf>,
```

## Clap Version in Jin

From `Cargo.toml`:
```toml
clap = { version = "4.5", features = ["derive"] }
```

**Key Points**:
- Version 4.5 is the latest stable (as of PRP creation)
- `derive` feature is required for all derive macros
- No other features needed for basic CLI

## Summary

This research provides:
1. Complete clap v4 derive API reference
2. Multi-level subcommand patterns
3. Common argument patterns for Jin's requirements
4. Testing patterns for validation
5. Best practices and gotchas to avoid

Use this as a reference when implementing `src/cli/args.rs`.
