# Product Requirement Prompt (PRP): P7.M3.T1 - Add Modes and Scopes Aliases

## Goal

**Feature Goal**: Add top-level commands `jin modes` and `jin scopes` as convenient aliases for `jin mode list` and `jin scope list`.

**Deliverable**: Two new top-level CLI commands that invoke the existing mode and scope list functionality.

**Success Definition**:
- `jin modes` produces identical output to `jin mode list`
- `jin scopes` produces identical output to `jin scope list`
- Both commands appear in help output
- All tests pass

## User Persona

**Target User**: Jin CLI users who frequently list modes and scopes.

**Use Case**: Users often want to quickly see available modes/scopes without typing the full `mode list` or `scope list` commands.

**User Journey**:
1. User types `jin modes` (instead of `jin mode list`)
2. User sees list of available modes with active mode marked
3. User can use `jin scopes` similarly for scopes

**Pain Points Addressed**:
- Reduces typing for common operations
- More intuitive: "modes" (plural) for listing all modes
- Consistent with common CLI patterns (e.g., `git branches`)

## Why

- **User convenience**: Listing modes/scopes is a frequent operation
- **CLI discoverability**: Plural form is natural for "list all" operations
- **Consistency**: Many CLIs use plural nouns for listing (git branches, docker images)
- **Minimal implementation**: Reuses existing tested code

## What

Add two new top-level commands to the CLI:

```bash
# New commands (aliases)
jin modes      # Equivalent to: jin mode list
jin scopes     # Equivalent to: jin scope list

# Existing commands (unchanged)
jin mode list  # Still works
jin scope list # Still works
```

### Success Criteria

- [ ] `jin modes` command executes and shows mode list
- [ ] `jin scopes` command executes and shows scope list
- [ ] Output is identical to `jin mode list` and `jin scope list`
- [ ] Commands appear in `jin --help` output
- [ ] All existing tests pass
- [ ] New tests added for alias commands

## All Needed Context

### Context Completeness Check

This PRP passes the "No Prior Knowledge" test - all required files, patterns, and implementation details are specified.

### Documentation & References

```yaml
# MUST READ - clap derive API for top-level commands
- url: https://docs.rs/clap/latest/clap/_derive/index.html
  why: Understanding clap derive API for adding new command variants
  critical: Use #[command(alias)] on top-level Commands enum variants

- file: src/cli/mod.rs
  why: Main CLI definition with Commands enum - where new variants are added
  pattern: Add new Commands enum variants for Modes and Scopes
  gotcha: Commands must be added to enum, not as separate structs

- file: src/commands/mode.rs (lines 123-163)
  why: Contains the list() function that Modes command will call
  pattern: pub fn list() -> Result<()> - function signature to match
  gotcha: Function is not pub - need to make it pub for external access

- file: src/commands/scope.rs (lines 190-255)
  why: Contains the list() function that Scopes command will call
  pattern: pub fn list() -> Result<()> - function signature to match
  gotcha: Function is not pub - need to make it pub for external access

- file: src/commands/mod.rs
  why: Command dispatcher - where new commands are wired
  pattern: Match arm pattern for Commands enum variants
  gotcha: Must import list functions from mode/scope modules

- file: plan/P7M3T1/research/clap_alias_research.md
  why: Research on clap alias patterns and alternatives
  section: Option 1 - Subcommand Aliases (not suitable for top-level)
  critical: Explains why visible_alias doesn't work for this use case

- docfile: plan/P7M3T1/research/cli_alias_patterns.md
  why: Examples from git, cargo, npm showing similar patterns
  section: Section 6 - Recommended Patterns for Jin
  critical: Shows industry standard patterns for list aliases
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs          # Shared argument types (not needed for this task)
│   └── mod.rs           # MODIFY: Add Modes/Scopes to Commands enum
├── commands/
│   ├── mod.rs           # MODIFY: Add dispatcher arms for Modes/Scopes
│   ├── mode.rs          # MODIFY: Make list() function pub
│   └── scope.rs         # MODIFY: Make list() function pub
├── main.rs
└── lib.rs
```

### Desired Codebase Tree with Files to be Added

```bash
# No new files - modifications to existing files only

src/
├── cli/
│   └── mod.rs           # MODIFIED: Modes/Scopes variants added to Commands enum
├── commands/
│   ├── mod.rs           # MODIFIED: Commands::Modes and Commands::Scopes match arms
│   ├── mode.rs          # MODIFIED: list() function made pub
│   └── scope.rs         # MODIFIED: list() function made pub
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: list() functions in mode.rs and scope.rs are currently private (no pub)
// Must add `pub` keyword to make them accessible from commands::mod.rs

// CLAP LIMITATION: clap's visible_alias only works for subcommands at the same level
// It CANNOT alias a top-level command to a subcommand (e.g., "modes" -> "mode list")
// The only solution is to create separate top-level Commands enum variants

// PATTERN: Command dispatcher uses match on cli.command
// Must add new match arms that call mode::list() and scope::list()

// GOTCHA: ModeAction::List and ScopeAction::List already exist
// Don't confuse these with the new Commands::Modes and Commands::Scopes variants
// Commands::Modes is a top-level command, ModeAction::List is a subcommand
```

## Implementation Blueprint

### Data Models and Structure

No new data models required. This task reuses existing functions.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/commands/mode.rs - Make list() function public
  - ADD: pub keyword to list() function definition (line ~123)
  - BEFORE: fn list() -> Result<()>
  - AFTER: pub fn list() -> Result<()>
  - REASON: Commands dispatcher needs access to this function
  - FILE: src/commands/mode.rs

Task 2: MODIFY src/commands/scope.rs - Make list() function public
  - ADD: pub keyword to list() function definition (line ~190)
  - BEFORE: fn list() -> Result<()>
  - AFTER: pub fn list() -> Result<()>
  - REASON: Commands dispatcher needs access to this function
  - FILE: src/commands/scope.rs

Task 3: MODIFY src/cli/mod.rs - Add Modes and Scopes variants to Commands enum
  - ADD: Commands::Modes variant (after Mode command, around line 44)
  - ADD: Commands::Scopes variant (after Scope command, around line 48)
  - PATTERN: Follow existing command pattern with doc comment
  - PLACEMENT: src/cli/mod.rs, Commands enum
  - EXAMPLE:
    ```rust
    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),

    /// List available modes (alias for `jin mode list`)
    Modes,

    /// Scope lifecycle management
    #[command(subcommand)]
    Scope(ScopeAction),

    /// List available scopes (alias for `jin scope list`)
    Scopes,
    ```

Task 4: MODIFY src/commands/mod.rs - Add dispatcher arms
  - ADD: Commands::Modes => mode::list() match arm
  - ADD: Commands::Scopes => scope::list() match arm
  - PATTERN: Follow existing pattern (e.g., Commands::Status => status::execute())
  - PLACEMENT: src/commands/mod.rs, execute() function
  - EXAMPLE:
    ```rust
    Commands::Modes => mode::list(),
    Commands::Scopes => scope::list(),
    ```

Task 5: CREATE tests for new alias commands
  - ADD: Integration test for jin modes command
  - ADD: Integration test for jin scopes command
  - FOLLOW: Existing test pattern in tests/ directory
  - VERIFY: Output matches jin mode list and jin scope list
  - FILE: tests/cli_aliases.rs (or add to existing test file)
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Public function export (src/commands/mode.rs, line ~123)
// BEFORE:
fn list() -> Result<()> {

// AFTER:
pub fn list() -> Result<()> {

// Pattern 2: Commands enum variant (src/cli/mod.rs)
/// List available modes (alias for `jin mode list`)
Modes,

// CRITICAL: No arguments needed - just calls the list() function directly

// Pattern 3: Dispatcher match arm (src/commands/mod.rs)
Commands::Modes => mode::list(),
Commands::Scopes => scope::list(),

// GOTCHA: Don't call execute() - call list() directly
// execute() is for subcommands (ModeAction), Modes is a direct command
```

### Integration Points

```yaml
CLI_ENUM:
  - file: src/cli/mod.rs
  - add: Commands::Modes variant
  - add: Commands::Scopes variant
  - location: In Commands enum, after Mode and Scope variants

COMMAND_DISPATCHER:
  - file: src/commands/mod.rs
  - add: Commands::Modes => mode::list()
  - add: Commands::Scopes => scope::list()
  - location: In execute() function match statement

MODULE_EXPORTS:
  - file: src/commands/mode.rs
  - modify: pub fn list() -> Result<()>

  - file: src/commands/scope.rs
  - modify: pub fn list() -> Result<()>
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --package jin                # Verify compilation
cargo clippy --package jin -- -D warnings  # Lint with warnings as errors
cargo fmt --all -- --check               # Check formatting

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test CLI parsing
cargo test --package jin cli::tests -- --nocapture

# Test mode/scope modules
cargo test --package jin commands::mode -- --nocapture
cargo test --package jin commands::scope -- --nocapture

# Expected: All tests pass. Existing mode/scope tests should still work.
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual testing - build first
cargo build --release

# Test modes command
./target/release/jin modes
# Expected: Same output as: ./target/release/jin mode list

# Test scopes command
./target/release/jin scopes
# Expected: Same output as: ./target/release/jin scope list

# Test help output includes new commands
./target/release/jin --help | grep -E "(Modes|Scopes)"
# Expected: Both "Modes" and "Scopes" appear in help

# Test mode list still works
./target/release/jin mode list
# Expected: Original command still works

# Test scope list still works
./target/release/jin scope list
# Expected: Original command still works
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Test that both commands produce identical output
diff <(./target/release/jin mode list) <(./target/release/jin modes)
# Expected: No diff (identical output)

diff <(./target/release/jin scope list) <(./target/release/jin scopes)
# Expected: No diff (identical output)

# Test with existing modes/scopes
./target/release/jin mode create test-mode
./target/release/jin modes | grep test-mode
# Expected: test-mode appears in output

./target/release/jin scope create test-scope
./target/release/jin scopes | grep test-scope
# Expected: test-scope appears in output

# Cleanup
./target/release/jin mode delete test-mode
./target/release/jin scope delete test-scope

# Run full test suite
cargo test --package jin
# Expected: All tests pass
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo check --package jin`
- [ ] No clippy warnings: `cargo clippy --package jin`
- [ ] Formatted correctly: `cargo fmt --all -- --check`
- [ ] All tests pass: `cargo test --package jin`
- [ ] list() functions are public in both mode.rs and scope.rs

### Feature Validation

- [ ] `jin modes` produces same output as `jin mode list`
- [ ] `jin scopes` produces same output as `jin scope list`
- [ ] Commands appear in `jin --help` output
- [ ] Original `jin mode list` still works
- [ ] Original `jin scope list` still works
- [ ] Error handling works for uninitialized Jin repo

### Code Quality Validation

- [ ] Follows existing code patterns
- [ ] No code duplication
- [ ] Proper doc comments on new Commands enum variants
- [ ] Public functions properly documented
- [ ] No breaking changes to existing functionality

### Documentation & Deployment

- [ ] Commands are self-documenting via clap help
- [ ] Implementation is minimal and clean
- [ ] No new dependencies added

---

## Anti-Patterns to Avoid

- ❌ Don't create new command modules (modes.rs, scopes.rs) - reuse existing list() functions
- ❌ Don't duplicate the list() logic - call the existing functions
- ❌ Don't use clap's visible_alias on ModeAction/ScopeAction - that won't create top-level commands
- ❌ Don't forget to make list() functions pub - dispatcher won't be able to call them
- ❌ Don't add unnecessary Args structs - these commands take no arguments
- ❌ Don't modify the ModeAction or ScopeAction enums - those are for subcommands only
- ❌ Don't break existing `jin mode list` and `jin scope list` commands
- ❌ Don't add complex routing logic - direct function calls only

---

## Confidence Score

**10/10** - One-pass implementation success likelihood

**Rationale**:
1. Minimal changes required (4 files, ~10 lines of code)
2. No new dependencies or complex logic
3. Reuses existing, well-tested functions
4. Pattern is clear and consistent across codebase
5. All file locations and function signatures specified
6. Validation steps are concrete and verifiable
7. No architectural changes or refactoring needed

---

## Quick Reference Implementation Summary

```bash
# Files to modify:
1. src/commands/mode.rs     - Add `pub` to list() function
2. src/commands/scope.rs    - Add `pub` to list() function
3. src/cli/mod.rs           - Add Modes and Scopes to Commands enum
4. src/commands/mod.rs      - Add match arms for Modes and Scopes

# Lines of code: ~10
# New files: 0
# Risk level: Low
# Testing required: Integration tests for new commands
```
