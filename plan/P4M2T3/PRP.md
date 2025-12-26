name: "PRP for P4.M2.T3: jin commit Command"
description: |

---

## Goal

**Feature Goal**: Implement the `jin commit` command that atomically commits staged files to their respective Jin layers using the existing commit pipeline infrastructure.

**Deliverable**: A fully functional `jin commit` CLI command that:
- Validates that files are staged before allowing commit
- Accepts a commit message via `-m` flag
- Optionally allows empty commits with `--allow-empty` flag
- Executes the atomic commit pipeline across all affected layers
- Clears the staging index on successful commit
- Provides user-friendly output showing what was committed

**Success Definition**: The command is complete when:
1. Running `jin commit -m "message"` with staged files commits them atomically to layers
2. Running `jin commit -m "message"` with no staged files shows helpful error
3. Running `jin commit -m "message" --allow-empty` allows empty commits
4. The staging index is cleared after successful commit
5. Error messages are clear and actionable
6. The implementation follows existing command patterns (init, add)

## User Persona

**Target User**: Developer using Jin to manage layered configuration files

**Use Case**: Developer has staged configuration files using `jin add` and wants to atomically commit those changes to the appropriate Jin layers

**User Journey**:
1. Developer runs `jin add config.toml` to stage a file
2. Developer runs `jin commit -m "Add database config"` to commit
3. Jin validates staging, builds trees, creates commits, updates refs atomically
4. Staging is cleared, developer receives confirmation

**Pain Points Addressed**:
- Without commit command, staged files cannot be persisted to layer history
- Manual layer commits would be error-prone and non-atomic
- Users expect git-like commit workflow

## Why

- **Core Workflow Completion**: The commit command is essential for Jin's staging workflow. Without it, users can stage files but never persist them to layer history.
- **Atomicity Guarante**: Leveraging the existing transaction system ensures all layer updates succeed or fail together, preventing partial/inconsistent state.
- **Integration**: The commit pipeline is already implemented (P3.M2.T1) - this PRP wires it to the CLI.

## What

Implement `jin commit` command with the following behavior:

### User-Visible Behavior

```bash
# Commit staged files with message
$ jin commit -m "Add production database config"
Committing 3 files to 2 layers...
  ModeBase(claude):
    - config/claude.toml
  ProjectBase(myapp):
    - config/database.toml
    - config/cache.toml
Committed successfully (transaction: abc-123-def)

# Error: No staged files
$ jin commit -m "message"
Error: No files staged for commit.
Use 'jin add <file>' to stage files first, or use --allow-empty to force an empty commit.

# Error: No message provided
$ jin commit
error: the following required arguments were not provided:
  --message <MESSAGE>

# Allow empty commit
$ jin commit -m "Initial commit" --allow-empty
Committed successfully (transaction: def-456-abc)
```

### Success Criteria

- [ ] Command accepts `-m` flag for commit message (required)
- [ ] Command accepts `--allow-empty` flag for empty commits (optional)
- [ ] Command validates staging has entries before committing (unless `--allow-empty`)
- [ ] Command calls `CommitPipeline::execute()` with staging index
- [ ] Command clears staging index after successful commit
- [ ] Command displays summary of committed files by layer
- [ ] Command shows transaction ID on success
- [ ] Error messages are clear and actionable
- [ ] Exit code 0 on success, non-zero on failure

## All Needed Context

### Context Completeness Check

_The following context enables one-pass implementation:_

- CLI command structure patterns from `init` and `add` commands
- Complete `CommitPipeline` API for orchestrating commits
- `StagingIndex` API for loading/managing staged files
- `TransactionManager` for atomic layer updates
- `JinError` variants for error handling
- Exit code mapping conventions
- Existing test patterns for CLI commands

### Documentation & References

```yaml
# MUST READ - Core Implementation Files

- file: src/commands/add.rs
  why: Complete example of CLI command implementation pattern to follow
  pattern: execute() function structure, error handling, output formatting
  gotcha: Note how add resolves paths and validates files

- file: src/commands/init.rs
  why: Simpler command example showing basic command structure
  pattern: Minimal command with status output

- file: src/cli/args.rs (lines 232-240)
  why: CommitCommand struct is already defined - must use existing definition
  pattern: #[arg(long, short, required = true)] for message field
  gotcha: message is required, allow_empty is optional bool flag

- file: src/commit/pipeline.rs
  why: Complete CommitPipeline API that commit command must use
  pattern: pipeline.execute(&mut staging) returns CommitResult
  critical: pipeline clears staging automatically on success

- file: src/staging/index.rs
  why: StagingIndex API for loading staged entries
  pattern: StagingIndex::load_from_disk() returns Err if no index exists
  gotcha: Use unwrap_or_else(|_| StagingIndex::new()) for missing index

- file: src/git/transaction.rs
  why: Transaction system for atomic commits (used by pipeline)
  pattern: Pipeline handles all transaction logic internally
  gotcha: Command does NOT interact with Transaction directly

- file: src/core/error.rs
  why: All error types available for error handling
  pattern: JinError variants with specific context fields
  critical: Use JinError::Message for custom error strings

- file: src/main.rs (lines 8-190)
  why: Command dispatch pattern in main()
  pattern: match commands::execute(&cmd) with error handling
  gotcha: Must export execute function from commands/mod.rs

# CRITICAL GOTCHAS

1. CommitCommand struct ALREADY EXISTS in args.rs with message (required) and allow_empty (optional)
   - DO NOT redefine - use existing struct at src/cli/args.rs:232-240

2. CommitPipeline.execute() AUTOMATICALLY clears staging on success
   - DO NOT manually clear staging - pipeline does this at line 223

3. StagingIndex::load_from_disk() returns Err if .jin/staging/index.json doesn't exist
   - Use unwrap_or_else(|_| StagingIndex::new()) to handle missing index

4. Exit code mapping is defined but main.rs currently uses ExitCode::FAILURE
   - TODO for future: Use e.exit_code() for proper exit codes (not required for this PRP)

5. Project name detection uses same pattern as add.rs
   - Use detect_project_name() helper or inline Git remote detection

6. JinRepo initialization pattern
   - JinRepo::open_or_create(workspace_root) for opening repo
```

### Current Codebase Tree

```bash
src/
├── main.rs                    # CLI entry point with command dispatch
├── lib.rs                     # Library exports
├── cli/
│   ├── mod.rs                 # CLI module exports
│   └── args.rs                # ALL command definitions including CommitCommand
├── commands/
│   ├── mod.rs                 # Command handler exports (add execute, init execute)
│   ├── init.rs                # Init command implementation
│   ├── add.rs                 # Add command implementation
│   └── commit.rs              # TODO: Create this file for commit implementation
├── commit/
│   ├── mod.rs
│   ├── pipeline.rs            # CommitPipeline with execute() method
│   ├── validate.rs            # Validation functions
│   ├── audit.rs               # Audit logging
│   └── jinmap.rs              # Jinmap management
├── staging/
│   ├── mod.rs
│   ├── index.rs               # StagingIndex with load_from_disk()
│   ├── entry.rs               # StagedEntry type
│   └── router.rs              # Layer routing
├── git/
│   ├── mod.rs
│   ├── repo.rs                # JinRepo wrapper
│   └── transaction.rs         # Transaction system (used by pipeline)
└── core/
    ├── mod.rs
    ├── error.rs               # JinError enum with all variants
    ├── config.rs              # ProjectContext
    └── layer.rs               # Layer enum
```

### Desired Codebase Tree (new files)

```bash
src/commands/
└── commit.rs                  # NEW: Commit command implementation
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: CommitCommand struct ALREADY EXISTS - do not redefine
// Location: src/cli/args.rs:232-240
// The struct is already added to the Commands enum
// Only need to implement the execute() function

// CRITICAL: CommitPipeline.execute() CLEARS STAGING AUTOMATICALLY
// Location: src/commit/pipeline.rs:223
// Do NOT call staging.clear() after pipeline.execute()

// CRITICAL: StagingIndex may not exist on first run
// Use: StagingIndex::load_from_disk(workspace).unwrap_or_else(|_| StagingIndex::new())

// CRITICAL: JinRepo pattern for opening repo
// Use: JinRepo::open_or_create(workspace_root)

// CRITICAL: Project name detection from Git remote or directory
// See add.rs:236-261 for detect_project_name() implementation

// CRITICAL: Git repository discovery pattern
// git2::Repository::discover(workspace_root) for finding .git dir

// CRITICAL: Workspace root is always current directory
// std::env::current_dir() for workspace_root
```

## Implementation Blueprint

### Data Models and Structure

No new data models required. The commit command uses existing types:

- **CommitCommand** (already defined in `args.rs`): CLI argument struct
- **CommitPipeline** (already implemented): Orchestrates commit flow
- **StagingIndex** (already implemented): Manages staged entries
- **CommitResult** (already defined): Contains transaction ID, commits, files

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/commit.rs
  - IMPLEMENT: execute() function with signature:
    pub fn execute(cmd: &CommitCommand) -> Result<()>
  - FOLLOW pattern: src/commands/add.rs (lines 51-116)
  - STRUCTURE:
    1. Get workspace_root with std::env::current_dir()
    2. Detect project_name (Git remote or directory name)
    3. Validate Git repo exists with git2::Repository::discover()
    4. Open JinRepo with JinRepo::open_or_create()
    5. Load StagingIndex (create new if missing)
    6. Validate staging has entries (unless allow_empty)
    7. Create CommitPipeline with repo, workspace_root, project
    8. Execute pipeline with staging
    9. Display success output with transaction ID and file summary
  - ERROR HANDLING:
    - JinError::RepoNotFound if Git repo missing
    - JinError::Message("No files staged...") if staging empty
    - Propagate pipeline errors (ValidationError, TransactionConflict, CommitFailed)
  - OUTPUT FORMAT: Follow add.rs pattern (lines 97-113)
  - NAMING: Function name must be execute() for consistency
  - PLACEMENT: src/commands/commit.rs

Task 2: MODIFY src/commands/mod.rs
  - ADD: pub mod commit;
  - ADD: pub use commit::execute as commit_execute;
  - FOLLOW pattern: Existing add and init exports
  - PLACEMENT: After init/add imports, before tests

Task 3: MODIFY src/main.rs
  - INTEGRATE: Add Commands::Commit(cmd) match arm
  - FIND pattern: Commands::Add and Commands::Init match arms (lines 16-31)
  - ADD:
    Commands::Commit(cmd) => match commands::commit_execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    },
  - PRESERVE: All existing command match arms
  - PLACEMENT: After Add command, before Status command

Task 4: CREATE src/commands/tests/test_commit.rs (optional but recommended)
  - IMPLEMENT: Unit tests following add.rs pattern (lines 282-842)
  - TEST CASES:
    - test_commit_with_staged_files()
    - test_commit_no_staged_files_error()
    - test_commit_allow_empty_flag()
    - test_commit_clears_staging()
  - FIXTURES: DirGuard, create_test_file, init_git_repo, init_jin helpers
  - PLACEMENT: In commit.rs module or separate test file
```

### Implementation Patterns & Key Details

```rust
// ===== PATTERN 1: Command Execute Function Structure =====
// From src/commands/add.rs:51-116

pub fn execute(cmd: &CommitCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Detect project name (Git remote or directory)
    let project_name = detect_project_name(&workspace_root)?;

    // 3. Validate Git repository exists
    let _git_repo = git2::Repository::discover(&workspace_root)
        .map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // 4. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 5. Load staging index (create new if doesn't exist)
    let mut staging_index = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());

    // 6. Validate staging has entries (unless allow_empty)
    if !cmd.allow_empty && staging_index.len() == 0 {
        return Err(JinError::Message(
            "No files staged for commit.\n\
             Use 'jin add <file>' to stage files first, or use --allow-empty to force an empty commit."
                .to_string(),
        ));
    }

    // 7. Show progress message
    let file_count = staging_index.len();
    let layer_count = count_unique_layers(&staging_index);
    println!("Committing {} file(s) to {} layer(s)...", file_count, layer_count);

    // 8. Create and execute pipeline
    let pipeline = CommitPipeline::new(&repo, &workspace_root, project_name);
    let result = pipeline.execute(&mut staging_index)?;

    // 9. Display success output
    println!("\nCommitted successfully (transaction: {})", result.transaction_id);
    println!("\nLayers updated:");
    for (layer, oid) in &result.commits {
        println!("  {:?}: {}", layer, oid);
    }

    Ok(())
}

// ===== PATTERN 2: Project Name Detection =====
// From src/commands/add.rs:236-261

fn detect_project_name(workspace_root: &Path) -> Result<String> {
    use git2::Repository;

    let repo = Repository::discover(workspace_root).map_err(|_| JinError::RepoNotFound {
        path: workspace_root.display().to_string(),
    })?;

    // Try to get from git remote origin
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.rsplit('/').next() {
                let name = name.trim_end_matches(".git");
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }

    // Fallback to directory name
    workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| JinError::Message("Cannot determine project name".to_string()))
}

// ===== PATTERN 3: Count Unique Layers Helper =====
fn count_unique_layers(staging: &StagingIndex) -> usize {
    use std::collections::HashSet;
    let mut layers = HashSet::new();
    for entry in staging.all_entries() {
        layers.insert(entry.layer.clone());
    }
    layers.len()
}

// ===== GOTCHA: CommitPipeline Handles Staging Clear =====
// The pipeline.execute() method AUTOMATICALLY clears staging at line 223
// Do NOT call staging.clear() after pipeline returns Ok

// ===== PATTERN 4: Error Message Formatting =====
// Match add.rs style for user-friendly errors
return Err(JinError::Message(
    "No files staged for commit.\n\
     Use 'jin add <file>' to stage files first, or use --allow-empty to force an empty commit."
        .to_string(),
));
```

### Integration Points

```yaml
MAIN.RS:
  - add_to: src/main.rs Commands match arm
  - pattern: |
    Commands::Commit(cmd) => match commands::commit_execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    },
  - location: After Add command, before Status (around line 35)

COMMANDS_MOD.RS:
  - add_to: src/commands/mod.rs
  - pattern: |
    pub mod commit;
    pub use commit::execute as commit_execute;
  - location: After existing module declarations
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating commit.rs - fix before proceeding
cargo check --bin jin                   # Check compilation
cargo clippy --bin jin -- -D warnings   # Lint checks

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues:
# - Missing use statements
# - Type mismatches (Result<> vs ())
# - Unused variables
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the commit command specifically
cargo test --package jin-glm --lib commands::commit::tests -- --nocapture

# Run all command tests
cargo test --package jin-glm --lib commands::tests -- --nocapture

# Expected: All tests pass. If failing, debug root cause and fix implementation.

# Test to verify:
# 1. test_commit_with_staged_files - succeeds with staged entries
# 2. test_commit_no_staged_files_error - fails without entries
# 3. test_commit_allow_empty_flag - succeeds with flag even if empty
# 4. test_commit_clears_staging - staging cleared after commit
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual integration test sequence
cd /tmp && mkdir -p test-jin-commit && cd test-jin-commit

# Initialize Git and Jin
git init
jin init

# Create and stage a test file
echo "test config" > config.toml
jin add config.toml

# Commit the staged file
jin commit -m "Add test config"
# Expected output:
# Committing 1 file(s) to 1 layer(s)...
# Committed successfully (transaction: <uuid>)

# Verify staging was cleared
jin status  # Should show no staged files

# Test error: commit without staging
jin commit -m "Should fail"
# Expected: Error: No files staged for commit...

# Test allow-empty flag
jin commit -m "Empty commit" --allow-empty
# Expected: Committed successfully...

# Cleanup
cd /tmp && rm -rf test-jin-commit
```

### Level 4: Domain-Specific Validation

```bash
# Verify transaction atomicity
# The commit pipeline creates atomic transactions
# Check that all layer refs update together or none do

# Inspect Jin references after commit
cd /tmp/test-jin-commit
git show-ref --heads | grep jin/
# Should show refs/jin/layers/* with commit OIDs

# Verify audit log was created
cat .jin/audit/*.json
# Should contain audit entry for the commit

# Verify jinmap was updated
cat .jin/jinmap.json
# Should contain file -> layer mappings
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin-glm`
- [ ] No clippy warnings: `cargo clippy --bin jin`
- [ ] No compilation errors: `cargo check --bin jin`
- [ ] Command is wired in main.rs match arm
- [ ] Command module is exported in commands/mod.rs

### Feature Validation

- [ ] Commit accepts `-m` flag (required)
- [ ] Commit accepts `--allow-empty` flag (optional)
- [ ] Error shown when no files staged (without --allow-empty)
- [ ] Success message shows transaction ID
- [ ] Staging is cleared after successful commit
- [ ] Output shows file/layer summary
- [ ] Git repo validation works

### Code Quality Validation

- [ ] Follows add.rs command pattern
- [ ] Uses existing CommitPipeline (no custom transaction logic)
- [ ] Error messages are user-friendly and actionable
- [ ] Helper functions follow naming conventions (snake_case)
- [ ] Proper use of JinError variants
- [ ] No duplication of existing code

### Documentation & Deployment

- [ ] Function has Rustdoc comments
- [ ] Error cases documented in Rustdoc
- [ ] Examples in Rustdoc (if applicable)

---

## Anti-Patterns to Avoid

- **Don't** redefine CommitCommand struct - it already exists in args.rs
- **Don't** manually clear staging after pipeline.execute() - pipeline does this
- **Don't** interact with Transaction directly - use CommitPipeline
- **Don't** hardcode project name - detect from Git remote or directory
- **Don't** skip Git repository validation - must verify repo exists
- **Don't** ignore allow_empty flag - must respect this for empty staging
- **Don't** use verbose output - keep output concise like add command
- **Don't** create new error types - use existing JinError variants
- **Don't** forget to export commit_execute from commands/mod.rs
- **Don't** forget to wire Commands::Commit in main.rs

---

## Confidence Score: 9/10

**One-pass implementation likelihood**: Very High

**Reasoning**:
1. Complete existing pattern to follow (add.rs is nearly identical structure)
2. CommitPipeline is fully implemented and tested
3. Clear error handling patterns established
4. All dependencies are in place
5. Command struct already defined
6. Only requires wiring existing components

**Remaining 10% uncertainty**:
- Minor adjustments to output format based on user preference
- Potential edge cases in project name detection
- Integration testing may reveal minor workflow adjustments
