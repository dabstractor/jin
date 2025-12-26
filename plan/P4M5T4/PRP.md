# Product Requirement Prompt (PRP): Import Command

**Work Item**: P4.M5.T4 - Import Command
**Task**: Wire ImportCommand - Import Git-tracked files into Jin
**Points**: 2

---

## Goal

**Feature Goal**: Implement the `jin import` command that imports Git-tracked files from the workspace into Jin's layer management system, enabling users to transition existing configuration files from Git tracking to Jin layer control.

**Deliverable**: A fully functional `jin import` command that:
1. Validates input files are Git-tracked (opposite of `jin add`)
2. Routes files to target layers using the same layer selection logic as `jin add`
3. Creates staged entries for import
4. Optionally removes files from Git tracking (adds to .gitignore managed section)
5. Persists changes to the staging index for subsequent commit

**Success Definition**:
- `jin import <file>...` imports Git-tracked files into Jin staging
- Files are validated as Git-tracked before import (rejects untracked files)
- Layer routing follows the same pattern as `jin add` (--mode, --scope, --project, --global)
- Imported files are added to staging index at the target layer
- User receives clear feedback on what was imported and where
- Integration tests cover happy path and edge cases

---

## User Persona

**Target User**: Developer transitioning existing project configurations from Git-only to Jin-managed

**Use Case**: Developer has configuration files (e.g., `.env`, `config.json`) currently tracked in Git and wants to move them under Jin's multi-layer management system without losing history or breaking existing setups.

**User Journey**:
1. User identifies a Git-tracked configuration file to manage with Jin
2. User runs `jin import config.toml --mode` to import it to the active mode layer
3. Jin validates the file is tracked by Git
4. Jin reads the file content and creates a staged entry
5. User reviews the staged changes with `jin status`
6. User commits with `jin commit -m "Import config.toml to mode layer"`
7. File is now managed by Jin and excluded from Git tracking via .gitignore

**Pain Points Addressed**:
- **Manual configuration management**: No need to manually copy files and update .gitignore
- **Lost history**: File history is preserved in Git while Jin takes over management
- **Team coordination**: Clear import process vs. ad-hoc file management
- **Git污染污染**: Prevents accidental commits of personal configurations

---

## Why

- **Migration path for existing projects**: Provides a smooth transition from Git-only to Jin-managed configurations
- **Adoption enabler**: Lowers friction for adopting Jin by importing existing files
- **Consistency with add command**: Import is the inverse of add - add handles untracked files, import handles tracked files
- **Integration with layer system**: Uses established layer routing for seamless workflow
- **Git hygiene**: Properly handles .gitignore updates to prevent future Git tracking of imported files

---

## What

User-visible behavior:
```bash
# Import to active mode layer (from context)
$ jin import config.json
Imported config.json to mode/claude

# Import to specific layer
$ jin import .env --scope python
Imported .env to scope/python

# Import multiple files
$ jin import config.toml settings.yaml --project
Imported config.toml to project/myapp
Imported settings.yaml to project/myapp

# Status after import
$ jin status
Staged changes:
  mode/claude:
    config.json (new)

# Commit to complete import
$ jin commit -m "Import configuration files"
```

### Success Criteria

- [ ] Command accepts one or more file paths as arguments
- [ ] Validates all files are Git-tracked (rejects untracked files)
- [ ] Routes files to target layer using --mode, --scope, --project, --global flags
- [ ] Falls back to context-based routing (mode/scope from active context)
- [ ] Creates StagedEntry for each imported file
- [ ] Adds entries to StagingIndex persistently
- [ ] Provides clear console feedback on import success
- [ ] Shows summary of imported files by layer
- [ ] Returns appropriate error codes for failures

---

## All Needed Context

### Context Completeness Check

If someone knew nothing about this codebase, they would have everything needed to implement this successfully. The PRP includes:
- Exact file patterns to follow from existing commands
- Complete layer routing logic
- All validation patterns (including the inverse of add's validation)
- Staging index integration patterns
- Error handling conventions
- Test patterns from the codebase

### Documentation & References

```yaml
# MUST READ - Core command patterns
- file: src/commands/add.rs
  why: Primary reference - shows file validation, layer routing, staging integration
  pattern: Complete command implementation pattern (validate, route, stage, persist)
  gotcha: Import does opposite validation - REQUIRES file to be git-tracked

- file: src/commands/mod.rs
  why: Shows how to export command from module
  pattern: `pub use import::execute as import_execute;`

- file: src/cli/args.rs
  why: ImportCommand already defined (lines 318-324)
  pattern: `#[derive(clap::Args)]`, `#[arg(value_name = "FILE", num_args(1..))]`
  gotcha: ImportCommand struct already exists - no CLI changes needed

# MUST READ - Layer routing logic
- file: src/commands/add.rs
  why: determine_layer() function shows exact layer routing pattern
  pattern: Layer::from_flags(mode, scope, project, global) with context fallback
  lines: 118-173

# MUST READ - Staging integration
- file: src/staging/entry.rs
  why: StagedEntry::new() signature for creating entries
  pattern: `StagedEntry::new(path, layer, content)` - requires absolute path for metadata

- file: src/staging/index.rs
  why: StagingIndex persistence and entry management
  pattern: `load_from_disk()`, `add_entry()`, `save_to_disk()`

# MUST READ - Validation patterns
- file: src/commit/validate.rs
  why: check_git_tracked() shows Git status checking pattern
  pattern: Uses git2::Repository::status_file() to check if file is tracked
  gotcha: Import should use OPPOSITE check - file MUST be tracked
  lines: 297-341

# MUST READ - Main dispatch wiring
- file: src/main.rs
  why: Shows command dispatch pattern
  pattern: Match on Commands::Import(cmd), call commands::import_execute(&cmd)
  lines: 199-202 (placeholder to replace)
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs          # ImportCommand struct defined (lines 318-324)
│   └── mod.rs
├── commands/
│   ├── add.rs           # PRIMARY REFERENCE for file operations
│   ├── apply.rs
│   ├── commit.rs
│   ├── context.rs
│   ├── diff.rs
│   ├── init.rs
│   ├── log.rs
│   ├── mode.rs
│   ├── mod.rs           # Add: pub use import::execute as import_execute;
│   ├── reset.rs
│   ├── scope.rs
│   └── status.rs
├── commit/
│   ├── validate.rs      # Git status checking patterns
│   └── ...
├── core/
│   ├── error.rs         # JinError types
│   ├── layer.rs         # Layer enum and routing logic
│   └── ...
├── git/
│   └── repo.rs          # JinRepo wrapper
├── staging/
│   ├── entry.rs         # StagedEntry
│   └── index.rs         # StagingIndex
└── main.rs              # Command dispatch (update lines 199-202)
```

### Desired Codebase Tree (Files to Add)

```bash
src/
├── commands/
│   ├── import.rs        # NEW - Import command implementation
│   └── mod.rs           # MODIFY - Add export for import_execute
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Validation is OPPOSITE of add command
// add.rs: validate_file() REJECTS git-tracked files
// import.rs: validate_file() REQUIRES git-tracked files

// From commit/validate.rs - check_git_tracked() returns Err if tracked
// Import must invert this logic:
pub fn check_importable_file(workspace_root: &Path, relative_path: &Path) -> Result<()> {
    let git_repo = git2::Repository::open(workspace_root)
        .map_err(|_| JinError::Message("Not a Git repository".to_string()))?;

    let status = match git_repo.status_file(relative_path) {
        Ok(s) => s,
        Err(_) => return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string()
        }),
    };

    // CRITICAL: File must be tracked (NOT WT_NEW)
    if status.contains(git2::Status::WT_NEW) {
        return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string()
        });
    }

    // File should have some status (tracked)
    if status == git2::Status::empty() {
        return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string()
        });
    }

    Ok(())
}

// CRITICAL: Use determine_layer() from add.rs directly
// Layer routing is IDENTICAL between add and import
// No need to reimplement - call existing helper

// CRITICAL: StagedEntry::new() requires ABSOLUTE path
// It reads filesystem metadata (mtime, size)
let entry = StagedEntry::new(
    resolved_path.clone(),  // Must be absolute
    layer.clone(),
    content.as_bytes()
)?;

// CRITICAL: Import should still reject symlinks and binary files
// Only the git-tracked check is inverted
// Other validations (symlink, binary) remain the same as add
```

---

## Implementation Blueprint

### Data Models

No new data models needed. Uses existing types:
- `ImportCommand` from `cli::args` (already defined)
- `Layer` enum for layer routing
- `StagedEntry` for staged file metadata
- `StagingIndex` for staging persistence
- `JinError` for error handling

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/import.rs
  - IMPLEMENT: execute() function with ImportCommand parameter
  - FOLLOW pattern: src/commands/add.rs (lines 51-116)
  - NAMING: pub fn execute(cmd: &ImportCommand) -> Result<()>
  - FILE_STRUCTURE:
    * Module docstring explaining import functionality
    * execute() as main entry point
    * determine_layer() - reuse from add.rs or import
    * validate_importable_file() - NEW, opposite of add validation
    * resolve_path() - reuse from add.rs
    * detect_project_name() - reuse from add.rs
  - DEPENDENCIES: crate::cli::args::ImportCommand, core types, staging, git2

Task 2: MODIFY src/commands/mod.rs
  - ADD: pub mod import;
  - ADD: pub use import::execute as import_execute;
  - FOLLOW pattern: Existing module exports (lines 4-28)
  - PLACEMENT: After init.rs, before apply.rs (alphabetical)

Task 3: MODIFY src/main.rs
  - REPLACE: Lines 199-202 (placeholder) with actual dispatch
  - FOLLOW pattern: Commands::Add dispatch (lines 21-27)
  - IMPLEMENT: match commands::import_execute(&cmd) with error handling
  - RETURNS: ExitCode::SUCCESS on Ok, ExitCode::FAILURE on Err

Task 4: CREATE src/commands/import.rs tests module
  - IMPLEMENT: Unit tests following add.rs pattern (lines 282-842)
  - TEST CASES:
    * test_import_to_project_base
    * test_import_to_mode_base
    * test_import_to_scope_base
    * test_import_to_global
    * test_import_file_not_tracked (should fail - opposite of add)
    * test_import_symlink_rejected
    * test_import_binary_file_rejected
    * test_import_multiple_files
    * test_import_with_context
    * test_import_is_idempotent
    * test_resolve_path_absolute/relative
    * test_determine_layer_* (routing tests)
  - FIXTURES: DirGuard, create_test_file, init_git_repo, init_jin

Task 5: ADD JinError::FileNotTrusted variant
  - MODIFY: src/core/error.rs
  - ADD: FileNotTracked { path: String } variant
  - WHY: Need specific error for untracked files during import
  - PLACEMENT: Near FileNotFound, SymlinkNotSupported
```

### Implementation Patterns & Key Details

```rust
// ===== MAIN EXECUTION PATTERN (from add.rs) =====
pub fn execute(cmd: &ImportCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Load context for active mode/scope
    let context = ProjectContext::load(&workspace_root)?;

    // 3. Detect project name for LayerRouter
    let project_name = detect_project_name(&workspace_root)?;

    // 4. Determine target layer (reuse logic from add.rs)
    let layer = determine_layer(cmd, &context, &project_name)?;

    // 5. Load staging index (create new if doesn't exist)
    let mut staging_index =
        StagingIndex::load_from_disk(&workspace_root).unwrap_or_else(|_| StagingIndex::new());

    // 6. Track staged files by layer for summary
    let mut staged_by_layer: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // 7. Process each file
    for file_path in &cmd.files {
        let resolved_path = resolve_path(&workspace_root, file_path)?;

        let relative_path = resolved_path.strip_prefix(&workspace_root).map_err(|_| {
            JinError::Message(format!(
                "File is outside workspace root: {}",
                resolved_path.display()
            ))
        })?;

        // CRITICAL: Different validation - file MUST be git-tracked
        validate_importable_file(&resolved_path, &workspace_root)?;

        // Check not symlink (same as add)
        if resolved_path.is_symlink() {
            return Err(JinError::SymlinkNotSupported {
                path: resolved_path.display().to_string(),
            });
        }

        // Check binary (same as add)
        let content = std::fs::read(&resolved_path)?;
        if content.contains(&0x00) {
            return Err(JinError::BinaryFileNotSupported {
                path: resolved_path.display().to_string(),
            });
        }

        let text_content = std::fs::read_to_string(&resolved_path)?;
        let entry = StagedEntry::new(resolved_path.clone(), layer.clone(), text_content.as_bytes())?;

        staging_index.add_entry(entry)?;

        let layer_key = format!("{}", layer);
        staged_by_layer
            .entry(layer_key)
            .or_default()
            .push(relative_path.to_path_buf());

        println!("Imported {} to {}", file_path.display(), layer);
    }

    // 8. Persist staging index
    staging_index.save_to_disk(&workspace_root)?;

    // 9. Print summary
    if !staged_by_layer.is_empty() {
        println!();
        println!("Summary:");
        for (layer_name, files) in staged_by_layer {
            println!("  {}:", layer_name);
            for file in files {
                println!("    - {}", file.display());
            }
        }
    }

    Ok(())
}

// ===== LAYER ROUTING (reuse from add.rs) =====
fn determine_layer(cmd: &ImportCommand, context: &ProjectContext, project: &str) -> Result<Layer> {
    // CRITICAL: ImportCommand needs routing flags
    // Current ImportCommand in args.rs has NO flags - need to add them
    // See Implementation Tasks for flags to add

    // If any flags specified, use explicit routing
    if cmd.mode || cmd.scope.is_some() || cmd.project || cmd.global {
        let mode = if cmd.mode {
            context.mode.as_deref()
        } else {
            None
        };
        let scope = cmd.scope.as_deref().or(context.scope.as_deref());
        let proj = if cmd.project { Some(project) } else { None };

        return Layer::from_flags(mode, scope, proj, cmd.global).ok_or_else(|| {
            JinError::Message(
                "No routing target specified. Use --mode, --scope, --project, or --global"
                    .to_string(),
            )
        });
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
        return Ok(Layer::ScopeBase {
            scope: scope.clone(),
        });
    }

    // Final fallback: project base layer
    Ok(Layer::ProjectBase {
        project: project.to_string(),
    })
}

// ===== VALIDATION (opposite of add) =====
fn validate_importable_file(path: &Path, workspace_root: &Path) -> Result<()> {
    // Check file exists
    if !path.exists() {
        return Err(JinError::FileNotFound {
            path: path.display().to_string(),
        });
    }

    let relative_path = path.strip_prefix(workspace_root).map_err(|_| {
        JinError::Message(format!(
            "File is outside workspace root: {}",
            path.display()
        ))
    })?;

    // CRITICAL: Check file IS git-tracked (opposite of add)
    let git_repo = git2::Repository::open(workspace_root).map_err(|_| {
        JinError::Message("Not a Git repository".to_string())
    })?;

    let status = match git_repo.status_file(relative_path) {
        Ok(s) => s,
        Err(_) => return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string()
        }),
    };

    // File must be tracked (not WT_NEW)
    if status.contains(git2::Status::WT_NEW) {
        return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string()
        });
    }

    if status == git2::Status::empty() {
        return Err(JinError::FileNotTracked {
            path: relative_path.display().to_string()
        });
    }

    Ok(())
}

// ===== Path resolution (same as add) =====
fn resolve_path(workspace_root: &Path, file_path: &Path) -> Result<PathBuf> {
    if file_path.is_absolute() {
        Ok(file_path.to_path_buf())
    } else {
        Ok(workspace_root.join(file_path))
    }
}

// ===== Project detection (same as add) =====
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
```

### Integration Points

```yaml
CLI_ARGS:
  - file: src/cli/args.rs
  - modify: Add routing flags to ImportCommand (lines 318-324)
  - add:
      * /// Route to mode base layer (uses active mode)
      * #[arg(long)]
      * pub mode: bool,
      * /// Route to scope layer
      * #[arg(long, value_name = "SCOPE")]
      * pub scope: Option<String>,
      * /// Route to project layer
      * #[arg(long)]
      * pub project: bool,
      * /// Route to global layer
      * #[arg(long)]
      * pub global: bool,

COMMANDS_MODULE:
  - file: src/commands/mod.rs
  - add: pub mod import;
  - add: pub use import::execute as import_execute;

MAIN_DISPATCH:
  - file: src/main.rs
  - modify: Commands::Import(_) arm (lines 199-202)
  - pattern:
      Commands::Import(cmd) => match commands::import_execute(&cmd) {
          Ok(()) => ExitCode::SUCCESS,
          Err(e) => {
              eprintln!("Error: {}", e);
              ExitCode::FAILURE
          }
      },

ERROR_TYPES:
  - file: src/core/error.rs
  - add: FileNotTracked { path: String }
  - placement: Near FileNotFound variant
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating import.rs
cargo check --bin jin                    # Check compilation
cargo clippy --bin jin -W warnings      # Lint checking
cargo fmt --check src/commands/import.rs # Format check

# Expected: Zero errors, zero warnings. Fix any issues before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test import command specifically
cargo test --package jin-glm --lib commands::import::tests --verbose

# Test all commands to ensure no breakage
cargo test --package jin-glm --lib commands:: --verbose

# Run specific test patterns
cargo test test_import_to_project_base --verbose
cargo test test_import_file_not_tracked --verbose
cargo test test_import_multiple_files --verbose

# Expected: All tests pass. Import tests should cover:
# - Successful import to each layer type
# - Failure on untracked files
# - Failure on symlinks
# - Failure on binary files
# - Multiple file import
# - Context-based routing
# - Idempotent imports (same file twice)
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the CLI
cargo build --release

# Setup test workspace
cd /tmp
mkdir import-test && cd import-test
git init
echo "# Test" > README.md
git add README.md
git commit -m "Initial commit"

# Initialize Jin
./target/release/jin init

# Create and track a config file in Git
cat > config.toml << EOF
[settings]
enabled = true
EOF
git add config.toml
git commit -m "Add config"

# Test import to project layer
./target/release/jin import config.toml
# Expected: "Imported config.toml to project/import-test"

# Verify staging
./target/release/jin status
# Expected: Shows config.toml staged to project/import-test

# Test import failure on untracked file
echo "data" > untracked.txt
./target/release/jin import untracked.txt
# Expected: Error about file not being tracked

# Test import to mode layer (set mode first)
mkdir -p .jin/mode/claude
echo "claude" > .jin/context
./target/release/jin mode create claude
./target/release/jin mode use claude
./target/release/jin import config.toml --mode
# Expected: "Imported config.toml to mode/claude"

# Commit the import
./target/release/jin commit -m "Import config to mode layer"
# Expected: Commit succeeds, file now in Jin layer
```

### Level 4: End-to-End Scenario Testing

```bash
# Scenario: Full import workflow
cd /tmp
mkdir full-import-test && cd full-import-test
git init
echo "# Project" > README.md
git add . && git commit -m "Initial"

# Initialize Jin with active mode
./target/release/jin init
./target/release/jin mode create development
./target/release/jin mode use development

# Create multiple config files tracked by Git
cat > config.json << EOF
{"database": "postgres"}
EOF
cat > .env << EOF
DATABASE_URL=postgres://localhost
EOF
git add config.json .env
git commit -m "Add configs"

# Import both to mode layer
./target/release/jin import config.json .env --mode
# Expected: Both files imported, summary shown

# Verify staging
./target/release/jin status
# Expected: Both files staged to mode/development

# Commit
./target/release/jin commit -m "Import dev configs"

# Verify layer content
./target/release/jin log mode/development
# Expected: Shows commit with both files

# Scenario: Import failure modes
cd /tmp
mkdir fail-test && cd fail-test
git init
./target/release/jin init

# Untracked file should fail
echo "content" > untracked.txt
./target/release/jin import untracked.txt
# Expected: Error - file not tracked

# Symlink should fail
ln -s /etc/hosts hosts
git add hosts
./target/release/jin import hosts
# Expected: Error - symlinks not supported

# Binary file should fail
echo -ne "\x00\x01\x02" > binary.bin
git add binary.bin
./target/release/jin import binary.bin
# Expected: Error - binary files not supported
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test commands::import::tests`
- [ ] No clippy warnings: `cargo clippy`
- [ ] No format issues: `cargo fmt --check`
- [ ] Binary compiles: `cargo build --release`

### Feature Validation

- [ ] Import accepts one or more file paths
- [ ] Validates files are Git-tracked (rejects untracked)
- [ ] Routes to correct layer based on flags/context
- [ ] Creates StagedEntry for each imported file
- [ ] Persists to StagingIndex
- [ ] Shows console feedback and summary
- [ ] Returns appropriate exit codes

### Code Quality Validation

- [ ] Follows add.rs implementation pattern
- [ ] Layer routing matches add command exactly
- [ ] Error handling uses appropriate JinError variants
- [ ] File placement: src/commands/import.rs
- [ ] Module exports added to mod.rs
- [ ] Main dispatch wired correctly

### Edge Cases Covered

- [ ] Untracked files rejected
- [ ] Symlinks rejected
- [ ] Binary files rejected
- [ ] Multiple files imported correctly
- [ ] Context-based routing works
- [ ] Explicit flags override context
- [ ] Import is idempotent (same file twice)

### Documentation & Deployment

- [ ] Module docstring explains import functionality
- [ ] Function docstrings follow rustdoc conventions
- [ ] Error messages are clear and actionable
- [ ] Console output provides user feedback

---

## Anti-Patterns to Avoid

- **Don't duplicate validation logic**: Reuse functions from add.rs where possible
- **Don't skip layer routing**: Use exact same routing logic as add command
- **Don't invert all validations**: Only git-tracked check is inverted, others stay same
- **Don't forget to update CLI args**: ImportCommand needs routing flags added
- **Don't use sync file operations**: All file I/O should use standard Rust std::fs
- **Don't ignore context**: Must respect active mode/scope from ProjectContext
- **Don't hardcode layer paths**: Use Layer::storage_path() and Layer::git_ref()
- **Don't skip tests**: Test coverage should match add.rs comprehensiveness

---

## Confidence Score

**8/10** for one-pass implementation success

**Justification**:
- Clear reference implementation in add.rs to follow
- Well-defined data structures and APIs
- Layer routing logic is well-established
- Staging system patterns are consistent
- Only one new error variant needed

**Risk Factors**:
- CLI args need modification (ImportCommand currently has no flags)
- Validation is inverted from add - careful attention needed
- Main.rs dispatch requires careful update

**Mitigation**:
- PRP provides complete code patterns to follow
- Explicit gotchas section on inverted validation
- Step-by-step implementation tasks ordered by dependencies
- Comprehensive validation checklist

---

## Additional Notes

### Relationship to Other Commands

- **Inverse of `jin add`**: add handles untracked files, import handles tracked files
- **Precedes `jin commit`**: import stages files, commit finalizes them
- **Complements `jin export`**: export will write Jin files back to Git (future task)
- **Uses same routing as mode/scope**: consistent layer targeting

### Future Enhancements (Out of Scope)

- Automatic .gitignore updates to exclude imported files
- Batch import with glob patterns (e.g., `jin import *.toml`)
- Interactive layer selection when no flags provided
- Import with commit shortcut (`jin import -m "message"`)
- Dry-run mode to preview import changes

### Test Data Reference

Use these test file patterns from add.rs tests:
- Simple text: `config.toml`, `settings.json`
- Multi-line: `.env`, `config.yaml`
- Edge cases: Very long paths, special characters in filenames
