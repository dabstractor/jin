# Product Requirement Prompt (PRP): Log Command

## Goal

**Feature Goal**: Implement `jin log` command to display commit history for Jin layers using git2-rs RevWalk API.

**Deliverable**: A working `jin log` command that:
1. Shows commit history for a specified layer or all layers
2. Supports limiting output with `--count` flag
3. Displays commits in git-log-like format with colored output
4. Follows existing codebase patterns for CLI commands

**Success Definition**:
- Running `jin log` displays commit history for all layers
- Running `jin log <layer>` displays history for a specific layer
- Running `jin log --count N` limits output to N entries
- Output is formatted similar to git log (short SHA, author, date, message)
- All tests pass (unit and integration)
- Command is wired into main.rs following existing patterns

## User Persona

**Target User**: Developer using Jin for multi-layer configuration management

**Use Case**: User wants to see the commit history for their Jin layers to understand what changes were made, when, and by whom. This is useful for:
- Auditing configuration changes over time
- Understanding what changed in a specific layer
- Debugging configuration issues
- Reviewing the history before making changes

**User Journey**:
1. User runs `jin log` to see all layer commits
2. User runs `jin log project` to see project layer commits only
3. User runs `jin log mode/claude --count 5` to see last 5 commits for a mode
4. User reviews commit history to understand changes

**Pain Points Addressed**:
- Currently no way to view layer commit history
- Cannot audit changes to configuration layers
- No visibility into what changed and when

## Why

- **Audit trail**: Developers need to see what changes were made to configuration layers over time
- **Debugging**: Understanding history helps diagnose configuration issues
- **Transparency**: Shows the evolution of layer configurations
- **Integration**: Completes the core inspection commands (status, diff, log)

## What

User-visible behavior and technical requirements:

### CLI Interface
```bash
# Show all layer commits
jin log

# Show commits for a specific layer
jin log project
jin log mode/claude
jin log scope/python

# Limit output
jin log --count 10
jin log mode/claude --count 5
```

### Output Format
Git-log-like one-line format with color:
```
abc1234 (2 hours ago) John Doe <john@example.com>
    Update global configuration

def5678 (1 day ago) Jane Smith <jane@example.com>
    Add Python-specific settings
```

### Success Criteria
- [ ] Command executes without errors
- [ ] Shows commits in reverse chronological order (newest first)
- [ ] Short SHA (8 characters) displayed
- [ ] Author name and email shown
- [ ] Relative time displayed (e.g., "2 hours ago", "1 day ago")
- [ ] Commit message displayed on following line
- [ ] Layer name shown in parentheses when querying specific layer
- [ ] `--count` flag limits output correctly
- [ ] Error message shown for invalid layer specifications
- [ ] Graceful handling of layers with no commits
- [ ] Color coding for SHA (yellow), author (green), time (blue)

## All Needed Context

### Context Completeness Check

**Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"**

This PRP provides:
- Exact file paths and line numbers for all references
- Complete code patterns to follow
- git2-rs RevWalk API documentation
- Layer specification format from existing code
- CLI wiring pattern from main.rs
- Test patterns from status.rs and diff.rs

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: /home/dustin/projects/jin-glm-doover/src/cli/args.rs
  lines: 306-316
  why: LogCommand struct definition - shows the CLI args structure
  pattern: |
    #[derive(clap::Args)]
    pub struct LogCommand {
        #[arg(value_name = "LAYER")]
        pub layer: Option<String>,
        #[arg(long, value_name = "N")]
        pub count: Option<usize>,
    }

- file: /home/dustin/projects/jin-glm-doover/src/main.rs
  lines: 175-178
  why: Current placeholder implementation to replace with proper wiring
  pattern: |
    Commands::Log(cmd) => match commands::log_execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }

- file: /home/dustin/projects/jin-glm-doover/src/commands/mod.rs
  lines: 1-25
  why: Module export pattern - must add `pub mod log;` and `pub use log::execute as log_execute;`
  pattern: Module declarations follow alphabetical order, add after 'diff'

- file: /home/dustin/projects/jin-glm-doover/src/commands/diff.rs
  lines: 46-105
  why: Layer parsing function - use exact same pattern for parsing layer argument
  pattern: |
    fn parse_layer_spec(spec: &str, _project: &str) -> Result<Layer> {
        let parts: Vec<&str> = spec.split('/').collect();
        match parts.as_slice() {
            ["global"] => Ok(Layer::GlobalBase),
            ["mode", name] => Ok(Layer::ModeBase { mode: name.to_string() }),
            // ... full pattern in file
        }
    }

- file: /home/dustin/projects/jin-glm-doover/src/commands/status.rs
  lines: 1-75
  why: Simple command structure for read-only display commands
  pattern: |
    pub fn execute(_cmd: &StatusCommand) -> Result<()> {
        let workspace_root = std::env::current_dir()?;
        let context = ProjectContext::load(&workspace_root)?;
        // Display logic
        Ok(())
    }

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  lines: 376-405
  why: list_layer_refs() method - lists all layer refs that have commits
  critical: Returns Vec<(Layer, Oid)> - use this for 'jin log' (no args)

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  lines: 548-597
  why: ref_name_to_layer() - private helper for parsing ref names back to Layer
  gotcha: This is private - may need to make public or replicate pattern

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  lines: 216-280
  why: Layer::git_ref() - returns Git reference string for each layer type
  gotcha: UserLocal and WorkspaceActive return None (not versioned)

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  lines: 481-523
  why: Display impl for Layer - shows how to format layer names for output

- docfile: /home/dustin/projects/jin-glm-doover/plan/P4M5T2/research/git_log_cli_patterns.md
  why: Git log display patterns and best practices
  section: "Git Log's Most Useful Display Formats"

- docfile: /home/dustin/projects/jin-glm-doover/plan/P4M5T2/research/git2_revwalk_api.md
  why: Complete git2-rs RevWalk API reference with examples
  section: "Core RevWalk Usage" and "Practical Implementation Patterns"

- url: https://docs.rs/git2/latest/git2/struct.Revwalk.html
  why: Official git2-rs RevWalk API documentation
  critical: Revwalk::new(), push(), sorting, iteration patterns
```

### Current Codebase tree

```bash
src/
├── cli/
│   ├── args.rs          # LogCommand defined here (lines 306-316)
│   └── mod.rs
├── commands/
│   ├── mod.rs           # Must add: pub mod log; + pub use log::execute as log_execute;
│   ├── status.rs        # Reference for simple display commands
│   ├── diff.rs          # Reference for layer parsing (parse_layer_spec)
│   └── log.rs           # TO CREATE - main implementation
├── core/
│   ├── layer.rs         # Layer enum with git_ref() and Display impl
│   └── error.rs         # JinError types
├── git/
│   └── repo.rs          # JinRepo with list_layer_refs() - may need walk_layer_history()
└── main.rs              # Lines 175-178: replace placeholder with proper wiring
```

### Desired Codebase tree with files to be added

```bash
src/
├── commands/
│   ├── mod.rs           # MODIFY: add log module imports
│   └── log.rs           # CREATE: main log command implementation (~400 lines)
│       ├── execute() function
│       ├── walk_layer_history() helper
│       ├── format_commit_entry() display helper
│       ├── parse_layer_spec() or re-use from diff.rs
│       └── tests module
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: git2-rs RevWalk requires explicit sorting configuration
// Default order is NOT chronological - must set sorting for newest-first
let mut walk = repo.revwalk()?;
walk.sort(git2::Sort::TIME | git2::Sort::REVERSE);  // Important!

// CRITICAL: Layer::git_ref() returns Option<String> - UserLocal/WorkspaceActive return None
// Always check is_versioned() before calling git_ref()
if !layer.is_versioned() {
    return Err(JinError::Message("Layer has no history".to_string()));
}

// CRITICAL: JinRepo.list_layer_refs() only returns layers that EXIST and have commits
// Empty Vec means no layers have commits, not an error

// CRITICAL: parse_layer_spec from diff.rs uses specific error format
// Return JinError::Message with helpful text for invalid specs

// CRITICAL: main.rs expects specific error handling pattern
// All commands return Result<()> and are wrapped in match with eprintln!

// CRITICAL: ProjectContext load will fail if not in a Jin-initialized directory
// Check context_path.exists() first for friendly error message

// CRITICAL: Repository discovery uses git2::Repository::discover()
// This searches upward for .git directory - standard Git behavior
```

## Implementation Blueprint

### Data models and structure

No new data models required. Use existing types:
- `LogCommand` from `cli::args` - already defined
- `Layer` from `core::layer` - use for layer identification
- `git2::Commit` - from git2-rs for commit data

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/log.rs
  - IMPLEMENT: module header with documentation
  - FOLLOW pattern: src/commands/status.rs (file structure, doc comments)
  - NAMING: File name is `log.rs` (matches command name)
  - PLACEMENT: New file in src/commands/ directory

Task 2: IMPLEMENT execute() function skeleton
  - IMPLEMENT: pub fn execute(cmd: &LogCommand) -> Result<()>
  - FOLLOW pattern: src/commands/status.rs lines 41-75 (command structure)
  - NAMING: Function name is `execute` (matches all other commands)
  - DEPENDENCIES: Import LogCommand from cli::args
  - PLACEMENT: Top-level function in log.rs

Task 3: IMPLEMENT environment setup
  - IMPLEMENT: workspace_root detection, context loading, git repo opening
  - FOLLOW pattern: src/commands/status.rs lines 42-56 (exact same checks)
  - GOTCHA: Check Jin initialization before proceeding
  - PLACEMENT: Beginning of execute() function

Task 4: IMPLEMENT layer resolution logic
  - IMPLEMENT: Parse layer argument or get all layers
  - FOLLOW pattern: src/commands/diff.rs lines 71-105 (parse_layer_spec)
  - NAMING: Re-use `parse_layer_spec` from diff.rs or create local version
  - DEPENDENCIES: If layer specified, parse it; otherwise get all from list_layer_refs()
  - PLACEMENT: After environment setup in execute()

Task 5: ADD walk_layer_history() method to JinRepo (optional but recommended)
  - IMPLEMENT: pub fn walk_layer_history(&self, layer: &Layer, count: Option<usize>) -> Result<Vec<git2::Commit>>
  - FOLLOW pattern: src/git/repo.rs existing methods (error handling, Result return)
  - NAMING: Method name `walk_layer_history`
  - DEPENDENCIES: Uses git2::Revwalk for commit iteration
  - PLACEMENT: Add to impl JinRepo block in src/git/repo.rs after list_layer_refs()
  - ALTERNATIVE: Implement as local function in log.rs if modifying JinRepo is not desired

Task 6: IMPLEMENT commit history retrieval
  - IMPLEMENT: Use RevWalk to get commits for layer(s)
  - FOLLOW pattern: See research/git2_revwalk_api.md "Basic Iteration Pattern"
  - GOTCHA: Set sorting to TIME | REVERSE for newest-first order
  - DEPENDENCIES: Need valid git2::Repository and layer OID
  - PLACEMENT: In execute() after layer resolution

Task 7: IMPLEMENT display formatting
  - IMPLEMENT: format_commit_entry() helper for one-line git-style output
  - FOLLOW pattern: Git's --oneline format (SHA author time)
  - NAMING: Function name `format_commit_entry` or `display_commit`
  - GOTCHA: Use ANSI colors for SHA (yellow), author (green), time (blue)
  - PLACEMENT: Helper function in log.rs

Task 8: IMPLEMENT --count limiting
  - IMPLEMENT: Take only first N commits from result
  - FOLLOW pattern: Use .take(count) on iterator or Vec::truncate()
  - DEPENDENCIES: After commit collection, before display
  - PLACEMENT: In execute() between retrieval and display

Task 9: UPDATE src/commands/mod.rs
  - ADD: pub mod log; declaration
  - ADD: pub use log::execute as log_execute; export
  - FIND pattern: Add after 'diff' module (alphabetical order)
  - PLACEMENT: src/commands/mod.rs lines 4-25 area

Task 10: UPDATE src/main.rs
  - REPLACE: Lines 175-178 placeholder with proper dispatch
  - FOLLOW pattern: Lines 42-47 (status command wiring)
  - ADD: match commands::log_execute(&cmd) pattern
  - PLACEMENT: src/main.rs line 175-178

Task 11: CREATE tests module in log.rs
  - IMPLEMENT: Unit tests for happy path, edge cases, error handling
  - FOLLOW pattern: src/commands/status.rs lines 147-343 (test structure)
  - NAMING: test_log_shows_all_layers, test_log_specific_layer, etc.
  - COVERAGE: Execute with no commits, with commits, invalid layer, --count flag
  - PLACEMENT: #[cfg(test)] mod tests at end of log.rs

Task 12: CREATE integration test scenario
  - IMPLEMENT: Multi-layer history with multiple commits
  - FOLLOW pattern: src/commands/status.rs test fixture setup
  - NAMING: test_log_displays_commit_history_correctly
  - DEPENDENCIES: Create commits, verify log output
  - PLACEMENT: In tests module
```

### Implementation Patterns & Key Details

```rust
// Show critical patterns and gotchas

// PATTERN: Command execute function signature (from status.rs)
pub fn execute(cmd: &LogCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Check Jin initialization
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize."
                .to_string(),
        ));
    }

    // 3. Open Git and Jin repositories
    let _git_repo = git2::Repository::discover(&workspace_root)?;
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 4. Resolve layer(s) to query
    let layers = if let Some(layer_spec) = &cmd.layer {
        // Parse single layer
        let project = detect_project_name(&workspace_root)?;
        vec![parse_layer_spec(layer_spec, &project)?]
    } else {
        // Get all layers with commits
        let refs = repo.list_layer_refs()?;
        refs.into_iter().map(|(layer, _oid)| layer).collect()
    };

    // 5. Walk and display commits for each layer
    for layer in layers {
        display_layer_history(&repo, &layer, cmd.count)?;
    }

    Ok(())
}

// GOTCHA: git2-rs RevWalk requires explicit sorting for chronological order
// PATTERN: Proper RevWalk setup for newest-first commits
fn walk_layer_history(
    repo: &JinRepo,
    layer: &Layer,
    count: Option<usize>,
) -> Result<Vec<git2::Commit>> {
    // Get layer reference
    let reference = repo.get_layer_ref(layer)?
        .ok_or_else(|| JinError::Message(format!("Layer {} has no history", layer)))?;

    let commit_oid = reference.target()
        .ok_or_else(|| JinError::Message("Layer reference points to nothing".to_string()))?;

    // CRITICAL: Set sorting for newest-first order
    let mut walk = repo.inner.revwalk()?;
    walk.push(commit_oid)?;
    walk.sort(git2::Sort::TIME | git2::Sort::REVERSE)?;

    // Collect commits with optional limit
    let commits: Vec<git2::Commit> = walk
        .take(count.unwrap_or(usize::MAX))
        .filter_map(|oid| repo.find_commit(oid).ok())
        .collect();

    Ok(commits)
}

// PATTERN: ANSI color coding (reuse constants from diff.rs lines 38-44)
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_RESET: &str = "\x1b[0m";

fn display_layer_history(repo: &JinRepo, layer: &Layer, count: Option<usize>) -> Result<()> {
    let commits = walk_layer_history(repo, layer, count)?;

    if commits.is_empty() {
        println!("{}: (no commits)", layer);
        return Ok(());
    }

    println!("{}:", layer);

    for commit in commits {
        let short_sha = &commit.id().to_string()[..8];
        let author = commit.author();
        let author_name = author.name().unwrap_or("(unknown)");
        let time = format_commit_time(commit.time().seconds());
        let msg = commit.message().unwrap_or("(no message)");

        println!("  {}{}{} {} <{}@{}> {}{}{}",
            ANSI_YELLOW, short_sha, ANSI_RESET,
            author_name,
            // Relative time calculation needed
            ANSI_BLUE, time, ANSI_RESET,
            // First line of message
            msg.lines().next().unwrap_or("")
        );
    }

    Ok(())
}

// CRITICAL: Use parse_layer_spec from diff.rs for consistency
// Either import it or replicate the exact pattern
fn parse_layer_spec(spec: &str, project: &str) -> Result<Layer> {
    let parts: Vec<&str> = spec.split('/').collect();
    match parts.as_slice() {
        ["global"] => Ok(Layer::GlobalBase),
        ["mode", name] => Ok(Layer::ModeBase { mode: name.to_string() }),
        ["scope", name] => Ok(Layer::ScopeBase { scope: name.to_string() }),
        ["project", name] => Ok(Layer::ProjectBase { project: name.to_string() }),
        ["mode", mode_name, "scope", scope_name] => Ok(Layer::ModeScope {
            mode: mode_name.to_string(),
            scope: scope_name.to_string(),
        }),
        ["mode", mode_name, "project", proj_name] => Ok(Layer::ModeProject {
            mode: mode_name.to_string(),
            project: proj_name.to_string(),
        }),
        ["mode", mode_name, "scope", scope_name, "project", proj_name] => {
            Ok(Layer::ModeScopeProject {
                mode: mode_name.to_string(),
                scope: scope_name.to_string(),
                project: proj_name.to_string(),
            })
        }
        _ => Err(JinError::Message(format!(
            "Invalid layer specification: '{}'. Expected format: global, mode/<name>, scope/<name>, project/<name>, mode/<m>/scope/<s>, mode/<m>/project/<p>, or mode/<m>/scope/<s>/project/<p>",
            spec
        ))),
    }
}
```

### Integration Points

```yaml
GIT:
  - dependency: git2::Repository::discover() for finding repo
  - dependency: git2::Revwalk for commit iteration
  - pattern: walk.sort(git2::Sort::TIME | git2::Sort::REVERSE) for chronological order

JINREPO:
  - use: repo.list_layer_refs() for getting all layers
  - use: repo.get_layer_ref() for getting specific layer ref
  - use: repo.find_commit() for loading commit objects
  - potential_addition: repo.walk_layer_history() method

CONFIG:
  - use: ProjectContext::load() for detecting initialization
  - use: ProjectContext::context_path() for checking .jin exists
  - use: detect_project_name() helper (from diff.rs or local)

ROUTES:
  - modify: src/commands/mod.rs (add module declaration)
  - modify: src/main.rs (replace placeholder at lines 175-178)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after file creation - fix before proceeding
cargo check --bin jin                   # Check compilation
cargo clippy --bin jin -W clippy::all   # Lint checks

# Format if needed
cargo fmt --bin jin

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test log command specifically
cargo test --package jin_glm --lib commands::log --verbose

# Test all commands to ensure no regression
cargo test --package jin_glm --lib commands --verbose

# Specific test scenarios
cargo test test_log_shows_all_layers -- --exact
cargo test test_log_specific_layer -- --exact
cargo test test_log_with_count_limit -- --exact
cargo test test_log_invalid_layer_error -- --exact

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual testing scenarios
cd /tmp
mkdir test-log && cd test-log
git init
jin init

# Create test commits
echo "test1" > config1.txt
jin add config1.txt --global
jin commit -m "Add global config"

echo "test2" > config2.txt
jin add config2.txt --mode claude
jin commit -m "Add claude mode config"

# Test log command
jin log                    # Should show both commits with layer names
jin log global             # Should show only global layer commits
jin log mode/claude        # Should show only claude mode commits
jin log --count 1          # Should show only 1 commit
jin log invalid_layer      # Should error with helpful message

# Expected: All commands work correctly, output is properly formatted
```

### Level 4: Domain-Specific Validation

```bash
# Test with multiple commits in same layer
for i in 1 2 3; do
    echo "test$i" > file$i.txt
    jin add file$i.txt --project
    jin commit -m "Commit $i"
done

# Verify chronological order (newest first)
jin log --project | head -1  # Should show "Commit 3"

# Test with layers that have no commits
jin log mode/nonexistent     # Should show "(no commits)" or similar

# Test relative time formatting
# (Create commits, wait, verify time display)

# Expected: Proper ordering, time display, handling of edge cases
```

## Final Validation Checklist

### Technical Validation
- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin_glm --lib commands`
- [ ] No linting errors: `cargo clippy --bin jin`
- [ ] No formatting issues: `cargo fmt --check --bin jin`

### Feature Validation
- [ ] `jin log` (no args) shows commits from all layers
- [ ] `jin log <layer>` shows commits for specific layer only
- [ ] `jin log --count N` limits output to N commits
- [ ] Commits displayed in reverse chronological order (newest first)
- [ ] Short SHA (8 chars) displayed in yellow
- [ ] Author name and email displayed in green
- [ ] Relative time displayed in blue
- [ ] Commit message shown on following line
- [ ] Layer name shown as header for each layer
- [ ] Invalid layer specification shows helpful error message
- [ ] Unversioned layers (UserLocal, WorkspaceActive) rejected with error
- [ ] Empty layer history shows "(no commits)" message

### Code Quality Validation
- [ ] Follows existing command patterns from status.rs and diff.rs
- [ ] Uses parse_layer_spec pattern from diff.rs
- [ ] Error handling matches other commands (JinError::Message with helpful text)
- [ ] File placement matches desired structure (log.rs in commands/)
- [ ] Module exports added to mod.rs correctly
- [ ] main.rs wiring follows established pattern

### Documentation & Deployment
- [ ] Module documentation comment at top of log.rs
- [ ] Function documentation comments for public functions
- [ ] Inline comments for non-obvious logic (RevWalk sorting, etc.)
- [ ] Test documentation comments explaining what each test validates
```

## Anti-Patterns to Avoid

- ❌ Don't create a new layer parsing function - reuse `parse_layer_spec` from diff.rs
- ❌ Don't skip RevWalk sorting - without sorting, order is undefined
- ❌ Don't ignore unversioned layers - explicitly reject UserLocal/WorkspaceActive
- ❌ Don't use synchronous git operations that could block - RevWalk is iterator-based (good)
- ❌ Don't hardcode commit limit defaults - respect user's `--count` or show all
- ❌ Don't panic on missing commits - use Result<()> with JinError
- ❌ Don't forget to check Jin initialization before accessing repository
- ❌ Don't display full SHA - use 8-character short SHA like git log
- ❌ Don't use complex output formats - stick to one-line git-style format
- ❌ Don't ignore the case where layer exists but has no commits - handle gracefully

---

## Confidence Score

**8/10** - High confidence in one-pass implementation success

### Reasoning:
- Clear precedents in status.rs (simple display) and diff.rs (layer parsing)
- Well-documented git2-rs RevWalk API
- Research documents provide implementation examples
- Exact file paths and line numbers provided
- Complete task breakdown with dependencies

### Risks:
- RevWalk sorting behavior may need testing
- Relative time formatting requires additional implementation
- JinRepo modifications (if adding walk_layer_history method) need careful integration

### Mitigation:
- Research docs include sorting patterns
- Can use simple timestamp formatting initially
- Option to implement walk_layer_history as local function first
