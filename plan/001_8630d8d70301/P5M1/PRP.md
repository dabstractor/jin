# P5M1 Remote Operations PRP

---

## Goal

**Feature Goal**: Implement bidirectional remote synchronization for Jin's phantom Git layer, enabling teams to share and collaborate on configurations across machines while maintaining local isolation and workspace safety.

**Deliverable**: Four fully functional commands (`jin fetch`, `jin pull`, `jin push`, `jin sync`) that synchronize Jin layer references between local and remote repositories using git2-rs, with comprehensive progress reporting, authentication support, conflict detection, and transaction-based atomicity guarantees.

**Success Definition**:
- User can fetch remote layer updates without modifying workspace
- User can pull remote updates with automatic merge and conflict detection
- User can push local layer changes to remote repository
- User can sync (fetch + pull + apply) in single operation
- All operations properly handle authentication, network errors, and conflicts
- All operations integrate with Jin's transaction system for atomicity
- Progress is reported for long-running network operations
- Test suite validates all success and error paths

## User Persona

**Target User**: Developer using Jin to manage development environment configurations across multiple machines or collaborating with team members on shared tooling configurations.

**Use Case**:
1. Developer works on laptop, configures Claude mode settings for a project
2. Developer pushes configurations to team's shared Jin repository
3. Teammate on desktop fetches updates, sees new configurations available
4. Teammate pulls configurations, which merge with their local settings
5. Both developers now have consistent base configurations with local overrides preserved

**User Journey**:
```bash
# Initial setup (already done via `jin link`)
jin link git@github.com:acme/jin-configs.git

# Developer A: Share new configuration
jin mode use claude
jin add .claude/project.json --mode --project
jin commit -m "Add Claude settings for dashboard project"
jin push                          # Upload to remote

# Developer B: Get updates
jin fetch                         # Download remote changes
# Output: Updates available for mode/claude/project/dashboard (1 file)
jin pull                          # Merge remote changes
# Output: Merged 1 file into workspace. Run 'jin apply' to update files.
jin apply                         # Regenerate workspace
# Output: Applied 1 file to workspace
```

**Pain Points Addressed**:
- Manual configuration synchronization across machines (copy/paste, USB drives)
- Inconsistent tool configurations across team members
- Lost configuration changes when switching machines
- Difficulty sharing team-wide tool defaults while preserving local customizations
- Risk of configuration conflicts when multiple team members modify same settings

## Why

**Business Value**:
- **Collaboration**: Teams can share standardized tool configurations without Git repository pollution
- **Consistency**: All team members start from same base configurations with local override capability
- **Portability**: Developers can work on multiple machines with synchronized settings
- **Safety**: Fetch-before-push workflow prevents accidental configuration overwrites
- **Auditability**: All configuration changes tracked in Git history with proper attribution

**Integration with Existing Features**:
- Builds on `jin link` command (already implements remote setup and connectivity testing)
- Uses existing layer system (refs/jin/layers/* namespace)
- Integrates with transaction system for atomic multi-layer updates
- Works with merge engine for structured file conflict resolution
- Respects layer hierarchy (global, mode, scope, project layers)
- Preserves user-local layer (never synced, machine-specific)

**Problems This Solves**:
- **Configuration Drift**: Without sync, team members' tool configurations diverge over time
- **Onboarding Friction**: New team members must manually replicate existing configurations
- **Multi-Machine Workflow**: Developers working on multiple machines need consistent environments
- **Team Standardization**: No centralized way to distribute team-wide tool defaults
- **Change Visibility**: No visibility into configuration updates from teammates

## What

### User-Visible Behavior

**Command: `jin fetch`**
- Downloads all layer refs from remote repository
- Updates local tracking refs without modifying workspace
- Reports which layers have updates available
- Does NOT modify workspace or active layers
- Safe operation - read-only from user perspective

```bash
$ jin fetch
Fetching from origin (git@github.com:acme/jin-configs.git)...
Received 45/45 objects (100%)
remote: Compressing objects: 100% (30/30), done.

Updates available:
  - mode/claude (3 files changed)
  - scope/language:typescript (1 file changed)
  - project/ui-dashboard (2 files changed)

Run 'jin pull' to merge updates
```

**Command: `jin pull`**
- Fetches remote updates (implicit fetch)
- Merges remote layers into local layers using Jin's deep merge engine
- Detects conflicts and prompts user for resolution
- Updates layer refs atomically via transaction system
- Requires clean workspace (no uncommitted changes)

```bash
$ jin pull
Fetching from origin...
Received 12/12 objects (100%)

Merging updates:
  ✓ mode/claude: Fast-forward merge (3 files)
  ✓ scope/language:typescript: 3-way merge (1 file)
  ⚠ project/ui-dashboard: Conflict in .vscode/settings.json

Conflicts detected. Run 'jin status' to see conflicts.
Resolve conflicts and run 'jin commit' to complete merge.
```

**Command: `jin push`**
- Uploads modified local layer refs to remote repository
- Requires fetch-before-push (ensures remote is up-to-date locally)
- Detects non-fast-forward scenarios and prompts for pull
- Supports `--force` flag for force-push (with safety warnings)
- Never pushes user-local layer (machine-specific)

```bash
$ jin push
Checking remote status...
Pushing to origin (git@github.com:acme/jin-configs.git)...

Uploading changes:
  → mode/claude/project/dashboard (1 commit)
  → scope/language:typescript (2 commits)

Sent 15/15 objects (100%)
Successfully pushed 2 layers
```

**Command: `jin sync`**
- Comprehensive workflow: fetch + pull + apply
- Single command for complete synchronization
- Handles conflicts during pull phase
- Regenerates workspace files after successful merge
- Equivalent to: `jin fetch && jin pull && jin apply`

```bash
$ jin sync
Fetching from origin...
Received 8/8 objects (100%)

Merging updates...
  ✓ mode/claude: 2 files merged
  ✓ Global base: 1 file updated

Applying to workspace...
  ✓ .claude/project.json
  ✓ .vscode/settings.json (merged)

Workspace synchronized successfully
```

### Technical Requirements

**Fetch Operation**:
1. Use git2::Remote::fetch() with custom refspec `refs/jin/layers/*:refs/jin/layers/*`
2. Implement RemoteCallbacks for authentication (SSH agent → SSH keys → credential helper)
3. Implement transfer_progress callback for download progress reporting
4. Compare fetched refs with local refs to detect updates
5. Print summary of available updates grouped by layer type
6. Handle network errors with user-friendly messages
7. Support timeout configuration (default: 30 seconds)

**Pull Operation**:
1. Perform implicit fetch (reuse fetch implementation)
2. Detect which layers have remote updates (compare ref OIDs)
3. For each updated layer:
   - Read remote tree and local tree
   - Invoke Jin's merge engine (structured merge for JSON/YAML/TOML, 3-way for text)
   - Detect conflicts and create .jinmerge files if needed
4. Use LayerTransaction for atomic multi-layer updates
5. Rollback on conflict (transaction abort)
6. Require clean workspace (check for uncommitted changes via staging index)

**Push Operation**:
1. Verify remote exists (load RemoteConfig from JinConfig)
2. Check all local layers for modifications (compare with remote refs)
3. Detect which layers to push based on active context and modifications
4. Verify fetch-before-push (remote refs must be present locally)
5. Use git2::Remote::push() with appropriate refspecs
6. Implement push_update_reference callback for validation
7. Handle non-fast-forward scenarios (prompt user to pull first)
8. Support `--force` flag with explicit confirmation prompt
9. Filter out user-local layer (refs/jin/layers/local - never pushed)

**Sync Operation**:
1. Execute fetch (capture any errors)
2. Execute pull (handle conflicts)
3. Execute apply (regenerate workspace)
4. Report combined status
5. Fail gracefully at each stage with clear error messages

### Success Criteria

- [ ] `jin fetch` downloads remote refs without modifying workspace
- [ ] `jin fetch` reports available updates grouped by layer
- [ ] `jin pull` merges remote changes using Jin's deep merge engine
- [ ] `jin pull` detects and reports conflicts clearly
- [ ] `jin pull` uses LayerTransaction for atomic updates
- [ ] `jin push` uploads modified layers to remote
- [ ] `jin push` prevents force-push without explicit `--force` flag
- [ ] `jin push` never uploads user-local layer
- [ ] `jin sync` completes fetch→pull→apply workflow
- [ ] All commands handle authentication via SSH agent/keys/credential helper
- [ ] All commands report progress for long-running network operations
- [ ] All commands handle network timeouts gracefully
- [ ] All commands validate remote configuration exists (via `jin link`)
- [ ] Test suite covers success paths, error paths, authentication failures, conflicts

## All Needed Context

### Context Completeness Check

**Validation**: This PRP provides complete implementation guidance including:
- Exact file locations and patterns to follow
- git2-rs API usage with specific method signatures
- Authentication callback patterns from link.rs
- Transaction integration patterns from commit_cmd.rs
- Testing patterns from existing integration tests
- Error handling patterns from JinError enum
- Progress reporting patterns from research documentation

Someone unfamiliar with this codebase would have:
- File structure guidance (where to implement each function)
- Existing code patterns to follow (link.rs, commit_cmd.rs)
- git2-rs documentation with specific examples
- Testing framework and assertion patterns
- Error types and conversion patterns

### Documentation & References

```yaml
# MUST READ - Core git2-rs Documentation
- url: https://docs.rs/git2/latest/git2/struct.Remote.html
  why: Remote struct methods for fetch() and push() operations
  critical: |
    - fetch() requires FetchOptions with RemoteCallbacks for auth/progress
    - push() requires refspec array and PushOptions
    - connect(Direction::Fetch) for connectivity testing (see link.rs:124)
    - disconnect() must be called to cleanup (see link.rs:127)

- url: https://docs.rs/git2/latest/git2/struct.RemoteCallbacks.html
  why: Authentication and progress callback configuration
  critical: |
    - credentials() callback must return Result<Cred, Error>
    - transfer_progress() for download/upload progress (return bool: true=continue)
    - push_update_reference() for validating push success per ref
    - sideband_progress() for remote server messages

- url: https://docs.rs/git2/latest/git2/struct.FetchOptions.html
  why: Configure fetch behavior (callbacks, prune, depth)
  critical: Set remote_callbacks() with authentication and progress handlers

- url: https://docs.rs/git2/latest/git2/struct.Cred.html
  why: Credential creation for authentication
  critical: |
    - Cred::ssh_key_from_agent() - preferred (no keys on disk)
    - Cred::ssh_key() - fallback (explicit key path)
    - Cred::userpass_plaintext() - HTTPS (least secure)

# Working Examples from git2-rs Repository
- url: https://github.com/rust-lang/git2-rs/blob/master/examples/fetch.rs
  why: Complete working fetch implementation with progress callbacks
  pattern: Three-callback pattern (credentials, transfer_progress, sideband_progress)
  gotcha: Must print \r for progress line overwrite, flush stdout

- url: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
  why: Pull implementation (fetch + merge analysis + merge)
  pattern: |
    1. Fetch remote refs
    2. Use repo.merge_analysis() to check if fast-forward possible
    3. If fast-forward: update ref directly
    4. If merge needed: perform merge commit
  gotcha: Merge analysis prevents unnecessary merge commits

# Jin Codebase - Patterns to Follow
- file: src/commands/link.rs
  why: Remote management, connectivity testing, authentication patterns
  pattern: |
    - validate_git_url() for URL format validation (lines 80-114)
    - test_connectivity() using remote.connect() (lines 116-159)
    - Error mapping for auth/network failures (lines 131-156)
    - RemoteConfig persistence to JinConfig (lines 58-62)
  gotcha: |
    - Must check ErrorCode::NotFound when finding remote
    - Use Direction::Fetch for read-only connectivity test
    - Always disconnect() after connect() to cleanup

- file: src/commands/commit_cmd.rs
  why: LayerTransaction usage for atomic multi-layer updates
  pattern: |
    - LayerTransaction::begin() to start transaction (line 40)
    - tx.add_layer_update() for each layer modification (lines 68-88)
    - tx.commit() for atomic apply (line 95)
    - Rollback automatic on error via Drop
  gotcha: |
    - Transaction log persisted to disk before ref updates
    - Recovery system handles crashes during commit phase

- file: src/git/refs.rs
  why: Reference operations under refs/jin/* namespace
  pattern: |
    - RefOps::set_ref() to update layer refs (lines 42-58)
    - RefOps::find_ref() to read current ref OID (lines 60-75)
    - Reference::is_valid_name() for validation (line 51)
  gotcha: All Jin refs must use refs/jin/layers/* prefix

- file: src/core/error.rs
  why: Error types and conversion patterns
  pattern: |
    - From<git2::Error> for JinError (lines 67-71)
    - JinError::Config for configuration issues
    - JinError::MergeConflict for conflicts
    - JinError::NotInitialized for missing setup
  gotcha: Always provide context string with error variants

- file: tests/cli_basic.rs
  why: Integration testing with assert_cmd
  pattern: |
    - Command::cargo_bin("jin") for CLI testing (line 5)
    - .args([...]).assert().success() for success path
    - .stdout(predicate::str::contains("...")) for output validation
    - Multi-outcome assertions with || for flexible matching (lines 265-283)
  gotcha: |
    - Some tests may fail due to global state (existing .jin repo)
    - Use TempDir for test isolation where possible

# Research Documentation (Already Created)
- docfile: plan/P5M1/research/00_START_HERE.md
  why: Entry point for all remote sync research (5000+ lines total)
  section: Implementation Roadmap for Jin (lines 112-241)

- docfile: plan/P5M1/research/QUICK_REFERENCE.md
  why: Code snippets and cheat sheets for quick implementation
  section: Essential Code Snippets (lines 1-322)

- docfile: plan/P5M1/research/rust_git_sync_examples.md
  why: Complete working examples with Cargo patterns
  section: Section 1 (Basic examples), Section 3 (Auth), Section 6 (Testing)

- docfile: plan/P5M1/research/jin_layer_system_remote_sync.md
  why: Jin-specific architecture and sync implications
  section: Remote Sync Implications (lines 62-255)
  critical: |
    - Fetch syncs ALL refs/jin/layers/* refs (line 68)
    - Pull requires merge with Jin's deep merge engine (line 106)
    - Push only modified layers, NEVER user-local (line 162)
    - Refspec: +refs/jin/layers/*:refs/jin/layers/* (line 52)
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── mod.rs                   # Commands enum with Fetch, Pull, Push, Sync variants
│   └── args.rs                  # PushArgs struct (force flag)
├── commands/
│   ├── mod.rs                   # execute() dispatcher
│   ├── fetch.rs                 # STUB - implement fetch
│   ├── pull.rs                  # STUB - implement pull
│   ├── push.rs                  # STUB - implement push
│   ├── sync.rs                  # STUB - implement sync
│   ├── link.rs                  # REFERENCE - remote setup, auth, connectivity
│   ├── commit_cmd.rs            # REFERENCE - LayerTransaction usage
│   ├── apply.rs                 # REFERENCE - workspace regeneration
│   └── status.rs                # REFERENCE - staging index inspection
├── core/
│   ├── config.rs                # RemoteConfig, JinConfig (remote URL storage)
│   ├── error.rs                 # JinError enum, Result type
│   ├── layer.rs                 # Layer enum, ref_path() method
│   └── mod.rs                   # Public exports
├── git/
│   ├── repo.rs                  # JinRepo wrapper (inner() for git2::Repository)
│   ├── refs.rs                  # RefOps trait (set_ref, find_ref, list_refs)
│   ├── objects.rs               # ObjectOps trait (create_commit, find_tree)
│   ├── tree.rs                  # TreeOps trait (walk_tree, read_blob_content)
│   ├── transaction.rs           # LayerTransaction, RecoveryManager
│   └── mod.rs                   # Public API
├── merge/
│   ├── layer.rs                 # layer_merge() - orchestrates merge across layers
│   ├── deep.rs                  # deep_merge() - structured file merging
│   └── text.rs                  # three_way_merge() - text file merging
├── staging/
│   ├── index.rs                 # StagingIndex - uncommitted changes tracking
│   └── entry.rs                 # StagedEntry
└── main.rs                      # Entry point

tests/
├── cli_basic.rs                 # Integration tests with assert_cmd
└── common/                      # (future) shared test utilities
```

### Desired Codebase Tree with Files to Add

```bash
src/commands/
├── fetch.rs                     # Implement: execute() with fetch logic
│   # Responsibility: Download remote refs, report updates
│   # Functions: execute(), fetch_updates(), report_updates()
│
├── pull.rs                      # Implement: execute() with pull logic
│   # Responsibility: Fetch + merge remote updates into local layers
│   # Functions: execute(), merge_layer_updates(), detect_conflicts()
│
├── push.rs                      # Implement: execute(PushArgs) with push logic
│   # Responsibility: Upload modified layers to remote
│   # Functions: execute(), detect_modified_layers(), push_refs()
│
└── sync.rs                      # Implement: execute() with sync orchestration
    # Responsibility: Fetch + pull + apply workflow
    # Functions: execute()

src/git/
└── remote.rs                    # NEW FILE - remote operation helpers
    # Responsibility: Shared remote utilities (auth callbacks, progress)
    # Functions: setup_callbacks(), setup_fetch_options(), setup_push_options()

tests/
├── cli_basic.rs                 # ADD TESTS - remote command integration tests
│   # New tests: test_fetch_*, test_pull_*, test_push_*, test_sync_*
│
└── common/
    └── fixtures.rs              # NEW FILE - test fixture utilities
        # Responsibility: Create test repos, mock remotes
        # Functions: create_bare_repo(), create_test_remote()
```

### Known Gotchas of Codebase & Library Quirks

```rust
// CRITICAL: git2::Transaction is NOT truly atomic
// From src/git/transaction.rs - Jin implements two-phase commit wrapper
// Use LayerTransaction, not git2::Transaction directly
let mut tx = LayerTransaction::begin(&repo, "merge remote updates")?;
tx.add_layer_update(layer, mode, scope, project, new_oid)?;
tx.commit()?; // This is atomic via transaction log

// CRITICAL: Jin uses bare repository at ~/.jin/
// From src/git/repo.rs - no working directory, only refs and objects
let jin_repo = JinRepo::open_or_create()?;  // Creates bare repo
let repo = jin_repo.inner();  // git2::Repository

// CRITICAL: All Jin refs live under refs/jin/layers/* namespace
// From src/core/layer.rs - prevents collision with user's Git workflow
let ref_path = layer.ref_path(&mode, &scope, &project);
// Returns: "refs/jin/layers/mode/claude" etc.

// CRITICAL: RemoteConfig must exist before fetch/pull/push
// From src/core/config.rs - set via `jin link` command
let config = JinConfig::load()?;
let remote_config = config.remote
    .ok_or(JinError::Config("No remote configured. Run 'jin link <url>'.".into()))?;

// CRITICAL: Custom refspec for Jin layer refs
// From src/commands/link.rs:50 - only sync Jin refs, not all refs
repo.remote_with_fetch("origin", &url, "+refs/jin/layers/*:refs/jin/layers/*")?;

// git2-rs: Authentication callback may be called multiple times
// Must track attempt count to prevent infinite retry loops
let mut auth_attempts = 0;
callbacks.credentials(move |_url, username, _allowed| {
    auth_attempts += 1;
    if auth_attempts > 3 {
        return Err(git2::Error::from_str("Authentication failed after 3 attempts"));
    }
    Cred::ssh_key_from_agent(username.unwrap_or("git"))
});

// git2-rs: Progress callback must return true to continue
// Return false to cancel the operation
callbacks.transfer_progress(|stats| {
    if user_cancelled {
        return false;  // Cancel download
    }
    print!("Received {}/{}\r", stats.received_objects(), stats.total_objects());
    true  // Continue
});

// git2-rs: Remote refs may not match local ref names exactly
// Use refspec mapping to translate remote refs to local refs
// Example: remote "refs/heads/main" → local "refs/jin/layers/mode/main"

// Jin Merge Engine: Requires base, ours, theirs OIDs for 3-way merge
// From src/merge/layer.rs - cannot merge without common ancestor
let base_oid = repo.merge_base(local_oid, remote_oid)?;
let merge_result = layer_merge(&repo, base_oid, local_oid, remote_oid)?;

// Jin Staging: Must check for uncommitted changes before pull
// From src/staging/index.rs - pull requires clean workspace
let staging_index = StagingIndex::load()?;
if !staging_index.is_empty() {
    return Err(JinError::Config(
        "Cannot pull with uncommitted changes. Commit or reset first.".into()
    ));
}

// Error Handling: Map git2::Error to JinError with context
// From src/core/error.rs - provides user-friendly messages
match remote.fetch(&[], Some(&mut opts), None) {
    Ok(()) => Ok(()),
    Err(e) => match e.code() {
        ErrorCode::Auth => Err(JinError::Config(
            "Authentication failed. Check SSH keys or credentials.".into()
        )),
        ErrorCode::Net => Err(JinError::Config(
            "Network error. Check connection and try again.".into()
        )),
        _ => Err(e.into()),
    }
}
```

## Implementation Blueprint

### Data Models and Structure

```rust
// No new data models required - use existing structures

// From src/core/config.rs - already defined
pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}

// From src/cli/args.rs - already defined
#[derive(Args, Debug)]
pub struct PushArgs {
    #[arg(long)]
    pub force: bool,
}

// Helper struct for internal use in fetch/pull
pub struct LayerUpdate {
    pub layer: Layer,
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub local_oid: Option<Oid>,   // Current local ref (None if new)
    pub remote_oid: Oid,           // Remote ref to merge
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/git/remote.rs - Shared remote utilities
  - IMPLEMENT: setup_callbacks() - Configure RemoteCallbacks with auth and progress
  - IMPLEMENT: setup_fetch_options() - Build FetchOptions with callbacks
  - IMPLEMENT: setup_push_options() - Build PushOptions with callbacks
  - FOLLOW pattern: src/commands/link.rs credentials callback (lines 120-130)
  - NAMING: snake_case for functions, clear descriptive names
  - PLACEMENT: New file in src/git/ module
  - DEPENDENCIES: git2, std::io for progress printing
  - EXPORTS: Add to src/git/mod.rs public API

Task 2: IMPLEMENT src/commands/fetch.rs - Fetch command
  - IMPLEMENT: execute() - Main fetch logic
  - IMPLEMENT: fetch_updates() - Call git2::Remote::fetch() with options
  - IMPLEMENT: report_updates() - Compare remote vs local refs, print summary
  - FOLLOW pattern: src/commands/link.rs remote operations (lines 53-56, 120-127)
  - FOLLOW pattern: plan/P5M1/research/QUICK_REFERENCE.md fetch example (lines 6-38)
  - NAMING: execute() matches command dispatcher pattern
  - PLACEMENT: Replace stub in src/commands/fetch.rs
  - DEPENDENCIES: JinRepo, RemoteConfig, git2::Remote, remote.rs utilities
  - VALIDATION: Check RemoteConfig exists, print error if not

Task 3: IMPLEMENT src/commands/pull.rs - Pull command
  - IMPLEMENT: execute() - Main pull logic (fetch + merge)
  - IMPLEMENT: detect_updates() - Compare local vs remote refs to find changes
  - IMPLEMENT: merge_layer_updates() - Merge each updated layer via merge engine
  - IMPLEMENT: detect_conflicts() - Check merge results for conflicts
  - FOLLOW pattern: src/commands/commit_cmd.rs LayerTransaction usage (lines 40-95)
  - FOLLOW pattern: src/merge/layer.rs merge orchestration
  - FOLLOW pattern: plan/P5M1/research/rust_git_sync_examples.md pull example
  - NAMING: execute() main entry, private helpers for sub-operations
  - PLACEMENT: Replace stub in src/commands/pull.rs
  - DEPENDENCIES: Task 2 (fetch), merge::layer_merge(), LayerTransaction
  - VALIDATION: Require clean staging index (no uncommitted changes)

Task 4: IMPLEMENT src/commands/push.rs - Push command
  - IMPLEMENT: execute(PushArgs) - Main push logic
  - IMPLEMENT: detect_modified_layers() - Find layers with local changes
  - IMPLEMENT: validate_refs() - Ensure local has remote refs (fetch-before-push)
  - IMPLEMENT: push_refs() - Upload refs using git2::Remote::push()
  - FOLLOW pattern: src/commands/link.rs remote operations and error handling
  - FOLLOW pattern: plan/P5M1/research/QUICK_REFERENCE.md push example (lines 54-72)
  - NAMING: execute(args) to receive PushArgs with force flag
  - PLACEMENT: Replace stub in src/commands/push.rs
  - DEPENDENCIES: git2::Remote, remote.rs utilities, RefOps
  - VALIDATION: |
      - Check RemoteConfig exists
      - Filter out user-local layer (refs/jin/layers/local)
      - Warn on non-fast-forward, require --force to override
      - Confirm before force push with explicit prompt

Task 5: IMPLEMENT src/commands/sync.rs - Sync orchestration
  - IMPLEMENT: execute() - Call fetch → pull → apply in sequence
  - FOLLOW pattern: src/commands/apply.rs workspace regeneration (reference for apply step)
  - NAMING: execute() orchestrator, delegates to other commands
  - PLACEMENT: Replace stub in src/commands/sync.rs
  - DEPENDENCIES: Task 2 (fetch), Task 3 (pull), apply::execute()
  - ERROR HANDLING: Stop at first failure, report which stage failed

Task 6: CREATE tests/common/fixtures.rs - Test utilities
  - IMPLEMENT: create_bare_repo() - Create temporary bare repository for tests
  - IMPLEMENT: setup_test_remote() - Initialize test remote with refs
  - IMPLEMENT: create_test_layer() - Add layer refs to test repository
  - FOLLOW pattern: src/git/repo.rs test helpers (create_test_repo)
  - FOLLOW pattern: tempfile::TempDir usage in existing tests
  - NAMING: Descriptive function names, return TempDir for auto-cleanup
  - PLACEMENT: New file tests/common/fixtures.rs
  - DEPENDENCIES: tempfile, git2, JinRepo
  - PURPOSE: Reusable test fixtures to avoid duplication

Task 7: ADD tests to tests/cli_basic.rs - Integration tests
  - IMPLEMENT: test_fetch_with_no_remote() - Error when remote not configured
  - IMPLEMENT: test_fetch_with_updates() - Successful fetch reports updates
  - IMPLEMENT: test_pull_requires_clean_workspace() - Error on uncommitted changes
  - IMPLEMENT: test_pull_merge_success() - Successful merge updates refs
  - IMPLEMENT: test_push_without_fetch() - Error when remote refs missing
  - IMPLEMENT: test_push_force_flag() - Force push with --force
  - IMPLEMENT: test_sync_workflow() - Complete fetch→pull→apply flow
  - FOLLOW pattern: tests/cli_basic.rs link command tests (lines 365-476)
  - FOLLOW pattern: assert_cmd usage with flexible assertions (lines 265-283)
  - NAMING: test_<command>_<scenario> pattern
  - PLACEMENT: Append to tests/cli_basic.rs
  - DEPENDENCIES: Task 6 (fixtures), assert_cmd, predicates
  - COVERAGE: |
      - Success paths for all commands
      - Error paths (no remote, auth failure, conflicts)
      - Validation (uncommitted changes, fetch-before-push)
      - Force operations (--force flag)

Task 8: UPDATE src/git/mod.rs - Export remote utilities
  - MODIFY: Add pub mod remote; pub use remote::*;
  - FOLLOW pattern: Existing module exports
  - PLACEMENT: src/git/mod.rs
  - DEPENDENCIES: Task 1 (remote.rs created)

Task 9: UPDATE Cargo.toml - Add any new dependencies (if needed)
  - CHECK: Verify git2 version supports required features
  - ADD: Any additional crates needed (currently none expected)
  - FOLLOW pattern: Existing dependency declarations
  - PLACEMENT: Cargo.toml [dependencies] section
```

### Implementation Patterns & Key Details

```rust
// ============================================================================
// Pattern 1: Fetch Implementation
// ============================================================================
// Location: src/commands/fetch.rs

use crate::core::{JinConfig, Result, JinError};
use crate::git::{JinRepo, RefOps};
use git2::{Direction, RemoteCallbacks, FetchOptions};
use std::io::{self, Write};

pub fn execute() -> Result<()> {
    // 1. Load configuration and validate remote exists
    let config = JinConfig::load()?;
    let remote_config = config.remote.ok_or(
        JinError::Config("No remote configured. Run 'jin link <url>'.".into())
    )?;

    // 2. Open Jin repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 3. Setup callbacks for authentication and progress
    let mut callbacks = RemoteCallbacks::new();

    // Authentication: SSH agent → SSH keys → fail
    let mut auth_attempts = 0;
    callbacks.credentials(move |_url, username, _allowed| {
        auth_attempts += 1;
        if auth_attempts > 3 {
            return Err(git2::Error::from_str("Authentication failed"));
        }
        git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
    });

    // Progress: Track download progress
    callbacks.transfer_progress(|stats| {
        if stats.total_objects() > 0 {
            let percent = (stats.received_objects() * 100) / stats.total_objects();
            print!("Received {}/{} objects ({}%)\r",
                stats.received_objects(),
                stats.total_objects(),
                percent);
            io::stdout().flush().unwrap();
        }
        true  // Continue
    });

    // Sideband: Show remote messages
    callbacks.sideband_progress(|data| {
        print!("remote: {}", String::from_utf8_lossy(data));
        io::stdout().flush().unwrap();
        true
    });

    // 4. Configure fetch options
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // 5. Perform fetch
    println!("Fetching from origin ({})...", remote_config.url);
    let mut remote = repo.find_remote("origin")?;

    // Fetch with custom refspec (only Jin layers)
    remote.fetch(&["refs/jin/layers/*"], Some(&mut fetch_opts), None)?;
    println!();  // New line after progress

    // 6. Compare local vs remote refs to find updates
    report_updates(&jin_repo)?;

    Ok(())
}

fn report_updates(jin_repo: &JinRepo) -> Result<()> {
    // List all remote refs
    let remote_refs = jin_repo.list_refs("refs/jin/layers/*")?;

    // Group by layer type and report differences
    let mut updates = Vec::new();
    for remote_ref in remote_refs {
        let local_ref = jin_repo.find_ref(&remote_ref.name);

        match local_ref {
            Ok(local) if local.target() != remote_ref.target() => {
                updates.push(remote_ref.name.clone());
            }
            Err(_) => {
                // New ref from remote
                updates.push(remote_ref.name.clone());
            }
            _ => {} // No changes
        }
    }

    if updates.is_empty() {
        println!("Already up to date");
    } else {
        println!("\nUpdates available:");
        for ref_name in updates {
            // Parse layer type from ref name
            let display_name = ref_name
                .strip_prefix("refs/jin/layers/")
                .unwrap_or(&ref_name);
            println!("  - {}", display_name);
        }
        println!("\nRun 'jin pull' to merge updates");
    }

    Ok(())
}

// ============================================================================
// Pattern 2: Pull Implementation with LayerTransaction
// ============================================================================
// Location: src/commands/pull.rs

use crate::git::{JinRepo, RefOps, LayerTransaction};
use crate::merge::layer_merge;
use crate::staging::StagingIndex;

pub fn execute() -> Result<()> {
    // 1. Verify clean workspace
    let staging = StagingIndex::load()?;
    if !staging.is_empty() {
        return Err(JinError::Config(
            "Cannot pull with uncommitted changes. Commit or reset first.".into()
        ));
    }

    // 2. Implicit fetch
    super::fetch::execute()?;

    // 3. Open repository and start transaction
    let jin_repo = JinRepo::open_or_create()?;
    let mut tx = LayerTransaction::begin(&jin_repo, "merge remote updates")?;

    // 4. Detect which layers need merging
    let updates = detect_updates(&jin_repo)?;

    if updates.is_empty() {
        println!("Already up to date");
        return Ok(());
    }

    println!("\nMerging updates:");

    // 5. Merge each updated layer
    let mut conflicts = Vec::new();

    for update in updates {
        let layer_name = format_layer_name(&update);

        // Get merge base (common ancestor)
        let base_oid = match jin_repo.merge_base(
            update.local_oid.unwrap(),
            update.remote_oid
        ) {
            Ok(oid) => oid,
            Err(_) => {
                // No common ancestor - use empty tree
                jin_repo.create_tree(&[])?
            }
        };

        // Perform merge using Jin's merge engine
        match layer_merge(&jin_repo, base_oid, update.local_oid.unwrap(), update.remote_oid) {
            Ok(merge_result) => {
                if merge_result.has_conflicts {
                    println!("  ⚠ {}: Conflicts detected", layer_name);
                    conflicts.push(layer_name);
                } else {
                    // Add successful merge to transaction
                    tx.add_layer_update(
                        update.layer,
                        update.mode,
                        update.scope,
                        update.project,
                        merge_result.tree_oid,
                    )?;
                    println!("  ✓ {}: Merged successfully", layer_name);
                }
            }
            Err(e) => {
                println!("  ✗ {}: Merge failed - {}", layer_name, e);
                conflicts.push(layer_name);
            }
        }
    }

    // 6. Abort on conflicts, commit otherwise
    if !conflicts.is_empty() {
        println!("\nConflicts detected in {} layers", conflicts.len());
        println!("Resolve conflicts and run 'jin commit' to complete merge.");
        return Err(JinError::MergeConflict {
            path: conflicts.join(", ")
        });
    }

    // 7. Commit transaction (atomic)
    tx.commit()?;

    println!("\nMerge completed successfully");
    println!("Run 'jin apply' to update workspace files");

    Ok(())
}

// ============================================================================
// Pattern 3: Push Implementation with Force Detection
// ============================================================================
// Location: src/commands/push.rs

use crate::cli::PushArgs;

pub fn execute(args: PushArgs) -> Result<()> {
    // 1. Validate remote configuration
    let config = JinConfig::load()?;
    let remote_config = config.remote.ok_or(
        JinError::Config("No remote configured. Run 'jin link <url>'.".into())
    )?;

    // 2. Open repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 3. Verify fetch-before-push (local must have remote refs)
    let remote_refs = jin_repo.list_refs("refs/jin/layers/*")?;
    if remote_refs.is_empty() {
        return Err(JinError::Config(
            "Cannot push without fetching first. Run 'jin fetch'.".into()
        ));
    }

    // 4. Detect modified layers (exclude user-local)
    let modified_refs = detect_modified_layers(&jin_repo)?;

    if modified_refs.is_empty() {
        println!("Nothing to push");
        return Ok(());
    }

    // 5. Setup callbacks
    let mut callbacks = RemoteCallbacks::new();

    // Auth callback (same as fetch)
    callbacks.credentials(|_url, username, _allowed| {
        git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
    });

    // Push validation callback
    callbacks.push_update_reference(|refname, status| {
        match status {
            Some(msg) => {
                eprintln!("Failed to push {}: {}", refname, msg);
                Err(git2::Error::from_str(msg))
            }
            None => {
                println!("  → {}", refname);
                Ok(())
            }
        }
    });

    // 6. Build refspecs for push
    let mut refspecs = Vec::new();
    for ref_name in &modified_refs {
        let refspec = if args.force {
            format!("+{}:{}", ref_name, ref_name)  // Force push
        } else {
            format!("{}:{}", ref_name, ref_name)   // Normal push
        };
        refspecs.push(refspec);
    }

    // 7. Warn on force push
    if args.force {
        println!("WARNING: Force push will overwrite remote changes!");
        // TODO: Add confirmation prompt in future
    }

    // 8. Perform push
    println!("Pushing to origin ({})...", remote_config.url);
    let mut remote = repo.find_remote("origin")?;

    let mut push_opts = git2::PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    let refspec_refs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();
    remote.push(&refspec_refs, Some(&mut push_opts))?;

    println!("\nSuccessfully pushed {} layers", modified_refs.len());

    Ok(())
}

fn detect_modified_layers(jin_repo: &JinRepo) -> Result<Vec<String>> {
    let local_refs = jin_repo.list_refs("refs/jin/layers/*")?;
    let mut modified = Vec::new();

    for local_ref in local_refs {
        // Skip user-local layer (never push)
        if local_ref.name.contains("/local") {
            continue;
        }

        // Check if ref differs from remote
        // For now, push all non-local refs
        // TODO: Optimize to only push actually modified refs
        modified.push(local_ref.name.clone());
    }

    Ok(modified)
}

// ============================================================================
// Pattern 4: Sync Orchestration
// ============================================================================
// Location: src/commands/sync.rs

pub fn execute() -> Result<()> {
    println!("=== Jin Sync: Fetch + Pull + Apply ===\n");

    // Step 1: Fetch
    println!("Step 1: Fetching remote updates...");
    super::fetch::execute()?;

    // Step 2: Pull (merge)
    println!("\nStep 2: Merging updates...");
    super::pull::execute()?;

    // Step 3: Apply to workspace
    println!("\nStep 3: Applying to workspace...");
    super::apply::execute(Default::default())?;

    println!("\n=== Sync completed successfully ===");

    Ok(())
}

// GOTCHA: Sync does NOT handle errors gracefully - stops at first failure
// PATTERN: Each step delegates to existing command implementations
// CRITICAL: Order matters - fetch before pull, pull before apply
```

### Integration Points

```yaml
CLI:
  - wire to: src/cli/mod.rs Commands enum
  - pattern: "Commands::Fetch => fetch::execute(),"
  - verify: All four commands (Fetch, Pull, Push, Sync) wired

GIT:
  - add module: src/git/remote.rs for shared utilities
  - export: Update src/git/mod.rs with pub mod remote
  - pattern: Follow existing RefOps, ObjectOps trait pattern

MERGE:
  - use: src/merge/layer.rs::layer_merge() for pull operation
  - pattern: Pass base, local, remote OIDs to merge engine

TRANSACTION:
  - use: src/git/transaction.rs::LayerTransaction for pull
  - pattern: begin() → add_layer_update() → commit()

ERROR:
  - extend: No new error variants needed
  - use: JinError::Config for configuration/network errors
  - use: JinError::MergeConflict for pull conflicts
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# After implementing each command file
cargo check --all-targets          # Type checking
cargo clippy -- -D warnings        # Linting
cargo fmt --check                  # Format validation

# Expected: Zero errors, zero warnings
# Fix any issues before proceeding to next task
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test individual command modules
cargo test --lib commands::fetch   # Fetch command tests
cargo test --lib commands::pull    # Pull command tests
cargo test --lib commands::push    # Push command tests
cargo test --lib commands::sync    # Sync command tests

# Test shared utilities
cargo test --lib git::remote       # Remote utilities tests

# Full library test suite
cargo test --lib

# Expected: All tests passing
# If failures, debug and fix implementation
```

### Level 3: Integration Testing (System Validation)

```bash
# Run CLI integration tests
cargo test --test cli_basic -- test_fetch    # Fetch integration tests
cargo test --test cli_basic -- test_pull     # Pull integration tests
cargo test --test cli_basic -- test_push     # Push integration tests
cargo test --test cli_basic -- test_sync     # Sync integration tests

# Run all integration tests
cargo test --test cli_basic

# Expected: All integration tests passing
# Validate stdout/stderr output matches expected patterns
```

### Level 4: Manual Testing (End-to-End Validation)

```bash
# Setup test scenario
cd /tmp
mkdir jin-test && cd jin-test
git init test-repo && cd test-repo
jin init

# Test: Link to remote
jin link git@github.com:your-org/jin-configs.git
# Expected: "Configured remote 'origin'"

# Test: Fetch from empty remote
jin fetch
# Expected: "Already up to date" or "No refs found"

# Test: Create local layer and push
jin mode use test-mode
jin add .test-config.json --mode
jin commit -m "Add test config"
jin push
# Expected: "Successfully pushed 1 layers"

# Test: Fetch on another machine (simulate)
cd /tmp
mkdir jin-test2 && cd jin-test2
git init test-repo2 && cd test-repo2
jin init
jin link git@github.com:your-org/jin-configs.git
jin fetch
# Expected: "Updates available: mode/test-mode"

# Test: Pull updates
jin pull
# Expected: "Merged successfully" or conflict message

# Test: Sync workflow
jin sync
# Expected: Fetch → Pull → Apply with success messages

# Test: Push without fetch (error case)
# (Simulate by having stale local state)
jin push
# Expected: "Cannot push without fetching first"

# Test: Force push
jin push --force
# Expected: Warning message + successful push

# Test: Pull with uncommitted changes (error case)
jin add .another-file.json --mode
jin pull
# Expected: "Cannot pull with uncommitted changes"
```

## Final Validation Checklist

### Technical Validation

- [ ] All commands compile without errors: `cargo check --all-targets`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted correctly: `cargo fmt --check`
- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test cli_basic`
- [ ] Remote utilities properly exported from git module

### Feature Validation

- [ ] `jin fetch` downloads remote refs without modifying workspace
- [ ] `jin fetch` reports available updates grouped by layer type
- [ ] `jin pull` requires clean workspace (errors on uncommitted changes)
- [ ] `jin pull` merges remote changes using LayerTransaction
- [ ] `jin pull` detects and reports conflicts clearly
- [ ] `jin push` validates remote configuration exists
- [ ] `jin push` filters out user-local layer
- [ ] `jin push` requires `--force` for force-push
- [ ] `jin push` validates fetch-before-push
- [ ] `jin sync` executes fetch → pull → apply workflow
- [ ] All commands handle authentication via SSH agent
- [ ] All commands report progress for network operations
- [ ] All commands handle network errors gracefully
- [ ] All commands provide user-friendly error messages

### Code Quality Validation

- [ ] Follows existing command implementation patterns (link.rs, commit_cmd.rs)
- [ ] Uses LayerTransaction for atomic multi-layer updates
- [ ] Properly converts git2::Error to JinError with context
- [ ] Authentication callback limits retry attempts (prevent infinite loop)
- [ ] Progress callbacks return true to continue, false to cancel
- [ ] Remote refs use refs/jin/layers/* namespace
- [ ] User-local layer never synced to remote

### Documentation & Testing

- [ ] Integration tests cover success paths for all commands
- [ ] Integration tests cover error paths (no remote, auth failure, conflicts)
- [ ] Integration tests use flexible assertions for multiple valid outcomes
- [ ] Test fixtures use TempDir for isolation and auto-cleanup
- [ ] Manual testing scenarios completed successfully

---

## Anti-Patterns to Avoid

- ❌ Don't use git2::Transaction directly - use LayerTransaction for atomicity
- ❌ Don't push user-local layer (refs/jin/layers/local) - it's machine-specific
- ❌ Don't allow pull with uncommitted changes - requires clean workspace
- ❌ Don't retry authentication infinitely - track attempts and fail after 3
- ❌ Don't forget to call remote.disconnect() - cleanup connections
- ❌ Don't hardcode "origin" everywhere - get from RemoteConfig.url
- ❌ Don't skip fetch-before-push validation - prevents accidental overwrites
- ❌ Don't allow force push without explicit --force flag
- ❌ Don't forget to filter refs by refs/jin/layers/* - avoid fetching all refs
- ❌ Don't modify workspace in fetch - it's a read-only operation
- ❌ Don't skip progress reporting - network operations can be slow
- ❌ Don't use generic error messages - provide context (auth failed, network error, etc.)

---

## Confidence Score

**9/10** - Very High Confidence for One-Pass Implementation Success

**Rationale**:
- Extensive research documentation (5000+ lines) with working examples
- Existing link.rs provides proven pattern for remote operations
- Existing commit_cmd.rs provides proven pattern for LayerTransaction usage
- Clear implementation blueprint with specific file locations and patterns
- Comprehensive testing strategy with existing test framework
- All required dependencies already in Cargo.toml (git2 0.19)
- Well-defined error handling with existing JinError variants
- git2-rs API well-documented with official examples

**Risk Areas** (-1 point):
- Authentication callback complexity (SSH agent fallback chain)
- Merge conflict detection and resolution UI/UX (may need iteration)
- Network timeout handling (may need tuning)
- Performance with large repositories (untested scenario)

**Mitigation**:
- Start with SSH agent-only auth (simplify), add fallbacks later
- Use existing conflict detection patterns from merge engine
- Set conservative timeouts initially, tune based on feedback
- Optimize refspec filtering to minimize data transfer

---

## Success Metrics

**Validation**: This PRP enables successful implementation because:

1. **Complete Context**:
   - Exact file paths and code patterns from existing codebase
   - git2-rs documentation with specific method signatures
   - Working examples from research documentation
   - Error handling patterns from JinError enum

2. **Proven Patterns**:
   - link.rs demonstrates remote operations successfully
   - commit_cmd.rs demonstrates transaction usage successfully
   - Existing tests demonstrate integration testing patterns

3. **Clear Dependencies**:
   - Task ordering ensures each task builds on previous work
   - All dependencies (git2, JinRepo, LayerTransaction) already exist
   - No new data structures required

4. **Comprehensive Testing**:
   - Integration tests cover all success and error paths
   - Manual testing scenarios validate end-to-end workflows
   - Existing test framework (assert_cmd) proven and working

An AI agent with access to this PRP and the codebase has everything needed to:
- Understand the requirements and architecture
- Follow proven implementation patterns
- Use correct git2-rs APIs with proper configuration
- Integrate with existing transaction and merge systems
- Write comprehensive tests using existing patterns
- Handle errors appropriately with user-friendly messages
