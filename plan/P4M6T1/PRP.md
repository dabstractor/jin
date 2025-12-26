# Product Requirement Prompt (PRP): Link Command

**Work Item**: P4.M6.T1 - Link Command
**Status**: Ready for Implementation
**Confidence Score**: 9/10

---

## Goal

**Feature Goal**: Enable users to link their local Jin workspace to a shared remote repository, establishing the foundation for team collaboration and synchronization.

**Deliverable**: A fully functional `jin link <url>` command that validates and stores the remote repository URL in the Jin configuration.

**Success Definition**:
- User can execute `jin link <url>` to configure a remote Jin repository
- URL is validated for supported Git URL formats (SSH, HTTPS, git://, file://)
- Remote URL is persisted to `~/.jin/config.yaml`
- Command is idempotent (can be run again to update the remote URL)
- Clear error messages for invalid URLs or configuration issues

## User Persona

**Target User**: Developer setting up Jin for team collaboration

**Use Case**: A developer wants to connect their local Jin workspace to a shared repository containing modes, scopes, and project configurations used by their team.

**User Journey**:
1. Developer initializes Jin in their project: `jin init`
2. Developer links to team's shared Jin repository: `jin link git@github.com:myorg/jin-config`
3. Developer can now fetch/pull/push/sync shared configurations
4. Team members can collaborate on shared layers

**Pain Points Addressed**:
- No way to share Jin configurations across team members
- Each developer must manually configure modes, scopes, and projects
- No centralized source of truth for team configuration standards

## Why

- **Team Collaboration**: Enables sharing of modes, scopes, and project configurations across development teams
- **Consistency**: Ensures all team members use the same configuration standards
- **Onboarding**: New team members can quickly get the correct Jin configuration by linking to the shared repository
- **Foundation for Sync**: This command is required before `jin fetch`, `jin pull`, `jin push`, and `jin sync` can function (P5 module)
- **Git-like UX**: Follows familiar patterns from `git remote add` that developers already know

## What

The `jin link` command configures a remote repository URL for Jin synchronization. The URL is stored in the global Jin configuration and used by subsequent sync commands.

**Command Syntax**:
```bash
jin link <url>
```

**Supported URL Formats**:
- SSH: `git@github.com:myorg/jin-config.git`
- HTTPS: `https://github.com/myorg/jin-config.git`
- Git protocol: `git://github.com/myorg/jin-config.git`
- Local path: `/path/to/jin-repo` or `file:///path/to/jin-repo`

**Behavior**:
- Validates the URL format
- Stores the URL in `~/.jin/config.yaml` under a new `remote` field
- Creates the config file if it doesn't exist
- Updates the URL if already set (idempotent)
- Prints confirmation message with the stored URL

### Success Criteria

- [ ] Command accepts valid Git URLs (SSH, HTTPS, git://, local paths)
- [ ] URL is validated before storing (not empty, recognizable format)
- [ ] Remote URL is persisted to `~/.jin/config.yaml`
- [ ] Running command again updates the existing URL
- [ ] Clear error messages for invalid URLs
- [ ] JinError variants properly map to exit codes
- [ ] User-friendly output confirms the link was successful

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: A developer unfamiliar with this codebase would have everything needed to implement this feature successfully using this PRP.

- **File structure**: Exact locations of all relevant files provided
- **Patterns**: Specific patterns to follow from existing commands
- **Error handling**: Complete error type hierarchy with exit code mapping
- **Configuration**: How config is read/written with YAML format
- **Validation**: URL validation patterns from Git and Cargo research
- **Testing**: Project has test framework setup

### Documentation & References

```yaml
# MUST READ - Critical implementation context
- url: https://git-scm.com/docs/git-remote
  why: Git remote add behavior and URL format patterns
  critical: SSH URLs use git@host:path format, not standard URL syntax

- url: https://docs.rs/git2/latest/git2/
  why: git2-rs API for remote operations (future-proofing)
  section: Remote struct and remote() method

- file: src/commands/init.rs
  why: Command structure pattern, config file creation, idempotency check
  pattern: execute() function returning Result<()>, context_path.exists() check
  gotcha: Always check if already initialized before proceeding

- file: src/commands/import.rs
  why: URL parsing pattern for Git remote origin detection
  pattern: detect_project_name() function showing URL parsing logic
  gotcha: SSH URLs need special parsing (rsplit on ':'), HTTPS URLs need path stripping

- file: src/core/config.rs
  why: JinConfig load/save patterns, YAML serialization
  pattern: load() with fallback, save() with parent dir creation
  gotcha: Uses serde_yaml_ng, not serde_yaml; version field required

- file: src/core/error.rs
  why: Complete error hierarchy, exit code mapping
  pattern: JinError enum variants with context fields
  gotcha: Exit codes: 1=general, 2=invalid arg, 3=not found, 4=conflict, 5=permission

- file: src/cli/args.rs
  why: LinkCommand struct already defined with url field
  pattern: #[arg(value_name = "URL")] annotation
  gotcha: Command dispatch in main.rs needs to be updated

- file: src/main.rs
  why: Command dispatch pattern, error handling
  pattern: match cli.command { Commands::Link(cmd) => ... }
  gotcha: Must call commands::link_execute(&cmd)

- file: src/commands/mod.rs
  why: Module export pattern for commands
  pattern: pub mod link; pub use link::execute as link_execute;
  gotcha: Must add module declaration

- docfile: plan/P4M1T1/PRP.md
  why: Similar command implementation (init command) for reference
  section: Implementation Tasks
  gotcha: Uses same patterns but LinkCommand modifies global config, not project context

- docfile: plan/docs/PRD.md
  why: Product requirements context for link command
  section: Lines 481, 596, 662 for link command usage
  gotcha: Part of Remote Operations section, enables P5 sync commands
```

### Current Codebase Tree

```bash
jin-glm-doover/
├── src/
│   ├── cli/
│   │   └── args.rs              # LinkCommand struct already defined
│   ├── commands/
│   │   ├── mod.rs               # Module exports (ADD link module here)
│   │   ├── init.rs              # Pattern for idempotency, config creation
│   │   └── import.rs            # URL parsing pattern (detect_project_name)
│   ├── core/
│   │   ├── config.rs            # JinConfig with load/save methods
│   │   ├── error.rs             # JinError enum with all variants
│   │   └── layer.rs             # Layer types (not needed for link)
│   └── main.rs                  # Command dispatch (ADD Link case)
├── plan/
│   └── P4M6T1/
│       └── PRP.md               # This document
└── Cargo.toml                   # Dependencies: git2, serde_yaml_ng
```

### Desired Codebase Tree with New Files

```bash
jin-glm-doover/
├── src/
│   ├── cli/
│   │   └── args.rs              # [MODIFY] No changes needed (LinkCommand exists)
│   ├── commands/
│   │   ├── mod.rs               # [MODIFY] ADD: pub mod link; pub use link::execute as link_execute;
│   │   ├── init.rs              # [REFERENCE] Pattern for command structure
│   │   ├── import.rs            # [REFERENCE] Pattern for URL parsing
│   │   └── link.rs              # [NEW FILE] Main implementation
│   ├── core/
│   │   ├── config.rs            # [MODIFY] ADD: pub remote: Option<String> field
│   │   ├── error.rs             # [REFERENCE] No new error types needed
│   │   └── layer.rs             # [NO CHANGE]
│   └── main.rs                  # [MODIFY] ADD: Commands::Link(cmd) => match commands::link_execute(&cmd)
└── Cargo.toml                   # [NO CHANGE] No new dependencies needed
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: JinConfig uses serde_yaml_ng, not serde_yaml
// The _ng variant has different API for some operations

// CRITICAL: SSH URLs don't follow standard URL format
// git@github.com:user/repo.git is NOT a valid URL per url crate
// Must parse manually: strip "git@", split on ':', strip ".git"

// CRITICAL: Config.save() creates parent directories automatically
// No need to manually create ~/.jin before calling save()

// CRITICAL: Exit codes are mapped in JinError -> i32 impl
// Invalid URLs should use exit code 2 (invalid argument)
// Config errors should use exit code 1 (general error)

// CRITICAL: The LinkCommand struct is already defined in src/cli/args.rs
// DO NOT redefine it - just use the existing definition

// CRITICAL: Always check for existing state (idempotency)
// Like init.rs: if context_path.exists() { print message; return Ok(()); }

// CRITICAL: Import command shows URL parsing at lines 59-82
// Use detect_project_name() as reference for parsing different URL formats

// CRITICAL: Global config location is ~/.jin/config.yaml
// Use JinConfig::jinx_config_path() to get the path

// CRITICAL: field-level conditionals in JinConfig
// Use #[serde(skip_serializing_if = "Option::is_none")] for Optional fields

// PATTERN: URL normalization
// Strip trailing ".git" if present for consistency
// Preserve original URL format (SSH vs HTTPS) for user display

// GOTCHA: URL validation should be lenient
// Don't try to verify connectivity (that's for `jin fetch`)
// Just validate the format looks like a Git URL
```

## Implementation Blueprint

### Data Models and Structure

The link command modifies the existing `JinConfig` struct to add a `remote` field for storing the repository URL.

**Existing JinConfig structure** (from `src/core/config.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JinConfig {
    pub version: u8,
    pub repository: PathBuf,
    pub default_mode: Option<String>,
    pub default_scope: Option<String>,
}

// ADD THIS FIELD:
pub remote: Option<String>,  // URL of remote Jin repository
```

**URL Parsing Helper** (new function in `src/commands/link.rs`):
```rust
/// Parse and validate a Git repository URL
/// Supports: SSH (git@...), HTTPS, git://, file://, and local paths
fn parse_and_validate_url(url: &str) -> Result<String>

/// Normalize URL by stripping .git suffix
fn normalize_url(url: &str) -> String
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/core/config.rs
  ADD_FIELD: pub remote: Option<String> to JinConfig struct
  FOLLOW: existing pattern with default_mode and default_scope
  ANNOTATION: Add #[serde(skip_serializing_if = "Option::is_none")]
  REASON: Remote URL is optional for local-only Jin usage
  VALIDATE: cargo check passes after change

Task 2: CREATE src/commands/link.rs
  IMPLEMENT: execute(cmd: &LinkCommand) -> Result<()> function
  FOLLOW: pattern from src/commands/init.rs (structure, idempotency)
  IMPORT: use crate::core::{config::JinConfig, error::{JinError, Result}};
  PARSE_URL: Implement parse_and_validate_url() supporting SSH, HTTPS, git://, file://
  NORMALIZE: Implement normalize_url() to strip .git suffix
  LOAD_CONFIG: let config = JinConfig::load()?;
  VALIDATE_URL: parse_and_validate_url(&cmd.url)?
  UPDATE_REMOTE: config.remote = Some(normalized_url);
  SAVE: config.save()?;
  OUTPUT: println!("Linked to remote repository: {}", normalized_url);
  PLACEMENT: src/commands/link.rs

Task 3: MODIFY src/commands/mod.rs
  ADD: pub mod link;
  ADD: pub use link::execute as link_execute;
  FOLLOW: existing pattern for other command modules
  PLACEMENT: after the last module import, before tests

Task 4: MODIFY src/main.rs
  ADD_CASE: Commands::Link(cmd) => match commands::link_execute(&cmd)
  FOLLOW: existing command dispatch pattern
  ERROR_HANDLING: Ok(()) => ExitCode::SUCCESS, Err(e) => { eprintln!("Error: {}", e); ExitCode::from(e) }
  PRESERVE: all existing command cases
  PLACEMENT: in the match cli.command block

Task 5: IMPLEMENT URL Validation in link.rs
  FUNCTION: parse_and_validate_url(url: &str) -> Result<String>
  CHECK_EMPTY: if url.is_empty() => Err(JinError::InvalidConfig { message: "URL cannot be empty".into() })
  CHECK_SSH: if url.starts_with("git@") => validate_ssh_url(url)
  CHECK_HTTP: if url.starts_with("http") => validate_http_url(url) using url crate or manual parse
  CHECK_GIT_PROTOCOL: if url.starts_with("git://") => validate_git_url(url)
  CHECK_LOCAL: if url.starts_with("/") || url.starts_with("file://") => validate_local_url(url)
  FALLBACK: If no pattern matches, try as-is (user might have custom format)
  ERROR: JinError::InvalidConfig with descriptive message
  GOTCHA: SSH URLs need special parsing (not valid per URL crate)

Task 6: IMPLEMENT URL Normalization in link.rs
  FUNCTION: normalize_url(url: &str) -> String
  STRIP_GIT: url.trim_end_matches(".git").to_string()
  PRESERVE_SCHEME: Don't convert SSH to HTTPS or vice versa
  RETURN: Normalized URL string
  REASON: Consistency when storing URLs

Task 7: IMPLEMENT Idempotency Check
  AFTER_LOAD: Check if config.remote == Some(normalized_url)
  IF_MATCH: println!("Already linked to: {}", url); return Ok(());
  IF_DIFFERENT: println!("Updating remote from {} to {}", old_url, url);
  REASON: Allow re-running command to update URL
  FOLLOW: pattern from init.rs lines 47-53

Task 8: ADD Error Variants if needed
  REVIEW: existing JinError variants in src/core/error.rs
  CONSIDER: InvalidConfig variant for URL validation errors
  EXIT_CODE: 2 for invalid URL (invalid argument)
  NO_NEW_VARIANTS: Existing variants should suffice

Task 9: BUILD and TEST
  RUN: cargo build --release
  CHECK: No compilation errors
  TEST: jin link git@github.com:myorg/jin-config
  VERIFY: ~/.jin/config.yaml contains remote field
  TEST_IDEMPOTENT: Run command again, should update without error
  TEST_INVALID: jin link "not-a-url" should fail with clear error
  TEST_FORMATS: SSH, HTTPS, git://, local paths all work
```

### Implementation Patterns & Key Details

```rust
// File: src/commands/link.rs
use crate::core::{config::JinConfig, error::{JinError, Result}};

/// Execute the link command
pub fn execute(cmd: &LinkCommand) -> Result<()> {
    let url = cmd.url.trim();

    // PATTERN: Validate URL first (fail fast)
    let validated = parse_and_validate_url(url)?;

    // PATTERN: Normalize URL for consistency
    let normalized = normalize_url(&validated);

    // PATTERN: Load existing config (from init.rs pattern)
    let mut config = JinConfig::load()?;

    // PATTERN: Idempotency check (from init.rs:47-53)
    if let Some(current) = &config.remote {
        if current == &normalized {
            println!("Already linked to remote: {}", normalized);
            return Ok(());
        }
        println!("Updating remote from {} to {}", current, normalized);
    } else {
        println!("Linking to remote: {}", normalized);
    }

    // GOTCHA: Use Option::set() or direct assignment
    config.remote = Some(normalized);

    // PATTERN: Save with automatic dir creation (config.rs:154-164)
    config.save()?;

    Ok(())
}

/// Parse and validate Git repository URL
/// Supports: SSH (git@host:path), HTTPS, git://, file://, local paths
fn parse_and_validate_url(url: &str) -> Result<String> {
    // CHECK: Empty URL
    if url.is_empty() {
        return Err(JinError::InvalidConfig {
            message: "Repository URL cannot be empty".to_string(),
        });
    }

    // CHECK: SSH URL format: git@github.com:user/repo.git
    if url.starts_with("git@") {
        return validate_ssh_url(url);
    }

    // CHECK: HTTP/HTTPS URLs
    if url.starts_with("http://") || url.starts_with("https://") {
        return validate_http_url(url);
    }

    // CHECK: Git protocol: git://host/path
    if url.starts_with("git://") {
        return validate_git_protocol_url(url);
    }

    // CHECK: Local file paths
    if url.starts_with("/") || url.starts_with("file://") {
        return Ok(url.to_string());
    }

    // FALLBACK: Accept as-is (might be a format we don't recognize)
    Ok(url.to_string())
}

/// Validate SSH URL format: git@host:path
fn validate_ssh_url(url: &str) -> Result<String> {
    // PATTERN: from import.rs:59-82 (detect_project_name)
    let rest = url.strip_prefix("git@").ok_or_else(|| JinError::InvalidConfig {
        message: "Invalid SSH URL format".to_string(),
    })?;

    // CHECK: Has colon separating host and path
    if !rest.contains(':') {
        return Err(JinError::InvalidConfig {
            message: "SSH URL must be in format: git@host:path".to_string(),
        });
    }

    // CHECK: Has path after colon
    let parts: Vec<&str> = rest.splitn(2, ':').collect();
    if parts[1].is_empty() {
        return Err(JinError::InvalidConfig {
            message: "SSH URL must include a repository path".to_string(),
        });
    }

    Ok(url.to_string())
}

/// Validate HTTP/HTTPS URL
fn validate_http_url(url: &str) -> Result<String> {
    // CHECK: Has host
    let url_parsed = url::Url::parse(url).map_err(|_| JinError::InvalidConfig {
        message: "Invalid HTTP URL format".to_string(),
    })?;

    // CHECK: Has a path
    if url_parsed.path().is_empty() || url_parsed.path() == "/" {
        return Err(JinError::InvalidConfig {
            message: "HTTP URL must include a repository path".to_string(),
        });
    }

    Ok(url.to_string())
}

/// Validate git:// protocol URL
fn validate_git_protocol_url(url: &str) -> Result<String> {
    // CHECK: Basic format validation
    if url.len() <= 7 || !url.contains('/') {
        return Err(JinError::InvalidConfig {
            message: "Invalid git:// URL format".to_string(),
        });
    }

    Ok(url.to_string())
}

/// Normalize URL by stripping .git suffix
fn normalize_url(url: &str) -> String {
    url.trim_end_matches(".git").trim().to_string()
}

// CRITICAL: Add url dependency to Cargo.toml if not already present
// [dependencies]
// url = "2.5"
```

### Integration Points

```yaml
CONFIG:
  - modify: src/core/config.rs
  - add_field: pub remote: Option<String>
  - annotation: #[serde(skip_serializing_if = "Option::is_none")]
  - pattern: Follow default_mode/default_scope pattern

CLI_ARGS:
  - already_exists: src/cli/args.rs LinkCommand struct
  - no_changes: Already defined with url field
  - verify: Confirm LinkCommand has #[arg(value_name = "URL")]

COMMAND_MODULE:
  - add_to: src/commands/mod.rs
  - add_lines: pub mod link; pub use link::execute as link_execute;
  - location: After last module, before tests

MAIN_DISPATCH:
  - add_to: src/main.rs
  - pattern: Commands::Link(cmd) => match commands::link_execute(&cmd)
  - error_handling: Follow existing pattern with eprintln! and ExitCode

CARGO_TOML:
  - check: url = "2.5" dependency
  - action: Add if not present (already likely present for other commands)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check                              # Check compilation
cargo clippy --all-targets -- -D warnings  # Lint checks
cargo fmt --check                         # Verify formatting

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - Missing use statements
# - Incorrect enum variant syntax
# - Missing serde attributes on config fields
```

### Level 2: Unit Tests (Component Validation)

```bash
# Note: This project does not have extensive unit tests yet
# Manual testing is the primary validation method

# Test URL parsing functions
cargo test parse_and_validate_url -- --nocapture
cargo test normalize_url -- --nocapture

# Test error handling
cargo test

# If no tests exist, skip to Level 3 for integration testing
# Future work: Add tests to src/commands/link_tests.rs
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the project
cargo build --release

# Test 1: Link with SSH URL
./target/release/jin link git@github.com:myorg/jin-config.git
# Expected: "Linking to remote: git@github.com:myorg/jin-config"
# Verify: cat ~/.jin/config.yaml shows remote: git@github.com:myorg/jin-config

# Test 2: Link with HTTPS URL
./target/release/jin link https://github.com/myorg/jin-config.git
# Expected: "Updating remote from git@... to https://..."
# Verify: cat ~/.jin/config.yaml shows updated URL

# Test 3: Idempotency - run same command again
./target/release/jin link https://github.com/myorg/jin-config.git
# Expected: "Already linked to remote: https://github.com/myorg/jin-config"

# Test 4: Invalid URL - empty string
./target/release/jin link ""
# Expected: "Error: Repository URL cannot be empty"
# Exit code: 2

# Test 5: Invalid SSH URL - missing path
./target/release/jin link "git@github.com"
# Expected: "Error: SSH URL must be in format: git@host:path"
# Exit code: 2

# Test 6: Local file path
./target/release/jin link /path/to/local/repo
# Expected: "Linking to remote: /path/to/local/repo"

# Test 7: Verify config file format
cat ~/.jin/config.yaml
# Expected YAML output:
# version: 1
# repository: /home/user/.jin/repo
# remote: git@github.com:myorg/jin-config
# default_mode: null
# default_scope: null

# Test 8: Test with .git suffix stripping
./target/release/jin link git@github.com:myorg/jin-config.git
cat ~/.jin/config.yaml | grep remote
# Expected: "remote: git@github.com:myorg/jin-config" (no .git suffix)
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test various URL formats from Git documentation

# SSH with custom port
./target/release/jin link "ssh://git@github.com:22/myorg/jin-config.git"

# Git protocol
./target/release/jin link "git://github.com/myorg/jin-config.git"

# File URL
./target/release/jin link "file:///path/to/local/repo"

# HTTPS with authentication (should be accepted but not validated)
./target/release/jin link "https://user:token@github.com/myorg/jin-config.git"

# Edge case: URL with trailing slash
./target/release/jin link "git@github.com:myorg/jin-config/"
# Should normalize to: git@github.com:myorg/jin-config

# Edge case: URL with multiple .git suffixes
./target/release/jin link "git@github.com:myorg/jin-config.git.git"
# Should strip all .git suffixes

# Test error messages are user-friendly
./target/release/jin link "not-a-url" 2>&1 | grep -i "invalid"
# Expected: Clear error message about invalid URL

# Test that Jin workspace still works after linking
./target/release/jin status
# Should work normally, status not affected by remote config

# Future validation (when P5 sync commands are implemented):
# ./target/release/jin fetch  # Should use the configured remote
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo check` passes with no errors
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt --check` passes (code is properly formatted)
- [ ] Code compiles with `cargo build --release`

### Feature Validation

- [ ] Success criterion: Command accepts valid Git URLs (SSH, HTTPS, git://, local paths)
- [ ] Success criterion: URL is validated before storing
- [ ] Success criterion: Remote URL is persisted to `~/.jin/config.yaml`
- [ ] Success criterion: Running command again updates the existing URL (idempotent)
- [ ] Success criterion: Clear error messages for invalid URLs
- [ ] Success criterion: JinError variants properly map to exit codes (2 for invalid URL)
- [ ] Success criterion: User-friendly output confirms the link was successful

### Code Quality Validation

- [ ] Follows existing command pattern from `init.rs`
- [ ] Uses existing error types (no new JinError variants needed)
- [ ] File placement matches desired codebase tree
- [ ] URL parsing follows pattern from `import.rs`
- [ ] Configuration save uses existing `JinConfig::save()` method
- [ ] Idempotency check prevents redundant operations
- [ ] Error messages are descriptive and actionable

### Integration Validation

- [ ] `src/commands/mod.rs` exports link module correctly
- [ ] `src/main.rs` dispatches Link command to handler
- [ ] `src/core/config.rs` has `remote` field with proper serde attributes
- [ ] No changes needed to `src/cli/args.rs` (LinkCommand already exists)
- [ ] Configuration file format is valid YAML

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Error messages explain what's wrong and how to fix
- [ ] URL formats supported are documented in error messages
- [ ] No environment variables added (no new configuration needed)

---

## Anti-Patterns to Avoid

- **Don't** try to verify connectivity to the remote URL (that's for `jin fetch`)
- **Don't** convert SSH URLs to HTTPS or vice versa (preserve user's format)
- **Don't** create a new config file if global config doesn't exist (let `JinConfig::load()` handle it)
- **Don't** use sync I/O operations when validating URLs (this is a fast config operation)
- **Don't** add new JinError variants when existing ones work (`InvalidConfig` covers URL validation)
- **Don't** hardcode path to `~/.jin/config.yaml` (use `JinConfig::jinx_config_path()`)
- **Don't** skip the idempotency check (commands should be rerunnable)
- **Don't** use unwrap() or expect() in production code (use proper error handling with `?`)
- **Don't** forget to add the `url` crate dependency to Cargo.toml if not present
- **Don't** modify `src/cli/args.rs` (LinkCommand is already defined)

---

## Implementation Notes

### Dependencies Required

- `url = "2.5"` - For HTTP/HTTPS URL parsing (may already be in Cargo.toml)
- Existing dependencies: `git2`, `serde_yaml_ng`, `thiserror`

### Files to Modify

1. `src/core/config.rs` - Add `remote` field to `JinConfig`
2. `src/commands/mod.rs` - Export link module
3. `src/main.rs` - Add Link command dispatch
4. `src/commands/link.rs` - **NEW FILE** - Main implementation

### Files to Reference (Don't Modify)

- `src/cli/args.rs` - LinkCommand already exists
- `src/commands/init.rs` - Reference for command structure
- `src/commands/import.rs` - Reference for URL parsing
- `src/core/error.rs` - Reference for error types

### Success Metrics

After implementation, the following should work:

```bash
# Fresh install
$ cargo build --release
$ ./target/release/jin link git@github.com:myorg/jin-config.git
Linking to remote: git@github.com:myorg/jin-config

# Idempotent
$ ./target/release/jin link git@github.com:myorg/jin-config.git
Already linked to remote: git@github.com:myorg/jin-config

# Update remote
$ ./target/release/jin link https://github.com/myorg/jin-config.git
Updating remote from git@github.com:myorg/jin-config to https://github.com/myorg/jin-config

# Verify
$ cat ~/.jin/config.yaml
version: 1
repository: /home/user/.jin/repo
remote: https://github.com/myorg/jin-config
```

---

**End of PRP**

Confidence Score: 9/10 - All necessary context provided, clear implementation path, specific patterns to follow, comprehensive validation steps.
