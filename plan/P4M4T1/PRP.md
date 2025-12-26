# PRP: Apply Command (P4.M4.T1)

---

## Goal

**Feature Goal**: Implement the `jin apply` command that merges all active layers (GlobalBase through ProjectBase, layers 1-7) based on the current mode/scope context and writes the consolidated configuration to the workspace directory (`.jin/workspace/`, Layer 9: WorkspaceActive).

**Deliverable**: A fully functional `jin apply` command that:
- Loads active mode/scope context from `.jin/context`
- Detects project name from Git remote or directory
- Merges active layers using `LayerMerge` orchestrator
- Serializes merged `MergeValue` results to workspace files
- Supports `--dry-run` for preview without modification
- Supports `--force` to bypass dirty workspace checks

**Success Definition**:
- All layers merge correctly per PRD §11 precedence rules
- Merged files written to `.jin/workspace/` preserving directory structure
- `--dry-run` shows what would change without writing files
- `--force` bypasses dirty workspace validation
- User sees clear summary of merge operation
- Unit tests cover merge execution, dry-run, and force modes

## User Persona

**Target User**: Developer working with Jin's multi-layer configuration system who needs to apply merged configurations to their workspace after changing modes, scopes, or committing layer updates.

**Use Case**: After committing staged files to layers or changing the active mode/scope context, the user needs to regenerate the workspace directory with the latest merged configuration.

**User Journey**:
1. Developer commits files to a layer: `jin commit -m "Add mode-specific config"`
2. Developer runs `jin apply` to regenerate workspace
3. Jin loads active context (mode/scope)
4. Jin merges layers 1-7 in precedence order
5. Jin writes merged result to `.jin/workspace/`
6. User sees confirmation of files applied

**Pain Points Addressed**:
- No manual merging of configuration files across layers
- Deterministic, reproducible workspace generation
- Clear preview of changes before applying (dry-run)
- Explicit apply workflow separates layer editing from workspace activation

## Why

- **Business value**: Enables Jin's core value prop of declarative, layered configuration merging into a usable workspace
- **Integration**: Completes the layer-to-workspace pipeline: add -> commit -> apply
- **Problems solved**:
  - Manual configuration file merging across layers is error-prone
  - No clear mechanism to activate merged configurations
  - Users need preview capability before applying changes

## What

The `jin apply` command merges active layers and writes the result to the workspace directory.

### Command Interface

```bash
# Apply merged layers to workspace
jin apply

# Show what would change without applying
jin apply --dry-run

# Force apply even if workspace has uncommitted changes
jin apply --force
```

### Success Criteria

- [ ] Loads active context from `.jin/context` (mode/scope)
- [ ] Detects project name from Git origin or directory
- [ ] Creates `LayerMerge` with project, mode, scope
- [ ] Calls `merge_all()` to get `IndexMap<String, MergeValue>`
- [ ] Serializes each `MergeValue` to appropriate format
- [ ] Creates directory structure under `.jin/workspace/`
- [ ] Writes files to workspace (unless --dry-run)
- [ ] `--dry-run` shows preview without modifications
- [ ] `--force` bypasses dirty workspace check
- [ ] Unit tests cover merge, dry-run, and force modes

## All Needed Context

### Context Completeness Check

**Passes "No Prior Knowledge" test**: This PRP provides complete file paths, exact code patterns, LayerMerge API usage, MergeValue serialization approach, workspace path constants, and testing patterns. An implementer needs only this PRP and codebase access.

### Documentation & References

```yaml
# MUST READ - Command implementation pattern
- file: src/commands/add.rs
  why: Complete command pattern (workspace_root, context, project detection, output)
  pattern: execute() -> detect_project_name() -> ProjectContext::load() -> operation -> save -> summary
  gotcha: detect_project_name() helper uses git remote or directory name

# MUST READ - Project context loading
- file: src/core/config.rs
  why: ProjectContext stores active mode/scope loaded from .jin/context
  pattern: ProjectContext::load(&Path) -> Result<ProjectContext>
  gotcha: context_path() is a class method: ProjectContext::context_path(&workspace_root)

# MUST READ - Layer definitions and workspace path
- file: src/core/layer.rs
  why: Layer enum with WORKSPACE_PATH constant and storage_path() method
  pattern: Layer::WORKSPACE_PATH = ".jin/workspace"
  gotcha: WorkspaceActive is NOT versioned (is_versioned() returns false)

# MUST READ - Layer merge orchestrator
- file: src/merge/layer.rs
  why: LayerMerge orchestrates merging of layers in precedence order
  pattern: LayerMerge::new(&repo, project).with_mode(mode).with_scope(scope).merge_all()
  gotcha: merge_all() returns IndexMap<String, MergeValue> (path -> merged value)
  critical:
    - Use: merger.determine_active_layers() to see what will be merged
    - Use: merger.merge_all() to get merged files
    - Skip: UserLocal and WorkspaceActive are NOT in Git, LayerMerge handles this

# MUST READ - MergeValue for serialization
- file: src/merge/value.rs
  why: MergeValue is the unified value type that needs serialization
  pattern: MergeValue derives Serialize, use serde_json::to_string_pretty() for JSON
  gotcha: For non-JSON formats, convert MergeValue to intermediate representation

# MUST READ - File format detection
- file: src/merge/layer.rs:42-78
  why: FileFormat enum with from_path() method for format detection
  pattern: FileFormat::from_path(path) -> FileFormat
  gotcha: Default to Text for unknown extensions

# MUST READ - Error handling
- file: src/core/error.rs
  why: JinError variants for proper error handling
  pattern: Use JinError::Message for custom errors
  gotcha: JinError::Io for file system errors

# MUST READ - CLI definition
- file: src/cli/args.rs:278-288
  why: ApplyCommand struct with force and dry_run flags
  pattern: #[arg(long)] for bool flags
  gotcha: Already wired in main.rs:156-159, just needs implementation

# MUST READ - Command module exports
- file: src/commands/mod.rs
  why: Need to add apply module and export execute function
  pattern: pub mod apply; pub use apply::execute as apply_execute;
  gotcha: Follow existing pattern for other commands

# MUST READ - JinRepo for Git operations
- file: src/git/repo.rs
  why: JinRepo wraps git2::Repository for layer operations
  pattern: JinRepo::open_or_create(&workspace_root)
  gotcha: Returns Result<JinRepo>
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── src/
│   ├── cli/
│   │   ├── args.rs          # ApplyCommand defined at line 278-288
│   │   └── mod.rs
│   ├── commands/
│   │   ├── add.rs           # Reference: command pattern
│   │   ├── commit.rs        # Reference: command pattern
│   │   ├── status.rs        # Reference: command pattern
│   │   ├── mod.rs           # ADD: pub mod apply; pub use apply::execute as apply_execute;
│   │   └── apply.rs         # CREATE THIS FILE
│   ├── core/
│   │   ├── config.rs        # ProjectContext::load()
│   │   ├── error.rs         # JinError variants
│   │   └── layer.rs         # Layer::WORKSPACE_PATH constant
│   ├── git/
│   │   └── repo.rs          # JinRepo::open_or_create()
│   ├── merge/
│   │   ├── layer.rs         # LayerMerge orchestrator
│   │   └── value.rs         # MergeValue with Serialize derive
│   └── main.rs              # Line 156-159: ApplyCommand dispatcher (add execute call)
└── plan/P4M4T1/
    ├── research/
    │   └── apply_command_research.md
    └── PRP.md               # This document
```

### Desired Codebase Tree (files to be added)

```bash
# NEW FILE TO CREATE
├── src/commands/apply.rs    # Main apply command implementation
│   ├── execute()            # Entry point
│   ├── detect_project_name() # Project name detection helper
│   ├── check_workspace_clean() # Dirty check (unless --force)
│   ├── serialize_to_format()  # MergeValue to file content
│   ├── apply_to_workspace()   # Write files to .jin/workspace/
│   └── tests module          # Unit tests

# MODIFY
├── src/commands/mod.rs      # Add: pub mod apply; pub use apply::execute as apply_execute;
├── src/main.rs              # Update: Commands::Apply(cmd) => match commands::apply_execute(&cmd)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: ProjectContext::load() needs &Path, not &PathBuf
let context = ProjectContext::load(&workspace_root)?;

// CRITICAL: context_path() is a class method on ProjectContext
let context_path = ProjectContext::context_path(&workspace_root);

// CRITICAL: LayerMerge::new() takes &JinRepo and project: Into<String>
let merger = LayerMerge::new(&repo, &project_name)
    .with_mode(&context.mode)
    .with_scope(&context.scope);

// CRITICAL: with_mode/with_scope take Option<&String> or impl Into<String>
// When context.mode is Option<String>, use context.mode.as_deref()
let merger = LayerMerge::new(&repo, &project_name)
    .with_mode(context.mode.as_deref())
    .with_scope(context.scope.as_deref());

// CRITICAL: merge_all() returns IndexMap<String, MergeValue>
// Keys are relative paths (e.g., "config/settings.json")
let merged_files = merger.merge_all()?;

// CRITICAL: MergeValue derives Serialize, use serde_json for JSON
let json_content = serde_json::to_string_pretty(&merge_value)?;

// CRITICAL: For non-JSON formats, need format-specific serialization
// YAML: Use serde_yml::to_string()
// TOML: Use toml::to_string_pretty()
// INI: Need custom serialization or write as key=value sections
// Text: Use String variant directly

// CRITICAL: Workspace path is .jin/workspace, not .jin/workspace/
// Files should be written to .jin/workspace/<relative_path>
let workspace_dir = workspace_root.join(".jin/workspace");
let file_path = workspace_dir.join(&relative_path);

// CRITICAL: Create parent directories before writing files
if let Some(parent) = file_path.parent() {
    std::fs::create_dir_all(parent)?;
}

// CRITICAL: JinRepo::open_or_create() takes &Path
let repo = JinRepo::open_or_create(&workspace_root)?;

// CRITICAL: detect_project_name() helper pattern (from add.rs)
// Try git remote origin, fallback to directory name
let project_name = detect_project_name(&workspace_root)?;

// CRITICAL: Check workspace dirty state unless --force
// Use StagingIndex to check for staged files
let staging = StagingIndex::load_from_disk(&workspace_root)?;
if !cmd.force && !staging.is_empty() {
    return Err(JinError::Message("Workspace has uncommitted changes. Use --force to override.".to_string()));
}

// CRITICAL: FileFormat::from_path() for format detection
let format = FileFormat::from_path(Path::new(&path));

// CRITICAL: MergeValue::String variant is for text files
if let MergeValue::String(content) = merge_value {
    // Write directly for text files
}
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed - using existing types:
- `ApplyCommand` (from `src/cli/args.rs`) - CLI arguments
- `ProjectContext` (from `src/core/config.rs`) - Active mode/scope
- `LayerMerge` (from `src/merge/layer.rs`) - Merge orchestrator
- `MergeValue` (from `src/merge/value.rs`) - Merged file content
- `FileFormat` (from `src/merge/layer.rs`) - Format detection
- `StagingIndex` (from `src/staging/index.rs`) - Dirty state checking

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/apply.rs with module structure
  - IMPLEMENT: Module skeleton with use statements
  - INCLUDE: crate::cli::args::ApplyCommand, crate::core::error::{JinError, Result}
  - INCLUDE: crate::core::{config::ProjectContext, layer::Layer}
  - INCLUDE: crate::git::JinRepo, crate::merge::{layer::LayerMerge, value::MergeValue}
  - INCLUDE: crate::staging::index::StagingIndex
  - INCLUDE: indexmap::IndexMap, std::path::{Path, PathBuf}
  - PATTERN: Follow src/commands/add.rs structure
  - NAMING: execute() function, helper functions below

Task 2: IMPLEMENT detect_project_name() helper function
  - SIGNATURE: fn detect_project_name(workspace_root: &Path) -> Result<String>
  - LOGIC: Try git remote origin URL, extract repo name
  - FALLBACK: Use workspace directory name
  - REFERENCE: src/commands/add.rs:236-261 (exact same pattern)
  - PATTERN:
    use git2::Repository;
    let repo = Repository::discover(workspace_root)?;
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = url.rsplit('/').next() {
                return Ok(name.trim_end_matches(".git").to_string());
            }
        }
    }
    workspace_root.file_name()...

Task 3: IMPLEMENT serialize_to_format() helper function
  - SIGNATURE: fn serialize_to_format(value: &MergeValue, format: &FileFormat, path: &str) -> Result<String>
  - LOGIC: Match on FileFormat variant, serialize MergeValue accordingly
  - JSON: serde_json::to_string_pretty(value)
  - YAML: serde_yml::to_string(value) [may need feature check]
  - TOML: toml::to_string_pretty(value) [may need conversion]
  - INI: Custom serialization (write sections as [header] key=value)
  - Text/Unknown: Extract String variant directly
  - ERROR: Return JinError::Message for unsupported variants
  - GOTCHA: INI requires special handling; for initial PRP, treat as simple text

Task 4: IMPLEMENT apply_to_workspace() helper function
  - SIGNATURE: fn apply_to_workspace(merged_files: &IndexMap<String, MergeValue>, workspace_root: &Path, dry_run: bool) -> Result<usize>
  - LOGIC:
    1. Create workspace directory if needed: .jin/workspace/
    2. For each (path, merge_value) in merged_files:
       a. Detect file format from extension
       b. Serialize MergeValue to format-specific content
       c. Build full output path: workspace_root/.jin/workspace/<path>
       d. Create parent directories if needed
       e. If dry_run: print what would change
       f. Otherwise: write file content
    3. Return count of files applied
  - PATTERN: Use std::fs::create_dir_all() for directories
  - PATTERN: Use std::fs::write() for file content
  - REFERENCE: src/commands/init.rs::update_gitignore() for directory creation pattern

Task 5: IMPLEMENT check_workspace_clean() helper function
  - SIGNATURE: fn check_workspace_clean(workspace_root: &Path) -> Result<bool>
  - LOGIC: Load StagingIndex, check if empty
  - RETURN: Ok(true) if clean, Ok(false) if dirty
  - REFERENCE: src/commands/commit.rs:73-79 for staging check pattern
  - PATTERN:
    let staging = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());
    Ok(staging.is_empty())

Task 6: IMPLEMENT execute() main function
  - SIGNATURE: pub fn execute(cmd: &ApplyCommand) -> Result<()>
  - STEP 1: Get workspace_root (std::env::current_dir()?)
  - STEP 2: Load ProjectContext from workspace_root
  - STEP 3: Detect project name using detect_project_name()
  - STEP 4: Open JinRepo using JinRepo::open_or_create()
  - STEP 5: Check workspace clean unless --force
  - STEP 6: Create LayerMerge with project, mode, scope
  - STEP 7: Call merge_all() to get IndexMap<String, MergeValue>
  - STEP 8: Print preview summary (files that will be merged)
  - STEP 9: If not --dry-run, call apply_to_workspace()
  - STEP 10: Print success summary
  - ERROR: Propagate all errors with context
  - PATTERN: Follow src/commands/add.rs::execute() structure
  - OUTPUT:
    println!("Applying {} file(s) to workspace...", merged_files.len());
    if dry_run { println!("(Dry run - no files written)"); }

Task 7: ADD src/commands/apply.rs to src/commands/mod.rs
  - ADD: pub mod apply;
  - EXPORT: pub use apply::execute as apply_execute;
  - PATTERN: Follow existing mod.rs structure (after status import)

Task 8: UPDATE command dispatcher in src/main.rs
  - MODIFY: Commands::Apply(cmd) branch (line 156-159)
  - CHANGE: From placeholder to:
    match commands::apply_execute(&cmd) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
  - PATTERN: Follow other command dispatchers (Add, Commit, Status)

Task 9: CREATE comprehensive unit tests in src/commands/apply.rs tests module
  - USE: tempfile::TempDir for isolated test directories
  - USE: DirGuard pattern for directory restoration
  - TEST: test_apply_with_empty_layers() - No files merged
  - TEST: test_apply_dry_run() - No files written
  - TEST: test_apply_force_dirty_workspace() - Bypasses dirty check
  - TEST: test_apply_rejects_dirty_workspace() - Error without --force
  - TEST: test_detect_project_name_from_git() - Git remote parsing
  - TEST: test_detect_project_name_from_directory() - Directory fallback
  - TEST: test_serialize_json() - JSON serialization
  - TEST: test_serialize_yaml() - YAML serialization
  - PATTERN: Follow src/commands/add.rs test structure

Task 10: RUN validation and fix any issues
  - LINT: cargo clippy --all-targets --all-features
  - FMT: cargo fmt
  - TEST: cargo test --lib apply
  - BUILD: cargo build --release
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Command execute() structure
pub fn execute(cmd: &ApplyCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Load project context
    let context = ProjectContext::load(&workspace_root)?;

    // 3. Detect project name
    let project_name = detect_project_name(&workspace_root)?;

    // 4. Open Jin repository
    let repo = JinRepo::open_or_create(&workspace_root)?;

    // 5. Check workspace state (unless --force)
    if !cmd.force {
        let clean = check_workspace_clean(&workspace_root)?;
        if !clean {
            return Err(JinError::Message(
                "Workspace has uncommitted changes. Use --force to override.".to_string()
            ));
        }
    }

    // 6. Create layer merger with context
    let merger = LayerMerge::new(&repo, &project_name)
        .with_mode(context.mode.as_deref())
        .with_scope(context.scope.as_deref());

    // 7. Merge all active layers
    let merged_files = merger.merge_all()?;

    // 8. Show preview
    println!("Applying {} file(s) to workspace...", merged_files.len());
    if cmd.dry_run {
        println!("(Dry run - no files will be written)");
    }
    for path in merged_files.keys() {
        println!("  {}", path);
    }

    // 9. Apply to workspace (unless dry-run)
    if !cmd.dry_run {
        let count = apply_to_workspace(&merged_files, &workspace_root, false)?;
        println!("\nApplied {} file(s) to workspace", count);
    } else {
        println!("\nDry run complete - no files written");
    }

    Ok(())
}

// Pattern 2: Project name detection (from add.rs)
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
        .ok_or_else(|| JinError::Message("Cannot determine project name".to_string()))
}

// Pattern 3: Workspace clean check
fn check_workspace_clean(workspace_root: &Path) -> Result<bool> {
    let staging = StagingIndex::load_from_disk(&workspace_root)
        .unwrap_or_else(|_| StagingIndex::new());
    Ok(staging.is_empty())
}

// Pattern 4: Serialize MergeValue to format
fn serialize_to_format(value: &MergeValue, format: &FileFormat, path: &str) -> Result<String> {
    match format {
        FileFormat::Json => {
            serde_json::to_string_pretty(value)
                .map_err(|e| JinError::Message(format!("JSON serialization error: {}", e)))
        }
        FileFormat::Yaml => {
            // Note: serde_yml may need feature flag
            #[cfg(feature = "yaml")]
            {
                serde_yml::to_string(value)
                    .map_err(|e| JinError::Message(format!("YAML serialization error: {}", e)))
            }
            #[cfg(not(feature = "yaml"))]
            {
                // Fallback: serialize as JSON then convert (not ideal but works)
                let json = serde_json::to_string_pretty(value)?;
                // TODO: Proper YAML serialization
                Err(JinError::Message("YAML support not enabled".to_string()))
            }
        }
        FileFormat::Toml => {
            toml::to_string_pretty(value)
                .map_err(|e| JinError::Message(format!("TOML serialization error: {}", e)))
        }
        FileFormat::Ini => {
            // INI requires custom serialization
            // For initial implementation, write as simple sections
            write_ini_format(value)
        }
        FileFormat::Text | FileFormat::Unknown => {
            // Extract string content
            if let MergeValue::String(content) = value {
                Ok(content.clone())
            } else {
                Err(JinError::Message(format!(
                    "Expected text content for {}: {:?}", path, value
                )))
            }
        }
    }
}

// Pattern 5: INI format helper (simplified)
fn write_ini_format(value: &MergeValue) -> Result<String> {
    if let MergeValue::Object(sections) = value {
        let mut output = String::new();
        for (section_name, section_value) in sections {
            if let MergeValue::Object(items) = section_value {
                output.push_str(&format!("[{}]\n", section_name));
                if let MergeValue::Object(settings) = items {
                    for (key, val) in settings {
                        let val_str = match val {
                            MergeValue::String(s) => s.clone(),
                            MergeValue::Integer(n) => n.to_string(),
                            MergeValue::Boolean(b) => b.to_string(),
                            MergeValue::Null => "false".to_string(),
                            _ => val.to_string(),
                        };
                        output.push_str(&format!("{} = {}\n", key, val_str));
                    }
                }
                output.push('\n');
            }
        }
        Ok(output)
    } else {
        Err(JinError::Message("INI format requires object with sections".to_string()))
    }
}

// Pattern 6: Apply merged files to workspace
fn apply_to_workspace(
    merged_files: &IndexMap<String, MergeValue>,
    workspace_root: &Path,
    dry_run: bool,
) -> Result<usize> {
    use crate::merge::layer::FileFormat;
    use std::fs;

    let workspace_dir = workspace_root.join(".jin/workspace");

    // Create workspace directory if it doesn't exist
    if !dry_run {
        fs::create_dir_all(&workspace_dir)?;
    }

    let mut count = 0;

    for (relative_path, merge_value) in merged_files {
        let path_obj = std::path::Path::new(relative_path);
        let format = FileFormat::from_path(path_obj);

        // Serialize to appropriate format
        let content = serialize_to_format(merge_value, &format, relative_path)?;

        // Build full output path
        let file_path = workspace_dir.join(relative_path);

        // Create parent directories
        if !dry_run {
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, content)?;
        } else {
            println!("Would write: {}", file_path.display());
        }

        count += 1;
    }

    Ok(count)
}
```

### Integration Points

```yaml
COMMAND_MODULE:
  - modify: src/commands/mod.rs
  - add: pub mod apply;
  - add: pub use apply::execute as apply_execute;
  - pattern: Follow existing module exports

MAIN_DISPATCHER:
  - modify: src/main.rs
  - update: Commands::Apply(cmd) branch (line 156-159)
  - pattern: match commands::apply_execute(&cmd) { Ok(()) => ..., Err(e) => ... }

LAYER_MERGE:
  - use: src/merge/layer.rs
  - method: LayerMerge::new(&repo, project)
  - method: .with_mode(mode)
  - method: .with_scope(scope)
  - method: .merge_all() -> IndexMap<String, MergeValue>

WORKSPACE_PATH:
  - constant: Layer::WORKSPACE_PATH = ".jin/workspace"
  - location: src/core/layer.rs:137

PROJECT_CONTEXT:
  - method: ProjectContext::load(&Path) -> Result<ProjectContext>
  - field: context.mode: Option<String>
  - field: context.scope: Option<String>

JINREPO:
  - method: JinRepo::open_or_create(&Path) -> Result<JinRepo>
  - use: For Git operations and layer reading

STAGING_INDEX:
  - method: StagingIndex::load_from_disk(&Path)
  - method: StagingIndex::is_empty()
  - use: For checking workspace dirty state
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
# Test apply command specifically
cargo test --lib apply

# Test all command tests
cargo test --lib commands

# Test with output
cargo test --lib apply -- --nocapture

# Expected: All tests pass
# Key test cases:
# - test_apply_with_empty_layers: Handles empty merge result
# - test_apply_dry_run: No files written on dry_run
# - test_apply_force_dirty_workspace: Bypasses dirty check
# - test_apply_rejects_dirty_workspace: Error without --force
# - test_detect_project_name_from_git: Git remote parsing
# - test_detect_project_name_from_directory: Directory fallback
# - test_serialize_json: JSON serialization works
# - test_serialize_yaml: YAML serialization works
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the project
cargo build --release

# Manual integration test in a temporary directory
cd /tmp
rm -rf test_jin_apply
mkdir test_jin_apply && cd test_jin_apply
git init
jin init                     # Initialize Jin

# Create and commit a config to a layer
echo "global = true" > global.conf
jin add global.conf --global
jin commit -m "Add global config"

# Apply to workspace
jin apply                    # Should create .jin/workspace/global.conf

# Verify workspace
cat .jin/workspace/global.conf  # Should contain merged content

# Test dry-run
jin apply --dry-run          # Should show preview without changes

# Test force flag
echo "modified" > local.conf
jin add local.conf           # Stage a file (dirty workspace)
jin apply --force            # Should succeed despite dirty state

# Test error without force
jin apply                    # Should fail with dirty workspace error

# Expected: All manual tests pass, correct merge, proper dry-run behavior
```

### Level 4: CLI & Domain-Specific Validation

```bash
# Test with mode and scope context
jin init
jin mode create claude
jin mode use claude
jin scope create language:rust
jin scope use language:rust

# Create configs in different layers
echo "mode_setting = 1" > mode.conf
jin add mode.conf --mode
jin commit -m "Add mode config"

echo "scope_setting = 2" > scope.conf
jin add scope.conf --scope=language:rust
jin commit -m "Add scope config"

# Apply should merge mode + scope configs
jin apply

# Verify workspace has both files
ls .jin/workspace/
# Expected: mode.conf and scope.conf present

# Test with project layer
echo "project_setting = 3" > project.conf
jin add project.conf
jin commit -m "Add project config"

# Apply should merge all three layers
jin apply

# Verify workspace has all files
ls .jin/workspace/
# Expected: mode.conf, scope.conf, and project.conf present

# Test empty workspace scenario
cd /tmp/empty_test
git init
jin init
jin apply                    # Should succeed with "0 file(s)" message

# Expected: All scenarios work correctly
```

## Final Validation Checklist

### Technical Validation

- [ ] Code compiles: `cargo build --release`
- [ ] No clippy warnings: `cargo clippy --all-targets`
- [ ] Code formatted: `cargo fmt --check`
- [ ] All tests pass: `cargo test --lib`
- [ ] No unused imports or dead code

### Feature Validation

- [ ] Loads active mode/scope from `.jin/context`
- [ ] Detects project name from Git origin or directory name
- [ ] Creates LayerMerge with correct parameters
- [ ] Calls `merge_all()` and gets `IndexMap<String, MergeValue>`
- [ ] Serializes files to correct formats (JSON, YAML, TOML, INI, Text)
- [ ] Creates directory structure under `.jin/workspace/`
- [ ] `--dry-run` shows preview without writing files
- [ ] `--force` bypasses dirty workspace check
- [ ] Errors when workspace dirty without `--force`
- [ ] Shows clear summary of files applied
- [ ] Handles empty merge result gracefully

### Code Quality Validation

- [ ] Follows `src/commands/add.rs` patterns
- [ ] Proper error handling with `Result<>`
- [ ] Comprehensive doc comments on public functions
- [ ] Unit tests cover all major scenarios
- [ ] Tests use `tempfile` and `DirGuard` pattern
- [ ] No `unwrap()` calls (use proper error handling)
- [ ] No `expect()` calls (use proper error handling)

### Documentation & Deployment

- [ ] `execute()` has comprehensive doc comment
- [ ] Helper functions have doc comments
- [ ] User-facing error messages are clear
- [ ] Success output shows file count and workspace path

---

## Anti-Patterns to Avoid

- **Don't** skip workspace dirty check - require `--force` for dirty workspaces
- **Don't** use `unwrap()` or `expect()` - propagate errors with `?`
- **Don't** forget to create parent directories - use `std::fs::create_dir_all()`
- **Don't** serialize all files as JSON - respect original file formats
- **Don't** ignore the `--dry-run` flag - must not write files when set
- **Don't** bypass `ProjectContext` - always load active mode/scope
- **Don't** hardcode workspace path - use `Layer::WORKSPACE_PATH` constant
- **Don't** forget to update `src/commands/mod.rs` - add module and export
- **Don't** forget to update `src/main.rs` dispatcher - wire up execute function
- **Don't** write files in dry-run mode - print what would change instead
- **Don't** skip INI serialization - handle it or provide clear error
- **Don't** ignore empty merge result - handle gracefully with "0 file(s)" message

## Confidence Score: 8/10

**Reasoning**:
- Complete codebase analysis with specific file references
- All required infrastructure (LayerMerge, MergeValue, ProjectContext) exists
- Clear implementation pattern from existing commands to follow
- Specific helper functions and serialization approach documented
- Unit test pattern established from other commands

**Remaining risks**:
1. **YAML/TOML serialization**: May need feature flag checks or library-specific handling
2. **INI serialization**: Custom implementation required, format may need iteration
3. **MergeValue to format conversion**: Some formats (YAML/TOML) may require intermediate representations

**Mitigation**: Start with JSON and Text formats (guaranteed to work), add YAML/TOML/INI with proper error handling for missing features. The PRP provides clear serialization patterns for all formats.
