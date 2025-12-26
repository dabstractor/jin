# Product Requirement Prompt (PRP): Init Command (P4.M2.T1)

---

## Goal

**Feature Goal**: Implement the `jin init` command that initializes Jin in a project directory by creating the necessary directory structure, configuration files, and setting up the initial state.

**Deliverable**: A working `jin init` command that:
- Creates `.jin/` directory with proper structure
- Initializes `.jin/context` with default ProjectContext
- Creates `.jin/staging/index.json` with empty staging index
- Creates `.jin/workspace/` directory for applied files
- Optionally adds Jin entries to `.gitignore`
- Provides clear user feedback on initialization

**Success Definition**:
- Running `jin init` in a directory creates all required Jin structure
- Command is idempotent (can be run again safely)
- Clear error messages if already initialized or other issues
- All files and directories are created with correct structure
- `cargo test` passes all init command tests
- Command integrates properly with existing CLI infrastructure

## User Persona

**Target User**: Developer using Jin for the first time in a project

**Use Case**: Developer wants to start using Jin in their existing project to manage layered configurations

**User Journey**:
1. Developer navigates to their project directory
2. Runs `jin init`
3. Jin creates the necessary directory structure
4. Developer sees confirmation message with what was created
5. Developer can now use other Jin commands (`jin add`, `jin commit`, etc.)

**Pain Points Addressed**:
- No manual directory creation needed
- No need to understand Jin's internal structure to get started
- Clear feedback on what was initialized
- Safe to run if already initialized (friendly error or no-op)

## Why

- **Entry point for using Jin**: First command users run to start using Jin in a project
- **Foundation for all Jin operations**: Creates the structure that all other commands depend on
- **Onboarding simplification**: Single command sets up everything needed
- **Problems this solves**:
  - Eliminates manual setup of Jin directory structure
  - Ensures consistent initialization across all projects
  - Validates environment before user starts working
  - Provides clear feedback if initialization fails

## What

Implement the `jin init` command to initialize Jin in the current project directory.

### Success Criteria

- [ ] `jin init` creates `.jin/` directory in current working directory
- [ ] `.jin/context` file created with default ProjectContext (version: 1, mode: null, scope: null)
- [ ] `.jin/staging/` directory created
- [ ] `.jin/staging/index.json` created with empty entries map
- [ ] `.jin/workspace/` directory created for applied files
- [ ] `.gitignore` updated with Jin managed block (if not already present)
- [ ] Command detects if already initialized and provides friendly message
- [ ] Clear success message showing what was created
- [ ] Proper error handling for permission issues, invalid directory, etc.
- [ ] Integration with existing CLI structure and error handling

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact directory structure to create
- File formats for all configuration files
- Integration points with existing types (ProjectContext, StagingIndex)
- Testing patterns used in the project
- Error handling conventions
- Code style and naming patterns

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/src/cli/args.rs
  why: InitCommand struct definition - already defined as empty struct
  pattern: Line 203: `pub struct InitCommand;`
  critical: Command structure is already defined, just needs implementation

- file: /home/dustin/projects/jin-glm-doover/src/main.rs
  why: Entry point where Init command is dispatched
  pattern: Lines 13-16 for Init command match arm
  section: Match statement in main() - need to replace placeholder with actual call
  critical: "This is where init command handler gets called"

- file: /home/dustin/projects/jin-glm-doover/src/core/config.rs
  why: ProjectContext struct and save() method for creating .jin/context
  pattern: Lines 164-290 for ProjectContext definition and save() method
  critical: Use ProjectContext::default().save(project_dir) to create context file
  section: save() method at lines 255-270

- file: /home/dustin/projects/jin-glm-doover/src/staging/index.rs
  why: StagingIndex for creating .jin/staging/index.json
  pattern: StagingIndex::new() for empty index, save() method to persist
  critical: "Create empty staging index with StagingIndex::new()"
  section: Look for new() and save() methods

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: JinRepo for global Jin repository operations
  pattern: JinRepo::open_or_create() for ensuring global repo exists
  critical: May need to verify/initialize global Jin repo at ~/.jin/repo
  section: Lines 161-167 for open_or_create()

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: JinError types for proper error handling
  pattern: Use existing JinError variants (ConfigError, Io, Message)
  gotcha: "Don't create new error types - use existing ones"
  section: All JinError enum variants

- file: /home/dustin/projects/jin-glm-doover/src/commands/mod.rs
  why: Module for command handlers - currently empty
  pattern: Create init.rs module with execute() function
  critical: "Commands are implemented in src/commands/ directory"

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Complete specification of init command behavior
  section: Line 476-477 for "jin init" command specification
  critical: Expected behavior and user-facing output

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Architecture documentation for Jin's directory structure
  section: .jin directory structure, file formats
  critical: Understanding what each directory/file is for

# EXTERNAL - Rust Best Practices

- url: https://doc.rust-lang.org/std/fs/index.html
  why: Standard library filesystem operations
  pattern: std::fs::create_dir_all() for creating directories
  critical: Use create_dir_all() to create nested directories in one call

- url: https://docs.rs/dirs/latest/dirs/
  why: For getting home directory for global Jin repo
  pattern: dirs::home_dir() for ~/.jin location
  critical: Already in dependencies as "dirs" crate

# EXTERNAL - Git Integration Patterns

- url: https://docs.rs/git2/latest/git2/
  why: For Git operations (checking if in git repo, updating .gitignore)
  pattern: Repository::open() and Repository::discover()
  critical: Used to detect if we're in a Git repository for .gitignore update

# EXTERNAL - Serde YAML

- url: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/
  why: YAML serialization for config files (already used in project)
  pattern: serde_yaml_ng::to_string() for serialization
  critical: This is the YAML library used in the project (not serde_yaml)
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                      # Dependencies: clap, dirs, serde_yaml_ng
├── PRD.md                          # Command specification
├── src/
│   ├── main.rs                     # CLI entry point - has Init placeholder
│   ├── lib.rs                      # Library exports
│   ├── cli/
│   │   ├── args.rs                 # InitCommand already defined (line 203)
│   │   └── mod.rs                  # CLI exports
│   ├── commands/
│   │   └── mod.rs                  # Empty stub - needs init implementation
│   ├── core/
│   │   ├── mod.rs
│   │   ├── error.rs                # JinError types
│   │   ├── layer.rs                # Layer enum
│   │   └── config.rs               # ProjectContext with save() method
│   ├── staging/
│   │   ├── mod.rs
│   │   ├── index.rs                # StagingIndex with save() method
│   │   ├── entry.rs
│   │   └── router.rs
│   ├── git/
│   │   ├── mod.rs
│   │   └── repo.rs                 # JinRepo
│   └── workspace/
│       └── mod.rs
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   ├── main.rs                     # MODIFY: Update Init dispatch to call handler
│   └── commands/
│       ├── mod.rs                  # MODIFY: Export init module
│       └── init.rs                 # CREATE: Init command implementation
└── tests/
    └── commands/
        └── init_test.rs            # CREATE: Init command tests
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Use serde_yaml_ng, NOT serde_yaml
// The project uses serde_yaml_ng (serde_yaml_ng = "0.10")
// Good:
use serde_yaml_ng::to_string;
// Bad:
use serde_yaml::to_string;  // Wrong crate

// CRITICAL: ProjectContext::save() creates parent directories automatically
// Don't manually create .jin directory - save() does it
// From config.rs lines 255-270:
// pub fn save(&self, project_dir: &Path) -> Result<()> {
//     let context_path = Self::context_path(project_dir);
//     if let Some(parent) = context_path.parent() {
//         std::fs::create_dir_all(parent)?;  // <-- creates .jin/
//     }
//     ...
// }

// CRITICAL: StagingIndex::new() creates empty index
// Use StagingIndex::new() not StagingIndex::default()
// Check staging/index.rs for the exact constructor

// CRITICAL: Directory structure MUST match this exact layout:
// .jin/
// ├── context           # ProjectContext YAML file
// ├── staging/
// │   └── index.json    # StagingIndex JSON file
// └── workspace/        # Directory for applied files (empty initially)

// CRITICAL: .gitignore managed block format
// Use exact markers to prevent user edits from being overwritten
// From PRD/jinmap specification:
// # BEGIN JIN MANAGED
// .jin/
// # END JIN MANAGED
// Check if this block exists before adding

// CRITICAL: Global Jin repo location
// Default: ~/.jin/repo (from JinConfig::default())
// May need to ensure it exists: JinRepo::open_or_create(path)

// CRITICAL: Current working directory for init
// init operates on current directory (std::env::current_dir()?)
// NOT a command-line argument - InitCommand has no fields

// CRITICAL: Idempotency check
// Detect if already initialized by checking if .jin/context exists
// If exists, show friendly message: "Jin already initialized in this directory"
// Return ExitCode::SUCCESS (not an error, just informational)

// CRITICAL: Error handling patterns
// Use JinError variants for all errors
// - JinError::Io { .. } for filesystem operations
// - JinError::ConfigError { .. } for serialization issues
// - JinError::Message { .. } for general errors
// Convert std::io::Error to JinError::Io using .map_err() or ?

// CRITICAL: Command handler return type
// Commands return Result<(), JinError>
// Use ? operator for error propagation
// main() converts to ExitCode

// CRITICAL: User feedback messages
// Use eprintln!() for error messages (to stderr)
// Use println!() for success messages (to stdout)
// Be informative but concise

// GOTCHA: Git repository detection is optional
// If in a Git repo, update .gitignore
// If not in a Git repo, skip .gitignore update (not an error)
// Use git2::Repository::discover() - ignore errors

// GOTCHA: Workspace directory starts empty
// Just create .jin/workspace/ directory
// No files in it until `jin apply` is run

// GOTCHA: ProjectContext defaults
// version: 1
// mode: None (no active mode initially)
// scope: None (no active scope initially)

// PATTERN: Follow existing codebase structure for commands
// Commands go in src/commands/<command>.rs
// Each command has an execute() or run() function
// Module re-exports in src/commands/mod.rs

// PATTERN: Integration with main.rs
// Replace placeholder in match statement
// Call command handler function
// Convert Result to ExitCode

// FUTURE: Other commands depend on init structure
// - jin add uses .jin/staging/index.json
// - jin commit uses .jin/context for mode/scope
// - jin apply writes to .jin/workspace/
```

---

## Implementation Blueprint

### Data Models and Structure

This task uses existing data models from the codebase:

```rust
// From src/core/config.rs:
pub struct ProjectContext {
    pub version: u8,
    pub mode: Option<String>,
    pub scope: Option<String>,
}

// Default creates: version: 1, mode: None, scope: None
impl Default for ProjectContext { ... }

// From src/staging/index.rs:
pub struct StagingIndex {
    pub entries: HashMap<String, StagedEntry>,
}

// Creates empty index
pub fn new() -> Self { ... }
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/init.rs
  - IMPLEMENT: execute() function with InitCommand parameter
  - IMPLEMENT: Directory structure creation (.jin/, .jin/staging/, .jin/workspace/)
  - IMPLEMENT: ProjectContext creation with default values
  - IMPLEMENT: StagingIndex creation with empty entries
  - IMPLEMENT: Optional .gitignore update for Git repositories
  - IMPLEMENT: Idempotency check (detect if already initialized)
  - IMPLEMENT: User feedback messages (success, errors)
  - IMPORTS:
    * use crate::core::config::ProjectContext
    * use crate::core::error::{JinError, Result}
    * use crate::staging::index::StagingIndex
    * use std::path::PathBuf
    * use std::fs
  - NAMING: execute() function, module init
  - ERROR HANDLING: Use JinError::Io for filesystem errors
  - DEPENDENCIES: None (foundation task)
  - PLACEMENT: New file src/commands/init.rs

Task 2: MODIFY src/commands/mod.rs
  - ADD: pub mod init;
  - ADD: pub use init::execute;
  - PRESERVE: Existing module structure and comments
  - FINAL FILE:
    // Command implementations
    pub mod init;

    pub use init::execute;
  - PLACEMENT: src/commands/mod.rs
  - DEPENDENCIES: Task 1 (init.rs must exist)

Task 3: MODIFY src/main.rs
  - UPDATE: Init command match arm to call execute()
  - UPDATE: Import execute function from commands module
  - UPDATE: Convert Result to ExitCode properly
  - PATTERN:
    Commands::Init(_) => {
        match commands::init::execute() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        }
    }
  - PRESERVE: All other command match arms
  - PLACEMENT: src/main.rs, Init command match arm (around line 13-16)
  - DEPENDENCIES: Task 2 (execute must be exported)

Task 4: CREATE tests/commands/init_test.rs
  - CREATE: tests directory and init_test.rs file
  - IMPLEMENT: test_init_creates_directory_structure()
  - IMPLEMENT: test_init_creates_context_file()
  - IMPLEMENT: test_init_creates_staging_index()
  - IMPLEMENT: test_init_is_idempotent()
  - IMPLEMENT: test_init_updates_gitignore()
  - USE: tempfile crate for temporary test directories
  - VERIFY: All files/directories created correctly
  - PLACEMENT: tests/commands/init_test.rs (create tests/commands/ first)
  - DEPENDENCIES: Task 1
```

### Implementation Patterns & Key Details

```rust
// ===== PATTERN 1: Init Command Handler Structure =====

/// Execute the init command.
///
/// Initializes Jin in the current working directory by creating:
/// - .jin/context with default ProjectContext
/// - .jin/staging/index.json with empty StagingIndex
/// - .jin/workspace/ directory for applied files
/// - .gitignore entry (if in a Git repository)
///
/// # Errors
///
/// Returns `JinError::Io` if directory creation fails.
/// Returns `JinError::ConfigError` if file serialization fails.
pub fn execute(_cmd: &InitCommand) -> Result<()> {
    // Get current directory
    let project_dir = std::env::current_dir()
        .map_err(|e| JinError::Io {
            message: format!("Failed to get current directory: {}", e),
        })?;

    // Check if already initialized
    let context_path = ProjectContext::context_path(&project_dir);
    if context_path.exists() {
        println!("Jin already initialized in this directory");
        return Ok(());
    }

    // Create .jin/context with default values
    println!("Creating Jin configuration...");
    let context = ProjectContext::default();
    context.save(&project_dir)?;
    println!("  Created .jin/context");

    // Create .jin/staging/index.json
    println!("Creating staging area...");
    let staging_dir = project_dir.join(".jin/staging");
    fs::create_dir_all(&staging_dir)
        .map_err(|e| JinError::Io {
            message: format!("Failed to create staging directory: {}", e),
        })?;
    let staging_index = StagingIndex::new();
    staging_index.save(&project_dir)?;
    println!("  Created .jin/staging/index.json");

    // Create .jin/workspace/ directory
    println!("Creating workspace directory...");
    let workspace_dir = project_dir.join(".jin/workspace");
    fs::create_dir_all(&workspace_dir)
        .map_err(|e| JinError::Io {
            message: format!("Failed to create workspace directory: {}", e),
        })?;
    println!("  Created .jin/workspace/");

    // Update .gitignore if in a Git repository
    update_gitignore(&project_dir)?;

    println!();
    println!("Jin initialized successfully!");
    println!("Run 'jin help' to see available commands");

    Ok(())
}

// ===== PATTERN 2: Gitignore Update (Optional) =====

/// Updates .gitignore with Jin managed block if in a Git repository.
///
/// Only updates if the managed block doesn't already exist.
/// Silently skips if not in a Git repository.
fn update_gitignore(project_dir: &Path) -> Result<()> {
    // Try to discover Git repository
    let _repo = match git2::Repository::discover(project_dir) {
        Ok(r) => r,
        Err(_) => {
            // Not in a Git repository, skip .gitignore update
            return Ok(());
        }
    };

    let gitignore_path = project_dir.join(".gitignore");
    let existing_content = fs::read_to_string(&gitignore_path).unwrap_or_default();

    // Check if Jin managed block already exists
    if existing_content.contains("# BEGIN JIN MANAGED") {
        return Ok(());
    }

    // Append Jin managed block
    let jin_block = "\n# BEGIN JIN MANAGED\n.jin/\n# END JIN MANAGED\n";
    let mut content = existing_content;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(jin_block);

    fs::write(&gitignore_path, content)
        .map_err(|e| JinError::Io {
            message: format!("Failed to update .gitignore: {}", e),
        })?;

    println!("  Updated .gitignore");

    Ok(())
}

// ===== PATTERN 3: Idempotency Check =====

/// Check if Jin is already initialized in the current directory.
fn is_initialized(project_dir: &Path) -> bool {
    ProjectContext::context_path(project_dir).exists()
}

// Use before creating any files:
if is_initialized(&project_dir) {
    println!("Jin already initialized in this directory");
    println!("Run 'jin status' to see current state");
    return Ok(());
}

// ===== PATTERN 4: Error Handling =====

// Convert std::io::Error to JinError::Io
fs::create_dir_all(&dir_path)
    .map_err(|e| JinError::Io {
        message: format!("Failed to create directory {}: {}", dir_dir.display(), e),
    })?;

// Convert serialization errors to JinError::ConfigError
context.save(&project_dir)
    .map_err(|e| JinError::ConfigError {
        message: format!("Failed to save context: {}", e),
    })?;

// ===== PATTERN 5: Main.rs Integration =====

// In src/main.rs:
use jin_glm::cli::{Cli, Commands, InitCommand};
use jin_glm::commands::init;

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => {
            match cli.command {
                // Init command
                Commands::Init(cmd) => {
                    match init::execute(&cmd) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                // ... other commands
                _ => ExitCode::SUCCESS,
            }
        }
        Err(e) => {
            eprint!("{e}");
            if matches!(e.kind(), clap::error::ErrorKind::DisplayHelp) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

// ===== GOTCHA: StagingIndex save() method =====

// Check the actual StagingIndex implementation for save() signature
// It likely takes the project directory path
// Good:
let staging_index = StagingIndex::new();
staging_index.save(&project_dir)?;

// Bad:
staging_index.save()?;  // May need path parameter

// ===== GOTCHA: Directory Creation Order =====

// Don't need to create .jin/ separately - save() does it
// But DO need to create .jin/staging/ before saving index
// And DO need to create .jin/workspace/ (no files in it)

// Order:
// 1. Check if already initialized
// 2. Create context (also creates .jin/ via save())
// 3. Create staging directory
// 4. Save staging index
// 5. Create workspace directory
// 6. Update .gitignore
// 7. Print success message

// ===== GOTCHA: Global Jin Repo =====

// The global Jin repo (~/.jin/repo) is created on-demand
// It will be created when first needed by other commands
// For init, we don't need to explicitly create it
// It will be created by JinRepo::open_or_create() when needed

// ===== PATTERN: User Feedback =====

// Be informative about what's happening
println!("Initializing Jin in this directory...");
println!();
println!("Creating Jin configuration...");
println!("  Created .jin/context");
println!("Creating staging area...");
println!("  Created .jin/staging/index.json");
println!("Creating workspace directory...");
println!("  Created .jin/workspace/");
println!("  Updated .gitignore");
println!();
println!("Jin initialized successfully!");
println!("Run 'jin help' to see available commands");

// For idempotent case:
println!("Jin already initialized in this directory");
println!("Run 'jin status' to see current state");

// ===== PATTERN: Testing with tempfile =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_init_creates_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Change to temp directory
        std::env::set_current_dir(project_dir).unwrap();

        // Run init
        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify directories
        assert!(project_dir.join(".jin").exists());
        assert!(project_dir.join(".jin/staging").exists());
        assert!(project_dir.join(".jin/workspace").exists());
    }

    #[test]
    fn test_init_creates_context_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        std::env::set_current_dir(project_dir).unwrap();

        let cmd = InitCommand;
        execute(&cmd).unwrap();

        // Verify context file exists and has correct content
        let context_path = project_dir.join(".jin/context");
        assert!(context_path.exists());

        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.version, 1);
        assert!(context.mode.is_none());
        assert!(context.scope.is_none());
    }

    #[test]
    fn test_init_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        std::env::set_current_dir(project_dir).unwrap();

        let cmd = InitCommand;

        // First init should succeed
        execute(&cmd).unwrap();

        // Second init should also succeed (not error)
        execute(&cmd).unwrap();

        // Context should still be valid
        let context = ProjectContext::load(project_dir).unwrap();
        assert_eq!(context.version, 1);
    }
}
```

### Integration Points

```yaml
PROJECT_CONTEXT:
  - use: src/core/config.rs
  - type: ProjectContext
  - method: save(project_dir) to create .jin/context
  - integration: Use ProjectContext::default() for initial values
  - pattern:
    let context = ProjectContext::default();
    context.save(&project_dir)?;

STAGING_INDEX:
  - use: src/staging/index.rs
  - type: StagingIndex
  - method: new() for empty index, save() to persist
  - integration: Create empty staging index for file staging
  - pattern:
    let index = StagingIndex::new();
    index.save(&project_dir)?;

GIT_OPERATIONS:
  - use: git2 crate (already in dependencies)
  - method: Repository::discover() to detect Git repository
  - integration: Optional .gitignore update for Git repos
  - pattern:
    match git2::Repository::discover(project_dir) {
        Ok(_) => update_gitignore(project_dir)?,
        Err(_) => {}, // Not in Git repo, skip
    }

ERROR_HANDLING:
  - use: src/core/error.rs
  - type: JinError variants
  - integration: Convert std::io::Error to JinError::Io
  - pattern:
    fs::create_dir_all(path)
        .map_err(|e| JinError::Io { message: format!("...: {}", e) })?;

MAIN_ENTRY:
  - modify: src/main.rs
  - integration: Call execute() from Init command match arm
  - pattern:
    Commands::Init(cmd) => match init::execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating init.rs - verify compilation
cargo check --package jin

# Expected: Zero errors. If errors exist:
# - Check imports are correct
# - Verify function signatures match
# - Ensure all types are in scope

# Format check
cargo fmt --check
# Auto-format if needed
cargo fmt

# Expected: Zero formatting issues
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test init command implementation
cargo test --package jin init:: --verbose

# Run specific tests
cargo test --package jin test_init_creates_directory_structure -- --exact
cargo test --package jin test_init_creates_context_file -- --exact
cargo test --package jin test_init_is_idempotent -- --exact
cargo test --package jin test_init_updates_gitignore -- --exact

# Expected: All tests pass. Verify:
# - Directory structure created correctly
# - Context file has correct default values
# - Running init twice is safe (idempotent)
# - .gitignore updated if in Git repo

# Test coverage
cargo test --package jin --lib commands::init
# Expected: All init tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual init command in a temporary directory
cd /tmp
mkdir test_jin_init
cd test_jin_init

# Run init
cargo run -- init

# Expected output:
# Initializing Jin in this directory...
#
# Creating Jin configuration...
#   Created .jin/context
# Creating staging area...
#   Created .jin/staging/index.json
# Creating workspace directory...
#   Created .jin/workspace/
#   Updated .gitignore
#
# Jin initialized successfully!
# Run 'jin help' to see available commands

# Verify directory structure
ls -la .jin/
# Expected: context, staging/, workspace/

cat .jin/context
# Expected:
# version: 1
# mode: ~
# scope: ~

# (or similar YAML format)

cat .jin/staging/index.json
# Expected: {"entries": {}} (or similar JSON)

# Test idempotency - run again
cargo run -- init
# Expected output:
# Jin already initialized in this directory
# Run 'jin status' to see current state

# Clean up
cd /
rm -rf /tmp/test_jin_init

# Test in a Git repository
cd /tmp
mkdir test_jin_git
cd test_jin_git
git init
cargo run -- init

# Verify .gitignore was updated
cat .gitignore | grep -A 2 "JIN MANAGED"
# Expected:
# # BEGIN JIN MANAGED
# .jin/
# # END JIN MANAGED

# Clean up
cd /
rm -rf /tmp/test_jin_git
```

### Level 4: Domain-Specific Validation

```bash
# Verify ProjectContext format matches specification
cat .jin/context | head -1
# Expected: version: 1

# Verify StagingIndex is empty JSON
cat .jin/staging/index.json
# Expected: {"entries": {}} or {"entries": []}

# Verify workspace directory is empty
ls -la .jin/workspace/
# Expected: No files (may have . and .. entries only)

# Test that other commands recognize initialized state
cargo run -- status
# Expected: Status command works (may show empty state)

# Verify no errors in integration test suite
cargo test --test integration_test
# Expected: All integration tests pass

# Expected: All domain-specific validations pass
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo check --package jin` passes with zero errors
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] All tests pass: `cargo test --package jin init::`
- [ ] Init command creates correct directory structure
- [ ] Init command is idempotent (safe to run multiple times)
- [ ] Error messages are clear and helpful

### Feature Validation

- [ ] `.jin/context` created with version: 1, mode: null, scope: null
- [ ] `.jin/staging/index.json` created with empty entries
- [ ] `.jin/workspace/` directory created
- [ ] `.gitignore` updated when in Git repository
- [ ] Friendly message when already initialized
- [ ] Clear success message showing what was created
- [ ] Proper error handling for permission issues

### Code Quality Validation

- [ ] Follows existing codebase patterns
- [ ] File placement matches desired tree structure
- [ ] Uses existing JinError types (no new error types)
- [ ] Integration with main.rs is clean
- [ ] Tests use tempfile for isolation

### Documentation & Deployment

- [ ] Function has doc comment explaining behavior
- [ ] Error messages are user-friendly
- [ ] Success messages are informative
- [ ] Code is self-documenting with clear variable names

---

## Anti-Patterns to Avoid

- ❌ Don't use serde_yaml - use serde_yaml_ng
- ❌ Don't manually create .jin/ - ProjectContext::save() does it
- ❌ Don't error on already initialized - be friendly/idempotent
- ❌ Don't require Git repository - .gitignore update is optional
- ❌ Don't hardcode paths - use PathBuf and project_dir
- ❌ Don't create new error types - use existing JinError variants
- ❌ Don't skip user feedback - show what's being created
- ❌ Don't forget to create workspace directory - even though it's empty
- ❌ Don't assume StagingIndex::save() signature - check the actual method
- ❌ Don't use synchronous operations in async context (not applicable here)

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 10/10 - High confidence in one-pass implementation success
