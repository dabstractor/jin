# PRP: P1.M1 - Project Scaffolding

---

## Goal

**Feature Goal**: Create a complete, compilable Rust project structure for Jin - a phantom Git layer system for managing developer-specific configuration without contaminating the main Git repository.

**Deliverable**: A fully structured Rust CLI project with:
1. Valid `Cargo.toml` with all required dependencies
2. Complete module directory structure (`src/core/`, `src/git/`, `src/merge/`, `src/staging/`, `src/commit/`, `src/cli/`, `src/commands/`)
3. `JinError` enum using `thiserror` for comprehensive error handling
4. `Layer` enum implementing the 9-layer hierarchy with precedence and routing logic
5. `JinConfig` and `ProjectContext` structs for configuration management
6. Basic CLI skeleton using `clap` with all planned commands stubbed

**Success Definition**:
- `cargo check` passes with zero errors
- `cargo build` produces a working binary
- `cargo test` passes (with placeholder tests)
- All module imports resolve correctly
- The binary executes and shows help output

---

## User Persona

**Target User**: Developer using Jin to manage tool-specific configuration files

**Use Case**: Setting up Jin infrastructure that will later support commands like `jin init`, `jin add`, `jin commit`, `jin mode use`, etc.

**User Journey**: This is foundational infrastructure - no direct user interaction in this milestone. Developers will interact with the CLI built on top of this scaffold.

**Pain Points Addressed**: Establishes the type system and module boundaries that prevent architectural debt and enable clean feature implementation.

---

## Why

- **Foundation for All Features**: Every subsequent milestone (Git integration, merge engine, staging, CLI commands) depends on this scaffold
- **Type-Safe Layer System**: The 9-layer hierarchy is Jin's core innovation; implementing it as Rust types ensures compile-time correctness
- **Error Handling Consistency**: A unified `JinError` type enables consistent error propagation across all modules
- **Configuration Management**: `JinConfig` and `ProjectContext` enable the active context system (mode/scope) that determines layer routing

---

## What

### User-Visible Behavior

After this milestone:
```bash
# Project compiles and builds
cargo build

# Binary exists and shows help
./target/debug/jin --help
# Output shows all command stubs

# All tests pass
cargo test
```

### Technical Requirements

1. **Cargo.toml**: All dependencies with exact versions
2. **Module Structure**: Clean separation of concerns
3. **JinError**: Variants for IO, Git, Config, Parse, Merge, Transaction, and general errors
4. **Layer Enum**: 9 variants with `precedence()`, `ref_path()`, `storage_path()` methods
5. **Config Types**: Serializable with `serde`, loadable from TOML/YAML

### Success Criteria

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` runs with all tests passing
- [ ] `cargo clippy` reports no warnings
- [ ] Binary executes: `./target/debug/jin --help` shows CLI structure
- [ ] All 9 Layer variants implemented with correct precedence
- [ ] JinError covers all error categories from PRD
- [ ] Config types match `.jin/context` schema from PRD Section 7.1

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement Jin's project scaffold from scratch, including exact dependency versions, module structure, type definitions, and validation commands._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Project Requirements
- file: PRD.md
  why: Contains complete system specification, 9-layer hierarchy, command list, invariants
  sections:
    - "Section 4.1 - Nine-Layer Hierarchy" for Layer enum definition
    - "Section 6 - Core API Contract" for staging/commit atomicity requirements
    - "Section 7.1 - Context Rules" for ProjectContext schema
    - "Section 16 - .jinmap" for configuration file format
    - "Section 18 - Core Commands" for CLI command list
    - "Section 25 - Non-Negotiable Invariants" for design constraints

# Rust CLI Best Practices
- url: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
  why: Clap derive API tutorial - use this pattern for CLI definition
  critical: Use struct with Command enum, not App as enum

- url: https://docs.rs/thiserror/latest/thiserror/
  why: Error derive macro usage for JinError
  critical: Use #[error("message")] and #[from] attributes

- url: https://docs.rs/git2/latest/git2/
  why: Git2 crate API for phantom Git layer
  critical: Repository, Reference, Oid types

- url: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
  why: Module organization patterns
  critical: Use mod.rs in directories, pub use for re-exports

- url: https://rust-cli.github.io/book/tutorial/testing.html
  why: CLI testing patterns with assert_cmd
  critical: Use Command::cargo_bin for integration tests

# Error Handling Patterns
- url: https://markaicode.com/rust-error-handling-2025-guide/
  why: Modern error handling patterns for CLI apps
  critical: thiserror for library code, anyhow for CLI layer

# Configuration Management
- url: https://docs.rs/config/latest/config/
  why: Layered configuration loading pattern
  critical: Config builder pattern for layered sources
```

### Current Codebase Tree

```bash
jin/
├── PRD.md              # Product requirements document (857 lines)
├── tasks.json          # Task tracking structure
├── plan/
│   ├── P1M1/
│   │   ├── PRP.md      # This file
│   │   └── research/   # Research artifacts
│   └── P1M3/
│       └── research/
└── tests/              # Empty - tests to be added
```

### Desired Codebase Tree After P1.M1

```bash
jin/
├── Cargo.toml                    # Package manifest with all dependencies
├── Cargo.lock                    # Generated lock file
├── PRD.md                        # Product requirements (unchanged)
├── tasks.json                    # Task tracking (unchanged)
├── src/
│   ├── main.rs                   # Entry point - minimal, calls lib
│   ├── lib.rs                    # Library root - re-exports all modules
│   │
│   ├── core/                     # Core types and infrastructure
│   │   ├── mod.rs                # Module exports
│   │   ├── error.rs              # JinError enum definition
│   │   ├── layer.rs              # Layer enum with 9 variants
│   │   └── config.rs             # JinConfig, ProjectContext structs
│   │
│   ├── git/                      # Git integration layer
│   │   ├── mod.rs                # Module exports
│   │   ├── repo.rs               # JinRepo wrapper (stub)
│   │   ├── refs.rs               # Reference operations (stub)
│   │   ├── objects.rs            # Object creation (stub)
│   │   └── transaction.rs        # Atomic transactions (stub)
│   │
│   ├── merge/                    # Merge engine
│   │   ├── mod.rs                # Module exports
│   │   ├── value.rs              # MergeValue enum (stub)
│   │   ├── deep.rs               # Deep merge logic (stub)
│   │   ├── text.rs               # 3-way text merge (stub)
│   │   └── layer.rs              # Layer merge orchestration (stub)
│   │
│   ├── staging/                  # Staging system
│   │   ├── mod.rs                # Module exports
│   │   ├── entry.rs              # StagedEntry type (stub)
│   │   ├── index.rs              # StagingIndex type (stub)
│   │   └── router.rs             # Layer routing logic (stub)
│   │
│   ├── commit/                   # Commit pipeline
│   │   ├── mod.rs                # Module exports
│   │   └── pipeline.rs           # CommitPipeline type (stub)
│   │
│   ├── cli/                      # CLI argument definitions
│   │   ├── mod.rs                # Cli struct and Commands enum
│   │   └── args.rs               # Shared argument types
│   │
│   └── commands/                 # Command implementations
│       ├── mod.rs                # Command dispatch
│       ├── init.rs               # jin init (stub)
│       ├── add.rs                # jin add (stub)
│       ├── commit_cmd.rs         # jin commit (stub)
│       ├── status.rs             # jin status (stub)
│       ├── mode.rs               # jin mode subcommands (stub)
│       ├── scope.rs              # jin scope subcommands (stub)
│       ├── apply.rs              # jin apply (stub)
│       ├── reset.rs              # jin reset (stub)
│       ├── diff.rs               # jin diff (stub)
│       ├── log.rs                # jin log (stub)
│       ├── context.rs            # jin context (stub)
│       ├── import_cmd.rs         # jin import (stub)
│       ├── export.rs             # jin export (stub)
│       ├── repair.rs             # jin repair (stub)
│       ├── layers.rs             # jin layers (stub)
│       ├── list.rs               # jin list (stub)
│       ├── link.rs               # jin link (stub)
│       ├── fetch.rs              # jin fetch (stub)
│       ├── pull.rs               # jin pull (stub)
│       ├── push.rs               # jin push (stub)
│       └── sync.rs               # jin sync (stub)
│
├── tests/
│   └── integration/
│       └── cli_basic.rs          # Basic CLI integration tests
│
└── plan/                         # Unchanged
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2 crate requires libgit2 system library
// On Linux: apt install libgit2-dev
// On macOS: brew install libgit2
// Use "vendored-openssl" feature to avoid OpenSSL issues

// CRITICAL: clap derive requires "derive" feature enabled
// clap = { version = "4.5", features = ["derive"] }

// CRITICAL: serde requires "derive" feature for struct serialization
// serde = { version = "1.0", features = ["derive"] }

// GOTCHA: thiserror #[from] attribute requires From<SourceError>
// Example: #[error("Git error")] Git(#[from] git2::Error)

// GOTCHA: indexmap preserves insertion order for JSON/YAML object merging
// Use IndexMap<String, Value> instead of HashMap for config objects

// GOTCHA: Layer paths use forward slashes even on Windows
// jin/mode/claude/ not jin\mode\claude\

// PATTERN: Active context stored in .jin/context as YAML
// version: 1
// mode: claude
// scope: language:javascript

// PATTERN: JinConfig stored at ~/.jin/config.toml (global)
// ProjectContext stored at .jin/context (per-project YAML)
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/core/error.rs ==================
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JinError {
    // IO and filesystem errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Git operations
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    // Parse errors (JSON, YAML, TOML, INI)
    #[error("Parse error in {format}: {message}")]
    Parse { format: String, message: String },

    // Merge conflicts
    #[error("Merge conflict in {path}")]
    MergeConflict { path: String },

    // Transaction failures
    #[error("Transaction failed: {0}")]
    Transaction(String),

    // Layer routing errors
    #[error("Invalid layer: {0}")]
    InvalidLayer(String),

    // Context errors
    #[error("No active {context_type}")]
    NoActiveContext { context_type: String },

    // File not found
    #[error("File not found: {0}")]
    NotFound(String),

    // Already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    // Not initialized
    #[error("Jin not initialized in this project")]
    NotInitialized,

    // General errors
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, JinError>;


// ================== src/core/layer.rs ==================
use serde::{Deserialize, Serialize};

/// The 9-layer hierarchy for Jin configuration.
/// Precedence flows bottom (1) to top (9) - higher overrides lower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// Layer 1: Shared defaults (jin/global/)
    GlobalBase,
    /// Layer 2: Mode defaults (jin/mode/<mode>/)
    ModeBase,
    /// Layer 3: Scoped mode configs (jin/mode/<mode>/scope/<scope>/)
    ModeScope,
    /// Layer 4: Project overrides for scoped mode
    ModeScopeProject,
    /// Layer 5: Project overrides for mode (jin/mode/<mode>/project/<project>/)
    ModeProject,
    /// Layer 6: Untethered scope configs (jin/scope/<scope>/)
    ScopeBase,
    /// Layer 7: Project-only configs (jin/project/<project>/)
    ProjectBase,
    /// Layer 8: Machine-only overlays (~/.jin/local/)
    UserLocal,
    /// Layer 9: Derived merge result (.jin/workspace/)
    WorkspaceActive,
}

impl Layer {
    /// Returns the precedence level (1-9, higher overrides lower)
    pub fn precedence(&self) -> u8 {
        match self {
            Layer::GlobalBase => 1,
            Layer::ModeBase => 2,
            Layer::ModeScope => 3,
            Layer::ModeScopeProject => 4,
            Layer::ModeProject => 5,
            Layer::ScopeBase => 6,
            Layer::ProjectBase => 7,
            Layer::UserLocal => 8,
            Layer::WorkspaceActive => 9,
        }
    }

    /// Returns the Git ref path for this layer
    pub fn ref_path(&self, mode: Option<&str>, scope: Option<&str>, project: Option<&str>) -> String {
        match self {
            Layer::GlobalBase => "refs/jin/layers/global".to_string(),
            Layer::ModeBase => format!("refs/jin/layers/mode/{}", mode.unwrap_or("default")),
            Layer::ModeScope => format!(
                "refs/jin/layers/mode/{}/scope/{}",
                mode.unwrap_or("default"),
                scope.unwrap_or("default")
            ),
            Layer::ModeScopeProject => format!(
                "refs/jin/layers/mode/{}/scope/{}/project/{}",
                mode.unwrap_or("default"),
                scope.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ModeProject => format!(
                "refs/jin/layers/mode/{}/project/{}",
                mode.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ScopeBase => format!("refs/jin/layers/scope/{}", scope.unwrap_or("default")),
            Layer::ProjectBase => format!("refs/jin/layers/project/{}", project.unwrap_or("default")),
            Layer::UserLocal => "refs/jin/layers/local".to_string(),
            Layer::WorkspaceActive => "refs/jin/layers/workspace".to_string(),
        }
    }

    /// Returns the storage directory path for this layer
    pub fn storage_path(&self, mode: Option<&str>, scope: Option<&str>, project: Option<&str>) -> String {
        match self {
            Layer::GlobalBase => "jin/global/".to_string(),
            Layer::ModeBase => format!("jin/mode/{}/", mode.unwrap_or("default")),
            Layer::ModeScope => format!(
                "jin/mode/{}/scope/{}/",
                mode.unwrap_or("default"),
                scope.unwrap_or("default")
            ),
            Layer::ModeScopeProject => format!(
                "jin/mode/{}/scope/{}/project/{}/",
                mode.unwrap_or("default"),
                scope.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ModeProject => format!(
                "jin/mode/{}/project/{}/",
                mode.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ScopeBase => format!("jin/scope/{}/", scope.unwrap_or("default")),
            Layer::ProjectBase => format!("jin/project/{}/", project.unwrap_or("default")),
            Layer::UserLocal => "~/.jin/local/".to_string(),
            Layer::WorkspaceActive => ".jin/workspace/".to_string(),
        }
    }

    /// Returns all layers in precedence order (lowest to highest)
    pub fn all_in_precedence_order() -> Vec<Layer> {
        vec![
            Layer::GlobalBase,
            Layer::ModeBase,
            Layer::ModeScope,
            Layer::ModeScopeProject,
            Layer::ModeProject,
            Layer::ScopeBase,
            Layer::ProjectBase,
            Layer::UserLocal,
            Layer::WorkspaceActive,
        ]
    }
}


// ================== src/core/config.rs ==================
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Global Jin configuration (stored at ~/.jin/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JinConfig {
    /// Version of the config schema
    #[serde(default = "default_version")]
    pub version: u32,

    /// Remote repository URL for sync
    pub remote: Option<RemoteConfig>,

    /// User information
    pub user: Option<UserConfig>,
}

fn default_version() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl JinConfig {
    /// Load config from default location (~/.jin/config.toml)
    pub fn load() -> crate::core::error::Result<Self> {
        let path = Self::default_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content).map_err(|e| {
                crate::core::error::JinError::Config(format!("Failed to parse config: {}", e))
            })
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to default location
    pub fn save(&self) -> crate::core::error::Result<()> {
        let path = Self::default_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::core::error::JinError::Config(format!("Failed to serialize config: {}", e))
        })?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns default config path (~/.jin/config.toml)
    pub fn default_path() -> crate::core::error::Result<PathBuf> {
        dirs::home_dir()
            .map(|h| h.join(".jin").join("config.toml"))
            .ok_or_else(|| crate::core::error::JinError::Config("Cannot determine home directory".into()))
    }
}

/// Per-project context (stored at .jin/context)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    /// Version of the context schema
    #[serde(default = "default_version")]
    pub version: u32,

    /// Currently active mode
    pub mode: Option<String>,

    /// Currently active scope
    pub scope: Option<String>,

    /// Project name (auto-inferred from Git remote)
    pub project: Option<String>,

    /// Last update timestamp
    #[serde(default)]
    pub last_updated: Option<String>,
}

impl ProjectContext {
    /// Load context from .jin/context in current directory
    pub fn load() -> crate::core::error::Result<Self> {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_yaml::from_str(&content).map_err(|e| {
                crate::core::error::JinError::Config(format!("Failed to parse context: {}", e))
            })
        } else {
            Err(crate::core::error::JinError::NotInitialized)
        }
    }

    /// Save context to .jin/context
    pub fn save(&self) -> crate::core::error::Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self).map_err(|e| {
            crate::core::error::JinError::Config(format!("Failed to serialize context: {}", e))
        })?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns default context path (.jin/context)
    pub fn default_path() -> PathBuf {
        PathBuf::from(".jin").join("context")
    }

    /// Check if Jin is initialized in current directory
    pub fn is_initialized() -> bool {
        Self::default_path().parent().map(|p| p.exists()).unwrap_or(false)
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE Cargo.toml
  - IMPLEMENT: Package manifest with all dependencies
  - DEPENDENCIES (with exact versions):
    # CLI
    clap = { version = "4.5", features = ["derive", "cargo"] }

    # Git operations
    git2 = { version = "0.19", default-features = false, features = ["vendored-libgit2"] }

    # Error handling
    thiserror = "2.0"
    anyhow = "1.0"

    # Serialization
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    serde_yaml = "0.9"
    toml = "0.8"

    # Data structures
    indexmap = { version = "2.0", features = ["serde"] }

    # Utilities
    dirs = "5.0"
    chrono = { version = "0.4", features = ["serde"] }

    # [dev-dependencies]
    assert_cmd = "2.0"
    predicates = "3.0"
    tempfile = "3.0"
  - OUTPUT: Valid Cargo.toml that passes `cargo check`

Task 2: CREATE src/main.rs
  - IMPLEMENT: Minimal entry point
  - PATTERN: Parse CLI args, call library execute function
  - CONTENT:
    fn main() -> anyhow::Result<()> {
        let cli = jin::cli::Cli::parse();
        jin::run(cli)
    }
  - OUTPUT: Working entry point

Task 3: CREATE src/lib.rs
  - IMPLEMENT: Library root with module declarations
  - EXPORTS: pub mod core, git, merge, staging, commit, cli, commands
  - EXPORTS: pub use core::{error::Result, layer::Layer}
  - FUNCTION: pub fn run(cli: cli::Cli) -> anyhow::Result<()>
  - OUTPUT: Library root that re-exports all modules

Task 4: CREATE src/core/mod.rs, error.rs, layer.rs, config.rs
  - IMPLEMENT: Core types as defined in Data Models section
  - EXPORTS: pub mod error, layer, config
  - EXPORTS: pub use error::{JinError, Result}
  - EXPORTS: pub use layer::Layer
  - EXPORTS: pub use config::{JinConfig, ProjectContext}
  - OUTPUT: Core module with error, layer, config types

Task 5: CREATE src/git/mod.rs and stubs
  - IMPLEMENT: Module structure with stub implementations
  - FILES: mod.rs, repo.rs, refs.rs, objects.rs, transaction.rs
  - STUBS: JinRepo, RefOps trait, create_blob, create_tree, create_commit, Transaction
  - OUTPUT: Compilable git module with stubs

Task 6: CREATE src/merge/mod.rs and stubs
  - IMPLEMENT: Module structure with stub implementations
  - FILES: mod.rs, value.rs, deep.rs, text.rs, layer.rs
  - STUBS: MergeValue enum, deep_merge fn, text_merge fn, merge_layers fn
  - OUTPUT: Compilable merge module with stubs

Task 7: CREATE src/staging/mod.rs and stubs
  - IMPLEMENT: Module structure with stub implementations
  - FILES: mod.rs, entry.rs, index.rs, router.rs
  - STUBS: StagedEntry, StagingIndex, route_to_layer fn
  - OUTPUT: Compilable staging module with stubs

Task 8: CREATE src/commit/mod.rs and pipeline.rs stub
  - IMPLEMENT: Module structure with stub implementation
  - FILES: mod.rs, pipeline.rs
  - STUBS: CommitPipeline struct with execute() method
  - OUTPUT: Compilable commit module with stub

Task 9: CREATE src/cli/mod.rs and args.rs
  - IMPLEMENT: CLI structure using clap derive
  - PATTERN: Struct Cli with Commands enum (see Implementation Patterns)
  - COMMANDS: All commands from PRD Section 18
  - SUBCOMMANDS: mode (create/use/list/delete/show/unset), scope (same)
  - OUTPUT: Complete CLI definition that compiles

Task 10: CREATE src/commands/mod.rs and all command stubs
  - IMPLEMENT: Command implementation stubs
  - FILES: See Desired Codebase Tree for complete list
  - PATTERN: pub struct XxxCommand with pub fn execute() -> Result<()>
  - OUTPUT: All command files with stub implementations

Task 11: CREATE tests/integration/cli_basic.rs
  - IMPLEMENT: Basic CLI integration tests
  - TESTS: --help works, --version works, subcommands parse correctly
  - PATTERN: Use assert_cmd::Command::cargo_bin
  - OUTPUT: Passing integration tests
```

### Implementation Patterns & Key Details

```rust
// ================== src/cli/mod.rs ==================
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "jin")]
#[command(author, version, about = "Phantom Git layer system for developer configuration")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize Jin in current project
    Init,

    /// Stage files to appropriate layer
    Add(AddArgs),

    /// Commit staged files atomically
    Commit(CommitArgs),

    /// Show workspace state and active contexts
    Status,

    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),

    /// Scope lifecycle management
    #[command(subcommand)]
    Scope(ScopeAction),

    /// Apply merged layers to workspace
    Apply(ApplyArgs),

    /// Reset staged or committed changes
    Reset(ResetArgs),

    /// Show differences between layers
    Diff(DiffArgs),

    /// Show commit history
    Log(LogArgs),

    /// Show/set active context
    Context,

    /// Import Git-tracked files into Jin
    Import(ImportArgs),

    /// Export Jin files back to Git
    Export(ExportArgs),

    /// Repair Jin state
    Repair(RepairArgs),

    /// Show current layer composition
    Layers,

    /// List available modes/scopes/projects
    List,

    /// Link to shared Jin config repo
    Link(LinkArgs),

    /// Fetch updates from remote
    Fetch,

    /// Fetch and merge updates
    Pull,

    /// Push local changes
    Push(PushArgs),

    /// Fetch + merge + apply
    Sync,
}

#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create { name: String },
    /// Activate a mode
    Use { name: String },
    /// List available modes
    List,
    /// Delete a mode
    Delete { name: String },
    /// Show current mode
    Show,
    /// Deactivate current mode
    Unset,
}

#[derive(Subcommand, Debug)]
pub enum ScopeAction {
    /// Create a new scope
    Create {
        name: String,
        #[arg(long)]
        mode: Option<String>,
    },
    /// Activate a scope
    Use { name: String },
    /// List available scopes
    List,
    /// Delete a scope
    Delete { name: String },
    /// Show current scope
    Show,
    /// Deactivate current scope
    Unset,
}

// Argument structs for commands with options
#[derive(clap::Args, Debug)]
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

#[derive(clap::Args, Debug)]
pub struct CommitArgs {
    /// Commit message
    #[arg(short, long)]
    pub message: String,

    /// Dry run - show what would be committed
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(clap::Args, Debug)]
pub struct ApplyArgs {
    /// Force apply even if workspace is dirty
    #[arg(long)]
    pub force: bool,

    /// Show what would be applied
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(clap::Args, Debug)]
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

#[derive(clap::Args, Debug)]
pub struct DiffArgs {
    /// First layer to compare
    pub layer1: Option<String>,

    /// Second layer to compare
    pub layer2: Option<String>,

    /// Show staged changes
    #[arg(long)]
    pub staged: bool,
}

#[derive(clap::Args, Debug)]
pub struct LogArgs {
    /// Layer to show history for
    #[arg(long)]
    pub layer: Option<String>,

    /// Number of entries to show
    #[arg(long, default_value = "10")]
    pub count: usize,
}

#[derive(clap::Args, Debug)]
pub struct ImportArgs {
    /// Files to import from Git
    pub files: Vec<String>,

    /// Force import even if files are modified
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args, Debug)]
pub struct ExportArgs {
    /// Files to export back to Git
    pub files: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RepairArgs {
    /// Show what would be repaired
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(clap::Args, Debug)]
pub struct LinkArgs {
    /// Remote repository URL
    pub url: String,
}

#[derive(clap::Args, Debug)]
pub struct PushArgs {
    /// Force push (overwrite remote)
    #[arg(long)]
    pub force: bool,
}


// ================== src/commands/mod.rs ==================
use crate::cli::{Commands, Cli};
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
pub mod import_cmd;
pub mod export;
pub mod repair;
pub mod layers;
pub mod list;
pub mod link;
pub mod fetch;
pub mod pull;
pub mod push;
pub mod sync;

pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => init::execute(),
        Commands::Add(args) => add::execute(args),
        Commands::Commit(args) => commit_cmd::execute(args),
        Commands::Status => status::execute(),
        Commands::Mode(action) => mode::execute(action),
        Commands::Scope(action) => scope::execute(action),
        Commands::Apply(args) => apply::execute(args),
        Commands::Reset(args) => reset::execute(args),
        Commands::Diff(args) => diff::execute(args),
        Commands::Log(args) => log::execute(args),
        Commands::Context => context::execute(),
        Commands::Import(args) => import_cmd::execute(args),
        Commands::Export(args) => export::execute(args),
        Commands::Repair(args) => repair::execute(args),
        Commands::Layers => layers::execute(),
        Commands::List => list::execute(),
        Commands::Link(args) => link::execute(args),
        Commands::Fetch => fetch::execute(),
        Commands::Pull => pull::execute(),
        Commands::Push(args) => push::execute(args),
        Commands::Sync => sync::execute(),
    }
}


// ================== COMMAND STUB PATTERN ==================
// src/commands/init.rs (example - all others follow this pattern)
use crate::core::Result;

pub fn execute() -> Result<()> {
    // TODO: Implement in P4.M2.T1
    println!("jin init - not yet implemented");
    Ok(())
}
```

### Integration Points

```yaml
FILESYSTEM:
  - JinConfig stored at: ~/.jin/config.toml (TOML format)
  - ProjectContext stored at: .jin/context (YAML format)
  - Workspace files at: .jin/workspace/
  - Staging index at: .jin/staging/

GIT:
  - Jin repository at: ~/.jin/ (bare repo)
  - Layer refs at: refs/jin/layers/*
  - Transaction refs at: refs/jin/staging/*

DEPENDENCIES:
  - git2 requires libgit2 (use vendored-libgit2 feature)
  - serde_yaml for .jin/context
  - toml for config.toml
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check - treat warnings as errors

# Expected: Zero errors, zero warnings
# If errors: READ output carefully, fix the specific issue, re-run
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build
cargo build --release                 # Release build (catches optimization issues)

# Binary execution test
./target/debug/jin --help             # Should show help text
./target/debug/jin --version          # Should show version

# Expected: Clean build, working binary
```

### Level 3: Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test core::                     # Test core module
cargo test layer::                    # Test Layer enum

# With output
cargo test -- --nocapture             # See println! output

# Expected: All tests pass
```

### Level 4: Integration Tests

```bash
# CLI integration tests
cargo test --test cli_basic

# Test specific command parsing
./target/debug/jin mode create test-mode
./target/debug/jin scope create test-scope --mode=test-mode
./target/debug/jin add --help
./target/debug/jin commit --help

# Expected: All commands parse without error, show appropriate output
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
- [ ] Binary executes: `./target/debug/jin --help` works

### Feature Validation

- [ ] All 9 Layer variants implemented with correct precedence ordering
- [ ] `Layer::precedence()` returns 1-9 in correct order
- [ ] `Layer::ref_path()` generates correct Git ref paths
- [ ] `Layer::storage_path()` generates correct storage paths
- [ ] JinError covers: Io, Git, Config, Parse, MergeConflict, Transaction, InvalidLayer, NoActiveContext, NotFound, AlreadyExists, NotInitialized, Other
- [ ] JinConfig loads/saves TOML at ~/.jin/config.toml
- [ ] ProjectContext loads/saves YAML at .jin/context
- [ ] CLI shows all commands from PRD Section 18
- [ ] All subcommands (mode/scope) properly defined

### Code Quality Validation

- [ ] Module structure matches Desired Codebase Tree
- [ ] All modules have proper pub exports in mod.rs
- [ ] No unused imports or dead code warnings
- [ ] Error types use thiserror #[error] attributes correctly
- [ ] serde derives present on all config types
- [ ] #[non_exhaustive] on JinError for future compatibility

### Documentation & Deployment

- [ ] Cargo.toml has proper package metadata (name, version, authors)
- [ ] lib.rs has module-level documentation
- [ ] Key types have doc comments

---

## Anti-Patterns to Avoid

- ❌ Don't use `unwrap()` or `expect()` in library code - use `?` and proper errors
- ❌ Don't use `HashMap` for config objects - use `IndexMap` to preserve order
- ❌ Don't hardcode paths - use `dirs` crate for home directory
- ❌ Don't forget `#[derive(Debug)]` on types - needed for error messages
- ❌ Don't mix sync/async - this milestone is purely synchronous
- ❌ Don't implement command logic - only stubs in this milestone
- ❌ Don't skip the `#[non_exhaustive]` attribute on JinError
- ❌ Don't use `String` where `&str` would work in Layer methods

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Clear, specific dependency versions provided
- Complete code examples for core types (JinError, Layer, Config)
- Detailed CLI structure with all argument types defined
- Step-by-step task ordering with dependencies
- Comprehensive validation commands
- All gotchas and patterns documented

**Remaining Risks:**
- git2 vendored build may have platform-specific issues (mitigated by feature flag)
- Some edge cases in Layer path generation may need refinement in later milestones

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M1/research/`

Key external references:
- Clap Derive Tutorial: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
- thiserror Documentation: https://docs.rs/thiserror/latest/thiserror/
- Rust Module Organization: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
- CLI Testing Patterns: https://rust-cli.github.io/book/tutorial/testing.html
- Config Crate: https://docs.rs/config/latest/config/
