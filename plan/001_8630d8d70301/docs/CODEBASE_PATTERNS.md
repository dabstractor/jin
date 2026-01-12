# Jin Codebase Patterns for Workspace Commands

## Command Implementation Pattern

### File Structure
- **Location:** `src/commands/<command_name>.rs`
- **Entry point:** Public `execute(args: <Command>Args) -> Result<()>` function
- **Exports:** Re-exported through `src/commands/mod.rs`

### Standard Command Flow
```rust
pub fn execute(args: CommandArgs) -> Result<()> {
    // 1. Validate inputs
    validate_args(&args)?;

    // 2. Load context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => return Err(JinError::NotInitialized),
        Err(_) => ProjectContext::default(),
    };

    // 3. Open repository
    let repo = JinRepo::open_or_create()?;

    // 4. Perform operation (with transactions if modifying refs)

    // 5. Report results
    println!("Operation successful");
    Ok(())
}
```

## CLI Argument Patterns

### Location
- **File:** `src/cli/args.rs`
- **Pattern:** Use `#[derive(Args, Debug)]` with `clap::Args`

### Existing Workspace Command Args

**ApplyArgs:**
```rust
pub struct ApplyArgs {
    #[arg(long)]
    pub force: bool,        // Force apply even if workspace dirty

    #[arg(long)]
    pub dry_run: bool,      // Show what would be applied
}
```

**ResetArgs:**
```rust
pub struct ResetArgs {
    #[arg(long)]
    pub soft: bool,         // Keep changes in staging

    #[arg(long)]
    pub mixed: bool,        // Unstage but keep in workspace (default)

    #[arg(long)]
    pub hard: bool,         // Discard all changes

    #[arg(long)]
    pub mode: bool,         // Reset mode layer

    #[arg(long)]
    pub scope: Option<String>,  // Reset scope layer

    #[arg(long)]
    pub project: bool,      // Reset project layer
}
```

## Workspace and Staging System

### Key Files
- `src/staging/workspace.rs` - File operations (read/write workspace files)
- `src/staging/index.rs` - Staging index management
- `src/staging/entry.rs` - Staged entry types
- `src/staging/router.rs` - Layer routing logic
- `src/staging/gitignore.rs` - .gitignore management

### Staging Index Pattern
```rust
// Load staging index
let mut staging = StagingIndex::load()
    .unwrap_or_else(|_| StagingIndex::new());

// Access entries
let entries = staging.entries_for_layer(Layer::ProjectBase);

// Modify
staging.add(entry);
staging.remove(&path);

// Save
staging.save()?;
```

### Workspace File Operations
```rust
// Read from workspace (project working directory)
let content = workspace::read_file(&workspace_path)?;

// Create blob in Jin repo
let oid = jin_repo.create_blob(&content)?;

// CRITICAL: Jin repo is BARE (no working directory)
// Always read from workspace path, create blobs in Jin repo
```

## Layer Merge System

### Key File
- `src/merge/layer.rs` - Layer merge orchestration

### Pattern
```rust
// Determine applicable layers from context
let layers = determine_active_layers(&context)?;

// Merge all layers
let merged_result = merge_layers(&config, &repo)?;

// Result contains:
// - merged_files: HashMap<PathBuf, MergedFile>
// - conflict_files: Vec<PathBuf>
// - added_files: Vec<PathBuf>
// - removed_files: Vec<PathBuf>
```

## Transaction System Patterns

### File
- `src/git/transaction.rs`

### LayerTransaction (Recommended for Multi-Layer Operations)
```rust
// Begin transaction
let mut tx = LayerTransaction::begin(&repo, "Update layers")?;

// Queue layer updates
tx.add_layer_update(
    Layer::ModeBase,
    context.mode.as_deref(),
    None,
    None,
    commit_oid
)?;

// Atomic commit - all succeed or all fail
tx.commit()?;
```

### Transaction Features
- Two-phase commit with persistent log at `.jin/.transaction_in_progress`
- Crash recovery via `RecoveryManager::auto_recover()`
- Automatic rollback on failure
- Best-effort ref restoration

### Recovery Pattern (Should be added to lib.rs)
```rust
pub fn run(cli: cli::Cli) -> anyhow::Result<()> {
    // Auto-recover incomplete transactions BEFORE any operation
    if let Ok(repo) = JinRepo::open() {
        if let Ok(recovered) = RecoveryManager::auto_recover(&repo) {
            if recovered {
                eprintln!("Recovered from incomplete transaction");
            }
        }
    }

    commands::execute(cli).map_err(|e| anyhow::anyhow!("{}", e))
}
```

## Error Handling Patterns

### Error Type
```rust
// From src/core/error.rs
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JinError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Jin not initialized in this project")]
    NotInitialized,

    #[error("No active {context_type}")]
    NoActiveContext { context_type: String },

    // ... more variants
}

pub type Result<T> = std::result::Result<T, JinError>;
```

### Error Handling Strategies

**Early return on critical errors:**
```rust
if critical_condition {
    return Err(JinError::SomeError);
}
```

**Collect errors, continue processing:**
```rust
let mut errors = Vec::new();
for item in items {
    match process(item) {
        Ok(_) => success_count += 1,
        Err(e) => errors.push(format!("{}: {}", item, e)),
    }
}

if !errors.is_empty() {
    for error in &errors {
        eprintln!("Error: {}", error);
    }
    if success_count == 0 {
        return Err(JinError::OperationFailed);
    }
}
```

## Testing Patterns

### Location
- Unit tests: `#[cfg(test)] mod tests` at bottom of each file
- Integration tests: `tests/cli_basic.rs`

### Unit Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        std::fs::create_dir(".jin").unwrap();
        let context = ProjectContext::default();
        context.save().unwrap();
        temp
    }

    #[test]
    fn test_happy_path() {
        let _temp = setup_test_env();
        let result = execute(/* args */);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_case() {
        let _temp = setup_test_env();
        let result = execute(/* invalid args */);
        assert!(matches!(result, Err(JinError::SomeError(_))));
    }
}
```

### Integration Test Structure
```rust
use assert_cmd::Command;
use predicates::prelude::*;

fn jin() -> Command {
    Command::cargo_bin("jin").unwrap()
}

#[test]
fn test_command() {
    jin()
        .args(["command", "subcommand"])
        .assert()
        .success()
        .stdout(predicate::str::contains("expected output"));
}
```

## Context Management

### Loading Context
```rust
// Standard pattern with fallback
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => {
        return Err(JinError::NotInitialized);
    }
    Err(_) => ProjectContext::default(),
};

// Requiring active mode/scope
let mode = context.require_mode()?;  // Returns error if None
let scope = context.require_scope()?;
```

### Saving Context
```rust
context.mode = Some("claude".to_string());
context.scope = Some("language:rust".to_string());
context.save()?;
```

## Layer Reference Paths

### Pattern
```rust
// Generate ref path from layer and context
let ref_path = layer.ref_path(
    context.mode.as_deref(),
    context.scope.as_deref(),
    context.project.as_deref()
);

// Examples:
// GlobalBase      → "refs/jin/layers/global"
// ModeBase        → "refs/jin/layers/mode/claude"
// ModeProject     → "refs/jin/layers/mode/claude/project/my-proj"
// ProjectBase     → "refs/jin/layers/project/my-proj"
// WorkspaceActive → "refs/jin/layers/workspace"
```

### Ref Existence Check (CRITICAL)
```rust
// ALWAYS check existence before resolving
if repo.ref_exists(&ref_path) {
    let oid = repo.resolve_ref(&ref_path)?;  // Safe
    // Use as parent commit
} else {
    // Initial commit, no parent
}
```

## Gitignore Management

### Pattern
```rust
use crate::staging::gitignore::{
    ensure_in_managed_block,
    remove_from_managed_block
};

// Add file to managed block
ensure_in_managed_block(&path)?;

// Remove file from managed block
remove_from_managed_block(&path)?;
```

### Managed Block Format
```gitignore
# --- JIN MANAGED START ---
.claude/
.vscode/settings.json
# --- JIN MANAGED END ---
```

**Features:**
- Auto-deduplication
- Sorting of entries
- Never modifies content outside markers
- Created automatically if missing

## Critical Implementation Notes

1. **Jin repository is BARE** - No working directory at ~/.jin/, only object database
2. **Always use workspace::read_file()** for reading from working directory
3. **Check ref existence** before resolving to avoid errors
4. **Use LayerTransaction** for all multi-layer commits
5. **Recovery should run at startup** (not yet implemented in lib.rs)
6. **Content hash stored as hex string** - Not binary, for JSON readability
7. **WorkspaceActive is DERIVED** - Layer 9 is merge output, never committed directly
8. **Staged entries use HashMap** - No guaranteed insertion order
9. **Platform-specific file modes** - Unix detects executable bit, Windows defaults to 0o100644

## Files to Reference in PRP

### Core Implementation Files
- `src/commands/add.rs` - Full command implementation example
- `src/commands/mode.rs` - Subcommand pattern example
- `src/commands/commit_cmd.rs` - Transaction usage example
- `src/commit/pipeline.rs` - LayerTransaction usage (lines 102-112)

### Staging System Files
- `src/staging/index.rs` - Staging index management
- `src/staging/entry.rs` - Staged entry types
- `src/staging/workspace.rs` - Workspace file operations
- `src/staging/gitignore.rs` - .gitignore management

### Transaction System Files
- `src/git/transaction.rs` - Transaction implementation

### Merge System Files
- `src/merge/layer.rs` - Layer merge orchestration

### Context Files
- `src/core/config.rs` - ProjectContext management
- `src/core/layer.rs` - Layer enum and ref paths
- `src/core/error.rs` - Error types
