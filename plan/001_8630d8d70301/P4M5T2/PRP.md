# PRP: P4.M5.T2 - Log Command

---

## Goal

**Feature Goal**: Wire the `jin log` command to show commit history for layers in the Jin configuration management system.

**Deliverable**: Fully functional `jin log` command that displays Git-like commit history for layers with support for filtering by layer and limiting output count.

**Success Definition**:
- User can run `jin log` to show history for all layers (default 10 entries)
- User can run `jin log --layer=<name>` to show history for specific layer
- User can run `jin log --count=N` to limit output entries
- Command shows commit hash, author, date, message, and file count
- Error handling for unknown layers and uninitialized projects
- Integration tests pass

---

## User Persona

**Target User**: Developer debugging configuration changes, tracking when specific settings were introduced, or auditing configuration history across layers.

**Use Case**: A developer notices an unexpected configuration value and needs to:
1. Find when the change was introduced
2. See who made the change
3. Understand the context through commit messages
4. Track history for specific layers (mode-base, project-base, etc.)

**User Journey**:
1. Developer runs `jin log` to see recent commits across all layers
2. Developer runs `jin log --layer=mode-base --count=20` for specific layer history
3. Developer identifies commit that introduced the change
4. Developer uses `jin diff` to see what changed in that commit

**Pain Points Addressed**:
- **No visibility into configuration history** - log provides Git-like history per layer
- **Can't track when settings changed** - shows timestamps and commit messages
- **No context for changes** - shows author and commit message for each change

---

## Why

**Business Value:**
- Provides audit trail for configuration changes
- Enables debugging of configuration drift
- Supports rollback decisions by showing change history
- Completes the inspection/debugging toolset for Jin

**Integration with Existing Features:**
- Uses JinRepo wrapper (P1.M2) for Git repository access
- Uses Layer enum (P1.M3) for layer reference paths
- Uses ProjectContext for active mode/scope/project context
- Follows command patterns established in P4.M2 (add, commit, status)

**Problems This Solves:**
- Users can't see history of configuration changes
- No way to identify when a setting was modified
- Can't track which layer introduced specific changes
- No audit trail for compliance/debugging

**PRD Requirements:**
- Section 18.6 specifies log as inspection command
- Section 26 Acceptance Criteria requires utility commands

---

## What

### User-Visible Behavior

#### jin log

Show commit history for layers.

**Usage:**
```bash
jin log                            # Show history for all layers (last 10)
jin log --layer=mode-base          # Show history for specific layer
jin log --layer=project-base --count=20  # Show last 20 commits
```

**Output:**
```
=== ModeBase ===

commit abc1234 (mode-base)
Author: Alice <alice@example.com>
Date:   2025-12-27 10:30:00

    Add Python IDE settings

    3 file(s) changed

commit def5678 (mode-base)
Author: Bob <bob@example.com>
Date:   2025-12-26 15:45:00

    Configure linter rules

    1 file(s) changed
```

**Behavior:**
- Uses git2::Revwalk to traverse commit history
- Defaults to showing last 10 commits per layer
- Groups commits by layer if no --layer specified
- Shows commit hash (short), author, date, message, file count
- Supports --count to limit output (default: 10)
- Filters layers based on active mode/scope context

**Error Conditions:**
- Unknown layer name → Error with list of valid layers
- Layer has no commits → Info message "No commits yet for layer: X"
- Jin not initialized → Error suggesting `jin init`

### Success Criteria

- [ ] `jin log` shows commit history for all applicable layers
- [ ] `jin log --layer=<name>` shows history for specific layer only
- [ ] `jin log --count=N` limits output to N commits
- [ ] Output shows commit hash, author, date, message, file count
- [ ] Error handling for unknown layers
- [ ] Error handling for uninitialized projects
- [ ] Integration tests pass

---

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement this command successfully?"_

**YES** - This PRP provides:
- Complete command specification with examples
- Existing implementation reference (already complete at src/commands/log.rs)
- Git2-rs Revwalk API documentation
- Error handling patterns from JinError enum
- Testing patterns from existing command tests
- Layer system context from src/core/layer.rs

### Documentation & References

```yaml
# MUST READ - Core Jin Patterns

- file: src/commands/log.rs:1-248
  why: COMPLETE IMPLEMENTATION - Full reference for log command
  pattern: |
    - Load ProjectContext for active mode/scope/project
    - Parse layer name from args.layer if provided
    - Open JinRepo with open_or_create()
    - Determine ref path using Layer::ref_path()
    - Create Revwalk and configure with push_ref() and set_sorting()
    - Iterate commits and format output
    - Count files changed in each commit
  gotcha: |
    - MUST check if reference exists before creating revwalk
    - MUST filter layers by requires_mode() and requires_scope()
    - Handle NotInitialized error for uninitialized projects

- file: src/cli/args.rs:102-112
  why: LogArgs struct definition for CLI argument parsing
  pattern: |
    #[derive(Args, Debug)]
    pub struct LogArgs {
        #[arg(long)]
        pub layer: Option<String>,
        #[arg(long, default_value = "10")]
        pub count: usize,
    }
  gotcha: |
    - layer is optional - None means show all layers
    - count has default value of 10

- file: src/cli/mod.rs:60
  why: Command registration in CLI enum
  pattern: |
    #[derive(Subcommand, Debug)]
    pub enum Commands {
        Log(LogArgs),
        // ...
    }
  gotcha: |
    - LogArgs is passed to log::execute()

- file: src/commands/mod.rs:43
  why: Command dispatch wiring
  pattern: |
    pub fn execute(cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Log(args) => log::execute(args),
            // ...
        }
    }
  gotcha: |
    - Pattern match extracts LogArgs and calls execute()

- file: src/core/layer.rs:1-284
  why: Complete Layer enum definition with precedence, ref_path, storage_path methods
  pattern: |
    - Layer::all_in_precedence_order() for iteration
    - layer.ref_path(mode, scope, project) for Git refs
    - layer.requires_mode(), requires_scope() for filtering
    - Display trait for user-facing layer names
  gotcha: |
    - Precedence flows 1-9, higher overrides lower
    - WorkspaceActive (9) is derived, never source of truth
    - Use Display trait for user-friendly layer names

- file: src/git/repo.rs:1-200
  why: JinRepo wrapper around git2::Repository
  pattern: |
    - JinRepo::open_or_create() for repository access
    - repo.inner() to access underlying git2::Repository
  gotcha: |
    - Jin repository is bare, stored at ~/.jin/
    - Use repo.inner() for git2 operations

- file: src/core/config.rs:1-150
  why: ProjectContext for active mode/scope/project
  pattern: |
    - ProjectContext::load() to load current context
    - Handle NotInitialized error gracefully
    - context.mode, context.scope, context.project for layer filtering
  gotcha: |
    - NotInitialized error should be returned, not handled silently
    - Use default context only for non-critical operations

# MUST READ - Git2-rs Documentation

- url: https://docs.rs/git2/latest/git2/struct.Revwalk.html
  why: Commit history traversal for log command
  critical: |
    - Repository::revwalk() creates walker
    - revwalk.push_ref() to start from specific ref
    - revwalk.set_sorting(Sort::TIME) for chronological order
    - revwalk is an iterator over Oid (commit IDs)
  section: "Methods - push, sorting, iteration"

- url: https://docs.rs/git2/latest/git2/struct.Commit.html
  why: Commit metadata extraction
  critical: |
    - commit.author() returns Signature with name() and email()
    - commit.message() returns commit message
    - commit.time() returns Time with seconds() for timestamp
    - commit.tree() returns the tree object
  section: "Methods - metadata access"

- url: https://docs.rs/git2/latest/git2/struct.Repository.html#method.find_reference
  why: Checking if layer refs exist
  critical: |
    - repo.find_reference(&ref_path) returns Result<Reference>
    - Use match to handle Err case (layer has no commits)
  section: "Reference lookup"

# MUST READ - Research Documentation

- docfile: plan/P4M5/research/log_research.md
  why: Comprehensive git log research for Jin implementation
  section: Full document
  critical: |
    - Traversal algorithms (BFS vs DFS with generation numbers)
    - git2 RevWalk API patterns
    - Commit metadata extraction techniques
    - Log formatting options
    - Filtering and limiting strategies

- docfile: plan/P4M5/research/log_quick_reference.md
  why: Quick reference for git2 RevWalk API
  section: Full document
  critical: |
    - Core RevWalk methods and sorting options
    - Commit metadata extraction key methods
    - Common format strings
    - Filtering patterns

# MUST READ - Existing Test Patterns

- file: tests/cli_basic.rs:1-340
  why: Integration test patterns using assert_cmd
  pattern: |
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_subcommand() {
        let temp = TempDir::new().unwrap();
        Command::cargo_bin("jin")
            .unwrap()
            .current_dir(temp.path())
            .arg("log")
            .assert()
            .success()
            .stdout(predicate::str::contains("commit"));
    }
  gotcha: |
    - Use TempDir for isolated test environments
    - Check stdout for expected output
    - Use .failure() for error cases

- file: src/commands/log.rs:185-247
  why: Unit tests for log command
  pattern: |
    #[cfg(test)]
    mod tests {
        #[test]
        fn test_parse_layer_name() { /* ... */ }
        #[test]
        fn test_execute_not_initialized() { /* ... */ }
        #[test]
        fn test_count_files_empty_commit() { /* ... */ }
    }
  gotcha: |
    - Test parse_layer_name for all valid layer names
    - Test error case for unknown layer
    - Test NotInitialized error handling
    - Use tempfile for isolated Git repositories

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs              # LogArgs struct (lines 102-112)
│   └── mod.rs               # Commands::Log(LogArgs) (line 60)
├── commands/
│   ├── log.rs               # COMPLETE IMPLEMENTATION (248 lines)
│   └── mod.rs               # Command dispatch (line 43)
├── core/
│   ├── layer.rs             # Layer enum with ref_path()
│   ├── config.rs            # ProjectContext
│   └── error.rs             # JinError enum
└── git/
    └── repo.rs              # JinRepo wrapper
```

### Desired Codebase Tree

**ALREADY ACHIEVED** - Implementation is complete at:
- `src/commands/log.rs` - Full implementation (248 lines)
- `src/cli/args.rs` - LogArgs struct (lines 102-112)
- `src/cli/mod.rs` - Commands::Log enum variant (line 60)
- `src/commands/mod.rs` - Command dispatch (line 43)

### Known Gotchas & Library Quirks

```rust
// CRITICAL: git2::Revwalk requires push_ref() before iteration
// Failure to push causes empty iterator
let mut revwalk = repo.revwalk()?;
revwalk.push_ref(&ref_path)?; // Required!
revwalk.set_sorting(git2::Sort::TIME)?;

// CRITICAL: Layer refs may not exist if no commits yet
// ALWAYS check find_reference() returns Ok before using
let reference = match repo.find_reference(&ref_path) {
    Ok(r) => r,
    Err(_) => {
        println!("No commits yet for layer: {}", layer);
        return Ok(());
    }
};

// GOTCHA: Layer::all_in_precedence_order() includes ALL 9 layers
// Filter by requires_mode(), requires_scope() for active layers only
if layer.requires_mode() && context.mode.is_none() {
    continue;
}
if layer.requires_scope() && context.scope.is_none() {
    continue;
}

// CRITICAL: Use repo.inner() to access git2::Repository
// JinRepo is a wrapper - need the inner git2 repo for operations
let git_repo = repo.inner();

// GOTCHA: Commit OID to_string() returns full hash (40 chars)
// Slice to get short hash (7 chars is Git standard)
let hash_short = &oid.to_string()[..7];

// GOTCHA: DateTime::from_timestamp can fail
// Provide fallback to UNIX_EPOCH if conversion fails
let timestamp = DateTime::from_timestamp(time.seconds(), 0)
    .unwrap_or_else(|| DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH));
```

---

## Implementation Blueprint

### Implementation Status

**COMPLETE** - The log command is fully implemented and wired to the CLI.

### Implementation Tasks (already completed)

```yaml
Task 1: IMPLEMENT src/commands/log.rs
  - IMPLEMENT: execute(args: LogArgs) function - COMPLETE
  - PARSE: args.layer to determine target layer(s) - COMPLETE
  - OPEN: JinRepo with open_or_create() - COMPLETE
  - DETERMINE: Ref path using Layer::ref_path() - COMPLETE
  - CREATE: Revwalk with repo.inner().revwalk() - COMPLETE
  - CONFIGURE: walk.push_ref() for target ref, walk.set_sorting(Sort::TIME) - COMPLETE
  - ITERATE: walk.next() up to args.count commits - COMPLETE
  - FETCH: Commit details with repo.inner().find_commit() - COMPLETE
  - FORMAT: Display commit hash, author, date, message, file count - COMPLETE
  - ERROR HANDLING: Unknown layer, layer has no commits, NotInitialized - COMPLETE
  - PLACEMENT: src/commands/log.rs (248 lines)

Task 2: IMPLEMENT src/cli/args.rs - LogArgs
  - IMPLEMENT: LogArgs struct with clap derive - COMPLETE
  - DEFINE: layer: Option<String> for layer filtering - COMPLETE
  - DEFINE: count: usize with default_value = "10" - COMPLETE
  - PLACEMENT: src/cli/args.rs:102-112

Task 3: WIRE Command in CLI
  - ADD: Commands::Log(LogArgs) enum variant - COMPLETE
  - ADD: Command dispatch in execute() - COMPLETE
  - PLACEMENT: src/cli/mod.rs:60, src/commands/mod.rs:43

Task 4: IMPLEMENT Helper Functions
  - IMPLEMENT: show_layer_history() for single layer - COMPLETE
  - IMPLEMENT: count_files_in_commit() for file stats - COMPLETE
  - IMPLEMENT: parse_layer_name() for name parsing - COMPLETE
  - PLACEMENT: src/commands/log.rs:73-183

Task 5: ADD Unit Tests
  - IMPLEMENT: test_parse_layer_name() - COMPLETE
  - IMPLEMENT: test_execute_not_initialized() - COMPLETE
  - IMPLEMENT: test_count_files_empty_commit() - COMPLETE
  - PLACEMENT: src/commands/log.rs:185-247
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Layer filtering based on active context
use crate::core::{Layer, ProjectContext};

let context = ProjectContext::load()?;
let all_layers = Layer::all_in_precedence_order();

for layer in &all_layers {
    // Skip layers that don't apply to current context
    if layer.requires_mode() && context.mode.is_none() {
        continue;
    }
    if layer.requires_scope() && context.scope.is_none() {
        continue;
    }
    // ... process layer
}

// Pattern 2: Commit history traversal with Revwalk
use git2::{Sort, Repository};

let git_repo = repo.inner();
let ref_path = layer.ref_path(mode, scope, project);

// Check if ref exists
let _reference = match git_repo.find_reference(&ref_path) {
    Ok(r) => r,
    Err(_) => {
        println!("No commits yet for layer: {}", layer);
        return Ok(());
    }
};

let mut revwalk = git_repo.revwalk()?;
revwalk.push_ref(&ref_path)?;
revwalk.set_sorting(Sort::TIME)?;

for (i, oid_result) in revwalk.enumerate() {
    if i >= count {
        break;
    }
    let oid = oid_result?;
    let commit = git_repo.find_commit(oid)?;

    // Format and display commit
    let hash_short = &oid.to_string()[..7];
    let author = commit.author();
    let author_name = author.name().unwrap_or("unknown");
    let author_email = author.email().unwrap_or("unknown");
    let message = commit.message().unwrap_or("(no message)");

    println!("commit {} ({})", hash_short, layer);
    println!("Author: {} <{}>", author_name, author_email);
    println!("Date:   {}", timestamp.format("%Y-%m-%d %H:%M:%S"));
    println!();
    println!("    {}", message.trim());
}

// Pattern 3: File count in commit
fn count_files_in_commit(repo: &Repository, commit: &Commit) -> Result<usize> {
    let tree = commit.tree()?;

    // If no parent, count all files in tree
    if commit.parent_count() == 0 {
        let mut count = 0;
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                count += 1;
            }
            git2::TreeWalkResult::Ok
        })?;
        return Ok(count);
    }

    // Otherwise, diff with parent
    let parent = commit.parent(0)?;
    let parent_tree = parent.tree()?;
    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;
    Ok(diff.deltas().count())
}

// Pattern 4: Layer name parsing
fn parse_layer_name(name: &str) -> Result<Layer> {
    match name {
        "global-base" => Ok(Layer::GlobalBase),
        "mode-base" => Ok(Layer::ModeBase),
        "mode-scope" => Ok(Layer::ModeScope),
        "mode-scope-project" => Ok(Layer::ModeScopeProject),
        "mode-project" => Ok(Layer::ModeProject),
        "scope-base" => Ok(Layer::ScopeBase),
        "project-base" => Ok(Layer::ProjectBase),
        "user-local" => Ok(Layer::UserLocal),
        "workspace-active" => Ok(Layer::WorkspaceActive),
        _ => Err(JinError::Other(format!(
            "Unknown layer: {}. Valid layers: global-base, mode-base, ...",
            name
        ))),
    }
}

// Pattern 5: Timestamp formatting
use chrono::{DateTime, Utc};

let time = commit.time();
let timestamp = DateTime::from_timestamp(time.seconds(), 0)
    .unwrap_or_else(|| DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH));
println!("Date:   {}", timestamp.format("%Y-%m-%d %H:%M:%S"));
```

### Integration Points

```yaml
DEPENDENCIES:
  - JinRepo (P1.M2) - Repository access via open_or_create()
  - Layer enum (P1.M3) - Layer iteration and ref_path generation
  - ProjectContext (P1.M4) - Active mode/scope/project context
  - git2::Revwalk - Commit history traversal
  - git2::Commit - Commit metadata extraction
  - chrono - DateTime formatting

CONFIG:
  - No new config needed
  - Uses existing .jin/context for active context

ROUTES:
  - Command registered in src/cli/mod.rs:60
  - Dispatched in src/commands/mod.rs:43
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run syntax check
cargo check

# Expected: Zero errors
# Output: Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs

# Run linting
cargo clippy -- -D warnings

# Expected: Zero warnings
# Output: Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs

# Format check
cargo fmt --check

# Expected: No formatting needed
# Output: All files properly formatted
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run log command unit tests
cargo test commands::log::tests

# Expected: All tests pass
# Output:
#   running 3 tests
#   test tests::test_parse_layer_name ... ok
#   test tests::test_execute_not_initialized ... ok
#   test tests::test_count_files_empty_commit ... ok
#   test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

# Run all unit tests
cargo test --lib

# Expected: All tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Run CLI integration tests
cargo test --test cli_basic test_log_subcommand

# Expected: Test passes with proper output validation

# Run all CLI tests
cargo test --test cli_basic

# Expected: All tests pass
```

### Level 4: Manual Validation & Edge Cases

```bash
# Build release binary
cargo build --release

# Initialize Jin project
mkdir /tmp/jin-test && cd /tmp/jin-test
../target/release/jin init

# Add and commit a file
echo "test" > config.json
../target/release/jin add config.json
../target/release/jin commit -m "Initial commit"

# Test: Show log for all layers
../target/release/jin log
# Expected: Shows commit with hash, author, date, message

# Test: Show log with --layer
../target/release/jin log --layer=project-base
# Expected: Shows only project-base layer commits

# Test: Show log with --count
../target/release/jin log --count=1
# Expected: Shows only 1 commit

# Test: Unknown layer
../target/release/jin log --layer=invalid
# Expected: Error "Unknown layer: invalid"

# Test: Not initialized
cd /tmp && mkdir /tmp/no-jin && cd /tmp/no-jin
../target/release/jin log
# Expected: Error "Jin not initialized"
```

---

## Final Validation Checklist

### Technical Validation

- [x] All 4 validation levels completed successfully
- [x] `cargo test` passes with zero failures
- [x] `cargo clippy` reports zero warnings
- [x] `cargo fmt --check` passes
- [x] Unit tests in src/commands/log.rs pass
- [x] Integration tests pass

### Feature Validation

- [x] `jin log` shows commit history for all layers
- [x] `jin log --layer=<name>` filters by specific layer
- [x] `jin log --count=N` limits output to N commits
- [x] Output shows commit hash (short), author, date, message
- [x] Output shows file count for each commit
- [x] Error handling for unknown layers
- [x] Error handling for uninitialized projects

### Code Quality Validation

- [x] Follows patterns from other commands (add, status, diff)
- [x] Uses JinError enum consistently
- [x] Proper error handling (no unwrap in production code)
- [x] Clear variable and function names
- [x] Helper functions extracted (show_layer_history, count_files_in_commit, parse_layer_name)
- [x] No code duplication

### User Experience Validation

- [x] Clear output format matches Git log style
- [x] Error messages are actionable
- [x] Help text explains command purpose
- [x] Exit codes follow Unix conventions
- [x] Command is responsive

---

## Anti-Patterns to Avoid

- ❌ Don't use `unwrap()` outside of tests - use `?` operator
- ❌ Don't assume refs exist - always check with `find_reference()`
- ❌ Don't forget to call `push_ref()` on Revwalk before iterating
- ❌ Don't show full 40-character hash - use short hash (7 chars)
- ❌ Don't iterate all commits without limit - respect `--count` argument
- ❌ Don't skip context validation - check mode/scope for layer applicability
- ❌ Don't ignore empty commits - handle `parent_count() == 0` case
- ❌ Don't show raw Git errors - wrap with JinError and provide context

---

## Implementation Summary

### Files Created/Modified

| File | Status | Lines | Description |
|------|--------|-------|-------------|
| `src/commands/log.rs` | **COMPLETE** | 248 | Full log command implementation |
| `src/cli/args.rs:102-112` | **COMPLETE** | 11 | LogArgs struct definition |
| `src/cli/mod.rs:60` | **COMPLETE** | 1 | Commands::Log enum variant |
| `src/commands/mod.rs:43` | **COMPLETE** | 1 | Command dispatch |

### Key Implementation Details

1. **Layer Filtering**: Uses `Layer::all_in_precedence_order()` and filters by `requires_mode()` and `requires_scope()` based on active context

2. **Revwalk Configuration**:
   - `revwalk.push_ref(&ref_path)` - start traversal from layer ref
   - `revwalk.set_sorting(Sort::TIME)` - chronological order

3. **Commit Metadata Extraction**:
   - Short hash: `&oid.to_string()[..7]`
   - Author: `commit.author().name().unwrap_or("unknown")`
   - Timestamp: `DateTime::from_timestamp(time.seconds(), 0)`

4. **File Count**: Uses `diff_tree_to_tree()` to compare with parent commit

5. **Error Handling**:
   - `JinError::NotInitialized` for uninitialized projects
   - `JinError::Other` for unknown layer names
   - Graceful "No commits yet" message for layers without commits

---

## Confidence Score

**10/10** - Implementation is complete and tested

**Reasoning:**
- ✅ Full implementation complete (248 lines)
- ✅ CLI wiring complete
- ✅ Unit tests pass
- ✅ Follows established patterns
- ✅ Error handling comprehensive
- ✅ Documentation clear

**Success Enablers:**
1. Existing command patterns to follow
2. git2-rs has excellent documentation
3. Layer system is well-designed
4. Testing infrastructure in place

---

## Appendix: Complete Implementation Reference

### src/commands/log.rs (lines 1-71 - Main execute function)

```rust
//! Implementation of `jin log`
//!
//! Shows commit history for layers.

use crate::cli::LogArgs;
use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::JinRepo;
use chrono::{DateTime, Utc};
use git2::Sort;

/// Execute the log command
///
/// Shows commit history.
pub fn execute(args: LogArgs) -> Result<()> {
    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    // Open Jin repository
    let repo = JinRepo::open_or_create()?;
    let git_repo = repo.inner();

    // Determine which layers to show history for
    if let Some(layer_name) = &args.layer {
        // Show history for specific layer
        let layer = parse_layer_name(layer_name)?;
        show_layer_history(git_repo, layer, &context, args.count)?;
    } else {
        // Show history for all layers with commits
        let all_layers = Layer::all_in_precedence_order();
        let mut shown_any = false;

        for layer in &all_layers {
            // Skip layers that don't apply to current context
            if layer.requires_mode() && context.mode.is_none() {
                continue;
            }
            if layer.requires_scope() && context.scope.is_none() {
                continue;
            }

            let ref_path = layer.ref_path(
                context.mode.as_deref(),
                context.scope.as_deref(),
                context.project.as_deref(),
            );

            // Check if ref exists
            if git_repo.find_reference(&ref_path).is_ok() {
                if shown_any {
                    println!();
                }
                println!("=== {} ===", layer);
                println!();
                show_layer_history(git_repo, *layer, &context, args.count)?;
                shown_any = true;
            }
        }

        if !shown_any {
            println!("No commits found in any layer");
        }
    }

    Ok(())
}
```

### src/commands/log.rs (lines 73-138 - show_layer_history function)

```rust
/// Show commit history for a specific layer
fn show_layer_history(
    repo: &git2::Repository,
    layer: Layer,
    context: &ProjectContext,
    count: usize,
) -> Result<()> {
    let ref_path = layer.ref_path(
        context.mode.as_deref(),
        context.scope.as_deref(),
        context.project.as_deref(),
    );

    // Check if ref exists
    let _reference = match repo.find_reference(&ref_path) {
        Ok(r) => r,
        Err(_) => {
            println!("No commits yet for layer: {}", layer);
            return Ok(());
        }
    };

    // Create revwalk
    let mut revwalk = repo.revwalk()?;
    revwalk.push_ref(&ref_path)?;
    revwalk.set_sorting(Sort::TIME)?;

    // Iterate through commits
    for (i, oid_result) in revwalk.enumerate() {
        if i >= count {
            break;
        }

        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Format commit hash (short)
        let hash_short = &oid.to_string()[..7];

        // Get commit metadata
        let author = commit.author();
        let author_name = author.name().unwrap_or("unknown");
        let author_email = author.email().unwrap_or("unknown");
        let message = commit.message().unwrap_or("(no message)");

        // Format timestamp
        let time = commit.time();
        let timestamp = DateTime::from_timestamp(time.seconds(), 0)
            .unwrap_or_else(|| DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH));

        // Count files changed in this commit
        let file_count = count_files_in_commit(repo, &commit)?;

        // Display commit
        println!("commit {} ({})", hash_short, layer);
        println!("Author: {} <{}>", author_name, author_email);
        println!("Date:   {}", timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!();
        println!("    {}", message.trim());
        println!();
        println!("    {} file(s) changed", file_count);
        println!();
    }

    Ok(())
}
```

### src/commands/log.rs (lines 140-183 - Helper functions)

```rust
/// Count files in a commit by comparing with parent
fn count_files_in_commit(repo: &git2::Repository, commit: &git2::Commit) -> Result<usize> {
    let tree = commit.tree()?;

    // If no parent, count all files in tree
    if commit.parent_count() == 0 {
        let mut count = 0;
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                count += 1;
            }
            git2::TreeWalkResult::Ok
        })?;
        return Ok(count);
    }

    // Otherwise, diff with parent
    let parent = commit.parent(0)?;
    let parent_tree = parent.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

    Ok(diff.deltas().count())
}

/// Parse layer name from string
fn parse_layer_name(name: &str) -> Result<Layer> {
    match name {
        "global-base" => Ok(Layer::GlobalBase),
        "mode-base" => Ok(Layer::ModeBase),
        "mode-scope" => Ok(Layer::ModeScope),
        "mode-scope-project" => Ok(Layer::ModeScopeProject),
        "mode-project" => Ok(Layer::ModeProject),
        "scope-base" => Ok(Layer::ScopeBase),
        "project-base" => Ok(Layer::ProjectBase),
        "user-local" => Ok(Layer::UserLocal),
        "workspace-active" => Ok(Layer::WorkspaceActive),
        _ => Err(JinError::Other(format!(
            "Unknown layer: {}. Valid layers: global-base, mode-base, \
             mode-scope, mode-scope-project, mode-project, scope-base, \
             project-base, user-local, workspace-active",
            name
        ))),
    }
}
```
