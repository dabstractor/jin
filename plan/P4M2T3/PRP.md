# Product Requirement Prompt (PRP): Commit Command (P4.M2.T3)

## Goal

**Feature Goal**: Implement the `jin commit` command that atomically commits staged files across all affected layers using the existing CommitPipeline infrastructure.

**Deliverable**: A fully functional `jin commit -m "<message>"` command that:
1. Validates staging is not empty
2. Loads and uses the existing CommitPipeline
3. Commits atomically across multiple layers
4. Displays commit results with layer-specific commit hashes
5. Supports `--dry-run` flag for previewing commits

**Success Definition**:
- `jin commit -m "message"` commits all staged files atomically
- Staging is cleared only after successful commit
- Proper error messages for edge cases (empty staging, not initialized, etc.)
- Dry-run mode previews without making changes
- Integration tests verify the complete add → commit workflow

## User Persona

**Target User**: Developers using Jin to manage layered configuration files across different contexts (modes, scopes, projects).

**Use Case**: Developer has staged configuration files using `jin add` and wants to atomically commit those changes to the appropriate layers with a descriptive commit message.

**User Journey**:
1. User runs `jin add config.json --mode` to stage a file to the mode layer
2. User reviews staged changes with `jin status`
3. User runs `jin commit -m "Add Claude mode configuration"`
4. System commits the file atomically to the mode layer
5. Staging is cleared and user receives confirmation with commit hash

**Pain Points Addressed**:
- Without commit, staged files remain in staging indefinitely
- Atomic commit ensures consistency across multiple layers
- Clear commit messages enable tracking configuration changes over time

## Why

- **Core Workflow Completion**: The commit command is essential to complete the add → commit → apply workflow
- **Data Integrity**: Atomic commits ensure all layer updates succeed or none do
- **Version History**: Commits create a traceable history of configuration changes
- **Staging Cleanup**: Commit clears staging, preparing for the next batch of changes
- **Integration**: Completes the CLI command set for P4.M2 (Core Commands)

## What

Implement `jin commit` command that:

1. **Accepts a commit message** via `-m` or `--message` flag (required)
2. **Supports dry-run mode** via `--dry-run` flag to preview changes
3. **Validates preconditions**:
   - Jin must be initialized in the project
   - Staging must not be empty
4. **Executes commit** using the existing CommitPipeline
5. **Displays results** showing:
   - Number of files committed
   - Layers affected
   - Commit hash for each layer
6. **Handles errors** gracefully with clear messages

### Success Criteria

- [ ] `jin commit -m "message"` commits staged files successfully
- [ ] `jin commit --dry-run -m "message"` previews without committing
- [ ] Empty staging produces clear error message
- [ ] Non-initialized project produces clear error message
- [ ] Multi-layer commits are atomic
- [ ] Staging is cleared after successful commit
- [ ] Commit hashes are displayed for each layer
- [ ] Integration tests pass for add → commit workflow

## All Needed Context

### Context Completeness Check

This PRP assumes the reader has **no prior knowledge** of this codebase. All necessary file paths, patterns, and gotchas are explicitly specified.

### Documentation & References

```yaml
# CRITICAL - Read these files first

# CLI Framework and Command Pattern
- file: src/cli/mod.rs
  why: Shows Commands enum and how subcommands are registered
  pattern: Commands enum with Commit(CommitArgs) variant
  gotcha: Already defined - just needs implementation

- file: src/cli/args.rs
  why: Contains CommitArgs struct (already defined)
  pattern: Args struct with derive(Args) - message and dry_run fields
  gotcha: CommitArgs is already defined with required fields

- file: src/commands/mod.rs
  why: Central command dispatcher - already has commit_cmd::execute wired
  pattern: pub mod commit_cmd; and Commands::Commit(args) => commit_cmd::execute(args)
  gotcha: Module is already exported and wired in match statement

# CommitPipeline - The Core Implementation
- file: src/commit/pipeline.rs
  why: The actual commit logic that the CLI command must call
  pattern: CommitPipeline::new(staging).execute(&config)
  critical: |
    - CommitPipeline takes ownership of StagingIndex
    - execute() returns Result<CommitResult> or JinError
    - CommitConfig is created via CommitConfig::new(message).dry_run(bool)
    - Empty staging returns JinError::Other("Nothing to commit")
    - dry_run mode previews without committing

- file: src/commit/mod.rs
  why: Shows commit module structure and exports
  pattern: pub use pipeline::CommitPipeline;
  gotcha: Need to add `use crate::commit::CommitPipeline;` to commit_cmd.rs

# Existing Command Implementations - Patterns to Follow
- file: src/commands/add.rs
  why: Best reference for command structure and error handling
  pattern: |
    1. Validate inputs (files not empty, Jin initialized)
    2. Load ProjectContext
    3. Load/create StagingIndex
    4. Perform operations
    5. Save staging if needed
    6. Print summary
  critical: |
    - Use ProjectContext::load() for initialization check
    - StagingIndex::load() returns Err if .jin doesn't exist
    - Print user-friendly output with println!
    - Return Result<()> for error handling

- file: src/commands/status.rs
  why: Simple read-only command pattern
  pattern: Load context, load staging, display information

- file: src/commands/init.rs
  why: Simple command with no arguments
  pattern: Direct operations, minimal validation

# Core Types and Error Handling
- file: src/core/error.rs
  why: All JinError variants for error handling
  pattern: |
    - JinError::NotInitialized - Jin not initialized in project
    - JinError::Other(String) - Generic errors with message
  gotcha: Use JinError::NotInitialized for init check, not custom string

- file: src/core/mod.rs
  why: Re-exports Result type alias
  pattern: pub type Result<T> = std::result::Result<T, JinError>;

- file: src/staging/mod.rs
  why: StagingIndex public interface
  pattern: StagingIndex::load() - loads from .jin/staging/index.json

# Testing Patterns
- file: tests/core_workflow.rs
  why: Integration test patterns for CLI commands
  pattern: |
    - Use tempfile::TempDir for isolation
    - Set JIN_DIR environment variable
    - Use assert_cmd for CLI testing
    - Test both success and failure cases

- file: tests/common/fixtures.rs
  why: Test fixture patterns (setup_test_repo, jin_init, etc.)
  pattern: Helper functions for common test operations
```

### Current Codebase Tree

```bash
src/
├── main.rs                    # Entry point - delegates to lib::run()
├── lib.rs                     # Library entry - routes to commands::execute()
├── cli/
│   ├── mod.rs                 # Commands enum (Commit already defined)
│   └── args.rs                # CommitArgs struct (already defined)
├── commands/
│   ├── mod.rs                 # Command dispatcher (commit_cmd wired)
│   ├── commit_cmd.rs          # TARGET FILE - only has TODO stub
│   ├── add.rs                 # Reference for command pattern
│   ├── init.rs                # Reference for simple command
│   └── status.rs              # Reference for read-only command
├── commit/
│   ├── mod.rs                 # Exports CommitPipeline
│   └── pipeline.rs            # CommitPipeline implementation (READY TO USE)
├── core/
│   ├── mod.rs                 # Exports Result, JinError, Layer, etc.
│   ├── error.rs               # JinError enum variants
│   ├── config.rs              # ProjectContext type
│   └── layer.rs               # Layer enum
├── staging/
│   ├── mod.rs                 # Exports StagingIndex
│   └── index.rs               # StagingIndex implementation
└── git/
    └── transaction.rs         # LayerTransaction (used by pipeline)

tests/
├── core_workflow.rs           # Integration tests - ADD COMMIT TEST HERE
├── cli_basic.rs               # Basic CLI command tests
└── common/
    ├── fixtures.rs            # Test fixtures
    └── mod.rs                 # Common test utilities
```

### Desired Codebase Tree (Changes Only)

```bash
# Only one file needs modification:
src/
└── commands/
    └── commit_cmd.rs          # IMPLEMENT - wire CommitPipeline to CLI

# Optional: Add integration test
tests/
└── core_workflow.rs           # ADD test_commit_workflow() test
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: The CommitPipeline is already fully implemented
// DO NOT re-implement commit logic - just wire the existing pipeline

// CRITICAL: StagingIndex is moved into CommitPipeline
let staging = StagingIndex::load()?;  // Load staging
let mut pipeline = CommitPipeline::new(staging);  // Staging is MOVED here
// Cannot use staging after this point - pipeline owns it

// CRITICAL: Check Jin initialization BEFORE loading staging
// ProjectContext::load() returns Err(JinError::NotInitialized) if not initialized
// Use this pattern for initialization check:
let _context = ProjectContext::load()?;  // Returns NotInitialized error

// CRITICAL: Empty staging handling
// CommitPipeline::execute() returns Err(JinError::Other("Nothing to commit"))
// You can catch this and provide a user-friendly message:
match pipeline.execute(&config) {
    Err(JinError::Other(ref msg)) if msg == "Nothing to commit" => {
        return Err(JinError::Other("No staged files to commit".to_string()));
    }
    // ...
}

// CRITICAL: CommitConfig builder pattern
let config = CommitConfig::new(&args.message)
    .dry_run(args.dry_run);
// Note: message is String, need to convert to &str or pass owned String

// CRITICAL: Display commit results
// CommitResult contains:
// - committed_layers: Vec<Layer>
// - file_count: usize
// - commit_hashes: Vec<(Layer, String)>
for (layer, hash) in result.commit_hashes {
    println!("  {}: {}", layer, hash);
}

// CRITICAL: Layer display formatting
// Layer enum implements Display, but format is like "ModeBase"
// Consider using helper from add.rs for user-friendly names

// CRITICAL: Error handling must use JinError variants
// Never return generic errors - always wrap in JinError:
// - JinError::NotInitialized for "Jin not initialized"
// - JinError::Other(message) for other errors
// - Propagate Git errors with ? operator

// CRITICAL: Dry-run mode output
// The pipeline already prints dry-run output in execute_dry_run()
// Don't duplicate this output in the CLI command

// CRITICAL: Test isolation
// Always use JIN_DIR environment variable for test isolation
// Use tempfile::TempDir for temporary test directories
// Set JIN_DIR before calling any Jin functions
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - all types already exist:
- `CommitArgs` from `src/cli/args.rs` (already defined)
- `CommitConfig` from `src/commit/pipeline.rs` (already defined)
- `CommitPipeline` from `src/commit/pipeline.rs` (already defined)
- `CommitResult` from `src/commit/pipeline.rs` (already defined)
- `StagingIndex` from `src/staging/index.rs` (already defined)

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: MODIFY src/commands/commit_cmd.rs
  - IMPLEMENT: execute() function with full commit logic
  - FOLLOW pattern: src/commands/add.rs (validation, context loading, error handling)
  - IMPORTS: |
      use crate::cli::CommitArgs;
      use crate::commit::{CommitConfig, CommitPipeline};
      use crate::core::{JinError, Result};
      use crate::staging::StagingIndex;
  - LOGIC: |
      1. Load ProjectContext to check initialization (returns NotInitialized error)
      2. Load StagingIndex
      3. Create CommitConfig from args.message and args.dry_run
      4. Create CommitPipeline with staging (staging is moved)
      5. Execute commit via pipeline.execute(&config)
      6. Display results (file count, layers, commit hashes)
  - ERROR HANDLING: |
      - NotInitialized: Clear error message
      - "Nothing to commit": User-friendly message
      - Other errors: Propagate with context
  - OUTPUT FORMAT: |
      Committed {file_count} file(s) to {layer_count} layer(s):
        {layer}: {commit_hash}
        ...

Task 2: ADD Integration Test to tests/core_workflow.rs
  - IMPLEMENT: test_commit_workflow() function
  - FOLLOW pattern: tests/core_workflow.rs (existing integration tests)
  - SETUP: |
      1. Create TempDir for isolation
      2. Set JIN_DIR environment variable
      3. Run jin init
      4. Create test file
      5. Run jin add to stage file
  - TEST CASES: |
      - Happy path: add → commit → verify success
      - Empty staging: commit without add → verify error
      - Dry-run: commit --dry-run → verify preview
      - Not initialized: commit without init → verify error
  - ASSERTIONS: |
      - Use assert_cmd for CLI testing
      - Check stdout for expected output
      - Check stderr for error messages
      - Verify staging cleared after commit

Task 3: RUN Validation Loop
  - EXECUTE: cargo test --package jin
  - VERIFY: All tests pass including new commit test
  - EXECUTE: cargo build --release
  - VERIFY: No compilation errors or warnings
  - MANUAL: Test jin commit manually with staged files
  - VERIFY: Expected output and behavior
```

### Implementation Patterns & Key Details

```rust
// Full implementation template for src/commands/commit_cmd.rs

//! Implementation of `jin commit`
//!
//! Commits staged files atomically across all affected layers.
//! Uses the CommitPipeline to handle multi-layer atomic commits.

use crate::cli::CommitArgs;
use crate::commit::{CommitConfig, CommitPipeline};
use crate::core::{JinError, Result};
use crate::staging::StagingIndex;

/// Execute the commit command
///
/// Commits staged files atomically across all affected layers.
///
/// # Arguments
///
/// * `args` - Command line arguments including message and dry_run flag
///
/// # Errors
///
/// Returns an error if:
/// - Jin is not initialized in the current project
/// - No files are staged (empty staging index)
/// - Commit operation fails (Git errors, transaction errors, etc.)
pub fn execute(args: CommitArgs) -> Result<()> {
    // PATTERN: Check initialization first (follow add.rs pattern)
    // ProjectContext::load() returns Err(JinError::NotInitialized) if not initialized
    let _context = ProjectContext::load()?;

    // PATTERN: Load staging index
    // This will fail if .jin doesn't exist (redundant with context check but safe)
    let staging = StagingIndex::load()?;

    // PATTERN: Build commit configuration
    // CommitConfig builder pattern - pass message as &str
    let config = CommitConfig::new(&args.message).dry_run(args.dry_run);

    // PATTERN: Create pipeline (staging is moved into pipeline)
    // CRITICAL: Cannot use staging after this line
    let mut pipeline = CommitPipeline::new(staging);

    // PATTERN: Execute commit with error handling
    // Handle "Nothing to commit" error with user-friendly message
    match pipeline.execute(&config) {
        Err(JinError::Other(ref msg)) if msg == "Nothing to commit" => {
            // GOTCHA: Empty staging check happens in pipeline
            return Err(JinError::Other("No staged files to commit. Use 'jin add' to stage files first.".to_string()));
        }
        Err(e) => return Err(e),
        Ok(result) => {
            // PATTERN: Display results in user-friendly format
            display_commit_result(&result);
        }
    }

    Ok(())
}

/// Display commit results to the user
fn display_commit_result(result: &crate::commit::CommitResult) {
    // PATTERN: Format output similar to Git commits
    if result.commit_hashes.is_empty() {
        // Dry-run mode - no actual hashes
        println!(
            "Would commit {} file(s) to {} layer(s)",
            result.file_count,
            result.committed_layers.len()
        );
    } else {
        // Actual commit - show hashes
        println!(
            "Committed {} file(s) to {} layer(s):",
            result.file_count,
            result.committed_layers.len()
        );

        // PATTERN: Show layer and hash for each committed layer
        for (layer, hash) in &result.commit_hashes {
            println!("  {}: {}", layer, hash);
        }
    }
}

// CRITICAL: The above is all that's needed for the command!
// CommitPipeline handles all the complex logic (tree building, commits, atomic updates)

// ========================================
// Integration Test Template (tests/core_workflow.rs)
// ========================================

#[test]
fn test_commit_workflow() {
    // PATTERN: Test isolation with TempDir
    let temp = tempfile::TempDir::unwrap();
    let project_path = temp.path();

    // PATTERN: Set JIN_DIR for isolation
    let jin_dir = temp.path().join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    // Step 1: Initialize Jin
    jin()
        .arg("init")
        .current_dir(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Jin"));

    // Step 2: Create a test file
    let test_file = project_path.join("config.json");
    std::fs::write(&test_file, r#"{"key": "value"}"#).unwrap();

    // Step 3: Stage the file
    jin()
        .current_dir(project_path)
        .args(["add", "config.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged"));

    // Step 4: Commit with message
    jin()
        .current_dir(project_path)
        .args(["commit", "-m", "Add config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Committed"));

    // Step 5: Verify staging is cleared
    jin()
        .current_dir(project_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged files: 0"));
}

#[test]
fn test_commit_empty_staging() {
    let temp = tempfile::TempDir::unwrap();

    // Initialize but don't stage anything
    jin()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success();

    // Try to commit without staging
    jin()
        .current_dir(temp.path())
        .args(["commit", "-m", "Test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No staged files"));
}

#[test]
fn test_commit_dry_run() {
    let temp = tempfile::TempDir::unwrap();
    let project_path = temp.path();

    // Setup
    std::env::set_var("JIN_DIR", temp.path().join(".jin_global"));
    jin().arg("init").current_dir(project_path).assert().success();

    let test_file = project_path.join("test.json");
    std::fs::write(&test_file, "{}").unwrap();

    jin()
        .current_dir(project_path)
        .args(["add", "test.json"])
        .assert()
        .success();

    // Dry-run commit
    jin()
        .current_dir(project_path)
        .args(["commit", "--dry-run", "-m", "Test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would commit"));

    // Verify staging still has files (not cleared in dry-run)
    jin()
        .current_dir(project_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged files: 1"));
}

#[test]
fn test_commit_not_initialized() {
    let temp = tempfile::TempDir::unwrap();

    // Try to commit without initializing
    jin()
        .current_dir(temp.path())
        .args(["commit", "-m", "Test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}
```

### Integration Points

```yaml
NO NEW INTEGRATIONS NEEDED:

# CLI Framework
# - CommitArgs already defined in src/cli/args.rs
# - Commands::Commit already registered in src/cli/mod.rs
# - Command dispatcher already wired in src/commands/mod.rs

# CommitPipeline
# - Fully implemented in src/commit/pipeline.rs
# - Exports CommitPipeline, CommitConfig, CommitResult
# - Just need to call from commit_cmd.rs

# Staging System
# - StagingIndex already implemented
# - .load() method ready to use
# - CommitPipeline clears staging on success

# Error Handling
# - All JinError variants defined
# - Use JinError::NotInitialized for init check
# - Use JinError::Other for custom messages

NO DATABASE, CONFIG, OR ROUTE CHANGES NEEDED
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after implementing commit_cmd.rs
cargo check --package jin                    # Check for compilation errors
cargo clippy --package jin -- -D warnings   # Lint checks

# Expected: Zero errors, zero warnings

# Format check
cargo fmt --all -- --check                  # Check formatting
cargo fmt --all                              # Auto-format if needed

# Expected: All code properly formatted
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run existing unit tests for related modules
cargo test --package jin commit::pipeline    # CommitPipeline tests
cargo test --package jin staging::           # Staging tests
cargo test --package jin commands::add       # Add command tests (for comparison)

# Run new unit tests if added to commit_cmd.rs
cargo test --package jin commands::commit_cmd

# Expected: All tests pass

# Run all unit tests
cargo test --lib

# Expected: All library tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Test the commit command manually
cd /tmp && mkdir test_commit && cd test_commit

# Initialize
cargo run -- init
# Expected: "Initialized Jin"

# Try commit without staging
cargo run -- commit -m "Test"
# Expected: Error "No staged files to commit"

# Create and stage a file
echo '{"test": true}' > config.json
cargo run -- add config.json
# Expected: "Staged 1 file(s) to project-base layer"

# Check status
cargo run -- status
# Expected: Shows 1 staged file

# Dry-run commit
cargo run -- commit --dry-run -m "Add config"
# Expected: "Would commit 1 file(s) to 1 layer(s):"

# Actual commit
cargo run -- commit -m "Add config"
# Expected: "Committed 1 file(s) to 1 layer(s):" with hash

# Verify staging cleared
cargo run -- status
# Expected: "Staged files: 0"

# Test multi-layer commit
echo '{"mode": true}' > mode_config.json
cargo run -- add mode_config.json --mode
cargo run -- add config.json
cargo run -- commit -m "Multi-layer commit"
# Expected: "Committed 2 file(s) to 2 layer(s):"

# Test error cases
cd /tmp
mkdir test_no_init && cd test_no_init
cargo run -- commit -m "Test"
# Expected: Error "Jin not initialized"

# Expected: All manual tests produce expected output
```

### Level 4: Integration Test Suite

```bash
# Run the integration test suite
cargo test --test core_workflow

# Expected: New test_commit_workflow tests pass

# Run all integration tests
cargo test --test cli_basic
cargo test --test core_workflow
cargo test --test error_scenarios

# Expected: All integration tests pass

# Run full test suite
cargo test

# Expected: All tests pass (unit + integration)
```

## Final Validation Checklist

### Technical Validation

- [ ] Level 1 validation passes (cargo check, clippy, fmt)
- [ ] Level 2 validation passes (unit tests)
- [ ] Level 3 validation passes (manual testing)
- [ ] Level 4 validation passes (integration tests)
- [ ] No compilation errors or warnings
- [ ] All test cases pass (happy path + edge cases)

### Feature Validation

- [ ] `jin commit -m "message"` commits staged files successfully
- [ ] `jin commit --dry-run -m "message"` previews without committing
- [ ] Empty staging produces clear error: "No staged files to commit"
- [ ] Non-initialized project produces clear error: "Jin not initialized"
- [ ] Multi-layer commits are atomic
- [ ] Staging is cleared after successful commit
- [ ] Commit hashes are displayed for each layer
- [ ] Dry-run mode does not clear staging

### Code Quality Validation

- [ ] Follows existing command pattern (add.rs reference)
- [ ] Uses existing CommitPipeline (no reimplementation)
- [ ] Proper error handling with JinError variants
- [ ] User-friendly output messages
- [ ] No code duplication
- [ ] Proper use of `?` operator for error propagation

### Documentation & Deployment

- [ ] Code is self-documenting with clear function names
- [ ] Public functions have doc comments
- [ ] Error messages are clear and actionable
- [ ] No new dependencies added
- [ ] Changes isolated to commit_cmd.rs only

## Anti-Patterns to Avoid

- **Don't re-implement commit logic** - The CommitPipeline is already fully implemented. Just wire it to the CLI.
- **Don't skip initialization check** - Always check Jin is initialized before loading staging.
- **Don't ignore empty staging** - Provide a clear error message when staging is empty.
- **Don't use staging after creating pipeline** - StagingIndex is moved into CommitPipeline.
- **Don't hardcode layer names** - Use Layer enum's Display implementation.
- **Don't create custom error types** - Use existing JinError variants.
- **Don't duplicate dry-run output** - Pipeline already handles dry-run output.
- **Don't forget to display results** - Show file count, layers, and commit hashes to user.
- **Don't use unwrap() in production code** - Use `?` operator for error propagation.
- **Don't skip testing** - Add integration tests for the commit workflow.

---

## Confidence Score

**9/10** - One-pass implementation success likelihood is very high.

**Reasoning**:
1. **Complete Infrastructure**: CommitPipeline is fully implemented and tested
2. **Clear Pattern**: add.rs provides an excellent reference for command structure
3. **Minimal Changes**: Only one file needs modification (commit_cmd.rs)
4. **Well-Defined Contract**: CommitPipeline has clear input/output types
5. **Comprehensive Testing**: Existing test patterns and fixtures available

**Risk Factors**:
- Must ensure proper error message handling for "Nothing to commit" case
- Need to verify staging is properly cleared after successful commit
- Dry-run mode should not clear staging

**Mitigation**:
- PRP provides exact implementation template
- Integration tests specified to catch any issues
- Manual testing steps included for verification
