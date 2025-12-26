# Product Requirement Prompt (PRP): CLI Structure (P4.M1.T1)

---

## Goal

**Feature Goal**: Define the complete CLI structure using clap derive macros, establishing the command and subcommand hierarchy for all Jin operations (init, add, commit, status, mode, scope, reset, apply, diff, log, context, import, export, repair, layers, list, link, fetch, pull, push, sync, rm, mv, completion).

**Deliverable**: A `src/cli/` module with:
- `args.rs` containing all clap derive structs for commands and subcommands
- Complete command hierarchy matching PRD specification
- Proper flag definitions for layer routing (`--mode`, `--scope`, `--project`, `--global`)
- Integration with existing `Layer::from_flags()` routing logic

**Success Definition**:
- All commands from PRD are defined in clap derive structs
- Command hierarchy matches PRD specification exactly
- Layer routing flags are properly defined
- `cargo check` passes with zero errors
- `cargo test --package jin --lib cli` passes all validation tests
- Module exported from `src/cli/mod.rs`

## User Persona

**Target User**: AI coding agent implementing Jin's CLI framework

**Use Case**: The agent needs to create the CLI structure that:
- Defines all commands and subcommands using clap derive macros
- Maps CLI flags to the existing layer routing system
- Provides the foundation for command handler implementation in subsequent tasks
- Integrates with existing `Layer`, `LayerRouter`, and error handling types

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/cli/args.rs` with complete CLI structure
3. Defines `Commands` enum with all subcommands
4. Defines argument structs for each command with appropriate flags
5. Updates `src/cli/mod.rs` to export the CLI types
6. Updates `src/main.rs` to use the CLI parser
7. Validates compilation and passes tests

**Pain Points Addressed**:
- No manual command line parsing - clap derive handles it
- Consistent layer routing across all commands
- Type-safe command dispatch
- Foundation for all command implementations

## Why

- **Foundation for all CLI commands**: Every user interaction goes through this CLI structure
- **Layer routing consistency**: Centralized definition of routing flags ensures consistent behavior
- **Type-safe command dispatch**: Derive macros provide compile-time safety for command structure
- **Problems this solves**:
  - Provides complete CLI specification in one place
  - Enables command handler implementation in subsequent tasks
  - Ensures all commands have proper argument definitions
  - Integrates with existing `Layer::from_flags()` for routing

## What

Define the complete CLI structure using clap derive macros for all Jin commands.

### Success Criteria

- [ ] `src/cli/args.rs` created with complete CLI structure
- [ ] `Commands` enum defines all top-level commands
- [ ] Subcommand enums define all nested commands (mode, scope)
- [ ] Layer routing flags (`--mode`, `--scope`, `--project`, `--global`) defined where applicable
- [ ] All structs use clap derive macros (`Parser`, `Subcommand`)
- [ ] Proper help text and documentation on all commands
- [ ] Module exported from `src/cli/mod.rs`
- [ ] `src/main.rs` updated to use CLI parser
- [ ] `cargo check` passes with zero errors
- [ ] Unit tests verify command structure

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Complete list of all commands and subcommands from PRD
- Exact clap derive patterns to follow
- Integration points with existing `Layer::from_flags()`
- File structure and naming conventions
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Complete specification of all CLI commands with flags and arguments
  section: Lines 474-555 for "Core Commands", Lines 492-545 for all command specifications
  critical: Every command, flag, and argument is specified here

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum with from_flags() for CLI flag routing
  pattern: Layer::from_flags(mode, scope, project, global) -> Option<Layer>
  section: Lines 331-379 for from_flags() routing implementation
  critical: "CLI flags must match the routing table in from_flags()"

- file: /home/dustin/projects/jin-glm-doover/src/staging/router.rs
  why: LayerRouter wrapper for routing with error handling
  pattern: LayerRouter::route(mode, scope, project_flag, global) -> Result<Layer>
  section: Lines 105-120 for route() method
  critical: Shows how to convert CLI flags to Layer

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: JinError type for error handling
  pattern: Use existing JinError variants for CLI errors
  gotcha: Don't create new error types - use existing ones

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify clap dependency and features
  section: Line 20: clap = { version = "4.5", features = ["derive"] }
  critical: "derive" feature is REQUIRED for Parser and Subcommand macros

- file: /home/dustin/projects/jin-glm-doover/src/main.rs
  why: Current entry point - needs to be updated
  pattern: Replace placeholder with clap CLI parsing
  section: Lines 1-7 for current placeholder implementation

- file: /home/dustin/projects/jin-glm-doover/src/cli/mod.rs
  why: Module stub - needs to export CLI types
  pattern: Add pub mod args; and pub use for CLI types

# EXTERNAL - Clap Documentation

- url: https://docs.rs/clap/4.5/clap/
  why: Official clap v4 documentation
  critical: Derive API reference for Parser, Subcommand, Args traits
  section: "Derive API" - complete reference for all derive macros

- url: https://docs.rs/clap/4.5/clap/derive/index.html
  why: Quick reference for common derive patterns
  pattern: #[command(name, about, long_about)] attribute usage
  critical: Parser trait for main command, Subcommand trait for subcommands

- url: https://docs.rs/clap/4.5/clap/struct.Arg.html
  why: Argument configuration options
  critical: id, long, short, help, long_help, required, value_name, num_args

- url: https://github.com/clap-rs/clap/tree/master/examples
  why: Example code demonstrating clap usage patterns
  pattern: Look for "derive" examples showing multi-level subcommands
  critical: examples/derive_ref/git.rs for complex CLI structure

# EXTERNAL - Rust CLI Best Practices

- url: https://rust-cli.github.io/book/tutorial/cli-args.html
  why: Guide on structuring CLI applications
  pattern: Commands enum with Subcommand derive
  critical: Shows proper command organization

- url: https://github.com/BurntSushi/ripgrep
  why: Excellent example of complex CLI in Rust
  pattern: src/cli.rs for command argument definitions
  gotcha: Uses both derive and builder patterns - we only use derive

- url: https://github.com/sharkdp/bat
  why: Another great CLI example with clap derive
  pattern: src/args.rs for clean argument structure
  critical: Shows how to organize many subcommands

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P4M1T1/research/clap_research.md
  why: Compiled clap v4 research with code examples
  section: Multi-level subcommands, derive patterns, flag definitions
  critical: Example code for complex command hierarchies
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                      # Has clap 4.5 with derive feature
├── PRD.md                          # Complete command specification
├── src/
│   ├── main.rs                     # Placeholder - needs CLI integration
│   ├── lib.rs                      # Library exports
│   ├── cli/
│   │   └── mod.rs                  # Empty stub - needs implementation
│   ├── commands/
│   │   └── mod.rs                  # Empty stub - for future command handlers
│   ├── core/
│   │   ├── mod.rs                  # Exports error, layer, config
│   │   ├── error.rs                # JinError enum
│   │   ├── layer.rs                # Layer enum with from_flags()
│   │   └── config.rs               # Config structs
│   ├── staging/
│   │   ├── mod.rs                  # Staging exports
│   │   ├── router.rs               # LayerRouter for routing flags to layers
│   │   ├── index.rs                # StagingIndex
│   │   └── entry.rs                # StagedEntry
│   ├── git/
│   │   ├── mod.rs                  # Git operations exports
│   │   ├── repo.rs                 # JinRepo
│   │   └── transaction.rs          # Transaction system
│   ├── merge/
│   │   └── mod.rs                  # Merge engine exports
│   ├── commit/
│   │   └── mod.rs                  # Commit pipeline exports
│   └── workspace/
│       └── mod.rs                  # Workspace operations
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   ├── main.rs                     # MODIFY: Add CLI parsing and dispatch
│   └── cli/
│       ├── mod.rs                  # MODIFY: Export CLI types
│       └── args.rs                 # CREATE: Complete CLI structure
└── tests/
    └── cli/
        └── args_test.rs            # CREATE: Unit tests for CLI structure
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: clap MUST use "derive" feature - we don't use builder API
// Already configured in Cargo.toml: clap = { version = "4.5", features = ["derive"] }
// Use #[derive(Parser, Subcommand)] not clap::Command::new()

// CRITICAL: Use existing Layer::from_flags() for routing logic
// Do NOT reimplement routing in CLI code
// Good:
//   let layer = Layer::from_flags(args.mode, args.scope, args.project, args.global);
// Bad:
//   let layer = match (args.mode, args.scope, ...) { ... };

// CRITICAL: Layer::from_flags() returns Option<Layer>
// None means no routing target - use default project behavior
// Good:
//   let layer = Layer::from_flags(mode, scope, project, global)
//       .unwrap_or_else(|| Layer::ProjectBase { project: inferred });
// Bad:
//   let layer = Layer::from_flags(...).unwrap();

// CRITICAL: --project flag is boolean, --scope=<value> takes a value
// From PRD routing table:
// - --mode: Boolean flag (uses active mode from context)
// - --scope=<scope>: Takes scope name as value
// - --project: Boolean flag (uses inferred project name)
// - --global: Boolean flag (routes to GlobalBase)
// Good:
//   #[arg(long)]
//   mode: bool,
//   #[arg(long, value_name = "SCOPE")]
//   scope: Option<String>,
//   #[arg(long)]
//   project: bool,
//   #[arg(long)]
//   global: bool,
// Bad:
//   mode: Option<String>,  // Wrong: --mode is boolean, uses active context

// CRITICAL: Command names must match PRD exactly
// - "jin init" -> Init command
// - "jin mode create" -> Mode subcommand with Create sub-subcommand
// - "jin add" -> Add command
// - "jin modes" (plural) -> Modes command (list)
// Good:
//   #[command(name = "init")]
//   Init,
//   Mode(ModeCommand),
//   Add(AddCommand),
//   Modes,
// Bad:
//   Initialize,  // Wrong name
//   CreateMode,  // Wrong hierarchy

// CRITICAL: Mode and Scope commands have subcommands
// "jin mode create <mode>" -> Mode::Create
// "jin mode use <mode>" -> Mode::Use
// "jin mode unset" -> Mode::Unset
// "jin mode delete <mode>" -> Mode::Delete
// "jin modes" -> Modes (list command, separate enum variant)
// Same pattern for Scope commands

// CRITICAL: "modes" (plural) and "scopes" (plural) are list commands
// These are top-level commands, not subcommands of mode/scope
// Good:
//   enum Commands {
//       Mode(ModeCommand),  // jin mode create/use/unset/delete/show
//       Modes,              // jin modes (list)
//       Scope(ScopeCommand),// jin scope create/use/unset/delete/show
//       Scopes,             // jin scopes (list)
//   }
// Bad:
//   Mode(ModeCommand),  // ModeCommand has "list" subcommand
//   // No Modes variant

// GOTCHA: clap's long_about only renders with --help flag
// Regular help shows "about" text
// Good:
//   #[command(about = "Short help", long_about = "Long detailed help")]
// Bad:
//   #[command(help = "Wrong attribute name")]

// GOTCHA: Value names in --flag=<value> syntax
// Use value_name for the placeholder in help text
// Good:
//   #[arg(long, value_name = "MODE")]
//   mode: Option<String>,
// Bad:
//   #[arg(long)]
//   mode: Option<String>,  // Help won't show the value placeholder

// GOTCHA: Multiple values for files argument
// Use num_args(1..) for "one or more" values
// Good:
//   #[arg(value_name = "FILE", num_args(1..))]
//   files: Vec<PathBuf>,
// Bad:
//   files: Vec<String>,  // Won't parse correctly

// GOTCHA: Main command needs version from Cargo.toml
// Use try_parse() instead of parse() for better error handling
// Good:
//   let cli = Cli::try_parse();
//   match cli { ... }
// Bad:
//   let cli = Cli::parse();  // Exits on error, can't customize

// PATTERN: Follow existing codebase naming conventions
// - Structs: PascalCase (AddCommand, not add_command)
// - Fields: snake_case (files, not Files)
// - Files: snake_case (args.rs, not args.rs or Args.rs)
// - Commands: PascalCase for variants, lowercase for CLI names

// PATTERN: Command handler stubs go in commands/ module
// This PRP only defines CLI structure - handlers are implemented in P4.M2-M4
// For now, commands are matched and show "not implemented" messages

// FUTURE: Shell completion generation (P6.M1)
// clap derive automatically generates completion scripts
// This PRP sets up the structure for completion generation later
```

---

## Implementation Blueprint

### Data Models and Structure

This task defines the CLI structure - data models exist in core/ module.

```rust
/// Main CLI structure for Jin.
///
/// This is the entry point for all command-line interactions.
#[derive(Parser)]
#[command(name = "jin")]
#[command(about = "Multi-layer Git overlay system", long_about = "
Jin is a meta-versioning system layered on top of Git that manages
developer-specific and tool-specific configuration without contaminating
a project's primary Git repository.

Use 'jin help <command>' for more information on a specific command.
")]
#[command(version)]
pub struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available Jin commands.
///
/// This enum defines all top-level commands from the PRD specification.
/// Commands are organized by functional area:
/// - Core operations (init, add, commit, reset, status)
/// - Context management (mode, modes, scope, scopes)
/// - Workspace operations (apply)
/// - Inspection (diff, log, context, layers, list)
/// - Import/export (import, export)
/// - Maintenance (repair)
/// - Remote operations (link, fetch, pull, push, sync)
/// - File operations (rm, mv)
#[derive(Subcommand)]
pub enum Commands {
    // ===== Core Commands =====
    /// Initialize Jin in the current project
    Init(InitCommand),

    /// Stage files to the appropriate layer
    Add(AddCommand),

    /// Commit staged files
    Commit(CommitCommand),

    /// Reset staged or committed changes
    Reset(ResetCommand),

    /// Show workspace state and status
    Status(StatusCommand),

    // ===== Mode Management =====
    /// Mode management commands
    #[command(subcommand)]
    Mode(ModeCommand),

    /// List all available modes
    Modes,

    // ===== Scope Management =====
    /// Scope management commands
    #[command(subcommand)]
    Scope(ScopeCommand),

    /// List all available scopes
    Scopes,

    // ===== Workspace Operations =====
    /// Merge layers into the workspace
    Apply(ApplyCommand),

    // ===== Inspection Commands =====
    /// Show differences between layers or workspace
    Diff(DiffCommand),

    /// Show commit history for layers
    Log(LogCommand),

    /// Show active context
    Context,

    /// Show current layer composition
    Layers,

    /// List available modes/scopes/projects
    List,

    // ===== Import/Export =====
    /// Import Git-tracked files into Jin
    Import(ImportCommand),

    /// Export Jin files back to Git
    Export(ExportCommand),

    // ===== Maintenance =====
    /// Repair Jin state
    Repair(RepairCommand),

    // ===== Remote Operations =====
    /// Link to shared Jin config repository
    Link(LinkCommand),

    /// Fetch updates from remote Jin repo
    Fetch,

    /// Fetch and merge updates
    Pull,

    /// Push local changes
    Push,

    /// Fetch, merge, and apply
    Sync,

    // ===== File Operations =====
    /// Remove file from layer
    Rm(RmCommand),

    /// Rename/move file within layer
    Mv(MvCommand),
}

/// Mode subcommands.
///
/// Manages mode lifecycle (create, activate, deactivate, delete, show).
#[derive(Subcommand)]
pub enum ModeCommand {
    /// Create a new mode
    Create {
        /// Mode name to create
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Activate a mode (set as active context)
    Use {
        /// Mode name to activate
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Deactivate the current mode
    Unset,

    /// Delete a mode
    Delete {
        /// Mode name to delete
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Show the current active mode
    Show,
}

/// Scope subcommands.
///
/// Manages scope lifecycle (create, activate, deactivate, delete, show).
#[derive(Subcommand)]
pub enum ScopeCommand {
    /// Create a new scope
    Create {
        /// Scope name to create
        #[arg(value_name = "SCOPE")]
        name: String,

        /// Bind to a specific mode
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
    },

    /// Activate a scope (set as active context)
    Use {
        /// Scope name to activate
        #[arg(value_name = "SCOPE")]
        name: String,
    },

    /// Deactivate the current scope
    Unset,

    /// Delete a scope
    Delete {
        /// Scope name to delete
        #[arg(value_name = "SCOPE")]
        name: String,
    },

    /// Show the current active scope
    Show,
}

/// Arguments for 'jin init'
pub struct InitCommand;

/// Arguments for 'jin add'
pub struct AddCommand {
    /// Files to stage
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,

    /// Route to mode base layer (uses active mode)
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Route to global layer
    #[arg(long)]
    pub global: bool,
}

/// Arguments for 'jin commit'
pub struct CommitCommand {
    /// Commit message
    #[arg(long, required = true)]
    pub message: String,

    /// Allow empty commit
    #[arg(long)]
    pub allow_empty: bool,
}

/// Arguments for 'jin reset'
pub struct ResetCommand {
    /// Paths to reset (optional, defaults to all)
    #[arg(value_name = "PATH", num_args(0..))]
    pub paths: Vec<PathBuf>,

    /// Route to mode layer
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Keep changes in staging area
    #[arg(long)]
    pub soft: bool,

    /// Unstage but keep in workspace (default)
    #[arg(long, conflicts_with = "soft", conflicts_with = "hard")]
    pub mixed: bool,

    /// Discard all changes
    #[arg(long, conflicts_with = "soft")]
    pub hard: bool,
}

/// Arguments for 'jin status'
pub struct StatusCommand;

/// Arguments for 'jin apply'
pub struct ApplyCommand {
    /// Skip dirty check and force apply
    #[arg(long)]
    pub force: bool,

    /// Show plan without applying
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for 'jin diff'
pub struct DiffCommand {
    /// First layer (optional)
    #[arg(value_name = "LAYER1")]
    pub layer1: Option<String>,

    /// Second layer (optional)
    #[arg(value_name = "LAYER2")]
    pub layer2: Option<String>,

    /// Show staged vs layer current
    #[arg(long)]
    pub staged: bool,
}

/// Arguments for 'jin log'
pub struct LogCommand {
    /// Layer to show history for (optional)
    #[arg(value_name = "LAYER")]
    pub layer: Option<String>,

    /// Limit number of entries
    #[arg(long, value_name = "N")]
    pub count: Option<usize>,
}

/// Arguments for 'jin import'
pub struct ImportCommand {
    /// Files to import
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,
}

/// Arguments for 'jin export'
pub struct ExportCommand {
    /// Files to export
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,
}

/// Arguments for 'jin repair'
pub struct RepairCommand {
    /// Show plan without repairing
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for 'jin link'
pub struct LinkCommand {
    /// Repository URL to link
    #[arg(value_name = "URL")]
    pub url: String,
}

/// Arguments for 'jin rm'
pub struct RmCommand {
    /// Files to remove
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,

    /// Route to mode layer
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,
}

/// Arguments for 'jin mv'
pub struct MvCommand {
    /// Source path
    #[arg(value_name = "OLD")]
    pub old_path: PathBuf,

    /// Destination path
    #[arg(value_name = "NEW")]
    pub new_path: PathBuf,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/cli/args.rs
  - IMPLEMENT: Cli struct with Parser derive
  - IMPLEMENT: Commands enum with Subcommand derive
  - IMPLEMENT: ModeCommand enum with Subcommand derive
  - IMPLEMENT: ScopeCommand enum with Subcommand derive
  - IMPLEMENT: All command structs (InitCommand, AddCommand, etc.)
  - IMPLEMENT: Proper help text with about/long_about
  - IMPORTS:
    * use clap::{Parser, Subcommand}
    * use std::path::PathBuf
  - NAMING: PascalCase structs, snake_case fields
  - PATTERN: Follow clap derive API documentation
  - FLAGS:
    * --mode (bool), --scope=<scope> (Option<String>), --project (bool), --global (bool)
    * Must match Layer::from_flags() routing table
  - HELP: Provide clear help text for all commands
  - PLACEMENT: New file src/cli/args.rs

Task 2: MODIFY src/cli/mod.rs
  - ADD: pub mod args;
  - ADD: pub use args::{Cli, Commands, ModeCommand, ScopeCommand};
  - ADD: pub use args::*; for all command structs
  - PRESERVE: Any existing comments
  - FINAL FILE:
    // CLI definitions using clap derive
    pub mod args;

    pub use args::{
        Cli, Commands, ModeCommand, ScopeCommand,
        InitCommand, AddCommand, CommitCommand, ResetCommand, StatusCommand,
        ApplyCommand, DiffCommand, LogCommand,
        ImportCommand, ExportCommand, RepairCommand, LinkCommand,
        RmCommand, MvCommand,
    };
  - PLACEMENT: src/cli/mod.rs
  - DEPENDENCIES: Task 1 (args.rs must exist)

Task 3: MODIFY src/main.rs
  - IMPLEMENT: Parse CLI arguments using Cli::try_parse()
  - IMPLEMENT: Match on Commands enum and dispatch to handlers
  - IMPLEMENT: Placeholder "not implemented" messages for each command
  - IMPLEMENT: Proper exit codes (ExitCode::SUCCESS, ExitCode::FAILURE)
  - IMPLEMENT: Error display for parse failures
  - IMPORTS:
    * use jin_glm::cli::Cli
    * use std::process::ExitCode
  - PATTERN:
    ```rust
    fn main() -> ExitCode {
        match Cli::try_parse() {
            Ok(cli) => {
                // TODO: Dispatch to command handlers
                match cli.command {
                    Commands::Init(_) => println!("jin init - not implemented"),
                    Commands::Add(_) => println!("jin add - not implemented"),
                    // ... all other commands
                    _ => println!("Command not yet implemented"),
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprint!("Error: {e}");
                ExitCode::FAILURE
            }
        }
    }
    ```
  - NAMING: main() function
  - PLACEMENT: src/main.rs
  - DEPENDENCIES: Task 2 (cli module must be exported)

Task 4: CREATE tests/cli/args_test.rs
  - IMPLEMENT: Unit tests for CLI structure
  - TESTS:
    * test_cli_basic_parsing() - verify basic parsing works
    * test_init_command() - verify init command parsing
    * test_add_command_with_files() - verify file argument parsing
    * test_add_command_with_mode_flag() - verify --mode flag parsing
    * test_add_command_with_scope_flag() - verify --scope=<value> parsing
    * test_mode_create_command() - verify mode create subcommand parsing
    * test_mode_use_command() - verify mode use subcommand parsing
    * test_scope_create_command() - verify scope create subcommand parsing
    * test_scope_create_with_mode() - verify scope create --mode parsing
    * test_commit_with_message() - verify -m required flag parsing
    * test_reset_with_soft_flag() - verify reset --soft parsing
    * test_reset_with_hard_flag() - verify reset --hard parsing
    * test_apply_with_force_and_dry_run() - verify multiple flag parsing
    * test_diff_with_layers() - verify optional layer arguments
    * test_log_with_count() - verify --count=N flag parsing
  - USE: clap::CommandFactory and clap::Parser for testing
  - VERIFY: All commands parse correctly
  - VERIFY: Help text renders correctly
  - PLACEMENT: tests/cli/args_test.rs (create tests/cli/ first)
  - DEPENDENCIES: Task 1
```

### Implementation Patterns & Key Details

```rust
// ===== PATTERN 1: Main CLI Structure =====

/// Main CLI structure using clap derive.
///
/// The top-level command parser that dispatches to subcommands.
#[derive(Parser)]
#[command(name = "jin")]
#[command(about = "Multi-layer Git overlay system")]
#[command(version)]
#[command(long_about = "
Jin is a meta-versioning system layered on top of Git that manages
developer-specific and tool-specific configuration without contaminating
a project's primary Git repository.

More information at https://github.com/jin-versioning/jin
")]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

// ===== PATTERN 2: Commands Enum =====

/// All available Jin commands.
///
/// This enum must include every command from the PRD specification.
/// Commands are organized by functional area with clear comments.
#[derive(Subcommand)]
pub enum Commands {
    // Core Commands
    /// Initialize Jin in the current project
    Init(InitCommand),

    /// Stage files to the appropriate layer
    Add(AddCommand),

    // ... other commands

    // Mode Management
    /// Mode management commands
    #[command(subcommand)]
    Mode(ModeCommand),

    /// List all available modes
    Modes,

    // ... other commands
}

// ===== PATTERN 3: Layer Routing Flags =====

/// Arguments for 'jin add'.
///
/// Layer routing follows the PRD §9.1 routing table.
/// Flags map to Layer::from_flags() parameters.
pub struct AddCommand {
    /// Files to stage
    #[arg(value_name = "FILE", num_args(1..))]
    pub files: Vec<PathBuf>,

    /// Route to mode base layer (uses active mode)
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Route to global layer
    #[arg(long)]
    pub global: bool,
}

// Converting CLI flags to Layer:
// let layer = Layer::from_flags(
//     if args.mode { Some(active_mode) } else { None },
//     args.scope.as_deref(),
//     if args.project { Some(project) } else { None },
//     args.global,
// );

// ===== PATTERN 4: Subcommand Enums =====

/// Mode management commands.
///
/// Subcommands for mode lifecycle management.
#[derive(Subcommand)]
pub enum ModeCommand {
    /// Create a new mode
    Create {
        /// Mode name to create
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Activate a mode (set as active context)
    Use {
        /// Mode name to activate
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Deactivate the current mode
    Unset,

    /// Delete a mode
    Delete {
        /// Mode name to delete
        #[arg(value_name = "MODE")]
        name: String,
    },

    /// Show the current active mode
    Show,
}

// ===== PATTERN 5: Main Entry Point =====

// src/main.rs:
use jin_glm::cli::Cli;
use std::process::ExitCode;

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => {
            // TODO: Dispatch to command handlers (P4.M2-M4)
            match cli.command {
                Commands::Init(cmd) => {
                    println!("jin init - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Add(cmd) => {
                    println!("jin add - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Mode(ModeCommand::Create { name }) => {
                    println!("jin mode create {name} - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                // ... all other commands
                _ => {
                    println!("Command not yet implemented");
                    ExitCode::SUCCESS
                }
            }
        }
        Err(e) => {
            // clap automatically displays help/error message
            // Just return failure exit code
            ExitCode::FAILURE
        }
    }
}

// ===== GOTCHA: Value Names for Placeholders =====

// Use value_name for better help text
// Good:
#[arg(long, value_name = "SCOPE")]
scope: Option<String>,

// Help shows: --scope=<SCOPE>

// Bad:
#[arg(long)]
scope: Option<String>,

// Help shows: --scope=<SCOPE> (but less explicit)

// ===== GOTCHA: Multiple Values =====

// Use num_args(1..) for one or more values
// Good:
#[arg(value_name = "FILE", num_args(1..))]
files: Vec<PathBuf>,

// Bad:
files: Vec<PathBuf>,  // Won't parse correctly as CLI args

// ===== GOTCHA: Conflicting Flags =====

// Use conflicts_with for mutually exclusive flags
#[arg(long, conflicts_with = "soft", conflicts_with = "hard")]
mixed: bool,

// clap will error if both --soft and --mixed are specified

// ===== GOTCHA: Version from Cargo.toml =====

// clap derives version automatically from Cargo.toml
#[command(version)]

// No need to manually specify version string

// ===== PATTERN: Help Text Organization =====

// about: Short help (shown in command list)
// long_about: Detailed help (shown with --help)

#[command(
    about = "Short description",
    long_about = "Longer description with more details...

Can span multiple lines.
"
)]

// For args, use 'help' attribute
#[arg(
    long,
    help = "Route to mode base layer",
    long_help = "Route files to the mode base layer (Layer 2).
Uses the currently active mode from context."
)]
mode: bool,
```

### Integration Points

```yaml
LAYER_ROUTING:
  - use: src/core/layer.rs
  - method: Layer::from_flags(mode, scope, project, global)
  - integration: CLI flags map directly to this method's parameters
  - pattern:
    let layer = Layer::from_flags(
        if args.mode { active_mode.as_deref() } else { None },
        args.scope.as_deref(),
        if args.project { Some(project_name.as_str()) } else { None },
        args.global,
    );

LAYER_ROUTER:
  - use: src/staging/router.rs
  - method: LayerRouter::route(mode, scope, project_flag, global)
  - integration: Alternative to Layer::from_flags() with error handling
  - pattern:
    let router = LayerRouter::new(project_name);
    let layer = router.route(
        args.mode.then_some(active_mode.as_str()),
        args.scope.as_deref(),
        args.project,
        args.global,
    )?;

ERROR_HANDLING:
  - use: src/core/error.rs
  - type: JinError for CLI errors
  - integration: Command handlers return Result<(), JinError>
  - pattern:
    fn run() -> Result<(), JinError> {
        // Command implementation
        Ok(())
    }

MAIN_ENTRY:
  - modify: src/main.rs
  - integration: Parse CLI with Cli::try_parse()
  - pattern:
    match Cli::try_parse() {
        Ok(cli) => dispatch(cli),
        Err(e) => {
            eprint!("Error: {e}");
            return ExitCode::FAILURE;
        }
    }

MODULE_EXPORTS:
  - modify: src/cli/mod.rs
  - add: pub mod args; pub use args::*;
  - integration: Makes CLI types available to main.rs
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating args.rs - verify compilation
cargo check --package jin

# Verify CLI parsing
cargo run -- --help
# Expected: Shows help with all top-level commands

# Verify subcommand help
cargo run -- mode --help
# Expected: Shows mode subcommands

# Expected: Zero compilation errors. If errors exist:
# - Check clap derive attributes are correct
# - Verify all imports are present
# - Ensure struct names match enum variants

# Format check
cargo fmt --check
# Auto-format if needed
cargo fmt
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test CLI structure
cargo test --package jin cli::args --verbose

# Run specific tests
cargo test --package jin test_init_command -- --exact
cargo test --package jin test_add_command_with_files -- --exact
cargo test --package jin test_mode_create_command -- --exact
cargo test --package jin test_scope_create_with_mode -- --exact

# Expected: All tests pass. Verify:
# - Init command parses correctly
# - Add command with files parses correctly
# - All flags (--mode, --scope, --project, --global) parse correctly
# - Mode and Scope subcommands parse correctly
# - Help text renders for all commands

# Test coverage
cargo test --package jin --lib cli
# Expected: All CLI tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual CLI invocation
cargo run -- --help
# Expected output:
# Jin - Multi-layer Git overlay system
#
# Usage: jin [OPTIONS] <COMMAND>
#
# Commands:
#   init        Initialize Jin in the current project
#   add         Stage files to the appropriate layer
#   commit      Commit staged files
#   reset       Reset staged or committed changes
#   status      Show workspace state and status
#   mode        Mode management commands
#   modes       List all available modes
#   scope       Scope management commands
#   scopes      List all available scopes
#   apply       Merge layers into the workspace
#   diff        Show differences between layers or workspace
#   log         Show commit history for layers
#   context     Show active context
#   layers      Show current layer composition
#   list        List available modes/scopes/projects
#   import      Import Git-tracked files into Jin
#   export      Export Jin files back to Git
#   repair      Repair Jin state
#   link        Link to shared Jin config repository
#   fetch       Fetch updates from remote Jin repo
#   pull        Fetch and merge updates
#   push        Push local changes
#   sync        Fetch, merge, and apply
#   rm          Remove file from layer
#   mv          Rename/move file within layer
#   help        Print this message or the help of the given subcommand(s)
#
# Options:
#   -h, --help     Print help
#   -V, --version  Print version

# Test subcommand help
cargo run -- mode --help
# Expected output:
# Mode management commands
#
# Usage: jin mode <COMMAND>
#
# Commands:
#   create  Create a new mode
#   use     Activate a mode (set as active context)
#   unset   Deactivate the current mode
#   delete  Delete a mode
#   show    Show the current active mode
#   help    Print this message or the help of the given subcommand(s)

# Test command parsing
cargo run -- add file1.txt file2.txt --mode
# Expected: Parses AddCommand with files = ["file1.txt", "file2.txt"], mode = true

cargo run -- mode create claude
# Expected: Parses ModeCommand::Create { name: "claude" }

# Test error handling
cargo run -- invalid-command
# Expected: Error message "unrecognized subcommand 'invalid-command'"
```

### Level 4: Domain-Specific Validation

```bash
# Verify all PRD commands are implemented
cargo test --package jin test_all_prd_commands_defined -- --exact
# Asserts: Every command from PRD is in Commands enum

# Verify layer routing flags are consistent
cargo test --package jin test_layer_routing_flags_consistent -- --exact
# Asserts: --mode, --scope, --project, --global flags match Layer::from_flags()

# Verify mode/scope subcommands
cargo test --package jin test_mode_scope_subcommands_complete -- --exact
# Asserts: All mode/scope subcommands from PRD are defined

# Verify help text quality
cargo run -- --help | grep -E "(init|add|commit|mode|scope)"
# Expected: All top-level commands appear in help output

# Expected: All domain-specific validations pass
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo check --package jin` passes with zero errors
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] All tests pass: `cargo test --package jin --lib cli`
- [ ] `--help` shows all commands with proper descriptions
- [ ] Subcommand help shows all subcommands

### Feature Validation

- [ ] All PRD commands are defined in Commands enum
- [ ] Mode and Scope subcommands are complete
- [ ] Layer routing flags are defined consistently
- [ ] Help text is clear and descriptive
- [ ] All arguments have value_name where applicable
- [ ] Conflicting flags are properly marked

### Code Quality Validation

- [ ] Follows clap derive API patterns
- [ ] File placement matches desired tree structure
- [ ] All structs have doc comments
- [ ] Help text follows PRD specification
- [ ] Command names match PRD exactly

### Documentation & Deployment

- [ ] Module-level doc comment explains CLI structure
- [ ] Complex flag combinations have long_help
- [ ] All commands have about text
- [ ] Version is derived from Cargo.toml

---

## Anti-Patterns to Avoid

- ❌ Don't use clap builder API - use derive macros exclusively
- ❌ Don't reimplement Layer::from_flags() - use existing method
- ❌ Don't hardcode layer routing - delegate to core/layer.rs
- ❌ Don't skip help text - all commands need about/long_about
- ❌ Don't forget value_name for value arguments
- ❌ Don't make --mode take a value - it's a boolean flag
- ❌ Don't combine "modes" into ModeCommand - it's a separate command
- ❌ Don't create command handlers in this task - that's P4.M2-M4
- ❌ Don't use #[path] attributes - use conventional module structure
- ❌ Don't skip testing - verify all commands parse correctly

---

## Appendix: Complete Command Reference

### PRD Commands to Implement

| Command | Type | Description |
|---------|------|-------------|
| `jin init` | Top-level | Initialize Jin in current project |
| `jin add <files>` | Top-level | Stage files to appropriate layer |
| `jin commit -m "msg"` | Top-level | Commit staged files |
| `jin reset [paths]` | Top-level | Reset staged/committed changes |
| `jin status` | Top-level | Show workspace state |
| `jin mode create <mode>` | Mode subcommand | Create new mode |
| `jin mode use <mode>` | Mode subcommand | Activate mode |
| `jin mode unset` | Mode subcommand | Deactivate mode |
| `jin mode delete <mode>` | Mode subcommand | Delete mode |
| `jin mode show` | Mode subcommand | Show active mode |
| `jin modes` | Top-level | List all modes |
| `jin scope create <scope>` | Scope subcommand | Create new scope |
| `jin scope use <scope>` | Scope subcommand | Activate scope |
| `jin scope unset` | Scope subcommand | Deactivate scope |
| `jin scope delete <scope>` | Scope subcommand | Delete scope |
| `jin scope show` | Scope subcommand | Show active scope |
| `jin scopes` | Top-level | List all scopes |
| `jin apply` | Top-level | Merge layers into workspace |
| `jin diff [layer1] [layer2]` | Top-level | Show differences |
| `jin log [layer]` | Top-level | Show commit history |
| `jin context` | Top-level | Show active context |
| `jin import <files>` | Top-level | Import Git-tracked files |
| `jin export <files>` | Top-level | Export Jin files to Git |
| `jin repair` | Top-level | Repair Jin state |
| `jin layers` | Top-level | Show layer composition |
| `jin list` | Top-level | List modes/scopes/projects |
| `jin link <url>` | Top-level | Link to remote Jin repo |
| `jin fetch` | Top-level | Fetch from remote |
| `jin pull` | Top-level | Fetch and merge |
| `jin push` | Top-level | Push local changes |
| `jin sync` | Top-level | Fetch + merge + apply |
| `jin rm <files>` | Top-level | Remove from layer |
| `jin mv <old> <new>` | Top-level | Rename/move in layer |

### Layer Routing Flag Combinations

| Flags | Target Layer |
|-------|--------------|
| `--global` | GlobalBase |
| `--mode --scope --project` | ModeScopeProject |
| `--mode --project` | ModeProject |
| `--mode --scope` | ModeScope |
| `--scope` | ScopeBase |
| `--mode` | ModeBase |
| `--project` | ProjectBase |
| (none) | ProjectBase (default) |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 10/10 - High confidence in one-pass implementation success
