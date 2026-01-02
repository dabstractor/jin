# Product Requirement Prompt: List Command

---

## Goal

**Feature Goal**: Implement a command that lists available modes, scopes, and projects from the Jin repository.

**Deliverable**: `jin list` CLI command that enumerates all available modes/scopes/projects from the Jin repository's layer refs, with categorized and alphabetically sorted output.

**Success Definition**:
- Command executes without errors (handles uninitialized state gracefully)
- Parses all layer refs under `refs/jin/layers/**` in Git repository
- Categorizes results into Modes, Scopes, and Projects sections
- Displays results in alphabetical order within each category
- Shows helpful usage hints at the end
- Handles empty repository state with informative message

## User Persona

**Target User**: Developers using Jin who need to discover available modes, scopes, and projects

**Use Case**: User wants to see what configuration resources are available in the shared Jin repository

**User Journey**:
1. User runs `jin list` from any directory
2. Command reads layer refs from global Jin repository at `~/.jin/`
3. Ref paths are parsed to extract mode/scope/project names
4. Results are displayed in categorized sections (Modes, Scopes, Projects)
5. User sees hints on how to activate discovered resources

**Pain Points Addressed**:
- Discoverability of available configuration modes
- Finding what scopes exist in the organization
- Identifying projects that can be cloned/used
- Understanding the naming patterns for modes/scopes/projects

## Why

- **Discovery**: New team members need to see what configuration resources exist
- **Onboarding**: Helps users understand available modes (e.g., editor configurations)
- **Organization**: Teams can list shared scopes and projects
- **Visibility**: Without list command, users must manually explore Git refs or ask others
- **Remote Sync**: After `jin pull`, users can see newly available resources

## What

### Command Behavior

```bash
# Basic usage - lists all available resources
jin list

# Output when modes/scopes/projects exist:
Available in Jin repository:

Modes:
  - claude
  - cursor
  - zed

Scopes:
  - language:javascript
  - language:python
  - language:rust

Projects:
  - ui-dashboard
  - api-server

Use 'jin mode use <mode>' to activate a mode
Use 'jin scope use <scope>' to activate a scope

# Output when nothing exists:
Available in Jin repository:

  (no modes, scopes, or projects found)

Use 'jin mode use <mode>' to activate a mode
Use 'jin scope use <scope>' to activate a scope

# Error when Jin not initialized:
$ jin list
Error: Jin not initialized. Run 'jin init' first.
```

### Success Criteria

- [ ] Command reads all refs under `refs/jin/layers/**` pattern
- [ ] Ref paths are parsed correctly to extract mode/scope/project names
- [ ] Results are grouped by category (Modes, Scopes, Projects)
- [ ] Each category is sorted alphabetically
- [ ] Empty categories are hidden (not shown)
- [ ] Empty repository shows helpful message
- [ ] Usage hints are displayed at the end
- [ ] Returns `JinError::NotInitialized` when Jin repo doesn't exist

## All Needed Context

### Context Completeness Check

_Before proceeding, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: YES - This PRP provides all necessary context including:
- Complete layer ref path structure specification
- Exact file patterns and naming conventions
- CLI command patterns to follow
- Code examples from similar commands
- Validation procedures specific to this codebase
- Git ref parsing patterns with concrete examples

### Documentation & References

```yaml
# MUST READ - Core Type Definitions
- file: src/core/layer.rs
  why: Defines Layer enum with ref_path() patterns for mode/scope/project layers
  critical: Shows exact ref path format: refs/jin/layers/mode/{mode}, refs/jin/layers/scope/{scope}, etc.
  pattern: Layer enum with ref_path() returning Git ref paths

- file: src/git/repo.rs
  why: Defines JinRepo wrapper for Git operations
  critical: JinRepo::open() for accessing bare repository at ~/.jin/
  pattern: All commands use this pattern for repo access

- file: src/commands/list.rs
  why: THIS FILE - The list command implementation
  critical: Main execute() function and parse_ref_path() helper
  pattern: HashSet-based collection, sorted display, categorized output

- file: src/commands/layers.rs
  why: Similar display command with layer iteration and categorization
  pattern: Category-based output with sorted items

- file: src/commands/context.rs
  why: Simple command showing context loading and basic display
  pattern: Straightforward println! output structure

# External Research - CLI List Command Best Practices
- url: https://kubernetes.io/docs/reference/kubectl/quick-reference/
  why: kubectl get command - industry standard for categorized list output
  section: "kubectl get" and "Listing resources"
  critical: Shows how to handle empty results, categorization, and sorting

- url: https://git-scm.com/docs/git-branch
  why: git branch - classic list command with sorting and filtering
  section: "Listing branches" and "--list" option
  critical: Pattern matching and sorting conventions

- url: https://docs.docker.com/engine/cli/formatting/
  why: Docker CLI formatting - table vs JSON output patterns
  section: "Format command and log output"
  critical: Shows clean, categorized output formatting

- url: https://cli.github.com/manual/gh_help_formatting
  why: GitHub CLI - modern Rust CLI with list commands
  section: Template formatting and JSON output
  critical: JSON vs table format patterns
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs          # Command argument structs (ListArgs not needed - no args)
│   └── mod.rs           # Commands enum - already has List variant at line 77-78
├── commands/
│   ├── mod.rs           # Command module exports and routing - list::execute() at line 49
│   ├── list.rs          # THIS FILE - List command implementation
│   ├── layers.rs        # Similar display command (pattern reference)
│   ├── context.rs       # Simple display command (pattern reference)
│   ├── mode.rs          # Mode management with subcommands
│   └── scope.rs         # Scope management with subcommands
├── core/
│   ├── config.rs        # ProjectContext struct
│   ├── layer.rs         # Layer enum definition with ref_path patterns
│   ├── error.rs         # JinError enum (NotInitialized variant)
│   └── mod.rs           # Core module exports
└── git/
    ├── repo.rs          # JinRepo::open() for repository access
    └── mod.rs           # Git module exports

tests/
├── cli_basic.rs         # Basic CLI integration tests - test_list_subcommand at line 556
├── cli_layers.rs        # Layers command integration tests (pattern reference)
└── common/
    ├── fixtures.rs      # Test helper functions (TestFixture, jin(), create_mode, create_scope)
    ├── assertions.rs    # Custom assertions (assert_layer_ref_exists, assert_context_mode)
    └── mod.rs           # Common test utilities
```

### Desired Codebase Tree

The command is already implemented. This PRP documents the existing implementation for maintenance and reference.

```bash
# No changes required - implementation complete at src/commands/list.rs
# This PRP serves as documentation of the implementation
# Potential enhancement: Add comprehensive integration tests in tests/cli_list.rs
```

### Known Gotchas of Codebase & Library Quirks

```rust
// CRITICAL: Jin repository is a BARE repository at ~/.jin/
// No working directory exists - only Git objects and refs
let repo = JinRepo::open()?;  // Use open(), not open_or_create()
// If repo doesn't exist, return JinError::NotInitialized

// CRITICAL: Ref paths follow specific patterns for layer system
// Must parse these patterns to extract mode/scope/project names:
// refs/jin/layers/global              -> ignore (special system layer)
// refs/jin/layers/mode/{mode}         -> extract mode
// refs/jin/layers/mode/{mode}/scope/{scope} -> extract both
// refs/jin/layers/mode/{mode}/scope/{scope}/project/{project} -> extract all three
// refs/jin/layers/mode/{mode}/project/{project} -> extract mode and project
// refs/jin/layers/scope/{scope}       -> extract scope
// refs/jin/layers/project/{project}   -> extract project
// refs/jin/layers/local               -> ignore (special system layer)
// refs/jin/layers/workspace           -> ignore (special system layer)

// GOTCHA: references_glob() uses ** pattern for recursive matching
// Must use "refs/jin/layers/**" to match all nested refs
let refs = git_repo.references_glob("refs/jin/layers/**")?;

// GOTCHA: Scope names may contain colons (e.g., "language:javascript")
// Colons are preserved in display - they are NOT converted to slashes in layer refs
// (Unlike mode/scope refs which use slashes for nesting)

// GOTCHA: HashSet automatically deduplicates
// Same mode/scope/project may appear in multiple ref paths
// Use HashSet<String> to collect, then convert to Vec for sorting
let mut modes = HashSet::new();
// ... collect into set
let mut mode_list: Vec<_> = modes.into_iter().collect();
mode_list.sort();  // Alphabetical sort

// PATTERN: Check for empty results and show appropriate message
if !modes.is_empty() && !scopes.is_empty() && !projects.is_empty() {
    println!("  (no modes, scopes, or projects found)");
}

// GOTCHA: println! doesn't add trailing newline automatically
// Must add println!() for blank line between sections
println!("Modes:");
for mode in mode_list {
    println!("  - {}", mode);
}
println!();  // Blank line after section

// GOTCHA: Test fixtures use std::process::id() for unique names
let mode_name = format!("test_mode_{}", std::process::id());
```

## Implementation Blueprint

### Data Models and Structure

The list command uses existing types - no new models required:

```rust
// From src/git/repo.rs - Already defined
pub struct JinRepo {
    repo: git2::Repository,
}

impl JinRepo {
    pub fn open() -> Result<Self>;  // Open existing, fail if not exists
    pub fn inner(&self) -> &git2::Repository;  // Access underlying git2 repo
}

// From src/core/error.rs - Already defined
pub enum JinError {
    NotInitialized,
    Git(git2::Error),
    // ... other variants
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
# Task 1: Verify CLI Registration (ALREADY DONE)
# File: src/cli/mod.rs
# Line 77-78: List variant exists in Commands enum
# No arguments needed for this command

# Task 2: Verify Command Routing (ALREADY DONE)
# File: src/commands/mod.rs
# Line 49: Commands::List => list::execute()

Task 3: IMPLEMENT src/commands/list.rs - execute() function
  - OPEN: JinRepo using JinRepo::open()
  - HANDLE: NotInitialized error specifically
  - ENUMERATE: All refs using references_glob("refs/jin/layers/**")
  - COLLECT: Parse ref paths into mode/scope/project sets
  - SORT: Convert sets to sorted vectors
  - DISPLAY: Categorized output (Modes, Scopes, Projects)
  - DISPLAY: Usage hints at end
  - RETURN: Ok(())

Task 4: IMPLEMENT src/commands/list.rs - parse_ref_path() helper
  - ACCEPT: ref_path string, mode/scope/project HashSets
  - VALIDATE: Path starts with "refs/jin/layers/"
  - PARSE: Split path by '/' and match patterns
  - UPDATE: Appropriate HashSet(s) based on pattern
  - IGNORE: Special layers (global, local, workspace)

Task 5: VERIFY src/commands/list.rs - unit tests (ALREADY EXISTS)
  - TEST: parse_ref_path with mode-only ref
  - TEST: parse_ref_path with mode/scope ref
  - TEST: parse_ref_path with mode/scope/project ref
  - TEST: parse_ref_path with project-only ref
  - TEST: parse_ref_path ignores global ref

Task 6: CREATE tests/cli_list.rs - comprehensive integration tests
  - TEST: list with empty repository
  - TEST: list after creating modes
  - TEST: list after creating scopes
  - TEST: list after creating projects
  - TEST: list with all categories
  - TEST: list shows items in alphabetical order
  - TEST: list not initialized error
  - FIXTURE: Use TestFixture from common/fixtures.rs
```

### Implementation Patterns & Key Details

```rust
// MAIN FUNCTION PATTERN
pub fn execute() -> Result<()> {
    // PATTERN: Open Jin repository (fail if not exists)
    let repo = match JinRepo::open() {
        Ok(r) => r,
        Err(_) => {
            return Err(JinError::NotInitialized);
        }
    };

    let git_repo = repo.inner();

    // PATTERN: Enumerate all layer refs
    let refs = match git_repo.references_glob("refs/jin/layers/**") {
        Ok(r) => r,
        Err(e) => {
            return Err(JinError::Git(e));
        }
    };

    // PATTERN: Collect names into HashSets for deduplication
    let mut modes = HashSet::new();
    let mut scopes = HashSet::new();
    let mut projects = HashSet::new();

    for ref_result in refs {
        let reference = ref_result?;
        if let Some(name) = reference.name() {
            parse_ref_path(name, &mut modes, &mut scopes, &mut projects);
        }
    }

    // PATTERN: Display results
    println!("Available in Jin repository:");
    println!();

    let has_modes = !modes.is_empty();
    let has_scopes = !scopes.is_empty();
    let has_projects = !projects.is_empty();

    // PATTERN: Show each non-empty category
    if has_modes {
        println!("Modes:");
        let mut mode_list: Vec<_> = modes.into_iter().collect();
        mode_list.sort();
        for mode in mode_list {
            println!("  - {}", mode);
        }
        println!();
    }

    if has_scopes {
        println!("Scopes:");
        let mut scope_list: Vec<_> = scopes.into_iter().collect();
        scope_list.sort();
        for scope in scope_list {
            println!("  - {}", scope);
        }
        println!();
    }

    if has_projects {
        println!("Projects:");
        let mut project_list: Vec<_> = projects.into_iter().collect();
        project_list.sort();
        for project in project_list {
            println!("  - {}", project);
        }
        println!();
    }

    // PATTERN: Handle all-empty case
    if !has_modes && !has_scopes && !has_projects {
        println!("  (no modes, scopes, or projects found)");
        println!();
    }

    // PATTERN: Show usage hints
    println!("Use 'jin mode use <mode>' to activate a mode");
    println!("Use 'jin scope use <scope>' to activate a scope");

    Ok(())
}

// HELPER FUNCTION PATTERN
fn parse_ref_path(
    ref_path: &str,
    modes: &mut HashSet<String>,
    scopes: &mut HashSet<String>,
    projects: &mut HashSet<String>,
) {
    // PATTERN: Validate prefix
    if !ref_path.starts_with("refs/jin/layers/") {
        return;
    }

    let path = &ref_path["refs/jin/layers/".len()..];
    let parts: Vec<&str> = path.split('/').collect();

    // PATTERN: Match against known ref path patterns
    match parts.as_slice() {
        ["mode", mode] => {
            modes.insert(mode.to_string());
        }
        ["mode", mode, "scope", scope] => {
            modes.insert(mode.to_string());
            scopes.insert(scope.to_string());
        }
        ["mode", mode, "scope", scope, "project", project] => {
            modes.insert(mode.to_string());
            scopes.insert(scope.to_string());
            projects.insert(project.to_string());
        }
        ["mode", mode, "project", project] => {
            modes.insert(mode.to_string());
            projects.insert(project.to_string());
        }
        ["scope", scope] => {
            scopes.insert(scope.to_string());
        }
        ["project", project] => {
            projects.insert(project.to_string());
        }
        _ => {
            // Ignore global, local, workspace, and unknown patterns
        }
    }
}

// TEST PATTERN
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ref_path() {
        let mut modes = HashSet::new();
        let mut scopes = HashSet::new();
        let mut projects = HashSet::new();

        parse_ref_path(
            "refs/jin/layers/mode/claude",
            &mut modes,
            &mut scopes,
            &mut projects,
        );
        assert!(modes.contains("claude"));
        assert!(scopes.is_empty());
        assert!(projects.is_empty());
    }
}
```

### Integration Points

```yaml
CLI_ENUM:
  - location: src/cli/mod.rs
  - lines: 77-78
  - pattern: |
      #[derive(Subcommand, Debug)]
      pub enum Commands {
          /// List available modes/scopes/projects
          List,
          // ... other commands
      }

COMMAND_ROUTING:
  - location: src/commands/mod.rs
  - line: 49
  - pattern: |
      match cli.command {
          // ... other commands
          Commands::List => list::execute(),
          // ... other commands
      }

GIT_REFS:
  - location: ~/.jin/ (bare Git repository)
  - pattern: refs under refs/jin/layers/
  - methods: references_glob(), find_reference()

MODE_SCOPE_STORAGE:
  - location: refs/jin/modes/{name}/_mode
  - location: refs/jin/scopes/{name}
  - location: refs/jin/modes/{mode}/scopes/{scope}

LAYER_REFS:
  - location: refs/jin/layers/mode/{mode}
  - location: refs/jin/layers/scope/{scope}
  - location: refs/jin/layers/project/{project}
  - location: refs/jin/layers/mode/{mode}/scope/{scope}
  - location: refs/jin/layers/mode/{mode}/scope/{scope}/project/{project}
  - location: refs/jin/layers/mode/{mode}/project/{project}
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating/modifying list.rs
cargo check --bin jin                # Check for compilation errors
cargo clippy --bin jin -- -D warnings  # Lint checking

# Format check
cargo fmt --check                    # Verify formatting
cargo fmt                            # Auto-format if needed

# Expected: Zero errors, zero warnings. If errors exist, READ output and fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run list command unit tests
cargo test list                      # Run all list tests
cargo test list::tests::test_parse_ref_path  # Run specific test
cargo test list::tests::test_execute_not_initialized  # Test error case

# Run all command tests
cargo test --bin jin commands

# Expected: All tests pass. If failing, debug root cause and fix.
```

### Level 3: Integration Testing (System Validation)

```bash
# Set up isolated test environment
export JIN_DIR=/tmp/jin-list-test-$$
mkdir -p $JIN_DIR

# Initialize Jin repository
jin init

# Test empty repository
jin list
# Expected: Shows "(no modes, scopes, or projects found)"

# Create a mode
jin mode create development
jin list
# Expected: Shows "Modes:" section with "- development"

# Create more modes
jin mode create staging
jin mode create production
jin list
# Expected: Shows all three modes in alphabetical order

# Create scopes
jin scope create "language:javascript"
jin scope create "language:rust"
jin list
# Expected: Shows both Scopes and Modes sections

# Test not initialized error
cd /tmp
mkdir not-a-jin-project && cd not-a-jin-project
jin list
# Expected: Error "Jin not initialized"

# Test with mode/scope refs directly (advanced)
# Manually create layer refs to test parsing
cd $JIN_DIR
git update-ref refs/jin/layers/mode/zed HEAD
git update-ref refs/jin/layers/scope/team HEAD
git update-ref refs/jin/layers/project/api-server HEAD
jin list
# Expected: Shows all three categories

# Cleanup
rm -rf /tmp/jin-list-test-*

# Expected: All scenarios work correctly, proper output formatting, appropriate errors.
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test with complex scope names (colons, special characters)
jin scope create "env:prod:us-east"
jin scope create "category:ui:theme:dark"
jin list
# Expected: Scopes displayed with colons preserved

# Test alphabetical sorting with varied names
jin mode create zulu
jin mode create alpha
jin mode create bravo
jin mode create charlie
jin list
# Expected: alpha, bravo, charlie, zulu (not insertion order)

# Test with mode-bound scopes
jin mode create editor
jin scope create "config:vim" --mode editor
jin list
# Expected: Both editor mode and config:vim scope shown

# Test deduplication (same name in multiple ref paths)
# Create refs that would cause duplicates:
# refs/jin/layers/mode/dev
# refs/jin/layers/mode/dev/scope/team
# refs/jin/layers/mode/dev/project/api
# Should show "dev" only once in Modes section

# Test with UTF-8 names
jin mode create "日本語"
jin scope create "catégorie:français"
jin list
# Expected: UTF-8 characters display correctly

# Performance test with many refs
for i in {1..100}; do jin mode create "mode-$i"; done
jin list
# Should complete quickly (< 1 second for 100 items)

# Integration with mode/scope commands
# Verify list output matches what mode/scope list show
jin mode list
jin scope list
jin list
# Should show consistent information

# Expected: All edge cases handled gracefully, performance acceptable, integration consistent.
```

## Final Validation Checklist

### Technical Validation

- [ ] Command compiles without errors: `cargo check --bin jin`
- [ ] No clippy warnings: `cargo clippy --bin jin -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`
- [ ] All unit tests pass: `cargo test list`
- [ ] No new dependencies added (uses existing)

### Feature Validation

- [ ] Lists all modes from layer refs
- [ ] Lists all scopes from layer refs
- [ ] Lists all projects from layer refs
- [ ] Items sorted alphabetically within each category
- [ ] Empty categories hidden from output
- [ ] Shows helpful message when all categories empty
- [ ] Usage hints displayed at end
- [ ] Returns NotInitialized error when appropriate
- [ ] Handles UTF-8 names correctly
- [ ] Deduplicates items appearing in multiple ref paths

### Code Quality Validation

- [ ] Follows existing command patterns (layers, context)
- [ ] Error handling consistent with other commands
- [ ] No unwrap() calls that could panic
- [ ] Uses JinError::NotInitialized correctly
- [ ] Unit tests cover all ref path patterns
- [ ] Helper function (parse_ref_path) tested
- [ ] Integration tests cover end-to-end scenarios

### Documentation & Deployment

- [ ] Function has doc comment: `/// Execute the list command`
- [ ] Helper function has doc comment
- [ ] Tests have descriptive names
- [ ] Code is self-documenting with clear variable names

---

## Anti-Patterns to Avoid

- **Don't** use unwrap() on Git operations - use ? propagation or match
- **Don't** forget to handle NotInitialized error specifically
- **Don't** hardcode ref path patterns - make them extensible
- **Don't** show empty categories (hide if HashSet is empty)
- **Don't** forget to sort output (use Vec sort())
- **Don't** skip deduplication (use HashSet for collection)
- **Don't** create new patterns when existing commands show the way
- **Don't** add table formatting dependencies without consensus
- **Don't** assume all refs are valid - handle unexpected patterns gracefully
- **Don't** forget the trailing println!() for section spacing

---

## Confidence Score

**9/10** - Very High confidence for one-pass implementation success

**Reasoning**:
- (+) Command is already fully implemented and unit tested
- (+) All context and patterns are well-documented in this PRP
- (+) Follows established codebase patterns consistently
- (+) No external dependencies or new patterns required
- (+) Validation procedures are specific and executable
- (+) External research provides industry best practices
- (+) Unit tests already cover parsing logic comprehensively
- (-) Could benefit from comprehensive integration tests (optional enhancement)

**Validation**: The completed PRP enables an AI agent (or human developer) to understand, maintain, and enhance the `jin list` command implementation using only this PRP content and codebase access.

---

## Appendix: External Research Summary

### CLI List Command Best Practices Researched

**Industry Standards Analyzed**:
1. **kubectl get** - Categorized resource listing with table/JSON formats
2. **git branch** - Simple list with current item indicator
3. **docker ps/images** - Template-based formatting options
4. **AWS CLI** - Multiple output formats (table, JSON, text)
5. **GitHub CLI (gh)** - JSON/template outputs with jq filtering

**Best Practices Applied to Jin List Command**:
- **Categorization**: Separate sections for Modes, Scopes, Projects (like kubectl)
- **Alphabetical Sorting**: Consistent with git branch and other CLIs
- **Empty State Handling**: Clear message like kubectl's "No resources found"
- **Deduplication**: Use HashSet to avoid showing duplicates (same name from multiple refs)
- **Usage Hints**: Show next steps after listing (common in modern CLIs)
- **Clean Format**: Simple text output without table dependencies (appropriate for simple list)

**Potential Future Enhancements** (not in current scope):
- `--json` output format for scripting
- `--filter type=mode` to show only one category
- `--sort modified` to show by last modified time
- `--verbose` to show additional metadata
