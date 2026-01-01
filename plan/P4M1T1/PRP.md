# PRP: P4.M1.T1 - CLI Structure Definition

---

## Goal

**Feature Goal**: Establish the complete clap-based CLI structure for Jin, defining all commands and subcommands with proper argument parsing, enabling the entire CLI framework to be used by subsequent command implementations.

**Deliverable**:
1. `src/cli/mod.rs` - Main `Cli` struct with `Parser` derive, `Commands` enum with all 22 command variants
2. `src/cli/args.rs` - All argument structs using `Args` derive for commands with parameters
3. `src/commands/mod.rs` - Command dispatcher pattern with match statement routing
4. `src/main.rs` - Entry point that parses CLI and calls library run function
5. Integration tests validating all commands are parseable
6. Unit tests verifying clap configuration

**Success Definition**:
- All 22 Jin commands parseable via clap derive API
- `jin --help` displays all commands with descriptions
- `jin <command> --help` shows command-specific help for each command
- `jin mode create <name>` and `jin scope create <name>` nested subcommands work
- `cargo build` succeeds with zero errors
- `cargo test --test cli_basic` validates all command parsing

---

## User Persona

**Target User**: Developer using Jin to manage tool-specific configuration files across different modes (development environments), scopes (contexts within modes), and projects.

**Use Case**: A developer runs `jin --help` to discover available commands, then runs specific commands like `jin mode create dev`, `jin add .vscode/settings.json --mode`, or `jin commit -m "Add VS Code settings"` to manage their configuration layers.

**User Journey**:
1. Developer installs Jin via `cargo install jin`
2. Developer runs `jin --help` to see all available commands
3. Developer runs `jin init` to initialize Jin in their project
4. Developer runs `jin mode create dev` to create a development mode
5. Developer runs `jin mode use dev` to activate the mode
6. Developer runs `jin add .vscode/settings.json --mode` to stage config files
7. Developer runs `jin commit -m "Add VS Code settings"` to commit staged files
8. Developer runs `jin status` to see current state

**Pain Points Addressed**:
- Intuitive Git-like command structure for familiarity
- Comprehensive help text for all commands via `--help`
- Clear error messages for invalid input
- Tab completion support (future P6.M1)

---

## Why

- **Foundation for All Commands**: P4.M2+ depends on CLI framework being complete
- **User-Facing Interface**: CLI is the primary way users interact with Jin
- **PRD Requirement**: Section 18 of PRD defines all required commands
- **Consistency**: Using clap derive ensures consistent argument parsing across all commands
- **Maintainability**: Structured CLI makes adding new commands straightforward
- **Developer Experience**: Good error messages and help text improve usability

---

## What

### User-Visible Behavior

The CLI provides access to all 22 Jin commands organized into logical categories:

#### Initialization Commands (3)
```bash
jin init                    # Initialize Jin in current project
jin link <url>              # Link to shared Jin config repo
jin completion <shell>      # Generate shell completion scripts
```

#### Staging & Committing (2)
```bash
jin add <files> [flags]     # Stage files to appropriate layer
jin commit -m "message"     # Commit staged files atomically
```

#### Status & Inspection (5)
```bash
jin status                  # Show workspace state and active contexts
jin diff [layer1] [layer2]  # Show differences between layers
jin log [--layer] [--count] # Show commit history
jin layers                  # Show current layer composition
jin list                    # List available modes/scopes/projects
```

#### Mode Management (6 subcommands)
```bash
jin mode create <name>      # Create a new mode
jin mode use <name>         # Activate a mode
jin mode list               # List available modes
jin mode delete <name>      # Delete a mode
jin mode show               # Show current mode
jin mode unset              # Deactivate current mode
```

#### Scope Management (6 subcommands)
```bash
jin scope create <name> [--mode=<mode>]  # Create a new scope
jin scope use <name>                     # Activate a scope
jin scope list                           # List available scopes
jin scope delete <name>                  # Delete a scope
jin scope show                           # Show current scope
jin scope unset                          # Deactivate current scope
```

#### Workspace Operations (2)
```bash
jin apply [--force] [--dry-run]  # Apply merged layers to workspace
jin reset [--soft|--mixed|--hard] [flags]  # Reset staged/committed changes
```

#### Synchronization (4)
```bash
jin fetch                    # Fetch updates from remote
jin pull                     # Fetch and merge updates
jin push [--force]           # Push local changes
jin sync                     # Fetch + merge + apply
```

#### Data Management (2)
```bash
jin import <files> [--force] # Import Git-tracked files into Jin
jin export <files>           # Export Jin files back to Git
```

#### Utility Commands (3)
```bash
jin context                  # Show/set active context
jin repair [--dry-run]       # Repair Jin state
```

### Technical Requirements

1. **Clap Derive API**: All structs must use `#[derive(Parser)]`, `#[derive(Subcommand)]`, `#[derive(Args)]`
2. **Command Enum**: Single `Commands` enum with 22 variants
3. **Argument Structs**: Dedicated `Args` struct for commands with arguments (AddArgs, CommitArgs, etc.)
4. **Nested Subcommands**: Mode and Scope have nested subcommand enums (ModeAction, ScopeAction)
5. **Command Dispatcher**: Pattern matching on Commands enum routes to implementations
6. **Error Integration**: Commands return `anyhow::Result<()>` for consistent error handling
7. **Help Text**: Doc comments automatically used as help text by clap

### Success Criteria

- [ ] 22 commands defined in Commands enum
- [ ] ModeAction enum with 6 subcommands: create, use, list, delete, show, unset
- [ ] ScopeAction enum with 6 subcommands: create, use, list, delete, show, unset
- [ ] All argument types defined with proper clap attributes
- [ ] Help text for all commands via doc comments
- [ ] Version flag works: `jin --version`
- [ ] Help flag works: `jin --help`
- [ ] Invalid commands produce errors
- [ ] Command dispatcher routes all commands to implementations
- [ ] Integration tests validate all command parsing

---

## All Needed Context

### Context Completeness Check

_If someone knew nothing about this codebase, would they have everything needed to implement this successfully?_

**YES**: This PRP provides:
1. Complete list of 22 commands with their arguments
2. clap derive API reference with specific patterns to follow
3. Existing codebase patterns showing exact structure to follow
4. Real Rust CLI project examples for inspiration
5. Step-by-step implementation tasks in dependency order
6. Validation procedures that match this codebase

### Documentation & References

```yaml
# CLAP DERIVE API - PRIMARY REFERENCE

- url: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
  why: Complete clap derive API tutorial with examples
  critical: |
    - #[derive(Parser)] for main CLI struct
    - #[derive(Subcommand)] for command enums
    - #[derive(Args)] for argument structs
    - Doc comments become help text automatically

- url: https://docs.rs/clap/latest/clap/_derive/index.html
  why: Clap derive API reference documentation
  critical: |
    - All available attributes for #[command()] and #[arg()]
    - Version propagation with propagate_version = true
    - Nested subcommand patterns

# EXISTING CODEBASE PATTERNS - MUST FOLLOW

- file: src/cli/mod.rs
  why: Shows exact CLI structure pattern to replicate
  pattern: |
    #[derive(Parser, Debug)]
    #[command(name = "jin")]
    #[command(author, version, about = "Phantom Git layer system for developer configuration")]
    #[command(propagate_version = true)]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Commands,
    }
  gotcha: |
    - propagate_version = true is required for version to work
    - Doc comment on struct becomes the main about text

- file: src/cli/args.rs
  why: Shows argument struct pattern
  pattern: |
    #[derive(Args, Debug)]
    pub struct AddArgs {
        pub files: Vec<String>,
        #[arg(long)]
        pub mode: bool,
        #[arg(long)]
        pub scope: Option<String>,
    }
  gotcha: |
    - Vec<T> allows zero or more values (optional positional)
    - Option<T> makes flag/value optional
    - bool creates --flag that defaults to false
    - #[arg(long)] creates --flag syntax

- file: src/commands/mod.rs
  why: Shows command dispatcher pattern
  pattern: |
    pub fn execute(cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Init => init::execute(),
            Commands::Add(args) => add::execute(args),
            Commands::Mode(action) => mode::execute(action),
        }
    }
  gotcha: |
    - Unit variants (Init) take no parameters
    - Struct variants (Add(args)) wrap their Args struct
    - Subcommand variants (Mode(action)) wrap their subcommand enum

- file: src/main.rs
  why: Shows entry point pattern
  pattern: |
    fn main() -> anyhow::Result<()> {
        let cli = jin::cli::Cli::parse();
        jin::run(cli)
    }
  gotcha: main() must return anyhow::Result<()> for clean error display

- file: src/lib.rs
  why: Shows library integration
  pattern: |
    pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
        commands::execute(cli).map_err(|e| anyhow::anyhow!("{}", e))
    }
  gotcha: Convert JinError to anyhow::Error for display

- file: src/commands/init.rs
  why: Example of simple command implementation
  pattern: |
    pub fn execute() -> Result<()> {
        if ProjectContext::is_initialized() {
            println!("Jin is already initialized in this directory");
            return Ok(());
        }
        // ... implementation
        println!("Initialized Jin in {}", jin_dir.display());
        Ok(())
    }
  gotcha: Always return Result<()>, use println! for output

- file: src/commands/mode.rs
  why: Example of nested subcommand implementation
  pattern: |
    pub fn execute(action: ModeAction) -> Result<()> {
        match action {
            ModeAction::Create { name } => create(&name),
            ModeAction::Use { name } => use_mode(&name),
            // ... other variants
        }
    }
  gotcha: Nested match to route subcommand to internal functions

- file: tests/cli_basic.rs
  why: Shows test patterns for CLI commands
  pattern: |
    #[test]
    fn test_init_subcommand() {
        jin().arg("init").assert().success();
    }
  gotcha: Use assert_cmd for CLI integration testing

# RUST CLI PATTERNS - INSPIRATION

- url: https://kbknapp.dev/cli-structure-01/
  why: Comprehensive guide on subcommand-based CLI structure
  critical: Written by clap contributor, authoritative patterns

- url: https://rust-cli-recommendations.sunshowers.io/handling-arguments.html
  why: Community-curated best practices for argument handling
  critical: Industry-standard patterns for Rust CLIs

# PROJECT STRUCTURE REFERENCE

- file: Cargo.toml
  why: Shows dependency configuration
  pattern: |
    [dependencies]
    clap = { version = "4.5", features = ["derive", "cargo"] }
    anyhow = "1.0"
    thiserror = "2.0"

    [dev-dependencies]
    assert_cmd = "2.0"
    predicates = "3.0"
    tempfile = "3.0"
  gotcha: derive feature is REQUIRED for clap derive API
```

### Current Codebase Tree

```bash
jin/
├── Cargo.toml                    # clap = "4.5" with derive feature
├── src/
│   ├── main.rs                   # Entry point (PARSE CLI, call run())
│   ├── lib.rs                    # Library root (run() function)
│   ├── cli/
│   │   ├── mod.rs                # TO CREATE: Cli, Commands, ModeAction, ScopeAction
│   │   └── args.rs               # TO CREATE: All argument structs
│   ├── commands/
│   │   ├── mod.rs                # TO CREATE: Command dispatcher
│   │   ├── init.rs               # EXISTS: Simple command example
│   │   ├── mode.rs               # EXISTS: Nested subcommand example
│   │   ├── status.rs             # EXISTS: Status command example
│   │   └── ...                   # Other commands (stubs or implementations)
│   ├── core/
│   │   ├── error.rs              # EXISTS: JinError enum
│   │   └── ...
│   └── ...
└── tests/
    ├── cli_basic.rs              # EXISTS: Integration tests
    └── ...
```

### Desired Codebase Tree (After Implementation)

```bash
jin/
├── Cargo.toml
├── src/
│   ├── main.rs                   # COMPLETE: Parse CLI, call run()
│   ├── lib.rs                    # COMPLETE: run() function
│   ├── cli/
│   │   ├── mod.rs                # COMPLETE: Cli, Commands (22), ModeAction (6), ScopeAction (6)
│   │   └── args.rs               # COMPLETE: 11 argument structs
│   ├── commands/
│   │   ├── mod.rs                # COMPLETE: execute() dispatcher with all 22 routes
│   │   ├── init.rs               # EXISTS: execute() function
│   │   ├── mode.rs               # EXISTS: execute(action) function
│   │   ├── status.rs             # EXISTS: execute() function
│   │   └── ...                   # Other commands (stubs OK for now)
│   └── ...
└── tests/
    └── cli_basic.rs              # COMPLETE: Tests for all 22 commands
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: clap derive feature requirement
// ============================================================
// Cargo.toml MUST have: clap = { version = "4.5", features = ["derive"] }
// Without "derive" feature, derive macros won't work!

// ============================================================
// PATTERN: Unit vs Struct variants in Commands enum
// ============================================================
// Commands with no arguments are unit variants
// Commands with arguments wrap an Args struct
// Commands with subcommands wrap a Subcommand enum
//
// CORRECT:
// Init,                              // No args - unit variant
// Add(AddArgs),                      // With args - struct variant
// Mode(ModeAction),                  // Subcommands - wraps Subcommand enum
//
// WRONG:
// Init { },                          // Don't use braces for unit variants
// Add { args: AddArgs },             // Don't wrap Args unnecessarily

// ============================================================
// PATTERN: Nested subcommands for Mode and Scope
// ============================================================
// Mode and Scope use #[command(subcommand)] attribute
// This tells clap these variants contain nested subcommands
//
// CORRECT:
// #[command(subcommand)]
// Mode(ModeAction),
//
// WRONG:
// Mode { action: ModeAction },       // Don't use struct for subcommands

// ============================================================
// GOTCHA: Short flags must not conflict
// ============================================================
// -m is used for commit message
// Don't use -m for other flags to avoid conflicts
//
// CORRECT:
// #[arg(short, long)]
// message: String,                   // -m, --message
//
// AVOID:
// #[arg(short = 'm')]               // Use #[arg(short, long)] instead

// ============================================================
// PATTERN: Optional vs Required arguments
// ============================================================
// Option<T> makes argument optional
// Vec<T> allows zero or more (optional positional)
// T alone is required
//
// EXAMPLES:
// files: Vec<String>,               // Zero or more files (optional positional)
// message: String,                  // Required (must be provided)
// scope: Option<String>,            // Optional flag value

// ============================================================
// GOTCHA: Boolean flags with clap derive
// ============================================================
// bool fields create flags that are false by default
// Use #[arg(long)] for --flag syntax
//
// CORRECT:
// #[arg(long)]
// pub force: bool,                  // --force flag, defaults to false

// ============================================================
// PATTERN: Default values for arguments
// ============================================================
// Use default_value or default_value_t for defaults
//
// EXAMPLE:
// #[arg(long, default_value = "10")]
// pub count: usize,                 // --count defaults to 10

// ============================================================
// GOTCHA: Positional vs Flag arguments
// ============================================================
// Positional arguments (no #[arg()]) come first
// Flag arguments (with #[arg(long)]) come after
//
// CORRECT:
// pub struct AddArgs {
//     pub files: Vec<String>,       // Positional, comes first
//     #[arg(long)]
//     pub mode: bool,                // Flag, comes after
// }
//
// WRONG:
// pub struct AddArgs {
//     #[arg(long)]
//     pub mode: bool,                // Flag before positional - confusing!
//     pub files: Vec<String>,        // Positional after flag - weird!
// }

// ============================================================
// PATTERN: Doc comments become help text
// ============================================================
// Use /// doc comments on commands and arguments
// These automatically become --help text
//
// CORRECT:
// /// Stage files to appropriate layer
// Add(AddArgs),
//
// /// Files to stage
// pub files: Vec<String>,

// ============================================================
// GOTCHA: Version propagation
// ============================================================
// MUST use #[command(propagate_version = true)] on main Cli struct
// Otherwise --version won't work on subcommands
//
// CORRECT:
// #[derive(Parser, Debug)]
// #[command(name = "jin")]
// #[command(author, version, about)]  // These come from Cargo.toml
// #[command(propagate_version = true)]  // CRITICAL: Enables --version
// pub struct Cli {
//     #[command(subcommand)]
//     pub command: Commands,
// }

// ============================================================
// PATTERN: Enum naming for subcommands
// ============================================================
// Subcommand enums use Action suffix
// ModeAction, ScopeAction, not ModeSubcommand, ScopeSubcommand
//
// CORRECT:
// #[derive(Subcommand, Debug)]
// pub enum ModeAction {
//     Create { name: String },
//     Use { name: String },
//     // ...
// }

// ============================================================
// GOTCHA: Command dispatcher exhaustiveness
// ============================================================
// Match statement in execute() must handle ALL Commands enum variants
// Rust will error if any variant is missing
//
// CORRECT:
// pub fn execute(cli: Cli) -> Result<()> {
//     match cli.command {
//         Commands::Init => init::execute(),
//         Commands::Add(args) => add::execute(args),
//         // ... ALL 22 variants must be handled
//     }
// }
```

---

## Implementation Blueprint

### Data Models and Structure

The CLI framework consists of these key types:

```rust
// ================== src/cli/mod.rs ==================

use clap::{Parser, Subcommand};
pub use args::*;

/// Jin - Phantom Git layer system for developer configuration
#[derive(Parser, Debug)]
#[command(name = "jin")]
#[command(author, version, about = "Phantom Git layer system for developer configuration")]
#[command(propagate_version = true)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available Jin commands (22 total)
#[derive(Subcommand, Debug)]
pub enum Commands {
    // Initialization (3)
    /// Initialize Jin in current project
    Init,

    /// Link to shared Jin config repo
    Link(LinkArgs),

    /// Generate shell completion scripts
    Completion {
        /// Shell type to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    // Staging & Committing (2)
    /// Stage files to appropriate layer
    Add(AddArgs),

    /// Commit staged files atomically
    Commit(CommitArgs),

    // Status & Inspection (5)
    /// Show workspace state and active contexts
    Status,

    /// Show differences between layers
    Diff(DiffArgs),

    /// Show commit history
    Log(LogArgs),

    /// Show current layer composition
    Layers,

    /// List available modes/scopes/projects
    List,

    /// Show/set active context
    Context,

    // Mode Management (nested subcommands)
    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),

    // Scope Management (nested subcommands)
    /// Scope lifecycle management
    #[command(subcommand)]
    Scope(ScopeAction),

    // Workspace Operations (2)
    /// Apply merged layers to workspace
    Apply(ApplyArgs),

    /// Reset staged or committed changes
    Reset(ResetArgs),

    // Synchronization (4)
    /// Fetch updates from remote
    Fetch,

    /// Fetch and merge updates
    Pull,

    /// Push local changes
    Push(PushArgs),

    /// Fetch + merge + apply
    Sync,

    // Data Management (2)
    /// Import Git-tracked files into Jin
    Import(ImportArgs),

    /// Export Jin files back to Git
    Export(ExportArgs),

    // Utility (1)
    /// Repair Jin state
    Repair(RepairArgs),
}

/// Mode subcommands (6 variants)
#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create {
        /// Name of the mode to create
        name: String,
    },

    /// Activate a mode
    Use {
        /// Name of the mode to activate
        name: String,
    },

    /// List available modes
    List,

    /// Delete a mode
    Delete {
        /// Name of the mode to delete
        name: String,
    },

    /// Show current mode
    Show,

    /// Deactivate current mode
    Unset,
}

/// Scope subcommands (6 variants)
#[derive(Subcommand, Debug)]
pub enum ScopeAction {
    /// Create a new scope
    Create {
        /// Name of the scope to create
        name: String,
        /// Associate with a mode
        #[arg(long)]
        mode: Option<String>,
    },

    /// Activate a scope
    Use {
        /// Name of the scope to activate
        name: String,
    },

    /// List available scopes
    List,

    /// Delete a scope
    Delete {
        /// Name of the scope to delete
        name: String,
    },

    /// Show current scope
    Show,

    /// Deactivate current scope
    Unset,
}

// ================== src/cli/args.rs ==================

use clap::Args;

/// Arguments for the `add` command
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Files to stage
    pub files: Vec<String>,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Target project layer
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,
}

/// Arguments for the `commit` command
#[derive(Args, Debug)]
pub struct CommitArgs {
    /// Commit message
    #[arg(short, long)]
    pub message: String,

    /// Dry run - show what would be committed
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `apply` command
#[derive(Args, Debug)]
pub struct ApplyArgs {
    /// Force apply even if workspace is dirty
    #[arg(long)]
    pub force: bool,

    /// Show what would be applied
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `reset` command
#[derive(Args, Debug)]
pub struct ResetArgs {
    /// Keep changes in staging
    #[arg(long)]
    pub soft: bool,

    /// Unstage but keep in workspace (default)
    #[arg(long)]
    pub mixed: bool,

    /// Discard all changes
    #[arg(long)]
    pub hard: bool,

    /// Reset mode layer
    #[arg(long)]
    pub mode: bool,

    /// Reset scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Reset project layer
    #[arg(long)]
    pub project: bool,
}

/// Arguments for the `diff` command
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// First layer to compare
    pub layer1: Option<String>,

    /// Second layer to compare
    pub layer2: Option<String>,

    /// Show staged changes
    #[arg(long)]
    pub staged: bool,
}

/// Arguments for the `log` command
#[derive(Args, Debug)]
pub struct LogArgs {
    /// Layer to show history for
    #[arg(long)]
    pub layer: Option<String>,

    /// Number of entries to show
    #[arg(long, default_value = "10")]
    pub count: usize,
}

/// Arguments for the `import` command
#[derive(Args, Debug)]
pub struct ImportArgs {
    /// Files to import from Git
    pub files: Vec<String>,

    /// Force import even if files are modified
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `export` command
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Files to export back to Git
    pub files: Vec<String>,
}

/// Arguments for the `repair` command
#[derive(Args, Debug)]
pub struct RepairArgs {
    /// Show what would be repaired
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `link` command
#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Remote repository URL
    pub url: String,

    /// Force update existing remote
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `push` command
#[derive(Args, Debug)]
pub struct PushArgs {
    /// Force push (overwrite remote)
    #[arg(long)]
    pub force: bool,
}

// ================== src/commands/mod.rs ==================

use crate::cli::{Cli, Commands};
use crate::core::Result;

pub mod init;
pub mod add;
pub mod commit_cmd;
pub mod status;
pub mod mode;
pub mod scope;
pub mod apply;
pub mod reset;
pub mod diff;
pub mod log;
pub mod context;
pub mod layers;
pub mod list;
pub mod link;
pub mod fetch;
pub mod pull;
pub mod push;
pub mod sync;
pub mod import_cmd;
pub mod export;
pub mod repair;
pub mod completion;

/// Execute the appropriate command based on CLI arguments
pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        // Initialization
        Commands::Init => init::execute(),
        Commands::Link(args) => link::execute(args),
        Commands::Completion { shell } => completion::execute(shell),

        // Staging & Committing
        Commands::Add(args) => add::execute(args),
        Commands::Commit(args) => commit_cmd::execute(args),

        // Status & Inspection
        Commands::Status => status::execute(),
        Commands::Diff(args) => diff::execute(args),
        Commands::Log(args) => log::execute(args),
        Commands::Layers => layers::execute(),
        Commands::List => list::execute(),
        Commands::Context => context::execute(),

        // Mode & Scope
        Commands::Mode(action) => mode::execute(action),
        Commands::Scope(action) => scope::execute(action),

        // Workspace Operations
        Commands::Apply(args) => apply::execute(args),
        Commands::Reset(args) => reset::execute(args),

        // Synchronization
        Commands::Fetch => fetch::execute(),
        Commands::Pull => pull::execute(),
        Commands::Push(args) => push::execute(args),
        Commands::Sync => sync::execute(),

        // Data Management
        Commands::Import(args) => import_cmd::execute(args),
        Commands::Export(args) => export::execute(args),

        // Utility
        Commands::Repair(args) => repair::execute(args),
    }
}

// ================== src/main.rs ==================

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = jin::cli::Cli::parse();
    jin::run(cli)
}

// ================== src/lib.rs ==================

use crate::cli;
use crate::commands;

pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    commands::execute(cli).map_err(|e| anyhow::anyhow!("{}", e))
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: CREATE src/cli/mod.rs with CLI structure
  IMPLEMENT: Cli struct with Parser derive
  IMPLEMENT: Commands enum with 22 variants
  IMPLEMENT: ModeAction enum with 6 subcommand variants
  IMPLEMENT: ScopeAction enum with 6 subcommand variants
  FOLLOW pattern: Use clap derive API attributes
  NAMING: CamelCase for types, snake_case for fields
  DEPENDENCIES: clap = { version = "4.5", features = ["derive"] } in Cargo.toml
  PLACEMENT: src/cli/mod.rs
  CRITICAL: #[command(propagate_version = true)] is required

Task 2: CREATE src/cli/args.rs with argument structs
  IMPLEMENT: AddArgs (files, mode, scope, project, global)
  IMPLEMENT: CommitArgs (message, dry_run)
  IMPLEMENT: ApplyArgs (force, dry_run)
  IMPLEMENT: ResetArgs (soft, mixed, hard, mode, scope, project)
  IMPLEMENT: DiffArgs (layer1, layer2, staged)
  IMPLEMENT: LogArgs (layer, count with default 10)
  IMPLEMENT: ImportArgs (files, force)
  IMPLEMENT: ExportArgs (files)
  IMPLEMENT: RepairArgs (dry_run)
  IMPLEMENT: LinkArgs (url, force)
  IMPLEMENT: PushArgs (force)
  FOLLOW pattern: Use #[derive(Args, Debug)]
  NAMING: CommandArgs naming convention (AddArgs, CommitArgs, etc.)
  PLACEMENT: src/cli/args.rs
  DEPENDENCIES: Task 1 (Commands enum references these types)

Task 3: MODIFY src/commands/mod.rs with dispatcher
  IMPLEMENT: execute(cli: Cli) -> Result<()> function
  IMPLEMENT: Match statement routing all 22 commands
  IMPLEMENT: Module declarations for all command modules
  FOLLOW pattern: src/commands/mod.rs in existing codebase
  NAMING: Unit variants call func(), struct variants pass args
  DEPENDENCIES: Task 1 (needs Cli and Commands types)
  PLACEMENT: src/commands/mod.rs
  CRITICAL: Match must be exhaustive (handle ALL 22 variants)

Task 4: VERIFY src/main.rs entry point
  VERIFY: Uses jin::cli::Cli::parse() to parse CLI
  VERIFY: Calls jin::run(cli) with parsed CLI
  VERIFY: Returns anyhow::Result<()>
  FOLLOW pattern: Existing src/main.rs
  PLACEMENT: src/main.rs
  DEPENDENCIES: Task 1, Task 2, Task 3

Task 5: VERIFY src/lib.rs library integration
  VERIFY: Exports pub fn run(cli: cli::Cli) -> anyhow::Result<()>
  VERIFY: Calls commands::execute(cli)
  VERIFY: Converts JinError to anyhow::Error
  FOLLOW pattern: Existing src/lib.rs
  PLACEMENT: src/lib.rs
  DEPENDENCIES: Task 3

Task 6: CREATE stub implementations for missing commands
  CREATE: src/commands/add.rs with execute(args) stub
  CREATE: src/commands/commit_cmd.rs with execute(args) stub
  CREATE: src/commands/apply.rs with execute(args) stub
  CREATE: src/commands/reset.rs with execute(args) stub
  CREATE: src/commands/diff.rs with execute(args) stub
  CREATE: src/commands/log.rs with execute(args) stub
  CREATE: src/commands/context.rs with execute() stub
  CREATE: src/commands/layers.rs with execute() stub
  CREATE: src/commands/list.rs with execute() stub
  CREATE: src/commands/link.rs with execute(args) stub
  CREATE: src/commands/fetch.rs with execute() stub
  CREATE: src/commands/pull.rs with execute() stub
  CREATE: src/commands/push.rs with execute(args) stub
  CREATE: src/commands/sync.rs with execute() stub
  CREATE: src/commands/import_cmd.rs with execute(args) stub
  CREATE: src/commands/export.rs with execute(args) stub
  CREATE: src/commands/repair.rs with execute(args) stub
  CREATE: src/commands/completion.rs with execute(shell) stub
  FOLLOW pattern: Use Err(JinError::Other("not yet implemented".to_string()))
  PLACEMENT: src/commands/*.rs
  DEPENDENCIES: Task 3 (dispatcher references these)

Task 7: CREATE integration tests in tests/cli_basic.rs
  IMPLEMENT: test_help_flag (jin --help)
  IMPLEMENT: test_version_flag (jin --version)
  IMPLEMENT: Tests for all 22 commands parsing correctly
  IMPLEMENT: test_invalid_command_error
  FOLLOW pattern: Use assert_cmd::Command and predicates
  PLACEMENT: tests/cli_basic.rs
  DEPENDENCIES: All CLI code must compile first
```

### Implementation Patterns & Key Details

```rust
// ================== MAIN CLI STRUCT PATTERN ==================
// src/cli/mod.rs

use clap::{Parser, Subcommand};
use clap_complete::Shell;

pub mod args;
pub use args::*;

/// Jin - Phantom Git layer system for developer configuration
#[derive(Parser, Debug)]
#[command(name = "jin")]
#[command(author, version, about = "Phantom Git layer system for developer configuration")]
#[command(propagate_version = true)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// GOTCHA: propagate_version = true is CRITICAL for --version to work
/// PATTERN: Doc comment on struct becomes main about text

// ================== COMMANDS ENUM PATTERN ==================
// Unit variants for commands without arguments
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize Jin in current project
    Init,                              // Unit variant - no arguments

    /// Stage files to appropriate layer
    Add(AddArgs),                      // Struct variant - wraps Args

    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),                  // Subcommand - wraps Subcommand enum
}

// PATTERN: Unit variants (Init), Struct variants (Add(AddArgs)), Subcommand variants (Mode(ModeAction))
// GOTCHA: #[command(subcommand)] is REQUIRED for nested subcommands

// ================== NESTED SUBCOMMAND PATTERN ==================
// ModeAction and ScopeAction enums for nested subcommands
#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create { name: String },           // Struct variant with fields

    /// List available modes
    List,                              // Unit variant
}

// PATTERN: Use Action suffix for subcommand enums
// PATTERN: Mix of struct variants and unit variants

// ================== ARGUMENT STRUCT PATTERN ==================
// src/cli/args.rs

use clap::Args;

/// Arguments for the `add` command
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Files to stage
    pub files: Vec<String>,            // Vec<T> = zero or more values

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,                    // bool = --flag, defaults to false

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,         // Option<T> = optional flag value

    /// Number of entries to show
    #[arg(long, default_value = "10")]
    pub count: usize,                  // default_value for defaults
}

// PATTERN: Positional args first (no #[arg()]), flags after
// PATTERN: Vec<T> for optional positional (zero or more)
// PATTERN: Option<T> for optional flag values
// PATTERN: bool for boolean flags (--flag)
// PATTERN: default_value for default values

// ================== COMMAND DISPATCHER PATTERN ==================
// src/commands/mod.rs

use crate::cli::{Cli, Commands};
use crate::core::Result;

pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        // Unit variant - no parameters
        Commands::Init => init::execute(),

        // Struct variant - destructure Args
        Commands::Add(args) => add::execute(args),

        // Subcommand variant - destructure Action
        Commands::Mode(action) => mode::execute(action),
    }
}

// PATTERN: Match on cli.command, route to execute functions
// GOTCHA: Match MUST be exhaustive - all 22 variants handled
// GOTCHA: Rust compile error if any variant is missing

// ================== STUB COMMAND PATTERN ==================
// src/commands/add.rs (stub for now, implemented in P3.M1)

use crate::cli::AddArgs;
use crate::core::{JinError, Result};

pub fn execute(_args: AddArgs) -> Result<()> {
    Err(JinError::Other("jin add not yet implemented".to_string()))
}

// PATTERN: Return Err with descriptive message for stubs
// PATTERN: Use _args prefix to silence unused warning

// ================== ENTRY POINT PATTERN ==================
// src/main.rs

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = jin::cli::Cli::parse();
    jin::run(cli)
}

// PATTERN: Parse CLI first, then pass to library
// PATTERN: Return anyhow::Result<()> for clean error display

// ================== LIBRARY INTEGRATION PATTERN ==================
// src/lib.rs

use crate::cli;
use crate::commands;

pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    commands::execute(cli).map_err(|e| anyhow::anyhow!("{}", e))
}

// PATTERN: Convert JinError to anyhow::Error for display
```

### Integration Points

```yaml
CARGO_TOML:
  dependencies:
    - clap = { version = "4.5", features = ["derive", "cargo"] }
    - anyhow = "1.0"
    - thiserror = "2.0"
  dev-dependencies:
    - assert_cmd = "2.0"
    - predicates = "3.0"
    - tempfile = "3.0"

CLI_MODULE (src/cli/mod.rs):
  - Exports: Cli, Commands, ModeAction, ScopeAction
  - Re-exports: All argument types from args.rs
  - Dependencies: clap::{Parser, Subcommand}, clap_complete::Shell

ARGS_MODULE (src/cli/args.rs):
  - Exports: All *Args structs
  - Dependencies: clap::Args

COMMANDS_MODULE (src/commands/mod.rs):
  - Exports: execute() function
  - Dependencies: All individual command modules

LIBRARY_ROOT (src/lib.rs):
  - Exports: run() function
  - Dependencies: cli, commands modules

ENTRY_POINT (src/main.rs):
  - Calls: jin::cli::Cli::parse(), jin::run()
  - Dependencies: jin library

TESTS (tests/cli_basic.rs):
  - Uses: assert_cmd::Command, predicates
  - Validates: All commands parse correctly
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Type checking - run after each file creation
cargo check

# Format checking - ensures consistent formatting
cargo fmt -- --check

# Lint checking - catches common mistakes
cargo clippy -- -D warnings

# Expected: Zero errors, zero warnings
# If errors exist, READ output carefully and fix before proceeding
```

### Level 2: Build Validation

```bash
# Debug build - must succeed before testing
cargo build

# Release build - verifies optimizations don't break anything
cargo build --release

# Expected: Clean build with no warnings
# If warnings exist, fix them before proceeding
```

### Level 3: Unit Tests (CLI Structure Tests)

```bash
# Run all unit tests in CLI module
cargo test cli::

# Run specific tests
cargo test test_cli_parser

# Expected: All tests pass
# If tests fail, debug root cause and fix
```

### Level 4: Integration Tests (Command Parsing)

```bash
# Run CLI integration tests
cargo test --test cli_basic

# Run with output for debugging
cargo test --test cli_basic -- --nocapture

# Run specific test
cargo test --test cli_basic test_help_flag

# Expected: All integration tests pass
# These tests verify:
# - All 22 commands parse correctly
# - --help flag works
# - --version flag works
# - Invalid commands produce errors
```

### Level 5: Manual CLI Validation

```bash
# Build the binary first
cargo build --release

# Test help flag
./target/release/jin --help
# Expected: Shows all 22 commands with descriptions

# Test version flag
./target/release/jin --version
# Expected: Shows "jin 0.1.0"

# Test individual command help
./target/release/jin init --help
./target/release/jin add --help
./target/release/jin commit --help
./target/release/jin mode --help
./target/release/jin mode create --help
./target/release/jin scope --help
./target/release/jin scope create --help
# Expected: Each shows command-specific help

# Test command parsing (will fail with "not yet implemented" but should parse)
./target/release/jin init
./target/release/jin add file.json
./target/release/jin add file.json --mode
./target/release/jin commit -m "test"
./target/release/jin status
./target/release/jin mode create test
./target/release/jin mode use test
./target/release/jin mode list
./target/release/jin mode show
./target/release/jin mode unset
./target/release/jin mode delete test
./target/release/jin scope create test
./target/release/jin scope create test --mode=dev
./target/release/jin scope use test
./target/release/jin scope list
./target/release/jin apply
./target/release/jin reset
./target/release/jin diff
./target/release/jin log
./target/release/jin layers
./target/release/jin list
./target/release/jin link https://example.com
./target/release/jin fetch
./target/release/jin pull
./target/release/jin push
./target/release/jin sync
./target/release/jin import file.json
./target/release/jin export file.json
./target/release/jin repair
./target/release/jin context
# Expected: Commands either execute or print "not yet implemented"

# Test invalid command
./target/release/jin invalid-command
# Expected: Error message with suggestion

# Test completion command
./target/release/jin completion bash
./target/release/jin completion zsh
# Expected: Outputs shell completion script

# Expected: All commands parse without argument errors
# (Implementation may fail with "not yet implemented" - that's OK)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` all tests pass
- [ ] `cargo test --test cli_basic` all integration tests pass

### CLI Structure Validation

- [ ] 22 commands defined in Commands enum
- [ ] ModeAction enum with 6 variants: create, use, list, delete, show, unset
- [ ] ScopeAction enum with 6 variants: create, use, list, delete, show, unset
- [ ] All command variants properly typed (unit, struct, or subcommand)
- [ ] All argument structs use clap Args derive
- [ ] Boolean flags use `#[arg(long)]`
- [ ] Optional arguments use `Option<T>`
- [ ] Multi-value arguments use `Vec<T>`
- [ ] Default values use `default_value` attribute
- [ ] `#[command(propagate_version = true)]` present on Cli struct
- [ ] Command dispatcher handles all 22 Commands enum variants

### Help & Documentation Validation

- [ ] `jin --help` shows all 22 commands
- [ ] Each command has a doc comment (/// description)
- [ ] `jin <command> --help` works for all commands
- [ ] `jin mode --help` shows mode subcommands
- [ ] `jin scope --help` shows scope subcommands
- [ ] `jin mode create --help` shows create subcommand help
- [ ] Version flag `--version` works
- [ ] All help text is descriptive and clear

### Error Handling Validation

- [ ] Invalid command shows error
- [ ] Missing required arguments shows error
- [ ] Command dispatcher handles all enum variants
- [ ] main() returns anyhow::Result for clean error display
- [ ] Stub commands return meaningful "not yet implemented" errors

### Code Quality Validation

- [ ] Follows clap derive best practices
- [ ] Consistent naming: CamelCase for types, snake_case for fields
- [ ] All public types have doc comments
- [ ] No duplicate short flags (checked -m, etc.)
- [ ] propagate_version enabled for subcommands
- [ ] Module structure follows established patterns

### Integration Validation

- [ ] src/cli/mod.rs exports Cli, Commands, ModeAction, ScopeAction
- [ ] src/cli/args.rs exports all *Args structs
- [ ] src/commands/mod.rs has execute() dispatcher
- [ ] src/main.rs parses CLI and calls run()
- [ ] src/lib.rs has run() function
- [ ] All imports resolve correctly
- [ ] No circular dependencies

---

## Anti-Patterns to Avoid

- **Don't mix builder and derive APIs**: Use pure derive API with `#[derive(Parser, Subcommand, Args)]`
- **Don't use hardcoded version strings**: Use clap's `#[command(version)]` attribute to read from Cargo.toml
- **Don't skip help text**: All commands need doc comments (`/// description`)
- **Don't create conflicting short flags**: Check all `-x` flags to avoid conflicts (e.g., `-m` is for message)
- **Don't use overly nested command structure**: Max 2 levels: `jin mode create` (not 3+ levels)
- **Don't use inconsistent argument patterns**: Follow same style across all commands (flags after positionals)
- **Don't skip propagate_version**: Without `#[command(propagate_version = true)]`, `--version` won't work
- **Don't use struct variants for unit commands**: Use `Init` not `Init {}` for commands without arguments
- **Don't wrap Args unnecessarily**: Use `Add(AddArgs)` not `Add { args: AddArgs }`
- **Don't forget #[command(subcommand)]**: Required for nested subcommands like Mode and Scope
- **Don't skip exhaustive match**: Command dispatcher must handle ALL 22 variants (Rust will error if not)
- **Don't use sync functions in async context** (if adding async later): Plan for async if needed
- **Don't catch all exceptions**: Use specific error types (JinError), not `catch-all`
- **Don't ignore Cargo.toml features**: Must have `clap = { version = "4.5", features = ["derive"] }`

---

## Command Reference Matrix

| Category | Command | Arguments | Subcommands | Status |
|----------|---------|-----------|-------------|--------|
| **Initialization** | `init` | None | None | To Define |
|  | `link` | LinkArgs | None | To Define |
|  | `completion` | shell enum | None | To Define |
| **Staging & Committing** | `add` | AddArgs | None | To Define |
|  | `commit` | CommitArgs | None | To Define |
| **Status & Inspection** | `status` | None | None | To Define |
|  | `diff` | DiffArgs | None | To Define |
|  | `log` | LogArgs | None | To Define |
|  | `layers` | None | None | To Define |
|  | `list` | None | None | To Define |
|  | `context` | None | None | To Define |
| **Mode Management** | `mode` | None | ModeAction (6) | To Define |
| **Scope Management** | `scope` | None | ScopeAction (6) | To Define |
| **Workspace Operations** | `apply` | ApplyArgs | None | To Define |
|  | `reset` | ResetArgs | None | To Define |
| **Synchronization** | `fetch` | None | None | To Define |
|  | `pull` | None | None | To Define |
|  | `push` | PushArgs | None | To Define |
|  | `sync` | None | None | To Define |
| **Data Management** | `import` | ImportArgs | None | To Define |
|  | `export` | ExportArgs | None | To Define |
| **Utility** | `repair` | RepairArgs | None | To Define |

**Total: 22 commands (including nested subcommands)**

---

## Appendix: Mode Subcommands

| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `create` | `<name>` | Create a new mode |
| `use` | `<name>` | Activate a mode |
| `list` | None | List available modes |
| `delete` | `<name>` | Delete a mode |
| `show` | None | Show current mode |
| `unset` | None | Deactivate current mode |

---

## Appendix: Scope Subcommands

| Subcommand | Arguments | Description |
|------------|-----------|-------------|
| `create` | `<name> [--mode=<mode>]` | Create a new scope (optionally tied to mode) |
| `use` | `<name>` | Activate a scope |
| `list` | None | List available scopes |
| `delete` | `<name>` | Delete a scope |
| `show` | None | Show current scope |
| `unset` | None | Deactivate current scope |

---

## Appendix: Argument Struct Reference

| Struct | Fields | Usage |
|--------|--------|-------|
| `AddArgs` | files: Vec<String>, mode: bool, scope: Option<String>, project: bool, global: bool | Layer routing flags |
| `CommitArgs` | message: String, dry_run: bool | Commit with message |
| `ApplyArgs` | force: bool, dry_run: bool | Workspace application |
| `ResetArgs` | soft: bool, mixed: bool, hard: bool, mode: bool, scope: Option<String>, project: bool | Reset modes and targets |
| `DiffArgs` | layer1: Option<String>, layer2: Option<String>, staged: bool | Comparison options |
| `LogArgs` | layer: Option<String>, count: usize (default 10) | History filtering |
| `ImportArgs` | files: Vec<String>, force: bool | Git import options |
| `ExportArgs` | files: Vec<String> | Git export targets |
| `RepairArgs` | dry_run: bool | Repair preview |
| `LinkArgs` | url: String, force: bool | Remote repository URL |
| `PushArgs` | force: bool | Force push option |

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success likelihood

**Justification:**
- Comprehensive clap derive API documentation with specific patterns
- Existing codebase patterns documented with exact file references
- Real Rust CLI project examples for inspiration
- Step-by-step implementation tasks in dependency order
- Complete command reference with all 22 commands specified
- Detailed validation procedures matching this codebase
- Anti-patterns section to avoid common mistakes
- All argument types defined with clap attributes

**Implementation Readiness: EXCELLENT**

This PRP provides everything needed to implement the CLI structure in one pass, including:
1. Exact code structure to follow
2. All 22 commands with their arguments
3. Nested subcommand patterns for Mode and Scope
4. Integration test examples
5. Validation procedures specific to this codebase

The only assumption is basic Rust knowledge and familiarity with derive macros, which is reasonable for this task.
