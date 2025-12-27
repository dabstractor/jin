# PRP: P4.M3 - Mode & Scope Commands

## Goal

**Feature Goal**: Implement complete lifecycle management for modes and scopes in Jin, enabling users to create, activate, list, delete, and inspect mode/scope contexts.

**Deliverable**: Fully functional `jin mode` and `jin scope` subcommands with 12 operations total (6 for modes, 6 for scopes) that integrate seamlessly with existing commands (add, commit, status, apply).

**Success Definition**:
- All mode/scope lifecycle commands work correctly (create, use, list, delete, show, unset)
- Active mode/scope contexts persist across sessions in `.jin/context`
- Mode/scope changes are atomic and crash-safe
- Integration tests pass for all workflows
- Users can manage modes and scopes without understanding Git internals

## User Persona (if applicable)

**Target User**: Developers using Jin to manage tool-specific configuration files across multiple editors, AI assistants, and environments.

**Use Case**: Developer needs to switch between different tool contexts (e.g., Claude AI, VS Code, Zed editor) and domain contexts (e.g., JavaScript projects, Python projects, infrastructure work) without manually managing configuration files.

**User Journey**:
1. Initialize Jin in project: `jin init`
2. Create mode for their primary tool: `jin mode create claude`
3. Activate the mode: `jin mode use claude`
4. Stage tool-specific files: `jin add .claude/ --mode`
5. Create scope for language context: `jin scope create language:javascript`
6. Activate scope: `jin scope use language:javascript`
7. Stage language-specific configs: `jin add webpack.config.js --mode --scope`
8. Switch contexts easily: `jin mode use zed`, `jin scope use language:python`
9. List available contexts: `jin mode list`, `jin scope list`
10. Inspect current state: `jin mode show`, `jin status`

**Pain Points Addressed**:
- **Manual context switching**: No more manually editing config files when switching tools
- **Configuration drift**: Explicit modes/scopes prevent accidentally using wrong configs
- **Discovery**: Easy to see what modes/scopes are available
- **State confusion**: Clear display of active contexts eliminates "which context am I in?" questions

## Why

- **Completes P4.M2 foundation**: Core commands (init, add, commit, status) work but lack context management
- **Enables full Jin workflow**: Without mode/scope management, users can't leverage Jin's multi-layer architecture
- **User-facing abstraction**: Modes/scopes hide Git complexity (refs, layers) behind intuitive commands
- **Critical for P4.M4+**: Future milestones (apply, merge, sync) depend on active context management

### Integration with Existing Features

- **P3.M1 Staging System**: Already handles `--mode` and `--scope` flags, waiting for lifecycle management
- **P3.M2 Commit Pipeline**: Commits to mode/scope layers work, need way to create/manage those layers
- **P4.M2 Status Command**: Shows active mode/scope, needs commands to change them
- **P1.M3 Transaction System**: Provides atomic guarantees for mode/scope activation

### Problems This Solves

- **Layer Creation**: No way to create mode/scope refs currently (they don't exist)
- **Context Activation**: No way to set active mode/scope in `.jin/context`
- **Discovery**: No way to list available modes/scopes
- **State Inspection**: No clear way to see current context beyond `jin status`
- **Lifecycle Management**: No way to delete unused modes/scopes

## What

This milestone implements 12 subcommands for comprehensive mode and scope lifecycle management:

### Mode Commands (6 operations)

1. **`jin mode create <name>`** - Create new mode with metadata
2. **`jin mode use <name>`** - Activate mode in project context
3. **`jin mode list`** - List all available modes with active indicator
4. **`jin mode delete <name>`** - Remove mode and associated layers
5. **`jin mode show`** - Display currently active mode
6. **`jin mode unset`** - Deactivate current mode

### Scope Commands (6 operations)

1. **`jin scope create <name> [--mode=<mode>]`** - Create new scope (mode-bound or untethered)
2. **`jin scope use <name>`** - Activate scope in project context
3. **`jin scope list`** - List all available scopes with mode bindings
4. **`jin scope delete <name>`** - Remove scope and associated layers
5. **`jin scope show`** - Display currently active scope
6. **`jin scope unset`** - Deactivate current scope

### Success Criteria

**Functional Completeness:**
- [ ] All 12 subcommands execute without errors
- [ ] Mode/scope names validate correctly (alphanumeric + underscore for modes, allow colons for scopes)
- [ ] Active context persists in `.jin/context` YAML file
- [ ] Mode/scope metadata stored in Git refs
- [ ] Context changes are atomic (use TransactionSystem)
- [ ] Duplicate create operations return helpful errors
- [ ] Delete operations handle active context gracefully (unset first)
- [ ] List operations show clear, formatted output with active indicators

**Integration Validation:**
- [ ] `jin add --mode` works after `jin mode create` + `jin mode use`
- [ ] `jin add --scope=<name>` works after `jin scope create` + `jin scope use`
- [ ] `jin status` displays active mode and scope correctly
- [ ] `jin commit` works with files staged to mode/scope layers
- [ ] Mode/scope activation survives `jin` command restarts

**Error Handling:**
- [ ] Using non-existent mode/scope shows helpful error with suggestion to create
- [ ] Creating duplicate mode/scope returns clear error
- [ ] Using `--mode` without active mode shows actionable error
- [ ] Deleting active mode/scope unsets it first or prompts user
- [ ] Invalid mode/scope names rejected with format guidance

**User Experience:**
- [ ] Help text is clear and includes examples
- [ ] Output is formatted and readable (not just debug dumps)
- [ ] Commands suggest next logical steps
- [ ] Error messages are actionable, not technical

## All Needed Context

### Context Completeness Check

✅ **Validation Complete:** This PRP provides:
- Complete understanding of mode/scope concepts and data model
- Exact file locations and patterns from existing implementations
- Specific routing logic and validation rules already in codebase
- Comprehensive error handling patterns from add command
- Testing patterns from existing test suites
- CLI best practices from external research

An AI agent with no prior knowledge of this codebase can successfully implement this feature using only:
- The information in this PRP
- Access to the specified files in the codebase
- The linked external documentation

### Documentation & References

```yaml
# MUST READ - Core Type Definitions
- file: src/core/config.rs
  why: ProjectContext struct holds active mode/scope, provides load/save/require_mode/require_scope methods
  pattern: See how ProjectContext.mode and .scope are Option<String>, how save() serializes to YAML
  gotcha: require_mode() and require_scope() return JinError::NoActiveContext if not set - use these for validation
  critical: The save() method writes to .jin/context - this is the ONLY place context should be persisted

- file: src/core/layer.rs
  why: Layer enum defines 9-layer hierarchy, ref_path() generates Git refs for mode/scope layers
  pattern: Layer::ModeBase.ref_path(Some("claude"), None, None) → "refs/jin/layers/mode/claude"
  gotcha: Layers have requires_mode() and requires_scope() predicates - respect these
  critical: ref_path() expects mode/scope as Option<&str> - None means not applicable for that layer

- file: src/core/error.rs
  why: JinError enum with thiserror derive, see NoActiveContext variant for mode/scope errors
  pattern: Return Err(JinError::NoActiveContext { context_type: "mode".to_string() }) when mode required but not set
  gotcha: Use thiserror error types consistently, don't create ad-hoc error strings
  critical: Error messages should be user-friendly and actionable (see GitTracked variant for example)

# MUST READ - Routing & Validation
- file: src/staging/router.rs
  why: Shows how mode/scope flags are validated and converted to target layers
  pattern: validate_routing_options() checks flag consistency, route_to_layer() maps to Layer enum
  gotcha: --project requires --mode flag, --global conflicts with other flags
  critical: Context validation BEFORE operations - validate_routing_options pattern is the model

# MUST READ - Reference Implementation
- file: src/commands/add.rs
  why: Complete command implementation with ProjectContext loading, validation, error handling, tests
  pattern: Lines 41-47 show ProjectContext loading with NotInitialized handling, lines 125-126 show error reporting
  gotcha: Load context early, handle NotInitialized specially, fall back to default for other errors
  critical: Lines 206-325 show testing patterns - create TempDir, test functions, validate errors with matches!

# MUST READ - Git Repository Operations
- file: src/git/repo.rs
  why: JinRepo provides Git operations - create_blob, create_tree, create_commit, set_ref, resolve_ref
  pattern: repo.set_ref("refs/jin/layers/mode/mymode", commit_oid, "create mode") updates Git ref
  gotcha: set_ref() requires commit OID - must create commit first (even empty one for initialization)
  critical: Use JinRepo::open_or_create() to get repo instance, handle NotInitialized error

# MUST READ - Transaction System
- file: src/git/transaction.rs
  why: LayerTransaction provides atomic multi-ref updates with crash recovery
  pattern: Lines 850-873 show setup pattern, LayerUpdate tracks ref changes, commit() applies atomically
  gotcha: Use transactions for mode/scope activation to ensure atomicity
  critical: Transactions MUST be committed or rolled back - no partial state

# Current Command Stubs
- file: src/commands/mode.rs
  why: Contains execute() dispatcher and stub functions for all mode operations
  pattern: execute() matches on ModeAction enum and calls private functions (create, use_mode, list, etc.)
  gotcha: Functions already exist but all are TODO stubs - replace placeholder println! with real logic
  critical: Keep execute() structure intact, implement private functions

- file: src/commands/scope.rs
  why: Contains execute() dispatcher and stub functions for all scope operations
  pattern: Same structure as mode.rs, create() takes optional mode parameter for mode-bound scopes
  gotcha: Scope names can contain colons (e.g., "language:javascript"), mode names cannot
  critical: Handle mode-bound vs untethered scope creation differently

# CLI Argument Definitions
- file: src/cli/mod.rs
  why: ModeAction and ScopeAction enums define subcommand structure with clap derive
  pattern: Lines 95-149 show complete enum definitions with field names
  gotcha: Mode uses 'Use' variant (keyword conflict), scope has optional --mode flag in Create variant
  critical: These enums are already wired to commands dispatcher - don't modify structure

# Testing Reference
- file: tests/cli_basic.rs
  why: Integration tests using assert_cmd and predicates crates
  pattern: jin() helper function gets Command, .args() passes arguments, .assert() validates output
  gotcha: Tests run actual binary, not library functions - full E2E validation
  critical: Use predicate::str::contains() for output assertions, test both success and failure cases

# External Documentation
- url: https://clig.dev/
  why: CLI design guidelines for user-friendly commands
  section: "Basics" and "Interactivity"
  critical: Error messages should be actionable, include suggestions, use plain language

- url: https://docs.rs/clap/latest/clap/_faq/index.html
  why: clap FAQ for CLI argument parsing patterns
  section: "Subcommands"
  critical: Jin already uses derive API correctly - maintain existing patterns

- url: https://rust-cli.github.io/book/in-depth/config-files.html
  why: Rust CLI configuration file patterns
  section: "Choosing a format" and "Where to store configuration"
  critical: Jin uses YAML for .jin/context (serde_yaml) - maintain consistency
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs              # AddArgs, CommitArgs, etc. (146 lines)
│   └── mod.rs               # Cli, Commands, ModeAction, ScopeAction (149 lines)
├── commands/
│   ├── add.rs               # Reference implementation (326 lines)
│   ├── commit_cmd.rs        # Commit command (working)
│   ├── context.rs           # Context display (stub)
│   ├── mode.rs              # Mode commands (53 lines - ALL STUBS)
│   ├── scope.rs             # Scope commands (59 lines - ALL STUBS)
│   ├── status.rs            # Status display (working)
│   └── mod.rs               # Command dispatcher (56 lines)
├── core/
│   ├── config.rs            # ProjectContext (231 lines - COMPLETE)
│   ├── error.rs             # JinError enum (145 lines - COMPLETE)
│   ├── layer.rs             # Layer enum, ref_path (284 lines - COMPLETE)
│   └── mod.rs               # Module exports
├── git/
│   ├── repo.rs              # JinRepo operations (COMPLETE)
│   ├── transaction.rs       # LayerTransaction (COMPLETE)
│   ├── refs.rs              # Reference operations (COMPLETE)
│   └── ...
├── staging/
│   ├── router.rs            # Routing logic (213 lines - COMPLETE)
│   ├── index.rs             # StagingIndex (209 lines - COMPLETE)
│   └── ...
└── lib.rs                   # Public API

tests/
└── cli_basic.rs             # Integration tests (319 lines)
```

### Desired Codebase Tree (After P4.M3)

```bash
src/
├── commands/
│   ├── mode.rs              # UPDATED: 200-250 lines (was 53)
│   │   # Public: execute(action: ModeAction) -> Result<()>
│   │   # Private: create(name: &str) -> Result<()>
│   │   # Private: use_mode(name: &str) -> Result<()>
│   │   # Private: list() -> Result<()>
│   │   # Private: delete(name: &str) -> Result<()>
│   │   # Private: show() -> Result<()>
│   │   # Private: unset() -> Result<()>
│   │   # Tests: #[cfg(test)] mod tests { ... }
│   │
│   ├── scope.rs             # UPDATED: 250-300 lines (was 59)
│   │   # Public: execute(action: ScopeAction) -> Result<()>
│   │   # Private: create(name: &str, mode: Option<&str>) -> Result<()>
│   │   # Private: use_scope(name: &str) -> Result<()>
│   │   # Private: list() -> Result<()>
│   │   # Private: delete(name: &str) -> Result<()>
│   │   # Private: show() -> Result<()>
│   │   # Private: unset() -> Result<()>
│   │   # Tests: #[cfg(test)] mod tests { ... }
│   │
│   └── context.rs           # UPDATED: 50-75 lines (was 13)
│       # Implements: `jin context` command
│       # Shows: active mode, scope, project, last updated

tests/
└── cli_basic.rs             # UPDATED: Add mode/scope tests (50+ new tests)
    # Test sections:
    # - Mode lifecycle tests (create, use, unset, delete)
    # - Scope lifecycle tests (create, use, unset, delete)
    # - Mode + scope integration tests
    # - Error condition tests (duplicate create, invalid names, etc.)
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: ProjectContext requires Jin initialization
// Pattern from src/commands/add.rs:41-47
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => {
        return Err(JinError::NotInitialized);
    }
    Err(_) => ProjectContext::default(),
};

// CRITICAL: Mode/scope names must be validated
// Modes: alphanumeric + underscore only (no colons, slashes, dots)
// Scopes: alphanumeric + underscore + colons (e.g., "language:javascript")
fn validate_mode_name(name: &str) -> Result<()> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(JinError::Other(format!(
            "Invalid mode name '{}'. Use alphanumeric and underscore only.", name
        )));
    }
    Ok(())
}

fn validate_scope_name(name: &str) -> Result<()> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') {
        return Err(JinError::Other(format!(
            "Invalid scope name '{}'. Use alphanumeric, underscore, and colon only.", name
        )));
    }
    Ok(())
}

// CRITICAL: Git refs need commits, even if empty
// Cannot create ref without commit OID
// Pattern: Create empty tree → create initial commit → set ref
let empty_tree = repo.create_tree(&[])?;
let commit_oid = repo.create_commit(
    None,
    &format!("Initialize mode: {}", name),
    empty_tree,
    &[]
)?;
repo.set_ref(&ref_path, commit_oid, &format!("create mode {}", name))?;

// CRITICAL: List operations must handle empty case gracefully
// Use repo.find_refs(pattern) to enumerate refs, handle case of zero matches
let mode_refs = repo.find_refs("refs/jin/modes/*")?;
if mode_refs.is_empty() {
    println!("No modes found. Create one with: jin mode create <name>");
    return Ok(());
}

// CRITICAL: Active mode/scope display must show clear indicator
// Pattern: Use asterisk (*) or [active] marker for current selection
for mode_name in modes {
    if Some(&mode_name) == context.mode.as_ref() {
        println!("* {} [active]", mode_name);
    } else {
        println!("  {}", mode_name);
    }
}

// GOTCHA: Scope can be mode-bound OR untethered
// Mode-bound: refs/jin/modes/<mode>/scopes/<scope>
// Untethered: refs/jin/scopes/<scope>
// When listing, show BOTH types and indicate which mode (if any)
```

## Implementation Blueprint

### Implementation Tasks (Dependency-Ordered)

```yaml
Task 1: IMPLEMENT Mode Creation (src/commands/mode.rs::create)
  - VALIDATE: Mode name using validate_mode_name() helper
  - CHECK: Mode doesn't already exist via repo.resolve_ref()
  - CREATE: Empty tree and initial commit for mode layer
  - SET: Git ref at refs/jin/modes/<name>
  - PRINT: "Created mode '<name>'. Activate with: jin mode use <name>"
  - FOLLOW pattern: src/commands/add.rs (context loading, error handling)
  - NAMING: Use lowercase for mode names, validate alphanumeric + underscore
  - PLACEMENT: Private function in src/commands/mode.rs after execute()
  - DEPENDENCIES: JinRepo for Git operations

Task 2: IMPLEMENT Mode Activation (src/commands/mode.rs::use_mode)
  - LOAD: ProjectContext from .jin/context
  - VALIDATE: Mode exists via repo.resolve_ref()
  - UPDATE: context.mode = Some(name.to_string())
  - SAVE: context.save() to persist change
  - PRINT: "Activated mode '<name>'. Stage files with: jin add --mode"
  - FOLLOW pattern: ProjectContext::load() and save() from src/core/config.rs
  - NAMING: use_mode() (not just "use" due to Rust keyword)
  - PLACEMENT: Private function after create()
  - DEPENDENCIES: Task 1 (mode must exist)

Task 3: IMPLEMENT Mode Listing (src/commands/mode.rs::list)
  - LOAD: ProjectContext to get active mode
  - ENUMERATE: All mode refs via repo.find_refs("refs/jin/modes/*")
  - PARSE: Extract mode names from ref paths
  - DISPLAY: Formatted list with active indicator (* or [active])
  - HANDLE: Empty case with helpful message
  - FOLLOW pattern: Git ref enumeration from src/git/refs.rs
  - NAMING: list() function
  - PLACEMENT: Private function after use_mode()
  - DEPENDENCIES: Task 1 (modes must exist to list)

Task 4: IMPLEMENT Mode Deletion (src/commands/mode.rs::delete)
  - VALIDATE: Mode exists via repo.resolve_ref()
  - CHECK: If mode is active, call unset() first or prompt user
  - DELETE: Git ref at refs/jin/modes/<name>
  - DELETE: Associated layer refs (mode-base, mode-scope, mode-project)
  - PRINT: "Deleted mode '<name>' and associated layers"
  - FOLLOW pattern: Ref deletion from src/git/refs.rs
  - NAMING: delete() function
  - PLACEMENT: Private function after list()
  - DEPENDENCIES: Task 2 (may need to unset), Task 6 (unset logic)

Task 5: IMPLEMENT Mode Show (src/commands/mode.rs::show)
  - LOAD: ProjectContext from .jin/context
  - CHECK: context.mode is Some vs None
  - DISPLAY: Current mode name or "No active mode"
  - FOLLOW pattern: Simple context access from ProjectContext
  - NAMING: show() function
  - PLACEMENT: Private function after delete()
  - DEPENDENCIES: None (just reads context)

Task 6: IMPLEMENT Mode Unset (src/commands/mode.rs::unset)
  - LOAD: ProjectContext from .jin/context
  - UPDATE: context.mode = None
  - SAVE: context.save() to persist
  - PRINT: "Deactivated mode. Mode layer no longer available for staging."
  - FOLLOW pattern: Context modification from Task 2
  - NAMING: unset() function
  - PLACEMENT: Private function after show()
  - DEPENDENCIES: None (just modifies context)

Task 7: IMPLEMENT Scope Creation (src/commands/scope.rs::create)
  - VALIDATE: Scope name using validate_scope_name() (allows colons)
  - VALIDATE: If mode parameter provided, ensure mode exists
  - CHECK: Scope doesn't already exist (check both mode-bound and untethered paths)
  - DETERMINE: Ref path based on mode parameter
    - Mode-bound: refs/jin/modes/<mode>/scopes/<scope>
    - Untethered: refs/jin/scopes/<scope>
  - CREATE: Empty tree and initial commit
  - SET: Git ref at determined path
  - PRINT: "Created scope '<scope>'" + mode binding info if applicable
  - FOLLOW pattern: Similar to Task 1 but with mode parameter handling
  - NAMING: create() function with mode: Option<&str> parameter
  - PLACEMENT: Private function in src/commands/scope.rs after execute()
  - DEPENDENCIES: JinRepo, mode validation if mode-bound

Task 8: IMPLEMENT Scope Activation (src/commands/scope.rs::use_scope)
  - LOAD: ProjectContext from .jin/context
  - VALIDATE: Scope exists (check both mode-bound and untethered)
  - UPDATE: context.scope = Some(name.to_string())
  - SAVE: context.save() to persist
  - PRINT: "Activated scope '<scope>'. Stage files with: jin add --scope=<scope>"
  - FOLLOW pattern: Similar to Task 2
  - NAMING: use_scope() function
  - PLACEMENT: Private function after create()
  - DEPENDENCIES: Task 7 (scope must exist)

Task 9: IMPLEMENT Scope Listing (src/commands/scope.rs::list)
  - LOAD: ProjectContext to get active scope
  - ENUMERATE: Mode-bound scopes via repo.find_refs("refs/jin/modes/*/scopes/*")
  - ENUMERATE: Untethered scopes via repo.find_refs("refs/jin/scopes/*")
  - PARSE: Extract scope names and mode associations
  - DISPLAY: Formatted list with mode bindings and active indicator
  - HANDLE: Empty case with helpful message
  - FOLLOW pattern: Similar to Task 3 but handle two ref patterns
  - NAMING: list() function
  - PLACEMENT: Private function after use_scope()
  - DEPENDENCIES: Task 7 (scopes must exist to list)

Task 10: IMPLEMENT Scope Deletion (src/commands/scope.rs::delete)
  - VALIDATE: Scope exists (check both mode-bound and untethered)
  - CHECK: If scope is active, call unset() first or prompt user
  - DELETE: Git ref(s) for scope (may be multiple if bound to different modes)
  - DELETE: Associated layer refs (mode-scope, scope-base, mode-scope-project)
  - PRINT: "Deleted scope '<scope>'" + info about removed layers
  - FOLLOW pattern: Similar to Task 4 but handle multiple possible refs
  - NAMING: delete() function
  - PLACEMENT: Private function after list()
  - DEPENDENCIES: Task 8 (may need to unset), Task 12 (unset logic)

Task 11: IMPLEMENT Scope Show (src/commands/scope.rs::show)
  - LOAD: ProjectContext from .jin/context
  - CHECK: context.scope is Some vs None
  - DISPLAY: Current scope name or "No active scope"
  - FOLLOW pattern: Similar to Task 5
  - NAMING: show() function
  - PLACEMENT: Private function after delete()
  - DEPENDENCIES: None (just reads context)

Task 12: IMPLEMENT Scope Unset (src/commands/scope.rs::unset)
  - LOAD: ProjectContext from .jin/context
  - UPDATE: context.scope = None
  - SAVE: context.save() to persist
  - PRINT: "Deactivated scope. Scope layers no longer available for staging."
  - FOLLOW pattern: Similar to Task 6
  - NAMING: unset() function
  - PLACEMENT: Private function after show()
  - DEPENDENCIES: None (just modifies context)

Task 13: IMPLEMENT Context Command (src/commands/context.rs::execute)
  - LOAD: ProjectContext from .jin/context
  - DISPLAY: Formatted output showing:
    - Active mode: <mode> or "(none)"
    - Active scope: <scope> or "(none)"
    - Project: <project> (auto-inferred)
    - Last updated: <timestamp>
  - FOLLOW pattern: Simple data display, human-readable format
  - NAMING: execute() function (no parameters)
  - PLACEMENT: Replace stub in src/commands/context.rs
  - DEPENDENCIES: None (just reads and displays context)

Task 14: ADD Unit Tests for Mode Commands (src/commands/mode.rs #[cfg(test)])
  - TEST: create() creates mode ref correctly
  - TEST: create() validates mode name format
  - TEST: create() rejects duplicate mode creation
  - TEST: use_mode() updates ProjectContext correctly
  - TEST: use_mode() errors if mode doesn't exist
  - TEST: unset() clears mode from context
  - TEST: list() handles empty case
  - TEST: delete() removes mode and updates context if active
  - FOLLOW pattern: src/commands/add.rs lines 206-325 (TempDir usage, error matching)
  - NAMING: test_<function>_<scenario> naming convention
  - PLACEMENT: #[cfg(test)] mod tests at end of mode.rs
  - DEPENDENCIES: All mode tasks complete (Tasks 1-6)

Task 15: ADD Unit Tests for Scope Commands (src/commands/scope.rs #[cfg(test)])
  - TEST: create() handles both mode-bound and untethered scopes
  - TEST: create() validates scope name format (allows colons)
  - TEST: use_scope() updates ProjectContext correctly
  - TEST: list() shows both mode-bound and untethered scopes
  - TEST: delete() handles mode-bound vs untethered correctly
  - FOLLOW pattern: Similar to Task 14
  - NAMING: test_<function>_<scenario> naming convention
  - PLACEMENT: #[cfg(test)] mod tests at end of scope.rs
  - DEPENDENCIES: All scope tasks complete (Tasks 7-12)

Task 16: ADD Integration Tests (tests/cli_basic.rs)
  - TEST: Mode lifecycle (create → use → list → show → unset → delete)
  - TEST: Scope lifecycle (create → use → list → show → unset → delete)
  - TEST: Mode + scope integration (both active simultaneously)
  - TEST: Mode/scope integration with jin add (routing validation)
  - TEST: Error cases (duplicate create, invalid names, non-existent use)
  - TEST: Help text for all subcommands
  - FOLLOW pattern: tests/cli_basic.rs existing tests (assert_cmd, predicates)
  - NAMING: test_mode_* and test_scope_* functions
  - PLACEMENT: Add to tests/cli_basic.rs
  - DEPENDENCIES: All implementation tasks complete (Tasks 1-13)
```

### Implementation Patterns & Key Details

```rust
// PATTERN: Mode/Scope Creation with Git Refs
fn create_mode_or_scope(repo: &JinRepo, ref_path: &str, description: &str) -> Result<Oid> {
    // 1. Create empty tree (no files yet)
    let empty_tree = repo.create_tree(&[])?;

    // 2. Create initial commit
    let commit_oid = repo.create_commit(
        None,                    // No parent for first commit
        description,             // Commit message
        empty_tree,              // Tree OID
        &[],                     // No parent commits
    )?;

    // 3. Set Git ref
    repo.set_ref(ref_path, commit_oid, description)?;

    Ok(commit_oid)
}

// PATTERN: Context Loading and Saving
fn update_context<F>(mutator: F) -> Result<()>
where
    F: FnOnce(&mut ProjectContext),
{
    // Load current context
    let mut context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // Apply mutation
    mutator(&mut context);

    // Save atomically
    context.save()?;

    Ok(())
}

// PATTERN: Name Validation
fn validate_mode_name(name: &str) -> Result<()> {
    // No empty names
    if name.is_empty() {
        return Err(JinError::Other("Mode name cannot be empty".to_string()));
    }

    // Alphanumeric and underscore only
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(JinError::Other(format!(
            "Invalid mode name '{}'. Use alphanumeric characters and underscores only.",
            name
        )));
    }

    // Reserved names (if any)
    let reserved = ["default", "global", "base"];
    if reserved.contains(&name) {
        return Err(JinError::Other(format!(
            "Mode name '{}' is reserved.",
            name
        )));
    }

    Ok(())
}

// PATTERN: Ref Existence Check
fn mode_exists(repo: &JinRepo, name: &str) -> Result<bool> {
    let ref_path = format!("refs/jin/modes/{}", name);
    match repo.resolve_ref(&ref_path) {
        Ok(_) => Ok(true),
        Err(JinError::NotFound(_)) => Ok(false),
        Err(e) => Err(e),
    }
}

// PATTERN: List with Active Indicator
fn list_modes(repo: &JinRepo, context: &ProjectContext) -> Result<()> {
    // Find all mode refs
    let mode_refs = repo.find_refs("refs/jin/modes/*")?;

    if mode_refs.is_empty() {
        println!("No modes found.");
        println!("Create one with: jin mode create <name>");
        return Ok(());
    }

    println!("Available modes:");

    // Extract names and display with active indicator
    for ref_path in mode_refs {
        let name = ref_path.strip_prefix("refs/jin/modes/")
            .unwrap_or(&ref_path);

        if Some(name) == context.mode.as_deref() {
            println!("  * {} [active]", name);
        } else {
            println!("    {}", name);
        }
    }

    Ok(())
}

// PATTERN: Delete with Active Check
fn delete_mode(repo: &JinRepo, name: &str) -> Result<()> {
    // Verify mode exists
    if !mode_exists(repo, name)? {
        return Err(JinError::NotFound(format!("Mode '{}' not found", name)));
    }

    // Load context to check if active
    let mut context = ProjectContext::load()?;

    // If active, unset first
    if Some(name) == context.mode.as_deref() {
        println!("Mode '{}' is currently active. Deactivating...", name);
        context.mode = None;
        context.save()?;
    }

    // Delete main mode ref
    repo.delete_ref(&format!("refs/jin/modes/{}", name))?;

    // Delete associated layer refs (mode-base, mode-scope, etc.)
    // These may not exist yet if no files committed
    let _ = repo.delete_ref(&format!("refs/jin/layers/mode/{}", name));
    let _ = repo.delete_ref(&format!("refs/jin/layers/mode/{}/scope/*", name));
    let _ = repo.delete_ref(&format!("refs/jin/layers/mode/{}/project/*", name));

    println!("Deleted mode '{}' and associated layers", name);
    Ok(())
}

// CRITICAL: Scope can be mode-bound OR untethered
fn scope_ref_path(name: &str, mode: Option<&str>) -> String {
    match mode {
        Some(m) => format!("refs/jin/modes/{}/scopes/{}", m, name),
        None => format!("refs/jin/scopes/{}", name),
    }
}

// CRITICAL: Output Formatting for User Feedback
fn print_success(action: &str, resource: &str, name: &str, suggestion: &str) {
    println!("{} {} '{}'", action, resource, name);
    if !suggestion.is_empty() {
        println!("{}", suggestion);
    }
}

// Usage:
// print_success("Created", "mode", "claude", "Activate with: jin mode use claude");
```

### Integration Points

```yaml
NO CHANGES NEEDED TO OTHER MODULES:
  - src/cli/mod.rs and args.rs already have ModeAction and ScopeAction enums defined
  - src/commands/mod.rs already dispatches to mode::execute and scope::execute
  - src/core/config.rs ProjectContext already has mode and scope fields
  - src/core/layer.rs already handles mode/scope ref path generation
  - src/staging/router.rs already validates mode/scope flags and routes to layers
  - src/git/repo.rs already provides all needed Git operations

VALIDATION DEPENDENCIES:
  - Mode/scope commands MUST use context.require_mode()/require_scope() for validation
  - Mode/scope commands MUST use validate_mode_name()/validate_scope_name() for input validation
  - Mode/scope commands MUST use Layer::ref_path() for consistent ref generation
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after implementing each command function
cargo check                          # Type checking
cargo clippy -- -D warnings          # Lint checking
cargo fmt -- --check                 # Format checking

# Expected: Zero errors, zero warnings
# Fix any issues before proceeding to next function
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run after completing Tasks 1-6 (mode commands)
cargo test --lib commands::mode      # Mode command unit tests

# Run after completing Tasks 7-12 (scope commands)
cargo test --lib commands::scope     # Scope command unit tests

# Run after completing Task 13 (context command)
cargo test --lib commands::context   # Context command unit tests

# Run all unit tests together
cargo test --lib                     # All library unit tests

# Expected: All tests pass
# Coverage: Each function tested with happy path, edge cases, error cases
```

### Level 3: Integration Testing (System Validation)

```bash
# Run after completing Task 16 (integration tests)
cargo test --test cli_basic          # CLI integration tests

# Run specific test groups
cargo test --test cli_basic test_mode_      # All mode tests
cargo test --test cli_basic test_scope_     # All scope tests

# Expected: All integration tests pass
# Coverage: Full workflows tested (create → use → list → delete)
```

### Level 4: Manual End-to-End Validation

```bash
# Build fresh binary
cargo build --release

# Test Mode Lifecycle
./target/release/jin init
./target/release/jin mode create claude
./target/release/jin mode list                    # Should show "claude"
./target/release/jin mode show                    # Should show "No active mode"
./target/release/jin mode use claude
./target/release/jin mode show                    # Should show "claude"
./target/release/jin status                       # Should show active mode
./target/release/jin mode unset
./target/release/jin mode show                    # Should show "No active mode"
./target/release/jin mode delete claude

# Test Scope Lifecycle
./target/release/jin scope create language:javascript
./target/release/jin scope list                   # Should show "language:javascript"
./target/release/jin scope use language:javascript
./target/release/jin scope show                   # Should show "language:javascript"
./target/release/jin scope unset
./target/release/jin scope delete language:javascript

# Test Mode + Scope Integration
./target/release/jin mode create python
./target/release/jin mode use python
./target/release/jin scope create testing --mode=python
./target/release/jin scope use testing
./target/release/jin context                      # Should show both mode and scope
./target/release/jin add test.py --mode --scope   # Should work (validation passes)

# Test Error Cases
./target/release/jin mode use nonexistent         # Should error with helpful message
./target/release/jin mode create claude           # Create mode
./target/release/jin mode create claude           # Should error: already exists
./target/release/jin mode create "invalid-name!"  # Should error: invalid characters
./target/release/jin add file.txt --mode          # Should error if no active mode

# Expected: All manual tests work as described
# User experience: Clear output, helpful error messages, intuitive workflow
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test --lib`
- [ ] All integration tests pass: `cargo test --test cli_basic`
- [ ] No linting errors: `cargo clippy -- -D warnings`
- [ ] No formatting issues: `cargo fmt -- --check`
- [ ] No type errors: `cargo check`

### Feature Validation

- [ ] All 12 subcommands work (6 mode + 6 scope)
- [ ] Mode lifecycle complete: create → use → list → show → unset → delete
- [ ] Scope lifecycle complete: create → use → list → show → unset → delete
- [ ] Mode-bound scopes work with `--mode` flag in create
- [ ] Untethered scopes work without `--mode` flag
- [ ] Active context persists across command invocations
- [ ] Context displayed correctly in `jin status` and `jin context`
- [ ] Mode/scope validation prevents invalid names
- [ ] Error messages are actionable and helpful

### Integration Validation

- [ ] `jin add --mode` works after mode activation
- [ ] `jin add --scope=<name>` works after scope activation
- [ ] `jin add --mode --scope` works with both active
- [ ] `jin commit` successfully commits to mode/scope layers
- [ ] `jin status` shows active mode/scope in context section
- [ ] Context survives process restarts (persistent in `.jin/context`)

### Code Quality Validation

- [ ] Follows patterns from `src/commands/add.rs`
- [ ] Uses `ProjectContext::load()` and `save()` correctly
- [ ] Uses `JinRepo` for all Git operations
- [ ] Error handling uses `JinError` variants consistently
- [ ] All public functions documented
- [ ] Tests cover happy path, edge cases, and error conditions
- [ ] No code duplication between mode.rs and scope.rs (extract helpers if needed)

### User Experience Validation

- [ ] Help text clear and includes examples
- [ ] Output formatted and readable (not debug dumps)
- [ ] Success messages confirm action and suggest next steps
- [ ] Error messages explain what went wrong and how to fix
- [ ] List commands show clear active indicators
- [ ] Empty list cases provide helpful guidance

### Documentation Validation

- [ ] Code comments explain non-obvious logic
- [ ] Function doc comments describe purpose, params, errors, examples
- [ ] No outdated TODOs or placeholder comments remain

---

## Anti-Patterns to Avoid

- ❌ Don't modify `.jin/context` directly - always use `ProjectContext::load()` and `save()`
- ❌ Don't create Git refs without commits - must have tree + commit first
- ❌ Don't silently ignore errors - return `Result<()>` and propagate with `?`
- ❌ Don't use string parsing for Git refs - use `JinRepo` methods
- ❌ Don't hardcode ref paths - use `Layer::ref_path()` or build paths consistently
- ❌ Don't skip name validation - always validate before operations
- ❌ Don't duplicate context loading - extract to helper if used multiple times
- ❌ Don't forget to check if mode/scope is active before deletion
- ❌ Don't use generic error messages - be specific and actionable
- ❌ Don't skip tests for error cases - they're as important as happy path
- ❌ Don't mix mode-bound and untethered scope logic - handle separately
- ❌ Don't assume `.jin/context` exists - handle `NotInitialized` error

---

## Confidence Score: 9/10

**Rationale for High Confidence:**

✅ **Complete context provided:**
- All data structures fully implemented (ProjectContext, Layer, routing)
- Reference implementation available (add.rs with 326 lines)
- Testing patterns established (unit + integration)
- External best practices researched
- All dependencies complete (P1-P4.M2)

✅ **Clear task breakdown:**
- 16 tasks in dependency order
- Each task has specific file location, pattern to follow, gotchas
- Implementation patterns provided with code examples
- Validation strategy at 4 levels

✅ **Minimal ambiguity:**
- Exact naming conventions specified
- Output formats defined
- Error messages templated
- Edge cases identified and handled

**Risk factors (−1 point):**
- Scope implementation slightly more complex (mode-bound vs untethered) - may need iteration
- Git ref deletion patterns less documented than creation - may need testing
- List formatting could vary based on preference - may need user feedback

**Mitigations:**
- Comprehensive testing will catch any issues with scope logic
- Integration tests validate ref lifecycle (create + delete)
- Manual E2E testing validates output formatting

**Validation:** An AI agent unfamiliar with this codebase can successfully implement all 12 commands using this PRP, with high likelihood of passing all tests on first iteration.
