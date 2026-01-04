name: "P3.M2.T2: Add Config Command"
description: "Implement a 'jin config' command to view and set Jin configuration options"

---

## Goal

**Feature Goal**: Implement a `jin config` command that provides a user-friendly interface to view and set Jin configuration values, including displaying the current JIN_DIR location.

**Deliverable**: A new `jin config` command with subcommands for getting, setting, and listing configuration values.

**Success Definition**:
- Users can view all configuration values with `jin config list`
- Users can get specific values with `jin config get <key>`
- Users can set values with `jin config set <key> <value>`
- Users can see current JIN_DIR location (with guidance on setting it)
- Command follows existing Jin command patterns and conventions
- Includes comprehensive tests

## User Persona

**Target User**: Jin users who need to configure Jin settings without manually editing configuration files.

**Use Case**: A developer wants to:
- View their current Jin configuration
- Set remote URL for syncing
- Check their JIN_DIR location
- Configure user information

**User Journey**:
1. User runs `jin config list` to see all current configuration
2. User runs `jin config get jin-dir` to see JIN_DIR location
3. User runs `jin config set remote.url https://github.com/user/jin-config` to set remote
4. User runs `jin config get remote.url` to verify the change

**Pain Points Addressed**:
- No need to manually edit `~/.jin/config.toml`
- Clear visibility into current configuration state
- Guidance on setting JIN_DIR (which is not in config file)

## Why

- **Better Developer Experience**: Provides a CLI interface for configuration instead of manual file editing
- **Documentation Companion**: Complements the JIN_DIR documentation in README (P3.M2.T1) with programmatic access
- **Consistency**: Matches patterns from other Jin commands like `jin mode` and `jin scope`
- **Safety**: Validates configuration values before writing

## What

Implement a `jin config` command with the following user-visible behavior:

### Command Structure

```bash
# List all configuration values
jin config list

# Get a specific configuration value
jin config get <key>

# Set a configuration value
jin config set <key> <value>

# Show JIN_DIR location
jin config get jin-dir
```

### Configuration Keys Supported

| Key | Type | Description |
|-----|------|-------------|
| `jin-dir` | special | Display JIN_DIR environment variable location |
| `remote.url` | string | Remote repository URL |
| `remote.fetch-on-init` | boolean | Whether to fetch on initialization |
| `user.name` | string | User's name |
| `user.email` | string | User's email |

### Special JIN_DIR Handling

`jin config get jin-dir` displays:
- Current JIN_DIR value if set
- Default location (`~/.jin`) if not set
- Guidance on how to set JIN_DIR persistently

### Success Criteria

- [ ] `jin config list` shows all global config values
- [ ] `jin config get <key>` returns specific value or error for unknown key
- [ ] `jin config set <key> <value>` updates and saves config
- [ ] `jin config get jin-dir` shows current JIN_DIR with usage guidance
- [ ] Invalid keys produce helpful error messages
- [ ] All tests pass (unit + integration)

## All Needed Context

### Context Completeness Check

_If someone knew nothing about this codebase, would they have everything needed to implement this successfully?_

**Yes** - This PRP provides:
- Exact file locations and patterns to follow
- Specific code examples from similar commands
- Configuration system details
- Test patterns used in this codebase
- Known gotchas and constraints

### Documentation & References

```yaml
# MUST READ - Include these in your context window

- url: https://doc.rust-lang.org/cargo/reference/config.html
  why: Reference for config command UX patterns (get/set/list)
  critical: Cargo's config command is the standard pattern for Rust CLIs

- url: https://docs.rs/clap/latest/clap/
  why: Clap v4 derive API documentation for subcommand patterns
  critical: Must use Subcommand derive for nested commands

- url: https://rust-cli-recommendations.sunshowers.io/hierarchical-config.html
  why: Best practices for config precedence (env vars override config files)
  critical: JIN_DIR is an env var, not a config file setting

- file: src/cli/mod.rs
  why: CLI structure - where to add Config command enum and ConfigAction subcommand
  pattern: Add to Commands enum after existing commands, follow ModeAction/ScopeAction pattern
  gotcha: Must also add to commands/mod.rs dispatcher

- file: src/commands/mode.rs
  why: Perfect template for config subcommand structure (create/use/list/delete/show/unset)
  pattern: Subcommand enum with execute() function that dispatches to helper functions
  gotcha: Each action is a separate function (create(), use_mode(), list(), etc.)

- file: src/commands/context.rs
  why: Template for simple display command that shows configuration state
  pattern: Load context, handle NotInitialized gracefully, display formatted output
  gotcha: Uses match for graceful error handling

- file: src/core/config.rs
  why: Contains JinConfig struct with load()/save() methods
  pattern: JinConfig::load() returns Result<JinConfig>, save() writes to default_path()
  critical: JinConfig has version, remote (Option<RemoteConfig>), user (Option<UserConfig>)
  gotcha: default_path() respects JIN_DIR environment variable

- file: src/core/error.rs
  why: Error types to use for consistent error handling
  pattern: JinError::Config(String), JinError::NotInitialized, JinError::NotFound(String)

- file: src/commands/mod.rs
  why: Command dispatcher - must add new command here
  pattern: Commands::Config(args) => config::execute(args),
  gotcha: Must also add pub mod config; at the top

- docfile: plan/P3M2T2/research/01_command_structure.md
  why: Detailed analysis of CLI command structure
  section: Complete

- docfile: plan/P3M2T2/research/02_configuration_system.md
  why: Configuration system details including JIN_DIR handling
  section: Complete

- docfile: plan/P3M2T2/research/03_command_examples.md
  why: Specific implementation examples from mode, context, and reset commands
  section: Complete
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── mod.rs           # CLI definition - add Config command here
│   └── args.rs          # Argument structs (not needed for ConfigAction pattern)
├── commands/
│   ├── mod.rs           # Command dispatcher - add config::execute here
│   ├── mode.rs          # Template for subcommand pattern
│   ├── context.rs       # Template for simple display
│   └── [other commands] # Various command implementations
├── core/
│   ├── config.rs        # JinConfig struct with load()/save()
│   ├── error.rs         # JinError types
│   └── mod.rs           # Core exports
├── lib.rs               # Command router
└── main.rs              # Entry point
```

### Desired Codebase Tree (New Files)

```bash
src/
├── cli/
│   └── mod.rs           # ADD: Commands::Config(ConfigAction)
├── commands/
│   ├── mod.rs           # MODIFY: Add pub mod config; and Config dispatcher
│   └── config.rs        # NEW: Config command implementation
└── commands/tests/
    └── test_config.rs   # NEW: Config command tests
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: JIN_DIR is an environment variable, NOT a config file setting
// It must be set BEFORE any Jin commands run
// The config command can only DISPLAY it, not SET it

// PATTERN: Subcommand enum for nested commands (like jin mode create)
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    List,
    Get { key: String },
    Set { key: String, value: String },
}

// GOTCHA: Nested config keys use dot notation (remote.url)
// Need to parse "remote.url" and traverse JinConfig struct

// GOTCHA: JinConfig fields use Option<T> for nullable values
// When setting, need to handle nested Option types

// PATTERN: Error handling uses crate::core::Result (alias for Result<T, JinError>)
// Use JinError::Config(String) for config-related errors

// PATTERN: All commands return Result<()> from execute()
// Use println! for user output, never return data

// CRITICAL: Must add command in THREE places:
// 1. src/cli/mod.rs - Commands enum
// 2. src/commands/mod.rs - dispatcher match arm
// 3. src/commands/mod.rs - pub mod config;

// PATTERN: Test files use #[serial] attribute due to Git lock contention
// All tests modifying Jin state must be serial
```

## Implementation Blueprint

### Data Models

No new data models needed - using existing `JinConfig` from `src/core/config.rs`:

```rust
// Existing structure (from src/core/config.rs:13-24)
pub struct JinConfig {
    pub version: u32,
    pub remote: Option<RemoteConfig>,
    pub user: Option<UserConfig>,
}

pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}

pub struct UserConfig {
    pub name: Option<String>,
    pub email: Option<String>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/cli/mod.rs
  - ADD: ConfigAction enum (follow ModeAction pattern at line 128-151)
  - ADD: Commands::Config(ConfigAction) variant to Commands enum (after line 108)
  - NAMING: ConfigAction with List, Get { key: String }, Set { key: String, value: String }
  - PATTERN: Copy ModeAction structure exactly, just rename and change variants
  - LOCATION: Add ConfigAction enum after ScopeAction enum (after line 180)

Task 2: CREATE src/commands/config.rs
  - IMPLEMENT: ConfigAction enum, execute() function, list(), get(), set() helper functions
  - FOLLOW pattern: src/commands/mode.rs (subcommand dispatch pattern)
  - NAMING: pub fn execute(action: ConfigAction) -> Result<()>
  - PLACEMENT: New file in src/commands/

Task 3: MODIFY src/commands/mod.rs
  - ADD: pub mod config; at top of file (after line 29)
  - ADD: Commands::Config(action) => config::execute(action), to dispatcher match (after line 58)
  - PRESERVE: All existing command registrations

Task 4: IMPLEMENT config::list() function in src/commands/config.rs
  - IMPLEMENT: fn list() -> Result<()>
  - LOAD: JinConfig::load() to get current config
  - DISPLAY: All config values with formatted output (follow context.rs pattern)
  - HANDLE: Missing config file gracefully (use default if not exists)
  - OUTPUT: Format similar to "jin context" command

Task 5: IMPLEMENT config::get() function in src/commands/config.rs
  - IMPLEMENT: fn get(key: &str) -> Result<()>
  - PARSE: Dot-notation keys (remote.url, user.name, etc.)
  - SPECIAL CASE: "jin-dir" key shows JIN_DIR environment variable
  - ERROR: JinError::NotFound for unknown keys
  - DISPLAY: Single value or "(not set)" for None values

Task 6: IMPLEMENT config::set() function in src/commands/config.rs
  - IMPLEMENT: fn set(key: &str, value: &str) -> Result<()>
  - LOAD: JinConfig::load() to get current config
  - PARSE: Key and value, handle nested structs (remote.*, user.*)
  - VALIDATE: Input values (e.g., fetch-on-init must be boolean)
  - SAVE: config.save() to persist changes
  - ERROR: Clear error message for invalid keys or values

Task 7: CREATE src/commands/tests/test_config.rs
  - IMPLEMENT: Unit tests for list, get, set operations
  - FOLLOW pattern: Existing test files with #[serial] attribute
  - SETUP: Use test_utils::setup_test_jin() for test isolation
  - COVERAGE: Happy path, error cases, edge cases
  - CLEANUP: Test cleanup to remove test artifacts
```

### Implementation Patterns & Key Details

```rust
// ===== File: src/commands/config.rs =====

use crate::core::{config::JinConfig, error::{JinError, Result}};

/// Config subcommands
#[derive(clap::Subcommand, Debug)]
pub enum ConfigAction {
    /// List all configuration values
    List,
    /// Get a specific configuration value
    Get {
        /// Configuration key (e.g., remote.url, user.name, jin-dir)
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., remote.url, user.name)
        key: String,
        /// Configuration value
        value: String,
    },
}

/// Main execute function - REQUIRED PATTERN
pub fn execute(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::List => list(),
        ConfigAction::Get { key } => get(&key),
        ConfigAction::Set { key, value } => set(&key, &value),
    }
}

/// List all configuration values - PATTERN: Simple display command
fn list() -> Result<()> {
    let config = JinConfig::load()?;

    println!("Jin Configuration:");
    println!("  jin-dir: {}", get_jin_dir_display()?);

    if let Some(ref remote) = config.remote {
        println!("  remote.url: {}", remote.url);
        println!("  remote.fetch-on-init: {}", remote.fetch_on_init);
    } else {
        println!("  remote.url: (not set)");
        println!("  remote.fetch-on-init: (not set)");
    }

    if let Some(ref user) = config.user {
        println!("  user.name: {}", user.name.as_deref().unwrap_or("(not set)"));
        println!("  user.email: {}", user.email.as_deref().unwrap_or("(not set)"));
    } else {
        println!("  user.name: (not set)");
        println!("  user.email: (not set)");
    }

    Ok(())
}

/// Get a specific configuration value
fn get(key: &str) -> Result<()> {
    match key {
        "jin-dir" => {
            println!("{}", get_jin_dir_display()?);
        }
        _ => {
            let config = JinConfig::load()?;
            let value = get_config_value(&config, key)?;
            println!("{}", value);
        }
    }
    Ok(())
}

/// Set a configuration value - GOTCHA: Must handle nested structs
fn set(key: &str, value: &str) -> Result<()> {
    let mut config = JinConfig::load()?;

    match key {
        "remote.url" => {
            config.remote.get_or_insert_with(|| RemoteConfig {
                url: String::new(),
                fetch_on_init: false,
            }).url = value.to_string();
        }
        "remote.fetch-on-init" => {
            let bool_val = value.parse::<bool>()
                .map_err(|_| JinError::Config(format!("Invalid boolean value: {}", value)))?;
            config.remote.get_or_insert_with(|| RemoteConfig {
                url: String::new(),
                fetch_on_init: false,
            }).fetch_on_init = bool_val;
        }
        "user.name" => {
            config.user.get_or_insert_with(UserConfig::default).name = Some(value.to_string());
        }
        "user.email" => {
            config.user.get_or_insert_with(UserConfig::default).email = Some(value.to_string());
        }
        _ => {
            return Err(JinError::NotFound(format!("Unknown config key: {}", key)));
        }
    }

    config.save()?;
    println!("Set {} = {}", key, value);
    Ok(())
}

/// Helper: Get config value by key - PATTERN: Match on dot notation
fn get_config_value(config: &JinConfig, key: &str) -> Result<String> {
    match key {
        "remote.url" => Ok(config.remote.as_ref()
            .and_then(|r| Some(r.url.clone()))
            .unwrap_or_else(|| "(not set)".to_string())),
        "remote.fetch-on-init" => Ok(config.remote.as_ref()
            .map(|r| r.fetch_on_init.to_string())
            .unwrap_or_else(|| "(not set)".to_string())),
        "user.name" => Ok(config.user.as_ref()
            .and_then(|u| u.name.as_ref())
            .cloned()
            .unwrap_or_else(|| "(not set)".to_string())),
        "user.email" => Ok(config.user.as_ref()
            .and_then(|u| u.email.as_ref())
            .cloned()
            .unwrap_or_else(|| "(not set)".to_string())),
        _ => Err(JinError::NotFound(format!("Unknown config key: {}", key))),
    }
}

/// Helper: Get JIN_DIR display with guidance
fn get_jin_dir_display() -> Result<String> {
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        Ok(jin_dir)
    } else {
        Ok("~/.jin".to_string())
    }
}

// ===== File: src/cli/mod.rs (MODIFICATIONS) =====

// Add after ScopeAction enum (around line 180):
/// Config subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// List all configuration values
    List,
    /// Get a specific configuration value
    Get {
        /// Configuration key (e.g., remote.url, user.name)
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., remote.url, user.name)
        key: String,
        /// Configuration value
        value: String,
    },
}

// Add to Commands enum (around line 100):
/// View/edit Jin configuration
#[command(subcommand)]
Config(ConfigAction),
```

### Integration Points

```yaml
CLI_DEFINITION:
  - file: src/cli/mod.rs
  - add: ConfigAction enum (follow ModeAction pattern)
  - add: Commands::Config(ConfigAction) variant
  - pattern: "Place after Completion variant"

COMMAND_DISPATCHER:
  - file: src/commands/mod.rs
  - add: "pub mod config;" at top
  - add: "Commands::Config(action) => config::execute(action)," to match
  - pattern: "Place after Link command dispatch"

TEST_MODULE:
  - file: src/commands/tests/test_config.rs
  - pattern: Use #[serial] attribute for tests modifying state
  - setup: Use test_utils::setup_test_jin() for isolation
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating src/commands/config.rs
cargo check --bin jin 2>&1 | head -50

# Expected: No compilation errors. Fix any type mismatches or missing imports.

# Format check
cargo fmt --check

# Expected: No formatting issues. Run cargo fmt if needed.

# Clippy lints
cargo clippy --bin jin 2>&1 | head -50

# Expected: No warnings. Fix any clippy suggestions.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run config command tests
cargo test --package jin --test '*' config -- --nocapture

# Expected: All config tests pass. Check test output carefully.

# Run all command tests to ensure no regressions
cargo test --package jin --lib commands:: 2>&1 | tail -20

# Expected: All existing tests still pass.

# Full test suite
cargo test --package jin 2>&1 | tail -30

# Expected: All tests pass.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the binary
cargo build --release

# Test: List all config
./target/release/jin config list

# Expected: Shows all config values, or "(not set)" for unset values

# Test: Get JIN_DIR
./target/release/jin config get jin-dir

# Expected: Shows current JIN_DIR or default "~/.jin"

# Test: Set a value
./target/release/jin config set remote.url "https://github.com/test/jin-config"

# Expected: Prints "Set remote.url = https://github.com/test/jin-config"

# Test: Verify the set value
./target/release/jin config get remote.url

# Expected: Shows the URL that was just set

# Test: Invalid key
./target/release/jin config get invalid.key

# Expected: Error message "Unknown config key: invalid.key"

# Test: Invalid boolean value
./target/release/jin config set remote.fetch-on-init "not-a-boolean"

# Expected: Error message "Invalid boolean value: not-a-boolean"
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test: Help output
./target/release/jin config --help

# Expected: Shows List, Get, Set subcommands with descriptions

# Test: Subcommand help
./target/release/jin config set --help

# Expected: Shows key and value arguments with descriptions

# Test: Edge case - empty value
./target/release/jin config set user.name ""

# Expected: Sets empty string (valid behavior)

# Test: Edge case - special characters in URL
./target/release/jin config set remote.url "https://user:pass@example.com/path"

# Expected: Handles special characters correctly

# Test: Verify config file persistence
cat ~/.jin/config.toml

# Expected: Shows the values set via jin config set

# Test: Config file survives process restart
./target/release/jin config get remote.url

# Expected: Shows the value from previous session
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo test --package jin` passes with no failures
- [ ] `cargo clippy --bin jin` produces no warnings
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] `cargo check --bin jin` completes with no errors

### Feature Validation

- [ ] `jin config list` displays all config values correctly
- [ ] `jin config get <key>` returns correct value or error
- [ ] `jin config set <key> <value>` updates and persists config
- [ ] `jin config get jin-dir` shows current JIN_DIR with guidance
- [ ] Invalid keys produce helpful error messages
- [ ] Invalid values produce helpful error messages
- [ ] Config file (~/.jin/config.toml) is correctly updated

### Code Quality Validation

- [ ] Follows ModeAction/ScopeAction pattern for subcommands
- [ ] Uses crate::core::Result and JinError types consistently
- [ ] File placement matches desired codebase tree
- [ ] Command added to all three required locations (cli/mod.rs, commands/mod.rs dispatcher, commands/mod.rs module)
- [ ] Uses existing JinConfig::load()/save() methods
- [ ] Error handling matches existing patterns

### Documentation & Deployment

- [ ] Doc comments on ConfigAction enum variants
- [ ] Help text is clear and descriptive
- [ ] JIN_DIR guidance is helpful
- [ ] Error messages are user-friendly

---

## Anti-Patterns to Avoid

- ❌ Don't create new config structs - use existing `JinConfig` from `src/core/config.rs`
- ❌ Don't try to SET JIN_DIR via config command - it's an environment variable
- ❌ Don't skip adding command to commands/mod.rs dispatcher - common oversight
- ❌ Don't use string matching for keys without proper error handling
- ❌ Don't forget to handle Option<T> for nullable config values
- ❌ Don't skip tests - use #[serial] attribute for state-modifying tests
- ❌ Don't ignore clippy warnings - fix them before committing
- ❌ Don't hardcode paths - use `JinConfig::default_path()` pattern
- ❌ Don't use complex config libraries - keep it simple with toml crate
- ❌ Don't modify shell profiles - just show guidance for JIN_DIR

## Confidence Score

**8/10** for one-pass implementation success

**Rationale**:
- Clear patterns to follow (mode command is nearly identical structure)
- Existing config infrastructure (JinConfig) is well-designed
- Comprehensive research and context provided
- Known gotchas documented (JIN_DIR is special, Option handling)
- Test patterns well-established in codebase

**Risk Factors**:
- Nested config value parsing (remote.url) requires careful Option handling
- JIN_DIR special case needs clear user messaging
- Boolean value parsing needs good error messages
