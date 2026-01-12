# Clap Derive API Research for Rust CLI Applications

## Overview

This research document covers the clap derive API for creating Rust CLI applications with focus on command hierarchies, nested subcommands, and argument parsing best practices.

---

## 1. Core Derive Macros

### `#[derive(Parser)]`
The main derive macro used to create CLI interfaces declaratively.

**Basic Usage:**
```rust
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // Fields become arguments
    name: Option<String>,

    // Optional arguments with flags
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    // Counting flag
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    // Subcommand field
    #[command(subcommand)]
    command: Option<Commands>,
}
```

### `#[derive(Subcommand)]`
Used for defining subcommands in your CLI:
```rust
#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },

    /// Another subcommand with its own subcommands
    Mode(ModeCommands),
}
```

---

## 2. Creating Command Hierarchies

### Basic Pattern for Nested Subcommands

To create nested subcommands like `jin mode create`, `jin mode use`, etc.:

```rust
use clap::{Parser, Subcommand};

// Top-level CLI
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// First level subcommands
#[derive(Subcommand)]
enum Commands {
    /// Manage modes
    Mode(ModeArgs),

    /// Other top-level commands
    Other { name: String },
}

// Second level subcommands (nested under Mode)
#[derive(Args)]
#[command(version, about, long_about = None)]
struct ModeArgs {
    #[command(subcommand)]
    command: ModeCommands,
}

// Third level subcommands (nested under Mode <command>)
#[derive(Subcommand)]
enum ModeCommands {
    /// Create a new mode
    Create {
        /// Mode name
        name: String,

        /// Optional description
        #[arg(long)]
        description: Option<String>,
    },

    /// Use an existing mode
    Use {
        /// Mode name to activate
        name: String,

        /// Force activation
        #[arg(long)]
        force: bool,
    },

    /// List all modes
    List {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Mode(mode_args) => {
            match &mode_args.command {
                ModeCommands::Create { name, description } => {
                    println!("Creating mode: {}", name);
                    if let Some(desc) = description {
                        println!("Description: {}", desc);
                    }
                },
                ModeCommands::Use { name, force } => {
                    println!("Using mode: {}", name);
                    if *force {
                        println!("Forced activation");
                    }
                },
                ModeCommands::List { verbose } => {
                    if *verbose {
                        println!("Listing modes with details...");
                    } else {
                        println!("Listing modes...");
                    }
                },
            }
        },
        Commands::Other { name } => {
            println!("Other command with name: {}", name);
        },
    }
}
```

---

## 3. Command Argument Parsing Best Practices

### 3.1 Argument Types

#### Positional Arguments
```rust
#[derive(Parser)]
struct Cli {
    // Required positional argument
    name: String,

    // Optional positional argument
    optional_name: Option<String>,

    // Multiple positional arguments
    files: Vec<String>,
}
```

#### Options and Flags
```rust
#[derive(Parser)]
struct Cli {
    // Short and long flags
    #[arg(short, long)]
    verbose: bool,

    // Options with values
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,

    // Multiple values
    #[arg(short = 'f', long)]
    files: Vec<PathBuf>,

    // Counting flag
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}
```

#### Default Values
```rust
#[derive(Parser)]
struct Cli {
    // With default value
    #[arg(default_value_t = 8080)]
    port: u16,

    // From environment variable
    #[arg(env = "MY_CONFIG")]
    config: PathBuf,
}
```

### 3.2 Validation and Constraints

#### Enum Values
```rust
#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Fast,
    Slow,
}
```

#### Custom Validation
```rust
#[derive(Parser)]
struct Cli {
    #[arg(value_parser = port_in_range)]
    port: u16,
}

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse()
        .map_err(|_| format!("`{}` isn't a port number", s))?;
    if (1..=65535).contains(&port) {
        Ok(port)
    } else {
        Err("port not in range 1-65535".to_string())
    }
}
```

### 3.3 Argument Relations

#### Required Arguments
```rust
#[derive(Parser)]
struct Cli {
    #[arg(long)]
    config: Option<String>,

    #[arg(short, requires = "config")]
    output: Option<String>,
}
```

#### Exclusive Arguments
```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, group = "input")]
    file: Option<String>,

    #[arg(long, group = "input")]
    stdin: bool,
}
```

---

## 4. Key Documentation URLs

### Official Documentation
- **[clap::_derive::_tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)** - Main derive tutorial
- **[clap::_derive](https://docs.rs/clap/latest/clap/_derive/index.html)** - Derive API reference
- **[clap::_derive::_cookbook::git_derive](https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html)** - Git-like CLI example
- **[clap::Command](https://docs.rs/clap/latest/clap/builder/struct.Command.html)** - Command builder reference

### Community Resources
- **[Nested subcommands in Rust with clap](https://dev.to/drazisil/nested-subcommands-in-rest-with-clap-4n5m)** - Practical nested subcommands example
- **[Clap Discussions #2945](https://github.com/clap-rs/clap/discussions/2945)** - Parse nested enums with clap_derive
- **[Clap Discussions #3695](https://github.com/clap-rs/clap/discussions/3695)** - Nested commands help discussion
- **[StackOverflow: Global options with subcommands](https://stackoverflow.com/questions/74987368/how-to-use-global-option-when-subcommands-are-defined-when-using-clap-in-rust)** - Global options pattern

---

## 5. Advanced Patterns

### 5.1 Propagating Version
```rust
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

### 5.2 Custom Help and Version
```rust
#[derive(Parser)]
#[command(
    version = "MyApp 1.0",
    about = "Does awesome things",
    long_about = None
)]
struct Cli {
    // Custom next-line help format
    #[command(next_line_help = true)]
    config: PathBuf,
}
```

### 5.3 Testing CLI Applications
```rust
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
```

---

## 6. Migration from clap v3 to v4

Key differences:
1. **Derive API is stable** in v4
2. **Builder API has breaking changes**
3. **`structopt` is deprecated** in favor of clap derive
4. **`ValueEnum`** replaces custom enum parsing
5. **`ArgAction`** replaces action attributes

For migration guides, see:
- [clap CHANGELOG](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#next-steps)
- [clap-rs/clap discussions](https://github.com/clap-rs/clap/discussions)

---

## 7. Implementation Pattern for "jin" CLI

Based on the research, here's the recommended pattern for implementing a CLI like `jin`:

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "jin")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and manage Jin projects
    Project(ProjectArgs),
    /// Manage Jin modes and environments
    Mode(ModeArgs),
    // ... other top-level commands
}

#[derive(Args)]
#[command(about = "Project management commands")]
struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommands,
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Create a new Jin project
    Create {
        /// Project name
        name: String,

        /// Project template
        #[arg(long)]
        template: Option<String>,
    },
    /// Initialize project in current directory
    Init {
        /// Skip git initialization
        #[arg(long)]
        no_git: bool,
    },
}

#[derive(Args)]
#[command(about = "Mode management commands")]
struct ModeArgs {
    #[command(subcommand)]
    command: ModeCommands,
}

#[derive(Subcommand)]
enum ModeCommands {
    /// Create a new mode
    Create {
        /// Mode name
        name: String,

        /// Base mode to inherit from
        #[arg(long)]
        from: Option<String>,
    },
    /// Use/activate a mode
    Use {
        /// Mode name
        name: String,

        /// Create if it doesn't exist
        #[arg(long)]
        create: bool,
    },
    /// List available modes
    List {
        /// Show mode details
        #[arg(long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Project(project_args) => {
            match project_args.command {
                ProjectCommands::Create { name, template } => {
                    // Implementation for create
                },
                ProjectCommands::Init { no_git } => {
                    // Implementation for init
                },
            }
        },
        Commands::Mode(mode_args) => {
            match mode_args.command {
                ModeCommands::Create { name, from } => {
                    // Implementation for mode create
                },
                ModeCommands::Use { name, create } => {
                    // Implementation for mode use
                },
                ModeCommands::List { verbose } => {
                    // Implementation for mode list
                },
            }
        },
    }
}
```

This pattern provides a clean hierarchy that supports commands like:
- `jin project create my-project`
- `jin project init --no-git`
- `jin mode create my-mode --from base`
- `jin mode use my-mode --create`
- `jin mode list --verbose`