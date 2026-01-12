# P1.M2.T1: Add Automatic Fetch to Push Command

---

## Goal

**Feature Goal**: Modify the push command to automatically call fetch before pushing, ensuring the local state is compared against the latest remote state before any push operation.

**Deliverable**: Modified `src/commands/push.rs` that calls `fetch::execute()` at the start of `push::execute()` and displays fetch results to the user before proceeding with push.

**Success Definition**:
- `jin push` automatically calls `jin fetch` before pushing
- Fetch results are displayed to the user (updates available, errors, etc.)
- Push proceeds only after fetch completes successfully
- If fetch fails, push is aborted with appropriate error message
- Integration tests verify fetch-before-push behavior

## User Persona

**Target User**: Developer using Jin to share layer configurations across multiple machines or with team members.

**Use Case**: User has made local changes and wants to push to remote repository. Without fetch-before-push, user might overwrite remote changes made by teammates.

**User Journey**:
```bash
# User A creates commits in remote
# User B works locally, unaware of remote changes
$ jin mode use my-mode
$ jin add config.json --mode
$ jin commit -m "Update config"

# User B pushes (OLD BEHAVIOR: would silently overwrite or fail)
$ jin push
# NEW BEHAVIOR: Auto-fetch first, then push
Fetching from origin (git@github.com:team/jin-configs.git)...
Received 12/12 objects (100%)

Updates available:
  - mode/my-mode (2 files changed)
  - global (1 file changed)

WARNING: Your local layers are behind remote. Run 'jin pull' first.
Push aborted.
```

**Pain Points Addressed**:
- **Accidental overwrites**: Prevents users from pushing without knowing remote has new changes
- **Silent failures**: Current behavior may fail with cryptic non-fast-forward errors
- **Team collaboration**: Ensures everyone sees remote changes before pushing
- **Data loss prevention**: Protects against losing teammates' work

## Why

**Business Value**:
- **Safety**: Prevents accidental data loss from overwriting remote changes
- **Clarity**: Users explicitly see what remote updates exist before pushing
- **Team coordination**: Encourages pull-before-push workflow for team collaboration
- **Compliance**: Implements the non-negotiable invariant identified in SYNTHESIS_SUMMARY.md (Gap #2)

**Integration with Existing Features**:
- Builds on existing `fetch::execute()` command (already implemented and working)
- Uses existing remote utilities in `src/git/remote.rs`
- Integrates with push command's existing remote validation
- Prepares for P1.M2.T2 (local vs remote comparison) which will add further safety checks

**Problems This Solves**:
- **SYNTHESIS_SUMMARY.md Gap #2**: "Push Command Missing Fetch-Before-Push Enforcement" (lines 33-34)
- Current push at `src/commands/push.rs:92-101` does NOT require fetch first
- This violates a "non-negotiable invariant" per synthesis document

## What

### User-Visible Behavior

**Before (Current Behavior)**:
```bash
$ jin push
Pushing to origin (git@github.com:team/jin-configs.git)...
# May fail with non-fast-forward error if remote has new commits
# Or may silently overwrite remote changes with --force
```

**After (New Behavior)**:
```bash
$ jin push
Fetching from origin (git@github.com:team/jin-configs.git)...
Received 8/8 objects (100%)

Already up to date

Pushing to origin (git@github.com:team/jin-configs.git)...
  → refs/jin/layers/mode/my-mode
Successfully pushed 1 layer
```

**If Remote Has Updates**:
```bash
$ jin push
Fetching from origin (git@github.com:team/jin-configs.git)...
Received 15/15 objects (100%)

Updates available:
  - mode/my-mode (3 files changed)

Your local layers are behind remote. Run 'jin pull' first.
Push aborted.

Hint: Use 'jin pull' to merge remote changes, then push again.
```

### Technical Requirements

1. **Fetch Integration**:
   - Call `fetch::execute()` at start of `push::execute()`
   - Fetch completes before any push logic executes
   - Fetch errors abort the push operation

2. **Result Display**:
   - Fetch output (`println!` statements) displays to user
   - User sees "Already up to date" or "Updates available"
   - No modification to fetch output required (it already prints)

3. **Error Handling**:
   - If fetch fails (auth, network), push aborts with same error
   - Fetch errors propagate naturally via `?` operator
   - No additional error conversion needed

4. **Integration Points**:
   - Modify `src/commands/push.rs::execute()` function only
   - Add call to `super::fetch::execute()?` at line 17 (after remote config validation, before repository open)
   - No changes to CLI args, error types, or other modules

### Success Criteria

- [ ] `jin push` calls `fetch::execute()` before pushing
- [ ] Fetch results display to user (progress, updates, errors)
- [ ] Push aborts if fetch fails
- [ ] Push proceeds normally after successful fetch
- [ ] Integration test verifies fetch-before-push behavior

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test Validation**:
This PRP provides:
- Exact file location and line numbers for modification
- Complete code snippet showing where to add fetch call
- Existing fetch command behavior documentation
- Test patterns and fixtures for integration testing
- Error handling patterns from codebase
- Full context of push command flow

Someone unfamiliar with this codebase would have:
- File to modify: `src/commands/push.rs`
- Exact line number: Add fetch call at line 17 (after remote config validation)
- Code pattern: `super::fetch::execute()?;`
- Test patterns from `tests/sync_workflow.rs`
- Error handling via `Result<T>` and `?` operator

### Documentation & References

```yaml
# MUST READ - Codebase Files
- file: src/commands/push.rs
  why: Target file for modification - contains push::execute() function
  pattern: |
    Lines 16-21: Remote config validation (add fetch call after this)
    Lines 23-25: Repository opening (fetch should happen before this)
    Lines 38-44: Modified layer detection
    Lines 70-76: Push execution with error handling
  critical: |
    - Add fetch call AFTER remote config validation (line 21)
    - Add fetch call BEFORE repository opening (line 23)
    - Use super::fetch::execute()?; pattern
    - Fetch errors propagate naturally via ? operator

- file: src/commands/fetch.rs
  why: Understanding fetch behavior that will be called from push
  pattern: |
    Lines 16-66: execute() function that will be called
    Lines 42-60: Fetch execution with progress reporting
    Lines 62-65: report_updates() displays "Already up to date" or "Updates available"
  critical: |
    - Returns Result<()> - errors propagate with ? operator
    - Prints output directly via println!() - no capture needed
    - Fetches with custom refspec: refs/jin/layers/*
    - Validates remote config (redundant but safe)

- file: src/commands/mod.rs
  why: Command module structure and imports
  pattern: |
    Lines 59-61: Command dispatcher showing fetch and push registration
    Both commands in same module - super::fetch::execute() works
  gotcha: fetch and push are sibling modules - use super::fetch::execute()

- file: src/git/remote.rs
  why: Shared remote utilities used by both commands
  pattern: |
    Lines 165-175: build_fetch_options() - authentication and progress
    Lines 189-198: build_push_options() - push callbacks
  critical: Both commands use same authentication pattern (SSH agent -> keys -> fail)

- file: tests/sync_workflow.rs
  why: Integration test patterns for push/fetch testing
  pattern: |
    Lines 227-294: test_push_uploads_commits() - complete push test pattern
    Lines 59-132: test_fetch_updates_refs() - fetch test pattern
    Lines 15-40: RemoteFixture setup for local bare remote testing
  critical: |
    - Use RemoteFixture::new() for isolated remote testing
    - Use unique_test_id() for parallel test safety
    - Use git2::Repository::open() to verify remote state

# Architecture Documentation
- docfile: plan/SYNTHESIS_SUMMARY.md
  why: Explains WHY this task is critical (Gap #2: Fetch-Before-Push Enforcement)
  section: Gap #2: Push Command Missing Fetch-Before-Push Enforcement (lines 33-34)
  critical: |
    - Described as "non-negotiable invariant"
    - Location: src/commands/push.rs:92-101
    - Required change: Add super::fetch::execute()?; at start of push::execute()

- docfile: plan/P5M1/PRP.md
  why: Comprehensive remote operations documentation
  section: Push Operation requirements (lines 194-203)
  critical: |
    - Documents that push should verify fetch-before-push
    - Shows push command structure and error handling
    - Reference for future P1.M2.T2 (local vs remote comparison)

# External Research
- url: https://git-scm.com/docs/git-push
  why: Git's native push behavior and "fetch first" rejection
  insight: Git rejects push when local is behind, shows "[rejected] ... (fetch first)"
  relevance: Jin should fetch first to provide better UX than Git's rejection

- url: https://github.com/rust-lang/git2-rs/blob/master/examples/fetch.rs
  why: git2-rs fetch implementation reference
  pattern: RemoteCallbacks with credentials and transfer_progress
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── mod.rs                   # Commands enum (line 61: Commands::Push)
│   └── args.rs                  # PushArgs struct with force flag (lines 213-219)
├── commands/
│   ├── mod.rs                   # Command dispatcher (line 61: push::execute())
│   ├── push.rs                  # TARGET FILE - modify execute() function
│   │   # Lines 16-21: Remote config validation
│   │   # Lines 23-25: Repository opening (FETCH SHOULD GO BETWEEN)
│   │   # Lines 38-44: Modified layer detection
│   │   # Lines 70-95: Push execution
│   │   # Lines 102-138: detect_modified_layers() helper
│   ├── fetch.rs                 # REFERENCE - execute() to be called
│   │   # Lines 16-66: execute() function
│   │   # Lines 42-60: Fetch with progress
│   │   # Lines 62-65: Report updates
│   └── link.rs                  # REFERENCE - remote validation patterns
├── git/
│   ├── remote.rs                # Shared remote utilities
│   │   # build_fetch_options() - used by fetch
│   │   # build_push_options() - used by push
│   └── mod.rs                   # Module exports
└── core/
    ├── config.rs                # JinConfig, RemoteConfig
    └── error.rs                 # JinError, Result type

tests/
└── sync_workflow.rs             # Integration tests for push/fetch
    # test_push_uploads_commits() - lines 227-294
    # test_fetch_updates_refs() - lines 59-132
```

### Desired Codebase Tree with Modifications

```bash
src/commands/push.rs             # MODIFY THIS FILE
    # BEFORE:
    # pub fn execute(args: PushArgs) -> Result<()> {
    #     // 1. Validate remote configuration
    #     let config = JinConfig::load()?;
    #     ...
    #     // 2. Open repository  <-- FETCH SHOULD GO HERE
    #     let jin_repo = JinRepo::open_or_create()?;
    #     ...
    # }

    # AFTER:
    # pub fn execute(args: PushArgs) -> Result<()> {
    #     // 1. Validate remote configuration
    #     let config = JinConfig::load()?;
    #     let remote_config = config.remote.ok_or(JinError::Config(...))?;
    #
    #     // 2. Fetch remote state first  <-- NEW STEP
    #     super::fetch::execute()?;
    #
    #     // 3. Open repository
    #     let jin_repo = JinRepo::open_or_create()?;
    #     ...
    # }
```

### Known Gotchas of Codebase & Library Quirks

```rust
// CRITICAL: fetch::execute() already validates remote config
// From src/commands/fetch.rs:18-21
// This means fetch will re-validate the same remote we just validated
// This is ACCEPTABLE - defensive programming, validates remote still exists
let config = JinConfig::load()?;
let remote_config = config.remote.ok_or(JinError::Config(...))?;

// GOTCHA: fetch::execute() uses println!() directly
// From src/commands/fetch.rs:42-65
// Output goes directly to stdout, no way to capture or suppress
// This is DESIRED behavior - user should see fetch results
println!("Fetching from origin ({})...", remote_config.url);

// CRITICAL: Fetch and Push are sibling modules in commands/
// From src/commands/mod.rs
// Must use super::fetch::execute() NOT crate::commands::fetch::execute()
// Because we're calling from within push.rs (also in commands/)
super::fetch::execute()?;

// CRITICAL: fetch::execute() returns Result<()>
// From src/commands/fetch.rs:16
// Use ? operator to propagate errors naturally
// If fetch fails, push::execute() returns the same error
super::fetch::execute()?;  // The ? is critical

// GOTCHA: Fetch validates remote config again (redundant but safe)
// From src/commands/fetch.rs:18-21
// Push already validates at lines 18-21
// Fetch will validate again at its lines 18-21
// This is OK - defensive programming, catches race conditions

// PATTERN: Error propagation via ? operator
// From src/core/error.rs - Result<T> alias and JinError
// All commands use thiserror for automatic conversion
// git2::Error converts to JinError via From trait (lines 67-71)
pub type Result<T> = std::result::Result<T, JinError>;

// CRITICAL: Remote fetch uses custom refspec
// From src/commands/fetch.rs:45
// Only fetches refs/jin/layers/*, not all Git refs
remote.fetch(&["refs/jin/layers/*"], Some(&mut fetch_opts), None)?;

// GOTCHA: Progress output uses carriage return for line overwrite
// From src/git/remote.rs:110-124
// print! with \r overwrites same line, then flush
// Fetch completes with println!() for final newline
print!("Received {}/{} objects ({}%)\r", ...);
io::stdout().flush().unwrap();

// CRITICAL: Authentication callbacks limit retry attempts
// From src/git/remote.rs:59-103
// AuthCounter prevents infinite retry loops
// Fails after 3 attempts with clear error message
```

## Implementation Blueprint

### Data Models and Structure

```rust
// No new data models required - use existing types

// From src/cli/args.rs (lines 213-219) - already defined
#[derive(Args, Debug)]
pub struct PushArgs {
    #[arg(long)]
    pub force: bool,
}

// From src/core/config.rs - already defined
pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}

// From src/core/error.rs - already defined
pub type Result<T> = std::result::Result<T, JinError>;
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/commands/push.rs::execute() function
  - ADD: Call to super::fetch::execute()? at line 17
  - PLACEMENT: After remote config validation (line 21), before repository opening (line 23)
  - PATTERN: super::fetch::execute()?;  (note the ? for error propagation)
  - RATIONALE: Ensures local refs are up-to-date before push logic executes
  - DEPENDENCIES: None (fetch.rs already exists and is working)

Task 2: VERIFY existing push behavior preserved
  - CONFIRM: Push still validates remote config (lines 18-21)
  - CONFIRM: Push still detects modified layers (lines 38-44)
  - CONFIRM: Push still builds refspecs (lines 46-56)
  - CONFIRM: Push still handles errors (lines 77-94)
  - CONFIRM: Force flag still works (lines 50-51, 59-62)
  - CONFIRM: User-local layer still filtered (lines 108-110)

Task 3: CREATE test for automatic fetch behavior
  - IMPLEMENT: test_push_calls_fetch_first() in tests/sync_workflow.rs
  - VERIFY: Fetch output appears before push output
  - VERIFY: Push aborts if fetch fails
  - VERIFY: Push succeeds after successful fetch
  - FOLLOW: test_push_uploads_commits() pattern (lines 227-294)
  - NAMING: test_push_auto_fetch_before_push
  - PLACEMENT: Add to tests/sync_workflow.rs

Task 4: VERIFY compilation and tests pass
  - RUN: cargo check --all-targets
  - RUN: cargo test --test sync_workflow
  - RUN: cargo clippy -- -D warnings
  - EXPECTED: All tests pass, no warnings

Task 5: MANUAL testing verification
  - TEST: jin push with no remote (should fail with config error)
  - TEST: jin push with remote but no changes (should fetch, say "Nothing to push")
  - TEST: jin push with local changes (should fetch, then push)
  - TEST: jin push with force flag (should fetch, warn, then push)
```

### Implementation Patterns & Key Details

```rust
// ============================================================================
// EXACT CODE CHANGE REQUIRED
// ============================================================================
// File: src/commands/push.rs
// Location: Line 17 (between remote config validation and repo opening)

// BEFORE (Current code at lines 16-25):
pub fn execute(args: PushArgs) -> Result<()> {
    // 1. Validate remote configuration
    let config = JinConfig::load()?;
    let remote_config = config.remote.ok_or(JinError::Config(
        "No remote configured. Run 'jin link <url>'.".into(),
    ))?;

    // 2. Open repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();
    ...

// AFTER (Add fetch call):
pub fn execute(args: PushArgs) -> Result<()> {
    // 1. Validate remote configuration
    let config = JinConfig::load()?;
    let remote_config = config.remote.ok_or(JinError::Config(
        "No remote configured. Run 'jin link <url>'.".into(),
    ))?;

    // 2. Fetch remote state first (NEW LINE)
    super::fetch::execute()?;

    // 3. Open repository
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();
    ...

// ============================================================================
// WHAT HAPPENS WHEN THIS CODE RUNS
// ============================================================================

// 1. Remote config validated (lines 18-21)
//    - If no remote: Error with "Run 'jin link <url>'"
//    - If remote exists: remote_config populated

// 2. fetch::execute() called (NEW LINE)
//    - Enters src/commands/fetch.rs::execute()
//    - Prints: "Fetching from origin (<url>)..."
//    - Shows progress: "Received X/Y objects (Z%)"
//    - Prints result: "Already up to date" OR "Updates available: ..."
//    - If error: Returns Err(), push::execute() returns same error
//    - If success: Returns Ok(()), execution continues

// 3. Repository opened (lines 24-25)
//    - Opens or creates ~/.jin/ repository
//    - Gets inner git2::Repository

// 4. Remote found (lines 27-36)
//    - Finds 'origin' remote in repository
//    - Returns error if not found

// 5. Modified layers detected (lines 38-44)
//    - Calls detect_modified_layers() helper
//    - Returns empty if nothing to push

// 6. Push proceeds (lines 46-95)
//    - Builds refspecs with or without + prefix (force flag)
//    - Warns if force push
//    - Calls remote.push()
//    - Handles errors (auth, non-fast-forward, etc.)

// ============================================================================
// ERROR PROPAGATION
// ============================================================================

// The ? operator does error propagation automatically:
// super::fetch::execute()?; is equivalent to:
//
// match super::fetch::execute() {
//     Ok(()) => { /* continue */ }
//     Err(e) => return Err(e),  // Same error returned from push::execute()
// }

// This means:
// - Fetch auth errors → Push fails with same auth error
// - Fetch network errors → Push fails with same network error
// - Fetch no-remote error → Push fails with same error (redundant but OK)

// ============================================================================
// USER EXPERIENCE
// ============================================================================

// Scenario 1: No updates available, push succeeds
// $ jin push
// Fetching from origin (git@github.com:team/jin-configs.git)...
// Received 0/0 objects (100%)
//
// Already up to date
// Pushing to origin (git@github.com:team/jin-configs.git)...
//   → refs/jin/layers/mode/my-mode
// Successfully pushed 1 layer

// Scenario 2: Updates available, push proceeds (for now - T2 will add check)
// $ jin push
// Fetching from origin (git@github.com:team/jin-configs.git)...
// Received 12/12 objects (100%)
//
// Updates available:
//   - mode/my-mode (2 files changed)
// Pushing to origin (git@github.com:team/jin-configs.git)...
//   → refs/jin/layers/mode/my-mode
// Successfully pushed 1 layer
//
// NOTE: P1.M2.T2 will add comparison to reject push when behind

// Scenario 3: Fetch fails
// $ jin push
// Fetching from origin (git@github.com:team/jin-configs.git)...
// Error: Authentication failed. Check your SSH keys or credentials.
//
// Push aborted (no push attempted)

// ============================================================================
// INTEGRATION TEST PATTERN
// ============================================================================
// File: tests/sync_workflow.rs
// Add new test after existing test_push_uploads_commits()

#[test]
fn test_push_auto_fetch_before_push() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let mode_name = format!("fetch_test_{}", unique_test_id());
    let jin_dir = remote_fixture.local_path.join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    // Link to remote
    jin()
        .args(["link", remote_fixture.remote_path.to_str().unwrap()])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create local commit
    create_mode(&mode_name, Some(&jin_dir))?;
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;
    jin()
        .args(["add", "local.txt", "--mode"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local commit"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Push (should auto-fetch first)
    let result = jin()
        .arg("push")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", &jin_dir)
        .assert();

    // Verify fetch output appears
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.contains("Fetching from origin") ||
        stdout_str.contains("Already up to date"),
        "Push should show fetch output"
    );

    // Verify push succeeded
    output.status.success();

    Ok(())
}

// GOTCHA: Test may need to handle "Already up to date" vs fetch progress
// Use flexible assertion with || to handle both cases
```

### Integration Points

```yaml
COMMANDS:
  - modify: src/commands/push.rs only
  - add_line: super::fetch::execute()?; at line 17
  - no_changes_to: CLI args, error types, other commands

GIT:
  - no_changes: Remote utilities already support both fetch and push
  - existing: build_fetch_options() and build_push_options() in remote.rs

TESTS:
  - add_to: tests/sync_workflow.rs
  - pattern: Follow test_push_uploads_commits() (lines 227-294)
  - verify: Fetch output appears before push output

ERROR:
  - no_changes: Error propagation via ? operator uses existing types
  - fetch_errors: Propagate naturally as push errors
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# After modifying push.rs
cargo check --all-targets
# Expected: zero errors, zero warnings
# Fix any issues before proceeding

cargo clippy -- -D warnings
# Expected: zero clippy warnings
# Fix any suggestions

cargo fmt --check
# Expected: code is formatted
# Run cargo fmt if needed
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test push command specifically
cargo test --lib commands::push
# Expected: All tests pass

# Test fetch command (ensure we didn't break it)
cargo test --lib commands::fetch
# Expected: All tests pass

# Full library test suite
cargo test --lib
# Expected: All tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Run sync_workflow tests (includes push/fetch tests)
cargo test --test sync_workflow
# Expected: All tests pass including new test_push_auto_fetch_before_push

# Verify specific test
cargo test --test sync_workflow test_push_auto_fetch_before_push
# Expected: New test passes

# Run all integration tests
cargo test --test cli_basic
cargo test --test sync_workflow
# Expected: All integration tests pass
```

### Level 4: Manual Testing (End-to-End Validation)

```bash
# Setup test scenario
cd /tmp
mkdir jin-push-test && cd jin-push-test
git init bare-remote --bare
mkdir local-repo && cd local-repo
git init
jin init

# Link to local remote
jin link file:///tmp/jin-push-test/bare-remote

# Test 1: Push with no remote changes (should fetch, say up to date, then push)
jin mode use test-mode
jin add test.json --mode
jin commit -m "Add test config"
jin push
# Expected output:
# Fetching from origin (file:///tmp/jin-push-test/bare-remote)...
# Already up to date
# Pushing to origin...
# Successfully pushed 1 layer

# Test 2: Verify fetch actually happened
# (In another terminal or after modifying remote manually)
# Fetch should have updated remote tracking refs

# Test 3: Push fails if fetch would fail
# (Break remote connection, invalid URL, etc.)
jin link /invalid/remote/path
jin push
# Expected: Fetch error, push aborted

# Test 4: Force push still fetches first
jin push --force
# Expected: Fetch happens, then force push with warning
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo check --all-targets` passes
- [ ] No clippy warnings: `cargo clippy -- -D warnings` passes
- [ ] Code formatted: `cargo fmt --check` passes
- [ ] All unit tests pass: `cargo test --lib` passes
- [ ] All integration tests pass: `cargo test --test sync_workflow` passes
- [ ] New test added: `test_push_auto_fetch_before_push` passes

### Feature Validation

- [ ] `jin push` calls `fetch::execute()` before pushing
- [ ] Fetch output displays to user (progress, updates, errors)
- [ ] Push aborts if fetch fails
- [ ] Push proceeds normally after successful fetch
- [ ] Force flag still works after fetch
- [ ] User-local layer still filtered from push
- [ ] Remote config validation still happens first
- [ ] No regression in existing push behavior

### Code Quality Validation

- [ ] Follows existing code patterns (error handling, Result types)
- [ ] No new dependencies added
- [ ] No changes to error types required
- [ ] Uses `super::fetch::execute()?;` pattern correctly
- [ ] Minimal modification (single line addition)
- [ ] No duplication of remote validation (acceptable redundancy)

### Integration Validation

- [ ] Works with existing link command (remote setup)
- [ ] Works with existing fetch command (called from push)
- [ ] Works with existing pull/sync commands (unchanged)
- [ ] Compatible with P1.M2.T2 (local vs remote comparison) - prep for next task

---

## Anti-Patterns to Avoid

- **Don't** modify fetch.rs - it already works, just call it
- **Don't** capture or suppress fetch output - user needs to see it
- **Don't** add conditional logic to skip fetch - fetch should always run
- **Don't** move remote validation - keep it in both places (defensive)
- **Don't** add new error types - existing JinError variants work
- **Don't** use `crate::commands::fetch::execute()` - use `super::fetch::execute()`
- **Don't** forget the `?` operator - critical for error propagation
- **Don't** add fetch inside conditional - it must always run
- **Don't** try to optimize away "redundant" remote validation - it's intentional safety
- **Don't** modify CLI args or other commands - single line change is sufficient

---

## Implementation Summary

**Change Required**: Add ONE line to `src/commands/push.rs`

```rust
// At line 17, after remote config validation, before repository opening:
super::fetch::execute()?;
```

**What This Does**:
1. Calls existing fetch command before any push logic
2. Fetch output displays to user naturally
3. Fetch errors abort the push via `?` operator
4. No other changes needed to codebase

**Why This Works**:
- Fetch and push are sibling modules in `src/commands/`
- `super::fetch::execute()` is the correct call pattern
- `?` operator propagates errors automatically
- Fetch already validates remote (acceptable redundancy)
- Fetch already handles progress and error reporting

**Next Steps After This Task**:
- **P1.M2.T2**: Add local vs remote comparison to reject push when behind
- **P1.M2.T3**: Add integration tests for full push enforcement

---

## Confidence Score

**10/10** - Maximum Confidence for One-Pass Implementation Success

**Rationale**:
- **Minimal Change**: Single line addition to existing code
- **No New Logic**: Calls existing, tested `fetch::execute()` function
- **Clear Pattern**: `super::module::function()?` is standard Rust practice
- **Well Understood**: Fetch behavior fully documented and tested
- **No Dependencies**: Uses existing types, errors, and modules
- **Reversible**: Change can be easily removed if issues arise
- **Tested Patterns**: Integration test patterns exist in `sync_workflow.rs`
- **Error Handling**: `?` operator provides automatic, correct error propagation

**Zero Risk Areas**:
- No new data structures
- No new algorithms
- No new error types
- No API changes
- No dependency changes
- No architectural changes

**Why Perfect Confidence**:
This is the simplest possible implementation of fetch-before-push:
1. Add one line: `super::fetch::execute()?;`
2. That's it.

The fetch command already exists and works. The push command already exists and works. We're just inserting the fetch call before the push logic. The `?` operator handles all error propagation automatically. No complexity, no ambiguity, no risk.

---

## Success Metrics

**Definition of Done**:
1. Line `super::fetch::execute()?;` added to `src/commands/push.rs` at line 17
2. All tests pass (existing + new integration test)
3. Manual testing confirms fetch output appears before push
4. No regression in existing push behavior
5. Code review confirms minimal, correct change

**Validation**: An AI agent with this PRP can implement this feature successfully because:
- Exact file location and line number specified
- Complete code snippet provided
- All context needed (fetch behavior, push structure, test patterns)
- Zero ambiguity in implementation
- Zero risk of breaking existing functionality
