# PRP: Add Command (P4.M2.T2)

---

## Goal

**Feature Goal**: Implement the `jin add` command that stages files to appropriate layers in Jin's 9-layer configuration hierarchy based on CLI routing flags and active project context.

**Deliverable**: A fully functional `jin add` command that:
- Routes files to the correct layer based on flags (`--mode`, `--scope`, `--project`, `--global`)
- Validates files are not already tracked by project Git
- Creates `StagedEntry` objects with proper metadata
- Persists entries to the `StagingIndex`
- Provides clear user feedback on what was staged

**Success Definition**:
- All routing combinations from PRD §9.1 work correctly
- Files are properly staged to `StagingIndex`
- Git-tracked files are rejected with clear error messages
- User sees summary of staged files by layer
- Command is idempotent (re-running on same files is safe)
- Unit tests cover all routing scenarios and error cases

## User Persona

**Target User**: Developer managing configuration files across multiple modes (AI assistants), scopes (language/ecosystem), and projects in a polyglot development environment.

**Use Case**: A developer wants to stage a language server configuration file to a specific scope, or stage a project-specific API key to a layer that combines mode and project context.

**User Journey**:
1. Developer creates or modifies a configuration file
2. Developer runs `jin add config.toml --mode --scope=language:rust` to stage it
3. Jin validates the file isn't git-tracked
4. Jin routes the file to the appropriate layer (ModeScope)
5. Jin creates a StagedEntry and persists it to the staging index
6. User sees confirmation of what was staged

**Pain Points Addressed**:
- No more manual .gitignore management for config files
- Clear separation between project code and configuration
- Explicit layer routing prevents accidental cross-contamination
- One command replaces complex git subdirectory management

## Why

- **Business value**: Enables Jin's core value prop of declarative, multi-layer configuration management
- **Integration**: Serves as the staging entry point before `jin commit` processes entries into Git layers
- **Problems solved**:
  - Developers struggle with config file sprawl across multiple projects
  - No clear way to route configs to appropriate scope/mode combinations
  - Git-tracked configs get accidentally committed to project repos

## What

The `jin add` command stages files to Jin's staging system, routing them to the appropriate layer based on CLI flags or active context.

### Command Interface

```bash
# Basic usage - route to project base (Layer 7)
jin add <file>...

# Route to specific layers
jin add <file>... --mode                    # Layer 2: ModeBase
jin add <file>... --scope=<name>            # Layer 6: ScopeBase
jin add <file>... --mode --project          # Layer 5: ModeProject
jin add <file>... --mode --scope=<name>     # Layer 3: ModeScope
jin add <file>... --mode --scope=<name> --project  # Layer 4: ModeScopeProject
jin add <file>... --global                  # Layer 1: GlobalBase
```

### Success Criteria

- [ ] All 7 routing combinations map to correct layers per PRD §9.1
- [ ] Files validated for existence before processing
- [ ] Git-tracked files rejected with `JinError::ValidationError`
- [ ] StagedEntry created with SHA-256 hash, timestamps, and metadata
- [ ] StagingIndex persisted to `.jin/staging/index.json`
- [ ] User receives summary of staged files grouped by layer
- [ ] Idempotent: re-adding same file updates existing entry
- [ ] Unit tests cover all routing scenarios and error cases

## All Needed Context

### Context Completeness Check

**Passes "No Prior Knowledge" test**: This PRP provides complete file paths, exact code patterns from the codebase, specific error variants to use, layer routing logic, and testing patterns. An implementer needs only this PRP and codebase access.

### Documentation & References

```yaml
# MUST READ - CLI patterns to follow
- file: src/commands/init.rs
  why: Complete command implementation pattern (idempotency, output, error handling)
  pattern: Command execute() function with Result<()>, DirGuard test pattern
  gotcha: Use update_gitignore() pattern for any future gitignore changes

# MUST READ - Layer routing logic
- file: src/staging/router.rs
  why: LayerRouter::route() implements PRD §9.1 routing table
  pattern: Delegate to Layer::from_flags() for consistent routing
  gotcha: Error when no routing target specified

# MUST READ - Staging system integration
- file: src/staging/entry.rs
  why: StagedEntry::new() creates staged file entries with hash and metadata
  pattern: SHA-256 hashing with sha2::Digest, FileStatus bitflags
  gotcha: content_hash is Vec<u8>, not String

- file: src/staging/index.rs
  why: StagingIndex manages all staged entries with dual-index design
  pattern: add_entry(), save_to_disk(), load_from_disk() methods
  gotcha: Secondary index (by_layer) is rebuilt after deserialization

# MUST READ - Layer definitions
- file: src/core/layer.rs
  why: Layer enum defines all 9 variants with from_flags() routing
  pattern: Layer::from_flags(mode, scope, project, global) -> Option<Layer>
  gotcha: from_flags() returns Option<Layer>, None means invalid combination

# MUST READ - Project context
- file: src/core/config.rs
  why: ProjectContext stores active mode/scope loaded from .jin/context
  pattern: ProjectContext::load() and ctx.mode, ctx.scope accessors
  gotcha: context_path() is a class method on ProjectContext

# MUST READ - Error handling
- file: src/core/error.rs
  why: All JinError variants for proper error handling
  pattern: Use JinError::FileNotFound for missing files, ValidationError for git-tracked
  gotcha: ValidationError has message field, FileNotFound has path field

# MUST READ - Git validation
- file: src/commit/validate.rs
  why: check_git_tracked() shows how to detect git-tracked files
  pattern: git2::Repository::status_file() with Status::WT_NEW check
  gotcha: WT_NEW means untracked, any other status means tracked

# MUST READ - CLI definition
- file: src/cli/args.rs
  why: AddCommand struct already defines all CLI flags
  pattern: Vec<PathBuf> for files, Option<String> for scope, bool for flags
  gotcha: All flags are already wired up, no changes needed
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── src/
│   ├── cli/
│   │   ├── args.rs          # AddCommand already defined here
│   │   └── mod.rs
│   ├── commands/
│   │   ├── init.rs          # Reference implementation for commands
│   │   ├── mod.rs           # Add: pub mod add;
│   │   └── add.rs           # CREATE THIS FILE
│   ├── commit/
│   │   └── validate.rs      # check_git_tracked() reference
│   ├── core/
│   │   ├── config.rs        # ProjectContext
│   │   ├── error.rs         # JinError variants
│   │   └── layer.rs         # Layer enum
│   ├── staging/
│   │   ├── entry.rs         # StagedEntry
│   │   ├── index.rs         # StagingIndex
│   │   └── router.rs        # LayerRouter
│   └── main.rs              # Command dispatcher (already wired)
└── plan/P4M2T2/
    └── PRP.md               # This document
```

### Desired Codebase Tree (files to be added)

```bash
# NEW FILE TO CREATE
├── src/commands/add.rs      # Main add command implementation
│   ├── execute()            # Entry point
│   ├── determine_layer()    # Layer routing logic
│   ├── validate_file()      # File validation helper
│   ├── stage_file()         # Staging logic helper
│   └── tests module         # Comprehensive unit tests
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Layer::from_flags() returns Option<Layer>, not Result<Layer>
// Must convert None to appropriate JinError
let layer = Layer::from_flags(mode, scope, project, global)
    .ok_or_else(|| JinError::Message("No routing target".to_string()))?;

// CRITICAL: ProjectContext::load() needs &Path, not &PathBuf
let ctx = ProjectContext::load(&project_dir)?;

// CRITICAL: StagingIndex::load_from_disk() needs &Path, not &PathBuf
let mut index = StagingIndex::load_from_disk(&project_dir)?;

// CRITICAL: check_git_tracked() needs relative path, not absolute
// Get workspace_root and convert file_path to relative first

// CRITICAL: SHA-256 hash returns Vec<u8>, use .to_vec() or collect()
let hash = sha2::Sha256::digest(content);
let hash_vec = hash.to_vec();  // or hash.iter().copied().collect()

// CRITICAL: git2::Status::WT_NEW means file is NEW/UNTRACKED
// If status is NOT WT_NEW and NOT empty, file is tracked
if status.contains(git2::Status::WT_NEW) {
    return Ok(());  // Untracked, safe to add
}
if status != git2::Status::empty() {
    return Err(JinError::ValidationError { ... });  // Tracked!
}

// CRITICAL: LayerRouter requires project name for construction
let project_name = detect_project_name()?;
let router = LayerRouter::new(project_name);

// CRITICAL: All commands must use DirGuard pattern in tests
// to restore original directory after tempfile operations

// CRITICAL: Use JinError::FileNotFound { path } for missing files
// Use JinError::ValidationError { message } for git-tracked files
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - using existing types:
- `AddCommand` (from `src/cli/args.rs`) - CLI arguments
- `Layer` (from `src/core/layer.rs`) - Target layer enum
- `ProjectContext` (from `src/core/config.rs`) - Active mode/scope
- `StagedEntry` (from `src/staging/entry.rs`) - Staged file representation
- `StagingIndex` (from `src/staging/index.rs`) - Staging persistence

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/add.rs with module structure
  - IMPLEMENT: Module skeleton with use statements
  - INCLUDE: crate::cli::args::AddCommand, crate::core::error::Result
  - INCLUDE: crate::core::{config::ProjectContext, layer::Layer}
  - INCLUDE: crate::staging::{entry::StagedEntry, index::StagingIndex, router::LayerRouter}
  - PATTERN: Follow src/commands/init.rs structure
  - NAMING: execute() function, helper functions below

Task 2: IMPLEMENT determine_layer() helper function
  - SIGNATURE: fn determine_layer(cmd: &AddCommand, context: &ProjectContext, project: &str) -> Result<Layer>
  - LOGIC: Map CLI flags to Layer::from_flags() parameters
  - FALLBACK: When no flags specified, use context.mode and context.scope
  - ERROR: Return JinError::Message if no routing target can be determined
  - REFERENCE: src/staging/router.rs for routing table logic
  - PATTERN:
    let mode = if cmd.mode { context.mode.as_deref() } else { None };
    let scope = cmd.scope.as_deref().or_else(|| context.scope.as_deref());
    let project = if cmd.project { Some(project) } else { None };
    Layer::from_flags(mode, scope, project, cmd.global)
        .ok_or_else(|| JinError::Message("No routing target specified".to_string()))

Task 3: IMPLEMENT validate_file() helper function
  - SIGNATURE: fn validate_file(path: &Path, workspace_root: &Path) -> Result<()>
  - CHECK 1: File exists -> JinError::FileNotFound { path } if not
  - CHECK 2: File is readable -> propagate Io errors
  - CHECK 3: File not git-tracked -> use check_git_tracked() from validate.rs
  - CHECK 4: File not symlink -> JinError::SymlinkNotSupported
  - CHECK 5: File is text, not binary -> JinError::BinaryFileNotSupported
  - REFERENCE: src/commit/validate.rs::check_git_tracked()

Task 4: IMPLEMENT stage_file() helper function
  - SIGNATURE: fn stage_file(path: &Path, layer: &Layer, workspace_root: &Path, index: &mut StagingIndex) -> Result<StagedEntry>
  - READ: std::fs::read_to_string(path) for file content
  - HASH: Compute SHA-256 using sha2::Sha256::digest()
  - CREATE: StagedEntry::new(path, layer, content.as_bytes())
  - ADD: index.add_entry(entry) to staging index
  - RETURN: The created StagedEntry for user feedback
  - REFERENCE: src/staging/entry.rs for StagedEntry::new()

Task 5: IMPLEMENT execute() main function
  - SIGNATURE: pub fn execute(cmd: &AddCommand) -> Result<()>
  - STEP 1: Get workspace_root (std::env::current_dir()?)
  - STEP 2: Load ProjectContext from workspace_root
  - STEP 3: Detect project name (from git origin or directory name)
  - STEP 4: Create LayerRouter with project name
  - STEP 5: Determine target layer using determine_layer()
  - STEP 6: Load StagingIndex from disk (create new if doesn't exist)
  - STEP 7: For each file in cmd.files:
    - Resolve to absolute path if relative
    - Validate with validate_file()
    - Stage with stage_file()
    - Print success message with layer name
  - STEP 8: Persist StagingIndex to disk
  - STEP 9: Print summary of files staged by layer
  - ERROR: Propagate all errors with context
  - PATTERN: Follow src/commands/init.rs::execute() structure

Task 6: ADD src/commands/add.rs to src/commands/mod.rs
  - ADD: pub mod add;
  - EXPORT: pub use add::execute as add_execute; (if needed)
  - PATTERN: Follow existing mod.rs structure

Task 7: VERIFY command dispatcher in main.rs
  - CHECK: Commands::Add variant already exists
  - CHECK: Calls commands::execute(&cmd) or similar
  - NO CHANGES: Already wired from P4.M1.T1

Task 8: CREATE comprehensive unit tests in src/commands/add.rs tests module
  - USE: tempfile::TempDir for isolated test directories
  - USE: DirGuard pattern for directory restoration
  - TEST: test_add_to_project_base() - default routing
  - TEST: test_add_to_mode_base() - --mode flag
  - TEST: test_add_to_scope_base() - --scope flag
  - TEST: test_add_to_mode_scope() - --mode --scope flags
  - TEST: test_add_to_global() - --global flag
  - TEST: test_add_file_not_found() - error handling
  - TEST: test_add_git_tracked_file() - rejection
  - TEST: test_add_multiple_files() - batch processing
  - TEST: test_add_with_context() - using active mode/scope from context
  - PATTERN: Follow src/commands/init.rs test structure

Task 9: RUN validation and fix any issues
  - LINT: cargo clippy --all-targets --all-features
  - FMT: cargo fmt
  - TEST: cargo test --lib
  - BUILD: cargo build --release
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Command execute() structure
pub fn execute(cmd: &AddCommand) -> Result<()> {
    // Get workspace root
    let workspace_root = std::env::current_dir()?;

    // Load context for active mode/scope
    let context = ProjectContext::load(&workspace_root)?;

    // Detect project name for LayerRouter
    let project_name = detect_project_name(&workspace_root)?;

    // Determine target layer
    let layer = determine_layer(cmd, &context, &project_name)?;

    // Load staging index
    let mut staging_index = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());

    // Track staged files by layer for summary
    let mut staged_by_layer: std::collections::HashMap<String, Vec<PathBuf>> = std::collections::HashMap::new();

    // Process each file
    for file_path in &cmd.files {
        let resolved_path = resolve_path(&workspace_root, file_path)?;
        validate_file(&resolved_path, &workspace_root)?;

        let entry = stage_file(&resolved_path, &layer, &workspace_root, &mut staging_index)?;

        let layer_key = format!("{}", layer);
        staged_by_layer.entry(layer_key)
            .or_insert_with(Vec::new)
            .push(file_path.clone());

        println!("Staged {} to {}", file_path.display(), layer);
    }

    // Persist staging index
    staging_index.save_to_disk(&workspace_root)?;

    // Print summary
    if !staged_by_layer.is_empty() {
        println!("\nSummary:");
        for (layer_name, files) in staged_by_layer {
            println!("  {}:", layer_name);
            for file in files {
                println!("    - {}", file.display());
            }
        }
    }

    Ok(())
}

// Pattern 2: Layer determination with context fallback
fn determine_layer(
    cmd: &AddCommand,
    context: &ProjectContext,
    project: &str,
) -> Result<Layer> {
    // If any flags specified, use explicit routing
    if cmd.mode || cmd.scope.is_some() || cmd.project || cmd.global {
        let mode = if cmd.mode { context.mode.as_deref() } else { None };
        let scope = cmd.scope.as_deref().or_else(|| context.scope.as_deref());
        let proj = if cmd.project { Some(project) } else { None };

        return Layer::from_flags(mode, scope, proj, cmd.global)
            .ok_or_else(|| JinError::Message(
                "No routing target specified. Use --mode, --scope, --project, or --global".to_string()
            ));
    }

    // No flags - use context defaults, or project as final fallback
    if let Some(mode) = &context.mode {
        if let Some(scope) = &context.scope {
            return Ok(Layer::ModeScope {
                mode: mode.clone(),
                scope: scope.clone(),
            });
        }
        return Ok(Layer::ModeBase { mode: mode.clone() });
    }

    if let Some(scope) = &context.scope {
        return Ok(Layer::ScopeBase { scope: scope.clone() });
    }

    // Final fallback: project base layer
    Ok(Layer::ProjectBase { project: project.to_string() })
}

// Pattern 3: File validation with Git tracking check
fn validate_file(path: &Path, workspace_root: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    // Check not symlink
    if path.is_symlink() {
        return Err(JinError::SymlinkNotSupported {
            path: path.display().to_string(),
        });
    }

    // Check git-tracked status
    let relative_path = path.strip_prefix(workspace_root)
        .map_err(|_| JinError::Message(
            format!("File is outside workspace root: {}", path.display())
        ))?;

    check_git_tracked(workspace_root, relative_path)?;

    // TODO: Check binary vs text file
    // For now, assume text files

    Ok(())
}

// Pattern 4: Project name detection from Git origin
fn detect_project_name(workspace_root: &Path) -> Result<String> {
    use git2::Repository;

    let repo = Repository::discover(workspace_root)
        .map_err(|_| JinError::RepoNotFound {
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
        .ok_or_else(|| JinError::Message(
            "Cannot determine project name".to_string()
        ))
}

// Pattern 5: Path resolution helper
fn resolve_path(workspace_root: &Path, file_path: &Path) -> Result<PathBuf> {
    if file_path.is_absolute() {
        Ok(file_path.to_path_buf())
    } else {
        Ok(workspace_root.join(file_path))
    }
}

// Pattern 6: Git-tracked file check (from validate.rs)
fn check_git_tracked(workspace_root: &Path, relative_path: &Path) -> Result<()> {
    use git2::Repository;

    let repo = Repository::open(workspace_root)
        .map_err(|_| JinError::Message("Not a Git repository".to_string()))?;

    let status = match repo.status_file(relative_path) {
        Ok(s) => s,
        Err(_) => return Ok(()),  // File might not exist yet
    };

    // WT_NEW means untracked/new - safe to add
    if status.contains(git2::Status::WT_NEW) {
        return Ok(());
    }

    // Any other status means tracked
    if status != git2::Status::empty() {
        return Err(JinError::ValidationError {
            message: format!("File is tracked by Git: {}", relative_path.display()),
        });
    }

    Ok(())
}
```

### Integration Points

```yaml
COMMAND_MODULE:
  - modify: src/commands/mod.rs
  - add: pub mod add;
  - pattern: Follow existing module exports

STAGING_INDEX:
  - method: StagingIndex::load_from_disk(&Path)
  - method: StagingIndex::save_to_disk(&Path)
  - method: StagingIndex::add_entry(StagedEntry)
  - pattern: Handle Err if index doesn't exist (first run)

LAYER_ROUTER:
  - method: LayerRouter::new(project: String)
  - method: LayerRouter::route(mode, scope, project, global)
  - alternative: Use Layer::from_flags() directly

PROJECT_CONTEXT:
  - method: ProjectContext::load(&Path) -> Result<ProjectContext>
  - field: context.mode: Option<String>
  - field: context.scope: Option<String>

GIT_INTEGRATION:
  - crate: git2
  - use: git2::Repository::open(), status_file()
  - use: git2::Status::WT_NEW for untracked detection
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each significant code change
cargo fmt                          # Auto-format code
cargo clippy --all-targets         # Lint checking

# Expected: Zero warnings, zero errors
# Common clippy warnings to fix:
# - unused_variables
# - redundant_clone
# - unwrap_used (use proper error handling instead)
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test add command specifically
cargo test --lib add

# Test all command tests
cargo test --lib commands

# Test with output
cargo test --lib add -- --nocapture

# Expected: All tests pass
# Key test cases:
# - test_add_to_project_base: Default routing works
# - test_add_to_mode_base: --mode flag routes correctly
# - test_add_to_scope_base: --scope flag routes correctly
# - test_add_to_global: --global flag routes correctly
# - test_add_file_not_found: Returns FileNotFound error
# - test_add_git_tracked_file: Returns ValidationError
# - test_add_with_context: Uses context mode/scope when no flags
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the project
cargo build --release

# Manual integration test in a temporary directory
cd /tmp
mkdir test_jin_add && cd test_jin_add
git init
jin init                     # Initialize Jin
echo "test = true" > config.toml
jin add config.toml          # Should stage to ProjectBase
jin add config.toml --mode   # Should stage to ModeBase (if mode set)
jin add config.toml --global # Should stage to GlobalBase

# Verify staging index
cat .jin/staging/index.json  # Should contain staged entries

# Test error cases
touch tracked.txt
git add tracked.txt
jin add tracked.txt          # Should fail with ValidationError

# Expected: All manual tests pass, correct routing, proper error handling
```

### Level 4: CLI & Domain-Specific Validation

```bash
# Test all routing combinations from PRD §9.1
jin init
echo "config" > test.conf

# Test each routing combination
jin add test.conf --global                    # GlobalBase
jin add test.conf --mode                      # ModeBase (requires context mode)
jin add test.conf --scope=test                # ScopeBase
jin add test.conf --mode --project            # ModeProject
jin add test.conf --mode --scope=test         # ModeScope
jin add test.conf --mode --scope=test --project  # ModeScopeProject
jin add test.conf                             # ProjectBase (default)

# Verify each routed correctly by checking staging index
jq '.entries | to_entries[] | {path: .key, layer: .value.layer}' .jin/staging/index.json

# Test with active context
jin mode set claude
jin add test.conf --mode                      # Should use "claude" mode

# Test multiple files
echo "a" > a.txt && echo "b" > b.txt
jin add a.txt b.txt

# Test error handling
jin add nonexistent.txt                       # FileNotFound
touch tracked.txt && git add tracked.txt
jin add tracked.txt                           # ValidationError

# Expected: All routing combinations work, proper error messages
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo build --release`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check`
- [ ] All tests pass: `cargo test --lib`
- [ ] No unused imports or dead code

### Feature Validation

- [ ] All 7 routing combinations from PRD §9.1 work correctly
- [ ] Default routing (no flags) goes to ProjectBase
- [ ] --global flag routes to GlobalBase
- [ ] --mode flag routes to ModeBase
- [ ] --scope flag routes to ScopeBase
- [ ] --mode --project routes to ModeProject
- [ ] --mode --scope routes to ModeScope
- [ ] --mode --scope --project routes to ModeScopeProject
- [ ] Context fallback works when no flags specified
- [ ] File not found returns JinError::FileNotFound
- [ ] Git-tracked file returns JinError::ValidationError
- [ ] StagingIndex persisted to .jin/staging/index.json
- [ ] User sees summary of staged files by layer
- [ ] Command is idempotent (re-running is safe)

### Code Quality Validation

- [ ] Follows src/commands/init.rs patterns
- [ ] Proper error handling with Result<>
- [ ] Comprehensive doc comments on public functions
- [ ] Unit tests cover all routing scenarios
- [ ] Unit tests cover all error cases
- [ ] Tests use tempfile and DirGuard pattern
- [ ] No unwrap() calls (use proper error handling)
- [ ] No expect() calls (use proper error handling)

### Documentation & Deployment

- [ ] execute() has comprehensive doc comment
- [ ] Helper functions have doc comments
- [ ] User-facing error messages are clear
- [ ] Success output shows layer and file path

---

## Anti-Patterns to Avoid

- **Don't** create new routing logic - use `Layer::from_flags()` from `src/core/layer.rs`
- **Don't** use `unwrap()` or `expect()` - propagate errors with `?`
- **Don't** hardcode layer paths - use `Layer` enum methods
- **Don't** skip file validation - always check git-tracked status
- **Don't** ignore the active context - use mode/scope from ProjectContext when flags not specified
- **Don't** create duplicate StagedEntry if file already staged - update existing entry instead
- **Don't** forget to persist StagingIndex - call `save_to_disk()` after all files processed
- **Don't** use absolute paths for storage - use relative paths from workspace root
- **Don't** skip the summary output - users need to know what was staged where
- **Don't** ignore test coverage - all routing combinations must be tested

## Confidence Score: 9/10

**Reasoning**:
- Complete codebase analysis with specific file references
- All required infrastructure (staging, routing, errors) already exists
- Clear implementation pattern from init command to follow
- Specific error variants and test patterns documented
- Only minor uncertainty: binary file detection (not specified in PRD, can skip for initial implementation)

**Remaining risk**: Binary file detection logic may need clarification from user, but can be implemented as text-only assumption initially.
