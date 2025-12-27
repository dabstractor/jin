# PRP: P4.M6 - Link Command

---

## Goal

**Feature Goal**: Implement `jin link` command to connect a local Jin installation to a shared remote configuration repository, enabling team-wide configuration synchronization.

**Deliverable**: A working `jin link <url>` command that:
1. Validates the remote repository URL
2. Configures the Jin repository at `~/.jin/` with the remote
3. Stores remote configuration in `~/.jin/config.toml`
4. Tests connectivity to the remote
5. Optionally performs initial fetch of available configurations

**Success Definition**:
- `jin link <url>` successfully adds a remote to the Jin repository
- Remote configuration is persisted in `~/.jin/config.toml`
- Remote uses Jin-specific refspec for layer synchronization
- URL validation rejects invalid formats
- Error messages guide users for common issues (auth, network, already linked)
- Command is idempotent (can re-link to update remote)

---

## User Persona

**Target User**: Developer setting up Jin for the first time on a new machine or project

**Use Case**: A developer wants to connect their local Jin installation to their team's shared configuration repository to access pre-configured modes, scopes, and project settings.

**User Journey**:
1. Developer initializes Jin: `jin init`
2. Developer links to team's config repo: `jin link git@github.com:org/jin-config`
3. Jin validates the URL and tests connectivity
4. Jin stores the remote configuration
5. Jin optionally fetches available modes/scopes (if configured)
6. Developer can now `jin fetch`, `jin pull` to sync configurations

**Pain Points Addressed**:
- **No manual remote setup** - `jin link` handles all Git remote configuration
- **Clear validation** - Immediate feedback on URL format and connectivity issues
- **Team onboarding** - Single command to connect to team configurations
- **Discoverability** - Linking shows what configurations are available from the remote

---

## Why

- **Foundation for Sync**: Link is the prerequisite for all sync operations (fetch, pull, push)
- **PRD Requirement**: Section 18.1 specifies `jin link <repo-url>` as a core initialization command
- **Team Collaboration**: Enables the primary use case of sharing configurations across team members
- **User Expectation**: Mirrors familiar Git workflow (`git remote add origin <url>`)
- **Milestone Dependency**: P5.M1 (Remote Operations) depends on link being implemented

---

## What

### User-Visible Behavior

#### `jin link <url>`
```bash
$ jin link git@github.com:myorg/jin-config
Testing connection to remote...
Connected successfully
Configured remote 'origin' for Jin repository
Stored in: ~/.jin/config.toml

Available configurations:
  Modes: claude, cursor, zed
  Scopes: language:javascript, language:python, infra:docker
  Projects: ui-dashboard, api-server

Use 'jin fetch' to download configurations
Use 'jin pull' to merge and apply configurations
```

**Behavior:**
- Validates URL format (HTTPS, SSH, Git protocol, File path)
- Opens Jin bare repository at `~/.jin/`
- Adds remote named 'origin' with Jin-specific refspec
- Tests connectivity by listing remote references
- Loads `~/.jin/config.toml` and updates remote configuration
- Saves updated config
- Optionally lists available modes/scopes/projects from remote

#### Error Cases

**Invalid URL format:**
```bash
$ jin link invalid-url
Error: Invalid remote URL format
Supported formats:
  HTTPS: https://github.com/org/repo.git
  SSH:   git@github.com:org/repo.git
  Git:   git://server.local/repo.git
  File:  /absolute/path or file:///absolute/path
```

**Remote already exists:**
```bash
$ jin link git@github.com:different-org/repo
Error: Remote 'origin' already configured
Current remote: git@github.com:myorg/jin-config

To change remote, use:
  jin link git@github.com:different-org/repo --force
```

**Connectivity issues:**
```bash
$ jin link git@github.com:org/private-repo
Testing connection to remote...
Error: Cannot access remote repository
Possible causes:
  - Repository does not exist
  - Missing SSH key or credentials
  - Network connectivity issues

Check your SSH keys: ssh -T git@github.com
```

### Success Criteria

- [ ] `jin link <url>` validates URL format for all supported protocols
- [ ] Remote is added to `~/.jin/` Git repository with name 'origin'
- [ ] Custom refspec configured: `+refs/jin/layers/*:refs/jin/layers/*`
- [ ] Remote URL is stored in `~/.jin/config.toml`
- [ ] Connectivity test succeeds before persisting configuration
- [ ] `--force` flag allows updating existing remote
- [ ] Error messages are actionable and user-friendly
- [ ] Command is idempotent (running twice produces same result)
- [ ] Works with HTTPS, SSH (both formats), Git protocol, and file:// URLs

---

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Yes** - This PRP includes:
- Complete git2-rs Remote API documentation with URLs and section anchors
- Exact patterns from existing jin commands (init, add, mode)
- URL validation patterns with Rust regex examples
- JinConfig structure and save/load methods from src/core/config.rs
- Error handling patterns from JinError enum
- Connectivity testing patterns from git2-rs examples
- Integration test patterns from tests/cli_basic.rs

### Documentation & References

```yaml
# MUST READ - Git2-rs Remote API

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.remote_with_fetch
  why: How to add a remote with custom refspec (RECOMMENDED for Jin)
  critical: |
    - repo.remote_with_fetch(name, url, refspec) creates remote with custom refspec
    - Jin requires custom refspec: +refs/jin/layers/*:refs/jin/layers/*
    - This syncs only layer refs, not standard Git branches
    - Returns Result<Remote, Error>
  section: Repository::remote_with_fetch

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.find_remote
  why: Check if remote already exists before adding
  critical: |
    - repo.find_remote(name) returns Result<Remote, Error>
    - ErrorCode::NotFound if remote doesn't exist
    - ErrorCode::Exists if trying to add duplicate remote
    - Use this to implement --force flag behavior
  section: Repository::find_remote

- url: https://docs.rs/git2/latest/git2/struct.Remote.html#method.connect
  why: Test connectivity to remote before persisting configuration
  critical: |
    - remote.connect(Direction::Fetch) tests if remote is accessible
    - Returns error if authentication fails, network unreachable, or repo not found
    - MUST disconnect after test: remote.disconnect()?
    - Provides immediate user feedback on configuration issues
  section: Remote::connect

- url: https://docs.rs/git2/latest/git2/struct.Remote.html#method.ls
  why: List available references on remote (for showing available configs)
  critical: |
    - remote.list() returns Vec<RemoteHead> with all refs
    - Use after connect() to discover modes/scopes on remote
    - Parse ref names like refs/jin/layers/mode/claude to extract "claude" mode
    - Disconnect after listing
  section: Remote::ls

- url: https://docs.rs/git2/latest/git2/enum.Direction.html
  why: Specify direction for remote connection
  critical: |
    - Direction::Fetch for read-only connection (connectivity test)
    - Direction::Push for write connection (not needed for link)
  section: Direction

# MUST READ - Jin Codebase Patterns

- file: src/core/config.rs:1-231
  why: JinConfig and RemoteConfig structure for storing remote URL
  pattern: |
    pub struct JinConfig {
        pub version: u32,
        pub remote: Option<RemoteConfig>,
        pub user: Option<UserConfig>,
    }

    pub struct RemoteConfig {
        pub url: String,
        pub fetch_on_init: bool,
    }

    impl JinConfig {
        pub fn load() -> Result<Self>  // Load from ~/.jin/config.toml
        pub fn save(&self) -> Result<()>  // Save to ~/.jin/config.toml
        pub fn default_path() -> Result<PathBuf>  // ~/.jin/config.toml
    }
  gotcha: |
    - Remote configuration is GLOBAL (stored in ~/.jin/config.toml)
    - NOT per-project (not in .jin/context)
    - Multiple projects share same remote repository
    - RemoteConfig is Option - can be None if not linked
    - Uses TOML serialization, not YAML

- file: src/commands/init.rs:1-13
  why: Command structure pattern for implementation
  pattern: |
    pub fn execute() -> Result<()> {
        // 1. Validate preconditions
        // 2. Open/create resources
        // 3. Perform operations
        // 4. Save state
        // 5. Print confirmation
        Ok(())
    }
  gotcha: |
    - Commands should be simple, delegate complex logic to modules
    - Use ? operator for error propagation
    - Print user-friendly messages, not debug output

- file: src/commands/mode.rs:25-49
  why: Name validation pattern
  pattern: |
    fn validate_mode_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(JinError::Other("Mode name cannot be empty".into()));
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(JinError::Other(
                "Mode name must contain only alphanumeric, underscore, or hyphen".into()
            ));
        }
        // Check reserved names
        let reserved = ["default", "global", "base"];
        if reserved.contains(&name) {
            return Err(JinError::Other(format!("'{}' is a reserved name", name)));
        }
        Ok(())
    }
  gotcha: |
    - Apply similar validation to remote URLs
    - Alphanumeric + specific chars only
    - Check for reserved or dangerous patterns

- file: src/git/repo.rs:1-200
  why: JinRepo wrapper for bare repository operations
  pattern: |
    JinRepo::open_or_create()? - Open ~/.jin/ or create it
    JinRepo::default_path()? - Get ~/.jin/ path
    repo.inner() - Access git2::Repository for remote operations
  gotcha: |
    - JinRepo wraps git2::Repository
    - Use .inner() to access git2 methods for remote management
    - Bare repository (no working directory)
    - Default path is ~/.jin/ (not .jin/ in project)

- file: src/core/error.rs:1-100
  why: JinError enum for consistent error handling
  pattern: |
    JinError::Config(String) - Configuration errors
    JinError::AlreadyExists(String) - Remote already configured
    JinError::NotFound(String) - Remote not found
    JinError::Other(String) - General errors
    JinError::Git(git2::Error) - Git operations (auto-converted)
  critical: |
    - Use thiserror #[error(...)] for Display implementation
    - Auto-convert from git2::Error with #[from]
    - Return Result<T> (alias for Result<T, JinError>)
    - Map git2 ErrorCode to user-friendly JinError variants

# EXTERNAL REFERENCES - URL Validation

- url: https://doc.rust-lang.org/regex/regex/index.html
  why: Regex patterns for URL validation
  critical: |
    Regex patterns for each URL format:
    - HTTPS: ^https://[^/]+/.*\.git$
    - SSH (colon): ^git@[^:]+:[^/].*\.git$
    - SSH (scheme): ^ssh://git@[^/]+/.*\.git$
    - Git: ^git://[^/]+/.*\.git$
    - File: ^(file://)?(/.*|[A-Za-z]:/.*)$

    Use regex crate: regex = "1.10"
  section: Regex patterns

# EXTERNAL REFERENCES - Research Documents

- file: plan/P4M6/research/git2_remote_api.md:1-1336
  why: Comprehensive git2-rs Remote API reference with 8 code examples
  pattern: |
    - Complete method signatures for all Remote operations
    - Working code examples for fetch, list, add remote
    - Error handling patterns for all ErrorCode variants
    - RemoteCallbacks for authentication and progress
    - Best practices and common pitfalls with solutions
  critical: |
    - Use repo.remote_with_fetch() for custom refspec
    - Always disconnect() after connect()
    - Test connectivity before persisting config
    - Map git2::ErrorCode to user-friendly messages

- file: plan/P4M6/research/remote_config_patterns.md:1-777
  why: Remote configuration patterns from similar tools
  pattern: |
    - Chezmoi, YADM, Homesick patterns analyzed
    - Store remote URL globally (~/.jin/config.toml)
    - Auto-fetch on link for better UX
    - Link → Fetch → Pull workflow
  critical: |
    - Remote URL is global, not per-project
    - Multiple projects share same remote
    - fetch_on_init flag for auto-fetch behavior

- file: plan/P4M6/research/git_remote_management.md:1-801
  why: Git remote concepts and Jin-specific requirements
  pattern: |
    - Custom refspec: +refs/jin/layers/*:refs/jin/layers/*
    - Syncs only layer refs, not Git branches
    - Minimal bandwidth, clean namespace
  critical: |
    - MUST use custom refspec, not default Git refspec
    - Default Git: +refs/heads/*:refs/remotes/origin/*
    - Jin: +refs/jin/layers/*:refs/jin/layers/*

- file: plan/P4M6/research/QUICK_REFERENCE.md:1-299
  why: One-page quick reference for implementation
  pattern: |
    - URL validation function examples
    - Error code to user message mapping
    - Configuration storage format
    - Common scenario walkthroughs
  critical: |
    - Use for quick lookup during implementation
    - Copy-paste ready validation functions
```

### Current Codebase Tree

```bash
jin/
├── Cargo.toml                    # Dependencies: clap, git2, serde, thiserror
├── src/
│   ├── main.rs                   # Calls jin::run(cli)
│   ├── lib.rs                    # Exports cli, commands, core modules
│   ├── cli/
│   │   ├── mod.rs                # Cli, Commands enum
│   │   └── args.rs               # LinkArgs { url: String }
│   ├── commands/
│   │   ├── mod.rs                # execute(cli) dispatcher
│   │   ├── link.rs               # STUB - line 10: "not yet implemented"
│   │   ├── fetch.rs              # STUB - for P5.M1
│   │   ├── pull.rs               # STUB - for P5.M1
│   │   ├── push.rs               # STUB - for P5.M1
│   │   ├── init.rs               # STUB - for P4.M2
│   │   └── [other commands]      # add.rs (complete), mode.rs, etc.
│   ├── core/
│   │   ├── config.rs             # JinConfig, RemoteConfig, ProjectContext
│   │   ├── layer.rs              # Layer enum, ref_path()
│   │   ├── error.rs              # JinError enum
│   │   └── mod.rs                # Re-exports
│   ├── git/
│   │   ├── repo.rs               # JinRepo wrapper
│   │   ├── refs.rs               # RefOps trait
│   │   ├── objects.rs            # ObjectOps trait
│   │   └── mod.rs                # Re-exports
│   └── [other modules]
├── tests/
│   └── cli_basic.rs              # Integration tests
└── plan/
    └── P4M6/
        ├── PRP.md                # This document
        └── research/             # Comprehensive research (6 documents)
            ├── git2_remote_api.md
            ├── remote_config_patterns.md
            ├── git_remote_management.md
            ├── QUICK_REFERENCE.md
            ├── INDEX.md
            └── README.md
```

### Desired Codebase Tree (After P4.M6)

```bash
src/commands/
└── link.rs          # ~150 lines: Full implementation replacing stub

# Modifications to existing files:
src/core/
└── config.rs        # NO CHANGES - RemoteConfig already exists

tests/
└── cli_basic.rs     # +80 lines: Integration tests for link command
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: Jin uses custom refspec, NOT default Git refspec
// ============================================================
// WRONG (default Git refspec):
repo.remote("origin", "git@github.com:org/repo")?;  // Uses +refs/heads/*

// CORRECT (Jin-specific refspec):
repo.remote_with_fetch(
    "origin",
    "git@github.com:org/repo",
    "+refs/jin/layers/*:refs/jin/layers/*"
)?;

// WHY: Jin stores configuration in refs/jin/layers/*, not refs/heads/*
// Default refspec would sync Git branches, which Jin doesn't use

// ============================================================
// CRITICAL: Must disconnect() after connect() for connectivity test
// ============================================================
// WRONG:
let mut remote = repo.find_remote("origin")?;
remote.connect(Direction::Fetch)?;
// Forgot to disconnect - leaves connection open!

// CORRECT:
let mut remote = repo.find_remote("origin")?;
remote.connect(Direction::Fetch)?;
remote.disconnect()?;  // REQUIRED - cleanup connection

// ============================================================
// CRITICAL: RemoteConfig is Option<RemoteConfig> in JinConfig
// ============================================================
let mut config = JinConfig::load()?;
config.remote = Some(RemoteConfig {
    url: url.to_string(),
    fetch_on_init: true,  // Enable auto-fetch on link
});
config.save()?;

// Access later:
if let Some(remote) = config.remote {
    println!("Remote URL: {}", remote.url);
}

// ============================================================
// PATTERN: Map git2::ErrorCode to user-friendly JinError
// ============================================================
use git2::ErrorCode;

match repo.find_remote("origin") {
    Ok(remote) => { /* use remote */ },
    Err(e) => {
        match e.code() {
            ErrorCode::NotFound => {
                // Remote doesn't exist - OK to add new one
            },
            ErrorCode::Exists => {
                return Err(JinError::AlreadyExists(
                    "Remote 'origin' already configured".into()
                ));
            },
            _ => return Err(e.into()),  // Other git2 errors
        }
    }
}

// ============================================================
// PATTERN: URL validation with regex
// ============================================================
use regex::Regex;

fn validate_git_url(url: &str) -> Result<()> {
    let patterns = vec![
        Regex::new(r"^https://[^/]+/.*\.git$").unwrap(),      // HTTPS
        Regex::new(r"^git@[^:]+:[^/].*\.git$").unwrap(),      // SSH (colon)
        Regex::new(r"^ssh://git@[^/]+/.*\.git$").unwrap(),    // SSH (scheme)
        Regex::new(r"^git://[^/]+/.*\.git$").unwrap(),        // Git protocol
        Regex::new(r"^(file://)?/.*$").unwrap(),              // File path
    ];

    if !patterns.iter().any(|p| p.is_match(url)) {
        return Err(JinError::Config(format!("Invalid URL format: {}", url)));
    }
    Ok(())
}

// ============================================================
// PATTERN: Test connectivity before persisting config
// ============================================================
fn test_connectivity(repo: &Repository, remote_name: &str) -> Result<bool> {
    let mut remote = repo.find_remote(remote_name)?;

    // Try to connect in Fetch direction (read-only)
    match remote.connect(Direction::Fetch) {
        Ok(_) => {
            remote.disconnect()?;
            Ok(true)
        },
        Err(e) => {
            // Map error codes to user messages
            match e.code() {
                ErrorCode::Auth => Err(JinError::Config(
                    "Authentication failed. Check SSH keys or credentials.".into()
                )),
                ErrorCode::Net => Err(JinError::Config(
                    "Network error. Check connectivity.".into()
                )),
                ErrorCode::NotFound => Err(JinError::Config(
                    "Repository not found or not accessible.".into()
                )),
                _ => Err(e.into()),
            }
        }
    }
}

// ============================================================
// GOTCHA: remote_with_fetch() doesn't add push refspec
// ============================================================
// Only adds fetch refspec, not push refspec
// For Jin, this is OK - we'll configure push in P5.M1 (push command)
// Link command only needs fetch refspec

// ============================================================
// PATTERN: --force flag to update existing remote
// ============================================================
// If remote exists and --force not specified, error
// If remote exists and --force specified, delete and re-add

if !args.force && repo.find_remote("origin").is_ok() {
    return Err(JinError::AlreadyExists(
        "Remote 'origin' already exists. Use --force to update.".into()
    ));
}

// Delete existing if --force
if args.force {
    let _ = repo.remote_delete("origin");  // Ignore error if doesn't exist
}

// ============================================================
// PATTERN: List available modes/scopes from remote
// ============================================================
fn list_remote_configs(repo: &Repository) -> Result<()> {
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Fetch)?;

    let refs = remote.list()?;
    let mut modes = Vec::new();
    let mut scopes = Vec::new();

    for head in refs {
        let name = head.name();
        // Parse refs like: refs/jin/layers/mode/claude
        if let Some(mode) = parse_mode_ref(name) {
            modes.push(mode);
        }
        if let Some(scope) = parse_scope_ref(name) {
            scopes.push(scope);
        }
    }

    remote.disconnect()?;

    if !modes.is_empty() {
        println!("Available modes: {}", modes.join(", "));
    }
    if !scopes.is_empty() {
        println!("Available scopes: {}", scopes.join(", "));
    }

    Ok(())
}
```

---

## Implementation Blueprint

### Data Models (Already Complete)

All necessary data structures exist in `src/core/config.rs`:

```rust
// src/core/config.rs (lines 13-43)
pub struct JinConfig {
    pub version: u32,
    pub remote: Option<RemoteConfig>,
    pub user: Option<UserConfig>,
}

pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}

impl JinConfig {
    pub fn load() -> Result<Self>  // Load from ~/.jin/config.toml
    pub fn save(&self) -> Result<()>  // Save to ~/.jin/config.toml
    pub fn default_path() -> Result<PathBuf>  // ~/.jin/config.toml
}
```

No new data structures needed - use existing JinConfig and RemoteConfig.

### Implementation Tasks (Dependency-Ordered)

```yaml
Task 1: IMPLEMENT URL validation helper function
  GOAL: Validate Git remote URL format

  STEPS:
    1. Add regex dependency to Cargo.toml (if not exists)
       - regex = "1.10"

    2. Create validate_git_url(url: &str) -> Result<()>
       - Check against 5 URL format patterns:
         * HTTPS: ^https://[^/]+/.*\.git$
         * SSH (colon): ^git@[^:]+:[^/].*\.git$
         * SSH (scheme): ^ssh://git@[^/]+/.*\.git$
         * Git: ^git://[^/]+/.*\.git$
         * File: ^(file://)?/.*$
       - Return JinError::Config if no match

    3. Add unit tests for all URL formats

  PATTERN: Use regex crate for pattern matching

  FOLLOW: plan/P4M6/research/QUICK_REFERENCE.md:50-80 for URL patterns

  ERROR HANDLING:
    - Invalid format -> JinError::Config with helpful message
    - Show example valid URLs in error message

  PLACEMENT: Private function in src/commands/link.rs

Task 2: IMPLEMENT connectivity test helper function
  GOAL: Test if remote is accessible before persisting config

  STEPS:
    1. Create test_connectivity(repo: &Repository, remote_name: &str) -> Result<()>

    2. Find remote by name
       - repo.find_remote(remote_name)?

    3. Connect in Fetch direction (read-only)
       - remote.connect(Direction::Fetch)?
       - Print "Testing connection to remote..."

    4. List remote refs to verify access
       - remote.list()? returns Vec<RemoteHead>
       - This confirms we can read from remote

    5. Disconnect (CRITICAL)
       - remote.disconnect()?
       - Cleanup connection resources

    6. Map git2 errors to user-friendly messages
       - ErrorCode::Auth -> "Authentication failed. Check SSH keys."
       - ErrorCode::Net -> "Network error. Check connectivity."
       - ErrorCode::NotFound -> "Repository not found or not accessible."

  PATTERN: Connect -> List -> Disconnect pattern

  FOLLOW:
    - plan/P4M6/research/git2_remote_api.md:200-250 for connect/disconnect
    - src/commands/mode.rs for error message patterns

  ERROR HANDLING:
    - Map all git2::ErrorCode to actionable user messages
    - Include hints for fixing each error type

  PLACEMENT: Private function in src/commands/link.rs

Task 3: IMPLEMENT list_remote_configs helper function (OPTIONAL)
  GOAL: Show available modes/scopes from remote after linking

  STEPS:
    1. Create list_remote_configs(repo: &Repository) -> Result<()>

    2. Connect and list remote refs
       - Same pattern as test_connectivity

    3. Parse ref names to extract modes and scopes
       - refs/jin/layers/mode/{name} -> mode name
       - refs/jin/layers/scope/{name} -> scope name
       - Use string matching or regex

    4. Print available configurations
       - "Available modes: claude, cursor, zed"
       - "Available scopes: language:javascript, infra:docker"

    5. Handle empty results gracefully
       - "No configurations found on remote yet"

  PATTERN: Same connect/disconnect pattern

  FOLLOW: Task 2 for connection pattern

  PLACEMENT: Private function in src/commands/link.rs

  NOTE: Optional enhancement, can be added in follow-up

Task 4: IMPLEMENT main execute function for link command
  GOAL: Complete implementation of jin link command

  STEPS:
    1. Extract URL from args
       - let url = &args.url;

    2. Validate URL format
       - validate_git_url(url)?;
       - Early validation before any state changes

    3. Load global Jin config
       - let mut config = JinConfig::load()?;
       - Creates default if doesn't exist

    4. Open Jin repository
       - let repo = JinRepo::open_or_create()?;
       - Opens ~/.jin/ or creates if first time

    5. Check if remote already exists
       - if repo.inner().find_remote("origin").is_ok() {
       -     if !args.force {
       -         return Err(AlreadyExists("Remote exists. Use --force."));
       -     }
       -     repo.inner().remote_delete("origin")?;
       - }

    6. Add remote with Jin-specific refspec
       - repo.inner().remote_with_fetch(
       -     "origin",
       -     url,
       -     "+refs/jin/layers/*:refs/jin/layers/*"
       - )?;

    7. Test connectivity
       - println!("Testing connection to remote...");
       - test_connectivity(repo.inner(), "origin")?;
       - println!("Connected successfully");

    8. Update and save global config
       - config.remote = Some(RemoteConfig {
       -     url: url.to_string(),
       -     fetch_on_init: true,  // Enable auto-fetch
       - });
       - config.save()?;

    9. Print confirmation
       - "Configured remote 'origin' for Jin repository"
       - "Stored in: ~/.jin/config.toml"

    10. Optionally list available configs
        - let _ = list_remote_configs(repo.inner());
        - Ignore errors, this is just informational

    11. Print next steps
        - "Use 'jin fetch' to download configurations"
        - "Use 'jin pull' to merge and apply configurations"

  PATTERN: Validate -> Load -> Modify -> Save -> Confirm

  FOLLOW:
    - src/commands/init.rs for command structure
    - src/commands/mode.rs:60-120 for error handling

  ERROR HANDLING:
    - All errors from helpers propagate up
    - If connectivity test fails, don't save config
    - Print helpful error messages with next steps

  DEPENDENCIES:
    - Task 1 (URL validation)
    - Task 2 (connectivity test)
    - Task 3 (optional - list configs)

  NAMING: execute(args: LinkArgs) -> Result<()>

  PLACEMENT: src/commands/link.rs (replace stub)

Task 5: ADD --force flag to LinkArgs (if needed)
  GOAL: Allow updating existing remote

  STEPS:
    1. Check if LinkArgs already has force flag
       - If not, add to src/cli/args.rs:

    2. Add force field to LinkArgs
       #[derive(Args, Debug)]
       pub struct LinkArgs {
           pub url: String,
           #[arg(long)]
           pub force: bool,
       }

    3. Use args.force in Task 4 step 5

  FOLLOW: src/cli/args.rs:40-50 for flag patterns

  PLACEMENT: src/cli/args.rs (if not already present)

Task 6: ADD integration tests for link command
  GOAL: Comprehensive test coverage for link command

  ADD TESTS:
    1. test_link_command_https()
       - Run jin link with HTTPS URL
       - Verify remote added to repository
       - Verify config saved with correct URL
       - Mock or use test repository

    2. test_link_command_ssh()
       - Run jin link with SSH URL (git@...)
       - Verify remote configured
       - Test both SSH URL formats (colon and scheme)

    3. test_link_invalid_url()
       - Run jin link with invalid URL
       - Expect error with helpful message
       - Test multiple invalid formats

    4. test_link_already_exists()
       - Link to a remote
       - Try to link again without --force
       - Expect AlreadyExists error
       - Try with --force, expect success

    5. test_link_connectivity_failure()
       - Link to non-existent repository
       - Expect connectivity error
       - Verify config NOT saved (rollback)

    6. test_link_file_protocol()
       - Create temporary bare repository
       - Link to it using file:// URL
       - Verify connectivity works

  FOLLOW:
    - tests/cli_basic.rs:1-300 for test patterns
    - Use assert_cmd::Command for CLI testing
    - Use tempfile::TempDir for isolation

  PLACEMENT: tests/cli_basic.rs (append new tests)

  NOTE: Some tests may require mocking or test git repositories
```

### Implementation Patterns & Key Details

```rust
// ============================================================
// PATTERN: Complete link command implementation
// ============================================================
use crate::cli::LinkArgs;
use crate::core::{JinError, JinConfig, RemoteConfig, Result};
use crate::git::JinRepo;
use git2::{Direction, ErrorCode};
use regex::Regex;

pub fn execute(args: LinkArgs) -> Result<()> {
    // 1. Validate URL format
    validate_git_url(&args.url)?;

    // 2. Load global config
    let mut config = JinConfig::load().unwrap_or_default();

    // 3. Open Jin repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 4. Check if remote exists
    match repo.find_remote("origin") {
        Ok(_) => {
            if !args.force {
                return Err(JinError::AlreadyExists(
                    "Remote 'origin' already exists. Use --force to update.".into()
                ));
            }
            // Delete existing remote
            repo.remote_delete("origin")?;
        },
        Err(e) if e.code() == ErrorCode::NotFound => {
            // No remote exists - OK to proceed
        },
        Err(e) => return Err(e.into()),
    }

    // 5. Add remote with Jin-specific refspec
    repo.remote_with_fetch(
        "origin",
        &args.url,
        "+refs/jin/layers/*:refs/jin/layers/*"
    )?;

    // 6. Test connectivity
    println!("Testing connection to remote...");
    test_connectivity(repo, "origin")?;
    println!("Connected successfully");

    // 7. Update and save config
    config.remote = Some(RemoteConfig {
        url: args.url.clone(),
        fetch_on_init: true,
    });
    config.save()?;

    // 8. Print confirmation
    println!("Configured remote 'origin' for Jin repository");
    let config_path = JinConfig::default_path()?;
    println!("Stored in: {}", config_path.display());
    println!();

    // 9. Optionally list available configs
    let _ = list_remote_configs(repo);

    // 10. Print next steps
    println!("Use 'jin fetch' to download configurations");
    println!("Use 'jin pull' to merge and apply configurations");

    Ok(())
}

// ============================================================
// HELPER: URL validation
// ============================================================
fn validate_git_url(url: &str) -> Result<()> {
    let patterns = vec![
        Regex::new(r"^https://[^/]+/.*\.git$").unwrap(),
        Regex::new(r"^git@[^:]+:[^/].*\.git$").unwrap(),
        Regex::new(r"^ssh://git@[^/]+/.*\.git$").unwrap(),
        Regex::new(r"^git://[^/]+/.*\.git$").unwrap(),
        Regex::new(r"^(file://)?/.*$").unwrap(),
    ];

    if !patterns.iter().any(|p| p.is_match(url)) {
        return Err(JinError::Config(format!(
            "Invalid remote URL format: {}\n\
            Supported formats:\n\
            HTTPS: https://github.com/org/repo.git\n\
            SSH:   git@github.com:org/repo.git\n\
            Git:   git://server.local/repo.git\n\
            File:  /absolute/path or file:///absolute/path",
            url
        )));
    }
    Ok(())
}

// ============================================================
// HELPER: Connectivity test
// ============================================================
fn test_connectivity(repo: &git2::Repository, remote_name: &str) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    // Try to connect
    match remote.connect(Direction::Fetch) {
        Ok(_) => {
            // Connection successful, disconnect
            remote.disconnect()?;
            Ok(())
        },
        Err(e) => {
            // Map error codes to user messages
            let msg = match e.code() {
                ErrorCode::Auth => {
                    "Authentication failed. Check your SSH keys or credentials.\n\
                    Test SSH: ssh -T git@github.com"
                },
                ErrorCode::Net => {
                    "Network error. Check your internet connection."
                },
                ErrorCode::NotFound => {
                    "Repository not found or not accessible.\n\
                    Verify the URL and your access permissions."
                },
                _ => "Cannot access remote repository",
            };
            Err(JinError::Config(msg.into()))
        }
    }
}

// ============================================================
// HELPER: List available configs (optional)
// ============================================================
fn list_remote_configs(repo: &git2::Repository) -> Result<()> {
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Fetch)?;

    let refs = remote.list()?;
    let mut modes = std::collections::HashSet::new();
    let mut scopes = std::collections::HashSet::new();

    for head in refs {
        let name = head.name();
        // Parse: refs/jin/layers/mode/{name}
        if let Some(captures) = Regex::new(r"refs/jin/layers/mode/([^/]+)")
            .unwrap()
            .captures(name)
        {
            modes.insert(captures[1].to_string());
        }
        // Parse: refs/jin/layers/scope/([^/]+)
        if let Some(captures) = Regex::new(r"refs/jin/layers/scope/([^/]+)")
            .unwrap()
            .captures(name)
        {
            scopes.insert(captures[1].to_string());
        }
    }

    remote.disconnect()?;

    if !modes.is_empty() || !scopes.is_empty() {
        println!("Available configurations:");
        if !modes.is_empty() {
            let mut mode_list: Vec<_> = modes.into_iter().collect();
            mode_list.sort();
            println!("  Modes: {}", mode_list.join(", "));
        }
        if !scopes.is_empty() {
            let mut scope_list: Vec<_> = scopes.into_iter().collect();
            scope_list.sort();
            println!("  Scopes: {}", scope_list.join(", "));
        }
        println!();
    }

    Ok(())
}
```

### Integration Points

```yaml
DEPENDENCIES (Add if missing):
  - regex = "1.10" in Cargo.toml for URL validation
  - git2 = "0.19" (already exists)

CORE MODULES (read-only):
  - src/core/config.rs: JinConfig, RemoteConfig (lines 13-76)
  - src/core/error.rs: JinError variants

GIT OPERATIONS (read-only):
  - src/git/repo.rs: JinRepo::open_or_create(), inner()

CLI FRAMEWORK (modify if needed):
  - src/cli/args.rs: LinkArgs (may need to add --force flag)
  - src/cli/mod.rs: Commands::Link routing (already exists)
  - src/commands/mod.rs: Dispatcher (already routes to link::execute)

EXTERNAL DEPENDENCIES:
  - git2 = "0.19": Repository, Remote, Direction, ErrorCode
  - regex = "1.10": Regex for URL validation
  - serde = "1.0": Serialize for config (already used)
  - toml = "0.8": TOML serialization for JinConfig (already used)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Type checking
cargo check
# Expected: Zero errors

# Format check
cargo fmt -- --check
# Expected: All files formatted

# Lint check
cargo clippy -- -D warnings
# Expected: Zero warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test URL validation function
cargo test --lib commands::link::tests::test_url_validation
# Expected: All URL formats validated correctly

# Test connectivity helper (if unit testable)
cargo test --lib commands::link::tests
# Expected: All unit tests pass
```

### Level 3: Integration Tests (System Validation)

```bash
# Build binary
cargo build
# Expected: Clean build

# Test link command with file:// protocol (safest for CI)
# Create temporary bare repository
mkdir /tmp/jin-test-remote
cd /tmp/jin-test-remote
git init --bare
cd -

# Test link command
cargo test --test cli_basic test_link_command
# Expected: Link succeeds, config saved

# Verify remote configuration
git --git-dir=~/.jin/ config --get remote.origin.url
# Expected: Shows configured URL

git --git-dir=~/.jin/ config --get remote.origin.fetch
# Expected: +refs/jin/layers/*:refs/jin/layers/*

# Verify JinConfig
cat ~/.jin/config.toml
# Expected: Contains [remote] section with URL

# Test --force flag
cargo test --test cli_basic test_link_force
# Expected: Can update existing remote
```

### Level 4: Manual End-to-End Validation

```bash
# Clean slate
rm -rf ~/.jin/config.toml
rm -rf ~/.jin/

# Create test remote repository
mkdir /tmp/jin-remote
cd /tmp/jin-remote
git init --bare

# Test 1: Link to local repository
jin init
jin link file:///tmp/jin-remote
# Expected:
# - "Testing connection to remote..."
# - "Connected successfully"
# - "Configured remote 'origin'"
# - "Stored in: ~/.jin/config.toml"

# Verify configuration
cat ~/.jin/config.toml
# Expected:
# [remote]
# url = "file:///tmp/jin-remote"
# fetch_on_init = true

git --git-dir=~/.jin/ remote -v
# Expected:
# origin  file:///tmp/jin-remote (fetch)

git --git-dir=~/.jin/ config --get remote.origin.fetch
# Expected: +refs/jin/layers/*:refs/jin/layers/*

# Test 2: Try to link again (should error)
jin link file:///tmp/jin-remote
# Expected: Error "Remote 'origin' already exists. Use --force."

# Test 3: Force re-link
jin link file:///tmp/jin-remote --force
# Expected: Success (remote updated)

# Test 4: Invalid URL
jin link invalid-url
# Expected: Error with helpful message showing valid formats

# Test 5: HTTPS URL (may fail connectivity, but should add remote)
jin link https://github.com/yourorg/jin-config.git --force
# Expected: Either success or connectivity error (depending on repo existence)

# Test 6: SSH URL
jin link git@github.com:yourorg/jin-config.git --force
# Expected: URL validation passes, connectivity may fail

# Cleanup
rm -rf /tmp/jin-remote
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo test --lib` all unit tests pass
- [ ] `cargo test --test cli_basic` all integration tests pass
- [ ] Binary builds successfully: `cargo build`

### Feature Validation (jin link)

- [ ] Validates HTTPS URL format correctly
- [ ] Validates SSH URL format (both colon and scheme)
- [ ] Validates Git protocol URL format
- [ ] Validates file:// path format
- [ ] Rejects invalid URL formats with helpful error
- [ ] Adds remote named 'origin' to ~/.jin/ repository
- [ ] Uses custom refspec: +refs/jin/layers/*:refs/jin/layers/*
- [ ] Stores remote URL in ~/.jin/config.toml
- [ ] Tests connectivity before persisting config
- [ ] Prints confirmation with next steps
- [ ] Supports --force flag to update existing remote
- [ ] Returns appropriate error codes (0 success, non-zero error)

### Error Handling Validation

- [ ] Invalid URL format -> Clear error with examples
- [ ] Remote already exists (no --force) -> AlreadyExists error
- [ ] Connectivity failure -> Maps git2 error to user message
- [ ] Authentication failure -> Helpful error with SSH test command
- [ ] Network error -> Clear error asking to check connection
- [ ] Repository not found -> Clear error to verify URL and permissions

### Configuration Validation

- [ ] ~/.jin/config.toml created with [remote] section
- [ ] Remote URL stored correctly in config
- [ ] fetch_on_init defaults to true
- [ ] Config persists across jin restarts
- [ ] Git remote viewable with: git --git-dir=~/.jin/ remote -v

### Integration Validation

- [ ] Link command works after jin init
- [ ] Can link to file:// repository
- [ ] Can link to HTTPS repository (with valid URL)
- [ ] Can link to SSH repository (with valid credentials)
- [ ] --force flag successfully updates existing remote
- [ ] Subsequent fetch command can use configured remote (when implemented)

### Code Quality Validation

- [ ] Follows existing patterns from src/commands/init.rs, mode.rs
- [ ] Consistent error handling with JinError
- [ ] Proper use of Result<T> return types
- [ ] No unwrap() calls (use ? operator or map_err)
- [ ] Clear variable names (config, repo, remote, url)
- [ ] Functions under 50 lines where possible
- [ ] Comments explain "why", not "what"
- [ ] Helper functions are private (not pub)

### Documentation Validation

- [ ] Module-level doc comments (//!) for link.rs
- [ ] Function doc comments for execute()
- [ ] Helper functions documented
- [ ] Error cases documented
- [ ] Integration test comments explain what they validate

---

## Anti-Patterns to Avoid

- ❌ **Don't use default Git refspec** - Must use Jin-specific refspec for layers
- ❌ **Don't forget to disconnect() after connect()** - Leaves connection open, resource leak
- ❌ **Don't persist config before testing connectivity** - Breaks on bad URL or auth failure
- ❌ **Don't use repo.remote()** - Use repo.remote_with_fetch() for custom refspec
- ❌ **Don't hardcode remote name** - Use "origin" consistently, but extract to constant if needed
- ❌ **Don't show raw git2 errors to users** - Map to JinError with actionable messages
- ❌ **Don't forget to handle --force flag** - Users need way to update existing remote
- ❌ **Don't validate URL after adding remote** - Validate BEFORE any state changes
- ❌ **Don't ignore connectivity errors** - Test connection and fail fast with clear message
- ❌ **Don't store remote in ProjectContext** - Remote is global, stored in JinConfig

---

## Confidence Score

**Rating: 9.5/10** for one-pass implementation success likelihood

**Justification:**

**Strengths:**
- ✅ All data structures already exist (JinConfig, RemoteConfig)
- ✅ Comprehensive git2-rs documentation with exact URLs and code examples
- ✅ Complete research documents (1336+ lines) covering all API methods
- ✅ URL validation patterns provided with working regex
- ✅ Connectivity test pattern clearly documented
- ✅ Error handling patterns from existing commands
- ✅ Clear helper function breakdown
- ✅ Integration test patterns from tests/cli_basic.rs
- ✅ Similar command patterns (init, mode) to follow
- ✅ All gotchas documented with solutions

**Potential Challenges:**
- ⚠️ regex dependency might need to be added to Cargo.toml (minor)
- ⚠️ --force flag might not exist in LinkArgs yet (easy to add)
- ⚠️ Integration tests may need mock or test git repositories (documented)

**Mitigations:**
- All challenges documented in implementation tasks with solutions
- Research documents provide copy-paste ready code examples
- Error mapping documented for all git2::ErrorCode values
- Validation commands test actual git repository state

**Missing Points (-0.5):**
- Could benefit from more detailed ref parsing examples for list_remote_configs
- Integration tests may need additional setup documentation for mock repositories

---

## Success Metrics

**Primary Metrics:**
- [ ] `jin link <url>` command works end-to-end
- [ ] All URL formats supported and validated
- [ ] Remote configured with correct refspec in ~/.jin/ repository
- [ ] Configuration persisted in ~/.jin/config.toml
- [ ] Connectivity tested before persistence
- [ ] `cargo test` passes with 0 failures

**Quality Metrics:**
- [ ] Code follows patterns from existing commands
- [ ] Error messages are actionable and user-friendly
- [ ] No clippy warnings
- [ ] All public functions have doc comments
- [ ] Integration tests cover success and error paths

**User Experience Metrics:**
- [ ] `jin link` completes in <3 seconds for reachable remote
- [ ] Error messages guide users to solutions
- [ ] Next steps printed after successful link
- [ ] --force flag allows updating remote without manual deletion

---

## Appendix: Supported URL Formats

| Format | Example | Pattern | Notes |
|--------|---------|---------|-------|
| HTTPS | `https://github.com/org/repo.git` | `^https://[^/]+/.*\.git$` | Works with credentials |
| SSH (colon) | `git@github.com:org/repo.git` | `^git@[^:]+:[^/].*\.git$` | Most common format |
| SSH (scheme) | `ssh://git@github.com/org/repo.git` | `^ssh://git@[^/]+/.*\.git$` | Explicit scheme |
| Git protocol | `git://server.local/repo.git` | `^git://[^/]+/.*\.git$` | Unauthenticated |
| File path | `/path/to/repo` or `file:///path/to/repo` | `^(file://)?/.*$` | Local testing |

---

## Appendix: Error Code Mapping

| git2::ErrorCode | User Message | Suggested Action |
|-----------------|--------------|------------------|
| `ErrorCode::Auth` | "Authentication failed. Check SSH keys or credentials." | Test SSH: `ssh -T git@github.com` |
| `ErrorCode::Net` | "Network error. Check your internet connection." | Verify network connectivity |
| `ErrorCode::NotFound` | "Repository not found or not accessible." | Verify URL and permissions |
| `ErrorCode::Exists` | "Remote 'origin' already exists. Use --force to update." | Add --force flag |
| Other | "Cannot access remote repository" | Check logs for details |

---

## Appendix: Jin-Specific Refspec

**Default Git Refspec:**
```
+refs/heads/*:refs/remotes/origin/*
```
- Syncs all branches from remote
- Stores in refs/remotes/origin/

**Jin Custom Refspec:**
```
+refs/jin/layers/*:refs/jin/layers/*
```
- Syncs only Jin layer refs
- Preserves namespace (refs/jin/layers/)
- Minimal bandwidth (no branch objects)
- Clean separation from Git branches

**Why Custom Refspec:**
- Jin stores configurations in `refs/jin/layers/mode/{name}`, `refs/jin/layers/scope/{name}`, etc.
- Default refspec would miss these refs (looks for refs/heads/*)
- Jin doesn't use Git branches for versioning
- Custom refspec ensures only relevant data synced

---

## Implementation Time Estimate

**Estimated Time: 4-6 hours**

- Task 1 (URL validation): 1 hour
- Task 2 (Connectivity test): 1 hour
- Task 3 (List configs - optional): 0.5 hour
- Task 4 (Main execute): 1.5 hours
- Task 5 (--force flag): 0.5 hour
- Task 6 (Integration tests): 1.5 hours
- Testing and debugging: 1 hour

**Dependencies:**
- Research complete (✅)
- Data structures exist (✅)
- No blocking issues identified (✅)
