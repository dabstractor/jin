# Product Requirement Prompt (PRP): Export Command

**Work Item**: P4.M5.T5 - Export Command
**Task**: Wire ExportCommand - Export Jin files back to Git
**Points**: 1

---

## Goal

**Feature Goal**: Implement the `jin export` command that writes files from Jin's workspace directory back to the Git working directory, enabling users to transition Jin-managed configurations back to standard Git tracking.

**Deliverable**: A fully functional `jin export` command that:
1. Validates files exist in the Jin workspace (`.jin/workspace/`)
2. Reads file content from workspace
3. Writes files to the Git working directory
4. Stages exported files in Git (optional/automatic)
5. Optionally updates .gitignore to remove files from Jin managed block

**Success Definition**:
- `jin export <file>...` copies files from `.jin/workspace/` to Git working directory
- Files are validated to exist in workspace before export
- Exported files are written to their original locations in the Git working tree
- Files are optionally staged in Git for commit
- User receives clear feedback on what was exported and where
- Integration tests cover happy path and edge cases

---

## User Persona

**Target User**: Developer transitioning configuration files from Jin management back to Git tracking

**Use Case**: Developer has configuration files managed in Jin layers and wants to move them back to standard Git tracking - for example, when:
- A configuration becomes project-standard rather than personal/tool-specific
- A project is being shared with team members who don't use Jin
- A tooling configuration is ready to be committed to the project repository

**User Journey**:
1. User has files managed in Jin workspace (`.jin/workspace/`)
2. User runs `jin export config.toml` to export the file back to Git
3. Jin validates the file exists in the workspace
4. Jin copies the file from workspace to the Git working directory
5. Jin optionally stages the file in Git (using `git add`)
6. User can commit the file with standard Git: `git commit -m "Add config.toml"`
7. User may remove file from Jin layers: `jin rm config.toml && jin commit -m "Remove from Jin"`

**Pain Points Addressed**:
- **Manual file copying**: No need to manually copy files from `.jin/workspace/` to project root
- **Forgotten files**: Clear export process ensures all intended files are moved
- **Git workflow integration**: Seamless integration with existing Git commit workflow
- **Configuration promotion**: Clear path for promoting personal/tool configs to project-standard

---

## Why

- **Bidirectional workflow**: Jin should support both import (Git→Jin) and export (Jin→Git) workflows
- **Configuration lifecycle**: Files may need to move between personal and project management over time
- **Team collaboration**: Enables sharing configurations when they become project-standard
- **Jin flexibility**: Users shouldn't be locked into Jin - easy exit path is important
- **Consistency with import**: Export completes the import/export symmetry
- **Git integration**: Direct Git staging makes export immediately useful

---

## What

User-visible behavior:
```bash
# Export a single file from workspace to Git
$ jin export config.toml
Exported config.toml to Git working directory
Staged in Git: config.toml

# Export multiple files
$ jin export config.json .env settings.yaml
Exported config.json to Git working directory
Exported .env to Git working directory
Exported settings.yaml to Git working directory
Staged 3 files in Git

# Verify Git status
$ git status
Changes to be committed:
  new file:   config.json
  new file:   .env
  new file:   settings.yaml

# Commit the exported files
$ git commit -m "Add configuration files"
```

### Success Criteria

- [ ] Command accepts one or more file paths as arguments
- [ ] Validates files exist in `.jin/workspace/` directory
- [ ] Reads file content from workspace location
- [ ] Writes files to Git working directory at their original paths
- [ ] Creates parent directories if needed
- [ ] Optionally stages exported files in Git using `git2::Index`
- [ ] Provides clear console feedback on export success
- [ ] Shows summary of exported files
- [ ] Returns appropriate error codes for failures
- [ ] Handles conflicts (warns if working directory file differs from workspace)

---

## All Needed Context

### Context Completeness Check

If someone knew nothing about this codebase, they would have everything needed to implement this successfully. The PRP includes:
- Exact file patterns to follow from existing commands
- Complete workspace structure understanding
- Git staging patterns using git2-rs
- File I/O patterns from apply.rs (reversed for export)
- Error handling conventions
- Test patterns from the codebase

### Documentation & References

```yaml
# MUST READ - Command implementation patterns
- file: src/commands/apply.rs
  why: Shows file writing patterns, serialization, workspace directory handling
  pattern: apply_to_workspace() function (lines 246-284) - reverse this for export
  gotcha: Export reads FROM workspace, writes TO Git working directory

- file: src/commands/diff.rs
  why: Simple command pattern with Git integration
  pattern: Command structure, error handling, Git repo validation

- file: src/commands/import.rs
  why: Shows file validation and Git staging patterns
  pattern: Git repository operations using git2::Repository

- file: src/commands/mod.rs
  why: Shows how to export command from module
  pattern: `pub use export::execute as export_execute;`

- file: src/cli/args.rs
  why: ExportCommand already defined (lines 342-348)
  pattern: `#[derive(clap::Args)]`, `#[arg(value_name = "FILE", num_args(1..))]`
  gotcha: ExportCommand struct exists - no CLI changes needed initially

# MUST READ - Workspace and Git patterns
- file: src/core/config.rs
  why: ProjectContext::load() for context, workspace path resolution
  pattern: `.jin/workspace/` is workspace_root.join(".jin/workspace")

- file: src/commands/apply.rs
  why: serialize_to_format() shows format detection and serialization
  pattern: FileFormat::from_path() for format detection
  lines: 161-195

# MUST READ - Git staging patterns (git2-rs)
- url: https://docs.rs/git2/latest/git2/struct.Index.html
  why: Git index operations for staging files
  critical: index.add_path(), index.write() methods

- url: https://docs.rs/git2/latest/git2/struct.Repository.html
  why: Repository operations for opening repo and accessing index
  critical: repo.index(), repo.workdir() methods

# MUST READ - Main dispatch wiring
- file: src/main.rs
  why: Shows command dispatch pattern
  pattern: Match on Commands::Export(cmd), call commands::export_execute(&cmd)
  lines: 203-206 (placeholder to replace)
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── args.rs          # ExportCommand struct defined (lines 342-348)
│   └── mod.rs
├── commands/
│   ├── add.rs           # File validation patterns
│   ├── apply.rs         # File writing patterns (reverse for export)
│   ├── commit.rs
│   ├── context.rs       # Simple command pattern
│   ├── diff.rs          # Git integration pattern
│   ├── import.rs        # Git staging patterns
│   ├── init.rs
│   ├── log.rs
│   ├── mod.rs           # Add: pub use export::execute as export_execute;
│   ├── reset.rs
│   ├── scope.rs
│   └── status.rs
├── core/
│   ├── config.rs        # ProjectContext, workspace paths
│   ├── error.rs         # JinError types
│   └── ...
├── git/
│   └── repo.rs          # JinRepo wrapper
├── merge/
│   ├── layer.rs         # FileFormat enum
│   └── value.rs         # MergeValue types
└── main.rs              # Command dispatch (update lines 203-206)
```

### Desired Codebase Tree (Files to Add)

```bash
src/
├── commands/
│   ├── export.rs        # NEW - Export command implementation
│   └── mod.rs           # MODIFY - Add export for export_execute
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Export is the REVERSE of apply command
// apply.rs: reads merged files, writes to .jin/workspace/
// export.rs: reads from .jin/workspace/, writes to Git working directory

// CRITICAL: Workspace path structure
// Source: workspace_root.join(".jin/workspace").join(relative_path)
// Target: workspace_root.join(relative_path)

// CRITICAL: Git staging using git2-rs
// Use git2::Repository::index() to get the index
// Use index.add_path(Path::new(relative_path)) to stage
// Use index.write() to persist the index

// CRITICAL: Paths must be relative for Git staging
// git2::Index::add_path() expects paths relative to repo root
// Use path.strip_prefix(workspace_root) to get relative path

// CRITICAL: File format handling
// Files in workspace are already serialized (JSON, YAML, TOML, etc.)
// No re-serialization needed - just read and write as-is
// This is simpler than apply which has to serialize MergeValues

// CRITICAL: Parent directory creation
// Use std::fs::create_dir_all() for parent directories
// Check if parent exists before creating

// CRITICAL: Error handling for conflicts
// If file exists in Git working directory and differs from workspace:
// - Warn user but don't abort (let user decide)
// - Or use --force flag to overwrite (future enhancement)

// CRITICAL: .gitignore handling (optional, out of scope for MVP)
// May want to remove exported files from Jin managed block
// This would require parsing and rewriting .gitignore
```

---

## Implementation Blueprint

### Data Models

No new data models needed. Uses existing types:
- `ExportCommand` from `cli::args` (already defined)
- `ProjectContext` for workspace path resolution
- `JinError` for error handling

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/commands/export.rs
  - IMPLEMENT: execute() function with ExportCommand parameter
  - FOLLOW pattern: src/commands/apply.rs (reverse of apply_to_workspace)
  - NAMING: pub fn execute(cmd: &ExportCommand) -> Result<()>
  - FILE_STRUCTURE:
    * Module docstring explaining export functionality
    * execute() as main entry point
    * export_file() helper for single file export
    * stage_in_git() helper for Git staging
    * validate_in_workspace() helper for file validation
  - DEPENDENCIES: crate::cli::args::ExportCommand, core types, git2, std::fs

Task 2: MODIFY src/commands/mod.rs
  - ADD: pub mod export;
  - ADD: pub use export::execute as export_execute;
  - FOLLOW pattern: Existing module exports (lines 4-28)
  - PLACEMENT: After diff.rs, before import.rs (alphabetical)

Task 3: MODIFY src/main.rs
  - REPLACE: Lines 203-206 (placeholder) with actual dispatch
  - FOLLOW pattern: Commands::Diff dispatch (lines 167-172)
  - IMPLEMENT: match commands::export_execute(&cmd) with error handling
  - RETURNS: ExitCode::SUCCESS on Ok, ExitCode::FAILURE on Err

Task 4: CREATE src/commands/export.rs tests module
  - IMPLEMENT: Unit tests following apply.rs pattern (lines 286-632)
  - TEST CASES:
    * test_export_single_file
    * test_export_multiple_files
    * test_export_creates_directories
    * test_export_file_not_in_workspace (should fail)
    * test_export_stages_in_git
    * test_export_with_conflict (warns user)
    * test_export_nested_path
    * test_export_is_idempotent
  - FIXTURES: DirGuard, create_test_file, init_git_repo, init_jin, init_workspace
```

### Implementation Patterns & Key Details

```rust
// ===== MAIN EXECUTION PATTERN =====
pub fn execute(cmd: &ExportCommand) -> Result<()> {
    // 1. Get workspace root
    let workspace_root = std::env::current_dir()?;

    // 2. Verify Jin is initialized
    let context_path = ProjectContext::context_path(&workspace_root);
    if !context_path.exists() {
        return Err(JinError::Message(
            "Jin is not initialized in this directory.\n\
             Run 'jin init' to initialize.".to_string(),
        ));
    }

    // 3. Open Git repository for staging
    let git_repo = git2::Repository::discover(&workspace_root).map_err(|_| {
        JinError::Message("Not a Git repository".to_string())
    })?;

    // 4. Get workspace directory
    let workspace_dir = workspace_root.join(".jin/workspace");

    // 5. Track exported files
    let mut exported_files = Vec::new();

    // 6. Process each file
    for file_path in &cmd.files {
        // Validate and export
        let exported = export_file(&workspace_root, &workspace_dir, &git_repo, file_path)?;
        exported_files.push(exported);
    }

    // 7. Stage all exported files in Git
    if !exported_files.is_empty() {
        stage_files_in_git(&git_repo, &exported_files)?;
    }

    // 8. Print summary
    println!("\nExported {} file(s) to Git working directory", exported_files.len());
    println!("Staged in Git:");
    for file in &exported_files {
        println!("  {}", file.display());
    }

    Ok(())
}

// ===== EXPORT SINGLE FILE =====
fn export_file(
    workspace_root: &Path,
    workspace_dir: &Path,
    git_repo: &git2::Repository,
    file_path: &Path,
) -> Result<PathBuf> {
    // Resolve to absolute path if relative
    let resolved_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        workspace_root.join(file_path)
    };

    // Get relative path for Git operations
    let relative_path = resolved_path.strip_prefix(workspace_root).map_err(|_| {
        JinError::Message(format!(
            "File is outside workspace root: {}",
            resolved_path.display()
        ))
    })?;

    // CRITICAL: Check file exists in workspace
    let workspace_file_path = workspace_dir.join(relative_path);
    if !workspace_file_path.exists() {
        return Err(JinError::Message(format!(
            "File not found in Jin workspace: {}\n\
             Use 'jin status' to see managed files.",
            file_path.display()
        )));
    }

    // Read content from workspace
    let content = std::fs::read(&workspace_file_path)?;

    // Build target path in Git working directory
    let target_path = workspace_root.join(relative_path);

    // Check for conflict (file exists and differs)
    if target_path.exists() {
        let existing_content = std::fs::read(&target_path)?;
        if existing_content != content {
            eprintln!("Warning: {} differs between workspace and Git working directory",
                      target_path.display());
            }
    }

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to Git working directory
    std::fs::write(&target_path, content)?;

    println!("Exported {} to Git working directory", file_path.display());

    Ok(relative_path.to_path_buf())
}

// ===== STAGE FILES IN GIT =====
fn stage_files_in_git(git_repo: &git2::Repository, files: &[PathBuf]) -> Result<()> {
    // Get the Git index
    let mut index = git_repo.index().map_err(|e| {
        JinError::Message(format!("Failed to get Git index: {}", e))
    })?;

    // Add each file to the index
    for file_path in files {
        // PATTERN: Git expects relative paths
        if let Err(e) = index.add_path(file_path) {
            eprintln!("Warning: Failed to stage {}: {}", file_path.display(), e);
        }
    }

    // Write the index to persist changes
    index.write().map_err(|e| {
        JinError::Message(format!("Failed to write Git index: {}", e))
    })?;

    Ok(())
}
```

### Integration Points

```yaml
COMMANDS_MODULE:
  - file: src/commands/mod.rs
  - add: pub mod export;
  - add: pub use export::execute as export_execute;

MAIN_DISPATCH:
  - file: src/main.rs
  - modify: Commands::Export(_) arm (lines 203-206)
  - pattern:
      Commands::Export(cmd) => match commands::export_execute(&cmd) {
          Ok(()) => ExitCode::SUCCESS,
          Err(e) => {
              eprintln!("Error: {}", e);
              ExitCode::FAILURE
          }
      },

CLI_ARGS:
  - file: src/cli/args.rs
  - note: ExportCommand already defined (lines 342-348)
  - optional: Add flags in future for --no-stage, --force-overwrite

ERROR_TYPES:
  - file: src/core/error.rs
  - note: Existing JinError::Message variant sufficient for errors
  - optional: Add ExportFailed { path: String } variant for clarity
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating export.rs
cargo check --bin jin                    # Check compilation
cargo clippy --bin jin -W warnings      # Lint checking
cargo fmt --check src/commands/export.rs # Format check

# Expected: Zero errors, zero warnings. Fix any issues before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test export command specifically
cargo test --package jin-glm --lib commands::export::tests --verbose

# Test all commands to ensure no breakage
cargo test --package jin-glm --lib commands:: --verbose

# Run specific test patterns
cargo test test_export_single_file --verbose
cargo test test_export_file_not_in_workspace --verbose
cargo test test_export_creates_directories --verbose

# Expected: All tests pass. Export tests should cover:
# - Successful export of single and multiple files
# - Failure when file not in workspace
# - Parent directory creation
# - Git staging verification
# - Conflict detection and warnings
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the CLI
cargo build --release

# Setup test workspace with Jin
cd /tmp
mkdir export-test && cd export-test
git init
echo "# Test" > README.md
git add README.md
git commit -m "Initial commit"

# Initialize Jin
./target/release/jin init

# Create a file in workspace manually
mkdir -p .jin/workspace
cat > .jin/workspace/config.toml << EOF
[settings]
enabled = true
EOF

# Test export
./target/release/jin export config.toml
# Expected: "Exported config.toml to Git working directory"
# Expected: "Staged in Git: config.toml"

# Verify file exists in Git working directory
cat config.toml
# Expected: Shows content from workspace

# Verify Git staging
git status
# Expected: "new file: config.toml" in staging area

# Test export multiple files
cat > .jin/workspace/.env << EOF
DATABASE_URL=postgres://localhost
EOF

cat > .jin/workspace/settings.json << EOF
{"api_key": "test"}
EOF

./target/release/jin export .env settings.json
# Expected: Both files exported and staged

# Verify all files staged
git status
# Expected: Shows all 3 files staged
```

### Level 4: End-to-End Scenario Testing

```bash
# Scenario: Full export workflow
cd /tmp
mkdir full-export-test && cd full-export-test
git init
echo "# Project" > README.md
git add . && git commit -m "Initial"

# Initialize Jin and add files
./target/release/jin init

# Create files via Jin workflow
cat > config.json << EOF
{"database": "postgres"}
EOF
./target/release/jin add config.json
./target/release/jin commit -m "Add config to Jin"

# Apply to workspace
./target/release/jin apply

# Now export back to Git
./target/release/jin export config.json

# Verify and commit
git status
git commit -m "Promote config.json to project standard"

# Verify Git history
git log --oneline
# Expected: Both Jin and Git commits present

# Scenario: Export failure modes
cd /tmp
mkdir fail-export-test && cd fail-export-test
git init
./target/release/jin init

# File not in workspace should fail
echo "content" > config.toml
./target/release/jin export config.toml
# Expected: Error - file not found in workspace

# Create workspace file and export successfully
mkdir -p .jin/workspace
cat > .jin/workspace/config.toml << EOF
[test]
key = value
EOF
./target/release/jin export config.toml
# Expected: Success
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All unit tests pass: `cargo test commands::export::tests`
- [ ] No clippy warnings: `cargo clippy`
- [ ] No format issues: `cargo fmt --check`
- [ ] Binary compiles: `cargo build --release`

### Feature Validation

- [ ] Export accepts one or more file paths
- [ ] Validates files exist in workspace
- [ ] Reads content from workspace directory
- [ ] Writes files to Git working directory
- [ ] Creates parent directories as needed
- [ ] Stages exported files in Git
- [ ] Shows console feedback and summary
- [ ] Returns appropriate exit codes
- [ ] Warns on conflicts (file differs between workspace and working directory)

### Code Quality Validation

- [ ] Follows apply.rs pattern (reversed)
- [ ] Error handling uses appropriate JinError variants
- [ ] File placement: src/commands/export.rs
- [ ] Module exports added to mod.rs
- [ ] Main dispatch wired correctly

### Edge Cases Covered

- [ ] Files not in workspace rejected
- [ ] Nested directory paths handled
- [ ] Multiple files exported correctly
- [ ] Parent directories created automatically
- [ ] Conflicts detected and warnings shown
- [ ] Git staging works correctly
- [ ] Idempotent exports (same file twice)

### Documentation & Deployment

- [ ] Module docstring explains export functionality
- [ ] Function docstrings follow rustdoc conventions
- [ ] Error messages are clear and actionable
- [ ] Console output provides user feedback

---

## Anti-Patterns to Avoid

- **Don't re-serialize files**: Files in workspace are already in correct format - just read/write
- **Don't use absolute paths for Git**: git2::Index needs relative paths from repo root
- **Don't forget parent directories**: Use create_dir_all() before writing files
- **Don't ignore conflicts**: Warn users when workspace and working directory files differ
- **Don't skip Git staging**: Export should stage files so users can immediately commit
- **Don't hardcode paths**: Use ProjectContext for workspace root resolution
- **Don't forget to update index**: After index.add_path(), call index.write()
- **Don't skip validation**: Always verify file exists in workspace before export

---

## Confidence Score

**9/10** for one-pass implementation success

**Justification**:
- Simple, well-defined operation (copy workspace → Git working directory)
- Clear pattern from apply.rs to reverse
- Git staging is straightforward with git2-rs
- No new data models or complex logic needed
- Existing error handling patterns sufficient

**Risk Factors**:
- Git index API requires careful path handling (relative vs absolute)
- Conflict handling may need user consideration

**Mitigation**:
- PRP provides complete code patterns
- Explicit gotchas section on Git path handling
- Step-by-step implementation tasks
- Comprehensive validation checklist

---

## Additional Notes

### Relationship to Other Commands

- **Inverse of `jin import`**: import brings Git files into Jin, export sends Jin files to Git
- **Complements `jin apply`**: apply writes to workspace, export reads from workspace
- **Precedes standard Git commit**: export stages files, user commits with `git commit`
- **Enables `jin rm`**: After export, user can remove file from Jin layers

### Future Enhancements (Out of Scope)

- `--no-stage` flag to skip Git staging
- `--force` flag to overwrite differing files
- Automatic removal from .gitignore managed block
- Batch export with glob patterns
- Dry-run mode to preview exports
- Export with layer targeting (export from specific layer)
- Interactive conflict resolution

### Git Staging Reference

```rust
// Complete example from git2-rs documentation
use git2::{Repository, Index};

fn stage_file(repo: &Repository, path: &Path) -> Result<(), git2::Error> {
    let mut index = repo.index()?;

    // Add file to staging area
    index.add_path(path)?;

    // Write index to disk
    index.write()?;

    Ok(())
}

// Usage in export:
stage_files_in_git(&git_repo, &exported_files)?;
```

### External Resources

- **git2-rs Documentation**: https://docs.rs/git2/
- **Git Index Operations**: https://docs.rs/git2/latest/git2/struct.Index.html
- **libgit2 Documentation**: https://libgit2.org/libgit2/

### Test Data Reference

Use these test file patterns:
- Simple config: `config.toml`, `settings.json`
- Environment: `.env`
- Nested paths: `config/development/database.json`
- Special characters: filenames with spaces (if supported)
