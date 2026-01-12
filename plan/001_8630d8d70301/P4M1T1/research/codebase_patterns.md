# Jin CLI Codebase Patterns & Conventions

This document documents the established patterns and conventions in the Jin CLI codebase for maintaining consistency when implementing new commands.

## 1. CLI Command Definition Patterns

### File: `src/cli/mod.rs`

#### Main CLI Structure
```rust
#[derive(Parser, Debug)]
#[command(name = "jin")]
#[command(author, version, about = "Phantom Git layer system for developer configuration")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
```

#### Command Enum Pattern
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // Simple commands without arguments
    Init,
    Status,
    Context,
    Fetch,
    Pull,
    Sync,
    Layers,
    List,
    
    // Commands with arguments
    Add(AddArgs),
    Commit(CommitArgs),
    Apply(ApplyArgs),
    
    // Nested subcommands
    #[command(subcommand)]
    Mode(ModeAction),
    
    #[command(subcommand)]
    Scope(ScopeAction),
}
```

### File: `src/cli/args.rs`

#### Argument Struct Pattern
```rust
#[derive(Args, Debug)]
pub struct AddArgs {
    // Positional arguments (required)
    pub files: Vec<String>,
    
    // Optional flags with long names
    #[arg(long)]
    pub mode: bool,
    
    // Optional flags with values
    #[arg(long)]
    pub scope: Option<String>,
    
    // Boolean flags
    #[arg(long)]
    pub global: bool,
}
```

## 2. Command Implementation Patterns

### File: `src/commands/mod.rs`

#### Command Dispatcher Pattern
```rust
pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => init::execute(),
        Commands::Add(args) => add::execute(args),
        Commands::Status => status::execute(),
        Commands::Mode(action) => mode::execute(action),
        // ... other commands
    }
}
```

### File: `src/commands/init.rs`

#### Simple Command Pattern
```rust
pub fn execute() -> Result<()> {
    // Check preconditions
    if ProjectContext::is_initialized() {
        println!("Jin is already initialized in this directory");
        return Ok(());
    }
    
    // Execute main logic
    let jin_dir = ProjectContext::default_path().parent().unwrap().to_path_buf();
    fs::create_dir_all(&jin_dir)?;
    
    // Create and save context
    let context = ProjectContext::default();
    context.save()?;
    
    // Output success message with next steps
    println!("Initialized Jin in {}", jin_dir.display());
    println!();
    println!("Next steps:");
    println!("  1. Create a mode:     jin mode create <name>");
    println!("  2. Activate the mode: jin mode use <name>");
    println!("  3. Add files:         jin add <file> --mode");
    
    Ok(())
}
```

### File: `src/commands/mode.rs`

#### Complex Command with Subcommands Pattern
```rust
pub fn execute(action: ModeAction) -> Result<()> {
    match action {
        ModeAction::Create { name } => create(&name),
        ModeAction::Use { name } => use_mode(&name),
        ModeAction::List => list(),
        ModeAction::Delete { name } => delete(&name),
        ModeAction::Show => show(),
        ModeAction::Unset => unset(),
    }
}

fn create(name: &str) -> Result<()> {
    // Validate input
    validate_mode_name(name)?;
    
    // Check if already exists
    if repo.ref_exists(&ref_path) {
        return Err(JinError::AlreadyExists(format!(
            "Mode '{}' already exists", name
        )));
    }
    
    // Execute main logic
    let empty_tree = repo.create_tree(&[])?;
    let commit_oid = repo.create_commit(None, &format!("Initialize mode: {}", name), empty_tree, &[])?;
    repo.set_ref(&ref_path, commit_oid, &format!("create mode {}", name))?;
    
    // Output success message
    println!("Created mode '{}'", name);
    println!("Activate with: jin mode use {}", name);
    
    Ok(())
}
```

### File: `src/commands/status.rs`

#### Read-Only Status Command Pattern
```rust
pub fn execute() -> Result<()> {
    // Check initialization
    if !ProjectContext::is_initialized() {
        return Err(JinError::NotInitialized);
    }
    
    // Load required data
    let context = ProjectContext::load()?;
    let staging = StagingIndex::load().unwrap_or_else(|_| StagingIndex::new());
    
    // Display structured information
    println!("Jin status:");
    println!();
    
    // Show active contexts
    match &context.mode {
        Some(mode) => println!("  Mode:  {} (active)", mode),
        None => println!("  Mode:  (none)"),
    }
    
    // Show staged files with count
    let staged_count = staging.len();
    if staged_count == 0 {
        println!("No staged changes.");
        println!();
        println!("Use 'jin add <file>' to stage files for commit.");
    } else {
        println!("Staged changes ({} file{}):", staged_count, if staged_count == 1 { "" } else { "s" });
        for entry in staging.entries() {
            println!("  {} -> {}", entry.path.display(), entry.target_layer);
        }
        println!();
        println!("Use 'jin commit -m <message>' to commit staged changes.");
    }
    
    Ok(())
}
```

## 3. Entry Point Patterns

### File: `src/main.rs`
```rust
fn main() -> anyhow::Result<()> {
    let cli = jin::cli::Cli::parse();
    jin::run(cli)
}
```

### File: `src/lib.rs`
```rust
pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    commands::execute(cli).map_err(|e| anyhow::anyhow!("{}", e))
}
```

## 4. Error Handling Patterns

### Error Types: `src/core/error.rs`

#### Standard Error Pattern
```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Jin not initialized in this project")]
    NotInitialized,
    
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, JinError>;
```

#### Error Usage Pattern
```rust
// Check preconditions and return error
if !ProjectContext::is_initialized() {
    return Err(JinError::NotInitialized);
}

// Check for existence and return specific error
if !repo.ref_exists(&ref_path) {
    return Err(JinError::NotFound(format!(
        "Mode '{}' not found. Create it with: jin mode create {}",
        name, name
    )));
}

// Handle context loading errors
let mut context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
    Err(_) => ProjectContext::default(), // Fallback
};
```

## 5. Output Patterns

### Success Messages
- Always use `println!` for user-facing output
- Include helpful next steps after successful operations
- Use consistent formatting with indentation for lists

```rust
println!("Created mode '{}'", name);
println!("Activate with: jin mode use {}", name);

println!("Initialized Jin in {}", jin_dir.display());
println!();
println!("Next steps:");
println!("  1. Create a mode:     jin mode create <name>");
println!("  2. Activate the mode: jin mode use <name>");
println!("  3. Add files:         jin add <file> --mode");
```

### Status/Information Output
- Use structured formatting with consistent indentation
- Show counts with proper pluralization
- Include helpful action suggestions

```rust
let staged_count = staging.len();
println!("Staged changes ({} file{}):", staged_count, if staged_count == 1 { "" } else { "s" });
```

## 6. Test Patterns

### File: `tests/cli_basic.rs`

#### Test Helper Pattern
```rust
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}
```

#### Command Test Pattern
```rust
#[test]
fn test_init_subcommand() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();
    
    // Set up isolated environment
    std::env::set_var("JIN_DIR", temp.path().join(".jin_global"));
    
    jin()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Jin"));
}
```

#### Error Test Pattern
```rust
#[test]
fn test_status_subcommand() {
    let temp = TempDir::new().unwrap();
    
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", temp.path().join(".jin_global"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}
```

## 7. Project Structure

### Cargo.toml Configuration
```toml
[package]
name = "jin"
version = "0.1.0"
edition = "2021"

[lib]
name = "jin"
path = "src/lib.rs"

[[bin]]
name = "jin"
path = "src/main.rs"
```

### Directory Structure
```
src/
├── cli/
│   ├── mod.rs          # Command definitions
│   └── args.rs         # Argument structs
├── commands/
│   ├── mod.rs          # Command dispatcher
│   ├── init.rs         # Simple command
│   ├── mode.rs         # Complex command with subcommands
│   ├── status.rs       # Read-only status
│   └── ...             # Other commands
├── core/
│   ├── error.rs        # Error types
│   └── ...             # Core functionality
├── lib.rs              # Library entry point
└── main.rs             # Binary entry point
```

## 8. Must Follow Patterns for Implementation

### Command Definition
1. Always use `#[derive(Subcommand, Debug)]` for command enums
2. Use `#[derive(Args, Debug)]` for argument structs
3. Place simple commands first in the enum
4. Group related subcommands under their own enums
5. Use descriptive docstrings for all commands and arguments

### Command Implementation
1. Always return `Result<()>`
2. Check preconditions early (initialization, existence, etc.)
3. Validate all input parameters
4. Use meaningful error messages with suggestions
5. Always include success messages with next steps
6. Use consistent error handling with `match` for context loading
7. Include comprehensive tests covering success and error cases

### Error Handling
1. Always use `JinError` types, never plain `Result`
2. Provide specific error messages with actionable suggestions
3. Handle `NotInitialized` consistently across commands
4. Use proper error chaining with `#[from]` attributes
5. Include fallback handling for recoverable errors

### Output Patterns
1. Always use `println!` for user output
2. Include helpful next steps after successful operations
3. Use consistent indentation for lists and structured output
4. Show counts with proper pluralization
5. Format paths consistently using `display()` method

### Testing
1. Use `tempfile::TempDir` for isolated test environments
2. Set `JIN_DIR` environment variable for isolation
3. Test both success and error cases
4. Use `assert_cmd::Command` for CLI testing
5. Verify both stdout and stderr as appropriate
6. Include edge case tests and validation

### Dependencies
1. All commands must use the established `Result<T, JinError>` type
2. Always use `ProjectContext::is_initialized()` for initialization checks
3. Use proper context loading with error handling
4. Follow the established Git repository patterns in `git/repo.rs`

